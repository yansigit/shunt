# Stack Research

**Domain:** Rust provider-protocol compatibility for Shunt v2
**Researched:** 2026-09-06
**Confidence:** HIGH for the dependency and architecture recommendation; MEDIUM for live proprietary-service behavior until account-backed probes are captured

## Recommendation

Keep Shunt on its existing Rust/Axum/Tokio stack and add **no new third-party
runtime dependency** for v2. The repository already has every primitive the
provider work needs: streaming HTTP/2, bounded byte streams, JSON, protobuf,
SSE framing, hashing, UUIDs, URL validation, retries, account selection, OAuth
callbacks, atomic credential storage, and wiremock integration tests.

Add only two new protocol verticals:

1. `openai_chat`: Anthropic Messages to/from OpenAI Chat Completions, including
   a bounded SSE/JSON state machine and tool-call assembly.
2. `command_code`: the proprietary subscription request plus NDJSON response
   wire used by `POST /alpha/generate`.

Preserve ChatGPT/Codex, Gemini, native Antigravity, and Cursor in their existing
adapters. Harden those adapters from evidence instead of layering generic
middleware over them. The Command Code API-key product should be a preset on
the new generic Chat adapter, not part of the proprietary adapter. OpenCode Go
is optional and must select a wire by **exact destination plus exact model**;
its catalog spans Chat, Responses, and Anthropic protocols.

## Evidence Baseline

| Repository | Revision audited | Method | Confidence |
|------------|------------------|--------|------------|
| Shunt | `e029ae2fb35eee9149855769a779343d7139d209` | Current shared working tree | HIGH |
| OpenCodex | `07b48da8fd63881e848d26e0bd50087864f5573e` (`upstream/main`) | `git show upstream/main:<path>` and `git ls-tree`; divergent checkout was not treated as source | HIGH |

OpenCodex is useful as current wire and regression evidence, not as an
architecture to port. Its current package depends on Bun, TypeScript, Zod,
`@bufbuild/protobuf`, MCP SDK, and a native keyring; none belongs in Shunt.

## Recommended Stack

### Core Technologies

| Technology | Resolved version | Purpose | Why Recommended |
|------------|------------------|---------|-----------------|
| Rust | edition 2021 | Gateway and protocol implementation | Existing ownership, memory safety, and explicit bounded state; changing languages would duplicate the gateway. |
| Axum | 0.8.9 | HTTP routes and response bodies | Already owns ingress, WebSockets, error surfaces, and streaming bodies. |
| Tokio | 1.52.3 | Async I/O, cancellation, deadlines, bounded channels | Existing adapters and shutdown semantics already use it; every new stream can share the same lifecycle rules. |
| reqwest | 0.12.28 with `rustls-tls`, `stream`, `http2`, `json` | Provider HTTP/SSE/NDJSON and Cursor Connect transport | Already negotiates HTTP/2 and exposes incremental body chunks; no second client or SSE library is needed. |
| serde / serde_json | 1.x / 1.0.150 | Request models, narrow envelope inspection, event parsing | Existing translation and configuration foundation; use typed outer envelopes and `Value` only where provider payloads are genuinely open-ended. |
| prost | 0.13.5 direct | Cursor protobuf messages | Already used by Cursor; `Message` supplies encode/decode without adding a Connect or generated-client runtime. |

The lockfile also contains newer transitive `reqwest` and `prost` versions.
New code must import Shunt's direct 0.12/0.13 dependencies rather than relying
on transitive 0.13/0.14 APIs.

### Existing Supporting Libraries

| Library | Resolved version | Purpose | When to Use |
|---------|------------------|---------|-------------|
| `bytes` | 1.12.1 | Incremental stream buffers | SSE, NDJSON, Connect frames, and bounded error bodies. |
| `futures-util` | 0.3.32 | Stream adapters | Relay upstream chunks without collecting successful streaming responses. |
| `http-body` / `http-body-util` | 1.x / 0.1.x | Axum body streaming | Preserve backpressure and attach RAII admission/account leases to response lifetime. |
| `sha2` | 0.10.9 | Domain-separated opaque session identifiers | Command Code and optional OpenCode Go affinity; never expose raw client thread IDs. |
| `uuid` | 1.23.4 (`v4`) | Fallback request/call IDs | Only when upstream/client identity is absent; stable conversation identity should be derived, not random per turn. |
| `url` | 2.5.8 | Canonical destination validation | Pin bearer-bearing proprietary transports to exact HTTPS origins and paths. |
| `base64`, `hmac`, `rand` | existing | OAuth/PKCE and opaque wire material | Reuse through `src/auth/shared.rs` and `src/auth/callback.rs`; do not add an OAuth framework. |
| `flate2` | existing | Cursor compressed Connect frames | Keep bounded offload and decompression-ratio checks already present. |
| `wiremock` | 0.6 (dev) | Provider transcript tests | HTTP, SSE, NDJSON, auth, retry, and truncation fixtures. |

