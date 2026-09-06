---
phase: "09"
slug: "provider-conformance-foundation"
status: ready
nyquist_compliant: true
wave_0_complete: true
created: "2026-09-06"
updated: "2026-09-06"
---

# Phase 09 — Validation Strategy

> Executable validation contract for all eleven planned tasks.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness + Tokio/Axum real-gateway integration tests |
| **Mock boundary** | Wiremock or raw loopback server for external provider endpoints only |
| **Config file** | `Cargo.toml` |
| **Focused baseline** | `cargo test --all-features --test failover --test retry --test codex_websocket_fallback` |
| **Full suite** | `cargo test --all-features --workspace` |
| **Final quality gates** | format, warnings-denied Clippy, full all-features workspace suite |

## Sampling Rate

- After every task: run its exact focused command below.
- After Wave 1: run the commitment, failover, retry, and WebSocket-fallback suites.
- After Wave 2: run the Responses/Codex-ingress suites and the passthrough/failover/account suites.
- After Wave 3: run native Codex HTTP/SSE, inbound and outbound WebSocket v2, compression, continuation, cancellation, and terminal suites, then all repository quality gates.
- Focused checks should complete within 60 seconds; the full workspace gate may take up to 120 seconds.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirements | Threats | Secure behavior | Test type | Automated command | Target exists | Status |
|---------|------|------|--------------|---------|-----------------|-----------|-------------------|---------------|--------|
| 09-01-01 | 01 | 1 | PRES-03, SAFE-04 | T-09-01, T-09-02 | Monotonic commitment permits pre-commit fallback and blocks post-text/post-tool fallback | unit + real gateway | `cargo test --all-features replay_commitment` | ✅ | ⬜ pending |
| 09-01-02 | 01 | 1 | PRES-03, SAFE-04 | T-09-01, T-09-03 | Same-provider retry and route failover consume one predicate without broadening eligibility | unit + integration | `cargo test --all-features redispatch_gate` | ✅ | ⬜ pending |
| 09-02-01 | 02 | 2 | PRES-01, PRES-04, SAFE-03 | T-09-05, T-09-06 | Native HTTP/SSE and inbound WebSocket reject incomplete/conflicting terminals with ingress-correct errors | unit + real gateway | `cargo test --all-features responses_terminal` | ✅ | ⬜ pending |
| 09-02-02 | 02 | 2 | SAFE-01, SAFE-02, SAFE-03 | T-09-04, T-09-05 | Byte-strict parser and non-streaming accumulator enforce below/exact/plus-one bounds | unit + integration | `cargo test --all-features responses_bounds` | ✅ | ⬜ pending |
| 09-02-03 | 02 | 2 | PRES-01, SAFE-03 | T-09-05 | HTTP and defensive WebSocket transports share one trustworthy-terminal rule | adapter | `cargo test --all-features responses_transport_terminal` | ✅ | ⬜ pending |
| 09-03-01 | 03 | 3 | PRES-02, SAFE-04, SAFE-06 | T-09-08, T-09-09 | One-shot recovery preserves paired tools and opaque metadata and consumes the shared commitment predicate | real gateway | `cargo test --all-features continuation_recovery` | ✅ | ⬜ pending |
| 09-03-02 | 03 | 3 | SAFE-01, SAFE-06 | T-09-10 | Continuation items, transcript bytes, and metadata are bounded and overflow disables reuse atomically | unit | `cargo test --all-features continuation_bounds` | ✅ | ⬜ pending |
| 09-03-03 | 03 | 3 | PRES-02, SAFE-06 | T-09-08, T-09-11 | Missing/invented tool or reasoning identity fails closed while valid opaque values remain exact; final repository gates pass | unit + integration + release gates | `cargo test --all-features authentic_tool_identity`, then format, Clippy, and full workspace commands below | ✅ | ⬜ pending |
| 09-04-01 | 04 | 2 | PRES-05, SAFE-05 | T-09-12, T-09-14 | Generic Anthropic Vercel route preserves request/SSE/error behavior and strips inbound credential slots | real gateway | `cargo test --all-features --test passthrough vercel_anthropic` | ✅ | ⬜ pending |
| 09-04-02 | 04 | 2 | SAFE-05 | T-09-13, T-09-14 | Credential diagnostics redact secrets and cross-origin failover rebinds credentials | unit + integration | `cargo test --all-features credential_redaction` | ✅ | ⬜ pending |
| 09-04-03 | 04 | 2 | SAFE-07 | T-09-15 | Client drop observably cancels upstream work and releases global/account capacity within a timeout | unit + real gateway | `cargo test --all-features response_drop_releases` | ✅ | ⬜ pending |

## Executable Preservation Sweep

PRES-01 is accepted only when this complete native Codex command passes:

`cargo test --all-features --test inbound_codex_endpoint --test inbound_codex_websocket --test codex_websocket_fallback --test codex_multi_account`

Together with `cargo test --all-features responses_transport_terminal` and the full workspace suite, this executes native HTTP/SSE, inbound WebSocket, outbound WebSocket v2, compression, continuation, cancellation, and terminal behavior.

PRES-02 additionally requires:

`cargo test --all-features --test codex_multi_account --test inbound_codex_endpoint --test inbound_anthropic_translation`

## Wave 0 Status

- [x] Every task has a focused automated command.
- [x] All referenced test targets exist before execution.
- [x] Failing-first cases are created inside the owning task before any production repair.
- [x] No test framework, runtime dependency, public configuration, live credential, or provider-specific later-phase fixture is required.

## Final Wave Release Gates

Run in Plan 09-03 after all earlier dependencies and Task 09-03-03 complete:

1. `cargo fmt --all --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --all-features --workspace`

## Manual-Only Verifications

None. Live provider credentials are intentionally unnecessary for Phase 9.

## Validation Sign-Off

- [x] All 11 tasks map to an automated check.
- [x] All 12 assigned requirements map to behavioral evidence.
- [x] PRES-01 names and executes every required native Codex transport/property suite, including `tests/inbound_codex_websocket.rs`.
- [x] SAFE-04 covers same-provider retry, route failover, WebSocket fallback, and continuation recovery through one commitment predicate.
- [x] Sampling continuity has no three-task gap.
- [x] Final format, Clippy, and full-suite gates are executable in the final wave.
- [x] `nyquist_compliant: true` and `wave_0_complete: true` reflect the complete planned verification graph.

**Approval:** ready for execution; task results remain pending.
