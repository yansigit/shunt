---
gsd_state_version: "1.0"
milestone: v2
milestone_name: Provider Compatibility
current_phase: 12
current_phase_name: Cursor Evidence-Backed Hardening
current_plan: 3
status: executing
stopped_at: Phase 12 plan 03 request-local KV architecture approved; implementation resumed
last_updated: "2026-09-07T22:45:12.678Z"
last_activity: 2026-09-07
last_activity_desc: User approved bounded memory-only per-request KV/blob support; plan 03 resumed
state_head: db2397e48b04ee1e5d0d9732fd8e2d4b08459f7d
progress:
  total_phases: 8
  completed_phases: 3
  total_plans: 26
  completed_plans: 20
  percent: 38
total_plans_in_phase: 8
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-09-07 after Phase 11)

**Core value:** Protocol-faithful, streaming-safe client behavior in a bounded, lean gateway.
**Current focus:** Phase 12 — Cursor Evidence-Backed Hardening

## Current Position

Phase: 12 (Cursor Evidence-Backed Hardening) — EXECUTING
Current Plan: 3
Total Plans in Phase: 8
Status: 2 of 8 plans summarized; plan 03 partial
Last activity: 2026-09-07 — Plans 01–02 verified; plan 03 preserves authentic output tool IDs. User approved bounded request-local KV/blob protocol support; implementation resumed.

## Performance Metrics

**Velocity:**

- Total plans completed: 43
- Average duration: —
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01 | 4 | - | - |
| 02 | 5 | - | - |
| 03 | 3 | - | - |
| 04 | 2 | - | - |
| 05 | 3 | - | - |
| 06 | 2 | - | - |
| 07 | 3 | - | - |
| 08 | 3 | - | - |
| 9 | 4 | - | - |
| 10 | 7 | - | - |
| 11 | 7 | - | - |
**Per-Plan Metrics:**

| Plan | Duration | Tasks | Files |
|------|----------|-------|-------|
| Phase 01 P01 | 37min | 3 tasks | 7 files |
| Phase 01 P02 | 37min | 2 tasks | 2 files |
| Phase 01 P03 | 22min | 2 tasks | 2 files |
| Phase 01 P04 | 22min | 3 tasks | 15 files |
| Phase 02 P01 | 25 | 2 tasks | 4 files |
| Phase 02 P03 | 12 | 2 tasks | 2 files |
| Phase 02 P04 | 6 | 2 tasks | 8 files |
| Phase 2 P5 | 13 min | 2 tasks | 12 files |
| Phase 09 P01 | 12min | 2 tasks | 6 files |

## Accumulated Context

### Decisions

