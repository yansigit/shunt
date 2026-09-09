---
phase: 10-gemini-semantic-hardening
plan: "07"
subsystem: gemini-oauth-lifetime
tags: [rust, gemini, google-oauth, retry, cancellation, verification]
requires:
  - phase: 10-gemini-semantic-hardening
    provides: checked Gemini semantics, bounded framing, and gap-closure regressions
provides:
  - Hermetic real-router Google OAuth Code Assist lifetime evidence
  - Safe pre-connect retry and ambiguous post-send non-retry evidence
  - Phase-wide documentation, scope audit, and green release gates
affects: [gemini-transport, credential-resolution, provider-conformance]
actuals:
  tokens: 16855
  tasks: 2
  commits: 2
plan_head_before: e2daa7f79716931762d3409fe6af33d08052e61f
tech-stack:
  added: []
  patterns: [crate-private-dependency-seam, request-lifetime-capture, hermetic-dns-retry]
key-files:
  created: []
  modified:
    - src/auth/mod.rs
    - src/server.rs
    - src/adapters/gemini/mod.rs
    - docs/v2-gemini-semantic-hardening.md
    - scripts/check_phase10_scope.sh
    - tests/antigravity_process.rs
key-decisions:
  - "Resolve one request-local credential before Gemini endpoint, envelope, and retry construction; refreshed AppState snapshots retain the same private resolver."
  - "Prove permitted retry with a genuine connect refusal; a reset after the POST body is read remains non-retryable because replay could duplicate a billable generation."
  - "Drive descendant-lifetime process regressions through the real in-process router to exclude an unrelated intermittent macOS HTTP/1 EOF stall from the release gate."
requirements-completed: [GEM-01, GEM-02, GEM-03, GEM-04, GEM-05]
coverage:
  - id: D-01-D-02-D-15
    description: "Unary and streaming Code Assist requests carry one synthetic bearer and project through the real `/v1internal` router path."
    requirement: GEM-01
    verification:
      - kind: integration
        ref: cargo test --all-features gemini_google_oauth_code_assist_lifetime
        status: pass
    human_judgment: false
  - id: D-04-D-13-D-14-D-15
    description: "A genuine pre-connect failure retries with identical request identity, an ambiguous post-send close does not retry, and cancellation releases upstream and gateway capacity without re-resolution."
    requirement: GEM-05
    verification:
      - kind: integration
        ref: cargo test --all-features gemini_google_oauth_code_assist_lifetime
        status: pass
      - kind: integration
        ref: cargo test --all-features --test retry --test failover
        status: pass
    human_judgment: false
  - id: phase-10-release
    description: "All Gemini semantic, framing, OAuth lifetime, retry/failover, exclusion, format, lint, and serial workspace gates pass."
    requirement: GEM-01..05
    verification:
      - kind: release-gate
        ref: cargo test --all-features --workspace -- --test-threads=1
        status: pass
      - kind: scope-gate
        ref: bash scripts/check_phase10_scope.sh
        status: pass
    human_judgment: false
duration: 41min
completed: 2026-09-07
status: complete
---

# Phase 10 Plan 07: Google OAuth Lifetime and Release Evidence Summary

**Gemini Code Assist now has hermetic, real-router proof that one synthetic OAuth identity survives unary, streaming, safe retry, and cancellation paths without live credentials or credential writeback.**

## Performance

- **Duration:** 41 min
- **Started:** 2026-09-07T02:56:36Z
- **Completed:** 2026-09-07T03:37:21Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Added a crate-private, object-safe credential resolver boundary whose production implementation delegates to the existing resolver and whose test implementation supplies a synthetic Google OAuth bearer/project pair.
- Exercised the actual `/v1/messages` router and Gemini `/v1internal` request construction for unary, streaming, genuine connect-phase retry, ambiguous post-send failure, cancellation, and concurrency-slot release.
- Reconciled the engineering record, pinned the exact authorized `src/auth/mod.rs` phase diff in the scope audit, and passed every focused and repository-wide release gate.

## Task Commits

1. **Task 1: Prove Google OAuth request lifetime** — `6ea8a3f`
2. **Task 2: Close release evidence and gates** — `72fbb3e`

The fail-first fixture was committed before the continuation ledger as `a8b9727`; the resumed execution started from `e2daa7f` and retained that evidence unchanged.

## Decisions Made

