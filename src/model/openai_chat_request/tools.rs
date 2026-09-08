use super::{bad_request, only_fields};
use crate::adapters::AdapterError;
use serde_json::{json, Map, Value};

pub(super) fn translate_tools(tools: &Value) -> Result<Value, AdapterError> {
    let tools = tools
        .as_array()
        .ok_or_else(|| bad_request("tools must be an array of tool declarations"))?;
    let mut translated = Vec::with_capacity(tools.len());
    for tool in tools {
        let tool = tool
            .as_object()
            .ok_or_else(|| bad_request("each tool declaration must be an object"))?;
        only_fields(tool, &["name", "description", "input_schema"])?;
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
        if name.is_empty()
            || tool
                .get("description")
                .is_some_and(|value| !value.is_string())
        {
            return Err(bad_request(
                "tool name must be nonempty and description must be a string",
            ));
        }
        function.insert("name".to_string(), json!(name));
        if let Some(description) = tool.get("description") {
            function.insert("description".to_string(), description.clone());
        }
        function.insert("parameters".to_string(), parameters.clone());
        translated.push(json!({"type": "function", "function": Value::Object(function)}));
    }
    Ok(Value::Array(translated))
}

pub(super) fn translate_tool_choice(tool_choice: &Value) -> Result<Value, AdapterError> {
    match tool_choice {
        Value::String(choice) if choice == "auto" => Ok(json!("auto")),
        Value::String(choice) if choice == "any" => Ok(json!("required")),
        Value::Object(choice) => {
            only_fields(choice, &["type", "name"])?;
            match choice.get("type").and_then(Value::as_str) {
                Some("auto") if choice.len() == 1 => return Ok(json!("auto")),
                Some("any") if choice.len() == 1 => return Ok(json!("required")),
                _ => {}
            }
            if choice.get("type").and_then(Value::as_str) != Some("tool") {
                return Err(bad_request(
                    "tool_choice must be \"auto\", \"any\", or a tool selection object",
                ));
            }
            let name = choice
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| bad_request("tool_choice tool selection must have a name"))?;
            if name.is_empty() {
                return Err(bad_request("tool_choice name must not be empty"));
            }
            Ok(json!({"type": "function", "function": {"name": name}}))
        }
        _ => Err(bad_request(
            "tool_choice must be \"auto\", \"any\", or a tool selection object",
        )),
    }
}
