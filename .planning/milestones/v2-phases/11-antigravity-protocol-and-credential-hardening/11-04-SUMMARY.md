---
phase: 11-antigravity-protocol-and-credential-hardening
plan: "04"
status: complete
---

# Plan 11-04 Summary

Correction: the initial checks below did not prove stable follow-up sessions or
native scoped tool-ID validation. Follow-up implementation now uses v2 context
tags and account plus canonical opening-turn sessions. Five new focused tests
pass, including two-round history and streaming/unary parallel-call parity.
The native real-router scoped-tool round-trip now passes in streaming and unary
modes, with pre-dispatch rejection of changed accounts, opening turns, arguments,
orphan results, and duplicate results. Full gates pass after the router fixture
addition. Independent GLM review completed; its numeric-representation finding
was fixed with exact integral-float normalization and regression coverage that
preserves distinct large integers. Account-switch rejection was confirmed as
the intended scoped-history behavior. The user-approved
stateless design cannot reject deliberately recomputed tags or establish
cryptographic signature provenance; see `docs/antigravity-tool-identities.md`.

- Added an opaque account/conversation-scoped session helper using a bounded, domain-separated digest; native envelopes now use it while preserving the exact agent field set and fresh UUID request IDs.
- Added native exact-admission/session focused checks and signature encoding coverage without exposing or persisting private identity material.

Current uncommitted follow-up verification (2026-09-07):

- `cargo fmt --all --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all-features --workspace -- --test-threads=1`: 2,625 passed,
  two ignored, no failures across 26 result groups.
- Five scoped-tool tests, the real-router round-trip/rejection test, and all 27
  Gemini conformance tests passed separately.
- Fixed stale catalog-cache fixtures from the earlier account-fingerprint API
  change, preserving account-isolation, stale-cache, and eviction assertions.
- Provider site pages updated in all four maintained languages; engineering
  note added. README surfaces reviewed: no setup/capability change there.
  Generated wiki left untouched. Site render not run (dependencies absent).
- Requested Muse Spark high-effort review failed before completion with HTTP
  429; Omen then failed with provider error 1210 despite a high-effort request.
  `opencode-go/glm-5.3-flash` then completed the high-effort review, verifying
  ordinal symmetry, key canonicalization, and streaming/unary parity. Its
  numeric replay finding is fixed and its unused decoder parameter nit removed.

Original partial verification (not evidence of completed scope binding):

- `cargo test --all-features antigravity_native_envelope --lib` (1 passed)
- `cargo test --all-features antigravity_native_tool_signature --lib` (1 passed)
- `cargo fmt --all` (passed)
- `cargo clippy --all-targets --all-features -- -D warnings` (passed)

Commit: `33ad017 feat(11-04): scope Antigravity sessions and signatures`
