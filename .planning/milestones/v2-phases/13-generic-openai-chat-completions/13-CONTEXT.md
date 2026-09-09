# Phase 13: Generic OpenAI Chat Completions — Context

**Gathered:** 2026-09-08
**Status:** Ready for planning
**Mode:** Autonomous synthesis; no new interactive answers invented.

<domain>
Add an independent OpenAI Chat Completions transport for Anthropic Messages
ingress with bounded request, JSON/SSE translation, stable tools, API-key
destination safety, cancellation and hermetic conformance. Preserve existing
Responses and Anthropic behavior.
</domain>

<decisions>
- **D-01:** Add an independent Chat provider/adapter using existing provider settings and API-key conventions, not a Responses mode. Phase 13–14 public additions are already approved. Settings and available credentials were backed up outside the repository; no new credential writeback is authorized.
- **D-02:** Accept an unambiguous API root and emit exactly one `/chat/completions` path. Reject malformed URLs, userinfo, query and fragment ambiguity; never silently reinterpret Responses endpoints.
- **D-03:** Use only the configured provider API-key source, strip inbound credential slots, and refuse credential-bearing redirects outside the selected destination. No new credential persistence or account pool.
- **D-04:** Translate system/user/assistant text, supported images, model and generation controls without Responses-only fields. Explicitly reject unsupported representations rather than silently dropping them.
- **D-05:** Preserve tool declarations, supported tool choice, parallel calls and paired results with original IDs. Reject orphan, duplicate, missing identity and incomplete object arguments; never guess pairing or synthesize results.
- **D-06:** Stream text/reasoning incrementally, assembling interleaved indexed tool deltas under bounds. Stable identities and ordered content must agree with equivalent JSON output. Ignore unknown fields only when supported shapes remain unambiguous.
- **D-07:** Require one trustworthy supported finish/terminal. Malformed events, UTF-8, incomplete tools, residual bytes, embedded errors, duplicate terminals and premature EOF fail closed. Use one checked semantic contract for both output modes.
- **D-08:** Bound events, residuals, tool count, arguments, aggregation, diagnostics and read deadlines. Cancellation before headers and during either response mode releases upstream and capacity through ownership, not a global registry.
- **D-09:** Generation is non-idempotent. Local errors and ambiguous post-send failures cannot trigger retries/failover; never redispatch after output or replay-unsafe tools. Reuse existing safety vocabulary, no new retry stack.
- **D-10:** Build a narrow real-gateway tracer then normal, streaming, image, tool-heavy, long-context, auth, malformed, truncation, error and cancellation conformance. Synthetic fixtures are not live evidence. Stateful command trees use fresh isolated OPENCODEX_HOME and non-10100 ports; production home is never parsed or tested.
- **D-11:** Update affected README/docs/site English and ko/ja/zh-cn translations during implementation. Never edit wiki. Preserve tests; format, clippy, full workspace and smoke checks are completion gates.

### Claude's Discretion
Private module names, exact bounds derived from existing limits, narrow helper
reuse and test layout. Prefer existing dependencies and the smallest design.
</decisions>

<canonical_refs>
- `.planning/ROADMAP.md` — Phase 13 goals and later-phase boundaries.
- `.planning/REQUIREMENTS.md` — CHAT-01–09 and shared SAFE/PRES guarantees.
- `.planning/PROJECT.md` — lean behavior port, exclusions and MIT provenance.
- `.planning/phases/09-provider-conformance-foundation/09-CONTEXT.md` — shared safety decisions.
- `.planning/research/ARCHITECTURE.md` — missing transport boundary.
- `.planning/research/PITFALLS.md` — tool, terminal, credential and retry hazards.
- `docs/upstreams-failover.md` — existing fallback contract.
- `site/src/content/docs/reference/configuration.md` — operator configuration.
</canonical_refs>

<code_context>
Axum/Tokio/reqwest and immutable AppState are established patterns.
`src/config.rs` owns ProviderKind/auth validation; `src/routing.rs` owns
AdapterKind; `src/proxy/failover.rs` owns dispatch and credential stripping;
`src/proxy/capability.rs` filters incompatible targets. These are integration
points, not permission to change existing provider semantics.
`src/auth/mod.rs` provides configured API-key reading, `src/retry.rs` replay
policy, and `src/adapters/gemini/sse.rs` bounded byte-framing patterns.
Reuse narrow error, keepalive and response-lifetime primitives, not the Responses
translator wholesale. Router tests use loopback upstreams and synthetic auth.
</code_context>

<specifics>
Research exact finish/usage/tool-delta conventions before implementation.
Adjacent OpenCodex may supply behavior evidence, not generalized storage/repair.
Every shell command must explicitly use `/Users/user/.codex/worktrees/0466/shunt`;
all apply_patch targets must be absolute to avoid main-checkout writes.
</specifics>

<deferred>
Command Code API-key naming and subscription NDJSON/session/auth: Phase 14.
Exact OpenCode Go: Phase 15. Opt-in isolated live tests and release audit: Phase 16.
Google AI Studio Web, credential writeback changes, durable history, generalized
repair/catalogs and new public retry knobs remain excluded.
</deferred>
