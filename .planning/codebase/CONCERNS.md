# Codebase Concerns

**Analysis Date:** 2026-09-05

## Tech Debt

**Monolithic source modules with inline test suites:**
- Issue: Several core source files have grown to several thousand lines, bundling production logic and extensive unit test modules together. This contradicts the architectural rule to keep Rust files focused and under 500 lines.
- Files: `src/accounts.rs` (9,236 lines; lines 3083+ are tests), `src/config.rs` (8,890 lines; lines 4024+ are tests), `src/admin/plan.rs` (3,320 lines), `src/adapters/responses/codex_ws.rs` (3,074 lines), `src/auth/antigravity/auth.rs` (2,702 lines), `src/usage_poll.rs` (2,648 lines)
- Impact: Slower compilation times, high cognitive overhead when navigating code, frequent merge conflicts, and decreased reviewability.
- Fix approach: Split large modules into dedicated sub-packages with distinct domain submodules, and extract extensive test suites into separate `tests/` integration files or dedicated sibling `tests.rs` submodules.

**Cursor transport retry and framing workarounds:**
- Issue: Cursor request framing reconstructs frames rather than reusing cheap `Bytes` clones, and streaming retry logic has deferred idempotency handling (`TODO(#126, cursor)`, `TODO(#170)`).
- Files: `src/adapters/cursor/mod.rs`, `src/adapters/cursor/client.rs`, `src/retry.rs`
- Impact: Unnecessary memory allocations and CPU overhead during request framing, and limited recovery options on transient Cursor streaming errors.
- Fix approach: Implement buffer cloning for reusable payload bytes and establish a stable idempotency identity for safe streaming pre-response retries.

**Antigravity late subagent error suppression:**
- Issue: `agy --print` exposes interactive multi-agent tools that it cannot service, causing Gemini to emit fatal `recipient "<name>" not found` errors. Shunt matches on this raw upstream error string to suppress failure if partial text was received.
- Files: `src/adapters/antigravity/stream.rs`
- Impact: Fragile dependency on exact upstream CLI error message strings; any upstream wording change breaks error detection or causes false successes.
- Fix approach: Remove string matching once `agy` CLI supports an explicit tool deny flag at spawn time.

**Quad-locale documentation synchronization burden:**
- Issue: Complete documentation parity must be maintained across four languages (English, Korean, Japanese, Simplified Chinese) for both root READMEs and all `site/src/content/docs/` pages, with fragile Astro-generated anchor headings.
- Files: `README.md`, `README.ko.md`, `README.ja.md`, `README.zh-CN.md`, `site/src/content/docs/`
- Impact: High maintenance drag on every PR modifying user-facing features, with risk of dead fragment links due to translated heading IDs.
- Fix approach: Automate translation checks and anchor verification in CI, or adopt a structured i18n content pipeline.

## Known Bugs

**Code Assist REST capacity exhaustion on Gemini 3.1 Pro and 3 Flash:**
- Symptoms: Requests return HTTP 429 `RESOURCE_EXHAUSTED` with message "No capacity available for model on the server".
- Files: `src/adapters/antigravity/`, `docs/notes/issue-gemini-3.1-capacity.md`
- Trigger: Routing `gemini-3.1-pro-preview` or `gemini-3-flash-preview` through the Google Code Assist REST endpoint (`cloudcode-pa.googleapis.com`).
- Workaround: Route Gemini 3.1 Pro and 3 Flash requests through `agy` CLI via gRPC/Antigravity infrastructure, or use Gemini 2.5 Pro / Flash.

**Antigravity print mode tool failure:**
- Symptoms: Model attempts to use multi-agent tools (e.g. `send_message`) in single-turn print mode, resulting in fatal upstream errors.
- Files: `src/adapters/antigravity/stream.rs`, `docs/notes/agy-print-mode-tool-surface.md`
- Trigger: Prompting Antigravity with subagent or collaborative instructions.
- Workaround: The translator parses the error and converts terminal failures into success if non-whitespace output was already streamed.

