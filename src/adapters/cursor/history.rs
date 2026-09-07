//! Request-owned structured Composer history. Schema evidence: OpenCodex
//! 055c3ecf0de6c35f59195fc434d6b08525182b7f, protobuf-request.ts and generated
//! agent_pb.ts, inspected 2026-09-07. No checkpoint database or durable store.

use super::{
    agent::{encode_protobuf_value, field_ld, field_str, field_varint},
    kv::RequestBlobStore,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

pub(super) struct PreparedHistory {
    pub conversation_id: String,
    pub state: Vec<u8>,
    pub blobs: RequestBlobStore,
    pub prompt: String,
}

fn session_id(req: &Value) -> Option<String> {
    // Explicit session identity only: never guess from prose, summary markers,
    // user identity, or a model-family name. Clients may supply session_id in
    // JSON metadata.user_id or directly in metadata.session_id.
    req.pointer("/metadata/session_id")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| {
            let metadata: Value =
                serde_json::from_str(req.pointer("/metadata/user_id")?.as_str()?).ok()?;
            metadata.get("session_id")?.as_str().map(str::to_owned)
        })
        .filter(|id| !id.is_empty() && id.len() <= 512)
}

fn digest_id(parts: &[&str]) -> String {
    let mut hash = Sha256::new();
    hash.update(b"shunt/cursor/session/v1");
    for part in parts {
        hash.update((part.len() as u64).to_be_bytes());
        hash.update(part.as_bytes());
    }
    let digest = hash.finalize();
    let mut bytes = [0; 16];
    bytes.copy_from_slice(&digest[..16]);
    // UUIDv8 reserves application-defined hash bits; keep the wire UUID shape.
    bytes[6] = (bytes[6] & 0x0f) | 0x80;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    uuid::Uuid::from_bytes(bytes).to_string()
}

