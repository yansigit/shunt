# Research Summary

**Project:** Shunt OpenCodex Behavior Port
**Domain:** Protocol-faithful, bounded AI gateway behavior
**Researched:** 2026-09-05

## Executive Summary

The audit and four research dimensions agree that Shunt should port OpenCodex's
observable protocol behavior and high-value fixtures, not its TypeScript platform
architecture. Shunt already has the difficult upstream primitives—typed Rust
configuration, ChatGPT OAuth pools, outbound Codex WS, streaming passthrough,
translation, limits, and broad integration coverage. The clearest missing boundary
is an inbound Responses WebSocket server.

The first phase should be a production-quality vertical slice using existing
Axum/Tokio facilities: authenticate before upgrade, accept bounded
`response.create` frames, synthesize warmup completions, reuse the existing inbound
HTTP/account-pool path, relay upstream SSE payloads as WS text, preserve terminal
and error semantics, and cancel replaced/disconnected turns. This requires no new
runtime subsystem and should not change public routing configuration.

## Key Findings

### Stack

- Existing Axum, Tokio, `http-body-util`, Serde, and WebSocket dependencies are
  sufficient; avoid a new SSE framework or second connection pool.
- Awaiting each WS send and using only bounded internal channels provides natural
  backpressure.
- Native Responses payloads should remain opaque except for the minimal control and
  event `type` fields needed at the transport boundary.

### Features

- Table stakes are auth-before-upgrade, local `generate:false` warmup, ack handling,
  one active turn, replacement/disconnect cancellation, exact terminal framing,
  safe error headers, and explicit resource bounds.
- JSON-success synthesis, native-provider routing, and compaction are useful but are
  not needed to prove the initial streaming transport.
- Focused conformance fixtures should be derived from OpenCodex transcripts while
  avoiding duplication of Shunt's mature HTTP passthrough coverage.

### Architecture

- Add `src/codex_endpoint/websocket.rs` beneath the existing endpoint module and
  register GET+POST on the same opt-in paths.
- Reuse endpoint auth, request limits, sticky pool selection, telemetry, and the
  existing `forward_codex_inbound` path through narrowly exposed helpers.
- Use a turn generation plus abortable task to prevent stale frames and release the
  upstream response body on replacement or disconnect.

### Pitfalls

- The principal hazards are authenticating too late, blocking inbound frame reads
  during a live turn, unbounded delimiter/frame buffers, treating `[DONE]` as a
  terminal success, reconstructing payload JSON, double-sending terminal errors,
  and writing timing-only cancellation tests.
- Routing changes, credential writeback, generalized repairs, persistent history,
  and collaboration recovery are explicitly outside Phase 1.

## Recommended Roadmap

1. **Inbound Responses WebSocket parity and conformance** — complete the missing
   transport boundary without public config changes.
2. **Exact native Responses routing** — approval-gated because it changes documented
   provider semantics.
3. **Quota and Retry-After resilience** — distinguish transient limits and hard
   exhaustion using bounded inspection.
4. **Native Responses compaction** — opaque forwarding without persistence.
5. **Responses-to-Anthropic translation** — isolated vertical subsystem with tools,
   images, reasoning, usage, and terminal fidelity.
6. **Capability-aware fallback** — reject incompatible heterogeneous targets before
   dispatch.
7. **Opt-in collaboration preservation** — only after native and translated paths
   are stable.
8. **Bounded graceful shutdown** — unify admission stop, drain deadline, and forced
   cancellation across streaming transports.

## Phase 1 Scope

### Include

- GET upgrade on all existing configured Responses paths
- handshake authentication
- bounded control-frame parsing and ack handling
- local warmup frames
- streaming SSE-to-WS event relay
- completed/failed/incomplete/error semantics
- replacement and disconnect cancellation
- safe header projection
- unit and real-socket integration tests
- required English and maintained translated documentation

### Defer

- JSON 200 success-to-event synthesis
- new provider selection behavior or config keys
- compaction
- cross-provider translation
- collaboration recovery
- persistent history and general response repair

## Success Standard

Phase 1 is complete only when tests demonstrate authenticated live delivery and
resource teardown, the existing HTTP endpoint remains unchanged, all buffers are
explicitly bounded, documentation is synchronized, and repository format, Clippy,
and full workspace tests pass.

## Sources

- `.planning/notes/opencodex-port-audit.md`
- `.planning/research/STACK.md`
- `.planning/research/FEATURES.md`
- `.planning/research/ARCHITECTURE.md`
- `.planning/research/PITFALLS.md`
- `.planning/codebase/` maps
- OpenCodex `src/server/ws-bridge.ts`, `src/server/index.ts`, and WebSocket tests
