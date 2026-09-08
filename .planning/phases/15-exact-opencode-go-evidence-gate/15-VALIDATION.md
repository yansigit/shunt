---
phase: "15"
slug: "exact-opencode-go-evidence-gate"
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-08"
---

# Phase 15 — Validation Strategy

## Test Infrastructure

Rust libtest and existing Axum/loopback fixtures; Cargo.toml owns the test setup.
No new dependency. All stateful commands run through the isolated wrapper.

- Quick: `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features opencode_go -- --test-threads=1`
- Full: `node /tmp/shunt-phase12-isolated-run.cjs env 'RUSTFLAGS=-Dwarnings' cargo test --all-features --workspace`
- Formatting and Clippy: isolated `cargo fmt --all --check` and `cargo clippy --all-targets --all-features -- -D warnings`.
- Site: isolated `npm --prefix site run build`.

## Sampling Rate

After every task run its exact nonzero-selecting focused checks. After each wave
run full regression. Before verify-work, all repository quality gates and owned
smoke must pass. Expected warm feedback under 120s; record actual latency.

## Per-Task Verification Map

Planner must replace these provisional groups with actual plan/task IDs.

| Group | Requirement | Threat / secure behavior | Test type | Planned evidence | Status |
|---|---|---|---|---|---|
| 15-02 Task 1 | OGO-01/02 | No unsupported positive claims | source + deterministic ledger validation | Four exact candidates; all required fields; no captures falsely labelled live | pending |
| 15-01 Task 1 | OGO-02/04 | Zero credential access and dispatch | real router | Explicit Go selections fail before resolver/network in both output modes | pending |
| 15-01 Task 2 | OGO-03/04 | No Go credential/session egress | router + config | Primary, fallback, inbound Codex, count-tokens, off-origin and wrong auth | pending |
| 15-02 Task 2 | OGO-01..04 | Generic providers preserved | workspace + CLI | All locales, generic-provider positive controls, isolated negative CLI smoke | pending |

## Wave 0 Requirements

- [ ] New Go gateway fixtures and ledger checks created before implementation.
- [ ] Exact plan filters and nonzero selection failure directions established.
- [ ] No actual credentials needed: injected counters and visibly synthetic values.

## Manual-Only Verifications

No live success claim in the empty-admission implementation. Promoting any tuple
requires separate bounded opt-in captured/live evidence and actual matching-wire
hermetic tests; existing source-only metadata cannot satisfy that future gate.

## Validation Sign-Off

- [ ] All tasks have automated checks and real failure directions.
- [ ] No three consecutive tasks without automated verification.
- [ ] All missing test targets covered by Wave 0 creation.
- [ ] No watch flags or disabled tests.
- [ ] Actual feedback latency recorded.
- [ ] Nyquist audit completed before marking validated.

**Approval:** pending executed checks.
