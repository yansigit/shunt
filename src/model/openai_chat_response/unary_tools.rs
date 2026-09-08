use super::{OpenAiChatSemanticError, MAX_RETAINED_SEMANTIC_BYTES};
use serde_json::{json, Value};
use std::collections::HashSet;

pub(super) const MAX_TOOL_CALLS: usize = 128;
pub(super) const MAX_TOOL_ARGUMENT_BYTES: usize = 1024 * 1024;

pub(super) fn translate(
    value: Option<&Value>,
    bytes: &mut usize,
) -> Result<Vec<Value>, OpenAiChatSemanticError> {
    let bad = || OpenAiChatSemanticError::protocol("invalid or oversized OpenAI Chat tool_calls");
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let calls = value
        .as_array()
        .filter(|calls| calls.len() <= MAX_TOOL_CALLS)
        .ok_or_else(bad)?;
    let mut ids = HashSet::new();
    calls
        .iter()
        .map(|call| {
            let id = call
                .get("id")
                .and_then(Value::as_str)
                .filter(|id| !id.is_empty())
                .ok_or_else(bad)?;
            if call.get("type").and_then(Value::as_str) != Some("function") || !ids.insert(id) {
                return Err(bad());
            }
            let function = call
                .get("function")
                .and_then(Value::as_object)
                .ok_or_else(bad)?;
            let name = function
                .get("name")
                .and_then(Value::as_str)
                .filter(|name| !name.is_empty())
                .ok_or_else(bad)?;
            let args = function
                .get("arguments")
                .and_then(Value::as_str)
                .filter(|args| args.len() <= MAX_TOOL_ARGUMENT_BYTES)
                .ok_or_else(bad)?;
            for size in [id.len(), name.len(), args.len()] {
                *bytes = bytes
                    .checked_add(size)
                    .filter(|bytes| *bytes <= MAX_RETAINED_SEMANTIC_BYTES)
                    .ok_or_else(bad)?;
            }
            let input: Value = serde_json::from_str(args).map_err(|_| bad())?;
            if !input.is_object() {
                return Err(bad());
            }
            Ok(json!({"type":"tool_use","id":id,"name":name,"input":input}))
        })
        .collect()
}
