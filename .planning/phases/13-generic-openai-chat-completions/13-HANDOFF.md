# Phase 13 planning handoff

## Explicit subagent thinking preference (user instruction)

Always pass `reasoning_effort: high` on EVERY subagent launch, including retries,
fallback models and nested delegation; do not rely on inheritance/defaults.
Use `fork_turns: none` (or a bounded positive history count) with the override.
Preferred model: opencode-go/muse-spark-1.3-contributor; fallback order:
opencode-go/omen-alpha, opencode-go/glm-5.3-flash,
opencode-go/deepseek-v4-flash. Never silently substitute another model.

Latest attempt: chat_research launched with explicit Muse/high/fork-none, but
failed before an artifact with provider HTTP 429 (retry limit exceeded).
No source changes or test runs occurred. Fallback dispatch still needs to run;
Omen was announced but has NOT actually been launched in this attempt.

Phase 12 is complete: 8/8 summaries, REVIEW and VERIFICATION, requirement and
roadmap transition committed. Latest commits: 9814a8f (HTTP error cap/deadline),
c1a2646 (phase verification), 02108cd (Phase 13 approved context).

Phase 13 CONTEXT and DISCUSSION-LOG are committed. No research, plans or new
Chat source exists yet. Resume GSD plan-phase 13 --auto, at researcher dispatch.
The complete plan-phase workflow was read. Init says Pending, research and
checker enabled, Nyquist enabled, standard granularity, auto_advance true.
Context drift is skipped for no upstream artifacts. Dispatch isolation none.
Active plan:pre hooks: research, pattern mapper, AI integration skill, UI skill;
contributions API coverage, assumption delta, schema detection, ASVS1 security
(block high); advisory codebase/context drift and blocking UI gate. Apply actual
domain guards, do not invent frontend or database work for this Rust adapter.

The current turn lost spawn/follow-up tools after the successful Muse-high Cursor
review. Only wait/list/interrupt remain. Installed CLIs: agent (Cursor) and codex;
no opencode executable. Do not replace the user's chosen subagent model silently
or mark independent research/checker gates passed. A fresh turn may restore the
launch controls. Use Muse high, fallback candidates per user, and fork_turns none.
Require explicit workdir for shell and ABSOLUTE apply_patch file paths in children:
the last reviewer wrote its report to main by mistake; it was relocated and only
that known generated misplaced report removed. No source changed in main.

Worktree: /Users/user/.codex/worktrees/0466/shunt, branch
codex/opencodex-provider-compatibility. Preserve user .planning/config.json and
.gsd/ dirt. No agents or test processes are active.

Last verified source: 9814a8f. Full suite passed (2193 active library tests plus
all integrations, two prior ignored benchmarks); the final additional HTTP test
passed separately. Final clippy passed; 18 focused Cursor filter chains all
selected nonzero tests and passed; site built 161 pages/four locales; rebuilt
binary smoke passed five checks. Production OpenCodex untouched throughout.
No Shunt live Cursor availability claim. CLI-only probe evidence is distinct.

Stateful commands must inherit fresh OPENCODEX_HOME and non-10100 ports.
Existing wrapper /tmp/shunt-phase12-isolated-run.cjs enforces before/after live
config mtime/SHA and invalid/backup inventory; read it before using.
User-required backups: /Users/user/shunt-settings-backup-fZlDSM and
/Users/user/shunt-cursor-backup-SjUJ0y (owner-only); never print their contents.

GSD phase.complete/state helpers miscount the archived 1–8 roadmap row as one v2
phase. Actual v2 progress is FOUR of EIGHT phases (9–12), not five. Correct the
display through supported state handling; do not reopen completed phase 12.
Verification fingerprint was refreshed after requirement checkbox transition.
