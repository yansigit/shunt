# OpenCodex → Shunt Port Audit and Delivery Plan

**Audit date:** 2026-09-05  
**Shunt revision:** `07ac6d4605fbe0953fc7a89fb23b74b85c728abc`  
**OpenCodex revision:** `566debc729bfa20b6ce109ae86b96888e1bdff89`

## Decision

Port OpenCodex's observed protocol behavior and focused conformance cases into
Shunt's existing Rust architecture. Do not port OpenCodex's TypeScript module
layout, storage platform, generalized routing engine, Compatibility Lab, or
repair-module forest.

The safest product boundary is:

```text
Anthropic client
  -> existing /v1/messages path (unchanged)

Codex / Responses client
  -> HTTP/SSE or inbound WebSocket ingress
  -> exact Responses-compatible route
       -> native Responses upstream: byte-preserving passthrough
       -> translated upstream: explicit Responses conversion adapter
```

Native ChatGPT passthrough and cross-provider translation are separate modes.
Opaque native payloads, especially encrypted reasoning and agent-task content,
must not be decoded or rewritten on the native path.

## Current Shunt baseline

Shunt already has substantial Codex support:

- HTTP/SSE ingress at `/backend-api/codex/responses`, `/responses`, and
  `/v1/responses` in `src/codex_endpoint.rs` and `src/server.rs`.
- Raw request and response passthrough with credential/header stripping in
  `src/adapters/responses/inbound.rs`.
- OpenAI-shaped gateway-owned errors at the Codex boundary in `src/error.rs`.
- ChatGPT OAuth account pooling, sticky selection, quota-aware admission, and
  account rotation in `src/adapters/responses/inbound.rs` and
  `src/adapters/responses/pool.rs`.
- Outbound Codex WebSocket v2, HTTP fallback before the first event,
  connection reuse, bounded overflow, liveness probes, cancellation by drop,
  and continuation in `src/adapters/responses/codex_ws.rs` and
  `src/adapters/responses/codex_continuation.rs`.
- Anthropic Messages -> Responses request translation and Responses ->
  Anthropic response translation in `src/model/responses.rs`.
- Existing coverage for text, tools, tool search/reference, images, documents,
  reasoning/encrypted content, incomplete terminals, usage, error
  classification, continuation rejection, early socket close, and bounded
  WebSocket backpressure in `tests/responses_translate.rs`,
  `tests/inbound_codex_endpoint.rs`, and `src/adapters/responses/codex_ws.rs`.

The largest confirmed ingress gap is that Codex clients cannot connect to
Shunt over WebSocket. `src/server.rs` registers POST only, while OpenCodex
accepts `GET /v1/responses` upgrades and bridges Responses frames in
`src/server/index.ts` and `src/server/ws-bridge.ts`.

## Port / adapt / reject matrix

