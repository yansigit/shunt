---
phase: "13"
slug: generic-openai-chat-completions
status: executed
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

Every task ID with plan/wave, CHAT requirement, threat reference, test type,
exact isolated command, fixture location and execution status. Fixture paths
marked (Wave 0) are created by plan 13-01; later plans extend those files.
Statuses below reflect executed evidence in plan summaries and 13-05-EVIDENCE.md.
All commands run through the isolated wrapper (fresh OPENCODEX_HOME, non-10100
port) from worktree /Users/user/.codex/worktrees/0466/shunt.

| Task ID | Plan/Wave | CHAT req | Threat ref | Test type | Command (isolated) | Fixture | Status |
|---|---|---|---|---|---|---|---|
| 13-01-T1 | 01/1 | CHAT-01, CHAT-05 | T-13-01 | Router conformance (red-first tracer) | \`cargo test --all-features --test openai_chat_conformance openai_chat_tracer_unary -- --test-threads=1\` | tests/openai_chat_conformance.rs (Wave 0) | executed — pass |
| 13-01-T2 | 01/1 | CHAT-06 | T-13-01 | Router conformance (streaming tracer) | \`cargo test --all-features --test openai_chat_conformance openai_chat_tracer_streaming -- --test-threads=1\` | tests/openai_chat_conformance.rs (Wave 0) | executed — pass |
| 13-02-T1 | 02/2 | CHAT-03 | T-13-02 | Pure translation unit | \`cargo test --all-features --test openai_chat_translate -- --test-threads=1\` | tests/openai_chat_translate.rs (Wave 0) | executed — pass |
| 13-02-T2 | 02/2 | CHAT-04 | T-13-02 | Pure translation unit + wire fixture | \`cargo test --all-features --test openai_chat_translate tool -- --test-threads=1\` | tests/openai_chat_translate.rs (Wave 0) | executed — pass |
| 13-02-T3 | 02/2 | CHAT-02 | T-13-02 | Pure unit + boot rejection fixture | \`cargo test --all-features --test openai_chat_translate endpoint -- --test-threads=1\` | tests/openai_chat_translate.rs (Wave 0) | executed — pass |
| 13-03-T1 | 03/3 | CHAT-05 | T-13-03 | Pure response-machine unit + wire fixture | \`cargo test --all-features --test openai_chat_translate response -- --test-threads=1\` | tests/openai_chat_translate.rs (Wave 0) | executed — pass |
| 13-03-T2 | 03/3 | CHAT-07 | T-13-03 | Pure unit + router terminal fixtures | \`cargo test --all-features --test openai_chat_conformance openai_chat_terminal -- --test-threads=1\` | tests/openai_chat_conformance.rs (Wave 0) | executed — pass |
| 13-04-T1 | 04/4 | CHAT-06 | T-13-04 | Pure assembly unit + streaming fixture | \`cargo test --all-features --test openai_chat_translate assembly -- --test-threads=1\` | tests/openai_chat_translate.rs (Wave 0) | executed — pass |
| 13-04-T2 | 04/4 | CHAT-06, CHAT-09 | T-13-04 | Boundary probes (plus/minus one) | \`cargo test --all-features --test openai_chat_translate bound -- --test-threads=1\` | tests/openai_chat_translate.rs (Wave 0) | executed — pass |
| 13-04-T3 | 04/4 | CHAT-08 | T-13-04 | Wire classification fixtures (redirect, timeout) | \`cargo test --all-features --test openai_chat_conformance openai_chat_auth -- --test-threads=1\` | tests/openai_chat_conformance.rs (Wave 0); extends tests/failover.rs, tests/retry.rs (existing) | executed — pass |
| 13-05-T1 | 05/5 | CHAT-09 | T-13-05 | Full 12-scenario conformance matrix | \`cargo test --all-features --test openai_chat_conformance -- --test-threads=1\` | tests/openai_chat_conformance.rs (Wave 0) | executed — pass |
| 13-05-T2 | 05/5 | CHAT-09 | T-13-05 | CHAT-09/boundary cap tri-probes | \`cargo test --all-features --test openai_chat_translate cap -- --test-threads=1\` | tests/openai_chat_translate.rs (Wave 0) | executed — pass |
| 13-05-T3 | 05/5 | CHAT-09 | T-13-05 | Docs surfaces + full chained gates | chained: fmt --check; clippy -D warnings; env 'RUSTFLAGS=-D warnings' cargo test --all-features --workspace; npm --prefix site run build (each step wrapped) | README x4, docs/, site EN+ko/ja/zh-cn; wiki/ untouched | executed — pass (2,875 tests; 165 pages) |

Threat references T-13-01 through T-13-05 map to the threat_model blocks of plans
13-01 through 13-05 respectively (ASVS level 1, block high).

Unclassified edge rows: CHAT-01/unclassified and CHAT-07/unclassified are authored in plans
13-01 and 13-05 as flagged behavioral assumptions — request-local credential/model
isolation and cancellation-by-ownership — asserted by fixtures, never silently
dropped. Both formerly unclassified rows now have explicit passing fixtures. All 14 EDGE-COVERAGE rows are resolved and covered by
the mapped tasks above.

## Wave 0 Requirements

- [x] tests/openai_chat_translate.rs: pure input/output semantics.
- [x] tests/openai_chat_conformance.rs: real router and isolated upstream.
- [x] End-to-end tracer verified before expansion; no stub-success implementation.
- [x] Named fixtures covering all12 AI-SPEC scenarios and14 edge-probe rows.

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
