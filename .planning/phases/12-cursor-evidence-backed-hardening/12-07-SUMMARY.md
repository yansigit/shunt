---
phase: 12-cursor-evidence-backed-hardening
plan: "07"
subsystem: cursor
tags: [retry, failover, safety]
requires: [12-05, 12-06]
provides: [ConnectOnly failure classification, no replay after acceptance]
affects: [12-08]
requirements-completed: [CUR-07]
completed: 2026-09-07
status: complete
---

# Phase 12 Plan 07 Summary

Cursor's existing AdapterFailure seam now permits fallback only for a proven
connect-phase transport failure. Local/header construction errors, ambiguous
post-send timeouts, and accepted HTTP statuses have no replay metadata. Stream
errors and tool pauses never return to dispatch. The existing ConnectOnly
taxonomy is explicitly assigned; no retry loop or Commitment wrapper was added.
TODO #170 remains honest: the paced transport has no same-provider retry loop.
All TODO(#126, cursor) statements were resolved, including an obsolete comment
in a preserved legacy idempotent-driver characterization test.

## Verification

- Red test exposed local construction marked BeforeHeaders; now fails closed.
- Two retry-safety tests passed: exact taxonomy, true refused connection,
  real invalid-header builder failure with no dispatch, and post-send timeout
  observed by a loopback upstream exactly once.
- Two classification tests passed: HTTP status table and real TLS/H2 full-router
  primary/fallback chain (explicitly asserted length/order). Fallback receives
  zero requests for 429/503, partial output then EOF, and authentic tool pause,
  in both JSON and SSE modes; the primary receives exactly one request.
- All-feature workspace passed: library 2,187 passed, 2 existing ignored;
  all integration suites passed. Format and all-target clippy -D warnings pass.
- Rebuilt binary smoke: five checks passed; owned processes/scratch cleaned up.
- Four-locale site: 161 pages built. Production OpenCodex config mtime/SHA and
  invalid/backup inventory unchanged through every isolated command tree.

## Deviations

Full parallel tests exposed two preexisting mixed-request tests using the
preparation pool without OFFLOAD_OBSERVER. They now hold that test observer
while retaining all concurrent mixed-request assertions. Original strict pool
capacity assertions are unchanged. The parallel suite then passed.

client.rs retains the connect-phase fact from reqwest; router_parity_tests.rs
owns actual fallback proof. These narrow additions are required to wire the
planned classification through real consumers. No shared proxy behavior or
public configuration key changed. Config's stale Cursor retry documentation
was corrected. README and provider pages updated in all four languages;
engineering note updated; generated wiki untouched. Plan 08 remains next.
