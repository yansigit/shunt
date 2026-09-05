# Pitfalls Research

**Domain:** Inbound Responses WebSocket transport
**Researched:** 2026-09-05

## Critical Pitfalls

| Pitfall | Warning sign | Mitigation |
|---------|--------------|------------|
| Authenticate after upgrade | Unauthorized client receives `101` or any WS frame | Resolve refreshed `InboundAuth` and reject with OpenAI HTTP error before `on_upgrade`. |
| Blocking socket reads during a turn | Replacement `response.create` cannot cancel until prior turn ends | Select over client input and active-turn output; make the turn independently abortable. |
| Stale events after replacement | Frames from response A appear after response B starts | Increment a turn generation before abort/start; discard outputs whose generation is no longer current. |
| Unbounded SSE/frame buffering | Memory grows with a missing delimiter or slow client | Bound client frame bytes, SSE block bytes, and any internal channel; await downstream sends. |
| Treating `[DONE]` as successful terminal | Truncated stream silently completes | Require `response.completed`, `.failed`, or `.incomplete`; `[DONE]` alone is not terminal. |
| Reconstructing event JSON | Opaque/encrypted/new fields disappear | Validate `type`, then forward the exact joined `data:` payload text. |
| Leaking auth/cookies | Client bearer or upstream cookies cross trust boundaries | Reuse existing inbound forwarding sanitization; allowlist only safe headers in error frames. |
| Wrong warmup semantics | Probe bills upstream or poisons continuation | Intercept only WS `response.create` with `generate:false`; emit empty-id created/completed locally and do not dispatch. |
| Holding concurrency only for handshake | Unlimited live turns bypass request limit | Ensure active-turn admission/lease lifetime spans the upstream body and is dropped on cancel. Verify how the existing router layer applies to upgrades before relying on it. |
| Assuming send-await alone bounds all memory | Producer task fills an unbounded channel | Use a small bounded channel or direct selected sends; never `unbounded_channel`. |
| Error double-send | Client receives protocol error after a valid terminal | Track terminal/error emission exactly once and stop reading immediately after a terminal. |
| Flaky cancellation tests | Tests race on sleeps and sometimes pass without abort | Use barriers/notifies and observable upstream connection/body-drop signals instead of timing-only assertions. |

## Scope Traps

- Do not add native Responses provider routing in this phase; it changes documented
  provider semantics and requires explicit approval.
- Do not implement `/v1/responses/compact`, collaboration recovery, JSON-success
  event synthesis, or general repair logic in the tracer slice.
- Do not modify credential-file writeback.
- Do not hand-edit `wiki/`; observable behavior requires README/docs/site English
  and all maintained site/root translations where the affected surface exists.

## Verification Checklist

- Unauthorized handshake is rejected before upgrade.
- All three configured Responses paths accept WS only when the endpoint is enabled.
- Warmup creates no upstream request.
- SSE frames survive split chunks, CRLF, multiline data, and an unterminated final
  event within the configured bounds.
- All three terminal types stop the pump; premature EOF and malformed upstream JSON
  yield one protocol error.
- Replacement and disconnect release upstream bodies and leases.
- Oversized input closes with 1009 after a 413 frame when delivery is possible.
- `cargo fmt --all --check`, Clippy with warnings denied, and the full workspace test
  suite pass.

## Sources

- `.planning/codebase/CONCERNS.md`
- `.planning/notes/opencodex-port-audit.md`
- OpenCodex `tests/ws-endpoint.test.ts`, `tests/bridge-live-delivery.test.ts`, and
  `tests/sse-client-frame-bounds.test.ts`
