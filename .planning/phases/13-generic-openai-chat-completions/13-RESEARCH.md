# Phase 13: Generic OpenAI Chat Completions - Research

**Researched:** 2026-09-08 (GLM high-effort researcher, worktree 0466)
**Domain:** Anthropic Messages <-> OpenAI Chat Completions translation adapter (JSON + SSE)
**Confidence:** HIGH for Shunt integration points (all read this session); HIGH for OpenCodex Chat protocol behavior (read this session); MEDIUM for reqwest redirect semantics (flagged below)

## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Independent Chat provider/adapter using existing provider settings and API-key conventions — not a Responses mode. Phase 13–14 public additions approved. No new credential writeback.
- **D-02:** Unambiguous API root; emit exactly one `/chat/completions` path. Reject malformed URLs, userinfo, query and fragment ambiguity; never reinterpret Responses endpoints.
- **D-03:** Only the configured provider API-key source; strip inbound credential slots; refuse credential-bearing redirects outside the selected destination. No credential persistence or account pool.
- **D-04:** Translate system/user/assistant text, supported images, model and generation controls without Responses-only fields. Explicitly reject unsupported representations rather than silently dropping.
- **D-05:** Preserve tool declarations, supported tool choice, parallel calls and paired results with original IDs. Reject orphan, duplicate, missing identity and incomplete object arguments; never guess pairing or synthesize results.
- **D-06:** Stream text/reasoning incrementally, assembling interleaved indexed tool deltas under bounds. Streaming and JSON must agree. Ignore unknown fields only when supported shapes remain unambiguous.
- **D-07:** One trustworthy supported finish/terminal. Malformed events, UTF-8, incomplete tools, residual bytes, embedded errors, duplicate terminals and premature EOF fail closed. One semantic contract for both output modes.
- **D-08:** Bound events, residuals, tool count, arguments, aggregation, diagnostics and read deadlines. Cancellation releases upstream and capacity through ownership, not a global registry.
- **D-09:** Generation is non-idempotent. No retry/failover after output or replay-unsafe tools; reuse existing safety vocabulary, no new retry stack.
- **D-10:** Narrow real-gateway tracer then full hermetic conformance (normal, streaming, image, tool-heavy, long-context, auth, malformed, truncation, error, cancellation). Synthetic fixtures are not live evidence. Isolated OPENCODEX_HOME, non-10100 ports.
- **D-11:** Update affected README/docs/site EN + ko/ja/zh-cn in the same PR. Never edit wiki. Format/clippy/full-suite/smoke are completion gates.

### Claude's Discretion

Private module names, exact bounds derived from existing limits, narrow helper reuse, test layout. Prefer existing dependencies and the smallest design.

### Deferred (OUT OF SCOPE)

Command Code API-key naming and subscription NDJSON (Phase 14); OpenCode Go (15); opt-in live tests and release audit (16); credential writeback changes; durable history; generalized repair/catalogs; new public retry knobs.

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CHAT-01 | Independent Chat upstream config | New `ProviderKind::OpenAiChat` + `AdapterKind::OpenAiChat`; kind string `"openai_chat"`; preset optional |
| CHAT-02 | Exactly one `/chat/completions` path, reject ambiguous roots | Port OpenCodex URL join but harden with the query/fragment/userinfo rejection Shunt already has [VERIFIED: src/config.rs:924-955] |
| CHAT-03 | Roles/text/image/model/generation translation without Responses-only fields | Whitelist-based request translation; reference grammar in OpenCodex adapter |
| CHAT-04 | Tools, tool_choice, parallel calls, paired results, stable IDs | Assistant `tool_calls` + role `tool` messages keyed by exact `tool_call_id`; orphans rejected (D-05), unlike OpenCodex repair |
| CHAT-05 | Bounded JSON response translation | `choices[0].message` grammar incl. `finish_reason` mapping, usage, embedded `error` |
| CHAT-06 | Incremental streaming, indexed tool deltas | `delta.tool_calls` accumulation keyed by `index` then `id`; fail-closed field validation |
| CHAT-07 | Trustworthy terminal, fail-closed EOF/malformed | Require `finish_reason` (and `[DONE]` framing discipline); EOF is an error; reuse Gemini byte-framing |
| CHAT-08 | API-key source, slot stripping, destination binding | `AuthMode::ApiKey`, `strip_consumed_slots`, redirect policy hardening |
| CHAT-09 | Hermetic conformance scenarios | Loopback gateway tests; pattern proven in tests/gemini_conformance.rs |

