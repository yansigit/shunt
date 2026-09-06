---
phase: "04"
slug: native-compaction
status: verified
threats_open: 0
asvs_level: 1
created: "2026-09-05"
---

# Phase 04 — Security

## Threat Register

| Threat ID | Category | Severity | Mitigation | Status |
|-----------|----------|----------|------------|--------|
| T-04-01-01 | Credential disclosure | high | Native compact capability permits only validated ChatGPT OAuth destinations or the exact canonical OpenAI API origin; lookalike/path variants fail closed | closed |
| T-04-01-02 | Request tampering | high | Complete bounded JSON parse requires one unique, non-empty string model; original body bytes remain untouched for forwarding | closed |
| T-04-01-03 | Denial of service | high | Existing concurrency, content-length, streamed body, zstd ratio/size, and upstream TTFB bounds apply to the compact path | closed |
| T-04-01-04 | Information disclosure | high | Existing reserved credential/cookie/hop-by-hop stripping and safe upstream response relay are reused | closed |
| T-04-02-01 | State disclosure | high | Opaque continuation and input fields are never logged, decrypted, persisted, or synthesized | closed |
| T-04-02-02 | Replay/cross-account state | high | Session stickiness is namespaced by authenticated client and compact uses the same account identity machinery as ordinary Responses | closed |
| T-04-02-03 | Method confusion | medium | Compact is a separate POST-only route and cannot enter the inbound WebSocket handler | closed |
| T-04-02-04 | Repudiation | medium | Focused zero-network, byte/header fidelity, request-limit, full regression, and locale/docs gates are recorded in validation | closed |

## Audit Notes

- Capability detection is based on destination identity, not generic Responses flavor. `api.openai.com.evil.test` and noncanonical path variants are covered by unit tests.
- Unsupported providers are rejected before credential resolution, so no provider token can reach an unverified compact URL.
- Compact error responses use the same OpenAI envelope boundary as ordinary inbound Responses.
- No credential writeback or persistent state path was added.

## Accepted Risks

None.

**Approval:** verified 2026-09-05; zero open high/critical threats.
