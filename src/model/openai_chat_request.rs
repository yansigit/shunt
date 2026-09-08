//! Anthropic Messages -> OpenAI Chat Completions request translation.
//!
//! Deliberate deny-by-default whitelist for this slice: text, image, and tool
//! turns (system/user/assistant content), the resolved model, the stream flag,
//! the generation controls, tool declarations/choice, and paired tool results
//! with a request-local id registry. Unknown or Responses-only fields are
//! refused instead of silently dropped, so an unsupported turn fails closed
//! with a typed Anthropic-shaped 400 rather than degrading upstream.

use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::{json, Map, Value};
use std::collections::HashSet;

use crate::adapters::AdapterError;

/// Maximum UTF-8 bytes accepted for any single translated text payload.
/// Enforced before dispatch with a plain byte count; CHAT-03 boundary row.
pub const MAX_TEXT_BLOCK_BYTES: usize = 8 * 1024 * 1024;

pub(crate) fn bad_request(message: impl Into<String>) -> AdapterError {
    let message = message.into();
    AdapterError {
        message: message.clone(),
        response: Box::new(
            (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({
                    "type": "error",
                    "error": {"type": "invalid_request_error", "message": message}
                })),
            )
                .into_response(),
        ),
        failure: None,
    }
}

/// Translate the inbound Anthropic request into an OpenAI Chat Completions body.
///
/// `stream` is decided by the adapter from the inbound flag (never trusted
/// from the translated tree alone) and forces the upstream
/// `stream_options.include_usage` contract so streaming turns keep their
/// usage accounting. The translation is pure: the tool id registry lives only
/// for the duration of this call, so concurrent requests cannot share state.
pub fn translate_request(
    request: &Value,
    model: &str,
    stream: bool,
) -> Result<Value, AdapterError> {
    let request = request
        .as_object()
        .ok_or_else(|| bad_request("request body must be a JSON object"))?;
    let mut out = Map::new();
    out.insert("model".to_string(), json!(model));
    out.insert("stream".to_string(), json!(stream));
    if stream {
        out.insert("stream_options".to_string(), json!({"include_usage": true}));
    }
    for key in request.keys() {
        match key.as_str() {
            "model" | "stream" | "messages" | "system" | "max_tokens" | "temperature" | "top_p"
            | "stop_sequences" | "tools" | "tool_choice" => {}
            "metadata" => {
                return Err(bad_request(
                    "metadata is not supported on the OpenAI Chat slice",
                ));
            }
            other => {
                return Err(bad_request(format!(
                    "unknown request field {other} is not supported on the OpenAI Chat slice"
                )));
            }
        }
    }
    if let Some(max_tokens) = request.get("max_tokens") {
        let max_tokens = max_tokens
            .as_u64()
            .ok_or_else(|| bad_request("max_tokens must be a non-negative integer"))?;
        out.insert("max_tokens".to_string(), json!(max_tokens));
    }
    for (key, outbound) in [("temperature", "temperature"), ("top_p", "top_p")] {
        if let Some(value) = request.get(key) {
            let value = value
                .as_f64()
                .ok_or_else(|| bad_request(format!("{key} must be a number")))?;
            out.insert(outbound.to_string(), json!(value));
        }
    }
    if let Some(stop_sequences) = request.get("stop_sequences") {
        let stop = translate_stop_sequences(stop_sequences)?;
        out.insert("stop".to_string(), stop);
    }
    if let Some(tools) = request.get("tools") {
        let tools = translate_tools(tools)?;
        out.insert("tools".to_string(), tools);
    }
    if let Some(tool_choice) = request.get("tool_choice") {
        let tool_choice = translate_tool_choice(tool_choice)?;
        out.insert("tool_choice".to_string(), tool_choice);
    }
    let mut messages: Vec<Value> = Vec::new();
    if let Some(system) = request.get("system") {
        messages.push(translate_system(system)?);
    }
    let inbound_messages = request
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| bad_request("messages must be an array"))?;
    if inbound_messages.is_empty() {
        return Err(bad_request("messages must not be empty"));
    }
    let mut tool_ids: HashSet<String> = HashSet::new();
    for message in inbound_messages {
        messages.extend(translate_message(message, &mut tool_ids)?);
    }
    out.insert("messages".to_string(), Value::Array(messages));
    Ok(Value::Object(out))
}

