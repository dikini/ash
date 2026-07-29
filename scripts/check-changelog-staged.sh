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

# Get list of staged files (excluding CHANGELOG.md itself and docs).
# Use awk instead of grep -v pipelines: under `set -euo pipefail`, a grep
# pipeline with no remaining rows exits non-zero and can make docs-only commits
# fail before the empty-list check below runs.
staged_files=$(git diff --cached --name-only | awk '
  $0 != "CHANGELOG.md" && $0 !~ /^docs\// && $0 !~ /^\.github\// { print }
')

if [[ -z "$staged_files" ]]; then
  echo "changelog-check: no relevant staged files, skipping"
  exit 0
fi

# Check if CHANGELOG.md is staged. Query its path directly so the check does
# not close a pipeline early when CHANGELOG.md sorts before a large staged list.
if [[ -n "$(git diff --cached --name-only -- CHANGELOG.md)" ]]; then
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
