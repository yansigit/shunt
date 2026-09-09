---
phase: "15"
slug: "exact-opencode-go-evidence-gate"
status: validated
nyquist_compliant: true
wave_0_complete: true
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
| 15-01 Task 1 | OGO-02/04 | behavioral RED via `Config::load_from_str` `is_ok()` assertion (compiles today; fails on `UnknownProviderPreset`), with enum/kind assertions deferred to GREEN | passed |
| 15-01 Task 2 | OGO-02/04 | additive `ProviderKind::OpenCodeGo` + canonical API-key preset/negative validation fixtures; Go fallback candidates removed by configured kind while generic primary remains; primary gate before credentials | passed |
| 15-01 Task 3 | OGO-03/04 | `opencode_go_router_boundaries`: crate-local server injection, zero Go lookup/HTTP counters, separate legitimate generic lookup counts, exact native rejection preserved and pinned/unknown native Go guarded | passed |
| 15-02 Task 1 | OGO-01/02 | four complete candidate records in the fenced ledger frame, rejected register, admitted `[]`, eight edge dispositions incl. flagged OGO-02 | passed |
| 15-03 Task 1 | OGO-01/02/03 | English provider/README/config/nav (`site/src/lib/i18n.ts`) + engineering note `docs/opencode-go-evidence-gate.md` | passed |
| 15-05 Task 1 | OGO-01/02/03 + REL-05 | nine locale files (ko/ja/zh-cn README + provider + configuration) with native anchors, no invented English anchors | passed |
| 15-06 Task 1 | OGO-03/REL-05 | four actual `guides/providers.mdx` overview files (en/ko/ja/zh-cn) updated with zero-admission wording and locale links | passed |
| 15-04 Task 1 | OGO-02/04 | real-rejection CLI smoke: canonical config, fake key, actual POST /v1/messages, Anthropic-shape gateway error, temp home cleaned; zero-egress proof separately owned by 15-01 | passed |
| 15-04 Task 2 | OGO-02/03/04 | fmt, warm-cache `RUSTFLAGS=-Dwarnings` Clippy/workspace, site build, fingerprint audit, 15-05 rows present | passed |

## Exact focused commands

- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --lib opencode_go_config_acceptance -- --test-threads=1`
- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --lib opencode_go_router_boundaries -- --test-threads=1`
- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test opencode_go_evidence ledger -- --test-threads=1`
- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test opencode_go_docs -- --test-threads=1`
- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test check_cli opencode_go_cli_negative -- --test-threads=1`
- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --lib config::presets::tests::preset_table_contains_the_supported_backends_in_documented_order -- --test-threads=1`

Each command uses ONE Cargo `--` separator: the test-name filter sits before
it, libtest options (`--test-threads=1`) after it. Zero selection is failure;
the first tracer run records a behavioral RED, never a compile failure.

Plans 15-03, 15-05 and 15-06 each combine documentation and per-file assertions
in Task 1. Run the entire docs binary and inspect actual named results/counts:
at least 5, 14 and 18 respectively. Record behavioral RED before content edits,
a missing-token mutation failure, and restored GREEN. Cargo's exit code alone
does not detect zero selection. Navigation assertions cover links/labels only;
other contract assertions are scoped to new Go sections, not unrelated providers.

## Release gates

Run as separate wrapper invocations (no chained conjunctions inside one call;
never HTML-encoded):

- `node /tmp/shunt-phase12-isolated-run.cjs cargo fmt --all --check`
- `node /tmp/shunt-phase12-isolated-run.cjs env RUSTFLAGS=-Dwarnings cargo clippy --all-targets --all-features -- -D warnings` (warm cache, `RUSTFLAGS=-Dwarnings`)
- `node /tmp/shunt-phase12-isolated-run.cjs env RUSTFLAGS=-Dwarnings cargo test --all-features --workspace`
- `node /tmp/shunt-phase12-isolated-run.cjs npm --prefix site run build`

Rejected evidence remains classified as failed evidence, unknown fields,
wrong wire, family inference, unsupported effort, wrong terminal, or
unverified live. OGO-02 unclassified remains a flagged manual assumption. No
live success claim is permitted in the empty-admission implementation.

## Executed release evidence (2026-09-08)

- Dedicated CLI smoke: 1/1 passed, real process boot plus unary and streaming
  POST rejection (Anthropic 400); fake key, canonical URL, fresh temporary home,
  no proxy-sentinel connection, process stopped and home removed.
- Focused config 1/1; router boundaries 5/5; ledger 2/2; docs 19/19;
  dedicated CLI 1/1; ordered-preset regression 1/1. Focused reruns used
  `env RUSTFLAGS=-Dwarnings` after the wrapper to retain the warmed test cache.
- Format check and warnings-denied Clippy: exit 0.
- Warnings-denied full workspace: 2,972 passed, zero failed, two pre-existing
  ignored; 33 result summaries including zero-example Rust doctests. Zero
  doctests is not substituted for any required focused test selection.
- Site build: 173 pages, exit 0. Twelve built provider/guide/configuration pages
  checked for exact key, canonical URL, kind and matching locale provider link.
  The initial link probe assumed trailing slashes; actual navigation normalizes
  them away. Normalizing only terminal slashes made the probe match valid links.
  Existing Vite deprecation/Pagefind stemming notices do not fail the build.
- Documentation RED and required-token mutation/restored GREEN are recorded in
  plans 03/05/06 summaries. Plan 03 honestly records its missing separate RED
  commit; no history or raw evidence was fabricated.
- Every isolated command reported production config mtime/SHA and invalid/backup
  inventory unchanged. Settings/credential backups remain outside the repository.

The crate-local Go HTTP-path counter alone cannot observe failed HTTPS attempts;
see 15-RUNTIME-REVIEW.md. Pre-credential counters and error outcomes discriminate
admission bypasses; DNS pinning contains attempted egress. The dedicated CLI
smoke additionally observes no connection to its loopback proxy sentinel.

## Validation Audit 2026-09-08

Infrastructure: Rust libtest via Cargo; no added test framework. All nine task
rows across six plans have executable verification and observed nonzero passing
selection. Requirements map:

| Requirement | Direct evidence | Current scope |
|---|---|---|
| OGO-01 | `opencode_go_ledger`, mutation validator | Four source-only records with explicit unknowns, canonical target and pinned provenance |
| OGO-02 | Config/admission/router tests, ledger admitted-empty assertion, docs tests | No tuple supported without evidence; empty set is valid |
| OGO-03 | Pre-credential router rejection, docs conditional session assertions | No current Go dispatch/session producer; future admission contract documented |
| OGO-04 | Router rejection/fallback filtering and real CLI negative | No permissive Go wire/EOF/retry bypass can be selected |
| REL-05 (phase contribution) | 19 per-file/scope tests, site build and 12 built-page checks | English plus ko/ja/zh-cn README/provider/config/guides and navigation; milestone-wide audit remains Phase 16 |

| Metric | Count |
|---|---|
| Current-scope missing or failing coverage | 0 |
| New tests required by this audit | 0 |
| Escalated implementation gaps | 0 |

### Manual-only evidence boundary

OGO-02's unclassified edge remains explicitly flagged: the assumption that a
future exact tuple will satisfy safe captured/live evidence is unresolved.
It does not authorize promotion and is not marked as executed. The current
empty-admission contract has automated coverage; future promotion requires a
new evidence review. No live/captured support or Computer acceptance is claimed.

**Approval:** Nyquist-compliant for the approved empty-admission scope.
Whole-phase verification remains separate. No tests were removed or weakened.
