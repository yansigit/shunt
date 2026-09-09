---
phase: "16"
status: review-ready
asvs_level: 1
blocking_threshold: high
created: "2026-09-09"
---

# Phase 16 security and provenance review

Plan 16-02 execution evidence; final independent reconciliation remains plan 16-05 work. No live credential contents were inspected.

## Authored boundaries and results

| Boundary | Evidence | Current result |
| --- | --- | --- |
| Translated source attribution | OpenCodex notice now covers Cursor (2026-09-07) and Command Code (2026-09-08), both revision `055c3ecf0de6c35f59195fc434d6b08525182b7f` | Notice regression passes |
| Pre-existing jcode derivative | Independent Omen/high research verified MIT copyright and original LICENSE at `e65e47c31af2ab79346458ff1511bea533930b59`, inspected 2026-09-09 | Full notice reproduced; historical copy revision explicitly unknown |
| Secret-bearing release metadata | `release_security_ledger_rejects_sensitive_fields`; nested secret-field and bearer-marker mutation cases | Pass; bounded heuristic scan, not proof that every possible secret format is detectable |
| Destination spoofing | `credential_boundary_release_canonical_subscription_config`: canonical HTTPS roots accepted; wrong scheme, suffix host, port, path, query, userinfo rejected | Eight cases pass without credential lookup or network |
| Source-file integrity | Existing `auth::command_code` tests exercise synthetic files, hashes/metadata, invalid sources, precedence, FIFO rejection, and cap−1/cap/cap+1 | Seven selected tests pass |
| Refresh/writeback | Inspected `src/auth/command_code.rs`: bounded read-only file resolver, canonical destination validation before resolution, no whoami/refresh/writeback | No change to this implementation; behavioral file tests pass |
| Test state containment | Every stateful command uses `/tmp/shunt-phase16-serialized-run.cjs` around the verified fresh-home wrapper | Production config mtime/SHA-256 and backup/invalid inventory unchanged after every run |
| Scope | Phase 16 diff against `44721a6` has no changes in `src/`, `Cargo.toml`, `Cargo.lock`, or `wiki/` | No runtime, dependency, provider semantics, or credential writeback change in this phase |

## Test transcript

All commands prefixed with `node /tmp/shunt-phase16-serialized-run.cjs`:

1. RED: `cargo test --all-features --test release_security release_security -- --test-threads=1`: 2 selected, 1 pass, 1 intended notice assertion failure, 1 filtered; exit 101. Compiled successfully.
2. GREEN: `cargo test --all-features --test release_security -- --test-threads=1`: 3 selected, 3 pass.
3. `cargo test --all-features --lib command_code_matrix_auth_shapes_file_invariance_and_byte_bounds -- --test-threads=1`: 1 selected, 1 pass, 2,244 filtered.
4. `cargo test --all-features --lib auth::command_code -- --test-threads=1`: 7 selected, 7 pass, 2,238 filtered.

Static notice checks do not substitute for behavioral credential tests. Permanent tests read durable docs and use pure config validation; they never read active planning, the external wrapper, git status, or credential stores.

## Scope distinctions

Fixed v2 milestone baseline: `e029ae2fb35eee9149855769a779343d7139d209` (milestone start). Its manifest diff contains three previously introduced **dev dependencies** (`h2`, `rcgen`, `tokio-rustls`) for hermetic transport/TLS tests, not new Phase 16 runtime dependencies. The milestone's Antigravity catalog and existing OAuth paths must not be mislabeled as newly forbidden generalized subsystems or writeback changes. No generated wiki changes appear in that milestone comparison. Final source/security review must assess the approved milestone changes separately from the zero-runtime-change Phase 16 diff.

The wrapper's live-home access is limited to the user-required raw config fingerprint and filename inventory tripwire, not configuration parsing or testing. Ordinary test process trees receive a fresh `OPENCODEX_HOME`; port 10100 is never a target. Source credentials remain uninspected and unused. Current smoke count is zero.

## Flagged assumptions retained for final review

- REL-02/unclassified: final independent review must reconcile every source/revision/date/sanitization mapping, including shared-adapter evidence scope. A passing schema validator alone cannot resolve it.
- REL-03/unclassified: final independent review must confirm the completed notice scope. Original-source research is available in `16-PROVENANCE-AUDIT.md`; the historical jcode copy revision remains unknown and is not invented.

No security risk waiver is used to turn an unknown result into a pass. Final release sign-off, complete milestone scope scans, and final automated gates remain pending.
