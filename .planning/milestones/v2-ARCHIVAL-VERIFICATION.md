# v2 archival verification — 2026-09-09

Archive safety commit: `00297b8`. Eight phase directories, 48 plans and all
63 requirements are retained under `.planning/milestones/`. No next milestone
is scoped. Cleanup found no remaining phase directories to move.

## Post-archive regression

The initial warnings-denied full workspace run failed (exit 101) when
`antigravity_process::streaming_turn_translates_stub_events_to_sse` exceeded
its existing 20-second guard at tests/antigravity_process.rs:320. This timeout
had also been recorded during Phase15. Its cause was not established here.

The unchanged test passed in isolation in 0.30 seconds (one passed, ten
filtered). The complete warnings-denied workspace suite then passed with
`--test-threads=1`: **2,979 passed, zero failed, two existing ignored**, exit 0.
All assertions and timeout limits were preserved. This is verification of the
serial run, not a fix or guarantee of parallel-run reliability.

See `v2-ARCHIVAL-GATES.json` for the command and every test result summary.
All three runs inherited fresh isolated OPENCODEX_HOME values, used no
production port, and reported unchanged live config mtime/SHA and backup/invalid
file inventory. No production recovery or credential write occurred.

## Other final gates

The final pre-archive format check, warnings-denied Clippy and full parallel
workspace suite passed; transcripts remain in
`v2-phases/16-cross-provider-release-gate/16-FINAL-GATES.json`. No source,
dependency, or test changes followed those gates. The five-check owned smoke,
173-page four-locale site build, 260 fragment-link checks and scoped actual
Computer evaluation are retained in the Phase16 evidence records.

Live generation was skipped: **0/8 attempts, US$0 planned**. This is not live
provider availability proof. OpenCode Go has zero admissions and remains
unsupported. The milestone audit retains its minor metadata/cosmetic advisories.

## Closeout boundaries

The redundant active REQUIREMENTS.md was removed only after its archive was
committed; it remains recoverable from Git. The archive header says completed
locally, not shipped. Local tag creation follows this passing regression.
No merge, push, publication or deployment is performed. Unrelated
`.planning/config.json`, `.planning/state.json` and `.gsd/` are preserved.
