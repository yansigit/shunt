use serde_json::{json, Value};
use shunt::{
    config::{AuthMode, Config, ProviderKind},
    server,
};
use std::{sync::OnceLock, time::Duration};
use tokio::sync::Mutex;
use wiremock::{
    matchers::{header, method, path},
    Mock, MockServer, ResponseTemplate,
};

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
async fn env_lock() -> tokio::sync::MutexGuard<'static, ()> {
    ENV_LOCK.get_or_init(|| Mutex::new(())).lock().await
}
struct Key(&'static str, Option<std::ffi::OsString>);
impl Key {
    fn set(name: &'static str, value: &str) -> Self {
        let previous = std::env::var_os(name);
        std::env::set_var(name, value);
        Self(name, previous)
    }
}
impl Drop for Key {
    fn drop(&mut self) {
        if let Some(value) = &self.1 {
            std::env::set_var(self.0, value);
        } else {
            std::env::remove_var(self.0);
        }
    }
}
struct Gateway {
    url: String,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for Gateway {
    fn drop(&mut self) {
        self.task.abort();
    }
}
async fn gateway(config: Config) -> Gateway {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (app, _, _) = server::build_router(config).unwrap();
    Gateway {
        url: format!("http://{addr}/v1/messages"),
        task: tokio::spawn(async move { axum::serve(listener, app).await.unwrap() }),
    }
}
fn fixture_config(base: &str, key: &str) -> Config {
    let mut config = preset_config(Some(&format!("{base}/provider/v1")));
    config.providers.get_mut("cc").unwrap().api_key_env = Some(key.into());
    config
}
fn request() -> Value {
    json!({"model":"fixture-model","max_tokens":64,"messages":[{"role":"user","content":"fixture"}]})
}
fn completion() -> Value {
    json!({"id":"chatcmpl-fixture","object":"chat.completion","created":1,"model":"fixture-model",
        "choices":[{"index":0,"message":{"role":"assistant","content":"fixture reply"},"finish_reason":"stop"}],
        "usage":{"prompt_tokens":5,"completion_tokens":2,"total_tokens":7}})
}
async fn send(gateway: &Gateway, request: &Value) -> reqwest::Response {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap()
        .post(&gateway.url)
        .json(request)
        .send()
        .await
        .unwrap()
}

fn preset_config(base: Option<&str>) -> Config {
    let mut config = Config::default();
    let mut upstream = json!({"name":"cc", "provider":"commandcode"});
    if let Some(base) = base {
        upstream["base_url"] = json!(base);
    }
    config.upstreams = serde_json::from_value(json!([upstream])).unwrap();
    config.server.default_provider = "cc".into();
    let result = config.validate();
    assert!(
        result.is_ok(),
        "commandcode preset must resolve: {:?}",
        result.err()
    );
    result.unwrap()
}

#[test]
fn command_code_api_preset_sibling_preservation() {
    let config = preset_config(None);
    let provider = &config.providers["cc"];
    assert_eq!(provider.kind, ProviderKind::OpenAiChat);
    assert_eq!(provider.auth, AuthMode::ApiKey);
    assert_eq!(provider.base_url, "https://api.commandcode.ai/provider/v1");
    assert_eq!(
        provider.api_key_env.as_deref(),
        Some("SHUNT_COMMANDCODE_API_KEY")
    );
    let names: Vec<_> = shunt::config::provider_presets().map(|p| p.name).collect();
    assert_eq!(
        &names[..10],
        &[
            "anthropic",
            "codex",
            "openai",
            "xai",
            "grok",
            "kimi",
            "cursor",
            "kimi-code",
            "zhipu",
            "minimax-cn"
        ]
    );
    for (name, kind, base, auth, env) in [
        (
            "anthropic",
            ProviderKind::Anthropic,
            "https://api.anthropic.com",
            AuthMode::Passthrough,
            None,
        ),
        (
            "codex",
            ProviderKind::Responses,
            "https://chatgpt.com/backend-api",
            AuthMode::ChatgptOauth,
            None,
        ),
        (
            "openai",
            ProviderKind::Responses,
            "https://api.openai.com/v1",
            AuthMode::ApiKey,
            Some("OPENAI_API_KEY"),
        ),
        (
            "xai",
            ProviderKind::Responses,
            "https://api.x.ai/v1",
            AuthMode::ApiKey,
            Some("XAI_API_KEY"),
        ),
        (
            "grok",
            ProviderKind::Responses,
            "https://cli-chat-proxy.grok.com/v1",
            AuthMode::XaiOauth,
            None,
        ),
        (
            "kimi",
            ProviderKind::Anthropic,
            "https://api.moonshot.ai/anthropic",
            AuthMode::ApiKey,
            Some("MOONSHOT_API_KEY"),
        ),
        (
            "cursor",
            ProviderKind::Cursor,
            "https://api2.cursor.sh",
            AuthMode::CursorOauth,
            None,
        ),
        (
            "kimi-code",
            ProviderKind::Anthropic,
            "https://api.kimi.com/coding",
            AuthMode::KimiOauth,
            None,
        ),
        (
            "zhipu",
            ProviderKind::Anthropic,
            "https://open.bigmodel.cn/api/anthropic",
            AuthMode::ApiKey,
            Some("ZHIPUAI_API_KEY"),
        ),
        (
            "minimax-cn",
            ProviderKind::Anthropic,
            "https://api.minimax.cn/anthropic",
            AuthMode::ApiKey,
            Some("MINIMAX_API_KEY"),
        ),
    ] {
        let mut sibling = Config {
            upstreams: serde_json::from_value(json!([{"name":"sibling","provider":name}])).unwrap(),
            ..Config::default()
        };
        sibling.server.default_provider = "sibling".into();
        let sibling = sibling.validate().unwrap();
        let p = &sibling.providers["sibling"];
        assert_eq!(
            (
                p.kind,
                p.base_url.as_str(),
                p.auth,
                p.api_key_env.as_deref()
            ),
            (kind, base, auth, env),
            "{name}"
        );
    }
}

#[tokio::test]
async fn command_code_api_scenarios_tools_and_long_context() {
    let _lock = env_lock().await;
    let _key = Key::set("SHUNT_CC_SCENARIO_FIXTURE", "synthetic-scenario-key");
    let backend = MockServer::start().await;
    let mut reply = completion();
    reply["choices"][0]["message"] = json!({"role":"assistant","content":null,"tool_calls":[
        {"id":"next-a","type":"function","function":{"name":"f","arguments":"{\"v\":1}"}},
        {"id":"next-b","type":"function","function":{"name":"f","arguments":"{\"v\":2}"}}]});
    reply["choices"][0]["finish_reason"] = json!("tool_calls");
    Mock::given(method("POST"))
        .and(path("/provider/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(reply))
        .expect(1)
        .mount(&backend)
        .await;
    let gateway = gateway(fixture_config(&backend.uri(), "SHUNT_CC_SCENARIO_FIXTURE")).await;
    let text = "context-é".repeat(16000);
    let mut req = request();
    req["tools"] = json!([{"name":"f","description":"fixture","input_schema":{"type":"object","properties":{"v":{"type":"integer"}}}}]);
    req["messages"] = json!([
        {"role":"user","content":text},
        {"role":"assistant","content":[{"type":"tool_use","id":"prior","name":"f","input":{"v":0}}]},
        {"role":"user","content":[{"type":"tool_result","tool_use_id":"prior","content":"recorded"}]}]);
    let response = send(&gateway, &req).await;
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["stop_reason"], "tool_use");
    let tools: Vec<_> = body["content"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|b| b["type"] == "tool_use")
        .collect();
    assert_eq!(tools.len(), 2);
    assert_eq!(
        (tools[0]["id"].as_str(), tools[1]["id"].as_str()),
        (Some("next-a"), Some("next-b"))
    );
    assert_eq!(tools[0]["input"], json!({"v":1}));
    assert_eq!(tools[1]["input"], json!({"v":2}));
    let captured = backend.received_requests().await.unwrap();
    let wire: Value = serde_json::from_slice(&captured[0].body).unwrap();
    assert!(
        wire["messages"][0]["content"] == text || wire["messages"][0]["content"][0]["text"] == text
    );
    assert_eq!(wire["messages"][2]["tool_call_id"], "prior");
}

#[tokio::test]
async fn command_code_api_scenarios_stream_terminal() {
    let _lock = env_lock().await;
    let _key = Key::set("SHUNT_CC_STREAM_FIXTURE", "synthetic-stream-key");
    let backend = MockServer::start().await;
    let chunks = [
        json!({"choices":[{"index":0,"delta":{"content":"hello"}}]}),
        json!({"choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}),
    ];
    let wire = format!(
        "data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
        chunks[0], chunks[1]
    );
    Mock::given(method("POST"))
        .and(path("/provider/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(wire),
        )
        .expect(1)
        .mount(&backend)
        .await;
    let gateway = gateway(fixture_config(&backend.uri(), "SHUNT_CC_STREAM_FIXTURE")).await;
    let mut req = request();
    req["stream"] = json!(true);
    let response = send(&gateway, &req).await;
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let text = response.text().await.unwrap();
    assert!(text.contains("hello"));
    assert_eq!(text.matches("event: message_start\n").count(), 1);
    assert_eq!(text.matches("event: message_stop\n").count(), 1);
    assert!(!text.contains("event: error"));
}

struct AbortOnDrop(tokio::task::JoinHandle<()>);
impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

async fn read_wire_request(socket: &mut tokio::net::TcpStream) {
    use tokio::io::AsyncReadExt;
    let mut bytes = Vec::new();
    loop {
        let mut buf = [0; 2048];
        let n = tokio::time::timeout(Duration::from_secs(5), socket.read(&mut buf))
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
            assert!(head.starts_with("post /provider/v1/chat/completions "));
            assert!(head.contains("authorization: bearer synthetic-drop-key"));
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

#[tokio::test]
async fn command_code_api_scenarios_incremental_drop_releases_capacity() {
    use std::sync::Arc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::time::timeout;
    let _lock = env_lock().await;
    let _key = Key::set("SHUNT_CC_DROP_FIXTURE", "synthetic-drop-key");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let closed = Arc::new(tokio::sync::Notify::new());
    let signal = closed.clone();
    let deadline = Duration::from_secs(5);
    let mut backend = AbortOnDrop(tokio::spawn(async move {
        let (mut socket, _) = timeout(deadline, listener.accept()).await.unwrap().unwrap();
        read_wire_request(&mut socket).await;
        let frame = format!(
            "data: {}\n\n",
            json!({"choices":[{"index":0,"delta":{"content":"early-fixture"}}]})
        );
        socket.write_all(format!("HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\ntransfer-encoding: chunked\r\n\r\n{:x}\r\n{frame}\r\n",frame.len()).as_bytes()).await.unwrap();
        socket.flush().await.unwrap();
        // Do not send a finish: downstream must see early text before EOF.
        let mut byte = [0];
        assert_eq!(
            timeout(deadline, socket.read(&mut byte))
                .await
                .expect("upstream closes on downstream drop")
                .unwrap(),
            0
        );
        signal.notify_one();
        let (mut next, _) = timeout(deadline, listener.accept()).await.unwrap().unwrap();
        read_wire_request(&mut next).await;
        let body = completion().to_string();
        next.write_all(format!("HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();
    }));
    let mut config = fixture_config(&format!("http://{addr}"), "SHUNT_CC_DROP_FIXTURE");
    config.server.max_concurrent_requests = 1;
    let gateway = gateway(config).await;
    let mut client = tokio::net::TcpStream::connect(
        gateway
            .url
            .trim_start_matches("http://")
            .trim_end_matches("/v1/messages"),
    )
    .await
    .unwrap();
    let mut req = request();
    req["stream"] = json!(true);
    let body = req.to_string();
    client.write_all(format!("POST /v1/messages HTTP/1.1\r\nHost: localhost\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();
    let mut observed = Vec::new();
    timeout(deadline, async {
        loop {
            let mut buf = [0; 2048];
            let n = client.read(&mut buf).await.unwrap();
            assert!(n > 0);
            observed.extend_from_slice(&buf[..n]);
            assert!(observed.len() < 65536);
            if String::from_utf8_lossy(&observed).contains("early-fixture") {
                break;
            }
        }
    })
    .await
    .expect("text relayed before upstream finish");
    assert_eq!(
        send(&gateway, &request()).await.status(),
        reqwest::StatusCode::SERVICE_UNAVAILABLE
    );
    drop(client);
    timeout(deadline, closed.notified())
        .await
        .expect("upstream closure observed");
    let response = send(&gateway, &request()).await;
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        response.json::<Value>().await.unwrap()["content"][0]["text"],
        "fixture reply"
    );
    timeout(deadline, &mut backend.0)
        .await
        .expect("owned backend joined")
        .unwrap();
}

async fn tracer_case() {
    let _lock = env_lock().await;
    let backend = MockServer::start().await;
    let mut config = preset_config(Some(&format!("{}/provider/v1", backend.uri())));
    // Use an isolated fixture env rather than any configured user key.
    config.providers.get_mut("cc").unwrap().api_key_env =
        Some("SHUNT_CC_API_TRACER_FIXTURE".into());
    let _key = Key::set("SHUNT_CC_API_TRACER_FIXTURE", "synthetic-cc-key");
    Mock::given(method("POST"))
        .and(path("/provider/v1/chat/completions"))
        .and(header("authorization", "Bearer synthetic-cc-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"chatcmpl-fixture", "object":"chat.completion", "created":1, "model":"fixture-model",
            "choices":[{"index":0,"message":{"role":"assistant","content":"fixture reply"},"finish_reason":"stop"}],
            "usage":{"prompt_tokens":5,"completion_tokens":2,"total_tokens":7}
        }))).expect(1).mount(&backend).await;
    let gateway = gateway(config).await;
    let response = send(&gateway, &request()).await;
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["content"][0]["text"], "fixture reply");
}

#[tokio::test]
async fn command_code_api_preset_tracer() {
    tracer_case().await;
}

#[tokio::test]
async fn command_code_api_tracer_unary() {
    tracer_case().await;
}

#[tokio::test]
async fn command_code_api_tracer_auth_error() {
    let _lock = env_lock().await;
    let _key = Key::set("SHUNT_CC_ERROR_FIXTURE", "synthetic-error-key");
    let backend = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/provider/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(401)
                .set_body_json(json!({"error":{"message":"upstream-secret-detail"}})),
        )
        .expect(1)
        .mount(&backend)
        .await;
    let gateway = gateway(fixture_config(&backend.uri(), "SHUNT_CC_ERROR_FIXTURE")).await;
    let response = send(&gateway, &request()).await;
    assert!(!response.status().is_success());
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["type"], "error");
    assert!(body["error"]["type"].is_string());
    assert!(!body.to_string().contains("synthetic-error-key"));
    assert_eq!(backend.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn command_code_api_tracer_isolation() {
    let _lock = env_lock().await;
    let _a = Key::set("SHUNT_CC_ISOLATION_A", "synthetic-key-a");
    let _b = Key::set("SHUNT_CC_ISOLATION_B", "synthetic-key-b");
    let a = MockServer::start().await;
    let b = MockServer::start().await;
    for (backend, key) in [(&a, "synthetic-key-a"), (&b, "synthetic-key-b")] {
        Mock::given(method("POST"))
            .and(path("/provider/v1/chat/completions"))
            .and(header("authorization", format!("Bearer {key}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(completion()))
            .expect(1)
            .mount(backend)
            .await;
    }
    let mut config = fixture_config(&a.uri(), "SHUNT_CC_ISOLATION_A");
    let mut invalid = config.clone();
    invalid.providers.get_mut("cc").unwrap().auth = AuthMode::ChatgptOauth;
    assert!(
        invalid.validate().is_err(),
        "Chat kind must reject subscription auth"
    );
    let mut second = config.providers["cc"].clone();
    second.base_url = format!("{}/provider/v1", b.uri());
    second.api_key_env = Some("SHUNT_CC_ISOLATION_B".into());
    config.providers.insert("other".into(), second);
    config.routes = serde_json::from_value(json!([
        {"model":"a","provider":"cc","upstream_model":"fixture-model"},
        {"model":"b","provider":"other","upstream_model":"fixture-model"}]))
    .unwrap();
    let gateway = gateway(config).await;
    let mut ra = request();
    ra["model"] = json!("a");
    let mut rb = request();
    rb["model"] = json!("b");
    let (ra, rb) = tokio::join!(send(&gateway, &ra), send(&gateway, &rb));
    assert_eq!(ra.status(), reqwest::StatusCode::OK);
    assert_eq!(rb.status(), reqwest::StatusCode::OK);
}
