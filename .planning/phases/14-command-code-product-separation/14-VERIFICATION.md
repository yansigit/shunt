---
phase: 14-command-code-product-separation
verified: 2026-09-09T03:52:11Z
status: passed
score: 12/12 consolidated must-haves verified
behavior_unverified: 0
overrides_applied: 0
human_verification: []
covered_files:
  - .planning/REQUIREMENTS.md
  - .planning/phases/14-command-code-product-separation/14-01-PLAN.md
  - .planning/phases/14-command-code-product-separation/14-01-SUMMARY.md
  - .planning/phases/14-command-code-product-separation/14-02-PLAN.md
  - .planning/phases/14-command-code-product-separation/14-02-SUMMARY.md
  - .planning/phases/14-command-code-product-separation/14-03-PLAN.md
  - .planning/phases/14-command-code-product-separation/14-03-SUMMARY.md
  - .planning/phases/14-command-code-product-separation/14-04-PLAN.md
  - .planning/phases/14-command-code-product-separation/14-04-SUMMARY.md
  - .planning/phases/14-command-code-product-separation/14-05-PLAN.md
  - .planning/phases/14-command-code-product-separation/14-05-SUMMARY.md
  - .planning/phases/14-command-code-product-separation/14-06-PLAN.md
  - .planning/phases/14-command-code-product-separation/14-06-SUMMARY.md
  - .planning/phases/14-command-code-product-separation/14-CONTEXT.md
  - .planning/phases/14-command-code-product-separation/14-PLAN-REVIEW.md
  - .planning/phases/14-command-code-product-separation/14-RESEARCH.md
  - .planning/phases/14-command-code-product-separation/14-REVIEW.md
  - .planning/phases/14-command-code-product-separation/14-SECURITY.md
  - .planning/phases/14-command-code-product-separation/14-VALIDATION.md
  - README.ja.md
  - README.ko.md
  - README.md
  - README.zh-CN.md
  - THIRD-PARTY-NOTICES.md
  - docs/m16-command-code.md
  - site/src/content/docs/guides/providers.mdx
  - site/src/content/docs/ja/guides/providers.mdx
  - site/src/content/docs/ja/providers/command-code.md
  - site/src/content/docs/ja/providers/openai-chat.md
  - site/src/content/docs/ja/reference/configuration.md
  - site/src/content/docs/ko/guides/providers.mdx
  - site/src/content/docs/ko/providers/command-code.md
  - site/src/content/docs/ko/providers/openai-chat.md
  - site/src/content/docs/ko/reference/configuration.md
  - site/src/content/docs/providers/command-code.md
  - site/src/content/docs/providers/openai-chat.md
  - site/src/content/docs/reference/configuration.md
  - site/src/content/docs/zh-cn/guides/providers.mdx
  - site/src/content/docs/zh-cn/providers/command-code.md
  - site/src/content/docs/zh-cn/providers/openai-chat.md
  - site/src/content/docs/zh-cn/reference/configuration.md
  - site/src/lib/i18n.ts
  - src/adapters/command_code/efforts.rs
  - src/adapters/command_code/history.rs
  - src/adapters/command_code/mod.rs
  - src/adapters/command_code/ndjson.rs
  - src/adapters/command_code/relay.rs
  - src/adapters/command_code/request.rs
  - src/adapters/command_code/router_tests.rs
  - src/adapters/command_code/router_tests/lifetime.rs
  - src/adapters/command_code/router_tests/matrix.rs
  - src/adapters/command_code/router_tests/products.rs
  - src/adapters/command_code/router_tests/replay.rs
  - src/adapters/mod.rs
  - src/adapters/responses/inbound.rs
  - src/adapters/responses/request.rs
  - src/adapters/responses/websocket.rs
  - src/auth/command_code.rs
  - src/auth/command_code/tests/matrix.rs
  - src/auth/mod.rs
  - src/auth/slots/tests.rs
  - src/config.rs
  - src/config/presets.rs
  - src/config/upstreams.rs
  - src/init.rs
  - src/model/command_code_response.rs
  - src/model/command_code_response/validation.rs
  - src/model/mod.rs
  - src/proxy/capability.rs
  - src/proxy/failover.rs
  - src/routing.rs
  - tests/command_code_api_conformance.rs
  - tests/command_code_conformance.rs
  - tests/command_code_translate.rs
  - tests/command_code_translate/response.rs
covered_digest: v1:sha256:81397d7a7afaf31ed3fca3dfd6363781fc2419356647d2e5fdb2b674e735c956
---

# Phase 14: Command Code Product Separation verification

## Phase 15 re-verification (2026-09-09)

