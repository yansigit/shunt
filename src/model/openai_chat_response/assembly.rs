use super::{
    unary_tools::{MAX_TOOL_ARGUMENT_BYTES, MAX_TOOL_CALLS},
    OpenAiChatSemanticError,
};
use serde_json::{json, Value};

#[derive(Default)]
pub(super) struct ToolAssembly {
    // At most MAX_TOOL_CALLS entries: bounded linear lookup avoids an auxiliary
    // index map and preserves first-arrival order even for sparse indices.
    calls: Vec<Call>,
}

struct Call {
    index: u64,
    id: Option<String>,
    name: Option<String>,
    arguments: String,
}

fn invalid() -> OpenAiChatSemanticError {
    OpenAiChatSemanticError::protocol("invalid OpenAI Chat tool delta or conflicting identity")
}

fn optional_string(value: Option<&Value>) -> Result<Option<&str>, OpenAiChatSemanticError> {
    value.map(|v| v.as_str().ok_or_else(invalid)).transpose()
}

impl ToolAssembly {
    /// Type-check and retain raw argument fragments. JSON is parsed ONLY by
    /// finish(), after the provider's authoritative finish reason arrives.
    pub(super) fn apply(&mut self, value: &Value) -> Result<usize, OpenAiChatSemanticError> {
        let deltas = value.as_array().ok_or_else(invalid)?;
        let mut added = 0usize;
        for delta in deltas {
            let index = delta
                .get("index")
                .and_then(Value::as_u64)
                .ok_or_else(invalid)?;
            let id = optional_string(delta.get("id"))?;
            if let Some(kind) = delta.get("type") {
                if kind.as_str() != Some("function") {
                    return Err(invalid());
                }
            }
            let function = delta
                .get("function")
                .map(|v| v.as_object().ok_or_else(invalid))
                .transpose()?;
            let name = optional_string(function.and_then(|f| f.get("name")))?;
            let args = optional_string(function.and_then(|f| f.get("arguments")))?.unwrap_or("");
            if id.is_some_and(|id| id.is_empty()) || name.is_some_and(|name| name.is_empty()) {
                return Err(invalid());
            }
            if id.is_some_and(|id| {
                self.calls
                    .iter()
                    .any(|call| call.index != index && call.id.as_deref() == Some(id))
            }) {
                return Err(invalid());
            }
            let position = match self.calls.iter().position(|call| call.index == index) {
                Some(position) => position,
                None => {
                    if self.calls.len() == MAX_TOOL_CALLS {
                        return Err(OpenAiChatSemanticError::protocol(
                            "OpenAI Chat MAX_TOOL_CALLS exceeded",
                        ));
                    }
                    self.calls.push(Call {
                        index,
                        id: None,
                        name: None,
                        arguments: String::new(),
                    });
                    self.calls.len() - 1
                }
            };
            let call = &mut self.calls[position];
            if id.zip(call.id.as_deref()).is_some_and(|(a, b)| a != b)
                || name.zip(call.name.as_deref()).is_some_and(|(a, b)| a != b)
            {
                return Err(invalid());
            }
            if call
                .arguments
                .len()
                .checked_add(args.len())
                .is_none_or(|size| size > MAX_TOOL_ARGUMENT_BYTES)
            {
                return Err(OpenAiChatSemanticError::protocol(
                    "OpenAI Chat MAX_TOOL_ARGUMENT_BYTES exceeded",
                ));
            }
            for text in [id, name, Some(args)].into_iter().flatten() {
                added = added.checked_add(text.len()).ok_or_else(invalid)?;
            }
            if call.id.is_none() {
                call.id = id.map(str::to_owned);
            }
            if call.name.is_none() {
                call.name = name.map(str::to_owned);
            }
            call.arguments.push_str(args);
        }
        Ok(added)
    }

    pub(super) fn finish(&mut self) -> Result<Vec<Value>, OpenAiChatSemanticError> {
        std::mem::take(&mut self.calls)
            .into_iter()
            .map(|call| {
                let id = call.id.ok_or_else(invalid)?;
                let name = call.name.ok_or_else(invalid)?;
                let input: Value = serde_json::from_str(&call.arguments).map_err(|_| {
                    OpenAiChatSemanticError::protocol(
                        "incomplete or invalid OpenAI Chat tool arguments at finish",
                    )
                })?;
                if !input.is_object() {
                    return Err(invalid());
                }
                Ok(json!({"type":"tool_use","id":id,"name":name,"input":input}))
            })
            .collect()
    }
}
