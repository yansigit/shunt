---
phase: "15"
slug: "exact-opencode-go-evidence-gate"
status: verified
threats_open: 0
threats_total: 13
register_authored_at_plan_time: true
asvs_level: 1
created: "2026-09-08"
---

# Phase 15 — Security

## Trust Boundaries

| Boundary | Description | Data crossing |
|---|---|---|
| Operator configuration | Explicit Go identity and canonical destination | Untrusted configuration; no secret values |
| Credential/dispatch | Admission before lookup or egress | Synthetic test credentials only |
| Evidence ledger | Source-only facts versus admission claims | Pinned revision/date, no captures or live data |
| Published documentation | English and maintained translations | Configuration, support and evidence claims |
| CLI verification | Owned gateway child and isolated state | Synthetic config/key, loopback requests |

## Threat Register

The identifiers below enumerate the 13 threats authored in plans 01–05. Plan 06
adds guide mirrors within the documentation boundaries already covered here.

| Threat ID | Category | Component | Severity | Disposition | Mitigation | Status |
|---|---|---|---|---|---|---|
| T-15-01 | Spoofing | Go identity | high | mitigate | Explicit ProviderKind/preset, canonical URL/auth/env validation; negative config tests | closed |
| T-15-02 | Tampering | Credential boundary | high | mitigate | Gate precedes credential/auth/dispatch; injected resolver counters and mutation failures | closed |
| T-15-03 | Information disclosure | Rejection diagnostics | medium | mitigate | Fixed concise gateway error; CLI asserts exact Anthropic shape/message | closed |
| T-15-04 | Spoofing | Ledger provenance | high | mitigate | Four exact records pinned to 055c3ecf/date; required provenance mutation tests; admits nothing | closed |
| T-15-05 | Tampering | Source-only promotion | high | mitigate | Ledger requires capture/live none and admitted empty; mutation tests reject promotion | closed |
| T-15-06 | Information disclosure | Ledger content | medium | mitigate | No credential/capture/private identity data; source-only descriptive fields | closed |
| T-15-07 | Spoofing | English support claims | high | mitigate | Per-file zero-support assertions and token-mutation check; no candidate promoted | closed |
| T-15-08 | Tampering | Navigation | medium | mitigate | Structured four-locale nav assertion and built-page link checks | closed |
| T-15-09 | Spoofing | CLI smoke quality | high | mitigate | Actual process readiness and two routed POSTs; expected 400 after boot, not missing-key exit | closed |
| T-15-10 | State escape | Test lifecycle | high | mitigate | Fresh isolated home, non-10100 listener, cleanup assertions, raw production fingerprints unchanged | closed |
| T-15-11 | Information disclosure | Child environment | medium | mitigate | env_clear plus explicit synthetic key and loopback proxy; no real inherited credentials | closed |
| T-15-12 | Spoofing | Locale claims | high | mitigate | Nine independently named native-language assertions, exact keys/URL/candidates, four guide assertions | closed |
| T-15-13 | Tampering | Locale links | high | mitigate | New locale links have no fragment assumptions; built provider routes and normalized links verified | closed |

## Accepted Risks Log

No security risks were accepted to close threats. OGO-02 remains a manual
evidence assumption, not a security waiver: the implementation admits zero
tuples. The plain-HTTP fixture cannot independently observe failed Go TLS
attempts; that limitation is explicit in 15-RUNTIME-REVIEW.md. Credential
counters, error outcomes, DNS pinning and the CLI proxy sentinel provide the
separate controls described above.

## Security Audit Trail

| Audit date | Threats total | Closed | Open | Run by |
|---|---|---|---|---|
| 2026-09-08 | 13 | 13 | 0 | Root, authored-register ASVS L1 verification |

The configured ASVS level is 1 and blocking threshold high. Every authored
mitigation was found; secure-phase's authored-register L1 short-circuit applies.
No deeper auditor or live-provider/Computer review is claimed. Related evidence:
15-01/02/03/05/06/04 summaries, 15-VALIDATION.md, 15-RUNTIME-REVIEW.md.

## Sign-Off

- [x] All authored threats have a mitigation disposition.
- [x] No accepted-risk waiver was required.
- [x] Zero open threats, including below-threshold threats.
- [x] Frontmatter status verified.

**Approval:** verified 2026-09-08, bounded to this phase's hermetic scope.
