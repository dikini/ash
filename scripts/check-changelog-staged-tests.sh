#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
SCRIPT_UNDER_TEST="$ROOT/scripts/check-changelog-staged.sh"

if [[ ! -x "$SCRIPT_UNDER_TEST" && ! -f "$SCRIPT_UNDER_TEST" ]]; then
  echo "test setup failed: $SCRIPT_UNDER_TEST not found" >&2
  exit 2
fi

tmp="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp"
}
trap cleanup EXIT

clean_git_env() {
  local env_args=()
  local var
  while IFS= read -r var; do
    env_args+=("-u" "$var")
  done < <(git rev-parse --local-env-vars)
  env "${env_args[@]}" "$@"
}

make_repo() {
  local repo
  repo="$(mktemp -d "$tmp/repo.XXXXXX")"
  mkdir -p "$repo/scripts" "$repo/docs" "$repo/.github" "$repo/src"
  cp "$SCRIPT_UNDER_TEST" "$repo/scripts/check-changelog-staged.sh"
  chmod +x "$repo/scripts/check-changelog-staged.sh"
  printf '# Changelog\n\n## [Unreleased]\n' >"$repo/CHANGELOG.md"
  touch "$repo/docs/.keep" "$repo/.github/.keep" "$repo/src/.keep"
  clean_git_env git -C "$repo" init -q
  clean_git_env git -C "$repo" config user.email "ash-tests@example.invalid"
  clean_git_env git -C "$repo" config user.name "Ash Test"
  clean_git_env git -C "$repo" add .
  clean_git_env git -C "$repo" -c commit.gpgsign=false commit -q -m "initial"
  printf '%s\n' "$repo"
}

run_check() {
  local repo="$1"
  local out="$2"
  set +e
  (cd "$repo" && clean_git_env bash scripts/check-changelog-staged.sh) >"$out" 2>&1
  local status=$?
  set -e
  return "$status"
}

assert_success() {
  local label="$1"
  local repo="$2"
  local out="$tmp/$label.out"
  if ! run_check "$repo" "$out"; then
    echo "FAIL: expected success for $label" >&2
    cat "$out" >&2
    exit 1
  fi
}

assert_failure() {
  local label="$1"
  local repo="$2"
  local out="$tmp/$label.out"
  if run_check "$repo" "$out"; then
    echo "FAIL: expected failure for $label" >&2
    cat "$out" >&2
    exit 1
  fi
}

assert_output_contains() {
  local label="$1"
  local expected="$2"
  local out="$tmp/$label.out"
  if ! grep -Fq "$expected" "$out"; then
    echo "FAIL: expected $label output to contain: $expected" >&2
    cat "$out" >&2
    exit 1
  fi
}

repo="$(make_repo)"
printf 'docs-only staged change\n' >"$repo/docs/docs-only.md"
clean_git_env git -C "$repo" add docs/docs-only.md
assert_success "docs-only" "$repo"
assert_output_contains "docs-only" "changelog-check: no relevant staged files, skipping"

repo="$(make_repo)"
printf 'workflow-only staged change\n' >"$repo/.github/workflow.yml"
clean_git_env git -C "$repo" add .github/workflow.yml
assert_success "github-only" "$repo"
assert_output_contains "github-only" "changelog-check: no relevant staged files, skipping"

repo="$(make_repo)"
printf 'fn main() {}\n' >"$repo/src/main.rs"
clean_git_env git -C "$repo" add src/main.rs
assert_failure "source-without-changelog" "$repo"
assert_output_contains "source-without-changelog" "changelog-check: FAILED"

repo="$(make_repo)"
printf 'fn main() {}\n' >"$repo/src/main.rs"
printf '\n- Test changelog entry.\n' >>"$repo/CHANGELOG.md"
clean_git_env git -C "$repo" add src/main.rs CHANGELOG.md
assert_success "source-with-changelog" "$repo"
assert_output_contains "source-with-changelog" "changelog-check: CHANGELOG.md is staged"

# CHANGELOG.md sorts first. Keep the remaining staged-name stream larger than a
# pipe buffer so an early-exiting grep -q makes the producer observe SIGPIPE.
repo="$(make_repo)"
printf '\n- Test changelog entry.\n' >>"$repo/CHANGELOG.md"
mkdir -p "$repo/src/sigpipe-regression"
for index in $(seq 1 5000); do
  printf -v staged_file 'src/sigpipe-regression/%05d-%080d.rs' "$index" 0
  : >"$repo/$staged_file"
done
clean_git_env git -C "$repo" add CHANGELOG.md src/sigpipe-regression
assert_success "changelog-first-large-staged-list" "$repo"
assert_output_contains "changelog-first-large-staged-list" "changelog-check: CHANGELOG.md is staged"

echo "check-changelog-staged-tests: OK"
