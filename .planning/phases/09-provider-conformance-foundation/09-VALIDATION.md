---
phase: "09"
slug: "provider-conformance-foundation"
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-06"
---

# Phase 09 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness + Tokio/Axum integration tests |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test --test failover --test retry --test passthrough` |
| **Full suite command** | `cargo test --all-features --workspace` |
| **Estimated runtime** | ~120 seconds |

---

## Sampling Rate

- **After every task commit:** Run the task's focused `cargo test` target or module filter.
- **After every plan wave:** Run `cargo test --all-features --workspace`.
- **Before `/gsd:verify-work`:** Format, clippy, and the full suite must be green.
- **Max feedback latency:** 120 seconds for the full suite; focused task checks should stay below 60 seconds.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 09-01-01 | 01 | 1 | PRES-01, SAFE-04 | T-09-01 | Redispatch is monotonic and closes before visible output or replay-unsafe tool activity | unit + integration | `cargo test redispatch` | ❌ W0 | ⬜ pending |
| 09-01-02 | 01 | 1 | SAFE-01, SAFE-03 | T-09-02 | Bounded parsing fails closed on malformed, oversized, duplicate-terminal, and cut streams | unit + integration | `cargo test --test failover truncated` | ✅ | ⬜ pending |
| 09-02-01 | 02 | 2 | PRES-02, SAFE-06 | T-09-03 | Tool pairs and authentic continuation metadata survive only safe replay | integration | `cargo test --test codex_websocket_fallback` | ✅ | ⬜ pending |
| 09-02-02 | 02 | 2 | SAFE-02, SAFE-07 | T-09-04 | Cancellation releases response-owned permits and upstream work | unit + integration | `cargo test cancellation` | ✅ | ⬜ pending |
| 09-03-01 | 03 | 2 | PRES-03, PRES-04, PRES-05 | T-09-05 | Existing Codex and Vercel behavior remains byte- and shape-compatible | integration | `cargo test --test passthrough --test inbound_codex_endpoint` | ✅ | ⬜ pending |
| 09-03-02 | 03 | 2 | SAFE-05 | T-09-06 | Credentials remain provider/destination bound and redact in diagnostics | unit + integration | `cargo test credential` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] The first plan creates the focused redispatch-state regression module before implementation.
- [ ] Each plan adds its own missing focused fixtures before changing production behavior.
- [ ] Existing Rust test infrastructure covers all other phase requirements; no framework installation is needed.

---

## Manual-Only Verifications

All phase behaviors have automated verification. Live provider credentials are not required for this conformance foundation.

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
