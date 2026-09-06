# Feature Research

**Domain:** Lean protocol-translation gateway for coding-agent providers
**Milestone:** Shunt v2 Provider Compatibility
**Researched:** 2026-09-06
**Confidence:** HIGH for current Shunt and OpenCodex `upstream/main`; MEDIUM for proprietary behavior still requiring live/captured verification

## Scope and Product Standard

The v2 acceptance unit is an **exact provider + authentication path + model + wire protocol**, not a provider name. A tuple is supported only when normal prompts, streaming, complex tools, subagents/continuation, long context, authentication, errors, and edge cases are observable and tested without silent loss, unbounded buffering, or mid-stream replay.

Vercel AI Gateway already works through Shunt's Anthropic-compatible route and needs preservation, not a new transport. Google AI Studio Web is excluded completely.

## Feature Landscape

### Table Stakes (Users Expect These)

| Feature | Why Expected | Complexity | Observable acceptance behavior |
|---|---|---:|---|
| ChatGPT/Codex preservation | This mature subscription path cannot regress as providers are added. | HIGH | Native Responses stays opaque over HTTP/SSE and outbound WS v2; OAuth pools, quota admission, continuation, compression, pre-output HTTP fallback, terminals, cancellation, compaction/collaboration policy, and Codex-ingress OpenAI errors retain existing behavior. Never hop after output. |
| Gemini / Google Code Assist verification | Shunt already exposes Code Assist, but existence is not conformance. | MEDIUM | Google OAuth and project envelope are correct; `v1internal:streamGenerateContent?alt=sse` relays incrementally; text/reasoning/functions/usage and multi-round tool history translate; errors are classified without buffering the stream. |
| Antigravity protocol fidelity | Its Cloud Code Assist destination adds contracts beyond the shared Gemini translator. | HIGH | Envelope includes project, request/session identity, model and request metadata; response wrapper/usage unwraps; model/effort mapping is exact; signed tool history survives; OAuth 401 receives at most one account-bound refresh/replay before output; approved HTTPS destinations are enforced. |
| Cursor native-agent hardening | Connect/protobuf framing, continuation and tool schemas are drift-prone. | HIGH | Exact model mapping, incremental text/tool stream, stable continuation, callable executable/freeform schemas, safe image behavior, one terminal outcome, cancellation, EOF/error handling, and evidence-backed pre-output retry. |
| Generic OpenAI Chat Completions | Many API-key gateways expose `/v1/chat/completions`, not Responses or Messages. | HIGH | `/v1/messages` maps system/roles, text/images, tools/results/choice, controls, finish reasons, usage, JSON and SSE. Interleaved tool arguments assemble by stable index/id under explicit bounds. |
| Command Code subscription/OAuth | Subscription users require proprietary `/alpha/generate`, not an OpenAI approximation. | HIGH | Existing login bearer is read without new writeback; bounded live models; required proprietary headers/config; opaque conversation affinity; incremental NDJSON text/reasoning/tools/usage; error finish is failure. |
| Command Code API key | API customers need the same proven wire and capability facts. | MEDIUM | OAuth and key differ only in credential acquisition/pool behavior; both use canonical model IDs, `/alpha/generate`, verified efforts, identical translation and the same conformance suite. |
| Exact-model OpenCode Go evaluation | Go spans Chat and Responses wires and has model-specific regressions. | HIGH | Each enabled tuple records exact ID, endpoint, context, modalities, effort mapping, tool policy, required headers and evidence. Unknown tuples remain unsupported/experimental. |
| Bounded resources | Long agent turns and malformed streams must not degrade the daemon. | HIGH | Request/decompression sizes, SSE/NDJSON residuals, tool assemblies, replay state, discovery, queues, retries and timeouts are bounded; cancel/drop releases permits and credentials. |
| Strict terminal/error fidelity | Clients distinguish invalid, truncated, quota, transient and malformed outcomes. | HIGH | Explicit errors never become clean completion; received usage survives failure; retry/failover is pre-output only; non-Codex gateway errors retain Anthropic shape. |
| Cross-provider fixtures | Proprietary transports regress beyond happy paths. | MEDIUM | Every supported tuple has sanitized JSON/SSE/NDJSON/protobuf fixtures with revision/capture provenance and no secrets/user content. |