## Summary

Phase 13 adds a fifth protocol surface to an architecture that already anticipates it: the dispatch site matches on `AdapterKind`, the config layer owns provider kind/auth validation, and the retry driver already exposes the exact non-idempotent safety class this phase needs. The whole feature is one new adapter module plus two pure translation modules and focused config/routing plumbing. No new dependencies, no new retry stack, no credential machinery.

The OpenCodex Chat adapter provides the protocol contract evidence (its grammar matches the public OpenAI Chat Completions spec), but it is deliberately *more permissive* than Shunt's contract in four places: orphan tool results are repaired rather than rejected, an opt-in EOF tolerance can flush complete JSON tool calls without a terminal, provider-specific model lists mutate history (`preserveReasoningContentModels` reasoning replay), and the URL join accepts anything ending in `/v1` without rejecting query/fragment/userinfo. Each divergence is a locked Shunt decision (D-02/04/05/07), so OpenCodex is provenance for the *grammar*, not for the *policy*.

**Primary recommendation:** Build `src/adapters/openai_chat/` (transport, bounded SSE/JSON) + `src/model/openai_chat_request.rs` + `src/model/openai_chat_response.rs` (pure translation, shared by both output modes), wire `ProviderKind::OpenAiChat` -> `AdapterKind::OpenAiChat` -> `RetrySafety::ConnectOnly`, and prove it with a loopback conformance suite cloned from the Gemini pattern.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Provider kind / auth / base-URL validation | `src/config.rs` (config layer) | presets | All existing kind/auth/host validation lives here; boot-time rejection per D-02 |
| Route resolution | `src/routing.rs` | — | `AdapterKind` mapping is a one-arm match |
| Request/response translation (pure) | `src/model/openai_chat_request.rs`, `openai_chat_response.rs` | — | Testable without IO; single semantic contract for JSON and SSE (D-06/07) |
| Transport: URL join, send, SSE framing, cancellation | `src/adapters/openai_chat/` | `src/adapters/gemini/sse.rs` framing, `src/keepalive.rs`, `src/upstream_timeout.rs` | Adapter owns response lifetime and ownership-based cancellation (D-08) |
| Retry classification | `src/retry.rs` via `RetrySafety::ConnectOnly` | — | Existing driver; D-09 forbids a new stack |
| Credential slots / destination binding | `src/proxy/failover.rs` + `src/auth/slots.rs` | `src/auth/mod.rs` | Strip sites already enumerated; adapter must not re-handle |
| Failover dispatch | `src/proxy/failover.rs` `dispatch` | `src/proxy/capability.rs` | New arm; capability notes only proven incompatibilities |
| Conformance evidence | `tests/openai_chat_*.rs` | loopback gateway pattern | CHAT-09 |

## Standard Stack (no new dependencies)

| Concern | Mechanism | Provenance |
|---|---|---|
| HTTP client | `reqwest` via existing adapter pattern | [VERIFIED: src/adapters/responses/http.rs exists as transport model] |
| Router/server | `axum` AppState, `Adapter` trait | [VERIFIED: src/adapters/mod.rs:58-65 — `trait Adapter { fn forward(...) -> AdapterFuture }`, `AdapterError { message, response, failure: Option<AdapterFailure> }`] |
| SSE byte framing | Gemini `Decoder` | [VERIFIED: src/adapters/gemini/sse.rs:5 — `pub(super) const MAX_EVENT_BYTES: usize = 8 * 1024 * 1024;` and `with_limit(max_event_bytes)` constructor] — re-export or parametrize rather than copy |
| Retry safety | `RetrySafety` enum | [VERIFIED: src/retry.rs:142-156 — variants `Idempotent`, `NonIdempotentPost`, `ConnectOnly`; Chat generation uses `NonIdempotentPost`: "only proven connect-phase failures may retry"] |
| Credential stripping | `ShuntCredentials::strip_consumed_slots` / `strip_reserved_slots` | [VERIFIED: src/proxy/failover.rs:510, 634] |
| URL validation precedent | Query/fragment/userinfo rejection with a rationale comment | [VERIFIED: src/config.rs:924-955 — telemetry destination validator rejects "a query string", "a fragment", "embedded credentials"] |
| Provider base_url today | scheme+host only | [VERIFIED: src/config.rs:2080-2082 — `ProviderBaseUrl`, `ProviderBaseUrlMissingHost` errors; no path/query checks for providers yet — Phase 13 adds Chat-specific strictness] |
| Keepalive/deadlines | `keepalive::with_pings`, `upstream_timeout` | [VERIFIED: src/keepalive.rs:46 — `pub fn with_pings<S, E>`] |
| Test harness | loopback axum + TcpListener | [VERIFIED: tests/gemini_conformance.rs:71,114-120,296-342 — binds `127.0.0.1:0` backends and gateways, spawns `axum::serve`] |

