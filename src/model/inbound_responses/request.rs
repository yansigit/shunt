use std::collections::HashSet;

use serde_json::{json, Map, Value};

use crate::request::RequestBody;

use super::collaboration::{self, Authority};

const DEFAULT_MAX_TOKENS: u64 = 8_192;
const OUTPUT_HEADROOM: u64 = 4_096;

#[derive(Debug)]
pub(crate) struct TranslatedRequest {
    pub(crate) body: RequestBody,
    pub(crate) stream: bool,
    pub(crate) collaboration: Authority,
}

pub(crate) fn translate(
    raw: Vec<u8>,
    upstream_model: &str,
    collaboration_enabled: bool,
) -> Result<TranslatedRequest, String> {
    let parsed =
        RequestBody::parse(raw).map_err(|error| format!("invalid Responses JSON: {error}"))?;
    let request = parsed
        .json()
        .as_object()
        .ok_or_else(|| "Responses request must be a JSON object".to_string())?;
    reject_unsupported_top_level(request)?;
    if collaboration_enabled
        && collaboration::has_unreadable_encrypted_agent_task(request.get("input"))
    {
        return Err("encrypted agent task cannot be translated without plaintext recovery".into());
    }

    let (tools, collaboration) = translate_tools(request, collaboration_enabled)?;

    let stream = bool_field(request, "stream")?.unwrap_or(false);
    let mut messages = Vec::<Value>::new();
    if let Some(instructions) = request.get("instructions") {
        let text = instructions
            .as_str()
            .ok_or_else(|| "`instructions` must be a string".to_string())?;
        if text.is_empty() {
            return Err("`instructions` must not be empty".to_string());
        }
    }
    translate_input(
        request.get("input"),
        &mut messages,
        collaboration_enabled,
        &collaboration,
    )?;
    if messages.is_empty() {
        return Err("`input` must contain at least one message or item".to_string());
    }

    let mut out = Map::new();
    out.insert("model".into(), Value::String(upstream_model.to_string()));
    out.insert("messages".into(), Value::Array(messages));
    out.insert("stream".into(), Value::Bool(stream));

    if let Some(instructions) = request.get("instructions").and_then(Value::as_str) {
        out.insert(
            "system".into(),
            json!([{ "type": "text", "text": instructions }]),
        );
    }
    let requested_max = u64_field(request, "max_output_tokens")?.unwrap_or(DEFAULT_MAX_TOKENS);
    if requested_max == 0 {
        return Err("`max_output_tokens` must be greater than zero".to_string());
    }
    out.insert("max_tokens".into(), requested_max.into());
    copy_number(request, &mut out, "temperature", "temperature")?;
    copy_number(request, &mut out, "top_p", "top_p")?;

    if !tools.is_empty() {
        out.insert("tools".into(), Value::Array(tools));
    }
    translate_tool_choice(request, &mut out, &collaboration)?;
    translate_reasoning(request.get("reasoning"), &mut out, requested_max)?;
    if collaboration_enabled {
        translate_collaboration_metadata(request.get("metadata"), &mut out)?;
    }

    let bytes = serde_json::to_vec(&Value::Object(out))
        .expect("translated serde_json::Value always serializes");
    let body = RequestBody::parse(bytes).expect("translated request has unique object keys");
    Ok(TranslatedRequest {
        body,
        stream,
        collaboration,
    })
}

fn translate_collaboration_metadata(
    metadata: Option<&Value>,
    out: &mut Map<String, Value>,
) -> Result<(), String> {
    let Some(metadata) = metadata.filter(|value| !value.is_null()) else {
        return Ok(());
    };
    let metadata = metadata
        .as_object()
        .ok_or_else(|| "`metadata` must be an object or null".to_string())?;
    let Some(task_id) = metadata.get("task_id") else {
        return Ok(());
    };
    let task_id = task_id
        .as_str()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "`metadata.task_id` must be a non-empty string".to_string())?;
    out.insert("metadata".into(), json!({"user_id":task_id}));
    Ok(())
}

