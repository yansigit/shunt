//! Source-derived synthetic fixtures: 14-PROTOCOL-EVIDENCE.md, not live captures.
#![allow(clippy::await_holding_lock)]

mod lifetime;
mod replay;

use crate::config::Config;
use serde_json::{json, Value};
use std::{ffi::OsString, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use tokio_rustls::{rustls, TlsAcceptor};

fn config(base: &str) -> Config {
    let mut value = serde_json::to_value(Config::default()).unwrap();
    value["providers"]["cc"] =
        json!({"kind":"command_code","auth":"command_code_oauth","base_url":base});
    value["server"]["default_provider"] = json!("cc");
    let parsed = serde_json::from_value::<Config>(value);
    assert!(
        parsed.is_ok(),
        "subscription kind/auth must be admitted: {:?}",
        parsed.as_ref().err()
    );
    parsed.unwrap()
}

struct EnvGuard(Option<OsString>);
impl EnvGuard {
    fn set(value: Option<&str>) -> Self {
        let old = std::env::var_os("SHUNT_COMMAND_CODE_TOKEN");
        if let Some(value) = value {
            std::env::set_var("SHUNT_COMMAND_CODE_TOKEN", value);
        } else {
            std::env::remove_var("SHUNT_COMMAND_CODE_TOKEN");
        }
        Self(old)
    }
}
impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(value) = &self.0 {
            std::env::set_var("SHUNT_COMMAND_CODE_TOKEN", value);
        } else {
            std::env::remove_var("SHUNT_COMMAND_CODE_TOKEN");
        }
    }
}
struct Task(tokio::task::JoinHandle<()>);
impl Drop for Task {
    fn drop(&mut self) {
        self.0.abort();
    }
}

async fn turn(wire: &str, status: u16) -> (reqwest::StatusCode, Value) {
    let (status, body) = turn_mode(wire, status, false).await;
    (status, serde_json::from_str(&body).unwrap())
}

