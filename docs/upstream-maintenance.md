# Keeping the fork compatible with upstream

Research and integration decision, 2026-09-09. The maintenance safeguards below
reduce merge risk; they do not promise conflict-free or behaviorally correct merges.
See [activation and verification evidence](upstream-maintenance-verification.md)
for the tested setup and its remaining limits.

## Recommendation for this repository

Keep an upstream-tracking baseline separate from the long-lived integration
branch. Merge upstream into the integration branch; do not rewrite the released
`v2` history or force-reset the fork to upstream. Develop future customizations
as small topic branches, with isolated commits for behavior, refactoring and
generated artifacts. Offer generally useful fixes upstream through separately
approved PRs. Acceptance upstream is the most direct way to remove a maintained
delta; merely moving code to another file is not sufficient.

For this fork, preserve the protocol adapters and exact compatibility tests,
but minimize changes inside upstream-owned dispatch/configuration code. Prefer
small functions with explicit inputs/outputs over a second copy of upstream's
handler. The sync bridge gives explicit endpoint routes priority, retains the
legacy global resolver as fallback, and shares admission and outcome recording.
Keep that precedence covered by real-router tests. Separate native passthrough,
strict local Anthropic translation, and upstream translation-core modules until
their behavioral contracts can actually be reconciled.