fn reject_unsupported_top_level(request: &Map<String, Value>) -> Result<(), String> {
    const ALLOWED: &[&str] = &[
        "type",
        "generate",
        "model",
        "input",
        "instructions",
        "stream",
        "tools",
        "tool_choice",
        "parallel_tool_calls",
        "max_output_tokens",
        "temperature",
        "top_p",
        "reasoning",
        "store",
        "metadata",
        "include",
        "prompt_cache_key",
        "previous_response_id",
    ];
    for key in request.keys() {
        if !ALLOWED.contains(&key.as_str()) {
            return Err(format!(
                "unsupported Responses field `{key}` for Anthropic translation"
            ));
        }
    }
    if request
        .get("type")
        .is_some_and(|value| value.as_str() != Some("response.create"))
    {
        return Err("`type` must be `response.create` when present".into());
    }
    if request
        .get("generate")
        .is_some_and(|value| value.as_bool() != Some(true))
    {
        return Err("translated turns require `generate: true`".into());
    }
    if request
        .get("previous_response_id")
        .is_some_and(|v| !v.is_null())
    {
        return Err(
            "`previous_response_id` cannot be translated without continuation state".into(),
        );
    }
    if request
        .get("store")
        .is_some_and(|v| v != &Value::Bool(false))
    {
        return Err("Anthropic translation supports only `store: false`".into());
    }
    if request
        .get("include")
        .is_some_and(|value| !matches!(value, Value::Array(items) if items.is_empty()))
    {
        return Err("non-empty `include` cannot be satisfied by Anthropic translation".into());
    }
    if request
        .get("metadata")
        .is_some_and(|value| !value.is_object() && !value.is_null())
    {
        return Err("`metadata` must be an object or null".into());
    }
    if request
        .get("prompt_cache_key")
        .is_some_and(|value| !value.is_string())
    {
        return Err("`prompt_cache_key` must be a string".into());
    }
    Ok(())
}

fn translate_input(
    input: Option<&Value>,
    messages: &mut Vec<Value>,
    collaboration_enabled: bool,
    collaboration: &Authority,
) -> Result<(), String> {
    let Some(input) = input else {
        return Err("Responses request is missing `input`".into());
    };
    if let Some(text) = input.as_str() {
        push_message(messages, "user", vec![json!({"type":"text", "text": text})]);
        return Ok(());
    }
    let items = input
        .as_array()
        .ok_or_else(|| "`input` must be a string or array".to_string())?;
    let mut calls = HashSet::new();
    let mut satisfied = HashSet::new();
    for (index, item) in items.iter().enumerate() {
        let item = item
            .as_object()
            .ok_or_else(|| format!("`input[{index}]` must be an object"))?;
        let kind = required_string(item, "type", &format!("input[{index}]"))?;
        match kind {
            "additional_tools" if collaboration_enabled => continue,
            "agent_message" if collaboration_enabled => {
                let content = translate_content(
                    item.get("content"),
                    true,
                    &format!("input[{index}].content"),
                )?;
                push_message(messages, "user", content);
            }
            "message" => {
                let role = required_string(item, "role", &format!("input[{index}]"))?;
                if !matches!(role, "user" | "assistant") {
                    return Err(format!("unsupported `input[{index}].role` `{role}`"));
                }
                let content = translate_content(
                    item.get("content"),
                    role == "user",
                    &format!("input[{index}].content"),
                )?;
                push_message(messages, role, content);
            }
            "function_call" => {
                let call_id = required_string(item, "call_id", &format!("input[{index}]"))?;
                if !calls.insert(call_id.to_string()) {
                    return Err(format!("duplicate function call id `{call_id}`"));
                }
                let logical_name = required_string(item, "name", &format!("input[{index}]"))?;
                let name = match item.get("namespace") {
                    Some(Value::String(namespace)) if collaboration_enabled => collaboration
                        .wire_name(namespace, logical_name)
                        .ok_or_else(|| {
                            format!("input[{index}] references an unauthorized collaboration tool")
                        })?,
                    Some(_) => {
                        return Err(format!(
                            "unsupported `input[{index}].namespace` for Anthropic translation"
                        ))
                    }
                    None => logical_name,
                };
                let arguments = required_string(item, "arguments", &format!("input[{index}]"))?;
                let arguments: Value = serde_json::from_str(arguments).map_err(|_| {
                    format!("`input[{index}].arguments` must contain a JSON object")
                })?;
                if !arguments.is_object() {
                    return Err(format!(
                        "`input[{index}].arguments` must contain a JSON object"
                    ));
                }
                push_message(
                    messages,
                    "assistant",
                    vec![json!({"type":"tool_use", "id":call_id, "name":name, "input":arguments})],
                );
            }
            "function_call_output" => {
                let call_id = required_string(item, "call_id", &format!("input[{index}]"))?;
                if !calls.contains(call_id) {
                    return Err(format!("orphan function result for call id `{call_id}`"));
                }
                if !satisfied.insert(call_id.to_string()) {
                    return Err(format!("duplicate function result for call id `{call_id}`"));
                }
                let content =
                    translate_tool_output(item.get("output"), &format!("input[{index}].output"))?;
                push_message(
                    messages,
                    "user",
                    vec![json!({"type":"tool_result", "tool_use_id":call_id, "content":content})],
                );
            }
            "reasoning" | "context_compaction" => {
                return Err(format!(
                    "`input[{index}]` type `{kind}` contains provider-owned state that cannot be translated"
                ));
            }
            other => return Err(format!("unsupported `input[{index}]` type `{other}`")),
        }
    }
    if let Some(call_id) = calls.difference(&satisfied).next() {
        return Err(format!("function call `{call_id}` has no matching result"));
    }
    Ok(())
}

