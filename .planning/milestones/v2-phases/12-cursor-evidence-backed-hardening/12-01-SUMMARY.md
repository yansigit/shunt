---
phase: 12-cursor-evidence-backed-hardening
plan: "01"
subsystem: testing
tags: [cursor, connect, protobuf, eof]
requires:
  - phase: 11-antigravity-protocol-and-credential-hardening
    provides: verified provider baseline
provides:
  - EOF failure through the whole gateway router
  - Schema-derived usage and malformed-wire fixtures
affects: [12-05, 12-06, 12-08]
tech-stack:
  added: [h2-dev, rcgen-dev, tokio-rustls-dev]
  patterns: [hermetic TLS with pinned production hostname]
key-files:
  created: [src/adapters/cursor/router_parity_tests.rs]
  modified: [src/adapters/cursor/agent.rs, src/adapters/cursor/connect.rs, src/adapters/cursor/test_frames.rs, src/server.rs, Cargo.toml, Cargo.lock]
key-decisions:
  - Bare EOF cannot authorize success; idle semantics remain for plan 05.
  - Schema encoding evidence is not a claim that the production parser consumes usage yet.
requirements-completed: [CUR-05, CUR-06]
coverage:
  - id: D1
    description: Whole-router EOF fails once while Connect END succeeds in JSON and SSE.
    verification:
      - kind: integration
        ref: src/adapters/cursor/router_parity_tests.rs#cursor_terminal_tracer
        status: pass
    human_judgment: false
  - id: D2
    description: Five usage facts and named malformed corpora lock evidence for later implementation.
    verification:
      - kind: unit
        ref: cargo test --lib --all-features cursor_w0
        status: pass
    human_judgment: false
completed: 2026-09-07
status: complete
---

# Phase 12 Plan 01 Summary

Bare upstream EOF now produces an error rather than a fabricated successful completion, proven through the real gateway router with an isolated TLS upstream.

## Accomplishments

- Red-first router fixtures failed with the former successful JSON/SSE response, then passed after the minimal production EOF fix. Existing EOF characterizations were tightened, not removed. Valid END still succeeds.
- Five schema-derived fixtures pin output deltas, absolute context checkpoints, inferred input, absent usage, negative delta encoding, and turn-ended encoding. These do not yet prove production usage relay.
- Named malformed protobuf/END corpora and valid/corrupt gzip contrasts prepare plan 06 without prematurely claiming strict production decoding.

## Task Commits

- `19e0499`: EOF production fix and hermetic whole-router regression.
- `bb0244b`: active Run usage encoding fixtures.
- `af159d3`: named malformed wire corpus.
- Follow-up test commit aligns the second old EOF characterization and removes a needless borrow found by clippy.

## Verification

- Cursor library suite: 227 passed, 0 failed, 1 ignored.
- All-feature workspace suite: passed, exit 0; existing ignored tests remain ignored.
- `cargo fmt --all --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- Existing binary smoke driver passed config validation, liveness, discovery, local upstream forwarding, and malformed-request rejection on isolated ports 31711/31712; owned processes and scratch config were cleaned up. This is a generic gateway smoke, not a live Cursor-provider proof. A mistaken temporary repository-path edit failed before build, was reverted, and the unchanged driver then passed.
- Every stateful test runner used a fresh isolated OPENCODEX_HOME and non-production port. Production config mtime/SHA-256 and invalid/backup inventory were unchanged before/after.

## Deviations and Issues

The full-router seam required a cfg(test) server wrapper and TLS-only dev dependencies (including lockfile updates). Production signatures, origin guards, credentials, and config keys are unchanged. Muse executor hit a rate limit before edits; the primary agent completed this slice locally. Two old EOF-success characterizations were explicitly flipped to the intended error behavior. Clippy identified one needless borrow, now corrected.

## Documentation and Readiness

README, engineering docs, and all maintained site locales were considered; coordinated Cursor behavior documentation remains owned by plan 08 in the same change set. Generated wiki is untouched. CUR-05/CUR-06 entries above identify this plan's contribution only, not phase-wide completion: idle/duplicate terminals, strict parsing, and usage relay still require plans 05–06. No live Shunt/Cursor success is claimed. Plan 02 proceeds next.
