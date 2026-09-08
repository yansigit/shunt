use super::{efforts, invalid};
use crate::adapters::AdapterError;
use axum::http::{HeaderMap, HeaderValue};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub const MAX_CONVERSATION_ID_BYTES: usize = 1024;
// Pinned source 055c3ecf, inspected 2026-09-08. Not a live version claim.
pub const CLIENT_VERSION: &str = "0.52.1";

pub fn session_id(token: &str, conversation: Option<&str>) -> Result<String, AdapterError> {
    let Some(conversation) = conversation else {
        // Request-local only: stable across retries, NOT across independent turns.
        return Ok(uuid::Uuid::new_v4().to_string());
    };
    if conversation.is_empty()
        || conversation.len() > MAX_CONVERSATION_ID_BYTES
        || !conversation.bytes().all(|b| b.is_ascii_graphic())
    {
        return Err(invalid("invalid Command Code conversation identity"));
    }
    let mut hash = Sha256::new();
    hash.update(b"shunt/command-code/session/v1\0");
    for value in [token, conversation] {
        hash.update((value.len() as u64).to_be_bytes());
        hash.update(value.as_bytes());
    }
    let digest = hash.finalize();
    let mut bytes = [0; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Ok(uuid::Uuid::from_bytes(bytes).to_string())
}

pub fn headers(token: &str, conversation: Option<&str>) -> Result<HeaderMap, AdapterError> {
    if token.is_empty() || token.len() > 16384 || !token.bytes().all(|b| b.is_ascii_graphic()) {
        return Err(crate::auth::auth_error(
            "Command Code subscription token is invalid",
        ));
    }
    let mut headers = HeaderMap::new();
    let mut bearer = HeaderValue::from_str(&format!("Bearer {token}"))
        .map_err(|_| crate::auth::auth_error("Command Code subscription token is invalid"))?;
    bearer.set_sensitive(true);
    headers.insert("authorization", bearer);
    for (name, value) in [
        ("content-type", "application/json"),
        ("user-agent", "cli"),
        ("x-command-code-version", CLIENT_VERSION),
        ("x-cli-environment", "production"),
        ("x-taste-learning", "false"),
        ("x-co-flag", "false"),
    ] {
        headers.insert(name, HeaderValue::from_static(value));
    }
    headers.insert(
        "x-session-id",
        HeaderValue::from_str(&session_id(token, conversation)?)
            .map_err(|_| invalid("invalid Command Code session identity"))?,
    );
    Ok(headers)
}

pub fn translate_request(
    body: &Value,
    model: &str,
    route_effort: Option<&str>,
) -> Result<Value, AdapterError> {
    let effort = efforts::resolve(body, model, route_effort).map_err(invalid)?;
    if body.get("stream").is_some_and(|v| !v.is_boolean()) {
        return Err(invalid("stream must be a boolean"));
    }
    for key in body
        .as_object()
        .ok_or_else(|| invalid("request must be an object"))?
        .keys()
    {
        if ![
            "model",
            "messages",
            "max_tokens",
            "stream",
            "system",
            "output_config",
            "temperature",
        ]
        .contains(&key.as_str())
        {
            return Err(invalid("unsupported Command Code request field"));
        }
    }
    let messages = body
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("messages must be an array"))?;
    if messages.is_empty() {
        return Err(invalid("messages must not be empty"));
    }
    let mut wire = Vec::new();
    for message in messages {
        if message
            .as_object()
            .is_none_or(|m| m.keys().any(|k| !["role", "content"].contains(&k.as_str())))
        {
            return Err(invalid("unsupported message field"));
        }
        let role = message
            .get("role")
            .and_then(Value::as_str)
            .filter(|r| matches!(*r, "user" | "assistant"))
            .ok_or_else(|| invalid("unsupported message role"))?;
        let content = message
            .get("content")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("text-only Command Code tracer requires string content"))?;
        wire.push(json!({"role":role,"content":[{"type":"text","text":content}]}));
    }
    let max_tokens = match body.get("max_tokens") {
        Some(v) => v
            .as_u64()
            .filter(|n| *n > 0)
            .ok_or_else(|| invalid("max_tokens must be a positive integer"))?,
        None => 64000,
    };
    let system = match body.get("system") {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Array(blocks)) => blocks
            .iter()
            .map(|block| {
                let object = block
                    .as_object()
                    .ok_or_else(|| invalid("invalid system block"))?;
                if object
                    .keys()
                    .any(|key| !["type", "text"].contains(&key.as_str()))
                    || block.get("type").and_then(Value::as_str) != Some("text")
                {
                    return Err(invalid("unsupported system block"));
                }
                block
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(|| invalid("system text must be a string"))
            })
            .collect::<Result<Vec<_>, _>>()?
            .join("\n\n"),
        Some(_) => return Err(invalid("system must be a string or text block array")),
        None => String::new(),
    };
    let mut payload = json!({"config":{},"memory":"","taste":null,"skills":null,"permissionMode":"standard","mode":"agent",
        "params":{"model":model,"messages":wire,"tools":[],"system":system,"max_tokens":max_tokens,"stream":true}});
    if let Some(effort) = effort {
        payload["params"]["reasoning_effort"] = json!(effort);
    }
    if let Some(temperature) = body.get("temperature") {
        if !temperature.as_f64().is_some_and(f64::is_finite) {
            return Err(invalid("temperature must be a finite number"));
        }
        payload["params"]["temperature"] = temperature.clone();
    }
    Ok(payload)
}
