# Phase 16 Research — Cross-Provider Release Gate

## User Constraints

Copied from `16-CONTEXT.md` (locked decisions and discretion areas):

### Implementation Decisions

- **D-01:** Preserve the implemented provider/auth/model/wire contracts. Map every support claim to named hermetic evidence; record not-applicable cases with rationale. Source-derived fixtures, actual captures, live results, and static/visual checks remain distinct.
- **D-02:** Go admits zero tuples. The passed Phase 15 review does not authorize admission, a support claim, or live wire inference from host subagent success. Future promotion needs its existing exact evidence gate.
- **D-03:** Retain MIT/provenance for translated material. Never commit credentials, account/project identifiers, private content, generated local configuration, or hand-edited wiki. No Google AI Studio Web surface, dependency, new runtime/parser dependency, credential writeback, or generalized catalog/history/repair subsystem.
- **D-04:** The user approved at most eight live requests total, using existing authorized credentials, no retries, at most 60 seconds and 128 output tokens per request, and US$1 total planned paid usage. Skip any provider whose complete cost/reasoning/input bounds cannot be established; do not spend first and measure afterward. Synthetic minimal prompts only, no tools or private data.
- **D-05:** All live tests are opt-in and executed only after credential/destination/budget preflight. Use fresh isolated homes and non-10100 ports. No calls to the production proxy or config parsing of /Users/user/.opencodex. Source credentials must remain byte-for-byte unchanged; preserve settings/credential backups, redact output, and stop immediately on unexpected mutation.
- **D-06:** Missing credentials, unsafe refresh/persistence behavior, unavailable service, unknown pricing/reasoning limits, or budget exhaustion produce explicit skip/blocked evidence, never a fabricated pass. Do not purchase credits, increase spending, log in, refresh source stores, or bypass provider admission. No repeated live sweeps.
- **D-07:** Fix Phase 15 cosmetic redundant emptiness wording and locale sidebar labels while preserving zero-support meaning and per-file regression assertions. Keep English/ko/ja/zh-cn parity and verify actual built locale links/anchors. README/docs/site considered; wiki untouched.
- **D-08:** Attempt visual evaluation against only an owned isolated local documentation server. Computer work is root-led or gpt-6-astra/high, per user direction. Prior blocked Computer checks are not a pass; record actual screenshots/observations or honest unavailability. No unrelated app/session interaction.
- **D-09:** Reuse existing Rust real-router tests, owned CLI smoke patterns, and site build. Add focused tests only for demonstrated gaps; never weaken/remove tests for green status. Final gates are format, warnings-denied Clippy, all-features workspace tests, site/docs validation, scope and security/code reviews.
- **D-10:** Every stateful test/build/smoke process tree inherits fresh OPENCODEX_HOME via node /tmp/shunt-phase12-isolated-run.cjs or an equivalently verified wrapper. Check production config mtime/SHA and backup/invalid inventory before and after. Work exclusively in /Users/user/.codex/worktrees/0466/shunt; preserve unrelated planning config/state/.gsd data. Serialize Cargo and live-budget ownership.
- **D-11:** Use explicit high effort for subagents: opencode-go/omen-alpha, opencode-go/glm-5.3-flash, or gpt-5.6-luna candidates. Model dispatch is not Shunt protocol evidence. Complete the phase only with honest verification; milestone audit/completion remain separate gates.

### Claude's Discretion

Compact artifact layout, named-test mapping format, fixture/test decomposition, safe owned smoke harness details, and execution order within these boundaries.

### Deferred Ideas

New Go model admission, additional provider capabilities, credential writeback, broader spending, and publication/merge are not authorized by this phase.

## Project Constraints (from CLAUDE.md)

- Preserve streaming semantics; never buffer upstream SSE unless the client requested non-streaming output.
- Gateway-owned errors stay Anthropic-shaped except Codex ingress, which uses OpenAI Responses errors.
- Prefer table-driven config additions; keep Rust files focused; update README/docs/site locale surfaces for observable changes; never hand-edit generated `wiki/`.
- Never inspect or mutate production OpenCodex state (`/Users/user/.opencodex`, port `10100`); use an isolated `OPENCODEX_HOME` for every stateful process.
- Run format, warnings-denied Clippy, and all-features workspace tests before completion; do not change credential-file writeback without approval.

## Existing Tuple Source of Truth

