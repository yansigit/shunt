---
phase: 07-collaboration-preservation
status: draft
wave_0_complete: false
created: 2026-09-06
---

# Phase 7 Validation

| Task | Requirement | Test seam | Status |
|------|-------------|-----------|--------|
| 07-01-01 | COLLAB-01 | Config absent/false/true parsing and validation | pending |
| 07-01-02 | COLLAB-01 | V2 catalog flattening, schema sanitization, and agent-message fixtures | pending |
| 07-01-03 | COLLAB-01 | Disabled and native byte-fidelity regressions | pending |
| 07-02-01 | COLLAB-01 | JSON/SSE authorized namespace restoration | pending |
| 07-02-02 | COLLAB-01 | Encrypted-task and continuation zero-dispatch guards | pending |
| 07-02-03 | COLLAB-01 | HTTP/WebSocket translated integration | pending |
| 07-03-01 | COLLAB-01 | English and maintained locale documentation | pending |
| 07-03-02 | COLLAB-01 | Review, security, full workspace, diff/wiki guards | pending |

## Final commands

```bash
cargo test inbound_responses:: --lib
cargo test --test inbound_anthropic_translation
cargo test --test inbound_codex_endpoint
cargo test --test inbound_codex_websocket
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features --workspace
git diff --check
test -z "$(git status --short -- wiki/)"
```
