---
phase: "03"
slug: "quota-aware-resilience"
status: validated
nyquist_compliant: true
wave_0_complete: true
created: "2026-09-06"
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for quota classification, bounded retry timing, account-pool exhaustion, and documentation parity.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness, Tokio, Axum, reqwest, wiremock |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test --test inbound_codex_endpoint hard_quota_rotates_without_revisit && cargo test --test codex_multi_account -- exhausted_pool_preserves_final_body` |
| **Full suite command** | `cargo test --all-features --workspace` |
| **Focused latency target** | under 180 seconds |

## Sampling Rate

- **After every task:** Run the exact automated command in that task row below.
- **After each implementation wave:** Run the focused quota, retry, pool, and failover commands from the wave row.
- **Before `/gsd:verify-work`:** Run the final format, strict Clippy, full workspace, diff, and generated-wiki guards.
- **Max feedback latency:** 180 seconds for focused phase tests.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 03-01-01 | 03-01 | 1 | RES-01 | T-03-01-01, T-03-01-02 | Hard-quota evidence is classified before output; account rotation avoids revisit; all-eligible exhaustion preserves final status/body/safe Retry-After | e2e/integration | `cargo test --test inbound_codex_endpoint hard_quota_rotates_without_revisit && cargo test --test codex_multi_account -- exhausted_pool_preserves_final_body` | ✅ | complete |
| 03-01-02 | 03-01 | 1 | RES-01 | T-03-01-01, T-03-01-03 | Translated and refreshed Responses pools reuse the typed decision and preserve existing non-quota behavior | unit/integration | `cargo test adapters::responses::pool:: --lib && cargo test --test codex_multi_account` | ✅ | complete |
| 03-02-01 | 03-02 | 2 | RES-02 | T-03-02-01 | Decimal/datetime Retry-After values are rounded, normalized, bounded, and consumed by retry policy | unit/integration | `cargo test retry_after --lib && cargo test retry:: --lib && cargo test --test retry` | ✅ | complete |
| 03-02-02 | 03-02 | 2 | RES-01, RES-02 | T-03-02-02, T-03-02-03 | Adversarial bodies fail closed; stream boundaries and unrelated route failover remain unchanged | unit/integration | `cargo test adapters::responses::pool:: --lib && cargo test --test codex_multi_account && cargo test --test failover` | ✅ | complete |
| 03-03-01 | 03-03 | 3 | RES-01, RES-02 | T-03-03-01 | English docs state exact transient/hard-quota, bounded timing, final relay, and pre-output semantics | source check | `git diff --check && test -s README.md && test -s site/src/content/docs/guides/inbound-codex-endpoint.md && grep -q 'Retry-After' README.md && grep -q '429' site/src/content/docs/guides/inbound-codex-endpoint.md` | ✅ | complete |
| 03-03-02 | 03-03 | 3 | RES-01, RES-02 | T-03-03-02, T-03-03-03 | Maintained locales match English, wiki remains untouched, and all focused/full quality gates pass | source/quality | `for f in README.ko.md README.ja.md README.zh-CN.md site/src/content/docs/ko/guides/inbound-codex-endpoint.md site/src/content/docs/ja/guides/inbound-codex-endpoint.md site/src/content/docs/zh-cn/guides/inbound-codex-endpoint.md; do test -s "$f" || exit 1; done && git diff --check && test -z "$(git status --short -- wiki/)" && cargo test retry_after --lib && cargo test retry:: --lib && cargo test adapters::responses::pool:: --lib && cargo test --test inbound_codex_endpoint && cargo test --test codex_multi_account && cargo test --test failover && cargo fmt --all --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features --workspace` | ✅ | complete |

## Wave Structure

| Wave | Plans | Required sampling |
|------|-------|-------------------|
| 0 | Test scaffolding/gaps listed below | Unit classifier/timing tables plus integration fixture names must exist before implementation execution. |
| 1 | 03-01 | `cargo test --test inbound_codex_endpoint hard_quota_rotates_without_revisit && cargo test --test codex_multi_account -- exhausted_pool_preserves_final_body` |
| 2 | 03-02 | `cargo test retry_after --lib && cargo test retry:: --lib && cargo test adapters::responses::pool:: --lib && cargo test --test codex_multi_account && cargo test --test failover` |
| 3 | 03-03 | Documentation checks plus final repository quality gates in row 03-03-02 |

## Wave 0 Requirements

These are test additions, not production dependencies. They must be created before the corresponding implementation task is considered executable:

| ID | Required evidence | Owner task | Readiness command |
|----|-------------------|------------|------------------|
| W0-01 | Table-driven quota classifier cases for exact hard codes, plain 429, 402, mismatches, and provider transient precedence | 03-01-01 / 03-02-02 | `cargo test accounts:: --lib` |
| W0-02 | Fail-closed body cases for duplicate keys, malformed/empty JSON, invalid UTF-8, truncation, timeout/cancellation, and oversized input | 03-02-02 | `cargo test adapters::responses::pool:: --lib` |
| W0-03 | Multi-account hard-quota rotation plus all-eligible exhaustion with final status/body/safe Retry-After assertions | 03-01-01 | `cargo test --test codex_multi_account -- exhausted_pool_preserves_final_body` |
| W0-04 | Decimal/date/past/zero/overlong/excessive Retry-After unit cases | 03-02-01 | `cargo test retry_after --lib` |

## Final Phase Gates

The phase cannot be marked complete until all six task rows pass and this command succeeds:

```text
cargo fmt --all --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features --workspace && git diff --check && test -z "$(git status --short -- wiki/)" && test "$(grep -c '^| 03-' .planning/phases/03-quota-aware-resilience/03-VALIDATION.md)" -eq 6 && grep -q 'nyquist_compliant: true' .planning/phases/03-quota-aware-resilience/03-VALIDATION.md
```

## Manual-Only Verifications

Locale prose requires human language review, but all behavior, file presence, parity markers, and quality gates are automated or source-checkable. No manual credential setup or external service is required.

## Validation Sign-Off

- [x] Every plan task has an executable automated verification command.
- [x] Sampling continuity: no three consecutive tasks without automated verification.
- [x] Wave 0 test additions are present and passing.
- [x] No watch-mode flags or unbounded fallback commands.
- [x] Focused feedback latency target is under 180 seconds.
- [x] `status: validated` and `wave_0_complete: true` set after execution evidence.

**Approval:** validated on 2026-09-05 after focused tests and repository quality gates passed.

## Validation Audit 2026-09-05

| Metric | Count |
|--------|-------|
| Gaps found | 1 |
| Resolved | 1 |
| Escalated | 0 |

The prior Retry-After command used the filter `accounts::retry_after`, which matched zero tests. It was corrected to `retry_after`; the corrected command ran 16 matching tests successfully. Quota classifier, pool rotation, exhausted-pool response fidelity, route failover, format, Clippy, and full workspace tests also passed.
