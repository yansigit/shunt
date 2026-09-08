//! Command Code subscription vertical slice, distinct from the Chat API product.
//! Wire provenance: .planning/phases/14-command-code-product-separation/14-PROTOCOL-EVIDENCE.md.
mod ndjson;
#[cfg(test)]
mod router_tests;

use crate::{
    adapters::{Adapter, AdapterError, AdapterFailure, AdapterFuture},
    auth::{
        command_code::{validate_provider, ENDPOINT},
        Credential,
    },
    request::RequestBody,
    routing::Route,
    server::AppState,
};
use axum::{
    body::Body,
    http::{HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use futures_util::StreamExt;
use serde_json::{json, Value};

pub(crate) const COMMAND_CODE_RETRY_SAFETY: crate::retry::RetrySafety =
    crate::retry::RetrySafety::ConnectOnly;
pub struct CommandCodeAdapter;
impl Adapter for CommandCodeAdapter {
    fn forward<'a>(
        &'a self,
        state: AppState,
        route: Route,
        _uri: &'a Uri,
        _headers: &'a HeaderMap,
        body: RequestBody,
    ) -> AdapterFuture<'a> {
        Box::pin(async move { forward(state, route, body).await })
    }
}

fn error(status: StatusCode, kind: &str, message: &str) -> AdapterError {
    AdapterError {
        message: message.into(),
        failure: None,
        response: Box::new(
            (
                status,
                axum::Json(json!({"type":"error","error":{"type":kind,"message":message}})),
            )
                .into_response(),
        ),
    }
}
fn protocol(message: &str) -> AdapterError {
    error(StatusCode::BAD_GATEWAY, "api_error", message)
}
fn invalid(message: &str) -> AdapterError {
    error(StatusCode::BAD_REQUEST, "invalid_request_error", message)
}

#[cfg(not(test))]
fn client() -> &'static reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .read_timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("subscription client builds")
    })
}

fn payload(body: &Value, model: &str) -> Result<Value, AdapterError> {
    // Narrow text tracer only. The next request plan expands the explicit contract.
    if body.get("stream").is_some_and(|v| !v.is_boolean()) {
        return Err(invalid("stream must be a boolean"));
    }
    if body.get("stream").and_then(Value::as_bool) == Some(true) {
        return Err(invalid(
            "Command Code streaming client support is not implemented in this slice",
        ));
    }
    for key in body
        .as_object()
        .ok_or_else(|| invalid("request must be an object"))?
        .keys()
    {
        if !["model", "messages", "max_tokens", "stream", "system"].contains(&key.as_str()) {
            return Err(invalid("unsupported Command Code request field"));
        }
    }
    let messages = body
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("messages must be an array"))?;
    if messages.is_empty() {
        return Err(invalid("messages must not be empty"));
    }
    let mut wire = Vec::new();
    for message in messages {
        if message
            .as_object()
            .is_none_or(|m| m.keys().any(|k| !["role", "content"].contains(&k.as_str())))
        {
            return Err(invalid("unsupported message field"));
        }
        let role = message
            .get("role")
            .and_then(Value::as_str)
            .filter(|r| matches!(*r, "user" | "assistant"))
            .ok_or_else(|| invalid("unsupported message role"))?;
        let content = message
            .get("content")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("text-only Command Code tracer requires string content"))?;
        wire.push(json!({"role":role,"content":content}));
    }
    let max_tokens = match body.get("max_tokens") {
        Some(v) => v
            .as_u64()
            .filter(|n| *n > 0)
            .ok_or_else(|| invalid("max_tokens must be a positive integer"))?,
        None => 64000,
    };
    let system = match body.get("system") {
        Some(v) => v
            .as_str()
            .ok_or_else(|| invalid("system must be a string"))?,
        None => "",
    };
    Ok(
        json!({"config":{},"memory":"","taste":null,"skills":null,"permissionMode":"standard","mode":"agent",
        "params":{"model":model,"messages":wire,"tools":[],"system":system,"max_tokens":max_tokens,"stream":true}}),
    )
}

