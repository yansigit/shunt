---
phase: 11-antigravity-protocol-and-credential-hardening
plan: "07"
subsystem: documentation
tags: [antigravity, release, scope, review]
requires:
  - phase: "11-06"
    provides: integrated native lifetime and conformance evidence
provides:
  - English, Korean, Japanese, and Chinese native contract parity
  - Exact baseline scope, exclusion, and heuristic secret release gate
  - Catalog redirect refusal and isolated replay fixtures
affects: ["12", "16"]
tech-stack:
  added: []
  patterns: [exact endpoint catalog evidence, executable narrow release checks]
key-files:
  created:
    - scripts/check_phase11_scope.sh
  modified:
    - README.md
    - README.ko.md
    - README.ja.md
    - README.zh-CN.md
    - docs/notes/antigravity-daily-host.md
    - site/src/content/docs/providers/antigravity.mdx
    - site/src/content/docs/ko/providers/antigravity.mdx
    - site/src/content/docs/ja/providers/antigravity.mdx
    - site/src/content/docs/zh-cn/providers/antigravity.mdx
    - src/auth/antigravity/catalog.rs
    - src/auth/shared.rs
    - src/model/antigravity_request.rs
    - src/server.rs
    - src/server/antigravity_replay_tests.rs
    - src/server/antigravity_replay_tests/fixtures.rs
requirements-completed: [ANT-01, ANT-02, ANT-03, ANT-04, ANT-05, ANT-06, ANT-07, ANT-08]
duration: 35min
completed: 2026-09-07
status: complete
plan_head_before: 7e92231
---

# Phase 11 Plan 07: Verified documentation and release gates

All eight maintained documentation surfaces now state exact native admission,
always-SSE parity, canonical destinations, contextual v2 history, one-shot
same-account recovery, cancellation, and synthetic-only confidence limits.
Deprecated CLI warnings and existing login/writeback behavior are preserved.
Historical engineering observations are explicitly separated from current
hermetic evidence; the old causal claim about the historical 429 is removed.

## Commits

- `ade4417`: reviewed catalog redirect fix, fixture isolation, and localized contracts.
- `c25cfa5`: exact executable scope gate and review/remediation record.

## Review-driven changes

The independent standard GLM review found no critical issue and three warnings.
The owner confirmed and corrected the unguarded auth-file environment override.
Replay fixture config loading now uses the same environment mutex; the deliberate
TTFB timeout is 2000ms, with exact hit counts and zero-refresh assertions unchanged.

Catalog redirect tests failed against the original implementation and passed
afterward: catalog evidence can no longer be supplied through any redirect,
including another path on the same origin. Initial roots use the existing
canonical predicate. The reviewer overstated cross-host bearer forwarding;
reqwest strips sensitive headers across hosts, but the endpoint-authority gap
was real. No auth-file persistence or public setting was added.
The redundant catalog membership check was removed and the legacy fixture
wrapper made private/test-only. The structural pre-output commitment comment
remains intentionally explicit. Full original findings plus triage are in
`11-REVIEW.md`.

## Release evidence

- `bash scripts/check_phase11_scope.sh`: passed; baseline is the actual first
  introduction of 11-01-PLAN, `608988fb9c364e05baf71f2e437b26c5785dd122`.
  Committed/worktree changes, renames/deletions, and Git-visible untracked paths
  are checked with NUL-safe path inventory. Only the three existing user-state
  exceptions are ignored. Exact GSD 09-VERIFICATION and 11-REVIEW paths are
  documented required evidence exceptions, not general scope widening.
- Scanner self-probes reject credential-shaped and high-entropy synthetic strings
  while accepting explicit placeholders/field names. A temporary untracked
  filename containing spaces was rejected and removed. The restored gate passes.
  Entropy detection is explicitly heuristic; ignored filesystem content is not
  claimed to be exhaustively scanned.
- Seven focused filters passed, each selecting nonzero tests:
  SSE **5**, origin **6**, affinity **10**, envelope **2**, tool signature **6**,
  401 **10**, lifetime **2**: **41 passed, 0 ignored**.
- Parallel `cargo test --all-features --lib antigravity_native_401`: **10 passed**.
- Serial all-feature workspace suite: **2,638 passed, 2 pre-existing ignored**,
  **26 result groups**, exit 0; log `/tmp/shunt-11-07-tests.log`.
- Format check, warnings-denied Clippy, and diff whitespace check: passed.
- `npm ci --no-audit --no-fund` used the existing site lockfile unchanged;
  `npm run build` passed: **161 pages, 4 languages**. MDX validation passed.
  Existing Vite deprecation/Pagefind stemming notices are not build failures.
  Log: `/tmp/shunt-11-07-site-build.log`.

## Safety and deviations

The three backed-up source files remain byte-for-byte equal to the owner-only
backup at `/Users/user/shunt-settings-backup-fZlDSM`. No production opencodex
home/port, real subscription request, generated wiki, secret, or config change
was used for verification. The user's dirty planning config remains untouched.

The scope-script subagent was interrupted after an extended wait without an
artifact; the owner implemented and tested the script inline. Review-driven
fixes were narrow execution corrections to already approved Phase 11 safety
invariants. No tests were weakened and no live-availability claims were made.

## Self-Check: PASSED

Both task commits and all required documentation/script artifacts exist.
All release commands completed; phase-level verification is the next gate.