### Provider-Specific Contracts

| Provider path | Preserve / implement | Fail-closed boundary |
|---|---|---|
| ChatGPT/Codex OAuth → Responses | Opaque request/stream, OAuth pools/quota, WS v2 continuation/reuse, compression, collaboration passthrough when enabled, HTTP fallback only before output. | Never rewrite opaque reasoning, agent-task, or compaction state; never replay an accepted/billable turn after output. |
| Gemini OAuth → Code Assist | Project envelope, Google OAuth refresh, stream URL, request/response conversion, functions, usage and Google error mapping; replay thought signatures only if verified. | Do not reuse Antigravity OAuth or hosts; make no Google AI Studio Web claim. |
| Antigravity OAuth → Cloud Code Assist | Fixed HTTPS hosts, project-bound token, stable session, exact models/efforts, signed sequential tool history, one account-bound 401 replay. | Reject redirect/off-origin, missing project, invalid signature state and unsupported Claude rewrites before dispatch. |
| Cursor OAuth → AgentService | Connect/protobuf stream, exact catalog, continuation, call IDs/arguments, executable/freeform schemas, verified images, EOF/error/cancel/retry. | No speculative repetition/no-progress killer; open issue #3506 shows model behavior is not a safe generic signal. |
| API key → OpenAI Chat | Correct `/chat/completions` join, bearer/optional-key policy, role/tool/image mapping, streaming usage, structured output and explicitly configured capability gates. | Malformed choices/tool deltas, incomplete arguments and premature EOF fail closed unless an exact provider has captured EOF-tolerance evidence. |
| Command Code OAuth/key → `/alpha/generate` | Canonical IDs, bounded workspace/git config, required headers, stable opaque session, filtered tools, paired call/result history, efforts, bounded NDJSON, usage/finish semantics. | No inferred effort for unknown models; close missing calls with synthetic error results, carry orphan results as user text; no credential writeback change. |
| OpenCode Go key → exact Chat/Responses wire | `x-opencode-session`, exact wire defaults, context/modalities/effort/tool restrictions, operator header override. | Never infer wire by family. Current upstream Responses candidates (`gpt-5.6-luna`, `grok-4.6`, Muse Spark Contributor 1.2/1.3) require independent Shunt verification. |
| Vercel → Anthropic Messages | Preserve configurable Anthropic-compatible auth/header safety, JSON/SSE and errors. | No Vercel-specific adapter or new Chat/Responses claim. |

### Observable Behavior Matrix

| Scenario | Required behavior | Edge cases to prove |
|---|---|---|
| Normal prompt | System/developer/user order and Unicode survive; exact wire model is used; text, stop reason and usage agree between streamed/buffered modes. | Empty/multipart text, aliases, unknown optional fields, wrapped JSON response. |
| Streaming | First useful delta is relayed incrementally; order is stable; padding/heartbeats do not corrupt output; exactly one terminal occurs; disconnect cancels upstream. | Split UTF-8/JSON/SSE lines, CRLF/comments, `data:` NDJSON, explicit error then terminal, duplicate terminal, premature EOF. |
| Complex tools | Schemas, names, descriptions and choice survive; parallel calls retain identity; JSON arguments assemble losslessly; results pair; images are preserved or explicitly rejected. | Null padding, late id/name/index, duplicate IDs, 24+ interleaved calls, freeform string tools, executable policy fields, empty/error/missing/orphan results. |
| Subagents / continuation | Native Codex remains opaque; translated providers receive supported plaintext only; affinity is stable across tool rounds, recovery and changed compacted history. | Encrypted task, lone seeded tool result without call ID, replacement, recovered history, shared cache cohort, same prompt in different threads. Unsupported encryption fails before dispatch. |
| Long context | Advertised limit equals verified usable limit; local memory stays bounded; truncation maps to incomplete/max-token; continuation needs no durable Shunt history. | Exact boundary, oversized request/decompression, large args/images, 1M metadata, 413/context-length errors. |
| Authentication | Credential type is injected only to approved origin; refresh is bounded; account/project stay paired; key/subscription paths are visible operationally. | Blank/expired token, first/second 401, redirect/host spoof, resolution failure, rotation. |
| Errors/retry | Invalid/policy/context errors are terminal; quota and transient rate limits differ; safe `Retry-After` survives; failed-stream usage survives. | HTML/secret-bearing errors, reset edge cases, 429 vs hard quota, 5xx/529, error finish, malformed/no-event body. |
| Model capability | Only exact verified models expose reasoning, vision, parallel tools, structured output or web search; supported operator override wins. | Retired/new IDs, prototype-like ID, unsupported effort, partial tiers, Chat-vs-Responses mismatch. |

