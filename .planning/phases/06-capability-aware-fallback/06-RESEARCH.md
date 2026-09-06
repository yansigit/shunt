---
phase: 06-capability-aware-fallback
status: complete
created: 2026-09-06
---

# Phase 6 Research

## Existing shunt seam

`proxy::failover::forward` parses and normalizes the bounded request once, resolves
the ordered route chain, then performs inbound-auth analysis and sequential
dispatch. Filtering immediately after `resolve_request_chain_value` avoids extra
parsing and guarantees excluded candidates cannot affect auth or reach credentials
or the network. The existing `record_failover` metric already accepts a fixed
low-cardinality state label.

## Evidence already in the codebase

- Responses translation has fixtures for caller tools, base64 and URL images, and
  `output_config.effort`; it does not map Anthropic `output_config.format`.
- Gemini and native Antigravity translate function tools and base64 images. Gemini
  explicitly rejects URL image sources and consumes Anthropic `thinking`, not the
  newer explicit effort field.
- Cursor sends function schemas and selected base64 images but renders URL images
  as text placeholders and has no explicit effort/structured-output mapping.
- The deprecated Antigravity CLI explicitly rejects caller tools and omits images.
- Anthropic routes preserve the Messages protocol rather than translating it.
- `[1m]` is currently only a client-side hint stripped for route/model forwarding;
  no trusted per-provider context-window metadata exists.

## OpenCodex adaptation

OpenCodex evaluates request requirements against a generalized compatibility
manifest. Shunt needs only the useful dispatch invariant, not that platform. A
small request-feature summary plus an internal adapter matrix fits the existing
static router, is independently testable, and cannot drift into runtime scoring.

## Risks

- Filtering the primary would be a breaking policy decision; keep it unchanged.
- Treating unknown model capacity as compatible after `[1m]` would recreate the
  context-loss failure; stop fallback for that clear requirement.
- Deep recursive scanning can consume stack; inspect only the known Messages and
  tool-result content arrays iteratively.
- Dynamic log values must not become metric labels; metric reasons remain in logs.
