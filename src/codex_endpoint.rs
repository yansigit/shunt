//! Inbound OpenAI Responses (Codex) endpoint (`[server.codex_endpoint]`).
//!
//! Lets the OpenAI Codex CLI point its `chatgpt_base_url` (or a custom
//! `model_provider`) at shunt and be load-balanced across a ChatGPT/Codex OAuth
//! account pool. Explicit endpoint-local native routes take precedence over
//! legacy exact global mappings, then the pinned pool is the fallback. Native
//! replies relay verbatim; explicit routes may rewrite the request model and
//! use a third-party header allowlist. Legacy exact Anthropic mappings retain
//! their strict translation/collaboration bridge. See `docs/m11-inbound-codex-endpoint.md`.

use std::time::Instant;

use axum::{
    body::{Body, Bytes},
    extract::{OriginalUri, State},
    http::{HeaderMap, Method, StatusCode},
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::Instrument;

use crate::{
    adapters::{anthropic::AnthropicAdapter, responses, Adapter, AdapterError},
    compression::BodyEncoding,
    error::ShuntError,
    server::AppState,
};

pub mod frame;
mod model;
mod routing;
pub mod websocket;

/// Inbound Responses routes this handler serves, registered by
/// [`crate::server::build_router`] when `[server.codex_endpoint]` is set.
///
/// This is the single source of truth for the path set: the router registers
/// exactly these, and `concurrency::is_codex_path` classifies against them so a
/// gateway-owned error on any of them uses the OpenAI Responses envelope rather
/// than the Anthropic one (AGENTS.md). Adding a path here registers it and gives
/// it the right error shape together — they cannot drift apart.
pub(crate) const PATHS: [&str; 3] = [
    "/backend-api/codex/responses",
    "/responses",
    "/v1/responses",
];

/// Remote compaction is HTTP-only and therefore registered separately from
/// [`PATHS`], whose entries also accept inbound WebSocket upgrades.
pub(crate) const COMPACT_PATH: &str = "/v1/responses/compact";

/// What the inbound body's `model` field turned out to be, classified *without*
/// materializing it.
///
/// Deliberately not `serde_json::Value`: only a string is ever used, and every
/// other shape is used solely to name the type in a log line. Deserializing into
/// a `Value` would make serde allocate and retain the field's entire contents
/// first — so a client sending `"model": [ ...megabytes... ]` would turn this
/// best-effort labels-only parse into a large client-controlled heap allocation,
/// on top of the arrival buffer and (on the zstd path) the decoded copy that are
/// already resident (issue #291 follow-up). The non-string arms below drain their
/// contents through [`IgnoredAny`], which walks the input without building it.
#[derive(Debug)]
enum ModelField {
    Str(String),
    /// A JSON type name (`"array"`, `"object"`, ...) — never any client content.
    Other(&'static str),
}

impl<'de> Deserialize<'de> for ModelField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ModelFieldVisitor)
    }
}

struct ModelFieldVisitor;

