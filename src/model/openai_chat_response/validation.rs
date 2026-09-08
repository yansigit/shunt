use super::{OpenAiChatSemanticError, SIGNED_REASONING_FIELDS};
use serde_json::{json, Value};

pub(super) fn provider_error_data(
    _message: &str,
    _provider_error: &serde_json::Map<String, Value>,
) -> Value {
    let inner = json!({"type": "api_error", "message": "OpenAI Chat backend error"});
    json!({"type": "error", "error": inner})
}

// Normalize the reasoning_content / reasoning aliases: null or empty is
// absent, non-string fails closed, both-present-and-differing fails closed.
pub(super) fn extract_reasoning(
    payload: &Value,
) -> Result<Option<String>, OpenAiChatSemanticError> {
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

pub(super) fn validate_usage(
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
