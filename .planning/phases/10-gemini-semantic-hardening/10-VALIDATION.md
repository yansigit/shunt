---
phase: "10"
slug: "gemini-semantic-hardening"
status: complete
nyquist_compliant: true
wave_0_complete: true
created: "2026-09-06"
updated: "2026-09-07"
---

# Phase 10 — Validation Strategy

> Executable validation contract for the thirteen completed baseline tasks and five Phase 10 gap-closure tasks.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test harness + Tokio/Axum real-gateway integration tests |
| **Mock boundary** | Wiremock or raw loopback server for Google Code Assist only |
| **Pure semantic suite** | `tests/gemini_translate.rs` and `src/model/gemini_request/tests.rs` |
| **Real gateway suite** | `tests/gemini_conformance.rs` plus the crate-private injected-resolver `/v1/messages` cases in `src/server.rs` |
| **Full suite** | `cargo test --all-features --workspace -- --test-threads=1` |

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirements | Threats | Automated command | Status |
|---------|------|------|--------------|---------|-------------------|--------|
| 10-01-01 | 01 | 1 | GEM-02, GEM-03, GEM-04 | T-10-01, T-10-03 | `cargo test --all-features --test gemini_translate semantic_parity` | passed |
| 10-01-02 | 01 | 1 | GEM-02, GEM-03, GEM-04 | T-10-01, T-10-03, T-10-04 | `cargo test --all-features --test gemini_translate gemini_semantic_strictness` | passed |
| 10-01-03 | 01 | 1 | GEM-02, GEM-04 | T-10-01, T-10-02 | `cargo test --all-features gemini_tool_signature_roundtrip` | passed |
| 10-02-01 | 02 | 2 | GEM-02, GEM-04 | T-10-05, T-10-06 | `cargo test --all-features gemini_sse_bounds` | passed |
| 10-02-02 | 02 | 2 | GEM-02, GEM-03, GEM-04 | T-10-05, T-10-06, T-10-07 | `cargo test --all-features --test gemini_conformance gemini_streaming_framing` | passed |
| 10-02-03 | 02 | 2 | GEM-03, GEM-04 | T-10-05, T-10-07 | `cargo test --all-features --test gemini_conformance gemini_unary_bounds` | passed |
| 10-03-01 | 03 | 3 | GEM-01, GEM-05 | T-10-08, T-10-09 | `cargo test --all-features --test gemini_conformance gemini_identity_retry` | passed-partial |
| 10-03-02 | 03 | 3 | GEM-02, GEM-03, GEM-05 | T-10-08, T-10-11 | `cargo test --all-features --test gemini_conformance gemini_no_post_header_replay` | passed |
| 10-03-03 | 03 | 3 | GEM-01, GEM-05 | T-10-09, T-10-10 | `cargo test --all-features --test gemini_conformance gemini_response_drop_releases` | passed-partial |
| 10-04-01 | 04 | 4 | GEM-01..05 | T-10-12, T-10-13 | `test -s docs/v2-gemini-semantic-hardening.md` plus invariant/exclusion `rg` checks | passed-partial |
| 10-04-02 | 04 | 4 | GEM-01..05 | T-10-12 | English configuration/troubleshooting Gemini terminal/retry `rg` checks | passed |
| 10-05-01 | 05 | 5 | GEM-01..05 | T-10-14 | six-file maintained-locale non-empty/Gemini contract loop | passed |
| 10-05-02 | 05 | 5 | GEM-01..05 | T-10-15, T-10-16 | `bash scripts/check_phase10_scope.sh` then format, Clippy, and workspace gates | passed |
| 10-06-01 | 06 | 6 | GEM-02, GEM-04 | T-10-G06-01, T-10-G06-02 | `cargo test --all-features gemini_parallel_tool_result_identity` | passed |
| 10-06-02 | 06 | 6 | GEM-02, GEM-03, GEM-04 | T-10-G06-03, T-10-G06-04 | `cargo test --all-features --test gemini_translate gemini_known_part_strictness` | passed |
| 10-06-03 | 06 | 6 | GEM-04 | T-10-G06-05 | `cargo test --all-features gemini_post_done_frames` | passed |
| 10-07-01 | 07 | 7 | GEM-01, GEM-05 | T-10-G07-02, T-10-G07-03, T-10-G07-04 | `cargo test --all-features gemini_google_oauth_code_assist_lifetime` | passed |
| 10-07-02 | 07 | 7 | GEM-01..05 | T-10-G07-05, T-10-G07-06 | focused gap suites plus scope, format, Clippy, and serial workspace gates | passed |

## Sampling Rate

- Run each exact focused command after its task.
- After Wave 1, run `cargo test --all-features --test gemini_translate` and request-translation tests.
- After Wave 2, run both Gemini suites together.
- After Wave 3, add retry/failover and the complete cancellation matrix.
- After Wave 4, verify English engineering/configuration/troubleshooting sources.
- After Wave 5, the baseline format, warnings-denied Clippy, full workspace suite, and deterministic exclusion diff checks passed.
- After Wave 6, run all three protocol-semantic gap filters and both complete Gemini suites.
- After Wave 7, add the OAuth lifetime filter, scope audit, retry/failover regressions, format, warnings-denied Clippy, and the serial all-feature workspace suite.

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
5. `bash scripts/check_phase10_scope.sh`
6. `cargo fmt --all --check`
7. `cargo clippy --all-targets --all-features -- -D warnings`
8. `cargo test --all-features --workspace -- --test-threads=1`

## Manual-Only Verifications

None. Live Gemini credentials are intentionally outside this phase's evidence needs.

## Validation Sign-Off

- [x] All thirteen baseline tasks and five gap-closure tasks have an automated pass/fail command.
- [x] GEM-01 through GEM-05 each map to complete real behavioral evidence, including hermetic OAuth Code Assist lifetime coverage.
- [x] Strict terminal, parser bounds, stream/unary parity, tool/signature authenticity, retry safety, and cancellation evidence is complete.
- [x] Final repository gates are in the last wave.
- [x] Google AI Studio Web and credential writeback remain exclusions only.
