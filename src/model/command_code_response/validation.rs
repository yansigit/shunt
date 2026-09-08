use super::SemanticError;
use serde_json::{json, Value};

fn integer(value: Option<&Value>) -> Result<u64, SemanticError> {
    value.map_or(Ok(0), |v| {
        v.as_u64()
            .ok_or_else(|| SemanticError::protocol("invalid subscription usage integer"))
    })
}

pub(super) fn record_usage(record: &Value) -> Result<Value, SemanticError> {
    // totalUsage has source-defined precedence, but an invalid second semantic
    // field must not be hidden behind it. Per-step and total values may differ.
    let total = record
        .get("totalUsage")
        .map(|v| usage(Some(v)))
        .transpose()?;
    let step = record.get("usage").map(|v| usage(Some(v))).transpose()?;
    match total.or(step) {
        Some(value) => Ok(value),
        None => usage(None),
    }
}

pub(super) fn usage(value: Option<&Value>) -> Result<Value, SemanticError> {
    let empty = json!({});
    let value = value.unwrap_or(&empty);
    if !value.is_object() {
        return Err(SemanticError::protocol(
            "subscription usage must be an object",
        ));
    }
    let input = integer(value.get("inputTokens"))?;
    let output = integer(value.get("outputTokens"))?;
    let total = input
        .checked_add(output)
        .ok_or_else(|| SemanticError::protocol("subscription usage overflow"))?;
    if value.get("totalTokens").is_some() && integer(value.get("totalTokens"))? != total {
        return Err(SemanticError::protocol(
            "inconsistent subscription total usage",
        ));
    }
    let details = value.get("inputTokenDetails").unwrap_or(&empty);
    if !details.is_object() {
        return Err(SemanticError::protocol(
            "invalid subscription cache details",
        ));
    }
    let read = integer(details.get("cacheReadTokens"))?;
    let write = integer(details.get("cacheWriteTokens"))?;
    let cache = read
        .checked_add(write)
        .ok_or_else(|| SemanticError::protocol("subscription cache usage overflow"))?;
    let uncached = input
        .checked_sub(cache)
        .ok_or_else(|| SemanticError::protocol("cache usage exceeds input tokens"))?;
    Ok(json!({"input_tokens":uncached,"output_tokens":output,
        "cache_read_input_tokens":read,"cache_creation_input_tokens":write}))
}

pub(super) fn reason(record: &Value) -> Result<&str, SemanticError> {
    let raw = record.get("rawFinishReason");
    let normalized = record.get("finishReason");
    for value in [raw, normalized].into_iter().flatten() {
        if !value.is_string() {
            return Err(SemanticError::protocol(
                "invalid subscription finish reason",
            ));
        }
    }
    if let (Some(raw), Some(normalized)) = (
        raw.and_then(Value::as_str),
        normalized.and_then(Value::as_str),
    ) {
        if stop(raw)? != stop(normalized)? {
            return Err(SemanticError::protocol(
                "conflicting subscription finish reasons",
            ));
        }
    }
    raw.or(normalized)
        .and_then(Value::as_str)
        .ok_or_else(|| SemanticError::protocol("missing subscription finish reason"))
}

pub(super) fn stop(reason: &str) -> Result<&'static str, SemanticError> {
    match reason {
        "stop" | "end_turn" => Ok("end_turn"),
        "length" | "max_tokens" => Ok("max_tokens"),
        "tool_use" | "tool-calls" | "tool_calls" => Ok("tool_use"),
        "error" => Ok("error"),
        _ => Err(SemanticError::protocol(
            "unsupported subscription finish reason",
        )),
    }
}

pub(super) fn identity<'a>(record: &'a Value, key: &str) -> Result<&'a str, SemanticError> {
    record
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty() && s.len() <= 256 && s.bytes().all(|b| b.is_ascii_graphic()))
        .ok_or_else(|| SemanticError::protocol("invalid subscription tool identity"))
}
