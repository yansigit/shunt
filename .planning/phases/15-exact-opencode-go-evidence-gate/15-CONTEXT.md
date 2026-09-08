# Phase 15: Exact OpenCode Go Evidence Gate - Context

**Gathered:** 2026-09-08
**Status:** Ready for planning
**Mode:** Autonomous continuation; public-surface approval received in this task

<domain>
## Phase Boundary

Expose only exact OpenCode Go model/wire tuples with dated captured or live
evidence and hermetic conformance. Reject unsupported selections before
credential lookup or dispatch. Zero admitted tuples is a valid outcome, not
permission to fabricate support. Existing generic provider behavior is preserved.
</domain>

<decisions>
## Implementation Decisions

### Public surface and identity
- **D-01:** User explicitly approved new opt-in OpenCode Go configuration after Phase 14, with backups first and no changes to existing settings or credential writeback. Fresh owner-only, byte-verified backups of project planning settings and the available Codex credential file are outside the repository at `/Users/user/shunt-phase15-backup-YQSxN3`. No production settings will be migrated or reconfigured.
- **D-02:** Use the smallest explicit table-driven product selection that permits pre-credential rejection. Never infer Go from a bearer, model family, or generic provider nickname; preserve existing generic Chat, Responses and Anthropic modes.
- **D-03:** Admission identity includes exact model, canonical destination, wire, effort and documented capabilities. Unknown, ambiguous, family-inferred, unsupported and failed tuples fail before credential lookup or network dispatch, including fallback and inbound Codex paths.

### Evidence and supported behavior
- **D-04:** Evaluate the user's candidates `glm-5.3-flash`, `omen-alpha`, `muse-spark-1.3-contributor`, and `deepseek-v4-flash` as candidates, not automatic support. Keep source-only claims, sanitized captures, live observations, and hermetic assertions distinct. Record unknown fields explicitly.
- **D-05:** Each candidate record includes date, exact model, canonical destination, wire, headers, context, modalities, effort, tools, filtering, terminal behavior and capture/live provenance. Source comments and successful host subagent dispatch do not prove Shunt wire compatibility.
- **D-06:** Admit only after exact hermetic plus credential-safe captured/live evidence passes. An empty admitted set is valid. Do not build dormant multi-wire dispatch or session infrastructure solely for hypothetical future tuples.
- **D-07:** Every admitted tuple must reuse its matching existing wire contract and send a stable opaque, conversation-scoped `x-opencode-session` only to the canonical Go destination. No prompt-derived identity, raw account/session disclosure, durable cache, or redirect credential forwarding. With zero admitted tuples no credential or session header is emitted.

### Safety and delivery
- **D-08:** Keep fail-closed bounded terminals, conservative non-idempotent retry and ownership cancellation. Never import permissive EOF completion, silent repair, dynamic catalog machinery or family aliases.
- **D-09:** Use only `/Users/user/.codex/worktrees/0466/shunt` for implementation. Preserve user `.planning/config.json` and `.gsd/` dirt. Every stateful test/build/smoke process tree runs through `node /tmp/shunt-phase12-isolated-run.cjs` with fresh OPENCODEX_HOME and non-10100 ports; production config mtime/SHA/inventory must remain unchanged. No test/config parser touches live OpenCodex state.
- **D-10:** Tracer-first real-gateway negative admission evidence, focused conformance, format, warnings-denied Clippy, full workspace tests and owned CLI smoke are required. Preserve existing tests and dependencies. Public behavior/docs update together in README/docs/site and en/ko/ja/zh-cn; generated wiki stays untouched.
- **D-11:** Bounded live smoke is a separately opt-in release activity. Public documentation/source inspection is permitted; do not inspect credential contents to discover schemas, silently bill broad live model sweeps, or call a production proxy. Never describe CLI/static evaluation as a Computer visual pass.

### Claude's Discretion
Private names, compact module layout, exact additive public spelling consistent
with existing conventions, fixture decomposition, evidence-led candidate scope,
and numeric bounds within current limits. These implement the approved boundary;
they do not authorize relaxation of the evidence threshold.
</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/config/presets.rs` supplies table-driven product defaults.
- `src/routing.rs`, `src/proxy/capability.rs` and `src/proxy/failover.rs` resolve and admit routes before provider dispatch.
- Existing Chat, Responses and Anthropic contracts provide protocol machinery; `src/server.rs` has injectable credential/client seams for real-router tests.

### Established Patterns
- Distinct opt-in product selection, exact model admission and zero-lookup negative fixtures already exist for Command Code and Antigravity.
- No new parser or runtime dependencies; explicit provenance and four-language docs are standard.

### Integration Points
- Provider config/preset validation, primary/fallback route admission, inbound Codex compatibility selection, count-tokens boundary, docs/provider guides.
</code_context>

<specifics>
## Specific Ideas
Evaluate local pinned OpenCodex evidence without executing it. Agent model
preferences do not establish endpoint/model protocol facts. Every subagent uses
explicit high thinking effort; use Luna when the requested routed models fail.
</specifics>

<deferred>
## Deferred Ideas
General catalogs, billing/pricing infrastructure, durable sessions and unsupported
wire adapters remain out of scope. Phase 16 owns cross-provider release acceptance.
</deferred>
