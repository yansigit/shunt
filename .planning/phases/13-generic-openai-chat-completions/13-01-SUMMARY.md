---
phase: 13-generic-openai-chat-completions
plan: "01"
subsystem: api
tags: [openai, chat-completions, sse, gateway, adapter, anthropic-messages, tdd]

requires:
  - phase: 12-and-prior-core-gateway
    provides: "adapter registry, config ProviderKind tables, retry/RetrySafety, SSE keepalive, upstream timeouts, failover dispatch"
provides:
  - "ProviderKind::OpenAiChat (kind string openai_chat) accepted with AuthMode::ApiKey + api_key_env, validated in config"
  - "OpenAiChatAdapter wired through routing, capability, failover dispatch with RetrySafety::ConnectOnly and redirect(Policy::none()) client"
  - "Bounded incremental SSE decoder (data-line framing, [DONE], per-event byte cap, fail-closed residual/UTF-8/duplicate-terminal)"
  - "Shared unary/streaming semantic machine: text deltas, finish_reason mapping, usage, trailing include_usage chunk, terminal gating, final_json_checked/transport_close_checked"
  - "Real-router loopback conformance harness with synthetic env key, mock Chat upstream, and eight tracer fixtures"
affects: [13-02-request-expansion, 13-03-response-expansion, docs]

actuals:
  tokens: 18135
  tasks: 3
  commits: 5

tech-stack:
  added: []
  patterns:
    - "Per-provider SSE semantic machine shared by unary and streaming output modes"
    - "stream::unfold-owned relay tuple (byte stream + decoder + machine) with deferred authoritative terminal"
    - "ConnectOnly send policy for non-idempotent generation POSTs; BeforeHeaders reserved for pre-send connect failures"

key-files:
  created:
    - src/adapters/openai_chat/mod.rs
    - src/adapters/openai_chat/sse.rs
    - src/model/openai_chat_request.rs
    - src/model/openai_chat_response.rs
    - tests/openai_chat_conformance.rs
  modified:
    - src/config.rs
    - src/routing.rs
    - src/adapters/mod.rs
    - src/proxy/failover.rs
    - src/proxy/capability.rs
    - src/model/mod.rs

key-decisions:
  - "ProviderKind::OpenAiChat carries an explicit serde rename to the kind string openai_chat (snake_case would have produced open_ai_chat)."
  - "count_tokens is forced to CountTokens::Estimate for OpenAiChat until a counting strategy exists; capability arm left empty with comments rather than guessed semantics."
  - "The framing [DONE] validates the remainder of the already-received batch before success, without waiting for HTTP EOF. Missing [DONE] fails closed."
  - "With stream_options.include_usage the official usage-only chunk (empty choices) after the finish_reason chunk is accepted; any other payload after finish_reason fails closed."

patterns-established:
  - "SSE fixture helper emits real data: lines (data: {json}\n\n) matching the official text/event-stream grammar."
  - "Tracer fixtures pin request-local credential/model resolution behaviorally via distinct bearer/model assertions, not internals."

requirements-completed: [CHAT-01]

coverage:
  - id: D1
    description: "Unary text turn: Anthropic Messages client to openai_chat provider over the full router, upstream JSON completion translated to the Anthropic response contract"
    requirement: CHAT-01
    verification:
      - kind: integration
        ref: "tests/openai_chat_conformance.rs#openai_chat_tracer_unary_happy_path"
        status: pass
      - kind: integration
        ref: "tests/openai_chat_conformance.rs#openai_chat_tracer_unary_isolation"
        status: pass
      - kind: integration
        ref: "tests/openai_chat_conformance.rs#openai_chat_tracer_unary_non_api_key_auth_rejected"
        status: pass
    human_judgment: false
  - id: D2
    description: "Streaming relay: incremental SSE with exactly one authoritative terminal; EOF without finish_reason or [DONE] emits exactly one Anthropic-shaped error and never a synthesized success; duplicate [DONE] fails closed; official trailing usage chunk accepted"
    requirement: CHAT-06
    verification:
      - kind: integration
        ref: "tests/openai_chat_conformance.rs#openai_chat_tracer_streaming_relay"
        status: pass
      - kind: integration
        ref: "tests/openai_chat_conformance.rs#openai_chat_tracer_streaming_eof_failclosed"
        status: pass
      - kind: integration
        ref: "tests/openai_chat_conformance.rs#openai_chat_tracer_streaming_duplicate_terminal"
        status: pass
    human_judgment: false
  - id: D3
    description: "Tracer happy path proves one upstream request; ConnectOnly and redirect refusal are wired, with adversarial replay/redirect coverage deferred to 13-04."
    requirement: CHAT-05
    verification:
      - kind: integration
        ref: "tests/openai_chat_conformance.rs#openai_chat_tracer_unary_happy_path (requests.len() == 1 assertion)"
        status: pass
    human_judgment: false