fn translate_stop_sequences(stop_sequences: &Value) -> Result<Value, AdapterError> {
    let sequences = stop_sequences
        .as_array()
        .ok_or_else(|| bad_request("stop_sequences must be an array of strings"))?;
    let mut stop = Vec::with_capacity(sequences.len());
    for sequence in sequences {
        let sequence = sequence
            .as_str()
            .ok_or_else(|| bad_request("stop_sequences must be an array of strings"))?;
        stop.push(json!(sequence));
    }
    Ok(Value::Array(stop))
}

fn translate_tools(tools: &Value) -> Result<Value, AdapterError> {
    let tools = tools
        .as_array()
        .ok_or_else(|| bad_request("tools must be an array of tool declarations"))?;
    let mut translated = Vec::with_capacity(tools.len());
    for tool in tools {
        let tool = tool
            .as_object()
            .ok_or_else(|| bad_request("each tool declaration must be an object"))?;
        let name = tool
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| bad_request("each tool declaration must have a string name"))?;
        let parameters = tool
            .get("input_schema")
            .ok_or_else(|| bad_request(format!("tool {name} must have an input_schema object")))?;
        if !parameters.is_object() {
            return Err(bad_request(format!(
                "tool {name} must have an input_schema object"
            )));
        }
        let mut function = Map::new();
        function.insert("name".to_string(), json!(name));
        if let Some(description) = tool.get("description") {
            function.insert("description".to_string(), description.clone());
        }
        function.insert("parameters".to_string(), parameters.clone());
        translated.push(json!({"type": "function", "function": Value::Object(function)}));
    }
    Ok(Value::Array(translated))
}

fn translate_tool_choice(tool_choice: &Value) -> Result<Value, AdapterError> {
    match tool_choice {
        Value::String(choice) if choice == "auto" => Ok(json!("auto")),
        Value::String(choice) if choice == "any" => Ok(json!("required")),
        Value::Object(choice) => {
            if choice.get("type").and_then(Value::as_str) != Some("tool") {
                return Err(bad_request(
                    "tool_choice must be \"auto\", \"any\", or a tool selection object",
                ));
            }
            let name = choice
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| bad_request("tool_choice tool selection must have a name"))?;
            Ok(json!({"type": "function", "function": {"name": name}}))
        }
        _ => Err(bad_request(
            "tool_choice must be \"auto\", \"any\", or a tool selection object",
        )),
    }
}

fn checked_text(text: &str) -> Result<(), AdapterError> {
    let bytes = text.len();
    if bytes > MAX_TEXT_BLOCK_BYTES {
        return Err(bad_request(format!(
            "text content of {bytes} bytes exceeds the {MAX_TEXT_BLOCK_BYTES} byte budget"
        )));
    }
    Ok(())
}

fn translate_system(system: &Value) -> Result<Value, AdapterError> {
    let text: String = match system {
        Value::String(text) => text.clone(),
        Value::Array(blocks) => collect_text_blocks(blocks)?,
        _ => {
            return Err(bad_request(
                "system must be a string or an array of text blocks",
            ))
        }
    };
    if text.is_empty() {
        return Err(bad_request("system text must not be empty"));
    }
    checked_text(&text)?;
    Ok(text_with_role("system", &text))
}

fn translate_message(
    message: &Value,
    tool_ids: &mut HashSet<String>,
) -> Result<Vec<Value>, AdapterError> {
    let message = message
        .as_object()
        .ok_or_else(|| bad_request("each message must be an object"))?;
    let role = message
        .get("role")
        .and_then(Value::as_str)
        .ok_or_else(|| bad_request("each message must have a role"))?;
    if !matches!(role, "system" | "user" | "assistant") {
        return Err(bad_request(format!(
            "unsupported OpenAI Chat message role {role}"
        )));
    }
    let content = message
        .get("content")
        .ok_or_else(|| bad_request("each message must have content"))?;
    match content {
        Value::String(text) => Ok(vec![translate_text_content(role, text)?]),
        Value::Array(blocks) => translate_block_content(role, blocks, tool_ids),
        _ => Err(bad_request(
            "message content must be a string or content blocks",
        )),
    }
}

