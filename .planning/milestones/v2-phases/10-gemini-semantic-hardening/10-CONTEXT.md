# Phase 10: Gemini Semantic Hardening - Context

**Gathered:** 2026-09-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Harden the existing `gemini` provider's Google Code Assist request lifetime, response framing, semantic translation, and replay policy so streaming and non-streaming clients observe equivalent, fail-closed behavior. This phase stays within the current provider kind, Code Assist envelope, Google OAuth source, routes, and public configuration; it does not add a provider, change credential persistence, implement Antigravity-specific policy, or introduce any Google AI Studio Web surface.

</domain>

<decisions>
## Implementation Decisions

### Identity and Request Lifetime
- **D-01:** Resolve one Google OAuth credential and its selected Code Assist project before generation dispatch, then keep that token/project pair immutable through every permitted pre-response attempt and until the response body completes or is cancelled.
- **D-02:** Keep the existing Code Assist `v1internal:{generateContent|streamGenerateContent}` destination and `{model, project, request}` envelope. Do not borrow Antigravity discovery identity, catalog, session, model-tier, header, or destination rules.
- **D-03:** Bind the bearer only through the existing `google_oauth` / `gemini` validation and destination policy. Phase 10 must not create, migrate, refresh-write, or otherwise change credential files.
- **D-04:** Keep response-lifetime state request-local. Cancellation or downstream drop must release the upstream response, parser buffers, credential ownership, and any admission/account lease through existing RAII and body-drop paths.

### Framing and Bounds
- **D-05:** Replace permissive line dropping with one incremental, byte-oriented, bounded SSE decoder that survives arbitrary chunk splits, UTF-8 code-point splits, CRLF, comments, and supported multi-line `data:` fields while preserving event order.
- **D-06:** Treat invalid UTF-8, malformed JSON data, structurally invalid supported fields, oversized chunks/events/residuals, duplicate terminal state, and bytes after a terminal as protocol errors. Unknown JSON fields may be ignored for forward compatibility, but an ambiguous supported shape must not be guessed or silently discarded.
- **D-07:** A clean streaming completion requires a structurally complete Gemini response carrying an authoritative supported `finishReason`; transport EOF and `[DONE]` are framing boundaries, not substitutes for provider completion.
- **D-08:** Use explicit crate-private bounds derived from the existing request/resource limits and exercise each boundary hermetically. Streaming success stays incremental; only a client-requested non-streaming response may accumulate content, tool state, or the upstream body, and every such accumulation is bounded.

### Semantic Parity
- **D-09:** Feed both Code Assist wrappers (`response`) and direct Gemini response objects through one semantic state machine. Streaming emits its events incrementally; non-streaming renders the final Anthropic message from the same validated state and terminal outcome rather than discarding generated events.
- **D-10:** Preserve upstream order when translating text, thinking, function calls, usage updates, and finish state. Do not merge ambiguous multiple-candidate output into one Anthropic assistant message; reject shapes outside the single supported candidate contract instead of inventing an order.
- **D-11:** Preserve function-call names, arguments, call/result pairing, and authentic non-empty `thoughtSignature` values through the existing opaque tool-use ID round trip. Never invent a signature; incomplete, invalid, orphaned, or ambiguous tool data fails before it can be represented as successful tool use.
- **D-12:** An embedded provider error is authoritative in both client modes and produces one Anthropic error outcome, never an HTTP 200 success or a later synthetic terminal. Usage and finish-reason mapping must agree between the streaming transcript and the corresponding non-streaming message.

### Retry and Recovery
- **D-13:** Classify Gemini generation as a non-idempotent POST and use the existing explicit retry-safety API. Retry only bounded, clearly transient transport failures that resolve before response headers; do not retry a returned status, an ambiguous body failure, or any failure after semantic parsing begins.
- **D-14:** Apply the Phase 9 commitment vocabulary to any Gemini redispatch/recovery seam: client-visible output or a function call closes the boundary monotonically, and no mid-stream retry, account switch, provider hop, or partial-response repair is permitted.
- **D-15:** Keep the selected identity/project pair on a permitted retry and rebuild the exact same request payload. A retry must not rediscover a project, rotate credentials, regenerate tool history, or otherwise change request meaning.
- **D-16:** Preserve existing HTTP status and Anthropic error-envelope behavior where it is already well-defined; hardening may make formerly silent malformed/truncated/embedded-error cases fail explicitly, but it must not add public configuration or a new documented provider mode.

### Claude's Discretion
Exact private type names, module decomposition, numeric byte/count caps, and whether parity assertions compare a normalized semantic trace or the rendered Anthropic outputs are implementation details. Prefer the smallest design that reuses Phase 9 commitment/cancellation primitives, existing dependencies, and the real gateway test harness.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Scope and acceptance
- `.planning/ROADMAP.md` — Phase 10 goal, dependency, success criteria, and the boundary before Antigravity work.
- `.planning/REQUIREMENTS.md` — `GEM-01` through `GEM-05`, shared `SAFE-*` invariants, documentation obligations, and explicit Google AI Studio Web / credential-writeback exclusions.
- `.planning/PROJECT.md` — milestone core value, architecture constraints, approved provider priorities, and non-negotiable exclusions.

