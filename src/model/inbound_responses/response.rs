use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Map, Value};
use uuid::Uuid;

use super::collaboration::Authority;

const MAX_TOOL_ARGUMENT_BYTES: usize = 4 * 1024 * 1024;
const MAX_TRANSLATED_RESPONSE_BYTES: usize = 16 * 1024 * 1024;
const MAX_OUTPUT_ITEMS: usize = 4_096;
const ESTIMATED_ITEM_OVERHEAD_BYTES: usize = 512;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Usage {
    input: u64,
    output: u64,
    cache_read: u64,
    cache_write: u64,
}

impl Usage {
    fn update(&mut self, value: Option<&Value>) {
        let Some(value) = value.and_then(Value::as_object) else {
            return;
        };
        self.input = number(value, "input_tokens").unwrap_or(self.input);
        self.output = number(value, "output_tokens").unwrap_or(self.output);
        self.cache_read = number(value, "cache_read_input_tokens").unwrap_or(self.cache_read);
        self.cache_write = number(value, "cache_creation_input_tokens").unwrap_or(self.cache_write);
    }

    fn responses(&self) -> Value {
        let input = self
            .input
            .saturating_add(self.cache_read)
            .saturating_add(self.cache_write);
        json!({
            "input_tokens": input,
            "input_tokens_details": {"cached_tokens": self.cache_read},
            "output_tokens": self.output,
            "output_tokens_details": {"reasoning_tokens": 0},
            "total_tokens": input.saturating_add(self.output)
        })
    }
}

fn number(object: &Map<String, Value>, field: &str) -> Option<u64> {
    object.get(field).and_then(Value::as_u64)
}

#[derive(Debug)]
enum OpenBlock {
    Text {
        id: String,
        output_index: usize,
        text: String,
    },
    Reasoning {
        id: String,
        output_index: usize,
        text: String,
    },
    Tool {
        id: String,
        call_id: String,
        name: String,
        collaboration: bool,
        output_index: usize,
        arguments: String,
    },
}

#[derive(Debug)]
pub(crate) struct StreamTranslator {
    response_id: String,
    model: String,
    created_at: u64,
    sequence: u64,
    output: Vec<Value>,
    open: Option<OpenBlock>,
    usage: Usage,
    stop_reason: Option<String>,
    terminal: bool,
    saw_message_start: bool,
    retained_bytes: usize,
    failure: Option<String>,
    collaboration: Authority,
}

impl StreamTranslator {
    pub(crate) fn new(model: impl Into<String>) -> Self {
        Self {
            response_id: format!("resp_{}", Uuid::new_v4().simple()),
            model: model.into(),
            created_at: now_seconds(),
            sequence: 0,
            output: Vec::new(),
            open: None,
            usage: Usage::default(),
            stop_reason: None,
            terminal: false,
            saw_message_start: false,
            retained_bytes: 0,
            failure: None,
            collaboration: Authority::default(),
        }
    }

    pub(crate) fn with_collaboration(model: impl Into<String>, collaboration: Authority) -> Self {
        Self {
            collaboration,
            ..Self::new(model)
        }
    }

    pub(crate) fn start(&mut self) -> Vec<String> {
        let created = self.emit(
            "response.created",
            json!({"response": self.snapshot("in_progress")}),
        );
        let progress = self.emit(
            "response.in_progress",
            json!({"response": self.snapshot("in_progress")}),
        );
        vec![created, progress]
    }

