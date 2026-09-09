//! Release ledger validator for the cross-provider evidence matrix
//! (docs/provider-release-evidence.md). Hermetic: parses one fenced JSON
//! block from the durable docs artifact and asserts exact-tuple identity,
//! scenario evidence, provenance, sanitization, and the D-02 empty Go
//! admission. Sends no network requests.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;

const LEDGER_PATH: &str = "docs/provider-release-evidence.md";
const FENCE: &str = "```release-ledger\n";
/// Canonical scenario keys; every row must account for each one.
const SCENARIOS: [&str; 9] = [
    "normal",
    "stream",
    "tools",
    "terminal",
    "malformed",
    "truncation",
    "auth",
    "cancellation",
    "retry",
];
/// Wire destination of the active Cursor agent adapter (src/adapters/cursor/agent.rs).
const CURSOR_AGENT_WIRE: &str = "https://agentn.global.api5.cursor.sh/agent.v1.AgentService/Run";

fn is_secret_like(s: &str) -> bool {
    ["sk-", "ghp_", "github_pat_", "AKIA", "Bearer ", "xoxb-"]
        .iter()
        .any(|marker| s.contains(marker))
}

fn walk_secret_like(value: &Value, path: &str, hits: &mut Vec<String>) {
    match value {
        Value::String(s) => {
            if is_secret_like(s) {
                hits.push(path.to_string());
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                walk_secret_like(item, &format!("{path}[{index}]"), hits);
            }
        }
        Value::Object(map) => {
            for (key, item) in map {
                walk_secret_like(item, &format!("{path}.{key}"), hits);
            }
        }
        _ => {}
    }
}

fn tuple(row: &Value) -> Result<(String, String, String, String), String> {
    let mut fields = Vec::new();
    for key in ["provider", "auth", "model", "wire"] {
        let value = row[key]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| format!("row identity field {key}"))?;
        fields.push(value.to_string());
    }
    Ok((
        fields.remove(0),
        fields.remove(0),
        fields.remove(0),
        fields.remove(0),
    ))
}

fn validate_scenario(row_id: &str, name: &str, entry: &Value) -> Result<(), String> {
    match entry["status"].as_str() {
        Some("covered") => {
            let evidence = entry["evidence"]
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| format!("{row_id}/{name}: covered without evidence"))?;
            let (file, test) = evidence
                .split_once("#")
                .ok_or_else(|| format!("{row_id}/{name}: evidence {evidence} is not file#test"))?;
            let path = format!("{}/{file}", env!("CARGO_MANIFEST_DIR"));
            let source = fs::read_to_string(&path)
                .map_err(|_| format!("{row_id}/{name}: evidence file {file} missing"))?;
            if test.is_empty() || !source.contains(&format!("fn {test}(")) {
                return Err(format!("{row_id}/{name}: test {evidence} not found"));
            }
        }
        Some("not_applicable") => {
            entry["rationale"]
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| format!("{row_id}/{name}: not_applicable without rationale"))?;
        }
        _ => {
            return Err(format!(
                "{row_id}/{name}: missing or invalid scenario entry"
            ))
        }
    }
    Ok(())
}

fn validate_row(row: &Value) -> Result<(), String> {
    let id = tuple(row)?;
    let row_id = format!("{}/{}/{}/{}", id.0, id.1, id.2, id.3);
    for key in ["scenarios", "provenance", "capture", "live"] {
        if row.get(key).is_none() || row[key].is_null() {
            return Err(format!("{row_id}: missing {key}"));
        }
    }
    for name in SCENARIOS {
        validate_scenario(&row_id, name, &row["scenarios"][name])?;
    }
    let provenance = &row["provenance"];
    match provenance["class"].as_str() {
        Some("source") => {
            for key in ["repository", "revision", "inspected", "sanitization"] {
                if provenance[key].as_str().filter(|s| !s.is_empty()).is_none() {
                    return Err(format!("{row_id}: source provenance missing {key}"));
                }
            }
        }
        _ => return Err(format!("{row_id}: invalid provenance class")),
    }
    for key in ["capture", "live"] {
        if row[key] != "none" {
            return Err(format!(
                "{row_id}: no captured or live proof has been recorded"
            ));
        }
    }
    Ok(())
}

