---
phase: "04"
slug: native-compaction
status: validated
nyquist_compliant: true
wave_0_complete: true
created: "2026-09-05"
---

# Phase 04 — Validation Strategy

| Task ID | Plan | Wave | Requirement | Threat Ref | Automated Command | Status |
|---------|------|------|-------------|------------|-------------------|--------|
| 04-01-01 | 04-01 | 1 | COMP-01 | T-04-01-01, T-04-01-02 | `cargo test routing:: --lib && cargo test --test inbound_codex_endpoint compact_` | complete |
| 04-01-02 | 04-01 | 1 | COMP-01 | T-04-01-03, T-04-01-04 | `cargo test --test inbound_codex_endpoint && cargo test --test codex_multi_account` | complete |
| 04-02-01 | 04-02 | 2 | COMP-01 | T-04-02-01 | `git diff --check && test -z "$(git status --short -- wiki/)"` | complete |
| 04-02-02 | 04-02 | 2 | COMP-01 | T-04-02-02 | `cargo fmt --all --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features --workspace` | complete |

## Wave 0 Requirements

- Compact route registration/auth fixture.
- Pinned ChatGPT and exact native compact URL/body/header fixtures.
- Missing/malformed/duplicate model and unsupported-provider zero-network fixtures.
- Final upstream response status/body/safe-header fidelity fixture.

## Final Gate

All four rows must be complete; focused compact/ordinary ingress and multi-account tests, formatting, strict Clippy, the full workspace, diff checks, and the empty wiki guard must pass.

**Approval:** validated 2026-09-05 after focused and full repository gates passed.
