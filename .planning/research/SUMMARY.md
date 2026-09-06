# Project Research Summary

**Project:** Shunt v2 Provider Compatibility
**Domain:** Streaming-safe protocol translation for coding-agent providers
**Researched:** 2026-09-06
**Confidence:** HIGH for Shunt architecture and verified wire boundaries; MEDIUM for volatile proprietary services

## Executive Summary

Shunt is a lean Rust gateway, not a provider platform. The correct v2 approach is to preserve its typed adapter, bounded-stream, credential-isolation, and pre-output failover architecture while porting only proven OpenCodex wire behavior and regression cases. No new runtime dependency is needed: Axum, Tokio, reqwest, serde/JSON, bytes, futures, prost, hashing, URL validation, retries, account leases, and wiremock already cover the required HTTP, SSE, NDJSON, protobuf, auth, and test primitives.

The missing transport families are generic OpenAI Chat Completions and Command Code's proprietary subscription endpoint. Add a dedicated `openai_chat` adapter, then reuse it for Command Code's API-key Provider API at `https://api.commandcode.ai/provider/v1`. Add a separate `command_code` adapter for subscription credentials and `POST /alpha/generate`. This resolves a conflict in the feature report: current OpenCodex registry and adapter source show that the API-key product is Chat-compatible, while only the subscription product uses proprietary NDJSON. Preserve ChatGPT/Codex, Gemini, and Vercel's existing Anthropic-compatible path; selectively harden Antigravity and Cursor inside their current modules.

The dominant risks are state-boundary failures: retrying after visible output or a tool side effect, accepting truncated EOF as success, losing tool-call/result or thought-signature continuity, switching credential/project identity during a turn, and leaking subscription tokens off-origin. Establish shared conformance invariants first, then deliver provider slices independently. Credential-file writeback changes remain excluded. Google AI Studio Web is wholly excluded—no cookie/SAPISIDHASH auth, browser extension/daemon, MakerSuite parser, session synchronization, WebKit, tests, support claims, or dependencies. OpenCode Go is optional and can ship only as dated, exact model + exact wire tuples backed by captured or live evidence; no family or provider-wide inference is acceptable.

## Key Findings

### Recommended Stack

Keep the existing Rust stack and add no third-party runtime dependency. New behavior should be implemented with narrow typed request compilers and bounded protocol-specific stream state machines, reusing Shunt's existing lifecycle and security primitives.

**Core technologies:**

- **Rust 2021 + Axum 0.8.9:** preserve the existing HTTP ingress, response bodies, WebSockets, middleware, and error surfaces.
- **Tokio 1.52.3:** deadlines, cancellation, blocking offload, and bounded async coordination.
- **reqwest 0.12.28:** streamed HTTP/2 bodies for SSE, NDJSON, Gemini, and Cursor; use the direct dependency rather than transitive 0.13 APIs.
- **serde / serde_json 1.x:** typed outer envelopes and controlled open-ended provider payloads.
- **bytes + futures-util:** incremental framing and backpressured relay without collecting streaming success bodies.
- **prost 0.13.5:** retain the existing Cursor protobuf implementation; do not add tonic, ConnectRPC, or code generation.
- **sha2 + uuid + url:** opaque session affinity, fallback IDs, and exact HTTPS destination validation.
- **wiremock 0.6:** hermetic JSON/SSE/NDJSON/auth/retry/truncation transcript tests.

Do not import Bun, Node, TypeScript, Zod, MCP runtimes, browser automation, native keyrings, generalized storage/catalog systems, a new SSE parser crate, or a second retry/account/WebSocket stack.

Detailed rationale: [STACK.md](./STACK.md).

### Expected Features

**Must have (table stakes):**

