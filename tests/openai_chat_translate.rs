//! Pure Anthropic Messages -> OpenAI Chat Completions request translation.
//!
//! These fixtures exercise the whitelist translator with no transport: every
//! accepted Anthropic request field must map to its Chat counterpart, every
//! unsupported representation must be a typed request error, and the outbound
//! body must never carry a key outside the whitelist.

use serde_json::{json, Value};
use shunt::model::openai_chat_request::{translate_request, MAX_TEXT_BLOCK_BYTES};

fn translate(request: Value) -> Result<Value, String> {
    translate_request(&request, "gpt-5", false).map_err(|error| error.message)
}

#[test]
fn translate_rejects_null_request_body() {
    let error = translate(json!(null)).expect_err("null body must be a typed request error");
    assert!(!error.is_empty(), "{error}");
}

#[test]
fn translate_rejects_non_object_request_body() {
    let error = translate(json!(["not", "an", "object"]))
        .expect_err("non-object body must be a typed request error");
    assert!(!error.is_empty(), "{error}");
}

#[test]
fn translate_rejects_empty_messages() {
    let error = translate(json!({"messages": []}))
        .expect_err("zero messages must be a typed request error");
    assert!(error.contains("messages"), "{error}");
}

#[test]
fn translate_rejects_message_without_content() {
    let error = translate(json!({"messages": [{"role": "user"}]}))
        .expect_err("a message without content must be a typed request error");
    assert!(error.contains("content"), "{error}");
}

#[test]
fn translate_rejects_empty_user_text() {
    let error = translate(json!({"messages": [{"role": "user", "content": ""}]}))
        .expect_err("empty required text content must be a typed request error");
    assert!(error.contains("empty"), "{error}");
}

#[test]
fn translate_rejects_empty_content_blocks() {
    let error = translate(json!({"messages": [{"role": "user", "content": []}]}))
        .expect_err("an empty content block list must be a typed request error");
    assert!(error.contains("empty"), "{error}");
}

#[test]
fn translate_single_message_input_is_accepted() {
    let out = translate(json!({"messages": [{"role": "user", "content": "hello"}]}))
        .expect("a single-message request is valid");
    assert_eq!(
        out["messages"],
        json!([{"role": "user", "content": "hello"}])
    );
}

#[test]
fn translate_system_becomes_a_system_message() {
    let request = json!({
        "system": "be brief",
        "messages": [{"role": "user", "content": "hello"}]
    });
    let out = translate(request).expect("system text must translate");
    assert!(
        out.get("system").is_none(),
        "Chat Completions has no top-level system field: {out}"
    );
    assert_eq!(
        out["messages"][0],
        json!({"role": "system", "content": "be brief"}),
        "{out}"
    );
    assert_eq!(out["messages"][1]["role"], "user", "{out}");
}

#[test]
fn translate_maps_generation_controls() {
    let request = json!({
        "max_tokens": 512,
        "temperature": 0.5,
        "top_p": 0.9,
        "stop_sequences": ["a", "b"],
        "messages": [{"role": "user", "content": "hello"}]
    });
    let out = translate(request).expect("generation controls must translate");
    assert_eq!(out["max_tokens"], 512, "{out}");
    assert_eq!(out["temperature"], 0.5, "{out}");
    assert_eq!(out["top_p"], 0.9, "{out}");
    assert_eq!(out["stop"], json!(["a", "b"]), "{out}");
}

#[test]
fn translate_absent_controls_stay_absent() {
    let out = translate(json!({"messages": [{"role": "user", "content": "hello"}]}))
        .expect("minimal request must translate");
    for control in ["max_tokens", "temperature", "top_p", "stop"] {
        assert!(
            out.get(control).is_none(),
            "absent {control} must not be invented: {out}"
        );
    }
}

#[test]
fn translate_emits_exact_whitelisted_body() {
    let request = json!({
        "model": "claude-via-chat",
        "max_tokens": 64,
        "temperature": 0.5,
        "stream": false,
        "messages": [{"role": "user", "content": "fixture"}]
    });
    let out = translate(request).expect("whitelisted request must translate");
    assert_eq!(
        out,
        json!({
            "model": "gpt-5",
            "stream": false,
            "max_tokens": 64,
            "temperature": 0.5,
            "messages": [{"role": "user", "content": "fixture"}]
        }),
        "the outbound body must be exactly the whitelist mapping: {out}"
    );
}

#[test]
fn translate_rejects_unknown_top_level_field() {
    let mut request = json!({"messages": [{"role": "user", "content": "hello"}]});
    request["top_k"] = json!(3);
    let error = translate(request)
        .expect_err("unknown top-level fields must be typed request errors, never dropped");
    assert!(error.contains("top_k"), "{error}");
}

