---
milestone: v2
audited: 2026-09-09
status: passed
scores:
  requirements: 63/63
  phases: 8/8
  integration: 7/7
  flows: 7/7
gaps:
  requirements: []
  integration: []
  flows: []
nyquist:
  compliant_phases: [13, 14, 15, 16]
  partial_phases: []
  not_validated_phases: [9, 10, 11, 12]
  missing_phases: []
  overall: legacy-status-normalization-advisory
tech_debt:
  - phase: 12
    items: [Existing cosmetic grpc-message percent-decoding deferral]
  - phase: metadata
    items: [Phases 9-12 retain legacy validation status vocabulary; historical checklists remain superseded by final audit sections]
---

# v2 Provider Compatibility milestone audit

All 63 committed requirements are satisfied within the agreed hermetic and
skip-capable release scope. Eight phases and 48 plans are complete. No missing
integration or runtime blocker was found. No live availability, Go admission,
merge, publication or deployment is implied.

## Method and independence

Root read all eight canonical phase verification reports and extracted all
48 SUMMARY requirement-completion fields, then cross-checked the 63 checked
requirements and traceability ownership. OGO-03/04 were implemented and verified
but omitted from 15-01 summary metadata; root repaired that omission against its
actual router/gate evidence, not by inventing work. No requirement is orphaned.
GEM-01..05 map to Phase10's explicit 14-truth evidence table; that older report
uses descriptions rather than a separate identifier table.

The cross-phase integration check was performed inline after native subagent
dispatch controls became unavailable; this is the plugin's permitted host
fallback, not a claimed independent integration-agent result. Root loaded the
bundled gsd-integration-checker contract and traced actual calls. Independent
GLM/high release evidence/security/license review is separately recorded in
16-INDEPENDENT-REVIEW.md and is not substituted for this integration check.

## Cross-phase connections and end-to-end flows

| Connection / flow | Actual call chain and negative boundary | Result |
|---|---|---|
| Existing Codex/native preservation | server opt-in Responses ingress → native routing → Responses HTTP/WS; failover/continuation use shared pre-output boundaries; endpoint/fallback/retry tests pass | WIRED |
| Gemini and native Antigravity | routing selects Gemini; one request credential supplies identity/project; native catalog validates exact facts; checked SSE handles both modes; same-account 401 replay is separate and bounded | WIRED |
| Cursor | failover dispatch → admission/history before auth → prepared Run/KV state → Connect parser/relay; TurnGuard Drop signals and aborts sender; cancellation/terminal tests pass | WIRED |
| Generic Chat | explicit kind → OpenAiChatAdapter → configured endpoint and API-key → strict compiler/machine; streaming uses bytes_stream; ambiguous post-send outcomes cannot advance outer failover | WIRED |
| Two Command Code products | API preset invokes Chat; subscription validates origin and translates before read-only credential resolution → fixed ENDPOINT and allowlist headers → Decoder/CommandCodeMachine → client; product-isolation and cancellation tests pass | WIRED |
| Go rejection and generic preservation | failover filters capabilities then enforces empty Go gate before check_inbound_auth/dispatch; native Codex also gates before dispatch; generic control, CLI and mutation tests pass | WIRED |
| Release artifacts to user-visible claims | durable model/contract ledger + notices → substantive validators; four-locale sources → 173-page site; actual Computer navigation/rendering; no source-only row promoted to live | WIRED |

Primary API consumers are external clients and real-router/CLI fixtures, not
a frontend application. /v1/messages and opt-in Responses routes have actual
test consumers. Inference authentication remains conditional on configured
inbound policy, with separate provider credentials; an intentionally open
passthrough is not mislabeled an unprotected application account page.

## Three-source requirements and integration map

Each row has a checked REQUIREMENTS entry, a completed owning-phase SUMMARY
entry and the owning VERIFICATION's passing behavioral evidence. The path
column identifies integration, not a claim that each requirement needs a new API.

