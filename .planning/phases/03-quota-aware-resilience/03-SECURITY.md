---
phase: "03"
slug: "quota-aware-resilience"
status: verified
threats_open: 0
asvs_level: 1
created: "2026-09-05"
---

# Phase 03 — Security

> Security audit for bounded quota evidence, retry scheduling, and pre-output account rotation.

## Trust Boundaries

| Boundary | Data crossing |
|----------|---------------|
| Upstream -> quota classifier | Untrusted status, headers, and error body bytes |
| Classifier -> account pool | Hard-exhaustion or transient decision and cooldown reason |
| Upstream -> retry scheduler | Untrusted `Retry-After` header value |
| Account pool -> downstream client | Final upstream status, safe headers, and response body |
| Stream boundary -> failover controller | Whether output has begun and replay is still safe |
| Documentation -> operator | Public explanation of rotation and cooldown behavior |

## Threat Register

| Threat ID | Category | Component | Severity | Disposition | Mitigation | Status |
|-----------|----------|-----------|----------|-------------|------------|--------|
| T-03-01-01 | Tampering | quota classifier | high | mitigate | Only exact structured `usage_limit_exceeded` or `insufficient_quota` evidence on 429/402 is accepted; malformed, conflicting, duplicate-key, and trailing JSON fail closed | closed — classifier unit matrix passes |
| T-03-01-02 | Denial of service | quota body inspection | high | mitigate | Candidate bodies are limited to 64 KiB and total inspection time is capped at two seconds; oversized or slow bodies remain transient | closed — bounded-size and paused-time tests pass |
| T-03-01-03 | Tampering | account pool | high | mitigate | Typed decisions are shared by initial and refresh paths; hard-exhausted accounts receive finite cooldown and are not revisited during the turn | closed — pool and multi-account fixtures pass |
| T-03-02-01 | Denial of service | Retry-After parser | high | mitigate | Header length, grammar, arithmetic, and resulting delay are bounded; decimal values round upward and HTTP dates cannot underflow | closed — Retry-After unit and retry-policy suites pass |
| T-03-02-02 | Information disclosure | final error relay | high | mitigate | Body inspection rebuilds the response with original status/headers; timeout and chunked overflow preserve the unread stream instead of truncating it | closed — response-fidelity tests pass |
| T-03-02-03 | Replay | stream/failover boundary | high | mitigate | Classification and rotation occur only before downstream output; existing mid-stream no-hop behavior remains unchanged | closed — inbound and failover suites pass |
| T-03-03-01 | Repudiation | documentation | medium | mitigate | English and maintained locale surfaces state the exact finite, pre-output contract | closed — docs/source guards pass |
| T-03-03-02 | Privilege/credential handling | pool integration | high | mitigate | No credential storage or writeback behavior changed; existing per-account credential resolution remains authoritative | closed — diff audit and credential-path tests pass |
| T-03-03-03 | Supply chain/config drift | implementation scope | medium | mitigate | No dependency, public configuration, or provider-semantic expansion was introduced | closed — manifest/config diff audit passes |

## Audit Finding and Remediation

The initial implementation bounded the number of buffered bytes but could wait indefinitely for a chunked 429/402 body. It also rebuilt an oversized chunked response from only the prefix already read. The audit added a fixed two-second total inspection deadline and a streaming response rebuild that chains the buffered prefix to the unread upstream remainder. Slow and oversized bodies now fail closed as transient while remaining byte-complete for the downstream client.

## Accepted Risks

No accepted risks. If the upstream body stream itself errors after returning a prefix, only that readable prefix can be relayed; this is an upstream transport failure rather than a gateway truncation decision.

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-09-05 | 9 | 9 | 0 | Codex / gsd-secure-phase |

## Sign-Off

- [x] Every registered threat has a disposition.
- [x] No critical or high-severity threat remains open.
- [x] Bounded-body failure modes preserve fail-closed classification and relay fidelity.
- [x] No public config or credential writeback behavior changed.
- [x] `threats_open: 0` and `status: verified` are set.

**Approval:** verified 2026-09-05
