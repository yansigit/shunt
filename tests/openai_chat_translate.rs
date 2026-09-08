//! Pure Anthropic Messages -> OpenAI Chat Completions request translation.
//!
//! These fixtures exercise the whitelist translator with no transport: every
//! accepted Anthropic request field must map to its Chat counterpart, every
//! unsupported representation must be a typed request error, and the outbound
//! body must never carry a key outside the whitelist.

use serde_json::{json, Value};

#[test]
fn response_root_null_empty_completion_has_text_block() {
    let out = unary_final(&completion_with(
        json!({"content":null}),
        json!("stop"),
        json!({"prompt_tokens":0,"completion_tokens":0}),
    ))
    .unwrap();
    assert_eq!(out["content"], json!([{"type":"text","text":""}]));
}

#[test]
fn response_root_unary_tool_validation() {
    for calls in [
        json!(null),
        json!({}),
        json!([{"id":"a","type":"function","function":{"name":"f","arguments":"[]"}}]),
        json!([{"id":"a","type":"function","function":{"name":"f","arguments":"{"}}]),
        json!([{"id":"a","type":"function","function":{"name":"","arguments":"{}"}}]),
    ] {
        assert!(unary_final(&completion_with(
            json!({"tool_calls":calls}),
            json!("tool_calls"),
            json!(null)
        ))
        .is_err());
    }
}

#[test]
fn response_root_unary_tools_are_not_discarded() {
    let mut machine =
        shunt::model::openai_chat_response::OpenAiChatSseMachine::new_for_upstream("m");
    machine.process_chunk_checked(&json!({"choices":[{"message":{"reasoning_content":"think","content":"text","tool_calls":[{"id":"call_a","type":"function","function":{"name":"f","arguments":"{\"x\":1}"}}]},"finish_reason":"tool_calls"}]})).unwrap();
    machine.transport_close_checked().unwrap();
    assert_eq!(
        machine.final_json_checked().unwrap()["content"],
        json!([
            {"type":"thinking","thinking":"think"}, {"type":"text","text":"text"},
            {"type":"tool_use","id":"call_a","name":"f","input":{"x":1}}
        ])
    );
}

#[test]
fn response_root_message_envelope_is_required_and_unambiguous() {
    for choice in [
        json!({"finish_reason":"stop"}),
        json!({"message":null,"finish_reason":"stop"}),
        json!({"message":[],"finish_reason":"stop"}),
        json!({"message":{},"delta":{},"finish_reason":"stop"}),
    ] {
        let mut machine =
            shunt::model::openai_chat_response::OpenAiChatSseMachine::new_for_upstream("m");
        assert!(
            machine
                .process_chunk_checked(&json!({"choices":[choice]}))
                .is_err(),
            "accepted {choice}"
        );
    }
}

#[test]
fn response_root_trailing_usage_requires_exact_shape() {
    for trailing in [
        json!({}),
        json!({"choices":[],"usage":null}),
        json!({"choices":[{}],"usage":{"prompt_tokens":1}}),
        json!({"choices":[],"usage":{}}),
    ] {
        let mut machine =
            shunt::model::openai_chat_response::OpenAiChatSseMachine::new_streaming_for_upstream(
                "m",
            );
        machine
            .process_chunk_checked(&json!({"choices":[{"delta":{},"finish_reason":"stop"}]}))
            .unwrap();
        assert!(
            machine.process_chunk_checked(&trailing).is_err(),
            "accepted {trailing}"
        );
        assert!(
            machine.transport_close_checked().is_err(),
            "invalid trailing chunk left success possible"
        );
    }
}

#[test]
fn response_root_malformed_postfinish_error_does_not_panic() {
    let mut machine =
        shunt::model::openai_chat_response::OpenAiChatSseMachine::new_streaming_for_upstream("m");
    machine
        .process_chunk_checked(&json!({"choices":[{"delta":{},"finish_reason":"stop"}]}))
        .unwrap();
    assert!(machine
        .process_chunk_checked(&json!({"error":null}))
        .is_err());
}

#[test]
fn response_root_reasoning_is_bounded_across_chunks() {
    let mut machine =
        shunt::model::openai_chat_response::OpenAiChatSseMachine::new_for_upstream("m");
    let chunk = json!({"choices":[{"delta":{"reasoning_content":"x".repeat(4*1024*1024+1)}}]});
    machine.process_chunk_checked(&chunk).unwrap();
    assert!(machine.process_chunk_checked(&chunk).is_err());
}

#[test]
fn response_root_error_diagnostics_are_provider_neutral() {
    let mut machine =
        shunt::model::openai_chat_response::OpenAiChatSseMachine::new_for_upstream("m");
    let events = machine
        .process_chunk_checked(&json!({"error":{"message":"secret-marker"}}))
        .unwrap();
    assert!(!events[0].data.to_string().contains("secret-marker"));
}

