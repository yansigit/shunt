# Architecture Patterns

**Domain:** Protocol-translating LLM gateway provider compatibility
**Project:** Shunt v2 Provider Compatibility
**Researched:** 2026-09-06
**Overall confidence:** HIGH for Shunt integration boundaries; MEDIUM for undocumented proprietary wire details

## Executive Recommendation

Extend Shunt through two new protocol kinds, not provider-name branches:

1. Add a reusable `openai_chat` adapter for OpenAI-compatible Chat Completions.
2. Add a dedicated `command_code` adapter for the proprietary `/alpha/generate` NDJSON wire.

Keep presets, credentials, protocol adapters, and exact-model routing separate. `commandcode` (API key) should be a preset over the Chat adapter; `command-code` (subscription credential) should use the proprietary adapter and a distinct host-pinned auth mode. OpenCode Go should not become a monolithic adapter: it is a destination with exact model-to-wire assignments, so only verified models should reuse the Chat, Responses, or Anthropic adapters selected by exact routes.

Protect ChatGPT/Codex and Gemini with characterization tests and otherwise leave them structurally unchanged. Antigravity should remain on the Gemini/Cloud Code Assist transport with narrowly isolated policies. Cursor should remain its own ConnectRPC subsystem. None of these changes justifies a universal event bus, catalog platform, browser integration, or generalized persistence layer.

## Recommended Architecture

```text
POST /v1/messages
  -> bounded parse + inbound auth
  -> exact/prefix/default route chain
  -> capability filter (fallbacks only; preserve primary)
  -> per-attempt credential-slot hygiene
  -> AdapterKind dispatch
       Anthropic ---------> native Messages relay
       Responses ---------> Anthropic <-> Responses translation
       OpenAiChat --------> Anthropic <-> Chat Completions translation [new]
       Gemini ------------> Gemini / Code Assist / Antigravity CCA
       Cursor ------------> Cursor ConnectRPC translation
       CommandCode -------> /alpha/generate NDJSON translation       [new]
       AntigravityCli ----> deprecated local subprocess
  -> adapter reports exact failure boundary
  -> bounded pre-output failover
  -> Anthropic response/error contract
```

Routing selects a protocol contract; a preset supplies location and authentication. A provider name must never silently change the wire through model-family inference.

## Component Boundaries

| Component | Status | Responsibility | Communicates With |
|-----------|--------|----------------|-------------------|
| `src/config.rs` | Modify | Add `ProviderKind::OpenAiChat`, `ProviderKind::CommandCode`, and a read-only Command Code subscription auth mode; validate kind/auth/HTTPS/host pairs | presets, routing, credential resolver |
| `src/config/presets.rs` | Modify | Add `commandcode` Provider API Chat and `command-code` subscription presets without adapter logic | init scaffolding, config defaults |
| `src/routing.rs` | Modify | Map provider kinds to adapter kinds and preserve exact model mapping | dispatch, capabilities |
| `src/proxy/capability.rs` | Modify | Describe only proven incompatibilities for new adapters; filter fallbacks but never replace primary | route chain |
| `src/proxy/failover.rs` | Modify | Dispatch new adapters and preserve local-error/pre-header/upstream-status distinctions | all adapters |
| `src/adapters/openai_chat/` | New | Translate Messages to Chat and JSON/SSE back, with bounded incremental tool-call assembly | API-key resolver, response framer |
| `src/model/openai_chat_request.rs` | New | Pure roles/content/images/tools/tool-choice/model/effort mapping | Chat adapter, fixtures |
| `src/model/openai_chat_response.rs` | New | Pure JSON/SSE delta-to-Anthropic mapping, stop reason, usage | Chat adapter, fixtures |
| `src/adapters/command_code/` | New | Proprietary envelope/headers, tool adjacency, bounded NDJSON, terminal semantics | credential resolver, model helpers |
| `src/auth/command_code.rs` | New | Read bearer from env or existing CLI file; never create, refresh, migrate, or write | `auth::resolve_credential` |
| `src/adapters/gemini/mod.rs` | Modify narrowly | Keep shared Code Assist send/response path; apply Antigravity-only destination, envelope, stream, and retry rules | Antigravity auth/catalog/model |
| `src/model/antigravity_request.rs` | Modify narrowly | Stable session, model/tier, tool-history, schema, and signature policy | Gemini adapter |
| `src/adapters/cursor/*` | Modify narrowly | Preserve framing/client/stream split; harden construction, terminal classification, tool continuation, model evidence | Cursor OAuth, failover |
| `tests/` | Add/modify | Fixtures, mock upstreams, failover classification, secret-safe live smokes | every provider slice |
| README/docs/site locale trees | Modify per phase | Synchronize config, support status, security, troubleshooting | operators |