fn translate_content(value: Option<&Value>, user: bool, path: &str) -> Result<Vec<Value>, String> {
    if let Some(text) = value.and_then(Value::as_str) {
        return Ok(vec![json!({"type":"text", "text":text})]);
    }
    let blocks = value
        .and_then(Value::as_array)
        .ok_or_else(|| format!("`{path}` must be a string or array"))?;
    let mut out = Vec::with_capacity(blocks.len());
    for (index, block) in blocks.iter().enumerate() {
        let block = block
            .as_object()
            .ok_or_else(|| format!("`{path}[{index}]` must be an object"))?;
        let kind = required_string(block, "type", &format!("{path}[{index}]"))?;
        match kind {
            "input_text" | "output_text" | "text" => {
                let text = required_string(block, "text", &format!("{path}[{index}]"))?;
                out.push(json!({"type":"text", "text":text}));
            }
            "input_image" if user => out.push(translate_image(block, &format!("{path}[{index}]"))?),
            "input_image" => {
                return Err(format!("`{path}[{index}]` images require a user message"))
            }
            "refusal" => {
                return Err(format!(
                    "`{path}[{index}]` refusal content cannot be translated faithfully"
                ))
            }
            other => return Err(format!("unsupported `{path}[{index}]` type `{other}`")),
        }
    }
    if out.is_empty() {
        return Err(format!("`{path}` must not be empty"));
    }
    Ok(out)
}

fn translate_image(block: &Map<String, Value>, path: &str) -> Result<Value, String> {
    if block.get("file_id").is_some() {
        return Err(format!("`{path}.file_id` cannot be resolved by Shunt"));
    }
    let url = required_string(block, "image_url", path)?;
    if let Some(data) = url.strip_prefix("data:") {
        let (media_type, encoded) = data
            .split_once(";base64,")
            .ok_or_else(|| format!("`{path}.image_url` must be a base64 data URL"))?;
        if !media_type.starts_with("image/") || encoded.is_empty() {
            return Err(format!("`{path}.image_url` must contain image data"));
        }
        Ok(
            json!({"type":"image", "source":{"type":"base64", "media_type":media_type, "data":encoded}}),
        )
    } else if url.starts_with("https://") || url.starts_with("http://") {
        Ok(json!({"type":"image", "source":{"type":"url", "url":url}}))
    } else {
        Err(format!(
            "`{path}.image_url` must be an HTTP(S) or image data URL"
        ))
    }
}

fn translate_tool_output(value: Option<&Value>, path: &str) -> Result<Value, String> {
    match value {
        Some(Value::String(text)) => Ok(Value::String(text.clone())),
        Some(Value::Array(_)) => Ok(Value::Array(translate_content(value, true, path)?)),
        _ => Err(format!("`{path}` must be a string or content array")),
    }
}

