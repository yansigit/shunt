---
phase: "14"
slug: command-code-product-separation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-08"
---

# Phase 14 — Validation Strategy

## Test Infrastructure

Rust libtest, Tokio, Axum and existing mock/TLS socket helpers; Cargo.toml. No new test framework is needed. Every stateful command inherits isolation via `node /tmp/shunt-phase12-isolated-run.cjs` from `/Users/user/.codex/worktrees/0466/shunt`.

Quick baseline: `node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test openai_chat_conformance -- --test-threads=1`.
Full suite: `node /tmp/shunt-phase12-isolated-run.cjs env 'RUSTFLAGS=-D warnings' cargo test --all-features --workspace`.
Phase-specific targets/filters must reject zero selected tests. Plans 14-01 and 14-02 are executed; later groups remain pending. Their summaries contain RED/GREEN evidence and exact commands. The combined wave-1 build, formatter, warnings-denied Clippy and workspace suite passed (2,891 passed, 0 failed, 2 existing ignored; 30 result groups). API-key CLI/curl smoke passed; subscription invalid-explicit-token CLI smoke returned 401 with zero upstream/proxy connections. These are isolated checks, not live-provider or GUI acceptance.

## Sampling Rate

- After each task: its focused automated check; actual RED recorded before GREEN for TDD tasks.
- After each plan wave: focused phase suite plus directly affected regressions.
- Before verification: format, warning-denied Clippy and full workspace suite, docs site build, owned CLI/curl smoke.
- Target focused feedback latency: 60 seconds, measured during execution; do not infer it from this draft.

## Per-Task Verification Map

Planner-provided concrete map. Waves: W1 = 14-01 ∥ 14-02; W2 = 14-03; W3 = 14-04; W4 = 14-05; W5 = 14-06. All commands run from the worktree root through `node /tmp/shunt-phase12-isolated-run.cjs`; every filter is guarded against zero selection; one cargo at a time; quoted 'RUSTFLAGS=-D warnings' on the full suite.

