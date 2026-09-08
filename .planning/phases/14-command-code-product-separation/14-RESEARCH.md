# Phase 14: Command Code Product Separation — Research

**Researched:** 2026-09-08
**Researcher:** independent GLM subagent, explicit high thinking effort.
**Delivery:** returned as final text after file-save tooling stalled; root persisted this report. No source edits, tests, live calls, or real credential reads were performed by the researcher.
**Evidence:** adjacent OpenCodex `/Volumes/PortableSSD/Projects/opencodex` at `055c3ecf0de6c35f59195fc434d6b08525182b7f`, clean when inspected. Older planning sources pin `07b48da8`; initial port audit pins `566debc7`. Facts below are dated source evidence, not live availability. Preserve MIT attribution for substantial translated material.

## User Constraints

The complete `14-CONTEXT.md` is authoritative: separate products; approved public additions, read-only credentials; environment precedence with invalid explicit values failing closed; canonical HTTPS destination validation before credentials and at dispatch; exact model/effort facts; opaque credential/conversation affinity; authentic adjacent tool units, explicit missing/orphan carriers; one bounded checked NDJSON contract; no EOF success, repair, ambiguous post-send replay, refresh, rotation or durable history. All tests use fresh isolated state. Existing provider behavior and maintained docs/locales remain preserved.

## Summary

Reuse Phase 13 Chat for `commandcode`; add a distinct subscription adapter and auth mode for `command-code`. Source establishes the split and proprietary event/envelope shapes. Source permissively drops malformed lines and synthesizes done at EOF: these are deliberately rejected under CCS-06, not requirements to port. Version and minimal-envelope live acceptance remain unverified.

## Two products

Source: OpenCodex `src/providers/registry.ts`, subscription near 1221, API-key near 1981.

| Product | ID | Adapter | Base | Authentication |
| --- | --- | --- | --- | --- |
| API-key | commandcode | openai-chat | https://api.commandcode.ai/provider/v1 | key |
| Subscription | command-code | command-code | https://api.commandcode.ai | oauth |

API-key uses `/chat/completions`; catalog `/provider/v1/models` is public with 256 KiB / 256-model bounds in source. `apiKeyValidation: unknown`; a 401 probe does not prove key validity. Registry sets `parallelToolCalls: false`. The subscription catalog is authenticated/account-scoped; source has no static complete model list. A source comment dates the API-key endpoint probe to 2026-08-03 and references `https://commandcode.ai/docs/provider`; this research did not fetch that documentation.

## Subscription request and identity

Source: `src/adapters/command-code.ts`, `createCommandCodeAdapter.buildRequest` and `fetchCommandCode`.

- POST `/alpha/generate`, always `stream: true`; source fetch uses manual redirects.
- Headers: Authorization Bearer, Content-Type application/json, User-Agent `cli`, `x-command-code-version` default `0.52.1`, `x-cli-environment: production`, `x-taste-learning: false`, `x-co-flag: false`, and `x-session-id`. Optional source cwd-derived `x-project-slug` is not needed and must not leak workspace identity.
- Envelope: `{config, memory: "", taste: null, skills: null, permissionMode: "standard", mode: "agent", params: {...}}`.
- Params: exact `model`, `messages`, tools `{name, description, input_schema}`, joined `system`, `max_tokens` (source default 64000), always-stream, optional temperature and reasoning_effort.
- Source config contains workingDir/date/environment/directory/git context. Do not copy filesystem scanning or workspace exfiltration. A minimal constant config is recommended but acceptance must be labelled unverified until actual evidence; source-derived mocks alone cannot establish it.
- Source session hashes thread/replay/cursor/cache identifiers and finally first-user-text into UUID-shaped values; tests near `tests/command-code-provider.test.ts:669` prove source stability. Reject prompt-text-only identity in Shunt. Scope opaque affinity to credential and conversation, retaining it across safe replay. Planner must explicitly define behavior when no genuine conversation identifier exists; a new random ID per request cannot be claimed stable across independent turns.