### Boundary Rationale

- Chat and Responses have different tool identity, reasoning, usage, terminal, and stream grammars; merging them produces fragile conditional parsing.
- Command Code is always NDJSON with a proprietary envelope. Treating it as Chat would contaminate a reusable adapter.
- Command Code's API-key product really is Chat-compatible at `/provider/v1/chat/completions`, so it should reuse Chat rather than copy translation.
- Antigravity and Code Assist share transport primitives, but client identity, destination, model catalog, session, signature, and tool history differ. Fork small policy functions, not the whole Gemini adapter.
- Cursor already has focused framing, request prep, response, and tool-bridge modules. Importing OpenCodex's larger runtime would also import execution and persistence responsibilities Shunt does not need.

## Protocol and Credential Data Flows

### Generic OpenAI Chat / Command Code Provider API

```text
Anthropic Messages JSON
  -> pure Chat request translation
       text/image -> Chat content parts
       tool_use -> assistant tool_calls
       tool_result -> tool message + matching tool_call_id
       tools/tool_choice -> Chat equivalents
  -> POST {normalized base}/chat/completions
       Authorization: Bearer $CONFIGURED_ENV_KEY
  -> stream=true: bounded SSE -> incremental Anthropic events
     stream=false: bounded JSON -> Anthropic message JSON
```

Normalize an API root to exactly one `chat/completions`; reject query/fragment ambiguity and avoid double `/v1`. Auth remains existing `AuthMode::ApiKey`. Because the gateway injects the operator's key, inbound credential slots must be stripped before dispatch.

### Command Code Subscription

```text
$COMMAND_CODE_TOKEN or read-only ~/.commandcode/auth.json
  -> bounded parse, non-empty apiKey only
  -> require HTTPS api.commandcode.ai (loopback allowed in tests)
  -> Credential::CommandCodeBearer

Messages -> proprietary envelope
  -> stable opaque x-session-id
  -> model/system/max_tokens/tools/stream:true
  -> tool-call/result adjacency repair
  -> POST /alpha/generate with identity headers
  -> bounded NDJSON
       text-delta / reasoning-delta / tool-call / finish / error
  -> Anthropic streaming or buffered output
```

The resolver may import an existing credential but must not add a callback, managed file, migration, refresh, or writeback. Read the file off the async worker, cap bytes, accept only the expected field, and never log its value. A distinct auth mode is required because subscription bearers must be host-pinned while generic API keys intentionally allow compatible custom destinations.

Before serialization, every assistant tool call must be followed by its result before a later user/assistant turn. Missing results become explicit error results; orphan results become user carriers. This matches the invariant demonstrated by OpenCodex issue #1383 and merged PR #1411 while preserving information.

Improve on OpenCodex's current EOF behavior: EOF without `finish`, `finish-step`, or explicit error is a typed protocol error, especially with an open/incomplete tool call. Retry/failover is permitted only before client-visible output.

### Antigravity

```text
existing Antigravity OAuth resolution
  -> token remains paired with discovered account/project
  -> canonical HTTPS CCA destination

Messages -> existing Gemini inner translation
  -> Antigravity-only exact catalog/tier
  -> stable session + schema/tool-history/signature rewrites
  -> CCA agent envelope
  -> always upstream streamGenerateContent?alt=sse
  -> one incremental parser
       streaming client: relay immediately
       non-streaming client: bounded accumulation
```

One upstream streaming contract avoids maintaining divergent unary/stream parsers. Buffer only because the inbound client requested non-streaming output.

Resolve and pin the OAuth destination before attaching the bearer. Redirects must not move it to another origin. Production-to-daily normalization needs unit coverage; quota/catalog traffic should use the same canonical-host proof without weakening private- or metadata-address rejection.

