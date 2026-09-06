---
phase: 06-capability-aware-fallback
status: ready
created: 2026-09-06
requirements: [CAP-01]
---

# Phase 6 Context

## Goal

Prevent an ordered heterogeneous fallback chain from dispatching a request to a
target whose existing adapter cannot preserve the request's required features.

## Decisions

- D-01: Capability filtering applies only to fallback candidates after the
  declared primary. The primary keeps current behavior and remains authoritative.
- D-02: Derive requirements once from the already bounded, parsed Anthropic
  Messages body. Do not tokenize, fetch URLs, inspect credentials, or mutate it.
- D-03: Detect non-empty caller tools, image source class (base64 or URL),
  `output_config.format`, explicit `output_config.effort`, and a trailing `[1m]`
  client context hint.
- D-04: Use a small internal adapter matrix based on behavior already implemented
  and tested in shunt. Do not add public config keys or a generalized capability
  manifest/evidence platform.
- D-05: Anthropic-native preserves all protocol fields. Responses preserves tools,
  both image classes, and explicit reasoning effort but not Anthropic structured
  output. Gemini/Antigravity preserve tools and base64 images but not URL images,
  structured output, or explicit `output_config.effort`. Cursor preserves tools
  and base64 images but not URL images, structured output, or explicit effort.
  The deprecated CLI target preserves none of these requested capabilities.
- D-06: A `[1m]` hint is a known one-million-token requirement but shunt has no
  trustworthy per-target context metadata. Keep the primary and exclude every
  fallback rather than silently hopping to an unknown window.
- D-07: Malformed or unknown request shapes are not repaired by this phase; normal
  adapter validation remains authoritative. Only clear, supported signals affect
  eligibility.
- D-08: Exclusions happen before inbound-auth chain analysis, credential
  resolution, and network I/O. Emit a warning naming provider/model and bounded
  low-cardinality reasons, plus a `shunt.failover{state=capability_excluded}`
  counter.
- D-09: Preserve existing response-status failover, remembered-error priority,
  body cloning, native inbound Responses routing, and no-mid-stream-hop rules.
- D-10: Observable behavior and all maintained documentation locales change in
  this phase; generated `wiki/` remains untouched.

## Scope boundaries

No public configuration, runtime probing, model catalog, token estimation,
credential writeback, persistent health history, or changes to the primary route.
