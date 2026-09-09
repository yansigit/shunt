//! End-to-end coverage for exact inbound Responses routes translated to Anthropic.

use std::{net::SocketAddr, time::Duration};

use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use futures_util::{SinkExt, StreamExt};
use reqwest::StatusCode;
use serde_json::{json, Value};
use shunt::{
    config::{AuthMode, CodexEndpointConfig, Config, RouteConfig},
    server,
};
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tungstenite::client::IntoClientRequest;
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
            && !request.headers.contains_key("content-encoding")
            && body["system"][0]["text"] == "Be concise"
            && body["messages"][0]["content"][0]["text"] == "hello"
            && body["messages"][1]["content"][0]["type"] == "tool_use"
            && body["messages"][2]["content"][0]["tool_use_id"] == "call_1"
            && body["tools"][0]["name"] == "lookup"
            && body["stream"] == false
    }
}

struct CollaborationRequest;

impl Match for CollaborationRequest {
    fn matches(&self, request: &Request) -> bool {
        let Ok(body) = serde_json::from_slice::<Value>(&request.body) else {
            return false;
        };
        body["messages"][0]["role"] == "user"
            && body["messages"][0]["content"][0]["text"]
                .as_str()
                .is_some_and(|text| text.contains("Task name: /root/worker"))
            && body["tools"][0]["name"] == "shunt_collaboration__spawn_agent"
            && body["tools"][0]["input_schema"]["properties"]["message"]
                .get("encrypted")
                .is_none()
            && body["metadata"]["user_id"] == "task_42"
    }
}

#[tokio::test]
async fn compressed_request_is_decoded_translated_and_sent_uncompressed() {
    let upstream = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(TranslatedRequest)
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"msg_zstd", "type":"message", "role":"assistant",
            "model":"claude-sonnet-upstream", "content":[{"type":"text","text":"ok"}],
            "stop_reason":"end_turn", "usage":{"input_tokens":1,"output_tokens":1}
        })))
        .expect(1)
        .mount(&upstream)
        .await;
    let gateway = start_gateway(&upstream).await;
    let encoded = zstd::stream::encode_all(request_body().to_string().as_bytes(), 1).unwrap();

    let response = reqwest::Client::new()
        .post(format!("{}/v1/responses", gateway.base_url))
        .header("x-api-key", "client-anthropic-key")
        .header("content-type", "application/json")
        .header("content-encoding", "zstd")
        .body(encoded)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.json::<Value>().await.unwrap()["status"],
        "completed"
    );
    upstream.verify().await;
}

#[tokio::test]
async fn upstream_error_is_openai_shaped_and_preserves_retry_headers() {
    let upstream = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "17")
                .insert_header("x-request-id", "req_upstream")
                .set_body_json(json!({
                    "type":"error",
                    "error":{"type":"rate_limit_error","message":"slow down"}
                })),
        )
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

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(response.headers()["retry-after"], "17");
    assert_eq!(response.headers()["x-request-id"], "req_upstream");
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["error"]["type"], "rate_limit_error");
    assert_eq!(body["error"]["message"], "slow down");
    assert!(body["error"]["code"].is_null());
    upstream.verify().await;
}

async fn start_gateway(upstream: &MockServer) -> TestGateway {
    start_gateway_with_collaboration(upstream, false).await
}

async fn start_gateway_with_collaboration(
    upstream: &MockServer,
    collaboration: bool,
) -> TestGateway {
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
        collaboration,
        routes: Vec::new(),
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

fn collaboration_body(stream: bool) -> Value {
    json!({
        "model":"claude-via-responses",
        "input":[
            {"type":"additional_tools","role":"developer","tools":[{
                "type":"namespace","name":"collaboration","tools":[{
                    "type":"function","name":"spawn_agent","description":"spawn child",
                    "parameters":{"type":"object","properties":{"message":{"type":"string","encrypted":true}}}
                }]
            }]},
            {"type":"agent_message","author":"/root","recipient":"/root/worker","content":[{
                "type":"input_text","text":"Message Type: NEW_TASK\nTask name: /root/worker\nSender: /root\nPayload:\nImplement the fixture"
            }]}
        ],
        "stream":stream,"store":false,
        "metadata":{"task_id":"task_42","subagent":"collab_spawn"}
    })
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
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["object"], "response");
    assert_eq!(body["status"], "completed");
    assert_eq!(body["model"], "claude-via-responses");
    assert_eq!(body["output"][0]["content"][0]["text"], "ok");
    assert_eq!(body["usage"]["input_tokens"], 10);
    assert_eq!(body["usage"]["total_tokens"], 12);
    upstream.verify().await;
}