## Exact model and effort facts

Source: `src/providers/command-code-efforts.ts` at the revision above. Account availability is not implied by an effort row.

| Exact model | Source effort ladder |
| --- | --- |
| zai-org/GLM-5.3 | low, high, max |
| deepseek/deepseek-v4-pro; deepseek/deepseek-v4-flash | high, max |
| zai-org/GLM-5; GLM-5.1; GLM-5.2; GLM-5.2-Fast | high, max |
| google/gemini-3.7-flash | low, medium, high |
| gpt-5.6-luna | source describes low through max; planner must read exact array before pinning |
| meta/muse-spark-1.2; meta/muse-spark-1.2-contributor; meta/muse-spark-1.1 | source describes low through max; ultra rejected in dated 2026-08-13 evidence |

Three vision-exp/flash rows are reporter-only evidence (#2647), not independently verified. Do not advertise them as proven. Source effort alias/clamping and retry-without-effort are excluded. Exact unknown or unsupported effort fails before generation. No need for a generalized catalog platform; research exact account-bound admission requirements before choosing a narrow mechanism.

## Tool-history contract

Source emits assistant `tool-call` immediately followed by tool `tool-result` using authentic toolCallId/toolName. Results are `{output:{type:"text"|"error-text", value}}`; missing results use explicit non-executed error-text. Orphan results are retained in a user text carrier. Result images appear in a following user message as `{type:"image", image, mediaType?}`. Keep these product-specific rules out of the Phase 13 Chat compiler. Duplicate/ambiguous identities fail closed. Plaintext subagent tools and continued tool history need positive cases; rejection of unsupported opaque continuation alone cannot establish CCS-08 coverage.

## NDJSON response contract

Source events:

- text-delta / reasoning-delta: `{type, text}`.
- tool-call: `{toolCallId, toolName, input|args}`; maps to start/argument/end.
- finish-step and finish: source considers both terminal and first wins. Usage from finish-step.usage or finish.totalUsage/usage; finishReason/rawFinishReason; `error` finish means failed turn.
- error: provider error, including missing-result error classification.
- Usage: inputTokens/outputTokens plus inputTokenDetails.cacheReadTokens/cacheWriteTokens.
- Credit depletion HTTP body: `{success:false,error:{code:"BAD_REQUEST",status,message,docs}}`.

Source tolerates SSE data prefixes, null/junk lines and missing finish. **Shunt does not inherit permissive malformed/EOF completion.** Planner must distinguish legitimate finish-step then finish lifecycle from duplicate actual terminal; source first-wins cannot prove either interpretation. Require authoritative grammar/captured evidence before building self-confirming fixtures. Unknown parseable records must have an explicit bounded classification. Use a shared checked machine for streaming/unary, strict UTF-8/JSON/object fields, integer usage, tool completeness and neutral redacted errors. In-batch terminal validation cannot promise inspection of future bytes after transport cancellation.

## Read-only authentication

Code-only evidence: `src/oauth/command-code.ts` reads `~/.commandcode/auth.json`, schema `{apiKey: nonempty string, userId?: string}`. Source validates using canonical GET `/alpha/whoami` with Bearer and receives `{user:{id,userName?}}`. Source copies credentials into its own store and defines an identity refresh; Shunt must do neither.

Recommended Shunt source is an explicitly named subscription-only environment variable via existing token-env conventions, then bounded read-only CLI file fallback only when no explicit source is selected/present. Invalid explicit source must fail rather than switch accounts. A proposed `SHUNT_COMMAND_CODE_API_KEY` name from the researcher risks confusing the API-key product; planner should choose an unambiguous subscription-token name under the already approved addition. Missing credentials is an authentication error in normal operation, and only an explicit live-test harness may label it SKIP. Never read real credentials merely to inspect schema. Validate origin before lookup and again before send; no public loopback bypass.

## Recommended integration points

`src/config.rs`, `src/config/presets.rs`, `src/config/upstreams.rs`: kind/auth pairs and presets. `src/auth/mod.rs`: read-only resolver integration with private helper. `src/routing.rs`, `src/proxy/failover.rs`, `src/proxy/capability.rs`: owned dispatch/capability wiring. `src/adapters/command_code/`: private request/envelope, NDJSON decoding and checked response rendering. Reuse Phase 13 Chat and existing retry/admission/session primitives rather than adding a universal abstraction.

## Validation Architecture

Existing overlap: `tests/openai_chat_translate.rs`, `tests/openai_chat_conformance.rs`, Phase 9 commitment assertions and Phase 13 cancellation/redaction patterns. New source-derived fixtures carry revision/date and synthetic content; never label them live captures.

| Area | Required evidence |
| --- | --- |
| CCK-01/02 | Preset routes Chat to /provider/v1/chat/completions, separate auth; subscription mode cannot select it |
| CCK-03 | Applicable normal/unary/stream/tools/long-context/drop/auth/error scenarios through actual gateway |
| CCS-01 | Explicit-env/absent/empty/malformed/CLI matrix; file bytes, mtime and inventory unchanged |
| CCS-02 | Canonical origin/path positive, http/lookalike/suffix/userinfo/query/redirect negatives before bearer access/send |
| CCS-03 | Exact envelope/headers/model/effort; stable credential-scoped opaque session across supported continuation; unsupported tuples fail |
| CCS-04 | Parallel authentic call/result units; missing/error-text, orphan carrier, result images, duplicate rejection |
| CCS-05 | Streaming incremental order and unary semantic twin for text/reasoning/tools/usage/finish/error |
| CCS-06 | Junk/null/invalid UTF-8/malformed/oversized/incomplete tool/provider error/duplicate terminal/EOF all explicit failure; grammar-resolved finish-step lifecycle |
| CCS-07 | Safe pre-connect replay preserves credential/session; no post-send/header/output/tool replay, refresh or rotation |
| CCS-08 | Positive plaintext subagent/continuation, ordinary and tool-heavy conversations, long-context bounds, real socket cancellation before headers and mid-body in both modes |

Boundary coverage: record/residual/semantic aggregate/tool count/argument bytes below-at-above, UTF-8 splits, slow drip/read deadline. Real listener cancellation must prove upstream closure and capacity reuse; await owned server tasks. Gateway-owned errors keep correct inbound protocol shape.

All cargo/build/test/smoke command trees use `node /tmp/shunt-phase12-isolated-run.cjs ...`; this creates fresh OPENCODEX_HOME and verifies production config SHA/mtime plus backup inventory. Use non-10100 fixture ports. Gates: isolated cargo fmt --all --check; cargo clippy --all-targets --all-features -- -D warnings; env 'RUSTFLAGS=-D warnings' cargo test --all-features --workspace; npm --prefix site run build; owned binary/CLI curl smoke. Never execute an unquoted RUSTFLAGS value containing spaces.

## Tracer-first delivery guidance

Researcher suggested four groups: product/auth, request/history, response/transport, conformance/docs. Root correction: an end-to-end subscription tracer must lead, not be deferred to final conformance. Each new adapter needs a minimal production-reaching text/terminal/auth slice before expansion. Partition shared-file ownership and serialize overlaps; docs/locales may be disjoint parallel work after semantics stabilize. All requirements and D-01–14 must appear in executable plan content.

## Confidence and unresolved evidence

High: source product split, endpoint/auth file schema, envelope field names. Medium: proprietary version and lifecycle details; minimal-envelope live acceptance unverified. Low: reporter-only model efforts. These labels describe evidence quality, not passed tests.

Before execution: resolve exact supported effort arrays, finish-step/finish grammar, session without client ID, and minimal-envelope contract with dated source/captured/official evidence where available. Do not treat a source-derived self-test as provider acceptance. Live availability remains Phase 16, explicitly skipped if unavailable. Public env naming fits already approved scope; no new prompt for an implementation detail is needed. No new version override config is justified merely because upstream has one.
