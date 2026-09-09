use serde_json::{json, Value};
use std::fs;
const PHASE: &str = ".planning/phases/15-exact-opencode-go-evidence-gate";
const IDS: [&str; 4] = [
    "glm-5.3-flash",
    "omen-alpha",
    "muse-spark-1.3-contributor",
    "deepseek-v4-flash",
];
const REV: &str = "055c3ecf0de6c35f59195fc434d6b08525182b7f";

fn validate(v: &Value) -> Result<(), String> {
    let rows = v["candidates"].as_array().ok_or("missing candidates")?;
    if rows.len() != IDS.len() {
        return Err("candidate count".into());
    }
    for (row, id) in rows.iter().zip(IDS) {
        if row["model"] != id || row["destination"] != "https://opencode.ai/zen/go/v1" {
            return Err("exact identity".into());
        }
        for key in [
            "wire",
            "headers",
            "context",
            "modalities",
            "effort",
            "tools",
            "filtering",
            "terminal",
            "session",
            "provenance",
            "capture",
            "live",
        ] {
            if row.get(key).is_none() || row[key].is_null() {
                return Err(format!("missing {id}/{key}"));
            }
        }
        if row["provenance"]["class"] != "source"
            || row["provenance"]["revision"] != REV
            || row["provenance"]["inspected"] != "2026-09-08"
            || row["capture"] != "none"
            || row["live"] != "none"
        {
            return Err("false provenance".into());
        }
        if ["omen-alpha", "muse-spark-1.3-contributor"].contains(&id) && row["wire"] != "unknown" {
            return Err("family inference".into());
        }
    }
    if v["admitted"] != json!([]) {
        return Err("admission must remain empty".into());
    }
    let rejected = v["rejected"].as_array().ok_or("missing rejected")?;
    for reason in [
        "failed-evidence",
        "unknown-fields",
        "wrong-wire",
        "family-inference",
        "unsupported-effort",
        "wrong-terminal",
        "unverified-live",
    ] {
        if !rejected.iter().any(|r| {
            r["reason"] == reason && r["disposition"].as_str().is_some_and(|s| !s.is_empty())
        }) {
            return Err(format!("missing rejection {reason}"));
        }
    }
    Ok(())
}

fn ledger() -> Value {
    let path = format!("{}/{PHASE}/15-EVIDENCE.md", env!("CARGO_MANIFEST_DIR"));
    assert!(
        std::path::Path::new(&path).is_file(),
        "required four-candidate evidence ledger has not been authored"
    );
    let text = fs::read_to_string(path).expect("ledger file exists");
    let (_, rest) = text
        .split_once("```opencode-go-ledger\n")
        .expect("ledger fence");
    let (block, _) = rest.split_once("\n```").expect("closing fence");
    serde_json::from_str(block).expect("valid JSON")
}

#[test]
fn opencode_go_ledger() {
    validate(&ledger()).expect("complete source-only ledger");
    let path = format!(
        "{}/{PHASE}/15-EDGE-COVERAGE.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let edges: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    let rows = edges["items"].as_array().unwrap();
    assert_eq!(rows.len(), 8);
    let unresolved: Vec<_> = rows
        .iter()
        .filter(|r| r["status"] == "unresolved")
        .collect();
    assert_eq!(unresolved.len(), 1);
    assert_eq!(unresolved[0]["requirement_id"], "OGO-02");
    assert!(unresolved[0]["verification"].is_null());
}

#[test]
fn opencode_go_ledger_rejects_mutations() {
    let base = ledger();
    for key in [
        "wire",
        "headers",
        "context",
        "modalities",
        "effort",
        "tools",
        "filtering",
        "terminal",
        "session",
        "provenance",
        "capture",
        "live",
    ] {
        let mut v = base.clone();
        v["candidates"][0].as_object_mut().unwrap().remove(key);
        assert!(validate(&v).is_err(), "missing {key}");
    }
    let mut v = base.clone();
    v["admitted"] = json!([{"model":"glm-5.3-flash"}]);
    assert!(validate(&v).is_err());
    for class in ["capture", "live"] {
        let mut v = base.clone();
        v["candidates"][0]["provenance"]["class"] = json!(class);
        assert!(validate(&v).is_err());
    }
    for index in [1, 2] {
        let mut v = base.clone();
        v["candidates"][index]["wire"] = json!("openai-chat");
        assert!(validate(&v).is_err());
    }
    let mut v = base.clone();
    v["rejected"] = json!([]);
    assert!(validate(&v).is_err());
}
