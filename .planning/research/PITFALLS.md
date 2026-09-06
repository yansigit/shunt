# Pitfalls Research

**Domain:** Milestone v2 Provider Compatibility
**Researched:** 2026-09-06
**Confidence:** High for current Shunt and pinned OpenCodex behavior; medium for live services

## Executive Warning

Provider compatibility usually fails at state boundaries rather than request serialization. A route must not retry or fail over after observable output or a tool side effect; a stream must not report success without a trustworthy terminal; and replayed history must preserve tool-call/result pairs plus provider-specific continuation metadata. Establish these invariants before adding providers.

Credential-file writeback is frozen. Google AI Studio Web is wholly out of scope: no implementation, tests, supported-provider documentation, or dependencies.

## Critical Pitfalls

| Pitfall | Failure mode / warning sign | Prevention | Likely phase |
|---|---|---|---|
| Retry or failover after commitment | Duplicate text, tool execution, or billing after a body begins. Shunt route failover is pre-body, but generation POST retry classification differs by adapter; Gemini currently uses the idempotent helper. | Define a shared commitment boundary. After any client-visible byte or replay-unsafe side effect, surface the error and never redispatch. Classify every POST explicitly. | Shared foundation; every provider |
| Treating EOF as success | A truncated stream becomes a normal stop. Gemini currently synthesizes STOP when finishReason is absent, and a test codifies it. | Require a documented terminal marker. Allow terminal-less recovery only as an exact provider/model exception when the buffered result is structurally complete; incomplete tool JSON fails closed. | Shared streaming; Gemini |
| Ignoring malformed or oversized frames | Silent data loss, false success, or memory exhaustion. Gemini currently drops invalid UTF-8 and malformed data JSON. | Bound chunks, lines/events, and accumulated tool arguments. Reject invalid UTF-8, JSON, indexes, and field types with one terminal error. Test splits, CRLF, multiline data, junk, oversize, and truncation. | Shared foundation |
| Breaking tool-call/result pairing | Providers reject history, repeat tools, or invent interruptions. OpenCodex issue #1383 showed calls omitted while results remained; PR #1411 repaired pairing and image carriers. | Normalize history as paired units and never retain one side after filtering. Define missing-result synthesis and orphan carriers per provider. Test multiple calls, images, compaction, retries, and partial results. Do not blindly reject missing call_id: issue #3807 shows valid client seed shapes can regress. | Shared tools; Command Code; Cursor |
| Dropping continuation signatures | Gemini/Antigravity reject or degrade continuations when thought signatures are lost, invented, or attached to the wrong call. | Cache only authentic signatures with bounded memory and stable identity; replay only with the matching call. Test multi-turn, multi-account, eviction, and unsigned calls. | Gemini; Antigravity |
| OAuth token exfiltration | A Google token reaches an attacker-controlled base URL or redirect. | Pin auth mode, provider kind, scheme, exact hosts, and redirect behavior before attaching credentials. Revalidate at startup, check, and reload. Test lookalike and suffix hosts. | Shared auth; Antigravity |
| Losing account/session affinity | Refresh, discovery, continuation, or retry silently switches accounts. | Select one account before dispatch and retain it for refresh, project identity, retry, and the full stream. Derive opaque session headers without exposing task IDs. Rotate only before commitment. | Gemini; Antigravity; OpenCode Go |
| Streaming/unary semantic drift | One mode propagates provider errors while the other returns HTTP 200 or a different stop reason. Gemini unary currently discards events returned by process_chunk, so embedded errors can be lost. | Run both modes through the same semantic state machine and terminal checks. Pair fixtures for text, tools, usage, finish reason, and errors. | Every adapter |
| Assuming one OpenAI-compatible dialect | Chat Completions services vary in SSE, DONE, finish reasons, tool deltas, usage, error bodies, and accepted fields. | Keep a conservative generic core with bounded parsing and explicit capability overrides. Never import Responses-only fields or tolerant EOF globally. | Generic Chat Completions |
| Conflating Command Code products | API-key Provider API and subscription/OAuth generation endpoints have different URLs, headers, catalogs, and events. | Model the transports separately and test each. PR #923 established only the API-key auth boundary; subscription behavior needs independent evidence. Preserve writeback exactly. | Command Code |
| Inferring OpenCode Go by model family | Near-name models use the wrong wire or unsupported fields. Issues #2156, #2329, #2442, #3344, #3362, #3378, and #3402 show exact contract failures. | Keep an evidence ledger by exact model ID: endpoint/wire, headers, removals, tools, reasoning, modalities, terminal rules, and date. Current upstream uses exact Responses overrides only for gpt-5.6-luna, grok-4.6, muse-spark-1.3-contributor, and muse-spark-1.2-contributor. Do not generalize. | OpenCode Go |
| Copying upstream without provenance | A substantial translation violates license duties or imports unrelated architecture. | Record repository, pinned commit, source location, translation extent, and verification. Retain the MIT notice for substantial OpenCodex translations; prefer Shunt-native code. | Every implementation; closeout |
| Unsafe live tests | CI flakes, secrets reach logs, or real credential files are refreshed or rewritten. | Keep defaults hermetic. Gate live tests, use temporary paths, redact data, bound time/cost, and assert no credential-file writes. | Provider phases; validation |

## Provider-Specific Traps

