---
phase: phase-15-exact-opencode-go-evidence-gate
plan: 04
status: complete
subsystem: verification
tags: [opencode-go, cli-smoke, release-gates]
requires:
  - phase: 15-01
    provides: Empty admission and real-router credential seam evidence
  - phase: 15-02
    provides: Four-candidate source-only ledger
  - phase: 15-03
    provides: English documentation
  - phase: 15-05
    provides: Maintained translations
  - phase: 15-06
    provides: Provider overview parity
provides: [Dedicated isolated CLI rejection smoke, Executed release verification map]
affects: [16]
tech-stack:
  added: []
  patterns: [Owned child-process cleanup, Loopback outbound sentinel]
key-files:
  created: []
  modified:
    - tests/check_cli.rs
    - .planning/phases/15-exact-opencode-go-evidence-gate/15-VALIDATION.md
requirements-completed: []
completed: 2026-09-08
commits: 2
plan_head_before: 90ee124
---

# Phase 15 Plan 04: Isolated CLI and release gates

The dedicated `opencode_go_cli_negative` test boots the owned gateway binary
with a fresh temporary home, synthetic key, canonical Go URL and non-10100 port.
It sends actual unary and streaming Messages requests and asserts Anthropic
400 unsupported-selection errors. A loopback proxy sentinel receives no
connection; the child is stopped and its home is removed. Crate-local counters
remain the separate pre-credential proof, with their HTTPS limitation documented.

## Verification

- Dedicated smoke: 1 passed, zero failed (also passed inside the full suite).
- Focused config 1/1, real router 5/5, ledger 2/2, documentation 19/19,
  ordered presets 1/1. No required focused invocation selected zero tests.
- `cargo fmt --all --check`: passed.
- `env RUSTFLAGS=-Dwarnings cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `env RUSTFLAGS=-Dwarnings cargo test --all-features --workspace`: 2,972 passed,
  zero failed, two pre-existing ignored; process exit 0.
- Site build: 173 pages, exit 0; 12 affected built pages passed exact token
  and locale-link checks after accounting for normalized trailing slashes.
- Every stateful command used `node /tmp/shunt-phase12-isolated-run.cjs`;
  production config mtime/SHA and backup inventory stayed unchanged.

## Commits

- `c5dc152` — dedicated CLI process smoke.
- `effaccc` — executed validation map and release results.

## Deviations and limits

Executed inline under the host delegation fallback. Task 1 adds tests of already
implemented behavior, so its first run was GREEN; it does not claim an invented
pre-implementation RED. Runtime gate mutation evidence belongs to plan 01.
The smoke also covers streaming-request admission, without claiming successful
Go SSE streaming. No credentials were inspected or written and no live provider
was called. OGO-02 remains a documented manual evidence assumption; zero tuples
are admitted. No GUI/Computer or Phase 16 acceptance is claimed.

## Self-Check: PASSED

Both owned files and task commits exist. No unexpected file deletions, generated
wiki edits, unrun owned gates, or blocking stubs were introduced. Whole-phase
review, Nyquist/security checks and final verification remain separate gates.
