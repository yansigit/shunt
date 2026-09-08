---
phase: "13"
slug: generic-openai-chat-completions
status: evidence-record
created: "2026-09-08"
---

# Phase 13 — Official Chat Completions protocol evidence

Provenance: the create/schema/function-calling facts below were fetched and read
from the official pages by the root agent during the independent-checker context
("root fetched official sources"). The streaming-events page was fetched from
this worktree shell during the plan-revision turn. No live provider smoke is
claimed or required. Derived wording stays under ~150 words per source; no long
quotations.

## Official facts

### Create method — POST /chat/completions

Source: https://developers.openai.com/api/reference/typescript/resources/chat/subresources/completions/methods/create
Provenance: root fetched official sources (independent-checker context).

The documented operation is POST .../chat/completions. The example request uses
role "developer". Tool declarations use type "function" with a function object
carrying name and a JSON-Schema parameters object. Tool-call arguments are a
JSON-encoded string on the wire, not a nested object. This grounds the
request-side whitelist and the serialize-once arguments rule in 13-02.

### chat.completion.chunk schema

Source: https://developers.openai.com/api/reference/typescript/__sdk_schema?declaration=(resource)+chat.completions+%3E+(model)+chat_completion_chunk+%3E+(schema)&selected=(resource)+chat.completions+%3E+(method)+create
Provenance: root fetched official sources (independent-checker context).

Official chunk schema: choices may be empty for the last chunk when
stream_options include_usage is set; usage is null except the final chunk that
carries full request usage; an interrupted or cancelled stream may never deliver
usage. finish_reason values are stop, length, tool_calls, content_filter,
function_call (deprecated) and null for interim chunks; "error" is not an
official finish reason. Delta fields are content, function_call, refusal, role,
tool_calls. The official schema has NO reasoning_content/reasoning textual
fields (reasoning_tokens is usage count only). Tool deltas require index; id and
function.name may arrive later across deltas; argument strings concatenate and
the completed JSON is validated after assembly.

### Function-calling guide

Source: https://developers.openai.com/api/docs/guides/function-calling
Provenance: root fetched official sources (independent-checker context).

Lines 4026-4084 demonstrate tool-call streaming accumulation: assemble by index,
with id and function.name optional until a delta provides them, concatenating
argument fragments and validating the completed JSON at the end. This directly
grounds 13-04's index-first, identity-deferring, single-boundary-parse design
and its unnamed-at-terminal failure rule.

### Stream options and terminal sentinel

Source: https://developers.openai.com/api/reference/resources/chat
Provenance: root fetched and read the ChatCompletionStreamOptions section after
planner revision, resolving the previously missing sentinel evidence.

With include_usage enabled, a usage-only chunk with empty choices precedes
`data: [DONE]`. Other chunks carry null usage. Interrupted streams may omit
final usage. The sentinel is official framing, not an invented hardening rule.
Shunt additionally requires a supported finish reason before that sentinel;
a missing finish or sentinel fails closed. Success is withheld until all bytes
already received in the current bounded batch pass terminal validation.

## Gateway hardening (beyond the official contract)

- RetrySafety::ConnectOnly; no retry or failover after the request could have
  reached the upstream (D-09).
- Redirect(Policy::none()) on the credential-bearing transport (D-03).
- Bounded event/residual/aggregate bytes, named consts, tri-probe coverage.
- Fail-closed EOF without finish, duplicate terminals, residual bytes.
- Empty choices rejected outside the documented usage-only trailing position.
- Usage parsed as integral JSON numbers only; no strings or lossy casts.
- Unsupported request fields, including metadata, are rejected before dispatch.

## Provider extensions (NOT official OpenAI fields)

message reasoning_content and delta reasoning_content / delta reasoning strings
are provider extensions emitted by specific upstreams; they do not exist in the
official delta schema. Shunt's extension handling (13-02 request side, 13-03
response side): supported plaintext string forms relay as Anthropic thinking
blocks with no fabricated signature; streaming preserves chunk arrival order,
with thinking before text within one chunk; unary emits thinking then text then
tools because JSON fields have no temporal order. Null and empty strings are
treated as absent; conflicting aliases, non-string values, and signed or
redacted representations fail closed. No thinking-block replay is synthesized
into request history beyond the documented assistant reasoning_content mapping.
