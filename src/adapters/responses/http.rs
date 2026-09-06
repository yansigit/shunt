//! The HTTP Responses transport: send the request, then relay the upstream
//! answer to the client as Anthropic SSE or a single JSON body. The default
//! path for every provider and the fallback when the websocket transport fails
//! to connect (see [`super::forward`]).

use axum::{
    body::{Body, Bytes},
    http::{Response, StatusCode},
    response::IntoResponse,
};
use futures_util::{stream, StreamExt};
use serde_json::json;

use crate::{
    adapters::AdapterError,
    auth::Credential,
    model::responses::{map_error_value, ResponseEvent, ResponsesProtocolError},
    routing::Route,
    server::AppState,
};

use super::body::{prepare_body, PreparedBody};
use super::context::{ForwardOptions, RelayOptions};
use super::error::{backend_error, mapped_upstream_error, own_error, transport_error};
use super::request::request_builder;

const MAX_RESPONSES_SSE_EVENT_BYTES: usize = 8 * 1024 * 1024;
const MAX_RESPONSES_EVENTS_PER_FEED: usize = 256;
const MAX_RESPONSES_JSON_RESPONSE_BYTES: usize = 32 * 1024 * 1024;

/// Send the upstream Responses HTTP request and return the raw response
/// without judging its status. Split out of [`forward_http`] so the account
/// pool path ([`forward_chatgpt_oauth`]) can classify a response for failover
/// before deciding whether to relay, retry, or rotate. Returns the raw
/// `reqwest::Error` so the bounded-retry layer can distinguish transient
/// transport failures from deterministic ones.
pub(super) async fn http_send(
    state: &AppState,
    route: &Route,
    credential: Credential,
    session_id: Option<&str>,
    body: PreparedBody,
) -> Result<reqwest::Response, crate::upstream_timeout::SendError<reqwest::Error>> {
    crate::upstream_timeout::wait(
        state.config.server.timeouts.upstream_ttfb_ms,
        body.attach(request_builder(state, route, credential, session_id))
            .send(),
    )
    .await
}

/// The bounded-retry policy for `route`'s provider (issue #48), or a disabled
/// policy when the provider somehow isn't found (it was validated at routing).
fn provider_retry_policy(state: &AppState, route: &Route) -> crate::retry::RetryPolicy {
    state
        .config
        .provider(&route.provider)
        .map(|provider| provider.retry.policy())
        .unwrap_or(crate::retry::RetryPolicy::DISABLED)
}

/// Drive a turn over the HTTP Responses path. The default transport for every
/// provider, and the fallback when the opt-in websocket transport fails to
/// connect (see [`forward`]).
pub(super) async fn forward_http(
    state: &AppState,
    route: &Route,
    forward: ForwardOptions,
    session_id: Option<&str>,
) -> Result<(StatusCode, axum::response::Response), AdapterError> {
    let ForwardOptions {
        upstream_body,
        credential,
        auth,
        turn,
        codex_quota_account,
        estimate_input,
    } = forward;
    // Kick off the CPU-bound tiktoken encode on the blocking pool *before* the
    // upstream request so it overlaps that round-trip; the result is not needed
    // until the response stream (and thus message_start) begins. `None` on
    // non-streaming turns and non-tiktoken providers (gated in `forward`).
    let estimate_handle = estimate_input.map(|request| {
        tokio::task::spawn_blocking(move || crate::count_tokens::count_input_tokens_value(&request))
    });
    // The account-pool path drives its own failover and deliberately does not
    // layer retry on top. This single-credential path retries only before any
    // response body is handed to the streaming/JSON relay.
    let policy = provider_retry_policy(state, route);
    let body = prepare_body(state, route, upstream_body.as_ref()).await;
    let upstream = crate::retry::send_with_retry_with_safety(
        policy,
        &route.provider,
        crate::retry::RetrySafety::NonIdempotentPost,
        || http_send(state, route, credential.clone(), session_id, body.clone()),
    )
    .await
    .map_err(|error| error.into_adapter_error(|error| transport_error(error.to_string())))?;
    if let Some(account) = &codex_quota_account {
        state
            .accounts
            .note_codex_quota(&route.provider, account, upstream.headers());
    }
    let status = upstream.status();
    if !status.is_success() {
        return Err(mapped_upstream_error(status, upstream, auth).await);
    }
    if turn.client_wants_stream {
        let input_tokens_estimate = match estimate_handle {
            Some(handle) => handle.await.unwrap_or(0),
            None => 0,
        };
        let keepalive = std::time::Duration::from_secs(state.config.server.sse_keepalive_seconds);
        Ok((
            StatusCode::OK,
            stream_response(
                upstream,
                turn.relay(route),
                input_tokens_estimate,
                keepalive,
            ),
        ))
    } else {
        // Thread the real response status: `json_response` returns a `502` when
        // a backend error event surfaced via `backend_error` (issue #113), so
        // the proxy's access log (`upstream_status`) and `record_proxied_request`
        // metrics reflect the failure instead of a hardcoded `200`.
        let response = json_response(upstream, turn.relay(route)).await?;
        Ok((response.status(), response))
    }
}

