# Phase 10: Gemini Semantic Hardening — Research

**Researched:** 2026-09-06  
**Scope:** existing `kind = "gemini"` Google Code Assist transport only  
**Upstream comparison:** OpenCodex `upstream/main` at `b0900e556e50984a651a4c72db000e9285a6952a`

## Recommendation

Harden the existing adapter in place. The smallest reliable design is a Gemini-local,
bounded byte SSE decoder feeding one fallible semantic state machine used by both
streaming and non-streaming responses. Resolve the Google OAuth token and Code Assist
project once, keep that pair and the serialized request immutable, and send generation
through `RetrySafety::NonIdempotentPost`. Do not add a generic provider framework, a new
dependency, public configuration, durable signature storage, or credential writeback.

OpenCodex is useful as a bug and fixture catalogue, especially for nested-shape checks,
thought-signature attachment, usage/finish parity, and truncated-tool behavior. Its
Google parser should not be ported wholesale: it mixes direct AI Studio, Vertex, and
Antigravity policy; treats usage metadata as a streaming terminal signal; skips scalar
JSON frames; reads only the first candidate without rejecting extras; and has replay and
request-repair behavior outside this phase's replay contract.

## Existing Shunt Interfaces and Data Flow

| Boundary | Current interface | Phase 10 use |
|---|---|---|
| Adapter dispatch | `src/adapters/gemini/mod.rs::forward` | Keep endpoint/envelope/auth selection; tighten send, body reading, framing, and error projection. |
| Request translation | `translate_request_for_model` and `wrap_code_assist_envelope` in `src/model/gemini_request.rs` | Keep Code Assist `{model, project, request}`. Make tool-history signature/name pairing fail closed where the current code invents values. |
| Response translation | `src/model/gemini.rs::GeminiSseMachine` | Evolve into the single checked semantic machine; do not add a parallel unary translator. |
| Retry | `send_with_retry_with_safety` plus `RetrySafety::NonIdempotentPost` in `src/retry.rs` | Replace the current idempotent-default `send_with_retry`. The retry loop ends at response headers, before semantic commitment is possible. |
| Commitment | crate-private `Commitment` in `src/retry.rs` | Use the same vocabulary in any future/reachable Gemini recovery seam. Phase 10 should have no body-time redispatch at all. |
| Response lifetime | `reqwest::Response::bytes_stream`, Axum `Body::from_stream`, and outer response-owned RAII guards | Retain lazy streaming; dropping the downstream body must drop the upstream stream/parser promptly. |
| Tests | `tests/gemini_translate.rs`, module tests in `src/adapters/gemini/mod.rs`, Wiremock + real-Axum patterns in `tests/passthrough.rs`/`tests/retry.rs` | Keep pure semantic tests and add a dedicated real-gateway Gemini integration suite rather than testing only helpers. |

The current request lifetime already resolves one credential before translation and
captures one `access_token`, `project_id`, payload, and endpoint for all attempts. Preserve
that shape. The violation is retry policy: `send_with_retry` currently permits retrying a
returned 429/5xx status for this non-idempotent generation POST.

## Concrete Gaps to Close

### Framing and body bounds

- `append_gemini_events` decodes one line at a time, accepts only `data: `, and silently
  drops invalid UTF-8, malformed JSON, comments, other fields, and unsupported shapes.
- The streaming `Vec<u8>` line residual and each transport chunk are unbounded; a stream
  with no newline grows until cancellation/OOM.
- SSE events are separated by blank lines, not individual lines. The current parser does
  not support CRLF or multiple `data:` fields and can mis-handle arbitrary chunk splits.
- EOF calls `machine.finish()`, which substitutes `STOP`; an unterminated or provider-
  truncated stream therefore becomes a clean Anthropic completion.
- The non-streaming path uses `response.text()` without a cap and builds a second parsed
  representation. Streaming retains a complete `content` copy even though the client has
  already received each delta.

Implement a private byte decoder with explicit `event_bytes`, `residual_bytes`, and
`events_per_feed` caps. It should recognize `\n\n`, `\r\n\r\n`, `\r\n\n`, and `\n\r\n`,
decode UTF-8 only after a full frame is assembled, ignore comments/unknown SSE fields,
join all `data:` fields with `\n`, accept optional one space after `:`, and treat empty
data as a no-op. `[DONE]` is only a framing boundary; it cannot authorize success, is
legal only after the provider finish has been accepted, and must reject later data.
Invalid UTF-8, invalid JSON, over-limit state, and a non-whitespace unterminated residual
dispose the decoder and return one protocol error.

Recommended private defaults align with the proven Phase 9 pattern: 8 MiB per SSE event,
256 events per feed, and 32 MiB total non-streaming wire/translated state. Exact values
remain implementation discretion, but tests must accept the cap and reject cap + 1 using
small injected test limits. Use checked addition before extending buffers.

### One strict semantic machine

Replace infallible `process_chunk -> Vec<SseEvent>`, `finish`, and unchecked `final_json`
on production paths with checked equivalents. A compact terminal state is sufficient:

