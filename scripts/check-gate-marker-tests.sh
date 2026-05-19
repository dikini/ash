#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
HELPERS_UNDER_TEST="$ROOT/scripts/gate-helpers.sh"
FULL_GATE_UNDER_TEST="$ROOT/scripts/check-full-gate.sh"

if [[ ! -f "$HELPERS_UNDER_TEST" || ! -f "$FULL_GATE_UNDER_TEST" ]]; then
  echo "test setup failed: gate helper/full gate scripts not found" >&2
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
  mkdir -p "$repo/scripts" "$repo/.git-hooks-placeholder"
  cp "$HELPERS_UNDER_TEST" "$repo/scripts/gate-helpers.sh"
  cp "$FULL_GATE_UNDER_TEST" "$repo/scripts/check-full-gate.sh"
  cat >"$repo/scripts/check-gate-classifier.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
echo "docs_only=false"
echo "rust_relevant=true"
echo "fuzz_relevant=false"
echo "gate_relevant=false"
echo "unknown_relevant=false"
SCRIPT
  cat >"$repo/scripts/check-pre-commit-gate.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
echo "PRE_COMMIT_RAN"
SCRIPT
  cat >"$repo/scripts/check-rust-tests.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
echo "RUST_TESTS_RAN $*"
SCRIPT
  cat >"$repo/scripts/check-fuzz.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
echo "FUZZ_RAN $*"
SCRIPT
  chmod +x "$repo"/scripts/*.sh
  printf '# Changelog\n' >"$repo/CHANGELOG.md"
  clean_git_env git -C "$repo" init -q
  clean_git_env git -C "$repo" config user.email "ash-tests@example.invalid"
  clean_git_env git -C "$repo" config user.name "Ash Test"
  clean_git_env git -C "$repo" add .
  clean_git_env git -C "$repo" -c commit.gpgsign=false commit -q -m initial
  printf '%s\n' "$repo"
}

run_script() {
  local repo="$1"
  local out="$2"
  set +e
  (cd "$repo" && clean_git_env bash scripts/check-full-gate.sh --no-marker) >"$out" 2>&1
  local status=$?
  set -e
  return "$status"
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

assert_output_not_contains() {
  local label="$1"
  local unexpected="$2"
  local out="$tmp/$label.out"
  if grep -Fq "$unexpected" "$out"; then
    echo "FAIL: expected $label output not to contain: $unexpected" >&2
    cat "$out" >&2
    exit 1
  fi
}

repo="$(make_repo)"
(
  cd "$repo"
  source scripts/gate-helpers.sh
  git_dir="$(git rev-parse --git-dir)"
  gate_write_marker "$git_dir/.pre-commit-gate.ok"
)
if ! run_script "$repo" "$tmp/fresh-marker.out"; then
  echo "FAIL: full gate with fresh marker failed" >&2
  cat "$tmp/fresh-marker.out" >&2
  exit 1
fi
assert_output_contains fresh-marker "full-gate: reusing fresh pre-commit marker"
assert_output_not_contains fresh-marker "PRE_COMMIT_RAN"
assert_output_contains fresh-marker "RUST_TESTS_RAN --workspace --all-targets"
assert_output_contains fresh-marker "FUZZ_RAN"

repo="$(make_repo)"
(
  cd "$repo"
  source scripts/gate-helpers.sh
  git_dir="$(git rev-parse --git-dir)"
  gate_write_marker "$git_dir/.pre-commit-gate.ok"
  printf 'changed\n' >>CHANGELOG.md
  clean_git_env git add CHANGELOG.md
)
if ! run_script "$repo" "$tmp/stale-marker.out"; then
  echo "FAIL: full gate with stale marker failed" >&2
  cat "$tmp/stale-marker.out" >&2
  exit 1
fi
assert_output_contains stale-marker "full-gate: no fresh pre-commit marker; running pre-commit gate"
assert_output_contains stale-marker "PRE_COMMIT_RAN"

repo="$(make_repo)"
cat >"$repo/scripts/check-gate-classifier.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
echo "docs_only=true"
echo "rust_relevant=false"
echo "fuzz_relevant=false"
echo "gate_relevant=false"
echo "unknown_relevant=false"
SCRIPT
cat >"$repo/scripts/check-docs-gate.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
echo "DOCS_GATE_RAN"
SCRIPT
chmod +x "$repo/scripts/check-gate-classifier.sh" "$repo/scripts/check-docs-gate.sh"
if ! run_script "$repo" "$tmp/docs-only.out"; then
  echo "FAIL: full gate docs-only path failed" >&2
  cat "$tmp/docs-only.out" >&2
  exit 1
fi
assert_output_contains docs-only "full-gate: docs-only change set; running docs gate only"
assert_output_contains docs-only "DOCS_GATE_RAN"
assert_output_not_contains docs-only "PRE_COMMIT_RAN"
assert_output_not_contains docs-only "RUST_TESTS_RAN"
assert_output_not_contains docs-only "FUZZ_RAN"

echo "check-gate-marker-tests: OK"