duration: not-measured
completed: 2026-09-07
status: complete
---

# Phase 13 Plan 01: OpenAI Chat Completions Tracer Summary

**openai_chat provider kind wired end to end through config, routing, and failover dispatch, with a real-router loopback proving one unary and one streaming text turn translated between Anthropic Messages and Chat Completions**

## Performance

- **Duration:** Not measured. One executor implemented the tracer; root review added regressions and corrections.
- **Started:** 2026-09-07
- **Completed:** 2026-09-07
- **Tasks:** 3 of 3 (Task 0 protocol gate, Task 1 unary tracer, Task 2 streaming tracer)
- **Files modified:** 11

## Accomplishments

- Task 0 gate consumed: 13-PROTOCOL-EVIDENCE.md separates official Chat facts (developers.openai.com provenance), gateway hardening, and non-official provider reasoning extensions, with the explicit official finding that reasoning_content/reasoning are NOT OpenAI Chat fields. No new network fetch was required.
- Task 1 (unary tracer): red-first loopback fixtures proved fail, then the openai_chat kind was wired through ProviderKind/serde, config validation (ApiKey + api_key_env required; passthrough auth rejected), routing, failover dispatch, capability arm, request translation whitelist skeleton, the shared semantic machine in unary mode, and a bounded unary collect path. 3/3 unary fixtures green.
- Task 2 (streaming tracer): the same machine extended with streaming terminal gating (EOF without terminal fails closed; duplicate [DONE] fails closed; residual-after-[DONE] can never emit success-then-error), a stream::unfold relay owning byte stream + decoder + machine, keepalive pings, ConnectOnly send policy, and acceptance of the official include_usage trailing usage-only chunk. 3/3 streaming fixtures green.
- Root reran the full all-features workspace suite successfully after the protocol fixes, then build, format, and strict Clippy. Clippy found unchecked fixture reads; bounded header reads fixed that warning. Strict Clippy and all nine focused Chat tests passed on the final code. GSD post-wave orchestration is blocked separately (see below).

## Test Counts and Evidence

- Final openai_chat_conformance: 8 passed (3 unary + 5 streaming), run serially under OPENAI_CHAT_ENV_LOCK; the private idle-read regression also passed.
- RED evidence recorded via the TAP adapter conversion (stable libtest has no --format json; precedent from phase 10-01), records at /tmp/openai-chat-13-01-red-unary-record.json and /tmp/openai-chat-13-01-red-streaming-record.json; RED_EVIDENCE_OK accepted both gates.
- No live provider or gateway was exercised; all upstreams are wiremock loopback fixtures with synthetic env keys and loopback port 0.

## Task Commits

1. **Task 1 RED: failing unary fixtures** - 0ac86ce (test)
2. **Task 1 GREEN: wire openai_chat kind through unary tracer** - bf4d74b (feat)
3. **Task 2 RED: failing streaming fixtures** - b4d55c2 (test)
4. **Task 2 GREEN: trailing usage chunk acceptance in the stream machine** - 8fe9c08 (feat)
5. **rustfmt/clippy across the tracer** - b6939e9 (style)

_TDD tasks each carry a test→feat pair; the style commit is mechanical only._

## Decisions Made

- count_tokens forced to Estimate for OpenAiChat (no counting strategy yet; documented in-code).
- The framing [DONE] only closes a provider-declared success; terminal output waits for validation of the already-received batch, not HTTP EOF.
- Tool_calls, reasoning, and other unknown choice fields are outside this slice's whitelist and fail/ignore rather than guess; expansion belongs to 13-02/13-03.

## Deviations from Plan

### Auto-fixed Issues

**1. Streaming fixture frames lacked the SSE data: prefix**
- **Found during:** Task 2 GREEN verification (openai_chat_tracer_streaming_relay)
- **Issue:** The fixture emitted bare JSON frames; the (correct) decoder requires real SSE data: lines, so no frame ever parsed and the relay failed closed at EOF. The RED gate had been satisfied by this fixture bug rather than by the machine gap, and the eof_failclosed/duplicate_terminal machine fixtures already passed at RED time because the machine owned terminal gating from Task 1.
- **Fix:** sse_body now emits data: {frame}\n\n per frame. The machine gap this plan targets (trailing include_usage usage-only chunk after finish_reason) is genuinely exercised and was fixed by the GREEN implementation.
- **Files modified:** tests/openai_chat_conformance.rs (in 8fe9c08)
- **Verification:** all 3 streaming fixtures pass after the fix
- **Committed in:** 8fe9c08 (disclosed in its commit message)

**2. Dead-code cleanups in Task 1 GREEN**
- **Found during:** Task 1 GREEN compile
- **Issue:** CheckedPart::Metadata and the upstream_model field were unused under the whitelist design.
- **Fix:** removed both; constructors take only the model.
- **Files modified:** src/model/openai_chat_response.rs
- **Verification:** compile clean, clippy -D warnings clean
- **Committed in:** bf4d74b

