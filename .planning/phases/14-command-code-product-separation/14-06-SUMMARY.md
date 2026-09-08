---
phase: 14-command-code-product-separation
plan: "06"
subsystem: verification
status: complete
completed: 2026-09-08
requirements-completed: [CCK-02, CCK-03, CCS-08]
key-files:
  created:
    - src/adapters/command_code/router_tests/products.rs
    - src/adapters/command_code/router_tests/matrix.rs
    - src/auth/command_code/tests/matrix.rs
    - docs/m16-command-code.md
    - site/src/content/docs/providers/command-code.md
    - site/src/content/docs/ko/providers/command-code.md
    - site/src/content/docs/ja/providers/command-code.md
    - site/src/content/docs/zh-cn/providers/command-code.md
  modified:
    - src/adapters/command_code/router_tests.rs
    - src/auth/command_code.rs
    - README.md
    - README.ko.md
    - README.ja.md
    - README.zh-CN.md
    - site/src/lib/i18n.ts
    - site/src/content/docs/guides/providers.mdx
    - site/src/content/docs/ko/guides/providers.mdx
    - site/src/content/docs/ja/guides/providers.mdx
    - site/src/content/docs/zh-cn/guides/providers.mdx
    - site/src/content/docs/providers/openai-chat.md
    - site/src/content/docs/ko/providers/openai-chat.md
    - site/src/content/docs/ja/providers/openai-chat.md
    - site/src/content/docs/zh-cn/providers/openai-chat.md
    - site/src/content/docs/reference/configuration.md
    - site/src/content/docs/ko/reference/configuration.md
    - site/src/content/docs/ja/reference/configuration.md
    - site/src/content/docs/zh-cn/reference/configuration.md
---

# Plan 14-06 — Scenario matrix, docs and smokes

## Delivered

4604759 adds real canonical TLS positive plaintext subagent, continuation and
32-tool histories in both modes, with captured native tool-result identity and
content assertions. Request bytes below/at 8,192 are accepted; above is 413
before any credential lookup or trap connection. Failed finishes retain usage;
401/403/429/500 error bodies are neutral. Private auth fixtures cover error shape,
missing/malformed/empty/null file keys, explicit-token failure without fallback,
file byte limits below/at/above 16 KiB, and SHA/mtime/inventory invariance. All four
matrix tests passed on their first executable runs: characterization, not RED.

8632b03 documents both distinct products across all four READMEs, a shipped
milestone note, provider/reference/guide pages and sidebar in en/ko/ja/zh-cn.
The generic Chat page's obsolete no-preset wording was corrected in all locales.
All ten exact model/effort rows, strict source precedence, canonical destination,
limits, read-only credentials, no version override and live-evidence caveat are
documented. Wiki was not touched. Full pinned MIT notice was rechecked against
OpenCodex LICENSE byte content and linked from docs/m16-command-code.md.

## Observed verification

- Final format, all-targets/all-features Clippy with -D warnings, build, then
  RUSTFLAGS=-D warnings all-features workspace suite passed: **2,942 passed,
  0 failed, 2 existing ignored**, 31 result groups.
- `npm --prefix site run build` passed: 169 pages, four languages, 165 indexed
  pages. Existing Vite deprecated-option and Pagefind stemming notices are
  warnings, not new failures. Built provider/reference/guide pages in every
  locale contain both correct env names, auth mode and local provider links.
  Source model tables are identical across locales. New links are fragment-free.
- Both isolated ordinary-binary CLI/curl smokes passed: API-key check, unary,
  SSE terminal, malformed model, exact path/model/key isolation (two requests);
  subscription check and explicit-empty-token 401 with zero proxy/upstream
  connections. Scripts: /tmp/shunt-commandcode-smoke-euQkwz/smoke.cjs and
  /tmp/shunt-subscription-smoke-agIzmL/smoke.cjs. They used current debug binary.
- All stateful commands used /tmp/shunt-phase12-isolated-run.cjs; production
  OpenCodex config mtime/SHA and backup inventory remained unchanged. Owned
  smoke listeners on 31981/31982 were verified stopped.
- Post-wave schema/UI gates block:false. Codebase drift remains non-blocking
  warn, 171 elements, spawn_mapper:false. Execute-post hooks currently contain
  code-review only: no active TDD checkpoint; settings were not changed.

## Honest limits

Final audit added `command_code_matrix_two_products_concurrent_no_crossover`:
both products are in flight through one gateway before either backend responds.
Captured requests prove separate endpoints, model mappings, envelopes and bearer
credentials; subscription-only headers never enter the API request. The focused
test passed (one selected test), followed by the full repeated gate above.
This closes the previously pending CCK-02 runtime isolation proof.

Native Computer controls were not available. Available Camoufox rejected the
isolated localhost docs preview under its private-IP policy. No bypass was
attempted and no visual pass is claimed. Owned preview PID 12162 on 31985 was
stopped and absence of all three listeners was verified. Static HTML checks are
not GUI evidence. Live provider/minimal-envelope/version acceptance remains the
Phase 16 opt-in gate, not inferred from these tests.

All six implementation plans are now executed. Phase-level security, code-review,
Nyquist and goal verification still control phase closure. The 14-05 behavioral
RED process gap remains recorded in its execution notes; no retrospective RED
or independent review result is fabricated.
