# Phase 11: Antigravity Protocol and Credential Hardening - Context

**Gathered:** 2026-09-07
**Status:** Ready for planning
**Mode:** Autonomous synthesis from locked roadmap, requirements, prior decisions, and upstream evidence

<domain>
## Phase Boundary

Harden Shunt's existing native HTTP Antigravity provider on the shared Cloud Code Assist/Gemini transport. Deliver canonical HTTPS destination safety, request-local account/token/project/session affinity, exact envelope and model/effort admission, authentic tool/signature continuation, one strict always-SSE response contract for both client modes, and at most one account-bound pre-commit 401 refresh/replay. Keep the deprecated local `agy` CLI transport available but do not treat it as native protocol evidence or expand it in this phase.

</domain>

<decisions>
## Implementation Decisions

### Destination and Credential Lifetime
- **D-01:** Native Antigravity credentials may leave Shunt only for the canonical daily or production HTTPS Cloud Code Assist origins. The initial destination and every redirect boundary must fail closed before a bearer can reach any other origin; lookalike, suffix, plaintext, query, and fragment variants are rejected.
- **D-02:** Select one Antigravity account identity, bearer, discovered project, catalog view, and opaque conversation session before dispatch, then retain that tuple through the entire response lifetime and any permitted replay. A login/account swap or concurrent discovery must never pair an old project/session with a new token.
- **D-03:** Preserve existing credential-file behavior exactly. Phase 11 may read and use the existing store and existing refresh machinery, but must not add a new credential file, migration, refresh-token rotation rule, project writeback rule, or unrestricted account rotation policy.
- **D-04:** Reuse request-local ownership and RAII for cancellation. Downstream drop must release the upstream body, semantic parser state, account/credential ownership, and gateway admission without background replay.

### Envelope, Session, and Model Admission
- **D-05:** Emit the proven Antigravity agent envelope exactly: model, project, Antigravity client metadata, fresh `agent-<uuid>` request ID, request type, request body, and a stable opaque conversation session. Do not expose prompt text, task identifiers, account names, or credential material in the session value.
- **D-06:** Session identity is stable only inside the intended conversation/account scope and distinct across unrelated conversations. Do not derive global identity solely from the first user text or persist conversation state durably.
- **D-07:** Fresh per-account catalog evidence is authoritative for exact Antigravity model/effort admission. Unsupported, ambiguous, cold-unknown, or stale-only negative combinations fail before credentials are attached or network inference begins; do not guess effort from names when exact evidence is unavailable.
- **D-08:** Preserve explicit request effort precedence only when the selected exact catalog tuple supports it. Do not silently clamp or rewrite unsupported user choices, Claude/GPT IDs, or bare Gemini IDs into a different product contract.

### Always-SSE Semantics and Bounds
- **D-09:** Native Antigravity always calls `streamGenerateContent?alt=sse`, including when the downstream client requested non-streaming output. Both client modes consume one incremental, byte-oriented, bounded SSE decoder and one checked semantic state machine.
- **D-10:** Streaming clients receive text, reasoning, sequential tools, usage, and terminal events incrementally. Only an explicitly non-streaming downstream request may accumulate the validated semantic result, and every byte/event/tool/argument/signature/residual collection has an exercised bound.
- **D-11:** A clean result requires a complete supported wrapper and one authoritative provider terminal. Invalid UTF-8, malformed JSON/SSE, mixed or incomplete wrappers, incomplete tools/arguments, embedded provider errors, duplicate terminal state, post-terminal data, and premature EOF fail closed in both modes.
- **D-12:** Streaming and non-streaming renderings of the same captured transcript must be semantically equivalent for ordered text, reasoning, tools, authentic signatures, usage, finish reason, and provider error classification.

### Tool and Signature Authenticity
- **D-13:** Preserve sequential assistant tool-call and user tool-result history with stable call identities and original order. Validate the complete call/result relation before emitting any batch; never pair by position when an authentic identity exists.
- **D-14:** Replay only non-empty thought signatures obtained from the matching Antigravity call and bound to the same account and conversation session. A signature or encoded tool ID copied across accounts/sessions, invented locally, orphaned, duplicated, or attached to the wrong call fails before dispatch.
- **D-15:** External seed/input items without a genuine tool-result contract are classified separately from tool results. Do not apply a blanket empty-call-ID rejection that breaks valid client seed items, and do not invent missing tool identities.