impl<'de> serde::de::Visitor<'de> for ModelFieldVisitor {
    type Value = ModelField;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(ModelField::Str(value.to_string()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(ModelField::Str(value))
    }

    fn visit_bool<E>(self, _: bool) -> Result<Self::Value, E> {
        Ok(ModelField::Other("boolean"))
    }

    fn visit_i64<E>(self, _: i64) -> Result<Self::Value, E> {
        Ok(ModelField::Other("number"))
    }

    fn visit_u64<E>(self, _: u64) -> Result<Self::Value, E> {
        Ok(ModelField::Other("number"))
    }

    fn visit_i128<E>(self, _: i128) -> Result<Self::Value, E> {
        Ok(ModelField::Other("number"))
    }

    fn visit_u128<E>(self, _: u128) -> Result<Self::Value, E> {
        Ok(ModelField::Other("number"))
    }

    fn visit_f64<E>(self, _: f64) -> Result<Self::Value, E> {
        Ok(ModelField::Other("number"))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(ModelField::Other("null"))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        // Drain rather than collect: the elements are never read, and building
        // them is the allocation this type exists to avoid.
        while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
        Ok(ModelField::Other("array"))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        while map
            .next_entry::<serde::de::IgnoredAny, serde::de::IgnoredAny>()?
            .is_some()
        {}
        Ok(ModelField::Other("object"))
    }
}

/// Handler for the inbound Responses routes (`/backend-api/codex/responses`,
/// `/responses`, `/v1/responses`). Mirrors `proxy::post`'s shape: snapshot the
/// live state, trace the request, and relay a gateway-owned error as a response.
pub async fn post(
    State(state): State<AppState>,
    method: Method,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: Body,
) -> axum::response::Response {
    post_operation(
        state,
        method,
        uri,
        headers,
        body,
        responses::inbound::InboundOperation::Responses,
    )
    .await
}

/// HTTP-only native Responses compaction entry point.
pub async fn compact(
    State(state): State<AppState>,
    method: Method,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: Body,
) -> axum::response::Response {
    post_operation(
        state,
        method,
        uri,
        headers,
        body,
        responses::inbound::InboundOperation::Compact,
    )
    .await
}

async fn post_operation(
    state: AppState,
    method: Method,
    uri: axum::http::Uri,
    headers: HeaderMap,
    body: Body,
    operation: responses::inbound::InboundOperation,
) -> axum::response::Response {
    let state = state.refreshed();
    let started_at = Instant::now();
    let path = uri.path().to_string();
    // The Codex CLI keys a conversation with a `session-id` header; fall back to
    // Claude Code's header for parity. Used both for the tracing span and as the
    // account-pool sticky key so one conversation stays on one account.
    let session_id = extract_session_id(&headers);
    // Withhold the request-derived id from exported spans unless the operator
    // opted in per backend (same rule as `proxy::post`).
    let span_session_id = if crate::telemetry::withhold_session_id() {
        ""
    } else {
        session_id.as_deref().unwrap_or("")
    };
    // See `proxy::post`'s equivalent span for why these start empty: the model
    // and outcome are only known inside `forward`, once the body is parsed and
    // the upstream has responded (`crate::observability`, #281).
    let span = tracing::info_span!(
        "codex_endpoint_request",
        method = %method,
        path = %path,
        session_id = span_session_id,
        gen_ai.request.model = tracing::field::Empty,
        shunt.provider = tracing::field::Empty,
        http.response.status_code = tracing::field::Empty,
        otel.status_code = tracing::field::Empty
    );

    async move {
        match forward(state, session_id, headers, body, started_at, operation).await {
            Ok((status, response)) => {
                tracing::info!(
                    upstream_status = status.as_u16(),
                    latency_ms = started_at.elapsed().as_millis(),
                    "proxied inbound codex request"
                );
                response
            }
            Err(error) => {
                // Log *why* the request failed before returning the client-facing
                // response — without this a shunt-owned failure (bad credential,
                // unreachable backend, exhausted pool) leaves no server-side signal
                // an operator could grep. Mirrors `proxy::post`.
                tracing::warn!(
                    latency_ms = started_at.elapsed().as_millis(),
                    error = %error.message,
                    "inbound codex request failed"
                );
                // Gateway-owned errors on this endpoint are built with the gateway's
                // Anthropic-shaped responders (`ShuntError` / `UpstreamError` /
                // adapter+auth `AdapterError`s). A Codex CLI (or any OpenAI Responses
                // client) pointed here expects the OpenAI `{"error":{...}}` envelope,
                // so re-shape at this single boundary (status preserved). Relayed
                // upstream errors never reach here — they return verbatim as `Ok`.
                crate::error::into_openai_error_shape(*error.response).await
            }
        }
    }
    .instrument(span)
    .await
}

/// A gateway-owned error from [`forward`] carrying a log message alongside the
/// client-facing response, so [`post`] can record *why* the request failed
/// (mirrors `proxy::ForwardError`). An upstream error response relayed verbatim is
/// an `Ok`, not this — only shunt-owned failures (config, auth, body read, account
/// resolution/transport) surface here.
pub(crate) struct ForwardError {
    pub(crate) message: String,
    /// Boxed to keep `Result<_, ForwardError>` small: an `axum` `Response` alone
    /// is 128 bytes, which trips `clippy::result_large_err` on [`forward`].
    pub(crate) response: Box<axum::response::Response>,
}

impl From<AdapterError> for ForwardError {
    fn from(error: AdapterError) -> Self {
        Self {
            message: error.message,
            response: error.response,
        }
    }
}

async fn forward(
    state: AppState,
    session_id: Option<String>,
    headers: HeaderMap,
    body: Body,
    started_at: Instant,
    operation: responses::inbound::InboundOperation,
) -> Result<(StatusCode, axum::response::Response), ForwardError> {
    // The routes are only registered when `[server.codex_endpoint]` is set, but
    // read the snapshot defensively; config validation guarantees the named
    // provider exists and uses `chatgpt_oauth`.
    let Some(codex_endpoint) = &state.config.server.codex_endpoint else {
        return Err(ForwardError {
            message: "codex endpoint is not configured".to_string(),
            response: Box::new(
                ShuntError::bad_gateway("codex endpoint is not configured".to_string())
                    .into_response(),
            ),
        });
    };
    let pinned_provider = codex_endpoint.provider.clone();

    let inbound_client =
        authenticate_inbound(state.inbound_auth.as_deref(), &headers, &pinned_provider).map_err(
            |err| ForwardError {
                message: "inbound authentication failed".to_string(),
                response: Box::new(err.into_response()),
            },
        )?;

    let max_request_bytes = state.config.server.limits.max_request_bytes;
    if crate::http_tuning::content_length_exceeds(&headers, max_request_bytes) {
        return Err(ForwardError {
            message: "request body exceeds the configured limit".to_string(),
            response: Box::new(crate::http_tuning::request_too_large(true).await),
        });
    }
    let body = crate::http_tuning::read_body(body, max_request_bytes, true)
        .await
        .map_err(|response| ForwardError {
            message: if response.status() == StatusCode::PAYLOAD_TOO_LARGE {
                "request body exceeds the configured limit"
            } else {
                "failed to read request body"
            }
            .to_string(),
            response,
        })?;

    // Ordinary Responses treats the model as a best-effort routing label for
    // compatibility. Compact must fail before network dispatch unless a valid,
    // unique, non-empty model can be routed to a verified native endpoint.
    let (label, model, decoded) = if operation == responses::inbound::InboundOperation::Compact {
        let model = compact_model(&headers, &body, max_request_bytes)
            .await
            .map_err(|message| ForwardError {
                message: message.clone(),
                response: Box::new(
                    ShuntError::new(StatusCode::BAD_REQUEST, "invalid_request_error", message)
                        .into_response(),
                ),
            })?;
        (model.clone(), Some(model), None)
    } else {
        // Keep an actual model named `unknown` distinct from an unreadable
        // model: only the latter must never select an endpoint-local route.
        let keep_decoded = !codex_endpoint.routes.is_empty();
        let resolved = model::resolve_model(&headers, &body, max_request_bytes, keep_decoded).await;
        let model = resolved.model;
        let label = model.clone().unwrap_or_else(|| UNKNOWN_MODEL.to_string());
        (label, model, resolved.decoded)
    };
    crate::observability::record_requested_model(&label);
    let pool_key = pool_sticky_key(inbound_client.as_deref(), session_id);

    forward_prepared_turn(
        state,
        PreparedTurn { model, decoded },
        pool_key,
        headers,
        body,
        started_at,
        operation,
    )
    .await
}

pub(crate) fn extract_session_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("session-id")
        .or_else(|| headers.get("x-claude-code-session-id"))
        .and_then(|value| value.to_str().ok())
        .filter(|session_id| !session_id.is_empty())
        .map(ToOwned::to_owned)
}

