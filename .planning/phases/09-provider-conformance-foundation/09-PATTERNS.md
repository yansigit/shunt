# Phase 9: Provider Conformance Foundation - Pattern Map

**Mapped:** 2026-09-06
**Files analyzed:** 22 proposed modified files
**Analogs found:** 22 / 22

## File Classification

| New/Modified File | Role | Data Flow | Closest Tracked Analog | Match Quality |
|---|---|---|---|---|
| `src/retry.rs` | retry policy service | request-response redispatch | `src/adapters/responses/websocket.rs::commit_or_fallback` | same policy flow |
| `src/proxy/failover.rs` | route orchestration service | request-response redispatch | current `AdapterFailure` match in the same file; `src/retry.rs::RetrySafety` | exact current seam |
| `src/adapters/mod.rs` | adapter failure and response-lifetime ownership | transport evidence + RAII admission | current `AdapterFailure` and `ResponseWithAdmission` in the same file | exact |
| `src/adapters/responses/websocket.rs` | provider transport adapter | streaming/fallback | current `open_ws_turn` + `commit_or_fallback` | exact |
| `src/adapters/responses/http.rs` | provider transport/parser | incremental streaming + bounded batch | `src/codex_endpoint/frame.rs::BoundedSseFrameBuffer` | exact flow |
| `src/model/responses.rs` | protocol state machine | event transform + terminal state | `src/codex_endpoint/frame.rs` terminal classification; current `AnthropicSseMachine` | role-match |
| `src/adapters/responses/ws_stream.rs` | provider response adapter | streaming + terminal conversion | `src/adapters/responses/http.rs::stream_response` | exact sibling |
| `src/adapters/responses/codex_ws.rs` | provider WebSocket/session store | streaming + bounded continuation | `src/adapters/responses/codex_continuation.rs::decide_with_signature` | same continuation flow |
| `src/adapters/responses/codex_continuation.rs` | continuation utility/store model | bounded transform + append-only validation | current `decide_with_signature` and transcript tests | exact |
| `src/model/responses_request.rs` | request translator/model | validation + transform | strict append-only/identity checks in `src/adapters/responses/codex_continuation.rs` | role-match |
| `src/auth/mod.rs` | credential model | redacted diagnostic formatting | structural credential assertions in `tests/passthrough.rs` | role-match |
| `src/concurrency.rs` | response-lifetime permit owner | downstream body polling + bounded release | current `PermitBody` and drop/error completion tests in the same file | exact |
| `tests/responses_translate.rs` | protocol integration test | request-response + streaming | parser boundary tests in `src/codex_endpoint/frame.rs` | flow-match |
| `tests/codex_websocket_fallback.rs` | real-gateway integration test | streaming/fallback | existing before/after-first-event pair in the same file | exact |
| `tests/failover.rs` | real-gateway integration test | request-response redispatch | existing truncated-body and credential-rebinding cases in the same file | exact |
| `tests/retry.rs` | real-gateway integration test | same-provider retry commitment | existing non-idempotent retry cases in the same file | exact |
| `tests/codex_multi_account.rs` | account-pool integration test | credential/account lease + safe replay | existing refresh, rotation, and byte-identical body tests in the same file | exact |
| `tests/inbound_anthropic_translation.rs` | translation integration test | tool pairing and cancellation | existing bounded translation and tool-result cases in the same file | exact |
| `tests/inbound_codex_endpoint.rs` | real-gateway integration test | request-response/streaming/error shape | existing `assert_openai_error_shape` helper | exact |
| `tests/inbound_codex_websocket.rs` | native Codex WebSocket integration test | bidirectional v2 frames, compression, cancellation, terminal | existing native WebSocket fixtures in the same file | exact |
| `tests/passthrough.rs` | real-gateway integration test | generic Anthropic/Vercel passthrough | existing `TestGateway`, header matchers, SSE/error cases | exact |
| `docs/m1-responses-translation.md` | engineering contract documentation | static documentation | existing upstream-event/robustness table in the same file | exact |