#[tokio::test]
async fn enabled_collaboration_translates_and_restores_json_tool_call() {
    let upstream = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(CollaborationRequest)
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"msg_collab", "type":"message", "role":"assistant",
            "model":"claude-sonnet-upstream",
            "content":[{"type":"tool_use","id":"call_spawn","name":"shunt_collaboration__spawn_agent","input":{"message":"child work"}}],
            "stop_reason":"tool_use", "usage":{"input_tokens":2,"output_tokens":1}
        })))
        .expect(1)
        .mount(&upstream)
        .await;
    let gateway = start_gateway_with_collaboration(&upstream, true).await;

    let response = reqwest::Client::new()
        .post(format!("{}/v1/responses", gateway.base_url))
        .header("x-api-key", "client-anthropic-key")
        .json(&collaboration_body(false))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["output"][0]["name"], "spawn_agent");
    assert_eq!(body["output"][0]["namespace"], "collaboration");
    assert_eq!(body["output"][0]["encrypted_function_args"], json!([]));
    upstream.verify().await;
}

#[tokio::test]
async fn encrypted_collaboration_task_fails_without_network_or_disclosure() {
    let upstream = MockServer::start().await;
    let gateway = start_gateway_with_collaboration(&upstream, true).await;
    let mut raw = vec![0x5a; 73];
    raw[0] = 0x80;
    let ciphertext = URL_SAFE.encode(raw);
    let mut body = collaboration_body(false);
    body["input"] = json!([{
        "type":"agent_message","author":"/root","recipient":"/root/worker","content":[
            {"type":"input_text","text":"Message Type: NEW_TASK\nTask name: /root/worker\nSender: /root\nPayload:\n"},
            {"type":"encrypted_content","encrypted_content":ciphertext}
        ]
    }]);

    let response = reqwest::Client::new()
        .post(format!("{}/v1/responses", gateway.base_url))
        .header("x-api-key", "client-anthropic-key")
        .json(&body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let text = response.text().await.unwrap();
    assert!(text.contains("encrypted agent task"));
    assert!(!text.contains(&ciphertext));
    assert!(upstream.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn response_stream_maps_text_tools_reasoning_usage_and_terminal_order() {
    let upstream = MockServer::start().await;
    let sse = concat!(
        "event: message_start\n",
        "data: {\"type\":\"message_start\",\"message\":{\"model\":\"claude-sonnet-upstream\",\"usage\":{\"input_tokens\":5}}}\n\n",
        "event: content_block_start\n",
        "data: {\"type\":\"content_block_start\",\"content_block\":{\"type\":\"thinking\"}}\n\n",
        "event: content_block_delta\n",
        "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"thinking_delta\",\"thinking\":\"check\"}}\n\n",
        "event: content_block_stop\n",
        "data: {\"type\":\"content_block_stop\"}\n\n",
        "event: content_block_start\n",
        "data: {\"type\":\"content_block_start\",\"content_block\":{\"type\":\"text\"}}\n\n",
        "event: content_block_delta\n",
        "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"hello\"}}\n\n",
        "event: content_block_stop\n",
        "data: {\"type\":\"content_block_stop\"}\n\n",
        "event: content_block_start\n",
        "data: {\"type\":\"content_block_start\",\"content_block\":{\"type\":\"tool_use\",\"id\":\"call_7\",\"name\":\"lookup\"}}\n\n",
        "event: content_block_delta\n",
        "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"q\\\":\"}}\n\n",
        "event: content_block_delta\n",
        "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"1}\"}}\n\n",
        "event: content_block_stop\n",
        "data: {\"type\":\"content_block_stop\"}\n\n",
        "event: message_delta\n",
        "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\"},\"usage\":{\"output_tokens\":4}}\n\n",
        "event: message_stop\n",
        "data: {\"type\":\"message_stop\"}\n\n"
    );
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse),
        )
        .expect(1)
        .mount(&upstream)
        .await;
    let gateway = start_gateway(&upstream).await;
    let mut body = request_body();
    body["stream"] = Value::Bool(true);

    let response = reqwest::Client::new()
        .post(format!("{}/v1/responses", gateway.base_url))
        .header("x-api-key", "client-anthropic-key")
        .json(&body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let text = response.text().await.unwrap();
    for event in [
        "response.created",
        "response.in_progress",
        "response.reasoning_summary_text.delta",
        "response.output_text.delta",
        "response.function_call_arguments.delta",
        "response.function_call_arguments.done",
        "response.completed",
    ] {
        assert!(text.contains(event), "missing {event}: {text}");
    }
    assert!(text.find("response.created").unwrap() < text.find("response.completed").unwrap());
    assert!(text.contains("\"call_id\":\"call_7\""));
    assert!(text.contains("\"total_tokens\":9"));
    assert!(text.ends_with("data: [DONE]\n\n"));
}

#[tokio::test]
async fn response_premature_stream_eof_is_failed_not_completed() {
    let upstream = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(
                    "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{}}\n\n",
                ),
        )
        .mount(&upstream)
        .await;
    let gateway = start_gateway(&upstream).await;
    let mut body = request_body();
    body["stream"] = Value::Bool(true);

    let text = reqwest::Client::new()
        .post(format!("{}/v1/responses", gateway.base_url))
        .header("x-api-key", "client-anthropic-key")
        .json(&body)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert!(text.contains("response.failed"));
    assert!(!text.contains("response.completed"));
}

