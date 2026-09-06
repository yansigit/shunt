# Phase 2: Native Responses Routing - Research

**Researched:** 2026-09-05
**Domain:** Rust/Axum inbound OpenAI Responses routing and opaque passthrough
**Confidence:** HIGH

## User Constraints

Copy of the locked phase decisions from `02-CONTEXT.md`:

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

### Deferred Ideas

- Prefix and heterogeneous capability-aware fallback remain Phase 6 work.
- `/v1/responses/compact` remains Phase 4 work.
- Responses-to-Anthropic translation remains Phase 5 work.
- Quota classification and retry timing remain Phase 3 work.

## Summary

Phase 2 is a narrow routing boundary, not a new provider abstraction. The current inbound POST and WebSocket handlers snapshot `AppState`, read a bounded body, and dispatch through a hard-coded `server.codex_endpoint.provider`; the HTTP path already relays request/response bytes lazily and strips gateway-owned credentials/headers. The existing generic resolver (`src/routing.rs`) knows exact model maps, legacy exact routes, prefix routes, ordered chains, `[1m]` normalization, and adapter kinds, but it is deliberately broader than this phase. A native inbound resolver must therefore be a separate policy entry point (or a strongly constrained wrapper) that ignores prefixes, permits only one exact eligible route, and falls back to the pinned endpoint provider when no eligible exact declaration exists. **[VERIFIED: src/codex_endpoint.rs:245-297; src/codex_endpoint/websocket.rs:196-212; src/routing.rs:122-177]**

The largest implementation seam is credential/adapter dispatch. `forward_codex_inbound` is currently a ChatGPT OAuth account-pool path and `passthrough_send` knows several credential variants, but account discovery is hard-coded to the ChatGPT store family. Exact routes to OpenAI/xAI-compatible `Responses` providers need an explicit, provider-aware credential path while retaining the same byte-relay and safe-header rules. A selected route must be rejected before any upstream request if its adapter is not `Responses`, if it expands to an ambiguous chain, or if its mapped upstream model differs from the client model (model rewriting would violate native opacity). No retry or provider hop is allowed after an observable body byte/WS event. **[VERIFIED: src/adapters/responses/inbound.rs:36-69,402-454; src/routing.rs:9-45; .planning/phases/02-native-responses-routing/02-CONTEXT.md D-03,D-05,D-09]**

**Primary recommendation:** Add one typed `resolve_native_inbound(config, model)` decision used by both HTTP and WebSocket turns; have it return pinned, selected-single-native, or rejected, and pass the resulting `Route` into a provider-aware raw passthrough dispatcher. Keep `src/routing.rs` as the source of existing configuration precedence, but do not reuse prefix/fallback-chain semantics for this endpoint. **[RECOMMENDED: derived from locked D-02,D-03,D-08]**

## Existing Architecture (verified)

