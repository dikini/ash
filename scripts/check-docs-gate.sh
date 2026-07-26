#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

source "$ROOT/scripts/gate-helpers.sh"

echo "docs-gate: running whitespace diff check"
git diff --check

echo "docs-gate: running staged changelog policy"
bash scripts/check-changelog-staged.sh

echo "docs-gate: running changelog policy regression tests"
bash scripts/check-changelog-staged-tests.sh

echo "docs-gate: checking changed markdown links"
mapfile -t md_paths < <({
  git diff --name-only
  git diff --cached --name-only
  git ls-files --others --exclude-standard
} | sed '/^$/d' | sort -u | grep -E '(^|/)CHANGELOG\.md$|\.md$' || true)

if [[ "${#md_paths[@]}" -eq 0 ]]; then
  echo "docs-gate: no changed markdown files"
else
  python3 - "${md_paths[@]}" <<'PY'
from pathlib import Path
import re
import sys
missing = []
checked = 0
for raw in sys.argv[1:]:
    p = Path(raw)
    if not p.exists() or not p.is_file():
        continue
    text = p.read_text(encoding='utf-8')
    for match in re.findall(r'\[[^\]]+\]\(([^)]+)\)', text):
        link = match.strip()
        if not link or link.startswith(('#', 'http://', 'https://', 'mailto:')):
            continue
        if link.startswith('`'):
            continue
        target = link.split('#', 1)[0]
        if not target:
            continue
        if re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:', target):
            continue
        checked += 1
        resolved = (p.parent / target).resolve()
        try:
            resolved.relative_to(Path.cwd().resolve())
        except ValueError:
            missing.append((raw, link, 'escapes repo'))
            continue
        if not resolved.exists():
            missing.append((raw, link, str(resolved)))
if missing:
    print('docs-gate: missing markdown links:', file=sys.stderr)
    for source, link, resolved in missing:
        print(f'  {source}: {link} -> {resolved}', file=sys.stderr)
    sys.exit(1)
print(f'docs-gate: markdown links checked={checked} missing=0')
PY
fi

echo "docs-gate: validating docs orientation indexes"
python3 tools/docs/validate_orientation_indexes.py

echo "docs-gate: validating Phase 202 semantic traceability"
python3 tools/docs/validate_semantic_traceability.py --root . \
  --graph docs/spec/SEMANTIC-TRACEABILITY.json --format json

echo "docs-gate: OK"
