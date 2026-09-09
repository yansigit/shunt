---
phase: "16"
slug: "cross-provider-release-gate"
status: draft
nyquist_compliant: false
wave_0_complete: false
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

Runtime will be measured during execution; no timing or test count is assumed.

## Sampling Rate

Run the relevant named focused suite after each task; reject zero-test filtered runs. Run the full suite after each implementation wave and before final verification. Serialize Cargo. A nonzero exit, missing expected assertions, or changed production fingerprints blocks completion.

## Per-Task Verification Map

Planner must replace this seed with concrete task IDs, requirements REL-01–REL-06, threat references, exact named commands, expected nonzero test counts, and existing/new file status before plans pass. All tasks remain pending; no Phase 16 check has passed yet.

## Wave 0 Requirements

Existing test infrastructure is available. Planner must identify any focused coverage gaps and schedule their tests before dependent work; no new general-purpose framework or runtime dependency is authorized.

## Manual-Only Verifications

Computer visual evaluation uses an owned isolated documentation server, performed by root or gpt-6-astra/high. Save real observations or an explicit blocked result. Built HTML checks are not visual evidence.

Live smoke is opt-in and root-owned: at most eight total requests, no retries, 60 seconds and 128 output tokens per request, US$1 total planned paid usage. Preflight exact destination, authorized immutable credentials, complete input/output/reasoning cost bounds and isolation before any request. Unsafe or unknown bounds produce explicit skips, never a pass. No source credential content enters artifacts.

## Validation Sign-Off

- [ ] Every task has automated verification or explicit manual evidence requirements.
- [ ] Sampling continuity: no three consecutive tasks without automated verification.
- [ ] Wave 0 covers all missing tests.
- [ ] No watch-mode verification or unbounded smoke requests.
- [ ] Actual feedback latency recorded.
- [ ] Final format, Clippy, full tests, site and scoped smoke results recorded.
- [ ] nyquist_compliant set only after evidence review.

Approval: bounded live-smoke scope approved by user; validation execution pending.