#[test]
fn response_root_choice_error_cannot_be_success() {
    let mut machine =
        shunt::model::openai_chat_response::OpenAiChatSseMachine::new_for_upstream("m");
    let events = machine.process_chunk_checked(&json!({"choices":[{"message":{"content":"hi"},"finish_reason":"stop","error":{"message":"failed"}}]})).unwrap();
    assert_eq!(events[0].event, "error");
    assert!(machine.transport_close_checked().is_err());
}
use shunt::model::openai_chat_request::{
    chat_completions_endpoint, translate_request, MAX_TEXT_BLOCK_BYTES,
};
use shunt::model::openai_chat_response::OpenAiChatSseMachine;

fn translate(request: Value) -> Result<Value, String> {
    translate_request(&request, "gpt-5", false).map_err(|error| error.message)
}

#[test]
fn root_review_tool_choice_objects() {
    for (kind, expected) in [("auto", "auto"), ("any", "required")] {
        let out = translate(
            json!({"tool_choice":{"type":kind},"messages":[{"role":"user","content":"hello"}]}),
        )
        .unwrap();
        assert_eq!(out["tool_choice"], expected);
    }
}

#[test]
fn root_review_duplicate_tool_result_rejected() {
    assert!(translate(json!({"messages":[
        {"role":"assistant","content":[{"type":"tool_use","id":"a","name":"f","input":{}}]},
        {"role":"user","content":[{"type":"tool_result","tool_use_id":"a","content":"first"},{"type":"tool_result","tool_use_id":"a","content":"second"}]}
    ]})).is_err());
}

#[test]
fn root_review_tool_results_precede_followup_text() {
    let out = translate(json!({"messages":[
        {"role":"assistant","content":[{"type":"tool_use","id":"a","name":"f","input":{}}]},
        {"role":"user","content":[{"type":"tool_result","tool_use_id":"a","content":"result"},{"type":"text","text":"continue"}]}
    ]})).unwrap();
    assert_eq!(out["messages"][1]["role"], "tool");
    assert_eq!(out["messages"][2]["role"], "user");
}

#[test]
fn root_review_combined_text_budget() {
    let half = "x".repeat(MAX_TEXT_BLOCK_BYTES / 2 + 1);
    assert!(translate(json!({"messages":[{"role":"user","content":[{"type":"text","text":half},{"type":"text","text":half}]}]})).is_err());
}

#[test]
fn root_review_endpoint_diagnostics_do_not_echo_userinfo() {
    let error =
        chat_completions_endpoint("https://user:secret-marker@host.example/v1").unwrap_err();
    assert!(!error.contains("secret-marker"));
}

#[test]
fn root_review_endpoint_rejects_silent_repairs() {
    for root in [
        " https://host.example/v1",
        "https://host.example/\nv1",
        "https://host.example/a/../v1",
        "https://host.example\\v1",
    ] {
        assert!(
            chat_completions_endpoint(root).is_err(),
            "accepted ambiguous root {root:?}"
        );
    }
}

#[test]
fn root_review_nested_fields_are_not_silently_lost() {
    for request in [
        json!({"messages":[{"role":"user","content":"hi","unknown":true}]}),
        json!({"messages":[{"role":"user","content":[{"type":"text","text":"hi","cache_control":{}}]}]}),
        json!({"tools":[{"name":"f","input_schema":{},"unknown":true}],"messages":[{"role":"user","content":"hi"}]}),
        json!({"stream":"yes","messages":[{"role":"user","content":"hi"}]}),
        json!({"tool_choice":{"type":"tool","name":"f","unknown":true},"messages":[{"role":"user","content":"hi"}]}),
    ] {
        assert!(translate(request).is_err());
    }
}

#[test]
fn root_review_reasoning_bytes_preserved() {
    let out = translate(json!({"messages":[{"role":"assistant","content":[
        {"type":"thinking","thinking":"one"},{"type":"thinking","thinking":"two"},{"type":"text","text":"answer"}
    ]}]})).unwrap();
    assert_eq!(out["messages"][0]["reasoning_content"], "onetwo");
}

#[test]
fn root_review_thinking_only_history_is_preserved() {
    let out = translate(json!({"messages":[{"role":"assistant","content":[{"type":"thinking","thinking":"reason"}]}]})).unwrap();
    assert_eq!(out["messages"][0]["reasoning_content"], "reason");
    assert!(out["messages"][0]["content"].is_null());
}

#[test]
fn root_review_endpoint_requires_explicit_authority() {
    for root in ["https:///host.example/v1", "https://@host.example/v1"] {
        assert!(chat_completions_endpoint(root).is_err(), "accepted {root}");
    }
}

