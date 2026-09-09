---
phase: 11-antigravity-protocol-and-credential-hardening
verified: 2026-09-09T03:52:11Z
status: passed
score: 10/10 must-haves verified
covered_files:
  - .planning/REQUIREMENTS.md
  - .planning/phases/11-antigravity-protocol-and-credential-hardening/11-01-PLAN.md
  - .planning/phases/11-antigravity-protocol-and-credential-hardening/11-01-SUMMARY.md
  - .planning/phases/11-antigravity-protocol-and-credential-hardening/11-02-PLAN.md
  - .planning/phases/11-antigravity-protocol-and-credential-hardening/11-02-SUMMARY.md
  - .planning/phases/11-antigravity-protocol-and-credential-hardening/11-03-PLAN.md
  - .planning/phases/11-antigravity-protocol-and-credential-hardening/11-03-SUMMARY.md
  - .planning/phases/11-antigravity-protocol-and-credential-hardening/11-04-PLAN.md
  - .planning/phases/11-antigravity-protocol-and-credential-hardening/11-04-SUMMARY.md
  - .planning/phases/11-antigravity-protocol-and-credential-hardening/11-05-PLAN.md
  - .planning/phases/11-antigravity-protocol-and-credential-hardening/11-05-SUMMARY.md
  - .planning/phases/11-antigravity-protocol-and-credential-hardening/11-06-PLAN.md
  - .planning/phases/11-antigravity-protocol-and-credential-hardening/11-06-SUMMARY.md
  - .planning/phases/11-antigravity-protocol-and-credential-hardening/11-07-PLAN.md
  - .planning/phases/11-antigravity-protocol-and-credential-hardening/11-07-SUMMARY.md
  - .planning/phases/11-antigravity-protocol-and-credential-hardening/11-CONTEXT.md
  - .planning/phases/11-antigravity-protocol-and-credential-hardening/11-PATTERNS.md
  - .planning/phases/11-antigravity-protocol-and-credential-hardening/11-RESEARCH.md
  - .planning/phases/11-antigravity-protocol-and-credential-hardening/11-REVIEW.md
  - .planning/phases/11-antigravity-protocol-and-credential-hardening/11-VALIDATION.md
  - .planning/phases/11-antigravity-protocol-and-credential-hardening/COVERAGE.md
  - README.ja.md
  - README.ko.md
  - README.md
  - README.zh-CN.md
  - docs/antigravity-tool-identities.md
  - docs/notes/antigravity-daily-host.md
  - scripts/check_phase11_scope.sh
  - site/src/content/docs/ja/providers/antigravity.mdx
  - site/src/content/docs/ko/providers/antigravity.mdx
  - site/src/content/docs/providers/antigravity.mdx
  - site/src/content/docs/zh-cn/providers/antigravity.mdx
  - src/adapters/gemini/mod.rs
  - src/adapters/gemini/sse.rs
  - src/adapters/responses/inbound.rs
  - src/adapters/responses/request.rs
  - src/adapters/responses/websocket.rs
  - src/auth/antigravity/auth.rs
  - src/auth/antigravity/catalog.rs
  - src/auth/mod.rs
  - src/auth/shared.rs
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
  - tests/antigravity_translate.rs
  - tests/gemini_conformance.rs
covered_digest: "v1:sha256:bd0c45a5f8e880441fc658b42fdf92cedb5a6b9df2a156ce223a5c6e4425b313"
behavior_unverified: 0
overrides_applied: 0
decision_coverage:
  honored: 21
  total: 21
  not_honored: []
human_verification: []
---

# Phase 11: Antigravity Verification

## Phase 15 re-verification (2026-09-09)

Independent source audit found only Phase 15 bookkeeping/docs plus a `#[cfg(test)]` router dependency-injection seam in the covered files; no Antigravity credential, catalog, protocol, retry, or cancellation implementation changed. Root executed `node /tmp/shunt-phase12-isolated-run.cjs env RUSTFLAGS=-Dwarnings cargo test --quiet --all-features --workspace` at `eb60174`: 2,972 passed, 0 failed, 2 existing ignored. No Phase 11 truth, artifact, key link, or negative boundary regressed. Production OpenCodex config mtime/SHA and backup/invalid inventory were unchanged.