### Shunt Components to Reuse

| Need | Existing component | Reuse decision |
|------|--------------------|----------------|
| Provider dispatch | `ProviderKind`, `AdapterKind`, `Route`, `Adapter` | Add narrow variants; do not replace with a dynamic plugin registry. |
| API-key injection | `AuthMode::ApiKey`, `api_key_env`, `ApiKeyHeader` | Use unchanged for generic Chat, Command Code API, and OpenCode Go. |
| OAuth mechanics | `auth/shared.rs`, `auth/callback.rs`, per-provider auth modules | Reuse callback, PKCE, locks, and atomic record handling; do not change existing credential writeback semantics. |
| Bounded SSE | `codex_endpoint::frame`, Responses/Gemini state machines | Extract/reuse framing mechanics, but keep protocol-specific event machines separate. |
| Streaming relay | reqwest `bytes_stream`, Axum `Body::from_stream` | Stream Chat, Gemini, and NDJSON output with backpressure; collect only when the client asked for non-streaming. |
| Retry/failover | `retry`, `upstream_status`, `AdapterFailure` | Retry only before visible output or side effects; provider event errors are not generic transport retries. |
| Account leases | `accounts`, `with_admission` | Hold selection/admission until the response body completes or drops. |
| Error sanitation | existing adapter error helpers and bounded readers | Parse recognized JSON fields, redact, cap, and never echo HTML or arbitrary upstream bodies. |
| Cursor wire | `adapters/cursor/{agent,connect,proto,stream,...}` | Extend in place; `prost`, HTTP/2, gzip bounds, tool encoding, and stream response code already exist. |
| Gemini/Antigravity wire | `adapters/gemini`, `model/gemini*`, `model/antigravity_request`, `auth/{google,antigravity}` | Preserve the common Code Assist envelope while keeping credentials, discovery identity, and provider policy distinct. |

## Minimal Rust and Configuration Additions

| Addition | Minimum shape | Reason |
|----------|---------------|--------|
| `ProviderKind::OpenAiChat` and `AdapterKind::OpenAiChat` | One new dispatch branch and `src/adapters/openai_chat/` | Chat Completions is neither Responses nor Anthropic and needs its own translation/terminal rules. |
| OpenAI Chat model module | Typed request compiler plus streaming/non-streaming response machine | Translate roles, system blocks, images, tool calls/results, finish reasons, usage, and malformed/truncated terminals without buffering streams. |
| `ProviderKind::CommandCode` and `AdapterKind::CommandCode` | Proprietary request compiler and bounded NDJSON parser | `/alpha/generate` is not OpenAI Chat despite serving some of the same models. |
| `AuthMode::CommandCodeOauth` | Only for the subscription preset | Keeps canonical-host bearer handling separate from API keys. Initial credential discovery should be read-only; any new Shunt-managed write flow needs explicit approval. |
| Two distinct presets | `command-code`: canonical subscription/proprietary wire; `commandcode`: API key/OpenAI Chat at `/provider/v1` | The names are physical identities, not aliases. Conflating them changes auth, endpoint, discovery, and response protocol. |
| Internal capability tables | Exact model aliases, effort ladders, modality/tool flags, endpoint profile | Keep time-sensitive provider facts out of branching logic. Do not expose speculative knobs. |
| Optional exact-wire table | `(canonical destination, exact model) -> adapter` | Supports OpenCode Go without a generalized model-adapter platform or family inference. |

These enum and preset changes alter public configuration/documented provider
semantics and therefore require the repository's explicit approval gate before
implementation. Avoid adding `command_code_version`, arbitrary proprietary
headers, or a public `model_adapters` map in the first slice. A tested internal
protocol profile is smaller and safer; add configuration only after a real
compatibility need is demonstrated.

## Provider-by-Provider Stack Decision

### ChatGPT/Codex and Gemini: Preserve

- Keep `ResponsesAdapter`, ChatGPT OAuth pools, HTTP/WS transports,
  continuation, quota admission, compaction, and existing Responses translators.
- Keep Gemini on the Code Assist `v1internal` envelope and its existing Google
  OAuth/project resolution.
- Add focused transcript tests and fail-closed terminal/tool-argument handling;
  no new provider abstraction or dependency is justified.