#[test]
fn translate_rejects_metadata_field() {
    let mut request = json!({"messages": [{"role": "user", "content": "hello"}]});
    request["metadata"] = json!({"user_id": "u1"});
    let error = translate(request).expect_err("metadata must be a typed request error");
    assert!(error.contains("metadata"), "{error}");
}

#[test]
fn translate_rejects_unknown_content_block_type() {
    let request = json!({
        "messages": [{
            "role": "user",
            "content": [{"type": "document", "source": {"type": "url", "url": "https://x/f.pdf"}}]
        }]
    });
    let error = translate(request)
        .expect_err("unsupported content block types must be typed request errors");
    assert!(error.contains("document"), "{error}");
}

#[test]
fn translate_image_blocks_map_to_image_url() {
    let request = json!({
        "messages": [{
            "role": "user",
            "content": [
                {"type": "text", "text": "look"},
                {"type": "image", "source": {"type": "url", "url": "https://example.com/cat.png"}},
                {"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": "QUJD"}}
            ]
        }]
    });
    let out = translate(request).expect("image blocks must translate");
    assert_eq!(
        out["messages"][0]["content"],
        json!([
            {"type": "text", "text": "look"},
            {"type": "image_url", "image_url": {"url": "https://example.com/cat.png"}},
            {"type": "image_url", "image_url": {"url": "data:image/png;base64,QUJD"}}
        ]),
        "{out}"
    );
}

#[test]
fn translate_rejects_image_in_non_user_message() {
    let request = json!({
        "messages": [{
            "role": "assistant",
            "content": [
                {"type": "text", "text": "look"},
                {"type": "image", "source": {"type": "url", "url": "https://example.com/cat.png"}}
            ]
        }]
    });
    let error = translate(request)
        .expect_err("images are a user-content feature and must be rejected elsewhere");
    assert!(error.contains("image"), "{error}");
}

#[test]
fn translate_rejects_unknown_image_source_type() {
    let request = json!({
        "messages": [{
            "role": "user",
            "content": [
                {"type": "image", "source": {"type": "teleport", "url": "https://example.com/cat.png"}}
            ]
        }]
    });
    let error = translate(request).expect_err("unsupported image sources must be typed errors");
    assert!(error.contains("image"), "{error}");
}

#[test]
fn translate_preserves_cjk_text_bytes() {
    let text = " longitudinally-invalid: \u{6f22}\u{5b57}\u{30c6}\u{30b9}\u{30c8} emoji \u{1f600} ";
    let request = json!({"messages": [{"role": "user", "content": text}]});
    let out = translate(request).expect("CJK text must translate");
    let translated = out["messages"][0]["content"]
        .as_str()
        .expect("string content");
    assert_eq!(
        translated.as_bytes(),
        text.as_bytes(),
        "no normalization allowed"
    );
}

#[test]
fn translate_rejects_text_over_byte_budget() {
    let oversized = "x".repeat(MAX_TEXT_BLOCK_BYTES + 1);
    let error = translate(json!({"messages": [{"role": "user", "content": oversized}]}))
        .expect_err("content over the UTF-8 byte budget must be a typed request error");
    assert!(error.contains("bytes"), "{error}");
}

#[test]
fn translate_plaintext_thinking_maps_to_reasoning_content() {
    let request = json!({
        "messages": [{
            "role": "assistant",
            "content": [
                {"type": "thinking", "thinking": "hmm"},
                {"type": "text", "text": "answer"}
            ]
        }]
    });
    let out = translate(request).expect("plaintext thinking must translate");
    assert_eq!(
        out["messages"][0],
        json!({"role": "assistant", "content": "answer", "reasoning_content": "hmm"}),
        "{out}"
    );
}

#[test]
fn translate_rejects_signed_thinking() {
    let request = json!({
        "messages": [{
            "role": "assistant",
            "content": [
                {"type": "thinking", "thinking": "hmm", "signature": "sig"},
                {"type": "text", "text": "answer"}
            ]
        }]
    });
    let error = translate(request)
        .expect_err("signed thinking cannot be forwarded losslessly and must be rejected");
    assert!(error.contains("signature"), "{error}");
}

#[test]
fn translate_rejects_redacted_thinking() {
    let request = json!({
        "messages": [{
            "role": "assistant",
            "content": [
                {"type": "redacted_thinking", "data": "opaque"},
                {"type": "text", "text": "answer"}
            ]
        }]
    });
    let error = translate(request)
        .expect_err("redacted thinking cannot be forwarded losslessly and must be rejected");
    assert!(error.contains("redacted"), "{error}");
}
