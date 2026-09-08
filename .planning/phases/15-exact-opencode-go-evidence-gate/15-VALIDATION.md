---
phase: "15"
slug: "exact-opencode-go-evidence-gate"
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-08"
---

# Phase 15 — Validation Strategy

All stateful commands use `node /tmp/shunt-phase12-isolated-run.cjs`, a fresh
`OPENCODEX_HOME`, and a non-10100 fixture port. Production config mtime/SHA and
backup inventory are checked before and after. No credentials are inspected.

## Per-task verification map

| Plan/task | Requirement | Evidence | Status |
|---|---|---|---|
| 15-01 Task 1 | OGO-02/04 | crate-local behavioral RED then GREEN; primary/fallback/count-tokens/Codex zero lookup and zero HTTP counters | pending |
| 15-01 Task 2 | OGO-03/04 | private seam proves no bearer/session header and generic positive controls | pending |
| 15-02 Task 1 | OGO-01/02 | four complete candidate records, rejected ledger, seven resolved edges plus flagged OGO-02 | pending |
| 15-03 Task 1 | OGO-01/02/03 | English provider/README/config contract | pending |
| 15-03 Task 2 | REL-05 | ko/ja/zh-cn README/provider/config parity and links | pending |
| 15-04 Task 1 | OGO-02/04 | synthetic isolated CLI status + zero outbound + cleanup | pending |
| 15-04 Task 2 | OGO-02/03/04 | fmt, warm-cache RUSTFLAGS=-Dwarnings Clippy/workspace, site build, fingerprint audit | pending |

## Exact focused commands

- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --lib opencode_go_empty_admission -- --test-threads=1`
- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --lib opencode_go_router_boundaries -- --test-threads=1`
- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test opencode_go_evidence -- opencode_go_ledger -- --test-threads=1`
- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test check_cli -- opencode_go_cli_negative -- --test-threads=1`

Each command must select at least one test; zero selection is failure. The
first tracer run records behavioral RED, not compile failure.

## Release gates

Run separately or with raw shell `&&` (never HTML-encoded):

`node /tmp/shunt-phase12-isolated-run.cjs cargo fmt --all --check`

`node /tmp/shunt-phase12-isolated-run.cjs cargo clippy --all-targets --all-features -- -D warnings`

`node /tmp/shunt-phase12-isolated-run.cjs env RUSTFLAGS=-Dwarnings cargo test --all-features --workspace`

`node /tmp/shunt-phase12-isolated-run.cjs npm --prefix site run build`

Rejected evidence must remain classified as failed evidence, unknown,
wrong-wire, family-inferred, unsupported effort, or wrong terminal. OGO-02
unclassified remains a flagged manual assumption. No live success claim is
permitted in the empty-admission implementation.

**Approval:** pending executed checks.
