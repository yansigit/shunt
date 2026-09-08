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

mod assembly;
mod checked;
mod unary_tools;
mod validation;
use validation::{extract_reasoning, provider_error_data, validate_usage};

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
    tools: Vec<Value>,
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
    tool_assembly: assembly::ToolAssembly,
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
            tool_assembly: assembly::ToolAssembly::default(),
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
            TerminalState::SuccessPending => {
                let result = self.process_trailing_usage_chunk(chunk);
                if result.is_err() {
                    self.terminal = TerminalState::ProtocolFailed;
                }
                return result;
            }
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

        let tool_bytes = if let Some(deltas) = chunk.pointer("/choices/0/delta/tool_calls") {
            self.tool_assembly.apply(deltas).map_err(|error| {
                self.terminal = TerminalState::ProtocolFailed;
                error
            })?
        } else {
            0
        };
        let streamed_tools = if checked.finish_reason.is_some() {
            self.tool_assembly.finish().map_err(|error| {
                self.terminal = TerminalState::ProtocolFailed;
                error
            })?
        } else {
            Vec::new()
        };

        self.retained_bytes = self
            .retained_bytes
            .checked_add(tool_bytes)
            .and_then(|bytes| bytes.checked_add(checked.retained_bytes))
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
        for tool in checked.tools.into_iter().chain(streamed_tools) {
            self.close_active_block(&mut events);
            let arguments = tool["input"].to_string();
            let mut start = tool.clone();
            start["input"] = json!({});
            events.push(SseEvent {
                event: "content_block_start".into(),
                data: json!({"type":"content_block_start","index":self.block_index,"content_block":start}),
            });
            events.push(SseEvent {
                event: "content_block_delta".into(),
                data: json!({"type":"content_block_delta","index":self.block_index,"delta":{"type":"input_json_delta","partial_json":arguments}}),
            });
            events.push(SseEvent {
                event: "content_block_stop".into(),
                data: json!({"type":"content_block_stop","index":self.block_index}),
            });
            self.block_index += 1;
            if self.accumulate_content {
                self.content.push(tool);
            }
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
            let error_val = chunk["error"].as_object().ok_or_else(|| {
                OpenAiChatSemanticError::protocol("OpenAI Chat provider error must be an object")
            })?;
            self.terminal = TerminalState::ProviderFailed;
            let message = chunk
                .get("error")
                .and_then(|error| error.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("OpenAI Chat backend error");
            return Ok(vec![SseEvent {
                event: "error".to_string(),
                data: provider_error_data(message, error_val),
            }]);
        }
        if !chunk
            .get("choices")
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty)
        {
            return Err(OpenAiChatSemanticError::protocol(
                "only empty choices are permitted after the finish_reason chunk",
            ));
        }
        let (input_tokens, output_tokens) = validate_usage(chunk.get("usage"))?;
        if input_tokens.is_none() || output_tokens.is_none() {
            return Err(OpenAiChatSemanticError::protocol(
                "trailing usage must include prompt_tokens and completion_tokens",
            ));
        }
        if let Some(tokens) = input_tokens {
            self.input_tokens = tokens;
        }
        if let Some(tokens) = output_tokens {
            self.output_tokens = tokens;
        }
        self.trailing_usage_accepted = true;
        Ok(Vec::new())
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
            "content": if self.content.is_empty() { vec![json!({"type":"text","text":""})] } else { self.content.clone() },
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
