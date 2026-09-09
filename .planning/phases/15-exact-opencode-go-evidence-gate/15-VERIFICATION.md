---
phase: 15-exact-opencode-go-evidence-gate
verified: 2026-09-09T03:17:42Z
status: passed
score: 13/13 truths verified
behavior_unverified: 0
overrides_applied: 0
decision_coverage:
  honored: 11
  total: 11
  not_honored: []
human_verification: []
human_review_completed: 2026-09-09T03:43:32Z
human_review_source: 15-UAT.md
covered_files:
  - .planning/REQUIREMENTS.md
  - .planning/ROADMAP.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-01-PLAN.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-01-SUMMARY.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-02-PLAN.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-02-RED.json
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-02-SUMMARY.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-03-PLAN.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-03-SUMMARY.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-04-PLAN.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-04-SUMMARY.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-05-PLAN.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-05-RED.json
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-05-SUMMARY.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-06-PLAN.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-06-SUMMARY.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-CONTEXT.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-EDGE-COVERAGE.json
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-EVIDENCE.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-PATTERNS.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-PLAN-PREFLIGHT.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-PLAN-REVIEW.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-RESEARCH.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-REVIEW.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-RUNTIME-REVIEW.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-SECURITY.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-UI-REVIEW.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/15-VALIDATION.md
  - .planning/phases/15-exact-opencode-go-evidence-gate/COVERAGE.md
  - README.ja.md
  - README.ko.md
  - README.md
  - README.zh-CN.md
  - docs/opencode-go-evidence-gate.md
  - site/src/content/docs/guides/providers.mdx
  - site/src/content/docs/ja/guides/providers.mdx
  - site/src/content/docs/ja/providers/opencode-go.md
  - site/src/content/docs/ja/reference/configuration.md
  - site/src/content/docs/ko/guides/providers.mdx
  - site/src/content/docs/ko/providers/opencode-go.md
  - site/src/content/docs/ko/reference/configuration.md
  - site/src/content/docs/providers/opencode-go.md
  - site/src/content/docs/reference/configuration.md
  - site/src/content/docs/zh-cn/guides/providers.mdx
  - site/src/content/docs/zh-cn/providers/opencode-go.md
  - site/src/content/docs/zh-cn/reference/configuration.md
  - site/src/lib/i18n.ts
  - src/codex_endpoint.rs
  - src/config.rs
  - src/config/presets.rs
  - src/proxy.rs
  - src/proxy/capability.rs
  - src/proxy/failover.rs
  - src/proxy/opencode_go_tests.rs
  - src/routing.rs
  - src/server.rs
  - tests/check_cli.rs
  - tests/opencode_go_docs.rs
  - tests/opencode_go_evidence.rs
covered_digest: "v1:sha256:220a0eb1ca677d2472ccd26f60495fe8795e980574e58362b0e9bc7b2c426afa"
---

# Phase 15: Exact OpenCode Go Evidence Gate — Verification

## Phase 16 regression reconciliation

Root reviewed Phase 16's diff against 44721a6: no runtime, manifest or wiki
changes. Covered changes are documentation/notice corrections, completion
bookkeeping and the byte-identical Go fixture relocation. No prior contract
or assertion was weakened. Current isolated final regression passed 2,979
tests, zero failures, two existing ignored; format, warnings-denied Clippy,
site and focused gates passed (16-FINAL-GATES.json). Independent GLM/high
review passed evidence/provenance/security and fixture identity. Production
fingerprints remained unchanged. This is hermetic, not live-provider evidence.
Bundled verification.fingerprint recomputed the covered digest afterward.

## Verdict

**UAT resolution:** On 2026-09-09 the user replied "sure pass" within verify-work after the recommendation to pass Phase 15's evidence-boundary review only. The consolidated judgment checkpoint is resolved. The historical human-needed assessment below is retained for provenance; canonical status is now passed. This does not admit any Go tuple or assert live/visual verification, and it does not waive Phase 16 release checks.

All 4 roadmap success criteria and all 13 consolidated plan truths are verified in the codebase and named behavioral evidence. The phase is **human-needed**, not failed: the plans deliberately classify 16 negative constraints as judgment-tier prohibitions, and the evidence ledger keeps OGO-02’s future captured/live promotion assumption unresolved. Those items require one consolidated maintainer judgment review; repeating the passing hermetic suite cannot resolve them.