pub(crate) fn authenticate_inbound(
    auth: Option<&crate::auth::inbound::InboundAuth>,
    headers: &HeaderMap,
    provider: &str,
) -> Result<Option<String>, ShuntError> {
    if let Some(auth) = auth {
        match auth.authenticate_bearer(headers) {
            Some(client) => Ok(Some(client.to_string())),
            None => {
                tracing::warn!(
                    provider = %provider,
                    "inbound codex auth failed: missing or invalid client token"
                );
                let message = format!(
                    "missing or invalid client token for the inbound codex endpoint: provide it via the `{}` header or `Authorization: Bearer <token>` (e.g. OPENAI_API_KEY); ask the operator for one",
                    auth.header()
                );
                Err(ShuntError::new(
                    StatusCode::UNAUTHORIZED,
                    "authentication_error",
                    message,
                ))
            }
        }
    } else {
        Ok(None)
    }
}

pub(crate) async fn forward_turn(
    state: AppState,
    model: Option<String>,
    pool_key: Option<String>,
    headers: HeaderMap,
    body: Bytes,
    started_at: Instant,
    operation: responses::inbound::InboundOperation,
) -> Result<(StatusCode, axum::response::Response), ForwardError> {
    forward_prepared_turn(
        state,
        PreparedTurn {
            model,
            decoded: None,
        },
        pool_key,
        headers,
        body,
        started_at,
        operation,
    )
    .await
}

