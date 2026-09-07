use std::{convert::Infallible, io::ErrorKind, net::SocketAddr, sync::Arc};

use axum::{body::Body, extract::State, routing::post, Router};
use futures_util::{stream, StreamExt};
use reqwest::StatusCode;
use serde_json::json;
use shunt::{
    config::{AuthMode, Config, RetryConfig, RouteConfig},
    server,
};
use tokio::{
    sync::{mpsc, Notify},
    task::JoinHandle,
};
use wiremock::{
    matchers::{method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

struct Gateway {
    base_url: String,
    task: JoinHandle<()>,
}

impl Drop for Gateway {
    fn drop(&mut self) {
        self.task.abort();
    }
}

fn can_bind_loopback() -> bool {
    match std::net::TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => {
            drop(listener);
            true
        }
        Err(error) if error.kind() == ErrorKind::PermissionDenied => false,
        Err(error) => panic!("unexpected loopback bind failure: {error}"),
    }
}

fn gemini_config_with_retry(base_url: String, max_retries: u32) -> Config {
    let mut config = Config::default();
    let provider = config.providers.get_mut("gemini").unwrap();
    provider.base_url = base_url;
    provider.auth = AuthMode::ApiKey;
    provider.api_key_env = Some("SHUNT_GEMINI_CONFORMANCE_KEY".to_string());
    provider.retry = RetryConfig {
        max_retries,
        initial_backoff_ms: 1,
        max_backoff_ms: 1,
        ..RetryConfig::default()
    };
    config.server.default_provider = "gemini".to_string();
    config.routes = vec![RouteConfig {
        model: "claude-via-gemini".to_string(),
        provider: "gemini".to_string(),
        upstream_model: Some("gemini-2.5-pro".to_string()),
        effort: None,
        service_tier: None,
    }];
    config
}

fn gemini_config(base_url: String) -> Config {
    gemini_config_with_retry(base_url, 0)
}

async fn start_gateway(base_url: String) -> Gateway {
    start_gateway_with_config(gemini_config(base_url)).await
}

async fn start_gateway_with_config(mut config: Config) -> Gateway {
    std::env::set_var("SHUNT_GEMINI_CONFORMANCE_KEY", "fixture-key");
    config.server.bind = "127.0.0.1:0".to_string();
    let listener = tokio::net::TcpListener::bind(config.server.bind_addr().unwrap())
        .await
        .unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    let (app, _, _) = server::build_router(config).unwrap();
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    Gateway {
        base_url: format!("http://{addr}"),
        task,
    }
}

#[tokio::test]
async fn gemini_identity_retry_never_retries_returned_transient_statuses() {
    if !can_bind_loopback() {
        return;
    }

    for status in [429, 502, 503, 504, 529] {
        let upstream = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(
                "/v1beta/models/gemini-2.5-pro:generateContent",
            ))
            .respond_with(ResponseTemplate::new(status).set_body_json(json!({
                "error": {"message": "synthetic transient"}
            })))
            .expect(1)
            .mount(&upstream)
            .await;

        let gateway = start_gateway_with_config(gemini_config_with_retry(upstream.uri(), 2)).await;
        let response = reqwest::Client::new()
            .post(format!("{}/v1/messages", gateway.base_url))
            .body(request(false))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status().as_u16(), status);
        let requests = upstream.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1, "status {status} was redispatched");
        assert_eq!(
            requests[0]
                .headers
                .get("x-goog-api-key")
                .and_then(|value| value.to_str().ok()),
            Some("fixture-key")
        );
        assert_eq!(
            requests[0].body_json::<serde_json::Value>().unwrap(),
            json!({
                "contents": [{"role": "user", "parts": [{"text": "fixture"}]}],
                "generationConfig": {"maxOutputTokens": 64}
            })
        );
    }
}

fn request(stream: bool) -> String {
    json!({
        "model": "claude-via-gemini",
        "max_tokens": 64,
        "stream": stream,
        "messages": [{"role": "user", "content": "fixture"}]
    })
    .to_string()
}

