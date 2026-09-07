//! Hermetic full-router Cursor tests. TLS identity and the Run URL remain real;
//! only DNS and the client's trust roots are injected. No live credentials.
#![allow(clippy::await_holding_lock)]

use std::{ffi::OsString, sync::Arc, time::Duration};

use axum::http::{Response, StatusCode};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use bytes::Bytes;
use serde_json::json;
use tokio::net::TcpListener;
use tokio_rustls::{rustls, TlsAcceptor};

use super::super::connect::encode_connect_frame;
use crate::config::Config;

struct EnvRestore(&'static str, Option<OsString>);
impl EnvRestore {
    fn set(key: &'static str, value: &std::path::Path) -> Self {
        let old = std::env::var_os(key);
        std::env::set_var(key, value);
        Self(key, old)
    }
}
impl Drop for EnvRestore {
    fn drop(&mut self) {
        match self.1.take() {
            Some(value) => std::env::set_var(self.0, value),
            None => std::env::remove_var(self.0),
        }
    }
}

struct TaskGuard(tokio::task::JoinHandle<()>);
impl Drop for TaskGuard {
    fn drop(&mut self) {
        self.0.abort();
    }
}

async fn router_turn(terminal: bool, stream: bool) -> (StatusCode, String) {
    router_case(terminal, stream, None).await
}

async fn router_case(
    terminal: bool,
    stream: bool,
    history: Option<serde_json::Value>,
) -> (StatusCode, String) {
    let hydrate = history.is_some();
    let host = "agentn.global.api5.cursor.sh";
    assert_eq!(super::agent_base_url(), super::AGENT_BASE_URL);
    let cert = rcgen::generate_simple_self_signed(vec![host.to_string()]).unwrap();
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
    let addr = listener.local_addr().unwrap();
    let client = reqwest::Client::builder()
        .no_proxy()
        .add_root_certificate(reqwest::Certificate::from_der(cert.cert.der()).unwrap())
        .resolve(host, addr)
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    let token = format!(
        "synthetic.{}.signature",
        URL_SAFE_NO_PAD.encode(br#"{"exp":4102444800}"#)
    );
    let dir = std::env::temp_dir().join(format!("shunt-cursor-router-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let auth_path = dir.join("cursor-auth.json");
    let auth_bytes = serde_json::to_vec(&json!({"accessToken":token})).unwrap();
    std::fs::write(&auth_path, &auth_bytes).unwrap();
    let _auth = EnvRestore::set("SHUNT_CURSOR_AUTH_FILE", &auth_path);
    let expected_auth = format!("Bearer {token}");
    let requests = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let observed = requests.clone();
    let upstream = TaskGuard(tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        let socket = TlsAcceptor::from(Arc::new(tls))
            .accept(socket)
            .await
            .unwrap();
        let mut connection = h2::server::handshake(socket).await.unwrap();
        let mut workers = tokio::task::JoinSet::new();
        while let Some(request) = connection.accept().await {
            let (request, mut respond) = request.unwrap();
            observed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            assert_eq!(request.method(), "POST");
            assert_eq!(request.uri().path(), super::AGENT_PATH);
            assert_eq!(request.headers()["authorization"], expected_auth);
            assert_eq!(
                request.headers()["content-type"],
                "application/connect+proto"
            );
            // Schema-derived AgentServerMessage.f1 / InteractionUpdate.f1 /
            // TextDeltaUpdate.f1, not the retired protobuf module.
            let text = [0x0a, 6, 0x0a, 4, 0x0a, 2, b'O', b'K'];
            let mut body = encode_connect_frame(text, 0).to_vec();
            if terminal {
                body.extend_from_slice(&encode_connect_frame(b"{}", 2));
            }
            let response = Response::builder()
                .status(200)
                .header("content-type", "application/connect+proto")
                .body(())
                .unwrap();
            let mut send = respond.send_response(response, false).unwrap();
            if hydrate {
                workers.spawn(async move {
                    hydrate_history(request.into_body(), &mut send).await;
                    send.send_data(Bytes::from(body), true).unwrap();
                });
            } else {
                send.send_data(Bytes::from(body), true).unwrap();
            }
        }
    }));
    let mut config = Config::default();
    config.server.default_provider = "cursor".into();
    let (router, _, _) = crate::server::build_router_with_test_client(config, client).unwrap();
    let gateway = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let gateway_addr = gateway.local_addr().unwrap();
    let serving = TaskGuard(tokio::spawn(async move {
        axum::serve(gateway, router).await.unwrap();
    }));
    let mut input =
        history.unwrap_or_else(|| json!({"messages":[{"role":"user","content":"fixture"}]}));
    input["model"] = json!("composer-2.5");
    input["stream"] = json!(stream);
    input["max_tokens"] = json!(16);
    let response = reqwest::Client::new()
        .post(format!("http://{gateway_addr}/v1/messages"))
        .json(&input)
        .timeout(Duration::from_secs(12))
        .send()
        .await
        .unwrap();
    let status = response.status();
    let body = response.text().await.unwrap();
    if status == StatusCode::BAD_REQUEST {
        assert_eq!(requests.load(std::sync::atomic::Ordering::SeqCst), 0);
    }
    assert_eq!(std::fs::read(&auth_path).unwrap(), auth_bytes);
    drop(serving);
    drop(upstream);
    std::fs::remove_file(&auth_path).unwrap();
    std::fs::remove_dir(&dir).unwrap();
    (status, body)
}

#[tokio::test]
async fn cursor_terminal_tracer_full_router_eof_and_terminal() {
    let _lock = crate::config::CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    for stream in [false, true] {
        let (status, body) = router_turn(false, stream).await;
        assert!(
            body.contains("EOF without an authoritative terminal"),
            "{body}"
        );
        if stream {
            assert_eq!(status, StatusCode::OK);
            assert_eq!(body.matches("event: error\n").count(), 1, "{body}");
            assert!(!body.contains("event: message_stop"), "{body}");
        } else {
            assert_eq!(status, StatusCode::BAD_GATEWAY, "{body}");
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&body).unwrap()["type"],
                "error"
            );
        }
        let (status, body) = router_turn(true, stream).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        if stream {
            assert_eq!(body.matches("event: message_stop\n").count(), 1, "{body}");
            assert!(!body.contains("event: error"), "{body}");
        } else {
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&body).unwrap()["stop_reason"],
                "end_turn"
            );
        }
    }
}

