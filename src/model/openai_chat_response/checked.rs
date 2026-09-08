use super::{
    extract_reasoning, provider_error_data, unary_tools, validate_usage, CheckedChunk,
    OpenAiChatSemanticError, OpenAiChatSseMachine,
};
use serde_json::Value;

impl OpenAiChatSseMachine {
    pub(super) fn validate_chunk(
        &self,
        chunk: &Value,
    ) -> Result<CheckedChunk, OpenAiChatSemanticError> {
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
                tools: Vec::new(),
                input_tokens: None,
                output_tokens: None,
                finish_reason: None,
                provider_error: Some(provider_error_data(message, error)),
                retained_bytes: 0,
            });
        }

        let (input_tokens, output_tokens) = validate_usage(chunk.get("usage"))?;

        let mut parts = Vec::new();
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
        if let Some(error) = choice.get("error") {
            let error = error.as_object().ok_or_else(|| {
                OpenAiChatSemanticError::protocol("OpenAI Chat choice error must be an object")
            })?;
            return Ok(CheckedChunk {
                parts: Vec::new(),
                reasoning: None,
                tools: Vec::new(),
                input_tokens: None,
                output_tokens: None,
                finish_reason: None,
                provider_error: Some(provider_error_data("OpenAI Chat backend error", error)),
                retained_bytes: 0,
            });
        }
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
                _ => {
                    return Err(OpenAiChatSemanticError::protocol(
                        "unsupported OpenAI Chat finish_reason",
                    ))
                }
            };
            finish_reason = mapped;
        }
        // Both output shapes share one field path check: a whole completion
        // carries message content, a stream frame carries delta content.
        let payload = match (choice.get("message"), choice.get("delta")) {
            (Some(payload), None) | (None, Some(payload)) if payload.is_object() => payload,
            _ => {
                return Err(OpenAiChatSemanticError::protocol(
                    "OpenAI Chat choice requires exactly one message or delta object",
                ))
            }
        };
        let reasoning = extract_reasoning(payload)?;
        retained_bytes += reasoning.as_ref().map_or(0, String::len);
        let tools = if choice.contains_key("message") {
            unary_tools::translate(payload.get("tool_calls"), &mut retained_bytes)?
        } else {
            Vec::new()
        };
        let content = payload.get("content");
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
            tools,
            input_tokens,
            output_tokens,
            finish_reason,
            provider_error,
            retained_bytes,
        })
    }
}
