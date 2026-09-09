# Milestones

## v2 Provider Compatibility (Completed: 2026-09-09)

**Delivered:** Bounded provider compatibility, strict translation and evidence
gates; locally complete and verified, not remotely published or deployed.

**Phases completed:** 9–16; 8 phases, 48 plans, 110 plan task elements.
The archive helper's 37-task statistic recognized only some summary schemas;
110 is counted from the actual plan task elements.

**Key accomplishments:**

- Preserved Codex/native and Vercel behavior with tested replay and cleanup boundaries.
- Hardened Gemini/Antigravity identity, tool history, terminals and account-bound replay.
- Hardened Cursor with request-local history/KV, strict Connect parsing and cancellation.
- Added generic Chat and isolated Command Code API/subscription contracts.
- Kept Go admission empty under exact-model evidence gates.
- Completed durable provenance/MIT notices, four-locale docs, Computer and release verification.

**Verification:** 63/63 requirements, 8/8 phases, 7/7 integration flows;
2,979 tests passed, zero failed, two existing ignored. Format, warnings-denied
Clippy, 173-page site, 260 local fragment links, five-check owned smoke and
scoped Computer review passed. Live smoke: 0/8, US$0 planned, explicit skips.
Known verification overrides: 0. Minor advisories remain in the audit.

**Stats:** 394 files changed, +56,549 / −2,085 lines across the implementation
and planning range e029ae2..0414881; 2026-09-06 → 2026-09-09.
These are mixed code/docs/planning diff counts, not pure runtime LOC.

**Archive:** [roadmap](milestones/v2-ROADMAP.md),
[requirements](milestones/v2-REQUIREMENTS.md),
[audit](milestones/v2-MILESTONE-AUDIT.md).

**Next:** No new milestone scoped. No merge, push or deployment performed.

---

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