### Differentiators (Competitive Advantage)

| Feature | Value Proposition | Complexity | Notes |
|---|---|---:|---|
| Evidence-gated exact compatibility | “Supported” means reproducible wire contract, not marketing label. | MEDIUM | Essential for OpenCode Go and changing subscription backends. |
| One lean conformance harness | Finds semantic drift across SSE, NDJSON, protobuf and Responses WS without OpenCodex's platform. | HIGH | Golden semantic assertions plus event-order assertions. |
| Streaming-first translation | Preserves latency and bounded memory on translated providers. | HIGH | Buffer only non-stream output or minimal bounded fragments. |
| Destination-bound credentials | Couples subscription tokens to protocol kind, TLS origin and provider identity. | MEDIUM | Google, Antigravity, Cursor and ChatGPT tokens fail before off-origin egress. |
| Evidence-scoped repair | Improves compatibility without a global mutation engine. | LOW | Every normalization cites a captured failure and exact tuple. |
| Honest degradation | Explicit unsupported errors replace silent loss or optimistic catalog rows. | MEDIUM | Filter before credential lookup/network dispatch. |

### Anti-Features

| Feature | Why Requested | Why Problematic | Alternative |
|---|---|---|---|
| Provider-wide OpenCode Go | One toggle is convenient. | Models span wires; recent bugs were exact-model specific. | Exact allowlisted tuples. |
| Google AI Studio Web | Another apparent Google path. | Cookie/SAPISIDHASH, extension/daemon/parser/session dependencies are known nonfunctional and out of scope. | Verified Gemini Code Assist only. |
| General repair framework | Seems to absorb all quirks. | Masks regressions and mutates unrelated tuples. | Small transcript-backed normalizations. |
| Cursor no-progress killer | Could stop loops. | Semantic progress is model-dependent; #3506 remains open after mitigation. | Transport fidelity, user cancellation/timeouts, proven signals only. |
| Mid-stream retry/hop | Looks resilient. | Duplicates output/side effects and billing. | Retry/fail over before first output only. |
| Durable history/signature database | Simplifies replay in theory. | Adds migrations, privacy and lifecycle burden. | Bounded in-memory state only when proven necessary. |
| New credential writeback | Could unify stores. | Outside authorization and risks provider-owned credentials. | Read existing approved sources unchanged. |
| Catalog/dashboard/pricing platform | Broad discoverability. | Recreates unrelated OpenCodex machinery. | Bounded required discovery, typed config, existing status. |
| Duplicate pools/retry/WS stacks | Local module convenience. | Divergent ownership leaks leases and semantics. | Reuse Shunt primitives. |
| New Vercel transport | Vercel is named. | Existing Messages route already fits. | Regression-test it. |

## Dependencies and Delivery

```text
[Preserve Codex + shared limits/errors/cancellation]
    └──enables──> [Conformance harness]
                     ├──> [Gemini] ──> [Antigravity deltas]
                     ├──> [Cursor]
                     ├──> [OpenAI Chat]
                     └──> [Command Code OAuth + key]

[OpenAI Chat] ──enables──> [OpenCode Go Chat tuples]
[Native Responses] ──────> [OpenCode Go Responses tuples]
[Exact evidence] ─requires─> [Any Go tuple]

[Mid-stream hop] ─conflicts─> [stream/tool fidelity]
[Google AI Studio Web] ─conflicts─> [scope]
```

### P1 — Preservation and Verification

- [ ] Freeze ChatGPT/Codex transport, auth, quota, continuation, collaboration and error fixtures.
- [ ] Run the full matrix against Gemini; preserve Vercel via the Messages suite.
- [ ] Add bounded reusable SSE/NDJSON/protobuf fixture helpers.

### P1 — Provider Hardening and Missing Wires

