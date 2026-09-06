# Phase 9: Provider Conformance Foundation - Context

**Gathered:** 2026-09-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Establish and exercise the shared conformance, safety, retry-commitment, credential, cancellation, and terminal invariants that preserve the existing ChatGPT/Codex and Vercel behavior and constrain all later provider phases. This phase may add test support and narrowly scoped internal helpers needed to make those invariants explicit, but it does not add a provider, public configuration, or new documented provider semantics.

</domain>

<decisions>
## Implementation Decisions

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

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `tests/passthrough.rs`, `tests/inbound_codex_endpoint.rs`, `tests/codex_multi_account.rs`, `tests/codex_websocket_fallback.rs`, `tests/retry.rs`, and collaboration/compaction suites already exercise real gateway paths with Wiremock upstreams.
- `src/proxy.rs`, `src/proxy/failover.rs`, `src/retry.rs`, `src/accounts.rs`, `src/adapters/responses/`, and `src/auth/` own the relevant commitment, retry, account, credential, and response-lifetime behavior.
- Existing `TestGateway`, temporary credential-store, exact header/body matchers, delayed/chunked responders, and RAII guards are the preferred fixture primitives.

### Established Patterns
- Integration tests start real Axum listeners on ephemeral loopback ports and mock only external provider endpoints.
- Streaming assertions consume chunks incrementally and use terminal events or dropped transports to distinguish completion from truncation.
- Credentials are represented with deliberately synthetic split strings and absence assertions so secret scanners and diagnostics cannot mistake fixtures for live tokens.
- Public errors are Anthropic-shaped except on the inbound Codex endpoint, where gateway-owned errors remain OpenAI Responses-shaped.

### Integration Points
- Add conformance coverage beside the existing provider-specific suites rather than creating an alternate gateway harness.
- Centralize a production helper only if both retry and failover callers can adopt it without changing their public behavior.
- Phase verification must map every PRES and SAFE requirement to behavioral evidence and retain the later-phase ownership boundaries from `REQUIREMENTS.md`.

</code_context>

<specifics>
## Specific Ideas

- Make the preservation suite useful as the baseline later Phase 10-15 work can reuse, especially around commitment, terminal, cancellation, and credential assertions.
- Keep tests deterministic, bounded, offline, and inexpensive enough for the normal all-features workspace CI run.

</specifics>

<deferred>
## Deferred Ideas

- Provider-specific semantic hardening belongs to Phases 10-15.
- Full release provenance, live smoke orchestration, translated documentation parity, and release-wide negative-scope checks belong to Phase 16.
- Google AI Studio Web and any credential-file writeback change remain outside the milestone.

</deferred>