### Gemini

- Capture baseline golden fixtures before refactoring.
- Replace silent malformed-frame drops and implicit EOF success with protocol errors.
- Audit generation POST retry classification; status retry can duplicate generation.
- Propagate embedded Google errors identically in streaming and unary modes.
- Any project discovery caching improvement must remain in-memory unless separately authorized.

### Antigravity

- Keep its OAuth identity, scopes, allowed daily/production hosts, project selection, and signature replay distinct from ordinary Gemini.
- Bound signature caches by entries and bytes, partition by account/session, and never manufacture a real signature.
- Verify affinity across refresh, retry, continuation, reload, and multiple accounts.

### Cursor

- Quarantine suspicious semantic echoes before emission. A one-shot retry is safe only before output, before replay-unsafe tool activity, and when not cancelled.
- Preserve strict Connect/frame truncation errors; do not downgrade quota/context ambiguity into replay.
- Validate flattened tool history as paired units so continuation repair cannot duplicate execution.

### Command Code

- Preserve adjacency of assistant calls and results, including image carriers.
- Do not copy OpenCodex permissive done-on-empty/junk EOF behavior without service evidence.
- Map reasoning effort only where the exact transport/model supports it.

### OpenCode Go

- Require exact-model evidence, not branding or family names.
- Test exact endpoint, wire, x-opencode-session, field filters, tool declarations, finish semantics, and unsupported-model rejection.
- A catalog entry is not proof of callability. Require a recorded live response for each supported exact model and retain credential-safe wire fixtures.

## Documentation, Scope, and Release Traps

- Update README.md and README.ko.md, README.ja.md, and README.zh-CN.md together.
- Update English site pages and ko/ja/zh-cn copies for each affected surface. Build the site to verify locale-derived heading anchors.
- Update engineering docs when implementation differs from prior specs. Do not hand-edit generated wiki content.
- Separate implemented, hermetically tested, and live-verified status; never list aspirational models as supported.
- At every phase close, search negatively for Google AI Studio Web names, endpoints, auth flows, fixtures, product documentation, and dependencies.

## Verification Checklist

- Retry/failover tests prove no redispatch after output or a tool side effect.
- Each stream tests normal terminal, DONE where applicable, premature EOF, malformed UTF-8/JSON, split and oversized frames, provider error, cancellation, and backpressure.
- Streaming and unary fixtures assert equivalent semantics.
- Tool histories stay paired through filtering, images, missing IDs, compaction, retry, and continuation.
- OAuth tests cover exact-host pinning, lookalikes, redirects, refresh affinity, multi-account selection, and redaction.
- Live suites are opt-in, bounded, redacted, temporary-path isolated, and prove credentials remain byte-for-byte unchanged.
- OpenCode Go support has dated exact-model evidence.
- Docs/translations match code, required MIT attribution is present, and Google AI Studio Web remains absent.
- Format, Clippy with warnings denied, and the full workspace suite pass before release.

## Recovery Strategies

| Late discovery | Recovery |
|---|---|
| Output was retried | Disable retry first; add a commitment regression before any narrower retry. |
| EOF was accepted | Fail closed and add captured truncation fixtures; add only evidence-backed exact-model tolerance. |
| Tool history is inconsistent | Stop continuation; reconstruct paired units and add client-shape fixtures before repair. |
| OAuth host/affinity is uncertain | Refuse authenticated dispatch rather than falling back. |
| Exact model lacks evidence | Remove it from supported docs/defaults until a safe live probe proves the contract. |
| Translation lacks provenance | Isolate it, record the pinned source, add the MIT notice if substantial, or rewrite. |

## Suggested Phase Ordering

1. Shared protocol/security foundation: commitment state, bounded parsers, terminal contract, tool pairing, auth pinning, provenance, and hermetic test utilities.
2. Preserve and harden Gemini.
3. Harden Antigravity OAuth, affinity, and signatures.
4. Harden Cursor retry quarantine and continuation.
5. Add conservative generic OpenAI Chat Completions.
6. Add Command Code with explicit product/auth separation.
7. Add OpenCode Go only for exact models supported by recorded evidence.
8. Cross-provider parity, credential-safe live validation, docs/i18n, licensing, and exclusion audit.

## Sources

### Current Shunt

- src/retry.rs and src/proxy/failover.rs
- src/adapters/gemini/mod.rs, src/model/gemini.rs, and src/model/gemini_request.rs
- src/auth/google/, src/auth/antigravity/, and src/config.rs
- src/adapters/cursor/ and tests/gemini_translate.rs
- README.md, AGENTS.md, and .planning/PROJECT.md

### OpenCodex

- Pinned upstream/main: 07b48da8fd63881e848d26e0bd50087864f5573e
- src/adapters/openai-chat.ts, command-code.ts, cursor.ts, and cursor/cursor-errors.ts
- src/adapters/google-antigravity-replay.ts, google-antigravity-wire.ts, and src/oauth/google-antigravity.ts
- src/providers/registry.ts, opencode-go-transport.ts, and src/adapters/opencode-go.ts
- Issues: #909, #1383, #1735, #2156, #2193, #2329, #2442, #3344, #3362, #3378, #3402, #3807
- Pull requests: #923, #1411, #3471

---
*Pitfalls research for milestone v2 Provider Compatibility*