#[test]
fn root_review_empty_tool_identity_rejected() {
    for (id, name) in [("", "f"), ("a", "")] {
        assert!(translate(json!({"messages":[{"role":"assistant","content":[{"type":"tool_use","id":id,"name":name,"input":{}}]}]})).is_err());
    }
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

#[test]
fn endpoint_single_append() {
    let endpoint =
        chat_completions_endpoint("https://host.example/api/v1").expect("base path must append");
    assert_eq!(endpoint, "https://host.example/api/v1/chat/completions");
}

#[test]
fn endpoint_trailing_slash_normalizes() {
    let endpoint = chat_completions_endpoint("https://host.example/v1/")
        .expect("trailing slash must normalize");
    assert_eq!(endpoint, "https://host.example/v1/chat/completions");
}

#[test]
fn endpoint_root_ending_with_chat_completions_is_used_as_is() {
    let endpoint = chat_completions_endpoint("https://host.example/v1/chat/completions")
        .expect("a root already ending in /chat/completions must not double");
    assert_eq!(endpoint, "https://host.example/v1/chat/completions");
}

#[test]
fn endpoint_determinism() {
    let root = "https://host.example/api/v1";
    let first = chat_completions_endpoint(root).unwrap();
    let second = chat_completions_endpoint(root).unwrap();
    assert_eq!(
        first.as_bytes(),
        second.as_bytes(),
        "repeated builds must be byte-identical"
    );
    let handles: Vec<_> = (0..4)
        .map(|_| std::thread::spawn(move || chat_completions_endpoint(root).unwrap()))
        .collect();
    for handle in handles {
        assert_eq!(handle.join().unwrap().as_bytes(), first.as_bytes());
    }
}

#[test]
fn endpoint_boot_reject_query() {
    let error = chat_completions_endpoint("https://host.example/v1?key=1")
        .expect_err("a query string is ambiguous and must be rejected");
    assert!(!error.is_empty(), "{error}");
}

#[test]
fn endpoint_boot_reject_fragment() {
    let error = chat_completions_endpoint("https://host.example/v1#frag")
        .expect_err("a fragment is ambiguous and must be rejected");
    assert!(!error.is_empty(), "{error}");
}

#[test]
fn endpoint_boot_reject_userinfo() {
    let error = chat_completions_endpoint("https://user:pass@host.example/v1")
        .expect_err("userinfo is ambiguous and must be rejected before credentials");
    assert!(!error.is_empty(), "{error}");
}
#[test]
fn tool_declaration_map() {
    let request = json!({
        "tools": [{
            "name": "get_weather",
            "description": "Look up weather",
            "input_schema": {"type": "object", "properties": {"city": {"type": "string"}}}
        }],
        "messages": [{"role": "user", "content": "hello"}]
    });
    let out = translate(request).expect("tool declarations must translate");
    assert_eq!(
        out["tools"],
        json!([{
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "Look up weather",
                "parameters": {"type": "object", "properties": {"city": {"type": "string"}}}
            }
        }]),
        "{out}"
    );
}

#[test]
fn tool_choice_auto_maps() {
    let out = translate(json!({
        "tool_choice": "auto",
        "messages": [{"role": "user", "content": "hello"}]
    }))
    .expect("tool_choice auto must translate");
    assert_eq!(out["tool_choice"], "auto", "{out}");
}

#[test]
fn tool_choice_any_maps_to_required() {
    let out = translate(json!({
        "tool_choice": "any",
        "messages": [{"role": "user", "content": "hello"}]
    }))
    .expect("tool_choice any must translate");
    assert_eq!(out["tool_choice"], "required", "{out}");
}

#[test]
fn tool_choice_by_name_maps_to_function() {
    let out = translate(json!({
        "tool_choice": {"type": "tool", "name": "get_weather"},
        "messages": [{"role": "user", "content": "hello"}]
    }))
    .expect("tool_choice tool must translate");
    assert_eq!(
        out["tool_choice"],
        json!({"type": "function", "function": {"name": "get_weather"}}),
        "{out}"
    );
}

#[test]
fn tool_choice_unknown_reject() {
    let error = translate(json!({
        "tool_choice": {"type": "mood", "name": "get_weather"},
        "messages": [{"role": "user", "content": "hello"}]
    }))
    .expect_err("unknown tool_choice must be a typed request error");
    assert!(error.contains("tool_choice"), "{error}");
}

#[test]
fn tool_parallel_calls_with_paired_results() {
    let request = json!({
        "messages": [
            {"role": "user", "content": "weather in two cities"},
            {
                "role": "assistant",
                "content": [
                    {"type": "tool_use", "id": "tu_1", "name": "get_weather", "input": {"city": "Tokyo"}},
                    {"type": "tool_use", "id": "tu_2", "name": "get_weather", "input": {"city": "Paris"}}
                ]
            },
            {
                "role": "user",
                "content": [
                    {"type": "tool_result", "tool_use_id": "tu_1", "content": "sunny"},
                    {"type": "tool_result", "tool_use_id": "tu_2", "content": "rainy"}
                ]
            }
        ]
    });
    let out = translate(request).expect("parallel tool calls with results must translate");
    let assistant = &out["messages"][1];
    assert_eq!(assistant["role"], "assistant", "{out}");
    assert_eq!(
        assistant["tool_calls"],
        json!([
            {
                "id": "tu_1",
                "type": "function",
                "function": {"name": "get_weather", "arguments": "{\"city\":\"Tokyo\"}"}
            },
            {
                "id": "tu_2",
                "type": "function",
                "function": {"name": "get_weather", "arguments": "{\"city\":\"Paris\"}"}
            }
        ]),
        "ids must be stable and ordered: {out}"
    );
    assert_eq!(
        out["messages"][2],
        json!({"role": "tool", "tool_call_id": "tu_1", "content": "sunny"}),
        "{out}"
    );
    assert_eq!(
        out["messages"][3],
        json!({"role": "tool", "tool_call_id": "tu_2", "content": "rainy"}),
        "{out}"
    );
}

