use std::{io::ErrorKind, net::SocketAddr, sync::OnceLock};

use reqwest::StatusCode;
use serde_json::{json, Value};
use shunt::{
    config::{Config, ConfigError},
    server,
};
use tokio::sync::Mutex;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn openai_chat_auth_redirect_refusal() {
    assert!(can_bind_loopback());
    let _lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let target = MockServer::start().await;
    let backend = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(307)
                .insert_header("location", format!("{}/chat/completions", target.uri())),
        )
        .mount(&backend)
        .await;
    let gateway = start_gateway(single_provider_config(&backend.uri())).await;
    let response = reqwest::Client::new()
        .post(format!("{}/v1/messages", gateway.base_url))
        .body(anthropic_request("claude-via-chat"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert_eq!(response.json::<Value>().await.unwrap()["type"], "error");
    assert_eq!(backend.received_requests().await.unwrap().len(), 1);
    assert!(target.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn openai_chat_auth_postsend_timeout_single_attempt() {
    assert!(can_bind_loopback());
    let _lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let backend = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(std::time::Duration::from_millis(200))
                .set_body_json(chat_completion_upstream()),
        )
        .mount(&backend)
        .await;
    let mut config = single_provider_config(&backend.uri());
    config.server.timeouts.upstream_ttfb_ms = 40;
    config
        .providers
        .get_mut("openai-chat-test")
        .unwrap()
        .retry
        .max_retries = 3;
    let gateway = start_gateway(config).await;
    let response = reqwest::Client::new()
        .post(format!("{}/v1/messages", gateway.base_url))
        .body(anthropic_request("claude-via-chat"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
    assert_eq!(response.json::<Value>().await.unwrap()["type"], "error");
    assert_eq!(backend.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn openai_chat_assembly_interleave_at_limit_multibyte() {
    assert!(can_bind_loopback());
    let _lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let args = format!("{{\"v\":\"{}é\"}}", "x".repeat(1024 * 1024 - 10));
    assert_eq!(args.len(), 1024 * 1024);
    let events = openai_chat_terminal_stream_events(vec![
        chat_delta(json!({"tool_calls":[{"index":9}]}), None),
        chat_delta(
            json!({"tool_calls":[{"index":2,"id":"b","function":{"name":"g","arguments":"{"}}]}),
            None,
        ),
        chat_delta(
            json!({"tool_calls":[{"index":9,"id":"a","function":{"name":"f","arguments":args}}]}),
            None,
        ),
        chat_delta(
            json!({"tool_calls":[{"index":2,"function":{"arguments":"}"}}]}),
            None,
        ),
        chat_delta(json!({}), Some("tool_calls")),
        "[DONE]".into(),
    ])
    .await;
    assert!(!events.iter().any(|(name, _)| name == "error"));
    let starts: Vec<_> = events
        .iter()
        .filter(|(name, data)| {
            name == "content_block_start" && data["content_block"]["type"] == "tool_use"
        })
        .collect();
    assert_eq!(starts.len(), 2);
    assert_eq!(starts[0].1["content_block"]["id"], "a");
    assert_eq!(starts[1].1["content_block"]["id"], "b");
    let deltas: Vec<_> = events
        .iter()
        .filter(|(_, data)| data["delta"]["type"] == "input_json_delta")
        .collect();
    let first: Value =
        serde_json::from_str(deltas[0].1["delta"]["partial_json"].as_str().unwrap()).unwrap();
    assert_eq!(first["v"].as_str().unwrap().len(), 1024 * 1024 - 8);
    assert_eq!(
        events
            .iter()
            .filter(|(name, _)| name == "message_stop")
            .count(),
        1
    );
}

struct EnvVarGuard {
    key: &'static str,
    previous: Option<std::ffi::OsString>,
}

#[tokio::test]
async fn openai_chat_auth_concurrent_keys_survive_no_redirect() {
    assert!(can_bind_loopback());
    let _lock = lock_openai_chat_env().await;
    let _a = EnvVarGuard::set("SHUNT_CHAT_REDIRECT_A", "fixture-key-a");
    let _b = EnvVarGuard::set("SHUNT_CHAT_REDIRECT_B", "fixture-key-b");
    let backend = MockServer::start().await;
    let target = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(307).insert_header("location", target.uri()))
        .mount(&backend)
        .await;
    let config = config_with_openai_chat_providers(
        json!({"a":openai_chat_provider(&backend.uri(),"SHUNT_CHAT_REDIRECT_A"),"b":openai_chat_provider(&backend.uri(),"SHUNT_CHAT_REDIRECT_B")}),
        json!([{"model":"model-a","provider":"a","upstream_model":"gpt-a"},{"model":"model-b","provider":"b","upstream_model":"gpt-b"}]),
    );
    let gateway = start_gateway(config).await;
    let client = reqwest::Client::new();
    let send = |model| {
        client
            .post(format!("{}/v1/messages", gateway.base_url))
            .header("x-api-key", "caller-secret")
            .header("authorization", "Bearer caller-secret")
            .body(anthropic_request(model))
            .send()
    };
    let (a, b) = tokio::join!(send("model-a"), send("model-b"));
    assert_eq!(a.unwrap().status(), StatusCode::BAD_GATEWAY);
    assert_eq!(b.unwrap().status(), StatusCode::BAD_GATEWAY);
    let requests = backend.received_requests().await.unwrap();
    let mut keys: Vec<_> = requests
        .iter()
        .map(|r| {
            assert!(!r.headers.contains_key("x-api-key"));
            r.headers["authorization"].to_str().unwrap()
        })
        .collect();
    keys.sort_unstable();
    assert_eq!(keys, vec!["Bearer fixture-key-a", "Bearer fixture-key-b"]);
    assert!(target.received_requests().await.unwrap().is_empty());
}

impl EnvVarGuard {
    fn set(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
        let previous = std::env::var_os(key);
        std::env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(previous) => std::env::set_var(self.key, previous),
            None => std::env::remove_var(self.key),
        }
    }
}

static OPENAI_CHAT_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

async fn lock_openai_chat_env() -> tokio::sync::MutexGuard<'static, ()> {
    OPENAI_CHAT_ENV_LOCK
        .get_or_init(|| Mutex::const_new(()))
        .lock()
        .await
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

/// Builds the fixture config through serde so the RED state fails at the
/// config parser ("unknown variant openai_chat"), not at compile time.
fn config_with_openai_chat_providers(providers: Value, routes: Value) -> Config {
    let mut value = serde_json::to_value(Config::default()).unwrap();
    for (name, provider) in providers.as_object().unwrap() {
        value["providers"][name] = provider.clone();
    }
    value["server"]["default_provider"] = json!(routes.as_array().unwrap()[0]["provider"]);
    value["routes"] = routes;
    serde_json::from_value(value).unwrap()
}

fn openai_chat_provider(base_url: &str, api_key_env: &str) -> Value {
    json!({
        "kind": "openai_chat",
        "base_url": base_url,
        "auth": "api_key",
        "api_key_env": api_key_env
    })
}

fn single_provider_config(base_url: &str) -> Config {
    config_with_openai_chat_providers(
        json!({ "openai-chat-test": openai_chat_provider(base_url, "SHUNT_OPENAI_CHAT_CONFORMANCE_KEY") }),
        json!([{
            "model": "claude-via-chat",
            "provider": "openai-chat-test",
            "upstream_model": "gpt-5",
            "effort": null,
            "service_tier": null
        }]),
    )
}

struct Gateway {
    base_url: String,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for Gateway {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn start_gateway(mut config: Config) -> Gateway {
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

fn anthropic_request(model: &str) -> String {
    json!({
        "model": model,
        "max_tokens": 64,
        "stream": false,
        "messages": [{"role": "user", "content": "fixture"}]
    })
    .to_string()
}

fn anthropic_streaming_request(model: &str) -> String {
    let mut value: Value = serde_json::from_str(&anthropic_request(model)).unwrap();
    value["stream"] = json!(true);
    value.to_string()
}

fn chat_completion_upstream() -> Value {
    json!({
        "id": "chatcmpl-fixture",
        "object": "chat.completion",
        "created": 1,
        "model": "gpt-5",
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": "fixture reply"},
            "finish_reason": "stop"
        }],
        "usage": {"prompt_tokens": 5, "completion_tokens": 2, "total_tokens": 7}
    })
}

/// Collect the gateway SSE relay into (event, data) frames for assertions.
async fn collect_sse_events(response: reqwest::Response) -> Vec<(String, Value)> {
    use futures_util::StreamExt;
    let mut buffer = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        buffer.extend_from_slice(&chunk.unwrap());
    }
    let text = String::from_utf8(buffer).expect("SSE relay must be UTF-8");
    let mut events = Vec::new();
    for frame in text.split("\n\n") {
        let frame = frame.trim();
        if frame.is_empty() {
            continue;
        }
        let mut event = "message".to_string();
        let mut data = String::new();
        for line in frame.lines() {
            if let Some(rest) = line.strip_prefix("event: ") {
                event = rest.to_string();
            }
            if let Some(rest) = line.strip_prefix("data: ") {
                data.push_str(rest);
            }
        }
        if data.is_empty() {
            continue;
        }
        let value = serde_json::from_str(&data).unwrap_or(Value::Null);
        events.push((event, value));
    }
    events
}

fn sse_body(frames: &[String]) -> ResponseTemplate {
    ResponseTemplate::new(200)
        .insert_header("content-type", "text/event-stream")
        // Real upstreams terminate the final frame with a blank line; EOF on
        // an unterminated frame is its own fail-closed case elsewhere.
        .set_body_raw(
            frames
                .iter()
                .map(|frame| format!("data: {frame}\n\n"))
                .collect::<String>(),
            "text/event-stream",
        )
}

fn chat_delta(delta: Value, finish: Option<&str>) -> String {
    let choice = json!({ "index": 0, "delta": delta, "finish_reason": finish });
    json!({ "choices": [choice], "usage": null }).to_string()
}

const CHAT_USAGE_CHUNK: &str =
    r#"{"choices":[],"usage":{"prompt_tokens":7,"completion_tokens":2,"total_tokens":9}}"#;

#[tokio::test]
async fn openai_chat_tracer_done_does_not_wait_for_eof() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        while !request.ends_with(b"\r\n\r\n") {
            assert!(request.len() < 16384, "fixture request headers too large");
            request.push(socket.read_u8().await.unwrap());
        }
        let frame = format!(
            "data: {}\n\ndata: [DONE]\n\n",
            chat_delta(json!({"content":"ready"}), Some("stop"))
        );
        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\n\r\n{:x}\r\n{}\r\n", frame.len(), frame);
        socket.write_all(response.as_bytes()).await.unwrap();
        // Deliberately omit the terminating HTTP chunk: [DONE] is authoritative.
        std::future::pending::<()>().await;
    });
    let gateway = start_gateway(single_provider_config(&format!("http://{address}"))).await;
    let response = reqwest::Client::new()
        .post(format!("{}/v1/messages", gateway.base_url))
        .body(anthropic_streaming_request("claude-via-chat"))
        .send()
        .await
        .unwrap();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        collect_sse_events(response),
    )
    .await;
    upstream.abort();
    let events = result.expect("[DONE] must finish without waiting for upstream EOF");
    assert_eq!(
        events
            .iter()
            .filter(|(event, _)| event == "message_stop")
            .count(),
        1
    );
    assert!(!events.iter().any(|(event, _)| event == "error"));
}