Every analog above passed the tracked-source gate with `git ls-files`. No `.gsd/`, plugin cache, generated `wiki/`, or runtime mirror is used as an analog.

## Pattern Assignments

### `src/retry.rs`, `src/proxy/failover.rs`, and `src/adapters/responses/websocket.rs`

**Roles:** internal monotonic commitment state, retry/failover policy, streaming fallback

**Primary analogs:** `src/retry.rs:102-115`, `src/adapters/mod.rs:45-59`, `src/proxy/failover.rs:161-223`, `src/adapters/responses/websocket.rs:119-185`, `src/lib.rs:1-40`

The commitment vocabulary should remain beside the existing retry policy, copying the small value-type style of `RetrySafety` while expressing semantic commitment rather than inventing a provider framework:

```rust
// src/retry.rs:102-115
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetrySafety {
    Idempotent,
    NonIdempotentPost,
}

impl RetrySafety {
    fn may_retry_response_status(self) -> bool {
        matches!(self, Self::Idempotent)
    }
}
```

Preserve `AdapterFailure` as transport/upstream evidence rather than conflating it with commitment:

```rust
// src/adapters/mod.rs:45-59
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdapterFailure {
    UpstreamStatus(StatusCode),
    BeforeHeaders,
}
```

Use the existing WebSocket gate as the behavioral baseline. It peeks without losing the first item, retries only a transport failure/empty stream before commitment, and treats a provider-sent error event as committed provider output:

```rust
// src/adapters/responses/websocket.rs:162-185
async fn peek_first_event(mut events: CodexWsEvents) -> (BufferedEvent, CodexWsEvents) {
    let first = events.recv().await;
    (first, events)
}

fn commit_or_fallback(
    first: BufferedEvent,
    events: CodexWsEvents,
) -> Result<(BufferedEvent, CodexWsEvents), AdapterError> {
    match first {
        Some(Ok(event)) => Ok((Some(Ok(event)), events)),
        Some(Err(error)) => Err(ws_transport_error(error)),
        None => Err(ws_before_headers_error(
            "codex websocket closed before any event".to_string(),
        )),
    }
}
```

The helper should be monotonic (`ReplaySafe` may advance to client-visible or replay-unsafe-tool commitment, never back) and expose one predicate such as `may_redispatch()`. Map existing facts into it without enabling retries that do not already occur. A tool activity transition must close redispatch even if no text was emitted. Keep this policy in the already planned modules; the final plans supersede the earlier optional `src/commitment.rs` and `src/lib.rs` proposal.

For route failover, retain the current decision shape at `src/proxy/failover.rs:161-223`: advance only on an configured pre-response status or `BeforeHeaders`; return all other failures immediately. Commitment is an additional gate, not a replacement for route/status capability checks.

**Tests to copy:** the table-like three-arm unit test at `src/adapters/responses/websocket.rs:484-530`, plus the real-gateway paired proof at `tests/codex_websocket_fallback.rs:529-624`. Add the replay-unsafe-tool arm beside the existing pre-first-event and post-output arms, asserting the HTTP/upstream hit counter remains zero after commitment.

---

### `src/adapters/responses/http.rs`, `src/model/responses.rs`, and `src/adapters/responses/ws_stream.rs`

**Roles:** byte parser, protocol state machine, HTTP/WS response adapters

**Primary analog:** `src/codex_endpoint/frame.rs::BoundedSseFrameBuffer` at lines 228-385

Copy the existing byte-first, fail-closed framing pattern rather than extending the lossy `String` parser:

```rust
// src/codex_endpoint/frame.rs:228-275
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SseFrameError {
    #[error("upstream SSE frame exceeded {0} bytes")]
    TooLarge(usize),
    #[error("upstream SSE chunk exceeded {0} frame limit")]
    CountLimit(usize),
}

pub struct BoundedSseFrameBuffer {
    max_frame_bytes: usize,
    max_frames_per_feed: usize,
    delimiter_tail: Vec<u8>,
    candidate: Vec<u8>,
    disposed: bool,
}

fn retain(&mut self, slice: &[u8]) -> Result<(), SseFrameError> {
    if self.candidate.len() + slice.len() > self.max_frame_bytes {
        let max = self.max_frame_bytes;
        self.clear();
        self.disposed = true;
        return Err(SseFrameError::TooLarge(max));
    }
    self.candidate.extend_from_slice(slice);
    Ok(())
}
```

Key details to preserve from `feed` (`src/codex_endpoint/frame.rs:277-363`): scan across arbitrary chunk boundaries, retain only a bounded candidate plus delimiter tail, enforce frames-per-feed before pushing, clear/dispose on overflow, and return a typed `Result`. The Responses parser also needs strict UTF-8 and JSON decoding; unlike current `parse_sse_events` at `src/model/responses.rs:866-887`, malformed data must not enter a `filter_map(... .ok())` discard path.

Make terminal state explicit in `AnthropicSseMachine`. The current safety violation is visible at `src/model/responses.rs:211-233`, where `finish()` manufactures `end_turn` and `final_json()` invokes it. Preserve the existing genuine terminal mapping at `src/model/responses.rs:175-197`, but separate “provider terminal observed” from EOF cleanup. `response.completed`, `response.done`, and `response.incomplete` are trusted success terminals; `error` and `response.failed` are trusted failure terminals. Bare EOF, malformed input, duplicate terminal, or semantic data after terminal must yield one error outcome and never a success terminal.

Keep the streaming implementation incremental, following `src/adapters/responses/http.rs:128-197`: consume `bytes_stream()`, feed parser state, apply events, and return chunks as soon as non-empty translated data exists. Do not collect the streaming success body. For client-requested JSON, replace `Response::text()` at `src/adapters/responses/http.rs:207-222` with an incremental bounded collector and the same parser/state machine.

Apply the same terminal rule to `src/adapters/responses/ws_stream.rs`: a channel close is success only if the machine has already observed the authoritative terminal. Do not repair a bare close with `final_json()` or synthetic `message_stop`.

**Boundary-test analog:** `src/codex_endpoint/frame.rs:587-690` covers split chunks, split delimiters, mixed CRLF, exact cap, cap-plus-one, frame amplification, terminal-before-overflow, and terminal classification. Reuse that below/exact/plus-one structure for residual bytes, event bytes, events per feed, bounded JSON collection, aggregate tool arguments, malformed UTF-8/JSON, duplicate terminal, and premature EOF.

---

### `src/adapters/responses/codex_ws.rs`, `src/adapters/responses/codex_continuation.rs`, and `src/model/responses_request.rs`

**Roles:** WebSocket response/session owner, continuation store utility, request translator

**Primary analog:** `src/adapters/responses/codex_continuation.rs:89-147` and its tests at lines 435-602

Preserve the existing fail-closed append-only decision:

```rust
// src/adapters/responses/codex_continuation.rs:110-137
pub fn decide_with_signature(
    stored: &StoredContinuation,
    current_body: &Value,
    current_signature: &str,
) -> Option<Decision> {
    if current_signature != stored.signature {
        return None;
    }
    let input = current_body.get("input").and_then(Value::as_array)?;
    if stored.transcript.len() > input.len() {
        return None;
    }
    for (previous, current) in stored.transcript.iter().zip(input.iter()) {
        if normalize_item(previous) != normalize_item(current) {
            return None;
        }
    }
    let delta = input[stored.transcript.len()..].to_vec();
    if delta.is_empty() {
        return None;
    }
    Some(Decision {
        previous_response_id: stored.response_id.clone(),
        input_delta: delta,
    })
}
```

