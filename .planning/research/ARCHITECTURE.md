# Architecture Research

**Domain:** Inbound Responses WebSocket vertical slice
**Researched:** 2026-09-05

## Module Boundary

Keep `codex_endpoint.rs` as the HTTP/auth/dispatch boundary and add
`src/codex_endpoint/websocket.rs` as a child module. The child may reuse private
parent helpers while isolating frame parsing and socket lifecycle from the mature
HTTP passthrough.

```text
GET /{codex responses path}
  -> codex_endpoint::websocket::get
  -> refresh state + authenticate request before upgrade
  -> WebSocket session loop
       -> response.processed: ignore
       -> response.create generate=false: local two-frame completion
       -> response.create: synthesize Body + call existing forward
            -> non-2xx/JSON error: WS error frame
            -> SSE body: bounded SSE framer -> validated WS text events
       -> replacement/close: abort active turn and drop upstream body
```

## Responsibilities

### `server.rs`

- Register `get(websocket::get).post(codex_endpoint::post)` for every existing
  `codex_endpoint::PATHS` entry.
- Preserve opt-in route registration and existing middleware ordering.

### `codex_endpoint.rs`

- Expose narrowly scoped helpers to the child module rather than duplicating auth,
  sticky-key, limits, routing, telemetry, or error conversion.
- Keep HTTP behavior byte-for-byte unchanged.

### `codex_endpoint/websocket.rs`

- Authenticate before returning `101 Switching Protocols`.
- Preserve handshake headers as the turn's inbound headers; replace content headers
  to match the synthesized JSON body.
- Parse bounded text JSON frames. Ignore acknowledgements/unknown/unparseable frames
  for OpenCodex compatibility; close oversized/binary request frames predictably.
- Generate local warmup terminal frames.
- Own a monotonic turn id and one abortable active-turn task.
- Convert successful upstream SSE `data:` payloads to WS text without mutation.
- Emit one OpenAI-style `type:error` envelope for non-streaming errors, malformed
  upstream SSE, or EOF before a terminal event.

## Streaming and Cancellation

The session loop must remain responsive while a turn streams. Split the socket and
run the active turn as a task whose output is sent through a bounded channel, or use
`tokio::select!` over inbound client frames and turn output. Replacement frames
abort the task and increment the turn id; emitted frames carry/check that generation
so stale output cannot leak. Socket close aborts the task. Await each actual WS send
before accepting another output item.

The upstream response body remains streaming. Dropping the body/task is the
cancellation primitive; existing RAII admission and account state must unwind on
all exits.

## Error Framing

Handshake/auth failures remain ordinary HTTP OpenAI-shaped responses. After upgrade,
errors use:

```json
{"type":"error","status":502,"error":{"type":"protocol_error","code":"websocket_protocol_error","message":"..."},"headers":{}}
```

Only safe response headers (`retry-after`, request ids, rate-limit headers) may be
copied into the frame. Upstream terminal payloads pass through unchanged.

## Test Seams

- Unit-test the bounded SSE framer, multiline/CRLF/split-chunk parsing, terminal
  detection, warmup frames, and safe-header allowlist.
- Integration-test real upgrade/auth, upstream request shape, live event ordering,
  error status preservation, replacement cancellation, disconnect, and all paths.
- Copy behavior, not test implementation, from OpenCodex fixtures.

## Implementation Order

1. Pure framing and warmup helpers with unit tests.
2. Upgrade/auth and one-turn end-to-end tracer test.
3. Replacement/disconnect cancellation and bounded frame tests.
4. Documentation translations and full repository quality gates.
