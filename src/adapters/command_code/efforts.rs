//! Exact source-derived tuples, not an account catalog. OpenCodex 055c3ecf,
//! inspected 2026-09-08; see THIRD-PARTY-NOTICES.md. Reporter-only rows excluded.
use serde_json::Value;

const HIGH_MAX: &[&str] = &["high", "max"];
const MUSE: &[&str] = &["low", "medium", "high", "xhigh", "max"];
pub const MODEL_EFFORTS: &[(&str, &[&str])] = &[
    ("deepseek/deepseek-v4-pro", HIGH_MAX),
    ("deepseek/deepseek-v4-flash", HIGH_MAX),
    ("zai-org/GLM-5", HIGH_MAX),
    ("zai-org/GLM-5.1", HIGH_MAX),
    ("zai-org/GLM-5.2", HIGH_MAX),
    ("zai-org/GLM-5.2-Fast", HIGH_MAX),
    ("zai-org/GLM-5.3", &["low", "high", "max"]),
    ("meta/muse-spark-1.2", MUSE),
    ("meta/muse-spark-1.2-contributor", MUSE),
    ("meta/muse-spark-1.1", MUSE),
];

pub fn validate(model: &str, effort: Option<&str>) -> Result<(), &'static str> {
    let (_, supported) = MODEL_EFFORTS
        .iter()
        .find(|(id, _)| *id == model)
        .ok_or("unsupported Command Code model")?;
    // Absence is accepted by the pinned source. Explicit values are never
    // stripped, case-folded, clamped, or converted to omission (including none).
    if effort.is_some_and(|e| !supported.contains(&e)) {
        return Err("unsupported Command Code reasoning effort");
    }
    Ok(())
}

pub fn resolve<'a>(
    body: &'a Value,
    model: &str,
    route_effort: Option<&'a str>,
) -> Result<Option<&'a str>, &'static str> {
    let explicit = match body.get("output_config") {
        None => None,
        Some(value) => {
            let object = value.as_object().ok_or("output_config must be an object")?;
            if object.keys().any(|key| key != "effort") {
                return Err("unsupported Command Code output_config field");
            }
            object
                .get("effort")
                .map(|v| v.as_str().ok_or("effort must be a string"))
                .transpose()?
        }
    };
    let effort = explicit.or(route_effort);
    validate(model, effort)?;
    Ok(effort)
}
