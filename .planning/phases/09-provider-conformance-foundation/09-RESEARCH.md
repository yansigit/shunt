# Phase 9: Provider Conformance Foundation — Research

**Researched:** 2026-09-06  
**Domain:** Rust HTTP/SSE/WebSocket provider conformance, replay safety, bounded state, credential isolation, and cancellation  
**Confidence:** High for the existing code and test inventory; medium for exact internal cap values, which the implementation plan must justify without adding public configuration

## User Constraints

### Regression Surface
- **D-01:** Treat current ChatGPT/Codex HTTP, SSE, outbound WebSocket v2, compaction, collaboration, authentication, cancellation, and error-shape behavior as the compatibility baseline; strengthen tests without intentionally changing those semantics.
- **D-02:** Cover the existing Vercel route through its Anthropic-compatible path and configuration behavior; do not introduce a Vercel-specific adapter.
- **D-03:** Prefer focused hermetic integration fixtures through the real Axum gateway and mock upstreams, reusing existing helpers and fixtures rather than adding a new test framework or runtime dependency.
- **D-04:** Keep source/provider provenance in test or planning documentation where evidence is translated, but never place credentials, account identifiers, project identifiers, or private content in fixtures.

### Commitment and Stream Safety
- **D-05:** Use one internal, testable commitment vocabulary for whether redispatch remains replay-safe: client-visible output and replay-unsafe tool activity close the retry/failover boundary.
- **D-06:** A stream succeeds only after one trustworthy terminal outcome; malformed input, explicit provider error, duplicate terminal, and premature EOF must never be normalized into clean completion.
- **D-07:** Streaming paths remain incremental. Any non-streaming accumulation, decompressed frame, parser residual, event, replay state, queue, retry budget, and timeout must have an explicit bound exercised by tests.
- **D-08:** Cancellation tests must prove response-lifetime resources are released using existing RAII ownership; do not add durable history or persistence to demonstrate cleanup.

### Credential and Continuation Integrity
- **D-09:** Assert that outbound credentials are selected by provider kind and bound to the approved destination, while inbound credential slots are stripped wherever the gateway owns upstream authentication.
- **D-10:** Diagnostics and fixtures must use structural/redaction assertions rather than recognizable bearer or API-key values; response-lifetime ownership ends on completion or cancellation.
- **D-11:** Safe replay preserves tool-call/result adjacency and authentic continuation metadata byte-for-byte where the existing path treats it as opaque; never synthesize thought signatures or duplicate tool execution.
- **D-12:** Credential discovery and persistence semantics remain unchanged. This phase must not create, refresh, migrate, or write credential files beyond existing behavior.

### Scope and Delivery
- **D-13:** Favor small provider-specific conformance modules plus shared test helpers only when duplication proves a stable invariant; avoid a speculative generalized provider framework.
- **D-14:** Production changes are permitted only when a new regression test exposes a concrete violation of a Phase 9 requirement; otherwise this phase is test-foundation and documentation of internal invariants.
- **D-15:** Google AI Studio Web remains explicitly excluded: add no implementation, fixture, support claim, authentication path, browser dependency, or test suggesting support.
- **D-16:** Defer Gemini/Antigravity, Cursor, generic Chat Completions, Command Code, and exact-model OpenCode Go behavior changes to their owning later phases.

### Claude's Discretion
- Exact test-file decomposition, helper naming, and whether an invariant is best proven at unit or gateway-integration level are implementation details, provided the real observable boundary is exercised and the project remains dependency-neutral.

### Deferred Ideas

- Provider-specific semantic hardening belongs to Phases 10-15.
- Full release provenance, live smoke orchestration, translated documentation parity, and release-wide negative-scope checks belong to Phase 16.
- Google AI Studio Web and any credential-file writeback change remain outside the milestone.

## Summary

Phase 9 should be planned as a preservation suite plus a small number of test-driven repairs, not as a provider framework. The existing code already has strong real-gateway coverage, bounded request bodies, conservative response-status retry behavior, response-body-owned permits, account RAII, strict inbound Codex WebSocket framing, and provider-specific credential injection. [VERIFIED: `tests/inbound_codex_endpoint.rs:311-2072`; `tests/inbound_codex_websocket.rs:290-810`; `src/http_tuning.rs:99-160`; `src/concurrency.rs:36-175`; `src/adapters/mod.rs:16-40`; `src/proxy/failover.rs:102-228`]

The critical gaps are concrete and testable. Translated Responses HTTP and a defensive WebSocket relay path can turn premature EOF into a clean Anthropic terminal; malformed Responses events can be silently discarded; non-streaming Responses bodies, SSE residuals/event batches, and outbound WebSocket continuation state have no aggregate bound. [VERIFIED: `src/adapters/responses/http.rs:128-223`; `src/adapters/responses/http.rs:231-267`; `src/adapters/responses/ws_stream.rs:66-143`; `src/model/responses.rs:211-233`; `src/model/responses.rs:866-888`; `src/adapters/responses/codex_ws.rs:922-1133`; `src/adapters/responses/codex_continuation.rs:139-147`]

Retry safety is conservative today but expressed through several unrelated concepts: `AdapterFailure`, `RetrySafety`, and “first successful WebSocket event.” There is no single state that names client-visible commitment separately from replay-unsafe tool activity. [VERIFIED: `src/adapters/mod.rs:45-60`; `src/retry.rs:102-115`; `src/proxy/failover.rs:161-228`; `src/adapters/responses/websocket.rs:119-185`]

Vercel is not a missing provider implementation. It is intentionally configured as a generic Anthropic-compatible upstream, has no preset, and should receive named characterization coverage rather than a new adapter. [VERIFIED: `src/config.rs:1763`; `src/config/presets.rs:19-79`; `site/src/content/docs/providers/vercel-ai-gateway.md:6-39`]

