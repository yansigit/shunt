---
phase: 16-cross-provider-release-gate
plan: 02
subsystem: security
tags: [mit, provenance, credentials, release]
requires:
  - phase: 16-01
    provides: Durable release ledger
provides:
  - Complete Cursor and Command Code upstream notices
  - Release metadata and canonical credential-boundary regression tests
affects: [16-04, 16-05]
actuals:
  tasks: 2
  commits: 3
tech-stack:
  added: []
  patterns: [Separate static metadata checks from behavioral auth verification]
key-files:
  created: [tests/release_security.rs, 16-SECURITY.md]
  modified: [THIRD-PARTY-NOTICES.md]
key-decisions:
  - Historical jcode copy revision stays unknown; pin present-day original-license verification separately
  - Preserve pre-existing OAuth semantics and dev dependencies
patterns-established:
  - Durable tests avoid active planning and production credential stores
requirements-completed: [REL-02, REL-03, REL-06]
coverage:
  - id: D1
    description: Complete upstream attribution and sanitized release metadata
    requirement: REL-03
    verification:
      - kind: integration
        ref: tests/release_security.rs#release_security_notices_cover_translated_material
        status: pass
    human_judgment: true
    rationale: Final independent notice and provenance reconciliation remains plan 16-05 work
  - id: D2
    description: Canonical subscription destination and read-only synthetic credential behavior
    requirement: REL-06
    verification:
      - kind: integration
        ref: tests/release_security.rs#credential_boundary_release_canonical_subscription_config
        status: pass
      - kind: unit
        ref: src/auth/command_code/tests/matrix.rs#command_code_matrix_auth_shapes_file_invariance_and_byte_bounds
        status: pass
    human_judgment: false
duration: 6min
completed: 2026-09-09
status: complete
---

# Phase 16 Plan 02 Summary

Cursor/OpenCodex and jcode notices now retain distinct provenance, with three passing release-security tests and seven passing behavioral Command Code auth tests.

## Task commits

1. RED checks: `4b9eba4` — 2 selected, 1 intended notice failure and 1 pass; compiled successfully.
2. Notice fix and GREEN: `fedc8bf` — all 3 release-security tests pass, including eight canonical-destination cases.
3. Behavioral auth and scope record: `09d2123` — 7 selected existing auth tests pass; separate exact matrix run also passes.

## Decisions and deviations

Original-source research by Omen/high established the jcode MIT copyright and inspected revision. No historical copy revision was fabricated. Scope scans remain root-owned git comparisons, not permanent tests that depend on an active planning directory or mutable git status. Reused the existing behavioral auth matrix instead of mistaking source comments for proof. No runtime edits or dependencies added.

## Remaining gates

Production fingerprint/inventory checks were unchanged throughout. No real credential contents or live provider requests were used. REL-02/REL-03 flagged assumptions remain for final independent review; this plan-local completion does not close global release requirements. Final repository gates, smoke dispositions, and milestone audit/completion remain pending.

## User Setup Required

None.
