//! Supported active-Run inputs. Rejection happens before credentials/dispatch;
//! the retired XML/tool bridge is not a fallback for unrepresentable input.
use serde_json::Value;

pub(super) fn tools(request: &Value) -> Result<Vec<super::agent::AgentTool>, String> {
    let Some(value) = request.get("tools") else {
        return Ok(Vec::new());
    };
    let tools = value.as_array().ok_or("Cursor tools must be an array")?;
    let mut names = std::collections::HashSet::new();
    tools
        .iter()
        .map(|tool| {
            let name = tool
                .get("name")
                .and_then(Value::as_str)
                .filter(|n| !n.trim().is_empty())
                .ok_or("Cursor tool requires a nonempty name")?;
            if !names.insert(name) {
                return Err("Cursor tool names must be unique".into());
            }
            if tool.get("type").is_some_and(|v| v != "custom") {
                return Err(
                    "Cursor supports custom MCP tool definitions, not server builtin tool types"
                        .into(),
                );
            }
            let schema = tool
                .get("input_schema")
                .filter(|v| v.is_object())
                .ok_or("Cursor tool input_schema must be an explicit object")?;
            if schema.get("type").is_some_and(|v| v != "object") {
                return Err("Cursor tool input_schema must describe object arguments".into());
            }
            let description = match tool.get("description") {
                None => "",
                Some(v) => v.as_str().ok_or("Cursor tool description must be text")?,
            };
            check_depth(schema, 0)?;
            Ok(super::agent::AgentTool {
                name: name.into(),
                description: description.into(),
                input_schema: schema.clone(),
            })
        })
        .collect()
}

fn check_depth(value: &Value, depth: usize) -> Result<(), String> {
    if depth >= 64 {
        return Err("Cursor arguments or schema exceed supported nesting depth".into());
    }
    match value {
        Value::Array(values) => {
            for v in values {
                check_depth(v, depth + 1)?;
            }
        }
        Value::Object(values) => {
            for v in values.values() {
                check_depth(v, depth + 1)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn validate(request: &Value) -> Result<(), String> {
    if let Some(choice) = request.get("tool_choice") {
        let choice = choice
            .as_object()
            .ok_or("Cursor tool_choice must be an auto object")?;
        if choice.get("type").is_none_or(|v| v != "auto") || choice.keys().any(|k| k != "type") {
            return Err("Cursor supports only automatic tool choice without forced or parallel-use guidance".into());
        }
    }
    if let Some(messages) = request.get("messages").and_then(Value::as_array) {
        for message in messages {
            if let Some(blocks) = message.get("content").and_then(Value::as_array) {
                for block in blocks {
                    validate_block(block, 0)?;
                }
            }
        }
    }
    Ok(())
}

fn validate_block(block: &Value, depth: usize) -> Result<(), String> {
    if depth >= 64 {
        return Err("Cursor content exceeds supported nesting depth".into());
    }
    match block.get("type").and_then(Value::as_str) {
        Some("image") => {
            let source = block
                .get("source")
                .ok_or("Cursor image requires a source")?;
            if source["type"] != "base64" {
                return Err(
                    "Cursor supports only base64 image sources; URL images are unsupported".into(),
                );
            }
            source
                .get("data")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .ok_or("Cursor image requires nonempty base64 data")?;
            if !matches!(
                source.get("media_type").and_then(Value::as_str),
                Some("image/png" | "image/jpeg" | "image/gif" | "image/webp")
            ) {
                return Err(
                    "Cursor image requires a supported media_type (PNG, JPEG, GIF or WebP)".into(),
                );
            }
        }
        Some("tool_use") => {
            let input = block
                .get("input")
                .filter(|v| v.is_object())
                .ok_or("Cursor tool arguments must be an object")?;
            check_depth(input, 0)?;
        }
        Some("tool_result") => {
            if let Some(blocks) = block.get("content").and_then(Value::as_array) {
                for child in blocks {
                    validate_block(child, depth + 1)?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn cursor_admission_rejects_unsupported_arguments_and_guidance() {
        for input in [Value::Null, json!("{\"path\":1}"), json!([]), json!(1)] {
            let req = json!({"messages":[{"role":"assistant","content":[{"type":"tool_use","input":input}]}]});
            assert!(validate(&req).unwrap_err().contains("arguments"));
        }
        for choice in [
            json!("auto"),
            json!({"type":"tool","name":"Read"}),
            json!({"type":"auto","disable_parallel_tool_use":true}),
        ] {
            assert!(validate(&json!({"tool_choice":choice})).is_err());
        }
        assert!(validate(&json!({"tool_choice":{"type":"auto"}})).is_ok());
        let mut nested = json!({});
        for _ in 0..64 {
            nested = json!({"child":nested});
        }
        assert!(tools(&json!({"tools":[{"name":"Read","input_schema":nested}]})).is_err());
    }

    #[test]
    fn cursor_admission_rejects_duplicate_names_and_bad_image_metadata() {
        let tool = json!({"name":"Read","input_schema":{"type":"object"}});
        assert!(tools(&json!({"tools":[tool,tool]})).is_err());
        for source in [
            json!({}),
            json!({"type":"base64","data":"aGk="}),
            json!({"type":"base64","media_type":"text/plain","data":"aGk="}),
        ] {
            assert!(validate(
                &json!({"messages":[{"role":"user","content":[{"type":"image","source":source}]}]})
            )
            .is_err());
        }
    }
}
