---
phase: "16"
slug: "cross-provider-release-gate"
status: in_review
nyquist_compliant: false
wave_0_complete: true
created: "2026-09-08"
---

# Phase 16 — Validation Strategy

## Test Infrastructure

Rust Cargo/Tokio integration tests, existing real-router fixtures, CLI subprocess tests, and Astro site build. Configuration: Cargo.toml and site/package.json. Every stateful command below runs from the dedicated worktree through the production-fingerprint isolation wrapper.

- Quick: `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test opencode_go_docs`
- Full: `node /tmp/shunt-phase12-isolated-run.cjs env RUSTFLAGS=-Dwarnings cargo test --all-features --workspace`
- Formatting: `node /tmp/shunt-phase12-isolated-run.cjs cargo fmt --all --check`
- Lints: `node /tmp/shunt-phase12-isolated-run.cjs env RUSTFLAGS=-Dwarnings cargo clippy --all-targets --all-features -- -D warnings`
- Site: `node /tmp/shunt-phase12-isolated-run.cjs npm --prefix site run build`

Actual final evidence: 2,979 tests passed, zero failed, two existing ignored.
Final site build took 4.17s; fixture-path focused compilation took 5.80s.
Exact Rust outputs and per-suite durations are retained in 16-FINAL-GATES.json.

## Sampling Rate

Run the relevant named focused suite after each task; reject zero-test filtered runs. Run the full suite after each implementation wave and before final verification. Serialize Cargo. A nonzero exit, missing expected assertions, or changed production fingerprints blocks completion.

## Per-Task Verification Map

| task | requirements | automated verification | manual/evidence requirement |
|---|---|---|---|
| 16-01.1 | REL-01/02 | `release_matrix release_matrix` (must select >0) | RED before ledger |
| 16-01.2 | REL-01/02 | same focused validator | mutation failure then GREEN |
| 16-02.1 | REL-02/03/06 | `release_security release_security` (must select >0) | MIT/provenance/scope record |
| 16-02.2 | REL-02/03 | `release_security credential_boundary` (must select >0) | no-writeback evidence |
| 16-03.1 | REL-05 | `opencode_go_docs opencode_go_docs` (must select >0) | RED/mutation/GREEN and locale parity |
| 16-03.2 | REL-05 | `npm --prefix site run build` | built anchors from `site/dist` |
| 16-04.1 | REL-04 | `check_cli opencode_go_cli_negative` (must select >0) | root-owned preflight/live skip/pass records |
| 16-05.1 | REL-01..06 | `env RUSTFLAGS=-Dwarnings cargo test --all-features --workspace` | serialized focused suites, fingerprints, code/security review |
| 16-05.2 | REL-05/06 | `opencode_go_docs opencode_go_docs` (must select >0) | root/Astrahigh visual screenshots or honest blocked record |

Every command is a separate `node /tmp/shunt-phase12-isolated-run.cjs`
invocation from the dedicated worktree; Cargo is serialized and zero-test
filters fail. Existing 2,972-pass output is baseline only.

## Wave 0 Requirements

Existing test infrastructure was reused. Plans 01 and 02 authored their new
release tests and completed GREEN and mutation checks; plan 03 completed its
docs RED/mutation/GREEN sequence. No framework or runtime dependency was added.
All Cargo executions were serialized through the exclusive wrapper. One busy
lock returned 73 without starting a child; root retried after its owner exited.

Execution deviation: the initial per-wave full-regression sampling proposal
was not followed. Focused feedback ran per task, and full workspace gates ran
at final phase verification (twice, including after fixture relocation).
No absent historical wave-level run is counted as a pass.

## Manual-Only Verifications

Computer visual evaluation uses an owned isolated documentation server, performed by root or gpt-6-astra/high. Save real observations or an explicit blocked result. Built HTML checks are not visual evidence.

Live smoke is opt-in and root-owned: at most eight total requests, no retries, 60 seconds and 128 output tokens per request, US$1 total planned paid usage. Preflight exact destination, authorized immutable credentials, complete input/output/reasoning cost bounds and isolation before any request. Unsafe or unknown bounds produce explicit skips, never a pass. No source credential content enters artifacts.

## Validation Sign-Off

- [x] Every task has automated verification or explicit manual evidence requirements.
- [x] Sampling continuity: no three consecutive tasks without automated verification.
- [x] Wave 0 covers all missing tests.
- [x] No watch-mode verification or unbounded smoke requests.
- [x] Actual feedback latency recorded.
- [x] Final format, Clippy, full tests, site and scoped smoke results recorded.
- [ ] nyquist_compliant set only after evidence review.

Approval: bounded live-smoke scope approved by user; all generations explicitly
skipped (0/8). Automated and Computer gates passed. Independent final review
is pending; nyquist_compliant remains false until that reconciliation.
