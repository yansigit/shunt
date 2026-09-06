---
phase: 02-native-responses-routing
plan: 03
requirements-completed: [ROUTE-01, ROUTE-02]
subsystem: inbound-responses-websocket
tags: [websocket, native-routing, snapshots, concurrency]
requires: [02-02]
provides: [websocket-native-route-parity, per-turn-config-snapshots, no-mid-stream-hop-coverage]
affects: [02-04, 02-05]
tech-stack:
  added: []
  patterns: [refresh-runtime-at-turn-start, shared-native-resolver]
key-files:
  created: []
  modified:
    - src/codex_endpoint/websocket.rs
    - tests/inbound_codex_websocket.rs
key-decisions:
  - WebSocket turns refresh AppState immediately before dispatch, preserving one immutable route decision per turn while allowing later turns to observe reloads.
  - HTTP and WebSocket parity is verified against the same exact native Responses route without requiring identical transport envelopes.
metrics:
  duration: 12m
  completed: 2026-09-06
  commits: 1
  plan_head_before: 016432e5981a4336137a892822a76fa41b5c5dc6
actuals:
  tokens: 5200
  tasks: 2
  commits: 1
status: complete
---

# Phase 02 Plan 03: WebSocket Native Routing and Snapshot Isolation Summary

WebSocket live turns now refresh the shared runtime snapshot at dispatch time, so each turn resolves one immutable native route while later turns can pick up hot-reloaded configuration. Focused integration coverage proves HTTP/WebSocket provider parity and the no-mid-stream-hop boundary while retaining Phase 1 cancellation and bounded SSE behavior.

## Accomplishments

- Refreshed `AppState` at the start of every WebSocket live turn before calling the shared `forward_turn` resolver.
- Added a native Responses mock provider fixture and HTTP/WebSocket parity test, preserving the request model without transport-envelope byte-equality assumptions.
- Added explicit no-mid-stream-hop coverage asserting one upstream request after observable response events.
- Extended the upstream fixture to serve both Codex and OpenAI-compatible Responses paths.

## Verification

- `cargo test --test inbound_codex_websocket -- --nocapture` — 9 passed
- `cargo fmt --all --check` — passed
- `cargo clippy --all-targets --all-features -- -D warnings` — passed
- `cargo test --all-features --workspace` — passed

## Deviations from Plan

None — the shared resolver and existing lifecycle remain unchanged except for the required per-turn refresh.

## Issues Encountered

None.

## User Setup Required

None.

## Next Phase Readiness

The inbound HTTP and WebSocket paths now share native exact-route selection with per-turn snapshot isolation. Documentation and locale synchronization remain for the subsequent plans.

## Self-Check: PASSED

- Production and test changes are committed in `8323259`.
- Focused and full workspace verification passed.
- `.gsd/` and `.planning/milestone.lock` remain uncommitted runtime artifacts.