Refactor `build_transcript` (`src/adapters/responses/codex_continuation.rs:139-147`) into a fallible bounded operation, or account before calling it. Bound both item count and serialized/retained bytes, including opaque response ID and turn-state metadata. On overflow, preserve a genuinely completed current response but discard the reusable continuation atomically; never truncate metadata and store the result as authentic.

Follow the existing tests' structural fidelity style. `continues_across_a_tool_turn` at `src/adapters/responses/codex_continuation.rs:435-459` proves exact reasoning → tool call → tool result adjacency and the exact delta. The fallback tests at lines 481-539 prove changed non-input fields, divergent prefix, empty delta, and shorter history all force full input. Extend those tables with item/byte/metadata exact-cap and cap-plus-one cases.

In request/response translation, require non-empty authentic tool call ID and name and a non-empty authentic reasoning ID before preserving continuation metadata. Return a typed translation/protocol error for missing identity or malformed arguments. Do not copy the current synthetic-ID or `{}` fallback behavior identified in research.

For `previous_response_not_found`, retain `src/adapters/responses/websocket.rs:124-157`: exactly one retry with full input, then feed the replacement stream into the ordinary commitment gate. Add a structural body matcher proving the recovered body retains the serialized tool call/result pair, session identity, and opaque continuation bytes exactly once.

---

### `src/auth/mod.rs`, `tests/passthrough.rs`, and `tests/failover.rs`

**Roles:** secret-bearing model, generic provider conformance integration, cross-route credential integration

**Primary analogs:** `tests/passthrough.rs:16-84`, `tests/passthrough.rs:580-681`, `tests/failover.rs:922-1035`

Implement a custom redacted `Debug` for `Credential` (or remove `Debug` if all callers compile without it), while preserving `Clone`, `PartialEq`, and `Eq`. Current `#[derive(Debug, Clone, PartialEq, Eq)]` at `src/auth/mod.rs:29-65` exposes every secret field. The diagnostic should reveal only safe structural information such as variant and API-key header kind; never token, account ID, project ID, device ID, or private source content. Add a split synthetic marker and assert the formatted value does not contain either the assembled marker or its distinctive fragments.

Reuse the local matcher and real-router harness rather than adding infrastructure:

```rust
// tests/passthrough.rs:16-41
struct ExactHeader(&'static str, &'static str);
impl Match for ExactHeader {
    fn matches(&self, request: &Request) -> bool {
        request.headers.get(self.0).and_then(|value| value.to_str().ok())
            == Some(self.1)
    }
}

struct HeaderAbsent(&'static str);
impl Match for HeaderAbsent {
    fn matches(&self, request: &Request) -> bool {
        !request.headers.contains_key(self.0)
    }
}
```

`TestGateway` at `tests/passthrough.rs:51-84` binds `127.0.0.1:0`, builds the real Axum router, and aborts the server task in `Drop`. Use it for named Vercel characterization by configuring an ordinary `kind = "anthropic"` provider at the mock origin. Keep Vercel absent from presets and adapter dispatch; this is a test name/config fixture only, not a new provider semantic.

Copy the SSE and provider-error cases at `tests/passthrough.rs:614-681`: use `set_body_raw(..., "text/event-stream")`, assert content type and byte fidelity, and assert relayed status/body. Add `retry-after`, incremental first-chunk-before-terminal, client-drop cancellation, missing-key/zero-upstream-hit, bearer vs `x_api_key`, and inbound-slot absence cases.

For cross-origin rebinding, copy `tests/failover.rs:922-1035`: distinct mock origins, per-attempt exact/absent header matchers, route-selected synthetic env credentials, expected hit counts, and gateway provenance headers. Avoid recognizable secret literals in any new fixture; build markers from harmless fragments. Do not change credential discovery, refresh, migration, persistence, URL policy, or response-header passthrough.

---

### `src/concurrency.rs`, adapter admission ownership, and cancellation integration tests

**Roles:** middleware/body owner and response-lifetime resource tests

