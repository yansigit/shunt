---
phase: 13-generic-openai-chat-completions
plan: "03"
subsystem: api
tags: [openai, chat-completions, response-machine, reasoning, usage, error-surfacing, tdd]

requires:
  - phase: 13-02-request-expansion
    provides: "OpenAiChat request whitelist translation, endpoint grammar, OpenAiChatSseMachine skeleton with text-only streaming relay, conformance harness"
provides:
  - "Full finish-reason map: stop->end_turn, length->max_tokens, tool_calls->tool_use; content_filter/function_call/unknown fail closed with finish_reason named; finish_reason error becomes a provider terminal that can never close as success"
  - "Usage precision: non-negative integer counters bounded to i64 range; floats, strings, negatives, and 2^63 rejected; trailing usage-only chunk relays exact counters into message_delta"
  - "Reasoning extensions: reasoning_content/reasoning aliases (null/empty absent, non-string reject, differing-alias conflict reject), signed representations (signature/reasoning_signature/redacted_reasoning/encrypted_reasoning) rejected by field name, thinking blocks emitted before text with stable block indices in both unary and streaming modes"
  - "Provider error surfacing: embedded error objects (chunk-level, choice-level, finish_reason error, trailing position) produce exactly one neutral error terminal; only bounded x-request-id/request-id response headers are forwarded, never body metadata"
  - "Choices shape validation: absent, empty, and multiple choices fail closed in the main path; exactly one usage-only trailing chunk permitted after finish_reason and any non-empty delta payload there rejects"
  - "11 router-level openai_chat_terminal conformance fixtures pinning usage positioning, reasoning order, single-terminal errors, duplicate [DONE], residual-after-[DONE], malformed JSON, missing-finish EOF, and unary 200-error request-id relay"
affects: [13-04-arg-byte-budget, 13-05-docs]

actuals:
  tokens: unknown
  tasks: 2
  commits: 5

tech-stack:
  added: []
  patterns:
    - "One semantic machine owns the terminal decision for both unary JSON and streamed SSE; transports may only close a provider-declared success"
    - "Provider errors use neutral gateway-owned messages; the adapter adds only a validated response-header request ID"
    - "Strict trailing-chunk grammar: after finish_reason, only an empty-choices usage chunk is legal; any non-empty delta is payload regardless of finish_reason presence"

key-files:
  created:
    - .planning/phases/13-generic-openai-chat-completions/13-03-SUMMARY.md
  modified:
    - src/model/openai_chat_response.rs
    - src/adapters/openai_chat/mod.rs
    - tests/openai_chat_translate.rs
    - tests/openai_chat_conformance.rs

key-decisions:
  - "Reasoning is accepted only as plaintext via reasoning_content or its lone reasoning alias; the signed/redacted family is rejected with the offending field name in the error, because an opaque blob cannot be losslessly relayed as a thinking block."
  - "Root correction: only x-request-id/request-id response headers are honored, restricted to 1..=128 ASCII identifier bytes; body metadata is not a header allowlist and is dropped."
  - "Root correction: content:null produces no streaming delta, but a completed unary response with no content yields the required empty text block."
  - "finish_reason error maps to a provider terminal (error event, ProviderFailed) rather than a protocol error, so an upstream-declared failure can never be upgraded or downgraded by the transport."
  - "Trailing usage gating is positional, not content-based: exactly one usage-only chunk immediately after finish_reason; a second one, any usage before finish, or any non-empty delta there fails closed."

patterns-established:
  - "Fixture functions carry contiguous filter substrings (response_, openai_chat_terminal) so the plan verify filters can never select zero tests."
  - "RED gates recorded as raw libtest counts with the failing subset identified before production edits."

requirements-completed: [CHAT-05, CHAT-07]

