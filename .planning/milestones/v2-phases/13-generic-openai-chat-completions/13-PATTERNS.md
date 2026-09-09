# Phase 13: Generic OpenAI Chat Completions - Pattern Map

**Mapped:** 2026-09-08
**Files analyzed:** 9 (6 new, 3 modified)
**Analogs found:** 9 / 9

All analog paths verified git-tracked via `git ls-files`. The Gemini vertical
slice (Phase 4-era) is the structural template for this phase: same ingress
(Anthropic Messages), same outbound shape (pure translation + transport + SSE
framing), same conformance harness. Corrections from the orchestrator are
embedded per file: `RetrySafety::ConnectOnly` (not `NonIdempotentPost`),
redirect refusal via `Policy::none()`, and split tool identity/name validation
at the assembly boundary.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/adapters/openai_chat/mod.rs` (new) | provider adapter | streaming + request-response | `src/adapters/gemini/mod.rs` | exact |
| `src/adapters/openai_chat/sse.rs` (new) | utility (byte framing) | streaming | `src/adapters/gemini/sse.rs` | exact |
| `src/model/openai_chat_request.rs` (new) | model / translator | transform | `src/model/gemini_request.rs` | exact |
| `src/model/openai_chat_response.rs` (new) | model / state machine | streaming + transform | `src/model/gemini.rs` (`GeminiSseMachine`) | exact |
| `tests/openai_chat_translate.rs` (new) | test (pure) | transform | `tests/gemini_translate.rs` | exact |
| `tests/openai_chat_conformance.rs` (new) | test (loopback integration) | streaming + request-response | `tests/gemini_conformance.rs` | exact |
| `src/routing.rs` (modified) | route enum | n/a | self (`AdapterKind` enum + `From<ProviderKind>`) | self |
| `src/config.rs` (modified) | config | n/a | self (`ProviderKind`/`AuthMode`) + `validate_gateway_telemetry_destination` for URL strictness | role-match |
| `src/proxy/failover.rs` (modified) | dispatch | request-response | self (`dispatch` match, count_tokens list) | self |

`src/retry.rs` is **not modified**: Chat consumes the existing
`RetrySafety::ConnectOnly` driver. A Chat-specific const (modeled on
`CURSOR_RETRY_SAFETY`, `src/retry.rs:152`) can live in the new adapter module.

## Pattern Assignments

### `src/adapters/openai_chat/mod.rs` (provider adapter, streaming + request-response)

**Analog:** `src/adapters/gemini/mod.rs` (tracked; verified this session)

**Structure pattern** (gemini/mod.rs:1-66): a thin `Adapter` impl delegating to
a free `forward` function, module-private helpers, and one error constructor:

```rust
mod sse;

use crate::adapters::{Adapter, AdapterError, AdapterFuture};
use crate::request::RequestBody;
use crate::routing::Route;
use crate::server::AppState;

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
```

**Local-error pattern** (gemini/mod.rs:113-126): gateway-owned errors in the
Anthropic error shape, `failure: None` so failover never redispatches local
validation failures (CHAT ingress is Anthropic-shaped; the Codex-endpoint
exception does not apply):

```rust
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
```

**Send + retry-safety pattern** (gemini/mod.rs:416-457, per the root
correction Chat uses `ConnectOnly`, never `NonIdempotentPost`):

```rust
let ttfb_ms = state.config.server.timeouts.upstream_ttfb_ms;
let retry_safety = retry_safety_for_auth(provider.auth);
let send_once = |token: String| { /* client.post(&endpoint)... */ };
let send_with_retry = |token: String| {
    crate::retry::send_with_retry_with_safety(
        policy,
        &route.provider,
        retry_safety,          // Chat: RetrySafety::ConnectOnly
        move || send_once(token.clone()),
    )
};
let map_send_error = |error: crate::upstream_timeout::SendError<reqwest::Error>| {
    error.into_adapter_error(|error| AdapterError {
        message: format!("network error calling Gemini backend: {error}"),
        response: Box::new(StatusCode::BAD_GATEWAY.into_response()),
        failure: None, // see note below: pre-header classification must use AdapterFailure::BeforeHeaders only for genuine pre-send failures
    })
};
```

**Streaming relay pattern** (gemini/mod.rs:520-640): the SSE relay is a
`futures_util::stream::unfold` tuple owning the byte stream, decoder, machine,
and a `pending: Option<Bytes>` residual — cancellation is ownership-based
(dropping the stream drops everything; D-08), and errors are emitted as
Anthropic `error` events then the stream terminates:

```rust
let byte_stream = response.bytes_stream();
let mut machine = GeminiSseMachine::new_streaming_for_upstream(&route.model, &route.upstream_model);
let decoder = GeminiSseDecoder::default();
let sse_stream = futures_util::stream::unfold(
    (byte_stream, decoder, machine, false, None::<Bytes>, None::<Bytes>),
    |(mut bytes, mut decoder, mut machine, finished, mut pending, _)| async move {
        // push_one consumes one frame; slice leftover bytes back into pending
        let (consumed, item) = decoder.push_one(&chunk) ...;
        if consumed < chunk.len() { pending = Some(chunk.slice(consumed..)); }
        ...
    });