    pub(crate) fn apply(&mut self, event: &str, data: Value) -> Vec<String> {
        if self.terminal {
            return Vec::new();
        }
        let mut out = Vec::new();
        match event {
            "message_start" => {
                if self.saw_message_start {
                    return self.fail("Anthropic stream contained duplicate message_start");
                }
                self.saw_message_start = true;
                let message = data.get("message").and_then(Value::as_object);
                self.usage
                    .update(message.and_then(|value| value.get("usage")));
            }
            "content_block_start" => {
                if self.open.is_some() {
                    return self.fail("Anthropic stream started a content block before closing the previous block");
                }
                let Some(block) = data.get("content_block").and_then(Value::as_object) else {
                    return self.fail("Anthropic content_block_start is missing content_block");
                };
                let Some(kind) = block.get("type").and_then(Value::as_str) else {
                    return self.fail("Anthropic content block is missing type");
                };
                let output_index = self.output.len();
                if output_index >= MAX_OUTPUT_ITEMS {
                    return self.fail("Anthropic response exceeded the output item limit");
                }
                let metadata_bytes = ESTIMATED_ITEM_OVERHEAD_BYTES.saturating_add(
                    block
                        .get("id")
                        .and_then(Value::as_str)
                        .map_or(0, str::len)
                        .saturating_add(
                            block
                                .get("name")
                                .and_then(Value::as_str)
                                .map_or(0, str::len),
                        ),
                );
                if self.retained_bytes.saturating_add(metadata_bytes)
                    > MAX_TRANSLATED_RESPONSE_BYTES
                {
                    return self.fail("Anthropic response exceeded the translation limit");
                }
                self.retained_bytes += metadata_bytes;
                match kind {
                    "text" => {
                        let id = item_id("msg");
                        out.push(self.emit("response.output_item.added", json!({
                            "output_index":output_index,
                            "item":{"type":"message","id":id,"status":"in_progress","role":"assistant","content":[]}
                        })));
                        out.push(self.emit("response.content_part.added", json!({
                            "item_id":id,"output_index":output_index,"content_index":0,
                            "part":{"type":"output_text","text":"","annotations":[]}
                        })));
                        self.open = Some(OpenBlock::Text { id, output_index, text: String::new() });
                    }
                    "thinking" | "reasoning" => {
                        let id = item_id("rs");
                        out.push(self.emit("response.output_item.added", json!({
                            "output_index":output_index,"item":{"type":"reasoning","id":id,"summary":[]}
                        })));
                        out.push(self.emit("response.reasoning_summary_part.added", json!({
                            "item_id":id,"output_index":output_index,"summary_index":0,
                            "part":{"type":"summary_text","text":""}
                        })));
                        self.open = Some(OpenBlock::Reasoning { id, output_index, text: String::new() });
                    }
                    "tool_use" => {
                        let Some(call_id) = block.get("id").and_then(Value::as_str).filter(|v| !v.is_empty()) else {
                            return self.fail("Anthropic tool_use is missing a usable id");
                        };
                        let Some(wire_name) = block.get("name").and_then(Value::as_str).filter(|v| !v.is_empty()) else {
                            return self.fail("Anthropic tool_use is missing a usable name");
                        };
                        let logical_name = self
                            .collaboration
                            .logical_name(wire_name)
                            .map(ToOwned::to_owned);
                        let (name, collaboration) = logical_name
                            .as_deref()
                            .map(|name| (name, true))
                            .unwrap_or((wire_name, false));
                        let id = item_id("fc");
                        let item = function_call_item(
                            &id,
                            call_id,
                            name,
                            "",
                            "in_progress",
                            collaboration,
                        );
                        out.push(self.emit(
                            "response.output_item.added",
                            json!({"output_index":output_index,"item":item}),
                        ));
                        self.open = Some(OpenBlock::Tool {
                            id,
                            call_id: call_id.to_string(),
                            name: name.to_string(),
                            collaboration,
                            output_index,
                            arguments: String::new(),
                        });
                    }
                    "redacted_thinking" => return self.fail("Anthropic redacted thinking cannot be represented as OpenAI encrypted state"),
                    other => return self.fail(&format!("unsupported Anthropic content block `{other}`")),
                }
            }
            "content_block_delta" => {
                let Some(delta) = data.get("delta").and_then(Value::as_object) else {
                    return self.fail("Anthropic content_block_delta is missing delta");
                };
                let kind = delta.get("type").and_then(Value::as_str).unwrap_or("");
                match (&mut self.open, kind) {
                    (
                        Some(OpenBlock::Text {
                            id,
                            output_index,
                            text,
                        }),
                        "text_delta",
                    ) => {
                        let Some(fragment) = delta.get("text").and_then(Value::as_str) else {
                            return self.fail("Anthropic text_delta is missing text");
                        };
                        if self.retained_bytes.saturating_add(fragment.len())
                            > MAX_TRANSLATED_RESPONSE_BYTES
                        {
                            return self.fail("Anthropic response exceeded the translation limit");
                        }
                        self.retained_bytes += fragment.len();
                        text.push_str(fragment);
                        let (id, output_index) = (id.clone(), *output_index);
                        out.push(self.emit("response.output_text.delta", json!({
                            "item_id":id,"output_index":output_index,"content_index":0,"delta":fragment
                        })));
                    }
                    (
                        Some(OpenBlock::Reasoning {
                            id,
                            output_index,
                            text,
                        }),
                        "thinking_delta" | "reasoning_delta",
                    ) => {
                        let fragment = delta
                            .get("thinking")
                            .or_else(|| delta.get("reasoning"))
                            .and_then(Value::as_str);
                        let Some(fragment) = fragment else {
                            return self.fail("Anthropic reasoning delta is missing text");
                        };
                        if self.retained_bytes.saturating_add(fragment.len())
                            > MAX_TRANSLATED_RESPONSE_BYTES
                        {
                            return self.fail("Anthropic response exceeded the translation limit");
                        }
                        self.retained_bytes += fragment.len();
                        text.push_str(fragment);
                        let (id, output_index) = (id.clone(), *output_index);
                        out.push(self.emit("response.reasoning_summary_text.delta", json!({
                            "item_id":id,"output_index":output_index,"summary_index":0,"delta":fragment
                        })));
                    }
                    (Some(OpenBlock::Reasoning { .. }), "signature_delta") => {}
                    (
                        Some(OpenBlock::Tool {
                            id,
                            output_index,
                            arguments,
                            ..
                        }),
                        "input_json_delta",
                    ) => {
                        let Some(fragment) = delta.get("partial_json").and_then(Value::as_str)
                        else {
                            return self.fail("Anthropic input_json_delta is missing partial_json");
                        };
                        if arguments.len().saturating_add(fragment.len()) > MAX_TOOL_ARGUMENT_BYTES
                        {
                            return self
                                .fail("Anthropic tool arguments exceeded the translation limit");
                        }
                        if self.retained_bytes.saturating_add(fragment.len())
                            > MAX_TRANSLATED_RESPONSE_BYTES
                        {
                            return self.fail("Anthropic response exceeded the translation limit");
                        }
                        self.retained_bytes += fragment.len();
                        arguments.push_str(fragment);
                        let (id, output_index) = (id.clone(), *output_index);
                        out.push(self.emit(
                            "response.function_call_arguments.delta",
                            json!({
                                "item_id":id,"output_index":output_index,"delta":fragment
                            }),
                        ));
                    }
                    _ => return self.fail("Anthropic content delta did not match its open block"),
                }
            }
            "content_block_stop" => out.extend(self.close_block()),
            "message_delta" => {
                self.usage.update(data.get("usage"));
                self.stop_reason = data
                    .get("delta")
                    .and_then(Value::as_object)
                    .and_then(|value| value.get("stop_reason"))
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
            }
            "message_stop" => {
                if self.open.is_some() {
                    return self.fail("Anthropic message_stop arrived with an open content block");
                }
                if !self.saw_message_start {
                    return self.fail("Anthropic message_stop arrived before message_start");
                }
                if self.stop_reason.is_none() {
                    return self.fail("Anthropic message_stop arrived without a stop_reason");
                }
                out.extend(self.finish_terminal());
            }
            "error" => {
                let message = data
                    .get("error")
                    .and_then(Value::as_object)
                    .and_then(|value| value.get("message"))
                    .and_then(Value::as_str)
                    .unwrap_or("Anthropic stream error");
                out.extend(self.fail(message));
            }
            "ping" => {}
            other => return self.fail(&format!("unsupported Anthropic SSE event `{other}`")),
        }
        out
    }