async fn turn_mode(wire: &str, status: u16, streaming: bool) -> (reqwest::StatusCode, String) {
    let lookups_before =
        crate::auth::command_code::LOOKUPS.load(std::sync::atomic::Ordering::SeqCst);
    let config = config("https://api.commandcode.ai");
    let cert = rcgen::generate_simple_self_signed(vec!["api.commandcode.ai".into()]).unwrap();
    let key = rustls::pki_types::PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());
    let tls = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::aws_lc_rs::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_no_client_auth()
    .with_single_cert(vec![cert.cert.der().clone()], key.into())
    .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let client = reqwest::Client::builder()
        .no_proxy()
        .http1_only()
        .redirect(reqwest::redirect::Policy::none())
        .add_root_certificate(reqwest::Certificate::from_der(cert.cert.der()).unwrap())
        .resolve("api.commandcode.ai", addr)
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let wire = wire.to_string();
    let (observed_delta, wait_for_delta) = tokio::sync::oneshot::channel();
    let mut upstream = Task(tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        let mut socket = TlsAcceptor::from(Arc::new(tls))
            .accept(socket)
            .await
            .unwrap();
        let mut bytes = Vec::new();
        loop {
            let mut buf = [0; 2048];
            let n = socket.read(&mut buf).await.unwrap();
            assert!(n > 0);
            bytes.extend_from_slice(&buf[..n]);
            assert!(bytes.len() < 65536);
            if let Some(end) = bytes.windows(4).position(|p| p == b"\r\n\r\n") {
                let head = std::str::from_utf8(&bytes[..end]).unwrap();
                let header = |name: &str| {
                    head.lines()
                        .filter_map(|line| line.split_once(':'))
                        .find(|(key, _)| key.eq_ignore_ascii_case(name))
                        .map(|(_, value)| value.trim())
                };
                let len: usize = header("content-length").unwrap().parse().unwrap();
                if bytes.len() < end + 4 + len {
                    continue;
                }
                assert!(head.starts_with("POST /alpha/generate "));
                assert_eq!(
                    header("authorization"),
                    Some("Bearer synthetic-subscription-token")
                );
                assert_eq!(header("host"), Some("api.commandcode.ai"));
                assert_eq!(header("user-agent"), Some("cli"));
                assert_eq!(header("x-command-code-version"), Some("0.52.1"));
                assert_eq!(header("x-project-slug"), None);
                let body: Value = serde_json::from_slice(&bytes[end + 4..end + 4 + len]).unwrap();
                assert_eq!(body["params"]["stream"], true);
                assert_eq!(body["params"]["model"], "zai-org/GLM-5.3");
                break;
            }
        }
        let location = if status == 302 {
            "location: https://redirect.invalid/steal\r\n"
        } else {
            ""
        };
        socket.write_all(format!("HTTP/1.1 {status} Fixture\r\ncontent-type: application/x-ndjson\r\ncontent-length: {}\r\n{location}connection: close\r\n\r\n",wire.len()).as_bytes()).await.unwrap();
        if streaming
            && ["text-delta", "reasoning-delta", "tool-call"]
                .iter()
                .any(|kind| wire.starts_with(&format!("{{\"type\":\"{kind}\"")))
        {
            let split = wire.find('\n').unwrap() + 1;
            socket.write_all(&wire.as_bytes()[..split]).await.unwrap();
            socket.flush().await.unwrap();
            tokio::time::timeout(Duration::from_secs(3), wait_for_delta)
                .await
                .expect("client must receive delta before upstream finish")
                .unwrap();
            socket.write_all(&wire.as_bytes()[split..]).await.unwrap();
        } else {
            socket.write_all(wire.as_bytes()).await.unwrap();
        }
        // Finish flushing TLS records and send close_notify. Dropping a buffered
        // TLS writer can truncate the final large record. Rejection cases may
        // already have closed their reader; client assertions still prove each
        // accepted body is complete and each rejected body fails explicitly.
        let _ = socket.shutdown().await;
    }));
    let (router, _, _) = crate::server::build_router_with_test_client(config, client).unwrap();
    let gateway = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = gateway.local_addr().unwrap();
    let _serving = Task(tokio::spawn(async move {
        axum::serve(gateway, router).await.unwrap();
    }));
    let response = reqwest::Client::new().post(format!("http://{addr}/v1/messages"))
        .timeout(Duration::from_secs(5)).json(&json!({"model":"zai-org/GLM-5.3","stream":streaming,"max_tokens":64,"messages":[{"role":"user","content":"fixture"}]}))
        .send().await.unwrap();
    let status = response.status();
    if streaming {
        assert_eq!(
            status,
            reqwest::StatusCode::OK,
            "streaming subscription turn must be admitted"
        );
    }
    use futures_util::StreamExt;
    let mut bytes = response.bytes_stream();
    let mut body = Vec::new();
    let mut observed_delta = Some(observed_delta);
    while let Some(chunk) = bytes.next().await {
        body.extend_from_slice(&chunk.unwrap());
        if observed_delta.is_some()
            && body
                .windows(b"content_block_delta".len())
                .any(|s| s == b"content_block_delta")
        {
            if let Some(signal) = observed_delta.take() {
                let _ = signal.send(());
            }
        }
    }
    let body = String::from_utf8(body).unwrap();
    tokio::time::timeout(Duration::from_secs(5), &mut upstream.0)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        crate::auth::command_code::LOOKUPS.load(std::sync::atomic::Ordering::SeqCst),
        lookups_before + 1
    );
    (status, body)
}

#[tokio::test]
async fn command_code_tracer_response_incremental_and_unary() {
    let _lock = crate::config::CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _key = EnvGuard::set(Some("synthetic-subscription-token"));
    let wire = "{\"type\":\"text-delta\",\"text\":\"first\"}\n{\"type\":\"text-delta\",\"text\":\"second\"}\n{\"type\":\"finish\",\"finishReason\":\"stop\"}\n";
    let (_, stream) = turn_mode(wire, 200, true).await;
    assert_eq!(stream.matches("event: message_start\n").count(), 1);
    assert_eq!(stream.matches("event: message_stop\n").count(), 1);
    assert_eq!(stream.matches("event: error\n").count(), 0);
    assert!(stream.find("first").unwrap() < stream.find("second").unwrap());
    let (_, unary) = turn(wire, 200).await;
    assert_eq!(unary["content"][0]["text"], "firstsecond");
}

