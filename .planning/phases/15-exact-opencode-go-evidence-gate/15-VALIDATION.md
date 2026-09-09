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

## Machine-readable ledger frame

The 15-EVIDENCE.md ledger lives in one fenced JSON code block tagged
`opencode-go-ledger` with the top-level shape
`{candidates: [...], rejected: [...], admitted: []}` (admitted exactly empty).
`tests/opencode_go_evidence.rs` parses that fenced block itself — validation
names the parseable artifact, not unspecified Markdown prose.

## Per-task verification map

| Plan/task | Requirement | Evidence | Status |
|---|---|---|---|
| 15-01 Task 1 | OGO-02/04 | behavioral RED via `Config::load_from_str` `is_ok()` assertion (compiles today; fails on `UnknownProviderPreset`), with enum/kind assertions deferred to GREEN | pending |
| 15-01 Task 2 | OGO-02/04 | additive `ProviderKind::OpenCodeGo` + canonical API-key preset/negative validation fixtures; Go fallback candidates removed by configured kind while generic primary remains; primary gate before credentials | pending |
| 15-01 Task 3 | OGO-03/04 | `opencode_go_router_boundaries`: crate-local server injection, zero Go lookup/HTTP counters, separate legitimate generic lookup counts, exact native rejection preserved and pinned/unknown native Go guarded | pending |
| 15-02 Task 1 | OGO-01/02 | four complete candidate records in the fenced ledger frame, rejected register, admitted `[]`, eight edge dispositions incl. flagged OGO-02 | pending |
| 15-03 Task 1 | OGO-01/02/03 | English provider/README/config/nav (`site/src/lib/i18n.ts`) + engineering note `docs/opencode-go-evidence-gate.md` | pending |
| 15-03 Task 2 | OGO-01/03 | per-file doc assertions in `tests/opencode_go_docs.rs` (one fn per English surface) | pending |
| 15-05 Task 1 | OGO-01/02/03 + REL-05 | nine locale files (ko/ja/zh-cn README + provider + configuration) with native anchors, no invented English anchors | pending |
| 15-05 Task 2 | REL-05 | per-file locale assertions extended in `tests/opencode_go_docs.rs` (one fn per locale surface) | pending |
| 15-06 Task 1 | OGO-03/REL-05 | four actual `guides/providers.mdx` overview files (en/ko/ja/zh-cn) updated with zero-admission wording and locale links | pending |
| 15-06 Task 2 | REL-05 | one independent guide assertion per overview file in `tests/opencode_go_docs.rs` | pending |
| 15-04 Task 1 | OGO-02/04 | real-rejection CLI smoke: boot with fake key, actual POST /v1/messages, Anthropic-shape gateway error, fixture request count 0, temp home cleaned | pending |
| 15-04 Task 2 | OGO-02/03/04 | fmt, warm-cache `RUSTFLAGS=-Dwarnings` Clippy/workspace, site build, fingerprint audit, 15-05 rows present | pending |

## Exact focused commands

- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --lib opencode_go_config_acceptance -- --test-threads=1`
- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --lib opencode_go_router_boundaries -- --test-threads=1`
- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test opencode_go_evidence ledger -- --test-threads=1`
- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test opencode_go_docs opencode_go_docs -- --test-threads=1`
- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test opencode_go_docs opencode_go_docs_locale -- --test-threads=1`
- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test check_cli opencode_go_cli_negative -- --test-threads=1`
- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --lib config::presets::tests::preset_table_contains_the_supported_backends_in_documented_order -- --test-threads=1`
- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test opencode_go_docs opencode_go_docs_guides -- --test-threads=1`

Each command uses ONE Cargo `--` separator: the test-name filter sits before
it, libtest options (`--test-threads=1`) after it. Zero selection is failure;
the first tracer run records a behavioral RED, never a compile failure.

## Release gates

Run as separate wrapper invocations (no chained conjunctions inside one call;
never HTML-encoded):

- `node /tmp/shunt-phase12-isolated-run.cjs cargo fmt --all --check`
- `node /tmp/shunt-phase12-isolated-run.cjs cargo clippy --all-targets --all-features -- -D warnings` (warm cache, `RUSTFLAGS=-Dwarnings`)
- `node /tmp/shunt-phase12-isolated-run.cjs env RUSTFLAGS=-Dwarnings cargo test --all-features --workspace`
- `node /tmp/shunt-phase12-isolated-run.cjs npm --prefix site run build`

Rejected evidence remains classified as failed evidence, unknown fields,
wrong wire, family inference, unsupported effort, wrong terminal, or
unverified live. OGO-02 unclassified remains a flagged manual assumption. No
live success claim is permitted in the empty-admission implementation.

**Approval:** pending executed checks.