**In-stream rate limit errors on Codex WebSocket:**
- Symptoms: Rate limit errors delivered mid-WebSocket stream instead of at connection establishment.
- Files: `src/adapters/responses/codex_ws.rs`
- Trigger: High volume of requests reaching upstream Codex quota during an active streaming response.
- Workaround: Synthetic translation of in-stream `rate_limit_exceeded` frames into standard HTTP 429 status and client steering signals.

## Security Considerations

**Degradation of file locks on non-Unix platforms:**
- Risk: Advisory file locking via `libc::flock` is unsupported on non-Unix systems and degrades to a no-op with a logged warning, risking credential file race conditions and corruption.
- Files: `src/auth/shared/file_lock.rs`
- Current mitigation: Logs a one-time runtime warning on non-Unix platforms.
- Recommendations: Implement native Windows file locking via `LockFileEx` through the `windows-sys` crate.

**Unsafe process management in Antigravity supervisor:**
- Risk: Direct `libc::kill`, `libc::getpgid`, and `libc::setpgid` calls in process group supervisor risk sending signals to PID 0 or unintended processes if PIDs are recycled.
- Files: `src/adapters/antigravity/child.rs`, `src/shutdown.rs`
- Current mitigation: PID validation checks and process group isolation.
- Recommendations: Wrap process management in a hardened abstraction and verify PID ownership before signaling.

**SSRF and open redirect vectors on OAuth and token refresh:**
- Risk: Upstream discovery or token refresh URLs could be manipulated to target internal or unsafe network resources.
- Files: `src/auth/shared.rs`, `src/auth/codex/auth.rs`, `src/auth/gateway/auth.rs`
- Current mitigation: Strict URL parsing, loopback verification, and rejection of plaintext or off-host hops in `is_safe_refresh_url`.
- Recommendations: Maintain continuous test coverage for all URL normalization and redirect paths.

**Sensitive token and session leakage in telemetry:**
- Risk: Session IDs, API keys, and authorization headers could be leaked into external tracing collectors (Sentry, OpenTelemetry) or local logs.
- Files: `src/proxy.rs`, `src/model/antigravity_request.rs`, `src/telemetry.rs`
- Current mitigation: Explicit span gating (`telemetry::withhold_session_id`), sanitization of logged IDs, and omission of sensitive request payloads.
- Recommendations: Add automated static analysis/linters to detect credentials in log arguments.

## Performance Bottlenecks

**Inbound request body buffering up to 32 MiB:**
- Problem: Inbound request bodies are fully buffered into memory before routing or parsing.
- Files: `src/proxy/failover.rs`, `src/http_tuning.rs`, `src/codex_endpoint.rs`
- Cause: Routing and failover require inspecting the JSON body to determine model, tools, and provider chain. The default limit `server.limits.max_request_bytes` is 32 MiB.
- Improvement path: Stream request bodies when possible or extract routing metadata (e.g. model name) from initial chunks without buffering full multi-megabyte payloads.

**Account pool and token store lock contention:**
- Problem: High-frequency concurrent requests contend on shared memory locks when selecting accounts and updating health/quota states.
- Files: `src/accounts.rs`, `src/gateway/refresh.rs`, `src/gateway/spend/store.rs`
- Cause: Synchronization uses `std::sync::Mutex` around complex shared structs (`AccountPool`, token stores).
- Improvement path: Transition to fine-grained sharded locks, lock-free structures, or read-mostly `ArcSwap` collections.

**Synchronous model catalog fetching on request path:**
- Problem: Cache misses on Antigravity model catalog trigger synchronous 5-second bounded network requests.
- Files: `src/auth/antigravity/catalog.rs`
- Cause: Dynamic model tier resolution requires a fresh catalog from `fetchAvailableModels`.
- Improvement path: Pre-fetch and background-refresh catalogs asynchronously outside the hot request path.

## Fragile Areas

**Multi-protocol translation matrices:**
- Files: `src/model/responses.rs`, `src/model/antigravity_request.rs`, `src/adapters/cursor/tool_use_xml.rs`, `src/adapters/cursor/agent.rs`
- Why fragile: Bidirectional translation between Anthropic Messages, OpenAI Responses, Google Code Assist, and Cursor Protobuf/XML formats. Subtleties like tool definitions, role alternations, and thinking tokens frequently break when upstreams alter schemas.
- Safe modification: Run full test suites in `tests/responses_translate.rs` and add targeted unit tests for every schema edge case before altering translation tables.
- Test coverage: High unit test coverage exists, but mocks cannot capture unannounced upstream wire changes.

