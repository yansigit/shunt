---
gsd_state_version: "1.0"
milestone: v2
milestone_name: Provider Compatibility
current_phase: 16
current_phase_name: Cross-Provider Release Gate
current_plan: 16-04 pending; 16-01 through 16-03 complete
status: executing
stopped_at: Phase 16 ledger, documentation, and security plans complete; smoke dispositions and final gates pending; live smoke 0/8
last_updated: "2026-09-09T05:54:45Z"
last_activity: 2026-09-09
last_activity_desc: Phase 16 plans 01, 02, and 03 complete
state_head: d42ce164524f716057125504e45dd1fd052ee841
progress:
  total_phases: 8
  completed_phases: 7
  total_plans: 48
  completed_plans: 46
  percent: 96
total_plans_in_phase: 5
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-09-08 after Phase 15)

**Core value:** Protocol-faithful, streaming-safe client behavior in a bounded, lean gateway.
**Current focus:** Phase 16 — Cross-Provider Release Gate

## Current Position

Phase: 16 (Cross-Provider Release Gate) — EXECUTING
Current Plan: 16-04 pending; 16-01 through 16-03 complete
Total Plans in Phase: 5
Status: Executing; wave 3 next
Last activity: 2026-09-09 — Ledger, docs, and security plans committed; final smoke and release gates pending

## Performance Metrics

**Velocity:**

- Total plans completed: 68
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
| 15 | 6 | - | - |
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

- Phase 15 is complete with zero Go admissions; captured/live evidence remains
  mandatory before any future promotion. No live-provider or GUI pass is claimed.
- Phase 16 must address cosmetic docs wording/sidebar warnings, visual verification,
  cross-provider coverage/provenance and isolated live smoke results or explicit skips.
- Subagent candidates: opencode-go/omen-alpha, opencode-go/glm-5.3-flash,
  gpt-5.6-luna; always set explicit high thinking effort. Prior planning/dispatch
  blockers were resolved; do not treat host model availability as Shunt wire proof.

## Deferred Items

| Category | Item | Status | Deferred At | Milestone |
|----------|------|--------|-------------|-----------|
| Persistence | Durable continuation/history | Deferred pending evidence | Initialization | OpenCodex port |
| Repair | General response repair | Deferred pending transcript | Initialization | OpenCodex port |

## Session Continuity

Last session: 2026-09-09T03:43:32Z
Stopped at: Phase 15 complete, ready to plan Phase 16
Resume file: None

## Operator Next Steps

- Latest autonomous preflight: independent Luna/high audit re-verified Phases
  9/11/12/13/14 after Phase 15 shared changes. All Phases 9–15 now have fresh
  passing verification. Root reran isolated full workspace tests at eb60174
  (2,972 passed, zero failed, two existing ignored), fmt and warnings-denied
  Clippy; production fingerprints stayed unchanged. No code changed.
- Phase 16 smart-discuss must resolve the separately opt-in live-smoke scope
  before writing final CONTEXT.md. Recommend one bounded request per distinct
  provider/auth path (maximum eight), existing authorized credentials only,
  no retries, isolated homes/ports and synthetic prompt, <=128 output tokens,
  <=60 seconds each; skip any path whose cost/reasoning bounds or credentials
  cannot be established. Cap planned paid usage at US$1 total. No calls yet.
  Source credentials remain immutable; no production proxy/home is used.

- Phase 15: 6/6 plans, 13/13 truths, 11/11 decisions; user replied "sure pass"
  during verify-work for the consolidated evidence-boundary review. UAT 1/1 passed.
  This is not release approval or a Go/live/visual support claim.
- Existing automated evidence: 2,972 passed, zero failures, two pre-existing ignored;
  fmt, warnings-denied Clippy, bounded regression, 173-page site build, 12 built-page
  checks, isolated unary/streaming CLI smoke passed. Production state unchanged.
- Phase 16 Cross-Provider Release Gate has not been planned or implemented. Start
  its context/planning, retain the two cosmetic UI-review warnings and unperformed
  visual check as release inputs; preserve no-writeback and production isolation.
- Owner-only byte-verified settings/credential backups remain at
  /Users/user/shunt-phase15-backup-YQSxN3. Do not inspect secrets or touch production.
- Transition warning about "node /tmp/shunt-phase12-isolated-run.cjs" in the CLI
  summary is a command misclassified as a repository artifact, not missing code.
  Graduation scan found no LEARNINGS files and skipped under its minimum-data guard.
- Preserve unrelated .planning/config.json, .planning/state.json, .gsd/ and milestone lock.
