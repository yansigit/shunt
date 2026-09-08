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
  - "Provider error surfacing: embedded error objects (chunk-level, finish_reason error, trailing position) produce exactly one error terminal; metadata.request_id forwarded only under the printable-ASCII 1..=128 allowlist and echoed through the unary adapter error body"
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
    - "Provider error bodies are relayed verbatim from the machine (including the allowlisted request_id) instead of being rebuilt message-only at the adapter boundary"
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
  - "metadata.request_id is the only request-id source honored, restricted to 1..=128 printable ASCII bytes (no spaces/controls); top-level request-id and non-allowlisted metadata keys are dropped, so untrusted strings cannot smuggle framing into error bodies."
  - "content: null is treated as absent (no text block) while an empty-string content is a real empty text block, preserving the existing empty-delta relay contract for role-only chunks."
  - "finish_reason error maps to a provider terminal (error event, ProviderFailed) rather than a protocol error, so an upstream-declared failure can never be upgraded or downgraded by the transport."
  - "Trailing usage gating is positional, not content-based: exactly one usage-only chunk immediately after finish_reason; a second one, any usage before finish, or any non-empty delta there fails closed."

patterns-established:
  - "Fixture functions carry contiguous filter substrings (response_, openai_chat_terminal) so the plan verify filters can never select zero tests."
  - "RED gates recorded as raw libtest counts with the failing subset identified before production edits."

requirements-completed: [CHAT-05, CHAT-06, CHAT-07]

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
- Provider errors surface safely: exactly one error terminal, no synthesized success, and metadata.request_id forwarded only under the printable-ASCII 1..=128 allowlist - relayed verbatim through the unary adapter error body.
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
- Relay the machine's error event body verbatim in the unary adapter path instead of rebuilding a message-only error, so the allowlisted request_id survives to the client and no new allowlist logic lives in the adapter.
- Keep the 13-02 empty-delta relay contract intact: content null is absent, role-only chunks still emit no fabricated blocks.

## Deviations from Plan

None blocking. Notes:

- One extra style commit (46fae55) applies cargo fmt across the touched files; no semantic changes.
- Tool-call delta assembly remains out of scope for 13-04 per the plan; tool_calls finish mapping here only sets stop_reason: tool_use.
- STATE.md, ROADMAP, and REQUIREMENTS were intentionally left untouched; root owns tracking reconciliation and phase/global completion marking.

## Issues Encountered

- A detached long-running full-suite invocation lost its captured output; it was re-run once (single cargo job at a time) with logged output to record the exact 2820/0/2 counts.

## Next Phase Readiness

- 13-04 can attach tool-call delta assembly on top of the machine's reasoning/text block-index management and the established strict trailing grammar.
- 13-05 owns README/site/docs updates for the response contract before phase closure; no doc surfaces were changed in this plan.

---
*Phase: 13-generic-openai-chat-completions*
*Completed: 2026-09-08*
