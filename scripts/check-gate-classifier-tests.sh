#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
SCRIPT_UNDER_TEST="$ROOT/scripts/check-gate-classifier.sh"

if [[ ! -f "$SCRIPT_UNDER_TEST" ]]; then
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
  mkdir -p "$repo/scripts" "$repo/docs/design" "$repo/crates/ash-core/src" "$repo/crates/ash-fuzz" "$repo/.githooks"
  cp "$SCRIPT_UNDER_TEST" "$repo/scripts/check-gate-classifier.sh"
  chmod +x "$repo/scripts/check-gate-classifier.sh"
  printf '# Changelog\n\n## [Unreleased]\n' >"$repo/CHANGELOG.md"
  printf '# Existing\n' >"$repo/docs/design/EXISTING.md"
  printf 'pub fn existing() {}\n' >"$repo/crates/ash-core/src/lib.rs"
  printf '#!/usr/bin/env bash\n' >"$repo/.githooks/pre-commit"
  clean_git_env git -C "$repo" init -q
  clean_git_env git -C "$repo" config user.email "ash-tests@example.invalid"
  clean_git_env git -C "$repo" config user.name "Ash Test"
  clean_git_env git -C "$repo" add .
  clean_git_env git -C "$repo" -c commit.gpgsign=false commit -q -m initial
  printf '%s\n' "$repo"
}

run_classifier() {
  local repo="$1"
  local out="$2"
  set +e
  (cd "$repo" && clean_git_env bash scripts/check-gate-classifier.sh) >"$out" 2>&1
  local status=$?
  set -e
  return "$status"
}

assert_success() {
  local label="$1"
  local repo="$2"
  local out="$tmp/$label.out"
  if ! run_classifier "$repo" "$out"; then
    echo "FAIL: expected classifier success for $label" >&2
    cat "$out" >&2
    exit 1
  fi
}

assert_contains() {
  local label="$1"
  local expected="$2"
  local out="$tmp/$label.out"
  if ! grep -Fxq "$expected" "$out"; then
    echo "FAIL: expected $label output line: $expected" >&2
    cat "$out" >&2
    exit 1
  fi
}

repo="$(make_repo)"
printf '# Design\n' >"$repo/docs/design/DESIGN-999.md"
printf '\n- Docs entry.\n' >>"$repo/CHANGELOG.md"
clean_git_env git -C "$repo" add docs/design/DESIGN-999.md CHANGELOG.md
assert_success docs_only "$repo"
assert_contains docs_only "docs_only=true"
assert_contains docs_only "rust_relevant=false"
assert_contains docs_only "fuzz_relevant=false"
assert_contains docs_only "gate_relevant=false"
assert_contains docs_only "unknown_relevant=false"

repo="$(make_repo)"
printf 'pub fn changed() {}\n' >>"$repo/crates/ash-core/src/lib.rs"
clean_git_env git -C "$repo" add crates/ash-core/src/lib.rs
assert_success rust_change "$repo"
assert_contains rust_change "docs_only=false"
assert_contains rust_change "rust_relevant=true"
assert_contains rust_change "fuzz_relevant=true"
assert_contains rust_change "gate_relevant=false"

repo="$(make_repo)"
printf 'fuzz config\n' >"$repo/crates/ash-fuzz/Cargo.toml"
clean_git_env git -C "$repo" add crates/ash-fuzz/Cargo.toml
assert_success fuzz_change "$repo"
assert_contains fuzz_change "docs_only=false"
assert_contains fuzz_change "rust_relevant=true"
assert_contains fuzz_change "fuzz_relevant=true"

repo="$(make_repo)"
printf '#!/usr/bin/env bash\necho changed\n' >"$repo/scripts/new-gate.sh"
clean_git_env git -C "$repo" add scripts/new-gate.sh
assert_success gate_change "$repo"
assert_contains gate_change "docs_only=false"
assert_contains gate_change "rust_relevant=true"
assert_contains gate_change "gate_relevant=true"

repo="$(make_repo)"
printf 'unknown\n' >"$repo/random.asset"
clean_git_env git -C "$repo" add random.asset
assert_success unknown_change "$repo"
assert_contains unknown_change "docs_only=false"
assert_contains unknown_change "rust_relevant=true"
assert_contains unknown_change "unknown_relevant=true"

echo "check-gate-classifier-tests: OK"
