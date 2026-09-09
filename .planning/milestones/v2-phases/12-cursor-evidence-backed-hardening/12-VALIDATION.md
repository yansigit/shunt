---
phase: "12"
slug: "cursor-evidence-backed-hardening"
status: passed
nyquist_compliant: true
wave_0_complete: true
created: "2026-09-07"
---

# Phase 12 — Validation Strategy

## Test Infrastructure

Rust cargo tests, existing Axum/loopback helpers and Cursor module tests. Every automated test invocation uses one simple exact filter: cargo test --all-features <exact_filter> -- --test-threads=1. Grouped checks are separate &&-chained invocations — no quoted OR filters, no broad 'cursor' filter swallowing focused checks. Every invocation must select nonzero tests; zero selected tests is a failure. Full suite: cargo test --all-features --workspace -- --test-threads=1. Formatting and Clippy remain mandatory. agent.rs measured 1708 lines: plans forbid implying a near-500-line cleanup; space savings only via narrow private helper modules with explicit ownership if shape demands. Expected feedback budget: 600 seconds excluding initial compilation; command timeout is an error, not a pass.

## Sampling Rate

Run the relevant non-empty focused filter after every task; run each plan's sequential verification chain after each plan wave. Before verification run formatting, Clippy with warnings denied, full workspace tests, and the strict docs gate cargo check --workspace --all-features && npm --prefix site run build for changed docs — no semicolon chaining, no suppression. Never treat zero selected tests or skipped live tests as evidence.

## Per-Task Verification Map

| Plan (wave, deps) | Task | Requirement | File(s) | Exact filter (single invocation) | Explicit ownership |
|---|---|---|---|---|---|
| 12-01 (1, none) | EOF-without-terminal error tracer (whole gateway router) | CUR-05/06 | src/adapters/cursor/agent.rs + router_parity_tests.rs + server.rs + Cargo.toml | cursor_terminal_tracer | whole-gateway-router tracer owner (hermetic TLS loopback + pinned DNS + synthetic SHUNT_CURSOR_AUTH_FILE) |
| 12-01 (1, none) | W0-1 usage/terminal fixtures | CUR-05/E1 | src/adapters/cursor/test_frames.rs | cursor_w0_usage_facts | none |
| 12-01 (1, none) | W0-2 corpora (gzip contrast) | CUR-06 | src/adapters/cursor/connect.rs | cursor_w0_corpora | none |
| 12-02 (1, none) | Destination regression | CUR-01 | src/adapters/cursor/mod.rs | cursor_destination | destination-byte owner (adapter dispatch level, not whole-router) |
| 12-02 (1, none) | Request-local isolation | CUR-02 | src/adapters/cursor/mod.rs | cursor_request_isolation | none |
| 12-03 (2, 02) | Structured identity roundtrip | CUR-03 | src/adapters/cursor/request.rs | cursor_history_identity | none |
| 12-03 (2, 02) | Continuation guard integration | CUR-03 | src/adapters/cursor/request.rs + mod.rs | cursor_continuation_guard | no-dispatch integration owner |
| 12-04 (3, 03) | Admission errors | CUR-04 | src/adapters/cursor/mod.rs | cursor_admission | none |
| 12-04 (3, 03) | Legacy off-path honesty | CUR-04 | src/adapters/cursor/mod.rs | cursor_admission_legacy_offpath | retired-code honesty owner |
| 12-05 (4, 01, 04) | Duplicate/post-terminal batch rejection + idle seam | CUR-05/D-05 | src/adapters/cursor/agent.rs | cursor_terminal_dedupe_idle | none (EOF tracer shipped by 12-01) |
| 12-05 (4, 01, 04) | Usage relay | CUR-05/E1 | src/adapters/cursor/mod.rs + agent.rs | cursor_usage_relay | none |
| 12-06 (5, 01, 05) | Framing rejection | CUR-06 | src/adapters/cursor/connect.rs + agent.rs | cursor_framing_rejection | none |
| 12-06 (5, 01, 05) | Strict protobuf/nested args | CUR-06 | src/adapters/cursor/agent.rs | cursor_proto_wire_strict | nested-argument strictness owner |
| 12-07 (6, 05, 06) | RetrySafety resolution | CUR-07 | src/retry.rs | cursor_retry_safety | none |
| 12-07 (6, 05, 06) | Classification + no-failover proof | CUR-07 | src/adapters/cursor/mod.rs | cursor_failure_classification | no-failover consult owner |
| 12-08 (7, 05, 07) | Cancellation release + router cancel | CUR-05/D-07 | src/adapters/cursor/agent.rs + router_parity_tests.rs | cursor_cancellation_release | cancellation/router-cancel owner |
| 12-08 (7, 05, 07) | No-heuristic absence | CUR-08/D-09 | src/adapters/cursor/agent.rs + mod.rs | cursor_no_heuristics | none |
| 12-08 (7, 05, 07) | Docs parity EN/ko/ja/zh | D-11 | README*, docs/, site reference | chained: cargo check --workspace --all-features && npm --prefix site run build | docs parity owner |