- [ ] Antigravity exact envelope/stream/model/signature/401/destination contracts.
- [ ] Cursor exact model/continuation/tool/error/EOF/retry contracts; no progress heuristic.
- [ ] Generic OpenAI Chat text → streaming → usage/errors → images/complex tools.
- [ ] Command Code proprietary headers/config, NDJSON, discovery, pairing, affinity and error finishes for OAuth and API key.

### P2 — Exact OpenCode Go

- [ ] Evaluate the live/captured roster one tuple at a time.
- [ ] Require/derive stable opaque `x-opencode-session`, honoring operator override.
- [ ] Verify context, modalities, efforts and tool/web-search restrictions; leave all unverified tuples unsupported.

## Prioritization

| Feature | Value | Cost | Priority |
|---|---:|---:|---:|
| ChatGPT/Codex preservation | HIGH | MEDIUM | P1 |
| Gemini verification | HIGH | MEDIUM | P1 |
| Antigravity hardening | HIGH | HIGH | P1 |
| Cursor hardening | HIGH | HIGH | P1 |
| Generic OpenAI Chat | HIGH | HIGH | P1 |
| Command Code OAuth + API key | HIGH | HIGH | P1 |
| Exact OpenCode Go | MEDIUM | HIGH | P2 |
| Vercel preservation | MEDIUM | LOW | P1 |

## Evidence and Confidence

| Finding | Confidence | Evidence |
|---|---|---|
| Shunt has ChatGPT/Codex, Gemini, Antigravity, Cursor, Responses and Anthropic paths; Chat and Command Code kinds are missing. | HIGH | Shunt `e029ae2f`: `src/config.rs`, adapters/models/tests, README. |
| Antigravity needs exact envelope/session/signature and account-bound recovery. | HIGH upstream; MEDIUM live | OpenCodex current wire/replay tests; merged #3691, #3799. |
| Cursor schemas must retain executable controls and freeform guidance. | HIGH | Merged #3707, #3715 and current tests. |
| Command Code is proprietary NDJSON with tool pairing and affinity. | HIGH | Current adapter/tests; merged #1411, #3692. |
| OpenCode Go needs a session header and exact wire/tool policy. | HIGH | Issues #3344/#3378; merged #3394/#3405; registry exact defaults. |
| Routed subagent seeds/recovery need dedicated tests. | MEDIUM/HIGH | Open issues #3661 and #3807. |

## Sources

### Local primary evidence

- Shunt at `e029ae2fb35eee9149855769a779343d7139d209`: `src/config.rs`, `src/proxy.rs`, `src/adapters/`, `src/model/`, and `tests/`.
- OpenCodex `upstream/main` at `07b48da8fd63881e848d26e0bd50087864f5573e`: `openai-chat.ts`, `command-code.ts`, Antigravity wire/replay/tools, Cursor modules, `opencode-go.ts`, Go transport, registry and tests.
- `.planning/PROJECT.md` and `.planning/notes/opencodex-port-audit.md`.

### GitHub issues and merged changes

- [Antigravity account-bound OAuth recovery #3691](https://github.com/lidge-jun/opencodex/pull/3691); [fixed-destination quota transport #3799](https://github.com/lidge-jun/opencodex/pull/3799)
- [Cursor executable/freeform schemas #3707](https://github.com/lidge-jun/opencodex/pull/3707); [freeform guidance #3715](https://github.com/lidge-jun/opencodex/pull/3715); [open no-progress recurrence #3506](https://github.com/lidge-jun/opencodex/issues/3506)
- [Command Code tool-result pairing #1411](https://github.com/lidge-jun/opencodex/pull/1411); [recovery affinity #3692](https://github.com/lidge-jun/opencodex/pull/3692)
- [OpenCode Go wire issue #3378](https://github.com/lidge-jun/opencodex/issues/3378); [Grok Responses #3394](https://github.com/lidge-jun/opencodex/pull/3394); [session/tool policy #3405](https://github.com/lidge-jun/opencodex/pull/3405)
- [Encrypted routed subagent recovery #3661](https://github.com/lidge-jun/opencodex/issues/3661); [Codex desktop seed incompatibility #3807](https://github.com/lidge-jun/opencodex/issues/3807)

---
*Feature research for Shunt v2 Provider Compatibility; researched 2026-09-06.*