| Requirement | Phase | Integration path | Three-source result |
|---|---|---|---|
| PRES-01 | 9 | Native Responses + generic Anthropic → shared failover/commitment → real ingress clients | Satisfied |
| PRES-02 | 9 | Native Responses + generic Anthropic → shared failover/commitment → real ingress clients | Satisfied |
| PRES-03 | 9 | Native Responses + generic Anthropic → shared failover/commitment → real ingress clients | Satisfied |
| PRES-04 | 9 | Native Responses + generic Anthropic → shared failover/commitment → real ingress clients | Satisfied |
| PRES-05 | 9 | Native Responses + generic Anthropic → shared failover/commitment → real ingress clients | Satisfied |
| GEM-01 | 10 | Google credential/project → Gemini envelope → checked SSE → unary/streaming client | Satisfied |
| GEM-02 | 10 | Google credential/project → Gemini envelope → checked SSE → unary/streaming client | Satisfied |
| GEM-03 | 10 | Google credential/project → Gemini envelope → checked SSE → unary/streaming client | Satisfied |
| GEM-04 | 10 | Google credential/project → Gemini envelope → checked SSE → unary/streaming client | Satisfied |
| GEM-05 | 10 | Google credential/project → Gemini envelope → checked SSE → unary/streaming client | Satisfied |
| ANT-01 | 11 | Account-bound catalog/admission → native Gemini agent envelope → one 401 replay/strict relay | Satisfied |
| ANT-02 | 11 | Account-bound catalog/admission → native Gemini agent envelope → one 401 replay/strict relay | Satisfied |
| ANT-03 | 11 | Account-bound catalog/admission → native Gemini agent envelope → one 401 replay/strict relay | Satisfied |
| ANT-04 | 11 | Account-bound catalog/admission → native Gemini agent envelope → one 401 replay/strict relay | Satisfied |
| ANT-05 | 11 | Account-bound catalog/admission → native Gemini agent envelope → one 401 replay/strict relay | Satisfied |
| ANT-06 | 11 | Account-bound catalog/admission → native Gemini agent envelope → one 401 replay/strict relay | Satisfied |
| ANT-07 | 11 | Account-bound catalog/admission → native Gemini agent envelope → one 401 replay/strict relay | Satisfied |
| ANT-08 | 11 | Account-bound catalog/admission → native Gemini agent envelope → one 401 replay/strict relay | Satisfied |
| CUR-01 | 12 | Request-local admission/history → Run + KV → checked Connect relay → response-owned TurnGuard | Satisfied |
| CUR-02 | 12 | Request-local admission/history → Run + KV → checked Connect relay → response-owned TurnGuard | Satisfied |
| CUR-03 | 12 | Request-local admission/history → Run + KV → checked Connect relay → response-owned TurnGuard | Satisfied |
| CUR-04 | 12 | Request-local admission/history → Run + KV → checked Connect relay → response-owned TurnGuard | Satisfied |
| CUR-05 | 12 | Request-local admission/history → Run + KV → checked Connect relay → response-owned TurnGuard | Satisfied |
| CUR-06 | 12 | Request-local admission/history → Run + KV → checked Connect relay → response-owned TurnGuard | Satisfied |
| CUR-07 | 12 | Request-local admission/history → Run + KV → checked Connect relay → response-owned TurnGuard | Satisfied |
| CUR-08 | 12 | Request-local admission/history → Run + KV → checked Connect relay → response-owned TurnGuard | Satisfied |
| CHAT-01 | 13 | Typed config → Chat endpoint/compiler → checked JSON/SSE → Anthropic client | Satisfied |
| CHAT-02 | 13 | Typed config → Chat endpoint/compiler → checked JSON/SSE → Anthropic client | Satisfied |
| CHAT-03 | 13 | Typed config → Chat endpoint/compiler → checked JSON/SSE → Anthropic client | Satisfied |
| CHAT-04 | 13 | Typed config → Chat endpoint/compiler → checked JSON/SSE → Anthropic client | Satisfied |
| CHAT-05 | 13 | Typed config → Chat endpoint/compiler → checked JSON/SSE → Anthropic client | Satisfied |
| CHAT-06 | 13 | Typed config → Chat endpoint/compiler → checked JSON/SSE → Anthropic client | Satisfied |
| CHAT-07 | 13 | Typed config → Chat endpoint/compiler → checked JSON/SSE → Anthropic client | Satisfied |
| CHAT-08 | 13 | Typed config → Chat endpoint/compiler → checked JSON/SSE → Anthropic client | Satisfied |
| CHAT-09 | 13 | Typed config → Chat endpoint/compiler → checked JSON/SSE → Anthropic client | Satisfied |
| CCK-01 | 14 | commandcode API preset → generic Chat adapter; separate subscription identity | Satisfied |
| CCK-02 | 14 | commandcode API preset → generic Chat adapter; separate subscription identity | Satisfied |
| CCK-03 | 14 | commandcode API preset → generic Chat adapter; separate subscription identity | Satisfied |
| CCS-01 | 14 | command-code kind → read-only auth/canonical headers → bounded NDJSON relay | Satisfied |
| CCS-02 | 14 | command-code kind → read-only auth/canonical headers → bounded NDJSON relay | Satisfied |
| CCS-03 | 14 | command-code kind → read-only auth/canonical headers → bounded NDJSON relay | Satisfied |
| CCS-04 | 14 | command-code kind → read-only auth/canonical headers → bounded NDJSON relay | Satisfied |
| CCS-05 | 14 | command-code kind → read-only auth/canonical headers → bounded NDJSON relay | Satisfied |
| CCS-06 | 14 | command-code kind → read-only auth/canonical headers → bounded NDJSON relay | Satisfied |
| CCS-07 | 14 | command-code kind → read-only auth/canonical headers → bounded NDJSON relay | Satisfied |
| CCS-08 | 14 | command-code kind → read-only auth/canonical headers → bounded NDJSON relay | Satisfied |
| OGO-01 | 15 | Explicit Go identity → empty pre-credential gate at both ingress paths | Satisfied |
| OGO-02 | 15 | Explicit Go identity → empty pre-credential gate at both ingress paths | Satisfied |
| OGO-03 | 15 | Explicit Go identity → empty pre-credential gate at both ingress paths | Satisfied |
| OGO-04 | 15 | Explicit Go identity → empty pre-credential gate at both ingress paths | Satisfied |
| SAFE-01 | 9 | Shared bounds, credential slots, retry and RAII → all provider adapters | Satisfied |
| SAFE-02 | 9 | Shared bounds, credential slots, retry and RAII → all provider adapters | Satisfied |
| SAFE-03 | 9 | Shared bounds, credential slots, retry and RAII → all provider adapters | Satisfied |
| SAFE-04 | 9 | Shared bounds, credential slots, retry and RAII → all provider adapters | Satisfied |
| SAFE-05 | 9 | Shared bounds, credential slots, retry and RAII → all provider adapters | Satisfied |
| SAFE-06 | 9 | Shared bounds, credential slots, retry and RAII → all provider adapters | Satisfied |
| SAFE-07 | 9 | Shared bounds, credential slots, retry and RAII → all provider adapters | Satisfied |
| REL-01 | 16 | Durable evidence/notices → tests/docs/local smokes/Computer → final release gates | Satisfied |
| REL-02 | 16 | Durable evidence/notices → tests/docs/local smokes/Computer → final release gates | Satisfied |
| REL-03 | 16 | Durable evidence/notices → tests/docs/local smokes/Computer → final release gates | Satisfied |
| REL-04 | 16 | Durable evidence/notices → tests/docs/local smokes/Computer → final release gates | Satisfied |
| REL-05 | 16 | Durable evidence/notices → tests/docs/local smokes/Computer → final release gates | Satisfied |
| REL-06 | 16 | Durable evidence/notices → tests/docs/local smokes/Computer → final release gates | Satisfied |

