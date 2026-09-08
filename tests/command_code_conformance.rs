//! Public config tests only. Canonical-host wire fixtures are crate-local TLS tests.
use serde_json::json;
use shunt::config::Config;

#[test]
fn command_code_config_product_pairing() {
    for (kind, auth, valid) in [
        ("command_code", "command_code_oauth", true),
        ("command_code", "api_key", false),
        ("openai_chat", "command_code_oauth", false),
        ("anthropic", "command_code_oauth", false),
    ] {
        let mut value = serde_json::to_value(Config::default()).unwrap();
        value["upstreams"] = json!([{"name":"cc","kind":kind,"auth":{"mode":auth},"base_url":"https://api.commandcode.ai"}]);
        value["server"]["default_provider"] = json!("cc");
        let config: Config = serde_json::from_value(value).unwrap();
        assert_eq!(config.validate().is_ok(), valid, "{kind}/{auth}");
    }
}