fn translate_text_content(role: &str, text: &str) -> Result<Value, AdapterError> {
    if text.is_empty() {
        return Err(bad_request("message text content must not be empty"));
    }
    checked_text(text)?;
    Ok(text_with_role(role, text))
}

fn translate_block_content(
    role: &str,
    blocks: &[Value],
    tool_ids: &mut HashSet<String>,
) -> Result<Vec<Value>, AdapterError> {
    if blocks.is_empty() {
        return Err(bad_request("message content blocks must not be empty"));
    }
    let mut text_parts: Vec<&str> = Vec::new();
    let mut reasoning_parts: Vec<String> = Vec::new();
    let mut content: Vec<Value> = Vec::new();
    let mut tool_calls: Vec<Value> = Vec::new();
    let mut tool_messages: Vec<Value> = Vec::new();
    let mut has_image = false;
    for block in blocks {
        let block = block
            .as_object()
            .ok_or_else(|| bad_request("content blocks must be objects"))?;
        let block_type = block
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| bad_request("content blocks must have a type"))?;
        match block_type {
            "text" => {
                let text = block
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(|| bad_request("text content block must have string text"))?;
                checked_text(text)?;
                text_parts.push(text);
                content.push(json!({"type": "text", "text": text}));
            }
            "image" => {
                if role != "user" {
                    return Err(bad_request(
                        "image blocks are only supported in user messages",
                    ));
                }
                content.push(translate_image_block(block)?);
                has_image = true;
            }
            "thinking" => {
                if role != "assistant" {
                    return Err(bad_request(
                        "thinking blocks are only supported in assistant messages",
                    ));
                }
                if block.contains_key("signature") {
                    return Err(bad_request(
                        "thinking blocks with a signature cannot be forwarded losslessly",
                    ));
                }
                let thinking = block
                    .get("thinking")
                    .and_then(Value::as_str)
                    .ok_or_else(|| bad_request("thinking block must have string thinking"))?;
                checked_text(thinking)?;
                reasoning_parts.push(thinking.to_string());
            }
            "redacted_thinking" => {
                return Err(bad_request(
                    "redacted_thinking blocks cannot be forwarded losslessly",
                ));
            }
            "tool_use" => {
                if role != "assistant" {
                    return Err(bad_request(
                        "tool_use blocks are only supported in assistant messages",
                    ));
                }
                let id = block
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| bad_request("tool_use must have a string id"))?;
                if !tool_ids.insert(id.to_string()) {
                    return Err(bad_request(format!(
                        "duplicate tool_use id {id} in the same request"
                    )));
                }
                let name = block
                    .get("name")
                    .and_then(Value::as_str)
                    .ok_or_else(|| bad_request("tool_use must have a string name"))?;
                let input = block
                    .get("input")
                    .ok_or_else(|| bad_request("tool_use input must be a JSON object"))?;
                let input = input.as_object().ok_or_else(|| {
                    bad_request(
                        "tool_use input must be a JSON object; incomplete arguments are rejected",
                    )
                })?;
                let arguments =
                    serde_json::to_string(&Value::Object(input.clone())).map_err(|error| {
                        bad_request(format!("tool_use input is not serializable: {error}"))
                    })?;
                tool_calls.push(json!({
                    "id": id,
                    "type": "function",
                    "function": {"name": name, "arguments": arguments}
                }));
            }
            "tool_result" => {
                if role != "user" {
                    return Err(bad_request(
                        "tool_result blocks are only supported in user messages",
                    ));
                }
                let tool_use_id = block
                    .get("tool_use_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| bad_request("tool_result must reference a tool_use id"))?;
                if !tool_ids.contains(tool_use_id) {
                    return Err(bad_request(format!(
                        "tool_result for {tool_use_id} has no preceding matching tool_use"
                    )));
                }
                let result_text = match block.get("content") {
                    None | Some(Value::Null) => String::new(),
                    Some(Value::String(text)) => text.clone(),
                    Some(Value::Array(parts)) => {
                        let mut text = String::new();
                        for part in parts {
                            let part = part.as_object().ok_or_else(|| {
                                bad_request("tool_result content parts must be objects")
                            })?;
                            if part.get("type").and_then(Value::as_str) != Some("text") {
                                return Err(bad_request(
                                    "tool_result content parts support text only",
                                ));
                            }
                            let piece =
                                part.get("text").and_then(Value::as_str).ok_or_else(|| {
                                    bad_request("tool_result text content must be a string")
                                })?;
                            checked_text(piece)?;
                            text.push_str(piece);
                        }
                        text
                    }
                    Some(_) => {
                        return Err(bad_request(
                            "tool_result content must be a string or text blocks",
                        ));
                    }
                };
                checked_text(&result_text)?;
                tool_messages.push(json!({
                    "role": "tool",
                    "tool_call_id": tool_use_id,
                    "content": result_text
                }));
            }
            other => {
                return Err(bad_request(format!(
                    "unsupported content block type {other}"
                )));
            }
        }
    }
    let text = text_parts.concat();
    if text.is_empty() && tool_calls.is_empty() && !has_image && tool_messages.is_empty() {
        return Err(bad_request("message text content must not be empty"));
    }
    let mut translated: Vec<Value> = Vec::new();
    if !text.is_empty() || has_image || !tool_calls.is_empty() {
        let mut message = Map::new();
        message.insert("role".to_string(), json!(role));
        if has_image {
            message.insert("content".to_string(), Value::Array(content));
        } else if text.is_empty() {
            message.insert("content".to_string(), Value::Null);
        } else {
            message.insert("content".to_string(), json!(text));
        }
        if !tool_calls.is_empty() {
            message.insert("tool_calls".to_string(), Value::Array(tool_calls));
        }
        if !reasoning_parts.is_empty() {
            message.insert(
                "reasoning_content".to_string(),
                json!(reasoning_parts.join("\n")),
            );
        }
        translated.push(Value::Object(message));
    }
    translated.extend(tool_messages);
    debug_assert!(
        !translated.is_empty(),
        "a nonempty block list must translate to at least one message"
    );
    Ok(translated)
}

