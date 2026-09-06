---
phase: 04-native-compaction
created: 2026-09-05
status: locked
---

# Phase 04 Context — Native Compaction

## Phase Boundary

Expose `POST /v1/responses/compact` on the existing opt-in inbound Codex surface and forward compatible requests to a verified native compact endpoint. Shunt remains an opaque bounded proxy: it does not summarize, rewrite, decrypt, persist, or interpret conversation history or continuation state.

## Locked Decisions

- D-01: The endpoint is registered only when `[server.codex_endpoint]` is enabled and uses the same inbound authentication, concurrency limits, request-size limit, tracing, and OpenAI Responses error boundary as ordinary inbound Responses requests.
- D-02: A compact request must be valid JSON with one non-empty string `model`; missing, malformed, duplicate, or wrong-typed model input fails before network dispatch.
- D-03: Preserve the accepted request body bytes and content headers verbatim. Do not strip `reasoning`, translate the model, synthesize a summary, or inspect `input`/opaque continuation fields.
- D-04: Ordinary pinned ChatGPT routing remains compatible. An exact native route may be selected only through the Phase 2 resolver; ambiguous, translated-only, or non-Responses routes fail before dispatch.
- D-05: Native compact capability is fail closed: ChatGPT/Codex OAuth backends and the canonical official OpenAI API endpoint are supported; xAI/Grok and arbitrary OpenAI-compatible gateways are rejected before network dispatch.
- D-06: The upstream URL is the selected provider's existing Responses URL plus `/compact` (`.../codex/responses/compact` for ChatGPT and `.../responses/compact` for official OpenAI).
- D-07: Reuse existing provider credential resolution, ChatGPT account-pool selection/refresh/quota rotation, safe request-header stripping, safe response-header relay, TTFB timeout, and stream lifecycle. Do not add a second retry or account pool.
- D-08: Compaction is HTTP POST only. No WebSocket compact transport is introduced.
- D-09: No public config keys, credential writeback, request-history database, continuation cache, encryption layer, or routed synthetic summarizer.
- D-10: Tests must prove authenticated route registration, byte/header fidelity, pinned ChatGPT and exact native selection, upstream URL, unsupported/no-network rejection, final status/body/header relay, request limits, and ordinary `/v1/responses` regression safety.
- D-11: Update README and the Nimbus inbound Codex guide in English and ko/ja/zh-cn. Never hand-edit `wiki/`.

## Deferred

- Synthetic compaction for providers without a native compact endpoint.
- Durable history, recovery tokens, and local summary generation.
- Capability overrides for custom gateways; evidence is required before adding public configuration.

## Smart Discuss Resolution

No new user question was required: these decisions directly implement the already-approved selective-port recommendation and the Phase 4 roadmap success criteria without expanding public configuration or provider semantics beyond the documented native endpoint.