- Preserve ChatGPT/Codex HTTP/SSE/WS, OAuth pool, quota, continuation, compression, opaque state, cancellation, compaction, collaboration policy, and OpenAI-shaped Codex-ingress errors.
- Verify Gemini Code Assist project envelope, incremental streaming, functions, reasoning, usage, tool history, auth, and error semantics.
- Harden Antigravity's exact Cloud Code Assist destination, agent envelope, model/effort mapping, stable session, account/project affinity, thought signatures, tools, and one bounded pre-output 401 recovery.
- Harden Cursor's exact model/profile evidence, Connect framing, continuation, executable/freeform tool schemas, images, usage, terminal/error classification, cancellation, and pre-output retry.
- Add generic OpenAI Chat request/response translation for text, images, tools/results/choice, reasoning controls, JSON, SSE, usage, and terminal errors.
- Add two Command Code products: API-key Chat and subscription NDJSON, with independent auth/endpoint tests and no new credential persistence.
- Enforce explicit resource bounds, one trustworthy terminal outcome, and no retry/failover after output or replay-unsafe tool activity.
- Maintain sanitized, provenance-tagged fixtures for every supported provider/auth/model/wire tuple.

**Should have (competitive):**

- Evidence-gated exact compatibility rather than provider-name marketing claims.
- One lean conformance harness spanning SSE, NDJSON, protobuf, and Responses WS semantics.
- Destination-bound subscription credentials and stable opaque session affinity.
- Evidence-scoped normalizations with honest unsupported errors instead of global repair.
- Optional exact-model OpenCode Go support after the core Chat adapter is complete.

**Defer or exclude:**

- Whole-provider OpenCode Go support, generic per-model adapter configuration, durable history/signature databases, catalog/dashboard/pricing machinery, Cursor semantic no-progress heuristics, mid-stream retry, and new credential writeback.
- Google AI Studio Web and every supporting auth/browser/session dependency are explicitly excluded.
- A new Vercel adapter is unnecessary; preserve its existing Anthropic-compatible route. A Chat preset is optional only after the generic adapter exists and has independent evidence.

Detailed landscape: [FEATURES.md](./FEATURES.md).

### Architecture Approach

Routing selects a protocol contract, presets select location/authentication, and exact model mappings select the wire. Never branch on a provider's display name in shared translation code. Chat and Responses remain separate adapters; Command Code subscription remains separate from both. Antigravity reuses Gemini transport machinery through small Antigravity-only policies, while Cursor remains an isolated ConnectRPC subsystem.

**Major components:**

1. **Conformance and commitment boundary** — shared fixtures and assertions for bounded parsing, terminal state, tool pairing, credential ownership, cancellation, and pre-output-only retry/failover.
2. **Existing-provider hardening** — Gemini/Antigravity policy functions and Cursor request/stream/profile corrections without changing mature ChatGPT/Codex behavior.
3. **`openai_chat` vertical** — new provider/adapter kind, pure request translation, bounded JSON response mapping, and incremental SSE state machine.
4. **`command_code` vertical** — host-pinned read-only subscription auth, proprietary envelope, stable session, tool adjacency, and bounded fail-closed NDJSON.
5. **Exact model/wire evidence table** — internal, canonical-destination-scoped rules for optional OpenCode Go; avoid a public generalized `model_adapters` system until multiple destinations require it.

Detailed boundaries and flows: [ARCHITECTURE.md](./ARCHITECTURE.md).

### Critical Pitfalls

1. **Retry or failover after commitment** — define commitment as any client-visible byte or replay-unsafe tool side effect; after it, emit the error and never redispatch.
2. **EOF or malformed frames accepted as success** — require a trustworthy terminal, reject invalid UTF-8/JSON/indexes/types, and bound chunks, residual lines, events, and tool arguments. Exact terminal-less tolerance requires exact captured evidence.
3. **Broken continuation state** — normalize tool call/result pairs as units; preserve authentic thought signatures only with matching call/account/session identity; never invent a real signature.
4. **Credential exfiltration or affinity loss** — bind subscription auth mode to provider kind, exact HTTPS origin/path, redirect policy, account, project, retry, and full response lifetime.
5. **Streaming/unary drift** — drive both modes through the same semantic event machine; buffer only when the client asked for non-streaming.
6. **Conflating protocols/products** — Chat is not Responses; Command Code API-key is not Command Code subscription; OpenCode Go is not one wire.
7. **Unsafe upstream copying or live tests** — retain MIT provenance for substantial translations, keep live tests opt-in/bounded/redacted, isolate temp credential paths, and prove source credentials remain unchanged.

