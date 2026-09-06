# Phase 2: Native Responses Routing - Context

**Gathered:** 2026-09-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Allow the existing opt-in inbound Responses HTTP and WebSocket surfaces to select a Responses-native upstream from an exact configured model mapping. Native request and response payloads remain opaque; unmatched traffic continues to use `[server.codex_endpoint].provider`. This phase does not add translation, prefix routing, compaction, capability negotiation, persistence, or new public configuration keys.

</domain>

<decisions>
## Implementation Decisions

### Selection Contract

- **D-01:** Preserve `[server.codex_endpoint].provider` as the compatibility default. Missing, malformed, or unmatched model values retain the Phase 1 pinned-provider behavior.
- **D-02:** Only exact existing configuration declarations may override the pinned provider. `[[route_prefixes]]` do not participate in inbound Responses routing.
- **D-03:** An exact selection must resolve uniquely to a Responses-native provider. Ambiguous chains and providers whose adapter is not Responses-compatible fail before any upstream request.
- **D-04:** Reuse the existing `[models.upstream_model]` and legacy `[[routes]]` configuration vocabulary; add no new routing key or endpoint toggle.

### Native Fidelity

- **D-05:** Provider selection may inspect the bounded decoded request only far enough to classify `model`; it must not rewrite the original request body, including the `model` field. A configured mapping that requires model translation is incompatible with native passthrough and must fail closed.
- **D-06:** HTTP successes and upstream errors remain byte-preserving. WebSocket transport continues Phase 1's bounded SSE-to-WebSocket framing without semantic translation.
- **D-07:** Credential and hop-by-hop header filtering remain gateway responsibilities; native payload opacity does not permit client credentials or cookies to cross provider boundaries.

### Dispatch and Compatibility

- **D-08:** HTTP and WebSocket live turns use one shared route-resolution policy so the same model cannot select different providers by transport.
- **D-09:** Do not replay or hop providers after any response body byte or WebSocket response event becomes observable. Existing provider-local account rotation may continue before output.
- **D-10:** Existing installations with no exact eligible route must behave identically to Phase 1, including hot-reloaded changes within the already-enabled endpoint and the boot-time endpoint opt-in.

### Verification and Documentation

- **D-11:** Prove exact route, exact alias, pinned fallback, ambiguous mapping, incompatible adapter, compressed-body inspection, HTTP/WS parity, byte equality, and no-mid-stream-hop behavior with focused Rust tests.
- **D-12:** Update M11 and every maintained README/Nimbus locale because the model-selection contract is observable. Do not hand-edit `wiki/`.

### Claude's Discretion

The planner may choose the internal resolver type, error-code names, and module placement, provided the selection and byte-fidelity contract above remains explicit and shared by both transports.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Scope and requirements

- `.planning/ROADMAP.md` § Phase 2 — goal, dependencies, and success criteria.
- `.planning/REQUIREMENTS.md` § Native Responses Routing — ROUTE-01 and ROUTE-02.
- `.planning/notes/opencodex-port-audit.md` § Phase 2 — approved routing design, tests, and replay boundary.
- `.planning/phases/01-inbound-responses-websocket/01-CONTEXT.md` — Phase 1 transport, lifecycle, fidelity, and deferred-routing decisions that remain binding.

### Existing public contracts

- `docs/m11-inbound-codex-endpoint.md` — current pinned-provider HTTP/SSE/WebSocket behavior that becomes the compatibility fallback.
- `docs/codex-configuration.md` — existing model-map, exact-route, provider, and inbound endpoint configuration semantics.
- `docs/upstreams-failover.md` — existing ordered-chain and pre-output failover guarantees; Phase 2 must not silently import prefix or translated-chain behavior.
- `site/src/content/docs/reference/configuration.md` — published configuration precedence and exact-route documentation requiring synchronized changes.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `src/routing.rs`: Existing exact model-map and `[[routes]]` resolution, provider adapter classification, `[1m]` normalization, and route metadata.
- `src/codex_endpoint.rs`: Bounded inbound body read, compressed model inspection, authentication, sticky-key construction, metrics, and shared `forward_turn` boundary.
- `src/codex_endpoint/websocket.rs`: Phase 1 live-turn dispatch, cancellation, generation filtering, and bounded SSE relay.
- `src/adapters/responses/inbound.rs`: Native inbound Responses URL/auth/header forwarding for ChatGPT, OpenAI, and xAI flavors.
- `src/config.rs`: Typed provider/model/route validation and hot-reload snapshots.

### Established Patterns

- Resolve policy before adapter dispatch, then pass a typed `Route` into the adapter.
- Preserve successful streaming bodies lazily and hold admission guards until termination or disconnect.
- Convert only gateway-owned inbound Codex failures into OpenAI Responses error shape.
- Capture one `AppState` snapshot per turn so hot reload cannot change routing mid-request.

### Integration Points

- Replace the pinned provider construction in `codex_endpoint::forward`/`forward_turn` with a shared exact-native resolver while retaining pinned fallback.
- Feed the same resolver from WebSocket `response.create` after transport-only normalization.
- Validate exact mapping eligibility using existing `AdapterKind::Responses` and provider auth/flavor metadata before calling `forward_codex_inbound`.

</code_context>

<specifics>
## Specific Ideas

Port OpenCodex's observable exact-native routing behavior and test matrix, not its generalized routing/platform architecture. Prefer a small typed decision result such as pinned, selected, or rejected over parallel endpoint-specific branching.

</specifics>

<deferred>
## Deferred Ideas

- Prefix and heterogeneous capability-aware fallback remain Phase 6 work.
- `/v1/responses/compact` remains Phase 4 work.
- Responses-to-Anthropic translation remains Phase 5 work.
- Quota classification and retry timing remain Phase 3 work.

</deferred>

---

*Phase: 02-native-responses-routing*
*Context gathered: 2026-09-05*