### Native Antigravity: Harden the Existing Gemini-Derived Vertical

- Keep `ProviderKind::Antigravity -> AdapterKind::Gemini` for the shared Cloud
  Code Assist envelope, but put Antigravity-only request mutations behind an
  explicit mode/profile.
- Reuse `sha2` for a stable thread-derived session ID, current catalog discovery,
  model mapping, schema sanitization, and signature validation/replay.
- Keep bearer token and project identity atomic across selection/refresh and pin
  bearer-bearing calls to canonical daily/prod HTTPS hosts.
- Always request the proven SSE upstream form; buffer through the same event
  machine only for a non-streaming client.
- Do not import OpenCodex's durable 24 MiB signature snapshot. Start with a
  bounded in-memory cache only if Shunt's existing encoded-call-id replay cannot
  cover a captured case; persistence is outside this milestone.

### Cursor: Harden In Place

- Retain Shunt's live `agentn.global.api5.cursor.sh/.../Run` evidence until a
  live/captured protocol-profile test proves another host. Current OpenCodex
  `upstream/main` still declares `api2.cursor.sh`, while Shunt records HTTP 464
  evidence for that older Run path; source constants alone cannot resolve the
  conflict.
- Add a small `CursorProtocolProfile` table for auth, discovery, and Run planes;
  they may have different hosts, RPC paths, client versions, and timeouts.
- Extend existing protobuf and state code with stable per-account conversation
  identity, checkpoint validation, bounded full replay, tool-result continuation,
  ordered structured error classification, and retry only before output or a
  tool side effect.
- Preserve the default denial of Cursor-native local execution. Caller prompt
  text must never authorize it.
- Do not add `connectrpc`, `tonic`, `prost-build`, or `@bufbuild` equivalents.

### Generic OpenAI Chat Completions: Add One Clean Vertical

- Build `POST <base>/chat/completions` with `Authorization: Bearer`, JSON, and
  `stream_options.include_usage = true` where accepted.
- Compile Anthropic system/user/assistant/tool history into Chat messages while
  preserving adjacent assistant tool calls and tool results.
- Implement a bounded SSE state machine for interleaved text/reasoning/tool-call
  deltas and parallel indexed calls. Reject incomplete JSON arguments and EOF
  during an open tool call. Permit provider-specific clean-EOF tolerance only
  from an exact tested profile.
- Keep optional provider quirks as compact tables. Do not port OpenCodex's broad
  repair/policy forest, hosted-search sidecars, pricing, catalog, or dashboard.

### Command Code: Two Paths

**Subscription (`command-code`)**

- Fixed canonical base `https://api.commandcode.ai`; `POST /alpha/generate`.
- Reuse existing JSON, streaming, hashing, URL, deadline, and auth primitives.
- Add bounded NDJSON framing that accepts the observed optional `data:` prefix,
  maps text/reasoning/whole tool-call/usage/finish/error events, makes the first
  terminal authoritative, and treats premature or junk-only EOF as failure.
- Preserve tool-call/result adjacency, synthesize explicit missing results,
  degrade orphan results to user-visible context, sort tools deterministically,
  and carry tool-result images in a following user message.
- Derive a stable, domain-separated `x-session-id`; do not use a shared prompt
  cache cohort as conversation identity.
- Read an existing Command Code credential source without modifying it. A
  Shunt-owned login/write path must reuse atomic-lock primitives but remains an
  approval-gated implementation decision.

**API key (`commandcode`)**

- Preset the generic OpenAI Chat adapter at
  `https://api.commandcode.ai/provider/v1` using `AuthMode::ApiKey`.
- Use bounded `/models` discovery only if model discovery is added as a general
  Shunt capability; it is not required for transport compatibility.
- Do not route this path through the proprietary NDJSON adapter or its OAuth
  lifecycle.

### Optional OpenCode Go: Exact Models Only

OpenCode Go's canonical base is `https://opencode.ai/zen/go/v1`. An initial
implementation should use an internal exact-wire allowlist scoped to that exact
HTTPS origin/path:

| Exact model(s) | Wire | Initial decision |
|----------------|------|------------------|
| `gpt-5.6-luna`, `grok-4.6` | Responses | Feasible after captured/live tests; existing Shunt Responses transport can be reused. |
| `muse-spark-1.3-contributor`, `muse-spark-1.2-contributor` | Responses upstream evidence | Defer until truncation/terminal behavior is captured; issue #2156 documents incomplete streaming tool calls in this family. |
| `minimax-m2.5`, `minimax-m2.7`, `minimax-m3` | Anthropic | Feasible only with exact transcript tests; use the existing Anthropic adapter. |
| All other discovered models | Chat by default | Do not claim support from family inference; require the new Chat adapter plus per-model evidence. |

