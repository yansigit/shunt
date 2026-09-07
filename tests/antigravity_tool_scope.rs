use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde_json::{json, Value};
use shunt::model::{
    antigravity_request::{antigravity_scoped_session_id, AntigravityToolContext},
    gemini::GeminiSseMachine,
    gemini_request::translate_request_for_model_with_context,
};

fn opening() -> Value {
    json!({"messages":[{"role":"user","content":"private opening"}]})
}

fn scope(account: &str, request: &Value) -> AntigravityToolContext {
    AntigravityToolContext::new(account, antigravity_scoped_session_id(account, request))
        .with_history(request)
}

#[test]
fn antigravity_native_envelope_session_survives_history_growth() {
    let request = opening();
    let initial = scope("private-account", &request);
    let mut followup = request.clone();
    followup["messages"].as_array_mut().unwrap().extend([
        json!({"role":"assistant","content":"answer"}),
        json!({"role":"user","content":"next question"}),
    ]);
    followup["temperature"] = json!(0.5);
    assert_eq!(initial.session, scope("private-account", &followup).session);
    assert_ne!(initial.session, scope("other-account", &request).session);
    followup["messages"][0]["content"] = json!("different opening");
    assert_ne!(initial.session, scope("private-account", &followup).session);
}

#[test]
fn antigravity_native_tool_signature_binds_fields_without_exposing_account() {
    let context = scope("private-account", &opening());
    let args = json!({"nested":{"b":2,"a":1}});
    let id = context.encode("authentic upstream signature", "lookup", &args, 0);
    assert_eq!(
        context.decode(&id, "lookup", &args, 0).unwrap().as_deref(),
        Some("authentic upstream signature")
    );
    let decoded = String::from_utf8(
        URL_SAFE_NO_PAD
            .decode(id.strip_prefix("call_antigravity_v2_").unwrap())
            .unwrap(),
    )
    .unwrap();
    assert!(!decoded.contains("private-account"));
    assert!(!decoded.contains("private opening"));
    assert!(!decoded.contains(&context.session));
    assert!(context.decode(&id, "changed", &args, 0).is_err());
    assert!(context.decode(&id, "lookup", &json!({}), 0).is_err());
    assert!(context.decode(&id, "lookup", &args, 1).is_err());
    assert!(scope("other-account", &opening())
        .decode(&id, "lookup", &args, 0)
        .is_err());
    assert!(context
        .decode("call_gemini_v1_c2ln", "lookup", &args, 0)
        .is_err());
    assert!(context
        .decode("call_antigravity_v2_!", "lookup", &args, 0)
        .is_err());
    let reordered: Value = serde_json::from_str(r#"{"nested":{"a":1,"b":2}}"#).unwrap();
    assert_eq!(
        id,
        context.encode("authentic upstream signature", "lookup", &reordered, 0)
    );
}

#[test]
fn antigravity_native_tool_signature_accepts_equivalent_json_numbers_without_rounding_integers() {
    let context = scope("account", &opening());
    for (upstream, replay) in [
        (
            r#"{"numbers":[1.0,-0.0,-2.0,1e18]}"#,
            r#"{"numbers":[1,0,-2,1000000000000000000]}"#,
        ),
        (
            r#"{"n":-9223372036854775808.0}"#,
            r#"{"n":-9223372036854775808}"#,
        ),
        (
            r#"{"n":18446744073709551616.0}"#,
            r#"{"n":18446744073709551616}"#,
        ),
    ] {
        let upstream: Value = serde_json::from_str(upstream).unwrap();
        let replay: Value = serde_json::from_str(replay).unwrap();
        let id = context.encode("signature", "lookup", &upstream, 0);
        assert!(
            context.decode(&id, "lookup", &replay, 0).is_ok(),
            "{upstream} -> {replay}"
        );
    }
    let exact = json!({"n":9_007_199_254_740_993u64});
    let rounded = json!({"n":9_007_199_254_740_992u64});
    let id = context.encode("signature", "lookup", &exact, 0);
    assert!(context.decode(&id, "lookup", &rounded, 0).is_err());
}

#[test]
fn antigravity_native_tool_signature_roundtrips_sequential_history() {
    let mut request = opening();
    for round in 0..2 {
        let context = scope("account", &request);
        assert_eq!(context.output_ordinal, round);
        let mut machine =
            GeminiSseMachine::new("gemini-3.8-flash").with_antigravity_context(context.clone());
        machine
            .process_chunk_checked(&json!({"candidates":[{"content":{"parts":[{
            "functionCall":{"name":"lookup","args":{"round":round}},
            "thoughtSignature":format!("signature-{round}")
        }]},"finishReason":"STOP"}]}))
            .unwrap();
        machine.transport_close_checked().unwrap();
        let response = machine.final_json_checked().unwrap();
        let tool = response["content"][0].clone();
        let messages = request["messages"].as_array_mut().unwrap();
        messages.push(json!({"role":"assistant","content":[tool.clone()]}));
        messages.push(json!({"role":"user","content":[{"type":"tool_result","tool_use_id":tool["id"],"content":"done"}]}));
        let next_context = scope("account", &request);
        assert_eq!(context.session, next_context.session);
        let translated = translate_request_for_model_with_context(
            &request,
            "gemini-3.8-flash",
            Some(&next_context),
        )
        .unwrap();
        let signatures: Vec<_> = translated["contents"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|content| content["parts"].as_array().unwrap())
            .filter_map(|part| part["thoughtSignature"].as_str())
            .collect();
        assert_eq!(signatures.len(), round + 1);
        assert_eq!(signatures[round], format!("signature-{round}"));
        assert!(translate_request_for_model_with_context(
            &request,
            "gemini-3.8-flash",
            Some(&scope("other", &request))
        )
        .is_err());
        assert!(
            translate_request_for_model_with_context(&request, "gemini-3.8-flash", None).is_err()
        );
    }
}

#[test]
fn antigravity_native_tool_signature_parallel_streaming_matches_unary() {
    let context = scope("account", &opening());
    let signed =
        json!({"functionCall":{"name":"lookup","args":{}},"thoughtSignature":"upstream-signature"});
    let unsigned = json!({"functionCall":{"name":"lookup","args":{}}});
    let mut unary =
        GeminiSseMachine::new("gemini-3.8-flash").with_antigravity_context(context.clone());
    unary.process_chunk_checked(&json!({"candidates":[{"content":{"parts":[signed.clone(), unsigned.clone()]},"finishReason":"STOP"}]})).unwrap();
    unary.transport_close_checked().unwrap();
    let content = unary.final_json_checked().unwrap()["content"].clone();
    let mut streaming = GeminiSseMachine::new_streaming("gemini-3.8-flash")
        .with_antigravity_context(context.clone());
    let mut events = streaming
        .process_chunk_checked(&json!({"candidates":[{"content":{"parts":[signed]}}]}))
        .unwrap();
    events.extend(
        streaming
            .process_chunk_checked(
                &json!({"candidates":[{"content":{"parts":[unsigned]},"finishReason":"STOP"}]}),
            )
            .unwrap(),
    );
    events.extend(streaming.transport_close_checked().unwrap());
    let streamed_ids: Vec<_> = events
        .iter()
        .filter_map(|event| {
            event
                .data
                .pointer("/content_block/id")
                .and_then(Value::as_str)
        })
        .collect();
    let unary_ids: Vec<_> = content
        .as_array()
        .unwrap()
        .iter()
        .map(|block| block["id"].as_str().unwrap())
        .collect();
    assert_eq!(streamed_ids, unary_ids);
    assert_ne!(unary_ids[0], unary_ids[1]);
    assert_eq!(
        context
            .decode(unary_ids[1], "lookup", &json!({}), 1)
            .unwrap(),
        None
    );
    let mut request = opening();
    request["messages"].as_array_mut().unwrap().extend([
        json!({"role":"assistant","content":content}),
        json!({"role":"user","content":[
            {"type":"tool_result","tool_use_id":unary_ids[1],"content":"second"},
            {"type":"tool_result","tool_use_id":unary_ids[0],"content":"first"}
        ]}),
    ]);
    let translated =
        translate_request_for_model_with_context(&request, "gemini-3.8-flash", Some(&context))
            .unwrap();
    assert_eq!(
        translated["contents"][2]["parts"][0]["functionResponse"]["response"]["output"],
        "first"
    );
    assert_eq!(
        translated["contents"][2]["parts"][1]["functionResponse"]["response"]["output"],
        "second"
    );
}
