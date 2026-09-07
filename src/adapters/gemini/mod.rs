//! Gemini adapter implementation for Google Code Assist / Gemini endpoints.

mod sse;

use axum::{
    body::{Body, Bytes},
    http::{HeaderMap, HeaderValue, Response, StatusCode, Uri},
    response::IntoResponse,
};
use futures_util::StreamExt;
use serde_json::Value;

use crate::{
    adapters::{Adapter, AdapterError, AdapterFuture},
    auth::{antigravity::auth::inference_base_url, Credential},
    config::AuthMode,
    model::antigravity_request::{
        antigravity_exact_catalog_admission, antigravity_request_id, antigravity_scoped_session_id,
        wrap_antigravity_envelope, AntigravityCatalog, AntigravityToolContext,
    },
    model::gemini::{map_gemini_error, GeminiSseMachine},
    model::gemini_request::wrap_code_assist_envelope,
    request::RequestBody,
    routing::Route,
    server::AppState,
};

use self::sse::{Decoder as GeminiSseDecoder, Item as GeminiSseItem};

const MAX_GEMINI_UNARY_RESPONSE_BYTES: usize = 32 * 1024 * 1024;

pub struct GeminiAdapter;

impl Adapter for GeminiAdapter {
    fn forward<'a>(
        &'a self,
        state: AppState,
        route: Route,
        uri: &'a Uri,
        headers: &'a HeaderMap,
        body: RequestBody,
    ) -> AdapterFuture<'a> {
        Box::pin(async move { forward(state, route, uri, headers, body).await })
    }
}

/// The Antigravity inference URL for a provider configured with `base_url`.
///
/// Split out so the production redirect is unit-testable without a live
/// request: `inference_base_url` carries the "production does not serve
/// Antigravity inference" rule that onboarding already applies.
fn antigravity_endpoint(base_url: &str, method: &str) -> String {
    let base_url = inference_base_url(base_url);
    format!("{base_url}/v1internal:{method}")
}

fn gemini_method(auth: AuthMode, streaming: bool) -> &'static str {
    if streaming || auth == AuthMode::AntigravityOauth {
        "streamGenerateContent?alt=sse"
    } else {
        "generateContent"
    }
}

/// Carry the effort tier a `-tiered` catalog id does not name into the request
/// body, where the backend reads it.
///
/// Additive on purpose: `thinkingBudget` is already translated from the
/// client's `thinking` block and the backend accepts both fields together, so
/// the level joins the existing `thinkingConfig` rather than replacing it.
///
/// The one case it must not join is a budget of `0`, which is how
/// [`crate::model::gemini_request`] renders an explicitly *disabled* `thinking`
/// block. Writing a level there would send "do not think" and "think at
/// medium" in one object — and if the backend gives the level precedence, a
/// client that opted out of reasoning is billed for it anyway. A configured
/// `effort` is dropped here too, while a suffixed id (`…-flash-high`) would
/// still carry it in the id: an accepted asymmetry, because the client's
/// explicit opt-out is the stronger signal in the one place the backend can
/// honour it.
fn set_thinking_level(inner_req: &mut Value, level: &str) {
    if inner_req
        .pointer("/generationConfig/thinkingConfig/thinkingBudget")
        .and_then(Value::as_u64)
        == Some(0)
    {
        return;
    }
    let Some(object) = inner_req.as_object_mut() else {
        return;
    };
    let generation_config = object
        .entry("generationConfig")
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    let Some(generation_config) = generation_config.as_object_mut() else {
        return;
    };
    let thinking_config = generation_config
        .entry("thinkingConfig")
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    if let Some(thinking_config) = thinking_config.as_object_mut() {
        thinking_config.insert(
            "thinkingLevel".to_string(),
            Value::String(level.to_string()),
        );
    }
}

fn append_sse_events(events: Vec<crate::model::gemini::SseEvent>, output: &mut Vec<u8>) {
    for event in events {
        let formatted = format!("event: {}\ndata: {}\n\n", event.event, event.data);
        output.extend_from_slice(formatted.as_bytes());
    }
}

fn append_protocol_error(message: impl Into<String>, output: &mut Vec<u8>) {
    append_sse_events(
        vec![crate::model::gemini::SseEvent {
            event: "error".to_string(),
            data: serde_json::json!({
                "type": "error",
                "error": {"type": "api_error", "message": message.into()}
            }),
        }],
        output,
    );
}

