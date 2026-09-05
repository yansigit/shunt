# Coding Conventions

**Analysis Date:** 2026-09-05

## Naming Patterns

**Files:**
- Rust module and source files use `snake_case.rs`: `src/codex_endpoint.rs`, `src/stream_metrics.rs`, `src/status_poll.rs`, `src/upstream_timeout.rs`
- Module directories use `snake_case/` matching module declarations: `src/adapters/`, `src/auth/`, `src/gateway/`, `src/model/`, `src/admin/`
- Submodule test files are named `tests.rs` either alongside module files or in subdirectories: `src/codex_endpoint/tests.rs`, `src/auth/slots/tests.rs`, `src/config/upstreams/tests.rs`
- Integration test files in `tests/` use `snake_case.rs`: `tests/passthrough.rs`, `tests/responses_translate.rs`, `tests/inbound_codex_endpoint.rs`
- Project documentation files use kebab-case or milestone-case: `docs/m1-responses-translation.md`, `docs/codex-configuration.md`

**Functions:**
- Standard Rust `snake_case`: `translate_request`, `into_openai_error_shape`, `build_router`, `stable_session_index`, `warn_if_routes_to_antigravity_cli`
- Conversion / constructor functions follow Rust conventions: `new`, `from_message`, `from_reqwest`, `into_response`, `as_bytes`
- Async handlers follow axum routing conventions: `post`, `get`, `serve`

**Variables:**
- Local variables and bindings use `snake_case`: `session_id`, `started_at`, `http_client`, `upstream_base_url`
- Constants and statics use `SCREAMING_SNAKE_CASE`: `MAX_TELEMETRY_BODY_BYTES`, `REFRESH_IDLE_TTL_SECS`, `AUDIENCE`, `USER_CODE_CHARSET`
- Unused variables or intentionally ignored parameters are prefixed with an underscore: `_shared`, `_state`, `_entered`

**Types:**
- Structs, enums, and traits use `UpperCamelCase`: `AppState`, `Config`, `ShuntError`, `UpstreamError`, `AdapterKind`, `ProviderKind`, `ApprovalProvider`, `RetryableError`
- Enum variants use `UpperCamelCase`: `AdapterKind::Anthropic`, `AdapterKind::Responses`, `AdapterKind::Cursor`, `AdapterKind::Gemini`
- Type aliases use `UpperCamelCase`: `OtelReloadHandle`, `CallbackResult`

## Code Style

**Formatting:**
- Standard `rustfmt` (Rust 2021 edition) enforced in CI via `cargo fmt --all --check`
- Standard line width and indent (4 spaces, no tabs)
- Clean vertical whitespace; group related statements and provide explanatory comments for complex blocks

**Linting:**
- Standard `clippy` enforced in CI with `cargo clippy --all-targets --all-features -- -D warnings` and `RUSTFLAGS=-D warnings`
- All compiler and clippy warnings treated as errors
- Targeted lint allows use scoped attributes with explicit justifications where necessary (e.g. `#[allow(clippy::too_many_arguments)]`)

## Import Organization

**Order:**
1. Standard library imports (`use std::...`)
2. External crate dependencies (`use axum::...`, `use serde::...`, `use tokio::...`, `use tracing::...`)
3. Internal crate imports (`use crate::...`, `use super::...`)
4. Module declarations and re-exports (`mod ...;`, `pub use ...;`)

**Path Aliases:**
- Standard Rust 2021 crate-relative paths: `crate::...`
- Parent module relative paths: `super::...`
- No external path alias remapping needed

## Error Handling

