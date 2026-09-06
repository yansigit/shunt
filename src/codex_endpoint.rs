//! Inbound OpenAI Responses (Codex) endpoint (`[server.codex_endpoint]`).
//!
//! Lets the OpenAI Codex CLI point its `chatgpt_base_url` (or a custom
//! `model_provider`) at shunt and be load-balanced across a ChatGPT/Codex OAuth
//! account pool. Unlike the Anthropic Messages path (`/v1/messages`), this is a
//! **raw passthrough**: the inbound Responses body is forwarded upstream
//! unchanged and the upstream response is relayed verbatim — only the M10
//! account-pool machinery (selection, failover, refresh) is reused. See
//! `docs/m11-inbound-codex-endpoint.md`.

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

/// Minimal view of the inbound Responses body: the `model` is read only for
/// metrics/logging labels — the body itself forwards upstream byte-for-byte, so
/// a missing or malformed model never blocks the request (the upstream rejects it).
/// `model` is deserialized as a [`ModelField`] rather than `Option<String>` so
/// [`parse_model`] can tell "field absent" apart from "field present but not a
/// string" instead of both silently becoming `None`.
#[derive(Debug, Deserialize)]
struct ModelView {
    model: Option<ModelField>,
}

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
    let (label, model) = if operation == responses::inbound::InboundOperation::Compact {
        let model = compact_model(&headers, &body, max_request_bytes)
            .await
            .map_err(|message| ForwardError {
                message: message.clone(),
                response: Box::new(
                    ShuntError::new(StatusCode::BAD_REQUEST, "invalid_request_error", message)
                        .into_response(),
                ),
            })?;
        (model.clone(), Some(model))
    } else {
        let label = model_label(&headers, &body, max_request_bytes).await;
        let model = (label != UNKNOWN_MODEL).then_some(label.clone());
        (label, model)
    };
    crate::observability::record_requested_model(&label);
    let pool_key = pool_sticky_key(inbound_client.as_deref(), session_id);

    forward_turn(state, model, pool_key, headers, body, started_at, operation).await
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
    mut headers: HeaderMap,
    body: Bytes,
    started_at: Instant,
    operation: responses::inbound::InboundOperation,
) -> Result<(StatusCode, axum::response::Response), ForwardError> {
    let decision = crate::routing::resolve_native_inbound(&state.config, model.as_deref());
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
            responses::forward_codex_inbound(state, route, operation, pool_key, headers, body).await
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

/// The label used when the request's model cannot be read (see [`model_label`]).
const UNKNOWN_MODEL: &str = "unknown";

/// Read the `model` for metrics/logging labels only — the body itself forwards
/// upstream byte-for-byte, so a body this cannot read never blocks the request
/// (the upstream rejects it).
///
/// Current Codex releases zstd-compress the Responses request body whenever both
/// of their gates pass, which includes the documented `chatgpt_base_url` client
/// shape pointed at this endpoint (issue #285). The compressed bytes relay
/// upstream fine — `content-encoding` is forwarded verbatim — but a plain
/// `from_slice` on them fails, which would silently label every metric, log line,
/// and span for the request `unknown`. So decode a zstd body for the label, and
/// log (rather than swallow) anything that still leaves the model unreadable.
///
/// [`MAX_REQUEST_BODY_BYTES`] is passed as [`decode_zstd_and_parse`]'s `cap`, the
/// same absolute limit this endpoint already applies to the arrival buffer — so
/// the arrival buffer and the decoded copy can be transiently resident together,
/// at worst two buffers each up to that cap (not one, as compressing surely
/// shrinks the wire size). What actually bounds the *decode work itself* for a
/// small, hostile body is `compression::MAX_DECODE_RATIO`, not this cap: it ties
/// worst-case decoded size to a multiple of what the peer actually uploaded
/// (issue #291). A small absolute cap here instead would be unsound for the
/// opposite reason — `serde_json::from_slice` needs a *complete* document, so any
/// truncation-style cap below a real turn's size would silently relabel every
/// large legitimate turn `unknown`, regressing issue #285's fix. The ratio bound
/// is what makes keeping the large absolute cap here safe.
///
/// The zstd branch fuses the decode with the `model` extraction inside one
/// bounded blocking task via [`decode_zstd_and_parse`], rather than decoding to
/// a [`Bytes`] here and parsing it afterward on the async executor: the decoded
/// body can be as large as [`MAX_REQUEST_BODY_BYTES`] (a ~1 MiB compressed
/// upload already buys a 64 MiB budget via the ratio bound), and a
/// `serde_json::from_slice` over a document that size is itself worker-blocking
/// work — a 400 KiB document alone is already milliseconds, far past Tokio's
/// ~100 µs budget. Doing both inside the same blocking task means the admission
/// permit covers the parse too, and only the extracted [`ParsedModel`] (never
/// the decoded bytes) crosses back to the async side (issue #291 follow-up).
/// The identity/`Other` branches below have the same worker-blocking parse
/// property but predate this fix — see the comment at their call site for why
/// they are deliberately left as-is.
async fn model_label(headers: &HeaderMap, body: &Bytes, max_request_bytes: usize) -> String {
    match crate::compression::body_encoding(headers) {
        BodyEncoding::Zstd => {
            match crate::compression::decode_zstd_and_parse(
                body.clone(),
                max_request_bytes,
                |decoded| {
                    let decoded_bytes = decoded.len();
                    (parse_model(&decoded), decoded_bytes)
                },
            )
            .await
            {
                Ok(Some((parsed, decoded_bytes))) => {
                    label_from_parsed(parsed, decoded_bytes, body.len())
                }
                Ok(None) => {
                    tracing::warn!(
                        wire_bytes = body.len(),
                        limit = max_request_bytes,
                        "inbound codex body decodes past the request size limit or the \
                         compressed-to-decoded ratio bound; model label unavailable"
                    );
                    UNKNOWN_MODEL.to_string()
                }
                Err(error) => {
                    // `error` here is a libzstd-authored message (allocation/format
                    // failure), not client-controlled content — unlike the parse
                    // error handled in `label_from_parsed`, so logging it verbatim
                    // does not risk echoing the request body.
                    tracing::warn!(
                        wire_bytes = body.len(),
                        error = %error,
                        "failed to decode zstd inbound codex body; model label unavailable"
                    );
                    UNKNOWN_MODEL.to_string()
                }
            }
        }
        // A coding shunt does not decode (anything other than zstd/identity) is
        // not fatal to the label: fall through and attempt a best-effort plain
        // parse below, same as `Identity`. Returning `unknown` unconditionally
        // here would let a client suppress its own model label by sending a
        // bogus `content-encoding` header on an otherwise-plain body.
        BodyEncoding::Other => {
            tracing::warn!(
                content_encoding = ?headers.get(axum::http::header::CONTENT_ENCODING),
                "inbound codex body uses an unsupported content-encoding; \
                 attempting a best-effort plain-JSON parse for the model label"
            );
            // Pre-existing (predates issue #291's fix, which only fuses the new
            // zstd decode with its parse — see the doc comment above): this parse
            // still runs synchronously on the async executor. Left as-is
            // deliberately so that asymmetry with the zstd branch above is legible
            // rather than accidental.
            label_from_parsed(parse_model(body), body.len(), body.len())
        }
        BodyEncoding::Identity => {
            // Pre-existing (predates issue #291's fix, which only fuses the new
            // zstd decode with its parse — see the doc comment above): this parse
            // runs synchronously on the async executor rather than the blocking
            // pool. Left as-is deliberately, out of scope for the zstd-only fix.
            label_from_parsed(parse_model(body), body.len(), body.len())
        }
    }
}

/// Turn a [`ParsedModel`] into the label string, logging *why* the label is
/// `unknown` when it is. Shared by every [`model_label`] branch so the log
/// shape is identical regardless of which path produced the [`ParsedModel`].
///
/// The `Malformed` arm deliberately logs only the error's classification
/// (`line`/`column`/`classify()`), never `error.to_string()` /
/// `error = %error`: `serde_json::Error`'s `Display` embeds the offending
/// value it choked on (e.g. `invalid type: string "<entire body>", expected
/// struct ModelView`), so logging it verbatim would echo the client-controlled
/// request body — up to `MAX_REQUEST_BODY_BYTES` of it — into `warn!`, which
/// becomes a Sentry breadcrumb (`observability`) and is exported by the OTel
/// logs bridge (`telemetry`). Do not "helpfully" restore `error = %error` here.
fn label_from_parsed(parsed: ParsedModel, decoded_bytes: usize, wire_bytes: usize) -> String {
    match parsed {
        ParsedModel::Model(model) => model,
        ParsedModel::Malformed(error) => {
            tracing::warn!(
                decoded_bytes,
                wire_bytes,
                error_line = error.line(),
                error_column = error.column(),
                error_kind = ?error.classify(),
                "inbound codex body is not valid JSON; labeling metrics and logs `unknown`"
            );
            UNKNOWN_MODEL.to_string()
        }
        ParsedModel::Missing => {
            tracing::warn!(
                decoded_bytes,
                wire_bytes,
                "inbound codex body has no `model` field; labeling metrics and logs `unknown`"
            );
            UNKNOWN_MODEL.to_string()
        }
        ParsedModel::NotAString(model_type) => {
            tracing::warn!(
                decoded_bytes,
                wire_bytes,
                model_type,
                "inbound codex body's `model` field is not a string; labeling metrics and logs `unknown`"
            );
            UNKNOWN_MODEL.to_string()
        }
    }
}

/// The distinguishable outcomes of reading `model` out of a decoded body, so
/// [`model_label`] can log *why* the label is unavailable instead of folding
/// malformed JSON, a missing field, and a wrong-typed field into one silent
/// `None` (as a bare `.ok().and_then(..)` chain over `Option<String>` would).
enum ParsedModel {
    Model(String),
    /// The body is not valid JSON at all.
    Malformed(serde_json::Error),
    /// Valid JSON with no `model` field (or an explicit `null`).
    Missing,
    /// Valid JSON with a `model` field that is not a string. Carries only the
    /// JSON type name, never the client-controlled value — see [`ModelField`].
    NotAString(&'static str),
}

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
        _ => Err("compaction request requires a non-empty string model".to_string()),
    }
}

fn parse_model(body: &[u8]) -> ParsedModel {
    match serde_json::from_slice::<ModelView>(body) {
        Ok(ModelView {
            model: Some(ModelField::Str(model)),
        }) => ParsedModel::Model(model),
        // `Option`'s deserializer maps an explicit `null` to `None` before
        // `ModelFieldVisitor` runs, so absent and `null` arrive here alike.
        Ok(ModelView { model: None }) => ParsedModel::Missing,
        Ok(ModelView {
            model: Some(ModelField::Other(model_type)),
        }) => ParsedModel::NotAString(model_type),
        Err(error) => ParsedModel::Malformed(error),
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
