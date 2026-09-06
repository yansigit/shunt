---
phase: 03-quota-aware-resilience
plan: 02
status: complete
---

# Plan 03-02 Summary

Hardened the shared retry and quota-classification boundaries. `accounts::retry_after`
now accepts bounded decimal delta-seconds and all standards-supported HTTP-date forms,
rounds fractional waits upward, preserves immediate zero/past semantics, and clamps
honored waits to one hour. Retry policy tests prove that parsed values still respect
the configured budget. The quota classifier now uses a streaming serde visitor to
reject duplicate keys at any nesting level, conflicting root/nested discriminators,
invalid discriminator types, malformed JSON, and trailing data fail-closed as
transient. Pool response rebuilding preserves upstream headers when bounded body
inspection overflows or terminates with a read error.

## Commits

- `a384fdc` feat(resilience): bound retry-after and quota evidence
- `6f632b7` fix(resilience): reject padded overlong retry headers

## Verification

- `cargo fmt --all --check`
- `cargo test retry:: --lib`
- `cargo test --test retry`
- `cargo test adapters::responses::pool:: --lib`
- `cargo test --test codex_multi_account`
- `cargo test --test failover`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features --workspace`

## Deviations

- No public configuration, route failover policy, credential writeback, streaming
  boundary, or generated wiki files were changed.
- The existing 64 KiB quota inspection boundary remains in place; overflow and
  body-read failures now rebuild responses with their original headers before
  failing closed to transient classification.