struct PreparedTurn {
    model: Option<String>,
    decoded: Option<Bytes>,
}

async fn forward_prepared_turn(
    state: AppState,
    prepared: PreparedTurn,
    pool_key: Option<String>,
    mut headers: HeaderMap,
    body: Bytes,
    started_at: Instant,
    operation: responses::inbound::InboundOperation,
) -> Result<(StatusCode, axum::response::Response), ForwardError> {
    let PreparedTurn { model, decoded } = prepared;
    // Endpoint-local routes are opt-in, byte-exact overrides. If none matches,
    // retain the fork's existing global-route and pinned-provider behavior.
    let explicit = state
        .config
        .server
        .codex_endpoint
        .as_ref()
        .and_then(|endpoint| model.as_deref().and_then(|model| endpoint.route_for(model)));
    let endpoint_routed = explicit.is_some();
    // No route can consume the reusable decode on the compatibility path.
    let decoded = if endpoint_routed {
        decoded
    } else {
        drop(decoded);
        None
    };
    let decision = match explicit {
        Some(configured) => {
            crate::routing::NativeInboundDecision::Selected(crate::routing::Route {
                provider: configured.provider.clone(),
                adapter: crate::routing::AdapterKind::Responses,
                model: model.clone().expect("matched route has a model"),
                upstream_model: configured.upstream_model().to_string(),
                effort: None,
                service_tier: None,
            })
        }
        None => crate::routing::resolve_native_inbound(&state.config, model.as_deref()),
    };
    let route = match decision {
        crate::routing::NativeInboundDecision::Pinned(route)
        | crate::routing::NativeInboundDecision::Selected(route) => route,
        crate::routing::NativeInboundDecision::Rejected(message) => {
            return Err(ForwardError {
                message: message.clone(),
                response: Box::new(
                    ShuntError::new(StatusCode::BAD_REQUEST, "invalid_request_error", message)
                        .into_response(),
                ),
            });
        }
    };
    let mut routes = vec![route.clone()];
    if let Err(message) =
        crate::proxy::capability::enforce_opencode_go_admission(&state.config, &mut routes)
    {
        return Err(ForwardError {
            message: message.to_string(),
            response: Box::new(
                ShuntError::new(StatusCode::BAD_REQUEST, "invalid_request_error", message)
                    .into_response(),
            ),
        });
    }
    if operation == responses::inbound::InboundOperation::Compact
        && !state
            .config
            .supports_native_responses_compact(&route.provider)
    {
        let message = format!(
            "native Responses compaction is not supported by provider `{}`",
            route.provider
        );
        return Err(ForwardError {
            message: message.clone(),
            response: Box::new(
                ShuntError::new(StatusCode::BAD_REQUEST, "invalid_request_error", message)
                    .into_response(),
            ),
        });
    }
    let provider = route.provider.clone();
    let model = route.model.clone();
    let collaboration_enabled = state
        .config
        .server
        .codex_endpoint
        .as_ref()
        .is_some_and(|endpoint| endpoint.collaboration);

    let result = match route.adapter {
        crate::routing::AdapterKind::Responses => {
            if endpoint_routed {
                forward_endpoint_route(state, route, operation, pool_key, headers, body, decoded)
                    .await
            } else {
                responses::forward_codex_inbound(state, route, operation, pool_key, headers, body)
                    .await
            }
        }
        crate::routing::AdapterKind::Anthropic
            if operation == responses::inbound::InboundOperation::Responses =>
        {
            let response_model = route.model.clone();
            let max_body_bytes = state.config.server.limits.max_request_bytes;
            let translation_body = match crate::compression::body_encoding(&headers) {
                BodyEncoding::Identity => body.to_vec(),
                BodyEncoding::Zstd => crate::compression::decode_zstd_and_parse(
                    body.clone(),
                    max_body_bytes,
                    |decoded| decoded.to_vec(),
                )
                .await
                .map_err(|error| {
                    translation_adapter_error(format!(
                        "failed to decode zstd Responses request: {error}"
                    ))
                })?
                .ok_or_else(|| {
                    translation_adapter_error(
                        "decoded Responses request exceeds the translation limit".into(),
                    )
                })?,
                BodyEncoding::Other => {
                    return Err(translation_adapter_error(
                        "unsupported content-encoding for Anthropic translation".into(),
                    )
                    .into())
                }
            };
            let translated = crate::model::inbound_responses::request::translate(
                translation_body,
                &route.upstream_model,
                collaboration_enabled,
            )
            .map_err(translation_adapter_error)?;
            headers.remove(axum::http::header::CONTENT_ENCODING);
            headers.remove(axum::http::header::CONTENT_LENGTH);
            let requested_stream = translated.stream;
            let (status, response) = AnthropicAdapter
                .forward(
                    state,
                    route,
                    &axum::http::Uri::from_static("/v1/messages"),
                    &headers,
                    translated.body,
                )
                .await?;
            let response = crate::adapters::anthropic::inbound::translate(
                status,
                response,
                requested_stream,
                &response_model,
                max_body_bytes,
                translated.collaboration,
            )
            .await;
            Ok((response.status(), response))
        }
        _ => unreachable!("inbound resolver returned an unsupported adapter"),
    };
    let status_code = match &result {
        Ok((status, _)) => *status,
        Err(error) => error.response.status(),
    };
    crate::observability::record_span_outcome(&provider, status_code);
    crate::observability::capture_upstream_outcome(&provider, &model, status_code);
    crate::metrics::record_proxied_request(
        &provider,
        &model,
        status_code.as_u16(),
        started_at.elapsed().as_secs_f64() * 1000.0,
    );
    result
        .map(|(status, response)| {
            let response = crate::stream_metrics::observe_response(
                response,
                crate::stream_metrics::Protocol::Responses,
                provider,
                model,
                started_at,
            );
            (status, response)
        })
        .map_err(ForwardError::from)
}

