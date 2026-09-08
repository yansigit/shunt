//! Source-derived synthetic fixtures: 14-PROTOCOL-EVIDENCE.md, not live captures.
#![allow(clippy::await_holding_lock)]

use std::{ffi::OsString, sync::Arc, time::Duration};
use serde_json::{json, Value};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener};
use tokio_rustls::{rustls, TlsAcceptor};
use crate::config::Config;

fn config(base: &str) -> Config {
    let mut value = serde_json::to_value(Config::default()).unwrap();
    value["providers"]["cc"] = json!({"kind":"command_code","auth":"command_code_oauth","base_url":base});
    value["server"]["default_provider"] = json!("cc");
    let parsed = serde_json::from_value::<Config>(value);
    assert!(parsed.is_ok(), "subscription kind/auth must be admitted: {:?}", parsed.as_ref().err());
    parsed.unwrap()
}

struct EnvGuard(Option<OsString>);
impl EnvGuard {
    fn set(value: Option<&str>) -> Self {
        let old = std::env::var_os("SHUNT_COMMAND_CODE_TOKEN");
        if let Some(value) = value { std::env::set_var("SHUNT_COMMAND_CODE_TOKEN", value); }
        else { std::env::remove_var("SHUNT_COMMAND_CODE_TOKEN"); }
        Self(old)
    }
}
impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(value) = &self.0 { std::env::set_var("SHUNT_COMMAND_CODE_TOKEN", value); }
        else { std::env::remove_var("SHUNT_COMMAND_CODE_TOKEN"); }
    }
}
struct Task(tokio::task::JoinHandle<()>);
impl Drop for Task { fn drop(&mut self) { self.0.abort(); } }

async fn turn(wire: &str, status: u16) -> (reqwest::StatusCode, Value) {
    let config = config("https://api.commandcode.ai");
    let cert = rcgen::generate_simple_self_signed(vec!["api.commandcode.ai".into()]).unwrap();
    let key = rustls::pki_types::PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());
    let tls = rustls::ServerConfig::builder_with_provider(Arc::new(rustls::crypto::aws_lc_rs::default_provider()))
        .with_safe_default_protocol_versions().unwrap().with_no_client_auth()
        .with_single_cert(vec![cert.cert.der().clone()],key.into()).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let client = reqwest::Client::builder().no_proxy().http1_only()
        .redirect(reqwest::redirect::Policy::none())
        .add_root_certificate(reqwest::Certificate::from_der(cert.cert.der()).unwrap())
        .resolve("api.commandcode.ai",addr).timeout(Duration::from_secs(5)).build().unwrap();
    let wire = wire.to_string();
    let mut upstream = Task(tokio::spawn(async move {
        let (socket,_) = listener.accept().await.unwrap();
        let mut socket = TlsAcceptor::from(Arc::new(tls)).accept(socket).await.unwrap();
        let mut bytes = Vec::new();
        loop {
            let mut buf = [0;2048];
            let n = socket.read(&mut buf).await.unwrap(); assert!(n > 0);
            bytes.extend_from_slice(&buf[..n]); assert!(bytes.len() < 65536);
            if let Some(end) = bytes.windows(4).position(|p| p == b"\r\n\r\n") {
                let head = std::str::from_utf8(&bytes[..end]).unwrap().to_ascii_lowercase();
                let len: usize = head.lines().find_map(|l| l.strip_prefix("content-length:")).unwrap().trim().parse().unwrap();
                if bytes.len() < end + 4 + len { continue; }
                assert!(head.starts_with("post /alpha/generate "));
                assert!(head.contains("authorization: bearer synthetic-subscription-token"));
                assert!(head.contains("host: api.commandcode.ai"));
                let body: Value = serde_json::from_slice(&bytes[end+4..end+4+len]).unwrap();
                assert_eq!(body["params"]["stream"],true);
                assert_eq!(body["params"]["model"],"zai-org/GLM-5.3");
                break;
            }
        }
        let location = if status == 302 { "location: https://redirect.invalid/steal\r\n" } else { "" };
        socket.write_all(format!("HTTP/1.1 {status} Fixture\r\ncontent-type: application/x-ndjson\r\ncontent-length: {}\r\n{location}connection: close\r\n\r\n{wire}",wire.len()).as_bytes()).await.unwrap();
    }));
    let (router,_,_) = crate::server::build_router_with_test_client(config,client).unwrap();
    let gateway = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = gateway.local_addr().unwrap();
    let _serving = Task(tokio::spawn(async move { axum::serve(gateway,router).await.unwrap(); }));
    let response = reqwest::Client::new().post(format!("http://{addr}/v1/messages"))
        .timeout(Duration::from_secs(5)).json(&json!({"model":"zai-org/GLM-5.3","max_tokens":64,"messages":[{"role":"user","content":"fixture"}]}))
        .send().await.unwrap();
    let status = response.status();
    let body = response.json().await.unwrap();
    tokio::time::timeout(Duration::from_secs(5),&mut upstream.0).await.unwrap().unwrap();
    (status,body)
}

#[tokio::test]
async fn command_code_tracer_unary() {
    let _lock = crate::config::CONFIG_ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let _key = EnvGuard::set(Some("synthetic-subscription-token"));
    let (status,body) = turn("{\"type\":\"text-delta\",\"text\":\"fixture reply\"}\n{\"type\":\"finish\",\"finishReason\":\"stop\",\"totalUsage\":{\"inputTokens\":5,\"outputTokens\":2}}\n",200).await;
    assert_eq!(status,reqwest::StatusCode::OK);
    assert_eq!(body["content"][0]["text"],"fixture reply");
    assert_eq!(body["usage"]["input_tokens"],5);
}
