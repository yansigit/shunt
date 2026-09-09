# Upstream maintenance safeguards — verification

Verified 2026-09-09 on the integration worktree, based on merge `db344e3` and
upstream `e1de716a887124cbc3de276ea2a150a531c6998b` (v0.44.0).
This is developer tooling, not a change to provider or credential semantics.

## Activated

- Repository-local `rerere.enabled=true`, `rerere.autoupdate=false`, confirmed
  with `git config --show-origin`. These apply to this repository's linked
  worktrees. No released tags or history were rewritten.
- Previous Git config copied to an owner-only external backup before the change
  (`/Users/user/shunt-sync-settings-backup-G5mg4x/git-config`, directory 0700,
  file 0600). No production settings or credentials were changed or copied into Git.
  Both rerere keys were previously absent locally; to undo only this activation,
  unset those two local keys after confirming no later intentional changes.
  Do not overwrite the whole Git config blindly from the backup.
- `Shunt upstream preflight` app-local heartbeat: active, Mondays 09:00 local
  time; no merges, commits, tests, provider calls, publishing, or deployment.
  Creation and stored schedule were verified; a future scheduled execution has
  not yet occurred. The same preflight command was exercised manually below.

## Results

| Gate | Result |
| --- | --- |
| `bash scripts/upstream-preflight.sh --fetch` | Pass: zero upstream commits behind, clean merge-tree; HEAD unchanged |
| `bash scripts/test_upstream_preflight.sh` | Pass: clean/divergent/conflicting histories, tracked and untracked dirt, staged index, invalid input, local-fixture fetch preservation |
| `node --test scripts/isolated-run.test.cjs` via the prior guarded runner | Pass: synthetic fingerprint changes, missing-state handling, inherited unsafe values replaced, exit propagation, missing executable, busy lock refusing to start |
| Guarded `cargo fmt --all --check` | Pass |
| Guarded warnings-denied Clippy, all targets/features | Pass |
| Guarded `cargo test --all-features --workspace --quiet -- --test-threads=1` | Pass; two existing ignored tests unchanged; no retries or test changes |
| Guarded mock gateway smoke | Pass: config, liveness, discovery, forwarding, malformed request; ports 31711/31712 |
| Independent read-only review | No blocking correctness/safety findings |

Cargo and smoke gates used the committed `scripts/isolated-run.cjs`, with fresh
temporary homes. Each completed guard reported the production OpenCodex config
mtime/SHA-256 and invalid/backup inventory unchanged. The smoke driver's SIGTERM
diagnostic is its intentional mock-server cleanup; its final exit status was 0.
The final fixture rerun includes the smoke-port override guard.

CI configuration now runs these tooling tests and mock smoke in addition to its
existing full gates, with isolated test/coverage processes. Hosted GitHub Actions
and coverage upload were not run: no push or PR was performed.

## Limits retained

Textually clean merges still need protocol/config/auth review and regression
tests. The previously observed Antigravity process timeout was not reproduced in
this full run and is not claimed fixed. Prior documentation-template breadcrumb
warnings are unchanged; this maintenance-only change did not rebuild or modify
the translated site, README surfaces, generated wiki, or runtime code.
Existing unrelated planning/state edits and `.gsd/` were preserved.