fn translate_image_block(block: &Map<String, Value>) -> Result<Value, AdapterError> {
    let source = block
        .get("source")
        .ok_or_else(|| bad_request("image block must have a source"))?
        .as_object()
        .ok_or_else(|| bad_request("image source must be an object"))?;
    let url = match source.get("type").and_then(Value::as_str) {
        Some("url") => source
            .get("url")
            .and_then(Value::as_str)
            .ok_or_else(|| bad_request("remote image source must have a url"))?
            .to_string(),
        Some("base64") => {
            let media_type = source
                .get("media_type")
                .and_then(Value::as_str)
                .ok_or_else(|| bad_request("base64 image source must have a media_type"))?;
            let data = source
                .get("data")
                .and_then(Value::as_str)
                .ok_or_else(|| bad_request("base64 image source must have data"))?;
            format!("data:{media_type};base64,{data}")
        }
        Some(other) => {
            return Err(bad_request(format!(
                "unsupported image source type {other}"
            )));
        }
        None => {
            return Err(bad_request(
                "image source must have a type of url or base64",
            ));
        }
    };
    Ok(json!({"type": "image_url", "image_url": {"url": url}}))
}

fn collect_text_blocks(blocks: &[Value]) -> Result<String, AdapterError> {
    let mut text = String::new();
    for block in blocks {
        let block = block
            .as_object()
            .ok_or_else(|| bad_request("content blocks must be objects"))?;
        if block.get("type").and_then(Value::as_str) != Some("text") {
            return Err(bad_request("system content blocks support text only"));
        }
        let part = block
            .get("text")
            .and_then(Value::as_str)
            .ok_or_else(|| bad_request("text content block must have string text"))?;
        checked_text(part)?;
        text.push_str(part);
    }
    Ok(text)
}

fn text_with_role(role: &str, text: &str) -> Value {
    json!({"role": role, "content": text})
}
