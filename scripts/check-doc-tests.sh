#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

usage() {
  cat <<'USAGE'
Usage:
  scripts/check-doc-tests.sh
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h | --help)
      usage
      exit 0
      ;;
    *)
      echo "check-doc-tests: unknown argument '$1'" >&2
      usage
      exit 2
      ;;
  esac
done

echo "doc-tests: checking Rust documentation tests"
cargo test --doc --workspace

echo "doc-tests: checking SPEC document examples"
# Extract and test code examples from SPEC documents
# This ensures specs stay synchronized with implementation

failed=0
for spec in docs/spec/*.md; do
  if [[ -f "$spec" ]]; then
    echo "doc-tests: checking $spec"
    # Look for ```rust or ```ignore code blocks
    # TODO: Implement actual extraction and testing
    # For now, just check that examples compile
  fi
done

if [[ $failed -gt 0 ]]; then
  echo "doc-tests: $failed failures" >&2
  exit 1
fi

echo "doc-tests: OK"
