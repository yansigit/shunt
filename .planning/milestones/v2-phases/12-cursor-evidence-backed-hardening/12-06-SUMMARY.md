---
phase: 12-cursor-evidence-backed-hardening
plan: "06"
subsystem: cursor
tags: [protobuf, framing, validation]
requires: [12-01, 12-05]
provides: [strict active Run decoding, explicit malformed trailer errors]
affects: [12-07, 12-08]
key-files:
  created: [src/adapters/cursor/strict.rs, src/adapters/cursor/protocol_tests.rs]
  modified: [src/adapters/cursor/agent.rs, src/adapters/cursor/connect.rs]
requirements-completed: [CUR-06]
completed: 2026-09-07
status: complete
---

# Phase 12 Plan 06 Summary

The active Run transport decompresses END frames before strict JSON parsing.
Malformed trailers, unsupported flags, invalid protobuf/UTF-8, and malformed
nested MCP arguments error rather than impersonating success or null values.
Empty trailers and valid gzip JSON retain success/authentication semantics.
Well-formed unknown protobuf fields remain compatible; omitted scalar map keys
retain the protobuf empty-key default. Missing Value kinds are errors.

The existing 64 MiB frame/decompression limit is unchanged. Argument allocations
use that aggregate budget (conservative escaping/node accounting) and depth 64.
Large decoding reuses the bounded response-work pool; no new runtime or state.
Old coercing argument helpers remain test-only for preserved characterization
tests. Active decoding uses fallible wire.rs, not the old permissive iterator.

## Verification

- Red observations: truncated END JSON returned End, and a malformed nested
  argument emitted a tool call with path:null. Both now explicitly error.
- Framing table: empty/object/gzip-object pass; gzip authentication retains 401;
  malformed JSON/error fields, corrupt gzip, unsupported flags fail once.
- Six strict tests passed: four malformed wire classes, unknown fields, typed
  values, default empty map key, invalid UTF-8, multiple kinds, non-finite values,
  duplicate keys, depth/aggregate cap+1, and large offloaded/inline parity.
- Full all-feature workspace passed: library 2,182 passed, 2 existing ignored,
  all integration suites passed. Subsequent default-key/offload additions passed
  focused tests and all-target clippy with warnings denied. Format passed.
- Rebuilt binary smoke: all five checks passed on 31711/31712. Owned processes
  and smoke scratch were cleaned up. No live Cursor inference claimed.
- Four-language site: 161 pages built. Production OpenCodex config mtime/SHA
  and invalid/backup inventory unchanged for all isolated command trees.
- Independent Muse Spark high-effort review found no concrete regression;
  conservative memory accounting and unknown-flag rejection are intentional.

## Deviations and documentation

Focused strict.rs/protocol_tests.rs keep new parsing out of the oversized agent
module. A test-only clippy allowance follows existing cross-runtime observer
serialization; no test assertion was weakened. Engineering and all four Cursor
provider pages document strict errors. README overview already describes error
semantics; final phase docs parity remains plan 08. Generated wiki untouched.
No configuration, credentials, endpoint, retry machinery, or heuristic changed.
