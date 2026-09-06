---
phase: 05-anthropic-translation
status: complete
created: 2026-09-05
---

# Phase 5 Research

## Existing Shunt seams

- `src/codex_endpoint.rs` already owns request limits, zstd decoding, inbound
  authentication, exact routing, OpenAI error conversion, metrics, and the common
  HTTP/WS `forward_turn` seam.
- `src/routing.rs::resolve_native_inbound` deliberately accepts only Responses
  adapters today. Extend the decision without widening matching precedence.
- `src/adapters/responses/inbound.rs` is the byte-faithful native path and must
  remain untouched by translated bodies.
- `src/adapters/anthropic/mod.rs` already owns API-key and Claude OAuth transport,
  account selection, refresh, retry, deadlines, safe relay headers, and admission.
  A translated Messages `RequestBody` can reuse its `Adapter::forward` contract.
- `src/codex_endpoint/frame.rs` and `websocket.rs` already bound and cancel inbound
  WS streams; translated SSE output can flow through the same event relay.
- `src/model/responses_request.rs` and `src/model/responses.rs` implement the
  opposite direction. They are useful protocol references, not inversion points.

## OpenCodex evidence

- `src/responses/parser.ts` parses Responses input items and tool definitions into
  a canonical turn model.
- `src/adapters/anthropic.ts::messagesToAnthropicFormat` preserves call/result
  adjacency, content parts, tool schemas, and Anthropic reasoning controls.
- `src/adapters/anthropic.ts::parseStream` handles message/content lifecycle,
  fragmented tool JSON, cumulative usage, malformed shapes, error stops, and EOF.
- `src/bridge.ts::bridgeToResponsesSSE` establishes Responses event ordering,
  stable IDs, sequence numbers, tool/reasoning item lifecycles, and terminal
  snapshots.
- Valuable behavior is the strict lifecycle and fidelity policy. Shunt should not
  port OpenCodex's dynamic policy router, persistence, compatibility lab, sidecar
  tools, recovery tokens, or broad repair framework.

## Minimal architecture

```text
Responses HTTP/WS body
  -> strict inbound parser + feature validation
  -> exact route decision
       Responses provider -> existing byte-faithful native path
       Anthropic provider -> request translator
          -> existing Anthropic Adapter transport
          -> Anthropic JSON/SSE response translator
          -> Responses JSON/SSE (same HTTP/WS relay boundary)
```

Use `src/model/inbound_responses/` for pure translation and state-machine logic,
and `src/adapters/anthropic/inbound.rs` for transport composition. This keeps the
new direction independent from the mature Anthropic-client translation path.

## Primary risks

1. Silent loss from permissive JSON walking. Mitigate with typed validation and
   path-specific errors before credential resolution.
2. Invalid Anthropic history when tool results are not immediately paired with
   assistant tool use. Validate and build paired turns explicitly.
3. Tool argument memory growth. Bound assembled arguments by the existing inbound
   limit and fail the stream once the cap is crossed.
4. False success on EOF. Require `message_stop` after a valid stop reason and emit
   exactly one failed terminal event otherwise.
5. Mixed protocol errors. Convert translated upstream non-success and malformed
   output to OpenAI Responses envelopes/events at the Codex boundary.
6. Native regressions. Pin byte identity with existing Phase 2/4 tests and add an
   explicit native-vs-translated dispatch matrix.

