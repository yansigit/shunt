#!/usr/bin/env bash
set -u

usage() {
  cat <<'EOF'
Usage: scripts/upstream-preflight.sh [--fetch] [<upstream-ref>]

Compare HEAD with upstream/main using git merge-tree. The default mode does
not contact the network and does not touch the index or working tree. --fetch
updates only the upstream remote before running the same check.
EOF
}
die() { echo "upstream-preflight: $*" >&2; exit 2; }

fetch=0
ref=upstream/main
ref_given=0
while (($#)); do
  case "$1" in
    --fetch) fetch=1; shift ;;
    -h|--help) usage; exit 0 ;;
    -*) die "unknown option: $1" ;;
    *)
      ((ref_given == 0)) || die "expected at most one upstream ref"
      ref_given=1
      ref=$1
      shift
      ;;
  esac
done

git rev-parse --git-dir >/dev/null 2>&1 || die "not inside a git repository"
git rev-parse --verify HEAD^{commit} >/dev/null 2>&1 || die "HEAD has no commit"

if ((fetch)); then
  [[ "$ref" == upstream/* && "${ref#upstream/}" != "$ref" ]] || \
    die "--fetch only permits refs under upstream/ (got $ref)"
  branch=${ref#upstream/}
  [[ -n "$branch" ]] || die "upstream branch is empty"
  git remote get-url upstream >/dev/null 2>&1 || die "remote 'upstream' is not configured"
  git check-ref-format "refs/remotes/upstream/$branch" >/dev/null 2>&1 || \
    die "invalid upstream branch: $branch"
  echo "Fetching upstream/$branch..."
  git fetch --no-tags --no-recurse-submodules upstream \
    "refs/heads/$branch:refs/remotes/upstream/$branch" || die "fetch failed"
fi

head=$(git rev-parse --verify HEAD^{commit}) || die "cannot resolve HEAD"
target=$(git rev-parse --verify --end-of-options "$ref^{commit}") || die "cannot resolve ref: $ref"
counts=$(git rev-list --left-right --count "$head...$target") || die "cannot calculate ahead/behind"
ahead=${counts%%[[:space:]]*}
behind=${counts#*[[:space:]]}
echo "Current: $head"
echo "Upstream: $ref ($target)"
echo "Ahead: $ahead  Behind: $behind"

set +e
merge_output=$(git merge-tree --write-tree --name-only --messages "$head" "$target" 2>&1)
merge_status=$?
set -e
case "$merge_status" in
  0)
    echo "Merge-tree preflight: clean"
    exit 0
    ;;
  1)
    echo "Merge-tree preflight: conflicts"
    conflict_files=$(printf '%s\n' "$merge_output" | sed -n 's/^CONFLICT .*: //p')
    if [[ -n "$conflict_files" ]]; then
      echo "Conflicting files:"
      printf '%s\n' "$conflict_files"
    fi
    echo "Merge-tree diagnostics:"
    printf '%s\n' "$merge_output"
    exit 1
    ;;
  *)
    echo "Merge-tree preflight: unable to complete" >&2
    printf '%s\n' "$merge_output" >&2
    exit 2
    ;;
esac
