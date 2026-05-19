#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"
source "$ROOT/scripts/gate-helpers.sh"

write_marker=true
git_dir="$(git rev-parse --git-dir)"
marker_file="$git_dir/.full-gate.ok"

usage() {
  cat <<'USAGE'
Usage:
  scripts/check-full-gate.sh
  scripts/check-full-gate.sh --no-marker
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-marker)
      write_marker=false
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

classifier_output="$(bash scripts/check-gate-classifier.sh)"
printf '%s\n' "gate-classifier:" "$classifier_output"
docs_only="$(gate_classifier_value "$classifier_output" docs_only)"
gate_relevant="$(gate_classifier_value "$classifier_output" gate_relevant)"
unknown_relevant="$(gate_classifier_value "$classifier_output" unknown_relevant)"
pre_commit_marker="$git_dir/.pre-commit-gate.ok"

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
