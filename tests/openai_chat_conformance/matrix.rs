use super::*;

// Complements AI-SPEC 3/8 with a complete tool-heavy wire round trip.
#[tokio::test]
async fn openai_chat_matrix_tool_heavy_unary_roundtrip() {
    assert!(can_bind_loopback());
    let _lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let backend = MockServer::start().await;
    let response = json!({"choices":[{"message":{"tool_calls":[{"id":"new_a","type":"function","function":{"name":"f","arguments":"{\"v\":1}"}},{"id":"new_b","type":"function","function":{"name":"f","arguments":"{\"v\":2}"}}]},"finish_reason":"tool_calls"}]});
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&backend)
        .await;
    let gateway = start_gateway(single_provider_config(&backend.uri())).await;
    let request = json!({"model":"claude-via-chat","max_tokens":32,"tools":[{"name":"f","input_schema":{"type":"object"}}],"tool_choice":{"type":"any"},"messages":[
        {"role":"user","content":"run both"},
        {"role":"assistant","content":[{"type":"tool_use","id":"old_a","name":"f","input":{"v":0}},{"type":"tool_use","id":"old_b","name":"f","input":{"v":1}}]},
        {"role":"user","content":[{"type":"tool_result","tool_use_id":"old_a","content":"a"},{"type":"tool_result","tool_use_id":"old_b","content":"b"}]}
    ]});
    let response = reqwest::Client::new()
        .post(format!("{}/v1/messages", gateway.base_url))
        .json(&request)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["stop_reason"], "tool_use");
    assert_eq!(body["content"][0]["id"], "new_a");
    assert_eq!(body["content"][1]["input"], json!({"v":2}));
    let requests = backend.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    let payload: Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert_eq!(payload["tool_choice"], "required");
    assert_eq!(payload["tools"][0]["function"]["name"], "f");
    assert_eq!(payload["messages"][1]["tool_calls"][0]["id"], "old_a");
    assert_eq!(payload["messages"][2]["tool_call_id"], "old_a");
    assert_eq!(payload["messages"][3]["tool_call_id"], "old_b");
}

#[tokio::test]
async fn openai_chat_matrix_image_wire_mapping() {
    assert!(can_bind_loopback());
    let _lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    const PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jU8cAAAAASUVORK5CYII=";
    let backend = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(chat_completion_upstream()))
        .mount(&backend)
        .await;
    let gateway = start_gateway(single_provider_config(&backend.uri())).await;
    let response = reqwest::Client::new().post(format!("{}/v1/messages",gateway.base_url)).json(&json!({"model":"claude-via-chat","messages":[{"role":"user","content":[{"type":"image","source":{"type":"base64","media_type":"image/png","data":PNG}}]}]})).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let requests = backend.received_requests().await.unwrap();
    let payload: Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert_eq!(
        payload["messages"][0]["content"][0]["image_url"]["url"],
        format!("data:image/png;base64,{PNG}")
    );
}

async fn aggregate_wire(size: usize) {
    let _lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let mut frames = Vec::new();
    let mut remaining = size;
    while remaining > 0 {
        let length = remaining.min(512 * 1024);
        frames.push(chat_delta(json!({"content":"x".repeat(length)}), None));
        remaining -= length;
    }
    frames.push(chat_delta(json!({}), Some("stop")));
    frames.push("[DONE]".into());
    let events = openai_chat_terminal_stream_events(frames).await;
    let excess = size > 8 * 1024 * 1024;
    assert_eq!(
        events.iter().filter(|(name, _)| name == "error").count(),
        usize::from(excess)
    );
    assert_eq!(
        events
            .iter()
            .filter(|(name, _)| name == "message_stop")
            .count(),
        usize::from(!excess)
    );
    let delivered: usize = events
        .iter()
        .filter_map(|(_, data)| data["delta"]["text"].as_str().map(str::len))
        .sum();
    assert_eq!(delivered, size.min(8 * 1024 * 1024));
}
#[tokio::test]
async fn openai_chat_cap_aggregate_minus_one_wire() {
    aggregate_wire(8 * 1024 * 1024 - 1).await;
}
#[tokio::test]
async fn openai_chat_cap_aggregate_at_wire() {
    aggregate_wire(8 * 1024 * 1024).await;
}
#[tokio::test]
async fn openai_chat_cap_aggregate_plus_one_wire() {
    aggregate_wire(8 * 1024 * 1024 + 1).await;
}

// AI-SPEC 9 also names the adapter's separate 32-MiB unary collection bound.
async fn unary_wire(size: usize) {
    let _lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let mut value = chat_completion_upstream();
    value["padding"] = json!("");
    let overhead = value.to_string().len();
    value["padding"] = json!("x".repeat(size - overhead));
    let body = value.to_string();
    assert_eq!(body.len(), size);
    let backend = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
        .mount(&backend)
        .await;
    let gateway = start_gateway(single_provider_config(&backend.uri())).await;
    let response = reqwest::Client::new()
        .post(format!("{}/v1/messages", gateway.base_url))
        .body(anthropic_request("claude-via-chat"))
        .send()
        .await
        .unwrap();
    if size > 32 * 1024 * 1024 {
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        assert_eq!(response.json::<Value>().await.unwrap()["type"], "error");
    } else {
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.json::<Value>().await.unwrap()["content"][0]["text"],
            "fixture reply"
        );
    }
}
#[tokio::test]
async fn openai_chat_cap_unary_minus_one_wire() {
    unary_wire(32 * 1024 * 1024 - 1).await;
}
#[tokio::test]
async fn openai_chat_cap_unary_at_wire() {
    unary_wire(32 * 1024 * 1024).await;
}
#[tokio::test]
async fn openai_chat_cap_unary_plus_one_wire() {
    unary_wire(32 * 1024 * 1024 + 1).await;
}
