use std::fs;

use crate::{
    config::{Config, ProviderKind},
    proxy::capability::enforce_opencode_go_admission,
    routing::{AdapterKind, Route},
};

#[test]
fn opencode_go_config_acceptance() {
    let path = std::env::temp_dir().join(format!(
        "shunt-opencode-go-acceptance-{}-{}.toml",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock before epoch")
            .as_nanos()
    ));
    let synthetic = r#"
[server]
default_provider = "go"

[[upstreams]]
name = "go"
provider = "opencode-go"
"#;
    fs::write(&path, synthetic).expect("write synthetic config");
    let result = Config::load(Some(&path));
    let _ = fs::remove_file(&path);
    assert!(result.is_ok(), "{result:?}");
}

fn route(provider: &str) -> Route {
    Route {
        provider: provider.into(),
        adapter: AdapterKind::OpenAiChat,
        model: "alias".into(),
        upstream_model: "model".into(),
        effort: None,
        service_tier: None,
    }
}

#[test]
fn opencode_go_admission_is_empty_and_preserves_generic_fallbacks() {
    let mut config = Config::default();
    config.providers.insert(
        "go".into(),
        crate::config::ProviderConfig {
            kind: ProviderKind::OpenCodeGo,
            base_url: "https://opencode.ai/zen/go/v1".into(),
            auth: crate::config::AuthMode::ApiKey,
            api_key_env: Some("SHUNT_OPENCODE_GO_API_KEY".into()),
            ..config.providers["openai"].clone()
        },
    );
    let mut routes = vec![route("openai"), route("go")];
    assert!(enforce_opencode_go_admission(&config, &mut routes).is_ok());
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].provider, "openai");

    let mut primary = vec![route("go")];
    assert!(enforce_opencode_go_admission(&config, &mut primary).is_err());
}