async fn streaming_gateway_response(upstream_body: &[u8]) -> String {
    let upstream = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1beta/models/gemini-2.5-pro:streamGenerateContent"))
        .and(query_param("alt", "sse"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_bytes(upstream_body.to_vec()),
        )
        .expect(1)
        .mount(&upstream)
        .await;
    let gateway = start_gateway(upstream.uri()).await;
    let response = reqwest::Client::new()
        .post(format!("{}/v1/messages", gateway.base_url))
        .body(request(true))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    response.text().await.unwrap()
}

async fn unary_gateway_response(upstream_body: Vec<u8>) -> reqwest::Response {
    unary_gateway_response_with_status(200, upstream_body).await
}

async fn unary_gateway_response_with_status(
    upstream_status: u16,
    upstream_body: Vec<u8>,
) -> reqwest::Response {
    let upstream = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1beta/models/gemini-2.5-pro:generateContent"))
        .respond_with(
            ResponseTemplate::new(upstream_status)
                .insert_header("content-type", "application/json")
                .set_body_bytes(upstream_body),
        )
        .expect(1)
        .mount(&upstream)
        .await;
    let gateway = start_gateway(upstream.uri()).await;
    reqwest::Client::new()
        .post(format!("{}/v1/messages", gateway.base_url))
        .body(request(false))
        .send()
        .await
        .unwrap()
}

fn padded_gemini_response(size: usize) -> Vec<u8> {
    let prefix = br#"{"candidates":[{"content":{"parts":[{"text":"ok"}]},"finishReason":"STOP"}],"padding":""#;
    let suffix = br#""}"#;
    assert!(size >= prefix.len() + suffix.len());
    let mut body = Vec::with_capacity(size);
    body.extend_from_slice(prefix);
    body.resize(size - suffix.len(), b'x');
    body.extend_from_slice(suffix);
    assert_eq!(body.len(), size);
    body
}

#[tokio::test]
async fn gemini_streaming_framing_rejects_malformed_json_once() {
    if !can_bind_loopback() {
        return;
    }
    let body = streaming_gateway_response(b"data: not-json\n\n").await;
    assert_eq!(body.matches("event: error").count(), 1, "{body}");
    assert!(!body.contains("event: message_stop"), "{body}");
}

#[tokio::test]
async fn gemini_streaming_framing_rejects_cut_stream_without_synthetic_success() {
    if !can_bind_loopback() {
        return;
    }
    let body = streaming_gateway_response(
        b"data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"partial\"}]}}]}\n\n",
    )
    .await;
    assert!(body.contains("partial"), "{body}");
    assert_eq!(body.matches("event: error").count(), 1, "{body}");
    assert!(!body.contains("event: message_stop"), "{body}");
}

#[tokio::test]
async fn gemini_streaming_framing_accepts_wrapped_crlf_and_authoritative_finish() {
    if !can_bind_loopback() {
        return;
    }
    let body = streaming_gateway_response(concat!(
        ": keepalive\r\n",
        "data: {\"response\":{\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"hello\"}]}}]}}\r\n\r\n",
        "data: {\"response\":{\"candidates\":[{\"finishReason\":\"STOP\"}],\"usageMetadata\":{\"promptTokenCount\":2,\"candidatesTokenCount\":1}}}\r\n\r\n",
    ).as_bytes()).await;
    assert!(body.contains("hello"), "{body}");
    assert_eq!(body.matches("event: message_stop").count(), 1, "{body}");
    assert!(!body.contains("event: error"), "{body}");
}

#[tokio::test]
async fn gemini_streaming_framing_embedded_provider_error_is_terminal() {
    if !can_bind_loopback() {
        return;
    }
    let body = streaming_gateway_response(
        b"data: {\"response\":{\"error\":{\"code\":429,\"status\":\"RESOURCE_EXHAUSTED\",\"message\":\"quota\"}}}\n\n",
    )
    .await;
    assert_eq!(body.matches("event: error").count(), 1, "{body}");
    assert!(!body.contains("event: message_stop"), "{body}");
}

async fn held_gemini_stream(State(dropped): State<Arc<Notify>>) -> Body {
    let (sender, receiver) = mpsc::channel::<Result<bytes::Bytes, Infallible>>(1);
    sender
        .send(Ok(bytes::Bytes::from_static(
            b"data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"early\"}]}}]}\n\n",
        )))
        .await
        .unwrap();
    tokio::spawn(async move {
        sender.closed().await;
        dropped.notify_one();
    });
    Body::from_stream(stream::unfold(receiver, |mut receiver| async move {
        receiver.recv().await.map(|item| (item, receiver))
    }))
}

async fn chunked_gemini_unary(State(body): State<Arc<Vec<u8>>>) -> Body {
    let body = bytes::Bytes::copy_from_slice(body.as_slice());
    Body::from_stream(stream::once(async move { Ok::<_, Infallible>(body) }))
}

async fn chunked_unary_gateway_response(body: Vec<u8>) -> reqwest::Response {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = Router::new()
        .route(
            "/v1beta/models/gemini-2.5-pro:generateContent",
            post(chunked_gemini_unary),
        )
        .with_state(Arc::new(body));
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let gateway = start_gateway(format!("http://{addr}")).await;
    let response = reqwest::Client::new()
        .post(format!("{}/v1/messages", gateway.base_url))
        .body(request(false))
        .send()
        .await
        .unwrap();
    task.abort();
    response
}

#[tokio::test]
async fn gemini_streaming_framing_delivers_early_and_drops_pending_upstream() {
    if !can_bind_loopback() {
        return;
    }
    let dropped = Arc::new(Notify::new());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = Router::new()
        .route(
            "/v1beta/models/gemini-2.5-pro:streamGenerateContent",
            post(held_gemini_stream),
        )
        .with_state(dropped.clone());
    let upstream_task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let gateway = start_gateway(format!("http://{addr}")).await;
    let response = reqwest::Client::new()
        .post(format!("{}/v1/messages", gateway.base_url))
        .body(request(true))
        .send()
        .await
        .unwrap();
    let mut body = response.bytes_stream();
    let first = tokio::time::timeout(std::time::Duration::from_secs(1), body.next())
        .await
        .expect("translated delta must arrive before upstream EOF")
        .unwrap()
        .unwrap();
    assert!(String::from_utf8_lossy(&first).contains("early"));
    drop(body);
    tokio::time::timeout(std::time::Duration::from_secs(1), dropped.notified())
        .await
        .expect("downstream drop must release the pending upstream body");
    upstream_task.abort();
}

#[tokio::test]
async fn gemini_streaming_framing_joins_multiline_data_before_json_decode() {
    if !can_bind_loopback() {
        return;
    }
    let body = streaming_gateway_response(
        concat!(
            "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"multi\"}]},\n",
            "data: \"finishReason\":\"STOP\"}]}\n\n",
        )
        .as_bytes(),
    )
    .await;
    assert!(body.contains("multi"), "{body}");
    assert_eq!(body.matches("event: message_stop").count(), 1, "{body}");
    assert!(!body.contains("event: error"), "{body}");
}

#[tokio::test]
async fn gemini_streaming_framing_late_data_invalidates_pending_success() {
    if !can_bind_loopback() {
        return;
    }
    let body = streaming_gateway_response(
        concat!(
            "data: {\"candidates\":[{\"finishReason\":\"STOP\"}]}\n\n",
            "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"late\"}]}}]}\n\n",
        )
        .as_bytes(),
    )
    .await;
    assert_eq!(body.matches("event: error").count(), 1, "{body}");
    assert!(!body.contains("event: message_stop"), "{body}");
}

#[tokio::test]
async fn gemini_unary_bounds_accepts_exact_cap_and_rejects_plus_one() {
    if !can_bind_loopback() {
        return;
    }
    const LIMIT: usize = 32 * 1024 * 1024;
    let exact = unary_gateway_response(padded_gemini_response(LIMIT)).await;
    assert_eq!(exact.status(), StatusCode::OK);
    let exact_body: serde_json::Value = exact.json().await.unwrap();
    assert_eq!(exact_body["content"][0]["text"], "ok");

    let oversized = unary_gateway_response(padded_gemini_response(LIMIT + 1)).await;
    assert_eq!(oversized.status(), StatusCode::BAD_GATEWAY);
    let error: serde_json::Value = oversized.json().await.unwrap();
    assert_eq!(error["type"], "error");
}

#[tokio::test]
async fn gemini_unary_bounds_maps_embedded_provider_error_out_of_success() {
    if !can_bind_loopback() {
        return;
    }
    let response = unary_gateway_response(
        br#"{"response":{"error":{"code":429,"status":"RESOURCE_EXHAUSTED","message":"fixture quota"}}}"#.to_vec(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["type"], "error");
    assert_eq!(body["error"]["type"], "rate_limit_error");
    assert!(!body.to_string().contains("fixture-key"));
}

#[tokio::test]
async fn gemini_unary_bounds_direct_and_wrapped_have_semantic_parity() {
    if !can_bind_loopback() {
        return;
    }
    let direct = json!({
        "candidates": [{
            "content": {"parts": [{"text": "same"}]},
            "finishReason": "STOP"
        }],
        "usageMetadata": {"promptTokenCount": 3, "candidatesTokenCount": 1}
    });
    let wrapped = json!({"response": direct.clone()});
    let direct_response = unary_gateway_response(serde_json::to_vec(&direct).unwrap()).await;
    let wrapped_response = unary_gateway_response(serde_json::to_vec(&wrapped).unwrap()).await;
    assert_eq!(direct_response.status(), StatusCode::OK);
    assert_eq!(wrapped_response.status(), StatusCode::OK);
    let direct_body: serde_json::Value = direct_response.json().await.unwrap();
    let wrapped_body: serde_json::Value = wrapped_response.json().await.unwrap();
    assert_eq!(direct_body["content"], wrapped_body["content"]);
    assert_eq!(direct_body["stop_reason"], wrapped_body["stop_reason"]);
    assert_eq!(direct_body["usage"], wrapped_body["usage"]);
}

#[tokio::test]
async fn gemini_unary_bounds_rejects_plus_one_without_content_length() {
    if !can_bind_loopback() {
        return;
    }
    const LIMIT: usize = 32 * 1024 * 1024;
    let response = chunked_unary_gateway_response(padded_gemini_response(LIMIT + 1)).await;
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["type"], "error");
}

#[tokio::test]
async fn gemini_unary_bounds_rejects_malformed_json_without_sensitive_context() {
    if !can_bind_loopback() {
        return;
    }
    let response = unary_gateway_response(b"not-json".to_vec()).await;
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["type"], "error");
    let body = body.to_string();
    assert!(!body.contains("fixture-key"));
    assert!(!body.contains("project"));
}

#[tokio::test]
async fn gemini_unary_bounds_rejects_oversized_http_error_body() {
    if !can_bind_loopback() {
        return;
    }
    const LIMIT: usize = 32 * 1024 * 1024;
    let response = unary_gateway_response_with_status(503, vec![b'x'; LIMIT + 1]).await;
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["type"], "error");
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("exceeded"));
}
