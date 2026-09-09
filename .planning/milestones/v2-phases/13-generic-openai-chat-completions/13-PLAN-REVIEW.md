---
status: pending_independent_check
phase: 13-generic-openai-chat-completions
---

# Local plan review

GLM/high produced five sequential plans, thirteen task verification rows and
an API coverage matrix. Independent checker has NOT launched: host launch and
follow-up controls disappeared after planner completion. This local root pass
does not stand in for the independent checker gate.

## Corrections applied

- 13-01 owns src/proxy/capability.rs, whose AdapterKind match is exhaustive.
- 13-02 owns the adapter call site that adopts the shared URL builder.
- Anthropic tool_use.input is an object, serialized once into Chat arguments;
  it is not an existing raw JSON string. Reject non-object inputs.
- Invalid UTF-8 tests belong at raw HTTP decoding, not inside a serde Value.
- Cancellation awaiting headers may happen after request send. Require bounded
  upstream closure and admission reclamation, not zero bytes/exact read counts.
- Usage accepts integral JSON numbers, not numeric strings or lossy casts.
- content_filter explicitly errors instead of emitting an invalid Anthropic
  stop_reason; independent checker must confirm consistency with locked scope.
- Streaming checks use bounded already-received bytes, not whole-response batches.
- Redirect refusal fixture requires zero target requests, not merely no bearer.
- 13-05 now owns its pure test file, validation map and exact docs paths rather
  than broad docs directories. Site build goes through the isolated wrapper.
- COVERAGE decisions use the exact allowed vocabulary and short capability IDs.

## Executed checks

API coverage verify-pre: passed, 25 capabilities (13 integrate,12 opt-out).
git diff --check: passed before save. No production source or tests changed;
prior full test/smoke evidence is not evidence for the unimplemented Chat adapter.

## Remaining before execution

1. Independent checker reads all five plans and actual source, audits all9
   requirements/all11 decisions, complete enum/module wiring, concrete test
   command paths, failure directions, strict terminal order and budgets.
2. Check exact Chat protocol grammar against official primary sources; source
   evidence to date is adjacent OpenCodex provenance, not verified official spec.
3. Reconcile edge identifiers in prose with the actual14-row JSON report; no
   unclassified row may be silently dropped or misrepresented as executed.
4. Verify helper paths in read_first from executor cwd and non-placeholder
   artifacts-produced symbols, then finalize state via GSD planned-phase.

## Independent checker round 1 — GLM/high (completed)

Result: ISSUES FOUND — 2 blockers, 5 warnings, 2 info. Recorded as issues
found, NOT a pass. The revision below closes each finding; an independent
recheck is still required before any execution claim.

| ID | Severity | Finding | Disposition |
|---|---|---|---|
| B1 | blocker | Reasoning claimed in 13-03 truth 1 with no executable task behavior | New 13-03 truth plus Task 1/2 behavior and actions now define the supported provider extensions (unary message reasoning_content; delta reasoning_content / delta reasoning), unary and streaming thinking mapping with no fabricated signature, alias-conflict, non-string type-error and empty-string handling, and text/reasoning arrival-order preservation. 13-02 Task 1 owns the request-history policy: supported plaintext thinking preserved into assistant reasoning_content; signed or redacted representations strictly rejected; nothing silently dropped. 13-PROTOCOL-EVIDENCE.md marks these as provider extensions, NOT official OpenAI fields. |
| B2 | blocker | Official protocol evidence must be an executable prerequisite before 13-01 fixtures/code | Created 13-PROTOCOL-EVIDENCE.md with exact source links, per-source provenance ("root fetched official sources" for the create/schema/function-calling facts), and separate official-facts vs gateway-hardening vs provider-extension sections. Added 13-01 Task 0 preflight gate plus read_first consumption before fixtures. The streaming-events page was fetched during this revision: include_usage/empty-last-chunk/usage-null wording verified; explicit [DONE] wording was NOT found and is recorded as unresolved official wording; the fail-closed EOF terminal (D-07) does not depend on it. No live provider smoke claimed. |
| W1 | warning | EDGE JSON has 14 rows with no IDs; plans cited invented E-xx IDs | All invented E-01/E-02/E-07/E-08/E-12/E-13/E-14 citations removed from every plan, VALIDATION and COVERAGE; replaced with compound requirement/category keys (e.g. CHAT-03/empty, CHAT-09/boundary). JSON rows unchanged: 12 explicit + 2 unclassified preserved. |
| W2 | warning | Mis-cited decisions: 13-02 truth 4 and Task 3 cite D-04 (should be D-02); 13-03 truth 3 cites D-08 (should be D-07) | Corrected to D-02 and D-07 respectively; other refs inspected (13-04 D-03/D-09, 13-05 D-10/D-11 are correct). |
| W3 | warning | 13-02 Task 1 lacked an explicit unsupported-input allow/reject policy | Added the explicit table: unknown top-level fields rejected (typed 400); supported metadata consumed locally with zero upstream presence asserted at the wire; unknown content block types rejected; assistant thinking history policy (plaintext preserved, signed/redacted rejected); no general unknown-field stripping anywhere (D-04). |
| W4 | warning | 13-01 modifies eleven files, over the >10 guideline | Conscious justification note added: inherent vertical tracer, minimal wiring kept as one slice. |
| W5 | warning | 13-05 touches sixteen files, mostly mechanical locale/doc mirrors | Conscious justification note added: bounded chunks; no split merely to satisfy the file-count guideline. |
| I1 | info | Verify-filter substrings must match actual fixture function names | Naming requirements added to actions and acceptance criteria for every substring filter (response, endpoint, assembly, bound, cap, openai_chat_terminal, openai_chat_auth, tracer names); zero-filter fails_when clauses preserved. |
| I2 | info | Truths should be observable evidence, not descriptive claims | 13-01 truth 3 now cites exactly-one-request and zero-redirect-target-request fixtures; other truths already carry fixture-grounded wording. |

Status after this revision: revision done, pending independent recheck. Not an
implementation pass; no source implementation or test execution was performed.

Computer-specific smoke remains blocked by host controls; user approved using
shell/network for continued work. Only primary or GPT-6 Astra/high performs
Computer interactions. No change to production isolation or backup boundary.

## Root corrections after revision

The root fetched the official Chat reference StreamOptions section and resolved
`[DONE]`: final usage precedes the sentinel. The preflight now independently
checks all three evidence sections and consumes already-fetched evidence without
requiring repeat network availability. Unary output has deterministic thinking,
text, tools order (JSON field order is not arrival order); streaming preserves
chunk order. Null reasoning means absent; non-string values and conflicting
aliases fail closed. Unsupported metadata is rejected before dispatch rather
than called locally consumed with no consumer. These corrections supersede the
corresponding provisional dispositions above. Independent recheck remains pending.

## Independent recheck — no blocking findings

The separate chat_plan_recheck GLM/high agent returned ISSUES FOUND with
zero blockers, two minor warnings and two advisory items. Both original
blockers are closed. Root applied the requested content_filter error truth
and embedded-error D-07 citation corrections. The cap-probe advisory is
covered by the phase-wide conformance run; both test files must be rerun after
adding those probes. Unclassified probe placeholders remain unresolved and
honestly disclosed; authored assumptions are specified in the plans.
Deterministic verify-command-paths and verify-failure-directions checks
returned status ok (19 commands). Plan structure passed all five plans.
Planning is accepted for execution, not an implementation or runtime pass.
