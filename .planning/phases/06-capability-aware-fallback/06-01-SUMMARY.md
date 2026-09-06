---
phase: 06-capability-aware-fallback
plan: 01
status: complete
completed: 2026-09-06
requirements: [CAP-01]
---

# Plan 06-01 Summary

Added a bounded, internal capability gate for ordered fallback chains. The
configured primary remains authoritative, while later candidates that cannot
faithfully preserve detected request features are removed before authentication,
credential lookup, or network dispatch.

## Delivered

- Detects non-empty tools, base64 and URL images (including tool-result content),
  structured output, explicit reasoning effort, and the trailing `[1m]` model
  hint from known Anthropic Messages fields.
- Applies an adapter-specific compatibility matrix to fallback candidates only;
  malformed or unknown request shapes remain the downstream adapter's concern.
- Conservatively suppresses every fallback for `[1m]` because Shunt has no
  trustworthy per-target context-window metadata.
- Emits provider/model/reason warnings and the fixed-cardinality
  `capability_excluded` failover metric state.
- Proves excluded candidates receive zero network calls, compatible later
  candidates remain reachable, the primary is never filtered, and ordinary
  requests preserve their complete chain.

## Verification

- `cargo test capability --lib`
- `cargo test --test failover capability`
- `cargo test --test failover`
- `cargo fmt --all --check`
- `cargo clippy --all-targets --all-features -- -D warnings`

All passed.