fn local_gemini_error(message: impl Into<String>) -> AdapterError {
    let message = message.into();
    let body = serde_json::json!({
        "type": "error",
        "error": {"type": "api_error", "message": message}
    });
    AdapterError {
        message,
        response: Box::new((StatusCode::BAD_GATEWAY, axum::Json(body)).into_response()),
        failure: None,
    }
}

fn embedded_gemini_error(data: Value) -> AdapterError {
    let error_type = data
        .pointer("/error/type")
        .and_then(Value::as_str)
        .unwrap_or("api_error");
    let status = if error_type == "rate_limit_error" {
        StatusCode::TOO_MANY_REQUESTS
    } else {
        StatusCode::BAD_GATEWAY
    };
    let message = data
        .pointer("/error/message")
        .and_then(Value::as_str)
        .unwrap_or("Gemini backend error")
        .to_string();
    AdapterError {
        message,
        response: Box::new((status, axum::Json(data)).into_response()),
        failure: None,
    }
}

async fn collect_unary_response(
    response: reqwest::Response,
    max_bytes: usize,
) -> Result<Vec<u8>, AdapterError> {
    if response
        .content_length()
        .is_some_and(|length| length > max_bytes as u64)
    {
        return Err(local_gemini_error(format!(
            "Gemini response exceeded {max_bytes} bytes"
        )));
    }
    let mut stream = response.bytes_stream();
    let mut body = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| {
            local_gemini_error(format!("failed to read Gemini response body: {error}"))
        })?;
        let new_len = body
            .len()
            .checked_add(chunk.len())
            .ok_or_else(|| local_gemini_error("Gemini response size overflow"))?;
        if new_len > max_bytes {
            return Err(local_gemini_error(format!(
                "Gemini response exceeded {max_bytes} bytes"
            )));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

async fn collect_antigravity_sse(
    response: reqwest::Response,
    route_model: &str,
    upstream_model: &str,
    context: Option<AntigravityToolContext>,
) -> Result<GeminiSseMachine, AdapterError> {
    let mut bytes = response.bytes_stream();
    let mut decoder = GeminiSseDecoder::default();
    let mut machine = GeminiSseMachine::new_for_upstream(route_model, upstream_model);
    if let Some(context) = context {
        machine = machine.with_antigravity_context(context);
    }
    let mut pending = None::<Bytes>;

    loop {
        if let Some(chunk) = pending.take() {
            let (consumed, item) = decoder.push_one(&chunk).map_err(local_gemini_error)?;
            if consumed < chunk.len() {
                pending = Some(chunk.slice(consumed..));
            }
            let Some(item) = item else { continue };
            let events = match item {
                GeminiSseItem::Json(value) => machine
                    .process_chunk_checked(&value)
                    .map_err(|error| local_gemini_error(error.to_string()))?,
                GeminiSseItem::Done => machine
                    .transport_close_checked()
                    .map_err(|error| local_gemini_error(error.to_string()))?,
            };
            if let Some(error) = events.into_iter().find(|event| event.event == "error") {
                return Err(embedded_gemini_error(error.data));
            }
            continue;
        }

        match bytes.next().await {
            Some(Ok(chunk)) => pending = Some(chunk),
            Some(Err(error)) => {
                return Err(local_gemini_error(format!(
                    "failed to read Gemini response body: {error}"
                )))
            }
            None => {
                decoder.finish().map_err(local_gemini_error)?;
                if !machine.is_started() || machine.final_json_checked().is_err() {
                    machine
                        .transport_close_checked()
                        .map_err(|error| local_gemini_error(error.to_string()))?;
                }
                return Ok(machine);
            }
        }
    }
}

async fn forward(
    state: AppState,
    route: Route,
    _uri: &Uri,
    _headers: &HeaderMap,
    body: RequestBody,
) -> Result<(StatusCode, Response<Body>), AdapterError> {
    let provider = state
        .config
        .provider(&route.provider)
        .ok_or_else(|| AdapterError {
            message: format!("unknown provider {}", route.provider),
            response: Box::new(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
            failure: None,
        })?;

    let credential = state.resolve_route_credential(&route).await?;

    let (access_token, project_id, account_fingerprint) = match credential {
        Credential::GoogleOauth {
            access_token,
            project_id,
        } => (access_token, project_id, None),
        Credential::AntigravityOauth {
            access_token,
            project_id,
            account_fingerprint,
        } => (access_token, project_id, Some(account_fingerprint)),
        Credential::ApiKey { value, .. } => (value, String::new(), None),
        _ => {
            return Err(AdapterError {
                message: "unsupported credential for Gemini adapter".to_string(),
                response: Box::new(StatusCode::UNAUTHORIZED.into_response()),
                failure: None,
            });
        }
    };

    let json_body = body.json();
    let is_streaming = json_body.get("stream").and_then(Value::as_bool) == Some(true);

    let antigravity_context = if provider.auth == AuthMode::AntigravityOauth {
        Some(
            AntigravityToolContext::new(
                account_fingerprint.as_deref().unwrap_or("legacy"),
                antigravity_scoped_session_id(
                    account_fingerprint.as_deref().unwrap_or("legacy"),
                    json_body,
                ),
            )
            .with_history(json_body),
        )
    } else {
        None
    };
    let mut inner_req = crate::model::gemini_request::translate_request_for_model_with_context(
        json_body,
        &route.upstream_model,
        antigravity_context.as_ref(),
    )?;

    let base_url = provider.base_url.trim_end_matches('/');

    let method = gemini_method(provider.auth, is_streaming);
    // Both subscription paths speak the Code Assist protocol: the same
    // `v1internal` methods under the `{model,project,request}` envelope. Only
    // the credential and the client identity differ, so the envelope is gated
    // on "is a subscription token", not on which provider issued it.
    let is_code_assist = matches!(
        provider.auth,
        AuthMode::GoogleOauth | AuthMode::AntigravityOauth
    );
    let (endpoint, payload) = if provider.auth == AuthMode::AntigravityOauth {
        // Antigravity speaks the same `v1internal` methods, but the client
        // identifies itself as the agent and names a session, and its catalog
        // ids carry their effort tier. See `wrap_antigravity_envelope` and
        // `antigravity_upstream_model`.
        //
        // The host is resolved rather than taken verbatim: production does not
        // serve Antigravity inference, and configs written against pre-0.40.0
        // docs pin it. Code Assist (`AuthMode::GoogleOauth`) genuinely lives
        // on production, so the redirect is scoped to this branch.
        let inference_base = inference_base_url(base_url);
        // Resolving twice is a no-op — the daily host resolves to itself — and
        // keeps the endpoint's own rule readable next to its unit tests.
        let endpoint = antigravity_endpoint(&inference_base, method);
        if !endpoint.starts_with(base_url) {
            tracing::debug!(
                provider = %route.provider,
                configured = %base_url,
                "antigravity base_url is pinned at the production Code Assist host, which does \
                 not serve inference; sending this request to the daily host instead"
            );
        }
        // The account's own catalog decides between the `-<tier>` and
        // `-tiered` forms of the same model; both exist in the wild and which
        // one an account is served changes over time. Discovery is cached, and
        // native admission fails closed when fresh evidence is unavailable.
        // It is skipped outright for an id no catalog could reshape — only
        // Gemini ids carry a tier — so rewritten Claude/GPT inputs are rejected.
        let catalog = crate::auth::antigravity::catalog::catalog_ids_for_account(
            &state.http_client,
            &inference_base,
            &access_token,
            account_fingerprint.as_deref().unwrap_or("legacy"),
            &project_id,
        )
        .await;
        let snapshot = catalog.as_ref().ok_or_else(|| {
            local_gemini_error(
                "Antigravity model admission requires fresh account catalog evidence",
            )
        })?;
        let model = antigravity_exact_catalog_admission(
            &route.upstream_model,
            route.effort.as_deref(),
            json_body,
            AntigravityCatalog {
                ids: &snapshot.ids,
                fresh: snapshot.fresh,
            },
        )
        .map_err(local_gemini_error)?;
        if let Some(level) = model.thinking_level {
            set_thinking_level(&mut inner_req, level);
        }
        let session_id = antigravity_context
            .as_ref()
            .map(|c| c.session.as_str())
            .unwrap_or("-");
        let envelope = wrap_antigravity_envelope(
            &model.id,
            &project_id,
            inner_req,
            &antigravity_request_id(),
            session_id,
        );
        (endpoint, envelope)
    } else if is_code_assist {
        let endpoint = format!("{base_url}/v1internal:{method}");
        let envelope = wrap_code_assist_envelope(&route.upstream_model, &project_id, inner_req);
        (endpoint, envelope)
    } else {
        let model_slug = &route.upstream_model;
        let endpoint = format!("{base_url}/v1beta/models/{model_slug}:{method}");
        (endpoint, inner_req)
    };

    let policy = provider.retry.policy();
    let http_client = if provider.auth == AuthMode::AntigravityOauth {
        crate::auth::shared::antigravity_inference_client(&inference_base_url(base_url))
            .map_err(|error| local_gemini_error(error.to_string()))?
    } else {
        state.http_client.clone()
    };
    let payload_clone = payload.clone();
    let endpoint_clone = endpoint.clone();
    let is_google_oauth = is_code_assist;
    // Antigravity's backend is addressed as the Antigravity client; the Gemini
    // Code Assist path is not, so the fingerprint is per auth mode rather than
    // sent on every Code Assist request.
    let user_agent = (provider.auth == AuthMode::AntigravityOauth)
        .then(crate::auth::antigravity::version::user_agent);
    let token = access_token.clone();

    let ttfb_ms = state.config.server.timeouts.upstream_ttfb_ms;
    let retry_safety = retry_safety_for_auth(provider.auth);
    let response =
        crate::retry::send_with_retry_with_safety(policy, &route.provider, retry_safety, || {
            let client = http_client.clone();
            let payload = payload_clone.clone();
            let endpoint = endpoint_clone.clone();
            let token = token.clone();
            let user_agent = user_agent.clone();
            async move {
                let mut req = client
                    .post(&endpoint)
                    .header("Content-Type", "application/json");

                if let Some(user_agent) = user_agent {
                    req = req.header("User-Agent", user_agent);
                }

                if is_google_oauth {
                    req = req.bearer_auth(&token);
                } else {
                    req = req.header("x-goog-api-key", &token);
                }

                crate::upstream_timeout::wait(ttfb_ms, req.json(&payload).send()).await
            }
        })
        .await
        .map_err(|error| {
            error.into_adapter_error(|error| AdapterError {
                message: format!("network error calling Gemini backend: {error}"),
                response: Box::new(StatusCode::BAD_GATEWAY.into_response()),
                failure: None,
            })
        })?;

    let status = response.status();
    if !status.is_success() {
        let body = collect_unary_response(response, MAX_GEMINI_UNARY_RESPONSE_BYTES).await?;
        let body_text = std::str::from_utf8(&body)
            .map_err(|_| local_gemini_error("invalid UTF-8 in Gemini error response"))?;
        return Err(map_gemini_error(status, body_text));
    }

    if is_streaming {
        let byte_stream = response.bytes_stream();
        let mut machine =
            GeminiSseMachine::new_streaming_for_upstream(&route.model, &route.upstream_model);
        if let Some(context) = antigravity_context.clone() {
            machine = machine.with_antigravity_context(context);
        }
        let decoder = GeminiSseDecoder::default();

        let sse_stream = futures_util::stream::unfold(
            (
                byte_stream,
                decoder,
                machine,
                false,
                None::<Bytes>,
                None::<Bytes>,
            ),
            |(
                mut bytes,
                mut decoder,
                mut machine,
                finished,
                mut pending,
                mut deferred_terminal,
            )| async move {
                if finished {
                    return None;
                }
                loop {
                    if let Some(chunk) = pending.take() {
                        let (consumed, item) = match decoder.push_one(&chunk) {
                            Ok(step) => step,
                            Err(error) => {
                                let mut output = Vec::new();
                                append_protocol_error(error, &mut output);
                                return Some((
                                    Ok::<_, std::convert::Infallible>(Bytes::from(output)),
                                    (bytes, decoder, machine, true, None, None),
                                ));
                            }
                        };
                        if consumed < chunk.len() {
                            pending = Some(chunk.slice(consumed..));
                        }
                        let Some(item) = item else {
                            continue;
                        };

                        let is_done = item == GeminiSseItem::Done;
                        let result = match item {
                            GeminiSseItem::Json(value) => machine.process_chunk_checked(&value),
                            GeminiSseItem::Done => machine.transport_close_checked(),
                        };
                        let result_succeeded = result.is_ok();
                        let mut output = Vec::new();
                        let terminal = match result {
                            Ok(events) => {
                                let terminal = events.iter().any(|event| {
                                    event.event == "error" || event.event == "message_stop"
                                });
                                append_sse_events(events, &mut output);
                                terminal
                            }
                            Err(error) => {
                                append_protocol_error(error.to_string(), &mut output);
                                true
                            }
                        };
                        if is_done && !terminal {
                            unreachable!("[DONE] completion must be terminal");
                        }
                        if is_done && terminal && result_succeeded {
                            deferred_terminal = Some(Bytes::from(output));
                            continue;
                        }
                        if !output.is_empty() {
                            return Some((
                                Ok(Bytes::from(output)),
                                (
                                    bytes,
                                    decoder,
                                    machine,
                                    terminal,
                                    pending,
                                    deferred_terminal,
                                ),
                            ));
                        }
                        continue;
                    }

                    match bytes.next().await {
                        Some(Ok(chunk)) => {
                            pending = Some(chunk);
                        }
                        Some(Err(error)) => {
                            let mut output = Vec::new();
                            append_protocol_error(
                                format!("Gemini response stream failed: {error}"),
                                &mut output,
                            );
                            return Some((
                                Ok(Bytes::from(output)),
                                (bytes, decoder, machine, true, None, None),
                            ));
                        }
                        None => {
                            let mut output = Vec::new();
                            match decoder.finish() {
                                Err(error) => append_protocol_error(error, &mut output),
                                Ok(()) => {
                                    if let Some(terminal) = deferred_terminal {
                                        output.extend_from_slice(&terminal);
                                    } else {
                                        match machine.transport_close_checked() {
                                            Ok(events) => append_sse_events(events, &mut output),
                                            Err(error) => append_protocol_error(
                                                error.to_string(),
                                                &mut output,
                                            ),
                                        }
                                    }
                                }
                            }
                            return Some((
                                Ok(Bytes::from(output)),
                                (bytes, decoder, machine, true, None, None),
                            ));
                        }
                    }
                }
            },
        );

        let res_builder = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/event-stream; charset=utf-8")
            .header("Cache-Control", "no-cache");

        let response_res = res_builder
            .body(Body::from_stream(sse_stream))
            .map_err(|error| AdapterError {
                message: format!("failed to build response: {error}"),
                response: Box::new(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
                failure: None,
            })?;

        Ok((StatusCode::OK, response_res))
    } else if provider.auth == AuthMode::AntigravityOauth {
        let machine = collect_antigravity_sse(
            response,
            &route.model,
            &route.upstream_model,
            antigravity_context.clone(),
        )
        .await?;
        let final_json = machine
            .final_json_checked()
            .map_err(|error| local_gemini_error(error.to_string()))?;
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        let response_res = (StatusCode::OK, headers, axum::Json(final_json)).into_response();
        Ok((StatusCode::OK, response_res))
    } else {
        let full_body = collect_unary_response(response, MAX_GEMINI_UNARY_RESPONSE_BYTES).await?;
        let parsed = serde_json::from_slice::<Value>(&full_body).map_err(|error| {
            local_gemini_error(format!("invalid JSON from Gemini backend: {error}"))
        })?;
        let mut machine = GeminiSseMachine::new_for_upstream(&route.model, &route.upstream_model);
        let events = machine
            .process_chunk_checked(&parsed)
            .map_err(|error| local_gemini_error(error.to_string()))?;
        if let Some(error) = events.into_iter().find(|event| event.event == "error") {
            return Err(embedded_gemini_error(error.data));
        }
        machine
            .transport_close_checked()
            .map_err(|error| local_gemini_error(error.to_string()))?;
        let final_json = machine
            .final_json_checked()
            .map_err(|error| local_gemini_error(error.to_string()))?;

        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));

        let response_res = (StatusCode::OK, headers, axum::Json(final_json)).into_response();
        Ok((StatusCode::OK, response_res))
    }
}

fn retry_safety_for_auth(auth: AuthMode) -> crate::retry::RetrySafety {
    if auth == AuthMode::AntigravityOauth {
        crate::retry::RetrySafety::Idempotent
    } else {
        crate::retry::RetrySafety::NonIdempotentPost
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn antigravity_retry_safety_remains_idempotent_until_phase_11() {
        assert_eq!(
            retry_safety_for_auth(AuthMode::AntigravityOauth),
            crate::retry::RetrySafety::Idempotent
        );
        assert_eq!(
            retry_safety_for_auth(AuthMode::ApiKey),
            crate::retry::RetrySafety::NonIdempotentPost
        );
    }

    #[test]
    fn a_production_pinned_antigravity_base_url_sends_inference_to_the_daily_host() {
        // Pre-0.40.0 docs told operators to write the production host, which
        // answers every Antigravity inference request with a fake 429. Such a
        // config still loads, so the request path is what has to redirect it.
        assert_eq!(
            antigravity_endpoint("https://cloudcode-pa.googleapis.com", "generateContent"),
            "https://daily-cloudcode-pa.googleapis.com/v1internal:generateContent"
        );
    }

    #[test]
    fn an_antigravity_base_url_in_front_of_the_backend_is_left_alone() {
        // A loopback proxy stands in front of the backend: redirecting past it
        // would egress straight around the endpoint the operator configured.
        assert_eq!(
            antigravity_endpoint("http://127.0.0.1:9999", "streamGenerateContent?alt=sse"),
            "http://127.0.0.1:9999/v1internal:streamGenerateContent?alt=sse"
        );
    }

    #[test]
    fn antigravity_native_sse_uses_always_sse_for_both_downstream_modes() {
        assert_eq!(
            gemini_method(AuthMode::AntigravityOauth, true),
            "streamGenerateContent?alt=sse"
        );
        assert_eq!(
            gemini_method(AuthMode::AntigravityOauth, false),
            "streamGenerateContent?alt=sse"
        );
        assert_eq!(gemini_method(AuthMode::ApiKey, false), "generateContent");
    }

    #[test]
    fn a_thinking_level_joins_the_translated_thinking_budget_rather_than_replacing_it() {
        // The `-tiered` catalog id names no effort, so the tier rides in
        // `thinkingLevel` — beside the `thinkingBudget` the client's own
        // `thinking` block already translated to. The backend accepts both.
        let mut inner_req = serde_json::json!({
            "contents": [],
            "generationConfig": {"thinkingConfig": {"thinkingBudget": 8192}}
        });

        set_thinking_level(&mut inner_req, "medium");

        assert_eq!(
            inner_req["generationConfig"]["thinkingConfig"]["thinkingLevel"],
            "medium"
        );
        assert_eq!(
            inner_req["generationConfig"]["thinkingConfig"]["thinkingBudget"],
            8192
        );
    }

    #[test]
    fn an_explicitly_disabled_thinking_block_is_not_overridden_by_the_default_tier() {
        // `thinking: {"type": "disabled"}` translates to `thinkingBudget: 0`,
        // and the tier the resolver derived for it is the `medium` default
        // nobody asked for. Writing the level here would send "do not think"
        // and "think at medium" in one object, and bill reasoning to a client
        // that opted out.
        let mut inner_req = serde_json::json!({
            "contents": [],
            "generationConfig": {"thinkingConfig": {"thinkingBudget": 0}}
        });

        set_thinking_level(&mut inner_req, "medium");

        assert!(
            inner_req["generationConfig"]["thinkingConfig"]
                .get("thinkingLevel")
                .is_none(),
            "{inner_req}"
        );
        assert_eq!(
            inner_req["generationConfig"]["thinkingConfig"]["thinkingBudget"],
            0
        );
    }

    #[test]
    fn a_thinking_level_creates_the_config_objects_it_needs() {
        // A non-thinking request carries no `generationConfig` at all, so the
        // level would be dropped if the path were only ever patched in.
        let mut inner_req = serde_json::json!({"contents": []});

        set_thinking_level(&mut inner_req, "high");

        assert_eq!(
            inner_req["generationConfig"]["thinkingConfig"]["thinkingLevel"],
            "high"
        );
        assert!(inner_req.get("contents").is_some());
    }

    #[test]
    fn complete_utf8_line_survives_arbitrary_byte_chunking() {
        let line = "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"Olá 🌊\"}]}}]}\n\n";
        let split = line.find('🌊').unwrap() + 1;
        let mut decoder = GeminiSseDecoder::default();

        assert_eq!(
            decoder.push_one(&line.as_bytes()[..split]).unwrap(),
            (split, None)
        );
        let (_, Some(GeminiSseItem::Json(value))) =
            decoder.push_one(&line.as_bytes()[split..]).unwrap()
        else {
            panic!("expected JSON event")
        };
        assert_eq!(
            value.pointer("/candidates/0/content/parts/0/text"),
            Some(&Value::String("Olá 🌊".to_string()))
        );
    }

    #[test]
    fn unterminated_final_data_line_is_rejected() {
        let mut decoder = GeminiSseDecoder::default();

        assert!(decoder
            .push_one(br#"data: {"candidates":[{"finishReason":"STOP"}]}"#)
            .unwrap()
            .1
            .is_none());
        assert!(decoder.finish().unwrap_err().contains("unterminated"));
    }
}