async fn forward(
    state: AppState,
    route: Route,
    body: RequestBody,
) -> Result<(StatusCode, Response<Body>), AdapterError> {
    let provider = state
        .config
        .provider(&route.provider)
        .ok_or_else(|| invalid("unknown subscription provider"))?;
    validate_provider(provider).map_err(invalid)?;
    let payload = payload(body.json(), &route.upstream_model)?;
    let credential = state.resolve_route_credential(&route).await?;
    let Credential::CommandCodeOauth { access_token } = credential else {
        return Err(crate::auth::auth_error(
            "subscription credential type mismatch",
        ));
    };
    validate_provider(provider).map_err(invalid)?;
    #[cfg(test)]
    let client = state.http_client.clone();
    #[cfg(not(test))]
    let client = client().clone();
    let session = uuid::Uuid::new_v4().to_string();
    let send_once = || {
        let request = client
            .post(ENDPOINT)
            .bearer_auth(&access_token)
            .header("user-agent", "cli")
            .header("x-command-code-version", "0.52.1")
            .header("x-cli-environment", "production")
            .header("x-taste-learning", "false")
            .header("x-co-flag", "false")
            .header("x-session-id", &session)
            .json(&payload);
        crate::upstream_timeout::wait(
            state.config.server.timeouts.upstream_ttfb_ms,
            request.send(),
        )
    };
    let response = crate::retry::send_with_retry_with_safety(
        provider.retry.policy(),
        &route.provider,
        COMMAND_CODE_RETRY_SAFETY,
        send_once,
    )
    .await
    .map_err(|e| match e {
        crate::upstream_timeout::SendError::Transport(ref transport) if transport.is_connect() => {
            let mut error = protocol("failed to connect to subscription backend");
            error.failure = Some(AdapterFailure::BeforeHeaders);
            error
        }
        other => other.into_adapter_error(|_| protocol("subscription transport failed")),
    })?;
    let status = response.status();
    if !status.is_success() {
        let mapped = if status.is_redirection() {
            StatusCode::BAD_GATEWAY
        } else {
            status
        };
        let kind = match mapped {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => "authentication_error",
            StatusCode::TOO_MANY_REQUESTS => "rate_limit_error",
            _ => "api_error",
        };
        return Err(error(
            mapped,
            kind,
            "Command Code subscription backend rejected the request",
        ));
    }
    let mut stream = response.bytes_stream();
    let mut decoder = ndjson::Decoder::default();
    let mut text = String::new();
    let mut terminal: Option<(String, String)> = None;
    let mut companion = false;
    let mut usage = json!({"input_tokens":0,"output_tokens":0});
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| protocol("subscription body transport failed"))?;
        for record in decoder.feed(&chunk).map_err(protocol)? {
            match record.get("type").and_then(Value::as_str) {
                Some("text-delta") if terminal.is_none() => {
                    let delta = record
                        .get("text")
                        .and_then(Value::as_str)
                        .ok_or_else(|| protocol("invalid text delta"))?;
                    text.push_str(delta);
                }
                Some(kind @ ("finish-step" | "finish")) => {
                    let reason = record
                        .get("rawFinishReason")
                        .or_else(|| record.get("finishReason"))
                        .and_then(Value::as_str)
                        .ok_or_else(|| protocol("missing finish reason"))?;
                    if reason != "stop" {
                        return Err(protocol("unsupported or failed subscription finish"));
                    }
                    if let Some((first, prior_reason)) = &terminal {
                        if first != "finish-step"
                            || kind != "finish"
                            || companion
                            || prior_reason != reason
                        {
                            return Err(protocol("conflicting or duplicate subscription terminal"));
                        }
                        companion = true;
                    } else {
                        if let Some(value) =
                            record.get("totalUsage").or_else(|| record.get("usage"))
                        {
                            for (wire, client) in [
                                ("inputTokens", "input_tokens"),
                                ("outputTokens", "output_tokens"),
                            ] {
                                if let Some(n) = value.get(wire) {
                                    usage[client] = json!(n
                                        .as_u64()
                                        .ok_or_else(|| protocol("invalid subscription usage"))?);
                                }
                            }
                        }
                        terminal = Some((kind.into(), reason.into()));
                    }
                }
                _ => return Err(protocol("invalid or unsupported subscription record")),
            }
        }
    }
    decoder.finish().map_err(protocol)?;
    if terminal.is_none() {
        return Err(protocol("subscription EOF without terminal"));
    }
    let result = json!({"id":format!("msg_{}",uuid::Uuid::new_v4()),"type":"message","role":"assistant","model":route.model,
        "content":[{"type":"text","text":text}],"stop_reason":"end_turn","stop_sequence":null,"usage":usage});
    Ok((StatusCode::OK, axum::Json(result).into_response()))
}
