#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

changed_paths() {
  {
    git diff --name-only
    git diff --cached --name-only
    git ls-files --others --exclude-standard
  } | sed '/^$/d' | sort -u
}

docs_only=true
rust_relevant=false
fuzz_relevant=false
gate_relevant=false
unknown_relevant=false
saw_path=false

while IFS= read -r path; do
  [[ -z "$path" ]] && continue
  saw_path=true
  case "$path" in
    CHANGELOG.md|README.md|TOOLS.md|AGENTS.md|docs/*.md|docs/**/*.md|*.md)
      ;;
    .githooks/*|scripts/*.sh|scripts/**/*.sh|.github/*|.github/**/*)
      docs_only=false
      rust_relevant=true
      gate_relevant=true
      ;;
    Cargo.toml|Cargo.lock|build.rs|crates/*|crates/**/*|src/*|src/**/*|tests/*|tests/**/*|examples/*|examples/**/*|benches/*|benches/**/*)
      docs_only=false
      rust_relevant=true
      case "$path" in
        crates/ash-core/*|crates/ash-core/**/*|crates/ash-parser/*|crates/ash-parser/**/*|crates/ash-typeck/*|crates/ash-typeck/**/*|crates/ash-engine/*|crates/ash-engine/**/*|crates/ash-interp/*|crates/ash-interp/**/*|crates/ash-fuzz/*|crates/ash-fuzz/**/*)
          fuzz_relevant=true
          ;;
      esac
      ;;
    *)
      docs_only=false
      rust_relevant=true
      unknown_relevant=true
      ;;
  esac
done < <(changed_paths)

if [[ "$saw_path" == false ]]; then
  docs_only=true
fi

cat <<EOF
changed_paths_present=$saw_path
docs_only=$docs_only
rust_relevant=$rust_relevant
fuzz_relevant=$fuzz_relevant
gate_relevant=$gate_relevant
unknown_relevant=$unknown_relevant
EOF
