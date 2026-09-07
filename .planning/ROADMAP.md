# Roadmap: Shunt OpenCodex Behavior Port

## Milestones

- ✅ **v1 OpenCodex Port** — Phases 1–8, 25 plans; shipped 2026-09-06. [Archive](milestones/v1-ROADMAP.md)
- 🚧 **v2 Provider Compatibility** — Phases 9–16; in progress.

## Overview

Provider Compatibility first makes Shunt's existing provider guarantees measurable, then hardens the shared Gemini and provider-specific Antigravity and Cursor paths. It adds generic OpenAI Chat before layering the two distinct Command Code products on the appropriate transports, treats OpenCode Go as an exact-model evidence gate, and ends with a cross-provider release audit. Every phase preserves streaming, bounded-resource, credential-destination, and pre-commit retry invariants; Google AI Studio Web and credential-file writeback remain excluded.

## Phases

<details>
<summary>✅ v1 OpenCodex Port (Phases 1–8) — SHIPPED 2026-09-06</summary>

- [x] **Phase 1: Inbound Responses WebSocket** - Completed 4/4 plans
- [x] **Phase 2: Native Responses Routing** - Completed 5/5 plans
- [x] **Phase 3: Quota-Aware Resilience** - Completed 3/3 plans
- [x] **Phase 4: Native Compaction** - Completed 2/2 plans
- [x] **Phase 5: Anthropic Translation** - Completed 3/3 plans
- [x] **Phase 6: Capability-Aware Fallback** - Completed 2/2 plans
- [x] **Phase 7: Collaboration Preservation** - Completed 3/3 plans
- [x] **Phase 8: Bounded Shutdown** - Completed 3/3 plans

Full phase goals, requirements, success criteria, and plan references are kept in the [v1 roadmap archive](milestones/v1-ROADMAP.md).

</details>

### 🚧 v2 Provider Compatibility (In Progress)

- [x] **Phase 9: Provider Conformance Foundation** - Make preservation, safety, credential, terminal, cancellation, and retry commitments executable across provider fixtures. (completed 2026-09-06)
- [ ] **Phase 10: Gemini Semantic Hardening** - Give streaming and non-streaming Google Code Assist turns one strict, replay-safe semantic contract.
- [ ] **Phase 11: Antigravity Protocol and Credential Hardening** - Enforce Antigravity's exact Cloud Code Assist envelope, identity, destination, model, signature, tool, and SSE rules.
- [ ] **Phase 12: Cursor Evidence-Backed Hardening** - Correct Cursor profiles, continuation, tools, terminal handling, cancellation, and pre-output failover from verified evidence.
- [ ] **Phase 13: Generic OpenAI Chat Completions** - Add a bounded OpenAI Chat transport with complete Anthropic message, tool, image, streaming, and error translation.
- [ ] **Phase 14: Command Code Product Separation** - Support the API-key Chat product and proprietary subscription product as distinct authenticated transports.
- [ ] **Phase 15: Exact OpenCode Go Evidence Gate** - Expose only exact model/wire tuples that pass dated captured and safe live evidence, with zero tuples as a valid result.
- [ ] **Phase 16: Cross-Provider Release Gate** - Prove provider coverage, provenance, credential safety, documentation parity, and repository quality before merge.

## Phase Details

### Phase 9: Provider Conformance Foundation

**Goal**: Operators can trust that existing providers and every later compatibility slice share explicit, testable safety and preservation guarantees.
**Depends on**: Phase 8
**Requirements**: PRES-01, PRES-02, PRES-03, PRES-04, PRES-05, SAFE-01, SAFE-02, SAFE-03, SAFE-04, SAFE-05, SAFE-06, SAFE-07
**Success Criteria** (what must be TRUE):

  1. Existing ChatGPT/Codex and Vercel paths pass regression fixtures for their current HTTP, SSE, WebSocket, authentication, compaction, collaboration, error-shape, cancellation, and terminal behavior.
  2. Every exercised stream has explicit size/time/state bounds, emits at most one authoritative terminal outcome, and releases upstream work and held resources after cancellation.
  3. A retry/failover test demonstrates that redispatch is possible only before client-visible output or replay-unsafe tool activity, while paired tools and authentic continuation metadata survive a safe replay.
  4. Credential tests prove provider-kind and destination binding, inbound credential stripping, diagnostic redaction, response-lifetime ownership, and no secret-bearing fixtures.

**Plans**: 4 plans

- [x] 09-01-PLAN.md
- [x] 09-02-PLAN.md
- [x] 09-03-PLAN.md
- [x] 09-04-PLAN.md

### Phase 10: Gemini Semantic Hardening