| Area | Current source of truth | Planning implication |
|---|---|---|
| Endpoint opt-in and paths | `src/codex_endpoint.rs:33-45` defines `PATHS` as `"/backend-api/codex/responses"`, `"/responses"`, `"/v1/responses"`; `src/server.rs:255-263` registers GET+POST only when configured | Keep path registration and disabled/hot-reload behavior unchanged. |
| Pinned provider | `src/config.rs:1225-1244` defines `CodexEndpointConfig.provider`, defaulting to `"codex"`; validation at `src/config.rs:3671-3684` requires an existing provider with `auth = "chatgpt_oauth"` | Preserve this as fallback and retain its ChatGPT pool semantics. |
| Generic route vocabulary | `src/config.rs:1971-1994` defines `RouteConfig { model, provider, upstream_model, effort, service_tier }`, `ModelConfig { id, display_name, upstream_model }`, and `RoutePrefixConfig { prefix, provider }` | No new public keys. Exact route and model-map declarations are the only inputs to the native resolver. |
| Generic precedence | `src/routing.rs:122-177` strips `[1m]`, checks `models`, then `routes`, then `route_prefixes`, then `default_provider`; `resolve_model_chain` can return multiple routes | Native policy must call/use only exact portions and reject multi-route chains; never use prefix matching. |
| Adapter classification | `src/routing.rs:9-35` defines `AdapterKind::{Anthropic, Responses, Cursor, Gemini, AntigravityCli}`; `ProviderKind::Responses` is quoted at `src/config.rs:1744-1755` as “OpenAI Responses API” | Eligibility is `AdapterKind::Responses`, not a provider-name allowlist. |
| Body/model inspection | `src/codex_endpoint.rs:47-150` has allocation-bounded `ModelView`/`ModelField`; `src/codex_endpoint.rs:385-487` decodes zstd only for inspection and leaves original bytes untouched; `parse_model` is at `src/codex_endpoint.rs:551-563` | Reuse this extraction or a shared equivalent. Missing/malformed/non-string model must select pinned fallback, not block the request. Compressed-body tests must assert upstream receives identical compressed bytes/header. |
| HTTP dispatch | `src/codex_endpoint.rs:245-297` authenticates, bounds/read body, labels model, creates sticky key, and calls `forward_turn`; `forward_turn` currently creates a `Route` pinned to the configured provider at `src/codex_endpoint.rs:337-379` | Replace only route construction with the shared native resolver; preserve metrics, auth, body limits, and one `AppState` snapshot. |
| WS dispatch | `src/codex_endpoint/websocket.rs:43-77` authenticates before upgrade; `:133-184` normalizes `response.create` into a streaming request; `:196-212` calls the same `forward_turn` | Pass model through the same resolver. Preserve Phase 1’s transport-only `type` removal/`stream=true` normalization and generation cancellation. |
| Raw relay | `src/adapters/responses/inbound.rs:36-70` enters inbound passthrough; `:402-454` sends original `Bytes`; `:460-490` streams response and strips framing/hop-by-hop/cookies only | Keep lazy body streaming and safe header filtering. Do not parse/re-serialize native payloads. |
| URL/flavor differences | `src/adapters/responses/request.rs:216-228` uses `/codex/responses` for ChatGPT and `/responses` otherwise; `:128-214` shows ChatGPT/OpenAI/xAI/Grok credential/header differences | Native route dispatch must preserve provider-specific URL/auth/header behavior; do not blindly reuse ChatGPT-only inbound pool code. |
| Hot reload | `forward` starts with `state.refreshed()` (`src/codex_endpoint.rs:162`) and WS GET does likewise (`src/codex_endpoint/websocket.rs:51`) | Resolver must consume that per-turn snapshot; no route mutation mid-turn. |

## Standard Stack

- Rust with existing `serde`/`serde_json`, Axum handlers, `reqwest` upstream client, Tokio cancellation/tasks, and `wiremock` integration tests. **[VERIFIED: src/codex_endpoint.rs:13-19; src/adapters/responses/inbound.rs:5-18; tests/inbound_codex_endpoint.rs:22-32]**
- Existing `Route`/`AdapterKind` types in `src/routing.rs`; no new crate or public dependency is needed. **[VERIFIED: src/routing.rs:9-45]**
- Existing bounded body helper and zstd decoder; do not add a second decompressor or unbounded JSON materialization. **[VERIFIED: src/codex_endpoint.rs:272-289,385-487]**

## Architecture Patterns

### 1. Typed native decision before adapter dispatch

Use a small internal decision/result, for example `Pinned(Route)`, `Selected(Route)`, and `Rejected(ShuntError)`. The resolver should receive the decoded `model` string (and config snapshot), apply `[1m]` normalization only for lookup, and retain the original request model for fallback/telemetry. It should:

1. Find exact `ModelConfig.id` declarations and exact `RouteConfig.model` declarations only.
2. Treat a model-map with more than one participating provider as ambiguous for native ingress, even though generic routing supports ordered fallback.
3. Reject a route whose provider resolves to any adapter other than `Responses`.
4. Reject a mapping whose `Route.upstream_model` differs from the client model after the existing lookup normalization; native passthrough cannot rewrite the body.
5. Return pinned provider for missing/malformed/unmatched input and for no exact declaration, as required by D-01/D-10.

Duplicate exact declarations need an explicit policy. Existing config validation rejects duplicate `[[models]]` ids with upstream maps (`src/config.rs:3719-3741`) and rejects a `[[models]]`/`[[routes]]` conflict (`:3782-3785`), but it does not establish a native resolver contract for duplicate legacy `[[routes]]`. The implementation should detect ambiguity rather than silently taking the first entry; add a focused config/resolver test and an actionable gateway-owned Responses error.

### 2. Shared resolver, separate transport adapters

HTTP `forward_turn` and WS `run_turn` should both call the resolver through the same function. The WS layer may continue to remove the client frame’s `type` and force `stream=true` as Phase 1 transport normalization; it must not implement an independent provider-selection branch. Capture one `AppState` snapshot before resolution, and never re-resolve while an upstream stream is active.