    pub(crate) fn eof(&mut self) -> Vec<String> {
        if self.terminal {
            Vec::new()
        } else {
            self.fail("Anthropic stream ended before message_stop")
        }
    }

    pub(crate) fn is_terminal(&self) -> bool {
        self.terminal
    }

    fn apply_checked(&mut self, event: &str, data: Value) -> Result<(), String> {
        self.apply(event, data);
        match &self.failure {
            Some(message) => Err(message.clone()),
            None => Ok(()),
        }
    }

    fn close_block(&mut self) -> Vec<String> {
        let Some(block) = self.open.take() else {
            return self.fail("Anthropic content_block_stop arrived without an open block");
        };
        match block {
            OpenBlock::Text {
                id,
                output_index,
                text,
            } => {
                let item = json!({"type":"message","id":id,"status":"completed","role":"assistant","content":[{"type":"output_text","text":text,"annotations":[]}]});
                let events = vec![
                    self.emit("response.output_text.done", json!({"item_id":id,"output_index":output_index,"content_index":0,"text":text})),
                    self.emit("response.content_part.done", json!({"item_id":id,"output_index":output_index,"content_index":0,"part":{"type":"output_text","text":text,"annotations":[]}})),
                    self.emit("response.output_item.done", json!({"output_index":output_index,"item":item})),
                ];
                self.output.push(item);
                events
            }
            OpenBlock::Reasoning {
                id,
                output_index,
                text,
            } => {
                let item = json!({"type":"reasoning","id":id,"summary":[{"type":"summary_text","text":text}]});
                let events = vec![
                    self.emit("response.reasoning_summary_text.done", json!({"item_id":id,"output_index":output_index,"summary_index":0,"text":text})),
                    self.emit("response.reasoning_summary_part.done", json!({"item_id":id,"output_index":output_index,"summary_index":0,"part":{"type":"summary_text","text":text}})),
                    self.emit("response.output_item.done", json!({"output_index":output_index,"item":item})),
                ];
                self.output.push(item);
                events
            }
            OpenBlock::Tool {
                id,
                call_id,
                name,
                collaboration,
                output_index,
                arguments,
            } => {
                let args = if arguments.is_empty() {
                    "{}".to_string()
                } else {
                    arguments
                };
                let valid =
                    serde_json::from_str::<Value>(&args).is_ok_and(|value| value.is_object());
                if !valid {
                    return self.fail("Anthropic tool_use arguments were not a JSON object");
                }
                let item =
                    function_call_item(&id, &call_id, &name, &args, "completed", collaboration);
                let mut arguments_done = json!({
                    "item_id":id,"output_index":output_index,"arguments":args
                });
                if collaboration {
                    arguments_done["encrypted_function_args"] = json!([]);
                }
                let events = vec![
                    self.emit("response.function_call_arguments.done", arguments_done),
                    self.emit(
                        "response.output_item.done",
                        json!({"output_index":output_index,"item":item}),
                    ),
                ];
                self.output.push(item);
                events
            }
        }
    }

