---
phase: 12-cursor-evidence-backed-hardening
verified: 2026-09-09T03:52:11Z
status: passed
score: 4/4 roadmap truths verified
behavior_unverified: 0
overrides_applied: 0
covered_files: [".planning/REQUIREMENTS.md",".planning/phases/12-cursor-evidence-backed-hardening/12-01-PLAN.md",".planning/phases/12-cursor-evidence-backed-hardening/12-01-SUMMARY.md",".planning/phases/12-cursor-evidence-backed-hardening/12-02-PLAN.md",".planning/phases/12-cursor-evidence-backed-hardening/12-02-SUMMARY.md",".planning/phases/12-cursor-evidence-backed-hardening/12-03-PLAN.md",".planning/phases/12-cursor-evidence-backed-hardening/12-03-SUMMARY.md",".planning/phases/12-cursor-evidence-backed-hardening/12-04-PLAN.md",".planning/phases/12-cursor-evidence-backed-hardening/12-04-SUMMARY.md",".planning/phases/12-cursor-evidence-backed-hardening/12-05-PLAN.md",".planning/phases/12-cursor-evidence-backed-hardening/12-05-SUMMARY.md",".planning/phases/12-cursor-evidence-backed-hardening/12-06-PLAN.md",".planning/phases/12-cursor-evidence-backed-hardening/12-06-SUMMARY.md",".planning/phases/12-cursor-evidence-backed-hardening/12-07-PLAN.md",".planning/phases/12-cursor-evidence-backed-hardening/12-07-SUMMARY.md",".planning/phases/12-cursor-evidence-backed-hardening/12-08-PLAN.md",".planning/phases/12-cursor-evidence-backed-hardening/12-08-SUMMARY.md","Cargo.lock","Cargo.toml","README.ja.md","README.ko.md","README.md","README.zh-CN.md","docs/cursor-request-history.md","site/src/content/docs/ja/providers/cursor.mdx","site/src/content/docs/ja/reference/configuration.md","site/src/content/docs/ko/providers/cursor.mdx","site/src/content/docs/ko/reference/configuration.md","site/src/content/docs/providers/cursor.mdx","site/src/content/docs/reference/configuration.md","site/src/content/docs/zh-cn/providers/cursor.mdx","site/src/content/docs/zh-cn/reference/configuration.md","src/adapters/cursor/admission.rs","src/adapters/cursor/agent.rs","src/adapters/cursor/aggregate.rs","src/adapters/cursor/cancellation_tests.rs","src/adapters/cursor/client.rs","src/adapters/cursor/connect.rs","src/adapters/cursor/history.rs","src/adapters/cursor/history_lifetime_tests.rs","src/adapters/cursor/history_tests.rs","src/adapters/cursor/kv.rs","src/adapters/cursor/kv_tests.rs","src/adapters/cursor/mod.rs","src/adapters/cursor/protocol_tests.rs","src/adapters/cursor/request.rs","src/adapters/cursor/request_isolation_tests.rs","src/adapters/cursor/response.rs","src/adapters/cursor/router_parity_tests.rs","src/adapters/cursor/sse.rs","src/adapters/cursor/strict.rs","src/adapters/cursor/test_frames.rs","src/adapters/cursor/usage.rs","src/adapters/cursor/wire.rs","src/config.rs","src/retry.rs","src/server.rs"]
covered_digest: "v1:sha256:48c8f77917d586ff9b31cecf85921b0bcdb1bfa35435a315256b9ae54b84821e"
deferred:
  - truth: Live Shunt Cursor provider availability
    addressed_in: Phase 16
    evidence: "Release criterion 3 requires opt-in isolated live smoke tests; CLI-only success is not Shunt proof."
---

# Phase 12 Verification

## Phase 15 re-verification (2026-09-09)

