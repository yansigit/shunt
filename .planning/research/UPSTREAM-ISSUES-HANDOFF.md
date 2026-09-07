# Upstream Provider Issues and Porting Handoff

**Researched:** 2026-09-06/07  
**Method:** parallel GPT-5.6 Luna research agents, direct `gh search issues`, `gh issue view`, local OpenCodex source inspection, and primary-source web search.  
**Scope:** remaining v2 provider work after Phase 9 and the completed semantic portion of Phase 10.

## Executive decision

OpenCodex is worth using as an evidence and implementation-pattern source, but it is not safe to port provider adapters wholesale. Port narrow, testable invariants: destination binding, account/project affinity, request-local sessions, bounded pre-output retry, strict terminal settlement, tool-call identity, exact-model wire selection, catalog exclusion, and bounded parsing. Do not port global mutable model evidence, heuristic no-progress cancellation, cross-account project reuse, permissive EOF, unbounded retry, guessed effort defaults, or repair rules that erase semantic distinctions.

The user's provider inventory is almost right:

- Existing Shunt providers needing hardening, not creation: Gemini, Antigravity, Cursor, Responses/Codex, and generic Anthropic/Vercel.
- New named products: `opencode-go`, Command Code API-key (`commandcode`), and Command Code subscription (`command-code`).
- Also missing is a reusable generic `openai_chat` provider kind/transport. It is not merely an alias: Phase 13 must establish the Chat Completions translation contract before the API-key Command Code product can reuse it.
- Antigravity native HTTP is a new transport for an existing provider. The CLI transport remains until parity is proven.

## Immediate Phase 10 finding

Three of the four previous verification gaps were closed by Plan 10-06: tool results are ID-addressed, known unsupported Gemini Parts fail closed, and every completed frame after `[DONE]` is rejected. Plan 10-07 is still incomplete.

The partial implementation proves unary, streaming, and cancellation cases, but the retry case currently fails:

```text
cargo test --all-features gemini_google_oauth_code_assist_lifetime -- --test-threads=1
2 passed; 1 failed
gemini_google_oauth_code_assist_lifetime_pre_header_retry_reuses_identity_and_body: expected 200, received 502
```

The loopback peer resets the first connection. `reqwest::Error::is_connect()` is false for the resulting `SendRequest/ConnectionReset`, so the shared retry classifier does not redispatch. The tree also contains a temporary `eprintln!("RETRY_DEBUG ...")` in `src/retry.rs`; remove it before any commit. The test uses deprecated `TcpStream::set_linger` and therefore fails the warnings-denied gate even if its assertion is fixed. The successor must decide whether the fixture should model a classifier-approved pre-header failure differently or whether the shared transient classifier should deliberately include this exact reset class. That decision must be made against Phase 9 retry safety, not just to make the test green.

## Gemini and Google Code Assist evidence

