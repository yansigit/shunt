# Phase 16 documentation review

Plan 16-03 completed on 2026-09-09 UTC. Static evidence only; Computer visual acceptance remains assigned to plan 16-05.

## Scope and results

Removed redundant empty-admission wording from English and maintained Korean, Japanese, and Simplified Chinese mirrors. All four navigation labels now read `OpenCode Go`. Zero admitted models, exact environment/endpoint tokens, candidate status, and conditional session semantics remain unchanged.

README: all four maintained files updated. Site: four provider pages, three translated configuration pages, and navigation labels updated. English configuration and all four guides were inspected and needed no wording changes. The engineering note `docs/opencode-go-evidence-gate.md` was considered and remains correct without edits. Generated `wiki/` was not modified.

Every command below ran under `node /tmp/shunt-phase16-serialized-run.cjs`, which wraps fresh isolated state and checks production config mtime/SHA-256 and backup/invalid-file inventory before and after execution. Every check reported unchanged production state.

| Check | Observed result |
| --- | --- |
| `cargo test --all-features --test opencode_go_docs opencode_go_docs -- --test-threads=1` before fixes | 20 selected; 18 passed, 2 assertion failures (navigation and plain wording) |
| Same focused suite after fixes | 20 passed, 0 failed |
| Reinsert README tautology; exact `opencode_go_docs_plain_empty_wording` test | 1 selected, 1 intended assertion failure, 19 filtered |
| Restore fix and rerun focused suite | 20 passed, 0 failed |
| `rustfmt --edition 2021 tests/opencode_go_docs.rs` | Exit 0 |
| `npm --prefix site run build` | Exit 0; 173 pages generated, 169 indexed across four languages |
| `node /tmp/shunt-phase16-doc-links.cjs` | 12 built pages passed; 260 local fragment references resolve |

The new wording regression test preserves existing semantic assertions. RED logs are retained in `16-03-RED.json` and `16-03-RED-TAP.json`. GSD's RED parser did not recognize Cargo output; mechanical TAP normalization retained the raw output and checked the 18/2 counts. The normalized record passed `tdd-red-evidence`; no test failure was bypassed.

## Built navigation and native anchors

The checked pages are each locale's `providers/opencode-go/`, `guides/providers/`, and `reference/configuration/`. Each rendered page has two responsive sidebar instances (desktop and mobile), each containing exactly one `OpenCode Go` link to its own locale route. All guide-to-provider links resolve to the corresponding locale.

| Locale | Provider admission heading ID read from built HTML | Guide/configuration Go heading ID | Local fragments checked across three pages |
| --- | --- | --- | --- |
| English | `opencode-go-is-not-supported-today` | `opencode-go-evidence-gated-zero-support` | 2 + 14 + 69 |
| Korean | `opencode-go는-현재-지원되지-않습니다` | `opencode-go-증거-게이트-지원-없음` | 2 + 12 + 45 |
| Japanese | `opencode-go-は現在サポートされていません` | `opencode-go証拠ゲートサポートなし` | 2 + 12 + 44 |
| Simplified Chinese | `opencode-go-目前不受支持` | `opencode-go证据门控不支持` | 2 + 12 + 44 |

The English provider's promotion heading is `future-promotion-contract`. IDs above were read from `site/dist`, not inferred from translated source headings.

## Limitations

Static HTML checks do not establish visual quality. Computer evaluation remains pending. Existing build warnings concern Vite's deprecated `optimizeDeps.esbuildOptions` and Pagefind's unsupported CJK stemming; the build succeeded without dependency or configuration changes. Full workspace, clippy, and format release gates remain plan 16-05 responsibilities.
