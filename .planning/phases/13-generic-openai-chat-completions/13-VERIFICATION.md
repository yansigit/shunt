---
phase: 13-generic-openai-chat-completions
verified: 2026-09-08T20:05:39Z
status: passed
score: 12/12 consolidated must-haves verified
behavior_unverified: 0
overrides_applied: 0
human_verification: []
covered_files:
  - .planning/REQUIREMENTS.md
  - .planning/phases/13-generic-openai-chat-completions/13-01-PLAN.md
  - .planning/phases/13-generic-openai-chat-completions/13-01-SUMMARY.md
  - .planning/phases/13-generic-openai-chat-completions/13-02-PLAN.md
  - .planning/phases/13-generic-openai-chat-completions/13-02-SUMMARY.md
  - .planning/phases/13-generic-openai-chat-completions/13-03-PLAN.md
  - .planning/phases/13-generic-openai-chat-completions/13-03-SUMMARY.md
  - .planning/phases/13-generic-openai-chat-completions/13-04-PLAN.md
  - .planning/phases/13-generic-openai-chat-completions/13-04-SUMMARY.md
  - .planning/phases/13-generic-openai-chat-completions/13-05-PLAN.md
  - .planning/phases/13-generic-openai-chat-completions/13-05-SUMMARY.md
  - README.ja.md
  - README.ko.md
  - README.md
  - README.zh-CN.md
  - docs/openai-chat-translation.md
  - site/src/content/docs/guides/providers.mdx
  - site/src/content/docs/ja/guides/providers.mdx
  - site/src/content/docs/ja/providers/openai-chat.md
  - site/src/content/docs/ja/reference/configuration.md
  - site/src/content/docs/ko/guides/providers.mdx
  - site/src/content/docs/ko/providers/openai-chat.md
  - site/src/content/docs/ko/reference/configuration.md
  - site/src/content/docs/providers/openai-chat.md
  - site/src/content/docs/reference/configuration.md
  - site/src/content/docs/zh-cn/guides/providers.mdx
  - site/src/content/docs/zh-cn/providers/openai-chat.md
  - site/src/content/docs/zh-cn/reference/configuration.md
  - site/src/lib/i18n.ts
  - src/adapters/mod.rs
  - src/adapters/openai_chat/diagnostic_tests.rs
  - src/adapters/openai_chat/mod.rs
  - src/adapters/openai_chat/sse.rs
  - src/adapters/openai_chat/timeout_tests.rs
  - src/auth/slots/tests.rs
  - src/config.rs
  - src/model/mod.rs
  - src/model/openai_chat_request.rs
  - src/model/openai_chat_request/endpoint.rs
  - src/model/openai_chat_request/tools.rs
  - src/model/openai_chat_response.rs
  - src/model/openai_chat_response/assembly.rs
  - src/model/openai_chat_response/checked.rs
  - src/model/openai_chat_response/unary_tools.rs
  - src/model/openai_chat_response/validation.rs
  - src/proxy/capability.rs
  - src/proxy/failover.rs
  - src/routing.rs
  - tests/failover.rs
  - tests/openai_chat_conformance.rs
  - tests/openai_chat_conformance/lifetime.rs
  - tests/openai_chat_conformance/matrix.rs
  - tests/openai_chat_translate.rs
  - tests/openai_chat_translate/assembly.rs
  - tests/openai_chat_translate/caps.rs
  - tests/retry.rs
covered_digest: "v1:sha256:0357844c326f6316dbcc983f992733cd76e49055a9e1be1eaebc06505f34e0c8"
decision_coverage:
  honored: 11
  total: 11
  not_honored: []
---

# Phase 13: Generic OpenAI Chat Completions verification

## Phase 14 regression re-verification (2026-09-08)

Root inline verification; no independent verifier pass is claimed.
Compared covered files to eaf0b91. Chat compiler, response machine, adapter and
conformance fixtures are unchanged. New routing, capability, config and counting
arms are limited to Command Code; Chat's API-key guard and estimator remain.
Header-site test allowlist adds only the independently reviewed allowlist-built
subscription producer, not a caller-header forwarding exemption. Docs replace
obsolete no-preset wording with the Command Code API preset in all locales.
All 47 active Chat conformance tests passed, including cancellation, byte caps,
credential isolation, exact body/tool history, malformed input and terminals.
Decision coverage remains 11/11.

