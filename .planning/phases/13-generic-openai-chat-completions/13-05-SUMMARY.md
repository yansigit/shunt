---
phase: 13-generic-openai-chat-completions
plan: "05"
subsystem: api
tags: [conformance, cancellation, docs, locales, bounds, failover]
requires:
  - phase: 13-04
    provides: Bounded indexed tools and ConnectOnly replay protections
provides:
  - Five real-transport cancellation ownership fixtures
  - Complete scenario matrix and named byte-cap triples
  - Locale-complete Chat documentation and verified CLI examples
  - Capability filtering aligned with the completed request whitelist
affects: [14, 15, 16]
tech-stack:
  added: []
  patterns: [bounded TCP ownership proof, explicit boundary triples]
key-files:
  created:
    - tests/openai_chat_conformance/lifetime.rs
    - tests/openai_chat_conformance/matrix.rs
    - tests/openai_chat_translate/caps.rs
    - docs/openai-chat-translation.md
    - site/src/content/docs/providers/openai-chat.md
    - site/src/content/docs/ko/providers/openai-chat.md
    - site/src/content/docs/ja/providers/openai-chat.md
    - site/src/content/docs/zh-cn/providers/openai-chat.md
  modified:
    - src/proxy/capability.rs
    - tests/openai_chat_conformance.rs
    - tests/openai_chat_translate.rs
    - README.md
    - README.ko.md
    - README.ja.md
    - README.zh-CN.md
    - site/src/lib/i18n.ts
    - site/src/content/docs/guides/providers.mdx
    - site/src/content/docs/ko/guides/providers.mdx
    - site/src/content/docs/ja/guides/providers.mdx
    - site/src/content/docs/zh-cn/guides/providers.mdx
    - site/src/content/docs/reference/configuration.md
    - site/src/content/docs/ko/reference/configuration.md
    - site/src/content/docs/ja/reference/configuration.md
    - site/src/content/docs/zh-cn/reference/configuration.md
requirements-completed: [CHAT-08, CHAT-09]
coverage:
  - id: CHAT-CANCELLATION
    requirement: CHAT-09
    description: Upstream closure and admission reclamation before headers, unary body, streamed text and tools
    verification:
      - kind: integration
        ref: tests/openai_chat_conformance/lifetime.rs
        status: pass
    human_judgment: false
  - id: CHAT-BOUNDARIES
    requirement: CHAT-09
    description: Named event, residual, aggregate and unary byte triples plus existing tool bounds
    verification:
      - kind: unit
        ref: tests/openai_chat_translate/caps.rs
        status: pass
      - kind: integration
        ref: tests/openai_chat_conformance/matrix.rs
        status: pass
    human_judgment: false
---

# Plan 13-05: Conformance and documentation closure

Completed the 12-scenario AI-SPEC matrix, five cancellation ownership checks,
and all named byte-bound triples. New regression fixtures passed existing
behavior; no fabricated RED failure or unnecessary production fix was used.
The matrix and exact commands are in `13-05-EVIDENCE.md`.

Documentation ships across all four READMEs, the English engineering note,
four provider pages, four configuration references, four provider guides, and
the shared sidebar. The original `.mdx` guide filenames are preserved. Wiki
is untouched. Published guarantees are restricted to the supported subset and
synthetic fixtures, with no live-provider or Computer UI success claim.

Independent code review returned one alleged blocker and two info notes. The
blocker was disproved by actual buffered-tool control flow and a new passing
first-event/exactly-one-start assertion. Info notes are now documented in every
provider locale. Root found and fixed the separate stale capability branch:
incompatible Chat fallbacks are excluded before dispatch, while the primary
and supported tools/images remain eligible.

## Verification

- Full warnings-denied workspace: **2,875 passed, 0 failed, 2 existing ignored**.
- Chat suites: **47 conformance + 116 translation**, all passing.
- Format check and Clippy with `-D warnings`: pass.
- Documentation contract: 1/1 pass; shared TOML example: CLI `config ok`.
- Astro: 165 pages built; all four generated Chat routes verified.
- Chat CLI/curl smoke: JSON, SSE, malformed model/config, endpoint/model/key
  binding pass, with exactly two mock upstream requests and owned teardown.
- GSD artifacts: 25/25; key links: 7/7; decisions: 11/11.
- Every stateful check used fresh isolated OPENCODEX_HOME and non-10100 ports;
  production config mtime/SHA and backup inventory stayed unchanged.

## Commits and deviations

- `09de192`: cancellation and byte-cap regressions.
- `7e91f8f`: documentation RED contract and persisted evidence.
- `733fbc5`: actual capability RED test and evidence.
- `00f1ce1`: capability GREEN fix and tool-start ordering assertion.

Both new RED gates were persisted and checked before their GREEN changes.
The earlier plan 13-04 gate-timing deviation remains disclosed in its summary.
The corrected `tool_assembly` key-link pattern reflects the shipped symbol;
it does not weaken the intended relationship. User configuration dirt was
preserved. Phase-level goal verification is separate from this plan summary
and must finish before advancing the roadmap.
