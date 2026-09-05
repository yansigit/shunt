# Codebase Structure

**Analysis Date:** 2026-09-05

## Directory Layout

```
shunt/
├── Cargo.toml                     # Rust package manifest, dependencies, and metadata
├── Cargo.lock                     # Pinned dependency graph
├── AGENTS.md                      # Development rules, test commands, and engineering constraints
├── ARCHITECTURE.md                # System architecture summary and high-level design
├── CONTRIBUTING.md                # Contribution guidelines and PR workflow
├── README.md                      # Main English project overview and quickstart
├── README.ja.md                   # Japanese translation of README
├── README.ko.md                   # Korean translation of README
├── README.zh-CN.md                # Simplified Chinese translation of README
├── shunt.toml.example             # Documented TOML configuration template
├── shunt.yaml.example             # Documented YAML configuration template
├── benches/                       # Performance benchmarks
│   ├── gateway.rs                 # Gateway throughput and latency benchmarks
│   └── perf_issues.rs             # Profiling benchmarks for high-load regressions
├── blueprints/                    # Embedded agent-assisted blueprints (shunt add)
│   ├── provider/                  # Provider implementation blueprints
│   └── upstream/                  # Upstream configuration blueprints
├── docs/                          # Architecture specs, milestone design docs, and guides
│   ├── m1-responses-translation.md# Milestone 1: Responses translation design
│   ├── m9-admin-surface.md        # Milestone 9: Admin web surface design
│   ├── gateway-login.md           # Claude Apps gateway login protocol
│   ├── gateway-managed-settings.md# Managed settings push specification
│   ├── gateway-spend-limits.md    # Spend limits and budget tracking design
│   └── upstreams-failover.md      # Multi-upstream failover design
├── packaging/                     # Distribution and packaging recipes
│   └── homebrew/                  # Homebrew formula definition
├── plugins/                       # Claude Code bundled plugins for providers
│   ├── shunt-codex/               # Codex provider plugin
│   ├── shunt-deepseek/            # DeepSeek provider plugin
│   ├── shunt-kimi/                # Kimi provider plugin
│   └── shunt-xai/                 # xAI provider plugin
├── site/                          # Nimbus documentation portal (Astro / Cloudflare Pages)
│   ├── astro.config.ts            # Astro framework configuration
│   ├── package.json               # Frontend dependencies
│   └── src/content/docs/          # Multilingual markdown documentation sources
├── src/                           # Core Rust library and binary implementation
│   ├── lib.rs                     # Library root declaring public and crate modules
│   ├── main.rs                    # CLI entry point, argument parsing, and runtime boot
│   ├── server.rs                  # Axum router construction and AppState snapshot management
│   ├── proxy.rs                   # Inbound message proxying and pipeline coordination
│   ├── routing.rs                 # Model matching and route chain resolution
│   ├── accounts.rs                # Multi-account pool, rate limiting, and storm control
│   ├── config.rs                  # Typed configuration, defaults, and validation
│   ├── error.rs                   # Error types and Anthropic/OpenAI envelope serialization
│   ├── reload.rs                  # Hot reload filesystem watcher and atomic state swapping
│   ├── request.rs                 # Inbound request parsing and body normalization
│   ├── retry.rs                   # Upstream retry logic and backoff calculation
│   ├── discovery.rs               # /v1/models model discovery implementation
│   ├── protocol.rs                # /protocol gateway metadata endpoint
│   ├── routes.rs                  # /routes active route inspection endpoint
│   ├── codex_endpoint.rs          # Inbound OpenAI Responses protocol handler
│   ├── codex_analytics.rs         # Inbound Codex analytics ingest handler
│   ├── count_tokens.rs            # Token counting via offline tiktoken encoder
│   ├── concurrency.rs             # Inbound concurrency limiting middleware
│   ├── http_tuning.rs             # Access control, body size, and header byte limits
│   ├── keepalive.rs               # SSE keepalive ping injection
│   ├── telemetry.rs               # Tracing subscriber, OpenTelemetry, and Sentry hooks
│   ├── metrics.rs                 # Prometheus/OTel metric instruments and recorders
│   ├── atomic_file.rs             # Atomic file writes with fsync and rename
│   ├── adapters/                  # Provider protocol adapters
│   │   ├── mod.rs                 # Adapter trait, AdapterError, with_admission wrapper
│   │   ├── anthropic/             # Anthropic Messages passthrough and re-keying
│   │   ├── responses/             # OpenAI Responses translation and WebSocket v2 pool
│   │   ├── gemini/                # Google Gemini / Code Assist adapter
│   │   ├── antigravity/           # Google Antigravity adapter
│   │   └── cursor/                # Cursor ConnectRPC AgentService adapter
│   ├── admin/                     # Operator admin web surface and session management
│   │   ├── mod.rs                 # Admin router and handler definitions
│   │   ├── html.rs                # Embedded HTML/CSS admin UI templates
│   │   ├── session.rs             # Cookie sessions and authentication state
│   │   └── oidc.rs                # OpenID Connect integration for admin login
│   ├── auth/                      # Provider authentication and credential storage
│   │   ├── mod.rs                 # Unified credential lookup and dispatch
│   │   ├── inbound.rs             # Inbound client token and gateway auth checks
│   │   ├── slots.rs               # Multi-credential slot resolution
│   │   ├── shared/                # Cross-process file locking and caching utilities
│   │   ├── anthropic/             # Anthropic subscription OAuth and token storage
│   │   ├── codex/                 # ChatGPT / Codex OAuth credentials and store
│   │   ├── cursor/                # Cursor OAuth login and token refresh
│   │   ├── gemini/                # Google One AI Pro / Gemini OAuth
│   │   ├── kimi/                  # Kimi Code subscription OAuth
│   │   └── xai/                   # xAI / Grok subscription OAuth
│   ├── config/                    # Config submodules and presets
│   │   ├── admin_keys.rs          # Admin keyring configuration
│   │   ├── http_tuning.rs         # HTTP tuning and CIDR access control config
│   │   ├── presets.rs             # Built-in provider templates (xAI, Kimi, DeepSeek, etc.)
│   │   ├── secrets.rs             # Secret string wrapper preventing accidental leakage
│   │   ├── session.rs             # Gateway session configuration
│   │   ├── spend.rs               # Spend limit configuration schema
│   │   └── upstreams.rs           # Upstream provider configuration normalization
│   ├── gateway/                   # Claude Apps Gateway superset implementation
│   │   ├── mod.rs                 # Gateway router and status definitions
│   │   ├── device.rs              # RFC 8628 OAuth device authorization flow
│   │   ├── approval.rs            # Browser device approval UI and logic
│   │   ├── oauth.rs               # OAuth 2.0 token endpoint and JWT minting
│   │   ├── jwt.rs                 # HMAC-SHA256 JWT creation and validation
│   │   ├── managed.rs             # GET /managed/settings per-user policy endpoint
│   │   ├── telemetry_ingest.rs    # OTLP trace and metric ingestion relay
│   │   └── spend/                 # Spend limit tracking, stores, and persistence
│   └── model/                     # Protocol translation data models
│       ├── mod.rs                 # Module definitions and exports
│       ├── responses.rs           # Responses SSE event translation to Anthropic SSE
│       ├── responses_request.rs   # Anthropic Messages request to Responses request
│       ├── gemini.rs              # Gemini streaming response to Anthropic SSE
│       ├── gemini_request.rs      # Anthropic Messages to Gemini request format
│       └── antigravity_request.rs # Antigravity-specific model parameter translation
├── tests/                         # Integration test suite
│   ├── passthrough.rs             # WireMock tests for Anthropic passthrough
│   ├── responses_translate.rs     # Integration tests for Responses translation
│   ├── failover.rs                # Multi-upstream failover and recovery tests
│   ├── inbound_auth.rs            # Client authentication and gateway JWT tests
│   ├── codex_multi_account.rs     # Codex account pool and rotation tests
│   ├── admin_surface.rs           # Admin dashboard integration tests
│   └── gateway_cli.rs             # Gateway device login and token tests
└── wiki/                          # Generated documentation wiki (managed by wiki-please)
```

