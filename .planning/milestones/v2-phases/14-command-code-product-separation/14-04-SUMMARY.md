---
phase: 14-command-code-product-separation
plan: "04"
subsystem: api
status: complete
completed: 2026-09-08
requirements-completed: [CCS-05, CCS-06]
---

# Plan 14-04 — Checked subscription responses

## Delivered

- Shared checked semantic machine for unary and incremental SSE output. Text,
  reasoning, authentic tool IDs, strict JSON arguments, and source-backed cache
  usage translate consistently. Streaming retains no final content collection.
- Only authoritative compatible finish records permit success, held until clean
  framed EOF. Unknown, malformed, duplicate, late, failed and truncated records
  fail closed. Failure is sticky, including after closure. Both usage fields are
  validated before totalUsage precedence; cache-inclusive input is checked and
  converted to uncached Anthropic input without clamping or double accounting.
- Bounded record/residual/wire, semantic bytes, tool arguments, tools and content
  blocks. CRLF framing does not consume the JSON byte allowance. An absolute
  120-second record-progress deadline defeats partial-byte slow drips; complete
  records refresh it. Dropping the relay drops the upstream body.
- Canonical-host private-CA gateway fixtures prove output arrives before finish,
  unary/SSE parity, tool-first ordering, neutral errors, strict terminal behavior,
  and below/at/above record limits. TLS fixture shutdown now flushes buffered
  bytes and close_notify; production EOF checks were not relaxed.

## Executed evidence

- Six 14-04 *-RED.json artifacts record observed assertion failures after
  successful compilation and passed GSD RED gates before corresponding fixes.
  Any TAP is explicitly labelled manual Rust-libtest transcription.
- All 23 translation tests passed. Boundary/progress and canonical TLS gateway
  filters passed with nonzero selected tests. Paused-time progress checks are
  synthetic transport tests, not captured network evidence.
- Final chain passed format, warnings-denied Clippy, build, and then full
  warnings-denied all-features workspace tests: **2,924 passed, 0 failed,
  2 existing ignored**, 31 workspace result groups. The separate focused group
  of 23 tests is not double-counted.
- Every stateful command used /tmp/shunt-phase12-isolated-run.cjs. All completed
  commands reported production OpenCodex config mtime/SHA and backup inventory
  unchanged. No real credential file or live generation was used.
- Post-wave schema/UI gates: block:false. Codebase drift is the existing
  non-blocking warn hook (171 global structural items, spawn_mapper:false).
  No gate was disabled. Relevant source was directly inspected.
- Implementation commits span 1c7a473 through a0b94dd; final fix a0b94dd.

## Remaining milestone gates

Read-only credential fallback, replay and real-socket cancellation proofs remain
14-05. Documentation across README/docs/site and all maintained locales, full
matrix and CLI smokes remain 14-06, in the same PR; wiki is untouched. Independent
GLM/high review is in flight and is not claimed passed here. Live provider and
GUI acceptance are not implied by hermetic fixtures; Phase 16 owns live gates.