If enabled, derive the `x-opencode-session` affinity header with existing
`sha2`, preserve an operator-provided header, and add it only for the exact
canonical destination. Do not introduce one provider-wide protocol choice.

## Installation

```bash
# No new dependency is recommended.
# Keep the existing Cargo.toml dependency set and feature flags.
cargo build
```

If implementation reveals a genuinely missing primitive, reassess before
adding a crate; the current audit found none.

## Alternatives Considered

| Recommended | Alternative | When the Alternative Would Be Justified |
|-------------|-------------|------------------------------------------|
| Handwritten bounded SSE/NDJSON machines using existing bytes/serde | `eventsource-stream`, `reqwest-eventsource`, or a generic NDJSON crate | Only if multiple new adapters prove the same tested parser semantics and the crate preserves explicit frame/total-time bounds. |
| Existing reqwest HTTP/2 + prost | `tonic`, `connectrpc`, generated clients | Only with a stable public Cursor schema and RPC surface broad enough to outweigh a second transport stack. |
| Existing auth primitives | OAuth framework or browser automation | Only for a standards-compliant provider whose flow cannot fit the existing PKCE/device/callback modules. Proprietary browser scraping is not such a case. |
| Internal exact model-wire table | Public generalized `model_adapters` configuration | Only after more than one supported provider needs operator-controlled per-model wire overrides and validation semantics are specified. |
| Provider-local state | SQLite/catalog platform | Only after measured durability requirements; no v2 target needs generalized request history. |
| Clean Rust implementation from behavior | Mechanical TypeScript-to-Rust translation | Use translation only for small, copyright-noticed algorithms where independent implementation would risk protocol drift. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Google AI Studio Web in any form | Explicitly excluded and known nonfunctional for this milestone | Gemini Code Assist and native Antigravity HTTP transports only. |
| Cookie/SAPISIDHASH auth, MakerSuite parser, browser extension, WebKit/Chromium daemon, or AI Studio session files | Expands credential and browser attack surface for an excluded transport | Existing Google OAuth and Antigravity OAuth modules. |
| Cookie-jar/headless-browser crates | No selected provider requires browser-session scraping | System-browser OAuth plus loopback/device callbacks. |
| Bun, Node, TypeScript, Zod, `@bufbuild/protobuf`, MCP SDK, native keyring | OpenCodex implementation dependencies, not Shunt protocol requirements | Existing Rust crates and Shunt abstractions. |
| New SSE/NDJSON parser dependency | Protocol subsets are small and require Shunt-specific bounds/terminal rules | Existing frame buffer pattern plus typed state machines. |
| `tonic`, `prost-build`, full ConnectRPC stack | Cursor's used wire is already encoded/decoded with `prost` and bounded framing | Extend `src/adapters/cursor/`. |
| Generic repair middleware | Can silently turn truncation or malformed tools into success | Provider-specific, transcript-backed fail-closed logic. |
| Durable Antigravity replay snapshot | Imports storage/writeback complexity and exceeds v2's no-persistence boundary | Existing signature carriage or bounded in-memory replay. |
| Unbounded live catalog/pricing machinery | Outside a lean gateway and creates stale global state | Static exact tested facts; add bounded discovery only where necessary. |
| Provider-family model inference for OpenCode Go | The same destination serves three protocols and model behavior changes independently | Exact canonical destination + exact model rules. |

## Version and Compatibility Notes

| Component | Compatible With | Notes |
|-----------|-----------------|-------|
| reqwest 0.12.28 | Rustls, streamed bodies, HTTP/2 | Direct dependency already has `stream` and `http2`; do not accidentally code against transitive reqwest 0.13. |
| prost 0.13.5 | Current Shunt Cursor message definitions | Direct dependency; transitive prost 0.14 must not leak into public/internal types. |
| Axum 0.8.9 + Tokio 1.52.3 | Existing server/body/cancellation model | No feature change needed for these provider additions. |
| ChatGPT/Codex WS | Existing pinned tungstenite forks | Provider v2 work does not justify changing the fork pins or publishing restriction. |
| Cursor | Volatile proprietary protocol profile | Pin host/path/client/schema facts in tests; update only from live/captured evidence. |
| Command Code subscription | Proprietary CLI-shaped NDJSON | Pin canonical host and tested headers internally; reject redirects carrying bearer credentials. |
| OpenCode Go | Mixed Chat/Responses/Anthropic | Compatibility is per exact model, not per provider. |