Every test used node /tmp/shunt-phase12-isolated-run.cjs with all-features and
nonzero selection. Production config mtime/SHA and backup inventory remained
unchanged. The preceding observed full workspace gate passed 2,942 tests with
0 failures and 2 existing ignored tests; site build passed 169 pages in four
languages. No disabled requirement tests were found in the rechecked groups.
Fingerprint was regenerated through the bundled GSD tool after code review and
successful execution. Historical results below are historical. Live/Computer
acceptance remains outside this hermetic re-verification.


**Goal:** Add a bounded Chat transport with Anthropic request/response, tool,
image, streaming and error translation, without weakening credential or replay safety.
**Scope:** Initial goal-backward verification by root. Independent GLM/high code
review is recorded separately in 13-REVIEW.md; this report does not claim a
second independent verifier. All production testing was isolated.

## Observable truths

The four roadmap success criteria and all 25 plan truths consolidate into these
12 outcomes. Plan coverage below retains the full contract, not just existence.

| # | Truth / requirement | Status | Behavioral evidence and actual wiring |
|---|---|---|---|
| 1 | Independent opt-in Chat provider (CHAT-01) | VERIFIED | Config ProviderKind and routing AdapterKind map to OpenAiChatAdapter in proxy/failover.rs; real-router unary/streaming tracers and concurrent isolation pass. Existing OpenAI Responses defaults remain unchanged. |
| 2 | Deterministic exact endpoint grammar (CHAT-02) | VERIFIED | config.rs and adapter use the shared endpoint builder; endpoint matrices and boot rejection fixtures cover roots/full endpoints, query, fragment, userinfo, malformed paths and repeated construction. CLI bad-query config fails. |
| 3 | Strict text/image/model/control translation (CHAT-03) | VERIFIED | Request whitelist and typed errors execute before the send; exact-body fixture plus pure Unicode, empty/null, image URL/base64 and controls cases pass. Unknown fields, metadata and malformed UTF-8 have zero-dispatch fixtures. |
| 4 | Authentic declared tools and paired results (CHAT-04) | VERIFIED | request/tools.rs uses request-local identity tracking; parallel pairing and tool-heavy wire round trip pass, with orphan/duplicate/missing/incomplete inputs rejected. |
| 5 | Bounded unary semantics and diagnostics (CHAT-05) | VERIFIED | checked.rs validates choice/payload/usage; unary_tools validates identities and object arguments; shared machine produces text, thinking, tools, usage and supported stop reasons. Empty/null, fractional/overflow counters, embedded errors and safe header-only request IDs are exercised. |
| 6 | Incremental text/reasoning and indexed tools (CHAT-06) | VERIFIED | adapter SSE loop feeds the shared machine; ToolAssembly applies deltas and finishes once at provider boundary. Interleaving/deferred identity/concurrent assembly tests pass; explicit tool-first message_start ordering passes. |
| 7 | One trustworthy terminal; malformed/truncated data fail (CHAT-07) | VERIFIED | Decoder bounds data and residual bytes; machine requires supported finish plus DONE, rejects duplicate terminal/current-batch residual/malformed UTF-8 or JSON, and does not retract emitted text. Dedicated EOF, usage-only position and error-after-text fixtures pass. |
| 8 | Configured key bound to its destination (CHAT-08) | VERIFIED | ApiKey-only config, per-request env resolution, inbound credential slot filtering and redirect-disabled client. Concurrent key fixture and redirect target zero-request assertions pass. CLI captures confirm only fixture key is forwarded. |
| 9 | Complete scenarios and exact named bounds (CHAT-09) | VERIFIED | 12 AI-SPEC scenarios mapped in 13-05-EVIDENCE.md; 47 conformance and 116 translation cases pass. Event/residual/aggregate/unary/tool count/tool args have explicit below/at/above probes, including UTF-8 residual accounting. |
| 10 | Cancellation releases upstream and admission | VERIFIED | Five lifetime.rs fixtures await backend readiness, prove a second request is blocked while capacity is held, disconnect, observe upstream EOF, admit a later successful request, and join the backend task within deadlines. No promise about already-in-flight bytes. |
| 11 | No unsafe generation redispatch; honest fallback eligibility | VERIFIED | ConnectOnly send policy plus failure=None after upstream statuses/post-send timeout; retry/failover positive pre-connect and negative post-send controls pass. Capability regression failed before fix and passes afterward, retaining primary and excluding only incompatible Chat fallbacks. |
| 12 | Complete docs and isolated release checks | VERIFIED | Four READMEs/provider pages/config references/provider guides and English engineering note agree with the supported subset. Node docs check, CLI example, 165-page build and four generated routes pass. Wiki unchanged; full warnings-denied suite 2,875 passed/0 failed/2 existing ignored. |