#[tokio::test]
async fn openai_chat_tracer_finish_without_done_is_error() {
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let backend = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(sse_body(&[chat_delta(
            json!({"content":"cut"}),
            Some("stop"),
        )]))
        .mount(&backend)
        .await;
    let gateway = start_gateway(single_provider_config(&backend.uri())).await;
    let response = reqwest::Client::new()
        .post(format!("{}/v1/messages", gateway.base_url))
        .body(anthropic_streaming_request("claude-via-chat"))
        .send()
        .await
        .unwrap();
    let events = collect_sse_events(response).await;
    assert_eq!(
        events.iter().filter(|(event, _)| event == "error").count(),
        1,
        "{events:?}"
    );
    assert!(
        !events.iter().any(|(event, _)| event == "message_stop"),
        "{events:?}"
    );
}

#[tokio::test]
async fn openai_chat_tracer_unary_happy_path() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");

    let backend = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(chat_completion_upstream()))
        .expect(1)
        .mount(&backend)
        .await;

    let gateway = start_gateway(single_provider_config(&backend.uri())).await;
    let response = reqwest::Client::new()
        .post(format!("{}/v1/messages", gateway.base_url))
        .header("x-api-key", "inbound-slot-key")
        .body(anthropic_request("claude-via-chat"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["content"][0]["text"], "fixture reply", "{body}");
    assert_eq!(body["stop_reason"], "end_turn", "{body}");
    assert_eq!(body["usage"]["input_tokens"], 5, "{body}");
    assert_eq!(body["usage"]["output_tokens"], 2, "{body}");

    let requests = backend.received_requests().await.unwrap();
    assert_eq!(
        requests.len(),
        1,
        "generation POST must not be re-dispatched"
    );
    // The exact bearer proves the synthetic env key was injected; the absent
    // x-api-key proves the inbound slot was stripped at the wire.
    assert_eq!(
        requests[0]
            .headers
            .get("authorization")
            .and_then(|value| value.to_str().ok()),
        Some("Bearer fixture-openai-key")
    );
    assert!(
        requests[0].headers.get("x-api-key").is_none(),
        "inbound x-api-key slot must be stripped"
    );
    let upstream_body: Value =
        serde_json::from_slice(&requests[0].body).expect("upstream body is JSON");
    assert_eq!(upstream_body["model"], "gpt-5", "{upstream_body}");
    assert_eq!(upstream_body["stream"], false, "{upstream_body}");
    assert_eq!(
        upstream_body["messages"][0]["role"], "user",
        "{upstream_body}"
    );
    assert_eq!(
        upstream_body["messages"][0]["content"], "fixture",
        "{upstream_body}"
    );
}

