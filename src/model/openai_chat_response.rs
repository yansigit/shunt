//! Anthropic response contract for OpenAI Chat Completions.
//!
//! One semantic machine serves both output modes (unary JSON and streamed SSE
//! frames) so the two cannot drift: the machine -- never the transport -- owns
//! the terminal decision, so a stream cut by EOF, a duplicate "[DONE]", or
//! residual bytes after the framing terminal can never synthesize a success
//! the provider did not declare. Reasoning extensions, the full finish-reason
//! map, usage precision, and provider error/request-id surfacing are covered
//! here; tool-call delta assembly belongs to a later plan (13-04).

use serde_json::{json, Value};

pub use crate::model::gemini::SseEvent;

const MAX_RETAINED_SEMANTIC_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TerminalState {
    Open,
    SuccessPending,
    SuccessEmitted,
    ProviderFailed,
    ProtocolFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiChatSemanticError {
    message: String,
}

impl OpenAiChatSemanticError {
    fn protocol(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for OpenAiChatSemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for OpenAiChatSemanticError {}

const SIGNED_REASONING_FIELDS: [&str; 4] = [
    "signature",
    "reasoning_signature",
    "redacted_reasoning",
    "encrypted_reasoning",
];

#[derive(Debug)]
struct CheckedChunk {
    parts: Vec<String>,
    reasoning: Option<String>,
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    finish_reason: Option<&'static str>,
    provider_error: Option<Value>,
    retained_bytes: usize,
}

/// State machine over OpenAI Chat chunks (SSE frames or a full completion
/// object) that emits Anthropic SSE events and the final Anthropic response.
pub struct OpenAiChatSseMachine {
    model: String,
    message_id: String,
    started: bool,
    terminal: TerminalState,
    block_index: usize,
    active_text_block: Option<usize>,
    active_thinking_block: Option<usize>,
    input_tokens: u64,
    output_tokens: u64,
    last_finish_reason: Option<&'static str>,
    accumulate_content: bool,
    content: Vec<Value>,
    retained_bytes: usize,
    trailing_usage_accepted: bool,
}

impl OpenAiChatSseMachine {
    pub fn new_for_upstream(model: impl Into<String>) -> Self {
        let random_id = format!("{:016x}", rand::random::<u64>());
        Self {
            model: model.into(),
            message_id: format!("msg_openai_chat_{random_id}"),
            started: false,
            terminal: TerminalState::Open,
            block_index: 0,
            active_text_block: None,
            active_thinking_block: None,
            input_tokens: 0,
            output_tokens: 0,
            last_finish_reason: None,
            accumulate_content: true,
            content: Vec::new(),
            retained_bytes: 0,
            trailing_usage_accepted: false,
        }
    }

    /// Construct a machine for incremental relay: streamed text is forwarded
    /// as deltas and never retained as a second complete response copy.
    pub fn new_streaming_for_upstream(model: impl Into<String>) -> Self {
        let mut machine = Self::new_for_upstream(model);
        machine.accumulate_content = false;
        machine
    }

    pub fn is_started(&self) -> bool {
        self.started
    }

    /// Validate one Chat Completions chunk (or whole completion) and atomically
    /// apply its semantic meaning.
    pub fn process_chunk_checked(
        &mut self,
        chunk: &Value,
    ) -> Result<Vec<SseEvent>, OpenAiChatSemanticError> {
        match self.terminal {
            TerminalState::Open => {}
            // With stream_options.include_usage the official stream appends a
            // usage-only chunk (empty choices) after the finish_reason chunk
            // and before [DONE]. Accept exactly that shape; content, a second
            // finish, or any other payload after finish_reason fails closed.
            TerminalState::SuccessPending => return self.process_trailing_usage_chunk(chunk),
            _ => {
                self.terminal = TerminalState::ProtocolFailed;
                return Err(OpenAiChatSemanticError::protocol(
                    "OpenAI Chat semantic data arrived after a terminal outcome",
                ));
            }
        }

        let checked = match self.validate_chunk(chunk) {
            Ok(checked) => checked,
            Err(error) => {
                self.terminal = TerminalState::ProtocolFailed;
                return Err(error);
            }
        };

        if let Some(error_val) = checked.provider_error {
            self.terminal = TerminalState::ProviderFailed;
            return Ok(vec![SseEvent {
                event: "error".to_string(),
                data: error_val,
            }]);
        }

        self.retained_bytes = self
            .retained_bytes
            .checked_add(checked.retained_bytes)
            .filter(|total| *total <= MAX_RETAINED_SEMANTIC_BYTES)
            .ok_or_else(|| {
                self.terminal = TerminalState::ProtocolFailed;
                OpenAiChatSemanticError::protocol(
                    "OpenAI Chat retained semantic state exceeds limit",
                )
            })?;

        if let Some(tokens) = checked.input_tokens {
            self.input_tokens = tokens;
        }
        if let Some(tokens) = checked.output_tokens {
            self.output_tokens = tokens;
        }

        let mut events = Vec::new();
        if !self.started
            && (!checked.parts.is_empty()
                || checked.reasoning.is_some()
                || checked.finish_reason.is_some())
        {
            self.started = true;
            events.push(self.message_start_event());
        }
        if let Some(reasoning) = checked.reasoning {
            self.apply_reasoning(&reasoning, &mut events);
        }
        for text in checked.parts {
            self.apply_part(text, &mut events);
        }
        if let Some(reason) = checked.finish_reason {
            self.last_finish_reason = Some(reason);
            self.terminal = TerminalState::SuccessPending;
        }
        Ok(events)
    }

    fn process_trailing_usage_chunk(
        &mut self,
        chunk: &Value,
    ) -> Result<Vec<SseEvent>, OpenAiChatSemanticError> {
        let chunk = chunk.as_object().ok_or_else(|| {
            OpenAiChatSemanticError::protocol("OpenAI Chat chunk must be an object")
        })?;
        if self.trailing_usage_accepted {
            self.terminal = TerminalState::ProtocolFailed;
            return Err(OpenAiChatSemanticError::protocol(
                "only one usage-only chunk is permitted after the finish_reason chunk",
            ));
        }
        if chunk.contains_key("error") {
            let error_val = &chunk["error"];
            self.terminal = TerminalState::ProviderFailed;
            let message = chunk
                .get("error")
                .and_then(|error| error.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("OpenAI Chat backend error");
            return Ok(vec![SseEvent {
                event: "error".to_string(),
                data: provider_error_data(
                    message,
                    error_val.as_object().expect("checked error object above"),
                ),
            }]);
        }
        if let Some(choices) = chunk.get("choices") {
            let choices = choices.as_array().ok_or_else(|| {
                OpenAiChatSemanticError::protocol("OpenAI Chat choices must be an array")
            })?;
            let carries_payload = choices.iter().any(|choice| {
                choice.get("finish_reason").is_some()
                    || choice
                        .get("delta")
                        .and_then(Value::as_object)
                        .is_some_and(|delta| !delta.is_empty())
            });
            if carries_payload {
                self.terminal = TerminalState::ProtocolFailed;
                return Err(OpenAiChatSemanticError::protocol(
                    "a payload delta or a second finish_reason arrived after the finish_reason chunk",
                ));
            }
        }
        let (input_tokens, output_tokens) = validate_usage(chunk.get("usage"))?;
        if let Some(tokens) = input_tokens {
            self.input_tokens = tokens;
        }
        if let Some(tokens) = output_tokens {
            self.output_tokens = tokens;
        }
        self.trailing_usage_accepted = true;
        Ok(Vec::new())
    }

    fn validate_chunk(&self, chunk: &Value) -> Result<CheckedChunk, OpenAiChatSemanticError> {
        let chunk = chunk.as_object().ok_or_else(|| {
            OpenAiChatSemanticError::protocol("OpenAI Chat chunk must be an object")
        })?;

        if let Some(error) = chunk.get("error") {
            let error = error.as_object().ok_or_else(|| {
                OpenAiChatSemanticError::protocol("OpenAI Chat provider error must be an object")
            })?;
            let message = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("OpenAI Chat backend error");
            return Ok(CheckedChunk {
                parts: Vec::new(),
                reasoning: None,
                input_tokens: None,
                output_tokens: None,
                finish_reason: None,
                provider_error: Some(provider_error_data(message, error)),
                retained_bytes: 0,
            });
        }

        let (input_tokens, output_tokens) = validate_usage(chunk.get("usage"))?;

        let mut parts = Vec::new();
        let mut reasoning = None;
        let mut finish_reason = None;
        let mut provider_error = None;
        let mut retained_bytes = 0usize;
        let choices = chunk.get("choices").ok_or_else(|| {
            OpenAiChatSemanticError::protocol("OpenAI Chat chunk must include choices")
        })?;
        let choices = choices.as_array().ok_or_else(|| {
            OpenAiChatSemanticError::protocol("OpenAI Chat choices must be an array")
        })?;
        if choices.len() > 1 {
            return Err(OpenAiChatSemanticError::protocol(
                "multiple OpenAI Chat choices have ambiguous ordering",
            ));
        }
        let choice = choices.first().ok_or_else(|| {
            OpenAiChatSemanticError::protocol("OpenAI Chat choices must not be empty")
        })?;
        let choice = choice.as_object().ok_or_else(|| {
            OpenAiChatSemanticError::protocol("OpenAI Chat choice must be an object")
        })?;
        if let Some(reason) = choice.get("finish_reason").filter(|value| !value.is_null()) {
            let reason = reason.as_str().ok_or_else(|| {
                OpenAiChatSemanticError::protocol("OpenAI Chat finish_reason must be a string")
            })?;
            let mapped = match reason {
                "stop" => Some("end_turn"),
                "length" => Some("max_tokens"),
                "tool_calls" => Some("tool_use"),
                "error" => {
                    provider_error = Some(provider_error_data(
                        "OpenAI Chat provider reported finish_reason \"error\"",
                        chunk,
                    ));
                    None
                }
                other => {
                    return Err(OpenAiChatSemanticError::protocol(format!(
                        "unsupported OpenAI Chat finish_reason {other}"
                    )))
                }
            };
            finish_reason = mapped;
        }
        // Both output shapes share one field path check: a whole completion
        // carries message content, a stream frame carries delta content.
        let payload = choice.get("message").or_else(|| choice.get("delta"));
        if let Some(payload) = payload {
            reasoning = extract_reasoning(payload)?;
        }
        let content = payload.and_then(|message| message.get("content"));
        if let Some(content) = content {
            let text = match content {
                Value::Null => None,
                Value::String(text) => Some(text.as_str()),
                _ => {
                    return Err(OpenAiChatSemanticError::protocol(
                        "OpenAI Chat content must be a string or null",
                    ))
                }
            };
            if let Some(text) = text {
                retained_bytes = retained_bytes.checked_add(text.len()).ok_or_else(|| {
                    OpenAiChatSemanticError::protocol(
                        "OpenAI Chat retained semantic state exceeds limit",
                    )
                })?;
                parts.push(text.to_string());
            }
        }
        Ok(CheckedChunk {
            parts,
            reasoning,
            input_tokens,
            output_tokens,
            finish_reason,
            provider_error,
            retained_bytes,
        })
    }

    fn message_start_event(&self) -> SseEvent {
        SseEvent {
            event: "message_start".to_string(),
            data: json!({
                "type": "message_start",
                "message": {
                    "id": self.message_id,
                    "type": "message",
                    "role": "assistant",
                    "content": [],
                    "model": self.model,
                    "stop_reason": null,
                    "stop_sequence": null,
                    "usage": {"input_tokens": self.input_tokens, "output_tokens": 0}
                }
            }),
        }
    }

    fn apply_part(&mut self, text: String, events: &mut Vec<SseEvent>) {
        if self.active_text_block.is_none() {
            self.close_active_block(events);
            let idx = self.block_index;
            events.push(SseEvent {
                event: "content_block_start".to_string(),
                data: json!({
                    "type": "content_block_start",
                    "index": idx,
                    "content_block": {"type": "text", "text": ""}
                }),
            });
            self.active_text_block = Some(idx);
        }
        let idx = self.active_text_block.unwrap();
        events.push(SseEvent {
            event: "content_block_delta".to_string(),
            data: json!({
                "type": "content_block_delta",
                "index": idx,
                "delta": {"type": "text_delta", "text": text}
            }),
        });
        if self.accumulate_content {
            let should_push = match self.content.last_mut() {
                Some(last) if last.get("type").and_then(Value::as_str) == Some("text") => {
                    if let Some(Value::String(s)) = last.get_mut("text") {
                        s.push_str(&text);
                    }
                    false
                }
                _ => true,
            };
            if should_push {
                self.content.push(json!({"type": "text", "text": text}));
            }
        }
    }

    fn apply_reasoning(&mut self, text: &str, events: &mut Vec<SseEvent>) {
        if self.active_thinking_block.is_none() {
            self.close_active_block(events);
            let idx = self.block_index;
            events.push(SseEvent {
                event: "content_block_start".to_string(),
                data: json!({
                    "type": "content_block_start",
                    "index": idx,
                    "content_block": {"type": "thinking", "thinking": ""}
                }),
            });
            self.active_thinking_block = Some(idx);
        }
        let idx = self.active_thinking_block.unwrap();
        events.push(SseEvent {
            event: "content_block_delta".to_string(),
            data: json!({
                "type": "content_block_delta",
                "index": idx,
                "delta": {"type": "thinking_delta", "thinking": text}
            }),
        });
        if self.accumulate_content {
            let should_push = match self.content.last_mut() {
                Some(last) if last.get("type").and_then(Value::as_str) == Some("thinking") => {
                    if let Some(Value::String(s)) = last.get_mut("thinking") {
                        s.push_str(text);
                    }
                    false
                }
                _ => true,
            };
            if should_push {
                self.content
                    .push(json!({"type": "thinking", "thinking": text}));
            }
        }
    }

    fn close_active_block(&mut self, events: &mut Vec<SseEvent>) {
        if let Some(idx) = self.active_thinking_block.take() {
            events.push(SseEvent {
                event: "content_block_stop".to_string(),
                data: json!({
                    "type": "content_block_stop",
                    "index": idx
                }),
            });
            self.block_index += 1;
        }
        if let Some(idx) = self.active_text_block.take() {
            events.push(SseEvent {
                event: "content_block_stop".to_string(),
                data: json!({
                    "type": "content_block_stop",
                    "index": idx
                }),
            });
            self.block_index += 1;
        }
    }

    /// The transport -- EOF or the "[DONE]" framing marker -- may only *close*
    /// a provider-declared success. Without a preceding accepted
    /// finish_reason this fails closed, so an upstream cut mid-stream can
    /// never become a synthesized success.
    pub fn transport_close_checked(&mut self) -> Result<Vec<SseEvent>, OpenAiChatSemanticError> {
        if self.terminal != TerminalState::SuccessPending {
            self.terminal = TerminalState::ProtocolFailed;
            return Err(OpenAiChatSemanticError::protocol(
                "OpenAI Chat transport closed without a finish_reason or [DONE]",
            ));
        }
        let mut events = Vec::new();
        if !self.started {
            self.started = true;
            events.push(self.message_start_event());
        }
        self.close_active_block(&mut events);
        events.push(SseEvent {
            event: "message_delta".to_string(),
            data: json!({
                "type": "message_delta",
                "delta": {
                    "stop_reason": self.stop_reason(),
                    "stop_sequence": null
                },
                "usage": {
                    "input_tokens": self.input_tokens,
                    "output_tokens": self.output_tokens
                }
            }),
        });
        events.push(SseEvent {
            event: "message_stop".to_string(),
            data: json!({"type": "message_stop"}),
        });
        self.terminal = TerminalState::SuccessEmitted;
        Ok(events)
    }

    /// Return the full final Anthropic response JSON for the unary path.
    pub fn final_json_checked(&self) -> Result<Value, OpenAiChatSemanticError> {
        if self.terminal != TerminalState::SuccessEmitted || !self.accumulate_content {
            return Err(OpenAiChatSemanticError::protocol(
                "OpenAI Chat final JSON requested before checked unary completion",
            ));
        }
        Ok(json!({
            "id": self.message_id,
            "type": "message",
            "role": "assistant",
            "content": self.content,
            "model": self.model,
            "stop_reason": self.stop_reason(),
            "stop_sequence": null,
            "usage": {
                "input_tokens": self.input_tokens,
                "output_tokens": self.output_tokens
            }
        }))
    }

    fn stop_reason(&self) -> &'static str {
        match self.last_finish_reason {
            Some("max_tokens") => "max_tokens",
            Some("tool_use") => "tool_use",
            _ => "end_turn",
        }
    }
}

fn provider_error_data(message: &str, provider_error: &serde_json::Map<String, Value>) -> Value {
    let mut inner = json!({"type": "api_error", "message": message});
    if let Some(request_id) = allowlisted_request_id(provider_error) {
        inner["request_id"] = json!(request_id);
    }
    json!({"type": "error", "error": inner})
}

// Forward a provider request id only from error.metadata.request_id when it
// is 1..=128 printable ASCII bytes (no spaces, no control characters), so
// untrusted strings cannot smuggle framing or log-injection payloads.
fn allowlisted_request_id(
    provider_error: &serde_json::Map<String, Value>,
) -> Option<&str> {
    let request_id = provider_error
        .get("metadata")
        .and_then(|metadata| metadata.get("request_id"))
        .and_then(Value::as_str)?;
    let allowed = !request_id.is_empty()
        && request_id.len() <= 128
        && request_id
            .bytes()
            .all(|byte| (0x21..=0x7E).contains(&byte));
    allowed.then_some(request_id)
}

// Normalize the reasoning_content / reasoning aliases: null or empty is
// absent, non-string fails closed, both-present-and-differing fails closed.
fn extract_reasoning(payload: &Value) -> Result<Option<String>, OpenAiChatSemanticError> {
    let Some(payload) = payload.as_object() else {
        return Ok(None);
    };
    for field in SIGNED_REASONING_FIELDS {
        if payload.contains_key(field) {
            return Err(OpenAiChatSemanticError::protocol(format!(
                "OpenAI Chat reasoning field {field} is an opaque signed representation and is not supported"
            )));
        }
    }
    let read_alias = |key: &str| -> Result<Option<String>, OpenAiChatSemanticError> {
        match payload.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(text)) => Ok(Some(text.clone())),
            Some(_) => Err(OpenAiChatSemanticError::protocol(format!(
                "OpenAI Chat {key} must be a string or null"
            ))),
        }
    };
    let primary = read_alias("reasoning_content")?;
    let alias = read_alias("reasoning")?;
    let primary = primary.filter(|text| !text.is_empty());
    let alias = alias.filter(|text| !text.is_empty());
    match (primary, alias) {
        (None, None) => Ok(None),
        (Some(text), None) | (None, Some(text)) => Ok(Some(text)),
        (Some(primary), Some(alias)) if primary == alias => Ok(Some(primary)),
        (Some(_), Some(_)) => Err(OpenAiChatSemanticError::protocol(
            "conflicting OpenAI Chat reasoning aliases reasoning_content and reasoning",
        )),
    }
}

fn validate_usage(
    usage: Option<&Value>,
) -> Result<(Option<u64>, Option<u64>), OpenAiChatSemanticError> {
    let Some(usage) = usage.filter(|value| !value.is_null()) else {
        return Ok((None, None));
    };
    let usage = usage
        .as_object()
        .ok_or_else(|| OpenAiChatSemanticError::protocol("OpenAI Chat usage must be an object"))?;
    let parse = |key: &str| -> Result<Option<u64>, OpenAiChatSemanticError> {
        match usage.get(key) {
            None => Ok(None),
            Some(value) => value
                .as_u64()
                .filter(|tokens| *tokens <= i64::MAX as u64)
                .map(Some)
                .ok_or_else(|| {
                    OpenAiChatSemanticError::protocol(format!(
                        "OpenAI Chat usage.{key} must be a non-negative integer within i64 range"
                    ))
                }),
        }
    };
    let input = parse("prompt_tokens")?;
    let output = parse("completion_tokens")?;
    Ok((input, output))
}
