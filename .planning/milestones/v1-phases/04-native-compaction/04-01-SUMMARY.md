---
phase: 04-native-compaction
plan: 01
status: complete
completed: 2026-09-05
requirements-completed: [COMP-01]
commits: [df8866b]
---

# 04-01 Summary — Native Compact Vertical Slice

## Delivered

- Registered HTTP-only `POST /v1/responses/compact` with the existing opt-in inbound Codex router and OpenAI-shaped gateway error classification.
- Added strict bounded compact model validation, including duplicate-model rejection and zstd-aware parsing while preserving the original wire body.
- Reused the Phase 2 route decision, rejecting ambiguous/translated/non-Responses targets and providers without a verified native compact endpoint before network dispatch.
- Added the internal Responses/Compact operation boundary so the existing credential resolution, ChatGPT account pool, refresh/quota rotation, request-header filtering, TTFB timeout, response relay, and admission lifecycle remain shared; only the upstream URL changes.
- Added exact capability detection for ChatGPT/Codex OAuth and the canonical official OpenAI API endpoint.
- Added five focused integration fixtures covering opt-in registration, inbound auth, pinned and exact routing, correct compact URL, byte/header/credential fidelity, upstream error relay, strict invalid input, and unsupported-provider zero-network behavior.

## Verification

- `cargo test routing:: --lib`: 26 passed.
- `cargo test --test inbound_codex_endpoint`: 38 passed.
- `cargo test --test codex_multi_account`: 20 passed.
- `cargo fmt --all --check`: passed.
- strict all-target/all-feature Clippy: passed.

## Scope

No public configuration, credential writeback, local history, continuation cache, body translation, synthetic summarization, WebSocket compaction, or generated wiki change was introduced.