The plan should therefore deliver four things in order: (1) executable preservation and commitment tests, (2) bounded/strict Responses parsing and terminal handling, (3) continuation/tool/cancellation hardening proven at gateway boundaries, and (4) named Vercel credential/stream/error characterization. Later-provider defects discovered in Gemini, Antigravity, and Cursor are inputs to their owning phases, not Phase 9 implementation work. [VERIFIED: `.planning/phases/09-provider-conformance-foundation/09-CONTEXT.md:43-65`; `.planning/ROADMAP.md:100-142`]

## Requirement Interpretation

| Requirement | What Phase 9 must prove |
|---|---|
| PRES-01 | Characterize native Responses HTTP/SSE/outbound WS v2, compression, continuation, and cancellation; change only terminal/malformed/bounds behavior that directly violates D-06/D-07. |
| PRES-02 | Keep the existing account selection, quota, compaction, and collaboration suites green and add only missing boundary cases. |
| PRES-03 | Show pre-commit redispatch succeeds and post-output/post-tool redispatch is impossible. |
| PRES-04 | Pin OpenAI Responses-shaped gateway errors on Codex ingress and Anthropic-shaped gateway errors elsewhere. |
| PRES-05 | Exercise a manual `kind = "anthropic"` Vercel configuration for request, stream, auth, cancellation, and relayed provider errors. |
| SAFE-01/02 | Name and test caps for all state Phase 9 touches; never collect a streaming success body in full. |
| SAFE-03 | Require exactly one trustworthy success terminal; malformed/error/duplicate/cut streams fail closed. |
| SAFE-04 | Make one monotonic commitment vocabulary govern all redispatch decisions exercised in this phase. |
| SAFE-05 | Prove credential selection/stripping/origin binding/redaction/lifetime without credential writeback changes. |
| SAFE-06 | Preserve authentic tool/result IDs, ordering, session, and opaque continuation metadata; reject invention and unsafe recovery. |
| SAFE-07 | Prove response drop cancels upstream work and releases all body-owned/account resources within a test timeout. |

These interpretations come directly from the locked Phase 9 decisions and assigned requirements. [VERIFIED: `.planning/phases/09-provider-conformance-foundation/09-CONTEXT.md:10-65`; `.planning/REQUIREMENTS.md:15-20`; `.planning/REQUIREMENTS.md:85-92`]

## Architectural Responsibility Map

| Concern | Current owner | Phase 9 responsibility |
|---|---|---|
| Route failover | `src/proxy/failover.rs` | Consult the shared commitment decision; retain local-error/status behavior and rebuild headers per attempt. |
| Same-provider HTTP retry | `src/retry.rs` | Map the current idempotency/transport rules onto the same commitment vocabulary without enabling new retries. |
| WS-to-HTTP fallback | `src/adapters/responses/websocket.rs` | Use the same commitment boundary; distinguish replay-unsafe tools from merely having received a transport item. |
| Responses HTTP parsing | `src/adapters/responses/http.rs`, `src/model/responses.rs` | Bound residual/event/body/assembly state and reject malformed, duplicate-terminal, and premature-EOF transcripts. |
| Responses outbound WS | `src/adapters/responses/codex_ws.rs`, `ws_stream.rs` | Preserve operational caps, bound continuation state, reject malformed/bare-close success, and clear state on failure/cancel. |
| Continuation safety | `src/adapters/responses/codex_continuation.rs`, `src/model/responses_request.rs` | Preserve exact IDs/adjacency/opaque metadata on safe replay; reject missing/invented identity. |
| Provider credentials | `src/auth/`, `src/auth/slots.rs`, `src/proxy/failover.rs`, provider adapter header builders | Preserve discovery/writeback behavior, strip reserved/shared inbound slots, inject only route-selected credentials, and redact diagnostic formatting. |
| Response lifetime | `src/concurrency.rs`, `src/adapters/mod.rs`, `src/accounts.rs`, `src/stream_metrics.rs` | Reuse the existing RAII ownership and add one end-to-end cancellation proof. |
| Vercel behavior | Generic Anthropic adapter and manual provider config | Add named conformance cases; no preset, provider kind, or adapter. |
| Public error shape | `src/error.rs`, `src/codex_endpoint.rs`, concurrency/error middleware | Preserve ingress-dependent shape with cross-surface tests. |

Current ownership in the table is verified by the listed implementation modules. [VERIFIED: `src/proxy/failover.rs:24-266`; `src/retry.rs:53-260`; `src/adapters/responses/websocket.rs:119-219`; `src/auth/slots.rs:1-175`; `src/concurrency.rs:36-175`; `src/adapters/anthropic/mod.rs:1011-1071`]

## Standard Stack

No new runtime or development dependency is needed. Axum supplies the real gateway, Tokio supplies bounded channels/timeouts, Reqwest supplies streaming upstream bodies, Bytes supports incremental framing, Serde handles typed events, and Wiremock supplies hermetic external-provider endpoints. [VERIFIED: `Cargo.toml:1-120`; `tests/passthrough.rs:1-80`; `tests/inbound_codex_endpoint.rs:1-80`]

The relevant declared versions are Axum 0.8, Tokio 1, Reqwest 0.12, Bytes 1, Serde/Serde JSON 1, and Wiremock 0.6. The inspected environment provides Cargo 1.98.0 and Rustc 1.98.0. [VERIFIED: `Cargo.toml:25-118`; environment probe `cargo --version`; environment probe `rustc --version`]

Use the existing test patterns:

- Real Axum listeners on ephemeral loopback addresses, with only provider endpoints mocked. [VERIFIED: `tests/passthrough.rs:29-80`; `tests/inbound_codex_endpoint.rs:35-90`]
- Delayed/chunked mock responders to prove incremental delivery, truncation, timeout, and cancellation. [VERIFIED: `tests/passthrough.rs:614-705`; `tests/inbound_codex_endpoint.rs:946-1036`]
- Existing `TestGateway`, temporary credential stores, header/body matchers, and RAII guards rather than a new harness. [VERIFIED: `.planning/phases/09-provider-conformance-foundation/09-CONTEXT.md:72-89`]
- Unit tests for parser/terminal/commitment state transitions, then integration tests for observable gateway behavior. This division is recommended because the state-space is cheaper to exhaust at unit level while provider dispatch, headers, error shapes, and resource ownership require the real router.

## Package Legitimacy Audit

**Result: no installs and no dependency changes are appropriate.** The locked decisions require a dependency-neutral phase, and the repository already includes all primitives required for bounded parsing, streaming, timeouts, real-gateway tests, and mock upstreams. [VERIFIED: `.planning/phases/09-provider-conformance-foundation/09-CONTEXT.md:21-24`; `Cargo.toml:1-120`; `.planning/REQUIREMENTS.md:119-119`]

Because no package is proposed, there is no package-name, publisher, maintenance, version, or source-repository legitimacy question to resolve. Adding an SSE/parser framework here would increase attack surface and create a second protocol abstraction beside existing bounded frame code. [VERIFIED: `src/codex_endpoint/frame.rs:228-385`; `src/adapters/anthropic/inbound.rs:131-205`]

## Don't Hand-Roll

- Do not build a second gateway harness; use the existing Axum-plus-Wiremock fixtures. [VERIFIED: `.planning/phases/09-provider-conformance-foundation/09-CONTEXT.md:72-89`]
- Do not invent another SSE grammar. Reuse or extract the repository's existing byte-oriented bounded frame-buffer pattern, adding only the terminal/event semantics required by Responses. [VERIFIED: `src/codex_endpoint/frame.rs:228-385`; `src/adapters/anthropic/inbound.rs:131-205`]
- Do not implement manual permit/account cleanup flags. Keep cleanup attached to response-body and guard `Drop` ownership. [VERIFIED: `src/concurrency.rs:36-175`; `src/adapters/mod.rs:16-40`; `src/accounts.rs:1641-1652`]
- Do not add a Vercel adapter, preset, hostname catalogue, or credential store; its existing supported surface is generic Anthropic configuration. [VERIFIED: `src/config.rs:1763`; `src/config/presets.rs:19-79`; `site/src/content/docs/providers/vercel-ai-gateway.md:6-39`]
- Do not create signature, tool ID, or continuation data to repair incomplete upstream input. Fail closed or fall back to a full-input request using only authentic retained bytes. [VERIFIED: `.planning/phases/09-provider-conformance-foundation/09-CONTEXT.md:39-47`]
- Do not add durable cancellation/history state or attempt to abort non-cooperative `spawn_blocking` work with custom machinery. Bound its input/lifetime impact and prove response-owned resources release independently. [VERIFIED: `src/offload.rs:19-38`; `src/proxy/failover.rs:293-304`]

## Existing Strengths to Preserve

### Retry and failover

- Request bodies are bounded before failover and the same bounded bytes are cloned for each permitted attempt. [VERIFIED: `src/proxy/failover.rs:31-48`; `src/proxy/failover.rs:102-146`]
- Route failover advances on configured upstream statuses before returning a response, advances on `BeforeHeaders`, and stops after a response body has been handed downstream. [VERIFIED: `src/proxy/failover.rs:161-223`; `src/proxy/failover.rs:393-410`]
- Non-idempotent POST retries are limited to transport failure before a response, not returned HTTP status. [VERIFIED: `src/retry.rs:102-115`; `src/retry.rs:167-260`]
- Existing tests already pin no failover for Responses in-body errors/rate limits and truncated 2xx Anthropic/Responses bodies. [VERIFIED: `tests/failover.rs:754-920`]
- Existing WS tests pin fallback before the first successful event and no fallback after output. [VERIFIED: `tests/codex_websocket_fallback.rs:529-665`; `tests/codex_websocket_fallback.rs:748-787`]

### Bounds and streaming

- Inbound request bodies use `LimitsConfig.max_request_bytes`; the default is `32 * 1024 * 1024`, and the bounded reader passes the limit to `to_bytes`. [VERIFIED: `src/config/http_tuning.rs:81-120`; `src/http_tuning.rs:99-160`]
- The outbound Codex WebSocket already has explicit connect/idle/reuse timeouts, a 10,000-entry pool, overflow limit 64, event channel 64, and deferred queue 8. [VERIFIED: `src/adapters/responses/codex_ws.rs:63-102`]
- Anthropic model-alias SSE rewriting bounds the first frame to 64 KiB and otherwise remains incremental. [VERIFIED: `src/adapters/anthropic/model_rewrite.rs:21-116`]
- The stream observer has a 256 KiB observation cap and does not rewrite response bytes. [VERIFIED: `src/stream_metrics.rs:22-44`; `src/stream_metrics.rs:560-595`]

### Credential and lifetime safety

- Shared upstream credential slots are exactly `authorization` and `x-api-key`; gateway-reserved inbound slots include Shunt tokens, the inbound-client marker, and `cookie`. [VERIFIED: `src/auth/slots.rs:135-175`]
- Off-origin routes strip shared credential headers, and provider-owned injection removes both shared slots before inserting the selected credential. [VERIFIED: `src/proxy/failover.rs:569-654`; `src/adapters/anthropic/mod.rs:1190-1211`]
- Gateway concurrency and provider admission permits ride the response body and release on EOF, body error, or drop. [VERIFIED: `src/concurrency.rs:36-67`; `src/concurrency.rs:111-175`; `src/adapters/mod.rs:16-40`]
- Codex WebSocket cancellation evicts/ends upstream work under receiver cancellation and backpressure. [VERIFIED: `src/adapters/responses/codex_ws.rs:986-992`; `src/adapters/responses/codex_ws.rs:2599-2653`]

