# Phase 12: Cursor Evidence-Backed Hardening — Pattern Map

**Mapped:** 2026-09-07
**Files analyzed:** 9
**Analogs found:** 9 / 9 (all role-match or exact; Phase 12 is in-place hardening, not new-file creation)

## File Classification

Phase 12 modifies no new subsystem; without any new files, analogs are the active-path modules themselves. Every analog below is git-TRACKED (verified via `git ls-files`).

| Target (modify/prove) | Role | Data Flow | Closest Analog (tracked) | Match Quality |
|-----------------------|------|-----------|--------------------------|---------------|
| `src/adapters/cursor/agent.rs` (response/terminal seam, CUR-05/06) | adapter (streaming) | streaming | `src/adapters/cursor/agent.rs` itself: `terminal_event` :349-355, `ReadState::ingest` loop :285-330 | exact (self) |
| `src/adapters/cursor/agent.rs` `iter_fields` malformed-protobuf surface (CUR-06) | utility (parse) | transform | `src/adapters/cursor/connect.rs` `decode_frame` error plumbing :31-118 | role-match |
| `src/adapters/cursor/connect.rs` END/trailer None-success seam (CUR-06) | utility (framing) | transform | self: `parse_connect_error` :252-278 + flag constants :4-5, 64 MiB bound :116-118 | exact (self) |
| `src/adapters/cursor/mod.rs` pre-dispatch admission of tools/images (CUR-04) | adapter (admission) | request-response | self: `extract_cursor_tools` :256-278, `decode_selected_images` :176-190 | exact (self) |
| `src/adapters/cursor/mod.rs` unary usage relay (CUR-05/W0-1) | adapter | request-response | self: hardcoded usage block :334-339; schema evidence from OpenCodex `gen/agent_pb.ts` (external, provenance-quoted only) | role-match |
| `src/adapters/cursor/request.rs` identity/pairing fixtures (CUR-03) | request renderer | transform | self: `mod tests` :326+; stay stateless per A2 | exact (self) |
| `src/retry.rs` classification/replay-safety proof (CUR-07) | driver | batch/control | self: `RetrySafety` :142-155 (incl. `ConnectOnly` precedent from Phase 11 Antigravity, D-17) ; `tests/retry.rs` | exact |
| `src/adapters/cursor/mod.rs` cancellation/RAII proof (CUR-05/D-07) | adapter (streaming) | streaming | `src/adapters/cursor/agent.rs` `TurnGuard` :244-252 + `CursorAgentTurn` ownership :254-370 | exact |
| New/extended hermetic tests (CUR-01..08) | test | all | `src/adapters/cursor/mod.rs` wiremock harness :526-558; `src/adapters/cursor/connect.rs` byte-corpora helpers :327-360 (`gzip`, `incompressible_bytes`); `src/adapters/cursor/test_frames.rs` | exact (role-match to existing in-module suites) |

OpenCodex evidence (`src/adapters/cursor/gen/agent_pb.ts`, `protobuf-events.ts`, `tests/cursor-protobuf-events.test.ts` — executed 2026-06-07/09-07 per 12-RESEARCH.md §5 follow-up) is **external provenance**: cite and qualify in comments/fixtures, never transplanted paths or copied architecture (D-01, E1).

## Pattern Assignments

### `agent.rs` terminal/trailer integrity (CUR-05/D-05, CUR-06)

**Analog:** self — `terminal_event` `:349-357` and the EOF/idle push sites `:319-332`.

```rust
// agent.rs:349-357 — current ownership: EOF validates frame boundary, then
// synthesizes End; hardening splits "clean decode finish" from "clean
// successful turn" so silence/EOF cannot manufacture success.
fn terminal_event(timed_out: bool, got_output: bool, finish: Result<(), ConnectError>)
    -> Result<CursorStreamEvent, CursorError> {
    if timed_out && !got_output {
        return Err(CursorError::internal(
            "cursor: upstream timed out before sending any output"));
    }
    match finish { Ok(()) => Ok(CursorStreamEvent::End), ... }
```

Copy the existing error shape: `CursorError::internal(...)`, never a new vocabulary. Disturb the pacing comments at :20-23 and :293-303 only surgically (load-bearing, D-05 review note).

### `connect.rs` END/trailer seam (CUR-06)

**Analog:** self — `parse_connect_error` `:252-278` returning `Option<ConnectEndError>`; callers must map `None-on-malformed` to an explicit error, preserving the auth-code passthrough pattern (401/403/429/400) seen at `:266-278`. Reuse `FLAG_GZIP`/`FLAG_END` (:4-5) ordering and the existing 64 MiB bound pattern (:116-118, :225); add no new cap vocabulary.

```rust
// connect.rs:252-262 — None currently means "empty/undecodable -> caller
// decides". Hardening: distinguish empty-valid END trailer from malformed
// payload; per RESEARCH Q3 preserve the valid-empty-trailer convention.
pub fn parse_connect_error(payload: &[u8]) -> Option<ConnectEndError> {
    if payload.is_empty() { return None; }
    let parsed: ConnectErrorPayload = serde_json::from_slice(payload).ok()?;
```

### `mod.rs` pre-dispatch admission (CUR-04/D-04)

**Analog:** self — `extract_cursor_tools` `:256-278` (`filter_map` silently drops nameless tools, defaults schemas) and `decode_selected_images` `:176-190` (`.ok()? ` drops undecodable base64). Replace silent drop with explicit unsupported `CursorError` before dispatch; keep the existing `:95` call-site order.

