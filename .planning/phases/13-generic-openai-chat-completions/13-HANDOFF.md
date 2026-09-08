# Phase 13 planning handoff

## Latest continuation — September 8

This section supersedes older progress notes below. AI-SPEC evaluation sections
5–7 are complete; root corrected pre-header/pre-send ambiguity, partial-stream
error wording and isolated CI commands. EDGE-COVERAGE.json contains all14
deterministic probe rows (12 explicit resolutions, two unclassified flagged
assumptions). VALIDATION.md is drafted, not prematurely marked compliant.
Commit7426c9b records these artifacts. Next is planner dispatch then independent
checker; no PLAN files or Chat source yet. Planner contributions are preserved
verbatim at /tmp/shunt-computer-eval-ikc2Ju/planner-contributions.md.
UI gate frontend=false/block=false; assumption-delta detected=false;
specless config absent means default ON; spec-section confirms both sections
absent. API coverage matrix and threat-model task mapping remain planner work.

Fresh full workspace tests passed:2194 active library tests, two ignored;
main/integration suites pass. Format, all-target/all-feature Clippy -Dwarnings,
and five-check rebuilt-binary smoke passed. Independent GLM source audit found
Phase09/11 stale fingerprints but no invariant regressions; regenerated only
after full regression evidence. Both now query passed (commit8013f19).

Computer was actually attempted by primary agent. Terminal access was denied
by Computer safety controls; in-app browser localhost31981/v1/models returned
ERR_BLOCKED_BY_CLIENT. Do not claim hands-on Computer evaluation passed or
bypass those restrictions. Separate shell smoke is passed and distinct.
User explicitly requires primary agent to perform Computer actions, or a
GPT-6 Astra subagent with explicit high effort; never delegate Computer to GLM.
The isolated candidate process59482 was stopped; wrapper verified production
config mtime/SHA and backup/invalid-file inventory unchanged. No live service
was restarted/reconfigured. No subagent currently running. Launch controls
again unavailable after audit dispatch, so planner was announced but NOT launched.

## User model and thinking instructions

Use opencode-go/glm-5.3-flash or opencode-go/omen-alpha. Testing
google-antigravity/gemini-3.8-flash is also authorized. EVERY launch,
retry and nested launch must explicitly set reasoning_effort high with
fork_turns none (or a bounded positive count). Never silently substitute models.

Latest observed outcomes:
- GLM/high: completed research, pattern mapping, framework selection,
  implementation guidance and domain rubric.
- Omen/high: launch rejected with provider400 thinking-mode validation,
  despite explicit high in the launch request; no artifact.
- Gemini/high: launch failed after HTTP429 retries; no artifact.
These are observed attempts, not guarantees of provider availability.

## Current state

Worktree /Users/user/.codex/worktrees/0466/shunt; branch
codex/opencodex-provider-compatibility. All commands explicit workdir and
all apply_patch paths absolute. Do not edit the main checkout at
/Volumes/PortableSSD/Projects/shunt. Preserve user .planning/config.json and .gsd/.

Phase12 complete 8/8 with verification. Phase13 CONTEXT, DISCUSSION-LOG,
RESEARCH, PATTERNS exist. AI-SPEC is explicitly DRAFT: selector, guidance
and domain sections done; evaluation sections5–7 not done. No Phase13
PLAN files, source implementation or new tests yet.

Next: resume GSD ai-integration-phase13 at gsd-eval-planner dispatch to fill
AI-SPEC sections5–7, then validate. Pydantic is explicitly N/A: approved
Rust-only/no-new-dependencies scope uses serde checked validation. Do not
install a Python framework or monitoring platform to satisfy a generic template.
No live official Chat specification was verified by the guidance author;
verify protocol claims against primary sources before implementation.

Then return plan-phase13 --auto: draft VALIDATION, deterministic spec-less
edge probe, API COVERAGE, remaining preflight gates, independent planner,
independent checker with bounded revisions, then execution. Research and
pattern mapping are complete; do not repeat them.

At handoff no child is running. Launch/follow-up controls disappeared again
after successful GLM domain work; list/wait/interrupt remain. Do not poll
empty agent waits to restore them. A fresh turn may restore launch controls.
Do not mark evaluation/planner/checker gates passed without actual results.

## Key design corrections

- RetrySafety::ConnectOnly, NOT NonIdempotentPost: latter permits ambiguous
  post-send timeouts. Pin actual post-send/pre-header timeout no-fallback.
- Disable redirects on Chat credential-bearing transport.
- Tool name/identity can arrive across deltas; enforce completeness at the
  assembly boundary, not necessarily the first delta.
- Adjacent OpenCodex source provenance HEAD055c3ecf0de6c35f59195fc434d6b08525182b7f.
- Pattern report is analogy, not a complete modified-files inventory:
  planner must own module exports, capabilities, tests and all affected docs.
- Root removed the domain author's unsupported regulatory-exemption claim;
  external compliance applicability is unassessed, not exempt.

## Safety and verification baseline

Stateful commands inherit fresh OPENCODEX_HOME and a non10100 port.
Read /tmp/shunt-phase12-isolated-run.cjs before using its wrapper.
It checks live config mtime/SHA and invalid/backup inventory before/after.
Never parse or test against /Users/user/.opencodex or port10100.
User-required owner-only backups:
- /Users/user/shunt-settings-backup-fZlDSM
- /Users/user/shunt-cursor-backup-SjUJ0y
Never print or commit their contents.

No tests or source changes in this continuation. Phase12 baseline:
full suite2193 active library tests plus integrations passed; final additional
HTTP test passed separately; fmt, clippy, 18 focused Cursor filters passed;
site161 pages/four locales built; rebuilt binary five-check smoke passed.
No Shunt live Cursor availability claim. CLI-only evidence is distinct.

GSD phase.complete/state helpers miscount archived1–8 row as a v2 phase.
Actual milestone progress is FOUR of EIGHT phases (9–12), not five.
Do not reopen Phase12. Preserve explicit outstanding Phase13–16 scope.
