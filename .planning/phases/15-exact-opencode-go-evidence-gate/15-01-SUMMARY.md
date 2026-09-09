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
  modified: [src/config.rs, src/config/presets.rs, src/routing.rs, src/proxy/capability.rs, src/proxy/failover.rs, src/codex_endpoint.rs, src/proxy.rs, src/server.rs]
key-decisions:
  - "OpenCode Go remains unsupported until an exact evidence tuple is admitted; the shipped allowlist is empty."
  - "ProviderKind::OpenCodeGo maps to the existing OpenAI Chat adapter without adding a dormant adapter or session producer."
  - "Admission is keyed only by explicit provider kind and runs before credential/client seams; generic fallbacks remain valid."
---

## Tasks

- Task 1 RED: added `opencode_go_config_acceptance`; pre-implementation run failed behaviorally with `UnknownProviderPreset` (1 selected, 1 failed).
- Task 2 GREEN: added the enum variant, canonical `opencode-go` preset, identity validation, adapter mapping, and shared empty admission gate. Focused acceptance tests passed (2 selected, 2 passed).
- Task 3 (corrected): the original Task 3 test was vacuous — atomic seam counters were created, read, and never connected to a resolver, client, router, or HTTP request — and it has been replaced. `src/server.rs` now exposes a `#[cfg(test)] build_router_with_test_dependencies` wrapper taking a `reqwest::Client` and `Arc<dyn CredentialResolver>`, and `src/proxy/opencode_go_tests.rs` drives a real axum router over a loopback fixture through it: a DNS-pinned client routes canonical Go egress into the fixture, injected counting resolver/client seam implementations separate Go from generic traffic, and each boundary test issues real POSTs. Coverage: Go-primary rejection before any Go seam; Go fallback removal on a `[generic_dead, go]` chain after a genuine dead-leg connect failure, with an e2e 200 positive control on the generic control model proving real traffic; count_tokens Go rejection with an honest 501 generic estimate control (OpenAiChat count_tokens is forced to estimation by design, so the previous 200 expectation was wrong); exact native Go inbound rejection (independent `resolve_native_inbound` layer); pinned/unknown native Go rejection before seams; plus canonical config/auth/env negative TOML fixtures and the empty-admission/preserved-generic-fallback gate test. The ordered-preset fixture builds a real `Config::upstreams` declaration order (generic_dead before go) so the model chain is representative.

## Verification

- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --lib opencode_go_config_acceptance -- --test-threads=1` — passed (1/1).
- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --lib opencode_go_ -- --test-threads=1` — 8 passed, 0 failed (5 real boundary tests + 3 config/gate tests).
- Mutation proof (gate removal): temporarily short-circuited `enforce_opencode_go_admission` behind `SHUNT_MUTATION_PROOF` in `src/proxy/capability.rs`, then ran the isolated `opencode_go_router_boundaries` filter — 1 passed, 4 failed. Fallback-drop and Go-primary returned 401, count_tokens returned 501, and pinned-native reached an unreachable branch. These are rejected expectations, not proof of a completed upstream Go HTTP exchange. Restoring the gate restored 8/8 green. Exact-native still passed under mutation because its independent routing rejection precedes the shared gate. No mutation marker remains.
- `node /tmp/shunt-phase12-isolated-run.cjs cargo clippy --all-targets --all-features -- -D warnings` — clean.
- `node /tmp/shunt-phase12-isolated-run.cjs cargo fmt --all --check` — clean.
- `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --workspace` — exit 0, 0 failed (one `antigravity_process::streaming_turn_translates_stub_events_to_sse` load flake in a first run; it passed in isolation in 0.19s and the full suite passed on re-run).
- Wrapper reported production OpenCodex config mtime/SHA and backup inventory unchanged after every stateful command.

## Commits

- `58b8b54` — `test(15-01): add failing OpenCode Go config acceptance`
- `b4a3f62` — `feat(15-01): add empty OpenCode Go admission gate`
- `39312fd` — `test(15-01): prove OpenCode Go router admission boundaries`
- `08c4af6` — `fix(15-01): replace vacuous Go boundary counters with real router evidence`

## Deviations and remaining limits

- Independent GLM/high review found no runtime safety hole. It identified an
  evidence wording defect: the loopback fixture is plain HTTP while canonical
  Go uses HTTPS. The Go HTTP-path counter cannot distinguish a leaked TLS
  attempt and is not independent zero-egress proof. The actual discriminating
  evidence is the injected credential lookup counter plus request outcomes;
  DNS pinning contains any attempted provider network access to loopback.
  Generic positive-control HTTP exchanges are genuine. Test comments now make
  this limit explicit; no test or assertion was weakened.
- The crate-local boundary harness uses injected resolver/client seams and the shared gate; it does not claim live provider, credential, socket, or Computer verification.
- No public docs, credential writeback, session header, EOF recovery, dynamic catalog, or admitted Go tuple was added.
- Corrected after root review: the original Task 3 summary over-claimed. The replaced harness uses a real router, real POSTs, a DNS-pinned loopback client, and injected resolver/client seams (synthetic credentials only); generic count_tokens control is an honest 501 estimate rather than 200, because OpenAiChat count_tokens is estimation-only by design. No dormant session producer, new wire format, or credential writeback was introduced.

## Self-Check: PASSED

- All plan-created files exist.
- Plan commits are present in git history.
- Focused acceptance and boundary tests pass with nonzero selected counts, and the boundary suite fails under gate mutation (evidence of real coverage).
