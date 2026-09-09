---
phase: "14"
slug: command-code-product-separation
status: verified
threats_open: 0
asvs_level: 1
register_authored_at_plan_time: true
created: "2026-09-08"
---

# Phase 14 — Security

ASVS L1 audit of all six authored `<threat_model>` blocks. IDs below preserve
plan order, including repeated threats with distinct phase surfaces. Blocking
threshold is high. Root checked code and fixtures, supported by the independent
GLM/high review in 14-REVIEW.md. This is not a live penetration test.

## Trust Boundaries

| Boundary | Description | Data crossing |
| --- | --- | --- |
| Client to gateway | Untrusted JSON, history and conversation ID | Prompt/tool data |
| Config to credentials | Product kind and origin checked before resolution | Environment token or read-only CLI file |
| Gateway to provider | Canonical subscription endpoint; separately configured Chat API | Bearer and compiled request |
| Provider to client | Bounded NDJSON and shared semantic machine | Untrusted content, usage and terminal records |
| Verification to user state | Temporary test homes and owned ports | Synthetic credentials only |
| Source to public docs | Pinned facts and maintained translations | No live credential contents |

## Threat Register

Paths below are relative to the worktree. All dispositions are mitigation.

| ID | Category | Component / authored plan | Severity | Disposition | Mitigation / evidence | Status |
| --- | --- | --- | --- | --- | --- | --- |
| T-14-01 | Spoofing | API preset / 01 | high | mitigate | src/config/presets.rs pins canonical default; tests/command_code_api_conformance.rs captures endpoint grammar. Explicit custom Chat endpoints retain existing operator-configured semantics, not subscription credential access. | closed |
| T-14-02 | Tampering | Product crossover / 01 | high | mitigate | config kind/auth pairing and router_tests/products.rs concurrent distinct bearer/model/path assertions | closed |
| T-14-03 | Information disclosure | Logging / 01 | medium | mitigate | Sensitive bearer headers; redacted Credential Debug; neutral errors and synthetic fixtures | closed |
| T-14-04 | Tampering | Subscription egress / 02 | high | mitigate | auth/command_code.rs validate_provider before resolution; request::dispatch_headers repeats validation; fixed ENDPOINT and redirect-none client | closed |
| T-14-05 | Information disclosure | Credential state / 02 | high | mitigate | resolve_sources temp-file injection; read-only open and byte/mtime/inventory fixture assertions | closed |
| T-14-06 | Spoofing | False terminal / 02 | high | mitigate | command_code_response machine rejects missing/duplicate/conflicting terminals; tracer and response fixtures | closed |
| T-14-07 | Denial of service | Record size / 02 | medium | mitigate | ndjson.rs MAX_RECORD_BYTES and residual boundary tests | closed |
| T-14-08 | Repudiation | Fixture provenance / 02 | medium | mitigate | 14-PROTOCOL-EVIDENCE.md pinned revision and explicit synthetic/source-derived labels; no live claim | closed |
| T-14-09 | Tampering | Tool history / 03 | high | mitigate | history.rs authentic identities and ordering; tests/command_code_translate.rs tool fixtures reject ambiguity | closed |
| T-14-10 | Information disclosure | Session and envelope / 03 | high | mitigate | request.rs length-delimited credential-scoped SHA-256; constant workspace-free config; header/envelope tests | closed |
| T-14-11 | Spoofing | Model admission / 03 | medium | mitigate | efforts.rs exact case-sensitive table and pre-credential request translation; unknown tuple fixtures | closed |
| T-14-12 | Tampering | Malformed response / 04 | high | mitigate | Shared sticky-failure machine, bounded parser, authoritative finish plus framed EOF; tests/command_code_translate/response.rs | closed |
| T-14-13 | Denial of service | Response accumulation / 04 | high | mitigate | Record/wire/semantic/tool/argument/block bounds; complete-record deadline; below/at/above and paused-clock fixtures | closed |
| T-14-14 | Information disclosure | Upstream diagnostics / 04 | medium | mitigate | Neutral gateway error bodies; matrix.rs upstream-status and failed-finish checks | closed |
| T-14-15 | Information disclosure | Redirect/mutation / 05 | high | mitigate | Two destination gates, lookup-counter negatives, replay.rs zero-follow-up redirect fixture | closed |
| T-14-16 | Tampering | Duplicate generation / 05 | high | mitigate | ConnectOnly safety; replay.rs post-send single attempt and original identity retained across pre-connect retry | closed |
| T-14-17 | Tampering | CLI file integrity / 05 | high | mitigate | O_RDONLY, bounded regular-file reads only; no refresh/writeback; auth fixture SHA/mtime/inventory checks | closed |
| T-14-18 | Denial of service | Cancellation / 05 | medium | mitigate | lifetime.rs pre-header/mid-body downstream drop in both modes proves upstream closure and capacity reuse | closed |
| T-14-19 | Information disclosure | Capability claims / 06 | high | mitigate | Ten exact source-derived rows in four provider pages, reporter-only exclusions, Phase 16 live caveat; no secret contents | closed |
| T-14-20 | Tampering | Production state / 06 | high | mitigate | /tmp/shunt-phase12-isolated-run.cjs used for all stateful verification; before/after production mtime/SHA and inventory unchanged; owned listeners stopped | closed |
| T-14-21 | Tampering | Locale security drift / 06 | medium | mitigate | Four README/provider/guide/reference surfaces updated; site build and built-content parity checks pass | closed |

## Accepted Risks Log

No accepted risks. Live provider acceptance and Computer evaluation are not
claimed by this audit. The five-second credential timeout bounds waiting for
the blocking read task; it cannot forcibly interrupt a stalled filesystem read.
The bounded file size and regular-file check remain enforced.

## Security Audit Trail

| Audit date | Threats total | Closed | Open | Run by |
| --- | --- | --- | --- | --- |
| 2026-09-08 | 21 | 21 | 0 | Root ASVS L1, with independent GLM/high code review |

## Sign-Off

- [x] All authored threats have a disposition and evidence.
- [x] No risks silently accepted on the user's behalf.
- [x] threats_open: 0 at high threshold; all 21 rows closed.
- [x] status: verified.

**Approval:** Verified 2026-09-08 at ASVS L1; not a live acceptance result.