Independent source audit reviewed the OpenCode Go additions in shared config, presets, routing, capability filtering, and failover. They add a distinct API-key identity with an empty evidence allowlist; Command Code API-key/subscription dispatch, credentials, history, terminal grammar, retry, and cancellation paths are untouched. Root executed `node /tmp/shunt-phase12-isolated-run.cjs env RUSTFLAGS=-Dwarnings cargo test --quiet --all-features --workspace` at `eb60174`: 2,972 passed, 0 failed, 2 existing ignored. No Phase 14 truth, artifact, key link, or negative boundary regressed. Production OpenCodex config mtime/SHA and backup/invalid inventory were unchanged.

Goal: API-key and subscription products preserve separate transport, credentials,
session and terminal semantics. Root goal-backward verification; independent
GLM/high code review is recorded separately in 14-REVIEW.md.

## Observable truths and requirements

The four roadmap criteria and all 25 plan truths consolidate into the 12
outcomes below. Implementation evidence was inspected, not inferred from files
existing or from SUMMARY claims.

| # | Truth / requirements | Status | Implementation and exercised evidence |
| --- | --- | --- | --- |
| 1 | Distinct API preset and canonical default (CCK-01) | VERIFIED | presets.rs commandcode uses OpenAiChat/ApiKey; API conformance captures /provider/v1/chat/completions and preserves siblings. No duplicate Chat compiler. |
| 2 | Products cannot cross credentials or endpoints (CCK-02) | VERIFIED | Config pairing is exclusive; resolver dispatch is typed. products.rs keeps both backends in flight through one gateway and asserts distinct bearer, model, path, envelope and subscription-only headers. Fresh one-test execution passed. |
| 3 | API scenarios (CCK-03) | VERIFIED | command_code_api_conformance covers real unary, SSE, tools, request bounds, auth errors and incremental downstream drop/capacity reuse. Shared Chat conformance: 47 active tests freshly passed. |
| 4 | Read-only strict credential sources (CCS-01) | VERIFIED | resolve_sources permits file fallback only for NotPresent, not empty/invalid env. O_RDONLY/nonblocking regular-file check, 16 KiB cap, bounded wait, no refresh/writeback. Fixture matrix asserts bytes/SHA/mtime/inventory unchanged. |
| 5 | Canonical subscription egress (CCS-02) | VERIFIED | validate_provider runs before lookup and dispatch_headers repeats it; POST uses fixed ENDPOINT, redirect-none client. Invalid destination lookup counters stay zero; redirect fixture observes no follow-up. |
| 6 | Exact request facts and private session (CCS-03) | VERIFIED | request.rs fixed workspace-free envelope and header allowlist; efforts.rs exact ten-row admission before auth; credential-scoped SHA-256 over explicit conversation, request-local UUID otherwise. Exact-value fixtures pass. |
| 7 | Authentic tool history (CCS-04) | VERIFIED | history.rs preserves adjacent call/result IDs, missing-result execution-unknown error text, orphan user carriers and result-image carriers; rejects duplicates/opaque/unsupported inputs. Positive continuation and 32-tool wire fixtures pass. |
| 8 | Ordered unary and streaming semantics (CCS-05) | VERIFIED | relay.rs feeds actual upstream bytes through one Decoder and CommandCodeMachine; new_streaming disables retained content. Early-delta acknowledgement precedes upstream terminal; unary twin and tool-first ordering tests pass. |
| 9 | Strict terminal grammar and malformed failures (CCS-06) | VERIFIED | First finish is authoritative, one compatible finish-step-to-finish companion allowed; final success waits for framed EOF. Sticky failures reject empty/null/junk, invalid UTF-8/JSON, late/duplicate/conflicting terminals and incomplete tools. |
| 10 | Safe retry and cancellation (CCS-07) | VERIFIED | ConnectOnly request retains headers/token/session across pre-connect retry; post-send and output/tool cases stay single-attempt. All four fresh pre-header/mid-body cancellation tests pass and prove upstream closure plus capacity reuse. |
| 11 | Full scenario/boundary matrix (CCS-08) | VERIFIED | Named wire/record/residual/semantic/argument/tool/block limits have boundary triples. Usage uses checked integers/cache subtraction; partial-byte drip cannot reset record deadline. Fresh 23-test translation suite and full workspace pass. |
| 12 | Provenance, docs and safety gates | VERIFIED | Pinned source ledger separates hardening from live uncertainty; full MIT notice retained. READMEs/provider/guide/reference en/ko/ja/zh-cn updated; build and built-page parity pass. Isolation wrapper preserves production state; no generated wiki changes. |

## Plan coverage