**Primary analogs:** `src/concurrency.rs:111-175`, `src/adapters/mod.rs:16-40`, `src/concurrency.rs:727-815`

Keep resources attached to the response body through ownership:

```rust
// src/concurrency.rs:131-165
impl HttpBody for PermitBody {
    type Data = Bytes;
    type Error = axum::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let frame = Pin::new(&mut self.inner).poll_frame(cx);
        let ended = match &frame {
            Poll::Ready(None) | Poll::Ready(Some(Err(_))) => true,
            Poll::Ready(Some(Ok(_))) => self.inner.is_end_stream(),
            Poll::Pending => false,
        };
        if ended {
            self.permit.take();
        }
        frame
    }
}
```

Provider admission uses the same ownership principle at `src/adapters/mod.rs:16-40`: move the guard into the body stream closure so exhaustion, error, or downstream drop releases it. Do not add cleanup booleans or durable cancellation history.

Copy the observable proof at `src/concurrency.rs:727-781`: consume one chunk, verify a second request is rejected while the body is pending, drop the body, then verify capacity is reacquired. The real-gateway variant should also observe upstream disconnect/cancellation within `tokio::time::timeout`; it must not infer cancellation from a sleep alone.

---

### `tests/failover.rs`, `tests/responses_translate.rs`, `tests/inbound_codex_endpoint.rs`, and `tests/codex_websocket_fallback.rs`

**Roles:** preservation and security integration tests

**Primary analogs:** existing tests in the same suites

Use the raw truncated-server fixture at `tests/failover.rs:155-177` when Wiremock cannot model declared-length mismatch. The existing cases at `tests/failover.rs:857-920` already prove a truncated 2xx body does not replay onto a second route; extend their response assertions to require failure rather than synthesized clean completion.

Use `tests/codex_websocket_fallback.rs:529-624` as the canonical paired commitment test: before-output failure falls back exactly once; after-output failure returns a stream error and records zero HTTP hits. Add the tool-activity boundary with the same hit-counter proof.

Use `assert_openai_error_shape` at `tests/inbound_codex_endpoint.rs:2002-2027` for Codex ingress. Its key detail is checking that top-level `type` is absent and that `error.code` is present and null, not merely relying on `Value` indexing. Non-Codex gateway errors must retain the Anthropic envelope.

Keep compaction, collaboration, multi-account, and inbound/outbound WebSocket suites as regression sweep evidence; do not duplicate already-covered happy paths in a new umbrella test binary. Add cases beside the owning surface.

---

### `docs/m1-responses-translation.md`

**Role:** internal engineering contract

**Analog:** the existing event table and robustness prose at lines 101-130

Update only the now-inaccurate robustness contract at lines 120-123 after the failing tests and production fix land: EOF without `response.completed`/`response.done`/`response.incomplete` is an upstream protocol error and must not synthesize `message_stop` or a successful JSON message. Keep unknown well-formed event types ignored if that remains the intended compatibility behavior; distinguish them from malformed frames/events.

No README or site support statement should change in Phase 9 unless implementation reveals actual drift in existing behavior. Record the required surface review in the execution summary. Do not edit generated `wiki/`, add translated provider claims, or imply Google AI Studio Web support.

## Shared Patterns

### Strict, byte-bounded parsing

**Source:** `src/codex_endpoint/frame.rs:228-385`

**Apply to:** Responses HTTP SSE residuals/events, client-requested JSON accumulation, tool-argument assembly, WebSocket frames where the existing transport cap does not already cover aggregate state.

- Operate on bytes until a complete frame exists.
- Use named internal limits and checked/saturating accounting.
- Test below-limit, exact-limit, and limit-plus-one.
- Clear/dispose state on overflow; never retain and retry attacker-controlled residuals.
- Treat malformed UTF-8/JSON as typed protocol failure.

### Exactly one trustworthy terminal