fn wire_fields(bytes: &[u8], number: u32) -> Vec<&[u8]> {
    super::super::wire::fields(bytes)
        .map(Result::unwrap)
        .filter(|f| f.number == number)
        .map(|f| f.bytes)
        .collect()
}
fn wire_one(bytes: &[u8], number: u32) -> &[u8] {
    let found = wire_fields(bytes, number);
    assert_eq!(found.len(), 1, "field {number}");
    found[0]
}

async fn next_payload(
    body: &mut h2::RecvStream,
    decoder: &mut super::super::connect::ConnectFrameDecoder,
    pending: &mut std::collections::VecDeque<Bytes>,
) -> Bytes {
    loop {
        if let Some(frame) = pending.pop_front() {
            return frame;
        }
        let chunk = tokio::time::timeout(Duration::from_secs(6), body.data())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        body.flow_control().release_capacity(chunk.len()).unwrap();
        for frame in decoder.push(&chunk).unwrap() {
            pending.push_back(frame.payload);
        }
    }
}

async fn fetch_blob(
    id: &[u8],
    serial: &mut u64,
    body: &mut h2::RecvStream,
    send: &mut h2::SendStream<Bytes>,
    decoder: &mut super::super::connect::ConnectFrameDecoder,
    pending: &mut std::collections::VecDeque<Bytes>,
) -> Vec<u8> {
    *serial += 1;
    let mut get = super::field_varint(1, *serial);
    get.extend(super::field_ld(2, &super::field_ld(1, id)));
    send.send_data(encode_connect_frame(super::field_ld(4, &get), 0), false)
        .unwrap();
    loop {
        let payload = next_payload(body, decoder, pending).await;
        let replies = wire_fields(&payload, 3);
        if replies.is_empty() {
            continue;
        } // paced context/heartbeat frames
        let reply = replies[0];
        let correlation = super::super::wire::fields(reply)
            .map(Result::unwrap)
            .find(|f| f.number == 1)
            .unwrap()
            .varint;
        assert_eq!(correlation, *serial);
        let data = wire_one(wire_one(reply, 2), 1).to_vec();
        use sha2::Digest;
        assert_eq!(sha2::Sha256::digest(&data).as_slice(), id);
        return data;
    }
}

