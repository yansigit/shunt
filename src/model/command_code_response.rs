//! Checked Command Code NDJSON semantics shared by streaming and unary modes.
//! OpenCodex 055c3ecf source-derived wire; see THIRD-PARTY-NOTICES.md.
pub use crate::model::gemini::SseEvent;
use serde_json::{json, Value};
use std::collections::HashSet;
mod validation;

pub const MAX_SEMANTIC_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_TOOL_ARGUMENT_BYTES: usize = 512 * 1024;
pub const MAX_TOOLS: usize = 128;
pub const MAX_CONTENT_BLOCKS: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    Protocol,
    UnknownRecord,
    Provider,
}

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub kind: FailureKind,
    pub message: &'static str,
    pub usage: Option<Value>,
}
impl SemanticError {
    pub(crate) fn protocol(message: &'static str) -> Self {
        Self {
            kind: FailureKind::Protocol,
            message,
            usage: None,
        }
    }
    pub fn body(&self) -> Value {
        let mut body = json!({"type":"error","error":{"type":"api_error","message":self.message}});
        if let Some(usage) = &self.usage {
            body["usage"] = usage.clone();
        }
        body
    }
}
impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message)
    }
}
impl std::error::Error for SemanticError {}

fn event(name: &str, data: Value) -> SseEvent {
    SseEvent {
        event: name.into(),
        data,
    }
}