## Observable truths

| # | Truth | Status | Evidence |
|---|---|---|---|
| 1 | Every candidate has a dated exact-model evidence record with destination, wire, headers, context, modalities, effort, tools, filtering, terminal, and sanitized capture/live fields. | VERIFIED | `15-EVIDENCE.md` contains the fenced `opencode-go-ledger` frame for all four exact IDs, pinned to revision `055c3ecf0de6c35f59195fc434d6b08525182b7f` and inspection date `2026-09-08`; `tests/opencode_go_evidence.rs::opencode_go_ledger` parses and checks every field. |
| 2 | Zero supported tuples is an explicit valid result; no source-only or ambiguous record can promote a tuple. | VERIFIED | Ledger asserts `admitted: []`, seven rejection classes are required, and mutation tests reject missing fields, false capture/live provenance, family-inferred wire, nonempty admission, and missing rejection accounting. OGO-02 remains visibly unresolved in `15-EDGE-COVERAGE.json`. |
| 3 | Any future accepted tuple is constrained to the matching existing wire contract and canonical-only opaque session behavior; today’s empty set emits no session or credential. | VERIFIED | `ProviderKind::OpenCodeGo` maps only to `AdapterKind::OpenAiChat`; no session producer or dynamic catalog exists. English/locale docs and `docs/opencode-go-evidence-gate.md` state the conditional `x-opencode-session` contract and empty-set behavior. This is vacuous for currently accepted tuples because the allowlist is empty. |
| 4 | Unknown, family-inferred, ambiguous, or failed selections reject before credential lookup or dispatch. | VERIFIED | `enforce_opencode_go_admission` keys exclusively on explicit provider kind, preserves generic primaries, removes later Go fallbacks, and is called before auth/dispatch in `failover.rs` and `codex_endpoint.rs`. Crate-local router tests use injected resolver/client counters and mutation evidence; exact-native rejection remains independently enforced. |
| 5 | Identity is additive and exact: one enum variant, canonical ordered preset, API-key auth/env, no extra config field. | VERIFIED | `src/config.rs`, `src/config/presets.rs`, routing conversion, and negative identity fixtures implement and exercise `opencode_go`, `https://opencode.ai/zen/go/v1`, `AuthMode::ApiKey`, and `SHUNT_OPENCODE_GO_API_KEY`. |
| 6 | Router boundaries prove Go primary rejection, fallback filtering, count-tokens handling, native pinned/unknown rejection, and generic preservation. | VERIFIED | `src/proxy/opencode_go_tests.rs` drives real router POSTs and generic positive controls; focused boundary run recorded 8/8 passing (including five real boundary tests), with gate-mutation failures demonstrating assertions are not vacuous. |
| 7 | English documentation and navigation expose only the zero-support contract. | VERIFIED | Per-file `tests/opencode_go_docs.rs` assertions cover README, provider, configuration, engineering note, and `site/src/lib/i18n.ts`; summary records 6/6 passing plus token mutation failure. |
| 8 | Maintained ko/ja/zh-CN README/provider/configuration mirrors preserve exact keys, URL, candidates, zero-support, and native-anchor rules. | VERIFIED | Nine locale functions assert their assigned file (15/15 full binary, with isolated mutation failure); no invented English fragment anchors are accepted. |
| 9 | Four provider overview guides are synchronized and independently asserted. | VERIFIED | Four guide-specific functions read only their own file; full docs binary recorded 19/19 passing and Japanese-token mutation failure. |
| 10 | The dedicated CLI smoke proves real post-boot rejection, not missing-credential startup failure. | VERIFIED | `tests/check_cli.rs::opencode_go_cli_negative` sets a synthetic key, starts the owned binary on a fresh non-10100 home/port, sends unary and streaming `/v1/messages`, asserts Anthropic 400 unsupported errors, checks proxy sentinel silence, and cleans the child/home. |
| 11 | Release gates and focused commands are executable, nonzero-selecting, isolated, and warnings-denied. | VERIFIED | `15-VALIDATION.md` records separate wrapper invocations for fmt, warnings-denied Clippy/workspace, site build, and every focused test; observed results include 2,972 passed/0 failed/2 existing ignored, 173 built pages, and all focused selections nonzero. |
| 12 | Security and state boundaries hold without credential writeback or production-state access. | VERIFIED | `15-SECURITY.md` closes all 13 authored threats; summaries record unchanged production config fingerprints/backups for every wrapper invocation, no credential inspection/writeback, no live provider call, and no wiki edits. |
| 13 | Phase decisions are represented in shipped artifacts. | VERIFIED | GSD decision-coverage check returned `honored: 11`, `total: 11`, `not_honored: []`. |

