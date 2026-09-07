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
    let upstream = TaskGuard(tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        let socket = TlsAcceptor::from(Arc::new(tls))
            .accept(socket)
            .await
            .unwrap();
        let mut connection = h2::server::handshake(socket).await.unwrap();
        while let Some(request) = connection.accept().await {
            let (request, mut respond) = request.unwrap();
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
            let mut body = encode_connect_frame(&text, 0).to_vec();
            if terminal {
                body.extend_from_slice(&encode_connect_frame(b"{}", 2));
            }
            let response = Response::builder()
                .status(200)
                .header("content-type", "application/connect+proto")
                .body(())
                .unwrap();
            let mut send = respond.send_response(response, false).unwrap();
            send.send_data(Bytes::from(body), true).unwrap();
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
    let response = reqwest::Client::new()
        .post(format!("http://{gateway_addr}/v1/messages"))
        .json(
            &json!({"model":"composer-2.5","max_tokens":16,"stream":stream,
            "messages":[{"role":"user","content":"fixture"}]}),
        )
        .timeout(Duration::from_secs(12))
        .send()
        .await
        .unwrap();
    let status = response.status();
    let body = response.text().await.unwrap();
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
