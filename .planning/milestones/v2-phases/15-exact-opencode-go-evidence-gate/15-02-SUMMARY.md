---
phase: phase-15-exact-opencode-go-evidence-gate
plan: 02
subsystem: testing
tags: [opencode-go, provenance, admission]
requires:
  - phase: 15-01
    provides: Empty runtime admission and real router boundary evidence
provides:
  - Four-candidate source-only ledger with explicit unknowns and empty admission
  - Deterministic field/provenance/admission mutation checks
affects: [15-03, 15-04, 16]
tech-stack:
  added: []
  patterns: [Fenced JSON provenance ledger]
key-files:
  created: [tests/opencode_go_evidence.rs, .planning/phases/15-exact-opencode-go-evidence-gate/15-EVIDENCE.md]
  modified: [.planning/phases/15-exact-opencode-go-evidence-gate/15-EDGE-COVERAGE.json]
key-decisions:
  - Absent Omen and Muse 1.3 candidates retain unknown wire; no family inference
requirements-completed: [OGO-01, OGO-02]
coverage:
  - id: ledger
    description: Exact candidate identities, source provenance, required fields and empty admission
    verification:
      - kind: integration
        ref: tests/opencode_go_evidence.rs#opencode_go_ledger
        status: pass
      - kind: integration
        ref: tests/opencode_go_evidence.rs#opencode_go_ledger_rejects_mutations
        status: pass
    human_judgment: false
duration: 10min
completed: 2026-09-08
status: complete
---

# Phase 15 Plan 02: Source-only evidence ledger

Four dated candidate records remain unadmitted; missing fields, false provenance,
family inference and nonempty admission are rejected by executable tests.

## Accomplishments

- Ledger records the exact evaluation destination, source pin, wire, headers,
  context, modalities, effort, tools, filtering, terminal and session evidence.
- Seven rejection categories are explicit. OGO-02's manual evidence assumption
  remains unresolved; no live or captured support is claimed.
- EDGE records retain their schema and distinguish planned dispositions from
  executed results; primary/fallback/native semantics now match the implementation.

## Task Commits

- `654d8c3`: RED ledger requirement assertions.
- GREEN commit immediately preceding this summary: ledger, tests and RED record.

## Verification

Wrapper-isolated warnings-denied ledger command selected 2 tests: 2 passed,
0 failed. Mutation checks remove each required field, invent admission or wire,
falsify provenance, and remove rejection accounting; each is rejected.
Initial file-load failure was not accepted as RED. Explicit missing-artifact
assertions then failed; the TAP bridge ran the real Rust tests and GSD returned
RED_EVIDENCE_OK (15-02-RED.json). Formatting completed. Full release gates belong
to 15-04; no new full-workspace or live-provider result is claimed here.
Every wrapper invocation confirmed unchanged production fingerprints/inventory.

## Deviations from Plan

Root executed inline after the host stopped exposing the next executor dispatch
control. Added a persisted RED evidence record and a temporary TAP bridge because
GSD's RED classifier parses TAP, not Rust libtest output. No dependency added.

## Next Phase Readiness

Ready for 15-03 English documentation. Phase 15 remains incomplete.

## Self-Check: PASSED

Ledger and test files exist; actual nonzero-selection checks passed. Requirements
shared with later plans are not marked complete in the milestone tracker yet.