async fn hydrate_history(mut body: h2::RecvStream, send: &mut h2::SendStream<Bytes>) {
    let mut decoder = super::super::connect::ConnectFrameDecoder::new();
    let mut pending = std::collections::VecDeque::new();
    let first = next_payload(&mut body, &mut decoder, &mut pending).await;
    let run = wire_one(&first, 1);
    let state = wire_one(run, 1);
    assert_eq!(wire_one(run, 5), wire_one(run, 16));
    uuid::Uuid::parse_str(std::str::from_utf8(wire_one(run, 5)).unwrap()).unwrap();
    let mut serial = 0;
    for root in wire_fields(state, 1) {
        let data = fetch_blob(
            root,
            &mut serial,
            &mut body,
            send,
            &mut decoder,
            &mut pending,
        )
        .await;
        let _: serde_json::Value = serde_json::from_slice(&data).unwrap();
    }
    let mut ids = Vec::new();
    for id in wire_fields(state, 8) {
        let blob = fetch_blob(id, &mut serial, &mut body, send, &mut decoder, &mut pending).await;
        let turn = wire_one(&blob, 1);
        let user = fetch_blob(
            wire_one(turn, 1),
            &mut serial,
            &mut body,
            send,
            &mut decoder,
            &mut pending,
        )
        .await;
        assert!(!wire_one(&user, 1).is_empty());
        for id in wire_fields(turn, 2) {
            let step =
                fetch_blob(id, &mut serial, &mut body, send, &mut decoder, &mut pending).await;
            let calls = wire_fields(&step, 2);
            if calls.is_empty() {
                continue;
            }
            let mcp = wire_one(calls[0], 15);
            let args = wire_one(mcp, 1);
            ids.push(String::from_utf8(wire_one(args, 3).to_vec()).unwrap());
            let content = wire_one(wire_one(wire_one(wire_one(wire_one(mcp, 2), 1), 1), 1), 1);
            assert_eq!(content, b"file contents");
        }
    }
    assert_eq!(ids, ["authentic-17"]);
}

#[tokio::test]
async fn cursor_history_identity_full_router_bidirectional_hydration() {
    let _lock = crate::config::CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    for stream in [false, true] {
        let history = json!({"metadata":{"session_id":"router-session"},"messages":[
            {"role":"user","content":"Read the file"},
            {"role":"assistant","content":[{"type":"tool_use","id":"authentic-17","name":"Read","input":{"path":"a.txt"}}]},
            {"role":"user","content":[{"type":"tool_result","tool_use_id":"authentic-17","content":"file contents"}]}
        ]});
        let (status, body) = router_case(true, stream, Some(history)).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert!(body.contains("OK"), "{body}");
        assert!(!body.contains("event: error"), "{body}");
    }
}

#[tokio::test]
async fn cursor_continuation_guard_full_router_zero_dispatch() {
    let _lock = crate::config::CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    for stream in [false, true] {
        for history in [
            json!({"checkpoint":"opaque","messages":[{"role":"user","content":"continue"}]}),
            json!({"messages":[{"role":"user","content":[{"type":"tool_result","tool_use_id":"unknown","content":"x"}]}]}),
        ] {
            let (status, body) = router_case(true, stream, Some(history)).await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
            let envelope: serde_json::Value = serde_json::from_str(&body).unwrap();
            assert_eq!(envelope["type"], "error");
            assert_eq!(envelope["error"]["type"], "invalid_request_error");
        }
    }
}
