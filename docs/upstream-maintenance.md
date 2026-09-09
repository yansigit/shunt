# Keeping the fork compatible with upstream

Research and integration decision, 2026-09-09. This is a maintenance recommendation,
not a promise of conflict-free merges or an enabled recurring automation.

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
2. Fetch `upstream`, inspect its PRs, and run
   `git merge-tree --write-tree --name-only HEAD upstream/main`. This performs a
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

Prefer smaller, regularly reviewed syncs and early preflight checks. A scheduled
check would be useful, but no schedule is installed by this research task.
[Git merge-tree](https://git-scm.com/docs/git-merge-tree),
[GitHub fork syncing](https://docs.github.com/en/pull-requests/how-tos/work-with-forks/syncing-a-fork)

`rerere` can reuse a previously recorded resolution when the same conflict
recurs. Recommended opt-in: repository-local `rerere.enabled=true` and
`rerere.autoupdate=false`, so reused edits still require inspection/staging.
It cannot decide whether two new protocol behaviors are compatible. These
settings were researched, not silently enabled, and this merge does not claim
to have recorded preimages before conflict resolution.
[Git rerere](https://git-scm.com/docs/git-rerere)

Avoid blanket `ours` merge strategies, union merges of source code, and
`gh repo sync --force`: suppressing conflict markers can discard changes or
produce an invalid combination. `git-imerge` is an optional tool for subdividing
large integrations, not the default here; the current semantic overlap needs
human policy decisions regardless of merge granularity.
[git-imerge repository](https://github.com/mhagger/git-imerge)

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
