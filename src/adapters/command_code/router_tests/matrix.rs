//! Synthetic full scenarios over the canonical-host TLS gateway harness.
use super::*;

const FINISH: &str = "{\"type\":\"text-delta\",\"text\":\"done\"}\n{\"type\":\"finish\",\"finishReason\":\"stop\"}\n";

fn request(streaming: bool) -> Value {
    json!({"model":"zai-org/GLM-5.3","stream":streaming,
        "messages":[{"role":"user","content":""}]})
}

#[tokio::test]
async fn command_code_matrix_subagent_continuation_and_tool_heavy() {
    let _lock = crate::config::CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _key = EnvGuard::set(Some("synthetic-subscription-token"));
    for count in [1, 32] {
        let calls: Vec<Value> = (0..count).map(|i| json!({"type":"tool_use","id":format!("agent-{i}"),"name":"Task","input":{"prompt":"inspect fixture"}})).collect();
        let results: Vec<Value> = (0..count).map(|i| json!({"type":"tool_result","tool_use_id":format!("agent-{i}"),"content":format!("subagent-{i} complete")})).collect();
        let mut body = request(false);
        body["tools"] = json!([{"name":"Task","description":"Delegate a plaintext task","input_schema":{"type":"object","properties":{"prompt":{"type":"string"}}}}]);
        body["messages"] = json!([
            {"role":"user","content":"inspect this fixture"},
            {"role":"assistant","content":calls},
            {"role":"user","content":results},
            {"role":"assistant","content":"analysis complete"},
            {"role":"user","content":"continue from the recorded results"}
        ]);
        for streaming in [false, true] {
            body["stream"] = json!(streaming);
            let (status, output, captured) = turn_request(FINISH, 200, body.clone(), None).await;
            assert_eq!(status, reqwest::StatusCode::OK);
            assert!(output.contains("done"));
            let messages = captured["params"]["messages"].as_array().unwrap();
            let tool_results: Vec<&Value> =
                messages.iter().filter(|m| m["role"] == "tool").collect();
            assert_eq!(tool_results.len(), count);
            for (i, result) in tool_results.iter().enumerate() {
                assert_eq!(result["content"][0]["toolCallId"], format!("agent-{i}"));
                assert_eq!(
                    result["content"][0]["output"]["value"],
                    format!("subagent-{i} complete")
                );
            }
            assert_eq!(
                messages.last().unwrap()["content"][0]["text"],
                "continue from the recorded results"
            );
        }
    }
}

#[tokio::test]
async fn command_code_matrix_request_bytes_below_at_above() {
    let _lock = crate::config::CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _key = EnvGuard::set(Some("synthetic-subscription-token"));
    const CAP: usize = 8192;
    for streaming in [false, true] {
        for size in [CAP - 1, CAP] {
            let mut body = request(streaming);
            body["messages"][0]["content"] = json!("x".repeat(size - body.to_string().len()));
            assert_eq!(body.to_string().len(), size);
            let (status, _, captured) = turn_request(FINISH, 200, body.clone(), Some(CAP)).await;
            assert_eq!(status, reqwest::StatusCode::OK);
            assert_eq!(
                captured["params"]["messages"][0]["content"][0]["text"],
                body["messages"][0]["content"]
            );
        }
        let before = crate::auth::command_code::LOOKUPS.load(std::sync::atomic::Ordering::SeqCst);
        let mut cfg = config("https://api.commandcode.ai");
        cfg.server.limits.max_request_bytes = CAP;
        let trap = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let client = reqwest::Client::builder()
            .no_proxy()
            .resolve("api.commandcode.ai", trap.local_addr().unwrap())
            .build()
            .unwrap();
        let (router, _, _) = crate::server::build_router_with_test_client(cfg, client).unwrap();
        let mut body = request(streaming);
        body["messages"][0]["content"] = json!("x".repeat(CAP + 1 - body.to_string().len()));
        assert_eq!(body.to_string().len(), CAP + 1);
        use tower::ServiceExt;
        let response = router
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/v1/messages")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(
            crate::auth::command_code::LOOKUPS.load(std::sync::atomic::Ordering::SeqCst),
            before
        );
        assert!(
            tokio::time::timeout(Duration::from_millis(100), trap.accept())
                .await
                .is_err()
        );
    }
}

#[tokio::test]
async fn command_code_matrix_failed_finish_keeps_usage_and_status_errors_are_neutral() {
    let _lock = crate::config::CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _key = EnvGuard::set(Some("synthetic-subscription-token"));
    let wire = "{\"type\":\"finish\",\"finishReason\":\"error\",\"usage\":{\"inputTokens\":3,\"outputTokens\":4}}\n";
    let (status, error) = turn(wire, 200).await;
    assert_eq!(status, reqwest::StatusCode::BAD_GATEWAY);
    assert_eq!(error["usage"]["input_tokens"], 3);
    assert_eq!(error["usage"]["output_tokens"], 4);
    for status in [401, 403, 429, 500] {
        let (actual, body) = turn("synthetic-private-upstream-error", status).await;
        assert_eq!(actual.as_u16(), status);
        assert_eq!(body["type"], "error");
        assert!(!body
            .to_string()
            .contains("synthetic-private-upstream-error"));
    }
}