## Phase 14 regression re-verification (2026-09-08)

Root inline verification; no independent verifier pass is claimed.
Compared covered files to 8013f19. Antigravity implementation, catalog, replay,
fixtures and plan contracts are unchanged. Shared credential additions are
redacted and explicitly excluded from all three Responses outbound header paths;
existing Antigravity exclusions remain intact. README changes add the new
providers in all four locales; ANT requirements are unchanged. Ten active
antigravity_native_401 unit tests passed, including same-account replay,
account swap, refresh failure, post-send timeout and output/tool boundaries.
Decision coverage remains 21/21.

Every test used node /tmp/shunt-phase12-isolated-run.cjs with all-features and
nonzero selection. Production config mtime/SHA and backup inventory remained
unchanged. The preceding observed full workspace gate passed 2,942 tests with
0 failures and 2 existing ignored tests; site build passed 169 pages in four
languages. No disabled requirement tests were found in the rechecked groups.
Fingerprint was regenerated through the bundled GSD tool after code review and
successful execution. Historical results below are historical. Live/Computer
acceptance remains outside this hermetic re-verification.


**Goal:** Antigravity users can run faithful Cloud Code Assist turns with
canonical-destination credential safety and account/session-consistent tools
and reasoning.

**Status:** passed. Initial phase verification by the primary agent, separately
from the standard GLM code review. This certifies the declared hermetic protocol
contract, not live subscription availability or cryptographic provenance of
client-replayed unkeyed tags.

## Observable truths and requirement coverage

The four roadmap success criteria and plan-specific truths are merged below;
none are omitted. Unsupported-before-dispatch means before **inference**:
account resolution and authenticated catalog discovery are prerequisites.

| # | Truth / requirements | Status | Behavioral evidence |
| --- | --- | --- | --- |
| 1 | Canonical inference destinations and credential-safe redirects (ANT-01) | VERIFIED | Six native origin tests pass: exact URL matrix, same-origin follow, off-origin zero-hit rejection, ten-hop cap, unsafe initial base, catalog redirect refusal. The latter failed before its fix. Catalog requests now validate the root and refuse every redirect. Loopback is an explicit hermetic/local-proxy exception, not a production host wildcard. |
| 2 | Immutable account/token/project/catalog tuple through response/retry (ANT-02) | VERIFIED | Injected and production-default resolvers are each called once; account/project matrices and in-flight swap fixture capture distinct upstream tuples. Refresh validates the original account before token exchange and again afterward. Native cancellation captures expected bearer/project and unchanged fixture bytes/mtime. |
| 3 | Fresh exact model/effort admission (ANT-04) | VERIFIED | Full tuple unit matrix plus real-router zero-inference-hit negatives cover stale/cold/unknown/rewritten/unsupported cases. Catalog keys include backend/account/project; successful TTL snapshots are fresh, failed refresh snapshots are not. The native adapter calls exact admission, never the legacy guesser. |
| 4 | Exact agent envelope and stable opaque session (ANT-03) | VERIFIED | Envelope fixtures assert metadata and UUID shape; account-scoped canonical opening is stable as history grows. Identical openings within an account share a session by design, rather than promising caller-distinct conversations without a caller identity. Account/prompt fields are hashed, not emitted as session text. |
| 5 | Matching ordered tool history and preserved signatures (ANT-05) | VERIFIED | Real-router roundtrip/rejection plus sequential/parallel parity and numeric-equivalence tests pass. V2 context tags bind original name/arguments/ordinal/account/session/signature; malformed, copied, mismatched, duplicate and orphaned history fails. Supplied upstream signatures are not synthesized; absent parallel-call signatures remain absent where allowed. Tags are unkeyed contextual checks, not proof against deliberate recomputation. |
| 6 | Always-SSE with incremental streaming and bounded unary parity (ANT-06) | VERIFIED | Real-loopback both-mode fixture captures exact SSE path/query and equivalent semantic output. Early-text test observes output before upstream completion. Decoder cap and cap-plus-one tests pass. The streaming path owns a lazy body and does not collect successful output. |
| 7 | Strict wrapper/tool/error/terminal handling (ANT-07) | VERIFIED | Both-mode malformed/truncated/invalid UTF-8/incomplete-tool/embedded-error/duplicate-terminal/post-DONE matrix fails closed. Checked semantic processing and EOF finalization are called in both modes, not merely declared. |
| 8 | One same-account pre-output 401 refresh/replay (ANT-08) | VERIFIED | Ten router tests assert initial 401 gives exactly two identical-body inference requests and one refresh; second 401/refresh failure/account swap/legacy grant rotation terminate. Text/tool/body-error/ambiguous-send cases have exact one inference and no refresh. File byte assertions and one initial resolve remain intact. Parallel and serial runs pass. |
| 9 | Cancellation releases upstream and admission (ANT-02/06/08; plan 06) | VERIFIED | Both native lifetime fixtures hold the sole slot, prove a second request is rejected, drop/cancel the first, await upstream body destruction, then admit an independently initiated request. No detached producer or automatic retry is used. |
| 10 | Locale parity, narrow scope, confidence limits (all ANT; plan 07) | VERIFIED | All eight maintained surfaces manually compared and checked by executable token/exclusion gate. Four-language build succeeds. Exact Git-visible path gate rejects an untracked filename with spaces; scanner self-probes pass. Live-provider guarantees, Google AI Studio Web, and wiki edits are excluded. |

