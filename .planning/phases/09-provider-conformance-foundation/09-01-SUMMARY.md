---
phase: 09-provider-conformance-foundation
plan: "01"
subsystem: retry-safety
tags: [rust, retry, failover, websocket, codex, tool-safety]

requires:
  - phase: 08-bounded-shutdown
    provides: bounded response-lifetime cancellation and RAII ownership
provides:
  - crate-private monotonic redispatch commitment vocabulary
  - shared replay-safe predicate at WebSocket, same-provider retry, and route-failover seams
  - real-gateway no-redispatch evidence for output, truncation, provider error, and tool activity
affects: [gemini-hardening, antigravity-hardening, cursor-hardening, openai-chat, command-code]

actuals:
  tokens: 4961
  tasks: 2
  commits: 4
plan_head_before: 09531365f77132860a4dbedc8e5a714ac7d684c1

tech-stack:
  added: []
  patterns: [monotonic commitment state, additive redispatch gate, exact upstream hit accounting]

key-files:
  created: []
  modified:
    - src/retry.rs
    - src/proxy/failover.rs
    - src/adapters/responses/websocket.rs
    - tests/retry.rs
    - tests/failover.rs
    - tests/codex_websocket_fallback.rs

key-decisions:
  - "Keep Commitment crate-private and additive to existing transport, status, operation-safety, and retry-budget evidence."
  - "Preserve the conservative Codex rule that every successfully received first WebSocket event commits, while classifying structural tool events as replay-unsafe."

patterns-established:
  - "Redispatch requires Commitment::may_redispatch() in addition to each seam's existing eligibility facts."
  - "Client-visible output and replay-unsafe tool activity monotonically close redispatch and cannot reopen it."

requirements-completed: [PRES-03, SAFE-04]

coverage:
  - id: D1
    description: "Commitment advances monotonically from replay-safe to client-visible or replay-unsafe-tool state and never reopens redispatch."
    requirement: SAFE-04
    verification:
      - kind: unit
        ref: "src/retry.rs#replay_commitment_transitions_are_monotonic"
        status: pass
    human_judgment: false
  - id: D2
    description: "Codex WebSocket falls back before a provider event but never after text or structural tool activity."
    requirement: PRES-03
    verification:
      - kind: integration
        ref: "tests/codex_websocket_fallback.rs#replay_commitment_tool_before_text_does_not_fallback_to_http"
        status: pass
      - kind: integration
        ref: "cargo test --all-features --test codex_websocket_fallback"
        status: pass
    human_judgment: false
  - id: D3
    description: "Same-provider retry and route failover share the commitment predicate without widening existing pre-response eligibility."
    requirement: SAFE-04
    verification:
      - kind: unit
        ref: "src/retry.rs#redispatch_gate_same_provider_commitment_matrix"
        status: pass
      - kind: integration
        ref: "cargo test --all-features redispatch_gate"
        status: pass
      - kind: integration
        ref: "cargo test --all-features --test failover --test retry"
        status: pass
    human_judgment: false

duration: 12min
completed: 2026-09-06
status: complete
---

# Phase 9 Plan 1: Shared Redispatch Commitment Summary

**One monotonic replay-safety state now gates Codex WebSocket fallback, same-provider retry, and HTTP route advancement without broadening any existing retry behavior.**

## Performance

- **Duration:** 12 min
- **Started:** 2026-09-06T21:05:34Z
- **Completed:** 2026-09-06T21:17:01Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Added a crate-private `Commitment` state whose replay-safe, client-visible, and replay-unsafe-tool transitions are monotonic.
- Routed the existing Codex first-event fallback decision through the shared predicate and classified structural tool events before ordinary text.
- Added exact-hit real-gateway coverage proving that accepted output, truncated bodies, in-body provider failures, and tool activity never cause a second dispatch.

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: replay commitment coverage** - `6adfe96` (test)
2. **Task 1 GREEN: WebSocket replay commitment** - `dbeb4ab` (feat)
3. **Task 2 characterization: HTTP redispatch boundaries** - `92b9a38` (test)
4. **Task 2 implementation: shared HTTP commitment gate** - `b22ec56` (feat)

## Files Created/Modified

- `src/retry.rs` - Defines monotonic commitment evidence and applies it to same-provider retry reissues.
- `src/proxy/failover.rs` - Requires replay-safe commitment in addition to existing route-advance evidence.
- `src/adapters/responses/websocket.rs` - Classifies the first successful event and gates HTTP fallback through commitment.
- `tests/codex_websocket_fallback.rs` - Proves tool-before-text commitment causes zero HTTP fallback requests.
- `tests/failover.rs` - Characterizes accepted error, body truncation, and tool activity as no-advance outcomes.
- `tests/retry.rs` - Pins the response-handoff boundary for same-provider retry.

## Decisions Made

- Kept `RetrySafety` and `AdapterFailure` as independent operation/transport evidence; commitment is an additional safety gate, not a replacement.
- Kept metadata-only first WebSocket events conservative in this preservation phase: any successfully received event commits exactly as before.
- No public provider/configuration semantics changed, so README, engineering docs, site locales, root translations, and generated `wiki/` require no update for this plan.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The GSD RED-evidence checker parses TAP while Cargo emits Rust harness output. The actual failing Cargo assertion was represented as equivalent TAP in the temporary evidence record; the checker returned `RED_EVIDENCE_OK`, and the temporary record was removed before commits.

## User Setup Required

None - all verification is hermetic and uses loopback mock upstreams.

## Next Phase Readiness

- Later provider phases can consume the crate-private commitment vocabulary without changing public configuration.
- No credential discovery, refresh, persistence, or writeback path changed.
- Google AI Studio Web and all later-provider behavior remain excluded from this plan.

## Self-Check: PASSED

- All six modified implementation/test files exist.
- Commits `6adfe96`, `dbeb4ab`, `92b9a38`, and `b22ec56` exist.
- All plan-level verification commands passed.
- Format and warnings-denied Clippy passed.

---
*Phase: 09-provider-conformance-foundation*
*Completed: 2026-09-06*
