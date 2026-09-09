---
phase: 14-command-code-product-separation
plan: "02"
subsystem: api
status: complete
completed: 2026-09-08
tags: [subscription, canonical-origin, read-only-auth, ndjson, tracer]
requires:
  - phase: 14-01
    provides: Distinct API-key baseline
provides:
  - Dated source-derived protocol ledger
  - command_code kind and exclusive command_code_oauth pairing
  - Env-only read-only subscription credential and canonical-origin text turn
  - Bounded NDJSON framing and minimal checked unary terminal grammar
affects: [14-03, 14-04, 14-05, 14-06, 16]
tech-stack:
  added: []
  patterns: [private TLS/DNS fixture, immutable credential snapshot, ConnectOnly]
key-files:
  created:
    - src/auth/command_code.rs
    - src/adapters/command_code/mod.rs
    - src/adapters/command_code/ndjson.rs
    - src/adapters/command_code/router_tests.rs
    - tests/command_code_conformance.rs
  modified:
    - src/config.rs
    - src/config/upstreams.rs
    - src/auth/mod.rs
    - src/routing.rs
    - src/proxy/failover.rs
    - src/proxy/capability.rs
    - src/adapters/mod.rs
    - src/init.rs
    - src/adapters/responses/request.rs
    - src/adapters/responses/inbound.rs
    - src/adapters/responses/websocket.rs
requirements-completed: []
coverage:
  - id: SUB-TRACER
    requirement: CCS-02
    description: Full router over canonical-host private TLS; exact path/bearer/host; destination gate and redirect refusal
    verification:
      - kind: integration
        ref: src/adapters/command_code/router_tests.rs
        status: pass
    human_judgment: false
  - id: SUB-ENV
    requirement: CCS-01
    description: Invalid explicit auth returns 401 before connection; injected env-stage absent/invalid matrix, immutable snapshots and temp-file invariance
    verification:
      - kind: unit
        ref: src/auth/command_code.rs
        status: pass
    human_judgment: false
  - id: SUB-NDJSON
    requirement: CCS-05
    description: Minimal text/finish grammar, compatible companion, duplicate/junk/EOF failure, record cap and split UTF-8
    verification:
      - kind: unit
        ref: src/adapters/command_code/ndjson.rs
        status: pass
      - kind: integration
        ref: src/adapters/command_code/router_tests.rs#command_code_tracer_terminal_grammar
        status: pass
    human_judgment: false
---

# Plan 14-02 — Subscription vertical tracer

## Delivered

- Explicit kind/auth pairing, flag-less AuthMap, dispatch/counting/capability wiring.
  The three Responses HTTP/WebSocket credential match sites explicitly do not
  inject the new credential. Its Debug representation remains redacted.
- Env-only `SHUNT_COMMAND_CODE_TOKEN`, header-safe bounded token validation and
  immutable request capture. No file lookup, writeback, refresh, whoami or migration.
- Canonical HTTPS host/root-or-exact-endpoint grammar before lookup and at dispatch;
  no custom port, userinfo, query, fragment, wrong path, lookalike or loopback bypass.
  Production client refuses redirects; private tests inject trust/DNS only.
- Unary text tracer with constant empty workspace config, required identity headers,
  NDJSON upstream stream:true, bounded strict framing, first terminal authority,
  one compatible finish-step→finish companion, and no EOF success synthesis.
- Seven new lib tests (including matrix subcases) and one public config test;
  the API-key product's eight tests remain green.

## TDD and executed gates

- 9bad613: protocol ledger before fixtures. 8875bd0: genuine RED assertion after
  successful compilation (unknown command_code_oauth). `14-02-RED.json` is an
  explicitly labelled Rust-to-TAP transcription accepted by RED_EVIDENCE_OK.
  Final module moved from the temporary test-only parent declaration into the
  actual command_code adapter; the named tracer/function and file are preserved.
- GREEN compiled and the real canonical-host TLS text turn passed. Subsequent
  existing-safety characterization tests did not manufacture additional failures.
- e83993d: committed production wiring, bounded tracer and env matrix together.
- Final formatter, build, warnings-denied Clippy, and warnings-denied workspace
  suite passed: **2,891 passed, 0 failed, 2 existing ignored**, 30 result groups.
  The full suite was repeated after tightening wire assertions to exact header
  values (no token case folding or host-prefix matching). Both first-wave plans
  are covered by this combined build/test gate.
- `/tmp/shunt-subscription-smoke-agIzmL/smoke.cjs`: ordinary binary config check
  and curl returned 401 authentication_error for an invalid explicit empty token.
  A local HTTP CONNECT trap observed **zero proxy/upstream connections**; no
  actual provider generation or credential-file fallback occurred. Owned child
  and trap were closed. API-key CLI/curl smoke is recorded in 14-01-SUMMARY.
- Every stateful command used the isolation wrapper and reported production
  config mtime/SHA and invalid/backup inventory unchanged. No live/GUI pass claimed.

## Scope discoveries and pending work

- Cargo required four additional exhaustive match updates in init.rs and the
  Responses transports. Plan ownership was extended; existing credentials did
  not gain any new injection path. No wildcard concealed missing cases.
- Root caught an earlier provenance mistake using exact source and git blame:
  all three #2647 reporter-only rows are Luna, Gemini 3.7 Flash and vision-exp.
  The corrected ledger/plans exclude all three from subscription admission
  pending evidence. Ten rows remain for 14-03. This does not restrict subagent
  model selection through other providers; user Luna fallback preference remains.
- This is deliberately the narrow tracer: client streaming/complex histories
  currently fail explicitly, not silently buffer/drop. Full request compilation,
  exact model/effort admission and credential-scoped conversation affinity are
  14-03; shared streaming/unary semantic machine and remaining budgets are 14-04;
  CLI fallback and complete lifetime coverage are 14-05.
- Env-stage unit tests inject source results, so absent-input tests never read
  the user's CLI path, even after fallback is added. 14-05 must use private
  temporary path injection for all actual file-resolution tests.
- Shared CCS requirements remain pending their later plans/final gate. All public
  README/docs/site locale updates remain assigned to 14-06 in the same phase/PR;
  wiki considered and untouched. Phase 16 live acceptance remains unverified.
- GSD wave hooks: schema drift and UI safety report block:false (no UI surface).
  Codebase drift is a non-blocking stale-global-map warning, spawn_mapper:false;
  actual implementation sources were inspected directly. No gate was disabled.
