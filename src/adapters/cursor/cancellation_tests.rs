//! Full-router cancellation with an actual TLS/H2 upstream and synthetic auth.
#![allow(clippy::await_holding_lock)]
use super::*;
use crate::adapters::cursor::agent as run;
use futures_util::StreamExt;
use std::sync::atomic::{AtomicUsize, Ordering};
use tower::ServiceExt;

struct Active {
    count: Arc<AtomicUsize>,
    released: Arc<tokio::sync::Notify>,
}
impl Drop for Active {
    fn drop(&mut self) {
        self.count.fetch_sub(1, Ordering::SeqCst);
        self.released.notify_one();
    }
}

fn request(stream: bool) -> axum::http::Request<axum::body::Body> {
    axum::http::Request::builder()
        .method("POST")
        .uri("/v1/messages")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            json!({"model":"composer-2.5", "max_tokens":16,
            "stream":stream,"messages":[{"role":"user","content":"fixture"}]})
            .to_string(),
        ))
        .unwrap()
}

#[tokio::test]
async fn cursor_cancellation_release_full_router_headers_stream_and_aggregation() {
    let _observer = crate::adapters::cursor::offload::OFFLOAD_OBSERVER
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _lock = crate::config::CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    for phase in ["headers", "stream", "aggregate"] {
        cancellation_case(phase).await;
    }
}

async fn cancellation_case(phase: &'static str) {
    let host = "agentn.global.api5.cursor.sh";
    let cert = rcgen::generate_simple_self_signed(vec![host.into()]).unwrap();
    let key = rustls::pki_types::PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());
    let mut tls = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::aws_lc_rs::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_no_client_auth()
    .with_single_cert(vec![cert.cert.der().clone()], key.into())
    .unwrap();
    tls.alpn_protocols = vec![b"h2".to_vec()];
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let client = reqwest::Client::builder()
        .no_proxy()
        .add_root_certificate(reqwest::Certificate::from_der(cert.cert.der()).unwrap())
        .resolve(host, listener.local_addr().unwrap())
        .build()
        .unwrap();
    let dir = std::env::temp_dir().join(format!("shunt-cursor-cancel-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let auth_path = dir.join("cursor-auth.json");
    let token = format!(
        "synthetic.{}.signature",
        URL_SAFE_NO_PAD.encode(br#"{"exp":4102444800}"#)
    );
    let auth_bytes = serde_json::to_vec(&json!({"accessToken":token})).unwrap();
    std::fs::write(&auth_path, &auth_bytes).unwrap();
    let _auth = EnvRestore::set("SHUNT_CURSOR_AUTH_FILE", &auth_path);
    let received = Arc::new(tokio::sync::Notify::new());
    let released = Arc::new(tokio::sync::Notify::new());
    let active = Arc::new(AtomicUsize::new(0));
    let hits = Arc::new(AtomicUsize::new(0));
    let upstream = {
        let received = received.clone();
        let released = released.clone();
        let active = active.clone();
        let hits = hits.clone();
        TaskGuard(tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let socket = TlsAcceptor::from(Arc::new(tls))
                .accept(socket)
                .await
                .unwrap();
            let mut conn = h2::server::handshake(socket).await.unwrap();
            let mut workers = tokio::task::JoinSet::new();
            while let Some(next) = conn.accept().await {
                let (req, mut respond) = next.unwrap();
                assert_eq!(req.uri().path(), run::AGENT_PATH);
                assert_eq!(req.headers()["authorization"], format!("Bearer {token}"));
                let first = hits.fetch_add(1, Ordering::SeqCst) == 0;
                let text = encode_connect_frame(
                    run::field_ld(1, &run::field_ld(1, &run::field_str(1, "OK"))),
                    0,
                );
                if !first {
                    let response = Response::builder().status(200).body(()).unwrap();
                    let mut send = respond.send_response(response, false).unwrap();
                    send.send_data(
                        Bytes::from(
                            [text.to_vec(), encode_connect_frame(b"{}", 2).to_vec()].concat(),
                        ),
                        true,
                    )
                    .unwrap();
                    continue;
                }
                active.fetch_add(1, Ordering::SeqCst);
                let guard = Active {
                    count: active.clone(),
                    released: released.clone(),
                };
                let received = received.clone();
                workers.spawn(async move {
                    let _guard = guard;
                    let mut body = req.into_body();
                    let _send = if phase == "headers" {
                        None
                    } else {
                        let response = Response::builder()
                            .status(200)
                            .header("content-type", "application/connect+proto")
                            .body(())
                            .unwrap();
                        let mut send = respond.send_response(response, false).unwrap();
                        send.send_data(text, false).unwrap();
                        Some(send)
                    };
                    // Keep the header sender alive for the pre-header case.
                    let _respond = respond;
                    received.notify_one();
                    while let Some(chunk) = body.data().await {
                        match chunk {
                            Ok(bytes) => body.flow_control().release_capacity(bytes.len()).unwrap(),
                            Err(_) => break,
                        }
                    }
                });
            }
        }))
    };
    let mut config = Config::default();
    config.server.default_provider = "cursor".into();
    config.server.max_concurrent_requests = 1;
    let (router, _, _) = crate::server::build_router_with_test_client(config, client).unwrap();
    let task = tokio::spawn(router.clone().oneshot(request(phase == "stream")));
    tokio::time::timeout(Duration::from_secs(2), received.notified())
        .await
        .unwrap();
    let saturated = router.clone().oneshot(request(false)).await.unwrap();
    assert_eq!(
        saturated.status(),
        StatusCode::SERVICE_UNAVAILABLE,
        "{phase}"
    );
    assert_eq!(hits.load(Ordering::SeqCst), 1);
    if phase == "stream" {
        let response = task.await.unwrap().unwrap();
        let mut body = response.into_body().into_data_stream();
        tokio::time::timeout(Duration::from_secs(2), async {
            while let Some(chunk) = body.next().await {
                if String::from_utf8_lossy(&chunk.unwrap()).contains("text_delta") {
                    return;
                }
            }
            panic!("expected incremental output");
        })
        .await
        .unwrap();
        drop(body);
    } else {
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
    }
    tokio::time::timeout(Duration::from_secs(2), released.notified())
        .await
        .unwrap_or_else(|_| panic!("{phase}: upstream request body leaked after cancel"));
    assert_eq!(active.load(Ordering::SeqCst), 0);
    let next = tokio::time::timeout(Duration::from_secs(2), router.oneshot(request(false)))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        next.status(),
        StatusCode::OK,
        "{phase}: capacity not released"
    );
    assert_eq!(hits.load(Ordering::SeqCst), 2);
    let bytes = axum::body::to_bytes(next.into_body(), 65536).await.unwrap();
    assert!(String::from_utf8_lossy(&bytes).contains("end_turn"));
    assert_eq!(std::fs::read(&auth_path).unwrap(), auth_bytes);
    drop(upstream);
    std::fs::remove_file(&auth_path).unwrap();
    std::fs::remove_dir(&dir).unwrap();
}