    fn finish_terminal(&mut self) -> Vec<String> {
        let reason = self.stop_reason.as_deref();
        let (kind, status) = match reason {
            Some("end_turn" | "stop_sequence" | "tool_use") => ("response.completed", "completed"),
            Some("max_tokens") => ("response.incomplete", "incomplete"),
            Some(other) => {
                return self.fail(&format!("unsupported Anthropic stop_reason `{other}`"))
            }
            None => return self.fail("Anthropic response is missing stop_reason"),
        };
        self.terminal = true;
        let mut snapshot = self.snapshot(status);
        if status == "incomplete" {
            snapshot["incomplete_details"] = json!({"reason":"max_output_tokens"});
        }
        vec![self.emit(kind, json!({"response":snapshot}))]
    }

    fn fail(&mut self, message: &str) -> Vec<String> {
        if self.terminal {
            return Vec::new();
        }
        self.terminal = true;
        self.open = None;
        self.failure = Some(message.to_string());
        let error = json!({"type":"upstream_error","code":"translation_error","message":message});
        let mut snapshot = self.snapshot("failed");
        snapshot["error"] = error.clone();
        snapshot["last_error"] = error;
        vec![self.emit("response.failed", json!({"response":snapshot}))]
    }

    fn snapshot(&self, status: &str) -> Value {
        json!({
            "id":self.response_id,"object":"response","created_at":self.created_at,
            "status":status,"model":self.model,"output":self.output,"usage":self.usage.responses(),
            "error":Value::Null,"incomplete_details":Value::Null
        })
    }

    fn emit(&mut self, kind: &str, data: Value) -> String {
        let mut value = data.as_object().cloned().unwrap_or_default();
        value.insert("type".into(), Value::String(kind.to_string()));
        value.insert("sequence_number".into(), self.sequence.into());
        self.sequence += 1;
        format!("event: {kind}\ndata: {}\n\n", Value::Object(value))
    }
}

#[cfg(test)]
pub(crate) fn translate_json(bytes: &[u8], requested_model: &str) -> Result<Vec<u8>, String> {
    translate_json_with_collaboration(bytes, requested_model, Authority::default())
}

