//! End-to-end coverage for exact inbound Responses routes translated to Anthropic.

use std::net::SocketAddr;

use reqwest::StatusCode;
use serde_json::{json, Value};
use shunt::{
    config::{AuthMode, CodexEndpointConfig, Config, RouteConfig},
    server,
};
use tokio::task::JoinHandle;
use wiremock::{
    matchers::{header, method, path},
    Match, Mock, MockServer, Request, ResponseTemplate,
};

struct TestGateway {
    base_url: String,
    task: JoinHandle<()>,
}

impl Drop for TestGateway {
    fn drop(&mut self) {
        self.task.abort();
    }
}

struct TranslatedRequest;

impl Match for TranslatedRequest {
    fn matches(&self, request: &Request) -> bool {
        let Ok(body) = serde_json::from_slice::<Value>(&request.body) else {
            return false;
        };
        body["model"] == "claude-sonnet-upstream"
            && body["system"][0]["text"] == "Be concise"
            && body["messages"][0]["content"][0]["text"] == "hello"
            && body["messages"][1]["content"][0]["type"] == "tool_use"
            && body["messages"][2]["content"][0]["tool_use_id"] == "call_1"
            && body["tools"][0]["name"] == "lookup"
            && body["stream"] == false
    }
}

async fn start_gateway(upstream: &MockServer) -> TestGateway {
    let mut config = Config::default();
    let anthropic = config.providers.get_mut("anthropic").unwrap();
    anthropic.base_url = upstream.uri();
    anthropic.auth = AuthMode::Passthrough;
    config.routes.push(RouteConfig {
        model: "claude-via-responses".into(),
        provider: "anthropic".into(),
        upstream_model: Some("claude-sonnet-upstream".into()),
        effort: None,
        service_tier: None,
    });
    config.server.codex_endpoint = Some(CodexEndpointConfig {
        provider: "codex".into(),
    });
    config.server.bind = "127.0.0.1:0".into();
    let listener = tokio::net::TcpListener::bind(config.server.bind_addr().unwrap())
        .await
        .unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    let (app, _, _) = server::build_router(config).unwrap();
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    TestGateway {
        base_url: format!("http://{addr}"),
        task,
    }
}

fn request_body() -> Value {
    json!({
        "model":"claude-via-responses",
        "instructions":"Be concise",
        "input":[
            {"type":"message","role":"user","content":[{"type":"input_text","text":"hello"}]},
            {"type":"function_call","call_id":"call_1","name":"lookup","arguments":"{\"q\":\"x\"}"},
            {"type":"function_call_output","call_id":"call_1","output":"found"}
        ],
        "tools":[{"type":"function","name":"lookup","parameters":{"type":"object"}}],
        "stream":false,
        "store":false
    })
}

#[tokio::test]
async fn request_exact_anthropic_route_translates_before_dispatch() {
    let upstream = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("x-api-key", "client-anthropic-key"))
        .and(TranslatedRequest)
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"msg_1", "type":"message", "role":"assistant",
            "model":"claude-sonnet-upstream", "content":[{"type":"text","text":"ok"}],
            "stop_reason":"end_turn", "usage":{"input_tokens":10,"output_tokens":2}
        })))
        .expect(1)
        .mount(&upstream)
        .await;
    let gateway = start_gateway(&upstream).await;

    let response = reqwest::Client::new()
        .post(format!("{}/v1/responses", gateway.base_url))
        .header("x-api-key", "client-anthropic-key")
        .json(&request_body())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    upstream.verify().await;
}

#[tokio::test]
async fn request_unsupported_state_fails_before_network_dispatch() {
    let upstream = MockServer::start().await;
    let gateway = start_gateway(&upstream).await;
    let mut body = request_body();
    body["previous_response_id"] = Value::String("resp_foreign".into());

    let response = reqwest::Client::new()
        .post(format!("{}/v1/responses", gateway.base_url))
        .header("x-api-key", "client-anthropic-key")
        .json(&body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["error"]["type"], "invalid_request_error");
    assert_eq!(upstream.received_requests().await.unwrap().len(), 0);
}
