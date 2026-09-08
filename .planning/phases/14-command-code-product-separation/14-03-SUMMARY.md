---
phase: 14-command-code-product-separation
plan: "03"
subsystem: api
status: complete
completed: 2026-09-08
requirements-completed: [CCS-03, CCS-04]
key-files:
  created:
    - src/adapters/command_code/efforts.rs
    - src/adapters/command_code/request.rs
    - src/adapters/command_code/history.rs
    - tests/command_code_translate.rs
  modified:
    - src/adapters/command_code/mod.rs
    - src/config/presets.rs
    - src/proxy/capability.rs
    - src/auth/slots/tests.rs
---

# Plan 14-03 — Subscription request contract

## Delivered

- Appended the distinct `command-code` subscription preset; API-key `commandcode`
  and earlier preset rows remain unchanged.
- Exact ten-row model/effort table from pinned OpenCodex 055c3ecf. Rejected all
  three reporter-only rows, unknown/case-mismatched IDs and unsupported efforts;
  no aliases/clamps/refresh. Omission is supported, while explicit `none` is not
  silently stripped. Explicit request effort takes precedence over route default.
  Both fallback filtering and primary dispatch consult the same validation.
- Exact workspace-free envelope, joined text system blocks, optional numeric
  temperature, default max_tokens=64000 and upstream stream:true. Pure compiler
  accepts either client stream mode; adapter still explicitly rejects streaming
  pending 14-04 (no hidden buffering).
- Fixed header allowlist and sensitive Authorization. Explicit conversation
  identity is SHA-256 derived with credential scoping, domain separation and
  length delimiters, UUID-shaped; without identity use request-local UUIDv4,
  stable only within that request's retries. Reject ambiguous/invalid identity;
  never send raw conversation, prompt, workspace or project-slug metadata.
- Product-local history compiler preserves text/reasoning and authentic tool
  identities, adjacent results, explicit execution-unknown missing-result carriers,
  visible orphan results and following image carriers. Images never break pending
  call/result adjacency. Duplicate/ambiguous identities and opaque representations
  fail closed. Tool catalog and named/required/none choices compile explicitly.
- All new Rust files remain under 500 lines. Generic Chat compiler untouched.

## Executed evidence

- Actual assertion RED runs for preset, fallback effort admission, envelope, and
  tool history are recorded in the four `14-03-*-RED.json` files; every record
  passed `check tdd-red-evidence` before corresponding behavior edits. TAP content
  is explicitly labelled manual transcription of Rust libtest, never raw TAP.
- New header/session helper tests are characterization under Task 2's envelope
  RED driver, not claimed as separately observed pre-implementation failures.
- 11 translation tests pass; the exact fallback admission lib test passes; all
  seven existing subscription tracer/framing/auth tests pass.
- Formatter and warnings-denied Clippy pass. Initial Clippy caught a fixture
  initializer style issue; fixed without suppression. Initial workspace run
  caught the outbound header inventory tripwire. After inspecting the fixed
  allowlist, added its precise classification; scanner and credential assertions
  were not weakened.
- Repeated warnings-denied all-features workspace suite: **2,903 passed, 0 failed,
  2 existing ignored**, 31 result groups. Every stateful command used
  `node /tmp/shunt-phase12-isolated-run.cjs`, and all completed checks reported
  production config mtime/SHA and backup inventory unchanged.
- Wave 2 build then repeated full suite (`--quiet` for compact output) also passed:
  2,903/0/2, 31 result groups. Schema and UI gates report block:false. Stale global
  codebase mapping remains a non-blocking warn directive with spawn_mapper:false;
  relevant implementation was inspected directly. No gate disabled.

## Process and remaining gates

- GLM/high executor produced no progress or files and was interrupted. Omen/high
  review failed with a provider thinking-mode error despite the high override.
  Dispatch controls then became unavailable, preventing the requested Luna/high
  fallback; root executed inline. No independent review completion is claimed.
- Main used GSD RED gates, source provenance checks and isolated verification.
  Coverage matrix check passes (31 surfaces, 15 integrate, 16 explicit opt-outs).
- Root preserved OpenCodex's full MIT notice outside planning in
  THIRD-PARTY-NOTICES.md, verified byte-for-byte against pinned source LICENSE.
  14-06 still owns its public docs link and all README/docs/site translations;
  those surfaces were considered and remain required in the same PR. Wiki untouched.
- Response machine/streaming/budgets remain 14-04; read-only CLI fallback and
  socket lifetime proofs remain 14-05; full matrix/docs/final smokes remain 14-06.
  No live service or GUI acceptance is claimed; Phase 16 remains the live gate.