fn translation_adapter_error(message: String) -> AdapterError {
    AdapterError {
        message: message.clone(),
        response: Box::new(
            ShuntError::new(StatusCode::BAD_REQUEST, "invalid_request_error", message)
                .into_response(),
        ),
        failure: None,
    }
}

/// Keep the upstream's routed wire contract isolated from the legacy bridge.
async fn forward_endpoint_route(
    state: AppState,
    route: crate::routing::Route,
    operation: responses::inbound::InboundOperation,
    pool_key: Option<String>,
    mut headers: HeaderMap,
    body: Bytes,
    decoded: Option<Bytes>,
) -> Result<(StatusCode, axum::response::Response), AdapterError> {
    let pooled = state.config.is_chatgpt_backend(&route.provider);
    let rewrite = (route.upstream_model != route.model).then_some(route.upstream_model.as_str());
    if rewrite.is_some() {
        headers.remove("x-codex-routing-hint");
    }
    let body = if pooled && rewrite.is_none() {
        drop(decoded);
        body
    } else {
        let prepared = routing::identity_body(
            &headers,
            &body,
            decoded,
            rewrite,
            state.config.server.limits.max_request_bytes,
        )
        .await
        .map_err(|error| match error {
            routing::BodyError::TooLarge => AdapterError {
                message: "request body exceeds the configured limit".into(),
                response: Box::new(
                    ShuntError::new(
                        StatusCode::PAYLOAD_TOO_LARGE,
                        "invalid_request_error",
                        "request body exceeds the configured limit",
                    )
                    .into_response(),
                ),
                failure: None,
            },
            routing::BodyError::Invalid => translation_adapter_error(
                "routed request body could not be prepared as a JSON object".into(),
            ),
        })?;
        headers.remove(axum::http::header::CONTENT_ENCODING);
        headers.remove(axum::http::header::CONTENT_LENGTH);
        prepared
    };
    if pooled || operation == responses::inbound::InboundOperation::Compact {
        responses::forward_codex_inbound(state, route, operation, pool_key, headers, body).await
    } else {
        responses::forward_codex_routed(state, route, headers, body).await
    }
}