### Refresh, Retry, and Recovery
- **D-16:** An Antigravity 401 may trigger at most one forced refresh and replay on the same account, with the refreshed bearer re-bound to that account's validated project/catalog/session tuple. A second 401 or refresh failure terminates the attempt.
- **D-17:** Refresh/replay is permitted only before response headers, client-visible output, or replay-unsafe tool activity. Returned transient statuses, ambiguous post-send failures, body failures, parser failures, and any mid-stream condition never retry, rotate accounts, hop providers, or synthesize repair.
- **D-18:** Strict SSE/EOF/terminal/cancellation lands before identity, catalog, signature, and 401-replay layers so later recovery cannot be built on permissive completion semantics.

### Scope and Delivery
- **D-19:** Port narrow, testable protocol invariants from current evidence, not OpenCodex's generalized caches, storage, dashboards, catalog platform, global mutable model state, cross-account project reuse, process scraping, or heuristic no-progress cancellation.
- **D-20:** Prefer focused hermetic fixtures through the real Axum gateway and loopback upstreams. All credentials, projects, accounts, signatures, sessions, and prompts are visibly synthetic; tests make no live calls and diagnostics stay redacted.
- **D-21:** Observable provider behavior and setup changes update README, engineering docs, site English, and maintained ko/ja/zh-cn copies in the same phase. Generated `wiki/` is never hand-edited.

### Claude's Discretion
Exact crate-private type names, module boundaries, cache/single-flight implementation, numeric bounds derived from existing limits, and test-file decomposition are implementation details. Prefer the smallest design that reuses Phase 9 commitment/retry primitives and Phase 10's checked Gemini semantic/SSE machinery without adding dependencies.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/adapters/gemini/mod.rs` is the native Antigravity transport path and already resolves one credential, builds `/v1internal`, captures request data in retry closures, and uses bounded Gemini response translation.
- `src/model/antigravity_request.rs` already shapes the agent envelope, request ID, session, and catalog-based model/effort mapping.
- `src/auth/antigravity/auth.rs`, `catalog.rs`, and `version.rs` already provide bounded OAuth refresh/discovery/onboarding, project-aware catalog caching, destination normalization, and client fingerprinting.
- Phase 9 supplies explicit retry safety, commitment, failover, cancellation, and destination-bound credential patterns; Phase 10 supplies checked Gemini tool/signature semantics and bounded SSE parsing.

### Established Patterns
- Native Antigravity is `ProviderKind::Antigravity -> AdapterKind::Gemini`; `src/adapters/antigravity/*` is the deprecated local `agy` CLI and is not the native HTTP path.
- Public config validation already restricts `antigravity_oauth` to Antigravity kind plus canonical HTTPS daily/production hosts, and production inference normalizes to the daily host.
- Existing auth refresh uses a redirect-hardened client, file locking, and compare-and-swap writeback. Those persistence semantics are a preservation boundary, not a template for new writes.
- Streaming paths stay lazy; non-streaming collectors and diagnostic bodies are explicitly capped. Gateway-owned errors on `/v1/messages` retain the Anthropic error shape.

### Integration Points
- Harden inference redirect behavior and request-lifetime capture around `src/adapters/gemini/mod.rs` without changing the public adapter API.
- Add explicit account/session affinity at the credential/request boundary in `src/auth/mod.rs`, `src/auth/antigravity/*`, and `src/model/antigravity_request.rs` while keeping secrets redacted.
- Reuse `src/adapters/gemini/sse.rs` and `src/model/gemini.rs` for one always-SSE checked path; extend only where Antigravity wrappers or policy genuinely differ.
- Add native Antigravity gateway conformance coverage separate from CLI tests; retain existing `tests/antigravity_catalog.rs` and socket-based process tests for their current scopes.

</code_context>

<specifics>
## Specific Ideas

- Land strict always-SSE EOF/terminal/cancellation evidence first, then destination and account/project/session affinity, exact catalog admission, tool/signature authenticity, and one bounded 401 refresh/replay.
- Include redirect lookalikes, project/token swap races, stale/unknown catalog cases, cross-account/session signature replay, repeated 401, post-output 401/error, and downstream-drop capacity release.
- Treat the current OpenCodex and upstream issue corpus as dated evidence, but describe hermetic tests as hermetic rather than live proof.

</specifics>

<deferred>
## Deferred Ideas

- Credential-file writeback changes, new migration behavior, unrestricted multi-account rotation, cross-account project reuse, API quota scraping/probing beyond required request admission, durable conversation/signature persistence, and generalized repair remain out of scope.
- Antigravity CLI parity/removal is deferred until the native provider independently proves every required behavior.

</deferred>
