---
phase: 04-native-compaction
created: 2026-09-05
confidence: high
---

# Phase 04 Research — Native Compaction

## Recommendation

Port only OpenCodex's native compact forwarding boundary. Do not port its routed summarizer, handoff cache, policy router, continuation crypto, response buffering platform, or history machinery. Shunt already has the necessary bounded ingress, exact native routing, provider auth, account pooling, response relay, and OpenAI-shaped gateway error primitives.

## Source Evidence

| Concern | OpenCodex evidence | Shunt insertion point |
|---------|--------------------|-----------------------|
| Endpoint ordering | `src/server/index.ts` matches `POST /v1/responses/compact` before the generic Responses branch | `src/server.rs` registers one compact POST beside `codex_endpoint::PATHS` |
| Required model | `src/server/responses/compact.ts` rejects invalid bodies and missing/non-string model before routing | strict compact model parser in `src/codex_endpoint.rs` |
| Capability gate | `supportsNativeResponsesCompactEndpoint` permits canonical ChatGPT and official OpenAI only | internal `Config::supports_native_responses_compact` |
| Native URL | OpenCodex forwards to `${base}/responses/compact`; ChatGPT uses its canonical Codex base | derive from `responses_url(...)+"/compact"` |
| Auth/account identity | Native compact resolves the same account context as normal Responses | reuse `forward_codex_inbound` and pool machinery with an operation enum |
| Opaque contract | Native branch forwards compact input to the native upstream rather than locally retaining history | preserve `Bytes` and headers through the existing passthrough path |

## Architecture

Add a distinct `COMPACT_PATH`, not a fourth ordinary `PATHS` entry, because it has POST-only method semantics and must not inherit the inbound WebSocket GET handler. Pass a small internal `InboundOperation::{Responses, Compact}` value from the endpoint to the inbound adapter. The operation changes only URL construction; all authentication, account rotation, safe header handling, and lazy response relay stay shared.

Strict request validation belongs at the ingress before `forward_turn`: decode zstd under the existing request and ratio bounds when necessary, reject unsupported encodings for compact, and require a complete JSON object with a unique non-empty string `model`. The ordinary endpoint retains its permissive labels-only parser.

Routing uses `resolve_native_inbound`. `Pinned` is accepted because the configured pinned provider is necessarily ChatGPT OAuth and therefore a verified compact backend. `Selected` is accepted only when the selected provider is ChatGPT OAuth or the exact canonical OpenAI API base with Responses kind/API-key auth. `Rejected` stays a 400 pre-dispatch Responses error. A selected but compact-incapable provider returns an actionable 400 before credential resolution or network I/O.

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Credential sent to an unsupported/lookalike host | exact capability predicate; reuse existing auth-origin validation |
| Local interpretation corrupts opaque state | byte-for-byte body forwarding; model read only for routing |
| Duplicate model smuggling | strict serde visitor/seed or duplicate-aware object parser; fail closed |
| Compact path accidentally upgrades to WebSocket | POST-only registration separate from `PATHS` |
| Pool behavior diverges from normal Responses | shared inbound adapter with URL operation parameter |
| Large/slow request consumes resources | existing content-length/read-body/zstd limits and concurrency layer |
| Upstream error loses metadata | existing lazy `relay_passthrough` and safe response header filter |

## Test Strategy

- Extend `tests/inbound_codex_endpoint.rs` with compact route absence/presence/auth/body-limit and pinned/exact provider fixtures.
- Assert the mock receives `/codex/responses/compact` or `/responses/compact`, exact bytes, caller identity headers, and selected credentials.
- Assert malformed/duplicate/missing model and unsupported xAI/custom providers produce OpenAI-shaped 400 responses with zero mock calls.
- Assert success and error response status/body/safe headers are relayed without SSE translation.
- Retain the Phase 2 ordinary endpoint suite, formatting, strict Clippy, and full workspace tests.

## Documentation Surfaces

README.md plus README.ko.md/README.ja.md/README.zh-CN.md, and the English/ko/ja/zh-cn Nimbus inbound Codex guide. A substantial new subsystem doc is unnecessary because this is a narrow extension of the existing endpoint; update the existing M11 engineering note if its endpoint table claims only ordinary Responses paths.
