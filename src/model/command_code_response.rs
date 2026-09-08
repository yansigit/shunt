//! Checked Command Code NDJSON semantics, extracted from the text tracer.
//! Source-derived protocol: OpenCodex 055c3ecf; see THIRD-PARTY-NOTICES.md.
use serde_json::{json, Value};

pub struct CommandCodeMachine {
    model: String,
    text: String,
    terminal: Option<(String, String)>,
    companion: bool,
    usage: Value,
}

impl CommandCodeMachine {
    pub fn new(model: impl Into<String>) -> Self {
        Self { model: model.into(), text: String::new(), terminal: None,
            companion: false, usage: json!({"input_tokens":0,"output_tokens":0}) }
    }

    pub fn process_record_checked(&mut self, record: &Value) -> Result<(), &'static str> {
        match record.get("type").and_then(Value::as_str) {
            Some("text-delta") if self.terminal.is_none() => {
                let delta = record.get("text").and_then(Value::as_str).ok_or("invalid text delta")?;
                self.text.push_str(delta);
            }
            Some(kind @ ("finish-step" | "finish")) => {
                let reason = record.get("rawFinishReason").or_else(|| record.get("finishReason"))
                    .and_then(Value::as_str).ok_or("missing finish reason")?;
                if reason != "stop" { return Err("unsupported or failed subscription finish"); }
                if let Some((first, prior_reason)) = &self.terminal {
                    if first != "finish-step" || kind != "finish" || self.companion || prior_reason != reason {
                        return Err("conflicting or duplicate subscription terminal");
                    }
                    self.companion = true;
                } else {
                    if let Some(value) = record.get("totalUsage").or_else(|| record.get("usage")) {
                        for (wire, client) in [("inputTokens","input_tokens"),("outputTokens","output_tokens")] {
                            if let Some(n) = value.get(wire) {
                                self.usage[client] = json!(n.as_u64().ok_or("invalid subscription usage")?);
                            }
                        }
                    }
                    self.terminal = Some((kind.into(), reason.into()));
                }
            }
            _ => return Err("invalid or unsupported subscription record"),
        }
        Ok(())
    }

    pub fn transport_close_checked(&self) -> Result<(), &'static str> {
        if self.terminal.is_none() { return Err("subscription EOF without terminal"); }
        Ok(())
    }

    pub fn final_ndjson_checked(&self) -> Result<Value, &'static str> {
        self.transport_close_checked()?;
        Ok(json!({"id":format!("msg_{}",uuid::Uuid::new_v4()),"type":"message","role":"assistant","model":self.model,
            "content":[{"type":"text","text":self.text}],"stop_reason":"end_turn","stop_sequence":null,"usage":self.usage}))
    }
}