| Evidence | State | Lesson for Shunt |
|---|---|---|
| [pleaseai/shunt#233](https://github.com/pleaseai/shunt/issues/233), [PR #234](https://github.com/pleaseai/shunt/pull/234) | closed/merged 2026-07-24 | Code Assist OAuth uses `loadCodeAssist` project discovery and `/v1internal`; Gemini Code Assist and Antigravity hosts/model namespaces are distinct. |
| [pleaseai/shunt#237](https://github.com/pleaseai/shunt/issues/237), [PR #240](https://github.com/pleaseai/shunt/pull/240) | closed/merged 2026-07-28 | Preserve the exact opaque thought signature on the same call. Parallel Gemini calls carry it only on the first call. Post-fix live continuation was quota-blocked, so deterministic evidence must not be described as live proof. |
| [pleaseai/shunt#373](https://github.com/pleaseai/shunt/issues/373) | open | A global refresh lock spanning refresh, file I/O, discovery, and onboarding can hold requests for about seven minutes. Network discovery needs separate single-flight/cache behavior. |
| [pleaseai/shunt#383](https://github.com/pleaseai/shunt/issues/383) | open | Fresh per-request auth stores can duplicate concurrent `loadCodeAssist`/onboard calls and trigger rate limits. |
| [pleaseai/shunt#384](https://github.com/pleaseai/shunt/issues/384), [PR #416](https://github.com/pleaseai/shunt/pull/416) | fixed | Project-ID merge/write must not overwrite a newly rotated refresh token. This is useful future evidence, but credential writeback changes remain outside the current authorization. |
| [pleaseai/shunt#385](https://github.com/pleaseai/shunt/issues/385) | open | OAuth token endpoint bodies are read without a cap. Add a bounded collector in a separately authorized hardening slice. |
| [PR #390](https://github.com/pleaseai/shunt/pull/390) | merged 2026-08-18 | Project discovery and generation must use the same configured endpoint; avoid split-host identity. The PR called out a residual analogous Google OAuth path. |
| [PR #454](https://github.com/pleaseai/shunt/pull/454) | merged 2026-09-04 | Gemini tuple-schema keywords can invalidate the entire request. Retain deterministic schema sanitization tests and do not overstate live confirmation. |
| [Google thought signatures](https://ai.google.dev/gemini-api/docs/generate-content/thought-signatures) | official, updated 2026-09-04 | Exact signature replay is mandatory for Gemini 3; parallel calls sign only the first part; all calls precede grouped results; interleaving calls/results is invalid. |
| [Google Gemini CLI Code Assist server](https://github.com/google-gemini/gemini-cli/blob/main/packages/core/src/code_assist/server.ts) | official source | `loadCodeAssist` binds the returned `cloudaicompanionProject` to subsequent credit and generation activity. |

No public Google document defines the complete private `/v1internal` refresh/lifetime contract. The correct Phase 10 claim is hermetic real-router evidence, not official or live authenticated proof.

## Antigravity evidence and selective port

Important current Shunt issues:

- [#368](https://github.com/pleaseai/shunt/issues/368): native HTTP is still an open proposal. It requires dedicated OAuth scopes/metadata, onboarding fallback, exact agent envelope, signatures, schema sanitization, max-output rules, and migration compatibility.
- [#305](https://github.com/pleaseai/shunt/issues/305), [#336](https://github.com/pleaseai/shunt/issues/336), [#337](https://github.com/pleaseai/shunt/issues/337): guessed effort defaults are unsafe. Cold/unknown models must be discovered or rejected; per-request effort needs explicit precedence.
- [#467](https://github.com/pleaseai/shunt/issues/467): any non-empty preamble can incorrectly forgive an undelivered CLI handoff and synthesize success.
- [#308](https://github.com/pleaseai/shunt/issues/308): process-path quota scraping no longer matches the installed app; prefer provider API quota evidence.
- [PR #470](https://github.com/pleaseai/shunt/pull/470): profile isolation is still under review, with relative-path, blocking-I/O, runtime-env-test, and locale-doc concerns. Do not build on it as settled behavior.

Worth porting from OpenCodex:

- Account-bound project validation and failover-only, bounded session affinity.
- Explicit cooldown taxonomy, capped `Retry-After`, and at most one short same-account pre-commit retry.
- HTTPS destination allowlists, host-specific envelopes, API quota probing, and project re-binding after an OAuth refresh.
- Exact thought-signature replay and strict tool-call/result identity.

Do not port:

- User-visible unrestricted account spreading or cross-account project reuse; official Antigravity docs describe profile login/logout and project/conversation association, not multi-account rotation.
- Name-based effort guesses, process scraping, unbounded retry, post-output replay, or synthetic clean completion.

Phase 11 should implement strict SSE/EOF/terminal and cancellation first; then account/token/project lifetime, narrow discovery single-flight, exact catalog/effort discovery, signature/tool ordering, bounded retry, and API quota probes. Keep account rotation an explicit policy rather than an inferred feature.

## Cursor evidence and selective port

Current Shunt risks:

- [pleaseai/shunt#29](https://github.com/pleaseai/shunt/issues/29): the tool bridge buffers the full response and has unresolved session-registry design.
- [#275](https://github.com/pleaseai/shunt/issues/275): local request/header construction failures are misclassified as transport failures and can advance ordered failover.
- [#426](https://github.com/pleaseai/shunt/issues/426): built-in tool-call detection still needs measured coverage.
- The current agent.v1 path discards inbound session headers, uses process-global CWD, treats four seconds of post-output silence as normal completion, has no pre-response retry, and hardcodes non-stream usage to 1/1 while omitting reasoning/session evidence.

Worth porting from OpenCodex:

- Authoritative Connect `END_STREAM`/`turnEnded` settlement and incomplete-frame failure.
- Credential-scoped conversation/session identity and clearing it when credentials rotate.
- Retry/rotation only before client-visible output; never after tool/output commitment.
- Bounded tool catalogs and explicit native/client tool namespace separation.
- Preserve opaque call IDs unless authenticated provenance proves a Shunt/OpenCodex-owned encoding.

Do not port:

- Old endpoint/framing assumptions: Shunt intentionally uses newer `agent.v1.AgentService/Run` on the api5 host.
- Process-global model evidence or the unresolved heuristic no-progress loop tracked by [OpenCodex #3506](https://github.com/lidge-jun/opencodex/issues/3506).
- Silent execution of Cursor built-ins; Shunt should continue failing explicitly until faithful execution exists.

Phase 12 P0 tests: clean terminal with held-open body, error terminal, zero-frame EOF, incomplete protobuf, post-output silence, pre-output-only retry/rotation, credential/session clearing, per-request CWD/session, opaque tool identity, catalog bounds, and no bearer leakage.

## Generic OpenAI Chat and Command Code

Phase 13 is independently necessary. A generic Chat Completions provider must own `/chat/completions`, provider-bound bearer auth, indexed streaming `tool_calls`, paired result IDs, images, bounds, error mapping, cancellation, and exactly one terminal. It must reject Responses-only fields instead of forwarding an accidental hybrid.

Command Code is two products, not one adapter:

- API-key `commandcode`: reuse Phase 13 Chat at canonical `/provider/v1/chat/completions`.
- Subscription `command-code`: proprietary bearer/session transport at `/alpha/generate`, with NDJSON parsing and its own terminal/tool/result rules.

Primary OpenCodex issue evidence:

- [#909](https://github.com/lidge-jun/opencodex/issues/909): introduced the two-product distinction.
- [#1383](https://github.com/lidge-jun/opencodex/issues/1383) and related #1411: missing tool-result pairing caused upstream 502s; adjacency and stable call identity are mandatory.
- [#2531](https://github.com/lidge-jun/opencodex/issues/2531): malformed/non-record NDJSON shapes escaped one parser path; streaming and buffered parsing must share one strict classifier.
- OpenCodex's Command Code session work supports credential-scoped opaque `x-session-id` and canonical-host auth checks. Port those invariants, not unrelated Cursor Connect behavior.

## OpenCode Go exact evidence gate

[OpenCode Go's official documentation](https://dev.opencode.ai/docs/go/) now states that clients must identify themselves and include `x-opencode-session`; it also publishes a per-model wire table. As of this research, Grok 4.6 and GPT-5.6 Luna use the Responses endpoint, while many sibling models use Chat Completions. This is direct evidence that provider-wide wire inference is wrong.

Relevant OpenCodex evidence:

- [#3344](https://github.com/lidge-jun/opencodex/issues/3344), [#3378](https://github.com/lidge-jun/opencodex/issues/3378): missing `x-opencode-session` and destination/model-specific unsupported fields.
- [#2156](https://github.com/lidge-jun/opencodex/issues/2156): mid-tool EOF/truncation must not become success.
- [#2329](https://github.com/lidge-jun/opencodex/issues/2329): Grok was incorrectly routed through Chat rather than Responses.
- [#2330](https://github.com/lidge-jun/opencodex/issues/2330): stale discovery exposed uncallable models.
- [#2410](https://github.com/lidge-jun/opencodex/issues/2410): reasoning-effort metadata can drift independently of model names.

Port exact-model routing, destination-scoped sanitization, stable opaque session headers, strict complete-tool EOF rules, and catalog exclusions. Do not enable a family or entire provider based on one successful sibling. Phase 15 must accept zero supported tuples when dated capture/live evidence is unavailable.

## Cross-cutting warning from current OpenCodex

[OpenCodex #3807](https://github.com/lidge-jun/opencodex/issues/3807) is open and shows how a globally strict-looking rule can break valid clients: a new empty-`call_id` rejection blocks Codex Desktop subagent seed items on every translated provider. This does not justify inventing pairing IDs everywhere. It requires classification of external seed/input items separately from actual tool results. Use this as a regression fixture for every new translated transport.

## Ordered action list for the successor

1. Finish Phase 10 Plan 10-07 without broadening scope: remove debug output, resolve the retry fixture/classifier decision safely, eliminate the deprecated warning, run focused and full gates, update summary/validation/roadmap/verification.
2. Plan Phase 11 from the Antigravity evidence above; keep writeback changes and unrestricted account rotation out of scope.
3. Plan Phase 12 against the current agent.v1 path, not OpenCodex's older transport assumptions.
4. Build Phase 13 generic `openai_chat` before either Command Code product.
5. Implement the two Command Code transports separately in Phase 14.
6. Run Phase 15 as a dated exact-tuple OpenCode Go evidence gate; no family inference.
7. Phase 16 performs complete provider matrix, locale/doc parity, provenance, security, format, Clippy, serial workspace, and optional bounded live-smoke gates.

