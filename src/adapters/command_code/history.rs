//! Product-local AI SDK history translation. Adapted from OpenCodex 055c3ecf;
//! see THIRD-PARTY-NOTICES.md. No changes to the generic Chat pairing policy.
use super::invalid;
use crate::adapters::AdapterError;
use base64::Engine;
use serde_json::{json, Value};
use std::collections::HashSet;

pub(super) fn fields(value: &Value, allowed: &[&str]) -> Result<(), AdapterError> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid("expected an object"))?;
    if object.keys().any(|k| !allowed.contains(&k.as_str())) {
        return Err(invalid("unsupported Command Code history or tool field"));
    }
    Ok(())
}

pub(super) fn identity<'a>(value: &'a Value, key: &str) -> Result<&'a str, AdapterError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty() && s.len() <= 256 && s.bytes().all(|b| b.is_ascii_graphic()))
        .ok_or_else(|| invalid("invalid tool identity"))
}

fn text(block: &Value) -> Result<Value, AdapterError> {
    fields(block, &["type", "text"])?;
    let text = block
        .get("text")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid("text must be a string"))?;
    Ok(json!({"type":"text", "text":text}))
}

fn image(block: &Value) -> Result<Value, AdapterError> {
    fields(block, &["type", "source"])?;
    let source = block
        .get("source")
        .ok_or_else(|| invalid("missing image source"))?;
    match source.get("type").and_then(Value::as_str) {
        Some("base64") => {
            fields(source, &["type", "media_type", "data"])?;
            let media = source
                .get("media_type")
                .and_then(Value::as_str)
                .filter(|m| ["image/png", "image/jpeg", "image/gif", "image/webp"].contains(m))
                .ok_or_else(|| invalid("unsupported image media type"))?;
            let data = source
                .get("data")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| invalid("missing image data"))?;
            base64::engine::general_purpose::STANDARD
                .decode(data)
                .map_err(|_| invalid("invalid base64 image"))?;
            Ok(
                json!({"type":"image", "image":format!("data:{media};base64,{data}"), "mediaType":media}),
            )
        }
        Some("url") => {
            fields(source, &["type", "url"])?;
            let raw = source
                .get("url")
                .and_then(Value::as_str)
                .ok_or_else(|| invalid("missing image URL"))?;
            let url = reqwest::Url::parse(raw).map_err(|_| invalid("invalid image URL"))?;
            if url.scheme() != "https"
                || url.host_str().is_none()
                || !url.username().is_empty()
                || url.password().is_some()
            {
                return Err(invalid("image URL must be credential-free HTTPS"));
            }
            let mut part = json!({"type":"image", "image":raw});
            let extension = url
                .path()
                .rsplit('.')
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();
            let media = match extension.as_str() {
                "png" => Some("image/png"),
                "jpg" | "jpeg" => Some("image/jpeg"),
                "gif" => Some("image/gif"),
                "webp" => Some("image/webp"),
                "bmp" => Some("image/bmp"),
                _ => None,
            };
            if let Some(media) = media {
                part["mediaType"] = json!(media);
            }
            Ok(part)
        }
        _ => Err(invalid("unsupported image source")),
    }
}

fn result_content(content: Option<&Value>) -> Result<(String, Vec<Value>), AdapterError> {
    match content {
        None => Ok((String::new(), Vec::new())),
        Some(Value::String(s)) => Ok((s.clone(), Vec::new())),
        Some(Value::Array(blocks)) => {
            let mut output = String::new();
            let mut images = Vec::new();
            for block in blocks {
                match block.get("type").and_then(Value::as_str) {
                    Some("text") => output.push_str(text(block)?["text"].as_str().unwrap()),
                    Some("image") => {
                        images.push(image(block)?);
                        output.push_str("[image]");
                    }
                    _ => return Err(invalid("unsupported tool result content")),
                }
            }
            Ok((output, images))
        }
        _ => Err(invalid("invalid tool result content")),
    }
}

#[derive(Default)]
struct History {
    out: Vec<Value>,
    pending: Vec<(String, String)>,
    image_carriers: Vec<Value>,
    calls: HashSet<String>,
    results: HashSet<String>,
}

impl History {
    fn close_pending(&mut self) {
        for (id, name) in self.pending.drain(..) {
            self.out.push(json!({"role":"tool", "content":[{
                "type":"tool-result", "toolCallId":id, "toolName":name,
                "output":{"type":"error-text", "value":"[shunt] no tool result was recorded for this tool call; execution status unknown."}
            }]}));
        }
        self.out.append(&mut self.image_carriers);
    }