fn validate(v: &Value) -> Result<(), String> {
    let rows = v["rows"].as_array().ok_or("missing rows")?;
    if rows.len() != 45 {
        return Err("the finite inventory requires 45 exact documented model rows".into());
    }
    let mut tuples = Vec::new();
    for row in rows {
        let id = tuple(row)?;
        if [
            "passthrough",
            "catalog-declared",
            "claude-builtin-catalog",
            "not-admitted",
        ]
        .contains(&id.2.as_str())
        {
            return Err("a contract placeholder is not an exact model identity".into());
        }
        if tuples.contains(&id) {
            return Err(format!("duplicate tuple {id:?}"));
        }
        tuples.push(id.clone());
        validate_row(row)?;
    }
    let mut sorted = tuples.clone();
    sorted.sort();
    if tuples != sorted {
        return Err("rows are not in deterministic tuple order".into());
    }
    // Freeze the reviewed finite identity set independently of the mutable ledger.
    // Changing a model/credential/destination requires a new explicit review.
    let digest = format!("{:x}", Sha256::digest(serde_json::to_vec(&tuples).unwrap()));
    if digest != "fac32c4f8a23712c0d372dc1743b2f9a14efdea88581a7376daef8a6881b59de" {
        return Err("reviewed exact tuple inventory changed".into());
    }
    let contracts = v["unbound_contracts"]
        .as_array()
        .ok_or("missing unbound contracts")?;
    let names: Vec<_> = contracts
        .iter()
        .map(|r| r["provider"].as_str().unwrap_or_default())
        .collect();
    if names
        != [
            "antigravity-cli",
            "commandcode",
            "custom-anthropic",
            "custom-openai-chat",
            "gemini",
            "kimi-code",
        ]
    {
        return Err("generic/account-catalog contract inventory changed".into());
    }
    for contract in contracts {
        if contract.get("model").is_some()
            || contract["selection"].as_str().is_none_or(str::is_empty)
        {
            return Err("unbound contract must not invent a model identity".into());
        }
        let mut checked = contract.clone();
        checked["model"] = json!("contract-only");
        validate_row(&checked)?;
    }
    let go = &v["go"];
    if go["admitted"] != json!([]) {
        return Err("Go admission must remain empty".into());
    }
    if go["policy"].as_str().filter(|s| !s.is_empty()).is_none() {
        return Err("missing Go rejection policy".into());
    }
    for row in rows {
        if row["provider"] == "cursor" {
            let wire = row["wire"].as_str().unwrap_or_default();
            if wire != CURSOR_AGENT_WIRE {
                return Err(format!(
                    "cursor row must record the active agent wire, got {wire}"
                ));
            }
        }
    }
    let mut hits = Vec::new();
    walk_secret_like(v, "$", &mut hits);
    if !hits.is_empty() {
        return Err(format!("unsanitized secret markers at {}", hits.join(", ")));
    }
    Ok(())
}

/// Parse the single fenced ledger block from markdown text.
fn parse_ledger_text(text: &str) -> Result<Value, String> {
    let (_, rest) = text.split_once(FENCE).ok_or("ledger fence missing")?;
    let (block, _) = rest.split_once("\n```").ok_or("closing fence missing")?;
    serde_json::from_str(block).map_err(|error| format!("invalid ledger JSON: {error}"))
}

fn ledger_from_path(path: &str) -> Value {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("release ledger {path} unreadable: {error}"));
    parse_ledger_text(&text).expect("release ledger parses")
}

fn ledger() -> Value {
    let path = format!("{}/{LEDGER_PATH}", env!("CARGO_MANIFEST_DIR"));
    assert!(
        std::path::Path::new(&path).is_file(),
        "required provider release evidence ledger has not been authored"
    );
    ledger_from_path(&path)
}

#[test]
fn release_matrix_ledger() {
    let ledger = ledger();
    validate(&ledger).expect("complete finite release ledger");
}

#[test]
fn release_matrix_ledger_validation_is_idempotent() {
    let first = ledger();
    let second = ledger();
    assert_eq!(first, second, "repeated parses must be identical");
    validate(&first).expect("first validation");
    validate(&second).expect("second validation");
}