1. `Open` — content/usage may arrive.
2. `SuccessPending` — one supported, non-empty candidate `finishReason` was accepted;
   downstream `message_stop` is withheld until transport closure.
3. `SuccessEmitted` — final Anthropic terminal emitted exactly once.
4. `ProviderFailed` or `ProtocolFailed` — terminal error; reject all later input.

Required transitions and invariants:

- Accept either the Code Assist wrapper `{ "response": <Gemini object> }` or a direct
  Gemini response object through the same validation function. A present `response` must
  be an object. Do not unwrap recursively or guess mixed wrapper/direct shapes.
- A present top-level or wrapped `error` is authoritative. Validate enough of the envelope
  to map it, record `ProviderFailed`, and emit/return one Anthropic error. Never continue
  into candidates or synthesize a terminal after it.
- `candidates` may be absent on a usage-only intermediate frame, but a present value must
  be an array with at most one candidate. Reject multiple candidates rather than merging
  ambiguous order. A candidate must be an object.
- A present `content` must be an object; present `parts` must be an array; every part must
  be an object. Unknown fields and metadata-only parts may be ignored, but malformed
  supported fields and parts claiming multiple incompatible payload kinds must fail.
- Process the one candidate's parts in array order. Across chunks, preserve exact arrival
  order between reasoning, visible text, and function calls. Close/open Anthropic content
  blocks only on a real kind transition.
- `finishReason` must be a non-empty string in the explicitly supported mapping. Record it
  only after validating the complete candidate and its parts. A second finish reason,
  supported semantic data after it, provider error after it, or duplicate terminal is a
  protocol error.
- EOF succeeds only from `SuccessPending`. EOF from `Open`, `[DONE]` without a provider
  finish, or transport error is failure. A unary JSON document supplies its transport
  closure immediately after the same checked `apply` call.
- `message_start`, blocks/deltas, terminal usage, finish mapping, and provider errors are
  derived from the same state. Streaming disables final-content reconstruction;
  non-streaming enables bounded reconstruction and calls `final_json_checked`.

Keep the current public stop mapping where defined (`STOP -> end_turn`, `MAX_TOKENS ->
max_tokens`, `SAFETY -> stop_sequence`, and a valid function call -> `tool_use`) and make
the accepted provider finish table explicit. Add evidenced filter values only if mapped
consistently in both modes. `MALFORMED_FUNCTION_CALL`, a token-limit finish with an
incomplete/started tool, and any unsupported finish value must fail closed rather than
becoming `end_turn`.

### Usage, reasoning, and tools

- Validate `usageMetadata` as an object and each supported count as a non-negative integer.
  Preserve the latest authoritative cumulative prompt/candidate counts. Do not treat usage
  alone as a terminal. Emit the same final counts in streamed `message_delta` and unary
  `message.usage`.
- `thought: true` plus non-empty string `text` is reasoning; ordinary non-empty string
  `text` is visible output. A present non-string `text`, non-boolean `thought`, or ambiguous
  `thinking` compatibility shape is a protocol error under the locked fail-closed rule;
  do not coerce or silently drop claimed supported fields.
- A `functionCall` is atomic on this wire. Require an object, a non-blank string `name`,
  representable JSON `args` (default `{}` only when absent), and no conflicting part kind.
  Validate the whole candidate before emitting any of its calls so a later malformed call
  cannot leave a dispatchable partial batch.
- Preserve an authentic non-empty `thoughtSignature` on the exact function-call part by
  encoding it in the existing `call_gemini_v1_...` opaque Anthropic ID. Bound both the
  signature and encoded ID. Never move a signature by matching name/arguments.
- Remove response-side random fallback for a Gemini 3 call that requires a signature and
  remove request-side `GEMINI_THOUGHT_SIGNATURE_PLACEHOLDER` synthesis. For a legacy model
  where unsigned calls are proven valid, a generated Anthropic join ID may identify the
  call but must never be decoded or sent back as a thought signature. Unknown/foreign or
  malformed encoded IDs must not manufacture signature metadata.
- Preserve tool-call name, arguments, and ID through the next client's `tool_result` and
  rebuild the matching Gemini `functionResponse`. Unknown/orphan result IDs and missing or
  ambiguous names must fail before dispatch rather than using `"unknown_tool"`. This
  round-trip integration test is the Phase 10 interpretation of function-result relay;
  Gemini model output itself does not have a faithful Anthropic assistant `tool_result`
  representation and should not be invented as text.

## Retry, Identity, and Cancellation

Call:

```rust
send_with_retry_with_safety(
    policy,
    &route.provider,
    RetrySafety::NonIdempotentPost,
    || send_same_token_project_endpoint_and_payload(),
)
```

This permits only bounded connect/timeout failures before response headers and forbids
status retry. Do not add INVALID_ARGUMENT body repair, mid-stream retry, identity rotation,
project rediscovery, or route hopping. Assert exact upstream hit counts for returned 429/
503, malformed body, provider error, partial output, tool output, and cancellation.

