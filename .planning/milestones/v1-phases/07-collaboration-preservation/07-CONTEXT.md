---
phase: 07-collaboration-preservation
status: ready
created: 2026-09-06
requirements: [COLLAB-01]
---

# Phase 7 Context

## Goal

Preserve Codex V2 collaboration semantics when an explicitly enabled inbound
Responses request is translated to Anthropic, while native Responses remains an
opaque passthrough and unreadable encrypted tasks never leak or trigger hidden
recovery requests.

## Decisions

- D-01: Add one opt-in `collaboration = true` field under
  `[server.codex_endpoint]`; absent and false retain Phase 5 behavior exactly.
- D-02: Native Responses routes are never parsed or rewritten for collaboration,
  regardless of the flag. Their request, task metadata, `previous_response_id`,
  encrypted content, and response events remain byte-faithful.
- D-03: On exact Anthropic routes, recognize only the Codex V2
  `collaboration` namespace and its function children from top-level `tools` or
  current/replayed `additional_tools` items. Flatten names for Anthropic and
  restore the authorized namespace on JSON and SSE output.
- D-04: Preserve agent messages as separate user turns and preserve their
  plaintext content. The client-owned routing envelope in the content remains
  authoritative for task name and sender; no roster is invented locally.
- D-05: Strip the Responses-only `encrypted: true` schema annotation only when it
  is a schema keyword. Never strip a property literally named `encrypted` or
  values inside `const`, `default`, `enum`, or `examples`.
- D-06: Only response calls for request-authorized flattened names may regain the
  `collaboration` namespace. Restored calls carry
  `encrypted_function_args: []`; ordinary tools are unchanged.
- D-07: Reject unreadable encrypted `agent_message` tasks and all
  `previous_response_id`/provider-owned state on translated routes before
  credentials or network dispatch. Do not decrypt, cache plaintext, persist
  history, or issue a recovery inference request.
- D-08: Bounds are inherited from request size, tool/output-item count, response
  bytes, and tool-argument limits. No new cache, task, timer, or retry loop exists.
- D-09: `tool_search`/`tool_reference`, arbitrary hosted/custom tools, encrypted
  task recovery, subagent model selection, and durable continuation remain out
  of scope until a compatible translated execution path and production evidence
  exist.
- D-10: Document the opt-in and fail-closed boundary across every maintained
  locale; do not edit generated `wiki/`.

## Scope boundaries

No credential writeback, decryption key, billable recovery, persistence,
generalized collaboration orchestrator, provider fallback policy, or native
ChatGPT mutation.