#[tokio::test]
async fn openai_chat_tracer_unary_isolation() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key_a = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY_A", "fixture-key-a");
    let _key_b = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY_B", "fixture-key-b");

    let backend = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(chat_completion_upstream()))
        .expect(2)
        .mount(&backend)
        .await;

    let config = config_with_openai_chat_providers(
        json!({
            "openai-chat-a": openai_chat_provider(&backend.uri(), "SHUNT_OPENAI_CHAT_CONFORMANCE_KEY_A"),
            "openai-chat-b": openai_chat_provider(&backend.uri(), "SHUNT_OPENAI_CHAT_CONFORMANCE_KEY_B")
        }),
        json!([
            {
                "model": "claude-via-a",
                "provider": "openai-chat-a",
                "upstream_model": "gpt-a",
                "effort": null,
                "service_tier": null
            },
            {
                "model": "claude-via-b",
                "provider": "openai-chat-b",
                "upstream_model": "gpt-b",
                "effort": null,
                "service_tier": null
            }
        ]),
    );
    let gateway = start_gateway(config).await;

    let client = reqwest::Client::new();
    let (response_a, response_b) = tokio::join!(
        client
            .post(format!("{}/v1/messages", gateway.base_url))
            .body(anthropic_request("claude-via-a"))
            .send(),
        client
            .post(format!("{}/v1/messages", gateway.base_url))
            .body(anthropic_request("claude-via-b"))
            .send()
    );
    for response in [response_a.unwrap(), response_b.unwrap()] {
        assert_eq!(response.status(), StatusCode::OK);
        let body: Value = response.json().await.unwrap();
        assert_eq!(body["content"][0]["text"], "fixture reply", "{body}");
    }

    let requests = backend.received_requests().await.unwrap();
    assert_eq!(requests.len(), 2);
    let mut seen = std::collections::HashSet::new();
    for request in &requests {
        let bearer = request
            .headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .unwrap()
            .to_string();
        let upstream_body: Value =
            serde_json::from_slice(&request.body).expect("upstream body is JSON");
        let model = upstream_body["model"].as_str().unwrap().to_string();
        seen.insert((bearer, model));
    }
    let expected: std::collections::HashSet<(String, String)> = [
        ("Bearer fixture-key-a".to_string(), "gpt-a".to_string()),
        ("Bearer fixture-key-b".to_string(), "gpt-b".to_string()),
    ]
    .into_iter()
    .collect();
    assert_eq!(
        seen, expected,
        "credentials and models must stay request-local"
    );
}

