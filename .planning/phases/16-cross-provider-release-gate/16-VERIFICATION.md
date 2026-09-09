---
phase: 16-cross-provider-release-gate
verified: 2026-09-09
status: pending
score: independent review pending
---

# Phase 16 release verification

Automated and owned-local visual gates passed. Independent provenance,
security and code-review reconciliation is still pending; this is not phase
completion or permission to merge.

## Current gate transcript

Executed from the dedicated worktree using separate serialized isolation-wrapper
invocations. Every child inherited fresh OPENCODEX_HOME; production config
mtime/SHA-256 and backup/invalid filename inventories were unchanged.
See 16-FINAL-GATES.json for exact focused and final Rust command outputs.

| Gate | Actual result |
|---|---|
| release_matrix / release_security | 3 + 3 pass |
| Gemini conformance | 29 pass |
| OpenAI Chat conformance / translation | 47 + 116 pass |
| Command Code API / subscription integration / translation | 8 + 1 + 23 pass |
| Command Code crate filter | 35 pass |
| Cursor crate filter | 285 pass, 1 existing ignored |
| Codex WS fallback / failover / inbound HTTP / inbound WS / retry | 9 + 24 + 40 + 12 + 8 pass |
| Go docs / evidence / owned CLI rejection smoke | 20 + 2 + 1 pass |
| Final format | exit 0 |
| Final warnings-denied all-target/all-feature Clippy | exit 0 |
| Final warnings-denied all-feature workspace suite | 2,979 pass, 0 fail, 2 existing ignored |
| Site build | 173 pages, 4.17s; 169 search-indexed in four languages |
| Built local fragments / navigation | 12 pages, 260 fragment links pass |
| Computer | Actual final screenshot plus unchanged-source four-locale/mobile observations; 16-UI-REVIEW.md |
| Live provider smoke | All skipped; 0/8 attempts, US$0 planned; 16-SMOKE.md |
| Phase scope | No changes since 44721a6 in src/, Cargo.toml, Cargo.lock or wiki/ |
| Milestone dependencies | Only previously approved dev dependencies h2, rcgen, tokio-rustls; no new runtime dependency |
| Excluded implementation markers | No aistudioweb, SAPISIDHASH or MakerSuite match in src/ or tests/ |
| Diff whitespace | git diff --check passed |

## Requirement coverage

| Requirement | Source plans | Evidence | Status |
|---|---|---|---|
| REL-01 | 01,05 | Durable 45 exact documented model rows + six unbound contracts, nine scenario dispositions each; nonzero focused suites | Await independent mapping review |
| REL-02 | 01,02,05 | Source repo/revision/date/sanitization fields, mutation tests, explicit capture/live none | Await independent provenance review |
| REL-03 | 02,05 | Full OpenCodex and jcode MIT notices; independent original-source research; historical jcode copy revision explicitly unknown | Await final notice review |
| REL-04 | 04,05 | Complete bounded preflight dispositions; no credential contents read, refresh, retry or generation | Passed skip-capable contract, not live compatibility |
| REL-05 | 03,05 | Four README/provider locale mirrors, configuration/navigation regression, site and Computer | Passed |
| REL-06 | 02,05 | Separate fmt, Clippy, full suite, scope, security and code-review gates | Independent review pending |

## Evidence limits and deviations

Shared adapter/auth synthetic tests prove Shunt contracts, not model availability
or subscription entitlement. No captures or live success are claimed. Go has
zero admitted tuples; future captured/live promotion evidence remains required.

Full workspace regression ran at final phase verification, not after every
intermediate wave as initially proposed. Every task had focused feedback; no
historical wave-level full passes are invented. Final all-feature regression
passed twice, including after the only late test path change.

Plan 05 additionally fixed an archival hazard: Go evidence tests previously read
active .planning inputs. Commit d45a1c0 preserves both inputs byte-for-byte in
tests/fixtures/opencode-go and changes only lookup paths. cmp passed for both;
all assertions and unresolved evidence statuses remain intact. Focused tests
passed 2/2, followed by the final format/Clippy/full-suite gates above. README,
engineering docs and site were considered: this internal test-retention change
has no observable behavior/configuration impact; fixture README documents it.
Wiki remains untouched.

Existing Vite deprecation and Pagefind CJK stemming notices are nonfatal build
warnings, not failures. Owned Computer server and tab were closed explicitly.
No production configuration, credential writeback, runtime provider semantics,
purchase, login or deployment changed.
