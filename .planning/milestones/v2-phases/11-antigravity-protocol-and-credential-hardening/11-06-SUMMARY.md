---
phase: 11-antigravity-protocol-and-credential-hardening
plan: "06"
subsystem: testing
tags: [antigravity, cancellation, lifetime, conformance]
requires:
  - phase: "11-05"
    provides: bounded same-account 401 replay
provides:
  - Native streaming and unary cancellation release upstream bodies and admission
  - Integrated 40-test native conformance evidence
affects: ["11-07", "16"]
tech-stack:
  added: []
  patterns: [response-owned drop notification without a producer task]
key-files:
  created: []
  modified: [tests/gemini_conformance.rs]
key-decisions:
  - New lifetime tests fail explicitly if loopback cannot be bound.
  - Pending upstream bodies own their drop notifications directly; no detached producer.
requirements-completed: [ANT-01, ANT-02, ANT-03, ANT-04, ANT-05, ANT-06, ANT-07, ANT-08]
coverage:
  - id: D1
    description: Native streaming drop and unary cancellation release upstream and the sole admission slot
    verification:
      - kind: e2e
        ref: cargo test --all-features --test gemini_conformance antigravity_native_lifetime -- --test-threads=1
        status: pass
    human_judgment: false
  - id: D2
    description: All existing native origin, affinity, envelope, tool, SSE, and 401 tests remain passing
    verification:
      - kind: e2e
        ref: cargo test --all-features antigravity_native -- --test-threads=1
        status: pass
    human_judgment: false
duration: 7min
completed: 2026-09-07
status: complete
plan_head_before: 58c6bfe
---

# Phase 11 Plan 06: Native lifetime conformance

Both client cancellation modes now have explicit native router evidence for bounded upstream release, admission reuse, stable credentials/project, and zero detached replay.

## Accomplishments

Two new `antigravity_native_lifetime` tests drive `Router::oneshot` against a held loopback SSE body. They prove early streaming text, no premature unary completion, saturation at one admitted request, upstream body destruction on cancellation, successful admission afterward, exactly two deliberately initiated requests, and unchanged synthetic credential bytes and modification time. Every wait is bounded to two seconds. The upstream producer has no detached task.

## Task Commit

`8a6f6d3` — add native streaming-drop and unary-cancellation evidence.

## Verification

- Lifetime filter: **2 passed**, none ignored; both new cases actually selected.
- Complete `antigravity_native` filter: **40 passed**, none ignored. Log: `/tmp/shunt-11-06-native-tests.log`.
- Serial all-feature workspace suite: **2,637 passed, 2 ignored**, 26 result groups; exit 0. Log: `/tmp/shunt-11-06-tests.log`.
- Format check, warnings-denied Clippy, and diff whitespace check: passed.
- No production code, dependency, public setting, credential writeback, deprecated CLI test, or generated wiki change in this plan.

## Deviations and Review

Independent GLM review correctly identified the missing native capacity fixture but missed existing module-local origin/affinity/tool tests. The primary checked those tests directly rather than adopting the incomplete gap report. Only the actual lifetime gap was filled.

The tests passed against the existing RAII implementation immediately. Inspection confirmed the behavior already existed; this is characterization coverage, not a fabricated RED/GREEN bug fix. No production change was needed.

## Documentation and Next Phase Readiness

README, engineering notes, and all maintained site locales are considered by the dependent same-phase documentation plan 11-07. Wiki remains untouched. Native behavior is ready for that evidence-backed documentation and final phase gates; these results do not claim live-provider verification.

## Self-Check: PASSED

The committed test file and both named cases exist; the recorded commands completed successfully.