#[tokio::test]
async fn openai_chat_tracer_unary_non_api_key_auth_rejected() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;

    let mut value = serde_json::to_value(Config::default()).unwrap();
    value["providers"]["openai-chat-test"] = json!({
        "kind": "openai_chat",
        "base_url": "http://127.0.0.1:9",
        "auth": "passthrough"
    });
    value["server"]["default_provider"] = json!("openai-chat-test");
    value["routes"] = json!([{
        "model": "claude-via-chat",
        "provider": "openai-chat-test",
        "upstream_model": "gpt-5",
        "effort": null,
        "service_tier": null
    }]);
    let config: Config = serde_json::from_value(value).unwrap();
    let error = config
        .validate()
        .expect_err("non-API-key auth must be rejected");
    let message = error.to_string();
    assert!(message.contains("openai_chat"), "{message}");
    assert!(message.contains("api_key"), "{message}");
}

#[tokio::test]
async fn openai_chat_tracer_streaming_relay() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");

    let backend = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(sse_body(&[
            chat_delta(json!({"role": "assistant"}), None),
            chat_delta(json!({"content": "hello "}), None),
            chat_delta(json!({"content": "world"}), Some("stop")),
            CHAT_USAGE_CHUNK.to_string(),
            "[DONE]".to_string(),
        ]))
        .expect(1)
        .mount(&backend)
        .await;

    let gateway = start_gateway(single_provider_config(&backend.uri())).await;
    let response = reqwest::Client::new()
        .post(format!("{}/v1/messages", gateway.base_url))
        .body(anthropic_streaming_request("claude-via-chat"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let events = collect_sse_events(response).await;

    let text: Vec<&str> = events
        .iter()
        .filter(|(event, _)| event == "content_block_delta")
        .filter_map(|(_, data)| data["delta"]["text"].as_str())
        .collect();
    assert_eq!(text, vec!["hello ", "world"], "{events:?}");
    let stops = events
        .iter()
        .filter(|(event, _)| event == "message_stop")
        .count();
    assert_eq!(stops, 1, "exactly one authoritative terminal: {events:?}");
    assert!(
        events.iter().any(|(event, _)| event == "message_start"),
        "{events:?}"
    );
    let delta = events
        .iter()
        .find(|(event, _)| event == "message_delta")
        .map(|(_, data)| data)
        .expect("message_delta carries the terminal decision");
    assert_eq!(delta["delta"]["stop_reason"], "end_turn", "{delta}");
    assert_eq!(delta["usage"]["input_tokens"], 7, "{delta}");
    assert_eq!(delta["usage"]["output_tokens"], 2, "{delta}");

    let requests = backend.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    let upstream_body: Value =
        serde_json::from_slice(&requests[0].body).expect("upstream body is JSON");
    assert_eq!(upstream_body["stream"], true, "{upstream_body}");
    assert_eq!(
        upstream_body["stream_options"]["include_usage"], true,
        "usage accounting must be forced upstream"
    );
}

#[tokio::test]
async fn openai_chat_tracer_streaming_eof_failclosed() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");

    let backend = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(sse_body(&[
            chat_delta(json!({"role": "assistant"}), None),
            chat_delta(json!({"content": "partial"}), None),
        ]))
        .expect(1)
        .mount(&backend)
        .await;

    let gateway = start_gateway(single_provider_config(&backend.uri())).await;
    let response = reqwest::Client::new()
        .post(format!("{}/v1/messages", gateway.base_url))
        .body(anthropic_streaming_request("claude-via-chat"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let events = collect_sse_events(response).await;

    let errors = events.iter().filter(|(event, _)| event == "error").count();
    assert_eq!(
        errors, 1,
        "EOF without a terminal must fail closed: {events:?}"
    );
    assert!(
        events.iter().all(|(event, _)| event != "message_stop"),
        "a cut stream must never emit a success terminal: {events:?}"
    );
}

#[tokio::test]
async fn openai_chat_tracer_streaming_duplicate_terminal() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");

    let backend = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(sse_body(&[
            chat_delta(json!({"role": "assistant"}), None),
            chat_delta(json!({"content": "done"}), Some("stop")),
            "[DONE]".to_string(),
            "[DONE]".to_string(),
        ]))
        .expect(1)
        .mount(&backend)
        .await;

    let gateway = start_gateway(single_provider_config(&backend.uri())).await;
    let response = reqwest::Client::new()
        .post(format!("{}/v1/messages", gateway.base_url))
        .body(anthropic_streaming_request("claude-via-chat"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let events = collect_sse_events(response).await;

    let errors = events.iter().filter(|(event, _)| event == "error").count();
    assert_eq!(
        errors, 1,
        "a duplicate [DONE] terminal must fail closed: {events:?}"
    );
    assert!(
        events.iter().all(|(event, _)| event != "message_stop"),
        "no success terminal may survive a duplicate [DONE]: {events:?}"
    );
}

/// Sends an inbound body and asserts it is rejected with a typed Anthropic
/// 400 before any upstream dispatch.
async fn reject_with_zero_upstream(
    gateway: &Gateway,
    backend: &MockServer,
    body: impl Into<reqwest::Body>,
) -> Value {
    let response = reqwest::Client::new()
        .post(format!("{}/v1/messages", gateway.base_url))
        .body(body)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let error: Value = response.json().await.unwrap();
    assert_eq!(
        error["error"]["type"], "invalid_request_error",
        "gateway-owned rejections must use the Anthropic error shape: {error}"
    );
    let requests = backend.received_requests().await.unwrap();
    assert!(
        requests.is_empty(),
        "rejections must never reach the upstream: {requests:?}"
    );
    error
}

#[tokio::test]
async fn openai_chat_translate_wire_exact_body() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");

    let backend = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(chat_completion_upstream()))
        .expect(1)
        .mount(&backend)
        .await;

    let gateway = start_gateway(single_provider_config(&backend.uri())).await;
    let inbound = json!({
        "model": "claude-via-chat",
        "system": "be brief",
        "max_tokens": 64,
        "temperature": 0.5,
        "top_p": 0.9,
        "stop_sequences": ["halt"],
        "stream": false,
        "messages": [{"role": "user", "content": "fixture"}]
    });
    let response = reqwest::Client::new()
        .post(format!("{}/v1/messages", gateway.base_url))
        .body(inbound.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let requests = backend.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    let upstream_body: Value =
        serde_json::from_slice(&requests[0].body).expect("upstream body is JSON");
    assert_eq!(
        upstream_body,
        json!({
            "model": "gpt-5",
            "stream": false,
            "max_tokens": 64,
            "temperature": 0.5,
            "top_p": 0.9,
            "stop": ["halt"],
            "messages": [
                {"role": "system", "content": "be brief"},
                {"role": "user", "content": "fixture"}
            ]
        }),
        "the wire body must be exactly the whitelist mapping: {upstream_body}"
    );
    for forbidden in [
        "system",
        "metadata",
        "top_k",
        "stop_sequences",
        "tool_choice",
    ] {
        assert!(
            upstream_body.get(forbidden).is_none(),
            "forbidden key {forbidden} must never reach the wire: {upstream_body}"
        );
    }
}

#[tokio::test]
async fn openai_chat_translate_wire_rejects_unknown_field() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let backend = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(chat_completion_upstream()))
        .mount(&backend)
        .await;
    let gateway = start_gateway(single_provider_config(&backend.uri())).await;

    let mut inbound: Value = serde_json::from_str(&anthropic_request("claude-via-chat")).unwrap();
    inbound["top_k"] = json!(3);
    let error = reject_with_zero_upstream(&gateway, &backend, inbound.to_string()).await;
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("top_k"),
        "{error}"
    );
}

