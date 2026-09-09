# Requirements: Shunt v2 Provider Compatibility

**Defined:** 2026-09-06
**Core Value:** Codex and Anthropic clients must receive protocol-faithful,
streaming-safe behavior while Shunt stays bounded, predictable, and
operationally lean.

## v2 Requirements

Requirements committed for the Provider Compatibility milestone. Each
requirement maps to exactly one roadmap phase after roadmap creation.

### Preservation and Conformance

- [x] **PRES-01**: Existing ChatGPT/Codex HTTP, SSE, and outbound WebSocket v2 requests retain their current opaque passthrough, continuation, compression, cancellation, and terminal behavior.
- [x] **PRES-02**: Existing ChatGPT/Codex account selection, quota admission, compaction, and opt-in collaboration behavior remain covered by regression tests.
- [x] **PRES-03**: ChatGPT/Codex requests never retry or fail over after any client-visible output or replay-unsafe tool activity.
- [x] **PRES-04**: Gateway-owned errors on Codex ingress remain OpenAI Responses-shaped, while gateway-owned errors on other ingress surfaces remain Anthropic-shaped.
- [x] **PRES-05**: The existing Vercel Anthropic-compatible path retains its request, streaming, authentication, and error behavior without requiring a new provider adapter.

### Gemini / Google Code Assist

- [x] **GEM-01**: Gemini requests preserve Google OAuth identity and the selected project in the Code Assist envelope for the full request lifetime.
- [x] **GEM-02**: Gemini streaming relays text, reasoning, function calls, function results, usage, and terminal state incrementally and in order.
- [x] **GEM-03**: Gemini streaming and non-streaming modes produce equivalent content, tool, usage, finish, and provider-error semantics from the same upstream transcript.
- [x] **GEM-04**: Gemini rejects malformed, oversized, invalid-UTF-8, or prematurely terminated upstream events with a protocol error instead of silently dropping them or synthesizing success.
- [x] **GEM-05**: Gemini generation requests retry only when the failure is proven replay-safe and no output or replay-unsafe tool activity has occurred.

### Antigravity

- [x] **ANT-01**: Antigravity OAuth credentials are sent only to the approved canonical daily or production HTTPS Cloud Code Assist destinations, and redirects cannot carry them off-origin.
- [x] **ANT-02**: An Antigravity request keeps its bearer token, account, discovered project, and retry identity paired for the full response lifetime.
- [x] **ANT-03**: Antigravity requests use the proven Cloud Code Assist agent envelope with a stable opaque conversation session and exact request metadata.
- [x] **ANT-04**: Antigravity accepts only exact supported model and reasoning-effort combinations and reports unsupported combinations before dispatch.
- [x] **ANT-05**: Antigravity preserves sequential tool-call/result history and replays only authentic thought signatures associated with the matching call, account, and session.
- [x] **ANT-06**: Antigravity consumes the upstream SSE contract in both client modes, relaying it incrementally for streaming clients and using bounded accumulation only when the client requested non-streaming output.
- [x] **ANT-07**: Antigravity unwraps text, reasoning, tools, usage, terminal states, and embedded provider errors consistently and fails closed on malformed wrappers or incomplete tool data.
- [x] **ANT-08**: An Antigravity 401 can trigger at most one account-bound refresh and replay, and only before output or replay-unsafe tool activity.

### Cursor

- [x] **CUR-01**: Cursor retains the currently proven AgentService Run destination until a captured or live protocol profile proves a replacement destination safe.
- [x] **CUR-02**: Cursor model, client, endpoint, and capability facts are exact and request-local so evidence from one model cannot alter another request.
- [x] **CUR-03**: Cursor preserves stable continuation identity and paired tool-call/result history across ordinary, compacted, recovered, and multi-round turns.
- [x] **CUR-04**: Cursor transmits executable and freeform tool schemas, tool guidance, image inputs, and tool arguments only in forms proven by fixtures or live evidence, otherwise returning an explicit unsupported error.
- [x] **CUR-05**: Cursor emits incremental reasoning, text, tool, usage, and exactly one terminal outcome, while cancellation releases upstream work and held resources.
- [x] **CUR-06**: Cursor treats malformed Connect/protobuf frames, decompression violations, provider errors, and premature EOF as errors rather than clean completion.
- [x] **CUR-07**: Cursor distinguishes local request/header construction failures from upstream transport failures and retries or fails over only before output or replay-unsafe tool activity.
- [x] **CUR-08**: Cursor does not add a semantic repetition or no-progress cancellation heuristic without a separately proven transport signal.