Token and project ID are one credential identity. Do not cache a project independently of its account/token. A process-lived keyed single-flight cache is preferable to constructing a fresh store per request, but changing the current refresh/write lock or credential writeback is outside approved scope. Transport/request hardening should land independently; lock-boundary work remains separately gated.

### Cursor

```text
existing Cursor OAuth
  -> HTTPS Cursor destination pin
  -> validate operator-derived headers before send
  -> build ConnectRPC frames on bounded worker
  -> POST AgentService/Run at Shunt's proven endpoint
  -> incremental session/thinking/text/tool/usage/end decoder
  -> Anthropic stream or bounded message
```

Do not replace Shunt's proven `agentn.global.api5.cursor.sh` Run endpoint merely because OpenCodex contains an `api2` constant. Endpoint changes require a live transcript or upstream confirmation. Keep model facts exact and request-local; never let process-global observations from one model reinterpret another request.

Header construction failures are local configuration errors (`failure: None`). Only genuine connect/send failures before headers are `BeforeHeaders`. HTTP/Connect errors retain upstream status and `Retry-After`. Never replay after output. Do not infer semantic progress from tool/text traffic; OpenCodex issue #3506 demonstrates a problem but not a safe gateway-owned heuristic.

### OpenCode Go Exact-Model Routing

```text
exact model -> exact adapter contract
  Chat       -> OpenAiChat, /chat/completions
  Responses  -> Responses, /responses
  Anthropic  -> Anthropic, documented Messages path
```

Shunt's `Route` points to one provider and thus one `ProviderKind`; it cannot safely switch adapter per model within an entry. Prefer separate provider presets per proven wire and exact `[[models]].upstream_model` mappings. Do not add generalized `model_adapters` until multiple supported destinations require it.

All OpenCode Go requests should carry a provider-scoped opaque `x-opencode-session` derived from existing inbound session/client identity. Destination-gate it; never add it to arbitrary compatible providers. Exact fixtures must pin model-specific reasoning replay, tool-name aliases, or field sanitation. Family inference is forbidden.

## Patterns to Follow

### Protocol Kind, Provider Preset, Exact Route

The adapter answers “what wire,” the preset “where/how to authenticate,” and the route “which exact model.” This preserves table-driven configuration and user-renamed providers.

```rust
match route.adapter {
    AdapterKind::OpenAiChat => OpenAiChatAdapter.forward(...),
    AdapterKind::CommandCode => CommandCodeAdapter.forward(...),
    // Existing adapters stay independent.
}
```

### Pure Translation Before I/O

Request and response conversion should be deterministic functions over JSON/events. URL construction, auth, retries, and body limits stay in adapters. This enables fixture tests and keeps secrets out of debug representations.

### Incremental Parser With Explicit Terminal State

Every parser tracks meaningful output, open calls, completed arguments, usage, and terminal signal.

```text
pre-output transient failure -> bounded retry/failover may proceed
terminal after output        -> close normally
EOF without terminal         -> typed stream error
open tool call + EOF         -> truncation/protocol error
```

### Credential-Class Host Pinning

Subscription credentials have typed auth variants and destination validators; only the matching adapter serializes them. Generic API keys remain configurable across custom compatible hosts.

### Characterization Before Hardening

Before shared enum/dispatch/Gemini edits, pin current ChatGPT/Codex, Gemini, Antigravity, and Cursor behavior: event order, header ownership, errors, terminal state, tool pairing, and failover. Live smokes skip with a reason when credentials are absent and never print credential contents.

## Anti-Patterns to Avoid

| Anti-pattern | Why harmful | Use instead |
|--------------|-------------|-------------|
| Provider-name branches in generic translators | Renames/aliases bypass them; quirks leak | Protocol kind or exact capability |
| One “OpenAI-compatible” Chat+Responses adapter | Different event/tool/reasoning contracts | Separate adapters |
| Buffering a streaming client to work around provider bugs | Breaks streaming semantics and memory bounds | Fail closed on truncation |
| Porting OpenCodex persistence/browser/tool runtime | Expands security/lifecycle ownership; violates no-writeback scope | Port wire invariants and fixtures |
| Cursor no-progress heuristic | Gateway cannot know semantic progress; replay risks side effects | Explicit client marker in a future scope |
| Whole-provider OpenCode Go inference | Mixed wires and model-specific defects | Exact evidence-backed model routes |
| Google AI Studio Web integration | Explicitly excluded and not Code Assist | Keep Gemini on Code Assist; add no code/tests/docs/deps |

