//! Compose inbound Responses translation around the existing Anthropic transport.

use std::{collections::VecDeque, pin::Pin};

use axum::{
    body::{Body, Bytes},
    http::{header, HeaderValue, Response, StatusCode},
    response::IntoResponse,
};
use futures_util::{stream, Stream, StreamExt};

use crate::{
    codex_endpoint::frame::{parse_sse_block, BoundedSseFrameBuffer, MAX_CLIENT_SSE_FRAME_BYTES},
    model::inbound_responses::response::{translate_json, StreamTranslator},
};

type InputStream = Pin<Box<dyn Stream<Item = Result<Bytes, axum::Error>> + Send + 'static>>;

struct TranslationState {
    upstream: InputStream,
    framer: BoundedSseFrameBuffer,
    translator: StreamTranslator,
    pending: VecDeque<Bytes>,
    done: bool,
}

pub(crate) async fn translate(
    status: StatusCode,
    response: axum::response::Response,
    requested_stream: bool,
    model: &str,
    max_body_bytes: usize,
) -> axum::response::Response {
    if !status.is_success() {
        return translate_upstream_error(response).await;
    }
    let is_sse = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("text/event-stream"));
    if requested_stream || is_sse {
        return translate_stream(response, model);
    }

    let (mut parts, body) = response.into_parts();
    match axum::body::to_bytes(body, max_body_bytes).await {
        Ok(bytes) => match translate_json(&bytes, model) {
            Ok(translated) => {
                parts.headers.remove(header::CONTENT_LENGTH);
                parts.headers.remove(header::CONTENT_ENCODING);
                parts.headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                );
                Response::from_parts(parts, Body::from(translated)).into_response()
            }
            Err(message) => translated_error(StatusCode::BAD_GATEWAY, message),
        },
        Err(_) => translated_error(
            StatusCode::BAD_GATEWAY,
            "Anthropic response exceeded the translation limit".into(),
        ),
    }
}

async fn translate_upstream_error(response: axum::response::Response) -> axum::response::Response {
    let (mut parts, body) = response.into_parts();
    let bytes = axum::body::to_bytes(body, 64 * 1024).await.ok();
    let (kind, message) = bytes
        .as_deref()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(bytes).ok())
        .and_then(|value| {
            let error = value.get("error")?.as_object()?;
            Some((
                error
                    .get("type")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("api_error")
                    .to_string(),
                error
                    .get("message")
                    .and_then(serde_json::Value::as_str)?
                    .to_string(),
            ))
        })
        .unwrap_or_else(|| {
            (
                "api_error".to_string(),
                bytes
                    .as_deref()
                    .map(|body| String::from_utf8_lossy(body).into_owned())
                    .unwrap_or_else(|| "upstream error body exceeded the translation limit".into()),
            )
        });
    let body = serde_json::to_vec(&serde_json::json!({
        "error":{"message":message,"type":kind,"code":null}
    }))
    .expect("OpenAI error envelope is serializable");
    parts.headers.remove(header::CONTENT_LENGTH);
    parts.headers.remove(header::CONTENT_ENCODING);
    parts.headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    Response::from_parts(parts, Body::from(body)).into_response()
}

fn translate_stream(response: axum::response::Response, model: &str) -> axum::response::Response {
    let (mut parts, body) = response.into_parts();
    parts.headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("text/event-stream"),
    );
    parts.headers.remove(header::CONTENT_LENGTH);
    let mut translator = StreamTranslator::new(model);
    let pending = translator
        .start()
        .into_iter()
        .map(Bytes::from)
        .collect::<VecDeque<_>>();
    let state = TranslationState {
        upstream: Box::pin(body.into_data_stream()),
        framer: BoundedSseFrameBuffer::new(MAX_CLIENT_SSE_FRAME_BYTES),
        translator,
        pending,
        done: false,
    };
    let output = stream::unfold(state, |mut state| async move {
        loop {
            if let Some(bytes) = state.pending.pop_front() {
                return Some((Ok::<_, std::io::Error>(bytes), state));
            }
            if state.done {
                return None;
            }
            match state.upstream.next().await {
                Some(Ok(chunk)) => match state.framer.feed(&chunk) {
                    Ok(frames) => {
                        for frame in frames {
                            apply_frame(&mut state, &frame);
                            if state.translator.is_terminal() {
                                state.framer.dispose();
                                state
                                    .pending
                                    .push_back(Bytes::from_static(b"data: [DONE]\n\n"));
                                state.done = true;
                                break;
                            }
                        }
                    }
                    Err(error) => {
                        state.pending.extend(
                            state
                                .translator
                                .apply(
                                    "error",
                                    serde_json::json!({"error":{"message":error.to_string()}}),
                                )
                                .into_iter()
                                .map(Bytes::from),
                        );
                        state
                            .pending
                            .push_back(Bytes::from_static(b"data: [DONE]\n\n"));
                        state.done = true;
                    }
                },
                Some(Err(error)) => {
                    state.pending.extend(
                        state
                            .translator
                            .apply(
                                "error",
                                serde_json::json!({"error":{"message":format!("Anthropic response stream failed: {error}")}}),
                            )
                            .into_iter()
                            .map(Bytes::from),
                    );
                    state
                        .pending
                        .push_back(Bytes::from_static(b"data: [DONE]\n\n"));
                    state.done = true;
                }
                None => {
                    if let Ok(Some(frame)) = state.framer.finish() {
                        apply_frame(&mut state, &frame);
                    }
                    state
                        .pending
                        .extend(state.translator.eof().into_iter().map(Bytes::from));
                    state
                        .pending
                        .push_back(Bytes::from_static(b"data: [DONE]\n\n"));
                    state.done = true;
                }
            }
        }
    });
    Response::from_parts(parts, Body::from_stream(output)).into_response()
}

fn apply_frame(state: &mut TranslationState, frame: &[u8]) {
    let Ok(block) = std::str::from_utf8(frame) else {
        state.pending.extend(
            state
                .translator
                .apply(
                    "error",
                    serde_json::json!({"error":{"message":"Anthropic SSE contained invalid UTF-8"}}),
                )
                .into_iter()
                .map(Bytes::from),
        );
        return;
    };
    let Some(payload) = parse_sse_block(block) else {
        return;
    };
    let Ok(data) = serde_json::from_str::<serde_json::Value>(&payload) else {
        state.pending.extend(
            state
                .translator
                .apply(
                    "error",
                    serde_json::json!({"error":{"message":"Anthropic SSE contained invalid JSON"}}),
                )
                .into_iter()
                .map(Bytes::from),
        );
        return;
    };
    let event = block
        .lines()
        .find_map(|line| line.strip_prefix("event:").map(str::trim))
        .or_else(|| data.get("type").and_then(serde_json::Value::as_str))
        .unwrap_or("")
        .to_string();
    state.pending.extend(
        state
            .translator
            .apply(&event, data)
            .into_iter()
            .map(Bytes::from),
    );
}

fn translated_error(status: StatusCode, message: String) -> axum::response::Response {
    (
        status,
        axum::Json(serde_json::json!({
            "error":{"message":message,"type":"upstream_error","code":"translation_error"}
        })),
    )
        .into_response()
}
