---
phase: 11-antigravity-protocol-and-credential-hardening
reviewed: 2026-09-07T19:23:52Z
depth: standard
files_reviewed: 20
files_reviewed_list:
  - src/adapters/gemini/mod.rs
  - src/adapters/responses/inbound.rs
  - src/adapters/responses/request.rs
  - src/adapters/responses/websocket.rs
  - src/auth/mod.rs
  - src/auth/shared.rs
  - src/auth/antigravity/auth.rs
  - src/auth/antigravity/catalog.rs
  - src/model/antigravity_request.rs
  - src/model/gemini.rs
  - src/model/gemini_request.rs
  - src/retry.rs
  - src/server.rs
  - src/server/antigravity_replay_tests.rs
  - src/server/antigravity_replay_tests/fixtures.rs
  - src/upstream_timeout.rs
  - tests/antigravity_catalog.rs
  - tests/antigravity_tool_scope.rs
  - tests/gemini_conformance.rs
  - docs/antigravity-tool-identities.md
findings:
  critical: 0
  warning: 3
  info: 3
  total: 6
status: issues_found
---

# Phase 11: Code Review Report

**Reviewed:** 2026-09-07T19:23:52Z
**Depth:** standard
**Files Reviewed:** 20
**Status:** issues_found

## Summary

Full diff 608988f..HEAD for the Phase 11 native Antigravity scope was read
file-by-file, with cross-file tracing of the actual call paths: credential
resolution -> envelope/tool-context construction -> catalog fetch -> exact
admission -> redirect-bounded send -> RetrySafety::ConnectOnly retry driver
-> 401 replay seam -> SSE/unary collection. Both streaming and non-streaming
downstream modes, and the module-local #[cfg(test)] tests in src/ that a
prior reviewer missed, were inspected.

Verified against the code, not the plan's claims:

- Exact admission (antigravity_exact_catalog_admission) fails closed on a
  stale catalog, on any non-Gemini upstream id, on every folded/clamped
  effort, and on rewritten model tuples; the module-local matrix test covers
  this.
- The redirect boundary (is_safe_inference_url + antigravity_inference_client)
  is fail-closed on the initial URL, on off-origin redirects, on
  path/query/fragment changes, and on explicit-:443 spellings, and bounds
  loops at ten; the loopback exception never broadens production admission
  because exact_origin is additionally required.
- Account binding holds at every seam: the catalog cache key, the scoped
  session, the tool-context scope tag, the pre- and post-refresh fingerprint
  comparison in force_refresh_in_memory, and the replay gate's tuple check
  (fingerprint == original && project == original && token non-empty).
- The 401 replay is structurally bounded to one reissue: the decision branch
  runs strictly before any body read, the replayed send carries no retry
  budget, a second 401 falls through to terminal mapping, and refresh
  failure never writes the credential file (asserted byte-for-byte in the
  replay fixtures).
- RetrySafety::ConnectOnly genuinely reaches the driver: only is_connect()
  transport errors retry; TTFB timeouts and returned transient statuses
  terminate. src/upstream_timeout.rs propagates connect_phase for the
  SendError wrapper.
- Unkeyed scope tags are consistently documented as contextual, not
  authentication, in both code comments and docs/antigravity-tool-identities.md;
  every doc claim I checked (v2 codec layout, null for omitted parallel-call
  signatures, integer normalization bounds including the 2^53 no-rounding
  rule, legacy-history migration requirement, shared-session fallback for
  identical openings) matches the implementation.

I re-ran the native suites directly (parallel runs proved load-sensitive --
see WR-03): replay 10 passed, native affinity 8 passed,
tests/antigravity_tool_scope 5 passed, tests/antigravity_catalog 2 passed,
tests/gemini_conformance antigravity 6 passed. No critical defects found.
The three warnings below are robustness/test-reliability gaps, not shipped
behavior bugs.

## Warnings

### WR-01: Replay test mutates SHUNT_ANTIGRAVITY_AUTH_FILE without the process env lock or a guard

**File:** src/server.rs:1399-1416
**Issue:**
antigravity_native_affinity_production_default_resolver_once captures, sets,
and restores SHUNT_ANTIGRAVITY_AUTH_FILE by hand. Unit tests in the same
binary (src/auth/mod.rs:803, src/auth/antigravity/mod.rs:846,
src/auth/antigravity/mod.rs:869) mutate the same variable under cargo's
default parallel test threads, and src/auth/antigravity/mod.rs:264-273
explicitly documents the established convention: env mutation must go
through the shared ANTIGRAVITY_AUTH_FILE_ENV_LOCK (CONFIG_ENV_LOCK) because
independent mutexes do not exclude each other and have already produced
~40% cross-test flake historically. This test follows neither that
convention nor the EnvVarGuard RAII pattern: if any assert between set_var
and the restore panics, the variable is left pointing at the synthetic
missing path for the rest of the binary's run, and concurrent set/restore
can race so the expected credential-missing failure turns into an
unexpected success.
**Fix:** reuse EnvVarGuard::set (from crate::auth::shared) inside
ANTIGRAVITY_AUTH_FILE_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner()),
matching the pattern already used in the auth tests, so restore is
drop-safe and serialized against the other env-mutating unit tests.

### WR-02: Catalog discovery still carries the account bearer under the default redirect policy