Detailed prevention and recovery guidance: [PITFALLS.md](./PITFALLS.md).

## Implications for Roadmap

### Phase 9: Provider Conformance Foundation

**Rationale:** All later work touches central adapter, dispatch, auth, or shared Gemini boundaries. Establish invariants before adding variants.

**Delivers:** Characterization fixtures for ChatGPT/Codex, Gemini, Cursor, and Vercel; reusable split-frame helpers; commitment-state assertions; strict terminal/error matrix; credential redaction and provenance conventions.

**Addresses:** Preservation, bounded resources, cross-provider fixtures, strict terminals, pre-output failover.

**Avoids:** Regression of mature providers, malformed-frame loss, duplicated output/tools/billing, secret-bearing fixtures.

### Phase 10: Gemini Semantic Hardening

**Rationale:** Antigravity shares Gemini's Code Assist transport, so correct the common semantic state machine before Antigravity-specific behavior.

**Delivers:** Streaming/unary parity, embedded Google error propagation, strict malformed-frame/EOF handling, and explicit generation-POST retry classification while preserving Google OAuth/project behavior.

**Addresses:** Gemini verification and shared Code Assist reliability.

**Avoids:** Unary error loss, implicit STOP synthesis, unsafe replay, and accidental Antigravity credential/host reuse.

### Phase 11: Antigravity Protocol and Credential Hardening

**Rationale:** Builds on the verified Gemini event machine while isolating Antigravity identity, catalog, and security contracts.

**Delivers:** Canonical daily/production HTTPS destinations, always-SSE upstream consumption, stable thread-derived session, exact model/effort resolution, bounded signature and tool-history handling, account/project affinity, and at most one pre-output account-bound 401 replay.

**Addresses:** Antigravity envelope, signatures, tools, models, auth, and errors.

**Avoids:** Off-origin OAuth leakage, stale/wrong-project tokens, invented signatures, cross-account cache contamination, and streaming/unary drift.

**Boundary:** Do not change credential-file writeback or broaden the existing refresh lock without separate authority.

### Phase 12: Cursor Evidence-Backed Hardening

**Rationale:** Cursor is independent of OpenAI/Gemini wires and can be reviewed as one contained proprietary transport slice.

**Delivers:** Correct local-versus-transport failover classification, exact endpoint/client/model profile fixtures, stable continuation, paired tool results, executable/freeform schemas, strict Connect/frame EOF/errors, and pre-output-only retry.

**Addresses:** Cursor model, tool, continuation, usage, cancellation, error, and retry behavior.

**Avoids:** Replacing Shunt's proven `api5` Run endpoint from an unverified `api2` constant, replay after tools/output, process-global model evidence, and speculative no-progress cancellation.

### Phase 13: Generic OpenAI Chat Completions

**Rationale:** This is the reusable missing wire needed by general compatible gateways and Command Code's API-key product.

**Delivers:** `ProviderKind`/`AdapterKind::OpenAiChat`, normalized `/chat/completions` URL construction, API-key injection, text/image/tool/tool-choice request mapping, bounded JSON output, incremental SSE with parallel indexed tool-call assembly, usage, stop reasons, and strict truncation/errors.

**Addresses:** Generic OpenAI Chat and the protocol half of Command Code API-key support.

**Avoids:** Responses fields on Chat, generic tolerant EOF, unbounded arguments, double `/v1`, and provider-name branches.

### Phase 14: Command Code Product Separation

**Rationale:** The API-key route can now reuse Chat; the proprietary subscription route can be implemented without contaminating it.

**Delivers:** `commandcode` API-key Chat preset at `/provider/v1`; separate `command-code` subscription adapter at `/alpha/generate`; host-pinned read-only env/local-CLI credential resolution; stable opaque session; exact model/effort facts; paired tool history and image carriers; bounded NDJSON text/reasoning/tool/usage/error/terminal translation.

**Addresses:** Both Command Code products, auth, normal/stream/tool-heavy/continuation/error behavior.

**Avoids:** Conflated URLs/auth/events, writeback expansion, shared-cache session IDs, missing tool results, junk-only or premature EOF success, and stale version-header assumptions.

