#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

property_file_patterns=(
  '*property*.rs'
  '*prop*.rs'
)

search_paths=(Cargo.toml)
for crate in crates/*; do
  if [[ -d "$crate" ]]; then
    for path in "$crate/src" "$crate/tests"; do
      if [[ -e "$path" ]]; then
        search_paths+=("$path")
      fi
    done
  fi
done

has_property_usage=false
if rg -n 'proptest|quickcheck|bolero|mod +property_tests|property_tests|proptest!' "${search_paths[@]}" -g '!target' >/dev/null 2>&1; then
  has_property_usage=true
fi

has_property_files=false
for crate in crates/*; do
  for pattern in "${property_file_patterns[@]}"; do
    if find "$crate/src" "$crate/tests" -type f -name "$pattern" -print -quit 2>/dev/null | grep -q .; then
      has_property_files=true
      break 2
    fi
  done
done

if [[ "$has_property_usage" == true || "$has_property_files" == true ]]; then
  echo "property-tests: property-oriented tests detected"
  echo "property-tests: covered by the Rust test suite already run in this gate"
else
  echo "property-tests: no property test harness detected, skipping"
fi