## Directory Purposes

**`src/`:**
- Purpose: Main Rust source tree containing the library, HTTP server, and CLI application.
- Contains: Rust source modules (`.rs`).
- Key files: `src/lib.rs`, `src/main.rs`, `src/server.rs`, `src/proxy.rs`, `src/config.rs`.

**`src/adapters/`:**
- Purpose: Encapsulates provider protocol translation and network transport implementations.
- Contains: Adapter modules implementing the `Adapter` trait.
- Key files: `src/adapters/mod.rs`, `src/adapters/anthropic/mod.rs`, `src/adapters/responses/mod.rs`, `src/adapters/gemini/mod.rs`, `src/adapters/cursor/mod.rs`.

**`src/auth/`:**
- Purpose: Manages credentials, tokens, OAuth device flows, refreshes, and disk persistence.
- Contains: Provider-specific auth clients, storage helpers, cross-process file locks.
- Key files: `src/auth/mod.rs`, `src/auth/inbound.rs`, `src/auth/shared/file_lock.rs`.

**`src/gateway/`:**
- Purpose: Implements the Claude Apps Gateway protocol superset (RFC 8628 device flow, managed settings, spend limits, OTLP relay).
- Contains: OAuth endpoints, JWT validators, device approval UI, telemetry ingestion handlers.
- Key files: `src/gateway/mod.rs`, `src/gateway/oauth.rs`, `src/gateway/managed.rs`, `src/gateway/spend/mod.rs`.

