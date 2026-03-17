#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

usage() {
  cat <<'USAGE'
Usage:
  scripts/check-changelog-staged.sh
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h | --help)
      usage
      exit 0
      ;;
    *)
      echo "check-changelog-staged: unknown argument '$1'" >&2
      usage
      exit 2
      ;;
  esac
done

# Check if CHANGELOG.md exists
if [[ ! -f CHANGELOG.md ]]; then
  echo "changelog-check: CHANGELOG.md not found, skipping"
  exit 0
fi

# Get list of staged files (excluding CHANGELOG.md itself and docs)
staged_files=$(git diff --cached --name-only | grep -v '^CHANGELOG\.md$' | grep -v '^docs/' | grep -v '^\.github/' | cat)

if [[ -z "$staged_files" ]]; then
  echo "changelog-check: no relevant staged files, skipping"
  exit 0
fi

# Check if CHANGELOG.md is staged
if git diff --cached --name-only | grep -q '^CHANGELOG\.md$'; then
  echo "changelog-check: CHANGELOG.md is staged ✓"
  exit 0
fi

echo "changelog-check: FAILED" >&2
echo "" >&2
echo "  Staged changes detected but CHANGELOG.md not updated." >&2
echo "" >&2
echo "  Please update CHANGELOG.md with your changes." >&2
echo "  Each task should include a changelog entry describing:" >&2
echo "    - What changed" >&2
echo "    - Why it changed" >&2
echo "    - Any breaking changes" >&2
echo "" >&2
echo "  To bypass this check (not recommended):" >&2
echo "    git commit --no-verify" >&2
echo "" >&2

exit 1
