# Milestones

## v1 OpenCodex Port (Shipped: 2026-09-06)

**Delivered:** OpenCodex-proven Responses behavior and tests adapted into a
bounded, protocol-faithful Rust gateway without importing its platform
architecture.

**Phases completed:** 8 phases, 25 plans, 56 tasks

**Key accomplishments:**

- Added authenticated, bounded inbound Responses WebSockets with local warmup,
  backpressure, terminal-aware relay, and prompt cancellation.
- Added exact native Responses routing, opaque compaction, and provider-aware
  account-pool resilience with bounded hard-quota and Retry-After handling.
- Added strict Responses-to-Anthropic request and JSON/SSE translation while
  preserving native byte fidelity and rejecting lossy input before dispatch.
- Added capability-aware heterogeneous fallback and a default-off,
  request-authorized collaboration bridge with no hidden recovery calls.
- Added a configurable process shutdown deadline spanning HTTP, SSE, and
  WebSocket work, with deterministic cancellation and RAII-release coverage.
- Kept README, engineering docs, examples, and Nimbus English/ko/ja/zh-cn
  surfaces synchronized through all observable changes.

**Stats:**

- 180 files changed across the milestone (`+15,629 / -237` in the planning range)
- 145,863 lines across current Rust source and integration tests
- 8 phases, 25 plans, 56 plan tasks
- 2 calendar days (2026-09-05 → 2026-09-06)

**Verification:** 20/20 v1 requirements, 8/8 phases, 18/18 integration seams,
and 9/9 end-to-end flows passed. Known verification overrides: 0.

**Git range:** `c6a9032` → `6be4fa9`

**What's next:** Evidence-driven v2 work only when production transcripts or
usage justify response repair or durable continuation support.

---
