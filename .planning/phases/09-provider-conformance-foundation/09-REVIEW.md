---
phase: 09-provider-conformance-foundation
reviewed: 2026-09-06T22:19:38Z
depth: standard
files_reviewed: 30
files_reviewed_list:
  - docs/m1-responses-translation.md
  - site/src/content/docs/ja/reference/configuration.md
  - site/src/content/docs/ja/reference/troubleshooting.md
  - site/src/content/docs/ko/reference/configuration.md
  - site/src/content/docs/ko/reference/troubleshooting.md
  - site/src/content/docs/reference/configuration.md
  - site/src/content/docs/reference/troubleshooting.md
  - site/src/content/docs/zh-cn/reference/configuration.md
  - site/src/content/docs/zh-cn/reference/troubleshooting.md
  - src/adapters/mod.rs
  - src/adapters/responses/codex_continuation.rs
  - src/adapters/responses/codex_ws.rs
  - src/adapters/responses/http.rs
  - src/adapters/responses/mod.rs
  - src/adapters/responses/websocket.rs
  - src/adapters/responses/ws_stream.rs
  - src/auth/mod.rs
  - src/concurrency.rs
  - src/model/responses.rs
  - src/model/responses_request.rs
  - src/proxy/failover.rs
  - src/retry.rs
  - tests/codex_multi_account.rs
  - tests/codex_websocket_fallback.rs
  - tests/failover.rs
  - tests/inbound_codex_endpoint.rs
  - tests/inbound_codex_websocket.rs
  - tests/passthrough.rs
  - tests/responses_translate.rs
  - tests/retry.rs
findings:
  critical: 5
  warning: 2
  info: 0
  total: 7
status: issues_found
---

# Phase 9: Code Review Report

**Reviewed:** 2026-09-06T22:19:38Z
**Depth:** standard
**Files Reviewed:** 30
**Status:** issues_found

## Narrative Findings (AI reviewer)

## Summary

The Phase 9 diff was reconciled against all four summaries and reviewed across the Responses HTTP/WebSocket transports, terminal state machine, continuation capture, redispatch gates, credential redaction, cancellation tests, and English plus maintained localized documentation. Credential diagnostics and response-lifetime ownership are sound, and no Google AI Studio Web or credential-writeback surface was introduced. However, the new fail-closed contract still has five correctness/resource-safety defects and two incomplete boundary integrations. The green suite does not cover these paths.

## Critical Issues

### CR-01: HTTP streaming does not stop after an authoritative provider failure

**File:** `/Users/user/.codex/worktrees/0466/shunt/src/adapters/responses/http.rs:165-179`
**Issue:** `AnthropicSseMachine::apply_checked` emits the mapped `error` frame and enters `ProviderFailed` for `error`/`response.failed`, but this loop only sets `finished` when `apply_checked` returns `Err`. If the upstream holds the body open, Shunt keeps the request and its resources alive after the terminal failure. If the upstream closes normally, the EOF branch calls `finish_checked`, receives a second provider-failure error, and emits a second terminal error. This violates SAFE-03 and the documented exactly-one-terminal contract. The WebSocket path already handles the same case correctly with `machine.has_backend_error()`.
**Fix:** After each successful `apply_checked`, detect `machine.has_backend_error()`, set `finished = true`, and stop consuming events. Add HTTP streaming regressions for provider-failure followed by EOF and provider-failure followed by a body that remains pending; each must emit one error and release the upstream body.

### CR-02: An unterminated SSE terminal at EOF is accepted as clean success

**File:** `/Users/user/.codex/worktrees/0466/shunt/src/adapters/responses/http.rs:371-388`
**Issue:** `SseParser::finish` parses any nonempty residual buffer as a complete event. A transport cut immediately after `event: response.completed\ndata: {}` without the required blank-line delimiter is therefore promoted into a trustworthy terminal and produces `message_stop` or successful JSON. The partial frame was never dispatched by SSE framing and must not convert premature EOF into success.
**Fix:** Make EOF with a nonempty unterminated candidate return a typed premature-frame protocol error (control-only whitespace can be handled explicitly if needed). Add streaming and non-streaming regressions for an unterminated success terminal and assert no clean terminal/message is produced.

### CR-03: Continuation capture applies the byte cap per item, not to retained aggregate state

