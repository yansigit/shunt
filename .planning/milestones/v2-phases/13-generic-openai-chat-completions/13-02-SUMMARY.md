---
phase: 13-generic-openai-chat-completions
plan: "02"
subsystem: api
tags: [openai, chat-completions, request-translation, whitelist, tools, endpoint-grammar, tdd]

requires:
  - phase: 13-01-tracer-slice
    provides: "OpenAiChat adapter skeleton, translate_request entry point, ProviderKind::OpenAiChat config plumbing, conformance harness with mock Chat upstream"
provides:
  - "Deny-by-default full whitelist request translation: roles, text/image content, generation controls (temperature/top_p/stop_sequences/max_tokens/stream) with exact outbound-body assertions"
  - "UTF-8 byte budgets (MAX_TEXT_BLOCK_BYTES = 8 MiB named const) with checked arithmetic and byte-preserving Unicode (CJK) translation"
  - "Thinking policy: plaintext assistant thinking maps to reasoning_content; signed/redacted thinking strictly rejected"
  - "Tools: declarations, tool_choice (auto/any/by-name), parallel tool_calls, paired role-tool results with a request-local id registry and typed 400 rejections (orphan, duplicate, missing identity, non-object arguments)"
  - "Shared deterministic chat_completions_endpoint grammar (single append, trailing-slash normalization, no doubling) enforced at boot for OpenAiChat base URLs (reject query/fragment/userinfo/non-http scheme) before credential lookup"
  - "Pure tests/openai_chat_translate.rs suite (58 tests after root review) plus router-level conformance coverage (19 tests incl. boot rejection)"
affects: [13-03-response-expansion, 13-04-arg-byte-budget, 13-05-docs]

actuals:
  tokens: 16625
  tasks: 3
  commits: 7

tech-stack:
  added: []
  patterns:
    - "Boot validation and request construction share one pure URL grammar function, so config and dispatch cannot disagree about the endpoint"
    - "Exactly-one-append endpoint builder with explicit rejection of ambiguous URL components instead of speculative repair"
    - "Request-local tool id registry inside translate_request: purity under concurrency is behavioral, not assumed"

key-files:
  created:
    - tests/openai_chat_translate.rs
    - src/model/openai_chat_request/endpoint.rs
    - src/model/openai_chat_request/tools.rs
  modified:
    - src/model/openai_chat_request.rs
    - src/config.rs
    - src/adapters/openai_chat/mod.rs
    - tests/openai_chat_translate.rs
    - tests/openai_chat_conformance.rs

key-decisions:
  - "metadata is rejected with a typed 400 despite the allowlist wording ambiguity; no outbound unknown keys, ever."
  - "Plaintext assistant thinking maps to the reasoning_content extension field per 13-PROTOCOL-EVIDENCE.md; signed/redacted thinking cannot be forwarded losslessly and is strictly rejected."
  - "MAX_TEXT_BLOCK_BYTES is a single named 8 MiB const enforced per text payload with plain byte counts before dispatch."
  - "Text-only content blocks collapse to plain strings; tool-only assistant messages emit content: null; tool_result-only user messages emit only role-tool messages."
  - "Endpoint grammar returns an Err(String) consumed by both Config::validate (ProviderBaseUrl family) and the adapter (bad_request 400); rejection precedes any credential lookup in the boot loop."

patterns-established:
  - "RED evidence is recorded as raw libtest log + TAP adapter (stable libtest has no --format json) validated through gsd-tools check tdd-red-evidence; verdict RED_EVIDENCE_OK authorizes production edits."
  - "Fixture functions carry contiguous filter substrings (endpoint_, tool_, translate_) so the verify filter can never select zero tests."

requirements-completed: [CHAT-02, CHAT-03, CHAT-04]