**Score:** 10/10, with runtime evidence for all state/lifetime/ordering truths.
All ANT-01 through ANT-08 IDs occur in the plans and the roadmap traceability
table; none are orphaned.

## Artifacts, links, and actual data flow

Bundled GSD `verify.artifacts` passes **21/21** declarations and
`verify.key-links` passes **9/9** declared links. Plans 06/07 declare no
additional links. Missing closing frontmatter delimiters originally prevented
GSD from reading the declarations; all seven delimiters were repaired and the
queries rerun successfully.

| Source → consumer | Substantive implementation and flow |
| --- | --- |
| `server.rs` → `auth/mod.rs` → native adapter | Private resolver returns real request credential once; adapter captures account/project/token instead of re-reading on each send. Default resolver remains wired. |
| `catalog.rs` → `antigravity_exact_catalog_admission` | Actual catalog JSON keys and freshness feed exact model/effort validation before inference. Cache identity is account-aware; private legacy wrapper compiles only in tests. |
| `antigravity_request.rs` → adapter serialization | Canonical opening/account scope and exact admitted model form the actual serialized envelope, retained for replay. |
| `gemini_request.rs` ↔ `gemini.rs` | Ordered assistant calls/results and original thought signatures traverse real request translation and checked output, including multi-round router tests. |
| Upstream bytes → `GeminiSseDecoder` → checked machine | Same parsed frames supply incremental SSE or bounded unary collection; malformed finish does not create success. |
| Initial response → private refresh seam → `send_once` | Only the initial pre-body 401 branch refreshes; replay cannot re-enter that branch or inherit another retry loop. ConnectOnly rejects ambiguous timeout retry. |
| Native body/future → RAII upstream/capacity | Drop ownership is exercised through saturation and reacquisition, not inferred from symbol presence. |
| Source MDX → static build | All four provider pages render; 161 pages and four language indexes build successfully. |

## Executed checks

- Seven native filters: SSE 5, origin 6, affinity 10, envelope 2, tool signatures 6,
  401 10, lifetime 2 — **41 passed, 0 ignored**, each filter nonzero.
- Release suite: **2,638 passed, 2 pre-existing ignored**, 26 result groups.
  Log: `/tmp/shunt-11-07-tests.log`.
- GSD regression gate, separate bounded run:
  `gsd-tools.cjs run-with-timeout 600 -- cargo test --all-features --workspace -- --test-threads=1`
  returned zero, **2,638 passed, 2 ignored**. Prior-phase integration coverage
  includes retry, failover, passthrough, Responses HTTP/WS/continuation,
  Gemini translation and Code Assist conformance.
  Log: `/tmp/shunt-11-regression-gate.log`.
- Format, warnings-denied Clippy, whitespace: pass.
- Site `npm run build`: pass, MDX validation and 161 pages. Existing toolchain
  deprecation/stemming notices do not represent failed checks.