### Continuation fidelity

- Stored Codex continuation is reused only when the non-input signature matches and the normalized transcript is append-only. [VERIFIED: `src/adapters/responses/codex_continuation.rs:89-147`]
- Continuation state is recorded only after clean completion and is cleared on failure. [VERIFIED: `src/adapters/responses/codex_ws.rs:982-1038`]
- Existing tests cover reconstructed tool calls followed by matching results and applying captured `previous_response_id`/turn state. [VERIFIED: `src/adapters/responses/codex_continuation.rs:435-459`; `src/adapters/responses/websocket.rs:424-481`]

## Concrete Violations That Justify Production Changes

### 1. Premature EOF becomes clean completion

`stream_response` calls `AnthropicSseMachine::finish()` when the upstream SSE ends without a provider terminal. `finish()` synthesizes an `end_turn` delta and `message_stop`; the private truncation comment affects observability but clients still receive success. The non-streaming path calls `final_json()`, which also invokes `finish()`. [VERIFIED: `src/adapters/responses/http.rs:161-223`; `src/model/responses.rs:211-233`]

The defensive outbound WS stream wrapper repeats this behavior on a bare channel close. [VERIFIED: `src/adapters/responses/ws_stream.rs:66-71`; `src/adapters/responses/ws_stream.rs:140-143`]

**Planning consequence:** first add failing tests for streaming and non-streaming premature EOF, then replace synthetic success with one protocol error terminal. Invert the existing unit test that currently blesses synthesized completion. [VERIFIED: `src/adapters/responses/http.rs:389-427`; `tests/responses_translate.rs:1272-1303`]

### 2. Malformed upstream events are silently ignored

`parse_sse_events` drops frames whose JSON does not deserialize, and the outbound Codex WS reader skips malformed or typeless text events. Malformed collected tool arguments can become `{}`. [VERIFIED: `src/model/responses.rs:733-742`; `src/model/responses.rs:866-888`; `src/adapters/responses/codex_ws.rs:964-970`]

**Planning consequence:** make parsing return a typed error, fail the active response exactly once, clear continuation state, and never reinterpret malformed tool arguments as an empty object.

### 3. Responses accumulation is not fully bounded

The HTTP SSE parser retains a delimiter-free residual without a cap and can allocate an unbounded vector of events from one feed; non-streaming response collection uses `Response::text()` without a byte bound. [VERIFIED: `src/adapters/responses/http.rs:207-223`; `src/adapters/responses/http.rs:231-267`]

The translated response machine accumulates tool arguments and non-streaming content without aggregate caps. [VERIFIED: `src/model/responses.rs:687-742`]

The outbound WS path retains arbitrary output items, response IDs, turn-state/signature strings, and a cloned continuation transcript per reusable connection. [VERIFIED: `src/adapters/responses/codex_ws.rs:922-932`; `src/adapters/responses/codex_ws.rs:1094-1133`; `src/adapters/responses/codex_continuation.rs:139-147`]

**Planning consequence:** add named internal limits for residual bytes, a single event, events processed per feed, bounded non-stream response bytes, aggregate tool assembly, continuation item count/bytes, and continuation metadata. A limit overflow must produce one error and discard replay state. The exact numeric caps are an implementation decision; choose values from protocol evidence or existing limits and test exactly-at-limit and limit-plus-one.

### 4. Commitment is fragmented

`AdapterFailure::{UpstreamStatus, BeforeHeaders}`, `RetrySafety::{Idempotent, NonIdempotentPost}`, and the WS fallback's first-`Ok` rule are separate policy vocabularies. None names replay-unsafe tool activity. [VERIFIED: `src/adapters/mod.rs:45-60`; `src/retry.rs:102-115`; `src/adapters/responses/websocket.rs:119-185`]

**Planning consequence:** introduce one small monotonic internal state, for example:

```rust
enum Commitment {
    ReplaySafe,
    ClientVisible,
    ReplayUnsafeTool,
}

impl Commitment {
    fn may_redispatch(self) -> bool {
        matches!(self, Self::ReplaySafe)
    }
}
```

This is a proposed shape, not a required public type. Retry, WS fallback, account rotation/recovery, and route failover should map their existing facts into it; the initial integration must not broaden the set of retries. Tests must prove the state is monotonic and that both committed states reject redispatch.

### 5. Synthetic or empty continuation identity can escape

Native ToolSearch invents `toolu_ts_<index>` when an upstream `call_id` is missing; ordinary function calls can emit empty IDs/names; outbound request translation accepts missing IDs as empty strings; reasoning continuation metadata can be emitted with an absent reasoning ID. [VERIFIED: `src/model/responses.rs:298-350`; `src/model/responses.rs:508-538`; `src/model/responses_request.rs:453-497`; `src/model/responses.rs:425-466`]

**Planning consequence:** add failing translation tests and reject missing tool identity/name, malformed arguments, and continuation metadata without a real non-empty reasoning ID. Preserve opaque authentic metadata byte-for-byte; do not invent signatures or IDs. Cryptographic signing of historic metadata is outside Phase 9 unless existing code already supplies a key/provenance mechanism.

### 6. Credential diagnostic redaction is not type-enforced

`Credential` derives ordinary `Debug` while containing bearer/API-key strings. No active production debug log of this type was found, but SAFE-05 calls for diagnostics to be safe by construction. [VERIFIED: `src/auth/mod.rs:29-39`]

