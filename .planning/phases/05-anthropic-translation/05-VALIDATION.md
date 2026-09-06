---
phase: 05-anthropic-translation
status: draft
wave_0_complete: false
created: 2026-09-05
---

# Phase 5 Validation

| Task | Requirement | Test seam | Status |
|------|-------------|-----------|--------|
| 05-01-01 | TRANS-01 | Pure request fixtures: instructions/messages/images/tools/results/reasoning | pending |
| 05-01-02 | TRANS-03 | Pre-dispatch rejection matrix and native byte-identity regression | pending |
| 05-02-01 | TRANS-02 | Non-stream output/tool/reasoning/usage/terminal fixtures | pending |
| 05-02-02 | TRANS-02 | Chunk-split SSE event-order/tool-fragment/usage/terminal fixtures | pending |
| 05-02-03 | TRANS-02 | HTTP and WebSocket translated integration tests | pending |
| 05-03-01 | TRANS-01, TRANS-03 | Docs and maintained locale parity | pending |
| 05-03-02 | TRANS-01, TRANS-02, TRANS-03 | fmt, clippy, full workspace, diff/wiki guards | pending |

## Final commands

```bash
cargo test inbound_responses --lib
cargo test --test inbound_anthropic_translation
cargo test --test inbound_codex_endpoint
cargo test --test inbound_codex_websocket
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features --workspace
git diff --check
test -z "$(git status --short -- wiki/)"
```