### 3. Provider-aware raw dispatch

The current `forward_codex_inbound` path resolves `StoreFamily::Chatgpt` and scans Codex accounts (`src/adapters/responses/inbound.rs:52-69`), so selected OpenAI/xAI routes cannot simply be passed to it. Plan a provider-aware dispatcher that either (a) reuses the existing credential/account resolution by provider auth mode, or (b) clearly restricts exact native routes to credentials supported by the inbound surface and rejects unsupported auth before dispatch. In both cases, preserve `passthrough_request_headers`, per-provider credential injection, `responses_url`, and `relay_passthrough` invariants. Do not route a native request through the translating `responses::forward` path.

## Don't Hand-Roll

- Do not create a second route syntax, endpoint toggle, model manifest, prefix matcher, generalized policy evaluator, or dynamic capability system; D-02/D-04 explicitly constrain this phase.
- Do not hand-roll header allowlists in the new route path. Reuse the existing credential stripping and hop-by-hop response filtering in `src/adapters/responses/inbound.rs:321-399,460-490` (the exact constants/functions should be located before implementation).
- Do not buffer SSE bodies or deserialize/re-serialize native JSON. `relay_passthrough` uses `Body::from_stream` and must remain lazy. **[VERIFIED: src/adapters/responses/inbound.rs:460-490]**
- Do not import OpenCodex’s policy/capability/history platform. The audit explicitly rejects generalized routing and durable history for this phase. **[VERIFIED: .planning/notes/opencodex-port-audit.md, Phase 2 section]**

## Common Pitfalls and Edge Cases

1. **Model mapping vs. model rewriting:** A configured alias such as `id = "claude-alias"`, `upstream_model = "gpt-5.6"` is valid for translated outbound routing but must fail closed for native ingress; sending the original alias to a provider that expects `gpt-5.6` is not native passthrough.
2. **Multi-provider model maps:** Generic `resolve_model_chain` intentionally returns an ordered chain (`src/routing.rs:558-586` tests). Native ingress must reject this as ambiguous, not try the first route or perform fallback after output.
3. **Prefix routes:** A `[[route_prefixes]]` match must not override the pinned endpoint provider, even if generic routing would choose it (`src/routing.rs:588-613` tests).
4. **`[1m]` suffix:** Existing lookup strips one trailing `[1m]`/`[1M]` (`src/routing.rs:98-110`). Validate the normalized lookup key, but never rewrite the body’s `model` field. Config validation already rejects model-map ids ending in this hint (`src/config.rs:3731-3735`).
5. **Malformed/missing/non-string model:** Current model labeling distinguishes these and continues (`src/codex_endpoint.rs:502-563`). Native routing must preserve pinned behavior rather than turn observability parsing into a new hard failure.
6. **Compressed request inspection:** zstd may be present; inspect decoded bytes but send the original compressed `Bytes` and `content-encoding` unchanged. Existing integration coverage is `tests/inbound_codex_endpoint.rs:475-523`.
7. **WS request normalization:** The WS frame is re-serialized by Phase 1 to force streaming (`src/codex_endpoint/websocket.rs:152-164`); do not claim HTTP byte equality for this transport. Assert provider parity and event payload fidelity instead.
8. **Header leakage:** Native payload opacity does not allow inbound `Authorization`, `x-api-key`, shunt tokens, cookies, or hop-by-hop headers to cross origins. Preserve account credential injection and `set-cookie` stripping.
9. **Mid-stream failover:** Account-local pre-output rotation may remain, but once `forward_codex_inbound` has returned a response and the first body/event is observable, no route hop/replay is safe. Add a test where the first SSE chunk is delivered and the upstream then fails; assert only that stream reaches the client.
10. **Hot reload race:** Resolve from the request’s refreshed snapshot. A reload after dispatch must not change provider for that turn; a later request should see the new exact declaration or pinned fallback.
11. **Provider URL/auth flavor:** ChatGPT uses `/codex/responses`, while non-ChatGPT Responses providers use `/responses` (`src/adapters/responses/request.rs:216-228`); xAI/Grok headers differ from ChatGPT (`:138-211`). Tests must assert path and credential/header behavior without real credentials.
12. **Gateway-owned vs upstream errors:** Pre-dispatch rejection should use the inbound OpenAI Responses error shape (`src/codex_endpoint.rs:200-216`); upstream status/body errors remain byte-preserved.

