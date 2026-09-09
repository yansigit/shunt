---
phase: "13"
slug: generic-openai-chat-completions
status: planned
created: "2026-09-08"
---

# Phase 13 — API Coverage Matrix (Generic OpenAI Chat Completions)

Scope is the approved Phase-13 surface (13-CONTEXT D-01/D-04): an operator-configurable
OpenAI Chat Completions-compatible upstream reachable through the existing Anthropic
Messages ingress. Decisions are reasoned against that scope; nothing below expands the
management, audio, or realtime endpoints.

| capability | decision | reason |
|---|---|---|
| POST /chat/completions (JSON, unary) | INTEGRATE | Core CHAT-01/CHAT-05 path: unary completion translated to the Anthropic response contract (plans 13-01/13-03). |
| POST /chat/completions (SSE streaming) | INTEGRATE | Core CHAT-06/CHAT-07 path: incremental relay with strict single-terminal semantics (plans 13-01/13-03). |
| stream_options.include_usage forcing | INTEGRATE | Required so the usage-only trailing chunk is obtainable; pinned by the 13-01 tracer. |
| Generation controls (temperature, top_p, stop, max_tokens, stream, n=1) | INTEGRATE | CHAT-03 whitelist; single-choice output keeps translation deterministic (13-02). |
| Tool declaration, tool_choice, parallel tool calls | INTEGRATE | CHAT-04 request side (13-02) and CHAT-06 streaming assembly (13-04). |
| Streamed tool-call deltas (indexed, identity across deltas) | INTEGRATE | CHAT-06; index-then-id assembly with one boundary JSON parse (13-04). |
| Usage accounting (prompt/completion tokens) | INTEGRATE | CHAT-05/CHAT-07 usage mapping with checked integral validation (13-03). |
| Finish reasons (stop, length, tool_calls, content_filter) | INTEGRATE | CHAT-05 stop-reason mapping incl. explicit content_filter (13-03). |
| Embedded provider errors (payload error, finish_reason error, 200-error body) | INTEGRATE | CHAT-05 provider-error surfacing in the Anthropic error shape (13-03). |
| Models-style discovery listing for the kind | INTEGRATE | D-01 wiring requires the kind to appear in existing capability/discovery listing; no new discovery endpoint. |
| Multimodal image input (image_url / data URLs) | INTEGRATE | CHAT-03/CHAT-09 image scenario (13-02, 13-05). |
| Long-context / byte-budget enforcement | INTEGRATE | CHAT-09/boundary named bounds incl. byte-cap triples (13-04/13-05). |
| Cancellation before headers and mid-stream (both modes) | INTEGRATE | CHAT-09 ownership proofs (13-05). |
| Redirect following | OPT-OUT | D-03: credential-bearing POST; redirect(Policy::none()) + refusal fixture (13-04). |
| Post-send retry/fallback | OPT-OUT | D-09: ConnectOnly only; ambiguous post-send replay forbidden (13-01/13-04). |
| Audio inputs/outputs (audio modality, TTS/STT payloads) | OPT-OUT | Explicitly outside approved Chat scope (D-01); not exercised by the Anthropic Messages ingress. |
| Moderation endpoint (/moderations) | OPT-OUT | Separate endpoint, outside scope; gateway exposes no management surface for it. |
| Realtime session endpoints | OPT-OUT | WebSocket protocol, outside scope. |
| Batch API (/batches) | OPT-OUT | Asynchronous job management, outside scope; no management endpoints in this phase. |
| Files API (/files) | OPT-OUT | File management surface, outside scope. |
| Fine-tuning endpoints | OPT-OUT | Training management, outside scope. |
| Images endpoint (/images/generations) | OPT-OUT | Different modality API, outside scope. |
| Responses-API interop fields | OPT-OUT | previous_response_id, Responses reasoning items, instructions and hosted web-search tools are not Chat fields; forwarding them is forbidden by the CHAT-03 whitelist (13-02). |
| prompt prefix caching parameters | OPT-OUT | Caching controls without an approved mapping from the Anthropic-ingress contract; revisit only if a later phase approves it. |
| n > 1 multiple choices | OPT-OUT | Anthropic ingress yields one response; ambiguous multi-choice output is rejected per CHAT-05 (13-03). |

Every INTEGRATE row maps to a named requirement and plan task; every OPT-OUT row is
either rejected with a typed error or never accepted by the whitelist, so no capability
is silently dropped.
