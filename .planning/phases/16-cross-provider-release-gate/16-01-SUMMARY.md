---
phase: 16-cross-provider-release-gate
plan: 01
subsystem: testing
tags: [release, provenance, conformance]
requires:
  - phase: 15-exact-opencode-go-evidence-gate
    provides: Source-only Go evidence and zero-admission policy
provides:
  - Durable ledger with 45 documented model tuples and six unbound contracts
  - Mutation-tested exact identity and provenance validator
affects: [16-02, 16-04, 16-05]
actuals:
  tokens: 38766
  tasks: 2
  commits: 2
tech-stack:
  added: []
  patterns: [Frozen finite tuple identity digest, separate unbound contract inventory]
key-files:
  created: [docs/provider-release-evidence.md, tests/release_matrix.rs]
  modified: []
key-decisions:
  - Generic and account-catalog contracts are not invented exact model identities
  - Shared adapter tests do not establish per-model live availability or account entitlement
patterns-established:
  - Permanent release tests read durable docs rather than active planning paths
requirements-completed: [REL-01, REL-02]
coverage:
  - id: D1
    description: Finite model and contract traceability with empty Go admission
    requirement: REL-01
    verification:
      - kind: integration
        ref: tests/release_matrix.rs#release_matrix_ledger
        status: pass
      - kind: integration
        ref: tests/release_matrix.rs#release_matrix_ledger_rejects_mutations
        status: pass
    human_judgment: false
  - id: D2
    description: Source provenance fields and explicit absence of live/capture proof
    requirement: REL-02
    verification:
      - kind: integration
        ref: tests/release_matrix.rs#release_matrix_ledger_validation_is_idempotent
        status: pass
    human_judgment: true
    rationale: Final independent provenance/license review remains a release gate in plans 02 and 05
duration: 29min
completed: 2026-09-09
status: complete
---

# Phase 16 Plan 01 Summary

Durable release evidence ties 45 documented model tuples and six generic/account-catalog contracts to named existing tests, without claiming live support.

## Performance

Recorded commit interval: 2026-09-09T05:19:02Z to 05:47:38Z. Two tasks; two source/artifact files. Realized task diff: 155,064 characters / 4 = 38,766 estimated tokens, not harness usage.

## Accomplishments and task commits

1. `079c328`: GLM/high executor committed the compiling RED validator. Its progress report recorded three behavioral failures on the missing ledger, with production fingerprints unchanged.
2. `f1cb456`: root completed and corrected the evidence mapping, exact identity guard, strict absent capture/live markers, and separate unbound contracts. Current isolated run selected all three tests: 3 passed, 0 failed.

An external mutation inserted a synthetic Go admission into the durable ledger. The exact `release_matrix_ledger` test failed with `Go admission must remain empty` (1 selected, 1 failed, 2 filtered). Restoring admitted=[] and rerunning all three tests passed. All runs used the serialized isolation wrapper and reported unchanged production config mtime/SHA-256 and backup/invalid inventory.

## Decisions and deviations

Root paused the executor when review found cross-transport mappings and nonexistent test references in its draft. Corrected native Antigravity versus CLI evidence, Command Code API versus subscription evidence, actual Cursor wire/provenance, and existing module-local lifetime/auth tests overlooked by the initial inventory. No runtime code or provider semantics changed.

Rule 1 correctness adjustment: exact model rows and unbound API/account-catalog contracts are separate. Generic placeholders cannot satisfy the exact-model inventory. The identity digest freezes the reviewed tuple set. Vercel's documented Anthropic route is explicitly represented instead of implying it was covered by an unrelated generic Chat route.

Cancellation evidence is described at the level actually exercised: some citations prove terminal-failure or process-shutdown cleanup, while others directly prove client-disconnect behavior. Neither source-derived tests nor development subagent calls are live provider evidence.

## Issues and remaining gates

The ledger is a source/contract traceability artifact, not a guarantee that every model is available to every account. Its schema checks cannot replace final semantic/provenance review. Plan-local requirement completion does not close the global release requirements before plans 02/04/05. Full suite, clippy, final scope/security review, smoke dispositions, and milestone closeout remain pending.

## User Setup Required

None.