**Planning consequence:** remove `Debug` if unused or implement redacted `Debug`; add a sentinel-based unit test that asserts the secret fragment is absent and only the credential kind/source is visible.

### 7. One response-buffering boundary exists on the Vercel-capable generic path

Generic Anthropic SSE and unaliased bodies stream, but non-SSE model-alias rewriting calls `upstream.bytes()` and can collect the entire body. [VERIFIED: `src/adapters/anthropic/mod.rs:1011-1071`]

**Planning consequence:** a Vercel characterization should include the documented non-alias path. If Phase 9 adds an alias test, first add a cap-plus-one regression and a bounded collector; otherwise record alias accumulation as a generic Anthropic hardening item only if the requirement matrix says it is exercised by Phase 9.

## Vercel Conformance Strategy

Vercel's supported path is a manual generic Anthropic upstream: `kind = "anthropic"`, a configured base URL, and API-key auth. There is no built-in Vercel preset. [VERIFIED: `src/config.rs:1763`; `src/config/presets.rs:19-79`; `site/src/content/docs/providers/vercel-ai-gateway.md:23-39`]

`AI_GATEWAY_API_KEY` resolves from the environment; the Codex credential-file fallback is specific to the env name `OPENAI_API_KEY`, so the documented Vercel path neither discovers nor writes Codex credential files. [VERIFIED: `src/auth/mod.rs:359-389`; `site/src/content/docs/providers/vercel-ai-gateway.md:23-39`]

Add named hermetic Vercel cases, preferably beside `tests/passthrough.rs`, that:

1. Configure a manual Anthropic provider at the mock origin and preserve an exact vendor/model slug and `/v1/messages` request.
2. Exercise default bearer and configured `x_api_key` injection separately.
3. Send distinct synthetic fragments in inbound `authorization` and `x-api-key`; assert neither reaches upstream and only the route-selected synthetic marker does.
4. Prove SSE bytes arrive incrementally and contain one trustworthy terminal; drop the client and observe upstream cancellation.
5. Relay representative upstream 400/401/429/503 status, body, and `retry-after` without a post-header retry.
6. Remove `AI_GATEWAY_API_KEY`; assert a gateway-owned Anthropic-shaped 401, zero upstream calls, the env-variable name but no credential fragment in the body/diagnostics.
7. Assert the provider remains absent from presets and adapter dispatch.

The existing generic path resolves credentials per route, strips both shared credential slots before injected routes, and injects only the configured bearer or `x-api-key`. [VERIFIED: `src/adapters/anthropic/mod.rs:45-105`; `src/proxy/failover.rs:569-654`; `src/adapters/anthropic/mod.rs:1190-1211`]

Do not tighten all generic provider URLs to a Vercel allowlist in this phase. Current generic validation accepts an operator-configured parseable URL with scheme and host; globally restricting schemes/userinfo/query/fragment or pinning a vendor hostname would be a public provider semantic change requiring separate approval. For Phase 9, “approved destination” means the exact configured origin, with redirects unable to carry gateway-owned credentials off-origin under the existing forwarding rules. [VERIFIED: `src/config.rs:4021-4046`; `src/proxy/failover.rs:569-654`]

Generic upstream response filtering currently removes hop-by-hop headers, not all credential-looking or cookie response headers. Stripping `set-cookie`, `authorization`, or `x-api-key` from generic provider responses may be desirable security hardening, but it changes observable passthrough behavior and is not authorized implicitly by this phase. Characterize current response behavior and escalate that policy separately rather than smuggling it into the Vercel tests. [VERIFIED: `src/headers.rs:1-40`; `src/adapters/anthropic/mod.rs:1011-1035`]

## Conformance Test Matrix

| Surface | Normal | Stream/terminal | Malformed/truncated | Retry/commitment | Credential/error | Cancellation/state |
|---|---|---|---|---|---|---|
| Responses HTTP | Preserve request/translation | Incremental SSE; one genuine terminal | Invalid UTF-8/JSON, oversized residual/event, EOF, duplicate terminal | No status/mid-body replay | Codex ingress OpenAI error shape | Drop body releases gateway + account permit |
| Responses outbound WS v2 | Compression and first turn | One terminal; existing pool bounds | Bad/typeless frame, bare close, oversize continuation | WS→HTTP only while replay-safe | Selected account headers only | Evict/cancel socket and clear continuation |
| Continuation/recovery | Exact prior response/turn state | Clean completion stores state | Failure clears state | One full-input recovery only | No account/session cross-pairing | Bound items/bytes/metadata |
| Compaction/collaboration | Existing fixtures remain green | Preserve opaque items/order | Unsupported/malformed still explicit | No duplicate tool execution | Existing auth/error contract | No durable new history |
| Vercel generic Anthropic | Manual kind/base URL/model | Incremental byte-faithful SSE | Provider error remains provider error | No post-header retry | Strip inbound creds; inject selected key | Drop streaming client releases work |
| Cross-ingress gateway errors | — | — | Local auth/limit/upstream failure | — | OpenAI on Codex, Anthropic elsewhere | Permit release on error |

The existing suites already cover much of this matrix: `tests/inbound_codex_endpoint.rs`, `tests/inbound_codex_websocket.rs`, `tests/codex_multi_account.rs`, `tests/codex_websocket_fallback.rs`, `tests/failover.rs`, `tests/retry.rs`, `tests/inbound_anthropic_translation.rs`, and `tests/passthrough.rs`. [VERIFIED: `.planning/phases/09-provider-conformance-foundation/09-CONTEXT.md:72-89`]

## Concrete File and Symbol Recommendations

### Shared commitment seam