Wave graph serialization (dependency edges, no same-wave file sharing): 12-01 (agent.rs/connect.rs/test_frames.rs/router_parity_tests.rs/server.rs/Cargo.toml dev-deps) and 12-02 (mod.rs) start in parallel with zero shared files; 12-03 (request.rs + mod.rs) depends on 12-02; 12-04 (mod.rs) depends on 12-03; 12-05 (agent.rs + mod.rs) depends on 12-01 + 12-04; 12-06 (connect.rs + agent.rs) depends on 12-01 + 12-05; 12-07 (retry.rs + mod.rs) depends on 12-05 + 12-06; 12-08 (wave 7: agent.rs + router_parity_tests.rs + docs) depends on 12-05 + 12-07. Shared-path edges hold transitively for every overlap: agent.rs ordering 12-01 → 12-05 → 12-06 → 12-08; connect.rs 12-01 → 12-06; mod.rs 12-02 → 12-03 → 12-04 → 12-05 → 12-07 → 12-08; router_parity_tests.rs 12-01 → 12-08; server.rs, test_frames.rs, retry.rs, and Cargo.toml each have a single owner (12-01 tracer, 12-07 retry). Every task has a matching automated check with an observable failure direction. Required groups covered: destination/request isolation (12-02), history/admission (12-03/04), incremental terminal/usage/cancellation (12-01 tracer, 12-05, 12-08 Task 1), bounded framing (12-06), pre-output failure classification (12-07), and no semantic heuristic (12-08 Task 2). Explicit owners assigned: whole-gateway-router tracer parity (12-01, hermetic TLS loopback via the server.rs test-only client seam), cancellation with router-side cancel probe (12-08 Task 1), no-failover consult (12-07 Task 2). No task may execute without a matching automated check or explicit evidence gate; nyquist_compliant stays false until actual execution.

## Wave 0 Requirements

- Tracer first: a tiny production end-to-end terminal fix (EOF without an authoritative terminal emits exactly one error, never a success terminal) proven over the whole gateway router ships from 12-01 before later W0 corpus consumption, closing the fixture-only tracer gap. Terminal detection is bounded to bytes received before EOF/cancellation; no post-cancellation future-byte detection is claimed.
- Add active-wire fixtures derived from the recorded OpenCodex schema, not retired protobuf types; W0-1 encoding assertions use a schema-derived local test decoder and never claim the unchanged production decoder consumes usage today (production consumption flips in 12-05); corrupt gzip is distinguished from valid gzip JSON and valid empty trailers.
- Prove each changed negative behavior red before the fix when feasible; disclose characterization tests that already pass.
- Add private loopback injection only; no new public endpoint overrides or credential-file writes.
- Distinguish context occupancy from output deltas and label derived/absent usage honestly.

## Manual-Only Verification

Live Cursor availability remains unexecuted. An isolated Cursor CLI (Composer 2.5) probe succeeded for CLI-driven generation in a temporary home, but this is not evidence of Shunt live-provider verification and no raw Connect capture exists yet. Hermetic/schema evidence is sufficient for implementing the supported fields and cannot claim live provider availability. Production opencodex config hash/mtime and invalid/backup inventory were unchanged around the 83-test reference run.

## Validation Sign-Off

- [x] Exact per-task map completed
- [x] Every task has automated verification with nonzero selected tests and observable failure direction
- [x] No three consecutive tasks without verification
- [x] All focused filters are single, exact invocations selecting tests
- [x] Overlapping files serialized by dependency edges only
- [x] Real-router parity, cancellation/router-cancel, no-failover, docs-parity owners assigned
- [x] Format, Clippy, workspace and relevant docs gates pass
- [x] Nyquist compliance marked true only after actual execution

**Approval:** execution verified. All 18 focused filter invocations passed with
nonzero test selection on the final source, including ordered JSON/SSE output.
The full workspace suite passed (2,193 active library tests, two existing ignored
benchmarks, all integration suites). The final additional HTTP error regression
and cap/deadline test passed separately. Formatting, all-target all-feature clippy
with warnings denied, 161-page four-locale site build and five rebuilt-binary smoke
checks passed. Every command inherited a fresh isolated OPENCODEX_HOME;
production config mtime/SHA and backup/invalid-file inventory remained unchanged.
Live Shunt Cursor availability is not asserted; it remains the Phase 16 opt-in gate.