#[tokio::test]
async fn openai_chat_translate_wire_rejects_metadata() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let backend = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(chat_completion_upstream()))
        .mount(&backend)
        .await;
    let gateway = start_gateway(single_provider_config(&backend.uri())).await;

    let mut inbound: Value = serde_json::from_str(&anthropic_request("claude-via-chat")).unwrap();
    inbound["metadata"] = json!({"user_id": "u1"});
    let error = reject_with_zero_upstream(&gateway, &backend, inbound.to_string()).await;
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("metadata"),
        "{error}"
    );
}

#[tokio::test]
async fn openai_chat_translate_wire_rejects_unknown_block() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let backend = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(chat_completion_upstream()))
        .mount(&backend)
        .await;
    let gateway = start_gateway(single_provider_config(&backend.uri())).await;

    let inbound = json!({
        "model": "claude-via-chat",
        "max_tokens": 64,
        "messages": [{
            "role": "user",
            "content": [{"type": "document", "source": {"type": "url", "url": "https://x/f.pdf"}}]
        }]
    });
    let error = reject_with_zero_upstream(&gateway, &backend, inbound.to_string()).await;
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("document"),
        "{error}"
    );
}

#[tokio::test]
async fn openai_chat_translate_wire_rejects_signed_thinking() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let backend = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(chat_completion_upstream()))
        .mount(&backend)
        .await;
    let gateway = start_gateway(single_provider_config(&backend.uri())).await;

    let inbound = json!({
        "model": "claude-via-chat",
        "max_tokens": 64,
        "messages": [{
            "role": "assistant",
            "content": [
                {"type": "thinking", "thinking": "hmm", "signature": "sig"},
                {"type": "text", "text": "answer"}
            ]
        }]
    });
    let error = reject_with_zero_upstream(&gateway, &backend, inbound.to_string()).await;
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("signature"),
        "{error}"
    );
}

#[tokio::test]
async fn openai_chat_translate_wire_rejects_redacted_thinking() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let backend = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(chat_completion_upstream()))
        .mount(&backend)
        .await;
    let gateway = start_gateway(single_provider_config(&backend.uri())).await;

    let inbound = json!({
        "model": "claude-via-chat",
        "max_tokens": 64,
        "messages": [{
            "role": "assistant",
            "content": [
                {"type": "redacted_thinking", "data": "opaque"},
                {"type": "text", "text": "answer"}
            ]
        }]
    });
    let error = reject_with_zero_upstream(&gateway, &backend, inbound.to_string()).await;
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("redacted"),
        "{error}"
    );
}

#[tokio::test]
async fn openai_chat_translate_wire_rejects_malformed_utf8() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let backend = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(chat_completion_upstream()))
        .mount(&backend)
        .await;
    let gateway = start_gateway(single_provider_config(&backend.uri())).await;

    // Raw bytes with invalid UTF-8 inside a JSON string: the decode boundary
    // must fail closed rather than lossily coercing the payload.
    let raw = b"{\"model\":\"claude-via-chat\",\"max_tokens\":64,\"messages\":[{\"role\":\"user\",\"content\":\"\xff\xfe\"}]}";
    let error =
        reject_with_zero_upstream(&gateway, &backend, reqwest::Body::from(raw.as_slice())).await;
    assert!(
        !error["error"]["message"].as_str().unwrap().is_empty(),
        "{error}"
    );
}
#[tokio::test]
async fn openai_chat_translate_wire_tool_pairing() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let backend = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(chat_completion_upstream()))
        .expect(1)
        .mount(&backend)
        .await;
    let gateway = start_gateway(single_provider_config(&backend.uri())).await;

    let inbound = json!({
        "model": "claude-via-chat",
        "max_tokens": 64,
        "tools": [{
            "name": "get_weather",
            "description": "Look up weather",
            "input_schema": {"type": "object", "properties": {"city": {"type": "string"}}}
        }],
        "tool_choice": "auto",
        "messages": [
            {"role": "user", "content": "weather?"},
            {
                "role": "assistant",
                "content": [
                    {"type": "tool_use", "id": "tu_1", "name": "get_weather", "input": {"city": "Tokyo"}},
                    {"type": "tool_use", "id": "tu_2", "name": "get_weather", "input": {"city": "Paris"}}
                ]
            },
            {
                "role": "user",
                "content": [
                    {"type": "tool_result", "tool_use_id": "tu_1", "content": "sunny"},
                    {"type": "tool_result", "tool_use_id": "tu_2", "content": "rainy"}
                ]
            }
        ]
    });
    let response = reqwest::Client::new()
        .post(format!("{}/v1/messages", gateway.base_url))
        .body(inbound.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let requests = backend.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    let upstream_body: Value =
        serde_json::from_slice(&requests[0].body).expect("upstream body is JSON");
    assert_eq!(
        upstream_body["tools"],
        json!([{
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "Look up weather",
                "parameters": {"type": "object", "properties": {"city": {"type": "string"}}}
            }
        }]),
        "{upstream_body}"
    );
    assert_eq!(upstream_body["tool_choice"], "auto", "{upstream_body}");
    let messages = upstream_body["messages"].as_array().unwrap();
    assert_eq!(
        messages[1]["tool_calls"][0]["id"], "tu_1",
        "{upstream_body}"
    );
    assert_eq!(
        messages[1]["tool_calls"][1]["id"], "tu_2",
        "{upstream_body}"
    );
    let args: Value = serde_json::from_str(
        messages[1]["tool_calls"][1]["function"]["arguments"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(args["city"], "Paris", "{upstream_body}");
    assert_eq!(
        messages[2],
        json!({"role": "tool", "tool_call_id": "tu_1", "content": "sunny"}),
        "{upstream_body}"
    );
    assert_eq!(
        messages[3],
        json!({"role": "tool", "tool_call_id": "tu_2", "content": "rainy"}),
        "{upstream_body}"
    );
}

// ---- CHAT-02/D-02: the endpoint grammar is enforced at config boot ----
//
// The shared chat_completions_endpoint grammar rejects query strings,
// fragments, and userinfo in provider base URLs; these fixtures pin that
// rejection at the Config::validate load boundary so a bad URL can never
// reach a request path.

fn boot_config_with_base_url(base_url: &str) -> Config {
    config_with_openai_chat_providers(
        json!({
            "openai-chat-test":
                openai_chat_provider(base_url, "SHUNT_OPENAI_CHAT_CONFORMANCE_KEY")
        }),
        json!([{"model": "claude-via-chat", "provider": "openai-chat-test"}]),
    )
}

#[test]
fn openai_chat_boot_rejects_query_string_in_base_url() {
    let error = boot_config_with_base_url("https://api.example.com/v1?key=secret")
        .validate()
        .expect_err("query-string base_url must be rejected at boot");
    assert!(
        matches!(
            &error,
            ConfigError::ProviderBaseUrl { provider, message }
                if provider == "openai-chat-test" && message.contains("query")
        ),
        "{error:?}"
    );
}

#[test]
fn openai_chat_boot_rejects_fragment_in_base_url() {
    let error = boot_config_with_base_url("https://api.example.com/v1#section")
        .validate()
        .expect_err("fragment base_url must be rejected at boot");
    assert!(
        matches!(
            &error,
            ConfigError::ProviderBaseUrl { provider, message }
                if provider == "openai-chat-test" && message.contains("fragment")
        ),
        "{error:?}"
    );
}

#[test]
fn openai_chat_boot_rejects_userinfo_in_base_url() {
    let error = boot_config_with_base_url("https://user:pass@api.example.com/v1")
        .validate()
        .expect_err("userinfo base_url must be rejected at boot");
    assert!(
        matches!(
            &error,
            ConfigError::ProviderBaseUrl { provider, message }
                if provider == "openai-chat-test" && message.contains("userinfo")
        ),
        "{error:?}"
    );
}

// ===================================================================
// 13-03 terminal conformance: usage-only positioning, reasoning order,
// provider errors, and fail-closed framing (CHAT-05/06/07).
// ===================================================================

async fn openai_chat_terminal_stream_events(frames: Vec<String>) -> Vec<(String, Value)> {
    let backend = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(sse_body(&frames))
        .mount(&backend)
        .await;
    let gateway = start_gateway(single_provider_config(&backend.uri())).await;
    let response = reqwest::Client::new()
        .post(format!("{}/v1/messages", gateway.base_url))
        .body(anthropic_streaming_request("claude-via-chat"))
        .send()
        .await
        .unwrap();
    collect_sse_events(response).await
}

#[tokio::test]
async fn openai_chat_terminal_usage_only_trailing_chunk_relays_usage() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let events = openai_chat_terminal_stream_events(vec![
        chat_delta(json!({"content": "hi"}), None),
        chat_delta(json!({}), Some("stop")),
        CHAT_USAGE_CHUNK.to_string(),
        "[DONE]".to_string(),
    ])
    .await;
    assert!(
        !events.iter().any(|(event, _)| event == "error"),
        "{events:?}"
    );
    assert_eq!(
        events
            .iter()
            .filter(|(event, _)| event == "message_stop")
            .count(),
        1,
        "{events:?}"
    );
    let delta = events
        .iter()
        .find(|(event, _)| event == "message_delta")
        .map(|(_, data)| data)
        .unwrap();
    assert_eq!(delta["usage"]["input_tokens"], 7, "{delta}");
    assert_eq!(delta["usage"]["output_tokens"], 2, "{delta}");
}

