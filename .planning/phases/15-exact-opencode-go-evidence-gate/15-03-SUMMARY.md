---
phase: phase-15-exact-opencode-go-evidence-gate
plan: 03
subsystem: documentation
tags: [opencode-go, docs, evidence-gate]
requires:
  - phase: 15-02
    provides: Source-only four-candidate ledger and empty admitted set
provides:
  - Synchronized English README, provider, configuration and engineering-note guidance
  - Single-source navigation entry with four locale labels
  - Five deterministic per-file documentation contract tests
affects: [15-04, 15-05, 16]
tech-stack:
  added: []
  patterns: [Pure-Rust per-file documentation assertions]
key-files:
  created:
    - site/src/content/docs/providers/opencode-go.md
    - docs/opencode-go-evidence-gate.md
    - tests/opencode_go_docs.rs
  modified:
    - README.md
    - site/src/content/docs/reference/configuration.md
    - site/src/lib/i18n.ts
requirements-completed: [OGO-01, OGO-02, OGO-03]
coverage:
  - id: english-docs
    description: Empty-admission and conditional session contract is synchronized across English surfaces
    verification:
      - kind: integration
        ref: tests/opencode_go_docs.rs#opencode_go_docs_readme
        status: pass
      - kind: integration
        ref: tests/opencode_go_docs.rs#opencode_go_docs_provider
        status: pass
      - kind: integration
        ref: tests/opencode_go_docs.rs#opencode_go_docs_configuration
        status: pass
      - kind: integration
        ref: tests/opencode_go_docs.rs#opencode_go_docs_engineering_note
        status: pass
  - id: navigation
    description: Provider navigation link and all locale labels are present in the single i18n source
    verification:
      - kind: integration
        ref: tests/opencode_go_docs.rs#opencode_go_docs_nav_i18n
        status: pass
human_judgment: false
duration: 15min
completed: 2026-09-08
status: complete
---

# Phase 15 Plan 03: English documentation evidence gate

Published synchronized English zero-support guidance. All explicit Go
selections are documented as pre-credential rejection with no credential or
session header emitted while the admitted set is empty. The README, provider
guide, configuration reference, and engineering note describe the four
source-only candidates, strict authoritative terminals, and future reuse of the
matching Chat contract with a conditional opaque conversation-scoped
`x-opencode-session` only at the canonical destination. Locale mirrors remain
owned by plan 05; generated `wiki/` remains untouched.

## Verification

- RED: isolated integration binary selected all 5 named tests and failed all 5
  on missing Go surfaces (behavioral failures, not compilation failures).
- GREEN: `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test opencode_go_docs -- --test-threads=1` — 5 passed, 0 failed.
- Mutation: temporarily removed the provider's `SHUNT_OPENCODE_GO_API_KEY`
  token; the same binary selected 5 tests and failed exactly
  `opencode_go_docs_provider` (exit 101); token restored and GREEN rerun.
- `cargo fmt --all -- --check` passes under the isolated wrapper.
- Production OpenCodex config SHA/mtime and backup inventory remained unchanged
  for every stateful command.

## Task commit

- `42c3089` — `test: gate OpenCode Go documentation surfaces`

## Deviations

None.

## Self-check: PASSED

All six owned implementation files and the summary are present; unrelated
`.planning/config.json`, `.planning/state.json`, `.gsd/`, and milestone lock
dirt were not staged.
