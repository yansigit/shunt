---
phase: 14-command-code-product-separation
plan: "01"
subsystem: api
status: complete
completed: 2026-09-08
tags: [commandcode, preset, openai-chat, conformance]
requires:
  - phase: 13
    provides: Generic Chat translation and conservative transport
provides:
  - Distinct commandcode API-key preset
  - Eight real-config and real-router conformance tests
affects: [14-02, 14-03, 14-06, 16]
tech-stack:
  added: []
  patterns: [table-driven preset, synthetic loopback transport]
key-files:
  created: [tests/command_code_api_conformance.rs]
  modified: [src/config/presets.rs]
requirements-completed: [CCK-01]
coverage:
  - id: CCK-PRESET
    requirement: CCK-01
    description: Canonical base, API-key env, unchanged sibling presets, real-router text turn
    verification:
      - kind: integration
        ref: tests/command_code_api_conformance.rs#command_code_api_preset_tracer
        status: pass
    human_judgment: false
  - id: CCK-ISOLATION
    requirement: CCK-02
    description: Existing non-admitted pairing rejected; concurrent API providers retain separate destinations and synthetic keys
    verification:
      - kind: integration
        ref: tests/command_code_api_conformance.rs#command_code_api_tracer_isolation
        status: pass
    human_judgment: false
  - id: CCK-SCENARIOS
    requirement: CCK-03
    description: Unary, single-terminal SSE, incremental text, paired history, parallel tool identities, 160KB context, disconnect and capacity recovery, single-attempt auth error
    verification:
      - kind: integration
        ref: tests/command_code_api_conformance.rs
        status: pass
    human_judgment: false
---

# Plan 14-01 — API-key product

Appended `commandcode` to the preset table: `OpenAiChat`, canonical
`https://api.commandcode.ai/provider/v1`, `ApiKey`, and
`SHUNT_COMMANDCODE_API_KEY`. No translator, retry, credential persistence,
existing preset fields, ordering, or configuration override semantics changed.

## Evidence

- Genuine RED: two compiled named assertions failed because the preset was
  missing (exit 101). `14-01-RED.json` contains the explicitly labelled Rust-to-TAP
  transcription; the GSD classifier returned RED_EVIDENCE_OK before the feature edit.
- GREEN: all eight conformance tests passed, including all ten sibling tuples.
- Final gates: format check and warnings-denied Clippy passed; warnings-denied
  full workspace suite: **2,883 passed, 0 failed, 2 existing ignored**, 29 result groups.
- The project's run-shunt/verify skills guided ordinary-binary smoke verification:
  existing `.claude/skills/run-shunt/smoke.sh` passed config, liveness, discovery,
  proxy, malformed-request checks. Its cleanup terminated only its owned processes.
- `/tmp/shunt-commandcode-smoke-euQkwz/smoke.cjs` passed CLI config check,
  curl unary, curl SSE, single terminal, and malformed-model 400. Both captured
  requests used `/provider/v1/chat/completions`, the configured `fixture-chat`
  model, synthetic bearer, no forwarded caller API key, and intact user text.
  The streaming body requested `stream_options.include_usage=true`.
- All commands used `/tmp/shunt-phase12-isolated-run.cjs`; each reported
  production config mtime/SHA and backup inventory unchanged. All four smoke
  ports were confirmed unbound after cleanup. No live provider or GUI pass claimed.

## Commits and deviations

- a8ab98e — genuine missing-preset RED tests and evidence.
- f7aa8be — preset row and ordered-name expectations.
- 6396e57 — conformance commit implements Tasks 2 and 3 together as one
  cohesive fixture harness; no artificial RED was required for existing Chat behavior.
- Independent GLM/high checker cleared 14-01. GLM/high executor prepared the
  approach but was interrupted before any edits when messaging controls disappeared;
  root completed the implementation inline. No model quota error occurred.
- The focused fixture file is slightly above the preferred 500-line guideline;
  it holds one small preset-driven harness and eight cases, with no production growth
  beyond the table row. This is a preference, not a hard gate.

## Remaining phase work

CCK-02 remains pending final two-product runtime isolation after the subscription
variant exists. CCK-03 remains tracked with shared 14-06 final verification.
All README/docs/site English and maintained locale updates are assigned to 14-06
in the same phase/PR; wiki was considered and left untouched. Phase 14 and live
Phase 16 acceptance are **not complete**. Continue with 14-02's protocol ledger
and canonical-origin subscription tracer.