Package Legitimacy Audit: N/A — zero new packages. Phase is Rust-only with existing crates (axum, reqwest, serde, tokio, futures-util).

## Protocol Provenance (OpenCodex, read-only evidence)

Source: `/Volumes/PortableSSD/Projects/opencodex/src/adapters/openai-chat.ts` (2047 lines) and `openai-chat-url.ts`, inspected this session. Shunt must record MIT provenance for any substantial translation (REL-03); grammar-only reimplementation in Rust with these findings recorded as evidence source is sufficient.

### URL construction (CHAT-02)

[VERIFIED: openai-chat-url.ts:1-10] — verbatim logic: trim trailing slashes, strip a trailing `/chat/completions`, append `/chat/completions`. Accepts `/v1`, `/v1/`, `/v1/chat/completions`, `/v1/chat/completions/`.

**SHUNT-MUST-HARDEN:** OpenCodex performs no userinfo/query/fragment rejection. Shunt rejects those at boot per D-02, reusing the rationale and pattern at [VERIFIED: src/config.rs:924-955].

### Credentials (CHAT-08)

[VERIFIED: openai-chat.ts:63-78 openAIChatTransport] — `headers.Authorization = "Bearer " + provider.apiKey`; throws when `authMode` is key/oauth and the credential is empty. Provider-level `headers` are merged after. Shunt maps this to `AuthMode::ApiKey` + `api_key_env` [VERIFIED: src/config.rs:1800-1805]. Inbound slots are stripped by the shared mechanism at failover.rs:634 before the adapter sees headers.

### Request translation (CHAT-03/04)

[VERIFIED: openai-chat.ts:40-60 CHAT_PASSTHROUGH_FIELDS] — the Chat whitelist, verbatim: `audio, frequency_penalty, logit_bias, logprobs, max_completion_tokens, max_tokens, metadata, modalities, n, prediction, presence_penalty, reasoning_effort, response_format, seed, stop, store, temperature, tool_choice, tools, top_logprobs, top_p, user, web_search_options`. Shunt's Anthropic ingress needs a *different* direction (Messages -> Chat), so this list informs which generation controls are Chat-native; Responses-only fields (`input`, `instructions`, `include`, `previous_response_id`, Responses tool types) must never be emitted [CITED: D-04].

[VERIFIED: openai-chat.ts:700-830 translation] — message grammar, verbatim key shapes:
- user text-only -> `{ role: "user", content: "<joined text>" }`; with images -> `content: [{ type: "text", text }, { type: "image_url", image_url: { url, detail? } }]`
- assistant -> `{ role: "assistant", content?: string, reasoning_content?: string, tool_calls?: [{ id, type: "function", function: { name, arguments: JSON.stringify(args) } }] }`
- tool result -> `{ role: "tool", tool_call_id, content: <text> }`

**SHUNT-MUST-HARDEN / divergence:** OpenCodex *repairs* orphan tool results by synthesizing an assistant `tool_calls` entry with `call_orphan_N` ids and `"{}"` arguments, and supports a bounded reasoning-replay cache (`peekReasoningForCall`, 64 entries/256 KiB/1 h) plus reasoning placeholders for DeepSeek-style providers. All three are model-specific permissiveness. Shunt D-05/D-06 rejects orphans, duplicates, missing identity and incomplete arguments instead, and emits no `reasoning_content` replay — a generic adapter cannot safely invent history.

