---
phase: "02"
slug: "native-responses-routing"
status: verified
# threats_open = count of OPEN threats at or above workflow.security_block_on severity (the blocking gate)
threats_open: 0
asvs_level: 1
created: "2026-09-05"
---

# Phase 02 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Client -> HTTP ingress | Untrusted compressed/uncompressed body and model classification input | Request bytes, model field, headers |
| Gateway -> upstream | Selected route and opaque request bytes cross provider boundary | Route, credentials, headers, body |
| Client -> gateway headers | Credentials, cookies, and hop-by-hop metadata are attacker-controlled | HTTP headers |
| Gateway -> provider auth | Gateway chooses credential flavor and upstream URL | Provider credential and request |
| Upstream stream -> client | Status/body bytes become observable and establish replay boundary | SSE/JSON status, headers, body |
| WebSocket client -> live-turn supervisor | Concurrent untrusted frames and replacement signals | WS frames, generation signals |
| Upstream SSE -> WebSocket | First event establishes an observable replay boundary | SSE frames and terminal state |
| Documentation -> operator | Incorrect contract can cause unsafe provider/configuration use | Routing and credential instructions |
| Validation -> implementation | Missing mapping can silently leave behavior untested | Task IDs, commands, threat refs |
| Translation -> operator | Stale or contradictory locale text can cause unsafe routing configuration | Locale documentation |
| Validation -> release | Incomplete gates can permit undocumented behavior drift | Quality-gate evidence |

## Threat Register

| Threat ID | Category | Component | Severity | Disposition | Mitigation | Status |
|-----------|----------|-----------|----------|-------------|------------|--------|
| T-02-01-01 | Tampering | native resolver | high | mitigate | Exact-only typed decisions, translation rejection, and field-level tests | closed — `src/routing.rs:129-205`; routing matrix and `exact_native_route`/`pinned_fallback` pass |
| T-02-01-02 | Information disclosure | HTTP passthrough | high | mitigate | Credential/header filtering and request-byte assertions | closed — `src/adapters/responses/inbound.rs:345-425,435-475`; `native_provider_auth`, header-filtering, and byte-fidelity tests pass |
| T-02-01-03 | Denial of service | model inspection | medium | mitigate | Bounded body and zstd inspection before resolver invocation | closed — `src/codex_endpoint.rs:394-496`; `zstd_native_route` and body-limit tests pass |
| T-02-02-01 | Information disclosure | inbound dispatcher | high | mitigate | Header stripping plus provider credential injection | closed — `src/adapters/responses/inbound.rs:345-475`; `native_provider_auth` and `unsupported_native_auth_no_network` pass |
| T-02-02-02 | Tampering | relay_passthrough | high | mitigate | Original `Bytes`, lazy stream, and synchronized first-output no-replay test | closed — `src/adapters/responses/inbound.rs:429-475,492-515`; `zstd_native_route`, SSE fidelity, and `no_mid_stream_hop` pass |
| T-02-03-01 | Tampering | turn route state | high | mitigate | Per-turn refreshed snapshot/decision and generation checks | closed — `src/codex_endpoint/websocket.rs:131-210`; `hot_reload_snapshot_routes_each_websocket_turn_once`, parity, and replacement tests pass |
| T-02-03-02 | Denial of service | WS relay | medium | mitigate | 4 MiB WS bound, bounded SSE frames, one-slot queue, cancellation, awaited sends | closed — `src/codex_endpoint/websocket.rs:75-189,275-315`; full WS suite including cancellation and bounded framing passes |
| T-02-04-01 | Tampering | English docs | medium | mitigate | Per-file diff, semantic fallback checks, and wiki-empty assertion | closed — seven source docs changed; `git diff --check`, semantic grep, and wiki guard pass |
| T-02-04-02 | Repudiation | validation matrix | medium | mitigate | Exact task IDs, commands, wave metadata, and sampling continuity | closed — `02-VALIDATION.md:37-48` has all 10 rows; lifecycle/Nyquist checks pass |
| T-02-05-01 | Tampering | locale docs | medium | mitigate | Every translation file non-empty and changed; diff/wiki guards fail closed | closed — all 12 locale files changed and non-empty; diff/wiki guards pass |
| T-02-05-02 | Repudiation | quality gates | high | mitigate | Focused/full tests, fmt, clippy, row count, and lifecycle assertions | closed — focused suites pass; `cargo fmt --all --check`, clippy `-D warnings`, and full workspace pass (2,024 passed, 2 ignored) |

*Status: open · closed · open — below high threshold (non-blocking)*
*Severity: critical > high > medium > low — only open threats at or above workflow.security_block_on count toward threats_open*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

## Accepted Risks Log

No accepted risks.

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-09-05 | 12 | 12 | 0 | gsd-security-auditor |

## Unregistered Flags

None. No `## Threat Flags` entries were present in the Phase 02 summaries.

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-09-05
