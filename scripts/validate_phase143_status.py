#!/usr/bin/env python3
"""Validate Phase 142/143 status surfaces after MCP remediation."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FILES = [
    ROOT / "docs/plan/PLAN-142-MCP-CROSS-LANGUAGE-INTEGRATION.md",
    ROOT / "docs/plan/PLAN-143-MCP-CROSS-LANGUAGE-COMPLETION-REMEDIATION.md",
]
FILES.extend((ROOT / "docs/plan/tasks").glob("TASK-142*.md"))
FILES.extend((ROOT / "docs/plan/tasks").glob("TASK-143*.md"))

STALE_MARKERS = ["📋 Planned", "📝 Planned", "Planning (Not Started)"]

bad = []
for path in FILES:
    text = path.read_text()
    if any(marker in text for marker in STALE_MARKERS):
        bad.append(str(path.relative_to(ROOT)))

if bad:
    raise SystemExit("stale phase status markers found: " + ", ".join(bad))

print("Phase 142/143 status surfaces are reconciled")
