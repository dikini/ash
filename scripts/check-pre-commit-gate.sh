#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"
source "$ROOT/scripts/gate-helpers.sh"

write_marker=true
git_dir="$(git rev-parse --git-dir)"
marker_file="$git_dir/.pre-commit-gate.ok"

usage() {
  cat <<'USAGE'
Usage:
  scripts/check-pre-commit-gate.sh
  scripts/check-pre-commit-gate.sh --no-marker
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
      echo "pre-commit-gate: unknown argument '$1'" >&2
      usage
      exit 2
      ;;
  esac
done

# Format check must pass before anything else
bash scripts/check-rust-format.sh

classifier_output="$(bash scripts/check-gate-classifier.sh)"
printf '%s\n' "gate-classifier:" "$classifier_output"
docs_only="$(gate_classifier_value "$classifier_output" docs_only)"
gate_relevant="$(gate_classifier_value "$classifier_output" gate_relevant)"
unknown_relevant="$(gate_classifier_value "$classifier_output" unknown_relevant)"

# Changelog check - ensure staged changes have corresponding changelog entry
bash scripts/check-changelog-staged.sh

# Regression tests for gate policy scripts.
bash scripts/check-changelog-staged-tests.sh
bash scripts/check-gate-classifier-tests.sh
bash scripts/check-gate-marker-tests.sh
bash scripts/check-pre-push-semantic-context-tests.sh
bash scripts/check-semantic-task-gate-tests.sh
python3 -m unittest tools.docs.tests.test_validate_direct_ast_reentry

# During the Phase-205 cutover, reject residual Rust paths listed for deletion
# and inspect staged additions for re-entry. The report remains visible in hook
# output so historical material cannot look like an approved execution route.
python3 tools/docs/validate_direct_ast_reentry.py \
  --root "$ROOT" \
  --manifest docs/plan/audits/AUDIT-204-direct-ast-retirement.json \
  --staged \
  --format json

# Semantic implementation changes require task-owned focused integration
# evidence even when the generic fast gate would only run library tests.
bash scripts/check-semantic-task-gate.sh --staged

if [[ "$docs_only" == true && "$gate_relevant" == false && "$unknown_relevant" == false ]]; then
  echo "pre-commit-gate: docs-only change set; running docs gate and skipping Rust/fuzz/doctest gate"
  bash scripts/check-docs-gate.sh
  if [[ "$write_marker" == true ]]; then
    GATE_MARKER_DOCS_ONLY="$docs_only" GATE_MARKER_TREE="$(gate_index_tree_ref)" gate_write_marker "$marker_file"
    echo "pre-commit-gate: marker updated at $marker_file"
  fi
  echo "pre-commit-gate: OK"
  exit 0
fi

# Fast checks
echo "pre-commit-gate: running cargo check"
cargo check --workspace

# Clippy with warnings as errors
bash scripts/check-rust-clippy.sh

# Run tests (fast mode for pre-commit)
bash scripts/check-rust-tests.sh --workspace --lib

# Note: property tests are included in cargo test above
bash scripts/check-property-tests.sh

# Smoke test fuzz targets if ash-fuzz crate exists
bash scripts/check-fuzz.sh --smoke

# Doc tests
echo "pre-commit-gate: running documentation tests"
bash scripts/check-doc-tests.sh

if [[ "$write_marker" == true ]]; then
  GATE_MARKER_DOCS_ONLY="$docs_only" GATE_MARKER_TREE="$(gate_index_tree_ref)" gate_write_marker "$marker_file"
  echo "pre-commit-gate: marker updated at $marker_file"
fi

echo "pre-commit-gate: OK"