fn translate_tools(
    request: &Map<String, Value>,
    collaboration_enabled: bool,
) -> Result<(Vec<Value>, Authority), String> {
    let mut lists = Vec::new();
    if let Some(value) = request.get("tools") {
        lists.push(
            value
                .as_array()
                .ok_or_else(|| "`tools` must be an array".to_string())?,
        );
    }
    if collaboration_enabled {
        if let Some(input) = request.get("input").and_then(Value::as_array) {
            for (index, item) in input.iter().enumerate() {
                if item.get("type").and_then(Value::as_str) == Some("additional_tools") {
                    lists.push(
                        item.get("tools")
                            .and_then(Value::as_array)
                            .ok_or_else(|| format!("`input[{index}].tools` must be an array"))?,
                    );
                }
            }
        }
    }
    let mut names = HashSet::new();
    let mut out = Vec::new();
    let mut authority = Authority::default();
    for (list_index, tools) in lists.into_iter().enumerate() {
        for (index, tool) in tools.iter().enumerate() {
            let tool = tool.as_object().ok_or_else(|| {
                format!("tool catalog {list_index} item {index} must be an object")
            })?;
            if tool.get("type").and_then(Value::as_str) == Some("namespace") {
                if !collaboration_enabled
                    || tool.get("name").and_then(Value::as_str) != Some("collaboration")
                {
                    return Err(format!("`tools[{index}]` must be a function tool"));
                }
                let children = tool
                    .get("tools")
                    .and_then(Value::as_array)
                    .ok_or_else(|| format!("`tools[{index}].tools` must be an array"))?;
                for (child_index, child) in children.iter().enumerate() {
                    let child = child.as_object().ok_or_else(|| {
                        format!("`tools[{index}].tools[{child_index}]` must be an object")
                    })?;
                    if child.get("type").and_then(Value::as_str) != Some("function") {
                        return Err(format!(
                            "`tools[{index}].tools[{child_index}]` must be a function tool"
                        ));
                    }
                    let name = required_string(
                        child,
                        "name",
                        &format!("tools[{index}].tools[{child_index}]"),
                    )?;
                    let (wire, newly_authorized) = authority.authorize(name)?;
                    if !newly_authorized {
                        continue;
                    }
                    if !names.insert(wire.clone()) {
                        return Err(format!("duplicate tool name `{wire}`"));
                    }
                    out.push(translate_tool_definition(
                        child,
                        &wire,
                        true,
                        &format!("tools[{index}].tools[{child_index}]"),
                    )?);
                }
                continue;
            }
            if tool.get("type").and_then(Value::as_str) != Some("function") {
                return Err(format!("`tools[{index}]` must be a function tool"));
            }
            let definition = tool
                .get("function")
                .and_then(Value::as_object)
                .unwrap_or(tool);
            let name = required_string(definition, "name", &format!("tools[{index}]"))?;
            if !names.insert(name.to_string()) {
                return Err(format!("duplicate tool name `{name}`"));
            }
            out.push(translate_tool_definition(
                definition,
                name,
                false,
                &format!("tools[{index}]"),
            )?);
        }
    }
    Ok((out, authority))
}

fn translate_tool_definition(
    definition: &Map<String, Value>,
    name: &str,
    sanitize: bool,
    path: &str,
) -> Result<Value, String> {
    let schema = definition
        .get("parameters")
        .cloned()
        .unwrap_or_else(|| json!({"type":"object","properties":{}}));
    if !schema.is_object() {
        return Err(format!("`{path}.parameters` must be an object"));
    }
    let schema = if sanitize {
        collaboration::sanitize_schema(&schema)?
    } else {
        schema
    };
    let mut translated = json!({"name":name, "input_schema":schema});
    if let Some(description) = definition.get("description") {
        translated["description"] = Value::String(
            description
                .as_str()
                .ok_or_else(|| format!("`{path}.description` must be a string"))?
                .to_string(),
        );
    }
    Ok(translated)
}

