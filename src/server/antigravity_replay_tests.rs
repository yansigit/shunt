//! Native 401 recovery through the real router and private credential dependency.
use serde_json::{json, Value};
use std::time::Duration;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};
mod fixtures;
use fixtures::*;

#[tokio::test]
async fn antigravity_native_401_account_swap_never_exchanges_another_accounts_grant() {
    let backend = MockServer::start().await;
    let token_server = MockServer::start().await;
    mount_catalog(&backend).await;
    let gateway = ReplayGateway::start(
        &backend,
        &token_server,
        "pre-401-token",
        Some("synthetic-user@example.com"),
        "",
    )
    .await;
    let mut swapped: Value = serde_json::from_slice(&gateway.credential_bytes).unwrap();
    swapped["email"] = json!("synthetic-other@example.com");
    swapped["refresh_token"] = json!("synthetic-other-refresh");
    let swapped = serde_json::to_vec(&swapped).unwrap();
    let path_to_swap = gateway.credential_path.clone();
    let swap_bytes = swapped.clone();
    Mock::given(method("POST"))
        .and(path("/v1internal:streamGenerateContent"))
        .respond_with(move |_: &wiremock::Request| {
            // Emulate an external login only after the initial resolution.
            std::fs::write(&path_to_swap, &swap_bytes).unwrap();
            ResponseTemplate::new(401)
        })
        .expect(1)
        .mount(&backend)
        .await;
    let response = gateway.post_messages(client_body()).await;
    assert_eq!(response.status(), 401);
    assert!(token_server.received_requests().await.unwrap().is_empty());
    // The only mutation is the explicit fixture login above, never writeback.
    assert_eq!(std::fs::read(&gateway.credential_path).unwrap(), swapped);
}

#[tokio::test]
async fn antigravity_native_401_text_then_auth_error_never_refreshes_or_replays() {
    let backend = MockServer::start().await;
    let token_server = MockServer::start().await;
    mount_catalog(&backend).await;
    Mock::given(method("POST"))
        .and(path("/v1internal:streamGenerateContent"))
        .respond_with(ResponseTemplate::new(200).set_body_string(concat!(
            "data: {\"response\":{\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"synthetic early output\"}]}}]}}\n\n",
            "data: {\"response\":{\"error\":{\"code\":401,\"status\":\"UNAUTHENTICATED\"}}}\n\n"
        ))).expect(1).mount(&backend).await;
    let gateway = ReplayGateway::start(
        &backend,
        &token_server,
        "pre-401-token",
        Some("synthetic-user@example.com"),
        "",
    )
    .await;
    let mut request = client_body();
    request["stream"] = json!(true);
    let text = gateway.post_messages(request).await.text().await.unwrap();
    assert!(text.contains("synthetic early output"), "{text}");
    assert_eq!(text.matches("event: error").count(), 1, "{text}");
    assert!(!text.contains("event: message_stop"), "{text}");
    assert!(token_server.received_requests().await.unwrap().is_empty());
    gateway.assert_credential_file_unchanged();
}

#[tokio::test]
async fn antigravity_native_401_first_preheader_401_refreshes_same_account_and_replays_once() {
    for streaming in [false, true] {
        let backend = MockServer::start().await;
        let token_server = MockServer::start().await;
        mount_catalog(&backend).await;
        mount_bearer_keyed_inference(&backend, 401, 1, 200, 1).await;
        mount_token_success(&token_server).await;

        let gateway = ReplayGateway::start(
            &backend,
            &token_server,
            "pre-401-token",
            Some("synthetic-user@example.com"),
            "",
        )
        .await;

        let mut request = client_body();
        request["stream"] = json!(streaming);
        let response = gateway.post_messages(request).await;
        let status = response.status();
        let text = response.text().await.unwrap();
        assert_eq!(status, 200, "{text}");
        if streaming {
            assert_eq!(text.matches("event: message_stop").count(), 1, "{text}");
            assert!(!text.contains("event: error"), "{text}");
        } else {
            assert_eq!(
                serde_json::from_str::<Value>(&text).unwrap()["stop_reason"],
                "end_turn"
            );
        }
        assert!(!text.contains("pre-401-token") && !text.contains("refreshed-token"));

        let inference = backend
            .received_requests()
            .await
            .unwrap()
            .into_iter()
            .filter(|request| request.url.path() == "/v1internal:streamGenerateContent")
            .collect::<Vec<_>>();
        assert_eq!(inference.len(), 2, "exactly one pre-commit replay");
        let first: Value = serde_json::from_slice(&inference[0].body).unwrap();
        let second: Value = serde_json::from_slice(&inference[1].body).unwrap();
        // Same account's project, session, request identity, and payload — only
        // the bearer binding changed between the two attempts.
        assert_eq!(first, second);
        assert_eq!(
            inference[0].body, inference[1].body,
            "wire bytes stay identical"
        );
        assert_eq!(first["project"], "proj-401");
        assert_eq!(
            first["request"]["sessionId"],
            second["request"]["sessionId"]
        );

        assert_eq!(
            token_server
                .received_requests()
                .await
                .unwrap()
                .into_iter()
                .filter(|request| request.url.path() == "/token")
                .count(),
            1,
            "the refresh endpoint must be hit exactly once"
        );
        gateway.assert_credential_file_unchanged();
        backend.verify().await;
        drop(gateway);
    }
}