    fn result(&mut self, block: &Value) -> Result<(), AdapterError> {
        fields(block, &["type", "tool_use_id", "content", "is_error"])?;
        let id = identity(block, "tool_use_id")?;
        if !self.results.insert(id.into()) {
            return Err(invalid("duplicate tool result identity"));
        }
        let is_error = match block.get("is_error") {
            None => false,
            Some(v) => v
                .as_bool()
                .ok_or_else(|| invalid("is_error must be a boolean"))?,
        };
        let (value, images) = result_content(block.get("content"))?;
        if let Some(index) = self.pending.iter().position(|(call, _)| call == id) {
            let (_, name) = self.pending.remove(index);
            self.out.push(json!({"role":"tool", "content":[{
                "type":"tool-result", "toolCallId":id, "toolName":name,
                "output":{"type":if is_error {"error-text"} else {"text"}, "value":value}
            }]}));
            if !images.is_empty() {
                self.image_carriers
                    .push(json!({"role":"user", "content":images}));
            }
        } else {
            self.close_pending();
            self.out
                .push(json!({"role":"user", "content":[{"type":"text",
                "text":format!("[tool result without adjacent tool call: {id}]\n{value}") }]}));
            if !images.is_empty() {
                self.out.push(json!({"role":"user", "content":images}));
            }
        }
        Ok(())
    }

    fn assistant(&mut self, content: &Value) -> Result<(), AdapterError> {
        self.close_pending();
        let mut wire = Vec::new();
        if let Some(text) = content.as_str() {
            wire.push(json!({"type":"text", "text":text}));
        } else {
            for block in content
                .as_array()
                .ok_or_else(|| invalid("invalid assistant content"))?
            {
                match block.get("type").and_then(Value::as_str) {
                    Some("text") => wire.push(text(block)?),
                    Some("thinking") => {
                        fields(block, &["type", "thinking"])?;
                        let thinking = block
                            .get("thinking")
                            .and_then(Value::as_str)
                            .ok_or_else(|| invalid("thinking must be a string"))?;
                        wire.push(json!({"type":"reasoning", "text":thinking}));
                    }
                    Some("tool_use") => {
                        fields(block, &["type", "id", "name", "input"])?;
                        let id = identity(block, "id")?;
                        let name = identity(block, "name")?;
                        if self.results.contains(id) || !self.calls.insert(id.into()) {
                            return Err(invalid("duplicate or ambiguous tool call identity"));
                        }
                        let input = block
                            .get("input")
                            .filter(|v| v.is_object())
                            .ok_or_else(|| invalid("tool input must be an object"))?;
                        wire.push(json!({"type":"tool-call", "toolCallId":id, "toolName":name, "input":input}));
                        self.pending.push((id.into(), name.into()));
                    }
                    _ => return Err(invalid("unsupported assistant history block")),
                }
            }
        }
        self.out.push(json!({"role":"assistant", "content":wire}));
        Ok(())
    }

    fn user(&mut self, content: &Value) -> Result<(), AdapterError> {
        if let Some(text) = content.as_str() {
            self.close_pending();
            self.out
                .push(json!({"role":"user", "content":[{"type":"text", "text":text}]}));
            return Ok(());
        }
        let mut ordinary = Vec::new();
        for block in content
            .as_array()
            .ok_or_else(|| invalid("invalid user content"))?
        {
            match block.get("type").and_then(Value::as_str) {
                Some("tool_result") => {
                    if !ordinary.is_empty() {
                        self.out.push(json!({"role":"user", "content":ordinary}));
                        ordinary = Vec::new();
                    }
                    self.result(block)?;
                }
                Some("text") | Some("image") => {
                    if ordinary.is_empty() {
                        self.close_pending();
                    }
                    ordinary.push(if block["type"] == "text" {
                        text(block)?
                    } else {
                        image(block)?
                    });
                }
                _ => return Err(invalid("unsupported user history block")),
            }
        }
        if !ordinary.is_empty() {
            self.out.push(json!({"role":"user", "content":ordinary}));
        }
        Ok(())
    }
}

pub(super) fn compile(messages: &Value) -> Result<Vec<Value>, AdapterError> {
    let messages = messages
        .as_array()
        .filter(|m| !m.is_empty())
        .ok_or_else(|| invalid("messages must be a nonempty array"))?;
    let mut history = History::default();
    for message in messages {
        fields(message, &["role", "content"])?;
        let content = message
            .get("content")
            .ok_or_else(|| invalid("missing message content"))?;
        match message.get("role").and_then(Value::as_str) {
            Some("assistant") => history.assistant(content)?,
            Some("user") => history.user(content)?,
            _ => return Err(invalid("unsupported message role")),
        }
    }
    history.close_pending();
    Ok(history.out)
}
