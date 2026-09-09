# Phase 12: Cursor Evidence-Backed Hardening — Context

**Gathered:** 2026-09-07
**Mode:** Autonomous synthesis of approved requirements and prior decisions

<domain>
Harden the existing Cursor AgentService Run transport in place. Preserve the proven destination and credential persistence. Deliver exact request-local protocol facts, stable supported continuation, authentic paired tool history, explicit unsupported errors for unproven input forms, strict bounded Connect semantics, cancellation, and conservative failure classification. Do not import the retired transport or OpenCodex's platform architecture.
</domain>

<decisions>
## Implementation Decisions

- **D-01:** Keep `https://agentn.global.api5.cursor.sh/agent.v1.AgentService/Run` as the default. No replacement destination without captured or live evidence. Retired `api2` code is not evidence for the active wire.
- **D-02:** Capture the exact model, mode, fast flag, client version, endpoint, and proven capabilities per request. Never let a model observation mutate another request's facts or infer wire support from model families.
- **D-03:** Preserve stable conversation identity and authentic tool call/result pairing through supported ordinary, compacted, recovered, and multi-round histories. Never fabricate missing result identities, repeat execution, or add durable history. Unsupported opaque continuation must fail explicitly, not silently become a fresh conversation.
- **D-04:** Tools, schemas, guidance, images, and argument representations need active-wire fixtures or captured evidence. Reject unsupported forms before dispatch; do not silently drop invalid images, schemas, or call/result relations, and do not revive the retired tool bridge by assumption.
- **D-05:** Stream reasoning, text, tools, and usage incrementally. Clean completion needs one authoritative supported provider terminal; silence, semantic repetition, transport EOF, and malformed input cannot manufacture success.
- **D-06:** Connect flags, protobuf fields, gzip output, frame residuals, queues, tools, arguments, and unary accumulation remain explicitly bounded. Malformed framing, provider errors, decompression violations, duplicate terminals, post-terminal data, and premature EOF are errors.
- **D-07:** Cancellation before headers and during either client response mode releases the paced sender, upstream body, parser state, and gateway capacity. Use ownership and RAII, not a global registry or detached recovery task.
- **D-08:** Local construction/header failures never become transport-retry or fallback candidates. Any permitted retry/failover is bounded, pre-output, and replay-safe; ambiguous post-send failures and emitted tools cannot trigger redispatch.
- **D-09:** Add no semantic no-progress/repetition heuristic, new public config, credential writeback, browser/AI Studio Web surface, generalized model catalog, or durable continuation store.
- **D-10:** Use narrow hermetic real-gateway/loopback fixtures with synthetic credentials and dated provenance. Synthetic tests establish conformance, not live availability. Preserve all existing tests and run format, Clippy, and full workspace tests before completion.
- **D-11:** Update affected English README/docs/site and maintained ko/ja/zh-cn copies for observable changes. Never hand-edit generated wiki. Preserve the user's `.planning/config.json` and `.gsd/` data.

### Claude's Discretion
Private type names, decomposition, and numeric bounds derived from existing limits. Prefer existing Connect decoder, semantic commitment vocabulary, bounded offload, and RAII primitives. An unproven wire capability must be marked unsupported rather than implemented from guesswork.
</decisions>

<code_context>
The active path is `src/adapters/cursor/mod.rs` → `agent.rs`, using `connect.rs`, `request.rs`, `model.rs`, and `sse.rs`. `client.rs` also owns the shared CursorError but its old CursorHttpClient is off the active path. `proto.rs`, `stream.rs`, and tool-bridge machinery are retained legacy code; distinguish them in research. Existing `agent.rs` credits the MIT jcode transport; preserve provenance. Existing request rendering flattens history; check its limitations against CUR-03 rather than claiming structured continuation from prompt text.
</code_context>

<canonical_refs>
- `.planning/ROADMAP.md`, Phase 12, and `.planning/REQUIREMENTS.md`, CUR-01 through CUR-08
- `.planning/PROJECT.md` and prior Phase 9–11 CONTEXT.md decisions
- `.planning/research/ARCHITECTURE.md`, Cursor section; `.planning/research/PITFALLS.md`, Cursor section
- `src/adapters/cursor/agent.rs`, `mod.rs`, `request.rs`, `connect.rs`, `model.rs`, `sse.rs`
- `src/retry.rs`, `src/proxy/failover.rs`, and existing provider lifetime fixtures
</canonical_refs>

<specifics>
Prove strict transport and cancellation first, then request-local admission and continuation/tool integrity. Explicitly distinguish existing supported native MCP tool handoff from ordinary assistant completion. An idle timer may signal an error, but is not proof of successful completion. No live Cursor call is authorized implicitly by this phase.
</specifics>

<deferred>
Generic Chat, Command Code, exact OpenCode Go evaluation, and release-wide conformance remain in Phases 13–16. Their public configuration approval does not expand Cursor scope. Settings and available credentials were backed up outside the repository before provider additions; no new credential-file writeback is authorized.
</deferred>