Independent source audit found Phase 15 bookkeeping/docs, the test-only router dependency seam, and an OpenCode Go config identity guard. Cursor source, transport, history, retry, and cancellation paths remain unchanged; the new provider kind is mapped to the existing OpenAI Chat adapter but is rejected by the explicit empty-evidence admission gate before credentials/network. Root executed `node /tmp/shunt-phase12-isolated-run.cjs env RUSTFLAGS=-Dwarnings cargo test --quiet --all-features --workspace` at `eb60174`: 2,972 passed, 0 failed, 2 existing ignored. No Phase 12 truth, artifact, key link, or negative boundary regressed. Production OpenCodex config mtime/SHA and backup/invalid inventory were unchanged.

## Phase 14 regression re-verification (2026-09-08)

Root inline verification; no independent verifier pass is claimed.
Compared covered files to c1a2646. Cursor source, fixtures, dependency files,
retry and server are unchanged. Config additions are separate Chat/Command Code
kind/auth guards and do not alter Cursor arms. CUR requirements and all phase
plans are unchanged; README/config additions maintain four-locale coverage.
Both cursor_cancellation_release tests passed: backpressured sender ownership
and full-router headers/stream/aggregation cleanup. Decision coverage is 11/11.

Every test used node /tmp/shunt-phase12-isolated-run.cjs with all-features and
nonzero selection. Production config mtime/SHA and backup inventory remained
unchanged. The preceding observed full workspace gate passed 2,942 tests with
0 failures and 2 existing ignored tests; site build passed 169 pages in four
languages. No disabled requirement tests were found in the rechecked groups.
Fingerprint was regenerated through the bundled GSD tool after code review and
successful execution. Historical results below are historical. Live/Computer
acceptance remains outside this hermetic re-verification.


Goal: request-local, evidence-backed Cursor behavior with stable continuation
and strict Connect stream safety. All four roadmap truths and CUR-01–08 have
implementation and behavioral evidence. No live-provider availability claim.

## Goal and requirement evidence

| Truth | Requirements | Implementation and exercised evidence |
| --- | --- | --- |
| Pinned destination and request-local facts | CUR-01, CUR-02 | agent.rs keeps the exact HTTPS AgentService Run origin; destination regression and concurrent mixed-fact tests pass. No production endpoint override was added. |
| Authentic paired continuation | CUR-03, CUR-04 | history.rs builds schema-derived root/turn blobs from explicit session metadata and paired original tool IDs; kv.rs serves bounded per-request blobs. Four history variants and full-router bidirectional hydration pass. Missing IDs, opaque history, unsupported tool/schema/image forms fail before credentials or dispatch. |
| Ordered outputs, bounded parsing, terminal and cancellation | CUR-05, CUR-06 | strict.rs/wire.rs/connect.rs validate active Run frames; usage.rs separates additive output and absolute context. aggregate.rs and active SSE retain ordered reasoning/text/tool meaning. Full-router EOF, duplicate terminal, cancellation, usage and output parity tests pass. Corruption cannot produce a successful terminal. |
| Safe failure classification, no semantic heuristic | CUR-07, CUR-08 | ConnectOnly permits only proven connect-phase classification. Accepted statuses and ambiguous post-send failures cannot advance fallback; real two-target tests observe zero fallback calls. Repeated text does not end a turn. No new same-provider retry loop or repetition detector. |

## Artifact and wiring checks

All eight plans have summaries and substantive implementation. Router dispatch
calls admission and structured-history preparation before auth lookup; prepared
blobs travel with the request into Run and are owned by ReadState. Strict frame
validation precedes extracting KV, usage, tool and text events. Those events feed
the JSON accumulator or active SSE framer. TurnGuard owns the paced sender before
headers; dropping the request or response cancels it, closes the H2 request body,
and releases capacity. Tests exercise these transitions rather than only asserting
symbol presence. Full-router tests use synthetic credentials and DNS-pinned TLS
loopback without weakening the production origin guard.

New known-protocol helpers are separate focused modules. Legacy Cursor transport
and coercing tests remain explicitly off the active Run path. No stub or disabled
requirement test is counted as proof.

## Commands observed