- [2026-09-07 user approval]: Phase 12 may add a bounded, memory-only, per-request KV/blob handler to preserve structured Cursor history. No disk persistence, cross-request cache, public configuration, or credential-file writeback change is authorized.
- [2026-09-07 user approval]: Phases 13–14 may add public configuration choices for generic OpenAI Chat and separate read-only Command Code subscription authentication. Existing provider settings and credential-file writeback must remain unchanged. User requires settings/credential backups first; existing project settings and the available provider credential file were copied outside the repository with owner-only permissions and verified byte-for-byte without displaying contents.
- Port observable OpenCodex behavior and fixtures, not its platform architecture.
- Phase 1 changes only the opt-in inbound Responses transport; HTTP stays stable.
- Phase 2 provider-semantics work requires explicit user approval.
- [Phase 02]: WebSocket live turns refresh the runtime snapshot at turn start so each turn keeps one immutable native route while later turns observe reloads.
- [Phase 02]: All maintained locales state exact compatible native selection, pinned fallback, pre-dispatch rejection, and no post-output hop consistently.
- [Phase 02]: Missing or malformed models retain pinned compatibility routing; only unique exact compatible declarations select a native provider.
- [Phase 03]: Hard quota requires bounded exact structured evidence; status-only and ambiguous failures stay transient.
- [Phase 03]: Retry-After parsing is centralized, strictly bounded, and supports decimal seconds plus HTTP dates.
- [Phase 03]: Quota inspection is capped by bytes and total time while preserving the downstream response stream.
- [Phase 04]: Native compaction is HTTP-only, byte-faithful, and shares the existing inbound Responses account/auth path.
- [Phase 04]: Compact capability is limited to ChatGPT/Codex and the canonical OpenAI API; arbitrary compatible gateways fail closed.
- [Phase 05]: Only unique exact Anthropic mappings enter translation; native Responses routes remain byte-faithful and fallback inference stays pinned.
- [Phase 05]: Stateful or lossy Responses features fail before dispatch; HTTP and WebSocket share one bounded Anthropic response state machine.
- [Phase 05]: Anthropic transport remains authoritative for credentials, account selection, retries, admission, timeouts, and safe headers.
- [Phase 06]: Capability filtering never removes the primary and runs before fallback credentials or network access.
- [Phase 06]: Eligibility uses one internal provider matrix; no public configuration or provider semantics were added.
- [Phase 06]: The `[1m]` context marker removes every fallback because no later target can guarantee the requested context window.
- [Phase 07]: Collaboration translation is default-off and request-authorized; native Responses traffic remains opaque regardless of the flag.
- [Phase 07]: Exact Anthropic routes bridge declared V2 collaboration tools and plaintext tasks, but ciphertext-only or provider continuation state fails before dispatch.
- [Phase 07]: shunt performs no collaboration decryption, persistence, cache, or hidden billable recovery call.
- [Phase 08]: The first signal stops admission and starts one validated process deadline, defaulting to 30 seconds with an inclusive 1..=3600 range.
- [Phase 08]: Deadline expiry drops the Axum server future; normal Tokio runtime teardown cancels remaining HTTP, SSE, WebSocket, and background work and releases RAII-owned resources.
- [Phase 08]: The existing second-signal immediate-exit escape hatch remains available, while timeout changes require restart because the coordinator captures the value at boot.
- [Phase 09]: Commitment remains crate-private and additive to existing transport, status, safety, and retry-budget evidence.
- [Phase 09]: Every successful first Codex WebSocket event remains conservative commitment; structural tool events are replay-unsafe.
- [Phase 09]: Complete bounded framing plus an authoritative provider terminal is required for clean completion; malformed, duplicate, or cut streams fail closed.
- [Phase 09]: Semantic commitment gates only production-reachable WebSocket fallback and continuation recovery; generic HTTP retry remains structurally pre-response.
- [Phase 09]: Provider credentials are redacted and rebound per destination without changing credential-file writeback.
- [Phase 10]: Parallel Gemini tool results are selected by authentic unique ID and emitted in original assistant call order.
- [Phase 10]: Known unsupported Part semantics and every completed post-DONE frame fail closed in streaming and unary paths.
- [Phase 10]: Google OAuth Code Assist credentials resolve once per request; only proven pre-connect failures retry, while ambiguous post-send failures remain single-attempt.

### Pending Todos

Phase 11 transition: native catalog admission is exact and account-bound; catalog redirects are refused; a pre-output 401 can refresh and replay once on the same account; cancellation releases upstream and capacity through ownership. Synthetic conformance is not a live availability claim.

None yet.

### Blockers/Concerns

The user authorized Cursor CLI probing and existing OpenCodex suites. Adjacent OpenCodex supplies schema-derived Run output-token deltas and context checkpoints; 83 hermetic tests passed in isolated state. After CLI installation, an isolated read-only Composer 2.5 probe succeeded with streamed events and result usage, including cache fields. CLI output is not raw Connect evidence: do not infer protobuf field mappings or claim Shunt live verification. Original CLI settings/credential files were backed up and remained unchanged. Plans 01–02 are verified. Plan 03 output tool identity is fixed, and the user approved the bounded request-local KV architecture recorded in 12-03-CHECKPOINT.md; implementation is in progress.

## Deferred Items

| Category | Item | Status | Deferred At | Milestone |
|----------|------|--------|-------------|-----------|
| Persistence | Durable continuation/history | Deferred pending evidence | Initialization | OpenCodex port |
| Repair | General response repair | Deferred pending transcript | Initialization | OpenCodex port |

## Session Continuity

Last session: 2026-09-07T19:45:00Z
Stopped at: Phase 12 plan 03 resumed after architecture approval
Resume file: None

## Operator Next Steps

- Finish the approved request-local KV/blob implementation and plan 03 verification, then execute plans 04–08 sequentially. Plans 01–02 are verified; no phase-wide completion is claimed.
- Continue the approved milestone sequentially through Phase 16, verifying each phase. Preserve credential-file behavior and request approval for new public configuration choices.