**Source:** genuine terminal dispatch at `src/model/responses.rs:175-197`; terminal classifier tests at `src/codex_endpoint/frame.rs:671-690`

**Apply to:** HTTP streaming, HTTP JSON, outbound WebSocket streaming/JSON, continuation capture.

Terminal state must be monotonic and authoritative. Success is legal only after a provider success terminal; explicit provider failure is terminal failure; duplicate terminal, post-terminal semantic input, malformed input, and premature EOF fail closed. Validate before emitting a clean terminal because an error emitted after `message_stop` cannot repair the contract.

### Semantic commitment before redispatch

**Source:** `src/adapters/responses/websocket.rs:119-185`; `tests/codex_websocket_fallback.rs:529-624`

**Apply to:** same-provider retry, route failover, WS-to-HTTP fallback, and one-shot continuation recovery. Preserve the current retry set. Client-visible output and replay-unsafe tool activity close the boundary; metadata-only transport activity does not necessarily close it.

### Credential isolation and diagnostic redaction

**Source:** `tests/passthrough.rs:16-41`; `tests/failover.rs:922-1035`

**Apply to:** generic Anthropic/Vercel characterization, failover, and `Credential` formatting. Assert exact selected outbound credentials and absence of both inbound shared slots. Use split, synthetic, non-recognizable markers and never print captured headers or environment values.

### Response-lifetime ownership

**Source:** `src/concurrency.rs:111-175`; `src/adapters/mod.rs:16-40`

**Apply to:** gateway permits, account admission guards, parser/continuation state, and cancellation tests. Keep cleanup in ordinary Rust ownership/`Drop`; tests prove release by reacquiring capacity after downstream cancellation.

### Real-gateway hermetic integration

**Source:** `tests/passthrough.rs:51-84`; `tests/failover.rs:135-209`

**Apply to:** every observable provider boundary. Bind an ephemeral loopback listener, build the real router from an in-memory config, mock only external upstreams, track exact hit counts/headers/bodies, and make the test self-cleaning. No new framework or dependency is needed.

## No Analog Found

None. The exact combined `Commitment` concept is new, but its value-type form, fallback semantics, and integration proof all have tracked local analogs. The planner should compose those narrow patterns rather than introduce a generalized provider abstraction.

## Scope Guardrails for the Planner

- Do not modify Gemini, Antigravity, Cursor, generic Chat Completions, Command Code, or OpenCode Go behavior in Phase 9.
- Do not add a Vercel provider kind, preset, adapter, hostname allowlist, or new documented semantic.
- Do not add Google AI Studio Web implementation, fixtures, tests, browser/cookie/SAPISIDHASH auth, dependencies, or support wording.
- Do not change credential discovery, refresh, migration, persistence, or writeback.
- Do not strip new classes of upstream response headers as an incidental hardening change.
- Do not add durable history, synthetic thought signatures/tool IDs, or cryptographic continuation infrastructure.
- Production changes require a failing Phase 9 regression that demonstrates the concrete invariant violation.

## Metadata

**Analog search scope:** `src/`, `tests/`, `docs/`, and Phase 9 planning inputs

**Tracked analogs read:** `src/lib.rs`, `src/retry.rs`, `src/adapters/mod.rs`, `src/proxy/failover.rs`, `src/adapters/responses/http.rs`, `src/adapters/responses/websocket.rs`, `src/model/responses.rs`, `src/codex_endpoint/frame.rs`, `src/adapters/responses/codex_continuation.rs`, `src/auth/mod.rs`, `src/concurrency.rs`, `tests/passthrough.rs`, `tests/failover.rs`, `tests/codex_websocket_fallback.rs`, `tests/inbound_codex_endpoint.rs`, `docs/m1-responses-translation.md`

**Strong analog set:** 5 pattern families (bounded SSE framing, commitment/fallback, continuation fidelity, response-lifetime RAII, real-gateway Wiremock integration)

**Pattern extraction date:** 2026-09-06
