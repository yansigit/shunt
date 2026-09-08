//! Both products concurrently traverse one gateway with independent credentials.
use super::*;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Barrier,
    time::timeout,
};

struct ApiKeyGuard(Option<OsString>);
impl Drop for ApiKeyGuard {
    fn drop(&mut self) {
        match &self.0 {
            Some(value) => std::env::set_var("SHUNT_CC_DUAL_API", value),
            None => std::env::remove_var("SHUNT_CC_DUAL_API"),
        }
    }
}

async fn serve_product<S: AsyncRead + AsyncWrite + Unpin>(
    mut socket: S,
    subscription: bool,
    barrier: Arc<Barrier>,
) {
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
                    .filter_map(|line| line.split_once(':'))
                    .find(|(k, _)| k.eq_ignore_ascii_case(name))
                    .map(|(_, v)| v.trim())
            };
            let len: usize = header("content-length").unwrap().parse().unwrap();
            if bytes.len() < end + 4 + len {
                continue;
            }
            let body: Value = serde_json::from_slice(&bytes[end + 4..end + 4 + len]).unwrap();
            assert_eq!(header("x-api-key"), None);
            assert!(!head.contains("inbound-tripwire"));
            if subscription {
                assert!(head.starts_with("POST /alpha/generate "));
                assert_eq!(header("host"), Some("api.commandcode.ai"));
                assert_eq!(
                    header("authorization"),
                    Some("Bearer synthetic-subscription-token")
                );
                assert_eq!(header("x-command-code-version"), Some("0.52.1"));
                assert_eq!(body["params"]["model"], "zai-org/GLM-5.3");
                assert_eq!(body["params"]["stream"], true);
                assert!(body.get("model").is_none());
                assert!(!head.contains("synthetic-api-only"));
            } else {
                assert!(head.starts_with("POST /provider/v1/chat/completions "));
                assert_eq!(header("authorization"), Some("Bearer synthetic-api-only"));
                assert_eq!(header("x-command-code-version"), None);
                assert_eq!(header("x-session-id"), None);
                assert_eq!(body["model"], "fixture-api-model");
                assert!(body.get("params").is_none());
                assert!(!head.contains("synthetic-subscription-token"));
            }
            break;
        }
    }
    // Neither backend responds until both independent requests are in flight.
    timeout(Duration::from_secs(5), barrier.wait())
        .await
        .unwrap();
    let (content_type, body) = if subscription {
        ("application/x-ndjson", "{\"type\":\"text-delta\",\"text\":\"subscription-only\"}\n{\"type\":\"finish\",\"finishReason\":\"stop\"}\n".to_string())
    } else {
        ("application/json", json!({"id":"fixture-api","object":"chat.completion","model":"fixture-api-model","choices":[{"index":0,"message":{"role":"assistant","content":"api-only"},"finish_reason":"stop"}],"usage":{"prompt_tokens":1,"completion_tokens":2,"total_tokens":3}}).to_string())
    };
    socket.write_all(format!("HTTP/1.1 200 OK\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();
    socket.shutdown().await.unwrap();
}

#[tokio::test]
async fn command_code_matrix_two_products_concurrent_no_crossover() {
    let _lock = crate::config::CONFIG_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _sub_key = EnvGuard::set(Some("synthetic-subscription-token"));
    let _api_key = ApiKeyGuard(std::env::var_os("SHUNT_CC_DUAL_API"));
    std::env::set_var("SHUNT_CC_DUAL_API", "synthetic-api-only");
    let api = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let api_addr = api.local_addr().unwrap();
    let sub = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let sub_addr = sub.local_addr().unwrap();
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
    let client = reqwest::Client::builder()
        .no_proxy()
        .http1_only()
        .redirect(reqwest::redirect::Policy::none())
        .add_root_certificate(reqwest::Certificate::from_der(cert.cert.der()).unwrap())
        .resolve("api.commandcode.ai", sub_addr)
        .build()
        .unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let api_barrier = barrier.clone();
    let mut api_task = Task(tokio::spawn(async move {
        let (socket, _) = timeout(Duration::from_secs(5), api.accept())
            .await
            .unwrap()
            .unwrap();
        serve_product(socket, false, api_barrier).await;
    }));
    let mut sub_task = Task(tokio::spawn(async move {
        let (socket, _) = timeout(Duration::from_secs(5), sub.accept())
            .await
            .unwrap()
            .unwrap();
        let socket = TlsAcceptor::from(Arc::new(tls))
            .accept(socket)
            .await
            .unwrap();
        serve_product(socket, true, barrier).await;
    }));
    let mut cfg = config("https://api.commandcode.ai");
    cfg.providers.insert("api".into(), serde_json::from_value(json!({"kind":"openai_chat","auth":"api_key","api_key_env":"SHUNT_CC_DUAL_API","base_url":format!("http://{api_addr}/provider/v1")})).unwrap());
    cfg.routes = serde_json::from_value(json!([
        {"model":"api-alias","provider":"api","upstream_model":"fixture-api-model"},
        {"model":"subscription-alias","provider":"cc","upstream_model":"zai-org/GLM-5.3"}
    ]))
    .unwrap();
    let before = crate::auth::command_code::LOOKUPS.load(std::sync::atomic::Ordering::SeqCst);
    let (router, _, _) = crate::server::build_router_with_test_client(cfg, client).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/v1/messages", listener.local_addr().unwrap());
    let _gateway = Task(tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    }));
    let http = reqwest::Client::new();
    let request = |model| {
        http.post(&url)
            .timeout(Duration::from_secs(5))
            .header("authorization", "Bearer inbound-tripwire")
            .header("x-api-key", "inbound-tripwire")
            .json(&json!({"model":model,"messages":[{"role":"user","content":"fixture"}]}))
            .send()
    };
    let (api, sub) = tokio::join!(request("api-alias"), request("subscription-alias"));
    let (api, sub) = (api.unwrap(), sub.unwrap());
    assert_eq!(api.status(), reqwest::StatusCode::OK);
    assert_eq!(sub.status(), reqwest::StatusCode::OK);
    assert_eq!(
        api.json::<Value>().await.unwrap()["content"][0]["text"],
        "api-only"
    );
    assert_eq!(
        sub.json::<Value>().await.unwrap()["content"][0]["text"],
        "subscription-only"
    );
    timeout(Duration::from_secs(5), &mut api_task.0)
        .await
        .unwrap()
        .unwrap();
    timeout(Duration::from_secs(5), &mut sub_task.0)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        crate::auth::command_code::LOOKUPS.load(std::sync::atomic::Ordering::SeqCst),
        before + 1
    );
}