### Prior conformance decisions
- `.planning/phases/09-provider-conformance-foundation/09-CONTEXT.md` — locked terminal, commitment, credential, continuation, cancellation, bounds, and fixture conventions Phase 10 must reuse.
- `.planning/research/PITFALLS.md` — evidenced Gemini gaps: synthetic EOF success, silently dropped malformed frames, unary/stream drift, signature integrity, account affinity, and replay risk.
- `.planning/research/ARCHITECTURE.md` — provider-specific adapter boundaries and the rule that Antigravity shares only narrow Code Assist transport machinery.
- `.planning/research/FEATURES.md` — expected Gemini Code Assist conformance surface and explicit separation from Antigravity and AI Studio Web.

### Existing operator contract
- `docs/upstreams-failover.md` — current Gemini adapter/auth routing and failover semantics that remain the public baseline.
- `site/src/content/docs/reference/configuration.md` — current English provider configuration contract; Phase 10 adds no key or provider mode.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/adapters/gemini/mod.rs`: existing endpoint/envelope selection, one-time credential resolution, HTTP dispatch, SSE relay, and unary response path; this is the narrow transport integration point.
- `src/model/gemini.rs`: existing `GeminiSseMachine`, Anthropic event rendering, content accumulation, tool signature encoding, usage/finish mapping, and Google error mapping; evolve this into the shared strict semantic machine rather than adding a parallel translator.
- `src/model/gemini_request.rs`: established Messages-to-Gemini request translation, tool/result name recovery, schema sanitization, thinking configuration, and Code Assist envelope construction.
- `src/retry.rs`: Phase 9 `Commitment` and `RetrySafety::NonIdempotentPost` primitives already express the required replay boundary.
- `tests/gemini_translate.rs` and the existing Wiremock gateway helpers: current request, tool, signature, streaming, accumulation, and error fixtures provide the baseline for parity and negative cases.

### Established Patterns
- Adapters stream upstream bodies lazily with backpressure and only collect when the inbound request explicitly asks for non-streaming output.
- A request captures one immutable runtime/config snapshot, and response bodies retain admission/account resources until completion or drop.
- Provider-owned translation stays within the provider adapter/model modules; routing and public configuration remain table-driven and unchanged for a semantic hardening phase.
- Gateway-level hermetic tests use ephemeral Axum listeners and mock only the external provider, including split chunks, transport cuts, exact headers/bodies, and cancellation.

### Integration Points
- Tighten framing and response-body ownership in `src/adapters/gemini/mod.rs`; keep Code Assist and API-key endpoint selection behavior intact.
- Make semantic validation and streaming/unary parity authoritative in `src/model/gemini.rs`; reuse request-history/signature handling from `src/model/gemini_request.rs` without introducing persistence.
- Route generation dispatch through the explicit non-idempotent retry policy in `src/retry.rs`, with tests proving no status or post-output redispatch.
- Extend focused Gemini unit tests and add real gateway/mock-upstream transcript tests for identity/project affinity, strict framing, parity, retry, terminal, and cancellation behavior.

</code_context>

<specifics>
## Specific Ideas

- Treat a captured upstream transcript as the semantic oracle: replay equivalent response objects as SSE and unary input, normalize the resulting Anthropic content/tool/usage/finish/error meaning, and assert equality while retaining incremental delivery assertions for SSE.
- Preserve the existing authentic-signature-in-tool-use-ID mechanism because it keeps continuation metadata request-local and opaque without adding a cache or durable history.
- Keep all new fixtures sanitized, bounded, deterministic, offline, and free of bearer, account, project, or private prompt material.

</specifics>

<deferred>
## Deferred Ideas

- Antigravity-specific OAuth scopes, canonical daily/production destination rules, account-bound project discovery/refresh, session metadata, catalog/model tiers, wrapper rewrites, and signature policy belong to Phase 11.
- Generic OpenAI Chat, Command Code, Cursor, exact-model OpenCode Go, release-wide provenance/live smokes, and broad documentation parity remain in their owning later phases.
- Google AI Studio Web, cookie/SAPISIDHASH authentication, browser extension/daemon/session synchronization, MakerSuite parsing, and browser/WebKit dependencies remain explicitly out of scope.
- Any credential-file writeback change, durable continuation/signature store, generalized repair layer, or mid-stream provider hopping requires separate scope and approval.

</deferred>

---

*Phase: 10-gemini-semantic-hardening*
*Context gathered: 2026-09-06*
