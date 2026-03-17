#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

usage() {
  cat <<'USAGE'
Usage:
  scripts/check-doc-tests.sh
  scripts/check-doc-tests.sh --with-specs
USAGE
}

with_specs=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --with-specs)
      with_specs=true
      shift
      ;;
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

if [[ "$with_specs" == true ]]; then
  echo "doc-tests: checking SPEC document code examples"
  
  # Build and run ash-doc-tests
  cargo run --bin ash-doc-tests -- docs/spec/*.md
fi

echo "doc-tests: OK"