#[cfg(test)]
use model::model_label;
use model::UNKNOWN_MODEL;

#[derive(Debug)]
struct CompactModelView {
    model: Option<ModelField>,
}

impl<'de> Deserialize<'de> for CompactModelView {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = CompactModelView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a compaction request object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut model = None;
                let mut model_seen = false;
                while let Some(key) = map.next_key::<String>()? {
                    if key == "model" {
                        if model_seen {
                            return Err(serde::de::Error::custom("duplicate model field"));
                        }
                        model_seen = true;
                        model = Some(map.next_value::<ModelField>()?);
                    } else {
                        map.next_value::<serde::de::IgnoredAny>()?;
                    }
                }
                Ok(CompactModelView { model })
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}

async fn compact_model(
    headers: &HeaderMap,
    body: &Bytes,
    max_request_bytes: usize,
) -> Result<String, String> {
    let parsed = match crate::compression::body_encoding(headers) {
        BodyEncoding::Zstd => {
            crate::compression::decode_zstd_and_parse(body.clone(), max_request_bytes, |decoded| {
                parse_compact_model(&decoded)
            })
            .await
            .map_err(|_| "invalid compressed compaction request body".to_string())?
            .ok_or_else(|| "compaction request body exceeds the decoded limit".to_string())?
        }
        BodyEncoding::Identity => parse_compact_model(body),
        BodyEncoding::Other => {
            return Err("unsupported compaction request content-encoding".to_string())
        }
    };
    parsed
}

fn parse_compact_model(body: &[u8]) -> Result<String, String> {
    let view: CompactModelView =
        serde_json::from_slice(body).map_err(|_| "invalid compaction request body".to_string())?;
    match view.model {
        Some(ModelField::Str(model)) if !model.is_empty() => Ok(model),
        Some(ModelField::Other(kind)) => {
            tracing::debug!(model_type = kind, "compaction model is not a string");
            Err("compaction request requires a non-empty string model".to_string())
        }
        _ => Err("compaction request requires a non-empty string model".to_string()),
    }
}

/// Namespace the account-pool sticky key with the authenticated inbound client so
/// that, in a multi-tenant deployment, one client cannot pin another client's Codex
/// session onto a chosen pool account by replaying its `session-id` header. Mirrors
/// the outbound Responses path's `{client}:{session_id}` key (`adapters/responses/mod.rs`).
/// With no inbound auth (`client == None`) the bare session id is used — single-tenant,
/// there is no client identity to bind. Returns `None` when the request carries no
/// session id (nothing to key the pool on).
pub(crate) fn pool_sticky_key(client: Option<&str>, session_id: Option<String>) -> Option<String> {
    session_id.map(|session_id| match client {
        Some(client) => format!("{client}:{session_id}"),
        None => session_id,
    })
}

#[cfg(test)]
mod tests;
