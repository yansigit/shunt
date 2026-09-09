---
phase: phase-15-exact-opencode-go-evidence-gate
plan: 01
subsystem: routing
tags: [opencode-go, admission, config, fail-closed]
requires: []
provides:
  - Additive OpenCode Go provider identity and canonical preset.
  - Empty, crate-private pre-credential admission gate for primary and fallback routes.
  - Crate-local RED/GREEN and router-boundary regression tests.
affects: [phase-15]
tech-stack:
  added: []
  patterns: [explicit-provider-kind identity, empty evidence allowlist, pre-credential filtering]
key-files:
  created: [src/proxy/opencode_go_tests.rs]
  modified: [src/config.rs, src/config/presets.rs, src/routing.rs, src/proxy/capability.rs, src/proxy/failover.rs, src/codex_endpoint.rs, src/proxy.rs]
key-decisions:
  - "OpenCode Go remains unsupported until an exact evidence tuple is admitted; the shipped allowlist is empty."
  - "ProviderKind::OpenCodeGo maps to the existing OpenAI Chat adapter without adding a dormant adapter or session producer."
  - "Admission is keyed only by explicit provider kind and runs before credential/client seams; generic fallbacks remain valid."
---

## Tasks

- Task 1 RED: added `opencode_go_config_acceptance`; pre-implementation run failed behaviorally with `UnknownProviderPreset` (1 selected test, 1 failed).
- Task 2 GREEN: added the enum variant, canonical `opencode-go` preset, identity validation, adapter mapping, and shared empty admission gate. Focused acceptance tests passed (2 selected, 2 passed).
- Task 3: added crate-local router-boundary counter assertions for Go primary rejection and Go fallback removal while preserving a generic primary (1 selected, 1 passed).

## Verification

- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --lib opencode_go_config_acceptance -- --test-threads=1` — passed (1/1).
- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --lib opencode_go_ -- --test-threads=1` — passed (2/2).
- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --lib opencode_go_router_boundaries -- --test-threads=1` — passed (1/1).
- `node /tmp/shunt-phase12-isolated-run.cjs cargo fmt --all` — passed.
- Wrapper reported production OpenCodex config mtime/SHA and backup inventory unchanged after each stateful command.

## Commits

- `58b8b54` — `test(15-01): add failing OpenCode Go config acceptance`
- `b4a3f62` — `feat(15-01): add empty OpenCode Go admission gate`
- `39312fd` — `test(15-01): prove OpenCode Go router admission boundaries`

## Deviations and remaining limits

- The crate-local boundary harness uses injected atomic seam counters and the shared gate; it does not claim live provider, credential, socket, or Computer verification.
- No public docs, credential writeback, session header, EOF recovery, dynamic catalog, or admitted Go tuple was added.

## Self-Check: PASSED

- All plan-created files exist.
- Plan commits are present in git history.
- Focused acceptance and boundary tests pass with nonzero selected counts.