**`src/model/`:**
- Purpose: Cross-protocol schema translation between Anthropic Messages API and backend wire protocols.
- Contains: Strongly-typed request/response models and streaming parsers.
- Key files: `src/model/responses_request.rs`, `src/model/responses.rs`, `src/model/gemini_request.rs`.

**`src/admin/`:**
- Purpose: Operator dashboard for real-time account pool inspection, quota metrics, and configuration.
- Contains: Web handlers, embedded HTML/JS templates, session management.
- Key files: `src/admin/mod.rs`, `src/admin/html.rs`, `src/admin/session.rs`.

**`tests/`:**
- Purpose: Out-of-tree black-box integration tests verifying HTTP endpoints, wire protocols, and failover behavior.
- Contains: Rust integration test files using WireMock and Axum test servers.
- Key files: `tests/passthrough.rs`, `tests/responses_translate.rs`, `tests/failover.rs`, `tests/inbound_auth.rs`.

**`docs/`:**
- Purpose: Specifications, milestone records (`m1` through `m15`), running guides, and operational notes.
- Contains: Markdown technical documentation.
- Key files: `docs/running.md`, `docs/upstreams-failover.md`, `docs/gateway-protocol.md`.

**`site/`:**
- Purpose: Nimbus documentation site deployed to Cloudflare Pages.
- Contains: Astro Starlight project files, TypeScript configs, Markdown content across locales.
- Key files: `site/astro.config.ts`, `site/src/content/docs/`.

**`wiki/`:**
- Purpose: Auto-generated documentation wiki maintained by `wiki-please`.
- Contains: Generated Starlight site assets and Markdown documentation.
- Key files: `wiki/astro.config.mjs`, `wiki/llms.txt`.

## Key File Locations

**Entry Points:**
- `src/main.rs`: CLI entry point parsing subcommands (`run`, `check`, `token`, `login`, `init`, `add`).
- `src/server.rs`: Router initialization (`build_router`) and Axum endpoint wiring.
- `src/proxy.rs`: HTTP request handler for `POST /v1/messages` and `POST /v1/messages/count_tokens`.
- `src/codex_endpoint.rs`: Inbound OpenAI Responses protocol handler for `POST /responses`.

**Configuration:**
- `Cargo.toml`: Workspace definitions, crate dependencies, and feature flags.
- `src/config.rs`: Typed configuration structs, TOML/YAML loading via Figment, and validation rules.
- `src/config/presets.rs`: Default upstream provider presets and model mappings.
- `src/config/secrets.rs`: Redaction wrappers and environment secret resolution.

