# Phase 14: Command Code Product Separation — Context

**Gathered:** 2026-09-08
**Status:** Ready for planning
**Mode:** Autonomous synthesis; recommendations selected by the agent under existing user authorization, not new user answers.

<domain>
Implement distinct `commandcode` API-key Chat and `command-code` subscription NDJSON products. Meet CCK-01–03 and CCS-01–08 without changing existing provider settings or credential-file writeback. Generic Chat is already implemented and verified in Phase 13.
</domain>

<decisions>
## Implementation Decisions

### Product and credential boundary
- **D-01:** The API-key product is a table-driven preset over `openai_chat` at canonical `/provider/v1/chat/completions`. The subscription product has its own `command_code` transport and read-only auth mode. Never infer a product from a bearer or share subscription acquisition with Chat.
- **D-02:** Phase 13–14 public additions are explicitly approved. Settings and available credentials were backed up outside the repository with owner-only permissions. No creation, refresh, migration, repair, or writeback of Command Code credential files is authorized.
- **D-03:** An explicitly configured subscription environment source takes precedence over an existing CLI file. A present but invalid explicit value fails closed rather than silently changing identity. Research must establish the exact supported source names and file schema from code/captured evidence; do not inspect real credentials merely to discover a schema.
- **D-04:** Subscription credentials may be attached only to the canonical HTTPS Command Code origin and `/alpha/generate`. Validate before credential access and again at dispatch; refuse redirects. Loopback fixtures must not introduce a public bypass permitting live subscription tokens off-origin.

### Session and history
- **D-05:** Preserve exact, evidence-backed model/effort combinations and required proprietary envelope/identity headers. Unknown combinations fail before generation; do not infer capability from model-family names or silently clamp effort. Version facts require dated provenance, not an invented current version.
- **D-06:** Capture one credential identity and stable opaque conversation session for the request and any permitted replay. Scope affinity to credential and conversation; expose no raw client identity, prompt, credential, filesystem path, or account name. Prefer existing stateless session primitives; no durable history or global conversation registry.
- **D-07:** Preserve adjacent tool-call/result units with authentic identities, supported result images, and original ordering. Explicitly represent missing results as non-executed errors and retain orphan results as visible context, as this product requires. Do not alter Phase 13's stricter Chat pairing policy. Reject ambiguous duplicate identities or unsupported opaque history before dispatch.

### Terminal and replay safety
- **D-08:** Both client modes consume one incremental bounded NDJSON decoder and checked semantic state machine. Stream text/reasoning incrementally; bound tool assembly and unary accumulation. Require one trustworthy supported finish; malformed/oversized records, invalid UTF-8, junk-only responses, incomplete tools, provider error finishes, duplicate terminals, and premature EOF fail closed.
- **D-09:** Bound record/residual bytes, cumulative semantic output, tool count and argument bytes, diagnostics, and read deadlines. Tests exercise below/at/above byte boundaries and distinguish wire framing from semantic budgets. No heuristic successful completion or response repair.
- **D-10:** Use the existing conservative non-idempotent retry vocabulary. Only proven pre-connect failure may retry/fail over by default; no ambiguous post-send replay, output replay, or replay-unsafe tool activity. Any evidence-backed stronger exception requires an exact test and must retain selected identity/session. No subscription refresh or account rotation is added.
- **D-11:** Cancellation before headers and during either response mode must drop upstream work, parser state and gateway capacity through ownership. No detached recovery task or persistent lease registry.

### Evidence and completion
- **D-12:** Build a narrow real-gateway tracer before expansion. Independently test both products' auth, destination and models; reuse applicable Chat scenarios, and exercise subscription normal/reasoning/tools/subagents/continuation/long-context/cancellation/error/terminal matrices. Use synthetic fixtures and record pinned source provenance; do not label source-derived fixtures as live captures.
- **D-13:** All stateful test/build/smoke trees inherit fresh isolated `OPENCODEX_HOME` and non-10100 ports. Verify production config metadata/hash and backup inventory before/after with the established isolation wrapper. Never parse, launch, test or reconfigure against `/Users/user/.opencodex` or port 10100.
- **D-14:** Preserve existing tests. Completion requires format, Clippy, full workspace tests and owned CLI/curl smoke. Update affected README/docs/site English and ko/ja/zh-cn copies together; never hand-edit wiki. Computer access was blocked previously; CLI fallback is approved and is not a passed visual evaluation. Live availability remains a separately labelled Phase 16 gate.

### Claude's Discretion
Private module layout, focused helper reuse, numeric bounds consistent with existing constraints, fixture decomposition, and implementation sequencing within tracer-first delivery. No new dependencies or catalogs without demonstrated necessity.
</decisions>

<canonical_refs>
- `.planning/ROADMAP.md` — Phase 14 boundary and success criteria.
- `.planning/REQUIREMENTS.md` — CCK-01–03, CCS-01–08 and shared safety/preservation requirements.
- `.planning/PROJECT.md` — lean behavior port and exclusions.
- `.planning/research/SUMMARY.md` — explicitly resolves the outdated FEATURES.md claim that both products use NDJSON.
- `.planning/research/UPSTREAM-ISSUES-HANDOFF.md` — product split and source-evidence priorities.
- `.planning/research/ARCHITECTURE.md` — dedicated transport, host-pinned auth and integration points.
- `.planning/research/PITFALLS.md` — pairing, terminal, replay and credential hazards.
- `.planning/phases/13-generic-openai-chat-completions/13-CONTEXT.md` — inherited Chat isolation and request semantics.
- `docs/openai-chat-translation.md` — completed reusable Chat contract.
- `docs/upstreams-failover.md` — existing conservative fallback semantics.
- `site/src/content/docs/reference/configuration.md` — public configuration conventions.
</canonical_refs>

<code_context>
`src/config.rs`, `src/config/presets.rs`, `src/config/upstreams.rs` provide typed kind/auth validation and table-driven presets. `src/auth/mod.rs` resolves credentials; new subscription reads must not use refresh/write helpers. `src/routing.rs`, `src/proxy/failover.rs`, `src/proxy/capability.rs` integrate dispatch and eligibility. `src/adapters/openai_chat/` and `src/model/openai_chat_request/endpoint.rs` already support an API path prefix such as `/provider/v1`; do not duplicate Chat translation. `src/retry.rs`, `src/config/session.rs`, and existing adapter lifetime wrappers supply narrow reusable safety primitives. The September 5 codebase maps predate Chat and are advisory; current source is authoritative.
</code_context>

<specifics>
Use only `/Users/user/.codex/worktrees/0466/shunt` for edits and command workdirs; default checkout is not the implementation target. Existing `.planning/config.json` and `.gsd/` changes belong to the user. Read-only adjacent OpenCodex source reconnaissance is allowed, but its production state is not. All delegated models have explicit high thinking effort.
</specifics>

<deferred>
Exact OpenCode Go tuples: Phase 15. Opt-in live release evidence: Phase 16. Credential writeback, durable history, generalized repair/catalog platform and Google AI Studio Web remain excluded.
</deferred>