#[tokio::test]
async fn command_code_bounds_wire_record_limit() {
    let _lock = crate::config::CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _key = EnvGuard::set(Some("synthetic-subscription-token"));
    let record = "{\"type\":\"text-delta\",\"text\":\"x\"}";
    let cap = super::ndjson::MAX_RECORD_BYTES;
    for size in [cap - 1, cap, cap + 1] {
        let wire = format!(
            "{}{record}\n{{\"type\":\"finish\",\"finishReason\":\"stop\"}}\n",
            " ".repeat(size - record.len())
        );
        let (status, _) = turn(&wire, 200).await;
        assert_eq!(
            status,
            if size <= cap {
                reqwest::StatusCode::OK
            } else {
                reqwest::StatusCode::BAD_GATEWAY
            }
        );
        let (_, stream) = turn_mode(&wire, 200, true).await;
        assert_eq!(
            stream.matches("event: message_stop\n").count(),
            usize::from(size <= cap),
            "record bytes {size}: {stream}"
        );
        assert_eq!(
            stream.matches("event: error\n").count(),
            usize::from(size > cap),
            "record bytes {size}: {stream}"
        );
    }
}

#[tokio::test]
async fn command_code_tracer_response_errors_never_emit_success() {
    let _lock = crate::config::CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _key = EnvGuard::set(Some("synthetic-subscription-token"));
    for tail in [
        "", "junk\n", "null\n", "{\"type\":\"unknown\"}\n",
        "{\"type\":\"error\",\"error\":\"synthetic-private-detail\"}\n",
        "{\"type\":\"finish\",\"finishReason\":\"error\",\"usage\":{\"inputTokens\":2,\"outputTokens\":3}}\n",
        "{\"type\":\"finish\",\"finishReason\":\"stop\"}",
        "{\"type\":\"finish\",\"finishReason\":\"stop\"}\n{\"type\":\"finish\",\"finishReason\":\"stop\"}\n",
    ] {
        let wire = format!("{{\"type\":\"text-delta\",\"text\":\"prefix\"}}\n{tail}");
        let (_, stream) = turn_mode(&wire, 200, true).await;
        assert_eq!(stream.matches("event: error\n").count(), 1, "{tail}");
        assert_eq!(stream.matches("event: message_stop\n").count(), 0, "{tail}");
        assert!(!stream.contains("synthetic-private-detail"));
        let (status, error) = turn(&wire, 200).await;
        assert_eq!(status, reqwest::StatusCode::BAD_GATEWAY);
        assert_eq!(error["type"], "error");
        if tail.contains("inputTokens") {
            assert_eq!(error["usage"]["output_tokens"], 3);
            assert!(stream.contains("\"output_tokens\":3"));
        }
    }
}

#[tokio::test]
async fn command_code_tracer_response_reasoning_tool_first_and_usage() {
    let _lock = crate::config::CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _key = EnvGuard::set(Some("synthetic-subscription-token"));
    let wire = "{\"type\":\"tool-call\",\"toolCallId\":\"a\",\"toolName\":\"lookup\",\"input\":{}}\n{\"type\":\"reasoning-delta\",\"text\":\"why\"}\n{\"type\":\"finish\",\"rawFinishReason\":\"tool_use\",\"totalUsage\":{\"inputTokens\":10,\"outputTokens\":4,\"inputTokenDetails\":{\"cacheReadTokens\":6,\"cacheWriteTokens\":2}}}\n";
    let (_, stream) = turn_mode(wire, 200, true).await;
    assert!(
        stream.find("event: message_start\n").unwrap()
            < stream.find("event: content_block_start\n").unwrap()
    );
    assert_eq!(stream.matches("event: message_stop\n").count(), 1);
    assert!(stream.contains("thinking_delta"));
    let (_, unary) = turn(wire, 200).await;
    assert_eq!(unary["content"][0]["id"], "a");
    assert_eq!(unary["usage"]["input_tokens"], 2);
    assert_eq!(unary["stop_reason"], "tool_use");
}

#[tokio::test]
async fn command_code_tracer_unary() {
    let _lock = crate::config::CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _key = EnvGuard::set(Some("synthetic-subscription-token"));
    let (status,body) = turn("{\"type\":\"text-delta\",\"text\":\"fixture reply\"}\n{\"type\":\"finish\",\"finishReason\":\"stop\",\"totalUsage\":{\"inputTokens\":5,\"outputTokens\":2}}\n",200).await;
    assert_eq!(status, reqwest::StatusCode::OK);
    assert_eq!(body["content"][0]["text"], "fixture reply");
    assert_eq!(body["usage"]["input_tokens"], 5);
}