### Phase 15: Exact OpenCode Go Evaluation and Optional Slice

**Rationale:** Depends on the Chat adapter and may reuse Responses or Anthropic per exact model; uncertainty must not shape core architecture.

**Delivers:** A dated evidence ledger and only those exact tuples that pass captured plus safe live tests, including endpoint, protocol, `x-opencode-session`, context, modalities, effort, tools, field sanitation, reasoning replay, terminal behavior, and unsupported-model rejection.

**Addresses:** Optional OpenCode Go support.

**Avoids:** Family inference, provider-wide support claims, unsafe EOF tolerance, catalog-as-callability assumptions, and known Muse truncation/tool regressions.

**Initial candidates:** `gpt-5.6-luna` and `grok-4.6` on Responses are feasible only after verification. Muse Spark Contributor 1.2/1.3 and Anthropic-wire MiniMax candidates remain deferred until exact transcripts and live results pass.

### Phase 16: Cross-Provider Release Gate

**Rationale:** Provider slices need one final security, compatibility, documentation, and provenance audit before merge.

**Delivers:** Format, Clippy with warnings denied, full workspace tests, opt-in redacted live smokes, no-write assertions, host/redirect/affinity audit, post-output retry audit, MIT provenance, and documentation parity across README plus site English/ko/ja/zh-cn.

**Addresses:** Release confidence, documentation, licensing, and operational honesty.

**Avoids:** Aspirational support claims, locale drift, credential mutation/leakage, missing attribution, hand-edited `wiki/`, and accidental Google AI Studio Web surface/dependency additions.

### Phase Ordering Rationale

- A conformance foundation must precede central enum/dispatch and shared Gemini changes so existing ChatGPT/Codex behavior is measurable rather than assumed.
- Gemini precedes Antigravity because Antigravity reuses its Cloud Code Assist semantic machine.
- Cursor remains a separate slice because its protobuf/Connect and tool-continuation concerns share no implementation with Chat or Command Code.
- Generic Chat precedes Command Code so the API-key product is configuration over a proven reusable adapter.
- OpenCode Go follows Chat because its exact tuples may choose Chat, Responses, or Anthropic; it must not create the architecture it consumes.
- Documentation belongs in each behavior phase; Phase 16 verifies parity rather than postponing all docs.

### Research Flags

Phases likely needing deeper research during planning:

- **Phase 11 (Antigravity):** volatile catalog/model IDs, signature semantics, canonical destination behavior, and account-bound 401 recovery need current captured/live evidence.
- **Phase 12 (Cursor):** proprietary endpoint/profile drift and continuation/tool schemas require paired Shunt/OpenCodex fixture review; do not infer from constants.
- **Phase 14 (Command Code):** validate the versioned identity header and credential precedence against a current, sanitized live/captured transaction.
- **Phase 15 (OpenCode Go):** mandatory per exact model/wire tuple; this phase is an evidence gate and may validly deliver no supported models.

Phases with established patterns (skip separate research unless planning finds new evidence):

- **Phase 9:** existing Shunt fixtures, bounded parsers, retry, auth-slot, and redaction patterns are well established.
- **Phase 10:** current Gemini source/tests identify the exact semantic gaps.
- **Phase 13:** Chat Completions protocol mapping is established; provider quirks still require explicit narrow fixtures during implementation.
- **Phase 16:** repository quality, documentation, locale, licensing, and exclusion rules are explicit.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Direct Cargo/source audit shows all necessary primitives already exist; official crate docs confirm versions/capabilities. |
| Features | HIGH for Shunt/current source; MEDIUM for live providers | Acceptance matrix is concrete, but proprietary services and entitlements change. |
| Architecture | HIGH for module boundaries; MEDIUM for proprietary fields | Typed adapter/preset/auth separation matches current Shunt design; Command Code/Go wire details require capture. |
| Pitfalls | HIGH for local failure modes; MEDIUM for upstream causes | Shunt source and merged regressions establish the invariants; issue hypotheses are not treated as final causes. |

**Overall confidence:** HIGH in the roadmap structure and exclusions; MEDIUM in exact proprietary model/header values until phase probes pass.

