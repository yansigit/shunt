---
phase: 09-provider-conformance-foundation
plan: "04"
subsystem: provider-conformance
tags: [anthropic, vercel, credentials, cancellation, concurrency]

requires:
  - phase: 09-provider-conformance-foundation
    provides: shared redispatch commitment gate from Plan 09-01
provides:
  - Generic Anthropic Vercel request, authentication, SSE, and error conformance
  - Structurally redacted Credential diagnostics and cross-origin credential rebinding proof
  - Bounded response-drop cancellation, global permit, and account-admission release evidence
affects: [gemini-hardening, antigravity-hardening, cursor-hardening, openai-chat]

actuals:
  tokens: 8971
  tasks: 3
  commits: 12
plan_head_before: 0c50146196a024fb7b1651cf269024c12987fa6c

tech-stack:
  added: []
  patterns:
    - Generic-provider conformance through real Axum gateways and loopback upstreams
    - Secret-bearing enums expose only structural Debug output
    - Response-scoped RAII release is proved with observable upstream cancellation

key-files:
  created: []
  modified:
    - src/auth/mod.rs
    - src/concurrency.rs
    - src/adapters/mod.rs
    - tests/passthrough.rs
    - tests/failover.rs
    - tests/codex_multi_account.rs

key-decisions:
  - "Vercel remains an ordinary manually configured Anthropic provider; no adapter, preset, or URL policy was added."
  - "Credential Debug output retains variant and API-key header structure but omits every secret and private identity field."
  - "Existing response-owned RAII is sufficient; cancellation coverage required no production ownership changes."

patterns-established:
  - "Cancellation proof: observe the upstream body being dropped, then reacquire capacity under a timeout."
  - "Credential proof: build harmless split markers and assert exact selected headers plus absence of both inbound slots."

requirements-completed: [PRES-05, SAFE-05, SAFE-07]

coverage:
  - id: D1
    description: "A manual generic Anthropic route preserves Vercel-compatible request, authentication, incremental SSE, and provider-error behavior."
    requirement: PRES-05
    verification:
      - kind: integration
        ref: "cargo test --all-features --test passthrough vercel_anthropic"
        status: pass
    human_judgment: false
  - id: D2
    description: "Credential diagnostics are structurally redacted and failover rebinds each selected credential to its own destination."
    requirement: SAFE-05
    verification:
      - kind: unit
        ref: "cargo test --all-features credential_redaction"
        status: pass
      - kind: integration
        ref: "tests/failover.rs#route_selected_credentials_are_rebound_per_destination"
        status: pass
    human_judgment: false
  - id: D3
    description: "Dropping a partial response cancels upstream work and releases global and account capacity within bounded time."
    requirement: SAFE-07
    verification:
      - kind: integration
        ref: "cargo test --all-features response_drop_releases"
        status: pass
    human_judgment: false

duration: 15min
completed: 2026-09-06
status: complete
---

# Phase 9 Plan 04: Vercel, Credential, and Cancellation Conformance Summary

**Generic Anthropic Vercel traffic is characterized end to end, credential diagnostics are safe by construction, and response cancellation demonstrably releases upstream work and gateway capacity.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-09-06T21:23:24Z
- **Completed:** 2026-09-06T21:38:00Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments

- Added named real-gateway Vercel-through-Anthropic cases for exact request/model forwarding, bearer and `x-api-key` injection, incremental SSE, missing credentials, and exact 400/401/429/503 relay.
- Replaced secret-bearing derived `Credential` diagnostics with a manual structural formatter and proved destination-specific credential rebinding across failover.
- Proved downstream response drop is observed upstream and makes global permits plus provider account admission reusable under bounded timeouts.

## Task Commits

1. **Task 1: Characterize Vercel through the existing generic Anthropic route** - `6930d32` (test)
2. **Task 2 RED: Expose credential diagnostic leakage** - `b81ecb4` (test)
3. **Task 2 GREEN: Redact and rebind provider credentials** - `3e55a24` (fix)
4. **Task 3: Prove response-drop resource release** - `d9822b6` (test)

The plan ledger measured 12 commits from `plan_head_before` through the production-complete HEAD because Plan 09-02 committed eight non-overlapping changes concurrently on the shared branch. The four commits above are the Plan 09-04 commits.

## Files Created/Modified

- `src/auth/mod.rs` - Manual redacted `Debug` implementation and all-variant structural regression test.
- `src/concurrency.rs` - Bounded global-permit response-drop proof.
- `src/adapters/mod.rs` - Partial-stream account-admission response-drop proof.
- `tests/passthrough.rs` - Named Vercel generic-Anthropic and observable upstream-cancellation cases.
- `tests/failover.rs` - Cross-origin, per-route credential rebinding test.
- `tests/codex_multi_account.rs` - Real gateway/account-pool cancellation and reuse test.

## Decisions Made

- Kept Vercel entirely on the existing generic Anthropic configuration surface.
- Kept credential discovery, refresh, persistence, and writeback unchanged.
- Retained the existing `PermitBody` and `with_admission` ownership design because the new cancellation tests passed without production changes.
- Reviewed README, engineering docs, site English/ko/ja/zh-cn surfaces, and generated `wiki/`; no documentation change is needed because no public behavior, configuration, provider claim, or endpoint changed.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - all tests use loopback upstreams and synthetic split markers.

## Next Phase Readiness

- PRES-05, SAFE-05, and SAFE-07 have executable evidence for later provider phases.
- No Vercel-specific runtime surface, credential writeback change, dependency, or Google AI Studio Web surface was introduced.

## Self-Check: PASSED

- All six declared implementation/test files exist.
- Plan commits `6930d32`, `b81ecb4`, `3e55a24`, and `d9822b6` exist.
- Focused verification, the three integration suites, formatting, and Clippy passed.

---
*Phase: 09-provider-conformance-foundation*
*Completed: 2026-09-06*