## Validation Architecture

### Unit tests

- Add resolver table tests near `src/routing.rs` or a dedicated native-routing module for: exact `[[routes]]`; exact single-entry `[models.upstream_model]` with same model; no match → pinned; missing/malformed model → pinned; prefix-only match ignored; multi-provider map rejected; duplicate exact declarations rejected; non-Responses adapter rejected; model-translation mapping rejected; `[1m]` lookup without body rewrite.
- Test the decision’s route fields (`provider`, `adapter`, `model`, `upstream_model`) from source definitions. Quote discrete enum/field values in test evidence; do not infer provider names from built-in defaults.

### HTTP integration tests

Extend `tests/inbound_codex_endpoint.rs` with separate mock upstreams/providers and exact body fixtures:

- exact native route reaches the selected Responses provider, uses its expected `/responses` or `/codex/responses` path and credential/header flavor, and receives request bytes exactly;
- unmatched model reaches the pinned ChatGPT provider exactly as Phase 1;
- translated-only (`anthropic`, `cursor`, `gemini`) and ambiguous routes fail before the mock records a request, with an OpenAI error envelope;
- zstd-compressed exact-route body arrives byte-for-byte with `content-encoding: zstd`;
- successful SSE and upstream error body/status remain byte-equal (apart from documented framing/safe-header filtering);
- after first response bytes, an upstream failure does not trigger another provider request;
- hot reload changes apply only to subsequent turns; disabled endpoint remains unchanged.

### WebSocket integration tests

Extend `tests/inbound_codex_websocket.rs` (Phase 1’s real listener suite) to assert the same resolver decision as HTTP: exact route provider parity, pinned fallback, incompatible/ambiguous pre-dispatch rejection, and no mid-stream hop. Preserve tests for generation cancellation, bounded queue, terminal event handling, and byte-faithful SSE payload text. Use synchronization (`Notify` or equivalent existing fixtures) so the “first event observed before upstream failure” assertion is deterministic.

### Quality gates

Run the repository-required checks after implementation: `cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-features --workspace`. Also run focused routing and inbound endpoint/websocket tests while iterating. No new external package is expected, so package-legitimacy work is not applicable.

## Documentation Surfaces

- `docs/m11-inbound-codex-endpoint.md`: replace the fixed-provider/model-label-only contract with exact native model selection, pinned fallback, rejection rules, and no mid-stream hop.
- `docs/codex-configuration.md`: document that existing exact `[[routes]]` / `[models.upstream_model]` declarations can select only a single Responses-native route for inbound Responses; model-rewriting aliases, prefixes, and ambiguous chains are not eligible.
- `docs/upstreams-failover.md`: clarify that its generic ordered-chain behavior does not become inbound native routing; no cross-provider replay after output.
- `README.md`, `README.ko.md`, `README.ja.md`, `README.zh-CN.md`: synchronize the observable inbound endpoint/model-selection description (D-12).
- Nimbus site source: update `site/src/content/docs/reference/configuration.md`, the inbound Codex endpoint guide, and maintained `ko`/`ja`/`zh-cn` counterparts where those pages exist. Verify locale heading anchors rather than copying English fragments.
- Do not edit generated `wiki/`.

## Implementation Checklist for Planner

1. Define resolver decision/error types and exact-declaration precedence without changing public config schema.
2. Add provider-aware native passthrough credential/account dispatch, or explicitly reject unsupported auth modes before network I/O; preserve ChatGPT fallback path.
3. Replace pinned `Route` construction in HTTP and WS with the shared resolver while retaining Phase 1 normalization/lifecycle.
4. Add unit, HTTP, WS, compressed-body, byte-fidelity, ambiguity, incompatibility, hot-reload, and no-mid-stream-hop tests.
5. Update all documentation/locale surfaces and run format, clippy, focused tests, and full workspace tests.

## Confidence Notes

- **HIGH:** Existing Rust symbols, route precedence, adapter enum, endpoint paths, zstd inspection, passthrough mechanics, WS dispatch, and test locations were read directly this session.
- **HIGH:** Phase constraints and required tests/docs are locked in `02-CONTEXT.md`, `REQUIREMENTS.md`, `ROADMAP.md`, and the OpenCodex port audit.
- **MEDIUM:** The exact credential/account-pool design for selected non-ChatGPT Responses providers remains an implementation choice; existing inbound code is ChatGPT-store-specific and must be reconciled with provider auth semantics before coding.