pub(super) fn stream_response(
    upstream: reqwest::Response,
    relay: RelayOptions,
    input_tokens_estimate: u64,
    keepalive: std::time::Duration,
) -> axum::response::Response {
    let bytes = upstream.bytes_stream();
    let parser = SseParser::default();
    let machine = relay
        .machine()
        .with_input_estimate(input_tokens_estimate)
        .without_content_accumulation();
    let output = stream::unfold((bytes, parser, machine, false), |state| async move {
        let (mut bytes, mut parser, mut machine, mut finished) = state;
        if finished {
            return None;
        }
        loop {
            match bytes.next().await {
                Some(Ok(chunk)) => {
                    let events = match parser.push(&chunk) {
                        Ok(events) => events,
                        Err(error) => {
                            return Some((
                                Ok::<_, std::convert::Infallible>(Bytes::from(protocol_error_sse(
                                    &error,
                                ))),
                                (bytes, parser, machine, true),
                            ));
                        }
                    };
                    let mut data = String::new();
                    for event in events {
                        match machine.apply_checked(event) {
                            Ok(frames) => data.extend(frames),
                            Err(error) => {
                                data.push_str(&protocol_error_sse(&error));
                                finished = true;
                                break;
                            }
                        }
                    }
                    if !data.is_empty() {
                        return Some((
                            Ok::<_, std::convert::Infallible>(Bytes::from(data)),
                            (bytes, parser, machine, finished),
                        ));
                    }
                }
                Some(Err(error)) => {
                    let error = ResponsesProtocolError::new(format!(
                        "failed to read upstream Responses stream: {error}"
                    ));
                    return Some((
                        Ok(Bytes::from(protocol_error_sse(&error))),
                        (bytes, parser, machine, true),
                    ));
                }
                None => {
                    let data = match parser.finish().and_then(|tail| {
                        if let Some(event) = tail {
                            machine.apply_checked(event)?;
                        }
                        machine.finish_checked()
                    }) {
                        Ok(frames) => frames.join(""),
                        Err(error) => protocol_error_sse(&error),
                    };
                    finished = true;
                    if data.is_empty() {
                        return None;
                    }
                    return Some((Ok(Bytes::from(data)), (bytes, parser, machine, finished)));
                }
            }
        }
    });

    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/event-stream")
        .body(Body::from_stream(crate::keepalive::with_pings(
            output, keepalive,
        )))
        .expect("response builder uses valid status and headers")
        .into_response()
}

/// Collect the full HTTP Responses SSE body into a single Anthropic message for
/// a non-streaming client. A backend-sent `error` / `response.failed` event
/// (delivered as a normal event on the `200 OK` stream — rate-limit,
/// content-policy refusal) is surfaced as a gateway error rather than a `200 OK`
/// with the partial content accumulated before it, so the client cannot mistake
/// a backend failure for a truncated-but-successful result (issue #113). This
/// mirrors the streaming path, which emits the same error inline as an SSE
/// `error` event.
pub(super) async fn json_response(
    upstream: reqwest::Response,
    relay: RelayOptions,
) -> Result<axum::response::Response, AdapterError> {
    json_response_with_limit(upstream, relay, MAX_RESPONSES_JSON_RESPONSE_BYTES).await
}