coverage:
  - id: D1
    description: "Full whitelist translation of roles, text/image content, and generation controls with exact outbound-body assertions, byte budgets, and byte-preserving Unicode"
    requirement: CHAT-03
    verification:
      - kind: unit
        ref: "tests/openai_chat_translate.rs#translate_emits_exact_whitelisted_body"
        status: pass
      - kind: unit
        ref: "tests/openai_chat_translate.rs#translate_rejects_text_over_byte_budget"
        status: pass
      - kind: integration
        ref: "tests/openai_chat_conformance.rs#openai_chat_translate_wire_exact_body"
        status: pass
    human_judgment: false
  - id: D2
    description: "Unsupported-input table: unknown top-level fields and metadata yield typed 400s; unknown block types and signed/redacted thinking rejected with zero upstream requests"
    requirement: CHAT-03
    verification:
      - kind: unit
        ref: "tests/openai_chat_translate.rs#translate_rejects_unknown_top_level_field"
        status: pass
      - kind: unit
        ref: "tests/openai_chat_translate.rs#translate_rejects_metadata_field"
        status: pass
      - kind: integration
        ref: "tests/openai_chat_conformance.rs#openai_chat_translate_wire_rejects_metadata"
        status: pass
    human_judgment: false
  - id: D3
    description: "Tools/tool_choice/parallel tool_calls/paired tool results with stable identities and typed 400 rejections for orphaned, duplicate, missing-identity, and non-object arguments"
    requirement: CHAT-04
    verification:
      - kind: unit
        ref: "tests/openai_chat_translate.rs#tool_parallel_calls_with_paired_results"
        status: pass
      - kind: unit
        ref: "tests/openai_chat_translate.rs#tool_orphan_result_reject"
        status: pass
      - kind: integration
        ref: "tests/openai_chat_conformance.rs#openai_chat_translate_wire_tool_pairing"
        status: pass
    human_judgment: false
  - id: D4
    description: "Shared deterministic chat_completions_endpoint grammar: exactly one /chat/completions path, trailing-slash normalization, no doubling, and boot rejection of query/fragment/userinfo base URLs before credential lookup"
    requirement: CHAT-02
    verification:
      - kind: unit
        ref: "tests/openai_chat_translate.rs#endpoint_single_append"
        status: pass
      - kind: unit
        ref: "tests/openai_chat_translate.rs#endpoint_determinism"
        status: pass
      - kind: integration
        ref: "tests/openai_chat_conformance.rs#openai_chat_boot_rejects_query_string_in_base_url"
        status: pass
      - kind: integration
        ref: "tests/openai_chat_conformance.rs#openai_chat_boot_rejects_fragment_in_base_url"
        status: pass
      - kind: integration
        ref: "tests/openai_chat_conformance.rs#openai_chat_boot_rejects_userinfo_in_base_url"
        status: pass
    human_judgment: false

# Metrics
duration: ~100min
completed: 2026-09-08
status: complete
---

# Phase 13 Plan 02 Summary: Complete Request Translation and Endpoint Grammar

**Deny-by-default Chat request whitelist (text/image/controls/tools/thinking policy) with byte-exact outbound bodies, plus a shared deterministic /chat/completions endpoint grammar enforced at boot**

## Performance

- **Duration:** ~100 min
- **Started:** 2026-09-08T00:00-06:00 (approx.)
- **Completed:** 2026-09-08T01:00-06:00 (approx.)
- **Tasks:** 3 of 3
- **Files modified:** 5

## Accomplishments

- Full whitelist request translation: roles, text/image blocks, generation controls, and thinking policy all map with exact serialized outbound JSON proven by pure and wire fixtures; unknown fields and metadata fail closed with typed 400s.
- Tools survive translation with stable identities: declarations, tool_choice variants, parallel tool_calls, and paired role-tool results, with request-local id registry proving purity under concurrency and four malformed-input rejection classes.
- Shared chat_completions_endpoint grammar: exactly one path append, trailing-slash normalization, no doubling, determinism under repetition/threads, and boot-time rejection (query/fragment/userinfo/non-http) before any credential lookup — the same function used by the adapter at request time.

## Task Commits

Each task was committed atomically (test RED -> feat GREEN):

1. **Task 1: Role/text/image/generation whitelist** - `19a4a23` (test RED), `bf8c090` (feat)
2. **Task 2: Tools, tool_choice, parallel calls, paired results** - `c9de463` (test RED), `4e74414` (feat)
3. **Task 3: Endpoint grammar + boot rejection** - `f100be2` (test RED), `9667dd3` (test RED boot conformance), `b829902` (feat)

## TDD RED Evidence

All three task RED loops were recorded and validated via gsd-tools (verdict RED_EVIDENCE_OK, genuine assertion failures — compilation/fixture errors were never accepted as RED):

- Task 1: /tmp/gsd13-02-red-task1.log (22 fixtures, 15 failing)
- Task 2: /tmp/gsd13-02-red-task2.log (18 tool fixtures, 16 failing)
- Task 3: /tmp/gsd13-02-red-task3.log (8 endpoint fixtures, 4-5 failing); boot-rejection conformance RED recorded separately: /tmp/gsd13-02-red-task3b.log (3 fixtures, 3 failing — validate() accepted the bad URLs before the grammar gate landed)