pub(crate) fn translate_json_with_collaboration(
    bytes: &[u8],
    requested_model: &str,
    collaboration: Authority,
) -> Result<Vec<u8>, String> {
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|_| "Anthropic response was not valid JSON".to_string())?;
    let object = value
        .as_object()
        .ok_or_else(|| "Anthropic response must be a JSON object".to_string())?;
    let content = object
        .get("content")
        .and_then(Value::as_array)
        .ok_or_else(|| "Anthropic response `content` must be an array".to_string())?;
    let mut translator = StreamTranslator::with_collaboration(requested_model, collaboration);
    translator.saw_message_start = true;
    translator.usage.update(object.get("usage"));
    for block in content {
        let block = block
            .as_object()
            .ok_or_else(|| "Anthropic content block must be an object".to_string())?;
        let kind = block
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| "Anthropic content block is missing type".to_string())?;
        let start = json!({"content_block":block});
        translator.apply_checked("content_block_start", start)?;
        let delta = match kind {
            "text" => {
                json!({"delta":{"type":"text_delta","text":block.get("text").and_then(Value::as_str).unwrap_or("")}})
            }
            "thinking" => {
                json!({"delta":{"type":"thinking_delta","thinking":block.get("thinking").and_then(Value::as_str).unwrap_or("")}})
            }
            "reasoning" => {
                json!({"delta":{"type":"reasoning_delta","reasoning":block.get("reasoning").and_then(Value::as_str).unwrap_or("")}})
            }
            "tool_use" => {
                let input = block.get("input").cloned().unwrap_or_else(|| json!({}));
                if !input.is_object() {
                    return Err("Anthropic tool_use input must be an object".into());
                }
                json!({"delta":{"type":"input_json_delta","partial_json":serde_json::to_string(&input).unwrap()}})
            }
            other => return Err(format!("unsupported Anthropic content block `{other}`")),
        };
        translator.apply_checked("content_block_delta", delta)?;
        translator.apply_checked("content_block_stop", json!({}))?;
    }
    translator.stop_reason = object
        .get("stop_reason")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    if translator.stop_reason.is_none() {
        return Err("Anthropic response is missing stop_reason".into());
    }
    translator.finish_terminal();
    if let Some(message) = translator.failure.clone() {
        return Err(message);
    }
    let status = match translator.stop_reason.as_deref() {
        Some("max_tokens") => "incomplete",
        Some("end_turn" | "stop_sequence" | "tool_use") => "completed",
        _ => unreachable!("finish_terminal accepts only mapped stop reasons"),
    };
    let mut response = translator.snapshot(status);
    if status == "incomplete" {
        response["incomplete_details"] = json!({"reason":"max_output_tokens"});
    }
    serde_json::to_vec(&response).map_err(|error| error.to_string())
}

fn function_call_item(
    id: &str,
    call_id: &str,
    name: &str,
    arguments: &str,
    status: &str,
    collaboration: bool,
) -> Value {
    let mut item = json!({
        "type":"function_call","id":id,"call_id":call_id,"name":name,
        "arguments":arguments,"status":status
    });
    if collaboration {
        item["namespace"] = Value::String("collaboration".into());
        item["encrypted_function_args"] = json!([]);
    }
    item
}