Use ordinary merges for the established integration branch. Rebase is useful for
unpublished, short-lived topics, not as an automatic conflict eliminator for the
480 local commits observed at preflight. Many of those commits are planning
history; commit count is not a measure of runtime divergence. The documented
rebase caveat is downstream history disruption, not simply graph aesthetics.
[Git rebase documentation](https://git-scm.com/docs/git-rebase)

## Practical next-sync procedure

1. Preserve a known-good tag and inventory dirty files. Never include credentials
   or local configuration in commits or broadly stash unrelated user work.
2. Run `bash scripts/upstream-preflight.sh --fetch`, then inspect upstream PRs.
   Without `--fetch` the script uses the existing local upstream ref offline.
   It uses `git merge-tree --write-tree --name-only HEAD upstream/main` for a
   real merge simulation without touching the index or worktree. A clean result
   is only a textual result, not a test result.
3. Review semantic overlap before applying the merge: config precedence,
   credentials, retry/continuation boundaries, and streaming terminal events.
4. Resolve in a development worktree and run format, warnings-denied Clippy,
   full tests, provider-boundary regressions, isolated smoke, and docs checks.
   All local stateful checks must inherit a fresh OPENCODEX_HOME; never use the
   production home or port 10100. Verify the production fingerprint before/after.
5. Commit the verified merge locally. Pushing, publishing, and deployment remain
   separate authorized actions.

Prefer smaller, regularly reviewed syncs and early preflight checks. Scheduled
checks must report changes without automatically merging or publishing them.
On this workstation, the approved `Shunt upstream preflight` task is scheduled
for Mondays at 09:00 local time. It checks this integration worktree and reports
new revisions, changed conflicts, or failures; unchanged state stays quiet.
The schedule is app-local, not installed by cloning the repository, and does not
run builds, provider calls, or merges.
[Git merge-tree](https://git-scm.com/docs/git-merge-tree),
[GitHub fork syncing](https://docs.github.com/en/pull-requests/how-tos/work-with-forks/syncing-a-fork)

`rerere` can reuse a previously recorded resolution when the same conflict
recurs. Approved and enabled in this repository: local `rerere.enabled=true` and
`rerere.autoupdate=false`, so reused edits still require inspection/staging.
It cannot decide whether two new protocol behaviors are compatible. These
settings affect all worktrees sharing this repository, not other repositories.
They were enabled after the v0.44.0 merge, so no preimages from that already
resolved merge are claimed. The previous Git configuration was backed up outside
the repository before changing these two settings. On a fresh clone, opt in with
`git config --local rerere.enabled true` and
`git config --local rerere.autoupdate false`; these settings are not versioned.
[Git rerere](https://git-scm.com/docs/git-rerere)

Avoid blanket `ours` merge strategies, union merges of source code, and
`gh repo sync --force`: suppressing conflict markers can discard changes or
produce an invalid combination. `git-imerge` is an optional tool for subdividing
large integrations, not the default here; the current semantic overlap needs
human policy decisions regardless of merge granularity.
[git-imerge repository](https://github.com/mhagger/git-imerge)

## Executable safeguards and merge gates

Requirements: a Git version supporting `merge-tree --write-tree`, Bash, Node.js
18 or newer, and the existing Rust/smoke prerequisites (Python 3, curl, jq).
The preflight exits 0 for a textually clean merge, 1 for conflicts, and 2 for an
invalid invocation or Git/tool failure. It never applies a merge, updates the
index, or overwrites dirty files. Git may write temporary merge objects; an
explicit fetch updates remote-tracking refs. Inspect the reported commits even
when the simulation passes.

Run these gates from the development worktree after every actual integration:

```bash
bash scripts/test_upstream_preflight.sh
OPENCODEX_HOME="$(mktemp -d)" OPENCODEX_PORT=31987 node --test scripts/isolated-run.test.cjs
node scripts/isolated-run.cjs cargo fmt --all --check
node scripts/isolated-run.cjs env RUSTFLAGS=-Dwarnings cargo clippy --all-targets --all-features -- -D warnings
node scripts/isolated-run.cjs env RUSTFLAGS=-Dwarnings cargo test --all-features --workspace -- --test-threads=1
node scripts/isolated-run.cjs env RUSTFLAGS=-Dwarnings bash .claude/skills/run-shunt/smoke.sh
```

The committed runner replaces the session-only `/tmp` verification helpers.
It creates a fresh `OPENCODEX_HOME`, pins `OPENCODEX_PORT=31987` and the mock smoke
ports `SHUNT_PORT=31711` / `MOCK_PORT=31712` (overriding inherited values), removes the
inherited Cursor overrides used by local sessions, and compares production
config modification time, SHA-256, and invalid/backup inventory before and after
the child. It serializes local runs and preserves temporary homes for diagnostics.
This is an environment guard, **not an OS sandbox**: never pass it a command that
explicitly targets the production home or port 10100, and ensure child servers
are shut down before the command exits. Never nest the runner. A busy lock means
no command started; do not delete another run's lock.

CI retains its full fmt/Clippy/workspace gates, adds tooling fixture tests and
the mock gateway smoke, and uses the runner for test and coverage processes.
Do not use retries or weaker assertions to hide failures. The existing intermittent
Antigravity process EOF timeout remains documented in
[the sync evidence](upstream-sync-v0.44.0.md); a green rerun does not fix it.

### Compatibility ownership map

Keep future patches within these modules when possible. This is a review map,
not a proposal to move working code merely to change its paths.

| Boundary | Focused implementation | Regression coverage included in the full suite |
| --- | --- | --- |
| Endpoint route precedence / native relay | `src/codex_endpoint.rs`, `src/adapters/responses/inbound.rs` | `tests/inbound_codex_routes.rs`, `tests/inbound_codex_endpoint.rs`, `tests/inbound_codex_websocket.rs` |
| Strict local translation vs upstream core | `src/model/inbound_responses/` | Inbound endpoint and translation unit tests |
| Generic Chat / subscription separation | `src/adapters/openai_chat/`, `src/adapters/command_code/`, `src/auth/command_code.rs` | `tests/openai_chat_conformance.rs`, `tests/command_code_conformance.rs`, `tests/command_code_api_conformance.rs` |
| Provider admission, credential and release claims | Shared proxy admission plus provider-owned modules | `src/proxy/opencode_go_tests.rs`, `tests/opencode_go_evidence.rs`, `tests/release_security.rs`, `tests/release_matrix.rs` |

Config/auth/dispatch overlap still requires semantic review regardless of this
layout. Public config/provider semantics and credential writeback remain approval
boundaries. For user-facing changes, update README and site in all four locales,
build with `node scripts/isolated-run.cjs npm --prefix site run build`, and verify
affected rendered links. This maintenance-only change affects engineering docs
and CI, not runtime behavior: README/site translations need no change; generated
wiki files remain untouched.

## What the requested research established

GitHub was queried with `gh api` for upstream PRs 478 and 481, plus the
git-imerge repository metadata. PR478 adds exact endpoint-local native Responses
routing. PR481 explicitly lands only translation core; dispatch and relaxed
provider-kind validation are deferred. Its title alone was insufficient to infer
runtime support. The integration therefore retains Responses-only endpoint-local
routes and the existing local Anthropic global-route bridge, without newly
enabling Chat dispatch.
[PR478](https://github.com/pleaseai/shunt/pull/478),
[PR481](https://github.com/pleaseai/shunt/pull/481)

OpenAlex anonymous keyword search returned a temporary-unavailability response.
Direct DOI lookups succeeded for both records below; titles, publication years,
DOIs and indexed abstracts were inspected. Web searches found the Git primary
documentation and the research papers. The papers support early detection and
checking behavioral conflicts, not a guarantee that a particular branching
strategy eliminates them:

- Brun et al., *Proactive detection of collaboration conflicts* (2011):
  early identification is intended to prevent conflicts becoming more severe
  and to keep relevant changes fresh in developers' memory.
  [OpenAlex W2167626029](https://openalex.org/W2167626029),
  [paper](https://doi.org/10.1145/2025113.2025139).
- *Automatic Detection and Resolution of Software Merge Conflicts: Are We There
  Yet?* (2021 preprint): its study distinguishes textual, compilation and dynamic
  conflicts; the latter two are harder to detect than textual overlap. This is
  why compilation and regressions remain required after conflict resolution.
  [OpenAlex W3132164479](https://openalex.org/W3132164479),
  [paper](https://arxiv.org/html/2102.11307v2).

The concrete branch/layout recommendation above is an engineering inference
from those sources and this repository's observed overlap, not an experimental
claim of measured savings for this fork.
