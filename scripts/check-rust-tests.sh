#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  scripts/check-rust-tests.sh --workspace
  scripts/check-rust-tests.sh --args <cargo-test-args...>
USAGE
}

if [[ $# -eq 0 ]]; then
  usage
  exit 2
fi

if [[ "$1" == "--workspace" ]]; then
  shift
  args=(--workspace "$@")
elif [[ "$1" == "--args" ]]; then
  shift
  args=("$@")
else
  echo "check-rust-tests: unknown argument '$1'" >&2
  usage
  exit 2
fi

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

jobs="${CARGO_BUILD_JOBS:-1}"

echo "check-rust-tests: repo root $ROOT"
echo "check-rust-tests: using cargo test (serial build/test execution)"
echo "check-rust-tests: running CARGO_BUILD_JOBS=$jobs cargo test ${args[*]} -- --test-threads=1"
CARGO_BUILD_JOBS="$jobs" cargo test "${args[@]}" -- --test-threads=1