REL-02/03 are self-contained evidence/license checks rather than new runtime
connections; they integrate through release tests and documentation. OGO-02/03
are conditional promotion contracts, valid with zero admissions, as explicitly
approved in Phase15 UAT. No fake session producer or live proof was required.

## Current executed verification

- 16-FINAL-GATES.json: 2,979 passed, zero failed, two existing ignored;
  format and warnings-denied all-target/all-feature Clippy passed. Every
  designated focused filter selected nonzero tests.
- Final site: 173 pages built, 169 indexed in four languages; all 12 inspected
  pages and 260 local fragment links pass. Owned Computer review passed its
  inspected desktop/locale/mobile scope.
- Additional project-skill smoke: serialized isolated
  bash .claude/skills/run-shunt/smoke.sh rebuilt in 31.88s and passed all five
  config/liveness/discovery/proxy/malformed checks. Its owned ports 31711/31712
  and temporary scratch were cleaned by the driver. Production fingerprint and
  backup/invalid inventory were unchanged.
- Live attempts: 0/8, US$0 planned; every unavailable/unsafe path has a reason
  in 16-SMOKE.md. CLI/subagent success is not Shunt live-wire evidence.

## Residuals and lifecycle safety

The listed cosmetic and legacy metadata advisories are not runtime/requirement
failures and do not require a risk waiver. Nyquist discovery is enabled: phases
9-12 use older ready/complete/passed status names with true compliance and green
executed evidence, so the current workflow classifies them NOT-VALIDATED pending
normalization, not failed. Phase13's final Validation Audit supersedes its
historical unchecked template; all 13 task rows are executed-pass. This audit
does not auto-run validate-phase or manufacture historical execution.

Phase12's earlier live deferral is now resolved as the explicitly permitted
Phase16 skip; it remains no live availability claim. Prior Computer blocks
remain historical; actual Phase16 observations are documented separately.
Phase14 initial compile failure was never relabeled behavioral RED. Phase16
per-wave sampling deviation remains recorded.

Durable Go fixture copies prevent archival from breaking permanent tests.
Existing user configuration/state/.gsd dirt and external owner-only backups
must be preserved. No generated wiki changes, new runtime dependencies,
Google AI Studio Web implementation or credential-writeback expansion was found.
Milestone archival/cleanup remains the next lifecycle operation.
