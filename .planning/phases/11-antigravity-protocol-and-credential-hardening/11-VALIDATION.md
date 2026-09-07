---
phase: "11"
slug: "antigravity-protocol-and-credential-hardening"
status: complete
nyquist_compliant: true
wave_0_complete: true
created: "2026-09-07"
---

# Phase 11 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness, Tokio, Axum `Router::oneshot`, synthetic loopback upstreams |
| **Config file** | `Cargo.toml`; existing fixtures in `tests/` and module-local unit tests |
| **Quick run command** | `cargo test --all-features antigravity_native` |
| **Full suite command** | `cargo test --all-features --workspace -- --test-threads=1` |
| **Estimated runtime** | ~180 seconds |

---

## Sampling Rate

- **After every task commit:** Run the task's focused `cargo test --all-features <filter>` command.
- **After every plan wave:** Run `cargo test --all-features antigravity_native` plus every focused filter introduced by that wave.
- **Before `/gsd:verify-work`:** Format, Clippy with warnings denied, the serial full workspace suite, scope/docs scans, and secret scans must be green.
- **Max feedback latency:** 300 seconds for focused work; the serial release gate may run longer.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 11-01-01 | 01 | 1 | ANT-06, ANT-07 | T-11-06 | One bounded, terminal-strict always-SSE parser serves both downstream modes | integration | `cargo test --all-features antigravity_native_sse` | ✅ yes | ✅ passed |
| 11-02-01 | 02 | 2 | ANT-01 | T-11-01 | Bearer credentials never reach a non-canonical origin or redirect | unit + integration | `cargo test --all-features antigravity_native_origin` | ✅ yes | ✅ passed |
| 11-03-01 | 03 | 3 | ANT-02, ANT-04 | T-11-02, T-11-04 | One immutable account tuple and fresh exact catalog evidence gate dispatch | integration | `cargo test --all-features antigravity_native_affinity` | ✅ yes | ✅ passed |
| 11-04-01 | 04 | 4 | ANT-03 | T-11-03 | Exact envelope and opaque account/conversation-scoped session identity | unit + integration | `cargo test --all-features antigravity_native_envelope` | ✅ yes | ✅ passed |
| 11-04-02 | 04 | 4 | ANT-05 | T-11-05 | Only authentic account/session-bound signatures and ordered tool history replay | unit + integration | `cargo test --all-features antigravity_native_tool_signature` | ✅ yes | ✅ passed |
| 11-05-01 | 05 | 5 | ANT-08 | T-11-08 | At most one same-account pre-commit 401 refresh/replay; all committed/ambiguous cases terminate | integration | `cargo test --all-features antigravity_native_401` | ✅ yes | ✅ passed |
| 11-06-01 | 06 | 6 | ANT-01–ANT-08 | T-11-01–T-11-08 | Real-router parity and cancellation evidence remain synthetic and bounded | integration | `cargo test --all-features antigravity_native_lifetime` | ✅ yes | ✅ passed |
| 11-07-01 | 07 | 7 | ANT-01–ANT-08 | T-11-01–T-11-08 | Documentation, scope, exclusion, and release evidence is executable and locale-complete | release | `bash scripts/check_phase11_scope.sh` | ✅ yes | ✅ passed |

---

## Wave 0 Requirements

- [x] Create a native Antigravity conformance test surface distinct from `tests/antigravity_process.rs`, using only synthetic credentials and loopback/oneshot fixtures.
- [x] Add focused filters `antigravity_native_sse`, `antigravity_native_origin`, `antigravity_native_affinity`, `antigravity_native_envelope`, `antigravity_native_tool_signature`, `antigravity_native_401`, and `antigravity_native_lifetime` as their corresponding tasks begin.
- [x] Reuse the existing crate-private credential resolver seam; do not read or mutate a real credential file.

---

## Manual-Only Verifications

All phase behaviors have automated verification. Live credentials and live Cloud Code Assist calls are intentionally excluded.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 300s for focused tests
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** Automated evidence verified 2026-09-07. Seven filters selected 41 passing tests, none ignored; full workspace 2,638 passed, two pre-existing ignored. See 11-07-SUMMARY.md and 11-VERIFICATION.md for review corrections, exact commands, and confidence limits. No live-provider verification is claimed.
