---
phase: 09-provider-conformance-foundation
fixed_at: 2026-09-06T22:36:11Z
review_path: .planning/phases/09-provider-conformance-foundation/09-REVIEW.md
iteration: 1
findings_in_scope: 7
fixed: 7
skipped: 0
status: all_fixed
---

# Phase 9: Code Review Fix Report

**Fixed at:** 2026-09-06T22:36:11Z  
**Source review:** `.planning/phases/09-provider-conformance-foundation/09-REVIEW.md`  
**Iteration:** 1

**Summary:**

- Findings in scope: 7
- Fixed: 7
- Skipped: 0

## Fixed Issues

### CR-01: HTTP streaming stops after provider failure

**Files modified:** `src/adapters/responses/http.rs`  
**Commit:** `4fcc3ea` (`44da3b5` formatting)  
**Applied fix:** Stop consuming after the machine records an authoritative provider failure; regressions prove exactly one error and prompt pending-body release.

### CR-02: Unterminated SSE terminals fail closed

**Files modified:** `src/adapters/responses/http.rs`  
**Commit:** `00fd022` (`44da3b5` formatting)  
**Applied fix:** Reject non-whitespace residual SSE candidates at EOF in streaming and non-streaming paths.

### CR-03: Continuation output uses aggregate accounting

**Files modified:** `src/adapters/responses/codex_ws.rs`  
**Commit:** `105c42b` (`44da3b5` formatting)  
**Applied fix:** Track exact serialized array bytes with checked arithmetic before retaining each output item and discard atomically on overflow.

### CR-04: Malformed WebSocket events terminate the turn

**Files modified:** `src/adapters/responses/codex_ws.rs`  
**Commit:** `3811571` (`44da3b5` formatting)  
**Applied fix:** Distinguish malformed frames from valid unknown event types; malformed input emits one protocol error and evicts the turn before a later success terminal.

### CR-05: `response.done` terminates WebSocket turns

**Files modified:** `src/adapters/responses/codex_ws.rs`  
**Commit:** `e03803b`  
**Applied fix:** Classify `response.done` as a reusable success terminal and close the client turn without waiting for timeout or EOF.

### WR-01: Commitment is limited to reachable redispatch seams

**Files modified:** `src/retry.rs`, `src/proxy/failover.rs`, `tests/retry.rs`, `09-01-SUMMARY.md`, `09-PATTERNS.md`, `09-VALIDATION.md`  
**Commit:** `03c343e`  
**Applied fix:** Remove unreachable commitment parameters/checks from structurally pre-response generic retry and route failover. Retain one monotonic predicate on real WebSocket fallback and continuation-recovery paths, with production gateway regressions.

### WR-02: Provider response IDs count toward retained state

**Files modified:** `src/model/responses.rs`, `tests/responses_translate.rs`  
**Commit:** `b2650a5`  
**Applied fix:** Use one ID-selection helper and reserve its bytes before cloning, with exact/plus-one, combined-state, and ignored-duplicate tests.

## Verification

Verification ran in the main checkout because `.planning/config.json` sets `workflow.use_worktrees` to `false`.

- `cargo fmt --all --check` — passed
- `cargo clippy --all-targets --all-features -- -D warnings` — passed
- `cargo test --all-features --workspace` — passed
- Focused HTTP, Codex WebSocket, Responses translation, retry, failover, fallback, and continuation-recovery suites — passed

No public configuration/provider semantics, credential writeback, Google AI Studio Web support, or Phase 10 work was introduced.

---

_Fixed: 2026-09-06T22:36:11Z_  
_Fixer: Codex (gsd-code-fixer)_  
_Iteration: 1_