All commands ran in the isolated worktree with the wrapper
`node /tmp/shunt-phase12-isolated-run.cjs`, fresh OPENCODEX_HOME and non-10100 ports.

- `cargo fmt --all --check`: passed; final test-only addition also formatted.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed on final source.
- `cargo test --all-features --workspace --quiet`: passed, 2,193 active library tests plus all integration suites; two pre-existing ignored benchmarks. The final added HTTP regression passed separately; production source did not change after the full run.
- `cargo test --lib --all-features cursor_error_body -- --test-threads=1`: two tests passed (cap/deadline and actual oversized HTTP 429 with preserved status/retry-after/no replay).
- Each of the 18 filters below ran separately with `cargo test --lib --all-features FILTER -- --test-threads=1`: all passed and selected nonzero tests.
- `npm --prefix site run build`: 161 pages, four locales, passed; existing Vite/Pagefind warnings only.
- `bash .claude/skills/run-shunt/smoke.sh`: rebuilt binary and five checks passed on ports 31711/31712. Owned mock/server processes and scratch directory cleaned up.
- GSD decision coverage: 11/11 trackable decisions honored, none missing.

Filters: cursor_terminal_tracer, cursor_w0_usage_facts, cursor_w0_corpora,
cursor_destination, cursor_request_isolation, cursor_history_identity,
cursor_continuation_guard, cursor_admission, cursor_admission_legacy_offpath,
cursor_terminal_dedupe_idle, cursor_usage_relay, cursor_framing_rejection,
cursor_proto_wire_strict, cursor_retry_safety, cursor_failure_classification,
cursor_cancellation_release, cursor_no_heuristics, cursor_output_parity.

## Prohibition dispositions

Plan frontmatter retains original flagged/unverified planning markers; their
execution disposition is recorded here, not inferred from the flag.

- Plans 01/02: origin bytes unchanged (visibility only changed for tests); no endpoint fallback. Active usage tags are schema-derived, not inferred from CLI caches. Exact supported history model is Composer 2.5, not a family inference.
- Plan 03: original call/result IDs survive; missing wire IDs fail. Only explicitly supplied session metadata produces identity. Blob storage is bounded memory-only and request-local, under the user's recorded approval; no durable continuation store.
- Plan 04: retired bridge remains off-path. Missing/non-object schemas and unsupported or undecodable images fail explicitly; admission and recovery tests observe no dispatch.
- Plan 05: repetition/idle cannot synthesize successful completion. Derived input and missing cache figures are labeled estimated/unavailable, not billing measurements.
- Plan 06: 64 MiB frame/decompression limits unchanged, malformed trailers/wire fail. The separate HTTP diagnostic cap added during review is not a change to Connect decompression limits.
- Plan 07: no Commitment wrapper/new redispatch loop. Local/accepted/post-send failures do not trigger fallback; full-router negative call-count assertions pass.
- Plan 08: wiki unchanged. Request-owned guard cancellation, no global cancellation registry or detached recovery task; cancellation tests observe upstream close and reclaimed capacity.

## Review and safety

Independent standard review found an unbounded HTTP error read. Fixed by
incremental 64 KiB collection with a five-second total deadline, then tested.
Do not buffer the whole body and truncate afterward. Cosmetic grpc-message
percent decoding remains an informational deferral, not a correctness gate.
All affected engineering/provider-site locales updated; README capability and
setup remain accurate. The report's accidental main-checkout write was relocated
and its misplaced copy removed; no source was changed there.

Production OpenCodex config mtime/SHA-256 and invalid/backup inventory remained
unchanged before and after every verification command. No real credential file
was modified, no production proxy was restarted, and no secrets were committed.

## Deferred release evidence

Live Shunt Cursor availability belongs to Phase 16's explicit isolated live gate.
The earlier authenticated Cursor CLI probe proves only that CLI invocation,
not the gateway's live Connect path. No raw capture is invented. Phase 12's
schema/hermetic acceptance criteria are fulfilled; no human-only UI requirement.
