---
phase: phase-15-exact-opencode-go-evidence-gate
plan: 06
status: complete
subsystem: documentation
tags: [opencode-go, locale-parity, regression]
requires:
  - phase: 15-03
    provides: English provider contract and shared assertion binary
  - phase: 15-05
    provides: Maintained locale provider contracts
provides: [Four synchronized overview guides, Four per-file regression assertions]
affects: [15-04, 16]
tech-stack:
  added: []
  patterns: [Heading-scoped per-file documentation assertions]
key-files:
  created: []
  modified:
    - tests/opencode_go_docs.rs
    - site/src/content/docs/guides/providers.mdx
    - site/src/content/docs/ko/guides/providers.mdx
    - site/src/content/docs/ja/guides/providers.mdx
    - site/src/content/docs/zh-cn/guides/providers.mdx
requirements-completed: []
completed: 2026-09-08
commits: 2
plan_head_before: 5c9227c77880c94e1d521223f5e14e817b62280e
---

# Phase 15 Plan 06: Provider overview parity

All four overview guides describe the explicit Go preset, canonical destination,
exact key, empty admission, primary rejection and fallback removal. Each links
to its matching locale provider page. Existing provider rows are unchanged;
no wiki, runtime, public configuration, or credential behavior changed.

## Verification

All runs used `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test opencode_go_docs -- --test-threads=1`.

- Behavioral RED: 19 selected; 15 passed and four guide tests failed on missing
  `## OpenCode Go` sections. Test-only commit: `827c822`.
- Initial content run: English phrase split by a newline failed the exact
  assertion. Kept the phrase contiguous; no assertion weakened.
- GREEN: 19 passed, zero failed.
- Mutation: replacing the Japanese guide's API key token failed exactly
  `opencode_go_docs_guides_ja` (18 passed, one failed, exit 101).
- Restored GREEN: 19 passed, zero failed. No mutation marker remains.
- All stateful runs confirmed production config SHA/mtime and backup inventory
  unchanged. No provider calls or credential inspection occurred.

Implementation commit: `151d0fc`. No skipped owned checks or blocking stubs.
The intentionally empty admitted set is the approved product contract.

## Deviations

Executed inline under the GSD host fallback after subagent dispatch controls
became unavailable. The suite contains 19 tests, exceeding the planned minimum
18 because plan 03 added a section-boundary regression test. An initial mutation
attempt also saw the pre-existing English phrase failure; only the later isolated
single-failure mutation above is accepted as mutation evidence.

## Self-Check: PASSED

All five owned files and both task commits exist. Only additive guide sections
were committed; unrelated planning/configuration dirt was preserved.
