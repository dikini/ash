#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

usage() {
  cat <<'USAGE'
Usage:
  scripts/check-fuzz.sh
  scripts/check-fuzz.sh --smoke
USAGE
}

mode="default"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --smoke)
      mode="smoke"
      shift
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *)
      echo "check-fuzz: unknown argument '$1'" >&2
      usage
      exit 2
      ;;
  esac
done

# Ash uses a dedicated ash-fuzz crate for fuzz testing
if [[ ! -d crates/ash-fuzz ]]; then
  echo "fuzz: no ash-fuzz crate detected, skipping"
  exit 0
fi

if ! cargo fuzz --help >/dev/null 2>&1; then
  echo "check-fuzz: ash-fuzz crate detected but cargo-fuzz is not installed" >&2
  echo "check-fuzz: install with: cargo install cargo-fuzz" >&2
  exit 1
fi

pushd "$ROOT/crates/ash-fuzz" >/dev/null
cleanup_cargo_fuzz_dir() {
  popd >/dev/null
}
trap cleanup_cargo_fuzz_dir RETURN

run_with_cargo_fuzz=true
if ! targets_output="$(cargo fuzz list 2>&1)"; then
  # `crates/ash-fuzz` is a standalone fuzz crate, not the default
  # `fuzz/Cargo.toml` layout that `cargo fuzz list` expects. Fall back to the
  # explicit bin targets declared in the crate manifest so the pre-commit smoke
  # gate still exercises a real libFuzzer target instead of failing before it
  # can discover one.
  if grep -q '^\[\[bin\]\]' Cargo.toml; then
    run_with_cargo_fuzz=false
    targets_output="$(awk '
      /^\[\[bin\]\]/ { in_bin = 1; next }
      in_bin && /^name = / {
        gsub(/"/, "", $3);
        print $3;
        exit;
      }
    ' Cargo.toml)"
  else
    echo "check-fuzz: cargo-fuzz harness detected but no runnable target list could be produced" >&2
    echo "$targets_output" >&2
    exit 1
  fi
fi

target="$(printf '%s\n' "$targets_output" | sed '/^$/d' | head -n 1 | awk '{print $1}')"
if [[ -z "$target" ]]; then
  echo "check-fuzz: cargo-fuzz crate detected but no runnable fuzz target was found" >&2
  exit 1
fi

echo "fuzz: backend cargo-fuzz (ash-fuzz crate)"
echo "fuzz: selected target $target"
if [[ "$mode" == "smoke" ]]; then
  if [[ "$run_with_cargo_fuzz" == true ]]; then
    echo "fuzz: running cargo fuzz run $target -- -max_total_time=1"
    cargo fuzz run "$target" -- -max_total_time=1
  else
    echo "fuzz: running cargo run --target-dir $ROOT/target/ash-fuzz-smoke --bin $target -- -runs=1"
    cargo run --target-dir "$ROOT/target/ash-fuzz-smoke" --bin "$target" -- -runs=1
  fi
else
  if [[ "$run_with_cargo_fuzz" == true ]]; then
    echo "fuzz: running cargo fuzz run $target -max_total_time=60"
    cargo fuzz run "$target" -- -max_total_time=60
  else
    echo "fuzz: running cargo run --target-dir $ROOT/target/ash-fuzz-smoke --bin $target -- -max_total_time=60"
    cargo run --target-dir "$ROOT/target/ash-fuzz-smoke" --bin "$target" -- -max_total_time=60
  fi
fi
