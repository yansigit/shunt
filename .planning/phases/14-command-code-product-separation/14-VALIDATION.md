---
phase: "14"
slug: command-code-product-separation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-08"
---

# Phase 14 — Validation Strategy

## Test Infrastructure

Rust libtest, Tokio, Axum and existing mock/TLS socket helpers; Cargo.toml. No new test framework is needed. Every stateful command inherits isolation via `node /tmp/shunt-phase12-isolated-run.cjs` from `/Users/user/.codex/worktrees/0466/shunt`.

Quick baseline: `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test openai_chat_conformance -- --test-threads=1`.
Full suite: `node /tmp/shunt-phase12-isolated-run.cjs env 'RUSTFLAGS=-D warnings' cargo test --all-features --workspace`.
Phase-specific targets/filters must be concretely added by the planner and must reject zero selected tests. No Command Code tests exist yet; this draft claims no execution.

## Sampling Rate

- After each task: its focused automated check; actual RED recorded before GREEN for TDD tasks.
- After each plan wave: focused phase suite plus directly affected regressions.
- Before verification: format, warning-denied Clippy and full workspace suite, docs site build, owned CLI/curl smoke.
- Target focused feedback latency: 60 seconds, measured during execution; do not infer it from this draft.

## Per-Task Verification Map

Planner must replace these provisional groups with actual task IDs, wave dependencies, threat IDs and runnable target/filter commands before plan checking.

| Group | Requirements | Secure behavior | Type | Status |
| --- | --- | --- | --- | --- |
| Product tracer | CCK-01/02, CCS-01/02 | Distinct authenticated destinations, no credential crossover | real gateway/TLS/mock | pending |
| Request/history | CCS-03/04 | Exact identity, supported model/effort, authentic tools and explicit missing/orphan carriers | unit + gateway | pending |
| Checked response | CCS-05/06 | Bounded ordered semantics, trustworthy terminal, explicit malformed/error/EOF failure | byte-split + gateway | pending |
| Lifetime/replay | CCS-07/08 | No unsafe replay; disconnect frees upstream and admission | real sockets | pending |
| Product matrix/docs | CCK-03, CCS-08 | Full scenarios and accurate maintained locales | integration + CLI/site | pending |

## Wave 0 Requirements

- [ ] Dated protocol-evidence ledger resolves proprietary terminal grammar and exact supported field/model facts before fixtures encode assumptions.
- [ ] Synthetic source-derived NDJSON corpus and real gateway tracer test target.
- [ ] Canonical-host TLS test seam without public off-origin bearer bypass.
- [ ] Credential fixtures use temporary files; byte/mtime/inventory checks verify read-only behavior.
- [ ] Every not-yet-created target referenced by plans has an owned creation task.

## Manual-Only Verifications

Live provider acceptance (including minimal envelope and version) is not established by hermetic tests; Phase 16 must separately label opt-in live results or explicit skips. Computer evaluation previously blocked; approved CLI fallback is not a passed visual audit. No GUI is changed by this phase.

## Validation Sign-Off

- [ ] All tasks have automated verification or owned Wave 0 dependency.
- [ ] No three consecutive tasks lack automated coverage.
- [ ] All missing target paths created; no watch mode; zero-selection guard enforced.
- [ ] All CCK/CCS matrix items actually executed with evidence.
- [ ] Feedback latency measured; full gates and owned smoke pass.
- [ ] nyquist_compliant set true only after actual validation.

**Approval:** Pending execution.