#[tokio::test]
async fn openai_chat_terminal_reasoning_order_streamed() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let events = openai_chat_terminal_stream_events(vec![
        chat_delta(
            json!({"role": "assistant", "reasoning_content": "why"}),
            None,
        ),
        chat_delta(json!({"reasoning_content": "more", "content": "so"}), None),
        chat_delta(json!({}), Some("stop")),
        "[DONE]".to_string(),
    ])
    .await;
    assert!(
        !events.iter().any(|(event, _)| event == "error"),
        "{events:?}"
    );
    let starts: Vec<&Value> = events
        .iter()
        .filter(|(event, _)| event == "content_block_start")
        .map(|(_, data)| data)
        .collect();
    assert_eq!(starts[0]["content_block"]["type"], "thinking", "{events:?}");
    assert_eq!(starts[1]["content_block"]["type"], "text", "{events:?}");
    let deltas: Vec<String> = events
        .iter()
        .filter(|(event, _)| event == "content_block_delta")
        .map(|(_, data)| {
            data["delta"]["thinking"]
                .as_str()
                .or_else(|| data["delta"]["text"].as_str())
                .unwrap_or_default()
                .to_string()
        })
        .collect();
    assert_eq!(deltas, vec!["why", "more", "so"], "{events:?}");
    assert_eq!(
        events
            .iter()
            .filter(|(event, _)| event == "message_stop")
            .count(),
        1
    );
}

#[tokio::test]
async fn openai_chat_terminal_error_after_text_single_terminal() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let events = openai_chat_terminal_stream_events(vec![
        chat_delta(json!({"content": "hel"}), None),
        chat_delta(json!({"content": "lo"}), None),
        json!({"error": {"message": "midstream failure"}}).to_string(),
        "[DONE]".to_string(),
    ])
    .await;
    assert_eq!(
        events.iter().filter(|(event, _)| event == "error").count(),
        1,
        "{events:?}"
    );
    assert!(
        !events.iter().any(|(event, _)| event == "message_stop"),
        "{events:?}"
    );
    let text_index = events
        .iter()
        .position(|(event, _)| event == "content_block_delta")
        .unwrap();
    let error_index = events
        .iter()
        .position(|(event, _)| event == "error")
        .unwrap();
    assert!(text_index < error_index, "{events:?}");
}

