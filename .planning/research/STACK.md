# Stack Research

**Domain:** Inbound OpenAI Responses WebSocket transport in Shunt
**Researched:** 2026-09-05

## Recommendation

Implement the transport with Shunt's existing Rust/Axum/Tokio stack. Axum already
provides server-side WebSocket upgrade and message types, while `http-body-util`
and the existing passthrough response body expose upstream SSE chunks without a
second HTTP client or connection pool. No new runtime dependency is needed.

## Existing Components to Reuse

| Need | Existing component | Use |
|------|--------------------|-----|
| HTTP/WS routing | `axum` | Register `GET` and `POST` on the same Responses paths; authenticate before `on_upgrade`. |
| Socket lifecycle | `axum::extract::ws` | Receive text frames, send text/error/close frames, and let send-awaiting enforce transport backpressure. |
| Turn execution | `codex_endpoint::forward` | Reuse request limits, client auth, sticky pool keys, telemetry, account rotation, and byte-preserving upstream forwarding. |
| Streaming body reads | `http_body_util::BodyExt` | Consume upstream body frames incrementally and never collect a successful SSE stream. |
| Cancellation | Tokio task ownership | Abort the active turn when a replacement frame arrives or the socket closes; dropping the response body cancels upstream consumption and releases RAII leases. |
| JSON inspection | `serde_json` | Parse only WS control envelopes and SSE event payloads; keep the forwarded request body byte-faithful after removing WS-only fields. |
| Integration tests | `tokio-tungstenite`, `wiremock` | Connect a real WS client to a spawned Axum listener and assert upstream requests plus returned frames. |

## Dependency Decision

- Keep the current `tokio-tungstenite`/`tungstenite` pins used by outbound Codex WS.
- Enable Axum's `ws` feature if it is not already enabled by the current feature set.
- Do not add a second SSE parser crate. The protocol subset needed here is small:
  bounded blocks split on blank lines, `data:` line joining, `[DONE]` handling,
  JSON validation, and terminal-type recognition.
- Do not introduce an application-level outbound queue. Await each Axum WebSocket
  send before reading more upstream body data so backpressure is naturally bounded.

## Bounds

- Reuse `[server.limits].max_request_bytes` as the inbound WS text-frame/request
  bound, with a small hard ceiling if the configured value is unlimited.
- Give the SSE framer an explicit maximum event size and fail with one protocol
  error instead of growing indefinitely.
- Permit one active turn per socket. A replacement turn invalidates and aborts the
  previous pump before a new one begins.

## Avoid

- Do not route through the outbound Codex WS pool; it solves the opposite boundary.
- Do not deserialize and reconstruct native Responses SSE payloads.
- Do not add SQLite, durable session state, or generalized protocol middleware.

## Sources

- `Cargo.toml`
- `src/server.rs`
- `src/codex_endpoint.rs`
- `src/adapters/responses/inbound.rs`
- `.planning/notes/opencodex-port-audit.md`
- OpenCodex `src/server/ws-bridge.ts` and `tests/ws-endpoint.test.ts`