**File:** `/Users/user/.codex/worktrees/0466/shunt/src/adapters/responses/codex_ws.rs:1145-1155`
**Issue:** Each `response.output_item.done` is checked independently against `transcript_bytes`, then cloned into `output_items`. Up to 10,000 individually valid large items can therefore be retained before `into_stored` finally computes the aggregate and rejects it. This permits memory growth orders of magnitude beyond the claimed 32 MiB continuation bound during an active response, defeating SAFE-06 and the summary's “atomically bounded continuation capture” guarantee.
**Fix:** Track cumulative serialized output bytes, including JSON-array separators/overhead, with checked arithmetic before cloning each item. Discard the whole candidate as soon as the next item crosses the aggregate budget. Test several individually-under-limit items whose sum is exact-limit and limit-plus-one before terminal capture.

### CR-04: Malformed outbound Codex WebSocket events are silently skipped

**File:** `/Users/user/.codex/worktrees/0466/shunt/src/adapters/responses/codex_ws.rs:969-975`
**Issue:** Non-JSON and typeless text frames are logged and ignored. A malformed provider frame can therefore be followed by `response.completed`, yielding a clean client completion even though part of the upstream transcript was discarded. This contradicts SAFE-01/D-06 and the engineering contract that distinguishes ignorable unknown well-formed events from malformed events.
**Fix:** Make parsing distinguish a valid unknown event from malformed JSON/required envelope data. Continue ignoring valid unknown event types, but forward a typed protocol error and terminate/evict the turn for malformed frames. Add a real WebSocket regression with malformed JSON followed by a success terminal and assert one error and no clean completion.

### CR-05: `response.done` is accepted by the translator but never terminates a WebSocket turn

**File:** `/Users/user/.codex/worktrees/0466/shunt/src/adapters/responses/codex_ws.rs:106-117`
**Issue:** The shared Responses machine treats `response.done` as an authoritative success terminal, and continuation capture recognizes its response ID, but `TERMINAL_EVENTS` omits it. On WebSocket transport the reader forwards `response.done` and then waits for another frame until EOF/idle timeout, converting a valid terminal into a transport failure. This is inconsistent with the Phase 9 terminal contract and HTTP behavior.
**Fix:** Include `response.done` in the WebSocket terminal classification and decide explicitly whether it is reusable or clean-but-evicted. Add an outbound WebSocket test proving `response.done` closes the client response exactly once without waiting for timeout.

## Warnings

### WR-01: The shared `Commitment` gates are vacuous on production retry and failover paths

**File:** `/Users/user/.codex/worktrees/0466/shunt/src/proxy/failover.rs:121`
**Issue:** `failover::forward` initializes a request-local `Commitment::ReplaySafe`; every mutation occurs only in a branch that immediately returns or after the loop, so every advancement check observes `ReplaySafe`. Likewise, `send_with_retry_with_safety` always passes `Commitment::default()` at `/Users/user/.codex/worktrees/0466/shunt/src/retry.rs:208`, while committed values are supplied only by private unit tests. Existing behavior remains conservative through older `AdapterFailure` classifications, but Phase 9 has not actually propagated shared commitment evidence to the redispatch decisions it claims to gate.
**Fix:** Carry commitment evidence in each adapter/transport attempt outcome and update request-level state before evaluating retry/failover, then add a production-path test combining an otherwise advance-eligible failure with committed evidence and assert zero second dispatch. If same-provider retry is intentionally always pre-header, remove its unreachable commitment parameter and narrow the claim.

### WR-02: Provider response IDs bypass translated-state accounting

**File:** `/Users/user/.codex/worktrees/0466/shunt/src/model/responses.rs:402-410`
**Issue:** `start` copies the provider response ID into retained machine state, but `reserve_retained_event` at lines 1012-1045 charges zero bytes for `response.created`/`response.in_progress`. HTTP permits an uncounted ID up to the event cap, while the outbound WebSocket transport can decode a message up to its 64 MiB ceiling, allowing retained state to exceed the advertised 32 MiB aggregate limit.
**Fix:** Account the selected response ID before copying it, using the same checked aggregate path, and add exact-limit/plus-one tests for both created and in-progress response shells.

---

_Reviewed: 2026-09-06T22:19:38Z_
_Reviewer: Codex (gsd-code-reviewer)_
_Depth: standard_
