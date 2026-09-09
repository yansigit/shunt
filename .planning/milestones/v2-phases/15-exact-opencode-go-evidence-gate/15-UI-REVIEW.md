# Phase 15 — UI Review

**Audited:** 2026-09-08
**Baseline:** No UI-SPEC.md exists; abstract 6-pillar standards, scoped to the Phase 15 documentation/locales/navigation surfaces only (no UI design changes, no UI-SPEC authored).
**Screenshots:** Not captured. Code-only audit per authorization; no dev server probed, no Computer use, no network screenshot tooling. Rendered layout is therefore **not visually verified**; findings below are grounded in source files and the prebuilt `site/dist` HTML (173 pages; 12 built-page checks previously passed).
**Scope:** Worktree `/Users/user/.codex/worktrees/0466/shunt` only. No commits made; no files outside this artifact were written (the skill's mandatory `.planning/ui-reviews/.gitignore` gate was executed).

---

## Pillar Scores

| Pillar | Score | Key Finding |
|--------|-------|-------------|
| 1. Copywriting | 3/4 | Precise zero-support contract everywhere; a few redundant "empty set is empty" phrasings and language-suffixed nav labels |
| 2. Visuals | 3/4 (static structure only — not visually verified) | Heading hierarchy, TOC anchors, and sidebar placement are sound in built HTML; rendered appearance unverified by policy |
| 3. Color | N/A (inapplicable) | No styling, theme, or color surface touched by this phase |
| 4. Typography | N/A (inapplicable) | No font/typography surface touched; content uses inherited site theme |
| 5. Spacing | N/A (inapplicable) | No layout/spacing surface touched; markdown-only content |
| 6. Experience Design | 4/4 | Nav entry discoverable in all four locales; every dist cross-link and anchor resolves; llms.txt/sitemap include the new page |

**Overall: 10/12 applicable points** (3 + 3 + 4 across the three applicable pillars; Color/Typography/Spacing are inapplicable to this docs-only phase and are excluded from the denominator — do not report a /24 total).

---

## Top 3 Priority Fixes

1. **Redundant emptiness phrasing (WARNING, copywriting)** — `README.md:304` reads "the empty admitted set is empty"; `site/src/content/docs/providers/opencode-go.md:3` reads "The admitted OpenCode Go set is empty (the empty admitted set)"; the ko/ja/zh-cn mirrors repeat the pattern as parenthetical glosses ("비어 있으며(빈 허용 집합)", "空（空の許可集合）", "为空（空准入集合）"). Impact: readers hit a tautology that looks like an editing artifact on the highest-traffic surfaces. Fix: keep one plain statement ("the admitted set is empty") in English and drop the parenthetical gloss in the three locale mirrors; `tests/opencode_go_docs.rs` asserts exact tokens, so coordinate the retained token before editing.
2. **Language-suffixed nav labels for OpenCode Go (WARNING, navigation consistency)** — `site/src/lib/i18n.ts` labels the item "OpenCode Go (한국어)" / "OpenCode Go (日本語)" / "OpenCode Go (简体中文)" while every other brand-name provider entry (Command Code, Antigravity, Cursor, Kimi, …) keeps its label untranslated across locales. Impact: the sidebar already renders in the reader's locale, so the suffix is noise and the entry breaks the established labeling pattern. Fix: use plain "OpenCode Go" in all locales, matching neighboring rows.
3. **Close the visual-verification gap (WARNING, process)** — rendered layout of the four new provider pages, the nav position in the locale sidebars, and the configuration/guide sections were never visually confirmed (static audit only, per constraint). Impact: static HTML cannot prove focal hierarchy or responsive sidebar rendering. Fix: run one authorized browser/screenshot pass against the built `site/dist` (or a local server on a non-10100 port, never production OpenCodex state) before Phase 16 release acceptance.

---

## Detailed Findings

### Pillar 1: Copywriting (3/4)

**Evidence (grep + full read of all 16 content surfaces):**

*Contract parity — verified verbatim in every surface's Go section:*

- `SHUNT_OPENCODE_GO_API_KEY` present in README.md:307, README.ko.md:297, README.ja.md:300, README.zh-CN.md:293, providers/opencode-go.md (en/ko/ja/zh-cn), reference/configuration.md (en:498; ko/ja/zh-cn), guides/providers.mdx (all four), docs/opencode-go-evidence-gate.md.
- Canonical `https://opencode.ai/zen/go/v1` present in the same set.
- All four candidate ids (`glm-5.3-flash`, `omen-alpha`, `muse-spark-1.3-contributor`, `deepseek-v4-flash`) appear in README/provider/configuration/note surfaces; guides use the provider-page link instead of repeating the ledger (per plan 06 — acceptable).
- Zero-support wording present in every section in the native language; no positive-support claim token found in any new Go section (all "supported"/"live-verified" occurrences are negated).
- Dist confirmation: `SHUNT_OPENCODE_GO_API_KEY` present exactly once in each built configuration page (en, ko, ja, zh-cn).

*Defects:*

- README.md:304 — "the empty admitted set is empty" (tautology).
- providers/opencode-go.md:3 (en) — "(the empty admitted set)" gloss; mirrored as parentheticals in ko/ja/zh-cn READMEs/configuration and ja/zh-cn provider pages.
- Nav labels carry language suffixes (fix 2).
- Minor: ko/ja/zh-cn prose leaves "hermetic", "wire", "effort", "capability", "conversation-scoped", "header" as inline English loanwords (ko configuration:429, ja:400, zh-cn:400). This preserves the exact-token contract demanded by plan 05 — a deliberate tradeoff, but it hurts first-read fluency in those locales.

### Pillar 2: Visuals (3/4 — static structure only, **not visually verified**)

*Code/built-HTML evidence:*

- Provider pages use a single H1 focal statement ("OpenCode Go is not supported today" and native equivalents), an H2 "future promotion" section, and a closing cross-reference — clean hierarchy in all four locales.
- Built TOC anchors match heading ids in every locale (`#opencode-go-증거-게이트-지원-없음`, `#opencode-go証拠ゲートサポートなし`, `#opencode-go证据门控不支持`, `#opencode-go-evidence-gated-zero-support`); the en page's `#future-promotion-contract` TOC anchor resolves in dist.
- Nav entry sits last in the Providers group (additive placement) and is present in the dist sidebars of all four locales (verified via href extraction).
- Guides sections are H2; id `opencode-go-evidence-gated-zero-support` present in the built en guides page.

*Why not higher:* rendered appearance (spacing, responsive sidebar behavior, mobile layout) could not be inspected under the no-server/no-Computer constraint. No invented fail/pass is asserted for layout.

### Pillar 3: Color — N/A (inapplicable)

No CSS, theme, or color-bearing markup was added or modified in this phase (documentation and i18n nav only); the color-audit method has no target surface here.

### Pillar 4: Typography — N/A (inapplicable)

No font sizes, weights, or typography-bearing components were added; content renders through the unchanged site theme.

### Pillar 5: Spacing — N/A (inapplicable)

No spacing classes or layout containers were added or modified.

### Pillar 6: Experience Design (4/4)

*Evidence:*

- **Discoverability:** NAVIGATION entry with embedded ko/ja/zh-cn translations in `site/src/lib/i18n.ts` (single source consumed by `buildLocaleSidebar`); guarded by `opencode_go_docs_nav_i18n` (tests/opencode_go_docs.rs:260).
- **Link integrity in the built site:** all four provider pages exist in dist; locale guides link their locale provider page and all resolve; the en provider page's `/reference/configuration/` link resolves, and all four built configuration pages exist. No invented English fragment anchors found on locale pages (all in-page hrefs checked; TOC anchors are native and match built ids, satisfying the AGENTS.md anchor rule).
- **Machine-readable surfaces:** providers/llms.txt indexes the new page (as do ko/ja/zh-cn llms.txt); `llms-full.txt` contains 12 Go mentions; `sitemap-0.xml` includes it.
- **Executable regression net:** 19 named per-file tests in `tests/opencode_go_docs.rs` (fns verified at lines 32–407), with RED/mutation/GREEN evidence recorded in the 15-03/15-05/15-06 summaries.
- **Error-path documentation:** the docs consistently describe the gateway-owned Anthropic-shape 400 rejection users will see, matching the implemented smoke evidence (15-04).

*No deduction:* no interactive state (loading/empty/error UI) exists in this change's surface, and the documented failure path is synchronized with implementation. The nav-label blemish is scored under Copywriting.

---

## Registry Safety

Skipped: no `components.json` exists and no UI-SPEC registry table exists; no third-party UI registry blocks were installed in this phase.

---

## Files Audited

- `site/src/content/docs/providers/opencode-go.md` (+ ko, ja, zh-cn mirrors)
- `site/src/content/docs/reference/configuration.md` (+ ko, ja, zh-cn mirrors; Go sections ~494/425/396/396)
- `site/src/content/docs/guides/providers.mdx` (+ ko, ja, zh-cn mirrors)
- `site/src/lib/i18n.ts` (NAVIGATION entry)
- `README.md`, `README.ko.md`, `README.ja.md`, `README.zh-CN.md` (Go sections)
- `docs/opencode-go-evidence-gate.md`
- `tests/opencode_go_docs.rs` (inventory only; not executed per no-stateful-commands constraint)
- Built artifacts: `site/dist/providers/opencode-go/**`, `site/dist/{ko,ja,zh-cn}/providers/opencode-go/index.html`, `site/dist/guides/providers/index.html`, `site/dist/**/reference/configuration/index.html`, `site/dist/**/llms.txt`, `site/dist/sitemap-0.xml`

## Verification Boundaries

- No dev server was started or probed; no port probe of any kind; no Computer use; no network screenshot tool. `site/dist` was reviewed as prebuilt static output only.
- No Cargo or other stateful commands were run; production OpenCodex state (`/Users/user/.opencodex`, port 10100) was not touched.
- Tests were not re-executed; pass/fail claims cite the recorded summaries (15-03: 6 passed; 15-05: 15 passed; 15-06: 19 passed; 15-04 release gates) and the test-file inventory was verified statically.
- Not visually verified: rendered layout, focal hierarchy, responsive behavior. Nothing above should be read as a visual pass.
