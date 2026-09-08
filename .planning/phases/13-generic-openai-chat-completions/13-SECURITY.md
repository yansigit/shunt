---
phase: "13"
slug: generic-openai-chat-completions
status: verified
threats_open: 0
asvs_level: 1
created: "2026-09-08"
register_authored_at_plan_time: true
---

# Phase 13 — Security

ASVS L1, block threshold high. State B: every plan contained a parseable prose
threat model. The register below expands those authored threats into stable
IDs; it is not an empty retroactive register. Root verified code controls and
the already executed adversarial fixtures. With all authored threats closed,
the workflow's L1 short-circuit applies; no independent security-auditor run
is claimed. Independent code review is preserved in 13-REVIEW.md.

## Trust boundaries

| Boundary | Data crossing |
|---|---|
| Operator config to destination/auth | Endpoint root and configured API-key source |
| Anthropic ingress to Chat request | Untrusted messages, tools, images and controls |
| Upstream JSON/SSE to client semantics | Untrusted bytes, usage, finish and errors |
| Response lifetime to retry/admission | Dispatch classification and owned resources |
| Implementation to published docs | Support/limit claims and maintained translations |

## Threat register

All dispositions are mitigate; no risk acceptance is inferred from user approval.

| Threat ID | Category | Component | Severity | Mitigation / executed evidence | Status |
|---|---|---|---|---|---|
| T-13-01-a | Spoofing/tampering | Endpoint/redirect | high | Shared strict endpoint builder; Policy::none; redirect target gets zero requests | closed |
| T-13-01-b | Tampering | EOF terminal | high | Machine requires authoritative finish and DONE; EOF fixture errors once | closed |
| T-13-01-c | DoS | SSE frames | medium | Decoder event/residual bounds; exact boundary triples | closed |
| T-13-01-d | Information disclosure | Bearer diagnostics | medium | Configured fixture-only keys, neutral diagnostics, header allowlist, inbound credential-slot tests | closed |
| T-13-02-a | Tampering | Request fields/payload | high | Deny-by-default whitelist; byte budgets; exact wire body and zero-dispatch rejection fixtures | closed |
| T-13-02-b | Spoofing | Ambiguous URL | high | Boot rejection of userinfo/query/fragment and malformed roots; same builder at send | closed |
| T-13-02-c | DoS | Tool arguments | medium | Request byte budget plus response assembly 1-MiB-per-call bound, below/at/above tests | closed |
| T-13-02-d | Information disclosure | Request errors | low | Typed fixed-context errors do not echo message content | closed |
| T-13-03-a | Spoofing | Embedded errors/EOF | high | Checked error envelopes and terminal state; embedded-200 and premature-EOF tests | closed |
| T-13-03-b | Tampering | Usage/finish | high | Checked integer range and exhaustive supported finish mapping; invalid counter/finish fixtures | closed |
| T-13-03-c | DoS | Retained response state | medium | Separate event, residual, semantic, unary and tool bounds; cap triples | closed |
| T-13-03-d | Information disclosure | Provider errors | low | Neutral messages; only bounded character-validated request-ID headers; raw body metadata dropped | closed |
| T-13-04-a | Spoofing | Redirect credentials | high | Redirect-disabled client and wire target isolation fixture | closed |
| T-13-04-b | Tampering | Interleaved tools | high | Request-local index-keyed assembly, identity conflict rejection, one boundary parse; concurrent/interleaved tests | closed |
| T-13-04-c | DoS | Tool state | medium | 128 calls and 1-MiB arguments; named boundary triples and incomplete-terminal errors | closed |
| T-13-04-d | Replay/availability | Post-send timeout | high | ConnectOnly and failure=None after possible acceptance; one-request and zero-fallback assertions with positive pre-connect control | closed |
| T-13-05-a | Spoofing | Support claims | medium | Engineering/provider pages explicitly state synthetic-only evidence and supported provider extensions; no live/Computer pass claim | closed |
| T-13-05-b | Tampering | Locale drift | medium | Same-change four locales, identical-config assertion, fragment-free new links and built routes | closed |
| T-13-05-c | Availability | Skipped scenarios | low | Named 12-scenario map, nonzero filters, no disabled Chat tests, full suite execution | closed |

## Accepted risks log

No accepted risks. Live availability is outside this phase and remains the
explicit Phase 16 gate; fixed resource bounds and unsupported semantics are
documented constraints, not silent risk acceptance.

## Audit trail

| Date | Authored threats | Closed | Open | Run by |
|---|---|---|---|---|
| 2026-09-08 | 19 | 19 | 0 | Root, ASVS L1 control and fixture audit |

All stateful checks inherited fresh isolated OPENCODEX_HOME and non-10100 ports.
Production config mtime/SHA and backup inventory were unchanged. Existing
credential-file writeback behavior and provider settings were not changed.

## Sign-off

- [x] Every authored threat has a mitigation and concrete evidence.
- [x] No unapproved accepted risks.
- [x] threats_open: 0.
- [x] status: verified.

Approval: verified 2026-09-08 at ASVS L1; not a claim of exhaustive external penetration testing.