### Streaming grammar (CHAT-06/07)

[VERIFIED: openai-chat.ts:1600-1915] — key behaviors:
- SSE `data:` lines; `[DONE]` -> done event with usage and optional stopReason (lines 1661-1664). `finish_reason` per `choices[]`; `finish_reason === "error"` -> upstream error event with `choice.error` (1703-1706).
- Reasoning: `delta.reasoning_content` or `delta.reasoning` string -> reasoning delta (1711-1712).
- Text: `delta.content` non-empty string -> text delta (1713-1715).
- Tool deltas: `delta.tool_calls` array; non-array fails closed (#1325 channel). Accumulator key: `i:<index>` when `index` is a number, else `id:<id>`, else the most recent pending call. Fields validated per-field with provenance tolerance: a field repeated as a non-string placeholder on a continuation delta is not re-judged, but a *first* malformed `name`/`arguments`/`id` fails the whole stream via `invalidToolCallsEvent` (diagnostic reasons verbatim: `tool_calls_not_array, tool_call_not_object, tool_call_id_invalid, tool_call_function_not_object, tool_call_function_name_invalid, tool_call_function_name_blank, tool_call_function_arguments_invalid`).
- A tool call that never receives `function.name` is a hard error: "upstream streamed a tool call without a function name — cannot dispatch" (`unnamedToolCallEvent`, ~364).
- Arguments accumulate as strings under a byte budget (`reserveTransient`/`commitRetained` with `tool_args` scope); SSE event buffer capped at `TRANSLATOR_MAX_SSE_EVENT_BYTES` — exceeding it is a translation error, not a truncation.
- Tool-call deltas are buffered until a terminal; a `heartbeat` event keeps watchdogs fed during silent argument streaming.
- Terminal rules (1884-1910): EOF with pending tool calls and no `finish_reason` -> error "upstream stream ended mid tool call without a terminal signal — possible truncation", *unless* provider opt-in `openaiChatEofTolerance` and all pending calls parse as complete JSON objects. EOF with no terminal and no user-facing output -> error "upstream stream ended without a terminal signal ([DONE] or finish_reason) — possible truncation".
  **SHUNT-MUST-HARDEN:** D-07 admits no EOF-tolerance opt-in in Phase 13; the default fail-closed branch is the contract.
- Stop mapping (`stopReasonFor`, 380-387, verbatim): `"length" -> "max_tokens"`, `"content_filter" -> "content_filter"`, otherwise none (normal stop).

### Usage (CHAT-05/06)

[VERIFIED: openai-chat.ts:1331-1338 usageFromOpenAIChat] — `prompt_tokens`/`completion_tokens` -> input/output tokens (0 when not numeric); `prompt_tokens_details`/`completion_tokens_details` read for cache/reasoning breakdowns. Streams force `stream_options = { ...caller, include_usage: true }` [VERIFIED: lines 162-165, 1542]. A trailing usage-only chunk (no choices) carries usage without a terminal.

### Non-streaming JSON (CHAT-05)

[VERIFIED: openai-chat.ts:1950-2040] — payload may be unwrapped from a `{ data: {...} }` envelope (`unwrapChatCompletionPayload`, 233-240). Embedded `payload.error` -> error event. Empty/invalid `choices` -> error. `finish_reason === "error"` -> error. `message.tool_calls` validated with the same diagnostic taxonomy; `message.content` string -> text. Same `stopReasonFor` mapping and usage extraction as streaming — this is the "one checked semantic contract" OpenCodex already demonstrates and D-07 requires.

### Error surface (provider errors)

[VERIFIED: openai-chat.ts:197-290 extractErrorDetail] — recognized error shapes: `error` as string; `error.message`; `error/detail/message/title` string fields; `detail` as array of `{ msg }`. Non-JSON bodies yield no detail (never echo markup). Request IDs extracted from `error.metadata.request_id` with a strict character allowlist and secret redaction. Shunt wraps these into its Anthropic-shaped gateway errors (CHAT ingress is not the Codex endpoint, so the Responses error shape exception does not apply [VERIFIED: AGENTS.md code-style rule; PRES-04]).

## Exact Shunt Source Paths

| File | Role | Verified lines |
|---|---|---|
| `src/routing.rs` | `AdapterKind { Anthropic, Responses, Cursor, Gemini, AntigravityCli }` + `From<ProviderKind>` | [VERIFIED: 10-18, 20-27] — add `OpenAiChat` variant + mapping arm |
| `src/config.rs` | `ProviderKind`, `AuthMode`, base_url errors, URL validation precedent | [VERIFIED: 1765-1785 (kinds), 1800-1818 (auth modes), 2080-2147 (base_url errors), 924-955 (strict URL pattern)] |
| `src/adapters/mod.rs` | `Adapter` trait, `AdapterError`, `AdapterFailure::{UpstreamStatus, BeforeHeaders}`, `with_admission` guard-ownership | [VERIFIED: 27-50, 58-65] — failure enum is how local vs upstream vs pre-header classification reaches failover |
| `src/proxy/failover.rs` | `dispatch` match on `AdapterKind`; credential slot stripping; count_tokens adapter list | [VERIFIED: 285-290 (count_tokens kind list), 352-376 (dispatch arms), 510, 634 (strips)] |
| `src/retry.rs` | `RetrySafety::ConnectOnly` — the D-09 vocabulary | [VERIFIED: 142-160; ConnectOnly checks is_connect; NonIdempotentPost would also admit timeouts] |
| `src/adapters/gemini/sse.rs` | Bounded byte-framing `Decoder` with `MAX_EVENT_BYTES = 8 MiB` and split-delimiter handling | [VERIFIED: 5, 22-27, 101-113] — reusable for Chat SSE framing |
| `src/adapters/responses/` | Largest existing adapter; transport (`http.rs` 863 lines), error mapping (`error.rs`), streaming framer patterns in `mod.rs` | [VERIFIED: file sizes; mod.rs message_start/message_delta framing comments 109-115] — reference for response-framing into Anthropic events, but do NOT import the Responses translator wholesale (CONTEXT.md code_context) |
| `src/auth/mod.rs` | `Credential` enum, `resolve_credential`, `auth_error` | [VERIFIED: 31, 151, 561] |
| `src/auth/slots.rs` | Exhaustive accept/strip slot enumeration; `strip_consumed_slots` (by value: `authorization`, `x-api-key`), `strip_reserved_slots` (by name) | [VERIFIED: doc header 1-27, 130-170, 240-254] |
| `src/keepalive.rs`, `src/upstream_timeout.rs` | Ping/deadline primitives | [VERIFIED: keepalive.rs:46] |
| `tests/gemini_conformance.rs`, `tests/responses_translate.rs`, `tests/retry.rs`, `tests/failover.rs` | Reusable harness patterns: loopback upstream, pure translation fixtures, retry-safety tests | [VERIFIED: gemini_conformance.rs:71,114-120,296-342] |

## Don't Hand-Roll

- **Retry/commitment boundary** — use `RetrySafety::ConnectOnly`; do not write a retry loop ([VERIFIED: src/retry.rs:214+ driver]).
- **SSE byte framing/residuals** — reuse the Gemini `Decoder` limit pattern for events and residuals; do not write a new line splitter.
- **Credential slot stripping** — `strip_consumed_slots`/`strip_reserved_slots`; the adapter must never filter auth headers itself.
- **Admission guard lifetime** — `with_admission` moves the guard into the response body; keep that ownership model for cancellation (D-08).
- **Anthropic event framing** — follow the existing adapter stream framers (Responses/Gemini) for `message_start`/`content_block`/`message_delta`/`message_stop`; the Chat SSE grammar maps onto the *shared response-side semantic state machine* (D-07), not onto bespoke per-mode logic.

## Common Pitfalls (phase-specific)

1. **URL ambiguity.** OpenCodex's permissive join would turn `https://host/v1?x=1` into a silently misrouted request. Reject query/fragment/userinfo at config validation; accept exactly: bare origin, `/v1`, or `/v1/chat/completions` suffix shapes, then append `/chat/completions` once (CHAT-02).
2. **Repair temptation.** OpenCodex synthesizes orphan tool results and reasoning placeholders. Shunt's contract is reject (D-05) — translating OpenCodex wholesale would violate it.
3. **EOF tolerance leakage.** The `openaiChatEofTolerance` branch exists upstream; do not port it in Phase 13 (D-07). Premature EOF, residual bytes, duplicate `[DONE]`/finish, embedded errors all fail closed.
4. **Index-vs-id keying.** Some providers resend `id` on every delta, some send only `index`. Key accumulation on `index` first, then `id`, mirroring the upstream accumulator; an unnamed first chunk is a hard error (per `unnamedToolCallEvent` rationale — synthesizing a name guesses intent).
5. **Streaming/JSON drift.** Use one semantic state machine feeding both framer paths; pair fixtures for text/tools/usage/finish/errors in both modes (Pitfalls doc: "streaming/unary semantic drift").
6. **Retry after commitment.** `NonIdempotentPost` already forbids status retries; ensure the adapter reports `AdapterFailure::BeforeHeaders` only for genuine pre-header transport failures so failover cannot redispatch post-send (SAFE-04, D-09).
7. **Redirects.** reqwest's default redirect policy follows redirects; whether `Authorization` survives cross-origin is a policy question, not an accident. [ASSUMED: reqwest strips sensitive headers on cross-origin redirect — verify against the pinned reqwest version at plan time.] Required behavior (D-03): disable or bound redirects, or reject credential-bearing redirects outside the configured destination. Cover with a loopback 30x test.
8. **Unknown-field discipline.** "Ignore unknown fields only when supported shapes remain unambiguous" (D-06): unknown *top-level* Chat request/response fields may be skipped; unknown *tool-delta field types* are errors, per the upstream per-field provenance rule.

## Validation Architecture

### Test Framework

| Property | Value |
|---|---|
| Framework | built-in `cargo test` + `axum` loopback servers (no new dev-deps) |
| Config file | none needed |
| Quick run command | `cargo test --test openai_chat_translate --test openai_chat_conformance` |
| Full suite command | `cargo test --all-features --workspace` (+ `cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`) |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|---|---|---|---|---|
| CHAT-01/02 | kind `openai_chat` accepted; ambiguous roots rejected | unit/config | `cargo test --lib config::` + focused case in `tests/openai_chat_translate.rs` | Wave 0 |
| CHAT-03 | role/text/image/model/generation translation | unit (pure) | `cargo test --test openai_chat_translate` | Wave 0 |
| CHAT-04 | tools/tool_choice/parallel/paired results; orphan rejected | unit (pure) | `cargo test --test openai_chat_translate` | Wave 0 |
| CHAT-05 | JSON response translation, usage, finish, embedded errors | unit+integration | translate file + conformance | Wave 0 |
| CHAT-06/07 | SSE deltas, indexed tools, terminal contract, malformed/EOF/oversize | integration (loopback) | `cargo test --test openai_chat_conformance` | Wave 0 |
| CHAT-08 | slot stripping, api_key source, redirect refusal | integration (loopback 30x) | `cargo test --test openai_chat_conformance` | Wave 0 |
| CHAT-09 | full scenario matrix incl. long-context boundary, cancellation, provider errors | integration | conformance file scenarios | Wave 0 |
| D-09 | no redispatch after output | integration (failover regression) | `cargo test --test failover --test retry` (extend) | exists, extend |

### Sampling Rate

- Per task commit: `cargo test --test openai_chat_translate --test openai_chat_conformance` + `cargo clippy`
- Per wave merge: full suite
- Phase gate: full suite green + fmt + clippy before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/openai_chat_translate.rs` — pure request/response translation fixtures (CHAT-02..05)
- [ ] `tests/openai_chat_conformance.rs` — loopback gateway: normal, streaming, image, tool-heavy, interleaved tool deltas, long-context boundary, auth stripping, redirect refusal, malformed/truncated SSE, embedded errors, cancellation, oversized event/argument bounds (CHAT-06..09)
- [ ] Optional shared SSE fixture helper extracted from `tests/gemini_conformance.rs` patterns if duplication grows

## Security Domain (ASVS L1, security_enforcement on)

| ASVS Category | Applies | Control |
|---|---|---|
| V2 Authentication | yes | `AuthMode::ApiKey` + `api_key_env`; key bound to provider kind at config validation (precedent: oauth host-pinning errors at config.rs:2086-2147) |
| V3 Session Management | no | stateless generation |
| V4 Access Control | no | inbound auth unchanged |
| V5 Input Validation | yes | strict URL grammar; whitelist generation controls; per-field tool-delta validation; explicit rejection of unsupported representations (D-04) |
| V6 Cryptography | no | TLS via reqwest; no hand-rolled crypto |
| V9 Communications | yes | HTTPS posture consistent with existing oauth modes; redirect/destination binding (D-03); `Authorization` never logged (redaction precedents: OpenCodex `redactSecretString`; Shunt diagnostics rules) |

Threat patterns: credential exfiltration via attacker-controlled base_url or redirect (Tampering/Spoofing) -> boot-time URL strictness + redirect policy test; history injection via orphan tool results (Tampering) -> reject; unbounded memory via oversized events/arguments (DoS) -> `MAX_EVENT_BYTES`-style bounds + argument byte budget + tool count bound (SAFE-01).

## Assumptions Needing Confirmation

| # | Claim | Section | Risk if Wrong |
|---|---|---|---|
| A1 | reqwest's redirect behavior on cross-origin `Authorization` under the pinned version (recommended: verify exact semantics or disable redirects for the Chat client) | Pitfalls #7 | Credential leak to off-origin host (D-03/CHAT-08 violation) |
| A2 | Exact kind string `"openai_chat"` for serde (name is discretionary; Command Code preset reuses it in Phase 14) | Requirements table | Cosmetic rename later; low risk |
| A3 | Gemini `Decoder` can be shared via `pub(crate)` widening or a small moved helper without breaking module privacy conventions | Standard Stack | Minor refactor churn |

## Open Questions

1. **Reasoning field naming for Anthropic output.** Chat `reasoning_content`/"reasoning" deltas must surface as Anthropic thinking blocks. Which upstream providers emit which field is grammar-verified, but whether Shunt relays reasoning on the generic path (vs treating it as optional text) should be fixed by the planner under D-06's "supported shapes" clause. Recommendation: relay as thinking block, no signature fabrication.
2. **Count-tokens behavior.** `AdapterKind` additions must be added to the count_tokens match list at failover.rs:285-290; recommendation: `CountTokens::Estimate` default like other translated adapters.

## Sources

### Primary (HIGH)
- Shunt worktree source, all cited files read this session (line refs above)
- /Volumes/PortableSSD/Projects/opencodex/src/adapters/openai-chat.ts, openai-chat-url.ts (actual adjacent HEAD 055c3ecf0de6c35f59195fc434d6b08525182b7f, checked by orchestrator; older PITFALLS revision is not this checkout)

### Secondary
- .planning/phases/13-generic-openai-chat-completions/13-CONTEXT.md, 13-HANDOFF.md, .planning/REQUIREMENTS.md, .planning/ROADMAP.md, .planning/research/ARCHITECTURE.md, .planning/research/PITFALLS.md

## Metadata

**Confidence breakdown:** Standard stack HIGH (all integration points read); Protocol grammar HIGH (OpenCodex adapter read with line cites); Policy divergences HIGH (locked by D-02/04/05/07); Redirect semantics MEDIUM (A1, verify at plan).

**Research date:** 2026-09-08. **Valid until:** 30 days (grammar stable; re-verify reqwest pin if Cargo.lock changes)

## Orchestrator source-check corrections

The researcher completed with explicit GLM/high. Its nested evidence agent failed
provider thinking-mode validation and contributed no evidence. The parent completed
the source survey independently; no live smoke or external-spec verification is claimed.

`src/retry.rs:171-179` shows NonIdempotentPost accepts transient timeouts even when
is_connect is false. Therefore Chat MUST use ConnectOnly, not the original research
recommendation. A pre-header failure is not necessarily pre-send. Pin a real post-send
timeout/no-fallback test. Disable redirects for this credential-bearing transport,
using existing redirect-hardened client conventions; do not rely on implicit stripping.

The exact Chat grammar (particularly split tool names/IDs, usage trailers and terminal
ordering) still needs fixture-level planning checks. A missing name in an initial delta
is not automatically malformed when later deltas can complete it; enforce required
identity/name at the supported assembly boundary, not by guessing or premature rejection.