### Generic OpenAI Chat Completions

- [x] **CHAT-01**: An operator can configure an OpenAI Chat Completions-compatible upstream independently from OpenAI Responses and Anthropic Messages upstreams.
- [x] **CHAT-02**: Chat endpoint construction produces exactly one `/chat/completions` path from an accepted API root and rejects ambiguous query, fragment, or malformed URL configurations.
- [x] **CHAT-03**: Chat requests translate Anthropic system, user, assistant, text, image, generation-control, and model fields without sending Responses-only fields.
- [x] **CHAT-04**: Chat requests preserve declared tools, tool choice, parallel assistant tool calls, and correctly paired tool-result messages with stable call identities.
- [x] **CHAT-05**: Non-streaming Chat responses translate bounded text, reasoning, tools, usage, finish reasons, and recognized provider errors into the Anthropic response contract.
- [x] **CHAT-06**: Streaming Chat responses relay text and reasoning incrementally and assemble interleaved indexed tool-call deltas without reordering or losing identities or arguments.
- [x] **CHAT-07**: Chat streaming requires a trustworthy terminal, emits exactly one terminal outcome, and rejects malformed events, incomplete tool arguments, oversized state, and premature EOF.
- [x] **CHAT-08**: Chat authentication uses the configured API-key source, strips inbound credential slots before dispatch, and does not expose credentials in requests to any destination other than the configured upstream.
- [x] **CHAT-09**: Chat supports normal, streaming, image, tool-heavy, long-context boundary, cancellation, and provider-error scenarios through hermetic conformance tests.

### Command Code API-Key Product

- [x] **CCK-01**: The `commandcode` API-key product uses the generic Chat contract at the canonical `/provider/v1/chat/completions` endpoint rather than the proprietary subscription endpoint.
- [x] **CCK-02**: Command Code API-key authentication, endpoint, and model facts are tested independently from subscription authentication and cannot be selected through the subscription credential mode.
- [x] **CCK-03**: The API-key product passes the applicable Chat normal, streaming, tool-heavy, long-context, cancellation, and error-path conformance scenarios.

### Command Code Subscription Product

- [x] **CCS-01**: The `command-code` subscription product reads a bearer from an approved environment source or existing local CLI credential file without creating, refreshing, migrating, or writing credential files.
- [x] **CCS-02**: Subscription bearers can be attached only to the canonical HTTPS Command Code origin and `/alpha/generate` transport, with off-origin redirects rejected.
- [x] **CCS-03**: Subscription requests carry the proven proprietary envelope and identity headers, an exact supported model/effort combination, and a stable opaque conversation session.
- [x] **CCS-04**: Subscription history preserves adjacent tool-call/result units, represents missing results explicitly, retains orphan results as user-visible context, and preserves supported tool-result images.
- [x] **CCS-05**: Subscription NDJSON relays text, reasoning, tool calls, usage, provider errors, and finish events in order for streaming clients and through bounded accumulation for non-streaming clients.
- [x] **CCS-06**: Subscription NDJSON rejects malformed or oversized records, junk-only responses, incomplete tools, provider error finishes, duplicate terminals, and EOF without a trustworthy terminal.
- [x] **CCS-07**: Subscription retry or failover is bounded to failures before output or replay-unsafe tool activity and preserves the selected credential and session across a safe replay.
- [x] **CCS-08**: The subscription product passes hermetic normal, streaming, tool-heavy, subagent/continuation, long-context boundary, cancellation, authentication, and error-path scenarios.

### Exact-Model OpenCode Go Evaluation