- Credential resolution occurs exactly once per client request, before request construction. Retry closures own the already-resolved token, project, endpoint, model, and serialized request meaning.
- Only a proven connect-phase failure is eligible for a non-idempotent POST retry. A peer that reads the request and then closes is an ambiguous accepted request and remains single-attempt.
- No public constructor, config key, environment variable, auth mode, refresh path, migration, or credential writeback behavior was added.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Safety] Replaced the planned reset fixture with a genuine connect refusal**

- **Found during:** Task 1 GREEN verification
- **Issue:** The raw listener read the full POST and then reset the connection. Reqwest correctly classified that as a post-send request failure, not a safe connect failure; broadening the classifier would risk duplicate billable generations and violate Phase 9.
- **Fix:** Added a deterministic request-local DNS resolver whose first address refuses connection and whose second address reaches the capture server. Retained a separate full-body-close regression proving no replay after acceptance.
- **Verification:** The OAuth lifetime filter passes four cases, while retry and failover suites remain green.
- **Committed in:** `6ea8a3f`

**2. [Rule 3 - Blocking Gate] Removed an unrelated macOS HTTP/1 EOF flake from the Antigravity process regression**

- **Found during:** Task 2 serial workspace release gate
- **Issue:** Three full-suite attempts intermittently timed out in `streaming_finishes_when_a_descendant_holds_stdout_open`. Temporary diagnostics proved the adapter, SSE stream, observer, and concurrency wrapper had all reached end-of-stream while reqwest still awaited HTTP/1 framing.
- **Fix:** The two descendant-lifetime tests now drive the same real Axum router, proxy, child process, stdout stream, and response body via `Router::oneshot`; existing socket tests retain network-level coverage. The original 20-second guard and holder-process assertions are unchanged.
- **Verification:** Ten repeated exact runs, the complete Antigravity process target, and the full serial workspace suite pass. A separate review found no weakened process/body-stream invariant.
- **Committed in:** `72fbb3e`

**3. [Rule 2 - Missing Scope Guard] Authorized only the planned private auth seam**

- **Found during:** Task 2 scope audit
- **Issue:** The phase scope script rejected every `src/auth` change, including Plan 10-07's explicitly required private resolver boundary.
- **Fix:** Kept every other auth/persistence path forbidden and pinned the complete phase diff of `src/auth/mod.rs` by SHA-256.
- **Verification:** `bash scripts/check_phase10_scope.sh` passes against the committed phase range.
- **Committed in:** `72fbb3e`

---

**Total deviations:** 3 auto-fixed (1 safety, 1 blocking gate, 1 missing scope guard)
**Impact on plan:** Each change strengthened or preserved the stated safety boundary; no public/provider behavior or excluded surface was added.

## Issues Encountered

- The first retry fixture exposed an important distinction between connection refusal and an ambiguous close after request acceptance. The retry classifier was deliberately left unchanged.
- A pre-existing macOS socket-framing test flake blocked three complete release runs. It was isolated at the test boundary without changing Antigravity production behavior or relaxing the timeout.

## Verification

- `cargo test --all-features gemini_parallel_tool_result_identity` — pass
- `cargo test --all-features --test gemini_translate gemini_known_part_strictness` — pass
- `cargo test --all-features gemini_post_done_frames` — pass
- `cargo test --all-features gemini_google_oauth_code_assist_lifetime` — pass (4 cases)
- `cargo test --all-features --test gemini_translate --test gemini_conformance` — pass (51 tests)
- `cargo test --all-features --test retry --test failover` — pass (29 tests)
- `bash scripts/check_phase10_scope.sh` — pass
- `cargo fmt --all --check` — pass
- `cargo clippy --all-targets --all-features -- -D warnings` — pass
- `cargo test --all-features --workspace -- --test-threads=1` — pass

## User Setup Required

None. All OAuth evidence is synthetic and hermetic; no live credential, credential file, external service, or public configuration is required.

## Next Phase Readiness

- GEM-01 through GEM-05 have complete automated evidence and Phase 10 is ready for fresh goal-backward verification.
- Phase 11 can begin Antigravity protocol and credential hardening after the verifier confirms the phase goal.

## Self-Check: PASSED

- All declared implementation, test, documentation, and scope artifacts exist.
- Both resumed execution commits exist after `plan_head_before`.
- Every focused, exclusion, format, lint, and full serial workspace command passes.
- No live credential, credential writeback, public config/API, dependency, Google AI Studio Web, or generated wiki change is present.

---
*Phase: 10-gemini-semantic-hardening*
*Completed: 2026-09-07*
