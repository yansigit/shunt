# Shunt OpenCodex Behavior Port

## What This Is

Shunt is a lean Rust gateway that exposes Anthropic Messages and OpenAI
Responses-compatible surfaces across multiple upstream providers. This
milestone ports the useful, proven protocol behavior and conformance coverage
from OpenCodex into Shunt without importing OpenCodex's broader platform
complexity.

## Current State

v1 OpenCodex Port shipped on 2026-09-06. Shunt now supports bounded inbound
Responses WebSockets, exact native routing and compaction, strict Anthropic
translation, capability-aware fallback, opt-in collaboration preservation, and
finite process shutdown. All 20 v1 requirements and the repository-wide quality
gate passed.

## Current Milestone: v2 Provider Compatibility

**Goal:** Make Shunt's useful OpenCodex-backed providers protocol-faithful and
streaming-safe while adding the missing transport families without importing
OpenCodex's broader platform complexity.

**Target features:**

- Preserve and verify ChatGPT/Codex plus Gemini / Google Code Assist behavior.
- Harden native Antigravity and Cursor using current upstream evidence.
- Add generic OpenAI Chat Completions compatibility.
- Add Command Code subscription and API-key paths without changing credential
  writeback behavior.
- Evaluate exact OpenCode Go model contracts and implement only the clean,
  testable subset supported by live or captured evidence.

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
- ✓ Native Responses compaction forwards opaque state to verified OpenAI-operated backends without local history persistence — Phase 4
- ✓ A dedicated inbound Responses translation path targets Anthropic with tools, images, reasoning, usage, and terminal fidelity — Phase 5
- ✓ Heterogeneous fallback routes exclude incompatible targets before credential lookup or dispatch — Phase 6
- ✓ Routed Codex collaboration semantics are preserved when explicitly enabled while native traffic remains opaque — Phase 7
- ✓ Graceful shutdown has a bounded drain deadline and cancels remaining turns safely — Phase 8
- ✓ Existing ChatGPT/Codex and generic-Anthropic Vercel paths preserve their verified streaming, authentication, error, retry, continuation, and cancellation behavior behind shared fail-closed bounds — Phase 9

### Active

- [ ] Gemini retains its verified streaming, tool-call, authentication, and
  error behavior under the shared Phase 9 conformance foundation.
- [ ] Antigravity implements the proven Cloud Code Assist request, stream,
  signature, tool-history, model, and destination-security contracts.
- [ ] Cursor's existing provider gains evidence-backed model, continuation,
  tool, error, and retry hardening without speculative no-progress heuristics.
- [ ] OpenAI Chat Completions-compatible upstreams can serve Anthropic Messages
  clients with bounded streaming and tool-call translation.
- [ ] Command Code subscription and API-key users can run normal, streaming,
  tool-heavy, subagent, long-context, and error-path turns without new
  credential persistence behavior.
- [ ] OpenCode Go support is limited to exact model/wire combinations whose
  behavior can be cleanly implemented and verified.

### Out of Scope

- OpenCodex's SQLite request-history and generalized storage platform — unnecessary in Shunt's hot path
- Compatibility Lab, management dashboard, pricing engine, and catalog machinery — outside a lean proxy's role
- Wholesale response-repair modules — repairs require a captured failing transcript
- Duplicate account pools, retry stacks, compression, keepalive, or WebSocket pools — Shunt already owns these concerns
- Durable continuation spill or request-history persistence — deferred until measurements justify it
- Credential-file writeback changes — require a separate explicit approval
- Google AI Studio Web, its cookie/SAPISIDHASH authentication, browser
  extension, daemon, MakerSuite parser, and session-file dependencies — known
  nonfunctional and explicitly excluded
- Whole-provider OpenCode Go claims based on family inference — its models span
  multiple protocols and require exact evidence

## Context

The source audit is `.planning/notes/opencodex-port-audit.md`, based on Shunt
revision `07ac6d4605fbe0953fc7a89fb23b74b85c728abc` and OpenCodex revision
`566debc729bfa20b6ce109ae86b96888e1bdff89`. The audit found that Shunt already
covers many transcript edge cases on its outbound WebSocket and existing
translation path. The highest-value missing boundary is inbound WebSocket
support; model routing, compaction, translated providers, and collaboration
should be layered behind that stable ingress.

The v2 provider audit compared current Shunt behavior with OpenCodex provider
implementations and GitHub issue/PR history. It found that ChatGPT/Codex,
Gemini, Antigravity, Cursor, OpenAI Responses, and Vercel's Anthropic-compatible
path already exist in Shunt. The missing transport families are generic OpenAI
Chat Completions and proprietary Command Code; OpenCode Go remains an
exact-model compatibility evaluation because its catalog spans Chat,
Responses, and Anthropic wires.

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
| Native compaction reuses the inbound passthrough stack | Preserves account, credential, limit, and relay semantics without a second proxy implementation | ✓ Validated in Phase 4 |
| Native compact capability is fail closed by destination | Avoids sending credentials or opaque session state to gateways that merely resemble Responses providers | ✓ Validated in Phase 4 |
| Anthropic translation is a dedicated vertical subsystem | Current translation direction cannot be safely inverted through ad-hoc mutation | ✓ Validated in Phase 5 |
| Capability filtering is internal and preserves the primary | Avoids a generalized manifest platform and never changes the configured primary | ✓ Validated in Phase 6 |
| Collaboration translation is opt-in and fails closed without plaintext | Hidden recovery would be sensitive, billable, and unnecessary for native ChatGPT traffic | ✓ Validated in Phase 7 |
| Shutdown uses one process deadline and runtime cancellation | Bounds drain without adding a global active-turn registry | ✓ Validated in Phase 8 |
| No persistence or repair layer without evidence | Keeps Shunt bounded and avoids speculative complexity | ✓ Validated for v1; deferred requirements remain in v2 |
| Port OpenCodex provider behavior selectively | Its issue history provides valuable wire invariants and fixtures, while its generalized infrastructure does not fit Shunt | ✓ Foundation validated in Phase 9 |
| Keep existing credential writeback behavior unchanged | Provider compatibility does not justify expanding persistence authority | ✓ Preserved in Phase 9 |
| Treat OpenCode Go as exact-model compatibility | The provider spans three wire protocols and has recent model-specific regressions | — Pending |
| Exclude Google AI Studio Web completely | It is known nonfunctional and unrelated to the supported Code Assist transport | ✓ Preserved in Phase 9 |
| Treat provider terminals as authoritative only after complete bounded framing | Prevents malformed input, provider errors, and transport cuts from becoming clean completion | ✓ Validated in Phase 9 |
| Keep replay commitment only at production-reachable semantic redispatch seams | Generic HTTP retry is structurally pre-response; WebSocket fallback and continuation recovery carry actual output/tool evidence | ✓ Validated in Phase 9 |

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
*Last updated: 2026-09-06 after Phase 9*
