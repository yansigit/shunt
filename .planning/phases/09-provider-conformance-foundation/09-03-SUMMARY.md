---
phase: 09-provider-conformance-foundation
plan: "03"
subsystem: codex-continuation
tags: [rust, responses, websocket, continuation, bounds, identity]
requires:
  - phase: 09-provider-conformance-foundation
    provides: monotonic redispatch commitment from plan 09-01
  - phase: 09-provider-conformance-foundation
    provides: trustworthy Responses terminals and bounded translation from plan 09-02
provides:
  - Exact one-shot full-input recovery guarded by the shared commitment predicate
  - Atomically bounded continuation capture and storage
  - Fail-closed tool and reasoning identity validation on production translation
affects: [codex-websocket, responses-translation, account-pools, compaction, collaboration]
actuals:
  tokens: 13863
  tasks: 3
  commits: 8
plan_head_before: 8cbbe13555550b8b03e6866f19fc54db895efa2c
tech-stack:
  added: []
  patterns: [one-shot-commitment-gate, atomic-overflow-discard, authentic-identity-validation]
key-files:
  created: []
  modified:
    - src/adapters/responses/codex_ws.rs
    - src/adapters/responses/codex_continuation.rs
    - src/adapters/responses/websocket.rs
    - src/adapters/responses/mod.rs
    - src/model/responses.rs
    - src/model/responses_request.rs
    - tests/responses_translate.rs
    - tests/codex_websocket_fallback.rs
key-decisions:
  - "Recovery may issue one full-input request only while the shared Commitment remains redispatch-safe."
  - "Continuation overflow discards the whole candidate instead of truncating provider-derived state."
  - "The production adapter uses a fallible translator while the trusted parsed-value helper retains its compatible signature."
patterns-established:
  - "Authentic continuation: retain paired tool identities and opaque reasoning metadata exactly, or reject the candidate."
  - "Bounded capture: store continuation only after a genuine terminal and successful aggregate validation."
requirements-completed: [PRES-02, SAFE-06]
coverage:
  - id: D1
    description: One previous-response miss produces exactly one safe full-input recovery with adjacent tool call/result and opaque state preserved once.
    requirement: SAFE-06
    verification:
      - kind: integration
        ref: cargo test --all-features continuation_recovery
        status: pass
      - kind: integration
        ref: cargo test --all-features --test codex_websocket_fallback
        status: pass
    human_judgment: false
  - id: D2
    description: Item, transcript-byte, response-ID, and turn-state limits accept exact caps and atomically reject cap-plus-one.
    requirement: SAFE-06
    verification:
      - kind: unit
        ref: cargo test --all-features continuation_bounds
        status: pass
    human_judgment: false
  - id: D3
    description: Invented or malformed tool/reasoning identities fail closed while account, quota, compaction, collaboration, and ingress behavior remain unchanged.
    requirement: PRES-02
    verification:
      - kind: integration
        ref: cargo test --all-features authentic_tool_identity
        status: pass
      - kind: integration
        ref: cargo test --all-features --test codex_multi_account --test inbound_codex_endpoint --test inbound_anthropic_translation
        status: pass
      - kind: workspace
        ref: cargo test --all-features --workspace
        status: pass
    human_judgment: false
duration: 22min
completed: 2026-09-06
status: complete
---

# Phase 9 Plan 03: Bounded Authentic Codex Continuation Summary

**Codex continuation recovery now replays one exact, bounded, authentic transcript only while redispatch remains safe.**

## Performance

- **Duration:** 22 min
- **Started:** 2026-09-06T21:49:17Z
- **Completed:** 2026-09-06T22:11:20Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments

- Proved at the real gateway boundary that a missing previous response permits exactly one full-input recovery and preserves paired tool IDs, opaque reasoning state, and turn metadata exactly once.
- Added 10,000-item, 32 MiB transcript, and 64 KiB response-ID/turn-state limits with checked crossing-item accounting and whole-candidate discard.
- Removed synthetic tool-search identities and fallback arguments, rejected malformed or unkeyed continuation state, and kept account, quota, compaction, collaboration, and ingress suites green.

## Task Commits