**Codex WebSocket state machine:**
- Files: `src/adapters/responses/codex_ws.rs`
- Why fragile: Manages connection pooling, HTTP fallback, idle timeouts, frame fragmentation, and compression negotiation across async tasks.
- Safe modification: Test state transitions under simulated network disconnects and frame delays.
- Test coverage: Comprehensive mock suite, but timing-dependent edge cases can cause race conditions.

**Dynamic model catalog resolution:**
- Files: `src/auth/antigravity/catalog.rs`, `src/adapters/antigravity/`
- Why fragile: Google's daily backend (`daily-cloudcode-pa.googleapis.com`) returns fluctuating model IDs (`-high`, `-medium`, `-tiered`) that change per account and over time.
- Safe modification: Rely on fallback heuristics when catalog discovery fails, and verify all tier-clamping rules against live probes.
- Test coverage: Mocked in tests, but depends on live behavior documented in `docs/notes/antigravity-daily-host.md`.

## Scaling Limits

**Ingress concurrency capacity:**
- Current capacity: Bounded CPU worker semaphores (`src/offload.rs`, clamping to 2-16 tasks), but unthrottled gateway ingress.
- Limit: Server resources can be exhausted under burst traffic; tracked in issue #260.
- Scaling path: Introduce an explicit global ingress concurrency limiter and active connection bounding in Axum middleware.

**Single-node in-memory state:**
- Current capacity: In-memory session tracking, quota windows, and account health state tied to a single process instance.
- Limit: Cannot scale horizontally across multiple instances without state desynchronization.
- Scaling path: Externalize shared state to distributed key-value storage (e.g. Redis) or coordinate via distributed gossip.

**Spend audit log buffer:**
- Current capacity: `MAX_AUDIT_RECORDS` capped at 10,000 entries in memory.
- Limit: High-throughput spend tracking rolls over or requires periodic disk writeback.
- Scaling path: Implement streaming persistence or ring-buffer compaction.

## Dependencies at Risk

**Forked tokio-tungstenite and tungstenite dependencies:**
- Risk: Pinned to custom GitHub forks under `[patch.crates-io]` from `openai-oss-forks` for RFC 7692 permessage-deflate support.
- Impact: Prevents publishing the crate to crates.io (`publish = false` in `Cargo.toml`, issue #292).
- Migration plan: Remove forks once upstream `snapview/tungstenite-rs#426` is merged and released to crates.io.

**Private / undocumented Google internal APIs:**
- Risk: Endpoints like `cloudcode-pa.googleapis.com/v1internal:streamGenerateContent` and `daily-cloudcode-pa.googleapis.com` are internal Google endpoints.
- Impact: Google can modify or decommission endpoints without warning.
- Migration plan: Provide fallback pathways to standard Gemini Developer APIs or CLI-based adapters (`agy`).

## Missing Critical Features

**Full Windows native platform support:**
- Problem: Reliance on Unix-specific `libc` calls (`flock`, process signaling) restricts first-class operation on Windows outside WSL.
- Blocks: Native Windows deployments without WSL2.

**PKCE loopback authentication fallback:**
- Problem: `shunt login` lacks a complete PKCE loopback fallback flow (`src/auth/mod.rs` TODO(M2)).
- Blocks: Seamless CLI authorization when manual copy-paste is undesirable.

## Test Coverage Gaps

**Live upstream authentication lifecycle:**
- What's not tested: Real OAuth token refresh loops and expiration across time cannot be validated in CI without live credentials.
- Files: `src/auth/codex/auth.rs`, `src/auth/antigravity/auth.rs`, `src/auth/kimi/auth.rs`
- Risk: Unannounced upstream OAuth flow or token schema changes break authentication silently.
- Priority: High

**High-concurrency account pool contention:**
- What's not tested: Simultaneous contention across dozens of threads rotating accounts under rapid rate-limiting.
- Files: `src/accounts.rs`, `src/admin/plan.rs`
- Risk: Latent deadlocks or starvation under enterprise load.
- Priority: Medium

---

*Concerns audit: 2026-09-05*