**Goal**: Gemini users receive equivalent, strict Google Code Assist semantics in streaming and non-streaming modes without unsafe replay.
**Depends on**: Phase 9
**Requirements**: GEM-01, GEM-02, GEM-03, GEM-04, GEM-05
**Success Criteria** (what must be TRUE):

  1. A Gemini turn keeps the selected OAuth identity and project in its Code Assist envelope until the response finishes or is cancelled.
  2. The same captured transcript produces equivalent ordered text, reasoning, tools, usage, finish, and provider-error meaning for streaming and non-streaming clients.
  3. Malformed, oversized, invalid-UTF-8, or prematurely terminated events fail as protocol errors without silent loss or synthesized success.
  4. Generation retries occur only for proven replay-safe pre-commit failures and never after output or replay-unsafe tool activity.

**Plans**: 5/7 plans executed; 2 gap-closure plans ready

Plans:
**Wave 1**

- [x] 10-01-PLAN.md — Checked Gemini semantic state and authentic tool/signature round trip

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 10-02-PLAN.md — Bounded byte SSE and unary transport integration

**Wave 3** *(blocked on Wave 2 completion)*

- [x] 10-03-PLAN.md — Immutable identity, non-idempotent retry, parity, and cancellation

**Wave 4** *(blocked on Wave 3 completion)*

- [x] 10-04-PLAN.md — English engineering, configuration, and troubleshooting contract

**Wave 5** *(blocked on Wave 4 completion)*

- [x] 10-05-PLAN.md — Maintained locale parity, deterministic scope checks, and final release gates

**Wave 6** *(blocked on Wave 5 completion)*

- [ ] 10-06-PLAN.md — ID-addressed parallel results, strict Part semantics, and post-DONE framing

**Wave 7** *(blocked on Wave 6 completion)*

- [ ] 10-07-PLAN.md — Hermetic Code Assist OAuth lifetime evidence and final release gates

### Phase 11: Antigravity Protocol and Credential Hardening

**Goal**: Antigravity users can run faithful Cloud Code Assist turns with canonical-destination credential safety and account/session-consistent tools and reasoning.
**Depends on**: Phase 10
**Requirements**: ANT-01, ANT-02, ANT-03, ANT-04, ANT-05, ANT-06, ANT-07, ANT-08
**Success Criteria** (what must be TRUE):

  1. Antigravity sends OAuth credentials only to approved daily or production HTTPS origins, rejects credential-bearing off-origin redirects, and keeps account, token, project, and retry identity paired through response completion.
  2. Supported exact model/effort combinations produce the proven agent envelope and stable opaque conversation session; unsupported combinations fail before dispatch.
  3. Streaming and non-streaming clients receive equivalent text, reasoning, sequential tool history, authentic matching thought signatures, usage, terminal state, and embedded provider errors from the always-SSE upstream.
  4. Malformed wrappers, incomplete tools, or unsafe signature state fail closed, while a 401 permits at most one account-bound pre-commit refresh and replay without changing credential-file writeback behavior.

**Plans**: TBD

### Phase 12: Cursor Evidence-Backed Hardening

**Goal**: Cursor users receive request-local, evidence-backed model and tool behavior with stable continuation and strict Connect stream safety.
**Depends on**: Phase 9
**Requirements**: CUR-01, CUR-02, CUR-03, CUR-04, CUR-05, CUR-06, CUR-07, CUR-08
**Success Criteria** (what must be TRUE):

  1. Cursor keeps Shunt's proven AgentService Run destination and selects exact request-local model, client, endpoint, and capability facts without cross-request mutation.
  2. Ordinary, compacted, recovered, and multi-round turns retain stable continuation identity and correctly paired executable or freeform tool history, with unproven schemas rejected explicitly.
  3. Valid Connect transcripts emit ordered reasoning, text, tools, usage, cancellation, and exactly one terminal outcome; malformed frames, decompression violations, provider errors, and premature EOF fail closed.
  4. Local construction failures do not advance provider failover, and transport retry/failover occurs only before commitment without adding speculative repetition or no-progress cancellation.

**Plans**: TBD

### Phase 13: Generic OpenAI Chat Completions

**Goal**: Operators can route Anthropic Messages clients to generic OpenAI Chat Completions upstreams with faithful bounded JSON and SSE translation.
**Depends on**: Phase 9
**Requirements**: CHAT-01, CHAT-02, CHAT-03, CHAT-04, CHAT-05, CHAT-06, CHAT-07, CHAT-08, CHAT-09
**Success Criteria** (what must be TRUE):

  1. An operator can configure a Chat upstream independently, with one unambiguous `/chat/completions` path, provider-bound API-key authentication, and no Responses-only request fields.
  2. System, user, assistant, text, image, generation-control, declared-tool, tool-choice, parallel-call, and paired-result inputs translate with stable call identities.
  3. JSON and SSE responses produce equivalent bounded text, reasoning, tools, usage, finish, and recognized error semantics while interleaved indexed tool deltas remain ordered and complete.
  4. Hermetic normal, streaming, image, tool-heavy, long-context, cancellation, malformed, truncation, authentication, and provider-error scenarios require one trustworthy terminal and never leak credentials.