#[tokio::test]
async fn response_translated_websocket_turn_uses_the_same_event_stream() {
    let upstream = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(concat!(
                    "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"model\":\"claude-sonnet-upstream\",\"usage\":{\"input_tokens\":1}}}\n\n",
                    "event: content_block_start\ndata: {\"type\":\"content_block_start\",\"content_block\":{\"type\":\"text\"}}\n\n",
                    "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"ws-ok\"}}\n\n",
                    "event: content_block_stop\ndata: {\"type\":\"content_block_stop\"}\n\n",
                    "event: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":1}}\n\n",
                    "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n"
                )),
        )
        .mount(&upstream)
        .await;
    let gateway = start_gateway(&upstream).await;
    let mut request = gateway
        .base_url
        .replacen("http://", "ws://", 1)
        .chars()
        .chain("/v1/responses".chars())
        .collect::<String>()
        .into_client_request()
        .unwrap();
    request
        .headers_mut()
        .insert("x-api-key", "client-anthropic-key".parse().unwrap());
    let (mut socket, _) = connect_async(request).await.unwrap();
    let mut body = request_body();
    body["type"] = Value::String("response.create".into());
    body["generate"] = Value::Bool(true);
    socket
        .send(Message::Text(body.to_string().into()))
        .await
        .unwrap();

    let mut event_types = Vec::new();
    loop {
        let frame = tokio::time::timeout(Duration::from_secs(5), socket.next())
            .await
            .expect("translated websocket response timed out")
            .expect("translated websocket closed early")
            .unwrap();
        let value: Value = serde_json::from_str(frame.to_text().unwrap()).unwrap();
        let kind = value["type"].as_str().unwrap().to_string();
        event_types.push(kind.clone());
        if matches!(
            kind.as_str(),
            "response.completed" | "response.failed" | "response.incomplete"
        ) {
            break;
        }
    }
    assert_eq!(
        event_types.first().map(String::as_str),
        Some("response.created")
    );
    assert!(event_types.contains(&"response.output_text.delta".into()));
    assert_eq!(
        event_types.last().map(String::as_str),
        Some("response.completed")
    );
}

#[tokio::test]
async fn websocket_restores_collaboration_tool_call_events() {
    let upstream = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(CollaborationRequest)
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(concat!(
                    "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{}}\n\n",
                    "event: content_block_start\ndata: {\"type\":\"content_block_start\",\"content_block\":{\"type\":\"tool_use\",\"id\":\"call_spawn\",\"name\":\"shunt_collaboration__spawn_agent\"}}\n\n",
                    "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"message\\\":\\\"work\\\"}\"}}\n\n",
                    "event: content_block_stop\ndata: {\"type\":\"content_block_stop\"}\n\n",
                    "event: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\"}}\n\n",
                    "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n"
                )),
        )
        .expect(1)
        .mount(&upstream)
        .await;
    let gateway = start_gateway_with_collaboration(&upstream, true).await;
    let mut request = format!(
        "{}/v1/responses",
        gateway.base_url.replacen("http://", "ws://", 1)
    )
    .into_client_request()
    .unwrap();
    request
        .headers_mut()
        .insert("x-api-key", "client-anthropic-key".parse().unwrap());
    let (mut socket, _) = connect_async(request).await.unwrap();
    let mut body = collaboration_body(true);
    body["type"] = Value::String("response.create".into());
    body["generate"] = Value::Bool(true);
    socket
        .send(Message::Text(body.to_string().into()))
        .await
        .unwrap();

    let mut frames = Vec::new();
    loop {
        let frame = tokio::time::timeout(Duration::from_secs(5), socket.next())
            .await
            .expect("collaboration websocket response timed out")
            .expect("collaboration websocket closed early")
            .unwrap();
        let value: Value = serde_json::from_str(frame.to_text().unwrap()).unwrap();
        let terminal = matches!(
            value["type"].as_str(),
            Some("response.completed" | "response.failed" | "response.incomplete")
        );
        frames.push(value);
        if terminal {
            break;
        }
    }
    let joined = serde_json::to_string(&frames).unwrap();
    assert!(joined.contains("\"namespace\":\"collaboration\""));
    assert!(joined.contains("\"name\":\"spawn_agent\""));
    assert!(joined.contains("\"encrypted_function_args\":[]"));
    assert!(!joined.contains("shunt_collaboration__spawn_agent"));
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
