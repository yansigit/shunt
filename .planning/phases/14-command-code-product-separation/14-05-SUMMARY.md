---
phase: 14-command-code-product-separation
plan: "05"
subsystem: auth
status: complete
completed: 2026-09-08
requirements-completed: [CCS-01, CCS-02, CCS-07]
---

# Plan 14-05 — Read-only credentials and request lifetime

Delivered in ae3b524. Production resolves SHUNT_COMMAND_CODE_TOKEN first; every
present value, including empty, is validated without file fallback. Only true
absence reads fixed ~/.commandcode/auth.json with apiKey and optional userId.
Reads are bounded at 16 KiB and waiting at five seconds. Unix nonblocking open
and fstat reject non-regular files. No refresh, write, repair, copy or whoami.
Tests use injected temporary paths, never the installed CLI home or global HOME.

Destination validation remains before lookup and is rechecked beside bearer
header construction. A mutated-snapshot test proves the second gate. Four real
canonical-host TLS replay tests prove one pre-connect retry retains credential
and session despite a changed synthetic env, and post-send timeout, redirects,
text and tools cannot replay. Four real-socket cancellation tests prove upstream
closure and actual follow-up admission for both modes before headers and mid-body.
No production origin override or shared retry vocabulary change was introduced.

## Executed gates

- 13 lifetime tests passed, including all file, dispatch, replay and cancellation
  filters; file tests also passed with parallel execution after counter isolation.
- Format check, all-targets/all-features warnings-denied Clippy, build, then full
  warnings-denied all-features workspace suite passed: **2,937 passed, 0 failed,
  2 existing ignored**, 31 result groups.
- Every stateful command used the fresh-home isolation wrapper. Production
  OpenCodex config mtime/SHA and backup inventory remained unchanged throughout.
- Schema/UI post-wave gates block:false. Existing codebase-map drift is the
  non-blocking warn hook (171 elements, spawn_mapper:false). No gate disabled.

## Evidence limits and remaining work

See 14-05-EXECUTION-NOTES.md: auth executor's initial compiler failure is not a
valid behavioral RED assertion and no TDD RED gate pass is claimed. Root rejected
and corrected its initial empty-env semantics, isolated parallel counters and
independently reran tests. Already-correct retry/cancellation behavior received
passing characterization tests, not manufactured RED runs. This process gap is
preserved for final audit; functional verification above is observed evidence.

GLM/high auth execution completed; the response reviewer returned no review
before interruption, so no independent review pass or Luna run is claimed.
Plan 14-06 retains the full scenario/auth matrix, all four documentation locales,
site build and owned CLI smokes. Phase 16 owns live provider acceptance; hermetic
TLS tests and CLI fallback are not a passed GUI or live inference evaluation.
