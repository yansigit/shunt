# Phase 11: Antigravity Protocol and Credential Hardening — Research

**Researched:** 2026-09-07  
**Confidence:** HIGH for repository facts; MEDIUM for dated upstream wire evidence  
**Scope:** native HTTP `kind = "antigravity"` only. The deprecated local `antigravity_cli` process remains out of scope.

## User Constraints

### Phase Boundary

Harden Shunt's existing native HTTP Antigravity provider on the shared Cloud Code Assist/Gemini transport. Deliver canonical HTTPS destination safety, request-local account/token/project/session affinity, exact envelope and model/effort admission, authentic tool/signature continuation, one strict always-SSE response contract for both client modes, and at most one account-bound pre-commit 401 refresh/replay. Keep the deprecated local `agy` CLI transport available but do not treat it as native protocol evidence or expand it in this phase.

### Locked Decisions

- D-01: credentials leave Shunt only for canonical daily or production HTTPS Cloud Code Assist origins; all redirect boundaries fail closed.
- D-02: select one account, bearer, project, catalog, and session tuple before dispatch and retain it through response lifetime/replay.
- D-03: preserve credential-file behavior; no new file, migration, rotation, project writeback, or unrestricted rotation policy.
- D-04: use request-local ownership and RAII cancellation; no background replay.
- D-05: emit the proven envelope (`model`, `project`, Antigravity metadata, fresh `agent-<uuid>` request id, request body, opaque session).
- D-06: session scope is account/conversation-local, distinct across unrelated conversations, non-durable.
- D-07: fresh per-account catalog evidence is authoritative; unknown/stale-negative combinations fail before credential/network dispatch.
- D-08: honor explicit effort only when exact catalog evidence supports it; no clamping or rewriting.
- D-09: always call `streamGenerateContent?alt=sse`, including downstream non-streaming requests.
- D-10: one incremental bounded SSE decoder/state machine; only non-streaming downstream requests accumulate.
- D-11: malformed, incomplete, duplicate, post-terminal, invalid-UTF-8, embedded-error, and premature-EOF cases fail closed.
- D-12: streaming and non-streaming renderings of one transcript are semantically equivalent.
- D-13: preserve sequential tool call/result order and authentic IDs; validate relations before emission.
- D-14: replay only non-empty matching signatures bound to account/session; reject copied, invented, orphaned, or duplicate signatures.
- D-15: distinguish external seed items from tool results; do not invent IDs.
- D-16: one forced same-account refresh/replay for a pre-commit 401; second 401/failure terminates.
- D-17: never retry after headers, output, tool activity, body/parser failure, ambiguous send, or mid-stream state.
- D-18: establish strict SSE/EOF/terminal/cancellation before identity/catalog/signature/replay layers.
- D-19: port narrow invariants, not generalized caches, storage, dashboards, cross-account reuse, or heuristic cancellation.
- D-20: use focused synthetic hermetic real-Axum/loopback fixtures; no live calls or secret-bearing diagnostics.
- D-21: update README, engineering docs, site English, and ko/ja/zh-cn copies; never hand-edit `wiki/`.

### Out of Scope

Credential writeback/migration, unrestricted multi-account rotation, cross-account project reuse, quota probing beyond request admission, durable conversation/signature persistence, generalized repair, and Antigravity CLI parity/removal are deferred.

## Existing Implementation (verified)

