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

## Milestone: v2 — Provider Compatibility

**Completed locally:** 2026-09-09 | **Phases:** 8 | **Plans:** 48

### What Was Built

Strict shared preservation boundaries; Gemini/Antigravity and Cursor hardening;
generic Chat; separate Command Code products; deny-by-default Go evaluation;
durable evidence/notices and four-locale release documentation.

### What Worked

Synthetic real-router fixtures, mutation checks and bounded cancellation probes
exposed defects without live credentials. Independent high-effort review caught
notice gaps. Production fingerprint isolation remained intact throughout.

### What Was Inefficient

Intermittent delegation controls, inconsistent historical metadata, and repeated
final artifact reconciliation consumed time. Source/schema evidence cannot
substitute for current live availability; preflight correctly skipped unknown
cost/output/refresh bounds. Full regression per intermediate wave was not run;
focused task feedback and final full gates were used and explicitly recorded.

### Patterns Established and Lessons

- Store permanent regression inputs outside active planning before archiving.
- Keep source, capture, live, static and Computer evidence distinct.
- Always set subagent thinking effort explicitly; do not wait on completed agents.
- Preserve correct assertions when correcting documentation or fixture paths.

### Cost Observations

No Shunt live generation: 0/8 approved attempts, US$0 planned. Development-agent
cost and model percentages were not measured and are not invented.

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Sessions | Phases | Key Change |
|-----------|----------|--------|------------|
| v1 | 1 | 8 | Established behavior-first porting with per-phase verification and final integration audit |
| v2 | Not measured | 8 | Durable test fixtures survive archival; source, live and Computer proof stay distinct |

### Cumulative Quality

| Milestone | Tests | Requirements | Verification Overrides |
|-----------|-------|--------------|------------------------|
| v1 | 2,049 library passed; 40 binary passed; all integrations passed | 20/20 | 0 |
| v2 | 2,979 passed; two existing ignored | 63/63 | 0 |

### Top Lessons (Verified Across Milestones)

1. Behavior-first fixtures and explicit verification boundaries remained useful across both milestones.