| Capability | Decision | Why | Shunt target |
|---|---|---|---|
| Inbound Responses WebSocket | Port first | Clear protocol and performance gap; Shunt already has most transport primitives | `src/codex_endpoint/`, `src/server.rs` |
| `generate: false` warmup response | Port with WebSocket | Required for faithful client behavior; local and cheap | inbound WS handler |
| Inbound transcript/conformance fixtures | Port selectively | High regression value; avoid duplicating outbound coverage | `tests/inbound_codex_websocket.rs`, shared fixtures |
| Exact model routing to Responses-native providers | Adapt narrowly | Unlocks OpenAI/xAI-compatible targets without translation | `src/codex_endpoint.rs`, `src/routing.rs` |
| `/v1/responses/compact` native forwarding | Adapt | Useful for long sessions; native path can remain opaque | new compact ingress module |
| `compaction_trigger` for routed providers | Later, adapt | Requires translated compaction output and terminal fidelity | translated Responses core |
| Responses -> Anthropic request translation | New subsystem informed by OpenCodex | Strategically valuable but not an inversion-free reuse of current code | new `src/model/inbound_responses/` |
| Anthropic SSE/JSON -> Responses output | New subsystem informed by OpenCodex | Must preserve event ordering, IDs, tools, reasoning, usage, and terminals | same subsystem |
| Granular quota/error classification | Port early | Small, measurable resilience gain; improves failover without storage | `src/upstream_status.rs`, `src/proxy/failover.rs` |
| `Retry-After` date parsing | Port early | Standards-correct and contained | `src/upstream_status.rs` |
| Small ordered-candidate circuit breaker | Adapt after multi-route ingress | Shunt already has static chains/status tracking | existing routing/failover/status modules |
| Full health scoring with SQLite history | Reject initially | High operational cost; duplicates metrics and in-memory status | none |
| Capability-aware candidate filtering | Adapt only for declared capabilities | Useful once heterogeneous fallback exists; avoid a dynamic manifest platform | typed config + routing |
| Bounded shutdown deadline and turn cancellation | Port independently | Current `src/shutdown.rs` explicitly allows SSE to drain forever | `src/shutdown.rs`, request admission guard |
| `additional_tools`, `tool_search`, `tool_reference` preservation | Adapt when translated subagents are enabled | Native ChatGPT passthrough already handles these | translated Responses adapter |
| Routed V2 collaboration bridge | Later, opt-in | Only needed when subagents leave native ChatGPT | dedicated optional module |
| Encrypted agent-task recovery | Experimental last | Sensitive, billable, cache-bound, and unnecessary for native passthrough | optional module with strict gates |
| Durable continuation spill/history | Reject until measured | Shunt's native continuation is connection-scoped and bounded | none |
| Compatibility Lab/dashboard/catalog platform | Reject | Outside a lean proxy's purpose | none |
| Provider-specific repair modules | Case-by-case only | Add only for a captured failing transcript | closest protocol boundary |

## Delivery phases

### Phase 0 — provenance and transcript corpus

Goal: establish a clean behavioral specification before implementation.

Work:

1. Record the source OpenCodex revision beside every imported fixture or
   behavior note.
2. Prefer independently written Rust implementations from observed wire
   behavior. If substantial OpenCodex code is translated, retain its MIT
   copyright and permission notice in a third-party notice file.
3. Create compact JSON/SSE/WS fixtures for only the missing cases:
   `response.create`, warmup, terminal variants, malformed frames, replacement
   turn cancellation, socket close, and backpressure.
4. Inventory overlap against existing Shunt tests before adding each case.

Exit gate:

- A test matrix labels every case `existing`, `new`, or `not applicable`.
- No fixture contains credentials, account IDs, or captured user content.

### Phase 1 — inbound WebSocket parity

Goal: let a Codex client use WebSocket v2 against Shunt while keeping HTTP/SSE
behavior unchanged.

Design:

- Register GET upgrade handling on the same configured Codex paths without
  changing POST routing.
- Authenticate before upgrade using the same endpoint rules and client
  namespace used by HTTP ingress.
- Accept text `response.create` frames; reject binary or malformed frames with
  OpenAI/Responses protocol errors.
- Treat `response.processed` as an acknowledgement and ignore it.
- Complete `generate: false` locally with deterministic created/completed
  frames and no upstream request.
- Permit one active turn per socket. A newer `response.create` cancels the old
  turn; disconnect cancels the upstream stream and releases admission.
- Bridge upstream SSE events to WS text frames using bounded frame parsing and
  bounded outbound capacity. Terminal events are `completed`, `incomplete`,
  `failed`, and `error`.
- Reuse existing inbound account selection and raw passthrough. Do not route to
  the outbound `codex_ws` client merely because ingress is a WebSocket.

Tests:

- upgrade enabled/disabled, auth rejection, origin/header sanitation;
- warmup creates no upstream call;
- created/delta/completed ordering and incomplete/failed fidelity;
- close before first output vs. after first output;
- malformed JSON, missing type, binary frame, oversized frame;
- replacement turn and socket-close cancellation;
- slow client backpressure never creates an unbounded queue;
- admission and account leases release on every terminal/drop path.

