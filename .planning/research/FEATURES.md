# Feature Landscape

**Domain:** Multi-Provider AI API Gateway & Responses Protocol Ingress
**Researched:** 2026-09-05

## Table Stakes

Features that clients (specifically OpenAI Codex CLI / ChatGPT Responses clients) strictly expect. Missing any of these causes client hangs, protocol errors, connection aborts, or broken turn lifecycles.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Inbound Responses WebSocket Ingress (`GET /v1/responses` upgrade)** | Modern Codex CLI clients use WebSocket transport (`wss://.../codex/responses` or `/v1/responses`) to avoid HTTP request overhead and connection drops on long sessions. | Med | Matches paths configured in `[server.codex_endpoint]` (`/backend-api/codex/responses`, `/responses`, `/v1/responses`). Axum WebSocket upgrade with identical auth rules. |
| **Handshake Authentication & Origin/Header Filtering** | Unauthenticated WS upgrades must be rejected at handshake before socket open. Forward headers must strip hop-by-hop/cookies while retaining turn state and auth. | Low | Reuses `inbound_auth` / token checks. Header sanitization matches `selectForwardHeaders` / `safeResponseHeaders`. |
| **Deterministic Local Warmup (`generate: false`)** | Codex CLI sends `generate: false` probe frames on session init to establish connectivity without triggering upstream model invocation. Must complete locally. | Low | Generates synthetic `response.created` (`sequence_number: 0`, `in_progress`) followed by `response.completed` (`sequence_number: 1`, `completed`), `output: []`, `id: ""`. No upstream dispatch. |
| **Text `response.create` Ingress & Ack Handling** | Client drives turns over WS by sending JSON text frames of type `response.create`. Client may also send `response.processed` acknowledgements. | Low | Parse text frames into Responses requests. Ignore `response.processed` as no-op ack. Reject binary or non-text frames with protocol error. |
| **Single Active Turn per Socket with Cancellation** | Codex protocol permits only one active turn per socket at a time. A replacement `response.create` on an active socket cancels the in-flight turn immediately. | Med | Monotonic `turn_id` per socket. Replacement aborts upstream fetch/stream and invalidates stale pump frames before starting new turn. Disconnect also aborts upstream turn. |
| **Faithful Terminal Framing (`completed`, `incomplete`, `failed`, `error`)** | Codex clients require exact terminal event semantics to finalize the UI/CLI turn state. Premature EOF without terminal must emit synthetic error. | Med | Re-frames upstream SSE into WS text frames. Terminal events (`response.completed`, `response.incomplete`, `response.failed`) finalize the turn and close pump. Stream ending before terminal emits 502 `error` frame. |
| **Non-2xx HTTP / Upstream Error Framing** | When upstream returns an error status (e.g. 429, 500), WS client expects an OpenAI Responses `type: "error"` JSON frame with status code and safe headers (`Retry-After`). | Low | Standalone error frame: `{"type":"error","status":code,"error":{...},"headers":{...}}`. Gateway-owned errors must also follow OpenAI shape. |
| **Application/JSON 200 Translation to Events** | Upstream or cached responses returned as buffered `application/json` must be reframed as an event sequence (`response.created`, `output_item.done`, `response.completed`). | Low | Required if upstream does not stream SSE or returns complete JSON. |
| **Bounded Frame Buffering & Backpressure** | Slow WS clients or fast upstream SSE streams must not cause unbounded memory buffering. | Med | Bounded frame parsing, bounded outbound queue, explicit `max_payload_length` (e.g. 4MB), and backpressure drain handling. |
| **Lease & Admission Teardown Lifecycle** | Account pool lease and concurrency slots must be held during the active turn and guaranteed to release on terminal event, client cancel, or error. | Low | RAII guard ensures pool leases and concurrency permits are returned promptly, preventing lease leakage across replacement turns. |

## Differentiators

