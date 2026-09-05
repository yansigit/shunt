# Phase 1 Context: Inbound Responses WebSocket

## Decisions

- **D-01 — Boundary:** Add WebSocket as a transport on the existing opt-in inbound Responses paths. Do not change the `/v1/messages` hot path or the existing Responses HTTP POST behavior.
- **D-02 — Reuse:** Route generated turn bodies through the existing inbound Codex endpoint/account-pool machinery so auth injection, sticky selection, quota admission, failover, metrics, and header sanitization are not duplicated.
- **D-03 — Fidelity:** Parse only WebSocket control fields and upstream SSE event type. Forward valid upstream `data:` payload text unchanged.
- **D-04 — Lifecycle:** Permit one active turn per socket. A replacement `response.create` or socket close aborts the prior turn, releases its response body/resources, and invalidates stale output.
- **D-05 — Warmup:** `response.create` with `generate:false` is local and emits empty-id `response.created` plus `response.completed`, with no upstream request.
- **D-06 — Bounds:** Client frame size, SSE event size, and any internal channel are explicitly bounded. Downstream sends are awaited so slow clients exert backpressure.
- **D-07 — Errors:** Authentication errors are HTTP OpenAI Responses errors before upgrade. Post-upgrade failures are `type:error` WebSocket frames with status and safe allowlisted headers.
- **D-08 — Scope:** Defer JSON-success event synthesis, native-provider routing, compaction, translation, collaboration recovery, persistence, and generalized repairs.
- **D-09 — Verification:** Use behavior-derived OpenCodex fixtures plus real Axum/WebSocket integration tests. Cancellation tests use synchronization signals, not sleeps alone.
- **D-10 — Documentation:** Update every affected English and maintained translated README/site surface; never hand-edit `wiki/`.

## Edge Coverage

| Edge | Resolution | Verification |
|------|------------|--------------|
| Endpoint disabled | GET remains 404 because WS routes are registered only with `[server.codex_endpoint]` | explicit integration test |
| Missing/invalid auth | Reject before upgrade; no socket opens and no upstream request occurs | explicit integration test |
| `response.processed` | Ignore as an acknowledgement no-op | explicit protocol test |
| Unknown or unparseable text frame | Ignore without crashing the connection, matching the source behavior | explicit protocol test |
| Binary frame | Reject as unsupported without treating bytes as JSON | explicit protocol test |
| Oversized frame/event | Emit bounded error/close behavior and release the turn | explicit unit/integration tests |
| CRLF, multiline data, split chunk, unterminated final SSE block | Reassemble within bounds and forward the joined JSON payload | explicit unit tests |
| `[DONE]` before terminal | Do not claim success; emit premature-EOF protocol error | explicit unit test |
| Terminal followed by stale data | Stop at first terminal and discard later bytes | explicit unit test |
| Replacement during live stream | Abort old upstream and prevent stale output | explicit synchronized integration test |
| Socket disconnect | Drop active upstream response and release resources | explicit synchronized integration test |

## Prohibitions

- Do not buffer or collect a successful upstream SSE response.
- Do not forward client credentials, cookies, hop-by-hop headers, or internal Shunt headers upstream.
- Do not expose upstream cookies or arbitrary response headers in WS error frames.
- Do not add public config keys or alter documented provider selection semantics.
- Do not change credential-file writeback behavior.
- Do not add persistent history, a second pool, or a general response-repair framework.
