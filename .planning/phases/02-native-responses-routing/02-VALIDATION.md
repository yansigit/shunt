---
phase: "02"
slug: "native-responses-routing"
status: draft
nyquist_compliant: true
wave_0_complete: true
created: "2026-09-05"
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness, Tokio, Axum, wiremock, tokio-tungstenite |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test routing:: --lib && cargo test --test inbound_codex_endpoint && cargo test --test inbound_codex_websocket` |
| **Full suite command** | `cargo test --all-features --workspace` |
| **Estimated runtime** | ~120 seconds focused; several minutes full workspace |

## Sampling Rate

- **After every task commit:** Run the narrowest affected unit or integration target.
- **After every plan wave:** Run `cargo test routing:: --lib && cargo test --test inbound_codex_endpoint && cargo test --test inbound_codex_websocket`.
- **Before `/gsd:verify-work`:** `cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-features --workspace` must be green.
- **Max feedback latency:** 180 seconds for focused phase tests.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 02-01 | 1 | ROUTE-01, ROUTE-02 | T-02-01-01 | Exact native HTTP tracer selects one Responses route with original body bytes | e2e | `cargo test routing:: --lib && cargo test --test inbound_codex_endpoint exact_native_route` | ✅ | ⬜ pending |
| 02-01-02 | 02-01 | 1 | ROUTE-01, ROUTE-02 | T-02-01-01 | Exact/fallback/rejection matrix preserves pinned behavior and rejects unsafe declarations | unit/e2e | `cargo test routing:: --lib && cargo test --test inbound_codex_endpoint pinned_fallback` | ✅ | ⬜ pending |
| 02-02-01 | 02-02 | 2 | ROUTE-01, ROUTE-02 | T-02-02-01 | Provider-aware native credentials and URL flavor replace client credentials; unsupported auth makes zero network calls | integration | `cargo test --test inbound_codex_endpoint native_provider_auth && cargo test --test inbound_codex_endpoint unsupported_native_auth_no_network` | ✅ | ⬜ pending |
| 02-02-02 | 02-02 | 2 | ROUTE-01, ROUTE-02 | T-02-02-02 | zstd/body/status fidelity and no post-output HTTP hop are preserved | integration | `cargo test --test inbound_codex_endpoint zstd_native_route && cargo test --test inbound_codex_endpoint no_mid_stream_hop` | ✅ | ⬜ pending |
| 02-03-01 | 02-03 | 3 | ROUTE-01, ROUTE-02 | T-02-03-01 | WebSocket uses the same resolver/provider parity, disabled/boot opt-in behavior, and pre-dispatch rejection | integration | `cargo test --test inbound_codex_websocket native_route && cargo test --test inbound_codex_websocket pinned_fallback && cargo test --test inbound_codex_websocket disabled_endpoint && cargo test --test inbound_codex_websocket boot_time_opt_in` | ✅ | ⬜ pending |
| 02-03-02 | 02-03 | 3 | ROUTE-01, ROUTE-02 | T-02-03-01 | Snapshot immutability, reload isolation, cancellation, and no WS hop hold | integration | `cargo test --test inbound_codex_websocket hot_reload_snapshot && cargo test --test inbound_codex_websocket no_mid_stream_hop` | ✅ | ⬜ pending |
| 02-04-01 | 02-04 | 4 | ROUTE-01, ROUTE-02 | T-02-04-01 | English/source documentation changes consistently; wiki status is empty | source check | `test -z "$(git status --short -- wiki/)" && git diff --check && for f in docs/m11-inbound-codex-endpoint.md docs/codex-configuration.md docs/upstreams-failover.md README.md site/src/content/docs/reference/configuration.md site/src/content/docs/guides/inbound-codex-endpoint.md site/src/content/docs/guides/configuration.mdx; do test -s "$f" || exit 1; git diff --numstat -- "$f" | grep -Eq '^[0-9]+[[:space:]]+[0-9]+' || exit 1; done; grep -q 'prefix-only' docs/codex-configuration.md; grep -q 'pinned' docs/m11-inbound-codex-endpoint.md; grep -q 'unmatched' README.md` | ✅ | ⬜ pending |
| 02-04-02 | 02-04 | 4 | ROUTE-01, ROUTE-02 | T-02-04-02 | Validation map contains all ten plan task IDs and draft/Nyquist lifecycle metadata | source check | `test "$(grep -c '^| 02-' .planning/phases/02-native-responses-routing/02-VALIDATION.md)" -eq 10 && grep -q '02-05-02' .planning/phases/02-native-responses-routing/02-VALIDATION.md && grep -q 'nyquist_compliant: true' .planning/phases/02-native-responses-routing/02-VALIDATION.md` | ✅ | ⬜ pending |
| 02-05-01 | 02-05 | 5 | ROUTE-01, ROUTE-02 | T-02-05-01 | All maintained README/Nimbus translation files change consistently; wiki status is empty | source check | `test -z "$(git status --short -- wiki/)" && git diff --check && for f in README.ko.md README.ja.md README.zh-CN.md site/src/content/docs/ko/reference/configuration.md site/src/content/docs/ja/reference/configuration.md site/src/content/docs/zh-cn/reference/configuration.md site/src/content/docs/ko/guides/inbound-codex-endpoint.md site/src/content/docs/ja/guides/inbound-codex-endpoint.md site/src/content/docs/zh-cn/guides/inbound-codex-endpoint.md site/src/content/docs/ko/guides/configuration.md site/src/content/docs/ja/guides/configuration.md site/src/content/docs/zh-cn/guides/configuration.md; do test -s "$f" || exit 1; git diff --numstat -- "$f" | grep -Eq '^[0-9]+[[:space:]]+[0-9]+' || exit 1; done` | ✅ | ⬜ pending |
| 02-05-02 | 02-05 | 5 | ROUTE-01, ROUTE-02 | T-02-05-02 | Focused/full tests, fmt, clippy, diff/wiki guards, and validation lifecycle assertions pass | quality | `cargo test routing:: --lib && cargo test --test inbound_codex_endpoint && cargo test --test inbound_codex_websocket && cargo fmt --all --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features --workspace && git diff --check && test -z "$(git status --short -- wiki/)" && test "$(grep -c '^| 02-' .planning/phases/02-native-responses-routing/02-VALIDATION.md)" -eq 10 && grep -q 'status: draft' .planning/phases/02-native-responses-routing/02-VALIDATION.md && grep -q 'wave_0_complete: true' .planning/phases/02-native-responses-routing/02-VALIDATION.md && grep -q 'nyquist_compliant: true' .planning/phases/02-native-responses-routing/02-VALIDATION.md` | ✅ | ⬜ pending |

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. Resolver tests can live beside the routing code, while the existing HTTP and WebSocket integration targets already provide real server and mock-upstream fixtures.

## Manual-Only Verifications

All phase behaviors have automated verification. Locale prose requires review, but file-presence/parity and the underlying behavior are automated or source-checkable.

## Validation Sign-Off

- [x] All tasks have an automated verify command.
- [x] Sampling continuity: no three consecutive tasks without automated verification.
- [x] Wave 0 has no missing test-infrastructure dependency.
- [x] No watch-mode flags.
- [x] Focused feedback latency target is under 180 seconds.
- [x] `nyquist_compliant: true` set after plan IDs were finalized and all checks were mapped.

**Approval:** pending
