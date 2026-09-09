//! Minimal capability gate for ordered heterogeneous fallback chains.
//!
//! The configured primary is always preserved. Only later candidates are
//! excluded, and only for request features whose current adapter behavior is
//! already known. This is deliberately not a general capability registry.

use serde_json::Value;

use crate::{
    config::{Config, ProviderKind},
    routing::{AdapterKind, Route},
};

/// Evidence-gated OpenCode Go admission. The allowlist is intentionally empty
/// until an exact captured/live tuple is verified; identity is taken only from
/// the explicit provider kind, never inferred from URLs or model names.
pub(crate) fn enforce_opencode_go_admission(
    config: &Config,
    routes: &mut Vec<Route>,
) -> Result<(), &'static str> {
    let mut index = 0;
    while index < routes.len() {
        let route = &routes[index];
        let is_go = config
            .providers
            .get(&route.provider)
            .is_some_and(|provider| provider.kind == ProviderKind::OpenCodeGo);
        if !is_go {
            index += 1;
            continue;
        }
        if index == 0 {
            return Err("OpenCode Go selection is not admitted by exact evidence");
        }
        routes.remove(index);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Requirements {
    tools: bool,
    base64_images: bool,
    url_images: bool,
    structured_output: bool,
    explicit_effort: bool,
    million_context: bool,
}

impl Requirements {
    fn extract(request: &Value, requested_model: &str) -> Self {
        let mut requirements = Self {
            tools: request
                .get("tools")
                .and_then(Value::as_array)
                .is_some_and(|tools| !tools.is_empty()),
            structured_output: request
                .pointer("/output_config/format")
                .is_some_and(|format| !format.is_null()),
            explicit_effort: request
                .pointer("/output_config/effort")
                .is_some_and(Value::is_string),
            million_context: requested_model.ends_with("[1m]") || requested_model.ends_with("[1M]"),
            ..Self::default()
        };
        let Some(messages) = request.get("messages").and_then(Value::as_array) else {
            return requirements;
        };
        let mut content = messages
            .iter()
            .filter_map(|message| message.get("content"))
            .collect::<Vec<_>>();
        while let Some(value) = content.pop() {
            let Some(blocks) = value.as_array() else {
                continue;
            };
            for block in blocks {
                let Some(block) = block.as_object() else {
                    continue;
                };
                match block.get("type").and_then(Value::as_str) {
                    Some("image") => match block
                        .get("source")
                        .and_then(Value::as_object)
                        .and_then(|source| source.get("type"))
                        .and_then(Value::as_str)
                    {
                        Some("base64") => requirements.base64_images = true,
                        Some("url") => requirements.url_images = true,
                        _ => {}
                    },
                    Some("tool_result") => {
                        if let Some(nested) = block.get("content") {
                            content.push(nested);
                        }
                    }
                    _ => {}
                }
            }
        }
        requirements
    }

    fn incompatibilities(self, adapter: &AdapterKind) -> Vec<&'static str> {
        if self.million_context {
            return vec!["1m-context-unknown"];
        }
        let mut reasons = Vec::new();
        match adapter {
            AdapterKind::Anthropic => {}
            AdapterKind::Responses => {
                if self.structured_output {
                    reasons.push("structured-output");
                }
            }
            AdapterKind::Gemini | AdapterKind::Cursor => {
                if self.url_images {
                    reasons.push("url-images");
                }
                if self.structured_output {
                    reasons.push("structured-output");
                }
                if self.explicit_effort {
                    reasons.push("reasoning-effort");
                }
            }
            AdapterKind::AntigravityCli => {
                if self.tools {
                    reasons.push("tools");
                }
                if self.base64_images || self.url_images {
                    reasons.push("images");
                }
                if self.structured_output {
                    reasons.push("structured-output");
                }
                if self.explicit_effort {
                    reasons.push("reasoning-effort");
                }
            }
            AdapterKind::CommandCode => {
                if self.structured_output {
                    reasons.push("structured-output");
                }
                // Exact model/effort validation needs the route and is below.
            }
            AdapterKind::OpenAiChat => {
                if self.structured_output {
                    reasons.push("structured-output");
                }
                if self.explicit_effort {
                    reasons.push("reasoning-effort");
                }
            }
        }
        reasons
    }
}

