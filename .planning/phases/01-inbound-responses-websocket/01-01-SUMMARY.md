---
phase: 01-inbound-responses-websocket
plan: 01
subsystem: api
tags: [axum, websocket, responses, sse, authentication]
requires: []
provides:
  - Authenticated WebSocket upgrades on all opt-in inbound Responses paths
  - Local generate-false warmup completion frames
  - Bounded byte-oriented SSE framing
affects: [01-02, 01-03, inbound-codex, streaming]
actuals:
  tokens: 10000
  tasks: 3
  commits: 1
tech-stack:
  added: [axum-ws]
  patterns: [pre-upgrade authentication, bounded byte framing, shared inbound pool dispatch]
key-files:
  created: [src/codex_endpoint/frame.rs, src/codex_endpoint/websocket.rs]
  modified: [Cargo.toml, Cargo.lock, src/server.rs, src/codex_endpoint.rs, src/codex_endpoint/tests.rs]
key-decisions:
  - "Reuse forward_codex_inbound through a narrow forward_turn helper so WebSocket turns retain existing pool, credential, quota, and lease behavior."
  - "Force stream=true for generated WebSocket turns while removing only the transport control type field."
patterns-established:
  - "Authenticate before on_upgrade so rejected clients receive an HTTP Responses error and never open a socket."
  - "Frame SSE on bytes with explicit event-size and per-feed object-count limits."
requirements-completed: [WS-01, WS-02, WS-03]
coverage:
  - id: D1
    description: "All configured Responses paths accept authenticated WebSocket upgrades and reject unauthorized upgrades before HTTP 101."
    requirement: WS-01
    verification:
      - kind: integration
        ref: "src/codex_endpoint/tests.rs#ws_tests"
        status: pass
    human_judgment: false
  - id: D2
    description: "generate:false produces the exact local created/completed warmup pair without entering upstream dispatch."
    requirement: WS-02
    verification:
      - kind: unit
        ref: "src/codex_endpoint/frame.rs#warmup_frames_have_expected_shape_and_empty_id"
        status: pass
    human_judgment: false
  - id: D3
    description: "response.create uses the existing inbound pool path and response.processed remains a no-op."
    requirement: WS-03
    verification:
      - kind: unit
        ref: "cargo check --all-targets; src/codex_endpoint/frame.rs#parse_response_processed_is_noop"
        status: pass
    human_judgment: false
duration: 37min
completed: 2026-09-05
status: complete
---

# Phase 1 Plan 1: Inbound Responses WebSocket Tracer Summary

**Authenticated WebSocket upgrades, local warmups, and bounded SSE framing now enter Shunt's existing Codex account-pool pipeline.**

## Performance

- **Duration:** 37 min
- **Started:** 2026-09-05T22:55:47Z
- **Completed:** 2026-09-05T23:32:24Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- Registered GET WebSocket upgrades beside the existing POST handler on all three opt-in Responses paths.
- Enforced inbound authentication before upgrade and implemented exact empty-ID warmup frames.
- Ported OpenCodex's byte-bounded SSE block scanner and routed live turns through Shunt's established account pool.

## Task Commits

1. **Tasks 1-3: WebSocket tracer, warmup, and SSE framer** - `d096686` (feat)

## Files Created/Modified

- `src/codex_endpoint/websocket.rs` - Upgrade handler and socket session transport.
- `src/codex_endpoint/frame.rs` - Control-frame parser, warmup builder, and bounded SSE framer.
- `src/codex_endpoint.rs` - Narrow shared auth/session/forwarding helpers.
- `src/server.rs` - GET plus POST registration on Responses paths.
- `Cargo.toml` / `Cargo.lock` - Axum WebSocket feature support.

## Decisions Made

- The socket path reuses the existing inbound dispatcher rather than building a second account pool.
- WebSocket turns always set `stream: true`, matching OpenCodex's transport contract.

## Deviations from Plan

### Auto-fixed Issues

**1. Recovery - combined task commit**
- **Found during:** Executor recovery
- **Issue:** The original executor left a compiling but uncommitted implementation spanning Plans 01 and 02.
- **Fix:** Audited, formatted, linted, corrected, and checkpointed the coherent transport core in one commit.
- **Files modified:** Plan-scoped source and tests.
- **Verification:** `cargo check --all-targets`, focused tests, and strict Clippy passed.
- **Committed in:** `d096686`

**Total deviations:** 1 auto-fixed recovery issue.
**Impact on plan:** Commit granularity changed; behavior and scope did not.

## Issues Encountered

The first executor stalled before verification or commit; its useful work was preserved and completed inline.

## User Setup Required

None - the existing `[server.codex_endpoint]` and inbound auth configuration apply.

## Next Phase Readiness

Ready for Plan 01-02 lifecycle verification and Plan 01-03 conformance coverage.

## Self-Check: PASSED

---
*Phase: 01-inbound-responses-websocket*
*Completed: 2026-09-05*