#[test]
fn tool_use_arguments_serialize_object_once() {
    let request = json!({
        "messages": [
            {
                "role": "assistant",
                "content": [
                    {"type": "tool_use", "id": "tu_1", "name": "get_weather", "input": {"city": "Tokyo", "days": 3}}
                ]
            }
        ]
    });
    let out = translate(request).expect("tool_use must translate");
    let arguments = out["messages"][0]["tool_calls"][0]["function"]["arguments"]
        .as_str()
        .expect("arguments must be a JSON string");
    let parsed: Value = serde_json::from_str(arguments).expect("arguments must be valid JSON");
    assert_eq!(
        parsed,
        json!({"city": "Tokyo", "days": 3}),
        "argument values must round-trip regardless of key order: {arguments}"
    );
}

#[test]
fn tool_use_with_text_keeps_content() {
    let request = json!({
        "messages": [
            {
                "role": "assistant",
                "content": [
                    {"type": "text", "text": "checking"},
                    {"type": "tool_use", "id": "tu_1", "name": "get_weather", "input": {"city": "Tokyo"}}
                ]
            }
        ]
    });
    let out = translate(request).expect("assistant text plus tool_use must translate");
    assert_eq!(out["messages"][0]["content"], "checking", "{out}");
    assert_eq!(out["messages"][0]["tool_calls"][0]["id"], "tu_1", "{out}");
}

#[test]
fn tool_result_block_content_concatenates() {
    let request = json!({
        "messages": [
            {
                "role": "assistant",
                "content": [
                    {"type": "tool_use", "id": "tu_1", "name": "get_weather", "input": {"city": "Tokyo"}}
                ]
            },
            {
                "role": "user",
                "content": [
                    {"type": "tool_result", "tool_use_id": "tu_1", "content": [
                        {"type": "text", "text": "sunny"},
                        {"type": "text", "text": " and warm"}
                    ]}
                ]
            }
        ]
    });
    let out = translate(request).expect("tool_result block content must translate");
    assert_eq!(out["messages"][1]["content"], "sunny and warm", "{out}");
}

#[test]
fn tool_orphan_result_reject() {
    let request = json!({
        "messages": [
            {"role": "user", "content": "hello"},
            {
                "role": "user",
                "content": [{"type": "tool_result", "tool_use_id": "tu_missing", "content": "sunny"}]
            }
        ]
    });
    let error = translate(request)
        .expect_err("a tool_result without a preceding matching tool_use must be rejected");
    assert!(error.contains("tool_result"), "{error}");
}

#[test]
fn tool_duplicate_use_id_reject() {
    let request = json!({
        "messages": [
            {
                "role": "assistant",
                "content": [
                    {"type": "tool_use", "id": "tu_1", "name": "get_weather", "input": {"city": "Tokyo"}},
                    {"type": "tool_use", "id": "tu_1", "name": "get_weather", "input": {"city": "Paris"}}
                ]
            }
        ]
    });
    let error = translate(request).expect_err("a duplicate tool_use id must be rejected");
    assert!(error.contains("duplicate"), "{error}");
}

#[test]
fn tool_use_missing_identity_reject() {
    let request = json!({
        "messages": [
            {
                "role": "assistant",
                "content": [
                    {"type": "tool_use", "name": "get_weather", "input": {"city": "Tokyo"}}
                ]
            }
        ]
    });
    let error = translate(request).expect_err("a tool_use without its id must be rejected");
    assert!(error.contains("id"), "{error}");
}

#[test]
fn tool_use_null_input_reject() {
    let error = translate(json!({
        "messages": [
            {
                "role": "assistant",
                "content": [
                    {"type": "tool_use", "id": "tu_1", "name": "get_weather", "input": null}
                ]
            }
        ]
    }))
    .expect_err("null tool_use input must be rejected, never coerced to an empty object");
    assert!(error.contains("input"), "{error}");
}

#[test]
fn tool_use_scalar_input_reject() {
    let error = translate(json!({
        "messages": [
            {
                "role": "assistant",
                "content": [
                    {"type": "tool_use", "id": "tu_1", "name": "get_weather", "input": 7}
                ]
            }
        ]
    }))
    .expect_err("scalar tool_use input must be rejected");
    assert!(error.contains("input"), "{error}");
}

#[test]
fn tool_use_array_input_reject() {
    let error = translate(json!({
        "messages": [
            {
                "role": "assistant",
                "content": [
                    {"type": "tool_use", "id": "tu_1", "name": "get_weather", "input": ["Tokyo"]}
                ]
            }
        ]
    }))
    .expect_err("array tool_use input must be rejected");
    assert!(error.contains("input"), "{error}");
}

#[test]
fn tool_use_string_input_reject() {
    let error = translate(json!({
        "messages": [
            {
                "role": "assistant",
                "content": [
                    {"type": "tool_use", "id": "tu_1", "name": "get_weather", "input": "{\"city\":\"Tokyo\"}"}
                ]
            }
        ]
    }))
    .expect_err("string tool_use input is incomplete arguments and must be rejected");
    assert!(error.contains("input"), "{error}");
}

