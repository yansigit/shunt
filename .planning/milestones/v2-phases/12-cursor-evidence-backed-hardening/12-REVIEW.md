---
phase: 12-cursor-evidence-backed-hardening
reviewed: 2026-09-08T00:00:00Z
depth: standard
files_reviewed: 24
files_reviewed_list:
  - src/adapters/cursor/mod.rs
  - src/adapters/cursor/agent.rs
  - src/adapters/cursor/client.rs
  - src/adapters/cursor/connect.rs
  - src/adapters/cursor/admission.rs
  - src/adapters/cursor/history.rs
  - src/adapters/cursor/kv.rs
  - src/adapters/cursor/wire.rs
  - src/adapters/cursor/strict.rs
  - src/adapters/cursor/usage.rs
  - src/adapters/cursor/aggregate.rs
  - src/adapters/cursor/sse.rs
  - src/adapters/cursor/request.rs
  - src/adapters/cursor/response.rs
  - src/adapters/cursor/offload.rs
  - src/adapters/cursor/router_parity_tests.rs
  - src/adapters/cursor/cancellation_tests.rs
  - src/adapters/cursor/protocol_tests.rs
  - src/adapters/cursor/request_isolation_tests.rs
  - src/adapters/cursor/history_tests.rs
  - src/adapters/cursor/history_lifetime_tests.rs
  - src/adapters/cursor/kv_tests.rs
  - src/retry.rs
  - src/config.rs
findings:
  critical: 0
  warning: 1
  info: 1
  total: 2
status: clean
---

# Phase 12: Code Review Report

**Reviewed:** 2026-09-08T00:00:00Z
**Depth:** standard
**Files Reviewed:** 24
**Status:** issues_found

## Summary

Reviewed the phase 12 Cursor hardening scope (diff eec0479 base to HEAD, summaries 12-01 through 12-08) at standard depth. Read the new Cursor modules (admission, history, kv, wire, strict, usage, aggregate) and the changed transport paths (agent ingest, client, connect framing, mod dispatch, retry classification), plus docs-parity spot checks.
The implementation is in good shape: admission runs before credential resolution and history preparation (mod.rs line 100 before line 102); retry taxonomy is ConnectOnly with no replay after acceptance; KV and history stores are request-owned and bounded (192 entries, 512 KiB per entry, 8 MiB total); protobuf decoding is strict with bounded budgets; usage derivation is labeled estimated; provider docs describe the strictness, bounds, and error semantics.

One real robustness defect found (unbounded error-body buffering, the known concern, confirmed) plus one cosmetic note. No source files modified, no tests run, no live credentials, ports, or production state touched.

## Warnings

### WR-01: map_upstream_error buffers the error body without a bound

**File:** src/adapters/cursor/mod.rs:495-506
**Issue:** On non-success HTTP statuses, map_upstream_error calls upstream.text().await, which reads the whole response body into memory with no size cap. Every other byte path here is bounded: Connect framing caps payloads at 64 MiB (connect.rs:50); gzip decompression is capped at MAX_DECOMPRESSED_FRAME_BYTES (connect.rs:226-228); strict argument decoding uses an aggregate budget (strict.rs:14-31); JSON aggregation is capped (aggregate.rs charge); blobs are capped per entry and per request (kv.rs). A degenerate or hostile upstream returning a huge body on e.g. HTTP 502 would allocate without limit on exactly the path meant to handle failure. unwrap_or_default hides read errors but not the unbounded allocation.
**Fix:** Bound the error body before parsing: collect incrementally up to 64 KiB before parsing, with a total read deadline. Do not buffer the whole body before truncating. Existing status-table tests (mod.rs:730-789) should keep passing; add one test with an oversized error body asserting bounded memory and a truncated message.

## Info

### IN-01: grpc-message header used verbatim without percent-decoding

**File:** src/adapters/cursor/mod.rs:498-503
**Issue:** The grpc-message value is percent-encoded per gRPC over HTTP/2 convention but is surfaced verbatim as the user-facing message. Cosmetic only (readability); taxonomy is unaffected since it is message text, not a decision input.
**Fix:** Percent-decode with a lossy fallback to the raw value. Low priority; safe to defer.

---

Checked: transport, history, KV, protobuf strictness, admission, usage, retry, cancellation, and aggregate parity plus four-locale docs are consistent with the eight plan summaries; no further concrete defects found. Planning artifacts and generated wiki excluded from scope as instructed.
Reviewed: 2026-09-08T00:00:00Z
Reviewer: Codex (gsd-code-reviewer, standard depth)

## Resolution follow-up

WR-01 is addressed by incremental `bounded_error_text` collection: at most
64 KiB of input bytes, with a five-second total read deadline. The focused
`cursor_error_body_collection_is_bounded_and_times_out` test passed, covering
cap+1 and a partial body followed by a permanently pending stream. Existing
status and no-replay mapping is unchanged. Full gates are being rerun.
IN-01 remains deferred: cosmetic decoding is outside the required safety fix.
Both focused regressions now pass, including a real HTTP oversized 429 response
preserving Retry-After and no-replay classification. Formatting, all-target clippy,
the full all-features workspace suite (2,193 active library tests at that run),
the four-locale 161-page site build and all five rebuilt-binary smoke checks pass.
The final added HTTP regression passes separately (no production code changed
after that full run). No unresolved critical or warning findings remain; counts
above retain the original findings for audit history.
The report was initially written to the main checkout by the reviewer; the
orchestrator relocated it to the specified worktree without changing source.
README capability/setup text remains accurate; engineering and all four
provider-site locale pages describe the new error-read limits. Wiki is generated
and was not edited.