```

**Unary collection pattern** (gemini/mod.rs:162-201): bounded
`collect_unary_response` with content-length pre-check, checked_add, and a
hard byte ceiling. Chat JSON mode copies this with its own
`MAX_CHAT_UNARY_RESPONSE_BYTES`.

**Redirect refusal (D-03, required correction):** build the Chat client with
`Policy::none()` exactly like the redirect-hardened relay client
(`src/gateway/telemetry_ingest.rs:371-384`):

```rust
reqwest::Client::builder()
    .redirect(reqwest::redirect::Policy::none())
    .build()
    .expect("build redirect-hardened telemetry relay client")
```

Rationale verbatim (telemetry_ingest.rs:371-376): reqwest's default policy
follows up to 10 hops and strips only `Authorization`/`Cookie`-class headers on
cross-host redirects — a configured `x-api-key` would follow a 3xx. Do not rely
on implicit stripping. The non-success branch then surfaces the 3xx. Loopback
30x refusal test mirrors `src/gateway/tests.rs:2979`
(`telemetry_relay_does_not_follow_redirects`).

### `src/adapters/openai_chat/sse.rs` (utility, streaming)

**Analog:** `src/adapters/gemini/sse.rs` (tracked)

The Gemini `Decoder` is the exact bounded byte-framing primitive Chat needs —
SSE data-line splitting, `[DONE]` detection, duplicate-terminal and residual
failures, UTF-8 gate, and per-event byte cap:

```rust
pub(super) const MAX_EVENT_BYTES: usize = 8 * 1024 * 1024;

pub(super) enum Item { Json(Value), Done }

pub(super) fn push_one(&mut self, chunk: &[u8]) -> Result<(usize, Option<Item>), String> {
    // bounded buffer -> frame -> parse; oversized frame => fail() clears buffer, marks disposed
    if self.done { return Err(self.fail("Gemini SSE frame arrived after [DONE]")); }
    ...
}

pub(super) fn finish(&mut self) -> Result<(), String> {
    // residual bytes that are not whitespace => "ended with an unterminated event frame"
}
```

Research verified Chat uses the same `data:`/line/JSON framing with `[DONE]`
as terminal, so the planner may (a) widen `gemini::sse::Decoder` to
`pub(crate)` and re-export it, or (b) move a shared helper; either is a small
refactor (research assumption A3). Keep messages provider-neutral if shared
(`"SSE event exceeded {} bytes"` rather than "Gemini ...").

### `src/model/openai_chat_request.rs` (translator, transform)

**Analog:** `src/model/gemini_request.rs` (tracked)

**Pure-function + typed-rejection pattern** (gemini_request.rs:1-58):

```rust
//! Anthropic Messages -> Gemini generateContent request translation.
use serde_json::{json, Map, Value};
use crate::adapters::AdapterError;

