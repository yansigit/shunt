use super::{efforts, invalid};
use crate::adapters::AdapterError;
use serde_json::{json, Value};

pub fn translate_request(body: &Value, model: &str, route_effort: Option<&str>) -> Result<Value, AdapterError> {
    let effort = efforts::resolve(body, model, route_effort).map_err(invalid)?;
    // Narrow text tracer only. The next request plan expands the explicit contract.
    if body.get("stream").is_some_and(|v| !v.is_boolean()) {
        return Err(invalid("stream must be a boolean"));
    }
    if body.get("stream").and_then(Value::as_bool) == Some(true) {
        return Err(invalid(
            "Command Code streaming client support is not implemented in this slice",
        ));
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
        wire.push(json!({"role":role,"content":content}));
    }
    let max_tokens = match body.get("max_tokens") {
        Some(v) => v
            .as_u64()
            .filter(|n| *n > 0)
            .ok_or_else(|| invalid("max_tokens must be a positive integer"))?,
        None => 64000,
    };
    let system = match body.get("system") {
        Some(v) => v
            .as_str()
            .ok_or_else(|| invalid("system must be a string"))?,
        None => "",
    };
    let mut payload = json!({"config":{},"memory":"","taste":null,"skills":null,"permissionMode":"standard","mode":"agent",
        "params":{"model":model,"messages":wire,"tools":[],"system":system,"max_tokens":max_tokens,"stream":true}});
    if let Some(effort) = effort {
        payload["params"]["reasoning_effort"] = json!(effort);
    }
    Ok(payload)
}