Exit gate:

- HTTP/SSE tests remain byte-for-byte unchanged.
- WebSocket tests run through a real Axum listener and mocked upstream.
- Memory bounds are explicit constants and covered by pressure tests.

Documentation in the same PR: `README.md` plus translations if the capability
is advertised there, `docs/m11-inbound-codex-endpoint.md`, a new milestone/spec
or amendment for inbound WS, and the English/ko/ja/zh-cn site Codex endpoint and
CLI pages. Do not edit `wiki/`.

### Phase 2 — exact native Responses routing

Goal: route inbound Responses requests by exact configured model to upstreams
that already speak Responses, without translating their bodies or streams.

Design:

- Preserve `[server.codex_endpoint].provider` as the default route.
- Extract `model` once from the bounded body and resolve only exact configured
  aliases/routes whose adapter kind is Responses-compatible.
- Fail closed before dispatch if the selected route is not Responses-native.
- Preserve request body and successful/error stream bytes. Only credentials,
  hop-by-hop headers, and gateway-owned error envelopes may change.
- Retry/fail over only before the first response event or body byte. Never
  replay after observable output.

Approval gate:

- This changes documented provider semantics and likely public config behavior;
  obtain explicit user approval before implementation.

Tests:

- exact match, alias match, default fallback, incompatible adapter rejection;
- OpenAI, ChatGPT, and xAI flavor URL/auth/header differences;
- no mid-stream route hop; byte equality for request and response fixtures;
- unknown model and disabled/reloaded endpoint behavior.

### Phase 3 — resilience primitives

Goal: improve routing decisions without importing generalized policy machinery.

Work:

- Separate transient request-rate limits from hard quota exhaustion using
  structured provider codes and messages.
- Parse integer and HTTP-date `Retry-After` values.
- Add a typed hop/stop decision that prevents retry on cancellation, invalid
  requests, context overflow, and policy rejection, while allowing explicit
  model-retirement and provider/quota failures.
- Reuse Shunt's in-memory upstream status and metrics. Add only a small
  cooldown deadline, recent terminal outcome, and incomplete-stream count if
  measurements show they change candidate choice.

Approval gate:

- Any new config key or change to documented failover semantics requires
  explicit approval. Start with internal defaults and observability where
  possible.

### Phase 4 — native compaction

Goal: support long Codex sessions without adopting general persistence.

Sequence:

1. Proxy `/v1/responses/compact` to native ChatGPT/OpenAI-compatible compact
   endpoints, preserving opaque encrypted compaction items.
2. Add routing tests for canonical native providers and explicit unsupported
   errors elsewhere.
3. Only after translated Responses exists, implement routed
   `compaction_trigger` by generating the exact synthetic compaction output
   expected by Codex.

Do not decrypt provider-owned compaction blobs. Do not add SQLite, durable
history, or background maintenance for this phase.

### Phase 5 — Responses-to-Anthropic vertical slice

Goal: let Codex use Anthropic through Shunt with one deliberately narrow,
testable bidirectional adapter.

Implementation order:

1. Non-streaming text request/response.
2. Streaming created/output-text/completed event sequence.
3. Function tools, argument fragments, tool results, stable call/item IDs.
4. Images and document inputs.
5. Reasoning summaries and encrypted/opaque state policy.
6. Usage accounting and `completed`/`incomplete`/`failed` fidelity.
7. Cancellation, disconnect, and pre-stream failover.

Rules:

- Keep the existing Anthropic-client path untouched.
- Use a dedicated inbound Responses model, not ad-hoc mutation of
  `src/model/responses.rs`.
- Make unsupported inputs explicit errors; never silently drop tools, images,
  or encrypted state.
- Add repair logic only after a minimal captured transcript demonstrates the
  failure and the fixture accompanies the fix.

