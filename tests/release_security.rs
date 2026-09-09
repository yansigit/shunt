//! Durable release-artifact checks. Behavioral credential-file invariance is
//! exercised by auth::command_code::tests::matrix, not inferred from comments.
use serde_json::{json, Value};
use shunt::config::Config;

fn text(path: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .expect("durable release artifact")
}

fn notices_valid(notices: &str) -> bool {
    let Some((open, jcode)) = notices.split_once("## jcode Cursor transport") else {
        return false;
    };
    [
        "Cursor",
        "Command Code",
        "2026-09-07",
        "2026-09-08",
        "055c3ecf0de6c35f59195fc434d6b08525182b7f",
        "Copyright (c) 2026 opencodex contributors",
    ]
    .iter()
    .all(|s| open.contains(s))
        && [
            "Copyright (c) 2025 Jeremy Huang",
            "1jehuang/jcode",
            "e65e47c31af2ab79346458ff1511bea533930b59",
            "2026-09-09",
            "historical copy revision is unknown",
        ]
        .iter()
        .all(|s| jcode.contains(s))
        && [open, jcode].iter().all(|section| {
            [
                "MIT License",
                "Permission is hereby granted, free of charge",
                "The above copyright notice and this permission notice",
                "THE SOFTWARE IS PROVIDED \"AS IS\"",
                "LIABILITY",
            ]
            .iter()
            .all(|s| section.contains(s))
        })
}

fn sanitized(value: &Value) -> bool {
    match value {
        Value::String(s) => ![
            "sk-",
            "ghp_",
            "github_pat_",
            "Bearer ",
            "xoxb-",
            "AKIA",
            "-----BEGIN PRIVATE KEY",
        ]
        .iter()
        .any(|marker| s.contains(marker)),
        Value::Array(values) => values.iter().all(sanitized),
        Value::Object(map) => map.iter().all(|(key, value)| {
            ![
                "access_token",
                "refresh_token",
                "apiKey",
                "userId",
                "account_id",
                "project_id",
                "private_prompt",
                "raw_session",
            ]
            .contains(&key.as_str())
                && sanitized(value)
        }),
        _ => true,
    }
}

#[test]
fn release_security_notices_cover_translated_material() {
    let notices = text("THIRD-PARTY-NOTICES.md");
    assert!(
        notices_valid(&notices),
        "both upstream notices and distinct provenance dates are required"
    );
    for marker in [
        "Cursor",
        "2026-09-07",
        "2026-09-08",
        "Copyright (c) 2025 Jeremy Huang",
        "Permission is hereby granted, free of charge",
    ] {
        assert!(
            !notices_valid(&notices.replace(marker, "removed")),
            "missing {marker} must fail"
        );
    }
}

#[test]
fn release_security_ledger_rejects_sensitive_fields() {
    let source = text("docs/provider-release-evidence.md");
    let block = source
        .split_once("```release-ledger\n")
        .unwrap()
        .1
        .split_once("\n```")
        .unwrap()
        .0;
    let ledger: Value = serde_json::from_str(block).unwrap();
    assert!(sanitized(&ledger));
    for mutation in [
        json!({"access_token":"synthetic"}),
        json!({"nested":[{"private_prompt":"synthetic"}]}),
        json!({"provenance":"Bearer synthetic"}),
    ] {
        assert!(!sanitized(&mutation));
    }
    assert_eq!(ledger["go"]["admitted"], json!([]));
}

#[test]
fn credential_boundary_release_canonical_subscription_config() {
    for (url, expected) in [
        ("https://api.commandcode.ai", true),
        ("https://api.commandcode.ai/alpha/generate", true),
        ("http://api.commandcode.ai", false),
        ("https://api.commandcode.ai.evil.invalid", false),
        ("https://api.commandcode.ai:8443", false),
        ("https://api.commandcode.ai/other", false),
        ("https://api.commandcode.ai?secret=synthetic", false),
        ("https://synthetic@api.commandcode.ai", false),
    ] {
        let mut value = serde_json::to_value(Config::default()).unwrap();
        value["upstreams"] = json!([{"name":"cc","kind":"command_code","auth":{"mode":"command_code_oauth"},"base_url":url}]);
        value["server"]["default_provider"] = json!("cc");
        let config: Config = serde_json::from_value(value).unwrap();
        assert_eq!(config.validate().is_ok(), expected, "{url}");
    }
}
