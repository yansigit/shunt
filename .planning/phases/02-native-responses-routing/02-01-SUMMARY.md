---
phase: 02-native-responses-routing
plan: 01
subsystem: api
tags: [rust, responses, routing, passthrough]
requires:
  - phase: 01-inbound-responses-websocket
    provides: bounded inbound Responses HTTP/WS passthrough and pinned provider behavior
provides:
  - exact native Responses resolver with pinned, selected, and rejected decisions
  - HTTP and WebSocket dispatch through one immutable native decision
  - exact-route and pinned-fallback byte-preserving integration fixtures
affects: [native-responses-routing, compaction, translated-responses]
actuals:
  tokens: 5200
  tasks: 2
  commits: 2
tech-stack:
  added: []
  patterns: [typed exact-only routing decisions, opaque body passthrough]
key-files:
  created: []
  modified: [src/routing.rs, src/codex_endpoint.rs, src/codex_endpoint/websocket.rs, tests/inbound_codex_endpoint.rs]
key-decisions:
  - "Exact native selection ignores prefixes and ordered chains, and rejects ambiguity, non-Responses adapters, and model translation."
  - "Missing or unusable model inspection preserves the configured codex endpoint provider."
patterns-established:
  - "Resolve once from the refreshed AppState snapshot before adapter dispatch."
  - "Retain the request model/body bytes while normalizing only [1m] lookup and upstream route metadata."
requirements-completed: [ROUTE-01, ROUTE-02]
coverage:
  - id: D1
    description: "Exact native model mapping reaches the Responses passthrough with byte-identical request body."
    requirement: ROUTE-01
    verification:
      - kind: integration
        ref: "cargo test --test inbound_codex_endpoint exact_native_route"
        status: pass
    human_judgment: false
  - id: D2
    description: "Resolver matrix enforces pinned fallback and rejects ambiguous, translated, prefix-only, and incompatible routes."
    requirement: ROUTE-02
    verification:
      - kind: unit
        ref: "cargo test routing:: --lib"
        status: pass
      - kind: integration
        ref: "cargo test --test inbound_codex_endpoint pinned_fallback"
        status: pass
    human_judgment: false
---

# Phase 2: Native Responses Routing Summary

Exact native Responses routing now selects only uniquely eligible configured providers while preserving pinned compatibility and opaque request bytes.

## Performance

- **Duration:** ~25 min
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added typed native decision resolution for exact model maps and legacy routes.
- Wired HTTP and WebSocket turns through the shared resolver from one refreshed state snapshot.
- Added resolver edge-case coverage plus exact-route and pinned-fallback HTTP fixtures.

## Task Commits

1. **Task 1: Trace one exact native HTTP model through shared resolution** - `0faa3c3` (feat)
2. **Task 2: Lock resolver edge-case and fallback contract** - `01e7b0f` (test)

## Files Created/Modified

- `src/routing.rs` - Native exact-only decision enum, resolver, and matrix tests.
- `src/codex_endpoint.rs` - Shared resolver invocation and rejection handling.
- `src/codex_endpoint/websocket.rs` - WebSocket dispatch parity through shared resolution.
- `tests/inbound_codex_endpoint.rs` - Exact native and pinned fallback byte-preserving fixtures.

## Decisions Made

Followed the locked D-01 through D-05 and D-10 decisions; no public configuration keys or credential writeback behavior changed.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

The existing `forward_turn` API was pinned-provider-specific; it was generalized to accept the native decision model while retaining the existing passthrough and account-pool machinery.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

The exact native routing seam and compatibility fallback are ready for subsequent native routing coverage and provider-specific expansion.

## Self-Check: PASSED

- Key files exist and focused verification commands pass.
- Task commits `0faa3c3` and `01e7b0f` are present.

---
*Phase: 02-native-responses-routing*
*Completed: 2026-09-06*