1. **Task 1 characterization: exact continuation recovery** - `02cc9ac`
2. **Task 1 implementation: shared commitment gate** - `878783e`
3. **Task 2 RED: expose unbounded continuation state** - `5cdeebe`
4. **Task 2 GREEN: atomically bound continuation capture** - `a321f7d`
5. **Task 2 formatting** - `cb875c5`
6. **Task 3 RED: expose invented Responses identities** - `736935b`
7. **Task 3 GREEN: reject invented continuation identities** - `1ff3e82`
8. **Task 3 compatibility: preserve trusted helper API** - `6eafe23`

## Files Created/Modified

- `src/adapters/responses/websocket.rs` - Uses the shared monotonic commitment predicate before the single recovery request.
- `src/adapters/responses/codex_continuation.rs` - Fallible bounded transcript construction and authentic retained-item validation.
- `src/adapters/responses/codex_ws.rs` - Atomic capture, overflow discard, and store-after-terminal behavior.
- `src/adapters/responses/mod.rs` - Production caller consumes typed fallible request translation.
- `src/model/responses_request.rs` - Strict request-history identity validation plus compatible trusted and fallible entry points.
- `src/model/responses.rs` - Strict native tool-search and reasoning identity handling.
- `tests/codex_websocket_fallback.rs` - Stateful real-gateway one-shot recovery fixture.
- `tests/responses_translate.rs` - Missing identity, malformed argument, and opaque reasoning regressions.

## Decisions Made

- Reused the shared `Commitment` gate rather than introducing a continuation-specific replay decision.
- Counted the crossing item and serialized bytes before accepting a continuation candidate; no partial transcript is retained.
- Preserved the established infallible parsed-value helper for trusted internal callers and benches, while the byte wrapper and production adapter use the typed fallible path.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Routed production through fallible request translation**
- **Found during:** Task 3 production integration
- **Issue:** Making translation validation fallible was insufficient unless the production adapter consumed the typed result before dispatch.
- **Fix:** Added `src/adapters/responses/mod.rs` to the approved write boundary and mapped validation failures to a gateway-owned Anthropic `400 invalid_request_error`.
- **Approval:** Explicitly approved by the Phase 9 orchestrator as a Rule 2 boundary deviation.
- **Files modified:** `src/adapters/responses/mod.rs`
- **Verification:** `cargo test --all-features authentic_tool_identity`; `cargo test --all-features --workspace`
- **Committed in:** `1ff3e82`, compatibility follow-up `6eafe23`

---

**Total deviations:** 1 auto-fixed (1 missing critical)
**Impact on plan:** Required for the approved fail-closed semantic to reach the production caller; no public config, provider-support, or credential decision changed.

## Issues Encountered

- The initial real-gateway recovery characterization passed after its fixture accurately modeled the existing wire envelope. Production still received the planned explicit shared-commitment refactor; no unrelated behavior repair was needed.
- Shared no-isolation work left unrelated site documentation edits in the worktree. They were preserved and excluded from every Plan 09-03 commit.
- README, engineering docs, English and localized site pages, and generated wiki implications were reviewed. This plan adds no setup, config key, endpoint, CLI, provider/model support, or default; the strict internal continuation behavior is already represented by the Phase 9 contract and focused tests, so no Plan 09-03 documentation file was changed.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test --all-features continuation_recovery` - pass
- `cargo test --all-features continuation_bounds` - pass
- `cargo test --all-features authentic_tool_identity` - pass
- `cargo test --all-features --test responses_translate` - pass (96 tests)
- `cargo test --all-features --test codex_multi_account --test inbound_codex_endpoint --test inbound_anthropic_translation` - pass
- `cargo test --all-features --test inbound_codex_endpoint --test inbound_codex_websocket --test codex_websocket_fallback --test codex_multi_account` - pass
- `cargo fmt --all --check` - pass
- `cargo clippy --all-targets --all-features -- -D warnings` - pass
- `cargo test --all-features --workspace` - pass (2 ignored pre-existing tests)

## Next Phase Readiness

- The bounded, authentic continuation path is ready for the remaining provider-hardening phases.
- No durable history, signing subsystem, credential writeback, later-provider adapter, or Google AI Studio Web artifact was added.
- No code blocker remains.

## Self-Check: PASSED

- All eight implementation/test files listed above exist and all eight Plan 09-03 commits exist on the current branch.
- The approved extra production-caller file is documented; no other Plan 09-03 write-boundary expansion occurred.
- No stub, skipped test, dependency, public config change, secret, generated wiki edit, or credential writeback was introduced.

---
*Phase: 09-provider-conformance-foundation*
*Completed: 2026-09-06*
