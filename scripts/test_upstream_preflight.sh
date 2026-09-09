#!/usr/bin/env bash
# Standalone fixture tests for upstream-preflight.sh; no project state is used.
set -u
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
preflight=$script_dir/upstream-preflight.sh
tmp_root=$(mktemp -d "${TMPDIR:-/tmp}/shunt-preflight.XXXXXX")
trap 'rm -rf "$tmp_root"' EXIT

fail() { echo "FAIL: $*" >&2; exit 1; }
expect_status() {
  expected=$1
  shift
  set +e
  output=$("$@" 2>&1)
  status=$?
  set -e
  [[ $status -eq $expected ]] || { printf '%s\n' "$output" >&2; fail "expected $expected, got $status"; }
  printf '%s\n' "$output"
}
run_preflight() {
  repo=$1
  shift
  (cd "$repo" && "$preflight" "$@")
}
new_repo() {
  dir=$1
  git init -q "$dir"
  git -C "$dir" config user.email test@example.invalid
  git -C "$dir" config user.name Fixture
  printf 'base\n' >"$dir/base.txt"
  git -C "$dir" add base.txt
  git -C "$dir" commit -qm base
  git -C "$dir" branch upstream/main
  git -C "$dir" checkout -q -b work
}

# Clean merge and ahead/behind reporting.
new_repo "$tmp_root/clean"
printf 'work\n' >"$tmp_root/clean/work.txt"
git -C "$tmp_root/clean" add work.txt
git -C "$tmp_root/clean" commit -qm work
clean_output=$(expect_status 0 run_preflight "$tmp_root/clean" 2>&1) || exit $?
grep -q 'Ahead: 1  Behind: 0' <<<"$clean_output" || fail "clean ahead/behind missing"
grep -q 'preflight: clean' <<<"$clean_output" || fail "clean result missing"

# Divergent, non-overlapping changes are mergeable.
new_repo "$tmp_root/divergent"
printf 'work\n' >"$tmp_root/divergent/work.txt"
git -C "$tmp_root/divergent" add work.txt
git -C "$tmp_root/divergent" commit -qm work
git -C "$tmp_root/divergent" checkout -q upstream/main
printf 'upstream\n' >"$tmp_root/divergent/upstream.txt"
git -C "$tmp_root/divergent" add upstream.txt
git -C "$tmp_root/divergent" commit -qm upstream
git -C "$tmp_root/divergent" checkout -q work
divergent_output=$(expect_status 0 run_preflight "$tmp_root/divergent" 2>&1) || exit $?
grep -q 'Ahead: 1  Behind: 1' <<<"$divergent_output" || fail "divergent ahead/behind missing"

# Same-path edits report a conflict and the file name.
new_repo "$tmp_root/conflict"
printf 'work\n' >"$tmp_root/conflict/base.txt"
git -C "$tmp_root/conflict" commit -qam work
git -C "$tmp_root/conflict" checkout -q upstream/main
printf 'upstream\n' >"$tmp_root/conflict/base.txt"
git -C "$tmp_root/conflict" commit -qam upstream
git -C "$tmp_root/conflict" checkout -q work
conflict_output=$(expect_status 1 run_preflight "$tmp_root/conflict") || exit $?
grep -q 'Conflicting files:' <<<"$conflict_output" || fail "conflict heading missing"
grep -q 'base.txt' <<<"$conflict_output" || fail "conflict path missing"

# A dirty worktree remains byte-for-byte and status-wise unchanged.
printf 'uncommitted\n' >"$tmp_root/conflict/dirty.txt"
printf 'staged\n' >"$tmp_root/conflict/base.txt"
git -C "$tmp_root/conflict" add base.txt
printf 'unstaged\n' >>"$tmp_root/conflict/base.txt"
before_status=$(git -C "$tmp_root/conflict" status --porcelain)
before_hash=$(shasum "$tmp_root/conflict/dirty.txt")
before_tracked=$(shasum "$tmp_root/conflict/base.txt")
before_tree=$(git -C "$tmp_root/conflict" write-tree)
before_index=$(shasum "$tmp_root/conflict/.git/index")
expect_status 1 run_preflight "$tmp_root/conflict" >/dev/null
after_status=$(git -C "$tmp_root/conflict" status --porcelain)
after_hash=$(shasum "$tmp_root/conflict/dirty.txt")
after_index=$(shasum "$tmp_root/conflict/.git/index")
[[ "$before_status" == "$after_status" ]] || fail "worktree status changed"
[[ "$before_hash" == "$after_hash" ]] || fail "dirty file changed"
[[ "$before_tracked" == "$(shasum "$tmp_root/conflict/base.txt")" ]] || fail "tracked dirty file changed"
[[ "$before_tree" == "$(git -C "$tmp_root/conflict" write-tree)" ]] || fail "index tree changed"
[[ "$before_index" == "$after_index" ]] || fail "index changed"

# Missing refs and invalid options are tool errors, not merge conflicts.
expect_status 2 run_preflight "$tmp_root/conflict" upstream/no-such-ref >/dev/null
expect_status 2 run_preflight "$tmp_root/conflict" --definitely-invalid >/dev/null
expect_status 2 run_preflight "$tmp_root/conflict" -x >/dev/null
expect_status 2 run_preflight "$tmp_root/conflict" --fetch HEAD >/dev/null

# Explicit fetch advances only the remote-tracking branch; local HEAD stays put.
new_repo "$tmp_root/remote"
git -C "$tmp_root/remote" branch -M main
git clone -q "$tmp_root/remote" "$tmp_root/fetch"
git -C "$tmp_root/fetch" remote rename origin upstream
fetch_head=$(git -C "$tmp_root/fetch" rev-parse HEAD)
printf 'new\n' >"$tmp_root/remote/new.txt"
git -C "$tmp_root/remote" add new.txt
git -C "$tmp_root/remote" commit -qm new
expect_status 0 run_preflight "$tmp_root/fetch" --fetch >/dev/null
[[ "$(git -C "$tmp_root/fetch" rev-parse upstream/main)" == "$(git -C "$tmp_root/remote" rev-parse HEAD)" ]] || fail "fetch did not update target"
[[ "$(git -C "$tmp_root/fetch" rev-parse HEAD)" == "$fetch_head" ]] || fail "fetch moved HEAD"

echo "upstream preflight fixture tests: PASS"