- [x] **OGO-01**: Each OpenCode Go candidate has a dated evidence record identifying the exact model, canonical destination, wire protocol, required headers, context, modalities, effort, tools, field filtering, terminal behavior, and capture or live source.
- [x] **OGO-02**: An OpenCode Go tuple is exposed or documented as supported only after its exact model and wire pass hermetic tests plus a credential-safe captured or live verification; the phase may validly ship no tuples.
- [x] **OGO-03**: Each supported OpenCode Go tuple reuses the matching Chat, Responses, or Anthropic contract and sends a stable opaque `x-opencode-session` only to the canonical OpenCode Go destination.
- [x] **OGO-04**: Unknown, family-inferred, ambiguous, or failed OpenCode Go tuples remain unsupported and are rejected before credential lookup or network dispatch.

### Security, Bounds, and Failure Invariants

- [x] **SAFE-01**: Every new request body, decompressed frame, SSE/NDJSON residual, event, tool assembly, replay state, queue, retry, and timeout has an explicit tested bound.
- [x] **SAFE-02**: Streaming success bodies are never fully buffered; accumulation is permitted only when the client explicitly requested non-streaming output and remains bounded.
- [x] **SAFE-03**: Every provider path emits at most one authoritative terminal outcome, and explicit upstream errors or incomplete streams never become clean completion.
- [x] **SAFE-04**: Retry and fallback share a tested commitment boundary that forbids redispatch after client-visible output or replay-unsafe tool activity.
- [x] **SAFE-05**: Provider credentials are bound to their provider kind and approved destination, redacted from diagnostics and fixtures, and held no longer than the response lifetime requires.
- [x] **SAFE-06**: Tool-call/result pairs and authentic continuation metadata survive supported filtering, retries, recovery, and translation without inventing signatures or duplicating execution.
- [x] **SAFE-07**: Dropping or cancelling a request releases upstream work, admission permits, account leases, parser state, and buffered data within a bounded time.

### Verification, Provenance, and Documentation

- [ ] **REL-01**: Every supported provider, authentication path, exact model, and wire tuple has sanitized hermetic fixtures covering normal, streaming, tool-heavy, terminal, malformed, truncation, authentication, and retry boundaries as applicable.
- [ ] **REL-02**: Captured or translated fixtures record their source repository, pinned revision, capture date, and sanitization status without retaining secrets, account/project identifiers, or private user content.
- [ ] **REL-03**: Any substantial implementation or fixture translated from OpenCodex retains the required MIT notice and provenance; independently implemented behavior records its evidence source.
- [ ] **REL-04**: Live smoke tests are opt-in, bounded in time and cost, redact credential material, use isolated temporary state, and verify source credential files remain byte-for-byte unchanged; missing credentials produce a documented skip.
- [ ] **REL-05**: Each observable provider/configuration change updates README, relevant engineering docs, and the English plus ko/ja/zh-cn site and root README surfaces in the same implementation phase; generated `wiki/` content is not hand-edited.
- [ ] **REL-06**: Release verification passes formatting, Clippy with warnings denied, the full all-features workspace suite, documentation/site validation, and negative scope checks before merge.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Google AI Studio Web support | Explicitly excluded and known nonfunctional for this milestone. No implementation, tests, support claims, or dependencies may be added. |
| Google AI Studio Web cookie/SAPISIDHASH authentication | Expands credential and browser attack surface for an excluded transport. |
| Google AI Studio Web browser extension, daemon, MakerSuite parser, session synchronization, or WebKit/browser dependencies | Unrelated to verified Gemini Code Assist and native Antigravity transports. |
| Credential-file writeback changes | Requires separate explicit approval; all new credential discovery in this milestone is read-only. |
| Whole-provider OpenCode Go support | The service spans Chat, Responses, and Anthropic wires; exact-model evidence is required. |
| Family-inferred OpenCode Go model routing | Similar model names have demonstrated different and changing wire contracts. |
| New Vercel-specific adapter | Existing Anthropic-compatible routing already covers the supported Vercel path. |
| Cursor semantic no-progress or repetition heuristic | No proven transport-level signal makes gateway-owned cancellation safe. |
| Mid-stream retry or provider hopping | Risks duplicate output, tool side effects, and billing. |
| Durable request history, continuation, or thought-signature persistence | Adds privacy, migration, and lifecycle scope without current evidence. |
| Generalized repair, provider catalog, pricing, dashboard, or storage platform | Outside a lean protocol gateway and not required by supported tuples. |
| New runtime or protocol-parser dependencies | Existing Rust dependencies provide the required bounded primitives. |