#[test]
fn tool_result_without_content_uses_empty_payload() {
    let request = json!({
        "messages": [
            {
                "role": "assistant",
                "content": [
                    {"type": "tool_use", "id": "tu_1", "name": "get_weather", "input": {"city": "Tokyo"}}
                ]
            },
            {
                "role": "user",
                "content": [{"type": "tool_result", "tool_use_id": "tu_1"}]
            }
        ]
    });
    let out = translate(request).expect("tool_result without content must translate");
    assert_eq!(out["messages"][1]["content"], "", "{out}");
}

#[test]
fn tool_translation_thread_isolation() {
    let make = |id: &str, city: &str| {
        json!({
            "messages": [
                {
                    "role": "assistant",
                    "content": [
                        {"type": "tool_use", "id": id, "name": "get_weather", "input": {"city": city}}
                    ]
                }
            ]
        })
    };
    let handles: Vec<_> = [("tu_a", "Osaka"), ("tu_b", "Lima")]
        .into_iter()
        .map(|(id, city)| {
            let request = make(id, city);
            std::thread::spawn(move || translate(request).unwrap())
        })
        .collect();
    for (index, handle) in handles.into_iter().enumerate() {
        let expected_id = if index == 0 { "tu_a" } else { "tu_b" };
        let expected_city = if index == 0 { "Osaka" } else { "Lima" };
        let out = handle.join().unwrap();
        let call = &out["messages"][0]["tool_calls"][0];
        assert_eq!(call["id"], expected_id, "per-call identity leaked: {out}");
        let parsed: Value =
            serde_json::from_str(call["function"]["arguments"].as_str().unwrap()).unwrap();
        assert_eq!(
            parsed["city"], expected_city,
            "per-call arguments leaked: {out}"
        );
    }
}
// ===================================================================
// 13-03 response machine fixtures (CHAT-05/06/07). Pure: no transport.
// ===================================================================

fn unary_final(chunk: &Value) -> Result<Value, String> {
    let mut machine = OpenAiChatSseMachine::new_for_upstream("gpt-5");
    let events = machine
        .process_chunk_checked(chunk)
        .map_err(|e| e.to_string())?;
    if let Some(error) = events.iter().find(|event| event.event == "error") {
        return Err(error.data.to_string());
    }
    machine
        .transport_close_checked()
        .map_err(|e| e.to_string())?;
    machine.final_json_checked().map_err(|e| e.to_string())
}

fn completion_with(message: Value, finish: Value, usage: Value) -> Value {
    json!({
        "id": "chatcmpl-x",
        "object": "chat.completion",
        "model": "gpt-5",
        "choices": [{"index": 0, "message": message, "finish_reason": finish}],
        "usage": usage
    })
}

fn stream_events(chunks: &[Value]) -> Vec<(String, Value)> {
    let mut machine = OpenAiChatSseMachine::new_streaming_for_upstream("gpt-5");
    let mut out = Vec::new();
    for chunk in chunks {
        match machine.process_chunk_checked(chunk) {
            Ok(events) => out.extend(events.into_iter().map(|e| (e.event, e.data))),
            Err(error) => {
                out.push(("error".to_string(), json!({"message": error.to_string()})));
                return out;
            }
        }
        if out.iter().any(|(event, _)| event == "error") {
            return out;
        }
    }
    match machine.transport_close_checked() {
        Ok(events) => out.extend(events.into_iter().map(|e| (e.event, e.data))),
        Err(error) => out.push(("error".to_string(), json!({"message": error.to_string()}))),
    }
    out
}

fn delta_chunk(delta: Value, finish: Value) -> Value {
    json!({"choices": [{"index": 0, "delta": delta, "finish_reason": finish}], "usage": null})
}

fn error_count(events: &[(String, Value)]) -> usize {
    events.iter().filter(|(event, _)| event == "error").count()
}

fn stop_count(events: &[(String, Value)]) -> usize {
    events
        .iter()
        .filter(|(event, _)| event == "message_stop")
        .count()
}

#[test]
fn response_finish_map_stop_length_tool_calls() {
    let stop = unary_final(&completion_with(
        json!({"role": "assistant", "content": "hi"}),
        json!("stop"),
        json!({"prompt_tokens": 3, "completion_tokens": 1}),
    ))
    .unwrap();
    assert_eq!(stop["stop_reason"], "end_turn", "{stop}");
    let length = unary_final(&completion_with(
        json!({"role": "assistant", "content": "hi"}),
        json!("length"),
        json!({"prompt_tokens": 3, "completion_tokens": 1}),
    ))
    .unwrap();
    assert_eq!(length["stop_reason"], "max_tokens", "{length}");
    let tools = unary_final(&completion_with(
        json!({"role": "assistant", "content": null}),
        json!("tool_calls"),
        json!({"prompt_tokens": 3, "completion_tokens": 1}),
    ))
    .unwrap();
    assert_eq!(tools["stop_reason"], "tool_use", "{tools}");
}