- Verifier separately reran four exact named tests, one active test each:
  `server::antigravity_replay_tests::antigravity_native_401_first_preheader_401_refreshes_same_account_and_replays_once`;
  `server::tests::antigravity_native_affinity_inflight_swap_keeps_a_tuple_immutable`;
  `antigravity_native_lifetime_unary_cancellation_releases_upstream_and_capacity`;
  `antigravity_native_tool_signature_roundtrips_sequential_history`.
- Verifier separately executed `bash scripts/check_phase11_scope.sh`: pass.
  Scanner rejects assembled Google-token, JWT, bearer and high-entropy probes;
  placeholders/field names pass. It does not claim exhaustive secret detection.

## Prohibitions and test quality

All nine plan prohibition entries are resolved by wired evidence, rather than
carrying their planning-time `flagged/unverified` defaults into an unexplained pass:

| Prohibition | Enforcement evidence |
| --- | --- |
| No unary native call / no successful-stream accumulation | Exact captured SSE endpoints in both modes; early output before EOF. |
| No off-origin inference bearer | Redirect receiver has zero requests; exact URL predicate and custom redirect policy. |
| No cross-account catalog/project reuse | Same-project different-account catalog tests; in-flight tuple swap and refresh-before-exchange account rejection. |
| No prompt-only session/signature identity | Different-account same-opening scope assertions and v2 field-binding negatives. |
| No signature synthesis/persistence | Exact upstream-signature roundtrips; missing signatures remain absent; source has no new signature store/write path. |
| No post-output/tool/body/ambiguous replay | Exact-hit 401 fixtures, including a valid tool that has already escaped downstream. |
| No Google AI Studio Web or wiki changes | Exact path and implementation-exclusion gate plus baseline diff. |
| No live credentials/service calls in fixtures | Inspected synthetic paths/private token-loopback resolver; captured loopback requests; unchanged fixture bytes and unchanged backed-up source files. |

Tests exercise value/behavioral assertions, not status-only success. Parity alone
is not treated as an independent oracle: fixture wire fields, ordering, terminal
counts, destination captures, unchanged bytes, drop notifications, and negative
hit traps independently constrain behavior. No requirement relies only on a
disabled test. The two ignored workspace cases are unrelated existing tests.
Some older loopback fixtures can return early if binding is unavailable; this run
successfully binds, and the new native lifetime fixtures explicitly assert
binding, so the release matrix cannot silently pass in a non-binding environment.

## Review and anti-pattern disposition

The independent review's three warnings were owner-triaged and corrected in
`ade4417`; the report preserves both original observations and corrections.
No unreferenced TBD/FIXME/XXX marker or new disabled test was found in the native
implementation/test scope. The fresh Commitment value is deliberately a
structural pre-output annotation, not a claimed mutable runtime tracker.
No new public setting, credential schema, or writeback behavior was introduced.

### Decision Coverage

All trackable CONTEXT decisions are honored: **21/21**, none missing (bundled
`check.decision-coverage-verify` result). Documentation limits unkeyed identity
claims and catalog availability explicitly.

## Human verification and confidence

None required for the scoped hermetic acceptance contract. Live credentials and
live Cloud Code Assist calls are explicitly excluded by the phase validation
strategy; they were not silently skipped and then called verified. This report
does not certify current upstream model availability or account eligibility.

The user-requested backup is owner-only and external to Git. All three source
files still compare byte-for-byte equal. Production opencodex state/port and
generated wiki were not touched.

No unresolved phase-goal gap remains. Phases 12–16 remain separate work.

## September 8 regression refresh

Independent GLM/high source audit found no invariant regressions in covered
changes since the prior verification. Phase12 adds the Cursor retry constant
and test-only server helper without changing the existing retry policies;
README additions retain four-locale parity. Historical evidence below remains
historical, not a claim of fresh execution.

Fresh root execution: full `cargo test --all-features --workspace` passed
(2194 active library tests, two prior ignored benchmarks, main and integration
suites all passed); format and warnings-denied all-target/all-feature Clippy
passed. The five-check rebuilt-binary mock smoke passed. Every process tree
used the isolated-home wrapper; live config mtime/SHA and backup/invalid-file
inventory stayed unchanged. This is hermetic evidence, not live provider
availability. Computer smoke was blocked by host Terminal/localhost controls
and is not claimed passed. Fingerprint regenerated with the bundled GSD tool
only after this audit and successful regression execution.