fn bad_request(message: impl Into<String>) -> AdapterError {
    AdapterError {
        message: message.into(),
        response: Box::new(axum::http::StatusCode::BAD_REQUEST.into_response()),
        failure: None,
    }
}

pub fn translate_request(request: &Value) -> Result<Value, AdapterError> { ... }
```

Copy this shape: pure `&Value -> Result<Value, AdapterError>`, 400 for input
problems, whitelist-driven field assembly (`out: Map`, insert only translated
fields — no passthrough of Responses-only keys, D-04), named bounds as consts.
Tool grammar targets (from research, OpenCodex openai-chat.ts:700-830):
assistant `tool_calls` = `[{id, type: "function", function: {name, arguments: string}}]`,
tool result = `{role: "tool", tool_call_id, content}` — orphan/duplicate/missing
identity results are **rejected** (`bad_request`), never repaired.

### `src/model/openai_chat_response.rs` (state machine, streaming + transform)

**Analog:** `src/model/gemini.rs` `GeminiSseMachine` (tracked; gemini.rs:117-198)

The "one checked semantic contract for both output modes" (D-06/07) is exactly
the Gemini machine design: one struct consuming upstream chunks (JSON or SSE
payloads), emitting `SseEvent`s, with typed terminal gating:

```rust
pub struct SseEvent { pub event: String, pub data: Value }

pub struct GeminiSemanticError { message: String } // typed protocol failure

pub struct GeminiSseMachine {
    model: String, upstream_model: String, message_id: String,
    started: bool,
    terminal: TerminalState,          // single-terminal enforcement
    block_index: usize,
    active_block: Option<ActiveBlock>,
    saw_tool_use: bool,
    tool_count: usize,
    input_tokens: u64, output_tokens: u64,
    last_finish_reason: Option<String>,
    accumulate_content: bool,         // new_streaming() flips to false: no second retained copy
    content: Vec<Value>,
    retained_bytes: usize, part_count: usize,
}
```

Key methods to mirror: `process_chunk_checked` (typed failure),
`transport_close_checked` (EOF/terminal contract — Chat must fail closed on
premature EOF, residual bytes, duplicate `[DONE]`/finish_reason; the OpenCodex
`openaiChatEofTolerance` branch is NOT ported, D-07), `is_started`,
`final_json_checked` for the JSON/unary mode reusing the same validation path.
Content blocks map Chat deltas to Anthropic `message_start`/`content_block_*`/
`message_delta`/`message_stop` exactly as the machine does today.

**Tool-delta accumulation (root correction — split identity/name at the
assembly boundary):** key deltas by `index` first, then `id` (OpenCodex
accumulator key `i:<index>` / `id:<id>`). Validate fields per-delta with
provenance tolerance: an initial delta may supply only `{index}` (or
`{index, id}`), with `function.name`/`arguments` completing on later deltas.
The hard errors are: non-array `delta.tool_calls`; a first malformed `id`/
`name`/`arguments` *type*; a call that reaches the terminal without ever
receiving `function.name` ("streamed a tool call without a function name —
cannot dispatch"); duplicate `index`/id pairing; arguments string exceeding
the byte budget. Never guess pairing or synthesize results (D-05). Arguments
accumulate as raw strings and are parsed once at close (one JSON parse of the
complete object, reject incomplete objects).

### `tests/openai_chat_translate.rs` (pure test)

**Analog:** `tests/gemini_translate.rs` (tracked; lines 1-55)

```rust
use serde_json::json;
use shunt::model::gemini::{...};
use shunt::model::gemini_request::{translate_request, ...};

