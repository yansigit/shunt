---
phase: 11-antigravity-protocol-and-credential-hardening
plan: "05"
subsystem: auth
tags: [antigravity, oauth, retry, conformance]
requires:
  - phase: "11-04"
    provides: account/session-bound envelope and tool identities
provides:
  - One same-account initial-401 refresh and replay with unchanged wire payload
  - Real-router commitment, account-swap, and source-file preservation evidence
affects: ["11-06", "11-07", "16"]
actuals:
  tokens: 11199
  tasks: 2
  commits: 1
tech-stack:
  added: []
  patterns: [private credential refresh dependency, single-send replay]
key-files:
  created:
    - src/server/antigravity_replay_tests.rs
    - src/server/antigravity_replay_tests/fixtures.rs
  modified:
    - src/adapters/gemini/mod.rs
    - src/auth/mod.rs
    - src/auth/antigravity/auth.rs
    - src/server.rs
    - src/retry.rs
    - src/upstream_timeout.rs
key-decisions:
  - OAuth token URL injection remains private to crate tests; no production override.
  - Check the expected account before exchanging its grant; retain the original project.
  - Drop the rejected upstream response before refresh; the replay has no retry loop.
  - Forced refresh is in-memory only; a legacy refresh-grant fingerprint change fails closed.
requirements-completed: [ANT-02, ANT-08]
coverage:
  - id: D1
    description: One replay in both client modes, identical wire body and one initial resolution
    requirement: ANT-08
    verification:
      - kind: e2e
        ref: src/server/antigravity_replay_tests.rs#antigravity_native_401_first_preheader_401_refreshes_same_account_and_replays_once
        status: pass
    human_judgment: false
  - id: D2
    description: Account swaps, repeated 401, refresh failure, ambiguous timeout, and committed text/tools never trigger unsafe replay
    requirement: ANT-02
    verification:
      - kind: e2e
        ref: cargo test --all-features --lib antigravity_native_401 -- --test-threads=1
        status: pass
    human_judgment: false
duration: 27min
completed: 2026-09-07
status: complete
plan_head_before: 3a39299
---

# Phase 11 Plan 05: One-shot account-bound 401 recovery

An initial native Antigravity 401 can refresh once without file writeback, replaying the exact envelope under a bearer bound to the original account and project.

## Accomplishments

- Private `CredentialResolver::refresh` keeps real-router tests hermetic without exposing a token-endpoint configuration knob.
- Account changes are rejected before token exchange; refresh errors and a second 401 terminate. The replay calls one send, not another retry driver.
- Native initial transport retries use connect-only evidence. Ambiguous sends, body/parser errors, and 401 errors after text/tool output do not replay.
- Ten named tests exercise both client modes, identical serialized request bytes, one resolution, exact inference/token counts, authoritative terminals, and unchanged synthetic credential bytes. The swap test explicitly simulates an external login and verifies the gateway leaves those externally replaced bytes intact.

## Task Commits

Tasks 1 and 2 share the same private test fixture and landed atomically in `f9cd0fc`.

## Verification

- Focused native 401 matrix: 10 passed, none ignored.
- `cargo test --all-features --workspace -- --test-threads=1`: **2,635 passed, 2 ignored**, 26 result groups; exit 0. Log: `/tmp/shunt-11-05-tests.log`.
- `cargo fmt --all --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed after final fixture split.
- `git diff --check`: passed; no tracked files deleted.
- User-requested backups were verified outside the repository with owner-only permissions. Both project settings files and the existing Codex credential source remained byte-for-byte unchanged after testing. No production opencodex state or port was used for development verification.

## Deviations from Plan

- **Rule 2 — credential-destination safety:** rejected an initial agent implementation that added a public token URL override. Moved integration fixtures into crate-local server modules and reused the private resolver instead. No such override shipped.
- **Rule 2 — request affinity:** expected account validation now precedes refresh, not merely replay.
- Added connect-only evidence to the existing retry policy and timeout wrapper; no new retry framework, dependency, config key, or credential write path.
- Scope paths were adjusted in the plan to record the private fixture modules. README, engineering-note, and maintained site updates remain in the dependent same-phase documentation plan (11-07); generated wiki is untouched.

## TDD Gate Compliance

The interrupted agent did not produce a trustworthy RED evidence record or RED commit. This is a TDD process deviation, not a claimed red-green run. The final behavior and full regression gates above were independently executed; the two tasks were committed together rather than manufacturing a historical failing-test commit.

## Issues Encountered

DeepSeek required a provider opt-in and was not enabled. GLM drafted the implementation; the primary agent interrupted it after review found the unsafe endpoint seam, corrected the implementation, strengthened the output assertions, and ran the final gates. A fixture-module import error during refactoring was corrected before successful verification.

## Next Phase Readiness

Ready for native lifetime/capacity evidence (11-06), then documentation and final phase gates (11-07). These are hermetic results, not live-provider proof.

## Self-Check: PASSED

All listed implementation and fixture files exist; the implementation commit and successful verification results were checked.