#[tokio::test]
async fn cursor_no_heuristics_repeated_text_never_infers_completion() {
    use crate::adapters::cursor::response::CursorStreamEvent;
    for terminal in [false, true] {
        let text = "I will continue. No progress yet. Repeating the same status.";
        let frame = encode_connect_frame(
            run::field_ld(1, &run::field_ld(1, &run::field_str(1, text))),
            0,
        );
        let mut frames = frame.repeat(100);
        if terminal {
            frames.extend(encode_connect_frame(b"{}", 2));
        }
        let events: Vec<_> = run::tests::turn_from_frames(frames)
            .await
            .into_event_stream()
            .collect()
            .await;
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Ok(CursorStreamEvent::TextDelta { .. })))
                .count(),
            100
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Ok(CursorStreamEvent::End)))
                .count(),
            usize::from(terminal)
        );
        assert_eq!(
            events.iter().filter(|event| event.is_err()).count(),
            usize::from(!terminal)
        );
    }
}

#[test]
fn cursor_no_heuristics_config_and_request_store_remain_local() {
    let config = Config::default();
    let provider = serde_json::to_value(&config.providers["cursor"]).unwrap();
    let parsed: crate::config::ProviderConfig = serde_json::from_value(provider.clone()).unwrap();
    assert_eq!(serde_json::to_value(parsed).unwrap(), provider);
    let keys: std::collections::BTreeSet<_> = provider
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        std::collections::BTreeSet::from([
            "kind",
            "base_url",
            "auth",
            "api_key_env",
            "api_key_header",
            "effort",
            "service_tier",
            "count_tokens",
            "accounts",
            "websocket",
            "request_compression",
            "retry",
            "workspace_roots",
            "sandbox",
        ])
    );
    let id = {
        let mut store = crate::adapters::cursor::kv::RequestBlobStore::new();
        store.store_content(b"request-owned history").unwrap()
    };
    assert!(crate::adapters::cursor::kv::RequestBlobStore::new()
        .get(&id)
        .is_none());
}

#[tokio::test]
async fn cursor_output_parity_full_router_keeps_reasoning_text_usage_order() {
    let _observer = crate::adapters::cursor::offload::OFFLOAD_OBSERVER
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _lock = crate::config::CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    for stream in [false, true] {
        let (status, body) = router_case(
            true,
            stream,
            Some(json!({"_test_rich":true,"messages":[{"role":"user","content":"fixture"}]})),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        if stream {
            let positions = ["reason-one", "answer-one", "reason-two", "answer-two"]
                .map(|text| body.find(text).unwrap());
            assert!(positions.windows(2).all(|p| p[0] < p[1]));
            assert!(body.contains("\"output_tokens\":42"));
            assert!(body.contains("\"input_tokens\":58"));
            assert_eq!(body.matches("event: message_stop").count(), 1);
            let starts: Vec<serde_json::Value> = body
                .lines()
                .filter_map(|line| line.strip_prefix("data: "))
                .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
                .filter(|event| event["type"] == "content_block_start")
                .collect();
            assert_eq!(starts.len(), 4);
            for (index, kind) in ["thinking", "text", "thinking", "text"].iter().enumerate() {
                assert_eq!(starts[index]["index"], index);
                assert_eq!(starts[index]["content_block"]["type"], *kind);
            }
        } else {
            let response: serde_json::Value = serde_json::from_str(&body).unwrap();
            assert_eq!(
                response["content"],
                json!([
                    {"type":"thinking","thinking":"reason-one","signature":""}, {"type":"text","text":"answer-one"},
                    {"type":"thinking","thinking":"reason-two","signature":""}, {"type":"text","text":"answer-two"}
                ])
            );
            assert_eq!(response["usage"]["output_tokens"], 42);
            assert_eq!(response["usage"]["input_tokens"], 58);
        }
    }
}