pub(super) fn filter_fallbacks(routes: &mut Vec<Route>, request: &Value, requested_model: &str) {
    if routes.len() < 2 {
        return;
    }
    let requirements = Requirements::extract(request, requested_model);
    let mut index = 1;
    while index < routes.len() {
        let route = &routes[index];
        let mut reasons = requirements.incompatibilities(&route.adapter);
        if route.adapter == AdapterKind::CommandCode
            && crate::adapters::command_code::efforts::resolve(
                request,
                &route.upstream_model,
                route.effort.as_deref(),
            )
            .is_err()
        {
            reasons.push("model-or-reasoning-effort");
        }
        if reasons.is_empty() {
            index += 1;
            continue;
        }
        let route = routes.remove(index);
        tracing::warn!(
            provider = %route.provider,
            model = %route.upstream_model,
            reasons = %reasons.join(","),
            "fallback excluded by request capabilities"
        );
        crate::metrics::record_failover(&route.provider, "capability_excluded");
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn route(provider: &str, adapter: AdapterKind) -> Route {
        Route {
            provider: provider.into(),
            adapter,
            model: "alias".into(),
            upstream_model: "upstream".into(),
            effort: None,
            service_tier: None,
        }
    }

    #[test]
    fn command_code_translate_effort_fallback_exact_admission() {
        for (model, effort, accepted) in [
            ("zai-org/GLM-5.3", Some("high"), true),
            ("zai-org/GLM-5.3", Some("low"), true),
            ("zai-org/GLM-5.3", Some("medium"), false),
            ("zai-org/GLM-5.3", Some("ultra"), false),
            ("zai-org/glm-5.3", None, false),
            ("gpt-5.6-luna", None, false),
            ("google/gemini-3.7-flash", None, false),
            ("deepseek/deepseek-v4-flash-vision-exp", None, false),
            ("unknown", None, false),
            ("deepseek/deepseek-v4-flash", None, true),
        ] {
            let mut candidate = route("subscription", AdapterKind::CommandCode);
            candidate.upstream_model = model.into();
            let mut routes = vec![route("primary", AdapterKind::Anthropic), candidate];
            let request =
                effort.map_or_else(|| json!({}), |e| json!({"output_config":{"effort":e}}));
            filter_fallbacks(&mut routes, &request, "alias");
            assert_eq!(routes.len() == 2, accepted, "{model} {effort:?}");
        }
    }

    #[test]
    fn extract_detects_known_requirements_without_recursive_schema_scanning() {
        let request = json!({
            "tools":[{"name":"lookup","input_schema":{"type":"object","properties":{"fake":{"type":"image"}}}}],
            "output_config":{"format":{"type":"json_schema"},"effort":"high"},
            "messages":[{"role":"user","content":[
                {"type":"image","source":{"type":"url","url":"https://example.com/a.png"}},
                {"type":"tool_result","tool_use_id":"x","content":[
                    {"type":"image","source":{"type":"base64","media_type":"image/png","data":"AA=="}}
                ]}
            ]}]
        });
        assert_eq!(
            Requirements::extract(&request, "alias[1M]"),
            Requirements {
                tools: true,
                base64_images: true,
                url_images: true,
                structured_output: true,
                explicit_effort: true,
                million_context: true,
            }
        );
        assert_eq!(
            Requirements::extract(&json!({"messages":"bad"}), "alias"),
            Requirements::default()
        );
    }

    #[test]
    fn openai_chat_fallback_matches_completed_request_whitelist() {
        for request in [
            json!({"output_config":{"format":{"type":"json_schema"}}}),
            json!({"output_config":{"effort":"high"}}),
        ] {
            let mut routes = vec![
                route("primary", AdapterKind::Anthropic),
                route("chat", AdapterKind::OpenAiChat),
                route("compatible", AdapterKind::Anthropic),
            ];
            filter_fallbacks(&mut routes, &request, "alias");
            assert_eq!(
                routes
                    .iter()
                    .map(|r| r.provider.as_str())
                    .collect::<Vec<_>>(),
                vec!["primary", "compatible"]
            );
        }
        let supported = Requirements {
            tools: true,
            base64_images: true,
            url_images: true,
            ..Requirements::default()
        };
        assert!(supported
            .incompatibilities(&AdapterKind::OpenAiChat)
            .is_empty());
        let mut primary = vec![
            route("chat", AdapterKind::OpenAiChat),
            route("compatible", AdapterKind::Anthropic),
        ];
        filter_fallbacks(
            &mut primary,
            &json!({"output_config":{"effort":"high"}}),
            "alias",
        );
        assert_eq!(primary[0].provider, "chat");
    }

    #[test]
    fn eligibility_matrix_matches_existing_adapter_fidelity() {
        let all = Requirements {
            tools: true,
            base64_images: true,
            url_images: true,
            structured_output: true,
            explicit_effort: true,
            million_context: false,
        };
        assert!(all.incompatibilities(&AdapterKind::Anthropic).is_empty());
        assert_eq!(
            all.incompatibilities(&AdapterKind::Responses),
            vec!["structured-output"]
        );
        assert_eq!(
            all.incompatibilities(&AdapterKind::Gemini),
            vec!["url-images", "structured-output", "reasoning-effort"]
        );
        assert_eq!(
            all.incompatibilities(&AdapterKind::Cursor),
            vec!["url-images", "structured-output", "reasoning-effort"]
        );
        assert_eq!(
            all.incompatibilities(&AdapterKind::AntigravityCli),
            vec!["tools", "images", "structured-output", "reasoning-effort"]
        );
    }

    #[test]
    fn primary_is_preserved_and_only_incompatible_fallbacks_are_removed() {
        let mut routes = vec![
            route("primary", AdapterKind::AntigravityCli),
            route("responses", AdapterKind::Responses),
            route("anthropic", AdapterKind::Anthropic),
        ];
        filter_fallbacks(
            &mut routes,
            &json!({"output_config":{"format":{"type":"json_schema"}}}),
            "alias",
        );
        assert_eq!(
            routes
                .iter()
                .map(|route| route.provider.as_str())
                .collect::<Vec<_>>(),
            vec!["primary", "anthropic"]
        );

        filter_fallbacks(&mut routes, &json!({}), "alias[1m]");
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].provider, "primary");
    }

    #[test]
    fn ordinary_requests_preserve_the_entire_chain() {
        let mut routes = vec![
            route("primary", AdapterKind::Anthropic),
            route("responses", AdapterKind::Responses),
            route("gemini", AdapterKind::Gemini),
            route("cursor", AdapterKind::Cursor),
            route("antigravity-cli", AdapterKind::AntigravityCli),
        ];

        filter_fallbacks(
            &mut routes,
            &json!({"messages":[{"role":"user","content":"hello"}]}),
            "alias",
        );

        assert_eq!(routes.len(), 5);
    }
}
