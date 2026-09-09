# Phase 11: Antigravity Protocol and Credential Hardening - Pattern Map

**Mapped:** 2026-09-07  
**Files analyzed:** 14 tracked implementation, test, and documentation files  
**Analogs found:** 14 / 14

This map covers the native HTTP Antigravity path only. `src/adapters/antigravity/*`
and `tests/antigravity_process.rs` are deliberately not implementation analogs:
they exercise the deprecated local `agy` process transport.

## File Classification

| New/Modified File | Role | Data Flow | Closest tracked analog | Match quality |
|---|---|---|---|---|
| `src/adapters/gemini/mod.rs` | provider/controller | request-response + streaming transform | same file (Gemini adapter) | exact |
| `src/adapters/gemini/sse.rs` | utility/parser | incremental streaming | same file (Gemini SSE decoder) | exact |
| `src/auth/antigravity/auth.rs` | service/auth transport | request-response, redirect-boundary I/O | same file (Antigravity auth store) | exact |
| `src/auth/antigravity/catalog.rs` | service/cache | request-response + bounded cache | same file (catalog discovery) | exact |
| `src/auth/shared.rs` | middleware/utility | request-response redirect policy + file I/O | same file (safe OAuth client) | exact |
| `src/auth/mod.rs` | model/dependency seam | request-response, request-local ownership | same file (`CredentialResolver`) | exact |
| `src/server.rs` | route/controller | request-response + RAII lifetime | same file (router/dependency injection tests) | exact |
| `src/model/antigravity_request.rs` | transform/model | request-response JSON transform | same file (agent envelope/model resolver) | exact |
| `src/model/gemini.rs` | semantic state machine | incremental streaming transform | same file (`GeminiSseMachine`) | exact |
| `src/retry.rs` | policy utility | request-response retry state | same file (`RetrySafety`/`Commitment`) | exact |
| `tests/antigravity_catalog.rs` | integration test | real gateway request-response | same file (native catalog wiring) | exact |
| `tests/gemini_conformance.rs` | integration/conformance test | loopback streaming + bounded body | same file (Gemini gateway fixtures) | exact |
| `tests/antigravity_translate.rs` | unit/integration test | pure transform + event stream | same file (legacy CLI only; use only fixture style) | partial |
| `docs/notes/antigravity-daily-host.md` | engineering documentation | static evidence/decision record | same file | exact |
| `site/src/content/docs/providers/antigravity.mdx` (and locale copies) | user documentation | static configuration/how-to | same English/locale page set | exact |

## Pattern Assignments

### `src/adapters/gemini/mod.rs` (provider, request-response + streaming)

Use the existing `forward` vertical flow as the primary template (lines 191-317):
resolve one credential at the start, translate the inner Gemini request, choose the
provider endpoint/envelope, and retain endpoint/payload/token in a request closure
(lines 319-357). Antigravity currently selects `streamGenerateContent?alt=sse`
only when downstream `stream` is true (lines 228-239); Phase 11 should change this
single decision point to always-SSE while leaving normal Gemini and API-key paths
unchanged. Keep the existing split between request construction and response parsing.

For streaming, copy the lazy ownership pattern at lines 375-520: `bytes_stream`,
`GeminiSseDecoder`, and `GeminiSseMachine` are moved into `Body::from_stream`; each
decoder item is checked before events are emitted, and decoder/machine/EOF errors
become protocol error events. For downstream non-streaming, retain the bounded
collector at lines 521-535 (`collect_unary_response` followed by JSON parse and
`transport_close_checked`). The Phase 11 implementation should feed both modes from
the same SSE decoder/machine rather than introducing a second translator.

### `src/adapters/gemini/sse.rs` (utility, incremental streaming)

`Decoder` (lines 13-97) is the exact bounded parser pattern: explicit buffer,
`max_event_bytes`, `done`, and `disposed` state; `push_one` returns a consumed count
so packet tails remain zero-copy; `finish` rejects an unterminated frame. Reuse its
byte-oriented UTF-8 and JSON checks (`parse_frame`, lines 117-143), `[DONE]` rejection
and event-size cap. Add Antigravity wrapper rules around this parser/state machine,
not a provider-specific unbounded parser.

### `src/model/gemini.rs` (semantic state machine, streaming transform)

