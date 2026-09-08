use serde_json::{json, Value};
use shunt::{config::{AuthMode, Config, ProviderKind}, server};
use wiremock::{matchers::{method, path, header}, Mock, MockServer, ResponseTemplate};

fn preset_config(base: Option<&str>) -> Config {
    let mut config = Config::default();
    let mut upstream = json!({"name":"cc", "provider":"commandcode"});
    if let Some(base) = base { upstream["base_url"] = json!(base); }
    config.upstreams = serde_json::from_value(json!([upstream])).unwrap();
    config.server.default_provider = "cc".into();
    let result = config.validate();
    assert!(result.is_ok(), "commandcode preset must resolve: {:?}", result.err());
    result.unwrap()
}

#[test]
fn command_code_api_preset_sibling_preservation() {
    let config = preset_config(None);
    let provider = &config.providers["cc"];
    assert_eq!(provider.kind, ProviderKind::OpenAiChat);
    assert_eq!(provider.auth, AuthMode::ApiKey);
    assert_eq!(provider.base_url, "https://api.commandcode.ai/provider/v1");
    assert_eq!(provider.api_key_env.as_deref(), Some("SHUNT_COMMANDCODE_API_KEY"));
}

#[tokio::test]
async fn command_code_api_preset_tracer() {
    let backend = MockServer::start().await;
    let mut config = preset_config(Some(&format!("{}/provider/v1", backend.uri())));
    // Use an isolated fixture env rather than any configured user key.
    config.providers.get_mut("cc").unwrap().api_key_env = Some("SHUNT_CC_API_TRACER_FIXTURE".into());
    std::env::set_var("SHUNT_CC_API_TRACER_FIXTURE", "synthetic-cc-key");
    Mock::given(method("POST"))
        .and(path("/provider/v1/chat/completions"))
        .and(header("authorization", "Bearer synthetic-cc-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"chatcmpl-fixture", "object":"chat.completion", "created":1, "model":"fixture-model",
            "choices":[{"index":0,"message":{"role":"assistant","content":"fixture reply"},"finish_reason":"stop"}],
            "usage":{"prompt_tokens":5,"completion_tokens":2,"total_tokens":7}
        }))).expect(1).mount(&backend).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (app, _, _) = server::build_router(config).unwrap();
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let response = reqwest::Client::new().post(format!("http://{addr}/v1/messages"))
        .json(&json!({"model":"fixture-model","max_tokens":64,"messages":[{"role":"user","content":"fixture"}]}))
        .send().await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["content"][0]["text"], "fixture reply");
    task.abort();
    let _ = task.await;
    std::env::remove_var("SHUNT_CC_API_TRACER_FIXTURE");
}
