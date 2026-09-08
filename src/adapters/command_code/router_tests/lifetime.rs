//! Real-socket lifetime characterization; synthetic credentials and private TLS.
use super::*;
use tokio::time::timeout;

const DEADLINE: Duration = Duration::from_secs(5);

async fn request_read<S: AsyncReadExt + Unpin>(socket: &mut S) {
    let mut bytes = Vec::new();
    loop {
        let mut buf = [0; 2048];
        let n = timeout(DEADLINE, socket.read(&mut buf))
            .await
            .unwrap()
            .unwrap();
        assert!(n > 0);
        bytes.extend_from_slice(&buf[..n]);
        assert!(bytes.len() < 65536);
        if let Some(end) = bytes.windows(4).position(|p| p == b"\r\n\r\n") {
            let head = std::str::from_utf8(&bytes[..end])
                .unwrap()
                .to_ascii_lowercase();
            assert!(head.starts_with("post /alpha/generate "));
            assert!(head.contains("authorization: bearer synthetic-subscription-token"));
            let len: usize = head
                .lines()
                .find_map(|l| l.strip_prefix("content-length:"))
                .unwrap()
                .trim()
                .parse()
                .unwrap();
            if bytes.len() >= end + 4 + len {
                return;
            }
        }
    }
}

async fn cancellation(before_headers: bool, streaming: bool) {
    let _lock = crate::config::CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _key = EnvGuard::set(Some("synthetic-subscription-token"));
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
    let client = reqwest::Client::builder()
        .no_proxy()
        .http1_only()
        .redirect(reqwest::redirect::Policy::none())
        .add_root_certificate(reqwest::Certificate::from_der(cert.cert.der()).unwrap())
        .resolve("api.commandcode.ai", listener.local_addr().unwrap())
        .build()
        .unwrap();
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    let (closed_tx, closed_rx) = tokio::sync::oneshot::channel();
    let mut backend = Task(tokio::spawn(async move {
        let acceptor = TlsAcceptor::from(Arc::new(tls));
        let (socket, _) = timeout(DEADLINE, listener.accept()).await.unwrap().unwrap();
        let mut first = acceptor.accept(socket).await.unwrap();
        request_read(&mut first).await;
        if !before_headers {
            let record = "{\"type\":\"text-delta\",\"text\":\"early\"}\n";
            first.write_all(format!("HTTP/1.1 200 OK\r\ncontent-type: application/x-ndjson\r\ntransfer-encoding: chunked\r\n\r\n{:x}\r\n{record}\r\n",record.len()).as_bytes()).await.unwrap();
            first.flush().await.unwrap();
        }
        ready_tx.send(()).unwrap();
        let mut byte = [0];
        match timeout(DEADLINE, first.read(&mut byte))
            .await
            .expect("upstream must close after downstream disconnect")
        {
            Ok(0) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::UnexpectedEof
                        | std::io::ErrorKind::ConnectionReset
                        | std::io::ErrorKind::ConnectionAborted
                ) => {}
            result => panic!("expected socket closure, got {result:?}"),
        }
        closed_tx.send(()).unwrap();
        let (socket, _) = timeout(DEADLINE, listener.accept()).await.unwrap().unwrap();
        let mut second = acceptor.accept(socket).await.unwrap();
        request_read(&mut second).await;
        let body = "{\"type\":\"text-delta\",\"text\":\"follow-up\"}\n{\"type\":\"finish\",\"finishReason\":\"stop\"}\n";
        second.write_all(format!("HTTP/1.1 200 OK\r\ncontent-type: application/x-ndjson\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();
        second.shutdown().await.unwrap();
    }));
    let mut cfg = config("https://api.commandcode.ai");
    cfg.server.max_concurrent_requests = 1;
    let (router, _, _) = crate::server::build_router_with_test_client(cfg, client).unwrap();
    let gateway = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = gateway.local_addr().unwrap();
    let _serving = Task(tokio::spawn(async move {
        axum::serve(gateway, router).await.unwrap();
    }));
    let body = json!({"model":"zai-org/GLM-5.3","stream":streaming,"messages":[{"role":"user","content":"fixture"}]}).to_string();
    let mut downstream = tokio::net::TcpStream::connect(addr).await.unwrap();
    downstream.write_all(format!("POST /v1/messages HTTP/1.1\r\nHost: localhost\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();
    timeout(DEADLINE, ready_rx).await.unwrap().unwrap();
    if streaming && !before_headers {
        timeout(DEADLINE, async {
            let mut seen = Vec::new();
            loop {
                let mut buf = [0; 2048];
                let n = downstream.read(&mut buf).await.unwrap();
                assert!(n > 0);
                seen.extend_from_slice(&buf[..n]);
                assert!(seen.len() < 65536);
                if seen.windows(5).any(|w| w == b"early") {
                    break;
                }
            }
        })
        .await
        .expect("observe output before cancellation");
    }
    let followup =
        json!({"model":"zai-org/GLM-5.3","messages":[{"role":"user","content":"fixture"}]});
    let http = reqwest::Client::new();
    let url = format!("http://{addr}/v1/messages");
    let saturated = http
        .post(&url)
        .timeout(DEADLINE)
        .json(&followup)
        .send()
        .await
        .unwrap();
    assert_eq!(saturated.status(), reqwest::StatusCode::SERVICE_UNAVAILABLE);
    drop(downstream);
    timeout(DEADLINE, closed_rx)
        .await
        .expect("upstream closure observed")
        .unwrap();
    let response = http
        .post(url)
        .timeout(DEADLINE)
        .json(&followup)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        response.json::<Value>().await.unwrap()["content"][0]["text"],
        "follow-up"
    );
    timeout(DEADLINE, &mut backend.0).await.unwrap().unwrap();
}

#[tokio::test]
async fn command_code_lifetime_cancel_before_headers_unary() {
    cancellation(true, false).await;
}
#[tokio::test]
async fn command_code_lifetime_cancel_before_headers_streaming() {
    cancellation(true, true).await;
}
#[tokio::test]
async fn command_code_lifetime_cancel_mid_body_unary() {
    cancellation(false, false).await;
}
#[tokio::test]
async fn command_code_lifetime_cancel_mid_body_streaming() {
    cancellation(false, true).await;
}
