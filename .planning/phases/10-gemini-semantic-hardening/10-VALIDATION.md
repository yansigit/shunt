---
phase: "10"
slug: "gemini-semantic-hardening"
status: ready
nyquist_compliant: true
wave_0_complete: true
created: "2026-09-06"
updated: "2026-09-06"
---

# Phase 10 — Validation Strategy

> Executable validation contract for all thirteen planned tasks.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test harness + Tokio/Axum real-gateway integration tests |
| **Mock boundary** | Wiremock or raw loopback server for Google Code Assist only |
| **Pure semantic suite** | `tests/gemini_translate.rs` and `src/model/gemini_request/tests.rs` |
| **Real gateway suite** | `tests/gemini_conformance.rs` created in Wave 2 |
| **Full suite** | `cargo test --all-features --workspace` |

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirements | Threats | Automated command | Status |
|---------|------|------|--------------|---------|-------------------|--------|
| 10-01-01 | 01 | 1 | GEM-02, GEM-03, GEM-04 | T-10-01, T-10-03 | `cargo test --all-features --test gemini_translate semantic_parity` | pending |
| 10-01-02 | 01 | 1 | GEM-02, GEM-03, GEM-04 | T-10-01, T-10-03, T-10-04 | `cargo test --all-features --test gemini_translate gemini_semantic_strictness` | pending |
| 10-01-03 | 01 | 1 | GEM-02, GEM-04 | T-10-01, T-10-02 | `cargo test --all-features gemini_tool_signature_roundtrip` | pending |
| 10-02-01 | 02 | 2 | GEM-02, GEM-04 | T-10-05, T-10-06 | `cargo test --all-features gemini_sse_bounds` | pending |
| 10-02-02 | 02 | 2 | GEM-02, GEM-03, GEM-04 | T-10-05, T-10-06, T-10-07 | `cargo test --all-features --test gemini_conformance gemini_streaming_framing` | pending |
| 10-02-03 | 02 | 2 | GEM-03, GEM-04 | T-10-05, T-10-07 | `cargo test --all-features --test gemini_conformance gemini_unary_bounds` | pending |
| 10-03-01 | 03 | 3 | GEM-01, GEM-05 | T-10-08, T-10-09 | `cargo test --all-features --test gemini_conformance gemini_identity_retry` | pending |
| 10-03-02 | 03 | 3 | GEM-02, GEM-03, GEM-05 | T-10-08, T-10-11 | `cargo test --all-features --test gemini_conformance gemini_no_post_header_replay` | pending |
| 10-03-03 | 03 | 3 | GEM-01, GEM-05 | T-10-09, T-10-10 | `cargo test --all-features --test gemini_conformance gemini_response_drop_releases` | pending |
| 10-04-01 | 04 | 4 | GEM-01..05 | T-10-12, T-10-13 | `test -s docs/v2-gemini-semantic-hardening.md` plus invariant/exclusion `rg` checks | pending |
| 10-04-02 | 04 | 4 | GEM-01..05 | T-10-12 | English configuration/troubleshooting Gemini terminal/retry `rg` checks | pending |
| 10-05-01 | 05 | 5 | GEM-01..05 | T-10-14 | six-file maintained-locale non-empty/Gemini contract loop | pending |
| 10-05-02 | 05 | 5 | GEM-01..05 | T-10-15, T-10-16 | `bash scripts/check_phase10_scope.sh` then format, Clippy, and workspace gates | pending |

## Sampling Rate

- Run each exact focused command after its task.
- After Wave 1, run `cargo test --all-features --test gemini_translate` and request-translation tests.
- After Wave 2, run both Gemini suites together.
- After Wave 3, add retry/failover and the complete cancellation matrix.
- After Wave 4, verify English engineering/configuration/troubleshooting sources.
- After Wave 5, run format, warnings-denied Clippy, the full workspace suite, and deterministic exclusion diff checks.

## Wave 0 Status

- [x] Existing pure-test targets are present.
- [x] `tests/gemini_conformance.rs` is created inside the first task that consumes it.
- [x] Every production behavior task starts with failing or explicit characterization fixtures.
- [x] All fixtures are hermetic and synthetic; no live credentials or credential files are needed.
- [x] No framework, runtime dependency, provider key, or public configuration is required.

## Final Release Gates

1. `cargo test --all-features --test gemini_translate`
2. `cargo test --all-features --test gemini_conformance`
3. `cargo test --all-features gemini`
4. `cargo test --all-features --test retry --test failover`
5. `cargo fmt --all --check`
6. `cargo clippy --all-targets --all-features -- -D warnings`
7. `cargo test --all-features --workspace`

## Manual-Only Verifications

None. Live Gemini credentials are intentionally outside this phase's evidence needs.

## Validation Sign-Off

- [x] All thirteen tasks have an automated pass/fail command.
- [x] GEM-01 through GEM-05 each map to real behavioral evidence.
- [x] Strict terminal, parser bounds, stream/unary parity, tool/signature authenticity, retry safety, and cancellation are covered.
- [x] Final repository gates are in the last wave.
- [x] Google AI Studio Web and credential writeback remain exclusions only.