- Add a tiny internal module such as `src/commitment.rs` or `src/replay.rs`; avoid provider traits/catalogues.
- Give it a monotonic state and `may_redispatch()`; unit-test every transition.
- Adapt `src/retry.rs::RetrySafety`, `src/proxy/failover.rs`'s `AdapterFailure` handling, and `src/adapters/responses/websocket.rs` fallback to consult that state without broadening current behavior.
- Mark tool-call emission/acceptance as replay-unsafe even before ordinary text where the provider path exposes that distinction.

### Responses terminal and bound seam

- Change `src/adapters/responses/http.rs::SseParser::push` to return `Result`, retain bytes rather than lossy text, and enforce residual/event/events-per-feed caps.
- Split `src/model/responses.rs::AnthropicSseMachine::finish` into explicit trusted-terminal completion and incomplete-EOF failure; do not let `final_json()` silently manufacture success.
- Make malformed JSON and malformed/incomplete tool arguments typed errors rather than skips/default `{}`.
- Track terminal state explicitly: no success before a provider terminal, no second terminal, and no post-terminal semantic event.
- Replace `Response::text()` with an incremental bounded collector used only for client-requested non-streaming mode.
- Update `src/adapters/responses/ws_stream.rs` so bare close is an error unless the machine already observed the authoritative terminal.

### Continuation and tool seam

- Add aggregate accounting to `src/adapters/responses/codex_ws.rs::capture_continuation` and `src/adapters/responses/codex_continuation.rs::build_transcript`.
- On overflow, finish the current delivered turn according to its genuine terminal but do not store reusable continuation; do not truncate opaque metadata and later replay it as authentic.
- Validate non-empty tool call ID/name and reasoning ID in `src/model/responses.rs` and `src/model/responses_request.rs`.
- Add a recovery integration case where `previous_response_not_found` causes exactly one full-input retry containing the same serialized tool call/result pair, session identity, and opaque continuation bytes.

### Credential and Vercel seam

- Redact or remove `Credential: Debug` in `src/auth/mod.rs`.
- Extend `tests/passthrough.rs` with a named Vercel module/cases using the existing gateway harness.
- Extend `tests/failover.rs` only for cross-provider credential rebinding and no post-commit advancement; do not duplicate generic passthrough cases.
- Keep the absence of a Vercel preset/adapter as a negative assertion or review check, not a new runtime branch.

### Cancellation seam

- Add an end-to-end HTTP streaming test that starts a never-ending/delayed mock body, consumes a first chunk, drops the downstream response, and under a bounded timeout observes upstream disconnect plus subsequent acquisition of both global and account capacity.
- Retain existing unit tests for `PermitBody` and `with_admission`; one provider-level proof is sufficient to connect those mechanisms.
- For non-cooperative `spawn_blocking` token estimates, document that input bytes are bounded and test that response-owned resources release independently; do not promise cancellation of a CPU task Tokio cannot abort. [VERIFIED: `src/proxy/failover.rs:293-304`; `src/offload.rs:19-38`]

## Validation Architecture

### Test levels

1. **Pure state-machine tests:** commitment transitions, terminal transitions, parser limits, malformed events, duplicate terminal, tool identity, and continuation accounting.
2. **Adapter tests:** translated HTTP streaming/non-streaming parity, WS bare-close/error behavior, and bounded collection.
3. **Real-gateway integration:** redispatch boundaries, exact error shapes, provider credential rebinding, Vercel behavior, continuation recovery, and cancellation/RAII.
4. **Regression sweep:** all existing Codex, account, compaction, collaboration, failover, retry, and passthrough suites.

### Wave 0 — missing tests before implementation

- Add failing unit cases in `src/adapters/responses/http.rs` for delimiter-free cap+1, event cap+1, invalid UTF-8/JSON, duplicate terminal, and EOF without terminal.
- Add failing non-streaming integration coverage for response cap+1 and incomplete EOF.
- Add WS cases for malformed/typeless frame, bare close, and continuation item/byte/metadata overflow.
- Add commitment unit tests plus a gateway matrix for pre-commit transport failure, post-text failure, and post-tool failure.
- Add exact safe-recovery serialization assertions for tool/result adjacency, IDs, session, and opaque continuation metadata.
- Add the named Vercel cases and credential `Debug` redaction test.
- Add one HTTP cancellation/permit/account-release integration test.

### Requirement-to-test routing

| Requirement | Primary evidence |
|---|---|
| PRES-01 | `inbound_codex_endpoint`, `inbound_codex_websocket`, `codex_websocket_fallback`, Responses adapter unit tests |
| PRES-02 | `codex_multi_account`, existing compaction/collaboration cases in `inbound_codex_endpoint` and inbound translation suites |
| PRES-03 / SAFE-04 | commitment unit tests, `failover`, `retry`, `codex_websocket_fallback`, new post-tool case |
| PRES-04 | cross-ingress error-shape cases in `inbound_codex_endpoint`, `passthrough`, and concurrency middleware |
| PRES-05 | named Vercel module in `passthrough` plus cancellation case |
| SAFE-01 / SAFE-02 | parser/body/continuation boundary tests and incremental first-chunk-before-terminal assertions |
| SAFE-03 | HTTP/WS malformed, explicit error, duplicate terminal, and premature EOF matrix |
| SAFE-05 | auth-slot unit tests, failover credential rebinding, Vercel auth/redaction tests |
| SAFE-06 | request/response translation negatives and one continuation recovery integration test |
| SAFE-07 | body-owner unit tests plus one real HTTP drop and existing WS cancellation tests |

### Fast feedback commands

Run focused tests after each task, then the repository gates:

```bash
cargo test --all-features --test failover --test retry --test codex_websocket_fallback
cargo test --all-features --test inbound_codex_endpoint --test inbound_codex_websocket
cargo test --all-features --test codex_multi_account --test passthrough
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features --workspace
```

These commands match the repository's documented CI contract. [VERIFIED: `AGENTS.md:3-18`]