fn item_id(prefix: &str) -> String {
    format!("{prefix}_{}", Uuid::new_v4().simple())
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_maps_text_tool_reasoning_usage_and_incomplete() {
        let body = json!({
            "model":"claude-upstream",
            "content":[
                {"type":"thinking","thinking":"consider"},
                {"type":"text","text":"hello"},
                {"type":"tool_use","id":"call_1","name":"lookup","input":{"q":1}}
            ],
            "stop_reason":"max_tokens",
            "usage":{"input_tokens":10,"cache_read_input_tokens":2,"cache_creation_input_tokens":3,"output_tokens":4}
        });
        let translated: Value =
            serde_json::from_slice(&translate_json(body.to_string().as_bytes(), "alias").unwrap())
                .unwrap();
        assert_eq!(translated["status"], "incomplete");
        assert_eq!(translated["output"][0]["type"], "reasoning");
        assert_eq!(translated["output"][1]["content"][0]["text"], "hello");
        assert_eq!(translated["output"][2]["call_id"], "call_1");
        assert_eq!(translated["usage"]["input_tokens"], 15);
        assert_eq!(translated["usage"]["total_tokens"], 19);
    }

    #[test]
    fn stream_orders_events_and_fails_premature_eof() {
        let mut translator = StreamTranslator::new("claude");
        let mut events = translator.start();
        events.extend(translator.apply(
            "message_start",
            json!({"message":{"model":"claude-upstream","usage":{"input_tokens":2}}}),
        ));
        events.extend(translator.apply(
            "content_block_start",
            json!({"content_block":{"type":"text"}}),
        ));
        events.extend(translator.apply(
            "content_block_delta",
            json!({"delta":{"type":"text_delta","text":"hi"}}),
        ));
        events.extend(translator.apply("content_block_stop", json!({})));
        events.extend(translator.apply(
            "message_delta",
            json!({"delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":1}}),
        ));
        events.extend(translator.apply("message_stop", json!({})));
        let joined = events.join("");
        assert!(
            joined.find("response.created").unwrap()
                < joined.find("response.output_text.delta").unwrap()
        );
        assert!(joined.contains("response.output_text.done"));
        assert!(joined.contains("response.completed"));
        assert!(!joined.contains("claude-upstream"));

        let mut truncated = StreamTranslator::new("claude");
        truncated.start();
        assert!(truncated.eof().join("").contains("response.failed"));
    }

    #[test]
    fn unknown_stop_reasons_fail_closed() {
        let body = json!({
            "model":"claude-upstream",
            "content":[{"type":"text","text":"partial"}],
            "stop_reason":"future_reason",
            "usage":{"input_tokens":1,"output_tokens":1}
        });
        let error = translate_json(body.to_string().as_bytes(), "alias").unwrap_err();
        assert!(error.contains("unsupported Anthropic stop_reason"));

        let mut stream = StreamTranslator::new("claude");
        stream.start();
        stream.apply("message_start", json!({"message":{}}));
        stream.apply(
            "message_delta",
            json!({"delta":{"stop_reason":"future_reason"}}),
        );
        let terminal = stream.apply("message_stop", json!({})).join("");
        assert!(terminal.contains("response.failed"));
        assert!(!terminal.contains("response.completed"));
    }

    #[test]
    fn restores_only_request_authorized_collaboration_calls() {
        let mut authority = Authority::default();
        authority.authorize("spawn_agent").unwrap();
        let body = json!({
            "content":[
                {"type":"tool_use","id":"call_1","name":"shunt_collaboration__spawn_agent","input":{"message":"work"}},
                {"type":"tool_use","id":"call_2","name":"shunt_collaboration__forged","input":{}}
            ],
            "stop_reason":"tool_use",
            "usage":{"input_tokens":1,"output_tokens":1}
        });
        let translated: Value = serde_json::from_slice(
            &translate_json_with_collaboration(body.to_string().as_bytes(), "alias", authority)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(translated["output"][0]["name"], "spawn_agent");
        assert_eq!(translated["output"][0]["namespace"], "collaboration");
        assert_eq!(
            translated["output"][0]["encrypted_function_args"],
            json!([])
        );
        assert_eq!(
            translated["output"][1]["name"],
            "shunt_collaboration__forged"
        );
        assert!(translated["output"][1].get("namespace").is_none());
    }

    #[test]
    fn stream_restores_authorized_collaboration_call_lifecycle() {
        let mut authority = Authority::default();
        authority.authorize("spawn_agent").unwrap();
        let mut translator = StreamTranslator::with_collaboration("alias", authority);
        translator.start();
        translator.apply("message_start", json!({"message":{}}));
        let mut events = translator.apply(
            "content_block_start",
            json!({"content_block":{
                "type":"tool_use","id":"call_1",
                "name":"shunt_collaboration__spawn_agent"
            }}),
        );
        events.extend(translator.apply(
            "content_block_delta",
            json!({"delta":{"type":"input_json_delta","partial_json":"{}"}}),
        ));
        events.extend(translator.apply("content_block_stop", json!({})));
        let joined = events.join("");
        assert!(joined.contains("\"namespace\":\"collaboration\""));
        assert!(joined.contains("\"name\":\"spawn_agent\""));
        assert!(joined.contains("\"encrypted_function_args\":[]"));
        assert!(!joined.contains("shunt_collaboration__spawn_agent"));
    }
}
