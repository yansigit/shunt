---
gsd_state_version: "1.0"
milestone: v2
milestone_name: Provider Compatibility
current_phase: 15
current_phase_name: Exact OpenCode Go Evidence Gate
current_plan: Not started
status: planning
stopped_at: Phase 15 research and pattern mapping complete; independent planning next
last_updated: "2026-09-08T20:18:14.128Z"
last_activity: 2026-09-08
last_activity_desc: Phase 14 complete, transitioned to Phase 15
state_head: 91c382d3d90bdf883c5e9146f91814137ce578f8
progress:
  total_phases: 8
  completed_phases: 6
  total_plans: 37
  completed_plans: 37
  percent: 75
total_plans_in_phase: 0
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-09-08 after Phase 14)

**Core value:** Protocol-faithful, streaming-safe client behavior in a bounded, lean gateway.
**Current focus:** Phase 15 — Exact OpenCode Go Evidence Gate

## Current Position

Phase: 15 — Exact OpenCode Go Evidence Gate
Current Plan: Not started
Total Plans in Phase: 0
Status: Research and pattern mapping complete; planning gates pending
Last activity: 2026-09-08 — Phase 15 approval, backups, context and GLM/high research recorded

## Performance Metrics

**Velocity:**

- Total plans completed: 62
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
| 12 | 8 | - | - |
| 13 | 5 | - | - |
| 14 | 6 | - | - |
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

Phase 15 dispatch recovered on the user's model retry request. Earlier GLM/high
research completed successfully. Omen without a reasoning override failed with
provider 400/1210: thinking cannot be disabled; use low, high or max. Luna/high
then completed pattern mapping and wrote 15-PATTERNS.md. Independent planning,
checking and implementation remain pending; no model dispatch is Shunt wire proof.
Source-only evidence admits zero tuples; strict EOF policy is already locked.

The user authorized Cursor CLI probing and existing OpenCodex suites. Adjacent OpenCodex supplies schema-derived Run output-token deltas and context checkpoints; 83 hermetic tests passed in isolated state. After CLI installation, an isolated read-only Composer 2.5 probe succeeded with streamed events and result usage, including cache fields. CLI output is not raw Connect evidence: do not infer protobuf field mappings or claim Shunt live verification. Original CLI settings/credential files were backed up and remained unchanged. All eight Cursor plans and phase verification are complete. The approved bounded request-local KV architecture is implemented and tested; live gateway availability remains the Phase 16 opt-in gate.

## Deferred Items

| Category | Item | Status | Deferred At | Milestone |
|----------|------|--------|-------------|-----------|
| Persistence | Durable continuation/history | Deferred pending evidence | Initialization | OpenCodex port |
| Repair | General response repair | Deferred pending transcript | Initialization | OpenCodex port |

## Session Continuity

Last session: 2026-09-08T20:18:14Z
Stopped at: Phase 15 research and pattern mapping complete; independent planning next
Resume file: .planning/phases/15-exact-opencode-go-evidence-gate/15-CONTEXT.md

## Operator Next Steps

- The user-approved installed GSD gate validator repair is complete and backed up at `/Users/user/gsd-gate-backup-mEVeg3`. Dotted query names validate; gate checks were rerun without disabling them. See 13-01-SUMMARY.md.
- Phase 13 is complete: 5/5 plans, 2,875 passed, 0 failed, 2 existing ignored; CLI/curl Chat smoke and all documentation locales pass. Validation has no gaps; all 19 authored security threats have mitigations. Computer visual checks remain skipped/blocked, never passed.
- Phase 14 is complete: 6/6 plans, 2,942 passed, zero failed, two pre-existing ignored; bounded regression, formatter, Clippy, docs build and owned CLI/curl smoke passed. All 21 authored security threats are mitigated; live/Computer acceptance is not claimed. No production mutation.
- Phase 15 public opt-in configuration is approved (2026-09-08), with backups first and no changes to existing provider settings or credential writeback. Fresh byte-verified owner-only backups: `/Users/user/shunt-phase15-backup-YQSxN3`. Continue Phase 15 research and planning from 15-CONTEXT.md.
- Transition warning about `node /tmp/shunt-phase12-isolated-run.cjs` in 14-03-SUMMARY is a prose command misclassified as a repository file, not a missing implementation artifact. Graduation scan found no LEARNINGS files and skipped under its minimum-data guard.