```rust
// mod.rs:176-190 — current evidence gap: undecodable images vanish.
images.into_iter().filter_map(|image| {
    let data = base64::engine::general_purpose::STANDARD
        .decode(image.data.as_bytes()).ok()?;
```

### Usage relay (CUR-05, W0-1/E1)

**Analog:** self — unary usage block `:334-337` (`input_tokens: 1` hardcoded). Follow the OpenCodex schema-derived reading: AgentServerMessage field 1 → InteractionUpdate field 4 thinking / text envelope; TokenDeltaUpdate field 1 int32 tokens; ConversationTokenDetails field 1 uint32 used_tokens is **context occupancy, not additive output** (checkpoints 10000+300+Δ42 → context 10300 not 20300, per `tests/cursor-protobuf-events.test.ts:968`). Label derived/absent input explicitly; never claim authentic billing/cache usage.

### Request identity/pairing fixtures (CUR-03, A2)

**Analog:** `request.rs` stateless re-render + `mod tests` :326+. Fixture style: ordinary/compacted/recovered/multi-round histories → stable conversation identity + authentic call/result pairing; unsupported opaque continuation fails explicitly. Do not fabricate result identities or structure from prompt text (CONTEXT <code_context>).

### Classification & replay-safety (CUR-07/D-08)

**Analog:** `src/retry.rs` `:142-155` — reuse the three-variant `RetrySafety` taxonomy; `ConnectOnly` is the Phase 11 precedent (Antigravity D-17) for "no repeat once it could have reached the upstream". Wire the existing `AdapterFailure::{UpstreamStatus, BeforeHeaders}` (already set in `mod.rs` :475, :481) into the seam; leave `TODO(#126, cursor)` :34, :206 tightening as the proof target. No new retry machinery, no Commitment wrapper (orchestrator constraint).

```rust
// retry.rs:142-155 — vec: Idempotent | NonIdempotentPost | ConnectOnly declarations
// mod.rs:475 — failure: Some(crate::adapters::AdapterFailure::UpstreamStatus(status)),
// mod.rs:481 — error.failure = Some(crate::adapters::AdapterFailure::BeforeHeaders);
```

### Cancellation/RAII (CUR-05/D-07)

**Analog:** `agent.rs` `TurnGuard` :244-252 — ownership struct holding `oneshot::Sender` + paced-send `JoinHandle` for the whole header-wait/read duration; test via the existing `from_response_for_test` :255-268 harness. Assert channel-drop/JoinHandle-abort release without adding any global registry or detached recovery task (D-07).

```rust
// agent.rs:244-247 — RAII release scope to prove
struct TurnGuard { _stop: oneshot::Sender<()>, _sender: JoinHandle<()> }
```

### Hermetic fixture style (CUR-01..08, D-10)

**Analog:** `src/adapters/cursor/mod.rs` :526+ wiremock `MockServer` harness (real HTTP responses, statuses at :548-558) and `connect.rs` :327-360 byte-corpora helpers (`gzip`, `gzip_with`, `incompressible_bytes`). New tests live in-module (or `tests/` for cross-crate) with synthetic credentials, dated provenance, and W0-1/W0-2 protos as cited evidence. CUR-01 is a constant regression assertion on `AGENT_BASE_URL` (`agent.rs` :52).

```rust
// agent.rs:52 — regression assertion target, do not change without captured/live evidence
const AGENT_BASE_URL: &str = "https://agentn.global.api5.cursor.sh";
```

## Shared Patterns

### Gateway-owned error shape
**Source:** `src/adapters/cursor/client.rs` (`CursorError`), e.g. `CursorError::internal` usage in `agent.rs` :351. **Apply to:** all new error surfaces (trailer, admission, continuation) — keep Anthropic-shaped mapping at the outer seam per AGENTS.md.

### Cross-module test serialization
**Source:** `mod.rs` :529 + `connect.rs` :330 `#![allow(clippy::await_holding_lock)]` + `offload::OFFLOAD_OBSERVER` mutex. **Apply to:** any new test touching the shared observer.

### Provenance comments
**Source:** `agent.rs` :1-12 MIT jcode credit; 12-RESEARCH §5 dated OpenCodex evidence. **Apply to:** any substantial schema-derived translation (W0-1) — comment cites OpenCodex rev `055c3ecf0de6c35f59195fc434d6b08525182b7f` + date, claims only what the schema proves.

## No Analog Found

None of the files lack an analog (all are existing active-path modules hardened in place).

| Concern | Reason planner should note |
|---------|---------------------------|
| Authentic usage relay | No in-repo active-path precedent (usage hardcoded at `mod.rs` :334); the analog is **external** OpenCodex schema evidence, qualified per E1 — not an architecture import. |
| Resumable streams | None exists or is required (E2); planner must not invent one for CUR-03. |

## Metadata

- **Analog search scope:** `src/adapters/cursor/`, `src/retry.rs`, `src/proxy/failover.rs`, `tests/` (retried/failover), in-scope OpenCodex evidence files (external).
- **Tracked-path gate (#3645):** all named paths verified via `git ls-files` from the worktree; no mirror paths.
- **Retired vs active:** active = `{agent,mod,connect,request,model,sse,offload,test_frames}.rs` + `retry.rs`/`failover.rs` seam; retired/legacy (do not cite as wire evidence, do not modify) = `{proto,stream,tool_bridge,tool_use_xml,response,client(CursorHttpClient)}.rs`.
- **Pattern extraction date:** 2026-09-07
