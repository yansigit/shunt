//! Command Code subscription vertical slice, distinct from the Chat API product.
//! Wire provenance: .planning/phases/14-command-code-product-separation/14-PROTOCOL-EVIDENCE.md.
pub mod efforts;
mod history;
mod ndjson;
pub mod request;
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
        headers: &'a HeaderMap,
        body: RequestBody,
    ) -> AdapterFuture<'a> {
        Box::pin(async move { forward(state, route, headers, body).await })
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

async fn forward(
    state: AppState,
    route: Route,
    headers: &HeaderMap,
    body: RequestBody,
) -> Result<(StatusCode, Response<Body>), AdapterError> {
    let provider = state
        .config
        .provider(&route.provider)
        .ok_or_else(|| invalid("unknown subscription provider"))?;
    validate_provider(provider).map_err(invalid)?;
    if body.json().get("stream").and_then(Value::as_bool) == Some(true) {
        return Err(invalid(
            "Command Code streaming client support is not implemented in this slice",
        ));
    }
    let payload =
        request::translate_request(body.json(), &route.upstream_model, route.effort.as_deref())?;
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
    let mut ids = headers.get_all("x-claude-code-session-id").iter();
    let conversation = ids
        .next()
        .map(|v| {
            v.to_str()
                .map_err(|_| invalid("invalid Command Code conversation identity"))
        })
        .transpose()?;
    if ids.next().is_some() {
        return Err(invalid("ambiguous Command Code conversation identity"));
    }
    let outbound_headers = request::headers(&access_token, conversation)?;
    let send_once = || {
        let request = client
            .post(ENDPOINT)
            .headers(outbound_headers.clone())
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
    let mut machine = crate::model::command_code_response::CommandCodeMachine::new(&route.model);
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| protocol("subscription body transport failed"))?;
        for record in decoder.feed(&chunk).map_err(protocol)? {
            machine.process_record_checked(&record).map_err(protocol)?;
        }
    }
    decoder.finish().map_err(protocol)?;
    let result = machine.final_ndjson_checked().map_err(protocol)?;
    Ok((StatusCode::OK, axum::Json(result).into_response()))
}