coverage:
  - id: R1
    description: "Unary reasoning/thinking, text ordering, usage precision, finish map, empty text, envelope, and safe error checks"
    requirement: CHAT-05
    verification:
      - kind: unit
        ref: "tests/openai_chat_translate.rs#response_reasoning_unary_thinking_before_text"
        status: pass
      - kind: unit
        ref: "tests/openai_chat_translate.rs#response_finish_map_stop_length_tool_calls"
        status: pass
      - kind: unit
        ref: "tests/openai_chat_translate.rs#response_usage_precision_rejects_invalid_counters"
        status: pass
      - kind: integration
        ref: "tests/openai_chat_conformance.rs#openai_chat_terminal_unary_embedded_200_error_request_id"
        status: pass
    human_judgment: false
  - id: R2
    description: "Streaming reasoning/text ordering, error-after-text single terminal, and fail-closed framing (missing finish, duplicate DONE, residual, malformed JSON, usage positioning)"
    requirement: CHAT-06
    verification:
      - kind: unit
        ref: "tests/openai_chat_translate.rs#response_stream_reasoning_order_within_and_across_chunks"
        status: pass
      - kind: integration
        ref: "tests/openai_chat_conformance.rs#openai_chat_terminal_reasoning_order_streamed"
        status: pass
      - kind: integration
        ref: "tests/openai_chat_conformance.rs#openai_chat_terminal_error_after_text_single_terminal"
        status: pass
      - kind: integration
        ref: "tests/openai_chat_conformance.rs#openai_chat_terminal_usage_payload_after_finish_rejected"
        status: pass
      - kind: integration
        ref: "tests/openai_chat_conformance.rs#openai_chat_terminal_duplicate_done_rejected"
        status: pass
    human_judgment: false
  - id: R3
    description: "Tool_calls finish maps to stop_reason tool_use without tool-delta assembly (deferred to 13-04); request-id allowlist enforced at both machine and adapter boundaries"
    requirement: CHAT-07
    verification:
      - kind: unit
        ref: "tests/openai_chat_translate.rs#response_embedded_error_request_id_allowlist"
        status: pass
      - kind: unit
        ref: "tests/openai_chat_translate.rs#response_finish_map_content_filter_fails_closed"
        status: pass
    human_judgment: false

# Metrics
duration: ~45min
completed: 2026-09-08
status: complete
---

# Phase 13 Plan 03 Summary: Response Expansion for OpenAI Chat Completions

**Finish-reason map, usage precision, reasoning extensions, and safe provider-error/request-id surfacing in one semantic response machine, with router-level terminal conformance**

## Performance

- **Duration:** ~45 min
- **Tasks:** 2 of 2
- **Files modified:** 4 (+1 summary)

## Accomplishments

- The OpenAI Chat response machine now covers the full finish-reason map (stop/length/tool_calls; content_filter, function_call, and unknown fail closed naming finish_reason; error as a provider terminal), usage counters validated as non-negative integers within i64 range, empty-string content as a real empty text block, and required non-empty single-choice envelopes.
- Reasoning extensions: reasoning_content/reasoning alias normalization (null/empty absent, non-string reject, conflict reject), signed-representation rejection by field name, and thinking blocks emitted before text with stable block indices in both unary and streaming modes.
- Provider errors surface safely: exactly one neutral error terminal, no synthesized success, and only validated `x-request-id`/`request-id` response headers are relayed. Untrusted body metadata and raw provider error text are not forwarded.
- Strict trailing-chunk grammar: exactly one usage-only chunk after finish_reason (usage lands in message_delta), any non-empty trailing delta rejected even without a finish_reason key, usage before finish rejected.

## Task Commits

Each task was committed atomically (test RED -> feat GREEN):

1. **Task 1: Response machine (unary + shared streaming semantics)** - 39e847d (test RED), 09839ee (feat)
2. **Task 2: Router terminal conformance + strict trailing gating** - c0dc542 (test RED), 745dce7 (feat)
3. **Formatting** - 46fae55 (style, cargo fmt)

plan_head_before: 5bf613bb

## TDD RED Evidence

- Task 1 RED (isolated wrapper, cargo test --all-features --test openai_chat_translate response -- --test-threads=1): 28 fixtures, **17 failed / 11 passed / 0 ignored** - the failing subset covered reasoning, finish map, choices validation, usage bounds, request-id allowlist, and trailing gating; stop/length mapping and duplicate-choices rejection already passing as expected.
- Task 2 RED (cargo test --all-features --test openai_chat_conformance openai_chat_terminal -- --test-threads=1): 11 fixtures, **2 failed / 9 passed / 0 ignored** - exactly the two genuine router gaps: a trailing payload delta without a finish_reason key was accepted, and the unary 200-error body dropped request_id. The 9 passing fixtures pin current-batch behavior (trailing usage relay, reasoning order, single terminal after error, duplicate [DONE], residual, malformed JSON, missing-finish EOF).

## Verification (via isolated wrapper, production config untouched)

