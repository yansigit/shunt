use serde_json::json;
use shunt::adapters::command_code::efforts::{resolve, validate, MODEL_EFFORTS};
use shunt::config::{AuthMode, Config, ProviderKind};
use shunt::adapters::command_code::request::translate_request;

#[test]
fn command_code_translate_envelope_system_and_temperature() {
    let body = json!({"model":"zai-org/GLM-5.3", "stream":false,
        "system":[{"type":"text","text":"first"},{"type":"text","text":"second"}],
        "temperature":0.25, "output_config":{"effort":"high"},
        "messages":[{"role":"user","content":"hello"}]});
    let result = translate_request(&body, "zai-org/GLM-5.3", None);
    assert!(result.is_ok(), "supported envelope must compile");
    assert_eq!(result.unwrap(), json!({"config":{}, "memory":"", "taste":null, "skills":null,
        "permissionMode":"standard", "mode":"agent", "params":{
            "model":"zai-org/GLM-5.3", "stream":true, "system":"first\n\nsecond",
            "temperature":0.25, "reasoning_effort":"high", "max_tokens":64000,
            "tools":[], "messages":[{"role":"user","content":[{"type":"text","text":"hello"}]}]
        }}));
}

#[test]
fn command_code_translate_effort_exact_table() {
    let expected: &[(&str, &[&str])] = &[
        ("deepseek/deepseek-v4-pro", &["high", "max"]),
        ("deepseek/deepseek-v4-flash", &["high", "max"]),
        ("zai-org/GLM-5", &["high", "max"]),
        ("zai-org/GLM-5.1", &["high", "max"]),
        ("zai-org/GLM-5.2", &["high", "max"]),
        ("zai-org/GLM-5.2-Fast", &["high", "max"]),
        ("zai-org/GLM-5.3", &["low", "high", "max"]),
        (
            "meta/muse-spark-1.2",
            &["low", "medium", "high", "xhigh", "max"],
        ),
        (
            "meta/muse-spark-1.2-contributor",
            &["low", "medium", "high", "xhigh", "max"],
        ),
        (
            "meta/muse-spark-1.1",
            &["low", "medium", "high", "xhigh", "max"],
        ),
    ];
    assert_eq!(MODEL_EFFORTS, expected);
    for (model, ladder) in expected {
        assert!(validate(model, None).is_ok());
        for effort in [
            "", "none", "minimal", "low", "medium", "high", "xhigh", "max", "ultra", "HIGH",
        ] {
            assert_eq!(
                validate(model, Some(effort)).is_ok(),
                ladder.contains(&effort),
                "{model} {effort}"
            );
        }
        assert!(validate(&format!("{model} "), None).is_err());
    }
    for unknown in [
        "gpt-5.6-luna",
        "google/gemini-3.7-flash",
        "deepseek/deepseek-v4-flash-vision-exp",
        "zai-org/glm-5.3",
        "made-up",
    ] {
        assert!(validate(unknown, None).is_err());
        assert!(validate(unknown, Some("high")).is_err());
    }
}

#[test]
fn command_code_translate_effort_request_and_route_validation() {
    let model = "zai-org/GLM-5.3";
    assert_eq!(resolve(&json!({}), model, Some("high")), Ok(Some("high")));
    assert_eq!(
        resolve(
            &json!({"output_config":{"effort":"low"}}),
            model,
            Some("high")
        ),
        Ok(Some("low"))
    );
    for body in [
        json!({"output_config":null}),
        json!({"output_config":{"effort":null}}),
        json!({"output_config":{"effort":4}}),
        json!({"output_config":{"effort":"ultra"}}),
        json!({"output_config":{"format":{}}}),
    ] {
        assert!(resolve(&body, model, Some("high")).is_err());
    }
    assert!(resolve(&json!({}), model, Some("ultra")).is_err());
}

#[test]
fn command_code_translate_preset_subscription() {
    let mut config = Config::default();
    config.upstreams = serde_json::from_value(json!([
        {"name":"subscription", "provider":"command-code"},
        {"name":"api", "provider":"commandcode"}
    ]))
    .unwrap();
    config.server.default_provider = "subscription".into();
    let result = config.validate();
    assert!(
        result.is_ok(),
        "subscription preset must resolve: {:?}",
        result.err()
    );
    let config = result.unwrap();
    let sub = config.provider("subscription").unwrap();
    assert_eq!(sub.kind, ProviderKind::CommandCode);
    assert_eq!(sub.auth, AuthMode::CommandCodeOauth);
    assert_eq!(sub.base_url, "https://api.commandcode.ai");
    assert_eq!(sub.api_key_env, None);
    let api = config.provider("api").unwrap();
    assert_eq!(api.kind, ProviderKind::OpenAiChat);
    assert_eq!(api.auth, AuthMode::ApiKey);
    assert_eq!(api.base_url, "https://api.commandcode.ai/provider/v1");
    assert_eq!(
        api.api_key_env.as_deref(),
        Some("SHUNT_COMMANDCODE_API_KEY")
    );
}