### Validation success conditions

- All 12 Phase 9 requirements have at least one behavioral test and no test treats a truncation/error as success.
- Limit tests cover below-limit, exact-limit, and limit-plus-one where boundary arithmetic matters.
- Streaming tests prove the first payload reaches the client before the terminal/upstream completion.
- Cancellation tests use timeouts and verify reusable capacity/state, not sleeps alone.
- Tests assert structure and absence of split synthetic markers rather than storing recognizable secret formats.
- Full format, Clippy-with-warnings-denied, and all-features workspace tests pass.

## Security Architecture

### Threat boundaries

1. **Inbound client → gateway:** strip gateway-reserved and shared upstream credential slots before provider-owned authentication. [VERIFIED: `src/proxy/failover.rs:493-654`; `src/auth/slots.rs:135-175`]
2. **Gateway → configured provider origin:** resolve credentials for each route/attempt; never copy a prior provider's key across an origin change. [VERIFIED: `src/proxy/failover.rs:102-146`; `src/proxy/failover.rs:569-689`]
3. **Provider stream → parser:** treat bytes, framing, JSON, terminal, tool identity, and continuation metadata as untrusted and bounded.
4. **Parser → client:** emit at most one authoritative terminal and never convert an upstream error/cut into success.
5. **Response lifetime → shared resources:** own permits/account guards/parser buffers/continuation candidates in response-scoped RAII state and release them on completion, error, or drop. [VERIFIED: `src/concurrency.rs:36-175`; `src/adapters/mod.rs:16-40`]

### Required security regression cases

- Both inbound credential slots populated with distinct synthetic markers; neither survives when Shunt owns upstream auth.
- Failover between distinct mock origins; each receives only its route-selected credential.
- Missing Vercel credential; no upstream dispatch and no credential value in response/debug output.
- Malformed and oversized events; one protocol error, zero success terminals, continuation cleared.
- Tool call without identity/name or malformed arguments; reject before any recovery can duplicate execution.
- Dropped HTTP/WS client; bounded-time upstream cancellation and permit/account reuse.

### Explicit non-goals

- No credential file creation, refresh-policy change, migration, or writeback.
- No durable request, continuation, tool, or signature history.
- No new credential-bearing redirect behavior or global provider URL policy.
- No browser/cookie/SAPISIDHASH path.

## Documentation Impact

Correcting Responses EOF-as-success changes behavior already documented in the M1 engineering note, so the implementation task that makes that correction must update `docs/m1-responses-translation.md` in the same change. [VERIFIED: `docs/m1-responses-translation.md:112-123`]

No README or provider-site support claim should change because Phase 9 adds no provider/configuration/endpoint and Vercel remains the documented generic Anthropic route. The implementation plan must nevertheless explicitly check `README.md`, all root translations, `site/src/content/docs/` and locale copies, and record “no change needed” where behavior is not described. `wiki/` remains generated and must not be hand-edited. [VERIFIED: `AGENTS.md:28-58`; `.planning/phases/09-provider-conformance-foundation/09-CONTEXT.md:10-13`]

If named Vercel tests expose documentation drift, update English and ko/ja/zh-cn copies together only when the user-facing instructions or behavior actually change; do not turn characterization into a new preset/support claim. [VERIFIED: `AGENTS.md:38-58`; `site/src/content/docs/providers/vercel-ai-gateway.md:6-39`]

## Recommended Build Order

1. **Preservation matrix and commitment vocabulary:** add the missing characterization tests and the smallest monotonic redispatch helper; preserve current retry/failover behavior.
2. **Strict Responses terminal/parsing:** land failing HTTP/WS tests, bounded parser/collector, trustworthy terminal handling, and malformed tool/metadata rejection.
3. **Bounded continuation and safe recovery:** account for retained state, discard overflowed continuation, and prove exact tool/session/metadata recovery without duplicate execution.
4. **Credential, Vercel, and cancellation conformance:** redacted credential diagnostics, named manual-Anthropic Vercel cases, cross-origin rebinding, and end-to-end response-drop proof.
5. **Docs and full gates:** update the M1 terminal contract, consider every required docs surface, and run format/Clippy/full workspace tests.

This order exposes violations before changing production code, gives later tests one commitment/terminal vocabulary, and keeps Vercel characterization independent of the Responses repairs.

## Pitfalls

### Turning a test phase into a generalized provider framework

The repository's provider paths have materially different wires and account/session rules. Share only commitment, bounds, terminal, and redaction primitives whose invariants are already common; keep provider transcript fixtures beside their adapters. [VERIFIED: `.planning/phases/09-provider-conformance-foundation/09-CONTEXT.md:55-65`]

### Preserving a bug because a legacy test expects it

The synthesized-EOF test captures current behavior but conflicts with locked D-06 and SAFE-03. Keep ordinary behavior stable, but invert tests for concrete safety violations before changing code. [VERIFIED: `src/adapters/responses/http.rs:389-427`; `.planning/phases/09-provider-conformance-foundation/09-CONTEXT.md:27-31`]

### Emitting an error after already emitting clean success

Terminal validation must occur before the success terminal is committed to the client. A state machine that emits success and then notices a duplicate cannot repair the contract with a second error terminal.

### Bounding frames but not aggregate state

Per-frame limits do not bound events-per-feed, tool argument assembly, output item vectors, transcript clones, or metadata strings. Account for all retained bytes and item counts, including the chunk that crosses a cap. [VERIFIED: `src/adapters/responses/http.rs:231-267`; `src/adapters/responses/codex_ws.rs:1094-1133`]

### Truncating authentic continuation metadata

Truncation would create inauthentic state. If continuation data exceeds its bound, deliver the current genuine response if safe, discard reusable state, and require a later full-input request.

