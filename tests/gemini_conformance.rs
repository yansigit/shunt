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

fn gemini_config(base_url: String) -> Config {
    let mut config = Config::default();
    let provider = config.providers.get_mut("gemini").unwrap();
    provider.base_url = base_url;
    provider.auth = AuthMode::ApiKey;
    provider.api_key_env = Some("SHUNT_GEMINI_CONFORMANCE_KEY".to_string());
    provider.retry = RetryConfig {
        max_retries: 0,
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

async fn start_gateway(base_url: String) -> Gateway {
    std::env::set_var("SHUNT_GEMINI_CONFORMANCE_KEY", "fixture-key");
    let mut config = gemini_config(base_url);
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