#[test]
fn response_finish_map_content_filter_fails_closed() {
    let error = unary_final(&completion_with(
        json!({"role": "assistant", "content": "x"}),
        json!("content_filter"),
        json!({"prompt_tokens": 1, "completion_tokens": 1}),
    ))
    .expect_err("content_filter must fail closed");
    assert!(error.contains("finish_reason"), "{error}");
}

#[test]
fn response_finish_map_deprecated_function_call_fails_closed() {
    let error = unary_final(&completion_with(
        json!({"role": "assistant", "content": "x"}),
        json!("function_call"),
        json!({"prompt_tokens": 1, "completion_tokens": 1}),
    ))
    .expect_err("deprecated function_call finish must fail closed");
    assert!(error.contains("finish_reason"), "{error}");
}

#[test]
fn response_finish_map_error_is_provider_error() {
    let mut machine = OpenAiChatSseMachine::new_for_upstream("gpt-5");
    let events = machine
        .process_chunk_checked(&completion_with(
            json!({"role": "assistant"}),
            json!("error"),
            json!(null),
        ))
        .unwrap();
    let frames: Vec<(String, Value)> = events.into_iter().map(|e| (e.event, e.data)).collect();
    assert_eq!(error_count(&frames), 1, "{frames:?}");
    assert!(
        machine.transport_close_checked().is_err(),
        "a provider error must never close as success",
    );
}

#[test]
fn response_empty_text_completion_accepted() {
    let out = unary_final(&completion_with(
        json!({"role": "assistant", "content": ""}),
        json!("stop"),
        json!({"prompt_tokens": 4, "completion_tokens": 0}),
    ))
    .unwrap();
    assert_eq!(out["content"], json!([{"type": "text", "text": ""}]));
    assert_eq!(out["stop_reason"], "end_turn");
    assert_eq!(out["usage"], json!({"input_tokens": 4, "output_tokens": 0}));
}

#[test]
fn response_rejects_null_and_non_object_body() {
    for body in [json!(null), json!([1, 2, 3]), json!("nope")] {
        let mut machine = OpenAiChatSseMachine::new_for_upstream("gpt-5");
        let error = machine
            .process_chunk_checked(&body)
            .expect_err("non-object response body must fail closed");
        assert!(error.to_string().contains("object"), "{error}");
    }
}

#[test]
fn response_rejects_absent_choices() {
    let body = json!({
        "id": "chatcmpl-x",
        "usage": {"prompt_tokens": 1, "completion_tokens": 1},
    });
    let error = unary_final(&body).expect_err("absent choices must fail closed");
    assert!(error.contains("choices"), "{error}");
}

#[test]
fn response_rejects_empty_choices_unary() {
    let body = json!({
        "choices": [],
        "usage": {"prompt_tokens": 1, "completion_tokens": 1},
    });
    let error = unary_final(&body).expect_err("empty choices must fail closed");
    assert!(error.contains("choices"), "{error}");
}

#[test]
fn response_rejects_multiple_choices() {
    let choice = json!({"index": 0, "message": {"content": "x"}, "finish_reason": "stop"});
    let body = json!({
        "choices": [choice.clone(), choice],
        "usage": {"prompt_tokens": 1, "completion_tokens": 1},
    });
    let error = unary_final(&body).expect_err("multiple choices must fail closed");
    assert!(
        error.contains("choices") || error.contains("ambiguous"),
        "{error}"
    );
}

#[test]
fn response_usage_precision_accepts_integral_bounds() {
    let zero = unary_final(&completion_with(
        json!({"role": "assistant", "content": "x"}),
        json!("stop"),
        json!({"prompt_tokens": 0, "completion_tokens": 0}),
    ))
    .unwrap();
    assert_eq!(zero["usage"]["input_tokens"], 0);
    let big = unary_final(&completion_with(
        json!({"role": "assistant", "content": "x"}),
        json!("stop"),
        json!({"prompt_tokens": i64::MAX, "completion_tokens": i64::MAX}),
    ))
    .unwrap();
    assert_eq!(big["usage"]["input_tokens"], i64::MAX);
    assert_eq!(big["usage"]["output_tokens"], i64::MAX);
}

#[test]
fn response_usage_precision_rejects_invalid_counters() {
    let build = |usage: Value| {
        completion_with(
            json!({"role": "assistant", "content": "x"}),
            json!("stop"),
            usage,
        )
    };
    for usage in [
        json!({"prompt_tokens": -1, "completion_tokens": 1}),
        json!({"prompt_tokens": 1.5, "completion_tokens": 1}),
        json!({"prompt_tokens": "5", "completion_tokens": 1}),
        json!({"prompt_tokens": 9223372036854775808u64, "completion_tokens": 1}),
        json!(5),
    ] {
        let error =
            unary_final(&build(usage.clone())).expect_err("invalid usage counter must fail closed");
        assert!(error.contains("usage"), "{error} for {usage}");
    }
}

