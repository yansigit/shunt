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

Computer-specific smoke remains blocked by host controls; user approved using
shell/network for continued work. Only primary or GPT-6 Astra/high performs
Computer interactions. No change to production isolation or backup boundary.