- Native Antigravity is selected by `AuthMode::AntigravityOauth` in the Gemini adapter; the adapter accepts `Credential::AntigravityOauth { access_token, project_id }` and resolves it once before translation [VERIFIED: src/adapters/gemini/mod.rs:191-231; src/auth/mod.rs:43-54,182-197].
- The current method is conditional: streaming uses `streamGenerateContent?alt=sse`, while non-streaming uses `generateContent` [VERIFIED: src/adapters/gemini/mod.rs:228-239]. This is the primary ANT-06 gap.
- Antigravity envelopes contain `model`, `project`, `userAgent: "antigravity"`, `requestType: "agent"`, `requestId`, and nested `request.sessionId` [VERIFIED: src/model/antigravity_request.rs:13-20,38-70]. Request IDs are `agent-` plus UUID v4 [VERIFIED: src/model/antigravity_request.rs:73-77].
- Session IDs currently hash the first non-empty user text with bounded FNV-1a, or choose random digits if absent [VERIFIED: src/model/antigravity_request.rs:79-123]. This is not account/conversation scoped and must be replaced or wrapped with request-local scope under D-06.
- Catalog discovery is cached by normalized base URL plus project (or bearer fingerprint when projectless), uses a ten-minute success TTL and five-second fetch timeout, and returns stale/`None` on failure [VERIFIED: src/auth/antigravity/catalog.rs:25-44,69-110,186-255]. The current caller documents discovery as fail-open and uses the returned set to reshape model IDs [VERIFIED: src/adapters/gemini/mod.rs:270-307]. D-07 requires changing negative admission semantics, not merely improving the cache.
- Production-pinned Antigravity inference is normalized to the daily host by `inference_base_url`; endpoint construction appends `/v1internal:{method}` [VERIFIED: src/adapters/gemini/mod.rs:45-54,248-268]. Existing unit tests cover production-to-daily and loopback preservation [VERIFIED: src/adapters/gemini/mod.rs:560-590], but not redirect-hop bearer safety or lookalike origins.
- The request closure captures endpoint, serialized payload, token, and user agent; current Antigravity retry safety is `Idempotent` [VERIFIED: src/adapters/gemini/mod.rs:319-357,548-569]. This permits status retry and has no forced account-bound 401 refresh/replay.
- Streaming already owns the upstream byte stream, decoder, and semantic machine inside `Body::from_stream`; decoder errors and EOF are projected through the stream [VERIFIED: src/adapters/gemini/mod.rs:375-520]. The Phase 10 decoder rejects invalid UTF-8/JSON, bounds events, and rejects data after `[DONE]` [VERIFIED: src/adapters/gemini/sse.rs:1-102].
- `GeminiSseMachine` has checked wrapper/direct validation, bounded semantic retention, terminal states, usage, reasoning, tools, and provider errors [VERIFIED: src/model/gemini.rs:1-120,180-260]. Reuse it after adding Antigravity-specific wrapper/identity checks rather than creating a second translator.
- Credential resolution has a crate-private injectable `CredentialResolver`, production `DefaultCredentialResolver`, and request-local `AppState::resolve_route_credential` seam [VERIFIED: src/auth/mod.rs:88-115; src/server.rs:147-154]. This is the preferred hermetic seam for account/token lifetime tests.

## Standard Stack and Architecture Pattern

Use existing Rust/Tokio/Axum/reqwest/serde_json primitives and no new dependency [VERIFIED: Cargo.toml:1-120]. Keep the vertical flow:

`route snapshot → exact account/catalog admission → one credential/project/session tuple → canonical endpoint validation → exact envelope → always-SSE byte decoder → checked Gemini/Antigravity semantic machine → streaming relay or bounded unary rendering`.

Implement private crate-local types in `src/auth/antigravity`, `src/model/antigravity_request.rs`, and `src/adapters/gemini`; preserve the public adapter/config API. Reuse `RetrySafety` and commitment vocabulary from `src/retry.rs` [VERIFIED: src/retry.rs:130-210] and response-owned RAII from Phase 10. Do not hand-roll a cache, persistence layer, or generic provider framework.

## Required Workstreams and File/Symbol Touchpoints