#[test]
fn response_reasoning_unary_thinking_before_text() {
    let out = unary_final(&completion_with(
        json!({"role": "assistant", "reasoning_content": "because", "content": "so"}),
        json!("stop"),
        json!({"prompt_tokens": 1, "completion_tokens": 2}),
    ))
    .unwrap();
    assert_eq!(out["content"][0]["type"], "thinking", "{out}");
    assert_eq!(out["content"][0]["thinking"], "because");
    assert!(out["content"][0].get("signature").is_none(), "{out}");
    assert_eq!(out["content"][1], json!({"type": "text", "text": "so"}));
}

#[test]
fn response_reasoning_null_and_empty_are_absent() {
    for reasoning in [json!(null), json!("")] {
        let mut message = json!({"role": "assistant", "content": "so"});
        message["reasoning_content"] = reasoning;
        let out = unary_final(&completion_with(
            message,
            json!("stop"),
            json!({"prompt_tokens": 1, "completion_tokens": 2}),
        ))
        .unwrap();
        assert!(
            out["content"]
                .as_array()
                .unwrap()
                .iter()
                .all(|block| block["type"] != "thinking"),
            "{out}",
        );
    }
}

#[test]
fn response_reasoning_non_string_fails_closed() {
    for reasoning in [json!(42), json!(true), json!({}), json!([])] {
        let mut message = json!({"role": "assistant", "content": "so"});
        message["reasoning_content"] = reasoning;
        let error = unary_final(&completion_with(
            message,
            json!("stop"),
            json!({"prompt_tokens": 1, "completion_tokens": 1}),
        ))
        .expect_err("non-string reasoning_content must fail closed");
        assert!(error.contains("reasoning"), "{error}");
    }
}

#[test]
fn response_reasoning_lone_alias_accepted() {
    let out = unary_final(&completion_with(
        json!({"role": "assistant", "reasoning": "why", "content": "so"}),
        json!("stop"),
        json!({"prompt_tokens": 1, "completion_tokens": 2}),
    ))
    .unwrap();
    assert_eq!(
        out["content"][0],
        json!({"type": "thinking", "thinking": "why"})
    );
}

#[test]
fn response_reasoning_alias_conflict_fails_closed() {
    let error = unary_final(&completion_with(
        json!({"role": "assistant", "reasoning_content": "a", "reasoning": "b"}),
        json!("stop"),
        json!({"prompt_tokens": 1, "completion_tokens": 1}),
    ))
    .expect_err("conflicting reasoning aliases must fail closed");
    assert!(error.contains("reasoning"), "{error}");
}

#[test]
fn response_reasoning_signed_representation_fails_closed() {
    for field in [
        "signature",
        "reasoning_signature",
        "redacted_reasoning",
        "encrypted_reasoning",
    ] {
        let mut message = json!({"role": "assistant", "reasoning_content": "x", "content": "so"});
        message[field] = json!("opaque-blob");
        let error = unary_final(&completion_with(
            message,
            json!("stop"),
            json!({"prompt_tokens": 1, "completion_tokens": 1}),
        ))
        .expect_err("signed or redacted reasoning must fail closed");
        assert!(error.contains(field), "{error}");
    }
}

#[test]
fn response_embedded_error_object_surfaces_provider_error() {
    let mut machine = OpenAiChatSseMachine::new_for_upstream("gpt-5");
    let chunk = json!({
        "error": {"message": "backend exploded", "metadata": {"request_id": "req_42"}},
    });
    let events = machine.process_chunk_checked(&chunk).unwrap();
    let error = events.iter().find(|event| event.event == "error").unwrap();
    assert_eq!(error.data["error"]["message"], "OpenAI Chat backend error");
    assert!(error.data["error"].get("request_id").is_none());
    assert!(
        machine.transport_close_checked().is_err(),
        "embedded provider error must never close as success",
    );
}

#[test]
fn response_embedded_error_request_id_allowlist() {
    let run = |error: Value| {
        let mut machine = OpenAiChatSseMachine::new_for_upstream("gpt-5");
        machine
            .process_chunk_checked(&json!({"error": error}))
            .unwrap()
            .into_iter()
            .find(|event| event.event == "error")
            .unwrap()
            .data
    };
    let forwarded = run(json!({"message": "boom", "metadata": {"request_id": "req_42"}}));
    assert!(
        forwarded["error"].get("request_id").is_none(),
        "body metadata is not a header allowlist: {forwarded}"
    );
    let dropped = run(json!({
        "message": "boom",
        "metadata": {"internal-trace-id": "trace-abc"},
    }));
    assert!(dropped["error"].get("request_id").is_none(), "{dropped}");
    let dropped = run(json!({"message": "boom", "request-id": "bad id\n"}));
    assert!(dropped["error"].get("request_id").is_none(), "{dropped}");
    let dropped = run(json!({
        "message": "boom",
        "metadata": {"request_id": "x".repeat(129)},
    }));
    assert!(dropped["error"].get("request_id").is_none(), "{dropped}");
}

#[test]
fn response_usage_only_trailing_chunk_accepted() {
    let events = stream_events(&[
        delta_chunk(json!({"content": "hi"}), json!(null)),
        delta_chunk(json!({}), json!("stop")),
        json!({"choices": [], "usage": {"prompt_tokens": 7, "completion_tokens": 2}}),
    ]);
    assert_eq!(error_count(&events), 0, "{events:?}");
    assert_eq!(stop_count(&events), 1, "{events:?}");
    let delta = events
        .iter()
        .find(|(event, _)| event == "message_delta")
        .map(|(_, data)| data)
        .unwrap();
    assert_eq!(
        delta["usage"],
        json!({"input_tokens": 7, "output_tokens": 2})
    );
}

