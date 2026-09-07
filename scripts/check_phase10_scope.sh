#!/usr/bin/env bash
set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

summary=.planning/phases/10-gemini-semantic-hardening/10-01-SUMMARY.md
if [[ ! -f "$summary" ]]; then
  echo "phase 10 scope audit: missing $summary" >&2
  exit 1
fi

phase_base=$(
  awk '/^plan_head_before:/ { value = $2; gsub(/["\047]/, "", value); print value; exit }' "$summary"
)
if [[ -z "$phase_base" ]]; then
  echo "phase 10 scope audit: plan_head_before is empty in $summary" >&2
  exit 1
fi
if ! git cat-file -e "$phase_base^{commit}" 2>/dev/null; then
  echo "phase 10 scope audit: plan_head_before is not a commit: $phase_base" >&2
  exit 1
fi

check_unchanged() {
  local label=$1
  shift
  if ! git diff --quiet "$phase_base"..HEAD -- "$@"; then
    echo "phase 10 scope audit: forbidden $label change" >&2
    git diff --name-only "$phase_base"..HEAD -- "$@" >&2
    exit 1
  fi
}

check_unchanged "credential persistence" \
  src/auth \
  ':(exclude)src/auth/mod.rs' \
  src/state_persist.rs \
  src/gateway/persist.rs \
  src/gateway/store.rs

# Plan 10-07 deliberately adds one crate-private resolver boundary to
# `src/auth/mod.rs` so hermetic tests can inject a synthetic Google OAuth
# identity. Pin that file's complete phase diff byte-for-byte: this permits the
# reviewed seam while continuing to reject any credential resolution or
# writeback change hidden elsewhere in the module.
expected_auth_mod_diff_sha256=d4931b1ab6407f4f0f9801c01cb95e9308ba9b69fda009b768be504be54fc1bd
auth_mod_diff_sha256=$(
  git diff --no-ext-diff "$phase_base"..HEAD -- src/auth/mod.rs | shasum -a 256 | awk '{print $1}'
)
if [[ "$auth_mod_diff_sha256" != "$expected_auth_mod_diff_sha256" ]]; then
  echo "phase 10 scope audit: unexpected src/auth/mod.rs change" >&2
  exit 1
fi
check_unchanged "Antigravity implementation" \
  src/adapters/antigravity.rs \
  src/adapters/antigravity \
  src/auth/antigravity \
  tests/antigravity.rs \
  tests/antigravity
check_unchanged "public config or provider declaration" \
  src/config.rs \
  src/config \
  src/adapters/mod.rs \
  src/routing.rs
check_unchanged "dependency manifest" \
  Cargo.toml \
  Cargo.lock \
  site/package.json \
  site/package-lock.json \
  wiki/package.json \
  wiki/package-lock.json
check_unchanged "generated wiki" wiki

# The Gemini adapter is shared by API-key Code Assist and Antigravity OAuth.
# Pin the pre-Phase-10 Antigravity retry contract behaviorally instead of
# pretending path-only checks can exclude changes in this shared file.
cargo test --all-features antigravity_retry_safety_remains_idempotent_until_phase_11 --lib

added_source_test_lines() {
  git diff --no-ext-diff --unified=0 "$phase_base"..HEAD -- src tests |
    sed -n '/^+[^+]/p'
}

added_lines=$(added_source_test_lines)

if printf '%s\n' "$added_lines" |
  grep -Ei '(SAPISIDHASH|MakerSuite|WebKit|browser[ _-]?daemon)' >/dev/null; then
  echo "phase 10 scope audit: AI Studio Web implementation identifier in added source/test lines" >&2
  exit 1
fi

fixture_suspects=$(
  printf '%s\n' "$added_lines" |
    awk '
      BEGIN { IGNORECASE = 1 }
      /(authorization[^[:cntrl:]]*bearer|bearer[[:space:]]+[A-Za-z0-9._~-]{12,}|"(project|project_id|projectId)"[[:space:]]*:[[:space:]]*"|private[ _-]?prompt|systemInstruction[^[:cntrl:]]*"text"[[:space:]]*:)/ &&
        !/(synthetic|test[-_ ]|example|dummy|redacted|fixture|not-a-real)/ { print }
    '
)
if [[ -n "$fixture_suspects" ]]; then
  echo "phase 10 scope audit: non-synthetic credential, project, or private prompt fixture material" >&2
  exit 1
fi

echo "phase 10 scope audit: pass ($phase_base..HEAD)"