#[test]
fn release_matrix_ledger_rejects_mutations() {
    let base = ledger();
    let mut invented = base.clone();
    invented["rows"][0]["model"] = json!("invented-model");
    assert!(
        validate(&invented).is_err(),
        "invented model identity must fail"
    );
    let mut omitted = base.clone();
    omitted["rows"].as_array_mut().unwrap().pop();
    assert!(validate(&omitted).is_err(), "missing model row must fail");
    let mut unbound = base.clone();
    unbound["unbound_contracts"][0]["model"] = json!("invented-model");
    assert!(
        validate(&unbound).is_err(),
        "contract must not infer a model"
    );
    for key in ["capture", "live"] {
        let mut fabricated = base.clone();
        fabricated["rows"][0][key] = json!("pass");
        assert!(validate(&fabricated).is_err(), "fabricated {key} must fail");
    }
    let row_keys = [
        "provider",
        "auth",
        "model",
        "wire",
        "scenarios",
        "provenance",
        "capture",
        "live",
    ];
    for key in row_keys {
        let mut v = base.clone();
        v["rows"][0].as_object_mut().unwrap().remove(key);
        assert!(validate(&v).is_err(), "missing {key} must be rejected");
    }
    for name in SCENARIOS {
        let mut v = base.clone();
        v["rows"][0]["scenarios"]
            .as_object_mut()
            .unwrap()
            .remove(name);
        assert!(validate(&v).is_err(), "empty {name} scenario must fail");
    }
    let mut v = base.clone();
    v["rows"][0]["scenarios"]["normal"] = json!({"status": "covered"});
    assert!(validate(&v).is_err(), "covered without evidence must fail");
    let mut v = base.clone();
    v["rows"][0]["scenarios"]["normal"] = json!({"status": "not_applicable"});
    assert!(
        validate(&v).is_err(),
        "not_applicable without rationale must fail"
    );
    let mut v = base.clone();
    let duplicate = base["rows"][0].clone();
    v["rows"].as_array_mut().unwrap().push(duplicate);
    assert!(validate(&v).is_err(), "duplicate tuple identity must fail");
    let mut v = base.clone();
    v["rows"][0]["provenance"]
        .as_object_mut()
        .unwrap()
        .remove("revision");
    assert!(
        validate(&v).is_err(),
        "missing provenance revision must fail"
    );
    let mut v = base.clone();
    v["rows"][0]["provenance"]["class"] = json!("fabricated");
    assert!(validate(&v).is_err(), "invalid provenance class must fail");
    let mut v = base.clone();
    v["rows"][0]["scenarios"]["normal"] =
        json!({"status": "covered", "evidence": "tests/nope.rs#missing_test"});
    assert!(validate(&v).is_err(), "nonexistent evidence test must fail");
    let mut v = base.clone();
    let cursor = v["rows"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["provider"] == "cursor")
        .unwrap();
    cursor["wire"] = json!("https://api2.cursor.sh");
    assert!(validate(&v).is_err(), "preset cursor wire must be rejected");
    let mut v = base.clone();
    v["go"]["admitted"] = json!([{"provider": "opencode-go"}]);
    assert!(validate(&v).is_err(), "non-empty Go admission must fail");
    let mut v = base.clone();
    v["rows"][0]["auth"] = json!("sk-unsanitized");
    assert!(validate(&v).is_err(), "secret-like marker must fail");
    // Parser round-trip: a valid fixture written to a temporary path must
    // validate through the same file-based entrypoint.
    let mut valid = base.clone();
    valid["rows"] = json!([]);
    let dir = std::env::temp_dir().join(format!("shunt-release-matrix-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let fixture = dir.join("fixture.md");
    std::fs::write(
        &fixture,
        format!("prose\n```release-ledger\n{}\n```\n", valid),
    )
    .unwrap();
    let parsed = ledger_from_path(fixture.to_str().unwrap());
    assert!(
        parse_ledger_text("no fence").is_err(),
        "missing fence must fail"
    );
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(parsed, valid);
    assert!(validate(&parsed).is_err(), "empty rows fixture must fail");
}
