---
phase: 11-antigravity-protocol-and-credential-hardening
plan: "03"
status: complete
---

# Plan 11-03 Summary

Implemented request-local Antigravity account affinity and strict catalog admission.

- Added an opaque, versioned, domain-separated SHA-256 fingerprint derived from normalized persisted email, with a separate non-durable refresh-token fallback for legacy stores.
- Carried the fingerprint with the Antigravity credential and keyed catalog caching by account fingerprint plus project, preserving single-flight behavior.
- Added an authoritative fresh-catalog gate before native inference dispatch; catalog misses and stale-only evidence now fail closed without an inference request. Native admission is limited to Gemini keys, so Claude/GPT catalog entries cannot become rewritten inference dispatches.
- Added `antigravity_exact_catalog_admission`, which accepts only fresh, exact catalog-declared model/effort tuples and rejects bare, unknown, ambiguous, unsupported-effort, and heuristic nearest-tier rewrites.
- Updated synthetic credential fixtures and the native catalog integration filter.

Verification:

- `cargo test --all-features --test antigravity_catalog antigravity_native_affinity -- --test-threads=1` (1 passed)
- `cargo fmt --all --check` (passed)
- `cargo clippy --all-targets --all-features -- -D warnings` (passed)

Added a full exact model/effort positive and negative matrix, same-project cross-account cache isolation, same-account multi-project isolation, request-local legacy refresh rotation coverage, and a real-router rejected-tuple fixture asserting zero inference hits. Native real-router lifetime fixtures now prove injected resolution once per request, production-default resolution once per request via a cfg(test)-only counter, distinct account/project/token observations, and prompt cancellation release of held streaming ownership. Seven focused library tests are selected by `antigravity_native_affinity`.

The native-path comments and discovery diagnostics now describe fail-closed admission accurately rather than the removed fail-open heuristic.

Commits: `97ede48 feat(11-03): bind Antigravity account catalog admission`, `ff9f5a2 fix(11-03): enforce exact Antigravity catalog tuples`, `f337037 test(11-03): prove native affinity admission matrix`, `4d333f8 test(11-03): prove native credential affinity lifetime`, `f86298e test(11-03): cover same-account project affinity`
