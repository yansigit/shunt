# Phase 12 Research: Cursor Evidence-Backed Hardening

**Status:** COMPLETE
**Date:** 2026-09-07
**Mode:** Local evidence only (no external research; no live Cursor calls; no opencodex touch)

---

## 1. Governing Decisions (verbatim from 12-CONTEXT.md)

The authoritative implementation decisions are D-01..D-11 in `.planning/phases/12-cursor-evidence-backed-hardening/12-CONTEXT.md`:

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

Orchestrator constraints for this phase: audit findings are hypotheses until source-verified; no retry added for CUR-07 beyond the existing bounded pre-output seam; no Commitment wrapper where no redispatch seam exists; legacy code/tests scoped honestly, never deleted; no usage protobuf fields invented from synthetic frames; actual evidence limitations reported. AGENTS.md gates (fmt, clippy -D warnings, full workspace tests, <500-line files, docs parity) apply. Generic Chat / Command Code config approval covers Phases 13–14 only and does not expand this phase (CONTEXT deferred).

---

## 2. Phase Requirements (REQUIREMENTS.md)

| Req | Requirement (abridged) | Research support |
|-----|------------------------|------------------|
| CUR-01 | Retain the proven AgentService Run destination until a captured/live profile proves a replacement | agent.rs:52 `https://agentn.global.api5.cursor.sh` + `/agent.v1.AgentService/Run`; no replacement evidence exists → keep destination (D-01). |
| CUR-02 | Exact, request-local model/client/endpoint/capability facts; no cross-request mutation | Per-request constants in agent.rs; plan proves concurrent requests cannot share/mutate observation state (D-02). |
| CUR-03 | Stable continuation identity + paired tool-call/result history across ordinary, compacted, recovered, multi-round turns | request.rs renders full history statelessly per turn; identity/pairing integrity proven with fixtures. **Stable history ≠ resumable stream**: CUR-03 does not mandate a resumable-stream protocol and none exists as evidence; unsupported opaque continuation must fail explicitly (D-03). |
| CUR-04 | Tools/schemas/guidance/images/arguments only in proven forms; explicit unsupported error otherwise | mod.rs `extract_cursor_tools` silently drops nameless tools and defaults missing schemas; `decode_selected_images` drops undecodable images via `.ok()` — pre-dispatch admission gap → explicit unsupported errors (D-04). |
| CUR-05 | Incremental reasoning/text/tool/usage + exactly one terminal; cancellation releases resources | mod.rs:334 hardcodes usage `input_tokens: 1` (not authentically relayed); agent.rs:319/332 push `terminal_event` on EOF/idle paths — silence/EOF must not manufacture the terminal (D-05); cancellation releases paced sender + upstream body via RAII (D-07). |
| CUR-06 | Malformed Connect/protobuf frames, decompression violations, provider errors, premature EOF are errors | `parse_connect_error` (connect.rs:252) returning None must not convert malformed/gzipped END trailers into success; `iter_fields` (agent.rs:687) silently truncates malformed protobuf. Decompression bound exists (connect.rs:118). Response-parsing seam (D-06). |
| CUR-07 | Distinguish local construction/header failures from upstream transport failures; retry/failover only pre-output and replay-safe | `AdapterFailure::{UpstreamStatus, BeforeHeaders}` classification exists; `src/retry.rs`:34,206 `TODO(#126, cursor)` marks the pre-output retry seam deliberately unfinished. No new retry machinery — classification proof + replay-safety tests only (D-08). |
| CUR-08 | No semantic repetition/no-progress heuristic without a proven transport signal | EOF/idle synthesis must not become a success or cancel heuristic; plan asserts absence and adds none (D-09, D-05). |

---

## 3. Summary

Source-verified gaps cluster into three areas:

1. **Response/terminal integrity (CUR-05 terminal + usage, CUR-06):** malformed or gzip-flagged END trailers risk None→success conversion at the caller seam; `iter_fields` silently truncates malformed protobuf; EOF/idle paths push a synthesized `terminal_event`; usage is hardcoded 1/1 instead of authentically relayed.
2. **Pre-dispatch admission (CUR-02, CUR-03, CUR-04):** unsupported tools/schemas/images are silently dropped before dispatch; continuation identity and tool pairing need fixture proof; per-request fact isolation needs concurrency proof. Distinct from the response-parsing seam above.
3. **Classification/cancellation (CUR-05 cancellation, CUR-07, CUR-08):** `AdapterFailure` classification exists; the pre-output retry seam is explicitly unfinished (TODO #126). No new retry or Commitment wrapper; prove classification, replay-safety boundaries, and RAII release. No semantic no-progress heuristic exists or is added.

**Recommended slices:** (a) terminal/trailer framing integrity, (b) pre-dispatch admission + pairing/identity fixtures, (c) classification/cancellation proof. Evidence-limited items are flagged in Section 5, not fabricated.

---

## 4. Key Verified Facts (re-verified against this worktree's source this session)

Active path per CONTEXT code_context: `src/adapters/cursor/{agent,mod,connect,request,model,sse}.rs`, `client.rs` owning shared `CursorError`, `src/retry.rs`, `src/proxy/failover.rs`.

### 4.1 Active transport — `src/adapters/cursor/agent.rs`

| Lines | Fact |
|-------|------|
| 6 | Module doc credits MIT jcode transport — provenance preserved |
| 20-23 | Pacing load-bearing: half-close after dispatch → "No exec result"; do not disturb send pacing |
| 52 | `AGENT_BASE_URL = "https://agentn.global.api5.cursor.sh"` (CUR-01) |
| 163, 200, 283 | Paced sender + periodic heartbeats while response is read (cancellation release scope, CUR-05/D-07) |
| 319, 332 | EOF/idle paths `pending.push_back(terminal_event(...))` — silence/EOF synthesizes a terminal (CUR-05/D-05) |
| 349 | `terminal_event` constructor |
| 687 | `iter_fields` — silently stops on malformed protobuf; no error surface (CUR-06) |
| 761-869 | Response-parsing call sites of `iter_fields` (response parsing, not pre-dispatch admission) |

### 4.2 Connect framing — `src/adapters/cursor/connect.rs`

| Lines | Fact |
|-------|------|
| 4-5 | `FLAG_GZIP = 0x01`, `FLAG_END = 0x02` |
| 31, 50 | End frames with empty/JSON payload returned as-is; 64 MiB frame cap |
| 116-118, 225 | Decompression bounds (64 MiB) — zip-bomb protection exists |
| 252 | `parse_connect_error(payload) -> Option<ConnectEndError>` — None on malformed payload; caller seam must not treat None as clean success (CUR-06) |

### 4.3 Adapter core — `src/adapters/cursor/mod.rs`

| Lines | Fact |
|-------|------|
| 125-126 | TODO(#170): no pre-response retry for the streaming turn — seam absent, deliberately (CUR-07/D-08) |
| 334-337 | Unary usage block hardcodes `"input_tokens": 1` — usage not authentically relayed (CUR-05/D-05) |

### 4.4 Rendering and shared classification

- `request.rs`: stateless full-history re-render each turn; its CUR-03 limitations are pairing/identity integrity, not resumable streams (CONTEXT warns against claiming structured continuation from prompt text).
- `src/retry.rs:22,34,206`: `RetrySafety` taxonomy (issue #126); cursor tightening explicitly unfinished TODO — classification vocabulary present, retry machinery not (CUR-07).
- `src/proxy/failover.rs:11`: CursorAdapter participates in failover — pre-output, replay-safety boundaries to prove (CUR-07).

### 4.5 Retained legacy code (scoped honestly, D-01)

`proto.rs`, `stream.rs`, `tool_bridge.rs`, `tool_use_xml.rs`, `response.rs`, and `client.rs`'s old `CursorHttpClient` are retained legacy code, off the active path. Not evidence for the active wire; not modified; existing tests preserved. `sse.rs`, `offload.rs`, `test_frames.rs` are active-path/support files.

### 4.6 Audit hypothesis verification

| Audit claim | Verdict | Evidence |
|-------------|---------|----------|
| iter_fields silently ignores malformed protobuf | **Confirmed** | agent.rs:687 (response-parsing seam) |
| END parser None converts malformed/gzipped trailer to success | **Confirmed risk at caller seam** | connect.rs:252 + END handling in agent.rs |
| EOF/idle may synthesize End | **Confirmed** | agent.rs:319/332 → terminal_event |
| Live usage arm unproduced; unary 1/1 synthetic | **Confirmed** | mod.rs:334-337 |
| Sender cancellation guard needs full-channel/header-wait review | **Confirmed as review item** | paced sender agent.rs:163-283; TurnGuard RAII coverage test to plan |

---

## 5. Evidence Limitations (honest; not fabricated)

### Follow-up: OpenCodex suites located and executed, 2026-09-07

The initial search was limited to Shunt and missed the adjacent OpenCodex checkout. The user authorized Cursor CLI probing and pointed to its prior suites. At OpenCodex revision `055c3ecf0de6c35f59195fc434d6b08525182b7f`, the following **schema-derived, hermetic** evidence is now available:

- `src/adapters/cursor/gen/agent_pb.ts`: AgentServerMessage field 1 is InteractionUpdate; its field 8 is TokenDeltaUpdate, whose field 1 is int32 tokens. InteractionUpdate field 14 is TurnEndedUpdate. These extend the same field 1 text/field 4 thinking envelope already used by Shunt; they do not justify changing the host.
- AgentServerMessage field 3 is ConversationStateStructure; its field 5 is ConversationTokenDetails, whose field 1 is uint32 used_tokens. This is absolute context occupancy, **not** additive output or authoritative per-turn input.
- `src/adapters/cursor/protobuf-events.ts:221` marks the assembled usage as estimated. `resolvedTurnUsage` separates output deltas from context occupancy and inferred input. Shunt must not describe derived input or cache counts as authentic upstream measurements.
- `tests/cursor-protobuf-events.test.ts:968` proves checkpoints 10000 then 10300 plus output delta 42 produce context 10300, not 20300, and inferred input 10258. Negative/overflow/absence handling still needs focused Shunt checks.
- The older `132_research_cursor_usage.md` discusses **StreamUnifiedChatWithTools**, not this Run stream; its field 27/30 claims must not be transplanted onto AgentService Run.

Executed `bun test ./tests/cursor-framing.test.ts ./tests/cursor-protobuf-events.test.ts ./tests/cursor-eof-terminal.test.ts` in the OpenCodex checkout: **83 passed, 0 failed, 260 assertions**. The top-level process inherited fresh `OPENCODEX_HOME`, no token/API-key environment variables, and a non-production port setting; HTTP/2 fixtures bound ephemeral loopback ports. Production config SHA-256 and mtime, and invalid/backup filename inventory, were identical before and after. No live provider call occurred.

Cursor CLI was not found in PATH, `/Users/user/.local/bin`, the usual application locations, or filename searches of the adjacent checkout/portable volume. CLI live evidence remains unexecuted, not failed. Existing schema/test evidence removes the claim that no usage wire fields are known; live availability remains unverified. Preserve provenance/MIT notices for substantial translations and do not copy the broader tracker/persistence/repair architecture.

| # | Limitation | Affected | Handling for PLAN |
|---|-----------|----------|-------------------|
| E1 | No live capture was made here. Adjacent OpenCodex now supplies schema-derived output-delta/context fields and 83 passing hermetic tests (follow-up above). | CUR-05 (usage half) | Plan bounded output-delta propagation and explicitly qualified derived/absent input usage; never invent complete billing/cache usage or claim live proof. |
| E2 | **No resumable-stream protocol exists or is required.** Stable continuation identity comes from stateless full-history re-render; recovered/compacted histories must be fixture-proven for identity + pairing. CUR-03 does not mandate a resumable-stream protocol. | CUR-03 | Prove identity/pairing via hermetic fixtures; no resumable-stream protocol in scope. |

## 6. Validation Architecture (nyquist_validation = true, D-10)

Narrow hermetic real-gateway/loopback fixtures, synthetic credentials, dated provenance. Testing style per requirement:

1. **Unit (response parsing):** malformed trailer bytes, gzip-flagged junk, truncated protobuf, both END flag orders → error, never clean completion (CUR-06). Valid empty trailers stay valid.
2. **Behavioral (loopback gateway):** EOF-then-no-trailer and idle-timeout streams must not emit a synthesized terminal (CUR-05, D-05).
3. **Fixture (request rendering):** ordinary, compacted, recovered, multi-round histories preserve identity + tool pairing (CUR-03).
4. **Admission (pre-dispatch):** nameless tools, missing schemas, undecodable images → explicit unsupported error (CUR-04, D-04); per-request fact isolation under concurrency (CUR-02, D-02).
5. **Classification/replay-safety:** local construction/header failures never retry candidates; upstream transport failures classified correctly; replay-safe only pre-output (CUR-07, D-08). No Commitment wrapper without a redispatch seam; no new retry machinery.
6. **Cancellation:** header-wait and both client response modes release paced sender, upstream body, parser state, gateway capacity via RAII (CUR-05/D-07).
7. **Absence assertions (CUR-08/D-09):** no semantic repetition/no-progress heuristic, no new public config, no durable continuation store.

### Wave 0 (pre-code gaps)

- W0-1: port the schema-derived usage/terminal cases with dated provenance; independently check semantic compatibility with Shunt's active envelope and qualify absent/derived metrics.
- W0-2: byte-level trailer/protobuf malformed corpora for the CUR-06 test tables.

### Test map

| Req | Tests to plan | Wave |
|-----|---------------|------|
| CUR-01 | destination const regression assertion | 1 |
| CUR-02 | request-local fact isolation tests | 1 |
| CUR-03 | identity/pairing fixtures across 4 history kinds | 1 |
| CUR-04 | unsupported-input explicit error tests | 1 |
| CUR-05 | single-terminal behavioral; usage per W0-1; cancellation release tests | 1-2 |
| CUR-06 | malformed framing/trailer rejection table | 1 |
| CUR-07 | classification + replay-safety boundary tests | 2 |
| CUR-08 | absence assertions (heuristic/config/store) | 2 |

### Gates and environment

- `cargo fmt --all --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features --workspace` (D-10).
- agent.rs pacing comments are load-bearing; surgical edits only (file near 500-line budget).
- Docs parity in same PR for observable changes (D-11); wiki not hand-edited.
- Never touch `/Users/user/.opencodex` or port 10100; no live Cursor call is authorized implicitly by this phase (D-10, AGENTS.md boundary).

## 7. Assumptions

- **A1:** Legacy retained modules (`proto.rs`, `stream.rs`, `tool_bridge.rs`, `tool_use_xml.rs`, old `CursorHttpClient`) stay untouched and off the active path through Phase 12.
- **A2:** `request.rs` stateless re-render remains the continuation mechanism; fixtures prove its identity/pairing properties rather than replacing it.
- **A3:** Connect framing bounds (flags, 64 MiB caps) are otherwise sound; only the END/trailer/None-success seam changes.

## 8. Open Questions (for PLAN/user)

1. **Q1 (W0-1):** Evidence now permits output-token deltas, but derived input/context metrics require explicit labeling and cannot be claimed as provider billing usage.
2. **Q2:** Does the cancellation review (audit item 5) get a dedicated plan item or fold into slice (c)?
3. **Q3:** Confirm the Connect standard's treatment of valid empty END trailers so rejection of malformed ones preserves the proven convention.

## 9. Sources

All local, re-verified this session:

- `src/adapters/cursor/agent.rs` (6, 20-23, 52, 163-283, 319-349, 687-869)
- `src/adapters/cursor/connect.rs` (4-5, 31, 50, 116-118, 225, 252)
- `src/adapters/cursor/mod.rs` (125-126, 334-337)
- `src/adapters/cursor/request.rs`, `model.rs`, `sse.rs`, `client.rs` (role only)
- `src/retry.rs` (22, 34, 206), `src/proxy/failover.rs` (11)
- `.planning/phases/12-cursor-evidence-backed-hardening/12-CONTEXT.md`, `.planning/REQUIREMENTS.md` (CUR-01..08), `.planning/ROADMAP.md` Phase 12, `.planning/STATE.md`, AGENTS.md/CLAUDE.md

## 10. Metadata

### Live CLI follow-up, 2026-09-07

After the user installed and authenticated Cursor CLI, `agent` and `cursor-agent` resolved to version `2026.09.02-c22c1a3`. Backed up `cli-config.json` and `agent-cli-state.json` to an owner-only directory outside the repository and verified byte equality. Copied config into a fresh temporary directory, redirected CLI config/data/store paths, and used an empty temporary workspace. The model-list command succeeded and included `composer-2.5`.

A bounded `--print --mode ask --model composer-2.5 --output-format stream-json --stream-partial-output` probe returned the requested synthetic marker and exited 0. The empty scratch workspace required explicit trust; only that newly created workspace was trusted, with ask mode retained and no force/yolo/MCP approval flags. Observed thinking deltas/completion, assistant events, and exactly one successful result event. Result usage was `inputTokens: 6793`, `outputTokens: 49`, `cacheReadTokens: 8144`, `cacheWriteTokens: 0`; duration was 3115 ms. No tool-call events were observed. Session/request IDs and credential values were not recorded.

This is **live CLI-output evidence**, not a raw Connect capture or a successful Shunt gateway test. In particular, CLI cache/input usage fields may come from aggregation or another source; they do not prove new protobuf field numbers and must not be claimed absent merely because earlier stream research lacked them. Both original CLI files remained byte-for-byte equal to their backups after the probe. Production proxy state was not used or reconfigured.

- **Agent:** gsd-phase-researcher (Phase 12)
- **Mode:** local evidence only; zero external lookups; no live calls
- **Safety:** no opencodex touch, no commits, no source/config edits
- **Valid until:** 2026-10-07
