---
phase: 07-collaboration-preservation
status: complete
created: 2026-09-06
---

# Phase 7 Research

## Existing Shunt seams

The inbound Codex resolver already separates native Responses passthrough from
the exact Anthropic translation vertical. Native requests are forwarded as the
original bounded bytes. Anthropic translation is isolated in
`model::inbound_responses::{request,response}` and currently rejects continuation
and provider-owned state before dispatch, making it the only safe insertion seam.

The response translator already owns both complete JSON and event-by-event SSE
tool-call projection, with bounded output items and arguments. A small immutable
request-authority map can therefore flatten namespaced request tools and restore
only calls the client authorized without buffering the stream.

## OpenCodex evidence adapted

OpenCodex's V2 routed-delegation bridge mirrors `spawn_agent`, `send_message`, and
`followup_task` into a plaintext namespace for routed models, then rewrites only
authorized response calls back to `collaboration` and supplies an empty
`encrypted_function_args` vector. Its tests also prove catalog detection in both
top-level `tools` and `additional_tools`, collision rejection, schema-marker
sanitization, JSON/SSE restoration, and native passthrough isolation.

OpenCodex's encrypted agent-task recovery performs an extra authenticated native
ChatGPT inference, maintains a bounded single-flight plaintext cache, and needs
crypto-sensitive policy. That is explicitly deferred by Shunt's RECOVERY-01.
Phase 7 ports its safer pre-dispatch unreadable-task detection and fail-closed
contract, not the recovery subsystem.

## Risks

- Restoring an undeclared namespace would let an upstream forge privileged calls;
  restoration must be request-authority-bound.
- A naive recursive removal of `encrypted` corrupts legitimate schemas; traversal
  needs schema-context rules and must be stack-safe or depth-bounded.
- Parsing native traffic would violate the central passthrough invariant; activate
  only inside the Anthropic translation branch.
- Logging ciphertext, routing envelopes, schemas, or arguments would disclose
  sensitive content; errors and telemetry use fixed descriptions only.