The closure must capture the one resolved token/project and the exact serialized payload.
A permitted retry must send byte-equivalent JSON and identical authorization/project.
Tests should use synthetic split credential strings and a synthetic project marker, assert
their outbound placement, and assert neither appears in downstream errors/log-shaped data.
No test may read or modify a real credential file.

The lazy streaming state should own the `reqwest` byte stream, decoder, machine, and only
the request-lifetime values needed after headers. Dropping the downstream body naturally
drops those values and the upstream response; prove this with a pending mock stream and a
bounded timeout, alongside the existing outer account/global permit RAII tests. Avoid a
detached producer task or channel that could outlive the body.

## OpenCodex Evidence Worth Porting Selectively

The following patterns in `src/adapters/google.ts` and its tests are useful evidence, not
drop-in code:

- Prevalidate the candidate/content/parts/function-call structure before emitting any
  event (`google-hardening.test.ts`, issues referenced there as #1325/#1332/#2232/#2233).
- Keep `thought: true` authoritative for reasoning visibility and attach a real signature
  to the exact call part (`google-signature-history-roundtrip.test.ts`, issue #1735).
- Match streaming and buffered stop/usage behavior and fail closed on malformed function
  calls or truncation (`google-buffered-stop-reason.test.ts`,
  `google-vertex-stream.test.ts`).
- Bound non-streaming reads even when `Content-Length` is missing or false, bound the SSE
  residual before a newline/frame, and cancel the reader on bound failure.
- Preserve tool call/result IDs and adjacency in request history rather than pairing only
  by name.

Do **not** copy OpenCodex's 100 MiB limits, scalar-frame skipping, usage-as-terminal rule,
first-candidate-only ambiguity, status/body request repair, durable replay cache, media
artifact subsystem, model catalogue/prompt rewrites, or cross-product Google HTTP policy.
Those either weaken this phase's strict contract or belong to Antigravity/other products.

## Test Plan and Commands

### Pure decoder and machine tests

Extend `tests/gemini_translate.rs` or move dense private-bound cases beside the private
types. Cover:

- arbitrary transport splits including every byte of a multibyte UTF-8 code point;
- LF/CRLF/mixed blank delimiters, comments, unknown SSE fields, and multiline `data:`;
- malformed UTF-8/JSON, unterminated residual, exact/plus-one event and residual bounds,
  event-count cap, and disposed-parser behavior;
- direct and wrapped equivalents; text/reasoning/tool ordering; usage-only frames;
- malformed candidate/content/parts/call/signature/usage shapes; multiple candidates;
- embedded error before/after content; duplicate/conflicting finish; bytes after finish;
- EOF and `[DONE]` without finish; one accepted terminal; streaming accumulation disabled;
- normalized semantic parity between streaming transcript and equivalent unary object;
- authentic signature round trip, foreign/malformed ID rejection, tool name/args/result
  pairing, and no placeholder signature.

Replace the existing `test_gemini_sse_machine_finishes_on_eof` assertion with a failure
assertion; it currently canonizes the bug this phase must remove.

### Real gateway + mock upstream tests

Add `tests/gemini_conformance.rs` (or an equivalently focused file) using the existing
Axum/Wiremock harness. Exercise the real `/v1/messages` path for streaming and unary
requests, exact Code Assist route/envelope/auth, chunked delivery before terminal, malformed
and cut streams, embedded errors, bounded unary collection, same-token/project safe retry,
no status/post-header/body retry, downstream cancellation, and call -> client result ->
next `functionResponse` round trip. Keep fixtures synthetic and sanitized.

Run during implementation:

```text
cargo test --all-features --test gemini_translate
cargo test --all-features --test gemini_conformance
cargo test --all-features gemini
cargo test --all-features --test retry --test failover
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features --workspace
```

## Documentation and Exclusions

No public key, provider kind, endpoint, auth source, model claim, or credential persistence
behavior changes, so this phase should need only an engineering note if observable malformed/
truncated response behavior is currently documented inaccurately. Confirm README, `docs/`,
English/ko/ja/zh-cn site pages, and translated root READMEs; update all maintained copies only
if an existing claim changes. Never hand-edit `wiki/`.

Google AI Studio Web is an explicit negative boundary only: add no implementation, fixture,
support claim, cookie/SAPISIDHASH auth, extension, daemon, MakerSuite/session parser, WebKit/
browser dependency, or test implying support. Direct Gemini API-key and Vertex/Antigravity
policy found in OpenCodex are not Phase 10 scope. Credential-file creation, migration,
refresh-write, or any other writeback remains unchanged and prohibited.

## Suggested Planning Cut

1. Checked Gemini semantic machine and authentic tool/signature round trip.
2. Bounded incremental SSE decoder plus bounded unary collection wired to that machine.
3. Non-idempotent retry, immutable identity/project proof, real-gateway parity/error/
   cancellation suite, and documentation audit.

These cuts can be planned in two waves: semantic/decoder RED fixtures first, then adapter
wiring and gateway verification after both foundations exist.