No truth is FAILED or behavior-unverified. The plain-HTTP loopback fixture is not TLS proof by itself; the test quality conclusion therefore relies on the injected credential counter, request outcomes, DNS pinning, and the separate CLI proxy sentinel, as documented in `15-RUNTIME-REVIEW.md` and `15-VALIDATION.md`.

## Artifact and wiring checks

- The empty allowlist is substantive and wired at both standard failover and native Responses boundaries; it is not a URL, bearer, nickname, family, or host-model heuristic.
- The canonical preset is validated at config load and uses the existing OpenAI Chat adapter without adding a dormant Go wire implementation.
- Ledger, docs, locale, guide, and CLI tests are active and make value/behavioral assertions. No disabled requirement-linked tests or circular expected-value generators were found in the reviewed files.
- Existing planning state files (`.planning/config.json`, `.planning/state.json`, `.gsd/`, and milestone lock) remain unrelated working-tree dirt and were not modified by this verification.

## Test-quality audit

| Test area | Active evidence | Disabled/circular | Assertion quality | Verdict |
|---|---:|---|---|---|
| Runtime admission/router | 8 focused tests (including real POST boundaries) | None found | Behavioral/status plus injected counter assertions | PASS, with documented plain-HTTP TLS limitation |
| Evidence ledger | 2 integration tests | None found | Exact values, provenance, mutation rejection | PASS |
| Docs/locales/guides | 19 integration tests | None found | Per-file token/value and forbidden-claim assertions | PASS |
| CLI smoke | 1 dedicated process test | None found | End-to-end status, body shape, cleanup, sentinel | PASS |

The HTTP fixture counter cannot independently prove that a leaked TLS attempt did not reach a real TLS server. This is a documented evidence limitation, not a failed truth: resolver counters, rejection status, DNS pinning, and the CLI sentinel provide the discriminating pre-credential/zero-egress checks.

## Judgment checkpoint (required before phase closure)

One maintainer review should explicitly resolve (or acknowledge as intentionally deferred) the 16 judgment-tier prohibitions across plans 15-01 through 15-06: crate-private-only test seams; no inference or generic behavior changes; no writeback; source/capture/live separation; unresolved OGO-02 edge; no unsupported positive claims/session producers; untouched wiki and unrelated provider rows; isolated state/fingerprint discipline; dedicated smoke identity; separate wrapper invocations; locale parity and overview non-promotion. The review must also acknowledge that no live/captured support or GUI/Computer acceptance is claimed. No additional automated rerun is necessary for this checkpoint.

## Decision coverage

All 11 trackable CONTEXT.md decisions are honored.


## Post-verification audit notes

**unverified-prohibition — human review recommended:** all 16 judgment-tier prohibitions retain their unresolved disposition. The model's favorable assessment above is non-authoritative, not human sign-off. No override was applied.

The bounded 600-second GSD regression rerun exited 0; production config mtime/SHA and backup inventory remained unchanged. The code-only UI audit reports cosmetic wording/nav-label warnings and explicitly unverified rendered layout. It does not add an implementation blocker or claim visual acceptance. These observations are retained in 15-UI-REVIEW.md and the consolidated 15-UAT.md checkpoint.

Decision gate output: `All trackable CONTEXT.md decisions are honored by shipped artifacts.` (11/11; warning-only gate.)

## Transition reconciliation

After the passing UAT/completion predicate, phase.complete changed only the covered ROADMAP completion markers and REQUIREMENTS checkboxes/status cells (diff inspected). No code, tests, plans, or evidence changed. The fingerprint was regenerated through verification.fingerprint to account for those bookkeeping edits, preserving the verified behavioral scope and original automated test evidence. Phase 16 remains incomplete.