The preset table is the release inventory seed. It currently contains the following exact values (read from source, not inferred): `anthropic` → `Anthropic`, `https://api.anthropic.com`, `Passthrough`; `codex` → `Responses`, `https://chatgpt.com/backend-api`, `ChatgptOauth`; `openai` → `Responses`, `https://api.openai.com/v1`, `ApiKey`, `OPENAI_API_KEY`; `xai` → `Responses`, `https://api.x.ai/v1`, `ApiKey`, `XAI_API_KEY`; `grok` → `Responses`, `https://cli-chat-proxy.grok.com/v1`, `XaiOauth`; `kimi` → `Anthropic`, `https://api.moonshot.ai/anthropic`, `ApiKey`, `MOONSHOT_API_KEY`; `cursor` → `Cursor`, `https://api2.cursor.sh`, `CursorOauth`; `kimi-code` → `Anthropic`, `https://api.kimi.com/coding`, `KimiOauth`; `zhipu` → `Anthropic`, `https://open.bigmodel.cn/api/anthropic`, `ApiKey`, `ZHIPUAI_API_KEY`; `minimax-cn` → `Anthropic`, `https://api.minimax.cn/anthropic`, `ApiKey`, `MINIMAX_API_KEY`; `commandcode` → `OpenAiChat`, `https://api.commandcode.ai/provider/v1`, `ApiKey`, `SHUNT_COMMANDCODE_API_KEY`; `opencode-go` → `OpenCodeGo`, `https://opencode.ai/zen/go/v1`, `ApiKey`, `SHUNT_OPENCODE_GO_API_KEY`; and `command-code` → `CommandCode`, `https://api.commandcode.ai`, `CommandCodeOauth`. [VERIFIED: src/config/presets.rs:19-110]

The release ledger should expand each configured model route into `(provider, auth path, exact model, wire)` rows, then point each row at named tests and fixture/provenance records. Do not treat a provider preset as proof that every model family or capability is supported. OpenCode Go must remain an explicit zero-admission row set: Phase 15's `admitted: []` ledger is evidence of rejection policy, not a live or captured support claim. [VERIFIED: .planning/phases/15-exact-opencode-go-evidence-gate/15-EVIDENCE.md:1-12]

## Coverage Inventory and Gaps

Reuse these existing suites rather than creating a parallel framework:

- `tests/gemini_conformance.rs`: Antigravity native SSE in unary/streaming modes (`antigravity_native_sse_real_loopback_both_downstream_modes`), incremental streaming, strict malformed/truncation/embedded-error cases, cap/cap+1 bounds, no-post-header replay, tool pairing, and cancellation/lifetime release (`antigravity_native_lifetime_*`, `gemini_response_drop_releases_upstream_and_gateway_capacity`). [VERIFIED: tests/gemini_conformance.rs:153-240, 423-495, 1416-1470]
- `tests/openai_chat_conformance.rs`: redirect refusal and post-send timeout single-attempt checks; bounded interleaved indexed tool assembly; unary/streaming tracer paths; unknown fields, metadata, unsupported blocks, signed/redacted thinking, malformed UTF-8, tool pairing; terminal, EOF, duplicate/residual terminal, malformed JSON, usage, reasoning, and embedded provider-error cases. [VERIFIED: tests/openai_chat_conformance.rs:21-117, 374-517, 632-739, 808-1061, 1241-1529]
- `tests/command_code_api_conformance.rs` and `tests/command_code_conformance.rs`: API-key product pairing, normal/tool/long-context, streaming terminal, cancellation/drop, tracer, and product separation. [VERIFIED: tests/command_code_api_conformance.rs:94-263, 337-480; tests/command_code_conformance.rs:5-6]
- Cursor's crate-local protocol/request/history/router/cancellation suites cover Connect/protobuf framing, tool/history identity, terminal and cancellation behavior; use `src/adapters/cursor/protocol_tests.rs`, `request_isolation_tests.rs`, `history_lifetime_tests.rs`, `cancellation_tests.rs`, and `router_parity_tests.rs` as the named evidence paths.
- ChatGPT/Codex preservation and retry boundaries are covered by `tests/codex_websocket_fallback.rs`, `tests/failover.rs`, `tests/retry.rs`, `tests/passthrough.rs`, `tests/inbound_codex_endpoint.rs`, and `tests/inbound_codex_websocket.rs`; map tests for pre-output fallback, replay-unsafe tools, credential rebinding, cancellation, and error-shape parity.
- `tests/opencode_go_evidence.rs` and `tests/opencode_go_docs.rs` provide the exact empty-admission ledger and per-file docs assertions. `tests/check_cli.rs` contains the real-process negative CLI smoke (`opencode_go_cli_negative`). [VERIFIED: tests/opencode_go_evidence.rs:87-107; tests/opencode_go_docs.rs:31-407; tests/check_cli.rs:202-207]

