//! Canonical TLS replay characterization; all identities are synthetic.
use super::*;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use std::{
    net::SocketAddr,
    sync::atomic::{AtomicUsize, Ordering},
};
use tokio::time::timeout;

struct RetryDns {
    calls: Arc<AtomicUsize>,
    bad: SocketAddr,
    good: SocketAddr,
    fail_first: bool,
}
impl Resolve for RetryDns {
    fn resolve(&self, name: Name) -> Resolving {
        assert_eq!(name.as_str(), "api.commandcode.ai");
        let first = self.calls.fetch_add(1, Ordering::SeqCst) == 0;
        let addr = if first && self.fail_first {
            // The request has already captured its credential and session.
            // Changing the synthetic source cannot change a safe retry.
            std::env::set_var("SHUNT_COMMAND_CODE_TOKEN", "synthetic-changed-token");
            self.bad
        } else {
            self.good
        };
        let addrs: Addrs = Box::new(std::iter::once(addr));
        Box::pin(async move { Ok(addrs) })
    }
}

async fn replay_case(case: &'static str) {
    let _lock = crate::config::CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _key = EnvGuard::set(Some("synthetic-subscription-token"));
    let before = crate::auth::command_code::LOOKUPS.load(Ordering::SeqCst);
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
    let refused = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let bad = refused.local_addr().unwrap();
    drop(refused);
    let calls = Arc::new(AtomicUsize::new(0));
    let client = reqwest::Client::builder()
        .no_proxy()
        .http1_only()
        .redirect(reqwest::redirect::Policy::none())
        .add_root_certificate(reqwest::Certificate::from_der(cert.cert.der()).unwrap())
        .dns_resolver(Arc::new(RetryDns {
            calls: calls.clone(),
            bad,
            good: listener.local_addr().unwrap(),
            fail_first: case == "connect",
        }))
        .build()
        .unwrap();
    let mut backend = Task(tokio::spawn(async move {
        let (socket, _) = timeout(Duration::from_secs(5), listener.accept())
            .await
            .unwrap()
            .unwrap();
        let mut socket = TlsAcceptor::from(Arc::new(tls))
            .accept(socket)
            .await
            .unwrap();
        let mut bytes = Vec::new();
        loop {
            let mut buf = [0; 2048];
            let n = timeout(Duration::from_secs(5), socket.read(&mut buf))
                .await
                .unwrap()
                .unwrap();
            assert!(n > 0);
            bytes.extend_from_slice(&buf[..n]);
            assert!(bytes.len() < 65536);
            if let Some(end) = bytes.windows(4).position(|p| p == b"\r\n\r\n") {
                let head = std::str::from_utf8(&bytes[..end]).unwrap();
                let header = |name: &str| {
                    head.lines()
                        .filter_map(|l| l.split_once(':'))
                        .find(|(k, _)| k.eq_ignore_ascii_case(name))
                        .map(|(_, v)| v.trim())
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
                let expected = crate::adapters::command_code::request::session_id(
                    "synthetic-subscription-token",
                    Some("retry-conversation"),
                )
                .unwrap();
                assert_eq!(header("x-session-id"), Some(expected.as_str()));
                break;
            }
        }
        if case == "timeout" {
            let mut byte = [0];
            let result = timeout(Duration::from_secs(5), socket.read(&mut byte))
                .await
                .unwrap();
            assert!(
                matches!(result, Ok(0) | Err(_)),
                "request must be cancelled on TTFB deadline"
            );
        } else {
            let (status, location, body) = match case {
                "redirect" => (302, "location: https://api.commandcode.ai/alpha/generate\r\n", ""),
                "output" => (200, "", "{\"type\":\"text-delta\",\"text\":\"partial\"}\n"),
                "tool" => (200, "", "{\"type\":\"tool-call\",\"toolCallId\":\"a\",\"toolName\":\"f\",\"input\":{}}\n"),
                _ => (200, "", "{\"type\":\"finish\",\"finishReason\":\"stop\"}\n"),
            };
            socket.write_all(format!("HTTP/1.1 {status} Fixture\r\ncontent-type: application/x-ndjson\r\ncontent-length: {}\r\n{location}connection: close\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
            let _ = socket.shutdown().await;
        }
        // The configured retry delay is at most 1ms; this window also covers
        // any immediate redirected connection. No alternate destination used.
        assert!(
            timeout(Duration::from_millis(150), listener.accept())
                .await
                .is_err(),
            "unexpected generation replay"
        );
    }));
    let mut cfg = config("https://api.commandcode.ai");
    let retry = &mut cfg.providers.get_mut("cc").unwrap().retry;
    retry.max_retries = 1;
    retry.initial_backoff_ms = 1;
    retry.max_backoff_ms = 1;
    cfg.server.timeouts.upstream_ttfb_ms = if case == "timeout" { 250 } else { 5000 };
    let (router, _, _) = crate::server::build_router_with_test_client(cfg, client).unwrap();
    use tower::ServiceExt;
    let streaming = matches!(case, "output" | "tool");
    let response = router.oneshot(axum::http::Request::builder().method("POST").uri("/v1/messages")
        .header("content-type", "application/json").header("x-claude-code-session-id", "retry-conversation")
        .body(axum::body::Body::from(json!({"model":"zai-org/GLM-5.3","stream":streaming,"messages":[{"role":"user","content":"fixture"}]}).to_string())).unwrap()).await.unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), 65536)
        .await
        .unwrap();
    if streaming {
        assert_eq!(status, reqwest::StatusCode::OK);
        let text = std::str::from_utf8(&body).unwrap();
        assert_eq!(text.matches("event: error\n").count(), 1);
        assert_eq!(text.matches("event: message_stop\n").count(), 0);
        assert!(text.contains("content_block_start"));
    } else {
        assert_eq!(
            status.as_u16(),
            match case {
                "connect" => 200,
                "timeout" => 504,
                _ => 502,
            }
        );
    }
    timeout(Duration::from_secs(5), &mut backend.0)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        calls.load(Ordering::SeqCst),
        if case == "connect" { 2 } else { 1 }
    );
    assert_eq!(
        crate::auth::command_code::LOOKUPS.load(Ordering::SeqCst),
        before + 1
    );
}

#[tokio::test]
async fn command_code_lifetime_replay_connect_retains_identity() {
    replay_case("connect").await;
}
#[tokio::test]
async fn command_code_lifetime_replay_post_send_timeout_once() {
    replay_case("timeout").await;
}
#[tokio::test]
async fn command_code_lifetime_replay_redirect_never_followed() {
    replay_case("redirect").await;
}
#[tokio::test]
async fn command_code_lifetime_replay_output_and_tools_never_repeat() {
    replay_case("output").await;
    replay_case("tool").await;
}
