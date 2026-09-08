---
phase: 14-command-code-product-separation
status: revised_root_review
reviewer: root-inline-fallback
independent: false
---

# Phase 14 plan review

The host stopped exposing spawn/follow-up tools after the planner returned. This is a root review, not an independent checker pass. Six plans read completely. Deterministic command-path and failure-direction probes returned status ok; wrapped Cargo commands are classified not_applicable by the path probe, not proven executable by it.

## Required corrections before execution

1. BLOCKER — Genuine RED evidence: 14-01 Task 2 asks for a deliberate RED after Task 1 adds the only missing preset, although its wire behavior already exists. Combine the real-router tracer with the initial missing-preset RED or label subsequent tests as characterization. Never fabricate a failure.
2. BLOCKER — Test seam available before tracer: 14-02 uses a canonical HTTPS-only origin but postpones TLS/DNS injection to 14-05. Existing server helper is cfg(test), pub(crate), unavailable to external integration tests. Own a crate-local router test module and reuse the existing client seam in 14-02; preserve canonical URL bytes and origin checks.
3. BLOCKER — Wiring ownership: 14-03 changes adapter mod.rs and capability.rs without listing them; 14-04 changes adapter mod.rs and must declare model/mod.rs without owning them. Explicit ownership and dependency edges must cover these call sites.
4. BLOCKER — Reachable redirect assertions: 14-02 and 14-05 demand zero credential reads for an upstream 302. An authenticated first request necessarily reads a credential before observing a redirect. Invalid configured destinations require zero reads; a redirect requires exactly the initial snapshot and zero redirected requests/new lookups.
5. BLOCKER — Locale routing: 14-06 invents providers/command-code.ko.md and reference/configuration.ko.md paths. Maintained locales live under docs/ko, docs/ja, docs/zh-cn. Include guides/providers.mdx and sidebar i18n where affected; verify built locale pages and anchors, not headings alone.
6. BLOCKER — Honest smoke: 14-06 promises successful subscription curl against a mock using an ordinary candidate binary, without a mechanism preserving HTTPS origin pinning. Specify a concrete isolated harness or limit ordinary-binary subscription smoke to config/auth failures and separately prove successful transport through the crate-local TLS router harness. No public origin bypass or system DNS changes.
7. BLOCKER — Provenance: source-derived facts are called Official facts and cited with a test line attributed to the adapter file. Use source-derived provenance, exact tests/command-code-provider.test.ts:643 reference, and retain explicit live acceptance uncertainty.
8. WARNING — Public spelling: plans alternate command-code and command_code for the new kind, while existing ProviderKind uses snake_case. Keep product preset command-code distinct from kind command_code and auth command_code_oauth, with serialization assertions.
9. WARNING — Capability coverage: COVERAGE claims parallelToolCalls:false is enforced by a preset field/capability rule that does not exist. Explicitly decide the API-key surface rather than claiming implementation; do not broaden generic Chat semantics through hardcoded product handling.
10. WARNING — Validation honesty: Wave 0 ledger checkbox is marked complete before its file exists; leave it pending until executed. Empty explicit env must remain invalid (no file fallback), not absent-equivalent after file support lands.
11. WARNING — Terminal wording: accepting finish-step then finish conflicts with saying every second terminal-class record fails. Define the companion exception explicitly; validate already received bytes before emitting success and never promise future-byte inspection.

All eleven requirement IDs are assigned. Positive subagent/continuation, strict read-only credentials, exact effort arrays, no unsafe replay, and full baseline gates are present. No phase-14 implementation or acceptance pass follows from this review.

## Revision disposition

Root corrected the listed plan defects: genuine missing-preset RED now belongs to Task 1; later checks are characterization; canonical-host TLS routing is owned in 14-02 in a crate-local module; adapter/capability/model wiring ownership is explicit; redirects distinguish initial reads from redirected requests; locale paths, guide/sidebar ownership and built-anchor checks are corrected; ordinary-binary smoke and test-client transport success are labelled separately; source evidence is no longer called official; kind spelling is command_code; nonexistent parallel-tool metadata enforcement is an explicit opt-out; the ledger checkbox is pending and invalid empty env cannot fall back; the supported terminal companion exception is explicit. CLI credential fixtures inject a private temporary path rather than assuming OPENCODEX_HOME changes the CLI home.

Re-run deterministic results: six valid plan structures, zero command-path/failure-direction findings, all 14/14 decisions covered. Structure emits one advisory for the public kind/auth one-way change; the recorded 2026-09-07 user approval already authorizes these additions, so no new approval is inferred. The larger tracer and translated-doc plans are conscious cohesive slices; serialize all Cargo work. Independent review is still unavailable in this host turn and is not claimed.