#[test]
fn test_translate_plain_user_message() {
    let request = json!({ "messages": [ { "role": "user", "content": "Hello, Gemini!" } ] });
    let translated = translate_request(&request).unwrap();
    assert_eq!(translated["contents"][0]["role"], "user");
}
```

Pattern: direct calls into `shunt::model::*` public functions, `json!`
fixtures, assert on translated shape; rejection cases assert `.is_err()` with
message content. Cover CHAT-02..05 plus both output modes' shared fixtures
(text/tools/usage/finish/errors paired JSON-vs-SSE, Pitfalls #5).

### `tests/openai_chat_conformance.rs` (loopback integration test)

**Analog:** `tests/gemini_conformance.rs` (tracked; lines 60-149)

```rust
async fn start_gateway_with_config(mut config: Config) -> Gateway {
    std::env::set_var("SHUNT_GEMINI_CONFORMANCE_KEY", "fixture-key");
    config.server.bind = "127.0.0.1:0".to_string();
    let listener = tokio::net::TcpListener::bind(config.server.bind_addr().unwrap()).await.unwrap();
    let (app, _, _) = server::build_router(config).unwrap();
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    ...
}

fn gemini_config_with_retry(base_url: String, max_retries: u32) -> Config {
    let provider = config.providers.get_mut("gemini").unwrap();
    provider.base_url = base_url;
    provider.auth = AuthMode::ApiKey;
    provider.api_key_env = Some("SHUNT_GEMINI_CONFORMANCE_KEY".to_string());
    ...
}
```

Copy the harness (Gateway struct with abort-on-drop, `can_bind_loopback`,
mock upstream via wiremock `Mock::given(...).mount(...)`, synthetic env API
key) renamed to a chat provider. Scenario list per D-10/CHAT-09: normal,
streaming, image, tool-heavy + interleaved tool deltas, long-context boundary,
auth stripping, **redirect refusal (loopback 30x, assert no second request)**,
malformed/truncated SSE, oversized event/argument, embedded errors, duplicate
terminals, cancellation (drop the client mid-stream), and **post-send
no-fallback** (respond with a late timeout/ambiguous failure after headers and
assert the failover chain never re-dispatched — the required ConnectOnly
regression, extendable into `tests/failover.rs` / `tests/retry.rs`).
Synthetic fixtures are unit evidence; live evidence stays a Phase 16 matter.

### `src/routing.rs` (modified)

**Analog:** self — add `OpenAiChat` to `AdapterKind` and one `From<ProviderKind>`
arm (routing.rs:9-27):

```rust
pub enum AdapterKind { Anthropic, Responses, Cursor, Gemini, AntigravityCli }
impl From<ProviderKind> for AdapterKind {
    fn from(kind: ProviderKind) -> Self {
        match kind {
            ProviderKind::Gemini => AdapterKind::Gemini,
            ...
        }
    }
}
```

### `src/config.rs` (modified)

**Analog A (kind/auth):** self — extend `ProviderKind` (config.rs:1765-1785,
`#[serde(rename_all = "snake_case")]`, so `OpenAiChat` serializes as
`"openai_chat"`) and reuse `AuthMode::ApiKey` + `api_key_env`
(config.rs:1800-1818) unchanged. Follow the existing doc-comment style on each
variant.

**Analog B (strict base-URL validation, CHAT-02):**
`validate_gateway_telemetry_destination` (config.rs:918-965) is the exact
boot-time rejection pattern to copy for provider Chat base URLs:

```rust
if let Some(offending) = offending { // query string / fragment / embedded credentials
    return Err(ConfigError::InvalidGatewayTelemetryUrl {
        index,
        message: format!(
            "must be a base OTLP endpoint (scheme, host, optional path) without {offending}; \
             shunt appends `/v1/<signal>` to it"
        ),
    });
}
```

Chat-specific grammar (on top of query/fragment/userinfo rejection): accept
scheme+host, optionally a path ending at bare origin, `/v1`, or
`/v1/chat/completions` suffix shapes (after trailing-slash trim), then append
`/chat/completions` exactly once. Add a new `ConfigError` variant in the
`ProviderBaseUrl*` family (config.rs:2080-2147) rather than reusing the
telemetry one.

### `src/proxy/failover.rs` (modified)

**Analog:** self — `dispatch` arms (failover.rs:349-376) and the count_tokens
kind list (failover.rs:283-290):

```rust
let result = if matches!(
    route.adapter,
    AdapterKind::Responses | AdapterKind::Cursor | AdapterKind::Gemini | AdapterKind::AntigravityCli
) { /* CountTokens::Tiktoken | CountTokens::Estimate */ } else { dispatch(...) };

match route.adapter {
    AdapterKind::Gemini => crate::adapters::gemini::GeminiAdapter
        .forward(state, route, uri, headers, body).await,
    ...
}
```