Exit gate:

- Golden non-streaming and event-by-event streaming fixtures agree on final
  semantic output.
- Tool, reasoning, image, cancellation, and terminal-state matrices pass.
- No queue or collected tool-argument buffer is unbounded.

### Phase 6 — capability filtering and additional providers

Goal: expand only after the Anthropic slice is stable.

- Declare a small typed capability set per route: Responses-native,
  translated, tools, images, reasoning, compact, and maximum context.
- Filter incompatible candidates before dispatch and expose a concise reason.
- Add Gemini only as a separate vertical slice with the same conformance suite.
- Avoid importing OpenCodex's dynamic manifest evaluator and Compatibility Lab.

### Phase 7 — routed collaboration V2

Goal: preserve Codex multi-agent semantics when a child turn is routed away
from native ChatGPT.

- Preserve `additional_tools` and agent roster/guidance items.
- Support `tool_search`/`tool_reference` and subagent fallback selection.
- Keep native ChatGPT traffic opaque and unchanged.
- Treat encrypted agent-task recovery as a separate experimental subphase:
  opt-in, authenticated, size/time bounded, zero plaintext persistence,
  bounded single-flight cache, and explicit billable recovery semantics.

This phase is not a prerequisite for ordinary Codex requests or native
ChatGPT subagents.

### Phase 8 — bounded shutdown

Goal: ensure an abandoned SSE/WS turn cannot prevent process shutdown forever.

- Add one absolute drain deadline.
- Stop new admission after the first signal.
- At the deadline, cancel remaining HTTP/WS turns, close transports, and allow
  normal RAII cleanup to release account and concurrency leases.
- Preserve the existing second-signal immediate-exit escape hatch.

This can ship independently once active-turn ownership is explicit. It does
not require OpenCodex's lease framework or storage shutdown hooks.

## Prioritized transcript matrix

| Case | Shunt status | Action |
|---|---|---|
| HTTP raw passthrough and headers | Covered | Retain |
| Gateway-owned OpenAI error shape | Covered | Retain |
| Account rotation and refresh | Covered | Extend only for new route modes |
| Outbound WS early close | Covered | Do not duplicate |
| Outbound rejected continuation | Covered | Do not duplicate |
| Outbound bounded backpressure/control frames | Covered | Reuse principles |
| Incomplete Responses terminal translation | Covered outbound | Add inbound WS fidelity case |
| Inbound WS warmup | Missing | Add in Phase 1 |
| Inbound WS cancellation/replacement | Missing | Add in Phase 1 |
| Inbound WS malformed/oversized frames | Missing | Add in Phase 1 |
| Missing/duplicate item IDs | Partly covered in translator/tool-search tests | Add only when translated ingress needs repair |
| Tool argument fragmentation | Covered in existing translation paths | Add inverse-direction cases in Phase 5 |
| Encrypted reasoning replay | Covered outbound | Add opaque-policy cases in Phase 5/7 |
| Compact v1/v2 terminals | Missing | Add in Phase 4 |
| Shutdown with never-ending stream | Missing | Add in Phase 8 |

## Explicit non-goals

- No SQLite request-history database in the proxy hot path.
- No management dashboard, pricing/cost engine, catalog system, MCP/A2A
  sidecars, or Compatibility Lab.
- No universal provider abstraction before one Anthropic vertical slice works.
- No response repair without a failing fixture.
- No durable continuation or request-history persistence without measurements.
- No duplicate account-pool, retry, compression, keepalive, or WebSocket-pool
  implementation.
- No changes to credential-file writeback without explicit approval.

## Recommended first delivery

Ship Phase 0 and Phase 1 together: selective conformance fixtures plus inbound
WebSocket parity. Follow with Phase 2 only after approval of the routing/config
semantics. In parallel, Phase 3's internal quota and `Retry-After` improvements
are a contained resilience win. Do not begin cross-provider translation until
these transport and routing boundaries are stable.