**File:** src/auth/antigravity/catalog.rs:256-268 (called from
src/adapters/gemini/mod.rs:351-356 with state.http_client)
**Issue:**
Phase 11 (11-02/D-16) hardened inference so the Bearer can never cross an
origin or endpoint boundary -- antigravity_inference_client
(src/adapters/gemini/mod.rs:401) installs a redirect policy that refuses
any target not passing is_safe_inference_url. Catalog discovery
(/v1internal:fetchAvailableModels) sends the same account bearer on the
same configured control plane but through state.http_client, whose default
policy follows up to 10 redirects. A 30x from the configured host therefore
forwards the credential to the Location target -- exactly the threat class
the inference hardening refuses, on a sibling call in the same request
path. The replay/origin tests cover only streamGenerateContent.
**Fix:** issue the catalog fetch through an origin-bounded client built
from the same policy restricted to the configured base origin (allowing
only /v1internal:fetchAvailableModels on the exact origin), or install the
equivalent Policy::custom on a shared Antigravity client factory so both
v1internal calls inherit it.

### WR-03: Replay fixtures are load-sensitive; exact hit counts can flake under parallel execution

**File:** src/server/antigravity_replay_tests.rs:403-428 (and
antigravity_native_401_success_response_never_replays, default TTFB)
**Issue:**
Running the replay subset in parallel on a busy machine reproduced two
failures that pass serially: the TTFB-400 fixture recorded 2 inference hits
instead of the asserted 1, and the success scenario 504'd instead of 200.
The cause is a real call-path interaction, not flaky assertions: under load
the loopback connect/TTFB can exceed the budget, and
RetrySafety::ConnectOnly by design retries a connect-phase error, so the
second (again timing-out) attempt doubles the wiremock hit count and the
run terminates with 504. This means (a) the suite flakes on loaded CI
machines, and (b) a future regression that silently adds a connect-phase
retry to the replay seam would be masked by the same flake rather than
failing deterministically.
**Fix:** either raise the TTFB budget in these fixtures far above realistic
loopback connect latency variance (e.g. 2s for the ambiguous-send case,
which only needs a timeout) or assert a bounded range (1..=2 hits, with the
exact-once replay assertions kept on the bearer-keyed mocks that are not
TTFB-sensitive); document in the fixtures that one connect-phase retry is
permitted by D-17 so an extra hit is not read as a replay.

## Info

### IN-01: Duplicate catalog membership check in exact admission

**File:** src/model/antigravity_request.rs:397 and 441
**Issue:** antigravity_exact_catalog_admission checks
catalog.ids.contains(upstream_model) inside the Gemini branch and again
unconditionally at the tail; the tail check only matters for the non-Gemini
branch, which already returned Err before reaching it.
**Fix:** drop the tail check (or the in-branch one) and keep a single check
after the tier/effort validation.

### IN-02: catalog_ids compatibility seam hardcodes the "legacy" fingerprint

**File:** src/auth/antigravity/catalog.rs:247-257
**Issue:** the pub wrapper passes the constant "legacy" account
fingerprint. A future production caller that reaches for the shorter name
would silently collapse two distinct no-project accounts onto one cache
key -- the comment is the only guard.
**Fix:** narrow it (pub(crate), or rename to catalog_ids_for_legacy_fixture)
so the account-aware variant is the only natural choice at call sites.

### IN-03: Replay-seam Commitment check is constant at its location

**File:** src/adapters/gemini/mod.rs:469-472
**Issue:** Commitment::default().may_redispatch() is unconditionally true
there -- the commitment is fresh and nothing has been read or sent
downstream at that point; the real bound is structural (branch runs once,
before any body read). The call is harmless documentation, but it can read
as if a runtime gate exists where none does.
**Fix:** keep it only if the intent is future-proofing; otherwise replace
it with a comment stating the structural bound so a reader does not hunt
for the transition that never happens in this branch.

---

_Reviewed: 2026-09-07T19:23:52Z_
_Reviewer: gsd-code-reviewer (standard depth)_
_Depth: standard_

## Owner triage and remediation

The original findings above are preserved as reviewer observations, not silently
rewritten into independent verification. The owner checked the actual code:

- WR-01 confirmed: the production-resolver test now uses the shared configuration
  environment lock and a panic-safe local guard that restores the previous value.
  The existing shared `EnvVarGuard` removes rather than restores a prior value,
  so it was not reused blindly.
- WR-02 partly confirmed: the default client follows catalog redirects, including
  sibling paths. The claim that it necessarily forwards authorization across
  hosts is overstated: reqwest strips sensitive headers across hosts. A new
  loopback test failed on the original same-origin redirect and now passes with
  catalog redirects refused entirely. Initial catalog origin validation reuses
  the existing canonical-root predicate; no endpoint override or credential
  writeback behavior is added.
- WR-03: the claimed connect-error explanation is not established by the
  supplied trace. The fixture called `Config::load` without the shared environment
  lock, allowing sibling configuration tests to affect its timeout/retry values.
  Loading is now serialized, and the deliberate delayed-response timeout is
  raised from 400ms to 2000ms. Exact one-inference and zero-refresh assertions
  remain unchanged; the suggested range relaxation was rejected.
- IN-01 fixed: redundant membership check removed.
- IN-02 fixed: the legacy wrapper is now private and `cfg(test)` only; production
  callers must provide an account fingerprint.
- IN-03 retained intentionally: the comment explicitly describes structural
  pre-output placement and one-shot control flow. The fresh commitment value is
  not claimed to track downstream events after this branch.

Release gates and final evidence are recorded in `11-07-SUMMARY.md` and
`11-VERIFICATION.md` after execution, not inferred from this remediation list.