pub struct CommandCodeMachine {
    model: String,
    id: String,
    terminal: Option<(String, String)>,
    companion: bool,
    failed: bool,
    closed: bool,
    started: bool,
    accumulate: bool,
    active: Option<(&'static str, usize)>,
    next_index: usize,
    content: Vec<Value>,
    semantic_bytes: usize,
    tools: HashSet<String>,
    usage: Value,
}
impl CommandCodeMachine {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            id: format!("msg_{}", uuid::Uuid::new_v4()),
            terminal: None,
            companion: false,
            failed: false,
            closed: false,
            started: false,
            accumulate: true,
            active: None,
            next_index: 0,
            content: Vec::new(),
            semantic_bytes: 0,
            tools: HashSet::new(),
            usage: json!({"input_tokens":0,"output_tokens":0,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}),
        }
    }
    pub fn new_streaming(model: impl Into<String>) -> Self {
        let mut machine = Self::new(model);
        machine.accumulate = false;
        machine
    }
    pub fn retained_content_blocks(&self) -> usize {
        self.content.len()
    }

    fn reserve(&mut self, bytes: usize) -> Result<(), SemanticError> {
        let total = self
            .semantic_bytes
            .checked_add(bytes)
            .ok_or_else(|| SemanticError::protocol("subscription semantic budget overflow"))?;
        if total > MAX_SEMANTIC_BYTES {
            return Err(SemanticError::protocol(
                "subscription semantic budget exceeded",
            ));
        }
        self.semantic_bytes = total;
        Ok(())
    }
    fn start(&mut self, out: &mut Vec<SseEvent>) {
        if self.started {
            return;
        }
        self.started = true;
        out.push(event(
            "message_start",
            json!({"type":"message_start","message":{
            "id":self.id,"type":"message","role":"assistant","model":self.model,
            "content":[],"stop_reason":null,"stop_sequence":null,"usage":self.usage}}),
        ));
    }
    fn close_block(&mut self, out: &mut Vec<SseEvent>) {
        if let Some((_, index)) = self.active.take() {
            out.push(event(
                "content_block_stop",
                json!({"type":"content_block_stop","index":index}),
            ));
        }
    }
    fn block(
        &mut self,
        kind: &'static str,
        initial: Value,
        out: &mut Vec<SseEvent>,
    ) -> Result<usize, SemanticError> {
        if self.next_index >= MAX_CONTENT_BLOCKS {
            return Err(SemanticError::protocol(
                "subscription content block budget exceeded",
            ));
        }
        self.close_block(out);
        self.start(out);
        let index = self.next_index;
        self.next_index += 1;
        if self.accumulate {
            self.content.push(initial.clone());
        }
        self.active = Some((kind, index));
        out.push(event(
            "content_block_start",
            json!({"type":"content_block_start","index":index,"content_block":initial}),
        ));
        Ok(index)
    }
    fn delta(&mut self, kind: &'static str, text: &str) -> Result<Vec<SseEvent>, SemanticError> {
        self.reserve(text.len())?;
        let mut out = Vec::new();
        let field = if kind == "thinking" {
            "thinking"
        } else {
            "text"
        };
        let index = match self.active {
            Some((active, index)) if active == kind => index,
            _ => self.block(kind, json!({"type":kind,(field):""}), &mut out)?,
        };
        if self.accumulate {
            if let Value::String(current) = &mut self.content[index][field] {
                current.push_str(text);
            }
        }
        let delta_kind = if kind == "thinking" {
            "thinking_delta"
        } else {
            "text_delta"
        };
        out.push(event(
            "content_block_delta",
            json!({"type":"content_block_delta","index":index,
            "delta":{"type":delta_kind,(field):text}}),
        ));
        Ok(out)
    }
    pub fn process_record_checked(
        &mut self,
        record: &Value,
    ) -> Result<Vec<SseEvent>, SemanticError> {
        if self.failed || self.closed {
            return Err(SemanticError::protocol(
                "subscription machine already terminated",
            ));
        }
        let result = self.process(record);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn process(&mut self, record: &Value) -> Result<Vec<SseEvent>, SemanticError> {
        let kind = record
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| SemanticError::protocol("subscription record requires a type"))?;
        if self.terminal.is_some() && !matches!(kind, "finish" | "finish-step") {
            return Err(SemanticError::protocol(
                "subscription content after terminal",
            ));
        }
        match kind {
            "text-delta" | "reasoning-delta" => {
                let text = record
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(|| SemanticError::protocol("invalid subscription text delta"))?;
                self.delta(
                    if kind == "text-delta" {
                        "text"
                    } else {
                        "thinking"
                    },
                    text,
                )
            }
            "tool-call" => {
                let id = validation::identity(record, "toolCallId")?;
                let name = validation::identity(record, "toolName")?;
                if self.tools.len() >= MAX_TOOLS || self.tools.contains(id) {
                    return Err(SemanticError::protocol(
                        "duplicate tool identity or tool budget exceeded",
                    ));
                }
                if record.get("input").is_some() && record.get("args").is_some() {
                    return Err(SemanticError::protocol(
                        "ambiguous subscription tool arguments",
                    ));
                }
                let raw = record
                    .get("input")
                    .or_else(|| record.get("args"))
                    .ok_or_else(|| {
                        SemanticError::protocol("missing subscription tool arguments")
                    })?;
                let arguments = if let Some(text) = raw.as_str() {
                    text.to_string()
                } else {
                    raw.to_string()
                };
                if arguments.len() > MAX_TOOL_ARGUMENT_BYTES {
                    return Err(SemanticError::protocol(
                        "subscription tool argument budget exceeded",
                    ));
                }
                let input: Value = serde_json::from_str(&arguments)
                    .map_err(|_| SemanticError::protocol("invalid subscription tool arguments"))?;
                if !input.is_object() {
                    return Err(SemanticError::protocol(
                        "subscription tool arguments must be an object",
                    ));
                }
                self.reserve(arguments.len() + id.len() + name.len())?;
                self.tools.insert(id.into());
                let mut out = Vec::new();
                let index = self.block(
                    "tool_use",
                    json!({"type":"tool_use","id":id,"name":name,"input":{}}),
                    &mut out,
                )?;
                if self.accumulate {
                    self.content[index]["input"] = input;
                }
                out.push(event(
                    "content_block_delta",
                    json!({"type":"content_block_delta","index":index,
                    "delta":{"type":"input_json_delta","partial_json":arguments}}),
                ));
                self.close_block(&mut out);
                Ok(out)
            }
            "finish-step" | "finish" => {
                let reason = validation::reason(record)?;
                let mapped = validation::stop(reason)?;
                let usage =
                    validation::usage(record.get("totalUsage").or_else(|| record.get("usage")))?;
                if let Some((first, prior)) = &self.terminal {
                    if first != "finish-step"
                        || kind != "finish"
                        || self.companion
                        || prior != reason
                    {
                        return Err(SemanticError::protocol(
                            "conflicting or duplicate subscription terminal",
                        ));
                    }
                    self.companion = true;
                } else {
                    self.usage = usage;
                    self.terminal = Some((kind.into(), reason.into()));
                }
                if mapped == "error" {
                    return Err(SemanticError {
                        kind: FailureKind::Provider,
                        message: "Command Code subscription backend reported a failed turn",
                        usage: Some(self.usage.clone()),
                    });
                }
                Ok(Vec::new())
            }
            "error" => {
                let usage = record
                    .get("usage")
                    .or_else(|| record.get("totalUsage"))
                    .map(|v| validation::usage(Some(v)))
                    .transpose()?;
                Err(SemanticError {
                    kind: FailureKind::Provider,
                    message: "Command Code subscription backend reported an error",
                    usage,
                })
            }
            _ => Err(SemanticError {
                kind: FailureKind::UnknownRecord,
                message: "unsupported subscription record type",
                usage: None,
            }),
        }
    }

    pub fn transport_close_checked(&mut self) -> Result<Vec<SseEvent>, SemanticError> {
        if self.failed {
            return Err(SemanticError::protocol("subscription machine failed"));
        }
        if self.closed {
            return Ok(Vec::new());
        }
        let Some((_, reason)) = &self.terminal else {
            self.failed = true;
            return Err(SemanticError::protocol("subscription EOF without terminal"));
        };
        let reason = validation::stop(reason)?;
        let mut out = Vec::new();
        self.start(&mut out);
        self.close_block(&mut out);
        out.push(event(
            "message_delta",
            json!({"type":"message_delta",
            "delta":{"stop_reason":reason,"stop_sequence":null},"usage":self.usage}),
        ));
        out.push(event("message_stop", json!({"type":"message_stop"})));
        self.closed = true;
        Ok(out)
    }
    pub fn final_ndjson_checked(&mut self) -> Result<Value, SemanticError> {
        if !self.accumulate {
            return Err(SemanticError::protocol(
                "streaming machine does not retain a unary response",
            ));
        }
        self.transport_close_checked()?;
        let reason = validation::stop(&self.terminal.as_ref().unwrap().1)?;
        Ok(
            json!({"id":self.id,"type":"message","role":"assistant","model":self.model,
            "content":self.content,"stop_reason":reason,"stop_sequence":null,"usage":self.usage}),
        )
    }
}
