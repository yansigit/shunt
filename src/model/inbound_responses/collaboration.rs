use std::collections::HashMap;

use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use serde_json::{Map, Value};

const WIRE_PREFIX: &str = "shunt_collaboration__";
const MAX_SCHEMA_DEPTH: usize = 64;

#[derive(Debug, Clone, Default)]
pub(crate) struct Authority {
    wire_to_name: HashMap<String, String>,
}

impl Authority {
    pub(crate) fn authorize(&mut self, name: &str) -> Result<(String, bool), String> {
        if name.is_empty() {
            return Err("collaboration tool name must not be empty".into());
        }
        let wire = format!("{WIRE_PREFIX}{name}");
        match self.wire_to_name.get(&wire) {
            Some(existing) if existing != name => Err("collaboration tool name collision".into()),
            Some(_) => Ok((wire, false)),
            _ => {
                self.wire_to_name.insert(wire.clone(), name.to_string());
                Ok((wire, true))
            }
        }
    }

    pub(crate) fn wire_name(&self, namespace: &str, name: &str) -> Option<&str> {
        (namespace == "collaboration")
            .then(|| format!("{WIRE_PREFIX}{name}"))
            .and_then(|wire| {
                self.wire_to_name
                    .get_key_value(&wire)
                    .map(|(key, _)| key.as_str())
            })
    }

    pub(crate) fn logical_name(&self, wire: &str) -> Option<&str> {
        self.wire_to_name.get(wire).map(String::as_str)
    }
}

pub(crate) fn sanitize_schema(schema: &Value) -> Result<Value, String> {
    sanitize(schema, false, 0)
}

fn sanitize(value: &Value, in_name_bag: bool, depth: usize) -> Result<Value, String> {
    if depth > MAX_SCHEMA_DEPTH {
        return Err("collaboration tool schema exceeded the nesting limit".into());
    }
    match value {
        Value::Array(items) => Ok(Value::Array(
            items
                .iter()
                .map(|item| sanitize(item, false, depth + 1))
                .collect::<Result<_, _>>()?,
        )),
        Value::Object(object) => {
            let mut out = Map::with_capacity(object.len());
            for (key, child) in object {
                if !in_name_bag && key == "encrypted" {
                    continue;
                }
                if matches!(key.as_str(), "const" | "default" | "enum" | "examples") {
                    out.insert(key.clone(), child.clone());
                    continue;
                }
                let child_name_bag = !in_name_bag
                    && matches!(
                        key.as_str(),
                        "properties"
                            | "patternProperties"
                            | "$defs"
                            | "definitions"
                            | "dependentSchemas"
                    );
                out.insert(key.clone(), sanitize(child, child_name_bag, depth + 1)?);
            }
            Ok(Value::Object(out))
        }
        _ => Ok(value.clone()),
    }
}

pub(crate) fn has_unreadable_encrypted_agent_task(input: Option<&Value>) -> bool {
    let Some(items) = input.and_then(Value::as_array) else {
        return false;
    };
    let Some(item) = items.iter().rev().find(|item| {
        !matches!(
            item.get("type").and_then(Value::as_str),
            Some("additional_tools" | "compaction_trigger")
        )
    }) else {
        return false;
    };
    if item.get("type").and_then(Value::as_str) != Some("agent_message") {
        return false;
    }
    let Some(content) = item.get("content").and_then(Value::as_array) else {
        return false;
    };
    let mut encrypted = false;
    let mut readable = String::new();
    for part in content {
        match part.get("type").and_then(Value::as_str) {
            Some("input_text" | "text") => {
                if let Some(text) = part.get("text").and_then(Value::as_str) {
                    readable.push_str(text);
                    readable.push('\n');
                }
            }
            Some("encrypted_content")
                if part
                    .get("encrypted_content")
                    .and_then(Value::as_str)
                    .is_some_and(is_fernet_wire) =>
            {
                encrypted = true;
            }
            _ => {}
        }
    }
    encrypted && strip_routing_envelope(&readable).trim().is_empty()
}

fn strip_routing_envelope(text: &str) -> String {
    let lines = text.lines().collect::<Vec<_>>();
    for (index, line) in lines.iter().enumerate() {
        let Some(kind) = line.trim().strip_prefix("Message Type:") else {
            continue;
        };
        if !matches!(kind.trim(), "NEW_TASK" | "MESSAGE" | "FINAL_ANSWER") {
            continue;
        }
        let Some(task) = lines.get(index + 1).map(|line| line.trim()) else {
            continue;
        };
        let Some(sender) = lines.get(index + 2).map(|line| line.trim()) else {
            continue;
        };
        let Some(payload) = lines.get(index + 3).map(|line| line.trim()) else {
            continue;
        };
        if task
            .strip_prefix("Task name:")
            .is_none_or(|value| value.trim().is_empty())
            || sender
                .strip_prefix("Sender:")
                .is_none_or(|value| value.trim().is_empty())
        {
            continue;
        }
        let Some(inline) = payload.strip_prefix("Payload:") else {
            continue;
        };
        let mut remainder = String::from(inline.trim_start());
        for line in &lines[index + 4..] {
            if !remainder.is_empty() {
                remainder.push('\n');
            }
            remainder.push_str(line);
        }
        return remainder;
    }
    text.to_string()
}

fn is_fernet_wire(value: &str) -> bool {
    if value.len() < 100 || !value.len().is_multiple_of(4) {
        return false;
    }
    let Ok(decoded) = URL_SAFE.decode(value) else {
        return false;
    };
    decoded.first() == Some(&0x80) && decoded.len() >= 73 && (decoded.len() - 57) % 16 == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE;
    use serde_json::json;

    fn fernet() -> String {
        let mut bytes = vec![0x5a; 73];
        bytes[0] = 0x80;
        URL_SAFE.encode(bytes)
    }

    #[test]
    fn schema_marker_is_removed_without_corrupting_named_or_literal_data() {
        let schema = json!({
            "type":"object", "encrypted":true,
            "properties":{
                "encrypted":{"type":"string","default":{"encrypted":true}},
                "message":{"type":"string","encrypted":true}
            },
            "examples":[{"encrypted":true}]
        });
        let clean = sanitize_schema(&schema).unwrap();
        assert!(clean.get("encrypted").is_none());
        assert_eq!(clean["properties"]["encrypted"]["type"], "string");
        assert!(clean["properties"]["message"].get("encrypted").is_none());
        assert_eq!(clean["examples"][0]["encrypted"], true);
    }

    #[test]
    fn unreadable_task_requires_valid_ciphertext_and_no_plaintext_payload() {
        let routing = "Message Type: NEW_TASK\nTask name: /root/worker\nSender: /root\nPayload:\n";
        let input = json!([{"type":"agent_message","content":[
            {"type":"input_text","text":routing},
            {"type":"encrypted_content","encrypted_content":fernet()}
        ]}]);
        assert!(has_unreadable_encrypted_agent_task(Some(&input)));
        let readable = json!([{"type":"agent_message","content":[
            {"type":"input_text","text":format!("{routing}implement the test")},
            {"type":"encrypted_content","encrypted_content":fernet()}
        ]}]);
        assert!(!has_unreadable_encrypted_agent_task(Some(&readable)));
        let unrelated = json!([{"type":"agent_message","content":[
            {"type":"input_text","text":"ordinary prose ending in Payload:"},
            {"type":"encrypted_content","encrypted_content":fernet()}
        ]}]);
        assert!(!has_unreadable_encrypted_agent_task(Some(&unrelated)));
    }
}