- cargo test --all-features --test openai_chat_translate response: 28 passed / 0 failed
- cargo test --all-features --test openai_chat_conformance openai_chat_terminal: 11 passed / 0 failed
- cargo test --all-features --test openai_chat_conformance openai_chat: 30 passed / 0 failed
- cargo test --all-features --test openai_chat_translate (full): 86 passed / 0 failed
- cargo fmt --all --check: exit 0
- cargo clippy --all-targets --all-features -- -D warnings: exit 0
- cargo build --all-features: exit 0
- cargo test --all-features --workspace: **2820 passed / 0 failed / 2 ignored** across 28 test binaries
- Every invocation ran through /tmp/shunt-phase12-isolated-run.cjs (temporary OPENCODEX_HOME, non-10100 port); the wrapper confirmed /Users/user/.opencodex config mtime/SHA and backup inventory unchanged after each run.

## Decisions Made

- Reject any non-empty delta in trailing position (not just content/reasoning/tool_calls), because the official usage-only chunk has empty choices - stricter and simpler than enumerating payload fields.
- Root review replaced the executor's body-metadata interpretation with the plan's response-header allowlist. The adapter owns this transport check, with explicit permitted-name and 127/128/129-byte tests.
- Keep the 13-02 empty-delta relay contract intact: content null is absent, role-only chunks still emit no fabricated blocks.

## Deviations from Plan

None blocking. Notes:

- One extra style commit (46fae55) applies cargo fmt across the touched files; no semantic changes.
- Streaming tool-call delta assembly remains deferred to 13-04. Root review implemented the required unary tool-use mapping, including bounded counts/arguments, unique IDs, valid object arguments, and thinking/text/tool ordering; 13-04 should reuse those named limits.
- STATE.md, ROADMAP, and REQUIREMENTS were intentionally left untouched; root owns tracking reconciliation and phase/global completion marking.

## Issues Encountered

- A detached long-running full-suite invocation lost its captured output; it was re-run once (single cargo job at a time) with logged output to record the exact 2820/0/2 counts.

## Next Phase Readiness

- 13-04 can attach tool-call delta assembly on top of the machine's reasoning/text block-index management and the established strict trailing grammar.
- 13-05 owns README/site/docs updates for the response contract before phase closure; no doc surfaces were changed in this plan.

---
*Phase: 13-generic-openai-chat-completions*
*Completed: 2026-09-08*

## Root review and final verification (supersedes executor-only evidence)

The root reproduced ten assertion failures after the executor's green suite:
discarded unary tools; absent/ambiguous message envelopes; permissive trailing
usage shapes; a panic on post-finish `error:null`; reasoning omitted from the
aggregate byte budget; raw embedded error text disclosure; ignored choice-level
errors; header request IDs replaced by body metadata; null empty completion
yielding no text block; and raw non-success error text disclosure. All were fixed
and rerun green. Nine added pure response tests and two diagnostic unit tests
remain; the existing request-ID fixture was strengthened to distinguish the
header from conflicting body metadata. Neutral-error assertions replace raw
provider-text assertions to enforce the locked redaction contract, not weaken it.

The full suite additionally caught the header-producer security tripwire. The
new Chat response-header map is now explicitly classified as an allowlist-built
downstream diagnostic map; the tripwire remains intact. Private checked-payload,
validation, and unary-tool helpers keep the central machine at 463 lines.
Unary tools share named limits of 128 calls and 1 MiB argument bytes with an
8 MiB aggregate semantic budget; 13-04 must reuse these bounds for streaming.

Final root verification on the corrected code:

- `cargo test --all-features --workspace --quiet`: **2,831 passed, 0 failed, 2 ignored**.
- Chat translation: **95 passed**; Chat conformance: **30 passed**, including real
  loopback gateway JSON and incremental SSE fixtures.
- `cargo fmt --all --check`, strict all-target/all-feature Clippy, and build pass.
- Project `.claude/skills/run-shunt/smoke.sh`: all five checks passed (CLI config check, liveness,
  model discovery, mock Anthropic proxy response, malformed-request error), on
  gateway port 31981 and mock port 31982. This baseline smoke is not a live Chat
  provider or Computer evaluation; Chat behavior is covered by the router suite.
- Every verification process tree used `/tmp/shunt-phase12-isolated-run.cjs`;
  production OpenCodex config mtime/SHA and invalid/backup inventory were unchanged.
- GSD schema-drift and UI safety gates: block false. Codebase-map drift remains
  advisory only (`directive:warn`, `spawn_mapper:false`); no gate was bypassed.

README, docs, and maintained site translations were considered and remain in
13-05's same-PR documentation scope; generated wiki files were not hand-edited.
Phase 13 and the milestone remain incomplete pending their remaining plans.