The main demonstrated release gap is cross-provider traceability: existing tests are provider-local and Phase 15 explicitly says the milestone-wide REL-05 audit remains Phase 16. Build a machine-readable matrix/ledger that marks each tuple's applicable scenarios (normal, stream, tools, terminal, malformed, truncation, auth, cancellation, retry), links exact tests, and records `not_applicable` rationale. Do not infer coverage from a passing full suite or from a host model/subagent dispatch.

## Provenance and Credential Schema Findings

Source-derived OpenCodex material is already identified by pinned revision/date in Phase 15's evidence ledger; preserve that structure for every translated fixture or copied protocol fragment. Captures must include source repository, pinned revision, capture date, and sanitization status, with secrets, account/project identifiers, and private content removed. Keep `THIRD-PARTY-NOTICES.md` and MIT provenance references; do not add generated wiki edits.

Credential schemas must be researched from source definitions only. The Command Code subscription resolver reads `SHUNT_COMMAND_CODE_TOKEN` first, and only true env absence falls back to a read-only CLI path; its exact endpoint constant is `https://api.commandcode.ai/alpha/generate`, CLI auth schema is camelCase `{apiKey: String, userId?: String}`, file reads are capped at 16 KiB and five seconds, and no whoami/refresh/writeback occurs. [VERIFIED: src/auth/command_code.rs:10-17, 19-38, 64-87, 108-119] Do not inspect any real credential file. Other auth stores (Google, Antigravity, Cursor, Codex, Claude) have distinct schemas and refresh/writeback behavior; release live-smoke planning must classify unsafe refresh/persistence as skip/blocked rather than probing them.

## Safe Live-Smoke Feasibility

Live smokes are a bounded verification activity, not a discovery sweep. Before any request, establish for the selected provider/auth/model/wire tuple: credential presence without reading secret contents, canonical destination, input/output/reasoning/token bounds, expected price ceiling, timeout ≤60 seconds, and whether the auth path can prove source-file immutability without refresh. Allocate at most eight total requests, no retries, synthetic minimal prompt, no tools/private data, and planned paid total ≤US$1. A provider is **required to skip** when any complete cost/reasoning/input bound cannot be established; never issue a request and measure billing afterward. Distinguish bounded request generation/output from hidden or unbounded provider-side reasoning/cost: a client-side `max_tokens=128` does not prove reasoning tokens or billing are bounded. Record `blocked`/`skipped` with the missing bound and evidence source.

Use an owned local destination only for harness validation. For any authorized live destination, validate origin/path before credential access, disable redirects, redact request/response logs, and snapshot source-file bytes/mtime/inventory before and after. Stop immediately on mutation, unexpected destination, timeout overrun, or budget uncertainty. Never call port `10100`, parse `/Users/user/.opencodex`, log in, refresh source stores, or purchase credits.

## Documentation and Visual Gate

Phase 15's UI review records two cosmetic warnings: redundant “empty admitted set is empty” wording and locale-suffixed OpenCode Go sidebar labels. [VERIFIED: .planning/phases/15-exact-opencode-go-evidence-gate/15-UI-REVIEW.md:25-30, 49-53] Fix these while retaining the exact zero-support tokens asserted by `tests/opencode_go_docs.rs`. Verify English plus ko/ja/zh-cn README, provider, configuration, and guide surfaces, built locale links, and native anchors; leave `wiki/` untouched. The prior review explicitly says screenshots/rendered layout were not captured, so a static `site/dist` check is not a visual pass. [VERIFIED: .planning/phases/15-exact-opencode-go-evidence-gate/15-UI-REVIEW.md:5-16, 113]

## Standard Stack and Architecture Patterns