async fn json_response_with_limit(
    upstream: reqwest::Response,
    relay: RelayOptions,
    max_wire_bytes: usize,
) -> Result<axum::response::Response, AdapterError> {
    let mut bytes = upstream.bytes_stream();
    let mut wire_bytes = 0usize;
    let mut parser = SseParser::default();
    let mut machine = relay.machine();
    while let Some(chunk) = bytes.next().await {
        let chunk =
            chunk.map_err(|error| own_error(format!("failed to read Responses body: {error}")))?;
        wire_bytes = wire_bytes
            .checked_add(chunk.len())
            .ok_or_else(|| own_error("upstream Responses body size overflow".to_string()))?;
        if wire_bytes > max_wire_bytes {
            return Err(own_error(format!(
                "upstream Responses body exceeded {max_wire_bytes} bytes"
            )));
        }
        let events = parser.push(&chunk).map_err(protocol_adapter_error)?;
        for event in events {
            let _ = machine
                .apply_checked(event)
                .map_err(protocol_adapter_error)?;
            if let Some((status, error)) = machine.take_backend_error() {
                return Err(backend_error(status, error));
            }
        }
    }
    if let Some(event) = parser.finish().map_err(protocol_adapter_error)? {
        let _ = machine
            .apply_checked(event)
            .map_err(protocol_adapter_error)?;
    }
    if let Some((status, error)) = machine.take_backend_error() {
        return Err(backend_error(status, error));
    }
    let message = machine
        .final_json_checked()
        .map_err(protocol_adapter_error)?;
    Ok((StatusCode::OK, axum::Json(message)).into_response())
}

fn protocol_adapter_error(error: ResponsesProtocolError) -> AdapterError {
    own_error(error.to_string())
}

fn protocol_error_sse(error: &ResponsesProtocolError) -> String {
    let value = map_error_value(
        &json!({ "message": error.to_string() }),
        StatusCode::BAD_GATEWAY,
    );
    format!("event: error\ndata: {value}\n\n")
}

/// Frame-buffers the upstream SSE byte stream. Buffering raw bytes — rather than
/// decoding each transport chunk with `from_utf8_lossy` — keeps a multi-byte
/// UTF-8 code point intact when it straddles a chunk boundary: the incomplete
/// trailing bytes stay in the buffer until the next chunk completes them. Frame
/// boundaries are the ASCII `\n\n`, which can never fall inside a multi-byte
/// sequence, so every extracted frame is already complete UTF-8.
struct SseParser {
    buffer: Vec<u8>,
    max_event_bytes: usize,
    max_events_per_feed: usize,
    disposed: bool,
}

impl Default for SseParser {
    fn default() -> Self {
        Self::with_limits(MAX_RESPONSES_SSE_EVENT_BYTES, MAX_RESPONSES_EVENTS_PER_FEED)
    }
}

impl SseParser {
    fn with_limits(max_event_bytes: usize, max_events_per_feed: usize) -> Self {
        assert!(max_event_bytes > 0);
        assert!(max_events_per_feed > 0);
        Self {
            buffer: Vec::new(),
            max_event_bytes,
            max_events_per_feed,
            disposed: false,
        }
    }

    fn fail(&mut self, message: impl Into<String>) -> ResponsesProtocolError {
        self.buffer.clear();
        self.disposed = true;
        ResponsesProtocolError::new(message)
    }

