use serde_json::json;
use shunt::config::{AuthMode, Config, ProviderKind};

#[test]
fn command_code_translate_preset_subscription() {
    let mut config = Config::default();
    config.upstreams = serde_json::from_value(json!([
        {"name":"subscription", "provider":"command-code"},
        {"name":"api", "provider":"commandcode"}
    ])).unwrap();
    config.server.default_provider = "subscription".into();
    let result = config.validate();
    assert!(result.is_ok(), "subscription preset must resolve: {:?}", result.err());
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
    assert_eq!(api.api_key_env.as_deref(), Some("SHUNT_COMMANDCODE_API_KEY"));
}