#[tokio::test]
async fn antigravity_native_401_second_401_terminates_without_a_third_attempt() {
    let backend = MockServer::start().await;
    let token_server = MockServer::start().await;
    mount_catalog(&backend).await;
    mount_bearer_keyed_inference(&backend, 401, 1, 401, 1).await;
    mount_token_success(&token_server).await;

    let gateway = ReplayGateway::start(
        &backend,
        &token_server,
        "pre-401-token",
        Some("synthetic-user@example.com"),
        "",
    )
    .await;

    let response = gateway.post_messages(client_body()).await;
    assert_eq!(response.status(), 401, "{:?}", response.text().await);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["error"]["type"], "api_error");

    let inference = backend
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .filter(|request| request.url.path() == "/v1internal:streamGenerateContent")
        .count();
    assert_eq!(inference, 2, "a second 401 must terminate, not replay");
    gateway.assert_credential_file_unchanged();
    backend.verify().await;
    drop(gateway);
}

#[tokio::test]
async fn antigravity_native_401_refresh_failure_terminates_without_replay() {
    let backend = MockServer::start().await;
    let token_server = MockServer::start().await;
    mount_catalog(&backend).await;
    mount_bearer_keyed_inference(&backend, 401, 1, 200, 0).await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "invalid_grant"
        })))
        .expect(1)
        .mount(&token_server)
        .await;

    let gateway = ReplayGateway::start(
        &backend,
        &token_server,
        "pre-401-token",
        Some("synthetic-user@example.com"),
        "",
    )
    .await;

    let response = gateway.post_messages(client_body()).await;
    assert!(
        !response.status().is_success(),
        "{:?}",
        response.text().await
    );
    let inference = backend
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .filter(|request| request.url.path() == "/v1internal:streamGenerateContent")
        .count();
    assert_eq!(inference, 1, "a failed refresh must never replay");
    gateway.assert_credential_file_unchanged();
    backend.verify().await;
    drop(gateway);
}

#[tokio::test]
async fn antigravity_native_401_refresh_rotating_a_legacy_account_fails_closed() {
    // A legacy store keys its identity on the refresh token. A token endpoint
    // that rotates the grant would change the account fingerprint, pairing an
    // old project/session with a new identity — so the replay must fail
    // closed before the second attempt instead.
    let backend = MockServer::start().await;
    let token_server = MockServer::start().await;
    mount_catalog(&backend).await;
    mount_bearer_keyed_inference(&backend, 401, 1, 200, 0).await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "refreshed-token",
            "refresh_token": "rotated-refresh",
            "expires_in": 3600
        })))
        .expect(1)
        .mount(&token_server)
        .await;

    let gateway = ReplayGateway::start(&backend, &token_server, "pre-401-token", None, "").await;

    let response = gateway.post_messages(client_body()).await;
    assert!(
        !response.status().is_success(),
        "{:?}",
        response.text().await
    );
    let inference = backend
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .filter(|request| request.url.path() == "/v1internal:streamGenerateContent")
        .count();
    assert_eq!(inference, 1, "an identity-changing refresh must not replay");
    gateway.assert_credential_file_unchanged();
    backend.verify().await;
    drop(gateway);
}

