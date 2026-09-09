---
phase: phase-15-exact-opencode-go-evidence-gate
plan: 05
subsystem: documentation
tags: [opencode-go, docs, locales, evidence-gate]
requires:
  - phase: 15-03
    provides: English OpenCode Go contract and six-test documentation binary
provides:
  - Korean, Japanese, and Simplified Chinese README, provider, and configuration mirrors
  - Nine executable per-file locale parity assertions in the existing opencode_go_docs binary
  - Behavioral RED evidence, mutation failure, and restored GREEN evidence
affects: [16]
requirements-completed: []
---

# Phase 15 Plan 05: Locale OpenCode Go mirrors

Mirrored the zero-support OpenCode Go contract across all nine maintained locale
surfaces. Native-language prose preserves exact keys, candidate ids, canonical
URL, strict-terminal policy, pre-credential rejection, and conditional
`x-opencode-session` wording. Locale links point to locale pages without invented
English fragment anchors.

## Verification

- RED: `15-05-RED.json` records the first run after adding tests: 15 tests
  selected, 6 existing passed, and 9 new locale tests failed behaviorally on
  missing sections/files (not compilation errors).
- GREEN: isolated full binary command reported `15 passed, 0 failed`.
- Mutation: removing `SHUNT_OPENCODE_GO_API_KEY` from the Korean provider page
  caused exactly `opencode_go_docs_locale_provider_ko` to fail (14 passed, 1
  failed); the token was restored and the full binary returned 15 passed.
- Format: isolated `cargo fmt --all -- --check` passed.
- Production OpenCodex config mtime/SHA and backup inventory were unchanged.

## Commits

- `703b64aefc1dcbf3734705250126a03cd62e2473` — test-only RED locale gates
- `fe31c14b2dd40219e64b8cb5ac377e51b0cc378d` — locale documentation and parity implementation

## Scope

Exactly ten implementation files were changed: nine locale docs plus
`tests/opencode_go_docs.rs`. No English, wiki, state, roadmap, config, or lock
files were staged.