Google AI Studio Web includes cookie/SAPISIDHASH auth, browser extensions/daemons, MakerSuite parsing, session synchronization, and WebKit. It must not be implemented, tested, documented as supported, or cause dependencies to be added.

## Dependency-Aware Build Order

### A. Existing-provider conformance baseline

Add ChatGPT/Codex Responses and Gemini Code Assist characterization fixtures, plus reusable test helpers for split SSE/NDJSON chunks and secret-redaction assertions. This must precede central enum, dispatch, and shared Gemini changes.

### B. Antigravity selective hardening

Stabilize session identity from trusted request metadata where available; centralize destination and upstream SSE selection; add bounded pure signature/history/schema rewrites; pin exact catalog/tier and credential/project affinity. Defer refresh-lock/writeback restructuring.

### C. Cursor selective hardening

Correct local versus transport failure classification; pin endpoint/header/model behavior; harden continuation/tool pairing and terminal/truncation behavior. Explicitly exclude no-progress heuristics and writeback.

### D. Generic OpenAI Chat vertical slice

Add enum/dispatch, pure request translation, bounded JSON response mapping, incremental SSE parsing, capability/failover integration, and API-key auth. Then add `commandcode` and optional Vercel Chat presets as configuration-only consumers.

### E. Command Code proprietary subscription

Add host-pinned read-only auth, envelope/headers/session, adjacency normalization, and bounded fail-closed NDJSON parsing. Cover missing results, malformed frames, error finishes, and truncated EOF; add secret-safe live smoke when credentials exist.

### F. Exact OpenCode Go evaluation

Obtain dated docs or redacted live/captured transcripts per model. Record endpoint/auth/session/request/terminal/tool/reasoning facts, then implement only passing pairs by reusing existing adapters and exact presets/routes. Ambiguous or failing pairs remain unsupported.

### G. Cross-provider closeout

Run format, Clippy, full workspace tests, safe live smokes, auth/redirect/bounds/post-output-retry review, and documentation parity checks. Documentation ships within each behavior phase; closeout verifies it. Never hand-edit `wiki/`.

## Scalability and Resource Considerations

| Concern | Concurrent risk | Required invariant |
|---------|-----------------|--------------------|
| Request translation | Large histories/schemas consume CPU | Linear passes; bounded depth |
| SSE/NDJSON | Slow/malformed upstream retains memory/tasks | Frame/body caps and total timeout |
| Tool assembly | Parallel calls retain argument JSON | Per-call and total pending-byte caps |
| Auth/project discovery | First-request storm duplicates discovery | Account-scoped single-flight; no expanded global lock |
| Retry/failover | Attempts multiply across chains | One bounded budget, pre-output only, capped `Retry-After` |
| Session affinity | Many conversations grow state | Fixed-size opaque derivation; no raw IDs in logs/caches |
| Live tests | CI lacks credentials | Safe skip; never print secrets |

## Architectural Decisions

| Decision | Recommendation | Confidence |
|----------|----------------|------------|
| Generic Chat | New `openai_chat` kind/adapter | HIGH |
| Command Code API | Preset over Chat | HIGH |
| Command Code subscription | Dedicated adapter + host-pinned read-only auth | HIGH boundary / MEDIUM version details |
| Antigravity response mode | Always consume CCA SSE; buffer only for unary client | HIGH |
| Antigravity lock/writeback | Do not alter without separate approval | HIGH |
| Cursor endpoint | Retain proven Shunt endpoint; harden in place | HIGH |
| Cursor no-progress | No speculative heuristic | HIGH |
| OpenCode Go | Exact model/wire routes only | HIGH |
| Vercel | Keep Anthropic; optional Chat preset after Chat adapter | HIGH |
| Google AI Studio Web | Exclude code, tests, docs, dependencies | HIGH |

## Open Questions and Research Flags