**Plans**: TBD

### Phase 14: Command Code Product Separation

**Goal**: Command Code API-key and subscription users can use their distinct products without transport, authentication, session, or terminal semantics bleeding between them.
**Depends on**: Phase 13
**Requirements**: CCK-01, CCK-02, CCK-03, CCS-01, CCS-02, CCS-03, CCS-04, CCS-05, CCS-06, CCS-07, CCS-08
**Success Criteria** (what must be TRUE):

  1. The `commandcode` API-key product uses the generic Chat contract at `/provider/v1/chat/completions`, cannot select subscription credentials, and passes the applicable Chat conformance matrix.
  2. The `command-code` subscription product reads approved environment or existing CLI credentials without writing them and sends bearers only to the canonical HTTPS `/alpha/generate` destination.
  3. Subscription turns preserve exact model/effort facts, identity headers, a stable opaque session, adjacent tool-call/result units, explicit missing/orphan results, and supported result images.
  4. Streaming and non-streaming NDJSON scenarios cover normal, reasoning, tool-heavy, subagent/continuation, long-context, cancellation, authentication, retry, malformed, truncation, provider-error, and terminal behavior with bounded fail-closed parsing.

**Plans**: TBD

### Phase 15: Exact OpenCode Go Evidence Gate

**Goal**: Operators see only OpenCode Go model/wire combinations whose exact behavior is proven, with unsupported combinations rejected before credentials or network activity.
**Depends on**: Phase 13
**Requirements**: OGO-01, OGO-02, OGO-03, OGO-04
**Success Criteria** (what must be TRUE):

  1. Every candidate has a dated evidence record covering exact model, canonical destination, wire, headers, context, modalities, effort, tools, field filtering, terminal behavior, and sanitized capture or live source.
  2. A tuple is supported and documented only after hermetic and credential-safe captured or live verification; an evidence result of zero supported tuples is accepted without weakening the gate.
  3. Each accepted tuple reuses its matching Chat, Responses, or Anthropic contract and sends a stable opaque `x-opencode-session` only to the canonical destination.
  4. Unknown, family-inferred, ambiguous, or failed tuples return an unsupported error before credential lookup or dispatch.

**Plans**: TBD

### Phase 16: Cross-Provider Release Gate

**Goal**: Users and maintainers can verify that every support claim is tested, sourced, documented in every maintained language, and safe to release.
**Depends on**: Phases 11, 12, 14, 15
**Requirements**: REL-01, REL-02, REL-03, REL-04, REL-05, REL-06
**Success Criteria** (what must be TRUE):

  1. Every supported provider/auth/model/wire tuple has sanitized hermetic coverage for applicable normal, streaming, tool, terminal, malformed, truncation, authentication, cancellation, and retry boundaries with recorded provenance.
  2. OpenCodex-derived material retains MIT notice/provenance, while captures contain dates and sanitization status but no secrets, account/project identifiers, or private user content.
  3. Opt-in live smokes are bounded, redacted, isolated, and prove source credential files remain byte-for-byte unchanged; unavailable credentials yield a documented skip.
  4. README, engineering docs, and English/ko/ja/zh-cn README and site surfaces agree with verified behavior; `wiki/` remains untouched, and negative checks find no Google AI Studio Web support or dependency.
  5. Formatting, Clippy with warnings denied, full all-features workspace tests, documentation/site validation, security review, and code review all pass before merge.

**Plans**: TBD

## Progress

**Execution Order:** 9 → 10 → 11 → 12 → 13 → 14 → 15 → 16

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1–8. OpenCodex Port | v1 | 25/25 | Complete | 2026-09-06 |
| 9. Provider Conformance Foundation | v2 | 4/4 | Complete    | 2026-09-06 |
| 10. Gemini Semantic Hardening | v2 | 5/5 | In Progress|  |
| 11. Antigravity Protocol and Credential Hardening | v2 | 0/TBD | Not started | - |
| 12. Cursor Evidence-Backed Hardening | v2 | 0/TBD | Not started | - |
| 13. Generic OpenAI Chat Completions | v2 | 0/TBD | Not started | - |
| 14. Command Code Product Separation | v2 | 0/TBD | Not started | - |
| 15. Exact OpenCode Go Evidence Gate | v2 | 0/TBD | Not started | - |
| 16. Cross-Provider Release Gate | v2 | 0/TBD | Not started | - |

## Scope Boundaries

- Google AI Studio Web remains excluded: no implementation, tests, support claims, cookie/SAPISIDHASH authentication, browser components, MakerSuite/session parser, or new dependencies.
- Credential-file writeback remains unchanged; new subscription discovery is read-only and any persistence change requires separate explicit approval.
- OpenCode Go support is exact-model and exact-wire only; Phase 15 may validly support no tuples.
- Observable provider/configuration changes update their affected documentation and translations within the implementation phase; Phase 16 verifies parity rather than deferring those updates.