## Live-Test Feasibility

| Surface | Hermetic coverage | Live feasibility | CI posture |
|---------|-------------------|------------------|------------|
| ChatGPT/Codex | Existing HTTP/SSE/WS fixtures, account rotation, translation | Feasible with a test ChatGPT account; quota and backend behavior are account-dependent | Hermetic required; live opt-in only. |
| Gemini Code Assist | Existing request/stream/signature fixtures | Feasible with Google Code Assist OAuth/project | Hermetic required; live opt-in only. |
| Native Antigravity | Mock discovery, envelope, signature, model, auth/error tests | Feasible with an Antigravity account; hosts/catalog are volatile | Captured fixtures required; live opt-in, never credentialed CI. |
| Cursor | Protobuf/framing/tool/error/checkpoint transcripts | Feasible with standalone Cursor OAuth, but endpoint/profile conflict makes paired captures essential | Hermetic required; live smoke opt-in and quarantined. |
| Generic OpenAI Chat | wiremock JSON/SSE/truncation matrix | Feasible against any operator-provided compatible endpoint | Hermetic required; optional provider matrix. |
| Command Code API key | Mock Chat contract and exact preset URL/auth | Feasible with Provider-plan key | Hermetic required; live opt-in. |
| Command Code subscription | Mock `/alpha/generate`, NDJSON, whoami, expiry/error paths | Feasible with a consenting subscription account and read-only credential import | Hermetic required; live opt-in, never print/store captured secrets. |
| OpenCode Go | Exact model/wire fixtures | Feasible with an API key, but model roster and truncation vary | No broad CI claim; opt-in exact-model probes only. |

Captured transcripts must be scrubbed of bearer tokens, account/project IDs,
user content, and proprietary opaque values that are not necessary to exercise
the parser. Live success is evidence for one account/model/time, not a permanent
provider-family guarantee.

## Licensing and Provenance

Shunt is `MIT OR Apache-2.0`; OpenCodex is MIT. Behavioral compatibility and
independently written Rust tests do not require importing OpenCodex's TypeScript
architecture. If any substantial OpenCodex implementation or fixture is
translated, retain the OpenCodex copyright and MIT permission notice in the
appropriate third-party notice/provenance record. Record the exact source commit
beside imported fixtures. Do not copy credentials, user transcripts, generated
catalog dumps, or code from third-party bridges with unverified licenses.

## Sources

- Shunt `Cargo.toml`, `Cargo.lock`, `src/config.rs`, `src/config/presets.rs`,
  `src/routing.rs`, `src/adapters/`, `src/model/`, and `src/auth/` at the audited
  working-tree revision.
- [OpenCodex `upstream/main` source at `07b48da8`](https://github.com/lidge-jun/opencodex/tree/07b48da8fd63881e848d26e0bd50087864f5573e) - adapter registry, provider registry, Chat, Command Code, Antigravity, Cursor, and OpenCode Go behavior (HIGH).
- [OpenCodex MIT license](https://github.com/lidge-jun/opencodex/blob/07b48da8fd63881e848d26e0bd50087864f5573e/LICENSE) - provenance obligations (HIGH).
- [reqwest 0.12.28 documentation](https://docs.rs/crate/reqwest/0.12.28) - streamed bodies and HTTP/2 feature surface (MEDIUM, official crate docs; cross-checked against Shunt code).
- [prost 0.13.5 documentation](https://docs.rs/crate/prost/0.13.5) - derive and message encode/decode support (MEDIUM, official crate docs; cross-checked against Shunt code).
- [OpenCodex issue #909](https://github.com/lidge-jun/opencodex/issues/909) - Command Code API-key versus subscription distinction and original integration evidence (MEDIUM; implementation cross-check raises it to HIGH for current source behavior).
- [OpenCodex issue #2156](https://github.com/lidge-jun/opencodex/issues/2156) - OpenCode Go streaming tool-call truncation evidence (MEDIUM; do not generalize beyond affected models/wires).
- [OpenCodex issue #1162](https://github.com/lidge-jun/opencodex/issues/1162), [#1661](https://github.com/lidge-jun/opencodex/issues/1661), and [#1992](https://github.com/lidge-jun/opencodex/issues/1992) - Cursor model, tool bridge, and prompt-policy failure evidence (MEDIUM; current source/commit tests are the authority for implemented behavior).
- `.planning/PROJECT.md` and `.planning/notes/opencodex-port-audit.md` - milestone boundary, prior decisions, and exclusions.

---
*Stack research for: Shunt v2 Provider Compatibility*
*Researched: 2026-09-06*