- Command Code sends a versioned `x-command-code-version` in OpenCodex. Validate the current value with a live/captured request before documenting a default.
- Choose and document Command Code credential precedence (dedicated env versus local CLI file); both remain read-only.
- Antigravity cross-restart signature replay would be new persistence and remains excluded without a failing transcript and new authority.
- Every OpenCode Go roster row needs dated exact-model evidence; do not generalize sibling models.
- OpenCode Go history shows both complete calls lacking terminal frames and real mid-tool truncation. Stay strict until an exact transcript proves a safe EOF predicate.
- Shunt discovery is configuration-driven. Do not build a generalized live catalog just for this milestone; separately evaluate a bounded `/models` probe only if explicit routes are inadequate.

## Evidence and Sources

Primary code was inspected at Shunt commit `e029ae2fb35eee9149855769a779343d7139d209` and OpenCodex `upstream/main` commit `07b48da8fd63881e848d26e0bd50087864f5573e`. Source is authoritative for current implementation shape; issues are evidence, never instructions.

- [Shunt provider kinds/auth validation](https://github.com/yansigit/shunt/blob/8c6bc8c5d75cd87465c43c707234f95f0aadeeb0/src/config.rs)
- [Shunt dispatch/failover](https://github.com/yansigit/shunt/blob/8c6bc8c5d75cd87465c43c707234f95f0aadeeb0/src/proxy/failover.rs) and [capability filter](https://github.com/yansigit/shunt/blob/8c6bc8c5d75cd87465c43c707234f95f0aadeeb0/src/proxy/capability.rs)
- [Shunt Responses](https://github.com/yansigit/shunt/tree/8c6bc8c5d75cd87465c43c707234f95f0aadeeb0/src/adapters/responses), [Gemini/Antigravity](https://github.com/yansigit/shunt/blob/8c6bc8c5d75cd87465c43c707234f95f0aadeeb0/src/adapters/gemini/mod.rs), and [Cursor](https://github.com/yansigit/shunt/tree/8c6bc8c5d75cd87465c43c707234f95f0aadeeb0/src/adapters/cursor)
- [OpenCodex Command Code adapter](https://github.com/lidge-jun/opencodex/blob/07b48da8fd63881e848d26e0bd50087864f5573e/src/adapters/command-code.ts) and [credential flow](https://github.com/lidge-jun/opencodex/blob/07b48da8fd63881e848d26e0bd50087864f5573e/src/oauth/command-code.ts)
- [OpenCodex OpenCode Go registry](https://github.com/lidge-jun/opencodex/blob/07b48da8fd63881e848d26e0bd50087864f5573e/src/providers/registry.ts) and [session transport](https://github.com/lidge-jun/opencodex/blob/07b48da8fd63881e848d26e0bd50087864f5573e/src/providers/opencode-go-transport.ts)
- [Command Code issue #1383](https://github.com/lidge-jun/opencodex/issues/1383) and [repair PR #1411](https://github.com/lidge-jun/opencodex/pull/1411)
- [OpenCode Go wire issue #3378](https://github.com/lidge-jun/opencodex/issues/3378), [reasoning issue #950](https://github.com/lidge-jun/opencodex/issues/950), and [truncation issue #2156](https://github.com/lidge-jun/opencodex/issues/2156)
- [OpenCodex Cursor issue #3506](https://github.com/lidge-jun/opencodex/issues/3506) and [Shunt Cursor issue #275](https://github.com/pleaseai/shunt/issues/275)
- [Shunt Antigravity effort #336](https://github.com/pleaseai/shunt/issues/336), [refresh lock #373](https://github.com/pleaseai/shunt/issues/373), and [discovery #383](https://github.com/pleaseai/shunt/issues/383)
- [OpenCodex Antigravity destination issue #3781](https://github.com/lidge-jun/opencodex/issues/3781)

## Confidence Assessment

| Area | Confidence | Basis |
|------|------------|-------|
| Shunt boundaries/dispatch | HIGH | Direct current Rust source/tests |
| Generic Chat split | HIGH | Stable protocol distinction and current adapter design |
| Command Code API | HIGH | Registry and endpoint evidence agree |
| Command Code proprietary wire | MEDIUM | Direct source/issues, but proprietary/versioned |
| Antigravity boundaries | HIGH | Current native implementation plus concrete issues |
| Cursor boundaries | HIGH | Current implementation plus reproducible issue evidence |
| Exact OpenCode Go mapping | MEDIUM | Exact registry/issues; mutable per-model roster |
| Google AI Studio Web exclusion | HIGH | Explicit project boundary |
