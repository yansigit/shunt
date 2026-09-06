---
phase: 02-native-responses-routing
reviewed: 2026-09-05T20:45:00Z
depth: standard
files_reviewed: 25
files_reviewed_list:
  - README.ja.md
  - README.ko.md
  - README.md
  - README.zh-CN.md
  - docs/codex-configuration.md
  - docs/m11-inbound-codex-endpoint.md
  - docs/upstreams-failover.md
  - site/src/content/docs/guides/configuration.mdx
  - site/src/content/docs/guides/inbound-codex-endpoint.md
  - site/src/content/docs/ja/guides/configuration.md
  - site/src/content/docs/ja/guides/inbound-codex-endpoint.md
  - site/src/content/docs/ja/reference/configuration.md
  - site/src/content/docs/ko/guides/configuration.md
  - site/src/content/docs/ko/guides/inbound-codex-endpoint.md
  - site/src/content/docs/ko/reference/configuration.md
  - site/src/content/docs/reference/configuration.md
  - site/src/content/docs/zh-cn/guides/configuration.md
  - site/src/content/docs/zh-cn/guides/inbound-codex-endpoint.md
  - site/src/content/docs/zh-cn/reference/configuration.md
  - src/adapters/responses/inbound.rs
  - src/codex_endpoint.rs
  - src/codex_endpoint/websocket.rs
  - src/routing.rs
  - tests/inbound_codex_endpoint.rs
  - tests/inbound_codex_websocket.rs
findings:
  critical: 0
  warning: 1
  info: 1
  total: 2
status: issues_found
---

# Phase 02: Code Review Report

**Reviewed:** 2026-09-05T20:45:00Z
**Depth:** standard
**Files Reviewed:** 25
**Status:** issues_found

## Summary

Reviewed Phase 02 native Responses routing across core routing logic, HTTP/SSE and WebSocket inbound passthrough, credential management, integration tests, and documentation across all four supported locales.

The core routing implementation in `src/routing.rs` adheres strictly to the exact-only specification: it matches only unique exact `[models.upstream_model]` or `[[routes]]` declarations, normalizes client `[1m]` hints safely for lookup, preserves uncompressed and compressed request bytes verbatim without rewriting, and rejects ambiguous, translated, or non-Responses configurations prior to network dispatch. Inbound authentication and credential injection strip caller tokens cleanly while enforcing provider credentials. WebSocket hot-reloading per turn was verified through end-to-end integration tests.

No Critical/Blocker defects were found. One Warning regarding model handling asymmetry in the WebSocket transport and one Info item regarding a stale comment were identified.

## Narrative Findings (AI reviewer)

## Warnings

### WR-01: WebSocket turn model prematurely collapses missing models to "unknown" instead of passing None

**File:** `src/codex_endpoint/websocket.rs:163`
**Issue:** In `src/codex_endpoint/websocket.rs`, when handling `ClientFrame::ResponseCreate`, line 163 evaluates `let turn_model = model.unwrap_or_else(|| "unknown".to_string());` and stores it in `TurnContext { model: turn_model, ... }`. When `run_turn` dispatches to `forward_turn` on line 210, it calls `forward_turn(state, Some(model), ...)`. In contrast, the HTTP path in `src/codex_endpoint.rs:294` evaluates `let model = (label != UNKNOWN_MODEL).then_some(label.clone());` and passes `None` when the model is absent or malformed. While `resolve_native_inbound` treats `None` as pinned fallback, passing `Some("unknown")` causes the resolver to attempt an exact match for a model literally named `"unknown"` in `config.models` and `config.routes`. If an operator happens to configure an exact route for `"unknown"`, WebSocket and HTTP dispatch will diverge.
**Fix:**
Preserve `model: Option<String>` in `TurnContext` and forward the original `Option<String>` directly to `forward_turn`:
```rust
active_turn_task = Some(tokio::spawn(async move {
    run_turn(TurnContext {
        state: state_clone,
        model,
        // ...
    }).await;
}));
```

## Info

### IN-01: Stale comment in `passthrough_send` regarding allowed auth modes

**File:** `src/adapters/responses/inbound.rs:453-455`
**Issue:** The comment preceding `Credential::ApiKey` states: `// A codex_endpoint provider is validated to be chatgpt_oauth, so only the arm above runs in practice; the rest keep the credential swap defensive without ever adding a synthetic client-identity header.` This comment is now obsolete because Phase 02 intentionally enables non-`chatgpt_oauth` native Responses providers (specifically `ApiKey` and `XaiOauth`), making this arm active in practice.
**Fix:**
Update the comment to document that `Credential::ApiKey` and `Credential::XaiOauth` are actively used by exact native Responses routes.

---

_Reviewed: 2026-09-05T20:45:00Z_
_Reviewer: Codex (gsd-code-reviewer)_
_Depth: standard_