| Plan | Truth count | Outcomes |
| --- | --- | --- |
| 14-01 | 3/3 | 1–3 |
| 14-02 | 4/4 | 4, 5, 8, 9, 12 |
| 14-03 | 5/5 | 6, 7 |
| 14-04 | 5/5 | 8, 9, 11 |
| 14-05 | 4/4 | 4, 5, 10 |
| 14-06 | 4/4 | 2, 3, 7, 11, 12 |

Plan02's absent-env failure describes its intentionally env-only intermediate
slice; plan05 explicitly implements the final approved read-only file fallback.
Timeout means a five-second bound on waiting, not forcible OS read cancellation.
No raw-spelling finish synonym compatibility is inferred beyond pinned grammar.

## Artifacts, wiring and data flow

GSD artifact checks passed 27/27 entries and key-link checks 9/9. Root traced
config/upstreams/presets → routing → failover dispatch → request validation →
typed credential resolution → allowlist headers → canonical POST → bounded
decoder → shared machine → actual JSON/SSE body. The static matcher is not the
only proof: socket fixtures observe the real requests, content and cleanup.
Header-site allowlisting names the restricted new producer, not caller maps.

All CCK-01..03 and CCS-01..08 are declared in plans and REQUIREMENTS, with
behavioral/value tests. No orphan requirement. No required test is disabled,
circular, existence-only or a generated expected value from this implementation.
Named protocol fixtures are synthetic and source-derived, not live captures.

## Verification observed

- Fresh product-isolation check: 1 passed; fresh cancellation group: 4 passed;
  fresh command_code_translate suite: 23 passed. Nonzero selection in each.
- Formatter, all-target/all-feature Clippy with warnings denied, and build pass.
- Full workspace regression ran through the isolated wrapper and GSD's
  600-second one-shot gate: 2,942 passed, 0 failed, 2 pre-existing ignored.
- Site build repeated after review docs clarifications: 169 pages, four languages.
- Owned API CLI/curl smoke passed unary/SSE/path/model/key/invalid-model checks;
  subscription explicit-empty-token smoke returned 401 and zero proxy traffic.
- Every stateful process tree used fresh OPENCODEX_HOME; production config
  mtime/SHA and invalid/backup inventory unchanged. Owned smoke ports stopped.
- Decision coverage: all 14/14 trackable decisions honored.

## Security, review and process

21/21 authored ASVS L1 threats have checked mitigations (14-SECURITY.md).
Nyquist maps all 11 requirements and 18 task groups to actual green checks.
No untracked TBD/FIXME/XXX or ignored requirement test in the new runtime and
conformance files. Independent review found no blockers; both compatibility
documentation warnings were addressed in every locale, without new semantics.

Plan05's initial failed compilation was not behavioral RED. That process gap
remains disclosed; no retrospective RED pass is manufactured. Fresh GSD init
resolves tdd_mode:false; no gate or user setting was changed to achieve this.

## Prohibition dispositions

- 01: Product choice is explicit in preset/kind/auth; concurrent fixture proves
  no bearer inference or crossover.
- 02/05: No auth writes, refresh, rotation or copying; file fixtures unchanged.
  Production rejects off-origin subscription configuration; TLS/DNS seams are
  cfg(test)-only. The planned later fallback supersedes the intermediate no-file slice.
- 03: Constant config and exact headers exclude workspace/git/project slug;
  session hashing is length-delimited and credential scoped, no prompt fallback;
  table admission rejects aliases/clamping and exposes no version override.
- 04: Checked terminal and malformed fixtures prove no EOF success/repair/junk tolerance.
- 06: All test traffic is isolated; required read-only production fingerprint
  snapshots are safety checks, not config parsing/test execution against the live home.
  Wiki unchanged and live/GUI acceptance not claimed.

## Human verification and explicitly deferred evidence

No additional manual step is needed for this phase's declared hermetic contract.
Phase16 explicitly owns bounded opt-in live acceptance, including minimal
envelope/client version and actual companion reason spellings. Computer
evaluation remains blocked, never passed; static documentation checks are not
visual evaluation. There is no UI-SPEC, so the UI review hook has no input.

## Disconfirmation

Audit found the original API fixture did not prove two products concurrently:
products.rs now supplies the missing same-gateway barrier and exact wire assertions.
Checked potential orphan code through actual dispatch/relay wiring, and checked
terminal-after-error and cancellation paths through behavioral fixtures. Nothing
is admitted as live-compatible merely because its hermetic tests pass.

**Result:** Phase14's implementation goal is met within the declared source-derived,
hermetic scope. The milestone and live-provider release gate are not complete.
