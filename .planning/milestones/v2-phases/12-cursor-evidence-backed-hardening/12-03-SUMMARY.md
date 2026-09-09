---
phase: 12-cursor-evidence-backed-hardening
plan: "03"
subsystem: cursor
tags: [cursor, history, protobuf, kv, h2]
requires: [12-02]
provides: [request-owned structured history, bidirectional blob hydration, pre-dispatch continuation guard]
affects: [12-04, 12-05, 12-06, 12-08]
tech-stack:
  added: []
  patterns: [bounded content-addressed request storage, explicit session identity, owned duplex lifetime]
key-files:
  created: [src/adapters/cursor/history.rs, src/adapters/cursor/history_tests.rs, src/adapters/cursor/history_lifetime_tests.rs, src/adapters/cursor/kv.rs, src/adapters/cursor/kv_tests.rs, src/adapters/cursor/wire.rs, docs/cursor-request-history.md]
  modified: [src/adapters/cursor/agent.rs, src/adapters/cursor/mod.rs, src/adapters/cursor/request.rs, src/adapters/cursor/router_parity_tests.rs]
key-decisions:
  - User explicitly approved bounded memory-only per-request KV architecture; no durable or shared cache.
  - Only exact composer-2.5 structured tool history is admitted, with explicit stable client session metadata and retained paired context.
  - Server keys remain opaque; local SHA-256 keys are immutable and conflicting replacement is rejected.
requirements-completed: [CUR-03]
completed: 2026-09-07
status: complete
---

# Phase 12 Plan 03 Summary

Authentic output tool IDs now survive into paired structured history, and the
active Run request actually serves the referenced blobs over its bidirectional
HTTP/2 body. Implementation commit: `a6b0ff3`; earlier output-ID fix: `69f1329`.

## Evidence and verification

- Ordinary, compacted, recovered, and multi-round fixtures decode actual turn,
  step, MCP argument, call-ID and result fields. Recovery reconstructs after
  dropping the previous store; summary wording never controls identity.
- Whole-router TLS/h2 fixtures request each root/user/turn/step blob and validate
  hashes and authentic paired IDs before returning JSON/SSE output.
- Whole-router malformed continuation returns Anthropic-shaped 400 and observes
  zero upstream dispatch, for both stream settings.
- Limits, unknown gets, opaque keys, immutable conflicts, malformed/trailing
  protobuf, proto3 omitted zero/empty defaults, independent stores, missing or
  closed KV channels, and cancellation of a backpressured sender are covered.
- Latest Cursor library suite: 256 passed, 0 failed, 1 existing ignored.
- All-feature workspace suite: passed; final small proto3-default adjustment was
  then rechecked with the complete Cursor suite and all-target clippy.
- Format and all-target/all-feature clippy with warnings denied: passed.
- Rebuilt binary smoke: all five checks passed on 31711/31712; owned temporary
  mock/gateway processes and smoke configuration cleaned up.
- Site: 161 pages built successfully across maintained locales.
- Every test tree inherited a fresh OPENCODEX_HOME; production config mtime/SHA
  and invalid/backup inventory unchanged. No live Cursor Run call was made.

## Scope and deviations

The approved addendum permits new focused private modules and duplex ownership.
Sender cancellation before headers was pulled forward from plan 08 because new
KV replies require a correctly owned lifetime. Plan 08 still owns broader
cancellation/capacity coverage. Muse reviewed schema read-only; GLM wired the
transport; the primary completed history, independent hydration fixtures,
strictness/default fixes, integration, and verification. An initial classifier
draft only matched prompt markers; it was replaced before commit, with its eight
tests migrated into stronger structural/admission assertions.

Supported history requires original paired IDs, retained user context and stable
metadata.session_id (or session_id in JSON-string metadata.user_id). Opaque
checkpoints, unsupported wire models, image history and mixed user text/results
fail explicitly. Locally derived UUIDv8 conversation identity is not a fabricated
tool ID. Request budgets are 192 entries, 512 KiB each, 8 MiB total; KV sends time
out after five seconds. No disk state, credential changes or cross-request cache.

README and provider docs were updated in English/ko/ja/zh-cn; engineering details
are in docs/cursor-request-history.md. Generated wiki was considered and left
untouched. Protocol fixtures are not a live model/account availability claim.
Plan 04 is next; Phase 12 and the milestone are not complete.
