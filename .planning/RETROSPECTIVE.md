# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1 — OpenCodex Port

**Shipped:** 2026-09-06
**Phases:** 8 | **Plans:** 25 | **Sessions:** 1 autonomous workflow

### What Was Built

- Bounded inbound Responses WebSockets and exact native routing/compaction.
- Quota-aware account resilience and strict Responses-to-Anthropic translation.
- Capability-aware fallback, opt-in collaboration fidelity, and bounded shutdown.

### What Worked

- Porting observable behavior and adversarial fixtures kept the design grounded
  without importing OpenCodex's generalized state and platform layers.
- Exact routing boundaries and native byte-fidelity tests made later translation
  and collaboration work safer to compose.
- Per-phase code, security, and full-suite gates caught issues before they could
  accumulate across the milestone.

### What Was Inefficient

- Early phase artifacts used several generations of GSD metadata, requiring a
  final normalization and verification-fingerprint refresh.
- The generated milestone task count recognized only one summary schema, so the
  authoritative 56-task count needed reconciliation from plan files.

### Patterns Established

- Keep native Responses traffic opaque; put translation behind unique exact
  route decisions.
- Make every parser, buffer, timeout, retry, and recovery boundary explicit and
  finite.
- Fail before credential lookup or network dispatch when fidelity cannot be
  guaranteed.

### Key Lessons

1. Behavioral fixtures are a better porting unit than another project's runtime
   architecture.
2. Verification fingerprints should be refreshed only after planning metadata
   reaches its final schema.
3. Process-owned runtime teardown can provide a lean cancellation boundary when
   tests prove future Drop and RAII release.

### Cost Observations

- Model mix: not recorded by the local workflow.
- Sessions: 1 autonomous workflow across two calendar days.
- Notable: parallel audits improved coverage; final metadata convergence was the
  main avoidable rework.

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Sessions | Phases | Key Change |
|-----------|----------|--------|------------|
| v1 | 1 | 8 | Established behavior-first porting with per-phase verification and final integration audit |

### Cumulative Quality

| Milestone | Tests | Requirements | Verification Overrides |
|-----------|-------|--------------|------------------------|
| v1 | 2,049 library passed; 40 binary passed; all integrations passed | 20/20 | 0 |

### Top Lessons (Verified Across Milestones)

1. Await evidence from a future milestone before declaring a cross-milestone trend.