**Patterns:**
- Typed errors for configuration and domain boundaries: `thiserror::Error` on domain enums such as `ConfigError` in `src/config.rs`
- Application-level / CLI entry points use `anyhow::Result` in `src/main.rs`, `src/init.rs`, `src/dashboard.rs`
- Gateway-owned error responses default to the Anthropic JSON error envelope (`{"type": "error", "error": {"type": kind, "message": message}}`) via `ShuntError` and `UpstreamError` in `src/error.rs`
- Inbound Codex endpoint re-shapes gateway errors to OpenAI Responses JSON error envelope (`{"error": {"message": ..., "type": ..., "code": null}}`) via `into_openai_error_shape` in `src/error.rs` (issue #127)
- Relayed upstream errors are preserved verbatim rather than wrapped or modified

## Logging

**Framework:** `tracing` facade with `tracing-subscriber` registry, integrating `tracing_subscriber::fmt` and optional OpenTelemetry / Sentry layers.

**Patterns:**
- Structured logging macros: `tracing::info!`, `tracing::warn!`, `tracing::debug!`, `tracing::error!`, `tracing::trace!`
- Field formatting uses field sigils: `tracing::warn!(%error, "gateway: identity-provider token exchange failed")`
- Logs write to `stderr` via `tracing_subscriber::fmt::layer().with_writer(std::io::stderr)` so stdout remains clean for CLI machine outputs (e.g., token helper)
- Security sensitivity: sensitive headers and credentials are redacted or omitted from trace spans; `session_id` span field is conditionally withheld when OTLP trace export is active (`src/telemetry.rs`)

## Comments

**When to Comment:**
- Explain *why* something is done rather than *what*
- Explicitly cite GitHub issues for bug fixes, edge-case handling, and protocol deviations (e.g., `issue #48`, `issue #127`, `issue #170`, `issue #285`, `issue #291`, `issue #292`)
- Explicitly cite external RFCs and specs for wire protocols (e.g., RFC 7231 for HTTP date, RFC 7692 for permessage-deflate, RFC 8628 for OAuth device authorization)
- Document unsafe/careful lifecycle invariants (e.g., WebSocket handshake TLS crypto provider initialization)

**JSDoc/TSDoc:**
- Rustdoc standard: triple-slash `///` for public items, types, fields, and functions
- Module-level documentation: inner doc comments `//! ...` at the top of subsystems (e.g., `src/gateway/telemetry_ingest.rs`, `src/gateway/refresh.rs`, `tests/inbound_codex_endpoint.rs`)

## Function Design

**Size:**
- Preferred guideline: keep functions focused and files under 500 lines (`CONTRIBUTING.md`); complex logic is factored out into helper functions or split across submodule files
- Large protocol translation pipelines use state machines (e.g., `AnthropicSseMachine`) or dedicated builders

**Parameters:**
- Generic parameter conversions where ergonomic: `impl Into<String>` for messages and names
- Borrowed slices and references: `&str`, `&[u8]`, `&Path`, `Option<&Path>`
- Axum handlers use typed extractors: `State(state): State<AppState>`, `headers: HeaderMap`, `body: Body`

**Return Values:**
- Standard Rust `Result<T, E>` with explicit domain error types or `Option<T>`
- Axum handlers return types implementing `IntoResponse` (e.g., `axum::response::Response`, `Json<T>`, `StatusCode`)

## Module Design

**Exports:**
- Controlled visibility with `pub(crate)` for internal subsystem helpers (e.g., `src/compression.rs`, `src/concurrency.rs`, `src/http_tuning.rs`)
- Public API re-exports in `src/lib.rs` (`pub mod accounts;`, `pub mod config;`, `pub mod server;`, etc.)
- Granular submodules re-exported cleanly with `pub use ...` in parent modules (e.g., `src/config.rs` re-exporting from `admin_keys`, `http_tuning`, `presets`, `secrets`, `session`, `spend`, `upstreams`)

**Barrel Files:**
- Standard Rust module convention with `mod.rs` in subdirectories or sibling file matching folder name (e.g., `src/adapters/mod.rs`, `src/auth/mod.rs`, `src/gateway/mod.rs`)
- Internal submodule test files: `src/<module>/tests.rs` included via `#[cfg(test)] mod tests;`

---

*Convention analysis: 2026-09-05*

