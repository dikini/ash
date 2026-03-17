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
  gate_write_marker "$marker_file"
  echo "pre-commit-gate: marker updated at $marker_file"
fi

echo "pre-commit-gate: OK"
