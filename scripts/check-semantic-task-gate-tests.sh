#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
RUNNER_UNDER_TEST="$ROOT/scripts/check-semantic-task-gate.sh"
VALIDATOR_UNDER_TEST="$ROOT/tools/docs/validate_semantic_task_records.py"

if [[ ! -f "$RUNNER_UNDER_TEST" ]]; then
  echo "test setup failed: missing semantic task gate runner: $RUNNER_UNDER_TEST" >&2
  exit 2
fi

if [[ ! -f "$VALIDATOR_UNDER_TEST" ]]; then
  echo "test setup failed: missing semantic task record validator: $VALIDATOR_UNDER_TEST" >&2
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

write_task_file() {
  local repo="$1"
  local task="$2"
  local number="${task#TASK-}"

  cat >"$repo/docs/plan/tasks/${task}-fixture.md" <<EOF
# ${task}: Semantic task gate fixture

**Status:** In progress

**Semantic task record:** [${task}](../semantic-task-records.json)

**Semantic coverage map:** [${task} workflow record](../SEMANTIC-RULE-COVERAGE.md#${task,,}-workflow-record)

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Fixture omits target-spec clauses.

## Evidence

This fixture owns the traceability anchors for TASK-${number}.
EOF
}

write_coverage_map() {
  local repo="$1"
  shift
  local task
  local number

  cat >"$repo/docs/plan/SEMANTIC-RULE-COVERAGE.md" <<'EOF'
# Semantic Rule Coverage Map
EOF
  for task in "$@"; do
    number="${task#TASK-}"
    cat >>"$repo/docs/plan/SEMANTIC-RULE-COVERAGE.md" <<EOF

## ${task} workflow record

**Task:** [${task}](tasks/${task}-fixture.md)
**Canonical rules:** \`SEM-RULE-${number}\`
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Fixture omits target-spec clauses.
**Layers:** type partial; core partial; cps partial; admission-runtime partial; verification partial.
**Evidence:**
- **Positive:** \`TEST-${number}-POSITIVE\`
- **Negative:** \`TEST-${number}-NEGATIVE\`
- **Mutation:** \`TEST-${number}-MUTATION\`
- **Parity:** not applicable; this fixture has no reference interpreter pair.
**Non-goals:** The fixture does not implement the target specification.
**Next obligation:** Add target-spec clauses and evidence.
EOF
  done
}

write_traceability() {
  local repo="$1"
  shift
  local task
  local number
  local first=true
  local node_first=true

  {
    printf '{"schema":"semantic-traceability-graph/v2","nodes":['
    for task in "$@"; do
      number="${task#TASK-}"
      for node in "SEM-RULE-${number}:canonical-rule" "TEST-${number}-POSITIVE:test" "TEST-${number}-NEGATIVE:test" "TEST-${number}-MUTATION:test"; do
        if [[ "$node_first" == false ]]; then
          printf ','
        fi
        node_first=false
        printf '{"id":"%s","kind":"%s","status":["%s"],"anchor":"docs/plan/tasks/%s-fixture.md#evidence"}' \
          "${node%%:*}" "${node##*:}" "$([[ "$node" == SEM-RULE-* ]] && printf specified || printf tested)" "$task"
      done
    done
    printf '],"edges":['
    for task in "$@"; do
      number="${task#TASK-}"
      for evidence in POSITIVE NEGATIVE MUTATION; do
        if [[ "$first" == false ]]; then
          printf ','
        fi
        first=false
        printf '{"kind":"tested_by","from":"SEM-RULE-%s","to":"TEST-%s-%s","anchor":"docs/plan/tasks/%s-fixture.md#evidence"}' \
          "$number" "$number" "$evidence" "$task"
      done
    done
    printf ']}'
  } >"$repo/docs/spec/SEMANTIC-TRACEABILITY.json"
}

write_manifest() {
  local repo="$1"
  shift
  local task
  local number
  local first=true

  {
    printf '{"schema":"semantic-task-records/v2","active_scope":{"kind":"fixture","tasks":['
    for task in "$@"; do
      if [[ "$first" == false ]]; then
        printf ','
      fi
      first=false
      printf '"%s"' "$task"
    done
    printf ']},"active_tasks":['
    first=true
    for task in "$@"; do
      if [[ "$first" == false ]]; then
        printf ','
      fi
      first=false
      printf '"%s"' "$task"
    done
    printf '],"records":['
    first=true
    for task in "$@"; do
      number="${task#TASK-}"
      if [[ "$first" == false ]]; then
        printf ','
      fi
      first=false
      printf '%s' \
        "{\"task\":\"${task}\",\"task_file\":\"docs/plan/tasks/${task}-fixture.md\",\"coverage_map\":\"docs/plan/SEMANTIC-RULE-COVERAGE.md#${task,,}-workflow-record\",\"canonical_rule_ids\":[\"SEM-RULE-${number}\"],\"implementation\":\"partial\",\"layers\":{\"type\":\"partial\",\"core\":\"partial\",\"cps\":\"partial\",\"admission_runtime\":\"partial\",\"verification\":\"partial\"},\"evidence\":{\"status\":\"tested\",\"proofs\":[],\"positive\":[\"TEST-${number}-POSITIVE\"],\"negative\":[\"TEST-${number}-NEGATIVE\"],\"mutation\":[\"TEST-${number}-MUTATION\"],\"parity\":{\"status\":\"not_applicable\",\"rationale\":\"The fixture has no reference interpreter pair.\"}},\"parity\":\"below_spec\",\"missing_spec_clauses\":[\"Fixture omits target-spec clauses.\"],\"non_goals\":[\"The fixture does not implement the target specification.\"],\"next_obligation\":\"Add target-spec clauses and evidence.\",\"verification\":[\"cargo test -p ash-engine --test task_${number}_fixture\"]}"
    done
    printf ']}'
  } >"$repo/docs/plan/semantic-task-records.json"
}

make_repo() {
  local repo
  repo="$(mktemp -d "$tmp/repo.XXXXXX")"
  local tasks=("$@")

  mkdir -p \
    "$repo/scripts" \
    "$repo/tools/docs" \
    "$repo/docs/plan/tasks" \
    "$repo/docs/spec" \
    "$repo/docs/design" \
    "$repo/crates/ash-engine/src" \
    "$repo/test-bin"
  cp "$RUNNER_UNDER_TEST" "$repo/scripts/check-semantic-task-gate.sh"
  cp "$VALIDATOR_UNDER_TEST" "$repo/tools/docs/validate_semantic_task_records.py"
  chmod +x "$repo/scripts/check-semantic-task-gate.sh"
  cat >"$repo/test-bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'cargo' >>"${CARGO_LOG:?}"
printf ' %s' "$@" >>"${CARGO_LOG:?}"
printf '\n' >>"${CARGO_LOG:?}"
printf '%s\n' "${CARGO_TARGET_DIR-}" >>"${CARGO_TARGET_DIR_LOG:?}"
printf '%s\n' "$PWD" >>"${CARGO_CWD_LOG:?}"
EOF
  chmod +x "$repo/test-bin/cargo"
  printf '# Changelog\n\n## [Unreleased]\n' >"$repo/CHANGELOG.md"
  printf 'pub fn semantic_task_fixture() {}\n' >"$repo/crates/ash-engine/src/semantic_task_fixture.rs"

  local task
  for task in "${tasks[@]}"; do
    write_task_file "$repo" "$task"
  done
  write_coverage_map "$repo" "${tasks[@]}"
  write_traceability "$repo" "${tasks[@]}"
  write_manifest "$repo" "${tasks[@]}"

  clean_git_env git -C "$repo" init -q
  clean_git_env git -C "$repo" config user.email "ash-gate-tests@example.invalid"
  clean_git_env git -C "$repo" config user.name "Ash Gate Tests"
  clean_git_env git -C "$repo" add .
  clean_git_env git -C "$repo" -c commit.gpgsign=false commit -q -m initial
  printf '%s\n' "$repo"
}

stage_matching_semantic_evidence() {
  local repo="$1"
  local task="$2"
  local omitted_path="${3:-}"

  printf '// staged semantic change\n' >>"$repo/crates/ash-engine/src/semantic_task_fixture.rs"
  printf '\n<!-- matching staged task evidence -->\n' >>"$repo/docs/plan/tasks/${task}-fixture.md"
  printf '\n' >>"$repo/docs/plan/semantic-task-records.json"
  printf '\n<!-- matching staged coverage evidence -->\n' >>"$repo/docs/plan/SEMANTIC-RULE-COVERAGE.md"
  printf '\n' >>"$repo/docs/spec/SEMANTIC-TRACEABILITY.json"
  printf '\n- Semantic task gate fixture evidence.\n' >>"$repo/CHANGELOG.md"

  clean_git_env git -C "$repo" add crates/ash-engine/src/semantic_task_fixture.rs
  if [[ "$omitted_path" != "task_file" ]]; then
    clean_git_env git -C "$repo" add "docs/plan/tasks/${task}-fixture.md"
  fi
  if [[ "$omitted_path" != "manifest" ]]; then
    clean_git_env git -C "$repo" add docs/plan/semantic-task-records.json
  fi
  if [[ "$omitted_path" != "coverage_map" ]]; then
    clean_git_env git -C "$repo" add docs/plan/SEMANTIC-RULE-COVERAGE.md
  fi
  if [[ "$omitted_path" != "traceability" ]]; then
    clean_git_env git -C "$repo" add docs/spec/SEMANTIC-TRACEABILITY.json
  fi
  if [[ "$omitted_path" != "changelog" ]]; then
    clean_git_env git -C "$repo" add CHANGELOG.md
  fi
}

gate_status=0
run_gate() {
  local repo="$1"
  local label="$2"
  shift 2
  local cargo_log="$tmp/${label}.cargo"
  local cargo_target_dir_log="$tmp/${label}.cargo-target-dir"
  local cargo_cwd_log="$tmp/${label}.cargo-cwd"
  local output="$tmp/${label}.out"
  : >"$cargo_log"
  : >"$cargo_target_dir_log"
  : >"$cargo_cwd_log"
  if (
    cd "$repo"
    export CARGO_LOG="$cargo_log"
    export CARGO_TARGET_DIR_LOG="$cargo_target_dir_log"
    export CARGO_CWD_LOG="$cargo_cwd_log"
    export PATH="$repo/test-bin:$PATH"
    unset CARGO_TARGET_DIR
    clean_git_env bash scripts/check-semantic-task-gate.sh "$@"
  ) >"$output" 2>&1; then
    gate_status=0
  else
    gate_status=$?
  fi
}

assert_success() {
  local label="$1"
  local repo="$2"
  shift 2
  run_gate "$repo" "$label" "$@"
  if [[ "$gate_status" -ne 0 ]]; then
    echo "FAIL: expected semantic task gate success for $label" >&2
    cat "$tmp/${label}.out" >&2
    exit 1
  fi
}

assert_failure_without_cargo() {
  local label="$1"
  local repo="$2"
  shift 2
  run_gate "$repo" "$label" "$@"
  if [[ "$gate_status" -eq 0 ]]; then
    echo "FAIL: expected semantic task gate failure for $label" >&2
    cat "$tmp/${label}.out" >&2
    exit 1
  fi
  if [[ -s "$tmp/${label}.cargo" ]]; then
    echo "FAIL: semantic task gate ran cargo before rejecting $label" >&2
    cat "$tmp/${label}.cargo" >&2
    cat "$tmp/${label}.out" >&2
    exit 1
  fi
}

assert_cargo_commands() {
  local label="$1"
  shift
  local expected
  local actual="$tmp/${label}.cargo"
  local expected_file="$tmp/${label}.expected"
  : >"$expected_file"
  for expected in "$@"; do
    printf '%s\n' "$expected" >>"$expected_file"
  done
  if ! cmp -s "$expected_file" "$actual"; then
    echo "FAIL: unexpected cargo commands for $label" >&2
    echo 'expected:' >&2
    cat "$expected_file" >&2
    echo 'actual:' >&2
    cat "$actual" >&2
    exit 1
  fi
}

assert_cargo_target_dir_is_snapshot_local() {
  local label="$1"
  local repo="$2"
  local target_dir
  local command_cwd
  target_dir="$(<"$tmp/${label}.cargo-target-dir")"
  command_cwd="$(<"$tmp/${label}.cargo-cwd")"

  if [[ "$command_cwd" != */index ]]; then
    echo "FAIL: semantic task command did not run from its staged snapshot for $label" >&2
    echo "actual command cwd: $command_cwd" >&2
    exit 1
  fi
  if [[ "$target_dir" != "$command_cwd/target" ]]; then
    echo "FAIL: CARGO_TARGET_DIR must be local to the staged snapshot for $label" >&2
    echo "expected: $command_cwd/target" >&2
    echo "actual: $target_dir" >&2
    exit 1
  fi
  if [[ "$target_dir" == "$repo/target" ]]; then
    echo "FAIL: CARGO_TARGET_DIR must never use the checkout target directory for $label" >&2
    exit 1
  fi
}

assert_output_contains() {
  local label="$1"
  local expected="$2"
  if ! grep -Fq "$expected" "$tmp/${label}.out"; then
    echo "FAIL: expected $label output to contain: $expected" >&2
    cat "$tmp/${label}.out" >&2
    exit 1
  fi
}

# A staged semantic Rust path must carry the entire matching workflow evidence
# set, and the runner must execute only that record's safe task-owned command.
repo="$(make_repo TASK-9001)"
stage_matching_semantic_evidence "$repo" TASK-9001
assert_success staged_matching_record "$repo"
assert_cargo_commands staged_matching_record \
  'cargo test -p ash-engine --test task_9001_fixture'
assert_cargo_target_dir_is_snapshot_local staged_matching_record "$repo"

# Each staged evidence path is independently mandatory; validation and evidence
# selection must reject before a task command could run.
for missing in task_file manifest coverage_map traceability changelog; do
  repo="$(make_repo TASK-9001)"
  stage_matching_semantic_evidence "$repo" TASK-9001 "$missing"
  assert_failure_without_cargo "missing_${missing}" "$repo" --staged
done

# Every production Rust crate is semantic implementation scope.  A new CLI
# source change cannot bypass the workflow merely because the classifier has
# an older, narrower crate allowlist.
repo="$(make_repo TASK-9001)"
mkdir -p "$repo/crates/ash-cli/src"
printf 'pub fn cli_semantic_fixture() {}\n' >"$repo/crates/ash-cli/src/semantic_task_fixture.rs"
clean_git_env git -C "$repo" add crates/ash-cli/src/semantic_task_fixture.rs
assert_failure_without_cargo semantic_ash_cli_without_evidence "$repo"

# Ordinary documentation changes must not cause semantic integration targets to
# run merely because a semantic-record manifest exists in the repository.
repo="$(make_repo TASK-9001)"
printf '# Non-semantic design note\n' >"$repo/docs/design/NOTE.md"
clean_git_env git -C "$repo" add docs/design/NOTE.md
assert_success docs_only_nonsemantic "$repo" --staged
assert_cargo_commands docs_only_nonsemantic

# --all is the pre-push mode: every active manifest record runs exactly once.
repo="$(make_repo TASK-9001 TASK-9002)"
assert_success all_active_records "$repo" --all
assert_cargo_commands all_active_records \
  'cargo test -p ash-engine --test task_9001_fixture' \
  'cargo test -p ash-engine --test task_9002_fixture'

# Workflow-enforcement tasks are deliberately outside the active semantic-task
# manifest.  Their explicit classification must keep a staged task document
# from being misidentified as an unrecorded semantic implementation task.
repo="$(make_repo TASK-9001)"
stage_matching_semantic_evidence "$repo" TASK-9001
cat >"$repo/docs/plan/tasks/TASK-2028-workflow-enforcement.md" <<'EOF'
# TASK-2028: Semantic workflow enforcement

**Status:** In progress

**Semantic task classification:** non-semantic-workflow-enforcement
EOF
clean_git_env git -C "$repo" add docs/plan/tasks/TASK-2028-workflow-enforcement.md
assert_success staged_nonsemantic_workflow_task "$repo"
assert_cargo_commands staged_nonsemantic_workflow_task \
  'cargo test -p ash-engine --test task_9001_fixture'

# The workflow classification only exempts an unregistered task document.  A
# registered task remains semantic work and must run its owned verification.
repo="$(make_repo TASK-9001)"
stage_matching_semantic_evidence "$repo" TASK-9001
printf '\n**Semantic task classification:** non-semantic-workflow-enforcement\n' \
  >>"$repo/docs/plan/tasks/TASK-9001-fixture.md"
clean_git_env git -C "$repo" add docs/plan/tasks/TASK-9001-fixture.md
assert_success classified_registered_semantic_task "$repo"
assert_cargo_commands classified_registered_semantic_task \
  'cargo test -p ash-engine --test task_9001_fixture'

# A staged TASK document is an explicit task selection.  It must never silently
# become an unowned semantic change when no manifest record declares that task.
repo="$(make_repo TASK-9001)"
cat >"$repo/docs/plan/tasks/TASK-9002-unrecorded.md" <<'EOF'
# TASK-9002: Unrecorded semantic task

**Status:** In progress
EOF
clean_git_env git -C "$repo" add docs/plan/tasks/TASK-9002-unrecorded.md
assert_failure_without_cargo selected_unrecorded_task "$repo" --staged
assert_output_contains selected_unrecorded_task TASK-9002

# Planned semantic task documents are an activation backlog, not active semantic
# implementation. They must be allowed to land before their task record, while
# a staged Rust change still requires an active record and focused evidence.
repo="$(make_repo TASK-9001)"
cat >"$repo/docs/plan/tasks/TASK-9002-planned.md" <<'EOF'
# TASK-9002: Planned semantic task

**Status:** Planned
EOF
clean_git_env git -C "$repo" add docs/plan/tasks/TASK-9002-planned.md
assert_success staged_planned_semantic_task "$repo" --staged
assert_cargo_commands staged_planned_semantic_task

# A planned task cannot piggyback on a co-staged semantic Rust change that is
# selected through another task's record. It must be activated and recorded in
# that same semantic implementation change.
repo="$(make_repo TASK-9001)"
stage_matching_semantic_evidence "$repo" TASK-9001
cat >"$repo/docs/plan/tasks/TASK-9002-planned-with-rust.md" <<'EOF'
# TASK-9002: Planned semantic task with co-staged Rust

**Status:** Planned
EOF
clean_git_env git -C "$repo" add docs/plan/tasks/TASK-9002-planned-with-rust.md
assert_failure_without_cargo planned_task_with_semantic_rust "$repo" --staged
assert_output_contains planned_task_with_semantic_rust TASK-9002

# Only the first status metadata line controls activation. A later prose or
# checklist mention of Planned must not mask an in-progress unrecorded task.
repo="$(make_repo TASK-9001)"
cat >"$repo/docs/plan/tasks/TASK-9002-status-history.md" <<'EOF'
# TASK-9002: In-progress semantic task

**Status:** In progress

## History

**Status:** Planned
EOF
clean_git_env git -C "$repo" add docs/plan/tasks/TASK-9002-status-history.md
assert_failure_without_cargo planned_status_history_does_not_opt_out "$repo" --staged
assert_output_contains planned_status_history_does_not_opt_out TASK-9002

# The runner delegates manifest semantics to the checked-in validator and must
# fail closed before cargo when that validator rejects the active records.
repo="$(make_repo TASK-9001)"
printf '{"schema":"semantic-task-records/v1"}\n' >"$repo/docs/plan/semantic-task-records.json"
clean_git_env git -C "$repo" add docs/plan/semantic-task-records.json
assert_failure_without_cargo invalid_manifest "$repo" --all
assert_output_contains invalid_manifest invalid_schema

echo "check-semantic-task-gate-tests: OK"
