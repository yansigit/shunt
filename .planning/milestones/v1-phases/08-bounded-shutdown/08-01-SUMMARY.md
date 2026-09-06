---
phase: 08-bounded-shutdown
plan: 01
status: complete
completed: 2026-09-06
requirements-completed: [OPS-01]
---

# Plan 08-01 Summary

Added a validated process-level shutdown deadline and connected it to Axum's
graceful-shutdown lifecycle.

## Delivered

- Adds `[server] shutdown_timeout_seconds` with a 30-second default and an
  inclusive `1..=3600` validation range.
- Starts the deadline only after the first SIGTERM or SIGINT has triggered
  Axum's admission fence.
- Distinguishes clean drain from deadline expiry while propagating server
  errors unchanged.
- Captures the deadline at process boot and warns when config reload attempts
  to change the restart-only value.
- Retains the existing process-group cleanup and second-signal immediate-exit
  behavior.

## Verification

- `cargo test --lib shutdown_timeout`
- `cargo test --bin shunt shutdown`
- `cargo check --all-features`

All passed.