Add `OpenAiChat` to the translated-adapter list (research recommendation:
`CountTokens::Estimate`) and one dispatch arm. Credential stripping needs **no
adapter-side work**: `headers_for_route` already removes `authorization`/
`x-api-key` and applies `strip_consumed_slots` (failover.rs:620-641) before
dispatch; the adapter only adds its own `Authorization: Bearer <api_key_env>`.

## Shared Patterns

### Retry safety (D-09, CORRECTED)
**Source:** `src/retry.rs:137-160` (`RetrySafety`), `src/retry.rs:152`
(`CURSOR_RETRY_SAFETY` const pattern), `src/adapters/gemini/mod.rs:711`
(`retry_safety_for_auth`).
```rust
pub enum RetrySafety { Idempotent, NonIdempotentPost, ConnectOnly }
impl RetrySafety {
    pub(crate) fn may_retry_transport(self, connect_phase: bool) -> bool {
        match self {
            Self::ConnectOnly => connect_phase,
            Self::Idempotent | Self::NonIdempotentPost => true,
        }
    }
}
```
**Apply to:** the Chat send path. MUST be `ConnectOnly` — `NonIdempotentPost`
admits transient timeouts at `retry.rs:171-179` even post-send. A pre-header
failure is not necessarily pre-send; the adapter reports
`AdapterFailure::BeforeHeaders` only for genuine pre-send transport failures.
Pin the post-send timeout/no-fallback test.

### Redirect refusal (D-03, CORRECTED)
**Source:** `src/gateway/telemetry_ingest.rs:371-384`
**Apply to:** the Chat upstream client (disable redirects outright; do not rely
on implicit header stripping). Cover with a loopback 30x conformance test.

### Bounded SSE framing + fail-closed EOF
**Source:** `src/adapters/gemini/sse.rs` (`Decoder`: byte cap, `[DONE]`
dedup, residual/UTF-8 failures) + `src/model/gemini.rs`
(`transport_close_checked`).
**Apply to:** Chat SSE relay and both terminal paths; no EOF-tolerance opt-in.

### Ownership-based cancellation (D-08)
**Source:** `src/adapters/mod.rs:19-41` (`with_admission` moves the guard into
the body stream) + gemini `stream::unfold` relay owning stream/decoder/machine.
**Apply to:** Chat relay: dropping the response stream releases upstream and
admission; no registry, no detached tasks.

### Credential handling
**Source:** `src/proxy/failover.rs:620-641` (`strip_consumed_slots`),
`src/config.rs:1800-1818` (`AuthMode::ApiKey`).
**Apply to:** Chat adapter consumes already-stripped headers; injects exactly
one `Authorization: Bearer` from `api_key_env`; never filters or persists
credentials itself.

### Keepalive/deadlines
**Source:** `src/keepalive.rs:46` (`with_pings`), `src/upstream_timeout.rs`
(`wait` around `.send()`).
**Apply to:** Chat streaming relay wraps the byte stream with `with_pings`;
TTFB deadline wraps the send. Reuse, don't re-implement.

## No Analog Found

None. Every planned surface has a same-role, same-data-flow analog above.
The Chat *protocol grammar* has no in-repo analog (OpenCodex
`openai-chat.ts` is grammar provenance only); its policy divergences — orphan
repair, EOF tolerance, reasoning replay, permissive URL join — are locked out
by D-02/04/05/07 and must not be copied.

## Metadata

**Analog search scope:** `src/adapters/`, `src/model/`, `src/config.rs`,
`src/routing.rs`, `src/retry.rs`, `src/proxy/failover.rs`, `src/keepalive.rs`,
`src/gateway/telemetry_ingest.rs`, `tests/`
**Files scanned:** ~25 (targeted reads; symbols via rg)
**Tracked-path gate:** all named analogs pass `git ls-files`
**Pattern extraction date:** 2026-09-08
