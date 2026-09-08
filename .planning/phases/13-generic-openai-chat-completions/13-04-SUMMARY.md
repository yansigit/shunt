---
phase: 13-generic-openai-chat-completions
plan: "04"
subsystem: api
tags: [openai, tools, streaming, bounds, failover, credentials, tdd]
requires:
  - phase: 13-03
    provides: Checked response machine and unary tool bounds
provides:
  - Indexed, request-local tool assembly with deferred identity and one boundary parse
  - Explicit tool count and argument-byte boundary triples with split UTF-8 coverage
  - ConnectOnly protection across both retry and outer failover layers
affects: [13-05]
tech-stack:
  added: []
  patterns: [bounded first-arrival vector, provider-finish argument parsing]
key-files:
  created:
    - src/model/openai_chat_response/assembly.rs
    - tests/openai_chat_translate/assembly.rs
    - .planning/phases/13-generic-openai-chat-completions/evidence/13-04-red-task1.json
    - .planning/phases/13-generic-openai-chat-completions/evidence/13-04-red-task2.json
    - .planning/phases/13-generic-openai-chat-completions/evidence/13-04-red-task3.json
  modified:
    - src/model/openai_chat_response.rs
    - src/adapters/openai_chat/mod.rs
    - src/adapters/openai_chat/diagnostic_tests.rs
    - tests/openai_chat_translate.rs
    - tests/openai_chat_conformance.rs
    - tests/failover.rs
    - tests/retry.rs
requirements-completed: [CHAT-06, CHAT-08]
coverage:
  - id: TOOL-ASSEMBLY
    requirement: CHAT-06
    description: Deferred identity, interleaving, conflict rejection, terminal validation, and request isolation
    verification:
      - kind: unit
        ref: tests/openai_chat_translate/assembly.rs#assembly_interleave_preserves_first_arrival_order
        status: pass
      - kind: integration
        ref: tests/openai_chat_conformance.rs#openai_chat_assembly_interleave_at_limit_multibyte
        status: pass
    human_judgment: false
  - id: TOOL-BOUNDS
    requirement: CHAT-06
    description: 127/128/129 calls and 1-MiB-minus-one/at/plus-one arguments, including a split multibyte fragment
    verification:
      - kind: unit
        ref: tests/openai_chat_translate/assembly.rs#bound_tool_count_plus_one
        status: pass
      - kind: unit
        ref: src/adapters/openai_chat/diagnostic_tests.rs#bound_tool_argument_split_utf8_at_and_plus_one
        status: pass
    human_judgment: false
  - id: AUTH-REPLAY
    requirement: CHAT-08
    description: No post-send status/timeout retry or fallback, credential-isolated redirect refusal, and genuine pre-send fallback control
    verification:
      - kind: integration
        ref: tests/failover.rs#openai_chat_auth_postsend_status_never_advances_fallback
        status: pass
      - kind: integration
        ref: tests/openai_chat_conformance.rs#openai_chat_auth_concurrent_keys_survive_no_redirect
        status: pass
      - kind: integration
        ref: tests/retry.rs#openai_chat_auth_status_is_connect_only_no_retry
        status: pass
    human_judgment: false
completed: 2026-09-08
status: complete
---

# Phase 13 Plan 04 — Indexed tool assembly and replay safety

The shared response machine now retains bounded raw argument fragments by index,
preserves first-arrival call order (including sparse indices), and permits IDs
and function names to arrive later. Conflicting IDs/names, duplicate IDs across
indices, malformed fields, absent terminal identity, incomplete JSON, and
non-object arguments fail closed. Arguments are JSON-parsed only once at the
provider finish boundary. Streamed tool blocks use an empty input start followed
by a complete input_json_delta; unary content receives the parsed tool object.
No cross-request cache, registry, task, dependency, public configuration, or
credential-file behavior was added.

The assembly reuses 13-03's named 128-call and 1-MiB-per-call argument limits,
within the machine's existing 8-MiB aggregate budget. All six limit probes are
explicit. A decoder-to-machine fixture splits `é` between its two UTF-8 bytes
and proves exact-limit acceptance and one-byte-over rejection during assembly.

## RED / GREEN evidence

- Tasks 1–2: twelve new assembly/bound fixtures failed before implementation,
  then all twelve passed. Test commit `3880a3e`; implementation `047120e`.
- Task 3: `openai_chat_auth_postsend_status_never_advances_fallback` reproduced
  an actual 429-to-fallback-200 error before the fix (test commit `e36e060`).
  The fix preserves safe HTTP error statuses, maps refused redirects to 502,
  and sets no outer failover metadata after a provider response. Connect-refused
  errors still carry BeforeHeaders and the positive fallback control passes.
- Wire coverage additionally proves one request after a TTFB timeout despite
  enabled retries; no fallback after a delayed primary; no redirect target
  request; and concurrent configured keys with no inbound-key forwarding.

Procedure deviation: failures were genuinely executed and captured before the
respective implementations, but the root persisted/validated their GSD RED
records after GREEN rather than before it. The records explicitly disclose this
timing and transcribe captured libtest output into TAP (not a claim that Cargo
emitted TAP). All three now return RED_EVIDENCE_OK. An initial adapter-field
mistake produced INVALID_RED/zero_tests_discovered; putting the adapted TAP in
the documented `output` field resolved that validation error. Future GREEN work
must wait for persisted validation, not just captured terminal evidence.

## Verification

- Full all-feature workspace: **2,852 passed, 0 failed, 2 ignored**.
- After the two lint-only map_err-to-inspect_err replacements: all **107** Chat
  translation, **34** Chat conformance, **24** failover, and **8** retry tests pass.
- Format check, strict all-target/all-feature Clippy, and build pass.
- Project CLI smoke: all five checks pass on ports 31981/31982. Chat JSON/SSE
  behavior is independently exercised by the hermetic router fixtures, not a
  live provider claim or Computer evaluation.
- All stateful commands inherited fresh OPENCODEX_HOME via the isolated wrapper;
  production config mtime/SHA and invalid/backup inventory remained unchanged.
- Schema/UI gates have block:false. Stale codebase mapping is advisory only
  (warn, no mapper requested). No gate was disabled.

Documentation surfaces were considered and remain in 13-05's same-PR scope:
README and translations, docs engineering note, and site plus translations.
Generated wiki files remain untouched. Phase 13 and the milestone are not yet
complete; cancellation/conformance breadth and documentation are next.