**Core Logic:**
- `src/routing.rs`: Model matching algorithm and route chain derivation.
- `src/proxy/failover.rs`: Failover execution loop, account quota rotation, and fallback retries.
- `src/accounts.rs`: Multi-account pool, sliding window utilization tracking, and slot admission guards.
- `src/adapters/responses/mod.rs`: OpenAI Responses translation adapter and WebSocket v2 connection manager.
- `src/adapters/anthropic/mod.rs`: Native Anthropic passthrough and auto-mode classification.
- `src/gateway/mod.rs`: Claude Apps Gateway device flow and managed policy enforcement.
- `src/reload.rs`: Hot reload watcher and atomic state swapping using `arc-swap`.

**Testing:**
- `tests/passthrough.rs`: WireMock tests for Anthropic Messages passthrough routing and error handling.
- `tests/responses_translate.rs`: Translation verification between Anthropic Messages and OpenAI Responses.
- `tests/failover.rs`: Verification of multi-upstream failover chains and error recovery.
- `tests/inbound_auth.rs`: Inbound client authentication, bearer verification, and gateway token validation.
- `tests/admin_surface.rs`: Integration test suite for the operator admin dashboard.

## Naming Conventions

**Files:**
- Rust Source Files: Lowercase snake_case (`src/proxy/failover.rs`, `src/http_tuning.rs`).
- Documentation Files: Lowercase kebab-case for guides (`docs/upstreams-failover.md`), uppercase for root records (`README.md`, `ARCHITECTURE.md`).
- Localized Documents: Suffix with BCP 47 language tag (`README.ja.md`, `README.ko.md`, `README.zh-CN.md`).

**Directories:**
- Rust Module Directories: Lowercase snake_case matching module name (`src/codex_endpoint/`, `src/stream_metrics/`).
- Top-level directories: Lowercase single words or kebab-case (`blueprints/`, `packaging/homebrew/`).

## Where to Add New Code

**New Feature (e.g., New Gateway Route or Policy):**
- Primary code: Register route in `src/server.rs` (`build_router`) and implement handler in `src/gateway/` or dedicated `src/<feature>.rs`.
- Configuration: Add typed fields to `ServerConfig` or sub-config in `src/config.rs` and add validation in `Config::validate()`.
- Tests: Add integration test file in `tests/<feature>.rs`.

**New Provider / Protocol Adapter:**
- Implementation:
  - If protocol is already supported (Anthropic-compatible or Responses-compatible): Add preset to `src/config/presets.rs` without code changes.
  - If new wire protocol: Create module `src/adapters/<provider>/mod.rs` implementing `Adapter` trait (`src/adapters/mod.rs`). Add translation logic to `src/model/<provider>.rs`.
  - Add enum variants to `ProviderKind` and `AdapterKind` in `src/config.rs` and `src/routing.rs`.
- Authentication: Add provider auth handler in `src/auth/<provider>/auth.rs` and register in `src/auth/mod.rs`.
- Tests: Add protocol translation tests in `tests/<provider>_translate.rs`.

**Utilities & Shared Helpers:**
- Atomic File Operations: Add to `src/atomic_file.rs`.
- Cross-Process Locks: Add to `src/auth/shared/file_lock.rs`.
- HTTP Header Utilities: Add to `src/headers.rs`.
- Token Counting Helpers: Add to `src/count_tokens.rs`.

## Special Directories

**`.planning/`:**
- Purpose: Contains GSD project planning artifacts, phase roadmaps, requirements, and codebase maps.
- Generated: Partially generated by GSD tools; manually curated.
- Committed: Yes.

**`wiki/`:**
- Purpose: Static documentation wiki powered by Starlight.
- Generated: Yes (generated via `wiki-please`). Hand-editing is prohibited.
- Committed: Yes.

**`site/`:**
- Purpose: Hosted documentation website deployed to Cloudflare Pages.
- Generated: No (hand-authored content in `site/src/content/docs/`).
- Committed: Yes.

**`blueprints/`:**
- Purpose: Embedded markdown blueprint prompts served by `shunt add` for coding agents.
- Generated: No (hand-authored templates).
- Committed: Yes.

**`target/`:**
- Purpose: Cargo build artifacts, compiled binaries, and intermediate object files.
- Generated: Yes.
- Committed: No (ignored via `.gitignore`).

---

*Structure analysis: 2026-09-05*