Features that set Shunt apart from heavier platforms (like OpenCodex's full SQLite-backed multi-tier architecture) while providing high-value protocol resilience and performance.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Zero-Copy Byte-Preserving Passthrough on Native Responses Upstreams** | Minimizes CPU overhead and latency. Eliminates serialization drift or subtle field drops (e.g., encrypted reasoning, client metadata, multi-agent coordination payloads). | Low | Pass upstream bytes directly to client without JSON deserialization/reserialization unless re-framing SSE to WS text. |
| **Exact Model Routing to Heterogeneous Responses-Native Upstreams** | Allows routing specific models (e.g. OpenAI official, xAI, Azure) directly to native endpoints without running through translation pipelines. | Med | Extracted `model` route lookup with fail-closed validation if provider adapter is not Responses-compatible. |
| **Standards-Compliant `Retry-After` Parsing (HTTP-Date & Seconds)** | Differentiates transient burst limits from hard quota exhaustion; honors provider cooldown periods accurately across providers. | Low | Parses both delta-seconds (`120`) and RFC 7231 IMF-fixdate strings into monotonic deadlines. |
| **Granular Quota vs Rate-Limit Classification** | Prevents dead loops rotating through accounts that hit hard account-level quota caps while handling transient 429s gracefully. | Low | Inspects error codes/messages (`insufficient_quota` vs `rate_limit_exceeded`) to mark account exhaustion accurately. |
| **Native `/v1/responses/compact` Opaque Forwarding** | Supports long Codex sessions that utilize OpenAI/ChatGPT compaction without requiring local storage or decrypting opaque tokens. | Med | Forwards compaction endpoint directly to compatible native Responses provider. |
| **Bounded Graceful Drain on Shutdown** | Prevents runaway SSE/WS streams from blocking server restart indefinitely without abrupt drops. | Low | Fixed shutdown deadline (e.g. 15–30s) cancels remaining active turns cleanly and releases pool leases. |
| **Responses-to-Anthropic Bidirectional Translation Vertical Slice** | Allows Codex CLI to run against Anthropic Claude models with full tool calling, streaming text, and reasoning preservation. | High | Dedicated isolated translator module with strict input validation, distinct from Messages-to-Responses translator. |

## Protocol Conformance Tests

To verify compliance with OpenCodex behaviors without importing platform complexity, the following conformance test suite must be implemented:

| Test Scenario | Focus / Assertion | Source Reference |
|---------------|-------------------|------------------|
| **WS Upgrade & Handshake Auth** | Verify GET upgrade succeeds on configured paths; rejects unauthorized or missing bearer token with 401/403. | OpenCodex `ws-bridge.ts`, `inbound_codex_endpoint.rs` |
| **Warmup Frame Generation** | Send `{"type":"response.create","model":"gpt-5.5","generate":false}`; verify local emission of `response.created` and `response.completed` with no upstream call. | OpenCodex `ws-endpoint.test.ts` ("generate=false warmup") |
| **SSE Re-Framing to WS Text** | Stream upstream SSE events; verify reframing into WS text frames in identical sequence, stopping at terminal. | OpenCodex `ws-endpoint.test.ts` ("re-frames SSE data payloads") |
| **Terminal Fidelity (`completed`, `failed`, `incomplete`)** | Verify SSE ending with `response.completed`, `response.failed`, or truncated stream correctly emits respective event or 502 error frame. | OpenCodex `ws-endpoint.test.ts` ("reports failed terminal status", "reports incomplete") |
| **Client Disconnect / Replacement Turn Cancellation** | Send replacement `response.create` or close client socket during active stream; assert upstream request is aborted and old pump emits nothing. | OpenCodex `ws-endpoint.test.ts` ("wires a cancel hook", "does not emit stale frames") |
| **Frame Size Bounds & Malformed Frames** | Verify frames > max payload (4MB) emit 413 error frame and close with 1009; verify binary/unparseable frames do not crash session. | OpenCodex `ws-endpoint.test.ts`, `server/index.ts` |
| **Backpressure & Drain Handling** | Slow consumer stalls socket write; verify buffer limits hold and pumping pauses until write drain without dropping frames. | OpenCodex `ws-endpoint.test.ts` ("backpressured websocket sends are accepted") |
| **Header Filtering on Ingress/Egress** | Forward headers drop cookies, keep auth and turn-state; egress headers strip `set-cookie`, preserve `retry-after` and rate limits. | OpenCodex `ws-endpoint.test.ts` ("stores only allowlisted inbound headers") |
| **JSON 200 Response Re-Framing** | Non-streaming upstream JSON response is reframed into `response.created`, `output_item.done`, and `response.completed`. | OpenCodex `ws-endpoint.test.ts` ("converts application/json 200 responses into event sequence") |

## Anti-Features

Features from OpenCodex or general gateways to explicitly NOT build in Shunt.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **SQLite Request-History & Generalized Database** | Adds disk I/O, schema migrations, lock contention, and operational bloat in Shunt's high-throughput proxy hot path. | Keep in-memory transient stream state; rely on client-side session management and connection-scoped state. |
| **Compatibility Lab, Management Web GUI & Dashboard** | Shunt is a lean CLI-managed daemon / headless gateway, not a full admin SaaS platform. | Expose simple status endpoints (`/health`, `/status`, admin tokens) and CLI check tools. |
| **Dynamic Manifest Platform & Plugin Framework** | Dynamic plugin engines and manifest evaluators add runtime unpredictability and security attack surface. | Use typed Rust configuration (`config.toml`) with compile-time checked adapters and validation. |
| **Wholesale / Speculative Response Repair Modules** | Arbitrary patching of upstream payloads masks upstream breakage and adds brittle regex/AST rewrites. | Add targeted repairs ONLY when backed by a verified captured failing transcript fixture. |
| **Durable Continuation Spill to Disk** | Storing continuation responses on disk adds complex invalidation, encryption, and lifecycle management. | Keep continuation strictly connection-scoped and in-memory as implemented in `codex_ws.rs` / `codex_continuation.rs`. |
| **Duplicated Connection / Account Pooling Stacks** | Shunt already has robust M10 account pooling, quota tracking, and outbound WS connection reuse. | Reuse existing `AccountPool` and `InboundAuth` primitives rather than creating a second WS-specific pool. |
| **Decrypting Opaque Payloads on Native Route** | Parsing/decrypting OpenAI agent tasks, reasoning tokens, or compaction payloads risks corruption and violates privacy. | Treat native Responses traffic as opaque byte-preserving passthrough. |

## Feature Dependencies

```
Inbound Responses WebSocket Ingress (Handshake & Upgrade)
  │
  ├──► Inbound Auth & Header Filtering (Table Stakes)
  │
  ├──► Local Warmup (generate: false) (Table Stakes)
  │
  ├──► Frame Parsing & Single-Turn Lifecycle (Table Stakes)
  │      │
  │      ├──► Replacement Turn & Disconnect Cancellation (Table Stakes)
  │      │
  │      └──► SSE Re-framing & Terminal Fidelity (Table Stakes)
  │             │
  │             ├──► Backpressure & Bounded Buffering (Table Stakes)
  │             │
  │             └──► Exact Native Responses Routing (Differentiator)
  │                    │
  │                    ├──► /v1/responses/compact Native Forwarding (Differentiator)
  │                    │
  │                    └──► Responses-to-Anthropic Translation Slice (Differentiator)
```

- **Inbound WebSocket Ingress** requires existing **Axum server routing** and **InboundAuth**.
- **Turn Execution** requires existing **AccountPool** (for ChatGPT accounts) or HTTP client dispatch.
- **Exact Native Responses Routing** requires **Inbound WebSocket** + **Model Extraction** without translation.
- **Responses-to-Anthropic Translation** requires **Turn Execution** + dedicated bidirectional translation module.

## MVP Recommendation

Prioritize for Immediate Delivery (Phase 0 & Phase 1):
1. **Conformance Test Harness & Fixtures (Phase 0)**:
   - Import focused test transcripts from OpenCodex for warmup, terminal variants, replacement turn cancellation, malformed frames, and header filtering.
2. **Inbound Responses WebSocket Parity (Phase 1)**:
   - Implement Axum WebSocket upgrade on `[server.codex_endpoint]` paths.
   - Local warmup (`generate: false`) evaluation.
   - SSE-to-WS text re-framing with terminal state enforcement (`completed`, `incomplete`, `failed`, `error`).
   - Replacement turn cancellation and disconnect abort.
   - Reuse existing AccountPool and raw passthrough.

Follow-up Priorities (Phase 2 & Phase 3):
1. **Exact Native Responses Routing (Phase 2)**: Route by `model` to native Responses upstreams with byte-preserving passthrough.
2. **Resilience Primitives (Phase 3)**: Granular quota classification and RFC 7231 `Retry-After` date parsing.

Defer:
- **Native Compaction Forwarding (Phase 4)**: Defer until basic WS session streaming is verified in production.
- **Responses-to-Anthropic Translation (Phase 5)**: Defer until native transport parity is solid.
- **Routed Collaboration V2 / Agent State Recovery (Phase 7)**: Defer; opt-in only.

## Sources

- `.planning/PROJECT.md` (Shunt OpenCodex Behavior Port specification and constraints)
- `.planning/notes/opencodex-port-audit.md` (Port/adapt/reject matrix and audit against OpenCodex revision `566debc729bfa20b6ce109ae86b96888e1bdff89`)
- `.planning/codebase/TESTING.md` (Existing test architecture, Wiremock patterns, and test runner configurations)
- `tests/inbound_codex_endpoint.rs` (Existing Shunt raw passthrough and account pool integration tests)
- `/Volumes/PortableSSD/Projects/opencodex/tests/ws-endpoint.test.ts` (OpenCodex inbound WebSocket bridge conformance test cases)
- `/Volumes/PortableSSD/Projects/opencodex/src/server/ws-bridge.ts` (OpenCodex WebSocket bridge implementation and framing contracts)