1. **Strict transport first:** `src/adapters/gemini/mod.rs::forward`, `antigravity_endpoint`, response status/body handling; make Antigravity always use `streamGenerateContent?alt=sse`, parse both modes through the same decoder/machine, and reject any non-clean EOF.
2. **Destination safety:** `src/auth/antigravity/auth.rs` URL predicates/client construction and `src/auth/shared.rs` redirect policy. Validate scheme, exact host, port/path/query/fragment policy before attaching bearer and every redirect target before following. Tests must include suffix/lookalike hosts, plaintext, query/fragment, production normalization, and off-origin redirect.
3. **Affinity lifetime:** `src/auth/mod.rs::Credential`, `CredentialResolver`; `src/server.rs::AppState`; Antigravity auth/catalog. Introduce an opaque request-local account identity and bind token/project/catalog/session/replay to one immutable tuple. Preserve existing file locking/writeback exactly.
4. **Exact envelope/session:** `src/model/antigravity_request.rs::{wrap_antigravity_envelope, antigravity_session_id}`. Session must be stable for an explicitly shared conversation/account scope, random/opaque otherwise, bounded, and free of prompt/account/secret material. Test repeated turns, unrelated conversations, account changes, and cancellation.
5. **Exact catalog admission:** `catalog_ids` and model resolver. Fresh exact account evidence must admit only exact model/effort tuples; unknown, ambiguous, stale-only negative, bare/Claude/GPT rewrites fail before credential attachment/network. Do not clamp.
6. **Authentic tools/signatures:** extend request translation and Gemini semantic validation. Maintain ordered call IDs/results; bind encoded signature metadata to account/session and reject foreign, invented, orphaned, duplicate, empty, or mismatched values. Keep external seed items separate.
7. **Bounded 401 replay:** add a narrow Antigravity-only pre-commit path around the initial send. Force refresh once for the same account, rebuild only bearer binding while retaining validated project/catalog/session/payload, and prohibit replay after headers/output/tool activity or any ambiguous failure. Do not alter file writeback rules.
8. **Docs/tests:** native tests separate from `tests/antigravity_process.rs`; update `README.md`, `docs/notes/antigravity-daily-host.md` or a new Phase 11 note, affected configuration/troubleshooting pages, and all maintained locale copies. `wiki/` excluded.

## API Capability Surface

| Capability | Current evidence | Phase 11 admission rule |
|---|---|---|
| Native HTTP Antigravity | Gemini adapter + Antigravity OAuth | Supported only on exact canonical HTTPS destinations. |
| Agent envelope | `wrap_antigravity_envelope` | Exact fields and opaque scoped session only. |
| Models/effort | `fetchAvailableModels` key set and resolver | Fresh per-account exact tuple; no family inference/clamping. |
| SSE | `GeminiSseDecoder` + `GeminiSseMachine` | One always-SSE path for both client modes, bounded and terminal-strict. |
| Text/reasoning/usage/tools/errors | checked Gemini semantic machine | Antigravity wrapper and terminal rules must be explicit and parity-tested. |
| Tool continuation/signatures | existing encoded Gemini tool IDs | Authentic same-call/account/session only; no local synthesis. |
| 401 recovery | absent for Antigravity | Max one same-account pre-commit refresh/replay. |
| Deprecated CLI | `src/adapters/antigravity/*`, process tests | Retain unchanged; never use as native evidence. |

## Common Pitfalls and Threat Model

| Threat | Mitigation and proof |
|---|---|
| Bearer exfiltration through redirect or lookalike host | Exact origin allowlist, no unsafe initial URL, redirect policy rejects before request; assert mock never receives bearer. |
| Token/project/catalog race after login or refresh | Immutable request tuple; account-keyed catalog; concurrent swap test proves old request never pairs new token/project. |
| Session or signature cross-account replay | HMAC/hash or equivalent opaque binding with account/session context; reject copied IDs before dispatch; no secret in rendered ID. |
| Unsupported model/effort guessed into billable request | Fresh catalog gate before credential/network; negative tests assert zero inference hits and no Authorization header. |
| 401 replay duplicates generation/tool activity | Replay state is pre-header/pre-output only and max one; returned 401, output, tool, body, parser, and second 401 are terminal. |
| SSE truncation/malformed wrapper creates false success | Byte-bound decoder plus checked machine requires supported finish and clean terminal/EOF; both modes share parser. |
| Unbounded body/event/tool/signature retention | Inject small limits; exact-cap passes, cap+1 fails; cancellation drops upstream body/parser/admission. |
| Credential leakage in diagnostics/docs/fixtures | Synthetic values only, redacted `Debug`, no live calls, scope scan for secrets and Google AI Studio artifacts. |

## Validation Architecture

Use Rust unit tests for pure URL, envelope, catalog, session, signature, and semantic rules; use real Axum `Router::oneshot` with a loopback mock for request ordering, headers, body, redirects, retry, and lifetime. Keep all tests synthetic and isolated; do not read `/Users/user/.opencodex` or any real credential file.

Minimum Nyquist matrix:

| ID | Automated evidence |
|---|---|
| ANT-01 | URL table: daily/prod exact success; suffix/lookalike/plaintext/query/fragment/port/path rejection; redirect to off-origin rejected before bearer. |
| ANT-02 | Injected resolver plus concurrent credential swap; inspect every upstream request for stable account/project/token tuple through streaming completion and cancellation. |
| ANT-03 | Exact JSON envelope assertions; request ID regex; session stable only within supplied conversation/account and distinct otherwise; no prompt/account/secret substrings. |
| ANT-04 | Fresh catalog positive/negative/ambiguous/stale/cold cases; unsupported effort/model returns before resolver/network, with zero mock hits. |
| ANT-05 | Multi-round sequential calls/results preserve IDs/order; valid signatures round-trip; copied account/session, empty, orphan, duplicate, mismatched, invented signatures fail pre-dispatch. |
| ANT-06 | Mock always receives `/v1internal:streamGenerateContent?alt=sse` for both downstream modes; split bytes at every boundary; streaming is incremental, unary bounded accumulation. |
| ANT-07 | Direct/wrapped parity for text/reasoning/tools/usage/finish/provider errors; malformed JSON/SSE/UTF-8, mixed wrappers, incomplete tools, duplicate/post-terminal data, premature EOF fail closed. |
| ANT-08 | First pre-header 401 triggers exactly one same-account refresh/replay; second 401/refresh failure/post-header 401/output/tool/body failure never replays; credential file bytes unchanged. |
| Lifetime | Downstream body drop releases upstream stream, parser, tuple, and admission under bounded timeout; no detached task remains. |

Run focused filters after each workstream, then `cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features --workspace -- --test-threads=1`, and deterministic scope/docs/secret scans. There is no manual/live verification requirement; live credentials are intentionally excluded.

## Sequencing Recommendation

Wave 1: always-SSE decoder/machine terminal and bounds.  
Wave 2: canonical destination and bearer redirect safety.  
Wave 3: request-local account/project/catalog/session tuple and exact catalog admission.  
Wave 4: envelope/session and authentic tool/signature history.  
Wave 5: one pre-commit 401 refresh/replay.  
Wave 6: native gateway conformance, docs/locales, scope/security/release gates.

Keep each wave vertical and add failing characterization tests first. Do not let a later retry layer bypass strict terminal or identity validation.

## Documentation Surfaces

Update behavior statements in `README.md`, `docs/notes/antigravity-daily-host.md` or a focused Phase 11 engineering note, relevant `docs/running.md`/configuration references, and matching English/ko/ja/zh-cn files under `site/src/content/docs/`. Preserve existing warnings about Antigravity terms and the deprecated CLI. Do not modify generated `wiki/`.

## Research Gaps / Planner Checkpoints

- The repository proves the envelope shape through existing code and dated probe notes, but not a current independent live Antigravity backend contract; treat live wire details as MEDIUM confidence and keep implementation hermetic.
- No public account identifier is currently carried by `Credential::AntigravityOauth`; the planner must choose a crate-private non-secret identity source and test its lifecycle without changing credential-file schema.
- Existing catalog cache intentionally fails open; changing it to strict admission is observable behavior and must be documented and covered before any credential/network dispatch.
- Existing Antigravity retry tests explicitly preserve `Idempotent` “until Phase 11” [VERIFIED: src/adapters/gemini/mod.rs:560-569]; this is a planned change, not a regression.

## Sources and Confidence

- Repository source and tests read this session: HIGH confidence, cited inline with file/line ranges.
- `docs/notes/antigravity-daily-host.md` dated probe matrix and reference-client comparison: MEDIUM confidence for provider wire behavior; it explicitly warns when a probe does not isolate causality [VERIFIED: docs/notes/antigravity-daily-host.md:1-30,76-145].
- Phase 10 artifacts (`10-RESEARCH.md`, `10-VALIDATION.md`, `10-VERIFICATION.md`) supply already-passed decoder, semantic, retry, and lifetime patterns; reuse only the patterns named above [VERIFIED: .planning/phases/10-gemini-semantic-hardening/10-VALIDATION.md:1-80].

## Project Constraints (from AGENTS.md)

- Preserve streaming semantics; do not buffer upstream SSE unless downstream requested non-streaming.
- Keep gateway-owned errors in Anthropic shape on `/v1/messages`.
- Prefer table-driven config additions; keep Rust files focused.
- Run format, Clippy, and full workspace tests before completion.
- Update affected docs and maintained translations in the same change; never hand-edit `wiki/`.
- Ask before changing credential-file writeback; do not add secrets or generated local config.