### Plan truth coverage

| Plan | Truths covered | Consolidated outcomes |
|---|---|---|
| 13-01 | 5/5: unary tracer, SSE tracer/EOF, retry/redirect, request isolation, Anthropic errors | 1, 7, 8, 11 |
| 13-02 | 5/5: empty/error admission, Unicode bytes, purity, endpoint grammar, tool pairing | 2, 3, 4 |
| 13-03 | 6/6: unary precision, empty choices, reasoning extensions/order, finish mapping, errors, terminal/current-batch semantics | 5, 6, 7 |
| 13-04 | 5/5: deferred indexed assembly, concurrency/order, tool bounds, credential destination, ConnectOnly classification | 6, 8, 9, 11 |
| 13-05 | 4/4: scenario breadth, cancellation, byte triples, all maintained docs | 9, 10, 12 |

All CHAT-01 through CHAT-09 appear in the plans and REQUIREMENTS.md; no orphaned
phase requirement was found. This covers each roadmap criterion: independent
configuration; request/tool fidelity; shared unary/stream semantics; and
hermetic safety/scenario coverage.

## Artifacts, links and data flow

GSD artifact checks passed 25/25 entries across the five plans; key links passed
7/7. The 13-04 scanner pattern was corrected from an anticipated spelling to
the actual `tool_assembly` member after checking its apply/finish calls.
Pattern checks are supplemented by real-router tests: decoded inbound JSON
flows through request translation into the captured mock request, and actual
upstream JSON/SSE flows through the semantic machine to asserted client values.
No hardcoded production response, orphan module, or placeholder was found.

## Behavioral spot-checks

After the final full-suite run, root independently reran these exact named
conformance cases with the isolation wrapper and warnings-denied Cargo flags:

| Case | Result |
|---|---|
| openai_chat_assembly_interleave_at_limit_multibyte | 1 passed, 46 filtered, 0.36s |
| lifetime::openai_chat_cancel_before_headers_streaming | 1 passed, 46 filtered, 0.01s |
| openai_chat_terminal_error_after_text_single_terminal | 1 passed, 46 filtered, 0.00s |

The full workspace regression run also includes earlier provider, auth,
inbound protocol, CLI, Responses, passthrough and failover suites. Every test
tree used fresh OPENCODEX_HOME and a non-10100 port; before/after production
config mtime/SHA and backup inventory checks passed.

## Test quality and prohibitions

Chat requirement-linked tests contain no ignore/skip markers, no circular
generation of expected fixtures, and no existence-only substitute for the
required values and transitions. Expected wire bodies and outcomes are authored
synthetic protocol fixtures, not evidence of live subscription availability.
Two ignored workspace tests are pre-existing and unrelated.

The plans' initially flagged prohibitions now have concrete enforcement:
zero-target redirect and exactly-one post-send request tests (13-01/04);
whitelist, Unicode and endpoint negatives (13-02); EOF/no-retraction/usage-position
negatives (13-03); deferred tool-argument parsing tests (13-04); empty wiki diff,
fragment-free new locale page assertions, and explicit synthetic-only evidence
in the docs/verification records (13-05). No prohibition was silently discarded.

## Review and residual scope

Independent code review CR-01 was not reproducible: delta tools are buffered
until finish, which starts the message before emitting blocks. Root added an
ordering assertion instead of the suggested unnecessary production change.
The separate capability gap was reproduced RED, gated, fixed, and tested.
The original review and its root adjudication remain intact. Token counting
is intentionally estimated and read-idle is fixed at 120 seconds; both are
documented rather than changed into unapproved public settings.

Decision coverage: 11/11 honored. Schema and UI safety checks did not block.
The stale codebase map remains an advisory, not a disabled blocking gate.
The earlier 13-04 evidence-persistence timing deviation remains disclosed.

## Human verification and deferred work

No additional manual action is required for this phase's hermetic protocol
contract. Computer UI remains blocked and is **not** marked passed. Live
provider/subscription smokes and release-wide audit belong to Phase 16;
Command Code separation and exact OpenCode Go evidence remain Phases 14–15.
This phase does not establish live availability and does not complete the milestone.
