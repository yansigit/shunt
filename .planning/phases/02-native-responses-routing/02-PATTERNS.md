# Phase 2: Native Responses Routing - Pattern Map

**Mapped:** 2026-09-05  
**Files analyzed:** 13 likely implementation, test, and documentation surfaces  
**Analogs found:** 13 / 13

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/routing.rs` (or a small native-routing module) | policy/resolver | request-response transform of config metadata | `resolve_model_chain`, `route_for` (`src/routing.rs:122-207`) | exact policy seam; constrain, do not broaden generic resolver |
| `src/codex_endpoint.rs` | HTTP controller/dispatch boundary | bounded request-response + streaming body | `forward`, `forward_turn` (`src/codex_endpoint.rs:245-379`) | exact |
| `src/codex_endpoint/websocket.rs` | transport controller | event-driven streaming | `run_turn` and generation supervisor (`src/codex_endpoint/websocket.rs:75-245`) | exact |
| `src/adapters/responses/inbound.rs` | provider/auth adapter | streaming request-response passthrough | `forward_codex_inbound`, `passthrough_send` (`src/adapters/responses/inbound.rs:36-470`) | exact for raw fidelity; provider credential selection must be extended carefully |
| `src/error.rs` | error/response utility | request-response | `into_openai_error_shape` (`src/error.rs:126-171`) | exact |
| `src/config.rs` (validation only if needed) | typed config/schema | config load/validation | `RouteConfig`, `ModelConfig`, codex endpoint validation (`src/config.rs:1971-1994,3671-3684`) | exact; no new public keys |
| `src/routing.rs` tests | unit tests | table-driven request-response | existing routing tests (`src/routing.rs:250-660`) | exact |
| `tests/inbound_codex_endpoint.rs` | HTTP integration fixture | streaming and byte-preserving I/O | zstd/body/header fixtures (`tests/inbound_codex_endpoint.rs:35-120,476-523,820-900`) | exact |
| `tests/inbound_codex_websocket.rs` | WS integration fixture | event-driven streaming/cancellation | real listener and synchronized upstream tests (`tests/inbound_codex_websocket.rs:150-450`) | exact |
| `docs/m11-inbound-codex-endpoint.md` | milestone contract | documentation | current fixed-provider contract, especially “Fixed provider routing” | exact |
| `docs/codex-configuration.md`, `docs/upstreams-failover.md` | configuration/semantics docs | documentation | existing exact routes and generic failover sections | exact |
| `README*.md`, `site/src/content/docs/{reference,guides}/...` | localized user docs | documentation | English source plus ko/ja/zh-cn copies | exact locale-parity pattern |

## Pattern Assignments

### Resolver: `src/routing.rs` (or dedicated native policy module)

**Analog:** `src/routing.rs:122-207` (`resolve_model_chain`, `route_for`). Reuse `Route`, `AdapterKind`, `strip_context_window_hint`, and provider-kind conversion. `AdapterKind::Responses` is the compatibility gate (`src/routing.rs:10-35`); do not use provider-name allowlists.

**Exact lookup pattern:** `resolve_model_chain` checks `[models.upstream_model]` before exact `[[routes]]`, then prefixes, then default (`src/routing.rs:128-177`). Native policy must copy only the first two exact branches, ignore `route_prefixes`, reject a map with more than one eligible provider, and return a typed `pinned | selected | rejected` decision. Preserve the original requested model in `Route.model`; only use `[1m]` stripping for lookup (`strip_context_window_hint`), never mutate the body.

**Anti-patterns to avoid:** Calling `resolve_model_chain` wholesale (it admits prefixes, ordered fallback chains, and default-provider routes); selecting by provider string; translating `upstream_model`; or silently choosing the first route from an ambiguous chain. Generic `resolve_request_chain` also turns malformed JSON into a 400 (`invalid_routing_request`), whereas phase decisions require missing/malformed/non-string model to retain pinned fallback.

### HTTP boundary: `src/codex_endpoint.rs`

**Analog:** `forward` reads one refreshed `AppState`, authenticates before dispatch, enforces `max_request_bytes`, reads the body once, and uses `model_label` only for observability (`src/codex_endpoint.rs:245-337`). `model_label` demonstrates the bounded zstd inspection pattern (`src/codex_endpoint.rs:385-487`): decode and parse an in-memory copy with `decode_zstd_and_parse`, while passing original `Bytes` and `content-encoding` onward.

**Dispatch pattern:** `forward_turn` constructs a `Route`, invokes `responses::forward_codex_inbound`, records status/metrics, and wraps the response with `stream_metrics::observe_response` (`src/codex_endpoint.rs:337-379`). Replace only route construction with the shared native resolver; keep one snapshot per turn so hot reload cannot change an in-flight route.

**Error pattern:** gateway-owned errors become OpenAI Responses shape at `forward` (`src/codex_endpoint.rs:200-216`) using `crate::error::into_openai_error_shape`; upstream error status/body remains an `Ok` passthrough response.

### WebSocket boundary: `src/codex_endpoint/websocket.rs`

**Analog:** `get` snapshots state and authenticates before upgrade (`:38-76`), while `handle_socket` parses `response.create`, removes transport-only `type`, forces `stream: true`, and reserializes only the WS envelope (`:145-182`). `run_turn` calls the same `forward_turn` as HTTP and uses generation checks to suppress stale frames (`:191-245`).

**Required reuse:** invoke the same native resolver on the parsed model used by HTTP; preserve warmup (`generate: false`), replacement abort, disconnect abort, bounded 4 MiB messages, one-slot output channel, and awaited sends. Do not claim HTTP byte equality for WS: Phase 1 intentionally reserializes the control envelope, but provider selection and payload event text must match HTTP policy.

### Raw provider/auth dispatch: `src/adapters/responses/inbound.rs`

**Analog:** `forward_codex_inbound` strips client credentials and hop-by-hop headers with `passthrough_request_headers`, resolves accounts, and delegates to `forward_codex_passthrough` (`:36-95`). Pool rotation and refresh are pre-output only (`:114-245`); once `FirstOutcome::Relay` returns, `relay_passthrough` streams the response lazily and holds admission through termination.

**Fidelity pattern:** `passthrough_send` posts `body.clone()` without translation and injects only selected credential headers (`:382-454`). `relay_passthrough` preserves status/body and safe metadata while dropping framing, hop-by-hop, and cookies (`PASSTHROUGH_STRIP_RESPONSE_HEADERS`, `:301-380`). Preserve this for selected native providers; reject unsupported auth/flavor before any request rather than forwarding a client credential.

**No-mid-stream-hop rule:** account-local 401/429/5xx rotation may remain inside the adapter before output. Never re-resolve or replay another provider after a response body byte (HTTP) or event (WS) is observable.

### Shared errors and safe headers

Use `into_openai_error_shape` (`src/error.rs:126-171`) for resolver/auth/config failures generated by the gateway. It preserves status, maps the Anthropic-shaped internal envelope to `{error:{message,type,code:null}}`, and caps body reads at 64 KiB. Do not apply it to upstream responses returned by `relay_passthrough`; those remain byte-preserving. Reuse the existing WS `build_ws_error_frame`/safe-header allowlist rather than inventing a second error envelope.

### Tests and fixtures

**Unit table style:** Add native decision cases beside the existing `routing.rs` tests. Existing tests construct `Config` directly and assert route fields (`provider`, `upstream_model`, `effort`, `service_tier`) rather than relying on built-in names (`:250-660`). Cover exact route, single-entry model map, `[1m]`, ignored prefix, pinned fallback, ambiguous map/duplicate exact declarations, non-Responses adapter, and model-translation rejection.

**HTTP fixture style:** Extend `tests/inbound_codex_endpoint.rs`. `INBOUND_BODY` is a literal Responses payload used with `wiremock::matchers::body_bytes/body_string` (`:35-45`); `TestGateway` owns an abortable server task (`:90-110`). Copy the zstd test (`:476-523`) to assert compressed request bytes and `content-encoding` are unchanged. Add separate mock servers/providers, assert selected URL/auth flavor, and assert no request reaches a mock for rejected routes. Use existing `HeaderAbsent`/`BearerToken` matchers for credential leakage checks.

**WS fixture style:** Extend `tests/inbound_codex_websocket.rs`’s real TCP listener tests, using synchronization (`tokio::sync::Notify` or existing gate helpers) to prove first event delivery before an upstream failure and no second-provider request. Retain tests for warmup, generation replacement, disconnect cancellation, terminal-event cutoff, malformed streams, bounds, and safe headers.

### Documentation and locale parity

Update the English source and all maintained translations together. The fixed-provider statement in `docs/m11-inbound-codex-endpoint.md` must become exact native selection plus pinned fallback/rejection/no-mid-stream-hop semantics. Update `docs/codex-configuration.md` and `docs/upstreams-failover.md` without implying generic prefix/chain behavior applies inbound. Synchronize `README.md`, `README.ko.md`, `README.ja.md`, `README.zh-CN.md`, and the Nimbus pages `site/src/content/docs/reference/configuration.md`, `guides/inbound-codex-endpoint.md`, plus their `ko/ja/zh-cn` copies where present. Do not edit `wiki/`; it is generated. Locale headings produce different anchors, so verify each target locale before adding fragments.

## Shared Patterns

1. **Snapshot once per turn:** `AppState::refreshed()` at HTTP/WS entry; route and provider decision are immutable for the turn.
2. **Bound before inspect:** enforce `max_request_bytes`; zstd inspection is ratio-bounded and never replaces the original body.
3. **Opaque native payloads:** inspect only `model`; no request-field rewriting or response JSON parsing on HTTP passthrough.
4. **Credentials belong to gateway:** strip inbound auth, shunt/admin tokens, cookies, and hop-by-hop headers; inject the selected provider/account credential only.
5. **Streaming lazily:** use `Body::from_stream`/`relay_passthrough`, preserve admission guards until stream completion or disconnect, and await WS sends for backpressure.
6. **Gateway vs upstream errors:** reshape only gateway-owned failures; preserve upstream status/body/allowed headers.
7. **Docs are a synchronized surface:** every observable endpoint/provider semantic change updates English plus ko/ja/zh-cn README/site sources; generated `wiki/` is out of scope.

## Anti-Patterns / Explicit Non-Reuse

- Do not route inbound Responses through Anthropic translation adapters or `translate_request`.
- Do not reuse generic prefix routing, heterogeneous ordered chains, or default-provider fallback as an exact native selection.
- Do not mutate `model` to `upstream_model`; a translation-required mapping is rejected.
- Do not retry another provider after the first output byte/event.
- Do not forward client `Authorization`, `x-api-key`, shunt/admin tokens, cookies, or hop-by-hop headers.
- Do not add a public config key, endpoint toggle, persistence store, compaction path, or capability-negotiation abstraction in Phase 2.
