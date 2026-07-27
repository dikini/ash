#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
HOOK_UNDER_TEST="$ROOT/.githooks/pre-push"

if [[ ! -f "$HOOK_UNDER_TEST" ]]; then
  echo "test setup failed: missing pre-push hook: $HOOK_UNDER_TEST" >&2
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
  mkdir -p "$repo/.githooks" "$repo/scripts"
  cp "$HOOK_UNDER_TEST" "$repo/.githooks/pre-push"
  cat >"$repo/scripts/check-full-gate.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
printf 'FULL_GATE_RAN\n' >>"${FULL_GATE_LOG:?}"
SCRIPT
  chmod +x "$repo/.githooks/pre-push" "$repo/scripts/check-full-gate.sh"
  printf 'tracked base\n' >"$repo/tracked.txt"

  clean_git_env git -C "$repo" init -q
  clean_git_env git -C "$repo" config user.email "ash-pre-push-tests@example.invalid"
  clean_git_env git -C "$repo" config user.name "Ash Pre-push Tests"
  clean_git_env git -C "$repo" add .
  clean_git_env git -C "$repo" -c commit.gpgsign=false commit -q -m initial
  printf '%s\n' "$repo"
}

zero_sha='0000000000000000000000000000000000000000'
hook_status=0
run_hook() {
  local repo="$1"
  local label="$2"
  local ref_line="$3"
  local output="$tmp/${label}.out"
  local full_gate_log="$tmp/${label}.full-gate"
  : >"$full_gate_log"

  if printf '%s\n' "$ref_line" | (
    cd "$repo"
    export FULL_GATE_LOG="$full_gate_log"
    clean_git_env bash .githooks/pre-push
  ) >"$output" 2>&1; then
    hook_status=0
  else
    hook_status=$?
  fi
}

failures=0
record_failure() {
  echo "FAIL: $*" >&2
  failures=$((failures + 1))
}

assert_hook_success() {
  local label="$1"
  if [[ "$hook_status" -ne 0 ]]; then
    record_failure "expected pre-push hook success for $label"
    cat "$tmp/${label}.out" >&2
  fi
}

assert_hook_rejected() {
  local label="$1"
  if [[ "$hook_status" -eq 0 ]]; then
    record_failure "expected pre-push hook rejection for $label"
    cat "$tmp/${label}.out" >&2
  fi
}

assert_full_gate_ran() {
  local label="$1"
  local log="$tmp/${label}.full-gate"
  if ! grep -Fxq FULL_GATE_RAN "$log"; then
    record_failure "expected full gate invocation for $label"
    cat "$tmp/${label}.out" >&2
  fi
}

assert_full_gate_not_ran() {
  local label="$1"
  local log="$tmp/${label}.full-gate"
  if [[ -s "$log" ]]; then
    record_failure "expected rejection before full gate for $label"
    cat "$log" >&2
    cat "$tmp/${label}.out" >&2
  fi
}

head_sha() {
  clean_git_env git -C "$1" rev-parse HEAD
}

head_push_line() {
  local head
  head="$(head_sha "$1")"
  printf 'refs/heads/main %s refs/heads/main %s' "$head" "$zero_sha"
}

# A normal update that names the current checkout is the only context that may
# advance to the expensive full gate.
repo="$(make_repo)"
run_hook "$repo" clean_head "$(head_push_line "$repo")"
assert_hook_success clean_head
assert_full_gate_ran clean_head

# The hook must consume standard pre-push stdin rather than silently validating
# a different checkout.  The pushed local object is a real, non-delete commit,
# but it is not the current HEAD.
repo="$(make_repo)"
stale_sha="$(head_sha "$repo")"
printf 'second commit\n' >>"$repo/tracked.txt"
clean_git_env git -C "$repo" add tracked.txt
clean_git_env git -C "$repo" -c commit.gpgsign=false commit -q -m second
run_hook "$repo" mismatched_local_object "refs/heads/main $stale_sha refs/heads/main $zero_sha"
assert_hook_rejected mismatched_local_object
assert_full_gate_not_ran mismatched_local_object

# Even a normal HEAD update must not run a full gate against an uncommitted
# tracked worktree, whose contents differ from the pushed object.
repo="$(make_repo)"
printf 'dirty worktree\n' >>"$repo/tracked.txt"
run_hook "$repo" dirty_worktree "$(head_push_line "$repo")"
assert_hook_rejected dirty_worktree
assert_full_gate_not_ran dirty_worktree

# Staged tracked changes create index/HEAD divergence and are equally invalid
# pre-push context; the hook must reject before the full gate sees them.
repo="$(make_repo)"
printf 'staged index change\n' >>"$repo/tracked.txt"
clean_git_env git -C "$repo" add tracked.txt
run_hook "$repo" dirty_index "$(head_push_line "$repo")"
assert_hook_rejected dirty_index
assert_full_gate_not_ran dirty_index

if [[ "$failures" -ne 0 ]]; then
  exit 1
fi

echo "check-pre-push-semantic-context-tests: OK"