## Files Created/Modified

- `src/model/openai_chat_request.rs` - complete translate_request whitelist, MAX_TEXT_BLOCK_BYTES budget, tool id registry, chat_completions_endpoint grammar, bad_request helper
- `src/config.rs` - OpenAiChat base-URL grammar validation at boot (ProviderBaseUrl family, before credential checks)
- `src/adapters/openai_chat/mod.rs` - replaced the 13-01 inline append with the shared grammar call
- `tests/openai_chat_translate.rs` - 47 pure tests (whitelist, tools, endpoint grammar, determinism)
- `tests/openai_chat_conformance.rs` - 19 router fixtures including boot rejection of query/fragment/userinfo base URLs

## Verification (via isolated wrapper, production config untouched)

- `cargo test --all-features --test openai_chat_translate`: 47/47 pass
- `cargo test --all-features --test openai_chat_conformance`: 19/19 pass
- `cargo test --all-features --workspace`: 2770 passed / 0 failed / 2 ignored across 28 test binaries, exit 0
- `cargo fmt --all --check`: exit 0
- `cargo clippy --all-targets --all-features -- -D warnings`: exit 0

## Decisions Made

- metadata rejected with typed 400 despite ambiguous allowlist wording (fail-closed over silent drop).
- Text-only blocks collapse to plain strings; tool-only assistant messages emit content:null; tool_result-only user messages emit only role-tool messages — minimal Chat shape per protocol evidence.
- Endpoint grammar lives in the model module (both consumers import it); boot loop runs the grammar check ahead of the MissingApiKeyEnv check so rejection precedes any credential lookup.
- Root review split endpoint grammar and tool-declaration/choice mapping into private helper modules. The main translator remains slightly over the 500-line preference; block pairing stays together to preserve its request-local ownership.

## Deviations from Plan

None blocking. Notes:

- Boot-rejection conformance fixtures were written in a second test commit (`9667dd3`) after the endpoint pure tests, keeping their RED evidence distinct; plan sequencing intent (RED before the config change) was preserved.
- Docs are deliberately deferred to plan 13-05 per the phase's task layout; no doc updates were made in this plan. **This plan is complete; phase closure is NOT claimed here.**

## Issues Encountered

- Two stray-comma syntax errors from an intermediate endpoint-function patch produced compile errors; fixed before any test run so they were never treated as RED evidence.
- Stable libtest has no --format json, so RED evidence used the established TAP-adapter conversion validated by gsd-tools.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Response-side expansion (13-03) can build on the outbound-body contract pinned by the exact-body fixtures.
- 13-04 can attach the tool-argument byte budget to the named-const pattern established for MAX_TEXT_BLOCK_BYTES.
- 13-05 must update README/site/docs surfaces for the OpenAI Chat provider before phase closure.

---
*Phase: 13-generic-openai-chat-completions*
*Completed: 2026-09-08*

## Root Review and Final Verification

The initial green suite missed eleven boundary regressions, each reproduced as an assertion failure before correction. RED commits: `b25695d`, `c30e857`, `67587c6`; fixes: `786accc`, `e2abb0c`, `2aec833`; helper extraction: `a68b1ee`.

- Standard object-form auto/any tool choices now map correctly, duplicate tool results fail locally, and tool-result messages precede follow-up user text.
- Combined text budgets are checked before allocation; reasoning fragments retain their exact bytes without inserted newlines. Thinking-only assistant history is preserved.
- Unsupported nested fields and malformed stream values fail rather than silently disappear; empty tool IDs/names fail. Unsupported tool-result error semantics are rejected explicitly.
- URL diagnostics no longer echo embedded userinfo. Whitespace, controls, backslashes, dot-segment repairs, missing authority, and empty userinfo are rejected instead of silently normalized.
- Final root verification on the corrected implementation: **2,781 passed, 0 failed, 2 ignored** across the full all-features workspace suite; **58 translation tests and 19 router tests** included. Build, format check, and strict all-targets/all-features Clippy passed. Every invocation used the isolated wrapper and confirmed production config mtime/SHA and backup inventory unchanged.
- Both blocking post-wave gates passed. The codebase-map gate remains a non-blocking stale-map advisory, not a code failure. No gate was disabled; no live-provider or Computer evaluation is claimed.

The earlier verification counts above describe the executor's initial result; this section supersedes them for acceptance. Documentation remains owned by 13-05 before phase closure, and tool-argument/resource-bound expansion remains in 13-04.
