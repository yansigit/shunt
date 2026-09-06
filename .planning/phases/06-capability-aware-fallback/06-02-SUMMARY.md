---
phase: 06-capability-aware-fallback
plan: 02
status: complete
completed: 2026-09-06
requirements: [CAP-01]
---

# Plan 06-02 Summary

Documented and verified capability-aware fallback across the supported provider
families without adding public configuration or changing primary routing.

## Delivered

- Exposes one stable `capability_excluded` metric reason and structured warnings
  containing the excluded provider, model, and capability reasons.
- Documents the provider capability matrix and the `[1m]` no-fallback rule in
  the README, engineering guide, and Nimbus provider guide.
- Keeps the root README and Nimbus documentation synchronized across English,
  Korean, Japanese, and Simplified Chinese.
- Confirms ordinary requests preserve the configured chain, incompatible
  fallback targets receive no request, and the primary target is never filtered.

## Verification

- `cargo test capability --lib`
- `cargo test --test failover capability`
- `cargo test --test failover`
- `cargo test --test passthrough`
- `cargo fmt --all --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features --workspace`
- `git diff --check`
- `test -z "$(git status --short -- wiki/)"`

All passed. The full library run completed 2,041 tests with two intentional
ignores, followed by every binary, integration, and doc-test target.