`GeminiSseMachine::new_for_upstream` and its streaming variant (lines 116-176)
show how to separate output retention from incremental relay via
`accumulate_content`. `process_chunk_checked` validates role, candidate, parts,
usage, and terminal state; retain its checked/monotonic approach. The part validator
(lines 398-528) is the closest tool/signature analog: reject incompatible semantic
keys and malformed function calls, require non-empty Gemini-3 signatures, enforce
signature/id/content bounds, then return typed checked parts. Error projection is
centralized in `protocol_error_event` and `translate_gemini_error_val` (lines
901-970). Extend these private invariants for Antigravity wrappers and account/session
signature binding instead of duplicating a machine.

### `src/model/antigravity_request.rs` (model, JSON transform)

`wrap_antigravity_envelope` (lines 38-71) is the wire-shape template: clone/mutate
the inner request to insert `sessionId`, then emit only `model`, `project`,
`userAgent`, `requestType`, `requestId`, and `request`. `antigravity_request_id`
(lines 73-77) demonstrates fresh `agent-<uuid-v4>` IDs. The current
`antigravity_session_id` (lines 79-124) hashes first user text with bounded FNV or
random fallback; replace the prompt-only identity with an opaque account/conversation
scope while preserving bounded output and no secret/prompt exposure. The model
resolver below this section is table-driven around catalog IDs and thinking tiers;
preserve exact tuple admission and explicit-effort precedence rather than adding
name heuristics.

### `src/auth/mod.rs` (credential model/dependency seam, request-response)

`Credential` and its redacted `Debug` implementation (lines 29-85) are the model and
secret-leak pattern: keep bearer/project/account metadata out of diagnostics.
`CredentialResolver` plus `DefaultCredentialResolver` (lines 88-115) is the
crate-private injection seam. `AppState::resolve_route_credential` delegates once
through that seam (`src/server.rs:147-154`); extend the private Antigravity tuple at
this boundary without changing public config or credential-file schema.

### `src/auth/antigravity/auth.rs` (auth service, redirect-boundary I/O)

`AntigravityAuthStore::new` (lines 250-293) normalizes trailing slashes and keeps
loopback proxies in scope while mapping the production host to the daily control
plane. `refresh_call` (lines 513-537) demonstrates bounded timeout, form POST, and
the deliberate use of the hardened shared token client for credential-bearing calls.
`inference_base_url` and `addresses_bare_backend` (lines 1161-1200) are the closest
URL predicate patterns: parse URLs, require exact scheme/host/port/path shape, and
preserve explicit proxy ports/paths. Phase 11 should extract/reuse this fail-closed
predicate at both initial inference and redirect hops; never log raw bearer values.

### `src/auth/shared.rs` (redirect middleware/utility, request-response + file I/O)

`sanitize_token_url` and `admin_token_url_override` (lines 130-171) validate parsed
URLs before use. `token_refresh_client` (lines 173-191) is the direct redirect policy
analog: `Policy::custom` rejects unsafe/excessive redirects before following and
allows only `is_safe_refresh_url`. Copy its boundary-before-bearer ordering for the
Cloud Code Assist client, with exact canonical origins rather than suffix matching.
`write_auth_file_atomic` (lines 193-198) is the preservation pattern: do not alter
credential writeback or introduce a migration.

### `src/auth/antigravity/catalog.rs` (catalog service/cache, bounded request-response)

The cache key and freshness model (lines 46-120) show account/project separation,
truncated SHA-256 only for projectless bearer fingerprints, and explicit `fresh`
metadata. `catalog_ids` (lines 186-257) uses a per-key async fetch slot, re-reads
after awaiting, and records failures without holding a blocking lock. `fetch_catalog`
(lines 259-325) applies bounded whole-fetch timeout, bearer + Antigravity user-agent
headers, drains error bodies, and parses only the `models` object. Keep these
single-flight and bound patterns, but change the caller's fail-open behavior to
authoritative fresh exact admission: unknown/stale-only negatives must stop before
credential/network inference.

### `src/retry.rs` (policy utility, retry state)

`Commitment` (lines 102-130) is the monotonic replay boundary: client-visible output
or tool activity makes `may_redispatch()` false. `RetrySafety` (lines 132-146) and
`send_with_retry_with_safety` (lines 193-214) distinguish response-status retries
from pre-header transport retries and never inspect a response body. Implement the
Antigravity 401 behavior as a narrow account-bound state machine around this policy:
one pre-commit refresh/replay only, no replay after headers/output/tool/body/parser
failure, and no general change to unrelated providers.