#[tokio::test]
async fn antigravity_native_401_success_response_never_replays() {
    let backend = MockServer::start().await;
    let token_server = MockServer::start().await;
    mount_catalog(&backend).await;
    Mock::given(method("POST"))
        .and(path("/v1internal:streamGenerateContent"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            "data: {\"response\":{\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"OK\"}]},\"finishReason\":\"STOP\"}]}}\n\ndata: [DONE]\n\n",
        ))
        .expect(1)
        .mount(&backend)
        .await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&token_server)
        .await;

    let gateway = ReplayGateway::start(
        &backend,
        &token_server,
        "pre-401-token",
        Some("synthetic-user@example.com"),
        "",
    )
    .await;

    let response = gateway.post_messages(client_body()).await;
    assert_eq!(response.status(), 200, "{:?}", response.text().await);
    gateway.assert_credential_file_unchanged();
    backend.verify().await;
    drop(gateway);
}

#[tokio::test]
async fn antigravity_native_401_body_parse_failure_terminates_without_replay() {
    let backend = MockServer::start().await;
    let token_server = MockServer::start().await;
    mount_catalog(&backend).await;
    Mock::given(method("POST"))
        .and(path("/v1internal:streamGenerateContent"))
        .respond_with(ResponseTemplate::new(200).set_body_string("data: {broken"))
        .expect(1)
        .mount(&backend)
        .await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&token_server)
        .await;

    let gateway = ReplayGateway::start(
        &backend,
        &token_server,
        "pre-401-token",
        Some("synthetic-user@example.com"),
        "",
    )
    .await;

    let response = gateway.post_messages(client_body()).await;
    assert_eq!(response.status(), 502, "{:?}", response.text().await);
    let inference = backend
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .filter(|request| request.url.path() == "/v1internal:streamGenerateContent")
        .count();
    assert_eq!(inference, 1, "a body failure must never replay");
    gateway.assert_credential_file_unchanged();
    drop(gateway);
}

#[tokio::test]
async fn antigravity_native_401_tool_output_boundary_terminates_without_replay() {
    let backend = MockServer::start().await;
    let token_server = MockServer::start().await;
    mount_catalog(&backend).await;
    // A successful response that streams a tool call: once this reaches the
    // client the attempt is committed, so no later 401 logic may ever apply.
    Mock::given(method("POST"))
        .and(path("/v1internal:streamGenerateContent"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            "data: {\"response\":{\"candidates\":[{\"content\":{\"parts\":[{\"functionCall\":{\"name\":\"lookup\",\"args\":{}},\"thoughtSignature\":\"synthetic-signature\"}]}}]}}\n\ndata: {\"response\":{\"error\":{\"code\":401,\"status\":\"UNAUTHENTICATED\",\"message\":\"synthetic midstream failure\"}}}\n\n",
        ))
        .expect(1)
        .mount(&backend)
        .await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&token_server)
        .await;

    let gateway = ReplayGateway::start(
        &backend,
        &token_server,
        "pre-401-token",
        Some("synthetic-user@example.com"),
        "",
    )
    .await;

    let mut body = client_body();
    body["stream"] = json!(true);
    let response = gateway.post_messages(body).await;
    assert_eq!(response.status(), 200, "{:?}", response.text().await);
    let text = response.text().await.unwrap();
    assert!(
        text.contains("tool_use"),
        "the tool must actually escape: {text}"
    );
    assert_eq!(text.matches("event: error").count(), 1, "{text}");
    assert_eq!(text.matches("event: message_stop").count(), 0, "{text}");
    let inference = backend
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .filter(|request| request.url.path() == "/v1internal:streamGenerateContent")
        .count();
    assert_eq!(inference, 1, "committed tool output must never replay");
    gateway.assert_credential_file_unchanged();
    drop(gateway);
}

#[tokio::test]
async fn antigravity_native_401_ambiguous_send_timeout_terminates_without_replay() {
    let backend = MockServer::start().await;
    let token_server = MockServer::start().await;
    mount_catalog(&backend).await;
    Mock::given(method("POST"))
        .and(path("/v1internal:streamGenerateContent"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("data: {}\n\n")
                .set_delay(Duration::from_secs(30)),
        )
        .mount(&backend)
        .await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&token_server)
        .await;

    let gateway = ReplayGateway::start(
        &backend,
        &token_server,
        "pre-401-token",
        Some("synthetic-user@example.com"),
        "\n[server.timeouts]\nupstream_ttfb_ms = 2000\n",
    )
    .await;

    let response = gateway.post_messages(client_body()).await;
    assert_eq!(response.status(), 504, "{:?}", response.text().await);
    let inference = backend
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .filter(|request| request.url.path() == "/v1internal:streamGenerateContent")
        .count();
    assert_eq!(inference, 1, "an ambiguous send must never replay");
    gateway.assert_credential_file_unchanged();
    drop(gateway);
}
