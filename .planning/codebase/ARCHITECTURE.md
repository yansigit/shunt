<!-- refreshed: 2026-09-05 -->
# Architecture

**Analysis Date:** 2026-09-05

## System Overview

```text
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                 Client Layer                                           │
│  Claude Code CLI / Desktop App   │   Inbound OpenAI/Codex Clients │  Admin Browser UI  │
└─────────────────────────────┬────┴─────────────────┬──────────────┴─────────┬──────────┘
                              │                      │                        │
                              ▼                      ▼                        ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                              HTTP & Gateway Ingress                                    │
│  `src/server.rs` (Axum Router & AppState Snapshot)                                     │
│  ├─ Liveness / Health: `/`, `/health`                                                  │
│  ├─ Discovery & Info: `/v1/models`, `/routes`, `/protocol`                             │
│  ├─ Messages & Tokens: `POST /v1/messages`, `POST /v1/messages/count_tokens`           │
│  ├─ Codex Endpoint: `POST /responses`, `POST /backend-api/conversation`                │
│  ├─ Gateway Auth & Mgmt: `/device/code`, `/oauth/token`, `/managed/settings`, `/usage`  │
│  ├─ Admin Surface: `/admin`, `/admin/session`, `/admin/plan`                           │
│  └─ Middleware: Concurrency Gate (`concurrency.rs`), HTTP Tuning (`http_tuning.rs`)   │
└─────────────────────────────┬──────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                           Request Pipeline & Routing                                   │
│  `src/proxy.rs` & `src/proxy/failover.rs`                                              │
│  ├─ Inbound Authentication & Access Control (`src/auth/inbound.rs`)                    │
│  ├─ Request Body Buffering & Normalization (`src/request.rs`)                          │
│  ├─ Model Chain Resolution (`src/routing.rs`: exact → prefix → default)                │
│  └─ Failover Loop & Multi-Account Selection (`src/accounts.rs`)                        │
└──────────────┬──────────────────────────┬─────────────────────────────┬────────────────┘
               │                          │                             │
               ▼                          ▼                             ▼
┌──────────────────────────┐ ┌──────────────────────────┐ ┌──────────────────────────────┐
│    Anthropic Adapter     │ │    Responses Adapter     │ │   Gemini & Cursor Adapters   │
│  `src/adapters/anthropic` │ │  `src/adapters/responses` │ │  `src/adapters/gemini`      │
│  Passthrough & Re-keying │ │  Translation to Responses│ │  `src/adapters/cursor`      │
│  Auto-mode classification│ │  HTTP & WebSocket pool   │ │  ConnectRPC & Code Assist    │
└──────────────┬───────────┘ └────────────┬─────────────┘ └─────────────┬────────────────┘
               │                          │                             │
               ▼                          ▼                             ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                             Upstream LLM Providers                                     │
│  Anthropic API │ OpenAI / ChatGPT │ xAI / Grok │ Cursor │ Google Gemini / Antigravity    │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

| Component | Responsibility | File |
|-----------|----------------|------|
| CLI Entry & Lifecycle | Parses CLI args (`run`, `check`, `token`, `init`, `add`, `login`), initializes telemetry/tracing, and manages graceful shutdown | `src/main.rs` |
| Server & Ingress Routing | Constructs Axum router, builds boot-fixed middlewares, and manages request-scoped `AppState` snapshots | `src/server.rs` |
| Proxy & Failover Pipeline | Orchestrates body reading, model routing, inbound auth verification, failover retries, and token counting | `src/proxy.rs`, `src/proxy/failover.rs` |
| Route Resolution | Resolves requested model names to provider route chains (exact match, prefix match, or fallback default), strips `[1m]` hints | `src/routing.rs` |
| Account Pool & Storm Control | Coordinates multi-account rotation, cooldowns, sliding-window rate tracking, and concurrent slot admissions (`AdmissionGuard`) | `src/accounts.rs` |
| Anthropic Protocol Adapter | Forwards Anthropic Messages requests to native Anthropic or compatible endpoints with optional re-keying and auto-mode classification | `src/adapters/anthropic/mod.rs` |
| Responses Protocol Adapter | Translates between Anthropic Messages and OpenAI Responses protocols, managing HTTP and WebSocket v2 connection pools | `src/adapters/responses/mod.rs` |
| Gemini Protocol Adapter | Translates Anthropic Messages into Google Gemini Code Assist / Antigravity wire format and streams responses | `src/adapters/gemini/mod.rs`, `src/model/gemini.rs` |
| Cursor Protocol Adapter | Translates Anthropic Messages into Cursor ConnectRPC AgentService protobuf frames over HTTP/2 | `src/adapters/cursor/mod.rs` |
| Protocol Translation Models | Defines bi-directional mappings between Anthropic Messages API structures and backend-specific representations | `src/model/mod.rs`, `src/model/responses.rs`, `src/model/gemini.rs` |
| Provider Authentication | Manages token extraction, OAuth PKCE login flows, disk credential persistence, and token refreshes for all providers | `src/auth/mod.rs` |
| Gateway Superset | Implements the RFC 8628 OAuth device flow, JWT generation/validation, per-user managed settings push, spend limits, and OTLP telemetry relay | `src/gateway/mod.rs` |
| Spend Tracking & Limits | Tracks hourly, daily, and monthly spend per user/group with persistence and audit caps | `src/gateway/spend/mod.rs` |
| Inbound Codex Endpoint | Serves OpenAI Responses API endpoints (`/responses`, `/backend-api/conversation`) to Codex clients with load balancing | `src/codex_endpoint.rs` |
| Configuration & Validation | Loads, merges, and validates TOML/YAML/env configuration and defines strongly-typed configuration models | `src/config.rs` |
| Hot Reload Engine | Monitors config files via `notify` and performs atomic lock-free runtime state swaps using `arc-swap` | `src/reload.rs` |
| Inbound Concurrency & HTTP Tuning | Sheds excess inbound traffic (HTTP 503) and enforces CIDR access control, body size, header bytes, and URL length limits | `src/concurrency.rs`, `src/http_tuning.rs` |
| Token Counting | Computes exact or offline tiktoken-based token counts for inbound requests | `src/count_tokens.rs` |
| Admin Web Surface | Serves operator dashboard and secure session management for live pool inspection and configuration | `src/admin/mod.rs` |

## Pattern Overview

**Overall:** Hexagonal / Ports-and-Adapters Gateway with Inverted Dependency Flow and Immutable Hot-Reloadable State Snapshots.

**Key Characteristics:**
- **Strict Separation of Concerns:** Inbound handlers parse and buffer protocol requests; routing determines *where* traffic goes; adapters determine *how* to communicate with upstreams. Adapters never access routing or endpoint policy.
- **Unbuffered Streaming Preservation:** Streaming responses from upstreams are forwarded immediately to the client via asynchronous data streams. Chunks are never buffered in memory unless the client explicitly requested non-streaming mode.
- **Atomic State Snapshots (`arc-swap`):** Runtime configuration and authentication state are stored in an `ArcSwap<RuntimeState>`. Every inbound request takes a shallow clone (`AppState::refreshed()`) that remains pinned for the duration of the request, preventing configuration drift mid-flight while enabling zero-downtime hot reloads.
- **Table-Driven Routing & Extensibility:** New providers and models are added via configuration tables rather than imperative code branching.
- **Storm-Control Admission Guards:** Concurrent in-flight slots per account are guarded by `AdmissionGuard`. The guard is wrapped into the lazy response body stream (`with_admission`), ensuring the account slot remains held until the stream completes or the client disconnects.

## Layers

**Interface Layer:**
- Purpose: Exposes HTTP endpoints, accepts TCP connections, enforces connection-level and request-level limits, and provides the CLI.
- Location: `src/main.rs`, `src/server.rs`, `src/routes.rs`, `src/discovery.rs`, `src/protocol.rs`, `src/admin/`, `src/codex_endpoint.rs`, `src/gateway/`.
- Contains: Axum route handlers, CLI clap definitions, middleware layers (`ConcurrencyLimit`, `HttpTuningLayer`).
- Depends on: Application and Infrastructure layers.
- Used by: External HTTP clients (Claude Code, Codex CLI, browsers, load balancers) and command-line operators.

**Application Layer:**
- Purpose: Coordinates request ingress, authentication verification, request normalization, route resolution, failover iteration, and token counting.
- Location: `src/proxy.rs`, `src/proxy/failover.rs`, `src/routing.rs`, `src/count_tokens.rs`, `src/request.rs`.
- Contains: Request processing workflows, route matching algorithms, body normalization logic, failover state machines.
- Depends on: Domain Layer (Adapters, Translation Models) and Infrastructure Layer (Auth, Accounts, Config, Error).
- Used by: Interface Layer handlers in `src/server.rs`.

**Domain Layer:**
- Purpose: Encapsulates provider protocol translation, upstream adapter implementations, and wire-level protocol representations.
- Location: `src/adapters/`, `src/model/`.
- Contains: `Adapter` trait implementations (`AnthropicAdapter`, `ResponsesAdapter`, `CursorAdapter`, `GeminiAdapter`), SSE translators, JSON schema converters.
- Depends on: Infrastructure Layer (`src/config.rs`, `src/error.rs`, `src/auth/`, `src/accounts.rs`).
- Used by: Application Layer (`src/proxy/failover.rs`, `src/count_tokens.rs`).

**Infrastructure Layer:**
- Purpose: Manages persistent resources, credentials, multi-account pools, network clients, configuration loading, telemetry, and OS primitives.
- Location: `src/config.rs`, `src/auth/`, `src/accounts.rs`, `src/reload.rs`, `src/telemetry.rs`, `src/metrics.rs`, `src/error.rs`, `src/atomic_file.rs`, `src/keepalive.rs`.
- Contains: File locks, token refresh logic, OAuth store persistence, OTLP exporters, Sentry integration, atomic file writes.
- Depends on: External libraries (`tokio`, `reqwest`, `figment`, `serde`, `arc-swap`).
- Used by: All higher layers.

## Data Flow

### Primary Request Path

1. **Client Request Entry:** Claude Code issues an HTTP `POST /v1/messages` request to `src/server.rs:224`, handled by `src/proxy.rs:20` (`post()`).
2. **Snapshot State & Metrics:** The handler snapshots current runtime configuration via `state.refreshed()` and initializes tracing spans in `src/proxy.rs:27`.
3. **Body Read & Limit Check:** `src/proxy/failover.rs:25` reads the incoming body up to `max_request_bytes` via `src/http_tuning.rs:17`.
4. **Body Normalization:** `src/proxy.rs:98` cleans empty text blocks and validates the payload structure into `src/request.rs:18` (`RequestBody`).
5. **Model Route Resolution:** `src/routing.rs:59` (`resolve_request_chain_value()`) strips client-side `[1m]` hints and resolves the model to a chain of `Route` items (exact match → prefix match → default provider).
6. **Inbound Authentication:** `src/proxy/failover.rs:69` verifies caller credentials against `src/auth/inbound.rs` (static client tokens or gateway JWT claims) and checks managed model policy.
7. **Failover & Account Selection Loop:** `src/proxy/failover.rs:107` iterates over routes in the chain. For the chosen provider, it selects an account and requests an admission slot from `src/accounts.rs:809` (`try_admit()`).
8. **Adapter Invocation:** The selected adapter's `forward()` method (`src/adapters/mod.rs:54`) is called with the request body, route metadata, and credentials.
   - For Anthropic: `src/adapters/anthropic/mod.rs:271` relays to the upstream URL with injected headers.
   - For Responses: `src/adapters/responses/mod.rs:37` translates Anthropic Messages to OpenAI Responses (`src/model/responses_request.rs:18`) and streams via HTTP or WebSocket.
9. **Streaming Transmission & Admission Guard:** Upstream SSE bytes stream back through `src/adapters/mod.rs:22` (`with_admission()`), which keeps the account's concurrency permit active until the client disconnects or the stream finishes.
10. **Telemetry & Outcome Recording:** Request latency, status code, token usage, and failover outcomes are logged and exported via `src/metrics.rs` and `src/observability.rs`.

### Secondary Flow 1: Inbound OpenAI Responses Protocol (Codex Endpoint)

1. **Ingress:** Codex CLI issues `POST /responses` or `POST /backend-api/conversation` to `src/server.rs:256`, entering `src/codex_endpoint.rs:86` (`post()`).
2. **Authentication:** Handler validates inbound authentication against `[server.auth]` or gateway tokens via `src/auth/inbound.rs`.
3. **Account Load Balancing:** Selects an account from the configured provider's account pool in `src/accounts.rs`.
4. **Direct Forwarding:** Relays the Responses request directly to upstream ChatGPT/Codex backend via `src/adapters/responses/inbound.rs`.
5. **Error Normalization:** Any gateway-generated errors are transformed into OpenAI Responses error shape via `src/error.rs:125` (`into_openai_error_shape()`).

### Secondary Flow 2: Claude Apps Gateway Device Flow & Managed Settings

1. **Device Authorization:** Client requests authorization via `POST /device/code` (`src/gateway/device.rs:25`), receiving a user code and verification URI.
2. **User Approval:** Operator completes approval in the browser at `GET /device` (`src/gateway/approval.rs:23`), authenticating via static approval users or external OIDC.
3. **Token Issuance:** Client polls `POST /oauth/token` (`src/gateway/oauth.rs:82`); once approved, the gateway issues a signed JWT access token and refresh token.
4. **Managed Settings Retrieval:** Client requests policy via `GET /managed/settings` (`src/gateway/managed.rs:22`); the gateway returns environment overrides, model restrictions, and OTLP endpoints.
5. **Telemetry Ingest:** Client forwards OTLP traces and metrics to `/v1/traces` and `/v1/metrics` (`src/gateway/telemetry_ingest.rs:27`), which are verified and relayed to upstream sinks.

### Secondary Flow 3: Hot Reload Configuration Workflow

1. **Watch Event:** Filesystem watcher in `src/reload.rs:55` detects modifications to `shunt.toml` or `shunt.yaml`.
2. **Parsing & Validation:** `src/reload.rs:120` (`reload_config()`) loads the file via `figment`, parses secrets, and validates structural constraints.
3. **State Compilation:** A new `RuntimeState` struct is compiled containing the new `Config`, `InboundAuth`, `AdminAuth`, and `GatewayAuth`.
4. **Lock-Free Swap:** The new state is stored in `SharedState` via `arc_swap::ArcSwap::store` (`src/reload.rs:36`).
5. **Zero-Interruption Adoption:** Existing in-flight requests complete using their captured `AppState` clone; subsequent requests immediately adopt the new runtime state.

**State Management:**
- **Process-Lifetime State:** Multi-account quota and cooldown tracking (`AccountPool`), admin sessions (`AdminStores`), and gateway device grants (`GatewayStores`) live across config reloads.
- **Reloadable State:** Upstream provider configs, model definitions, routing rules, inbound auth tokens, and spend limits live inside `ArcSwap<RuntimeState>`.
- **Persistent Disk State:** Account OAuth tokens and refresh state are persisted safely via atomic file writes with file locking (`src/atomic_file.rs`, `src/auth/shared/file_lock.rs`).

## Key Abstractions

**`Adapter` Trait:**
- Purpose: Unified interface implemented by all upstream protocol translators.
- Examples: `src/adapters/anthropic/mod.rs`, `src/adapters/responses/mod.rs`, `src/adapters/gemini/mod.rs`, `src/adapters/cursor/mod.rs`.
- Pattern: Strategy / Adapter Pattern. Decouples proxy routing from upstream wire formats.

**`AppState` Snapshot:**
- Purpose: Per-request snapshot holding references to both hot-reloadable configuration and long-lived stores.
- Examples: `src/server.rs:27` (`struct AppState`).
- Pattern: Immutable Snapshot / Reader-Writer separation via `ArcSwap`.

**`AccountPool` & `AdmissionGuard`:**
- Purpose: Manages a pool of authenticated provider accounts, balances load, detects rate limits, and throttles concurrency.
- Examples: `src/accounts.rs:177` (`struct AccountPool`), `src/accounts.rs:232` (`struct AdmissionGuard`).
- Pattern: Resource Pool with RAII Storm-Control Guard.

**`ShuntError` & `UpstreamError`:**
- Purpose: Uniform error types converted into spec-compliant Anthropic or OpenAI JSON error envelopes.
- Examples: `src/error.rs:55` (`struct ShuntError`), `src/error.rs:12` (`struct UpstreamError`).
- Pattern: Error Translation / Domain Error Envelope.

## Entry Points

**CLI Binary Entry Point:**
- Location: `src/main.rs:77` (`fn main()`).
- Triggers: Invocation of `shunt` from shell.
- Responsibilities: Parses command-line subcommands (`run`, `check`, `token`, `init`, `add`, `login`), configures tracing/OTel, sets up Tokio runtime, and initiates server.

**HTTP Server Startup:**
- Location: `src/server.rs:163` (`pub fn build_router()`).
- Triggers: Called by `src/main.rs` during `shunt run` or by integration tests.
- Responsibilities: Validates initial configuration, derives boot-fixed middleware (concurrency semaphores, CIDR filters), builds routes, and starts background reload and poller tasks.

**HTTP Request Entry Point:**
- Location: `src/proxy.rs:20` (`pub async fn post()`).
- Triggers: Inbound `POST /v1/messages` or `POST /v1/messages/count_tokens`.
- Responsibilities: Refreshes request state snapshot, records tracing spans, and dispatches to the failover forwarder.

## Architectural Constraints

- **Threading:** Multi-threaded asynchronous event loop driven by Tokio. CPU-intensive operations (token encoding via `CoreBPE`, gzip/zstd decompression, large JSON parsing) must be offloaded to blocking threads or bounded semaphore pools (`src/compression.rs`, `src/offload.rs`).
- **Global State:** Minimized to process-wide primitives wrapped in `OnceLock` or `LazyLock`:
  - Tracing reload handle: `OTEL_RELOAD` (`src/main.rs:27`).
  - WebSocket pool: `POOL` (`src/adapters/responses/codex_ws.rs:262`).
  - Compression semaphores: `SLOTS` (`src/compression.rs:116`).
  - File lock waiter tracking: `WAITERS` (`src/auth/shared/file_lock.rs:220`).
  - Event throttles: `THROTTLE` (`src/observability/throttle.rs:211`).
  - Metric instruments: `INSTRUMENTS` (`src/metrics.rs:78`).
- **Circular Imports:** Avoided via strict downward layering (Interface → Application → Domain → Infrastructure). Crate-internal helpers use `pub(crate)` scoped to specific module hierarchies.
- **Streaming Preservation:** Upstream SSE frames must never be buffered or re-framed in memory unless non-streaming was requested by the client. Responses must stream incrementally to prevent latency degradation.
- **Boot-Fixed Gates:** Network bind addresses and concurrency limits are fixed at process startup. A config reload cannot alter the active listening socket without a full process restart.

## Anti-Patterns

### Buffering Streaming Responses

**What happens:** Accumulating entire upstream SSE response bodies before returning them to the client.
**Why it's wrong:** Destroys real-time token streaming, increases time-to-first-token latency for Claude Code, and exhausts gateway memory under load.
**Do this instead:** Stream raw response chunks immediately via `axum::body::Body::from_stream` and tie concurrency permits to the stream via `with_admission` (`src/adapters/mod.rs:22`).

### Premature Release of Admission Slots

**What happens:** Dropping the concurrency admission guard when the HTTP adapter function returns rather than when the body finishes streaming.
**Why it's wrong:** The adapter returns as soon as headers arrive; dropping the guard early allows subsequent requests to overload account concurrency limits while the SSE stream is still in flight.
**Do this instead:** Wrap the `AdmissionGuard` into the lazy data stream closure so it drops only when the stream terminates or the client disconnects (`src/adapters/mod.rs:22`).

### Leaking Credentials Across Failovers

**What happens:** Reusing client credentials when falling back from a credential-injecting provider to a passthrough provider.
**Why it's wrong:** Client tokens or gateway secrets presented to shunt could be forwarded upstream to third-party endpoints.
**Do this instead:** Explicitly verify origin compatibility and strip injected or client credentials when switching route boundaries (`src/proxy/failover.rs:591`).

### Misaligned Error Envelopes

**What happens:** Emitting Anthropic error envelopes on the inbound Codex endpoint, or raw OpenAI JSON on Anthropic routes.
**Why it's wrong:** Client protocol parsers fail to deserialize mismatched error formats, displaying confusing or garbled error messages.
**Do this instead:** Ensure all gateway-owned errors on standard routes use `ShuntError` (`src/error.rs:55`) and transform errors on the Codex endpoint using `into_openai_error_shape` (`src/error.rs:125`).

## Error Handling

**Strategy:** Failures are categorized at boundary layers into client errors, upstream recoverable errors (eligible for failover/retry), and unrecoverable gateway faults. Errors returned to clients always conform to the protocol shape expected by that client.

**Patterns:**
- **Anthropic Error Envelope:** Standard errors return `{"type":"error","error":{"type":"<kind>","message":"..."}}` with matching HTTP status codes (`src/error.rs:30`).
- **OpenAI Error Envelope:** Codex endpoint errors return `{"error":{"message":"...","type":"<kind>","code":null}}` (`src/error.rs:100`).
- **Failover Classification:** `AdapterError` captures whether a failure occurred before headers or with a specific HTTP status code (`AdapterFailure`), driving intelligent failover and account rotation in `src/proxy/failover.rs`.
- **Safe Degradation:** Missing non-critical headers, telemetry errors, and analytics ingestion failures degrade silently or log warnings without failing client requests.

## Cross-Cutting Concerns

**Logging:** Structured, context-aware logging using the `tracing` crate. Spans track request method, path, session ID, model, provider, and response status. Exportable to OpenTelemetry (OTLP) and Sentry via opt-in configuration (`src/telemetry.rs`).
**Validation:** Configuration input is strictly validated on startup and reload via `Config::validate()` in `src/config.rs`. Inbound HTTP requests undergo header validation, body-size enforcement (`src/http_tuning.rs`), and model field verification (`src/routing.rs`).
**Authentication:** Layered authentication system supporting:
- Inbound client tokens (`[server.auth]`) and gateway JWTs (`src/auth/inbound.rs`).
- Provider outbound credentials (API keys, Claude OAuth, ChatGPT OAuth, xAI OAuth, Cursor OAuth, Kimi OAuth, Google OAuth, Antigravity OAuth) in `src/auth/`.
- Mutual exclusion and cross-process file locking for credential refreshes (`src/auth/shared/file_lock.rs`).

---

*Architecture analysis: 2026-09-05*
