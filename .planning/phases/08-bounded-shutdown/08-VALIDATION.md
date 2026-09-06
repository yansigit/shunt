---
phase: 08-bounded-shutdown
status: draft
wave_0_complete: false
created: 2026-09-06
---

# Phase 8 Validation

| Task | Requirement | Test seam | Status |
|------|-------------|-----------|--------|
| 08-01-01 | OPS-01 | Config absent/default/valid and invalid zero/over-cap parsing | pending |
| 08-01-02 | OPS-01 | Deadline begins only after first signal notification | pending |
| 08-01-03 | OPS-01 | Clean server completion propagates unchanged | pending |
| 08-02-01 | OPS-01 | Timeout drops pending server future and owned lease sentinel | pending |
| 08-02-02 | OPS-01 | Existing SIGTERM and second-signal behavior remains intact | pending |
| 08-02-03 | OPS-01 | HTTP/SSE/WebSocket transport regression suites | pending |
| 08-03-01 | OPS-01 | English and maintained locale documentation | pending |
| 08-03-02 | OPS-01 | Review, security, full workspace, diff/wiki guards | pending |

## Final commands

```bash
cargo test shutdown --lib
cargo test --test inbound_codex_endpoint
cargo test --test inbound_codex_websocket
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features --workspace
git diff --check
test -z "$(git status --short -- wiki/)"
```