### `src/server.rs` (route/controller, request-response + RAII lifetime)

Use `AppState::resolve_route_credential` (lines 147-154) and
`from_shared_with_dependencies` (lines 156-170) for request-local dependency
capture. The Phase 10 real-router tests around lines 727-746 build a router with an
injected resolver, call `Router::oneshot`, consume the body, and assert exact upstream
identity. The retry and cancellation fixture around lines 748-790 demonstrates a
custom DNS resolver and genuine connect refusal; the capacity/drop fixture in
`tests/gemini_conformance.rs:830-893` proves upstream body release after downstream
drop. Extend these fixtures for Antigravity account/project/session stability and
refresh lifetime, retaining real router/body consumption and bounded timeouts.

### `tests/antigravity_catalog.rs` (integration test, real gateway request-response)

The test builds a `wiremock::MockServer`, mounts catalog and inference POSTs, writes
synthetic OAuth JSON in a temp directory, injects `SHUNT_ANTIGRAVITY_AUTH_FILE`, and
starts the actual Axum server (`tests/antigravity_catalog.rs:65-153`). It then
inspects `received_requests` and the parsed envelope (`lines 156-171`). Copy this
for fresh/ambiguous/stale catalog admission, asserting zero inference hits and no
Authorization header on rejected combinations. Keep fixture credentials synthetic
and restore environment state with `EnvVarGuard` (lines 19-43).

### `tests/gemini_conformance.rs` (integration/conformance, loopback streaming)

Use `start_gateway_with_config` and loopback `Router` helpers (lines 50-94) for
isolated real-gateway tests. `HeldStreamState` and `held_gemini_stream`
(`lines 781-802`) provide the body-drop signal; the capacity test (`830-893`) proves
incremental output, saturation, stream drop, and reacquisition. Framing and bounds
tests (`895-940`) show how to split SSE bytes, assert terminal/error semantics, and
exercise exact-cap versus cap+1. Adapt these helpers to verify always-SSE parity,
malformed wrapper/EOF rejection, bounded tool/signature retention, and one-shot 401
replay.

### `tests/antigravity_translate.rs` (test fixture style only, partial analog)

This target is explicitly for the deprecated CLI adapter. Its table-driven effort
and terminal tests (for example `tests/antigravity_translate.rs:58-181` and
`326-377`) are useful only as style examples for synthetic inputs and explicit
unsupported-effort assertions. Do not use its process spawning, CLI model discovery,
or permissive line parser as native HTTP evidence.

### Documentation files

Use `docs/notes/antigravity-daily-host.md` as the dated engineering-note pattern:
state the observed host distinction and confidence limits, then record the exact
invariant and hermetic proof rather than implying live backend coverage. Update the
English provider page and the tracked `ko`, `ja`, and `zh-cn` copies under
`site/src/content/docs/*/providers/antigravity.mdx` in parallel; preserve existing
warnings about Antigravity terms and the deprecated CLI. Never edit generated
`wiki/`.

## Shared Patterns

- **Fail closed at boundaries:** parse and validate exact URL origin before attaching
  bearer; reject unsafe redirect attempts in the redirect policy callback.
- **Request-local ownership:** resolve one immutable account/token/project/catalog/
  session tuple, move upstream body/parser into the response stream, and let RAII
  drop release all state on cancellation.
- **One checked semantic path:** incremental byte decoder → checked semantic machine
  → streaming relay or bounded non-streaming accumulation; terminal/EOF errors are
  never converted into success.
- **Monotonic replay safety:** use `Commitment` vocabulary; only pre-commit transport
  or explicitly permitted pre-header 401 can replay, and a refresh cannot change the
  selected project/catalog/session.
- **Synthetic, exact integration evidence:** real Axum/loopback or wiremock fixtures,
  per-test temp files and environment guards, exact request/body/header assertions,
  and bounded timeout/drop checks. No live credentials or production home access.

## Anti-Patterns to Avoid

- Do not copy `src/adapters/antigravity/*` process/CLI behavior into native HTTP.
- Do not retain the current prompt-only global session hash or catalog fail-open
  negative admission.
- Do not infer model/effort from names, clamp unsupported effort, or rewrite Claude/GPT
  IDs into Gemini IDs.
- Do not use free-following redirects, bearer-bearing lookalike origins, detached
  refresh tasks, durable signature state, or post-output retries.

## PATTERN MAPPING COMPLETE
