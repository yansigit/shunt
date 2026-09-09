# Phase 16: Cross-Provider Release Gate - Context

**Gathered:** 2026-09-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Validate the v2 provider compatibility release against REL-01 through REL-06:
traceable conformance coverage, sanitized provenance/MIT notices, opt-in safe
live smokes, documentation parity, visual checks, and repository quality gates.
This is verification of implemented scope, not authorization to broaden support.
</domain>

<decisions>
## Implementation Decisions

### Evidence and scope (existing locked decisions)
- **D-01:** Preserve the implemented provider/auth/model/wire contracts. Map every support claim to named hermetic evidence; record not-applicable cases with rationale. Source-derived fixtures, actual captures, live results, and static/visual checks remain distinct.
- **D-02:** Go admits zero tuples. The passed Phase 15 review does not authorize admission, a support claim, or live wire inference from host subagent success. Future promotion needs its existing exact evidence gate.
- **D-03:** Retain MIT/provenance for translated material. Never commit credentials, account/project identifiers, private content, generated local configuration, or hand-edited wiki. No Google AI Studio Web surface, dependency, new runtime/parser dependency, credential writeback, or generalized catalog/history/repair subsystem.

### Live smoke (explicit user approval)
- **D-04:** The user approved at most eight live requests total, using existing authorized credentials, no retries, at most 60 seconds and 128 output tokens per request, and US$1 total planned paid usage. Skip any provider whose complete cost/reasoning/input bounds cannot be established; do not spend first and measure afterward. Synthetic minimal prompts only, no tools or private data.
- **D-05:** All live tests are opt-in and executed only after credential/destination/budget preflight. Use fresh isolated homes and non-10100 ports. No calls to the production proxy or config parsing of /Users/user/.opencodex. Source credentials must remain byte-for-byte unchanged; preserve settings/credential backups, redact output, and stop immediately on unexpected mutation.
- **D-06:** Missing credentials, unsafe refresh/persistence behavior, unavailable service, unknown pricing/reasoning limits, or budget exhaustion produce explicit skip/blocked evidence, never a fabricated pass. Do not purchase credits, increase spending, log in, refresh source stores, or bypass provider admission. No repeated live sweeps.

### Documentation and visual release checks
- **D-07:** Fix Phase 15 cosmetic redundant emptiness wording and locale sidebar labels while preserving zero-support meaning and per-file regression assertions. Keep English/ko/ja/zh-cn parity and verify actual built locale links/anchors. README/docs/site considered; wiki untouched.
- **D-08:** Attempt visual evaluation against only an owned isolated local documentation server. Computer work is root-led or gpt-6-astra/high, per user direction. Prior blocked Computer checks are not a pass; record actual screenshots/observations or honest unavailability. No unrelated app/session interaction.

### Delivery and verification
- **D-09:** Reuse existing Rust real-router tests, owned CLI smoke patterns, and site build. Add focused tests only for demonstrated gaps; never weaken/remove tests for green status. Final gates are format, warnings-denied Clippy, all-features workspace tests, site/docs validation, scope and security/code reviews.
- **D-10:** Every stateful test/build/smoke process tree inherits fresh OPENCODEX_HOME via node /tmp/shunt-phase12-isolated-run.cjs or an equivalently verified wrapper. Check production config mtime/SHA and backup/invalid inventory before and after. Work exclusively in /Users/user/.codex/worktrees/0466/shunt; preserve unrelated planning config/state/.gsd data. Serialize Cargo and live-budget ownership.
- **D-11:** Use explicit high effort for subagents: opencode-go/omen-alpha, opencode-go/glm-5.3-flash, or gpt-5.6-luna candidates. Model dispatch is not Shunt protocol evidence. Complete the phase only with honest verification; milestone audit/completion remain separate gates.

### Claude's Discretion
Compact artifact layout, named-test mapping format, fixture/test decomposition,
safe owned smoke harness details, and execution order within these boundaries.
</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- tests/*conformance.rs, provider-local real-router/cancellation/replay tests, tests/check_cli.rs, and Phase 9–15 verification artifacts.
- THIRD-PARTY-NOTICES.md and docs provider engineering notes for provenance.
- site build and existing per-file docs assertions; site/dist for actual locale anchors.
- /tmp/shunt-phase12-isolated-run.cjs verifies production fingerprints around isolated process trees. Outside-repository owner-only backups already exist.

### Established Patterns
Real Axum requests and loopback upstreams, independent fixture expectations,
RAII-owned temporary state/processes, explicit terminal/retry/admission boundaries.

### Integration Points
Release documentation and focused tests; no new runtime provider contract.
Phases 9–15 were revalidated after Phase 15 shared changes: 2,972 tests passed,
two existing ignored, format/Clippy passed, production unchanged at eb60174.
</code_context>

<specifics>
User approved the bounded live-smoke proposal explicitly during autonomous
smart discuss. Remaining defaults inherit prior decisions rather than reopening
already settled provider or credential semantics.
</specifics>

<deferred>
## Deferred Ideas
New Go model admission, additional provider capabilities, credential writeback,
broader spending, and publication/merge are not authorized by this phase.
</deferred>
