# Requirements: Shunt OpenCodex Behavior Port

**Defined:** 2026-09-05
**Core Value:** Codex and Anthropic clients receive protocol-faithful, streaming-safe behavior while Shunt stays bounded, predictable, and operationally lean.

## v1 Requirements

### Inbound Responses WebSocket

- [x] **WS-01**: A client can upgrade any configured inbound Responses path to WebSocket, and configured client authentication is enforced before the upgrade succeeds.
- [x] **WS-02**: A `response.create` frame with `generate: false` completes locally with deterministic `response.created` and `response.completed` frames and causes no upstream request.
- [x] **WS-03**: A normal `response.create` frame is forwarded through the existing ChatGPT OAuth account-pool path as a streaming Responses request, while `response.processed` acknowledgements are harmless no-ops.
- [x] **WS-04**: Upstream SSE data payloads are delivered as byte-faithful WebSocket text frames through the first `response.completed`, `response.failed`, or `response.incomplete` terminal event.
- [x] **WS-05**: Handshake, gateway, upstream, malformed-stream, and premature-EOF failures use the correct HTTP or WebSocket OpenAI Responses error envelope and expose only safe response headers.
- [x] **WS-06**: A replacement turn or disconnected socket promptly cancels the active upstream turn, releases its resources, and cannot leak stale frames into a newer turn.
- [x] **WS-07**: Client frames, SSE events, and downstream delivery are explicitly bounded and respect backpressure without buffering an upstream SSE response.

### Protocol Conformance

- [x] **CONF-01**: Focused Rust fixtures cover OpenCodex-proven warmup, frame splitting, CRLF/multiline SSE, terminal variants, malformed input, premature EOF, and safe-header behavior without duplicating existing Shunt tests.
- [x] **CONF-02**: End-to-end tests cover authenticated upgrade, all registered inbound paths, live event delivery, replacement/disconnect cancellation, and disabled-endpoint behavior.

### Native Responses Routing

- [x] **ROUTE-01**: An inbound Responses request can resolve an exact configured model route to a Responses-native provider without altering the request body.
- [x] **ROUTE-02**: Native Responses passthrough rejects ambiguous, translated-only, or incompatible targets before dispatch and preserves the existing pinned ChatGPT endpoint behavior as the compatibility default.

### Resilience

- [x] **RES-01**: Shunt distinguishes transient request-rate limiting from hard quota exhaustion using provider status, codes, and bounded body inspection.
- [x] **RES-02**: Retry scheduling honors both delta-seconds and standards-compliant HTTP-date `Retry-After` values without wall-clock underflow or unbounded cooldowns.

### Compaction

- [ ] **COMP-01**: Compatible native Responses routes can perform `/v1/responses/compact` without local request-history persistence or inspection of opaque continuation state.

### Responses-to-Anthropic Translation

- [ ] **TRANS-01**: A dedicated inbound translator maps Responses instructions, messages, tools, tool results, images, and reasoning controls to an Anthropic Messages request without modifying native passthrough paths.
- [ ] **TRANS-02**: Anthropic streaming output maps to Responses event ordering with tool-call, reasoning, usage, completion, failure, and incomplete terminal fidelity.
- [ ] **TRANS-03**: Unsupported or lossy features fail before dispatch with an actionable Responses error instead of silent degradation.

### Compatibility and Collaboration

- [ ] **CAP-01**: Heterogeneous fallback routes exclude targets that cannot satisfy required tools, images, structured output, reasoning effort, or known context constraints.
- [ ] **COLLAB-01**: When collaboration routing is explicitly enabled, task metadata and encrypted/opaque continuation state survive routing without affecting native ChatGPT traffic.

### Operations

- [ ] **OPS-01**: Graceful shutdown stops admission, drains active HTTP/SSE/WebSocket turns for a bounded deadline, then cancels remaining work and releases resources.

## v2 Requirements

### Evidence-Driven Repairs

- **REPAIR-01**: Add a narrowly scoped response repair only when a captured failing transcript demonstrates a client compatibility defect.
- **RECOVERY-01**: Add encrypted recovery-token or durable continuation support only when production evidence shows connection-scoped state is insufficient.

## Out of Scope

| Feature | Reason |
|---------|--------|
| SQLite request-history and generalized persistence | Adds migrations and hot-path I/O without supporting the core proxy value. |
| Compatibility Lab, management dashboard, pricing engine, and catalog platform | These are broader product surfaces rather than protocol gateway behavior. |
| Wholesale response-repair framework | Repairs without a failing transcript are speculative and brittle. |
| Duplicate account, retry, compression, keepalive, or WebSocket pools | Shunt already owns these concerns. |
| Credential-file writeback changes | Explicitly excluded and require separate user approval. |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| WS-01 | Phase 1 | Complete |
| WS-02 | Phase 1 | Complete |
| WS-03 | Phase 1 | Complete |
| WS-04 | Phase 1 | Complete |
| WS-05 | Phase 1 | Complete |
| WS-06 | Phase 1 | Complete |
| WS-07 | Phase 1 | Complete |
| CONF-01 | Phase 1 | Complete |
| CONF-02 | Phase 1 | Complete |
| ROUTE-01 | Phase 2 | Complete |
| ROUTE-02 | Phase 2 | Complete |
| RES-01 | Phase 3 | Complete |
| RES-02 | Phase 3 | Complete |
| COMP-01 | Phase 4 | Pending |
| TRANS-01 | Phase 5 | Pending |
| TRANS-02 | Phase 5 | Pending |
| TRANS-03 | Phase 5 | Pending |
| CAP-01 | Phase 6 | Pending |
| COLLAB-01 | Phase 7 | Pending |
| OPS-01 | Phase 8 | Pending |

**Coverage:**

- v1 requirements: 20 total
- Mapped to phases: 20
- Unmapped: 0 ✓

---
*Requirements defined: 2026-09-05*
*Last updated: 2026-09-05 after OpenCodex port research*
