# Technology Stack

**Analysis Date:** 2026-09-05

## Languages

**Primary:**
- Rust (2021 edition) - Gateway core runtime, HTTP/SSE proxy, protocol adapters, CLI binary (`src/main.rs`, `src/lib.rs`, `src/**`, `tests/**`, `benches/**`)

**Secondary:**
- TypeScript / JavaScript (Node.js >= 22.12) - Documentation site (`site/`) and documentation wiki (`wiki/`)
- HTML / CSS - Documentation layouts and Tailwind CSS v4 styling (`site/src/**`)

## Runtime

**Environment:**
- Native compiled binary (target platforms: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`)
- Node.js (version >= 22.12) for building documentation artifacts (`site/package.json`)

**Package Manager:**
- Cargo (Rust) - Lockfile: present (`Cargo.lock`)
- npm (Node.js) - Lockfiles: present (`site/package-lock.json`, `wiki/package-lock.json`)

## Frameworks

**Core:**
- Axum 0.8 - High-performance asynchronous HTTP and SSE web server framework (`src/server.rs`)
- Tokio 1 (features: `full`) - Asynchronous multi-threaded runtime (`src/main.rs`)
- Tower 0.5 - Middleware abstractions and service combinators (`src/server.rs`)

**Testing:**
- Cargo Test - Built-in Rust test runner (`cargo test --all-features --workspace`)
- Wiremock 0.6 - HTTP mock server for protocol and integration tests (`tests/**`)
- Divan 5.0.1 (`codspeed-divan-compat`) - Benchmark framework integrated with CodSpeed (`benches/gateway.rs`, `benches/perf_issues.rs`)

**Build/Dev:**
- Cargo - Compiler orchestrator and package build tool (`Cargo.toml`)
- Astro 7.2.0 - Documentation site generator with Cloudflare Nimbus Docs (`site/astro.config.mjs`)
- Astro 7.0.7 / Starlight 0.41.3 - Documentation wiki generator (`wiki/astro.config.mjs`)
- Tailwind CSS 4.1.4 - Utility-first CSS framework for docs (`site/package.json`)

## Key Dependencies

**Critical:**
- `reqwest` 0.12 (`rustls-tls`, `stream`, `http2`, `json`) - Async HTTP client forwarding requests to upstream providers (`src/adapters/`)
- `tokio-tungstenite` 0.28 (patched to `openai-oss-forks`) & `tungstenite` 0.27 (`deflate`) - WebSocket transport with RFC 7692 permessage-deflate for OpenAI/Codex Responses v2 protocol (`src/adapters/responses/codex_ws.rs`)
- `tiktoken-rs` 0.12.0 - BPE token counting (`o200k_base`, `cl100k_base`) for request token estimation (`src/count_tokens.rs`)
- `rusqlite` 0.32 (`bundled`) - Embedded SQLite database for state persistence, account tracking, and quota budgets (`src/state_persist.rs`)
- `figment` 0.10 (`toml`, `yaml`, `env`) - Hierarchical configuration loader supporting TOML, YAML, and environment overrides (`src/config.rs`)
- `rustls` 0.23 (`aws_lc_rs`) - Pure-Rust TLS implementation without native OpenSSL dependencies (`src/main.rs`)
- `zstd` 0.13 & `flate2` 1 - Streaming request and response compression for upstream payloads (`src/compression.rs`)
- `clap` 4 (`derive`) - Command-line argument parsing for CLI subcommands (`src/main.rs`)

**Infrastructure:**
- `opentelemetry` 0.32 & `opentelemetry-otlp` 0.32 - Distributed telemetry, metric capture, and OTLP HTTP/protobuf export (`src/observability.rs`)
- `tracing` 0.1 & `tracing-subscriber` 0.3 - Structured application logging and spans (`src/main.rs`)
- `sentry` 0.48.4 - Error reporting and panic capture (`src/telemetry.rs`)
- `notify` 8.2.0 - Cross-platform filesystem watcher for configuration hot reloading (`src/reload.rs`)
- `arc-swap` 1.9.2 - Atomic lock-free configuration swapping during reload (`src/server.rs`)
- `uuid` 1 (`v4`) - UUID generation for request tracking and session IDs (`src/proxy.rs`)

## Configuration

**Environment:**
- Configured via `shunt.toml` or `shunt.yaml` in working directory or designated path via `SHUNT_CONFIG`
- Environment variable overrides supported through Figment (`SHUNT_*`)
- Inbound authentication via `[server.auth]` token lists or Bearer headers
- Upstream API credentials loaded from operator environment variables (e.g. `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `XAI_API_KEY`)

**Build:**
- `Cargo.toml` - Workspace package definitions, dependencies, features, and profile configurations
- `Cargo.lock` - Pinned crate dependency graph
- `release-please-config.json` & `.release-please-manifest.json` - Automated semantic release configuration
- `site/astro.config.mjs` & `wiki/astro.config.mjs` - Static site generation configs

## Platform Requirements

**Development:**
- Rust 1.80+ (stable toolchain) with `rustfmt` and `clippy` components
- Node.js >= 22.12 and npm (for docs development)
- Git

**Production:**
- Self-contained binary deployed on macOS (Darwin arm64/x64) or Linux (x64/arm64 musl)
- Optional service management via Homebrew services (`brew services start shunt`) or systemd
- Cloudflare Pages for documentation site hosting (`site/`, `wiki/`)

---

*Stack analysis: 2026-09-05*