#[tokio::test]
async fn openai_chat_terminal_usage_payload_after_finish_rejected() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    // A trailing chunk whose delta carries tool_calls without declaring a
    // finish_reason is still payload: it must fail closed.
    let events = openai_chat_terminal_stream_events(vec![
        chat_delta(json!({"content": "hi"}), None),
        chat_delta(json!({}), Some("stop")),
        json!({"choices": [{"index": 0, "delta": {"tool_calls": []}}]}).to_string(),
        "[DONE]".to_string(),
    ])
    .await;
    assert_eq!(
        events.iter().filter(|(event, _)| event == "error").count(),
        1,
        "{events:?}"
    );
    assert!(
        !events.iter().any(|(event, _)| event == "message_stop"),
        "{events:?}"
    );
}

#[tokio::test]
async fn openai_chat_terminal_second_usage_only_chunk_rejected() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let events = openai_chat_terminal_stream_events(vec![
        chat_delta(json!({"content": "hi"}), None),
        chat_delta(json!({}), Some("stop")),
        CHAT_USAGE_CHUNK.to_string(),
        CHAT_USAGE_CHUNK.to_string(),
        "[DONE]".to_string(),
    ])
    .await;
    assert_eq!(
        events.iter().filter(|(event, _)| event == "error").count(),
        1,
        "{events:?}"
    );
    assert!(
        !events.iter().any(|(event, _)| event == "message_stop"),
        "{events:?}"
    );
}

#[tokio::test]
async fn openai_chat_terminal_usage_only_before_finish_rejected() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let events = openai_chat_terminal_stream_events(vec![
        json!({"choices": [], "usage": null}).to_string(),
        chat_delta(json!({"content": "hi"}), Some("stop")),
        "[DONE]".to_string(),
    ])
    .await;
    assert_eq!(
        events.iter().filter(|(event, _)| event == "error").count(),
        1,
        "{events:?}"
    );
    assert!(
        !events.iter().any(|(event, _)| event == "message_stop"),
        "{events:?}"
    );
}

#[tokio::test]
async fn openai_chat_terminal_missing_finish_eof_fails() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let events =
        openai_chat_terminal_stream_events(vec![chat_delta(json!({"content": "partial"}), None)])
            .await;
    assert_eq!(
        events.iter().filter(|(event, _)| event == "error").count(),
        1,
        "{events:?}"
    );
    assert!(
        !events.iter().any(|(event, _)| event == "message_stop"),
        "{events:?}"
    );
}

#[tokio::test]
async fn openai_chat_terminal_duplicate_done_rejected() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let events = openai_chat_terminal_stream_events(vec![
        chat_delta(json!({"content": "hi"}), Some("stop")),
        "[DONE]".to_string(),
        "[DONE]".to_string(),
    ])
    .await;
    assert_eq!(
        events.iter().filter(|(event, _)| event == "error").count(),
        1,
        "{events:?}"
    );
    assert!(
        !events.iter().any(|(event, _)| event == "message_stop"),
        "{events:?}"
    );
}

#[tokio::test]
async fn openai_chat_terminal_residual_after_done_rejected() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let events = openai_chat_terminal_stream_events(vec![
        chat_delta(json!({"content": "hi"}), Some("stop")),
        "[DONE]".to_string(),
        chat_delta(json!({"content": "late"}), None),
    ])
    .await;
    assert_eq!(
        events.iter().filter(|(event, _)| event == "error").count(),
        1,
        "{events:?}"
    );
    assert!(
        !events.iter().any(|(event, _)| event == "message_stop"),
        "{events:?}"
    );
}

#[tokio::test]
async fn openai_chat_terminal_malformed_json_rejected() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let events = openai_chat_terminal_stream_events(vec![
        "{not json".to_string(),
        chat_delta(json!({"content": "hi"}), Some("stop")),
        "[DONE]".to_string(),
    ])
    .await;
    assert_eq!(
        events.iter().filter(|(event, _)| event == "error").count(),
        1,
        "{events:?}"
    );
    assert!(
        !events.iter().any(|(event, _)| event == "message_stop"),
        "{events:?}"
    );
}

#[tokio::test]
async fn openai_chat_terminal_unary_embedded_200_error_request_id() {
    if !can_bind_loopback() {
        return;
    }
    let _env_lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let backend = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("x-request-id", "req_header_42")
                .set_body_json(json!({
                    "error": {
                        "message": "backend exploded",
                        "metadata": {"request_id": "req_42"}
                    }
                })),
        )
        .mount(&backend)
        .await;
    let gateway = start_gateway(single_provider_config(&backend.uri())).await;
    let response = reqwest::Client::new()
        .post(format!("{}/v1/messages", gateway.base_url))
        .body(anthropic_request("claude-via-chat"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["type"], "error", "{body}");
    assert_eq!(
        body["error"]["message"], "OpenAI Chat backend error",
        "{body}"
    );
    assert_eq!(body["error"]["request_id"], "req_header_42", "{body}");
}