    fn push(&mut self, chunk: &[u8]) -> Result<Vec<ResponseEvent>, ResponsesProtocolError> {
        if self.disposed {
            return Err(ResponsesProtocolError::new(
                "upstream Responses SSE parser is disposed",
            ));
        }
        let mut events = Vec::new();
        let mut frames_seen = 0usize;
        for &byte in chunk {
            self.buffer.push(byte);
            if let Some(delimiter_len) = terminal_delimiter_len(&self.buffer) {
                let frame_len = self.buffer.len() - delimiter_len;
                if frame_len > self.max_event_bytes {
                    return Err(self.fail(format!(
                        "upstream Responses SSE event exceeded {} bytes",
                        self.max_event_bytes
                    )));
                }
                if frames_seen >= self.max_events_per_feed {
                    return Err(self.fail(format!(
                        "upstream Responses SSE chunk exceeded {} event limit",
                        self.max_events_per_feed
                    )));
                }
                frames_seen += 1;
                let frame = self.buffer[..frame_len].to_vec();
                self.buffer.clear();
                match parse_sse_frame(&frame) {
                    Ok(Some(event)) => events.push(event),
                    Ok(None) => {}
                    Err(error) => return Err(self.fail(error.to_string())),
                }
            } else if retained_candidate_len(&self.buffer) > self.max_event_bytes {
                return Err(self.fail(format!(
                    "upstream Responses SSE event exceeded {} bytes",
                    self.max_event_bytes
                )));
            }
        }
        Ok(events)
    }

    fn finish(&mut self) -> Result<Option<ResponseEvent>, ResponsesProtocolError> {
        if self.disposed {
            return Err(ResponsesProtocolError::new(
                "upstream Responses SSE parser is disposed",
            ));
        }
        self.disposed = true;
        if self.buffer.is_empty() {
            return Ok(None);
        }
        if self.buffer.len() > self.max_event_bytes {
            return Err(self.fail(format!(
                "upstream Responses SSE event exceeded {} bytes",
                self.max_event_bytes
            )));
        }
        let frame = std::mem::take(&mut self.buffer);
        parse_sse_frame(&frame)
    }
}

fn retained_candidate_len(buffer: &[u8]) -> usize {
    const PREFIXES: &[&[u8]] = &[b"\r", b"\n", b"\r\n", b"\n\r", b"\r\n\r", b"\n\r\n"];
    let delimiter_prefix = PREFIXES
        .iter()
        .filter(|prefix| buffer.ends_with(prefix))
        .map(|prefix| prefix.len())
        .max()
        .unwrap_or(0);
    buffer.len().saturating_sub(delimiter_prefix)
}

fn terminal_delimiter_len(buffer: &[u8]) -> Option<usize> {
    [b"\r\n\r\n".as_slice(), b"\n\r\n", b"\r\n\n", b"\n\n"]
        .into_iter()
        .find(|delimiter| buffer.ends_with(delimiter))
        .map(<[u8]>::len)
}

