---
phase: "01"
slug: inbound-responses-websocket
status: validated
nyquist_compliant: true
wave_0_complete: true
created: "2026-09-06"
---

# Phase 01 — Validation Strategy

> Retroactively reconstructed from executed plans, phase verification, and the
> green repository-wide test run.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test harness with Tokio, Axum, wiremock, and tokio-tungstenite fixtures |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test codex_endpoint::frame --lib` |
| **Full suite command** | `cargo test --test inbound_codex_websocket -- --test-threads=1` |
| **Estimated runtime** | under 5 seconds after compilation |

## Sampling Rate

- **After framing changes:** Run `cargo test codex_endpoint::frame --lib`.
- **After transport changes:** Run `cargo test --test inbound_codex_websocket -- --test-threads=1`.
- **Before phase verification:** Run the full repository format, Clippy, and workspace test gate.
- **Max feedback latency:** under 60 seconds after compilation.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | WS-01, WS-03 | T-01-01 | Auth precedes upgrade and live turns reuse the bounded pool path | integration | `cargo test --test inbound_codex_websocket` | ✅ | ✅ green |
| 01-01-02 | 01 | 1 | WS-02, WS-03 | — | Warmup completes locally with zero dispatch; acknowledgements are harmless | unit + integration | `cargo test codex_endpoint::frame --lib && cargo test --test inbound_codex_websocket` | ✅ | ✅ green |
| 01-01-03 | 01 | 1 | WS-03 | T-01-03 | SSE framing is bounded across arbitrary chunk boundaries | unit | `cargo test codex_endpoint::frame --lib` | ✅ | ✅ green |
| 01-02-01 | 02 | 2 | WS-06 | — | Replacement and disconnect abort work and suppress stale output | integration | `cargo test --test inbound_codex_websocket` | ✅ | ✅ green |
| 01-02-02 | 02 | 2 | WS-04, WS-05, WS-07 | T-01-02, T-01-03 | Relay stops at terminals, projects safe errors, and applies backpressure | unit + integration | `cargo test codex_endpoint::frame --lib && cargo test --test inbound_codex_websocket` | ✅ | ✅ green |
| 01-03-01 | 03 | 3 | CONF-01 | — | OpenCodex framing and delimiter transcripts remain deterministic | unit | `cargo test codex_endpoint::frame --lib` | ✅ | ✅ green |
| 01-03-02 | 03 | 3 | CONF-01 | T-01-02, T-01-03 | Malformed input, premature EOF, header filtering, and bounds fail safely | unit + integration | `cargo test codex_endpoint::frame --lib && cargo test --test inbound_codex_websocket` | ✅ | ✅ green |
| 01-04-01 | 04 | 4 | CONF-02 | — | Real sockets cover upgrade, live relay, replacement, and disconnect | integration | `cargo test --test inbound_codex_websocket -- --test-threads=1` | ✅ | ✅ green |
| 01-04-02 | 04 | 4 | CONF-02 | — | Maintained docs stay synchronized and generated wiki stays untouched | source guard | `git diff --check HEAD && test -z "$(git status --short -- wiki/)"` | ✅ | ✅ green |
| 01-04-03 | 04 | 4 | CONF-02 | — | Full workspace remains warning-free and regression-clean | repository gate | `cargo fmt --all --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features --workspace` | ✅ | ✅ green |

## Wave 0 Requirements

Existing unit and real-network integration infrastructure covers every phase
requirement; no test stubs or dependencies are missing.

## Manual-Only Verifications

All phase behaviors have automated verification.

## Validation Sign-Off

- [x] All tasks have an automated verification command.
- [x] Sampling continuity has no three-task gap.
- [x] Existing tests cover every requirement; no Wave 0 additions are needed.
- [x] No watch-mode flags are used.
- [x] Feedback latency is below 60 seconds after compilation.
- [x] `nyquist_compliant: true` is set in frontmatter.

**Approval:** approved 2026-09-06

## Validation Audit 2026-09-06

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |
