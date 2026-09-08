# Phase 15: Exact OpenCode Go Evidence Gate — Research

**Researched:** 2026-09-08
**Researcher:** gsd-phase-researcher (GLM subagent), working only in worktree `/Users/user/.codex/worktrees/0466/shunt`.
**Evidence base:** adjacent pinned OpenCodex `/Volumes/PortableSSD/Projects/opencodex` at `055c3ecf0de6c35f59195fc434d6b08525182b7f` (commit dated 2026-08-30T00:37:27-06:00), inspected read-only 2026-09-08. All `[VERIFIED]` tags below carry verbatim quotes read this session; unknowns are stated explicitly, never inferred. No build, test, live, or credential operations were performed.

## User Constraints

The complete `15-CONTEXT.md` is authoritative; D-01–D-11 bind planning. Highlights the planner must honor:

- **D-01/D-09:** additive opt-in Go config only; byte-verified owner backups already exist at `/Users/user/shunt-phase15-backup-YQSxN3`; no migration of production settings; every stateful test/build/smoke process tree runs through `node /tmp/shunt-phase12-isolated-run.cjs` with fresh `OPENCODEX_HOME` and non-10100 ports; production config mtime/SHA/inventory unchanged. Preserve user `.planning/config.json` and `.gsd/` dirt.
- **D-02/D-03:** smallest explicit table-driven selection; never infer Go from bearer, family, or nickname; admission identity = exact model + canonical destination + wire + effort + documented capabilities; unknown/ambiguous/family-inferred/unsupported/failed tuples fail **before credential lookup or network dispatch**, including fallback and inbound Codex paths.
- **D-04/D-05:** candidates, not auto-support; keep source claims, sanitized captures, live observations, and hermetic assertions distinct; record date, exact model, canonical destination, wire, headers, context, modalities, effort, tools, filtering, terminal behavior, and capture/live provenance. Source comments and successful host subagent dispatch do not prove Shunt wire compatibility.
- **D-06:** admit only after hermetic + credential-safe captured/live evidence; empty admitted set is valid; no dormant multi-wire dispatch or session infrastructure for hypothetical tuples.
- **D-07:** admitted tuples reuse the matching existing Chat/Responses/Anthropic contract and send a stable opaque, conversation-scoped `x-opencode-session` **only** to the canonical Go destination; no prompt-derived identity, raw account/session disclosure, durable cache, or redirect credential forwarding. With zero admitted tuples, no credential or session header is emitted.
- **D-08:** fail-closed bounded terminals, conservative non-idempotent retry, ownership cancellation; never import permissive EOF completion, silent repair, dynamic catalog machinery, or family aliases.
- **D-10/D-11:** tracer-first real-gateway negative admission evidence, focused conformance, fmt, warnings-denied Clippy, full workspace tests, owned CLI smoke; docs update together in README + en/ko/ja/zh-cn (root and site); generated wiki untouched; bounded live smoke is separately opt-in; never inspect credential contents; production proxy (port 10100, `/Users/user/.opencodex`) never touched.

**Claude's discretion:** private names, compact module layout, exact additive public spelling consistent with existing conventions, fixture decomposition, evidence-led candidate scope, numeric bounds within current limits. These implement the approved boundary; they do not relax the evidence threshold.

## Summary

The OpenCode Go provider is a real, pinned upstream product: `openai-chat` adapter, base URL `https://opencode.ai/zen/go/v1`, key auth, live-discovered roster. Of the four candidates, only `glm-5.3-flash` and `deepseek-v4-flash` have dated source-level Chat-wire evidence at the pin; `omen-alpha` has zero evidence anywhere in the pinned source, and `muse-spark-1.3-contributor` is absent from the pin (only 1.2 is documented, on a different wire) — both must remain unsupported and family/name inference is forbidden. The smallest sufficient surface is one additive `opencode-go` preset plus a crate-private exact-tuple admission gate consulted before credential resolution; no Responses or Anthropic Go surface is justified by current evidence. One planner decision is forced: pinned upstream enables bounded EOF tolerance for the Go destination (complete-JSON tool calls only), which must be explicitly reconciled with D-08 before any Chat tuple is admitted.

## Candidate evidence records (OGO-01)

Pinned revision `055c3ecf0de6c35f59195fc434d6b08525182b7f`, inspected 2026-09-08. "Unknown" means the pinned source does not evidence the field for this candidate on this destination — it is not an absence-derived claim of support or non-support beyond what is quoted.

### Provider anchor (applies to all candidates)

```
id: "opencode-go", label: "opencode go", adapter: "openai-chat", baseUrl: "https://opencode.ai/zen/go/v1",
authKind: "key", featured: true, dashboardUrl: "https://opencode.ai/auth", defaultModel: "kimi-k2.7-code",
jawcodeBundle: "opencode-go", note: "GLM, DeepSeek, Kimi, Qwen, MiMo…",
```
[VERIFIED: src/providers/registry.ts:1426-1428]

The Go roster is live-discovered: a registry comment states "the Go roster is // discovered live, so it applies the moment the gateway serves the id" [VERIFIED: src/providers/registry.ts:1452-1453]. Static tables are hints, not wire facts — which is exactly why candidates absent from them stay unadmitted.

### `glm-5.3-flash` — dated source evidence, Chat wire

| Field | Finding | Evidence |
| --- | --- | --- |
| Date / revision | 2026-09-08 inspection of 055c3ecf (2026-08-30) | this report header |
| Canonical destination | `https://opencode.ai/zen/go/v1` | [VERIFIED: registry.ts:1426-1427, quote above] |
| Wire | openai-chat (Chat Completions) via provider adapter | [VERIFIED: registry.ts:1426, `adapter: "openai-chat"`] |
| Effort | `"glm-5.3-flash": ZAI_GLM_53_REASONING_EFFORTS` wired inside the Go block; ladder `const ZAI_GLM_53_REASONING_EFFORTS = ["low", "high", "max"];` | [VERIFIED: registry.ts:1460 and :377] |
| Effort semantics | three effective tiers; upstream folds minimal/light→low, medium/high→high, xhigh/max/ultra→max; max is default and unknown-value fallback | [VERIFIED: registry.ts:370-376, quote: "docs.z.ai/devpack/latest-model folds every // incoming effort into three effective tiers — low/minimal/light -> low, medium/high -> high, // xhigh/max/ultra -> max — with max as both the default and the unknown-value fallback."] |
| Reasoning summaries / replay | `"glm-5.3-flash": true` in `modelSupportsReasoningSummaries`; member of `preserveReasoningContentModels` | [VERIFIED: registry.ts:1483-1484, :1504] |
| Modality on Go | **explicitly unknown**. Go block declares modalities only for kimi-k3, the DeepSeek vision preview, and muse-spark-1.2-contributor; flash is absent from both Go `modelInputModalities` and Go `noVisionModels`. Text-only catalog advertising follows from absence per upstream's own rule, but other providers document it as a native VLM, so Go-side vision handling is genuinely undetermined | [VERIFIED: registry.ts:1450-1456 (Go modalities), :1490-1500 (Go noVisionModels lists glm-5.3/5.2/5/5.1/deepseek rows without flash), :358-359 ("glm-5.3-flash` is a native VLM (docs.z.ai/guides/vlm/glm-5.3-flash)"), :1467-1469 muse-spark comment "Without this declaration the catalog // advertises it text-only"] |
| Context / output on Go | **unknown** — absent from the Go metadata rows (rows exist for glm-5, glm-5.1, glm-5.2, glm-5.3 but not flash) | [VERIFIED: src/generated/model-metadata.ts:53, `"opencode-go"` row has no `glm-5.3-flash` entry] |
| Tools / filtering | Zen tool-schema sanitization applies to the Go base URL; no Go-specific field filtering evidenced for flash | [VERIFIED: src/adapters/openai-chat.ts:924-928, quote: "return baseUrl === \"https://opencode.ai/zen/v1\" // || baseUrl === \"https://opencode.ai/zen/go/v1\";"] |
| Terminal behavior | Go provider sets `openaiChatEofTolerance: true`; see D-08 conflict below | [VERIFIED: registry.ts:1429-1431, quote: "// Zen Go can close a Chat stream after a fully assembled function call without sending // finish_reason or [DONE] (#2260). The adapter still rejects incomplete argument JSON."] |
| Capture/live provenance | none — source-only | this row |

### `deepseek-v4-flash` — dated source evidence, Chat wire

| Field | Finding | Evidence |
| --- | --- | --- |
| Date / revision | 2026-09-08 inspection of 055c3ecf | this report header |
| Canonical destination / wire | Go base, openai-chat | [VERIFIED: registry.ts:1426-1427] |
| Effort | `DEEPSEEK_FLASH_THINKING_EFFORTS = ["low", "high", "max"];`; compat map low→low, medium→high, high→high, xhigh→high, max→max — `xhigh` is a non-advertised compatibility alias, medium→high is upstream's own OpenAI-ladder choice | [VERIFIED: registry.ts:597-608, quotes: "const DEEPSEEK_FLASH_THINKING_EFFORTS = [\"low\", \"high\", \"max\"];" and "`xhigh` is a COMPATIBILITY ALIAS, not a native tier." and "medium` has no row in the vendor table — mapping it to `high` is OUR // compatibility choice"] |
| Context / output on Go | ctx 1,000,000 / output 384,000, modality `"text"`, metadata row present | [VERIFIED: src/generated/model-metadata.ts:53, `["deepseek-v4-flash",1000000,384000,"text",1,...]`] |
| Modality on Go | text-only — explicitly listed in Go `noVisionModels` | [VERIFIED: registry.ts:1490-1500, quote: `"deepseek-v4-flash", "deepseek-v4-pro",`] |
| Reasoning content replay | required (#78): deepseek thinking models are in `preserveReasoningContentModels` | [VERIFIED: registry.ts:1504, comment: "// Issue #78: DeepSeek V4 thinking mode requires reasoning_content replay on tool-call turns."] |
| Vendor doc date | api-docs.deepseek.com comment dated 2026-08-13 in source | [VERIFIED: registry.ts:578-594 comment block] |
| Terminal behavior | same Go `openaiChatEofTolerance: true` | [VERIFIED: registry.ts:1429-1431] |
| Capture/live provenance | none — source-only | this row |

### `omen-alpha` — zero evidence

`rg "omen-alpha"` over the entire pinned repository returns zero matches at 055c3ecf (checked this session, excluding lockfiles). No registry entry, no metadata row, no devlog, no docs. Every OGO-01 field is **unknown**. Host routing that exposes `opencode-go/omen-alpha` (e.g. this agent environment's own model list) is live-discovery, not sanitized wire evidence, and cannot admit the tuple (D-04/D-05).

### `muse-spark-1.3-contributor` — zero evidence at pin; 1.2 is a different tuple

`rg "muse-spark-1\.3" src/` returns zero matches at 055c3ecf (checked this session). The pinned evidence is exclusively `muse-spark-1.2-contributor`: `modelWireDefaults: { "gpt-5.6-luna": "openai-responses", "muse-spark-1.2-contributor": "openai-responses" }` [VERIFIED: registry.ts:1440], `"muse-spark-1.2-contributor": ["text", "image"]` modalities "parts over /responses (probed 2026-08-26)" [VERIFIED: registry.ts:1454-1455], and `autoToolChoiceOnlyModels` including it [VERIFIED: registry.ts:1502]. Per D-02/D-05 the 1.3 candidate must not be inferred from the 1.2 record; every OGO-01 field for 1.3 on Go is **unknown**. (OpenRouter metadata rows for muse-spark-1.3 exist in the pin but belong to a different provider/destination and are not Go evidence.)

## Wire, transport, and session facts

- **Headers (openai-chat):** `"Content-Type": "application/json"`, `Authorization: Bearer ${provider.apiKey}` when a credential exists, plus optional provider header overrides [VERIFIED: src/adapters/openai-chat.ts:90-93, quote: "const headers: Record<string, string> = { \"Content-Type\": \"application/json\" }; // if (hasCredential) headers.Authorization = `Bearer ${provider.apiKey}`; // if (provider.headers) Object.assign(headers, provider.headers);"]. No Go-specific mandatory headers beyond these are evidenced.
- **Wire-default resolution is exact-model:** `modelWireDefaults` is an exact lowercase-model allowlist gated by transport/allowed-wire checks [VERIFIED: registry.ts:1440 and decision-log comment at :1436-1439: "the map stays an exact-model allowlist rather than a family or provider-wide rule."]. This matches D-02 directly.
- **EOF tolerance mechanics (the D-08 conflict):** the openai-chat stream-end path recovers at EOF only when pending tool calls have complete JSON argument objects; otherwise it errors "upstream stream ended mid tool call without a terminal signal — possible truncation" [VERIFIED: src/adapters/openai-chat.ts:1885-1894, quote: "if (provider.openaiChatEofTolerance === true && pendingToolCallsAreCompleteJsonObjects()) { // if ((yield* flushToolCalls()) === \"terminate\") return; // yield { type: \"done\", usage: pendingUsage }; // return; }"]. **Planner decision required:** D-08 says "never import permissive EOF completion", but this upstream behavior is bounded (complete-JSON-only, #2260), not permissive. The plan must either (a) frame this bounded recovery as evidence-backed, captured-verified Go terminal behavior implemented explicitly for admitted Go tuples only, or (b) admit Go Chat tuples with strict terminals and reject EOF-without-finish even for complete JSON — accepting a real compatibility gap with the pinned upstream. Option (a) needs a sanitized capture proving the exact upstream stream shape before admission (OGO-02); hermetic fixtures alone cannot establish it.
- **`x-opencode-session` upstream status:** it is **not** an OpenCodex wire header at the pin. It appears only as a proposed item in a backlog table with status "absent | absent", target "conversation-derived or UUID", plus an explicit warning [VERIFIED: devlog/_plan/260820_bug_pr_backlog_consolidation/080_residual_dispositions.md:129,160-161, quotes: "| `x-opencode-session` | absent | absent | conversation-derived or UUID |" and "Deliberately NOT copied from omniroute: `x-opencode-project`, `x-opencode-request`, // `x-opencode-session`. None is needed to fix the demonstrated failure, and adding a // conversation-derived session identifier is a privacy-relevant change that needs its own // evidence rather than a sibling project's precedent."]. Therefore OGO-03's header is a Shunt-defined opaque, conversation-scoped header with **zero upstream acceptance evidence**; whether the Go destination tolerates an unknown header is explicitly unknown and belongs in the credential-safe verification step. Mirror the command-code session mechanics in `src/adapters/command_code/` for stability semantics; with zero admitted tuples no session or credential header is emitted (D-07).
- **Rate limiting (context only, not re-verified this session):** prior research recorded `src/providers/opencode-zen-rate-limit.ts` (~15-20 RPM community-measured, synthetic 15s retry-after when the header is missing). Treat as [ASSUMED prior pass, unverified this session]; do not build retry policy on it without re-reading.

## Recommended smallest surface and admission seam

1. **Public surface:** one additive provider preset `opencode-go` in `src/config/presets.rs`, modeled directly on the existing commandcode preset: OpenAiChat kind, ApiKey auth, base URL `https://opencode.ai/zen/go/v1`, explicit env name (planner's choice; e.g. `SHUNT_OPENCODE_GO_API_KEY`). Preset ordering tests must be updated [VERIFIED: src/config/presets.rs:90-96, quote: `name: "commandcode", kind: ProviderKind::OpenAiChat, base_url: "https://api.commandcode.ai/provider/v1", auth: AuthMode::ApiKey, api_key_env: Some("SHUNT_COMMANDCODE_API_KEY")`; :150-153 exact-string test `"anthropic, codex, openai, xai, grok, kimi, cursor, kimi-code, zhipu, minimax-cn, commandcode, command-code"` must gain the new name]. No Responses or Anthropic Go surface: no candidate is evidenced on those wires on Go, and D-06 forbids dormant multi-wire dispatch.
2. **Admission gate:** crate-private exact-tuple table (like `src/adapters/command_code/efforts.rs`) keyed by exact model, consulted before credential resolution. The existing pattern already rejects pre-dispatch: `ok_or("unsupported Command Code model")` / `return Err("unsupported Command Code reasoning effort")` [VERIFIED: src/adapters/command_code/efforts.rs:8-31]. Go gate checks model + effort (+ wire) against the evidence table above; unknown, family-inferred, and out-of-ladder tuples fail closed. Wire-default selection must remain exact-lowercase-model like upstream's `modelWireDefaults` rule.
3. **Pre-credential ordering:** capability filtering already runs before fallback credentials in the failover chain [VERIFIED: src/proxy/failover.rs:63, `super::capability::filter_fallbacks(&mut routes, body.json(), &requested_model);`]. Reuse `src/routing.rs` route resolution, `src/proxy/capability.rs` `AdapterKind::OpenAiChat` filtering, and the injectable credential/client seams in `src/server.rs` for real-router tests. Inbound Codex compatibility (`resolve_native_inbound` + `codex_endpoint` config) must route through the same gate so inbound-Codex Go selections are rejected identically.
4. **Count tokens:** `src/count_tokens.rs` is local tiktoken counting whose per-provider applicability is decided by provider config — the Go preset must explicitly decide applicability; do not silently enable or disable it.
5. **Admission verdicts at this evidence level:** `deepseek-v4-flash` has the most complete source record (context/output/modality/effort/replay all evidenced) and is the primary admission candidate pending hermetic fixtures + credential-safe verification; `glm-5.3-flash` is second (modality and Go context unknown — planner must decide text-only admission vs. wait-for-capture). `omen-alpha` and `muse-spark-1.3-contributor` stay unsupported at this revision. **Zero admitted tuples is a valid shipped outcome (OGO-02)** if captures/live evidence cannot be obtained.

## Validation Architecture (OGO-02..04)

| Requirement | Test evidence |
| --- | --- |
| OGO-01 | Evidence records above (dated, per-field, provenance-tagged); fixture header comments cite the pinned revision |
| OGO-02 | Hermetic conformance through the real router for each admitted tuple (request shape, headers, effort ladder, streaming/unary twins, usage, terminal behavior); credential-safe captured or live verification as a separate, labeled step; empty-set path compiles and ships |
| OGO-03 | Positive fixtures prove `x-opencode-session` present, stable across safe replay, conversation-scoped, and sent only to the canonical Go base; negative fixtures prove it absent for every non-Go provider and absent entirely with zero admitted tuples |
| OGO-04 | Real-router negative admission fixtures (unknown model, family-inferred name, out-of-ladder effort, wrong wire) asserting an error response with **zero** credential lookups and **zero** network dispatch — assert via injectable credential/client seams that no lookup or socket was touched; repeat through fallback and inbound-Codex paths |

Process constraints: all cargo/build/test/smoke command trees use `node /tmp/shunt-phase12-isolated-run.cjs` with fresh `OPENCODEX_HOME` and non-10100 fixture ports; verify production config SHA/mtime/backup inventory before and after. Gates: isolated `cargo fmt --all --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `env 'RUSTFLAGS=-D warnings' cargo test --all-features --workspace`; `npm --prefix site run build`; owned CLI smoke. Precedent fixtures: `tests/openai_chat_conformance.rs`, `tests/command_code_conformance.rs`, `tests/inbound_codex_endpoint.rs`. Docs updated in the same change: README (+ko/ja/zh-CN), `site/src/content/docs/` providers page + reference if config keys are added, with all three locale copies; `wiki/` untouched (generated). Never describe static evaluation as a live pass; live smoke remains a separately opt-in release activity (D-11).

## Confidence and unresolved evidence

| Item | Confidence | Basis |
| --- | --- | --- |
| Provider anchor, wire, effort ladders, metadata rows, EOF mechanics, header building | HIGH | verbatim pinned-source quotes, this session |
| Zero evidence for omen-alpha / muse-spark-1.3 at pin | HIGH (as of revision) | exhaustive rg this session; live roster may differ — that is discovery, not evidence |
| glm-5.3-flash Go modality/context | explicit UNKNOWN | absent declarations; other-provider VLM docs are different-platform evidence |
| Go acceptance of `x-opencode-session` | explicit UNKNOWN | upstream proposal only; needs credential-safe verification |
| Bounded EOF recovery as admissible terminal behavior | PLANNER DECISION | D-08 vs pinned #2260 behavior; needs sanitized capture before any admission |
| Zen rate-limit numbers | LOW / [ASSUMED] | prior pass only |

Open questions for planning: (1) EOF framing — bounded-recovery-with-capture vs strict terminal; (2) env/preset naming; (3) glm-5.3-flash modality policy (admit text-only now vs hold for capture); (4) session-header behavior when no genuine conversation identifier exists (mirror the Phase 14 planner requirement); (5) count-tokens applicability for the Go preset. No further subagents were used; no state, roadmap, config, or code changes were made.

## Sources

### Orchestrator dispositions (2026-09-08)

- D-08 already resolves EOF policy: strict authoritative terminals are mandatory.
  Complete argument JSON does not prove generation completed; bounded EOF recovery
  is still forbidden completion synthesis. The proposed option (a) above is rejected,
  not an unresolved permission to relax D-08.
- No candidate currently has the required captured/live proof (including the
  session-header contract). Plan the explicitly permitted empty admission set.
  Do not admit GLM text-only or DeepSeek from source hints alone.
- A preset alone may lose product identity after expansion. The planner must
  verify the actual config representation and retain explicit opt-in identity
  without URL/name inference or changes to generic provider semantics.
- Do not add a speculative session-header producer or multi-wire dispatch while
  admission is empty. Document the conditional promotion checks for OGO-03.
- The unverified rate-limit aside is not an implementation input.

- Pinned OpenCodex `055c3ecf0de6c35f59195fc434d6b08525182b7f` (2026-08-30), read-only: `src/providers/registry.ts`, `src/adapters/openai-chat.ts`, `src/generated/model-metadata.ts`, `src/providers/opencode-zen-rate-limit.ts`, `devlog/_plan/260820_bug_pr_backlog_consolidation/080_residual_dispositions.md`.
- Shunt worktree `/Users/user/.codex/worktrees/0466/shunt`: `src/config/presets.rs`, `src/adapters/command_code/efforts.rs`, `src/proxy/failover.rs`, `src/routing.rs`, `src/proxy/capability.rs`, `src/count_tokens.rs`, `tests/{openai_chat,command_code}_conformance.rs`, `tests/inbound_codex_endpoint.rs`.
- Planning: `.planning/phases/15-exact-opencode-go-evidence-gate/15-CONTEXT.md`, `.planning/REQUIREMENTS.md` (OGO-01..04 at lines 82-85), `.planning/phases/14-command-code-product-separation/14-RESEARCH.md` (format precedent).