#[tokio::test]
async fn command_code_tracer_destination_negatives() {
    let _lock = crate::config::CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _key = EnvGuard::set(Some("synthetic-subscription-token"));
    let before = crate::auth::command_code::LOOKUPS.load(std::sync::atomic::Ordering::SeqCst);
    for base in [
        "http://api.commandcode.ai",
        "https://api.commandcode.ai.evil.invalid",
        "https://api.commandcode.ai/wrong",
        "https://user@api.commandcode.ai",
        "https://api.commandcode.ai?x=1",
        "https://api.commandcode.ai#x",
        "https://api.commandcode.ai:8443",
        "http://127.0.0.1:10101",
    ] {
        let config = config(base);
        assert!(config.clone().validate().is_err(), "{base}");
        assert!(
            crate::auth::command_code::resolve(&config.providers["cc"])
                .await
                .is_err(),
            "direct resolver must also gate {base}"
        );
    }
    assert_eq!(
        crate::auth::command_code::LOOKUPS.load(std::sync::atomic::Ordering::SeqCst),
        before
    );
    let (status, body) = turn("", 302).await;
    assert_eq!(status, reqwest::StatusCode::BAD_GATEWAY);
    assert_eq!(body["type"], "error");
}

#[test]
fn command_code_lifetime_destination_dispatch_rechecks_snapshot() {
    let _lock = crate::config::CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let before = crate::auth::command_code::LOOKUPS.load(std::sync::atomic::Ordering::SeqCst);
    let mut cfg = config("https://api.commandcode.ai");
    let provider = cfg.providers.get_mut("cc").unwrap();
    crate::auth::command_code::validate_provider(provider).unwrap();
    // Simulate a changed snapshot after the initial validation. Header
    // construction must independently reject it before attaching the bearer.
    for base in [
        "http://api.commandcode.ai",
        "https://api.commandcode.ai.evil.invalid",
        "https://api.commandcode.ai/wrong",
        "https://user@api.commandcode.ai",
        "https://api.commandcode.ai?x=1",
        "https://api.commandcode.ai#x",
    ] {
        provider.base_url = base.into();
        assert!(super::request::dispatch_headers(provider, "synthetic-token", None).is_err());
    }
    assert_eq!(
        crate::auth::command_code::LOOKUPS.load(std::sync::atomic::Ordering::SeqCst),
        before
    );
}

#[tokio::test]
async fn command_code_tracer_terminal_grammar() {
    let _lock = crate::config::CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _key = EnvGuard::set(Some("synthetic-subscription-token"));
    let text = "{\"type\":\"text-delta\",\"text\":\"ok\"}\n";
    let step = "{\"type\":\"finish-step\",\"finishReason\":\"stop\"}\n";
    let finish = "{\"type\":\"finish\",\"finishReason\":\"stop\"}\n";
    for (wire, success) in [
        (format!("{text}{step}"), true),
        (format!("{text}{finish}"), true),
        (format!("{text}{step}{finish}"), true),
        (format!("{text}{step}{step}"), false),
        (format!("{text}{finish}{finish}"), false),
        (format!("{text}{finish}{step}"), false),
        (format!("{finish}{text}"), false),
        (text.into(), false),
        ("junk\n".into(), false),
        ("null\n".into(), false),
        ("data: {}\n".into(), false),
        (
            "{\"type\":\"finish\",\"finishReason\":\"error\"}\n".into(),
            false,
        ),
        (
            "{\"type\":\"finish\",\"finishReason\":\"stop\"}".into(),
            false,
        ),
    ] {
        let (status, body) = turn(&wire, 200).await;
        assert_eq!(status.is_success(), success, "{wire}: {body}");
        if !success {
            assert_eq!(body["type"], "error");
        }
    }
}

#[tokio::test]
async fn command_code_tracer_env_auth() {
    let _lock = crate::config::CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    // Invalid explicit sources must never become eligible for future CLI fallback.
    for token in ["", "with space", "unsafe\r\nheader"] {
        let _key = EnvGuard::set(Some(token));
        let config = config("https://api.commandcode.ai");
        let (router, _, _) = crate::server::build_router(config).unwrap();
        use tower::ServiceExt;
        let response = router.oneshot(axum::http::Request::builder().method("POST").uri("/v1/messages")
            .header("content-type","application/json").body(axum::body::Body::from(json!({"model":"zai-org/GLM-5.3","messages":[{"role":"user","content":"fixture"}]}).to_string())).unwrap()).await.unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), 65536)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["error"]["type"], "authentication_error");
    }
}