#[test]
fn response_usage_only_chunk_rejected_before_finish() {
    let events = stream_events(&[
        json!({"choices": [], "usage": null}),
        delta_chunk(json!({"content": "hi"}), json!("stop")),
    ]);
    assert_eq!(error_count(&events), 1, "{events:?}");
    assert_eq!(stop_count(&events), 0, "{events:?}");
}

#[test]
fn response_usage_only_chunk_with_payload_rejected() {
    for payload in [
        json!({"content": "late"}),
        json!({"reasoning_content": "late"}),
        json!({"tool_calls": []}),
    ] {
        let events = stream_events(&[
            delta_chunk(json!({"content": "hi"}), json!("stop")),
            json!({"choices": [{"index": 0, "delta": payload, "finish_reason": null}]}),
        ]);
        assert_eq!(error_count(&events), 1, "{events:?}");
        assert_eq!(stop_count(&events), 0, "{events:?}");
    }
}

#[test]
fn response_second_usage_only_chunk_rejected() {
    let events = stream_events(&[
        delta_chunk(json!({"content": "hi"}), json!("stop")),
        json!({"choices": [], "usage": {"prompt_tokens": 1, "completion_tokens": 1}}),
        json!({"choices": [], "usage": {"prompt_tokens": 2, "completion_tokens": 2}}),
    ]);
    assert_eq!(error_count(&events), 1, "{events:?}");
    assert_eq!(stop_count(&events), 0, "{events:?}");
}

#[test]
fn response_stream_missing_finish_fails_closed() {
    let events = stream_events(&[delta_chunk(json!({"content": "partial"}), json!(null))]);
    assert_eq!(error_count(&events), 1, "{events:?}");
    assert_eq!(stop_count(&events), 0, "{events:?}");
}

#[test]
fn response_error_after_text_emits_single_terminal() {
    let events = stream_events(&[
        delta_chunk(json!({"content": "hel"}), json!(null)),
        delta_chunk(json!({"content": "lo"}), json!(null)),
        json!({"error": {"message": "midstream failure"}}),
    ]);
    assert_eq!(error_count(&events), 1, "{events:?}");
    assert_eq!(stop_count(&events), 0, "{events:?}");
    let text_index = events
        .iter()
        .position(|(event, _)| event == "content_block_delta")
        .unwrap();
    let error_index = events
        .iter()
        .position(|(event, _)| event == "error")
        .unwrap();
    assert!(
        text_index < error_index,
        "delivered text must precede the error: {events:?}",
    );
}

#[test]
fn response_after_success_no_more_chunks() {
    let mut machine = OpenAiChatSseMachine::new_streaming_for_upstream("gpt-5");
    machine
        .process_chunk_checked(&delta_chunk(json!({"content": "hi"}), json!("stop")))
        .unwrap();
    machine.transport_close_checked().unwrap();
    let error = machine
        .process_chunk_checked(&delta_chunk(json!({"content": "late"}), json!(null)))
        .expect_err("chunks after the terminal must fail closed");
    assert!(!error.to_string().is_empty());
}

#[test]
fn response_stream_reasoning_order_within_and_across_chunks() {
    let events = stream_events(&[
        delta_chunk(json!({"reasoning_content": "why"}), json!(null)),
        delta_chunk(
            json!({"reasoning_content": "more", "content": "so"}),
            json!(null),
        ),
        delta_chunk(json!({}), json!("stop")),
    ]);
    assert_eq!(error_count(&events), 0, "{events:?}");
    let starts: Vec<&Value> = events
        .iter()
        .filter(|(event, _)| event == "content_block_start")
        .map(|(_, data)| data)
        .collect();
    assert_eq!(starts[0]["content_block"]["type"], "thinking", "{events:?}");
    assert_eq!(starts[1]["content_block"]["type"], "text", "{events:?}");
    let deltas: Vec<String> = events
        .iter()
        .filter(|(event, _)| event == "content_block_delta")
        .map(|(_, data)| {
            format!(
                "{}/{}",
                data["delta"]["thinking"].as_str().unwrap_or(""),
                data["delta"]["text"].as_str().unwrap_or(""),
            )
        })
        .collect();
    assert_eq!(
        deltas,
        vec!["why/".to_string(), "more/".to_string(), "/so".to_string()],
        "{events:?}",
    );
    assert_eq!(stop_count(&events), 1);
}

#[test]
fn response_stream_reasoning_conflict_fails_closed() {
    let events = stream_events(&[
        delta_chunk(
            json!({"reasoning_content": "a", "reasoning": "b"}),
            json!(null),
        ),
        delta_chunk(json!({}), json!("stop")),
    ]);
    assert_eq!(error_count(&events), 1, "{events:?}");
    assert_eq!(stop_count(&events), 0, "{events:?}");
}