### Gaps to Address

- **Command Code version header:** validate the current `x-command-code-version` value before setting/documenting an internal default.
- **Command Code credential precedence:** choose and document whether a dedicated environment variable overrides the read-only local CLI file.
- **Feature-report inconsistency:** the API-key path is resolved to OpenAI Chat from current registry/source evidence; tests must pin `/provider/v1/chat/completions` and prevent regression to `/alpha/generate`.
- **Antigravity signature lifetime:** begin with request history or bounded in-memory state; cross-restart persistence remains out of scope.
- **Antigravity refresh concurrency:** open Shunt issues identify discovery/lock concerns, but changing writeback/lock ownership requires separate approval.
- **Cursor endpoint conflict:** retain Shunt's captured working `api5` endpoint until live evidence proves a safe profile change.
- **OpenCode Go roster:** exact mapping and terminal behavior are mutable; every shipped tuple needs dated evidence and can be omitted if verification fails.
- **Live credentials:** absence of safe credentials should skip a live probe with a reason, not weaken hermetic acceptance or print credential data.

## Sources

### Primary (HIGH confidence)

- Shunt source and tests at local milestone baseline `e029ae2fb35eee9149855769a779343d7139d209`: `Cargo.toml`, `Cargo.lock`, `src/config.rs`, `src/routing.rs`, `src/proxy/`, `src/adapters/`, `src/model/`, `src/auth/`, and `tests/`.
- [OpenCodex source at pinned `upstream/main` `07b48da8`](https://github.com/lidge-jun/opencodex/tree/07b48da8fd63881e848d26e0bd50087864f5573e): current Chat, Command Code, Antigravity, Cursor, OpenCode Go transport/registry, and tests.
- [OpenCodex MIT license](https://github.com/lidge-jun/opencodex/blob/07b48da8fd63881e848d26e0bd50087864f5573e/LICENSE): provenance requirement for substantial translated material.
- [reqwest 0.12.28 docs](https://docs.rs/crate/reqwest/0.12.28) and [prost 0.13.5 docs](https://docs.rs/crate/prost/0.13.5): direct dependency capabilities, cross-checked against Shunt source.
- `.planning/PROJECT.md` and `.planning/notes/opencodex-port-audit.md`: approved milestone boundary and exclusions.

### Secondary (MEDIUM confidence)

- [Command Code tool-result repair PR #1411](https://github.com/lidge-jun/opencodex/pull/1411), [affinity PR #3692](https://github.com/lidge-jun/opencodex/pull/3692), and [integration issue #909](https://github.com/lidge-jun/opencodex/issues/909).
- [Antigravity account-bound recovery PR #3691](https://github.com/lidge-jun/opencodex/pull/3691), [fixed-destination PR #3799](https://github.com/lidge-jun/opencodex/pull/3799), and Shunt issues [#336](https://github.com/pleaseai/shunt/issues/336), [#373](https://github.com/pleaseai/shunt/issues/373), and [#383](https://github.com/pleaseai/shunt/issues/383).
- Cursor schema PRs [#3707](https://github.com/lidge-jun/opencodex/pull/3707) and [#3715](https://github.com/lidge-jun/opencodex/pull/3715), OpenCodex issue [#3506](https://github.com/lidge-jun/opencodex/issues/3506), and Shunt issue [#275](https://github.com/pleaseai/shunt/issues/275).
- OpenCode Go issues [#3378](https://github.com/lidge-jun/opencodex/issues/3378), [#950](https://github.com/lidge-jun/opencodex/issues/950), [#2156](https://github.com/lidge-jun/opencodex/issues/2156), and exact-wire/session fixes [#3394](https://github.com/lidge-jun/opencodex/pull/3394), [#3405](https://github.com/lidge-jun/opencodex/pull/3405).

### Tertiary (validation required)

- Open issue hypotheses or third-party reports that are not reproduced in Shunt are phase-research leads only. They cannot justify provider-wide support, permissive EOF, retries after commitment, or credential behavior changes.

---
*Research completed: 2026-09-06*
*Ready for roadmap: yes*