### Equating “first upstream event” with client commitment

Some events are metadata-only, while a tool call may be replay-unsafe even before text. The shared vocabulary must be driven by semantic exposure/side effects, not just transport activity.

### Over-broad credential hardening

Global URL restrictions or stripping arbitrary upstream response headers can change documented passthrough semantics. Prove configured-origin binding and request-header stripping now; escalate new allowlists/response-header policy separately. [VERIFIED: `src/config.rs:4021-4046`; `src/headers.rs:1-40`]

### Flaky cancellation assertions

Do not infer cancellation from elapsed sleep. Observe mock disconnect/state eviction and prove capacity can be reacquired under a bounded timeout.

### Accidentally pulling future providers into Phase 9

Gemini, Antigravity, and Cursor have similar terminal/bounds defects, but their behavior changes belong to Phases 10-12. Phase 9 may create reusable primitives; it must not alter those adapters or add generic Chat, Command Code, OpenCode Go, or Google AI Studio Web behavior. [VERIFIED: `.planning/phases/09-provider-conformance-foundation/09-CONTEXT.md:58-65`; `.planning/ROADMAP.md:112-172`]

## Phase Exclusions

- No new provider kind, preset, public config key, endpoint, or support claim.
- No Vercel-specific adapter.
- No Gemini, Antigravity, Cursor, generic Chat Completions, Command Code, or OpenCode Go behavior changes.
- No Google AI Studio Web implementation, fixture, test, auth path, browser dependency, or support implication.
- No credential discovery/persistence/writeback change.
- No live provider calls, live smoke orchestration, release-wide fixture provenance programme, or translation-parity audit; those belong to Phase 16.
- No durable history or cryptographic continuation subsystem.
- No hand edits to generated `wiki/`.

These exclusions are locked by Phase 9 context and milestone scope. [VERIFIED: `.planning/phases/09-provider-conformance-foundation/09-CONTEXT.md:10-65`; `.planning/REQUIREMENTS.md:105-119`]

## Open Questions for Planning

1. **Exact cap values:** code inspection proves which state is unbounded, but not the correct numeric limit for every Responses aggregate. The plan should require named constants, rationale from existing protocol/config limits, and boundary tests rather than silently choosing arbitrary values.
2. **Generic response credential-like headers:** current passthrough can relay upstream `set-cookie`, `authorization`, or `x-api-key`. Changing that is security-relevant but observable; treat it as a separately approved decision, not an implicit Phase 9 change. [VERIFIED: `src/headers.rs:1-40`; `src/adapters/anthropic/mod.rs:1011-1035`]
3. **Unkeyed historical reasoning metadata:** structural decode is not cryptographic provenance. Phase 9 should stop generating empty/invented metadata and preserve accepted opaque bytes, while deferring a new signing system unless separately authorized. [VERIFIED: `src/model/responses_request.rs:263-280`; `src/model/responses.rs:425-466`]

These questions do not block planning: each has a conservative default that stays inside the locked boundary.

## Assumptions

- The configured provider origin is the approved destination for generic Anthropic/Vercel in Phase 9; vendor-host allowlisting is not inferred. [ASSUMED]
- A continuation overflow may disable reuse while preserving a genuinely completed current response; it must never truncate and reuse opaque state. [ASSUMED]
- Internal caps can be added without public configuration if they are large enough for supported existing transcripts, justified, and covered at exact boundaries. [ASSUMED]

## Sources

### Primary project sources

- `.planning/phases/09-provider-conformance-foundation/09-CONTEXT.md`
- `.planning/REQUIREMENTS.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `AGENTS.md`
- `Cargo.toml`
- `src/adapters/mod.rs`
- `src/adapters/anthropic/mod.rs`
- `src/adapters/anthropic/model_rewrite.rs`
- `src/adapters/responses/http.rs`
- `src/adapters/responses/ws_stream.rs`
- `src/adapters/responses/websocket.rs`
- `src/adapters/responses/codex_ws.rs`
- `src/adapters/responses/codex_continuation.rs`
- `src/model/responses.rs`
- `src/model/responses_request.rs`
- `src/proxy/failover.rs`
- `src/retry.rs`
- `src/auth/mod.rs`
- `src/auth/slots.rs`
- `src/concurrency.rs`
- `src/http_tuning.rs`
- `src/config/http_tuning.rs`
- `src/stream_metrics.rs`
- `src/headers.rs`
- `tests/passthrough.rs`
- `tests/failover.rs`
- `tests/retry.rs`
- `tests/inbound_codex_endpoint.rs`
- `tests/inbound_codex_websocket.rs`
- `tests/codex_multi_account.rs`
- `tests/codex_websocket_fallback.rs`
- `tests/inbound_anthropic_translation.rs`
- `site/src/content/docs/providers/vercel-ai-gateway.md`
- `docs/m1-responses-translation.md`

### Prior-phase evidence

- Phase 1-8 summaries under `.planning/phases/01-*` through `.planning/phases/08-*` were reviewed for the implemented ingress, routing, quota, compaction, translation, fallback, collaboration, and shutdown contracts. [VERIFIED: `.planning/phases/01-inbound-responses-websocket/01-01-SUMMARY.md`; `.planning/phases/08-bounded-shutdown/08-03-SUMMARY.md`]

No external package or web research was required: Phase 9 concerns the checked-out implementation's existing behavior and adds no dependency. [VERIFIED: `.planning/phases/09-provider-conformance-foundation/09-CONTEXT.md:10-65`; `Cargo.toml:1-120`]

## Research Metadata

**Research type:** Codebase-first implementation research  
**External dependencies proposed:** None  
**New public semantics proposed:** None  
**Credential writeback changes proposed:** None  
**Google AI Studio Web included:** No  
**Ready for planning:** Yes
