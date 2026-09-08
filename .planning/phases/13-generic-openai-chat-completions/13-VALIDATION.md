---
phase: "13"
slug: generic-openai-chat-completions
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-08"
---

# Phase 13 — Validation Strategy

## Test Infrastructure

Existing Rust libtest, Tokio and Axum loopback fixtures; Cargo.toml/Cargo.lock.
No dependency installation. Read /tmp/shunt-phase12-isolated-run.cjs before use;
every test process tree must inherit fresh OPENCODEX_HOME and a non10100 port.
Wrapper snapshots production config mtime/SHA and invalid/backup inventory.

Quick command after fixtures exist:
`node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test openai_chat_translate`

Full suite:
`node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --workspace`

## Sampling Rate

Every task needs a concrete automated command and failure direction. A filter
selecting zero tests fails validation. Run focused tests after each task and
full suite before phase verification, plus fmt and warnings-denied Clippy.
Target focused feedback under60 seconds; measure actual duration, do not claim it.

## Per-Task Verification Map

Planner must replace this pending map with every real task ID, plan/wave,
CHAT requirement, threat reference, test type, exact command, fixture existence
and execution status. No task or test is claimed implemented yet.

## Wave 0 Requirements

- [ ] tests/openai_chat_translate.rs: pure input/output semantics.
- [ ] tests/openai_chat_conformance.rs: real router and isolated upstream.
- [ ] End-to-end tracer verified before expansion; no stub-success implementation.
- [ ] Named fixtures covering all12 AI-SPEC scenarios and14 edge-probe rows.

## Manual-Only Verifications

Computer UI checks supplement automated protocol tests. Primary agent performs
Computer actions; delegated Computer work requires GPT-6 Astra/high.
Current host blocks Terminal UI and isolated localhost browser navigation.
Record as blocked, not passed. Do not weaken safety controls to bypass it.
Synthetic tests do not establish subscription/provider live availability.

## Validation Sign-Off

- [ ] Every task has automated verification or explicit Wave0 dependency.
- [ ] No three consecutive tasks without automated feedback.
- [ ] All missing fixture paths implemented and nonempty filters confirmed.
- [ ] No watch-mode commands or swallowed nonzero statuses.
- [ ] Actual fmt, Clippy, full tests and smoke pass.
- [ ] Compliance set true only after executed evidence.

Approval: pending independent plan check and execution.
