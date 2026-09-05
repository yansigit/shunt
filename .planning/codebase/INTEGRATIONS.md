# External Integrations

**Analysis Date:** 2026-09-05

## APIs & External Services

**LLM Inference Providers:**
- Anthropic Messages API - Passthrough proxy for Claude models and default upstream fallback
  - SDK/Client: `reqwest` (`src/adapters/anthropic/mod.rs`)
  - Auth: Client request header passthrough or operator `ANTHROPIC_API_KEY` (`src/auth/mod.rs`)
- OpenAI & ChatGPT / Codex - Translation from Anthropic Messages to OpenAI Responses API over HTTP and WebSocket
  - SDK/Client: `tokio-tungstenite` (`src/adapters/responses/codex_ws.rs`), `reqwest` (`src/adapters/responses/http.rs`)
  - Auth: `OPENAI_API_KEY` or `~/.codex/auth.json` OAuth token (`src/auth/codex/mod.rs`)
- Google Gemini & Code Assist - Translation from Anthropic Messages to Gemini `generateContent`/`streamGenerateContent`
  - SDK/Client: `reqwest` (`src/adapters/gemini/mod.rs`)
  - Auth: `~/.gemini/oauth_creds.json` via `google_oauth`, with refresh support via `SHUNT_GOOGLE_CLIENT_ID` and `SHUNT_GOOGLE_CLIENT_SECRET` (`src/auth/google/mod.rs`)
- Antigravity - Native HTTP upstream backend (`v1internal:generateContent`) with `ideType: ANTIGRAVITY`
  - SDK/Client: `reqwest` (`src/adapters/antigravity/mod.rs`, `src/model/antigravity_request.rs`)
  - Auth: `~/.shunt/antigravity-auth.json` via `shunt login antigravity` (`src/auth/antigravity/mod.rs`)
- Cursor - ConnectRPC AgentService protocol (`agent.v1.AgentService/Run`) over HTTP/2
  - SDK/Client: `reqwest` with HTTP/2 ALPN (`src/adapters/cursor/agent.rs`, `src/adapters/cursor/connect.rs`)
  - Auth: `~/.shunt/cursor-auth.json` via `shunt login cursor` (`src/auth/cursor/mod.rs`)
- xAI & Grok - xAI Responses API and Grok subscription proxy support
  - SDK/Client: `reqwest` (`src/adapters/responses/http.rs`)
  - Auth: `XAI_API_KEY` or `~/.shunt/xai-auth.json` via `shunt login xai` (`src/auth/xai/mod.rs`)
- Kimi Code (Moonshot AI) - Kimi Code subscription and Anthropic-compatible API endpoint
  - SDK/Client: `reqwest` (`src/adapters/anthropic/mod.rs`)
  - Auth: `~/.shunt/accounts/kimi/<name>.json` via `shunt login kimi` or `KIMI_API_KEY` (`src/auth/kimi/mod.rs`)
- Anthropic-Compatible Backends (DeepSeek, Zhipu GLM, MiniMax, OpenRouter, Vercel AI Gateway) - Table-configured upstream inference destinations
  - SDK/Client: `reqwest` (`src/adapters/anthropic/mod.rs`)
  - Auth: Injected API keys configured per provider via `api_key_env` (e.g. `DEEPSEEK_API_KEY`)

## Data Storage

**Databases:**
- SQLite (bundled embedded database)
  - Connection: Local database path configured via `[server.state_file]` (default: `~/.shunt/state.db`)
  - Client: `rusqlite` (`src/state_persist.rs`)

**File Storage:**
- Local filesystem only - Stores configuration files, OAuth tokens (`~/.shunt/*`, `~/.codex/auth.json`, `~/.gemini/oauth_creds.json`), and atomic state dumps (`src/atomic_file.rs`)

**Caching:**
- In-memory connection and session caching - Responses connection pool (`src/adapters/responses/pool.rs`), concurrency slots (`src/concurrency.rs`), and account quotas (`src/accounts.rs`)

## Authentication & Identity

**Auth Provider:**
- Custom multi-tier identity and credential engine
  - Implementation: Inbound client token authentication (`[server.auth]` in `src/auth/inbound.rs`), Admin keyring authentication (`[server.admin]` in `src/admin/mod.rs`), Claude apps gateway OIDC/JWT verification (`src/gateway/auth.rs`), and outbound upstream credential resolution supporting API keys, OAuth2 refresh tokens, PKCE loopback, and device-code flows (`src/auth/mod.rs`)

## Monitoring & Observability

**Error Tracking:**
- Sentry - Automatic exception and panic reporting (`src/telemetry.rs`), configured via `[sentry]` table or `SENTRY_DSN`

**Logs:**
- Structured logging via `tracing` and `tracing-subscriber` to standard output or log files (e.g. `$(brew --prefix)/var/log/shunt.log`)
- OpenTelemetry (OTel) metrics and distributed tracing via OTLP HTTP/protobuf exporter (`src/observability.rs`)

## CI/CD & Deployment

**Hosting:**
- Cloudflare Pages - Documentation site (`shunt-docs`) and documentation wiki (`shunt-wiki`)
- GitHub Releases & Homebrew Tap (`pleaseai/tap/shunt`) - Cross-platform binary distribution

**CI Pipeline:**
- GitHub Actions - Formatting, clippy, unit/integration testing (`.github/workflows/ci.yml`), CodSpeed benchmarks (`.github/workflows/codspeed.yml`), automated releases via Release Please (`.github/workflows/release-please.yml`, `.github/workflows/release.yml`), and docs deployment (`.github/workflows/deploy-docs.yml`, `.github/workflows/deploy-wiki.yml`)

## Environment Configuration

**Required env vars:**
- None required for default unmapped Anthropic passthrough
- Provider keys when mapped: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `XAI_API_KEY`, `DEEPSEEK_API_KEY`, etc.
- Operational overrides: `SHUNT_CONFIG`, `SHUNT_GOOGLE_CLIENT_ID`, `SHUNT_GOOGLE_CLIENT_SECRET`, `SENTRY_DSN`, `OTEL_EXPORTER_OTLP_ENDPOINT`

**Secrets location:**
- Environment variables or local JSON credential files (`~/.shunt/`, `~/.codex/auth.json`, `~/.gemini/oauth_creds.json`); never logged or exposed via endpoints

## Webhooks & Callbacks

**Incoming:**
- Local HTTP callback endpoint (`http://127.0.0.1:<port>/callback`) for browser OAuth authentication flows (`src/auth/callback.rs`)

**Outgoing:**
- OpenTelemetry OTLP trace, metric, and log export endpoints (`src/observability.rs`)
- Sentry event ingestion endpoint (`src/telemetry.rs`)

---

*Integration audit: 2026-09-05*