**3. clippy err_expect in the config rejection fixture**
- **Found during:** full-gate clippy run
- **Issue:** .err().expect() flagged by clippy::err_expect
- **Fix:** expect_err (semantically identical assertion)
- **Files modified:** tests/openai_chat_conformance.rs
- **Verification:** clippy --all-targets --all-features -D warnings clean; 6/6 fixtures still pass
- **Committed in:** b6939e9

### Deferred to Later Plans (not deviations of scope)

- README/docs/site documentation for the new provider kind was NOT updated in this slice and is deliberately deferred to the later 13-xx plans that complete the kind's user-facing story; this slice is the minimal internal tracer. Flagging explicitly so docs ship before the phase closes.
- Request expansion (tools, system shapes beyond the skeleton) and response expansion (tool_calls, reasoning extensions) belong to 13-02/13-03 per plan.

**Total deviations:** 3 auto-fixed (1 fixture correction, 2 mechanical)
**Impact on plan:** The fixture correction was required for the RED/GREEN loop to test the intended behavior; no scope creep.

## Issues Encountered

- Debugging the streaming relay initially looked like a machine defect; instrumentation traced it to the missing data: fixture prefix (see Deviations). Tracing code was removed before commit.
- Production-safety wrapper (fresh OPENCODEX_HOME, non-10100 port, live-config mtime/SHA + backup-inventory verification) ran on every cargo invocation; it reported the live config unchanged after every run, and the user's backup directories were never touched.

## User Setup Required

None - no external service configuration required (loopback fixtures only).

## Next Phase Readiness

- The tracer slice is committed on codex/opencodex-provider-compatibility with base 396fbb1 and 5 commits (0ac86ce, bf4d74b, b4d55c2, 8fe9c08, b6939e9).
- 13-02 (request expansion) and 13-03 (response expansion) build directly on translate_request and the semantic machine; the architecture accommodates them without rework.
- The broader CHAT requirements beyond this slice (and docs surfaces) remain open; this summary does not mark the phase complete.

---
*Phase: 13-generic-openai-chat-completions*
*Completed: 2026-09-07*

## Self-Check: PASSED

## Root Review Corrections and Final Verification

- Official `finish_reason: null` and `usage: null` fixtures reproduced a valid-stream rejection (RED `6c11df0`); nullable-field handling fixed it (`794d114`).
- Two real-router terminal regressions failed in `dc54f49`: waiting for HTTP EOF after `[DONE]`, and accepting a finish reason without `[DONE]`. `3225319` validates the already-received batch before emitting success and rejects a missing sentinel.
- An idle-body socket fixture failed with the original HTTP client. `4592778` adds a private 120-second read-idle limit (no new public config), with a 30-millisecond injected limit proving timeout behavior. Fixture headers are read completely within a bounded buffer; the terminal fixture deliberately keeps the HTTP connection open.
- Successful commands, all through `/tmp/shunt-phase12-isolated-run.cjs`: `cargo test --all-features --workspace`, `cargo build`, `cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-features openai_chat -- --test-threads=1`. The final focused run follows fixture-only Clippy corrections; all eight integration tests plus the idle-read test passed.
- Each wrapper invocation confirmed production config mtime/SHA and backup inventory unchanged. These are loopback tests, not live-provider or Computer evaluation.
- Broader CHAT-05/CHAT-06 completion is deliberately not claimed; their adversarial and expanded semantics remain in later plans.

## Post-Wave Workflow Blocker

Resolved with explicit user approval on 2026-09-08: backed up `loop-hook-dispatch.md` to `/Users/user/gsd-gate-backup-mEVeg3/loop-hook-dispatch.md`, then allowed bounded dotted namespace segments for gate queries only, retaining shell-metacharacter rejection and requiring a full-string match. Fifteen valid/invalid cases passed. Schema and UI checks returned `block: false`; codebase drift returned a non-blocking stale-map advisory (`directive: warn`, `spawn_mapper: false`). No gate was disabled. This is a local installed-plugin correction, outside the repository diff; a plugin update may replace it.

The installed GSD `loop render-hooks execute:wave:post --raw` emits `verify.schema-drift`, `verify.codebase-drift`, and `ui.safety-gate`. Its `/Users/user/.codex/plugins/cache/gsd-codex-community/gsd-codex-plugin/1.13.0+gsd.ae40529d31d7.wrapper.2/gsd-core/references/loop-hook-dispatch.md` permits only `^[a-z][a-z0-9-]*( [a-z][a-z0-9-]*)*$`, which rejects all three dotted query names. The first two malformed entries specify skip-on-error; the UI entry specifies halt-on-error. No malformed command was executed and no gate was marked passed. The bundled check router explicitly supports these dotted names, so this is a plugin contract mismatch, not a Shunt code failure. Resolve the plugin validator contract before advancing to 13-02; changing the installed plugin is outside the repository implementation scope.