| Group | Plans/Tasks | Requirements | Secure behavior | Automated check (filter) | Type | Status |
| --- | --- | --- | --- | --- | --- | --- |
| API-key preset + siblings | 14-01 Task 1 | CCK-01 | commandcode preset over openai_chat, sibling rows untouched, distinct env name | cargo test --test command_code_api_conformance command_code_api_preset | unit/config | pass — 14-01-SUMMARY |
| API-key tracer + isolation | 14-01 Task 2 | CCK-01/02, D-12/13 | real-router unary, concurrent no-crossover, auth error no-retry; RED recorded before GREEN | cargo test --test command_code_api_conformance command_code_api_tracer | real gateway | pass — 14-01-SUMMARY |
| API-key scenarios | 14-01 Task 3 | CCK-03, D-11 | unary/stream/tools/long-context/drop through actual gateway | cargo test --test command_code_api_conformance command_code_api_scenarios | real gateway | pass — 14-01-SUMMARY |
| Protocol evidence gate | 14-02 Task 0 | D-12 | dated ledger, three sections, pinned revision 055c3ecf, MIT attribution, finish grammar | test -s + rg section/revision checks | artifact gate | pass — 14-02-SUMMARY; three reporter-only rows excluded by corrected provenance |
| Subscription tracer + destination | 14-02 Task 1 | CCS-02, D-01..04, D-08 | /alpha/generate canonical, bearer env-first, destination-before-credential + at dispatch, single terminal, EOF fail-closed; RED recorded | cargo test --lib command_code_tracer | real gateway | pass — narrow text/unary slice, 14-02-SUMMARY |
| Env credential matrix | 14-02 Task 2 | CCS-01, D-03 | valid/invalid/absent/empty env; fail-closed; snapshots immutable; file unchanged | cargo test --lib command_code_tracer_env_matrix | resolver + fixture files | pass — injected env stage only; CLI fallback remains 14-05 |
| Effort table + subscription preset | 14-03 Task 1 | CCS-03, D-05 | exact pinned arrays, ultra rejected, no clamp/alias, sibling preservation | cargo test --test command_code_translate command_code_translate_effort; ... command_code_translate_preset_subscription | unit/config | pass — 14-03-SUMMARY; full suite 2,903/0/2 |
| Envelope + headers + session | 14-03 Task 2 | CCS-03, D-05/06 | exact envelope/headers, no x-project-slug, credential-scoped session, positive continuation, no-ID non-stable | cargo test --test command_code_translate command_code_translate_session; ... _envelope; ... _headers | unit | pass — 14-03-SUMMARY; full suite 2,903/0/2 |
| Tool history | 14-03 Task 3 | CCS-04, D-07 | adjacent units, missing error-text, orphan carrier, image carrier, ordering, duplicate/unsupported reject, positive subagent | cargo test --test command_code_translate command_code_translate_tools | unit | pass — 14-03-SUMMARY; full suite 2,903/0/2 |
| Response machine | 14-04 Task 1 | CCS-05/06, D-08 | pinned terminal grammar, usage precision (integers only), typed unknown, empty/null fail | cargo test --test command_code_translate command_code_translate_machine | unit | pass |
| Streaming/unary relay | 14-04 Task 2 | CCS-05, D-08 | incremental relay, unary twin, one terminal, mid-body error, EOF fail | cargo test --lib command_code_tracer_response | real gateway | pass |
| Byte boundaries | 14-04 Task 3 | CCS-06/08, D-09 | per-record/residual/semantic/tool/argument caps below/at/above; UTF-8 split; deadline; no repair | cargo test --test command_code_translate command_code_bounds; --lib command_code_bounds | byte-split + gateway | pass |
| CLI fallback + dispatch gate | 14-05 Task 1 | CCS-01/02, D-03/04 | read-only auth.json matrix, env precedence, at-dispatch destination re-check | cargo test --lib command_code_lifetime_file; ... command_code_lifetime_destination_dispatch | fixture files + gateway | pending |
| Replay discipline | 14-05 Task 2 | CCS-07, D-10 | ConnectOnly, pre-connect retry retains credential+session, post-send single attempt, no redirect | cargo test --lib command_code_lifetime_replay | real sockets | pending |
| Cancellation ownership | 14-05 Task 3 | CCS-07, D-11 | pre-header + mid-body cancel both modes close upstream, capacity reused, no detached task | cargo test --lib command_code_lifetime_cancel | real sockets | pending |
| Full matrix | 14-06 Task 1 | CCK-03, CCS-08, D-12 | positive subagent/continuation, tool-heavy, long-context bounds, auth-error matrix, error finish | cargo test --lib command_code_matrix | real gateway | pending |
| Docs parity | 14-06 Task 2 | D-14 | README en+ko+ja+zh-cn, milestone note, site provider page + locales, configuration reference + locales; wiki untouched | npm --prefix site run build + rg env-name/locale checks | docs/site | pending |
| Final gates + smoke | 14-06 Task 3 | D-13/14 | fmt; clippy -D warnings; full workspace suite; site build; owned CLI/curl smoke; live-home mtime/SHA invariant | chained node /tmp/shunt-phase12-isolated-run.cjs gates | CI-equivalent | pending |

## Wave 0 Requirements

- [x] Dated protocol-evidence ledger preceded tracer RED fixtures; exact reporter-only rows corrected before model admission implementation.
- [x] Synthetic minimal text/terminal NDJSON corpus and real gateway tracer test target; full semantic corpus remains 14-04.
- [x] Canonical-host TLS test seam without public off-origin bearer bypass.
- [ ] Credential fixtures use temporary files; byte/mtime/inventory checks verify read-only behavior.
- [x] Every not-yet-created target referenced by plans has an owned creation task. (14-01 Task 1 creates tests/command_code_api_conformance.rs; 14-02 Task 1 creates tests/command_code_conformance.rs, src/adapters/command_code/*, 14-03 Task 1 creates tests/command_code_translate.rs; 14-06 Task 2 creates docs/site surfaces.)

## Manual-Only Verifications

Live provider acceptance (including minimal envelope and version) is not established by hermetic tests; Phase 16 must separately label opt-in live results or explicit skips. Computer evaluation previously blocked; approved CLI fallback is not a passed visual audit. No GUI is changed by this phase.

## Validation Sign-Off

- [ ] All tasks have automated verification or owned Wave 0 dependency.
- [ ] No three consecutive tasks lack automated coverage.
- [ ] All missing target paths created; no watch mode; zero-selection guard enforced.
- [ ] All CCK/CCS matrix items actually executed with evidence.
- [ ] Feedback latency measured; full gates and owned smoke pass.
- [ ] nyquist_compliant set true only after actual validation.

**Approval:** Pending execution.
