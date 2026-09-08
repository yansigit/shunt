---
phase: 12-cursor-evidence-backed-hardening
plan: "04"
subsystem: cursor
tags: [admission, tools, images]
requires: [12-03]
provides: [explicit pre-credential input rejection, no silent tool or image drops]
affects: [12-05, 12-08]
tech-stack:
  added: []
  patterns: [pure admission in bounded preparation pool]
key-files:
  created: [src/adapters/cursor/admission.rs]
  modified: [src/adapters/cursor/mod.rs, src/adapters/cursor/router_parity_tests.rs]
requirements-completed: [CUR-04]
completed: 2026-09-07
status: complete
---

# Phase 12 Plan 04 Summary

Malformed or unsupported tool/image/guidance inputs now return an Anthropic-shaped
400 before credential resolution or upstream dispatch. No legacy bridge fallback
is invoked. Tools require unique nonempty names and explicit object schemas;
only automatic tool choice is supported. Images require valid nonempty base64
and explicit PNG/JPEG/GIF/WebP media types; URLs fail explicitly. Arguments must
be objects and arguments/schema nesting must stay below 64 levels.

## Verification

- Red fixture observed the original bug: nameless tool silently dropped, HTTP
  200 instead of 400. The implemented admission changes made the fixture pass.
- Six cursor_admission tests passed. Full-router JSON/SSE fixtures cover ten
  invalid forms with zero dispatch, deliberately missing credentials, and valid
  requests before and after. Additional fixtures cover duplicate names,
  argument representation/depth, media types, and inline/offloaded decode errors.
- Full all-feature workspace tests passed: library 2,169 passed, 2 existing
  ignored; all integration suites passed.
- Format and all-target/all-feature clippy with warnings denied passed.
- Site build passed: 161 pages, maintained locales included.
- Isolated OPENCODEX_HOME throughout; production config mtime/SHA and backup
  inventory unchanged. Latest rebuilt-binary smoke passed immediately before
  this slice (plan 03); phase-wide final smoke remains required.

## Deviations and documentation

Moved the pure validator into a focused private admission.rs rather than growing
mod.rs further. Reused the whole-router fixture file for zero-dispatch proof.
Two characterization tests previously expected missing-schema defaults and
image dropping; they now require rejection while retaining positive byte/name/
schema assertions. No test was removed to hide a failure. The doc-hidden image
benchmark helper now returns Result; benchmark targets compile unchanged.

README, engineering note, and provider pages updated in English/ko/ja/zh-cn.
Generated wiki remains untouched. No new config or credential writeback change.
Plan 05 may proceed; phase-wide CUR-05..08 and milestone completion remain open.
