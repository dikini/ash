#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"
source "$ROOT/scripts/gate-helpers.sh"

write_marker=true
pre_push_context=false
git_dir="$(git rev-parse --git-dir)"
marker_file="$git_dir/.full-gate.ok"

usage() {
  cat <<'USAGE'
Usage:
  scripts/check-full-gate.sh
  scripts/check-full-gate.sh --no-marker
  scripts/check-full-gate.sh --pre-push
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-marker)
      write_marker=false
      shift
      ;;
    --pre-push)
      pre_push_context=true
      shift
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *)
      echo "full-gate: unknown argument '$1'" >&2
      usage
      exit 2
      ;;
  esac
done

validate_pre_push_context() {
  local current_head
  local local_ref
  local local_sha
  local remote_ref
  local remote_sha
  local extra
  local zero_sha

  current_head="$(git rev-parse --verify HEAD)" || {
    echo "full-gate: pre-push checkout has no HEAD" >&2
    return 1
  }
  zero_sha="$(printf '%*s' "${#current_head}" '' | tr ' ' 0)"

  # Git sends one `local-ref local-sha remote-ref remote-sha` line per update.
  # A zero local SHA is a deletion; every other pushed object must be exactly
  # the checkout that this gate will validate.
  while IFS=' ' read -r local_ref local_sha remote_ref remote_sha extra || [[ -n "${local_ref:-}${local_sha:-}${remote_ref:-}${remote_sha:-}${extra:-}" ]]; do
    if [[ -z "${local_ref:-}" || -z "${local_sha:-}" || -z "${remote_ref:-}" || -z "${remote_sha:-}" || -n "${extra:-}" ]]; then
      echo "full-gate: malformed pre-push ref update input" >&2
      return 1
    fi
    if [[ "$local_sha" == "$zero_sha" ]]; then
      continue
    fi
    if [[ "$local_sha" != "$current_head" ]]; then
      echo "full-gate: pushed local object $local_sha is not current HEAD $current_head" >&2
      return 1
    fi
  done

  if ! git diff --quiet --; then
    echo "full-gate: tracked worktree changes are not valid pre-push context" >&2
    return 1
  fi
  if ! git diff --cached --quiet; then
    echo "full-gate: staged index changes are not valid pre-push context" >&2
    return 1
  fi
}

if [[ "$pre_push_context" == true ]]; then
  validate_pre_push_context
fi

classifier_output="$(bash scripts/check-gate-classifier.sh)"
printf '%s\n' "gate-classifier:" "$classifier_output"
docs_only="$(gate_classifier_value "$classifier_output" docs_only)"
gate_relevant="$(gate_classifier_value "$classifier_output" gate_relevant)"
unknown_relevant="$(gate_classifier_value "$classifier_output" unknown_relevant)"
pre_commit_marker="$git_dir/.pre-commit-gate.ok"

# Pre-push verifies the entire active semantic-task set, including focused
# integration targets that the generic workspace test command may not select.
bash scripts/check-semantic-task-gate.sh --all

if [[ "$docs_only" == true && "$gate_relevant" == false && "$unknown_relevant" == false ]]; then
  echo "full-gate: docs-only change set; running docs gate only"
  bash scripts/check-docs-gate.sh
  if [[ "$write_marker" == true ]]; then
    GATE_MARKER_DOCS_ONLY="$docs_only" gate_write_marker "$marker_file"
    echo "full-gate: marker updated at $marker_file"
  fi
  echo "full-gate: OK"
  exit 0
fi

# Run full pre-commit gate first unless the current committed tree already passed it.
if gate_marker_matches_current_head_with_empty_content "$pre_commit_marker"; then
  echo "full-gate: reusing fresh pre-commit marker"
else
  echo "full-gate: no fresh pre-commit marker; running pre-commit gate"
  bash scripts/check-pre-commit-gate.sh --no-marker
fi

# Full test suite including integration tests
bash scripts/check-rust-tests.sh --workspace --all-targets

# Full fuzz run (longer duration)
bash scripts/check-fuzz.sh

if [[ "$write_marker" == true ]]; then
  GATE_MARKER_DOCS_ONLY="$docs_only" gate_write_marker "$marker_file"
  echo "full-gate: marker updated at $marker_file"
fi

echo "full-gate: OK"
