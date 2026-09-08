use serde_json::json;
use shunt::adapters::command_code::efforts::{resolve, validate, MODEL_EFFORTS};
use shunt::adapters::command_code::request::translate_request;
use shunt::config::{AuthMode, Config, ProviderKind};

#[test]
fn command_code_translate_tools_adjacent_and_missing_result() {
    let body = json!({"model":"zai-org/GLM-5.3",
        "tools":[{"name":"delegate","description":"Plaintext subagent","input_schema":{"type":"object"}}],
        "messages":[
            {"role":"assistant","content":[{"type":"tool_use","id":"call-a","name":"delegate","input":{"task":"summarize"}}]},
            {"role":"user","content":[{"type":"tool_result","tool_use_id":"call-a","content":"subagent result"}]},
            {"role":"assistant","content":[{"type":"tool_use","id":"call-b","name":"delegate","input":{}}]},
            {"role":"user","content":"continue"}
        ]});
    let result = translate_request(&body, "zai-org/GLM-5.3", None);
    assert!(result.is_ok(), "supported tool history must compile");
    let wire = result.unwrap();
    let messages = wire["params"]["messages"].as_array().unwrap();
    assert_eq!(messages[0]["content"][0]["toolCallId"], "call-a");
    assert_eq!(messages[1], json!({"role":"tool","content":[{"type":"tool-result","toolCallId":"call-a","toolName":"delegate","output":{"type":"text","value":"subagent result"}}]}));
    assert_eq!(messages[3]["content"][0]["toolCallId"], "call-b");
    assert_eq!(messages[3]["content"][0]["output"]["type"], "error-text");
    assert!(messages[3]["content"][0]["output"]["value"].as_str().unwrap().contains("execution status unknown"));
    assert_eq!(messages[4]["content"][0]["text"], "continue");
}

#[test]
fn command_code_translate_session_scope_and_request_local_fallback() {
    use shunt::adapters::command_code::request::{session_id, MAX_CONVERSATION_ID_BYTES};
    let a = session_id("synthetic-token-a", Some("conversation-a")).unwrap();
    assert_eq!(
        a,
        session_id("synthetic-token-a", Some("conversation-a")).unwrap()
    );
    assert_ne!(
        a,
        session_id("synthetic-token-b", Some("conversation-a")).unwrap()
    );
    assert_ne!(
        a,
        session_id("synthetic-token-a", Some("conversation-b")).unwrap()
    );
    assert_ne!(
        session_id("ab", Some("c")).unwrap(),
        session_id("a", Some("bc")).unwrap()
    );
    assert_eq!(uuid::Uuid::parse_str(&a).unwrap().get_version_num(), 4);
    assert!(!a.contains("synthetic") && !a.contains("conversation"));
    assert_ne!(
        session_id("synthetic-token-a", None).unwrap(),
        session_id("synthetic-token-a", None).unwrap()
    );
    for invalid in [
        "".to_string(),
        "a b".into(),
        "a\r\nb".into(),
        "x".repeat(MAX_CONVERSATION_ID_BYTES + 1),
    ] {
        assert!(session_id("synthetic", Some(&invalid)).is_err());
    }
    assert!(session_id("synthetic", Some(&"x".repeat(MAX_CONVERSATION_ID_BYTES))).is_ok());
}

#[test]
fn command_code_translate_headers_exact_and_sensitive() {
    use shunt::adapters::command_code::request::headers;
    let h = headers("synthetic-MixedCase-key", Some("conversation-a")).unwrap();
    assert_eq!(h.len(), 8);
    for (name, value) in [
        ("authorization", "Bearer synthetic-MixedCase-key"),
        ("content-type", "application/json"),
        ("user-agent", "cli"),
        ("x-command-code-version", "0.52.1"),
        ("x-cli-environment", "production"),
        ("x-taste-learning", "false"),
        ("x-co-flag", "false"),
    ] {
        assert_eq!(h[name], value);
    }
    assert!(h["authorization"].is_sensitive());
    assert!(!format!("{h:?}").contains("synthetic-MixedCase-key"));
    assert!(h.get("x-project-slug").is_none());
    assert_eq!(
        h["x-session-id"],
        headers("synthetic-MixedCase-key", Some("conversation-a")).unwrap()["x-session-id"]
    );
    for token in ["", "bad token", "bad\r\nheader"] {
        assert!(headers(token, None).is_err());
    }
}

#[test]
fn command_code_translate_envelope_defaults_and_rejections() {
    let model = "zai-org/GLM-5.3";
    let base = json!({"model":model, "messages":[{"role":"user","content":"hello"}]});
    let plain = translate_request(&base, model, None).unwrap();
    assert_eq!(plain["config"], json!({}));
    assert_eq!(plain["params"]["max_tokens"], 64000);
    assert!(plain["params"].get("reasoning_effort").is_none());
    assert!(plain["params"].get("temperature").is_none());
    for (field, value) in [
        ("temperature", json!("0.2")),
        ("system", json!([{"type":"image"}])),
        ("max_tokens", json!(0)),
        ("stream", json!("true")),
        ("cwd", json!("private")),
    ] {
        let mut bad = base.clone();
        bad[field] = value;
        assert!(translate_request(&bad, model, None).is_err(), "{field}");
    }
    let mut streaming = base;
    streaming["stream"] = json!(true);
    assert_eq!(
        translate_request(&streaming, model, None).unwrap()["params"]["stream"],
        true
    );
}

#[test]
fn command_code_translate_envelope_system_and_temperature() {
    let body = json!({"model":"zai-org/GLM-5.3", "stream":false,
        "system":[{"type":"text","text":"first"},{"type":"text","text":"second"}],
        "temperature":0.25, "output_config":{"effort":"high"},
        "messages":[{"role":"user","content":"hello"}]});
    let result = translate_request(&body, "zai-org/GLM-5.3", None);
    assert!(result.is_ok(), "supported envelope must compile");
    assert_eq!(
        result.unwrap(),
        json!({"config":{}, "memory":"", "taste":null, "skills":null,
        "permissionMode":"standard", "mode":"agent", "params":{
            "model":"zai-org/GLM-5.3", "stream":true, "system":"first\n\nsecond",
            "temperature":0.25, "reasoning_effort":"high", "max_tokens":64000,
            "tools":[], "messages":[{"role":"user","content":[{"type":"text","text":"hello"}]}]
        }})
    );
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