fn blocks(message: &Value) -> &[Value] {
    message
        .get("content")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn is_result(message: &Value) -> bool {
    blocks(message)
        .iter()
        .any(|block| block["type"] == "tool_result")
}

fn plain_text(message: &Value) -> Result<String, String> {
    if let Some(text) = message.get("content").and_then(Value::as_str) {
        return Ok(text.to_owned());
    }
    if !message.get("content").is_some_and(Value::is_array) {
        return Err("history content must be text or an array of blocks".into());
    }
    let mut text = Vec::new();
    for block in blocks(message) {
        if block["type"] != "text" {
            return Err("structured history supports text user messages; opaque or image history is unsupported".into());
        }
        text.push(
            block["text"]
                .as_str()
                .ok_or("history text must be a string")?,
        );
    }
    Ok(text.join("\n"))
}

fn put(store: &mut RequestBlobStore, bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    store
        .store_content(&bytes)
        .map(|id| id.to_vec())
        .map_err(|e| e.to_string())
}

fn tool_step(call: &Value, result: &Value) -> Result<Vec<u8>, String> {
    let id = call["id"].as_str().ok_or("missing call id")?;
    let name = call["name"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or("missing tool name")?;
    let arguments = call["input"]
        .as_object()
        .ok_or("tool arguments must be an object")?;
    let mut args = field_str(1, name);
    for (key, value) in arguments {
        let mut entry = field_str(1, key);
        // McpArgs.args is map<string, bytes>; each byte value encodes a
        // google.protobuf.Value (the reference client's argBytes contract).
        entry.extend(field_ld(2, &encode_protobuf_value(value)));
        args.extend(field_ld(2, &entry));
    }
    args.extend(field_str(3, id));
    args.extend(field_str(4, "shunt"));
    args.extend(field_str(5, name));
    let mut success = Vec::new();
    let content = result
        .get("content")
        .ok_or("tool result content is required")?;
    if let Some(text) = content.as_str() {
        success.extend(field_ld(1, &field_ld(1, &field_str(1, text))));
    } else {
        for item in content
            .as_array()
            .ok_or("tool result content must be text or blocks")?
        {
            if item["type"] != "text" {
                return Err("unsupported structured tool result content".into());
            }
            let text = item["text"]
                .as_str()
                .ok_or("tool result text must be a string")?;
            success.extend(field_ld(1, &field_ld(1, &field_str(1, text))));
        }
    }
    let is_error = match result.get("is_error") {
        None => false,
        Some(v) => v.as_bool().ok_or("is_error must be boolean")?,
    };
    success.extend(field_varint(2, u64::from(is_error)));
    let mut mcp = field_ld(1, &args);
    mcp.extend(field_ld(2, &field_ld(1, &success)));
    // ConversationStep.tool_call=2 / ToolCall.mcp_tool_call=15.
    Ok(field_ld(2, &field_ld(15, &mcp)))
}

/// Construct the actual graph of content-addressed turns and paired steps.
/// Rejects unsupported history before credentials are resolved by the caller.
pub(super) fn prepare(req: &Value, model: &str) -> Result<Option<PreparedHistory>, String> {
    for key in [
        "previous_response_id",
        "continuation",
        "checkpoint",
        "cursor_checkpoint",
    ] {
        if req.get(key).is_some_and(|v| !v.is_null()) {
            return Err(format!("opaque {key} continuation is unsupported"));
        }
    }
    let messages = req
        .get("messages")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let mut calls = HashMap::new();
    let mut results = HashMap::new();
    let mut seen = HashSet::new();
    for message in messages {
        for block in blocks(message) {
            match block["type"].as_str() {
                Some("tool_use") => {
                    if message["role"] != "assistant" {
                        return Err("tool_use requires assistant role".into());
                    }
                    let id = block["id"]
                        .as_str()
                        .filter(|s| !s.is_empty() && s.len() <= 512)
                        .ok_or("invalid tool_use id")?;
                    if !seen.insert(id) {
                        return Err("duplicate tool_use id".into());
                    }
                    calls.insert(id, block);
                }
                Some("tool_result") => {
                    if message["role"] != "user" {
                        return Err("tool_result requires user role".into());
                    }
                    let id = block["tool_use_id"].as_str().ok_or("missing tool_use_id")?;
                    if !calls.contains_key(id) || results.insert(id, block).is_some() {
                        return Err("unpaired or duplicate tool result".into());
                    }
                }
                Some("text" | "thinking" | "image") => {}
                _ => return Err("unsupported opaque history block".into()),
            }
        }
    }
    if calls.len() != results.len() {
        return Err("tool call has no result".into());
    }
    if !calls.is_empty() && messages.last().is_none_or(|m| m["role"] != "user") {
        return Err("structured history must end with a user message or tool result".into());
    }
    let session = session_id(req);
    if calls.is_empty() && session.is_none() {
        return Ok(None);
    }
    let session = session.ok_or(
        "structured history needs a stable metadata.session_id or JSON metadata.user_id session_id",
    )?;
    if !calls.is_empty() && model != "composer-2.5" {
        return Err("structured tool history is not verified for this exact Cursor model".into());
    }
    let mut store = RequestBlobStore::new();
    let conversation_id = digest_id(&[model, &session]);
    if calls.is_empty() {
        return Ok(Some(PreparedHistory {
            conversation_id,
            state: Vec::new(),
            blobs: store,
            prompt: super::request::render_cursor_prompt(req),
        }));
    }
    let continuation = messages.last().is_some_and(is_result);
    let end = if continuation {
        messages.len()
    } else {
        messages.len().saturating_sub(1)
    };
    let mut state = Vec::new();
    if let Some(system) = super::request::render_system(req) {
        let json = serde_json::to_vec(&serde_json::json!({"role":"system","content":system}))
            .map_err(|e| e.to_string())?;
        state.extend(field_ld(1, &put(&mut store, json)?));
    }
    let mut turn: Option<Vec<u8>> = None;
    for (index, message) in messages[..end].iter().enumerate() {
        // Roots are model-visible prompt history; turns additionally carry
        // native paired MCP state. Neither substitutes for the other.
        let root_text = match message["role"].as_str() {
            Some("user") if !is_result(message) => plain_text(message)?,
            Some("user") => blocks(message)
                .iter()
                .map(|result| {
                    let id = result["tool_use_id"]
                        .as_str()
                        .ok_or("missing tool result id")?;
                    let call = calls[id];
                    let name = call["name"].as_str().ok_or("missing tool name")?;
                    let text = plain_text(result)?;
                    let is_error = result
                        .get("is_error")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    let label = if is_error {
                        "Tool error"
                    } else {
                        "Tool output"
                    };
                    Ok(format!(
                        "{label} for {name} (call_id: {id}, is_error: {is_error}):\n{text}"
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?
                .join("\n"),
            Some("assistant") => message["content"]
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    blocks(message)
                        .iter()
                        .filter_map(|b| match b["type"].as_str() {
                            Some("text") => b["text"].as_str(),
                            Some("thinking") => b["thinking"].as_str(),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }),
            _ => String::new(),
        };
        if !root_text.is_empty() {
            let root = serde_json::to_vec(&serde_json::json!({
                "role": if is_result(message) { "assistant" } else { message["role"].as_str().unwrap_or("") },
                "content": [{"type":"text", "text":root_text}]
            })).map_err(|e| e.to_string())?;
            state.extend(field_ld(1, &put(&mut store, root)?));
        }
        match message["role"].as_str() {
            Some("user") if is_result(message) => {
                if blocks(message).iter().any(|b| b["type"] != "tool_result") {
                    return Err(
                        "mixed user text and tool results are unsupported in structured replay"
                            .into(),
                    );
                }
            }
            Some("user") => {
                if let Some(previous) = turn.take() {
                    state.extend(field_ld(8, &put(&mut store, field_ld(1, &previous))?));
                }
                let mut user = field_str(1, &plain_text(message)?);
                user.extend(field_str(
                    2,
                    &digest_id(&[&conversation_id, &index.to_string()]),
                ));
                user.extend(field_varint(4, 1));
                turn = Some(field_ld(1, &put(&mut store, user)?));
            }
            Some("assistant") => {
                let turn = turn.as_mut().ok_or(
                    "history starts without its user context; opaque recovery is unsupported",
                )?;
                if let Some(text) = message["content"].as_str() {
                    turn.extend(field_ld(
                        2,
                        &put(&mut store, field_ld(1, &field_str(1, text)))?,
                    ));
                }
                for block in blocks(message) {
                    let step = match block["type"].as_str() {
                        Some("text") => field_ld(
                            1,
                            &field_str(1, block["text"].as_str().ok_or("invalid assistant text")?),
                        ),
                        Some("thinking") => field_ld(
                            3,
                            &field_str(
                                1,
                                block["thinking"].as_str().ok_or("invalid thinking text")?,
                            ),
                        ),
                        Some("tool_use") => {
                            tool_step(block, results[block["id"].as_str().unwrap()])?
                        }
                        _ => return Err("unsupported assistant history block".into()),
                    };
                    turn.extend(field_ld(2, &put(&mut store, step)?));
                }
            }
            _ => return Err("unsupported history role".into()),
        }
    }
    if let Some(previous) = turn {
        state.extend(field_ld(8, &put(&mut store, field_ld(1, &previous))?));
    }
    let prompt = if continuation {
        // composer-2.5 uses UserMessageAction rather than a bare ResumeAction
        // in the referenced exact-model continuation path.
        "Continue: the requested tool results are provided in the conversation history above. Answer the user request or proceed with the next step directly without repeating status summaries or greetings.".to_owned()
    } else {
        plain_text(messages.last().ok_or("empty history")?)?
    };
    Ok(Some(PreparedHistory {
        conversation_id,
        state,
        blobs: store,
        prompt,
    }))
}

#[cfg(test)]
#[path = "history_tests.rs"]
mod tests;