- Use existing Rust unit/integration tests, `wiremock`/loopback Axum real-router fixtures, RAII temporary state, and the existing site build. No new dependencies or provider adapters.
- Keep the release matrix/ledger as a static, sanitized planning/evidence artifact validated by focused Rust tests or a small existing test binary; do not build a generalized catalog/history/repair subsystem.
- For live/CLI process checks, use the isolation wrapper. It snapshots `/Users/user/.opencodex/config.json` mtime/SHA and `config.json.invalid-*`/backup inventory, creates a fresh temp `OPENCODEX_HOME`, sets `OPENCODEX_PORT=31987`, strips Cursor override env vars, and fails with status 90 on mutation. [VERIFIED: /tmp/shunt-phase12-isolated-run.cjs:4-16]
- Preserve the existing commitment boundary: no retry/failover after client-visible output or replay-unsafe tool activity; streaming remains incremental, while non-streaming accumulation is bounded.

## Don't Hand-Roll

- Do not create another provider adapter, credential parser, live pricing system, or persistent evidence database.
- Do not duplicate existing conformance machines or invent a generic “all providers support these scenarios” helper; link exact provider-local tests.
- Do not hand-edit `wiki/`, inspect credential values, or infer support from provider family names, host model availability, or a successful process boot.

## Common Pitfalls

1. A passing full workspace test can hide zero-selected focused filters; every release command must use an exact named filter and verify nonzero selection.
2. Treating source-derived fixtures as live captures, or treating a captured upstream wire on one model as evidence for another exact tuple.
3. Calling a live provider before pricing/reasoning/input bounds are complete; this violates the explicit eight-request/$1 gate.
4. Letting OAuth refresh/writeback happen during a smoke and then claiming credential immutability; classify such paths as unsafe/skip unless an isolated, read-only path is proven.
5. Reporting static HTML as visual acceptance; screenshots/observations or honest unavailability are required.
6. Adding a positive OpenCode Go claim: zero admissions remain the only supported state until exact captured/live evidence is separately accepted.

## Validation Architecture (runnable commands)

All stateful commands must be separate invocations through the wrapper, from the required worktree:

```text
node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test gemini_conformance antigravity_native -- --test-threads=1
node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test openai_chat_conformance openai_chat_ -- --test-threads=1
node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test command_code_api_conformance command_code_api_ -- --test-threads=1
node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test opencode_go_evidence opencode_go_ledger -- --test-threads=1
node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test opencode_go_docs opencode_go_docs -- --test-threads=1
node /tmp/shunt-phase12-isolated-run.cjs cargo test --all-features --test check_cli opencode_go_cli_negative -- --test-threads=1
node /tmp/shunt-phase12-isolated-run.cjs cargo fmt --all --check
node /tmp/shunt-phase12-isolated-run.cjs env RUSTFLAGS=-Dwarnings cargo clippy --all-targets --all-features -- -D warnings
node /tmp/shunt-phase12-isolated-run.cjs env RUSTFLAGS=-Dwarnings cargo test --all-features --workspace
node /tmp/shunt-phase12-isolated-run.cjs npm --prefix site run build
```

The focused commands above are starting points; the plan must add exact filters for every tuple row and reject any command that selects zero tests. Existing Phase 15 results provide a baseline (2,972 passed, zero failed, two pre-existing ignored; 173 site pages), but Phase 16 must record its own run and keep source/capture/live/visual outcomes separate. Cargo and live-budget ownership are serialized. Visual work is root-led against an owned local documentation server on a non-10100 port; if Computer/browser access is unavailable, record that honestly rather than marking pass.

## Plan Shape Recommendation

1. Inventory and ledger: enumerate configured routes/presets, map exact tests and not-applicable cases, and add provenance/sanitization assertions without changing runtime semantics.
2. Evidence/security: validate MIT notices, fixture metadata, negative Google AI Studio Web/wiki/dependency scans, and credential-boundary/read-only checks.
3. Documentation: fix the two Phase 15 cosmetic issues, run all per-file locale assertions, build the site, and inspect built links/anchors.
4. Opt-in smoke harness: preflight bounds and credentials, then at most eight serialized requests; write explicit skip/blocked records for anything that cannot be proven safe.
5. Release gates: run isolated fmt, Clippy, workspace tests, site validation, scope/security/code review, and (only if available) owned visual screenshots; do not claim release completion until every REL-01–06 row has evidence.

## Confidence

- **HIGH:** preset tuple values and Command Code credential schema (opened source definitions); existing test names and isolation wrapper behavior; Phase 15 zero-admission/provenance/UI findings.
- **MEDIUM:** completeness of the manually assembled cross-provider matrix until every configured route/model declaration is checked against the final ledger.
- **LOW/UNVERIFIED:** any live pricing, provider reasoning caps, credential availability, or rendered visual outcome; these require explicit preflight/source evidence and must fail closed when unavailable.
