---
phase: 12-cursor-evidence-backed-hardening
plan: "05"
subsystem: cursor
tags: [terminal, usage, streaming]
requires: [12-01, 12-04]
provides: [idle and duplicate terminal rejection, schema-derived incremental usage]
affects: [12-06, 12-08]
tech-stack:
  added: []
  patterns: [absolute context replacement, incremental output counters]
key-files:
  created: [src/adapters/cursor/usage.rs]
  modified: [src/adapters/cursor/agent.rs, src/adapters/cursor/mod.rs, src/adapters/cursor/sse.rs, src/adapters/cursor/history_lifetime_tests.rs, src/adapters/cursor/router_parity_tests.rs]
requirements-completed: []
completed: 2026-09-07
status: complete
---

# Phase 12 Plan 05 Summary

Idle expiry is now an explicit upstream error, never an inferred successful
terminal. Duplicate Connect END or post-terminal bytes already received in the
batch fail before End. Run interaction.turn_ended is recognized; an immediately
following Connect trailer is inspected before success. No claim is made about
future bytes after accepted completion/cancellation. EOF rejection remains the
plan-01 implementation.

Output deltas accumulate while checkpoint occupancy replaces previous absolute
context. JSON and incremental SSE relay the same counters; negative/overflowing
deltas and malformed counter encoding fail. Input is derived by saturating
context minus output. `usage.estimated: true` labels that derivation; numeric
zero for unavailable input/cache/output is documented as a placeholder, not a
measured count. No CLI cache totals or retired-wire fields were invented.

## Verification

- Red tests observed duplicate END accepted as success and 42 output tokens
  reported as the hardcoded 1. Both regressions now pass.
- Usage tests prove 20 then 42 arrives before terminal, absolute 10300 context
  (not 20300), derived input 10258, absent metrics, and negative/overflow errors.
- Terminal tests cover duplicates, trailing complete/partial bytes, semantic
  turn-ended with/without valid trailing END, and whole-router real idle timeout
  for JSON/SSE (one error, never message_stop).
- Full all-feature workspace: library 2,176 passed, 2 existing ignored;
  integration suites all passed. All-target clippy with warnings denied and
  format passed.
- Rebuilt binary smoke: five checks passed on 31711/31712; owned processes and
  temporary smoke configuration cleaned up. No live Cursor inference claimed.
- Four-locale site build: 161 pages passed. Production OpenCodex config mtime/SHA
  and invalid/backup inventory unchanged across isolated command trees.

## Deviations and remaining work

Added focused usage.rs and extended the shared SSE framer with an active-Run
mode, leaving retained buffered protocol behavior unchanged. Required numeric
usage fields use labeled zero placeholders instead of null to retain the
Anthropic shape; documented explicitly in all maintained languages. The old
idle characterization assertion was intentionally flipped to error.

A parallel full-suite run exposed an offload permit-count race: the new router
tests did not hold the existing OFFLOAD_OBSERVER mutex. All router tests now
hold that observer before the config environment lock; the original strict
permit-count assertion is unchanged. The parallel suite then passed.

README, engineering note and provider pages updated in English/ko/ja/zh-cn; wiki
left generated and untouched. CUR-05 remains phase-level open until plan 08
cancellation/capacity verification. Plan 06 strict framing/arguments is next.
