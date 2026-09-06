# Shunt OpenCodex Behavior Port

## What This Is

Shunt is a lean Rust gateway that exposes Anthropic Messages and OpenAI
Responses-compatible surfaces across multiple upstream providers. This
milestone ports the useful, proven protocol behavior and conformance coverage
from OpenCodex into Shunt without importing OpenCodex's broader platform
complexity.

## Core Value

Codex and Anthropic clients must receive protocol-faithful, streaming-safe
behavior while Shunt stays bounded, predictable, and operationally lean.

## Requirements

### Validated

- ✓ Anthropic Messages ingress routes through typed provider adapters — existing
- ✓ Inbound Codex/OpenAI Responses HTTP and SSE passthrough — existing
- ✓ ChatGPT/Codex OAuth account pools with rotation and quota admission — existing
- ✓ Outbound Codex WebSocket v2 with continuation and pre-output HTTP fallback — existing
- ✓ Anthropic-to-Responses request and Responses-to-Anthropic response translation — existing
- ✓ Bounded request, decompression, concurrency, and WebSocket resources — existing
- ✓ OpenAI-shaped gateway-owned errors on the inbound Codex endpoint — existing
- ✓ Codex clients can use inbound Responses WebSocket transport with faithful terminal, error, cancellation, warmup, and backpressure semantics — Phase 1
- ✓ Missing OpenCodex protocol transcript cases are represented as focused Rust fixtures without duplicating existing Shunt coverage — Phase 1
- ✓ Inbound Responses can route exactly to compatible native Responses providers while preserving byte-level passthrough behavior — Phase 2
- ✓ Transient rate limiting is distinguished from exact hard-quota exhaustion and `Retry-After` scheduling is standards-compatible and bounded — Phase 3

### Active

- [ ] Native Responses compaction works without adding general request-history persistence
- [ ] A dedicated inbound Responses translation path can target Anthropic with tools, images, reasoning, usage, and terminal fidelity
- [ ] Heterogeneous routes reject incompatible capabilities before dispatch
- [ ] Routed Codex collaboration semantics are preserved when explicitly enabled
- [ ] Graceful shutdown has a bounded drain deadline and cancels remaining turns safely

### Out of Scope

- OpenCodex's SQLite request-history and generalized storage platform — unnecessary in Shunt's hot path
- Compatibility Lab, management dashboard, pricing engine, and catalog machinery — outside a lean proxy's role
- Wholesale response-repair modules — repairs require a captured failing transcript
- Duplicate account pools, retry stacks, compression, keepalive, or WebSocket pools — Shunt already owns these concerns
- Durable continuation spill or request-history persistence — deferred until measurements justify it
- Credential-file writeback changes — require a separate explicit approval

## Context

The source audit is `.planning/notes/opencodex-port-audit.md`, based on Shunt
revision `07ac6d4605fbe0953fc7a89fb23b74b85c728abc` and OpenCodex revision
`566debc729bfa20b6ce109ae86b96888e1bdff89`. The audit found that Shunt already
covers many transcript edge cases on its outbound WebSocket and existing
translation path. The highest-value missing boundary is inbound WebSocket
support; model routing, compaction, translated providers, and collaboration
should be layered behind that stable ingress.

Directly translated substantial OpenCodex code must retain its MIT notice.
Clean Rust implementations based on observed behavior and independently
written tests are preferred.

## Constraints

- **Architecture**: Keep `/v1/messages` and the existing Claude hot path unchanged — Codex additions belong at the Responses boundary.
- **Streaming**: Never buffer an upstream SSE response unless the client requested non-streaming output.
- **Resources**: Every queue, frame buffer, decompression path, cache, and recovery loop must have an explicit bound.
- **Errors**: Gateway-owned Codex ingress errors use OpenAI Responses shape; other gateway surfaces retain their existing error contracts.
- **Configuration**: Ask before adding public config keys or changing documented provider semantics.
- **Credentials**: Ask before changing credential-file writeback behavior.
- **Documentation**: Observable behavior changes update README/docs/site English and ko/ja/zh-cn copies in the same PR; never hand-edit `wiki/`.
- **Quality**: Preserve existing tests and run format, Clippy, and the full workspace test suite before completing code phases.
- **Licensing**: OpenCodex is MIT; retain notices for substantial translated material.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Port behavior and tests, not the TypeScript architecture | Preserves proven semantics without importing generalized infrastructure | ✓ Validated in Phases 1–2 |
| Inbound WebSocket is the first delivery slice | It is the clearest missing Codex transport boundary and reuses Shunt primitives | ✓ Validated in Phase 1 |
| Native passthrough and cross-provider translation stay separate | Native encrypted/opaque traffic must remain untouched | ✓ Validated in Phase 2 |
| Exact Responses-native routing precedes universal translation | Delivers provider choice with a smaller failure surface | ✓ Validated in Phase 2 |
| Hard quota requires bounded exact structured evidence | Prevents ambiguous or attacker-controlled error text from suppressing healthy accounts | ✓ Validated in Phase 3 |
| Quota inspection is bounded by bytes and total time | Prevents slow error bodies from occupying the pre-stream failover boundary while preserving relay fidelity | ✓ Validated in Phase 3 |
| Anthropic translation is a dedicated vertical subsystem | Current translation direction cannot be safely inverted through ad-hoc mutation | — Pending |
| Collaboration recovery is opt-in and last | It is sensitive, billable, and unnecessary for native ChatGPT traffic | — Pending |
| No persistence or repair layer without evidence | Keeps Shunt bounded and avoids speculative complexity | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition:**
1. Move validated requirements from Active to Validated with a phase reference.
2. Move invalidated requirements to Out of Scope with the reason.
3. Add newly discovered requirements only after scope review.
4. Record consequential decisions and whether they proved sound.
5. Update the project description if the product boundary changes.

**After each milestone:**
1. Review the full requirement set and exclusions.
2. Confirm protocol fidelity remains the core priority.
3. Revisit deferred work only with evidence from usage or failing transcripts.

---
*Last updated: 2026-09-05 after Phase 3*
