---
phase: 12-cursor-evidence-backed-hardening
plan: "08"
subsystem: cursor
tags: [cancellation, parity, documentation]
requires: [12-05, 12-07]
provides: [router cancellation proof, ordered bounded JSON and SSE output, no heuristic proof]
affects: [16]
key-files:
  created: [src/adapters/cursor/cancellation_tests.rs, src/adapters/cursor/aggregate.rs]
  modified: [src/adapters/cursor/mod.rs, src/adapters/cursor/sse.rs, src/adapters/cursor/router_parity_tests.rs]
requirements-completed: [CUR-05, CUR-08]
completed: 2026-09-07
status: complete
---

# Phase 12 Plan 08 Summary

Actual TLS/H2 upstream requests now prove full-router cancellation before
headers, during streamed text, and during non-streaming accumulation. The
test saturates a single gateway slot, cancels the owning service future/body,
observes upstream close/reset and zero active turns, then completes a new
request through the reclaimed slot. The existing backpressured sender test
also proves its JoinHandle aborts and its channel closes. This is router-side
cancellation with real upstream transport, not a claim about every client UI.

Repeated semantic payloads retain all 100 text deltas and require an actual
terminal; EOF errors. Existing idle tests remain explicit failures. Config
round-trip pins the current Cursor keys, and a fresh request store cannot
retrieve a previous request's blob. No registry, persistence or heuristic added.

## Gap found and repaired

Goal-level review found JSON discarded reasoning and merged text across it.
A full-router red test returned only answer-oneanswer-two instead of four
ordered thinking/text blocks. JSON now preserves those blocks. Active SSE
closes a text block before returning to thinking, preserving the same block
indices and order. Both return input 58/output 42 for the fixture and one
terminal. JSON aggregation is bounded using the existing 64 MiB budget with
conservative escaping/node charges; cap+1 and invalid tool JSON tests pass.
The old reasoning-discard characterization was deliberately flipped to
preservation, with terminal assertions retained. Empty signatures match SSE.

## Verification

- Cancellation filter: full-router three-phase test passed; existing sender
  abort test remains active under both history/cancellation filter names.
- No-heuristics filter: 2 tests passed; output-parity filter: 2 tests passed.
- Full all-feature workspace: library 2,192 passed, 2 existing ignored;
  all integration suites passed. Format, all-target clippy -D warnings,
  workspace cargo check passed.
- Rebuilt binary smoke: all five checks passed on 31711/31712; owned processes
  and temporary configuration cleaned up. No live Cursor inference claimed.
- Four-locale documentation build: 161 pages passed. README, provider pages,
  configuration references and engineering note updated; wiki untouched.
- Production OpenCodex config mtime/SHA and invalid/backup inventory remained
  unchanged throughout isolated command trees. Synthetic test auth bytes were
  unchanged and test files removed.
- GSD decision coverage: 11/11 honored, no missing decisions.

The new small aggregate helper repairs the discovered parity/buffering gap;
the phase still requires goal-level verification before transition.