fn parse_sse_frame(frame: &[u8]) -> Result<Option<ResponseEvent>, ResponsesProtocolError> {
    let frame = std::str::from_utf8(frame)
        .map_err(|_| ResponsesProtocolError::new("invalid UTF-8 in upstream Responses SSE"))?;
    let mut event = None;
    let mut data = Vec::new();
    for line in frame.lines() {
        if let Some(value) = line.strip_prefix("event:") {
            event = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("data:") {
            data.push(value.strip_prefix(' ').unwrap_or(value));
        }
    }
    let data = data.join("\n");
    if data.is_empty() || data == "[DONE]" {
        return Ok(None);
    }
    let data = serde_json::from_str(&data)
        .map_err(|_| ResponsesProtocolError::new("invalid JSON in upstream Responses SSE"))?;
    Ok(Some(ResponseEvent { event, data }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use serde_json::Value;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// The default relay options for these tests: the `gpt-5.2-codex` model with
    /// both protocol toggles off.
    fn relay_opts() -> RelayOptions {
        RelayOptions {
            model: "gpt-5.2-codex".to_string(),
            thinking_enabled: false,
            tool_search_native: false,
        }
    }

    /// Serves `body` at `status` from a mock server and returns the resulting
    /// `reqwest::Response`, mirroring the shape `json_response` reads in
    /// production (a response off the wire, not built in-process).
    async fn upstream_response(status: u16, body: &str) -> reqwest::Response {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/e"))
            .respond_with(ResponseTemplate::new(status).set_body_string(body.to_string()))
            .mount(&server)
            .await;
        reqwest::Client::new()
            .get(format!("{}/e", server.uri()))
            .send()
            .await
            .expect("mock request should succeed")
    }

    async fn response_body_json(response: axum::response::Response) -> Value {
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        serde_json::from_slice(&bytes).expect("response body should be JSON")
    }

    /// A backend-sent `response.failed` event on the HTTP JSON path surfaces as a
    /// `502` gateway error rather than a `200 OK` with the partial content
    /// collected before it (issue #113).
    #[tokio::test]
    async fn json_response_surfaces_backend_error_event_as_gateway_error() {
        let sse = concat!(
            "event: response.created\n",
            "data: {\"response\":{\"id\":\"resp_1\"}}\n\n",
            "event: response.output_text.delta\n",
            "data: {\"delta\":\"partial\"}\n\n",
            "event: response.failed\n",
            "data: {\"type\":\"response.failed\",\"response\":{\"error\":{\"code\":\"server_error\",\"message\":\"Upstream failed\"}}}\n\n",
        );
        let upstream = upstream_response(200, sse).await;
        let error = json_response(upstream, relay_opts())
            .await
            .expect_err("backend error event should stop failover");

        assert!(error.failure.is_none());
        assert_eq!(error.response.status(), StatusCode::BAD_GATEWAY);
        let body = response_body_json(*error.response).await;
        assert_eq!(body["type"], "error");
        assert_eq!(body["error"]["type"], "api_error");
        assert_eq!(body["error"]["message"], "Upstream failed");
    }

    /// An in-stream `rate_limit_exceeded` on the HTTP JSON path is a throttle the
    /// backend delivered on a `200 OK` stream: it must reach the client as `429`
    /// `rate_limit_error` (not a generic `502`) but stay terminal — the upstream
    /// already accepted the turn, so it is never replayed on the next one.
    #[tokio::test]
    async fn json_response_maps_in_stream_rate_limit_to_429_without_failover() {
        let sse = concat!(
            "event: response.created\n",
            "data: {\"response\":{\"id\":\"resp_1\"}}\n\n",
            "event: response.failed\n",
            "data: {\"type\":\"response.failed\",\"response\":{\"error\":{\"code\":\"rate_limit_exceeded\",\"message\":\"Rate limit reached\"}}}\n\n",
        );
        let upstream = upstream_response(200, sse).await;
        let error = json_response(upstream, relay_opts())
            .await
            .expect_err("in-stream rate limit is an error");

        assert!(error.failure.is_none());
        assert_eq!(error.response.status(), StatusCode::TOO_MANY_REQUESTS);
        let body = response_body_json(*error.response).await;
        assert_eq!(body["error"]["type"], "rate_limit_error");
        assert_eq!(body["error"]["message"], "Rate limit reached");
    }

    /// A clean turn still returns the collected Anthropic message as `200 OK` —
    /// the backend-error gate must not regress the success path.
    #[tokio::test]
    async fn json_response_returns_ok_for_a_clean_turn() {
        let sse = concat!(
            "event: response.created\n",
            "data: {\"response\":{\"id\":\"resp_1\"}}\n\n",
            "event: response.output_item.added\n",
            "data: {\"item\":{\"type\":\"message\"}}\n\n",
            "event: response.output_text.delta\n",
            "data: {\"delta\":\"hello\"}\n\n",
            "event: response.output_text.done\n",
            "data: {}\n\n",
            "event: response.completed\n",
            "data: {\"response\":{\"usage\":{\"input_tokens\":3,\"output_tokens\":1}}}\n\n",
        );
        let upstream = upstream_response(200, sse).await;
        let response = json_response(upstream, relay_opts())
            .await
            .expect("json_response builds a response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_json(response).await;
        assert_eq!(body["type"], "message");
        assert_eq!(body["content"][0]["text"], "hello");
    }

    /// The streaming path prefixes a synthesized completion with
    /// `stream_metrics::UPSTREAM_TRUNCATED_MARKER` when the upstream
    /// connection ends before a real terminal event, so the observer can
    /// still classify the stream as an upstream cut instead of a normal
    /// completion (see that constant's doc comment for the full rationale).
    #[tokio::test]
    async fn responses_terminal_stream_rejects_a_truncated_upstream() {
        let sse = concat!(
            "event: response.created\n",
            "data: {\"response\":{\"id\":\"resp_1\"}}\n\n",
            "event: response.output_item.added\n",
            "data: {\"item\":{\"type\":\"message\"}}\n\n",
            "event: response.output_text.delta\n",
            "data: {\"delta\":\"partial\"}\n\n",
        );
        let upstream = upstream_response(200, sse).await;
        let response = stream_response(
            upstream,
            relay_opts(),
            0,
            std::time::Duration::from_secs(30),
        );
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("streamed body should be readable");
        let body = std::str::from_utf8(&bytes).expect("body is utf8");
        let marker = std::str::from_utf8(crate::stream_metrics::UPSTREAM_TRUNCATED_MARKER).unwrap();

        assert!(!body.contains(marker));
        assert!(body.contains("event: error"));
        assert!(!body.contains("event: message_stop"));
    }

    /// A clean turn that reaches `response.completed` must not carry the
    /// truncation marker — it exists only for the EOF-before-terminal path.
    #[tokio::test]
    async fn stream_response_does_not_mark_a_genuine_completion() {
        let sse = concat!(
            "event: response.created\n",
            "data: {\"response\":{\"id\":\"resp_1\"}}\n\n",
            "event: response.output_item.added\n",
            "data: {\"item\":{\"type\":\"message\"}}\n\n",
            "event: response.output_text.delta\n",
            "data: {\"delta\":\"hello\"}\n\n",
            "event: response.output_text.done\n",
            "data: {}\n\n",
            "event: response.completed\n",
            "data: {\"response\":{\"usage\":{\"input_tokens\":3,\"output_tokens\":1}}}\n\n",
        );
        let upstream = upstream_response(200, sse).await;
        let response = stream_response(
            upstream,
            relay_opts(),
            0,
            std::time::Duration::from_secs(30),
        );
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("streamed body should be readable");
        let body = std::str::from_utf8(&bytes).expect("body is utf8");
        let marker = std::str::from_utf8(crate::stream_metrics::UPSTREAM_TRUNCATED_MARKER).unwrap();

        assert!(
            !body.contains(marker),
            "clean completion must not carry the marker, got: {body}"
        );
        assert!(body.contains("event: message_stop"));
    }

    /// A multi-byte code point split across two transport chunks must survive
    /// intact. Decoding each chunk with `from_utf8_lossy` in isolation would
    /// replace the straddling bytes with U+FFFD; buffering raw bytes until a
    /// frame boundary keeps the text whole.
    #[test]
    fn sse_parser_preserves_multibyte_char_split_across_chunks() {
        let frame = "event: delta\ndata: {\"text\":\"안녕\"}\n\n";
        // Split one byte into the 3-byte '녕' so the first chunk ends
        // mid-code-point.
        let split = frame.find('녕').unwrap() + 1;
        let (head, tail) = frame.as_bytes().split_at(split);

        let mut parser = SseParser::default();
        // No frame boundary yet, and the incomplete byte must be held back
        // rather than decoded and corrupted.
        assert!(parser.push(head).unwrap().is_empty());

        let events = parser.push(tail).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.as_deref(), Some("delta"));
        assert_eq!(events[0].data["text"], "안녕");
    }

    /// A completed frame followed by an incomplete frame is emitted immediately,
    /// while the trailing bytes remain buffered and are not rescanned from the
    /// beginning when the next chunk arrives.
    #[test]
    fn sse_parser_retains_an_incomplete_trailing_frame() {
        let mut parser = SseParser::default();
        let events = parser
            .push(b"event: a\ndata: {\"n\":1}\n\nevent: b\ndata: {\"n\":")
            .unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data["n"], 1);

        let events = parser.push(b"2}\n\n").unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.as_deref(), Some("b"));
        assert_eq!(events[0].data["n"], 2);
    }

    /// A frame terminator split across chunks is detected by rescanning the
    /// previous chunk's final byte.
    #[test]
    fn sse_parser_detects_terminator_split_across_chunks() {
        let mut parser = SseParser::default();
        assert!(parser
            .push(b"event: a\ndata: {\"n\":1}\n")
            .unwrap()
            .is_empty());

        let events = parser.push(b"\n").unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data["n"], 1);
    }

    /// A frame that arrives split at an arbitrary ASCII byte still parses once
    /// the terminator lands, and only completed frames are emitted per push.
    #[test]
    fn sse_parser_emits_only_completed_frames() {
        let mut parser = SseParser::default();
        assert!(parser
            .push(b"event: a\ndata: {\"n\":1}\n")
            .unwrap()
            .is_empty());
        let events = parser.push(b"\nevent: b\ndata: {\"n\":2}\n\n").unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].data["n"], 1);
        assert_eq!(events[1].data["n"], 2);
    }

    #[test]
    fn responses_bounds_sse_residual_plus_one_disposes_immediately() {
        let mut parser = SseParser::with_limits(8, 16);
        assert!(parser.push(b"12345678").unwrap().is_empty());
        let error = parser.push(b"9").unwrap_err();
        assert!(error.to_string().contains("exceeded 8 bytes"));
        assert!(parser.push(b"data: {}\n\n").is_err());
    }

    #[test]
    fn responses_bounds_counts_every_completed_frame_per_feed() {
        let mut parser = SseParser::with_limits(64, 2);
        let error = parser.push(b": one\n\n: two\n\n: three\n\n").unwrap_err();
        assert!(error.to_string().contains("exceeded 2 event limit"));
    }

    #[test]
    fn responses_bounds_malformed_json_disposes_parser() {
        let mut parser = SseParser::with_limits(64, 4);
        assert!(parser.push(b"data: not-json\n\n").is_err());
        assert!(parser.push(b"data: {}\n\n").is_err());
    }

    #[test]
    fn responses_bounds_split_crlf_is_incremental() {
        let mut parser = SseParser::with_limits(64, 4);
        assert!(parser
            .push(b"event: response.completed\r\ndata: {}\r\n")
            .unwrap()
            .is_empty());
        let events = parser.push(b"\r\n").unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.as_deref(), Some("response.completed"));
    }

    #[test]
    fn responses_bounds_sse_event_exact_cap_is_accepted() {
        let frame = b"data: {}";
        let mut parser = SseParser::with_limits(frame.len(), 1);
        let mut input = frame.to_vec();
        input.extend_from_slice(b"\n\n");
        assert_eq!(parser.push(&input).unwrap().len(), 1);
    }

    #[test]
    fn responses_bounds_sse_invalid_utf8_disposes_parser() {
        let mut parser = SseParser::with_limits(64, 4);
        assert!(parser.push(b"data: \xff\n\n").is_err());
        assert!(parser.push(b"data: {}\n\n").is_err());
    }

    #[tokio::test]
    async fn responses_bounds_json_collector_exact_and_plus_one() {
        let sse = "event: response.completed\ndata: {}\n\n";
        let exact = upstream_response(200, sse).await;
        let response = json_response_with_limit(exact, relay_opts(), sse.len())
            .await
            .expect("exact wire limit should pass");
        assert_eq!(response.status(), StatusCode::OK);

        let oversized = upstream_response(200, sse).await;
        let error = json_response_with_limit(oversized, relay_opts(), sse.len() - 1)
            .await
            .expect_err("exact limit plus one should fail");
        assert_eq!(error.response.status(), StatusCode::BAD_GATEWAY);
    }
}