fn translate_tool_choice(
    request: &Map<String, Value>,
    out: &mut Map<String, Value>,
    collaboration: &Authority,
) -> Result<(), String> {
    let choice = request.get("tool_choice");
    let mut translated = match choice {
        None => json!({"type":"auto"}),
        Some(Value::String(value)) if value == "auto" => json!({"type":"auto"}),
        Some(Value::String(value)) if value == "none" => json!({"type":"none"}),
        Some(Value::String(value)) if value == "required" => json!({"type":"any"}),
        Some(Value::Object(object))
            if object.get("type").and_then(Value::as_str) == Some("function") =>
        {
            let name = required_string(object, "name", "tool_choice")?;
            let name = match object.get("namespace") {
                Some(Value::String(namespace)) => {
                    collaboration.wire_name(namespace, name).ok_or_else(|| {
                        "tool_choice references an unauthorized collaboration tool".to_string()
                    })?
                }
                Some(_) => return Err("`tool_choice.namespace` must be a string".into()),
                None => name,
            };
            json!({"type":"tool", "name":name})
        }
        _ => return Err("unsupported `tool_choice` for Anthropic translation".into()),
    };
    if request.get("parallel_tool_calls") == Some(&Value::Bool(false)) {
        translated["disable_parallel_tool_use"] = Value::Bool(true);
    } else if request
        .get("parallel_tool_calls")
        .is_some_and(|v| !v.is_boolean())
    {
        return Err("`parallel_tool_calls` must be a boolean".into());
    }
    if choice.is_some() || request.contains_key("parallel_tool_calls") {
        out.insert("tool_choice".into(), translated);
    }
    Ok(())
}

fn translate_reasoning(
    value: Option<&Value>,
    out: &mut Map<String, Value>,
    requested_max: u64,
) -> Result<(), String> {
    let Some(value) = value else { return Ok(()) };
    let object = value
        .as_object()
        .ok_or_else(|| "`reasoning` must be an object".to_string())?;
    for key in object.keys() {
        if !matches!(key.as_str(), "effort" | "summary") {
            return Err(format!("unsupported `reasoning.{key}` field"));
        }
    }
    if let Some(summary) = object.get("summary") {
        if !summary.is_null() && !matches!(summary.as_str(), Some("auto" | "concise" | "detailed"))
        {
            return Err("`reasoning.summary` must be auto, concise, detailed, or null".into());
        }
    }
    let Some(effort) = object.get("effort") else {
        return Ok(());
    };
    let effort = effort
        .as_str()
        .ok_or_else(|| "`reasoning.effort` must be a string".to_string())?;
    if effort == "none" {
        out.insert("thinking".into(), json!({"type":"disabled"}));
        return Ok(());
    }
    let budget = match effort {
        "minimal" | "low" => 1_024,
        "medium" => 4_096,
        "high" => 8_192,
        "xhigh" | "max" => 16_384,
        _ => return Err(format!("unsupported `reasoning.effort` `{effort}`")),
    };
    out.insert(
        "thinking".into(),
        json!({"type":"enabled", "budget_tokens":budget}),
    );
    out.insert(
        "max_tokens".into(),
        requested_max.max(budget + OUTPUT_HEADROOM).into(),
    );
    out.remove("temperature");
    out.remove("top_p");
    Ok(())
}

fn push_message(messages: &mut Vec<Value>, role: &str, mut content: Vec<Value>) {
    if let Some(last) = messages.last_mut().and_then(Value::as_object_mut) {
        if last.get("role").and_then(Value::as_str) == Some(role) {
            if let Some(existing) = last.get_mut("content").and_then(Value::as_array_mut) {
                existing.append(&mut content);
                return;
            }
        }
    }
    messages.push(json!({"role":role, "content":content}));
}

fn required_string<'a>(
    object: &'a Map<String, Value>,
    field: &str,
    path: &str,
) -> Result<&'a str, String> {
    object
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("`{path}.{field}` must be a non-empty string"))
}

fn bool_field(object: &Map<String, Value>, field: &str) -> Result<Option<bool>, String> {
    object
        .get(field)
        .map(|value| {
            value
                .as_bool()
                .ok_or_else(|| format!("`{field}` must be a boolean"))
        })
        .transpose()
}

fn u64_field(object: &Map<String, Value>, field: &str) -> Result<Option<u64>, String> {
    object
        .get(field)
        .map(|value| {
            value
                .as_u64()
                .ok_or_else(|| format!("`{field}` must be a positive integer"))
        })
        .transpose()
}