## Traceability

Each committed requirement has exactly one owning phase.

| Requirement | Phase | Status |
|-------------|-------|--------|
| PRES-01 | Phase 9 | Complete |
| PRES-02 | Phase 9 | Complete |
| PRES-03 | Phase 9 | Complete |
| PRES-04 | Phase 9 | Complete |
| PRES-05 | Phase 9 | Complete |
| GEM-01 | Phase 10 | Complete |
| GEM-02 | Phase 10 | Complete |
| GEM-03 | Phase 10 | Complete |
| GEM-04 | Phase 10 | Complete |
| GEM-05 | Phase 10 | Complete |
| ANT-01 | Phase 11 | Complete |
| ANT-02 | Phase 11 | Complete |
| ANT-03 | Phase 11 | Complete |
| ANT-04 | Phase 11 | Complete |
| ANT-05 | Phase 11 | Complete |
| ANT-06 | Phase 11 | Complete |
| ANT-07 | Phase 11 | Complete |
| ANT-08 | Phase 11 | Complete |
| CUR-01 | Phase 12 | Complete |
| CUR-02 | Phase 12 | Complete |
| CUR-03 | Phase 12 | Complete |
| CUR-04 | Phase 12 | Complete |
| CUR-05 | Phase 12 | Complete |
| CUR-06 | Phase 12 | Complete |
| CUR-07 | Phase 12 | Complete |
| CUR-08 | Phase 12 | Complete |
| CHAT-01 | Phase 13 | Complete |
| CHAT-02 | Phase 13 | Complete |
| CHAT-03 | Phase 13 | Complete |
| CHAT-04 | Phase 13 | Complete |
| CHAT-05 | Phase 13 | Complete |
| CHAT-06 | Phase 13 | Complete |
| CHAT-07 | Phase 13 | Complete |
| CHAT-08 | Phase 13 | Complete |
| CHAT-09 | Phase 13 | Complete |
| CCK-01 | Phase 14 | Complete |
| CCK-02 | Phase 14 | Complete |
| CCK-03 | Phase 14 | Complete |
| CCS-01 | Phase 14 | Complete |
| CCS-02 | Phase 14 | Complete |
| CCS-03 | Phase 14 | Complete |
| CCS-04 | Phase 14 | Complete |
| CCS-05 | Phase 14 | Complete |
| CCS-06 | Phase 14 | Complete |
| CCS-07 | Phase 14 | Complete |
| CCS-08 | Phase 14 | Complete |
| OGO-01 | Phase 15 | Complete |
| OGO-02 | Phase 15 | Complete |
| OGO-03 | Phase 15 | Complete |
| OGO-04 | Phase 15 | Complete |
| SAFE-01 | Phase 9 | Complete |
| SAFE-02 | Phase 9 | Complete |
| SAFE-03 | Phase 9 | Complete |
| SAFE-04 | Phase 9 | Complete |
| SAFE-05 | Phase 9 | Complete |
| SAFE-06 | Phase 9 | Complete |
| SAFE-07 | Phase 9 | Complete |
| REL-01 | Phase 16 | Pending |
| REL-02 | Phase 16 | Pending |
| REL-03 | Phase 16 | Pending |
| REL-04 | Phase 16 | Pending |
| REL-05 | Phase 16 | Pending |
| REL-06 | Phase 16 | Pending |

**Coverage:**

- v2 requirements: 63 total
- Mapped to phases: 63
- Unmapped: 0 ✓

---
*Requirements defined: 2026-09-06*
*Last updated: 2026-09-06 after v2 roadmap creation*
