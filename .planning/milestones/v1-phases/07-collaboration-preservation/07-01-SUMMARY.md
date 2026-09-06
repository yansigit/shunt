---
phase: 07-collaboration-preservation
plan: 01
status: complete
completed: 2026-09-06
requirements-completed: [COLLAB-01]
---

# Plan 07-01 Summary

Added the default-off request-side Codex V2 collaboration bridge for exact
Anthropic translation routes.

## Delivered

- Adds `[server.codex_endpoint] collaboration = true` as the sole activation
  switch; absent and false preserve the prior rejection behavior.
- Collects `collaboration` namespace function tools from top-level and
  `additional_tools` catalogs, flattens them to collision-resistant Anthropic
  names, and retains an immutable authority map.
- Removes only schema-position `encrypted` markers while retaining property
  names and literal values, with a 64-level nesting bound.
- Preserves plaintext `agent_message` content as a separate user turn and maps
  `metadata.task_id` to Anthropic `metadata.user_id`.
- Detects structurally valid ciphertext-only trailing tasks and rejects them
  locally without logging ciphertext or issuing recovery requests.

## Verification

- `cargo test inbound_responses:: --lib`
- `cargo test codex_endpoint_collaboration_is_explicit --lib`
- `cargo fmt --all --check`
- `cargo clippy --all-targets --all-features -- -D warnings`

All passed.