fn copy_number(
    request: &Map<String, Value>,
    out: &mut Map<String, Value>,
    source: &str,
    target: &str,
) -> Result<(), String> {
    if let Some(value) = request.get(source) {
        if !value.is_number() {
            return Err(format!("`{source}` must be a number"));
        }
        out.insert(target.into(), value.clone());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::translate;
    use serde_json::{json, Value};

    fn translated(value: Value) -> Value {
        let result = translate(
            serde_json::to_vec(&value).unwrap(),
            "claude-sonnet-4-6",
            false,
        )
        .unwrap();
        result.body.json().clone()
    }

    #[test]
    fn translates_text_tools_results_images_and_reasoning() {
        let result = translated(json!({
            "model":"claude",
            "instructions":"Be brief",
            "input":[
                {"type":"message","role":"user","content":[
                    {"type":"input_text","text":"look"},
                    {"type":"input_image","image_url":"data:image/png;base64,AA=="}
                ]},
                {"type":"function_call","call_id":"call_1","name":"lookup","arguments":"{\"q\":1}"},
                {"type":"function_call_output","call_id":"call_1","output":"ok"}
            ],
            "tools":[{"type":"function","name":"lookup","description":"find","parameters":{"type":"object"}}],
            "tool_choice":{"type":"function","name":"lookup"},
            "parallel_tool_calls":false,
            "reasoning":{"effort":"high","summary":"auto"},
            "stream":true,
            "store":false
        }));
        assert_eq!(result["model"], "claude-sonnet-4-6");
        assert_eq!(result["system"][0]["text"], "Be brief");
        assert_eq!(
            result["messages"][0]["content"][1]["source"]["type"],
            "base64"
        );
        assert_eq!(result["messages"][1]["content"][0]["id"], "call_1");
        assert_eq!(result["messages"][2]["content"][0]["tool_use_id"], "call_1");
        assert_eq!(result["tools"][0]["input_schema"]["type"], "object");
        assert_eq!(result["tool_choice"]["disable_parallel_tool_use"], true);
        assert_eq!(result["thinking"]["budget_tokens"], 8192);
        assert_eq!(result["max_tokens"], 12288);
    }

    #[test]
    fn rejects_state_and_malformed_tool_arguments() {
        for value in [
            json!({"model":"claude","input":"hi","previous_response_id":"resp_1"}),
            json!({"model":"claude","input":[{"type":"reasoning","encrypted_content":"opaque"}]}),
            json!({"model":"claude","input":[{"type":"function_call","call_id":"c","name":"x","arguments":"no"}]}),
            json!({"model":"claude","input":[{"type":"function_call_output","call_id":"missing","output":"x"}]}),
            json!({"model":"claude","input":[{"type":"function_call","call_id":"c","name":"x","arguments":"{}"}]}),
            json!({"model":"claude","input":"hi","tools":[{"type":"web_search"}]}),
        ] {
            assert!(translate(serde_json::to_vec(&value).unwrap(), "claude", false).is_err());
        }
    }

    #[test]
    fn enabled_collaboration_flattens_catalog_and_preserves_agent_message() {
        let value = json!({
            "model":"claude",
            "input":[
                {"type":"additional_tools","role":"developer","tools":[{
                    "type":"namespace","name":"collaboration","tools":[{
                        "type":"function","name":"spawn_agent","description":"spawn",
                        "parameters":{"type":"object","properties":{"message":{"type":"string","encrypted":true}}}
                    }]
                }]},
                {"type":"agent_message","author":"/root","recipient":"/root/worker","content":[
                    {"type":"input_text","text":"Message Type: NEW_TASK\nTask name: /root/worker\nSender: /root\nPayload:\nImplement it"}
                ]}
            ],
            "stream":false,"store":false
            ,"metadata":{"task_id":"task_42","subagent":"collab_spawn"}
        });
        let result = translate(serde_json::to_vec(&value).unwrap(), "claude", true).unwrap();
        let body = result.body.json();
        assert_eq!(body["messages"][0]["role"], "user");
        assert!(body["messages"][0]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Task name: /root/worker"));
        assert_eq!(body["tools"][0]["name"], "shunt_collaboration__spawn_agent");
        assert!(body["tools"][0]["input_schema"]["properties"]["message"]
            .get("encrypted")
            .is_none());
        assert_eq!(body["metadata"]["user_id"], "task_42");
        assert_eq!(
            result
                .collaboration
                .logical_name("shunt_collaboration__spawn_agent"),
            Some("spawn_agent")
        );

        assert!(translate(serde_json::to_vec(&value).unwrap(), "claude", false).is_err());
    }
}
