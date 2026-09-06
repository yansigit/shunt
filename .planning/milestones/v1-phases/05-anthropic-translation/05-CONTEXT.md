---
phase: 05-anthropic-translation
status: discussed
created: 2026-09-05
requirements: [TRANS-01, TRANS-02, TRANS-03]
---

# Phase 5 Context: Anthropic Translation

## Boundary

Add one dedicated inbound OpenAI Responses to Anthropic Messages vertical slice.
An exact configured model route may select an Anthropic adapter; the configured
`[server.codex_endpoint]` provider remains the compatibility fallback for missing,
unknown, or unrouted models. Native Responses routes continue to forward request
and response bytes unchanged.

## Locked decisions

- D-01: Translation is selected only by one unambiguous exact `[[routes]]` or
  single-provider `[[models]]` mapping whose provider kind is `anthropic`.
- D-02: Prefix routes, default-provider inference, and multi-provider model maps
  do not opt a Responses request into translation.
- D-03: Parse translated requests strictly before credential resolution or any
  network dispatch; duplicate top-level keys and malformed JSON are errors.
- D-04: Translate `instructions`, user and assistant message content, function
  calls/results, inline URL or data-URL images, function tools, tool choice,
  output-token limit, temperature, top-p, and supported reasoning effort.
- D-05: Preserve function-call pairing and caller call IDs. Orphan results,
  duplicate call IDs, malformed JSON arguments, and invalid message ordering are
  actionable `400 invalid_request_error` failures.
- D-06: `previous_response_id`, provider-encrypted reasoning/compaction items,
  hosted tools, remote file IDs, and other state that cannot be represented
  faithfully fail before dispatch. No local history or recovery store is added.
- D-07: Reuse the existing Anthropic adapter for auth, account pools, refresh,
  retry, request deadlines, header policy, and admission. The new code owns only
  protocol parsing/translation and response projection.
- D-08: Streaming translation is incremental and bounded. It must not collect an
  Anthropic SSE response or allow unbounded SSE blocks/tool arguments.
- D-09: Emit Responses events in protocol order with stable response/item/call
  IDs, monotonic sequence numbers, one terminal event, and a final accumulated
  response snapshot.
- D-10: Map Anthropic `end_turn`, `stop_sequence`, and `tool_use` to completed;
  `max_tokens` to incomplete; explicit upstream error/`stop_reason:error` and
  malformed or premature streams to failed.
- D-11: Usage is inclusive: Anthropic input plus cache-read/cache-creation tokens,
  output tokens, and total tokens. Reasoning output tokens are reported when
  derivable without guessing.
- D-12: Anthropic thinking output is exposed as Responses reasoning summary
  events/items. Anthropic signatures are not relabeled as OpenAI encrypted
  content and are not persisted.
- D-13: HTTP and inbound WebSocket turns share the same translator. WebSocket
  cancellation/backpressure remains owned by the Phase 1 session loop.
- D-14: No public config keys, credential-file writeback changes, generalized
  capability platform, response repair framework, or `wiki/` edits.

## Accepted request surface

- Top level: `model`, `input`, `instructions`, `stream`, `tools`, `tool_choice`,
  `parallel_tool_calls`, `max_output_tokens`, `temperature`, `top_p`, `reasoning`,
  `store`, `metadata`, `include`, `prompt_cache_key`.
- Input items: `message`, `function_call`, `function_call_output`.
- Content blocks: `input_text`, `output_text`, `text`, `input_image` with a URL
  or supported image data URL.
- Tools: ordinary `type:function` definitions in both current Responses shapes.
- Reasoning controls: `none` disables where supported; `low`, `medium`, `high`,
  and `max` map to bounded Anthropic thinking budgets/adaptive effort.

Accepted no-op metadata (`store:false`, metadata, include, prompt cache key) must
not affect semantic translation. A request that asks for behavior Shunt cannot
preserve is rejected rather than quietly weakened.

## Out of scope

- Native Responses passthrough rewriting.
- Durable continuation, encrypted state conversion, compaction synthesis.
- Hosted web search, image generation, computer use, custom/freeform tools.
- Cross-provider fallback or capability manifests (Phase 6).
- Repairs for malformed provider output beyond deterministic protocol errors.

