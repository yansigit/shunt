---
phase: 12-cursor-evidence-backed-hardening
plan: "02"
subsystem: testing
tags: [cursor, concurrency, routing, h2]
requires: []
provides: [pinned Run destination regression, concurrent request fact isolation]
affects: [12-03, 12-08]
tech-stack:
  added: []
  patterns: [owned TLS connection and request task sets, schema-derived request projection]
key-files:
  created: [src/adapters/cursor/request_isolation_tests.rs]
  modified: [src/adapters/cursor/mod.rs, src/adapters/cursor/agent.rs]
key-decisions:
  - Observe real outgoing h2 requests through the adapter client without changing the production destination.
  - Decode mode and fast fields structurally; assert exact response-to-request pairing.
requirements-completed: [CUR-01, CUR-02]
coverage:
  - id: D1
    description: Three concurrent model requests retain the pinned HTTPS Run destination and required request headers.
    verification:
      - kind: integration
        ref: cursor_destination_pin_regression_enforces_run_url_for_mixed_models
        status: pass
    human_judgment: false
  - id: D2
    description: Each request preserves its own model, fast flag, mode, tool marker, request ID, and response pairing.
    verification:
      - kind: integration
        ref: cursor_request_isolation_runs_concurrent_mixed_facts_without_shared_state
        status: pass
    human_judgment: false
completed: 2026-09-07
status: complete
---

# Phase 12 Plan 02 Summary

The Run destination and request-local facts are verified against concurrent outgoing TLS/h2 traffic with synthetic credentials.

## Accomplishments

- Pinned the existing origin/path constants, actual HTTP method/authority/path, protocol headers, client-version presence, and unique paired request IDs.
- Three joined client futures use different model/mode/fast/tool facts. Captured protobuf fields are decoded independently, and each response must echo its own request model.
- The test server owns connection and request tasks using JoinSet; test completion aborts owned work. These are adapter-client tests, not whole-router or live-model availability proofs.

## Verification

- Destination test: 1 passed.
- Request-isolation test: 1 passed.
- Combined Cursor library suite: 229 passed, 0 failed, 1 ignored.
- Format and all-target/all-feature clippy with warnings denied: passed.
- Isolated OPENCODEX_HOME inherited by test trees; production config mtime/SHA and backup inventory unchanged.

## Deviations and Issues

The GLM high-effort executor drafted the harness. The primary completed it after identifying a one-connection accept loop and premature client drop that caused failed observations. Both were corrected; the tests were strengthened rather than relaxed. TLS/h2 was used instead of wiremock because the existing Run request is a paced bidirectional body. The two constants gained adapter-local visibility for tests, with no public export or changed value. The 479-line test implementation lives in a focused included test file instead of enlarging mod.rs. No production behavior, settings, credential writeback, or dependency changed.

## Documentation and Readiness

README, docs, and site surfaces were considered; this slice changes no user-facing behavior or configuration, so no user documentation update is needed. Wiki remains untouched. Plan 03 may now proceed; phase-wide verification remains outstanding.
