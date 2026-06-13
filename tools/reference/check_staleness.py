#!/usr/bin/env python3
"""Reference staleness checker for the Ash corpus.

Scans reference/ pages for YAML frontmatter, runs git diff against
verified_against.git_commit, and flags pages that need inspection.

Usage:
    python3 tools/reference/check_staleness.py
    python3 tools/reference/check_staleness.py --slice reference-slice-2
    python3 tools/reference/check_staleness.py --json
    python3 tools/reference/check_staleness.py --slice reference-slice-3 --json
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

# Slice definitions: map slice name to path patterns
SLICE_PATTERNS: dict[str, list[str]] = {
    "reference-slice-2": [
        "reference/getting-started/**",
        "reference/stdlib/act.md",
        "reference/stdlib/proc.md",
        "reference/stdlib/workflow.md",
        "reference/stdlib/result.md",
        "reference/tools/**",
        "reference/runtime/**",
        "reference/status/**",
        "reference/maintenance/**",
        "reference/agents/cards/stdlib-act.md",
        "reference/agents/cards/stdlib-proc.md",
        "reference/agents/cards/stdlib-workflow.md",
        "reference/agents/cards/stdlib-result.md",
    ],
    "reference-slice-3": [
        "reference/stdlib/algebra.md",
        "reference/agents/cards/stdlib-algebra.md",
        "reference/stdlib/README.md",
        "reference/INDEX.md",
    ],
}


def parse_frontmatter(text: str, path: Path) -> tuple[dict[str, Any] | None, list[str]]:
    """Parse YAML frontmatter from markdown text."""
    errors: list[str] = []
    if not text.startswith("---\n"):
        return None, [f"{path}: missing frontmatter fence"]
    end = text.find("\n---\n", 4)
    if end == -1:
        return None, [f"{path}: missing closing frontmatter fence"]
    lines = text[4:end].splitlines()
    data: dict[str, Any] = {}
    current_key: str | None = None
    current_subkey: str | None = None
    for raw in lines:
        if not raw.strip():
            continue
        if not raw.startswith(" ") and ":" in raw:
            key, value = raw.split(":", 1)
            key = key.strip()
            value = value.strip()
            current_key = key
            current_subkey = None
            if value:
                # Parse scalar
                if value == "[]":
                    data[key] = []
                elif value == "null":
                    data[key] = None
                elif value.startswith("[") and value.endswith("]"):
                    body = value[1:-1].strip()
                    data[key] = [part.strip().strip('"\'') for part in body.split(",")] if body else []
                else:
                    data[key] = value.strip('"\'')
            else:
                data[key] = {}
            continue
        if raw.startswith("  ") and not raw.startswith("    ") and current_key:
            stripped = raw.strip()
            if stripped.startswith("- "):
                if data.get(current_key) == {}:
                    data[current_key] = []
                data.setdefault(current_key, [])
                if isinstance(data[current_key], list):
                    data[current_key].append(stripped[2:].strip().strip('"\''))
                continue
            if ":" in stripped:
                subkey, value = stripped.split(":", 1)
                subkey = subkey.strip()
                value = value.strip()
                if not isinstance(data.get(current_key), dict):
                    data[current_key] = {}
                current_subkey = subkey
                data[current_key][subkey] = value.strip().strip('"\'') if value else []
                continue
        if raw.startswith("    - ") and current_key and current_subkey:
            container = data.setdefault(current_key, {}).setdefault(current_subkey, [])
            if isinstance(container, list):
                container.append(raw.strip()[2:].strip().strip('"\''))
            continue
    return data, errors


def get_git_changed_files(root: Path, since_commit: str) -> set[str]:
    """Get set of files changed since a given commit."""
    try:
        result = subprocess.run(
            ["git", "diff", "--name-only", f"{since_commit}..HEAD"],
            cwd=root,
            capture_output=True,
            text=True,
            check=True,
        )
        return set(result.stdout.strip().split("\n")) if result.stdout.strip() else set()
    except subprocess.CalledProcessError as e:
        print(f"ERROR: git diff failed: {e.stderr}", file=sys.stderr)
        return set()


def matches_refresh_trigger(changed_files: set[str], triggers: list[str]) -> list[str]:
    """Check which refresh triggers match changed files."""
    matched = []
    for trigger in triggers:
        trigger = trigger.strip()
        if not trigger:
            continue
        # Handle glob patterns
        if "*" in trigger:
            # Convert glob to path check
            import fnmatch
            for f in changed_files:
                if fnmatch.fnmatch(f, trigger) or fnmatch.fnmatch(f, trigger.lstrip("/")):
                    matched.append(trigger)
                    break
        else:
            # Exact path match
            if trigger.lstrip("/") in changed_files or trigger in changed_files:
                matched.append(trigger)
    return matched


def check_page_staleness(root: Path, path: Path, changed_files: set[str]) -> dict[str, Any]:
    """Check staleness of a single reference page."""
    rel = path.relative_to(root)
    text = path.read_text()
    data, parse_errors = parse_frontmatter(text, path)
    
    result: dict[str, Any] = {
        "path": str(rel),
        "status": "unknown",
        "reason": "",
    }
    
    if data is None:
        result["status"] = "needs-frontmatter"
        result["reason"] = "missing or invalid frontmatter"
        return result
    
    verified = data.get("verified_against", {})
    if not isinstance(verified, dict):
        result["status"] = "needs-inspection"
        result["reason"] = "verified_against is not a mapping"
        return result
    
    git_commit = verified.get("git_commit")
    if not git_commit:
        result["status"] = "needs-inspection"
        result["reason"] = "missing verified_against.git_commit"
        return result
    
    # Check if commit exists in repo
    try:
        subprocess.run(
            ["git", "cat-file", "-t", git_commit],
            cwd=root,
            capture_output=True,
            check=True,
        )
    except subprocess.CalledProcessError:
        result["status"] = "needs-inspection"
        result["reason"] = f"commit {git_commit} not found in repo"
        return result
    
    triggers = data.get("refresh_trigger", [])
    if not isinstance(triggers, list):
        triggers = [str(triggers)]
    
    matched = matches_refresh_trigger(changed_files, triggers)
    
    if matched:
        result["status"] = "stale"
        result["reason"] = f"refresh_trigger matched: {', '.join(matched)}"
        result["last_verified_commit"] = git_commit
    else:
        result["status"] = "fresh"
        result["reason"] = "no refresh_trigger changes since last verification"
        result["last_verified_commit"] = git_commit
    
    return result


def matches_slice(path: Path, root: Path, slice_name: str) -> bool:
    """Check if a path matches a slice definition."""
    import fnmatch
    rel = str(path.relative_to(root))
    patterns = SLICE_PATTERNS.get(slice_name, [])
    for pattern in patterns:
        if fnmatch.fnmatch(rel, pattern):
            return True
    return False


def format_human(results: list[dict[str, Any]]) -> str:
    """Format results as human-readable table."""
    lines = []
    lines.append(f"{'Path':<60} {'Status':<18} {'Reason'}")
    lines.append("-" * 120)
    for r in results:
        status = r["status"]
        reason = r.get("reason", "")
        path = r["path"]
        if len(path) > 58:
            path = "..." + path[-55:]
        lines.append(f"{path:<60} {status:<18} {reason}")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description="Check reference corpus staleness")
    parser.add_argument("--slice", help="filter by slice name (e.g., reference-slice-2)")
    parser.add_argument("--json", action="store_true", help="output JSON")
    parser.add_argument("--root", default=".", help="repository root")
    args = parser.parse_args()
    
    root = Path(args.root).resolve()
    reference_dir = root / "reference"
    
    if not reference_dir.exists():
        print(f"ERROR: reference/ directory not found at {reference_dir}", file=sys.stderr)
        return 2
    
    # Get current HEAD commit for comparison
    try:
        head_result = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=root,
            capture_output=True,
            text=True,
            check=True,
        )
        head_commit = head_result.stdout.strip()
    except subprocess.CalledProcessError:
        print("ERROR: could not determine HEAD commit", file=sys.stderr)
        return 2
    
    # Find all markdown files in reference/
    all_paths = sorted(reference_dir.rglob("*.md"))
    
    # Filter by slice if requested
    if args.slice:
        paths = [p for p in all_paths if matches_slice(p, root, args.slice)]
        if not paths:
            print(f"WARNING: no pages match slice '{args.slice}'", file=sys.stderr)
    else:
        paths = all_paths
    
    # Check each page
    results: list[dict[str, Any]] = []
    stale_count = 0
    inspection_count = 0
    frontmatter_count = 0
    
    for path in paths:
        # Get the verified commit for this page to check against
        text = path.read_text()
        data, _ = parse_frontmatter(text, path)
        
        if data is None:
            results.append({
                "path": str(path.relative_to(root)),
                "status": "needs-frontmatter",
                "reason": "missing or invalid frontmatter",
            })
            frontmatter_count += 1
            continue
        
        verified = data.get("verified_against", {})
        if not isinstance(verified, dict):
            results.append({
                "path": str(path.relative_to(root)),
                "status": "needs-inspection",
                "reason": "verified_against is not a mapping",
            })
            inspection_count += 1
            continue
        
        git_commit = verified.get("git_commit")
        if not git_commit:
            results.append({
                "path": str(path.relative_to(root)),
                "status": "needs-inspection",
                "reason": "missing verified_against.git_commit",
            })
            inspection_count += 1
            continue
        
        # Get changed files since this page's verification commit
        changed_files = get_git_changed_files(root, git_commit)
        
        result = check_page_staleness(root, path, changed_files)
        results.append(result)
        
        if result["status"] == "stale":
            stale_count += 1
        elif result["status"] in ("needs-inspection", "needs-frontmatter"):
            inspection_count += 1
    
    # Output
    if args.json:
        output = {
            "head_commit": head_commit,
            "slice": args.slice,
            "checked": len(paths),
            "fresh": len([r for r in results if r["status"] == "fresh"]),
            "stale": stale_count,
            "needs_inspection": inspection_count,
            "results": results,
        }
        print(json.dumps(output, indent=2))
    else:
        print(f"Staleness check: checked={len(paths)} fresh={len([r for r in results if r['status'] == 'fresh'])} stale={stale_count} needs_inspection={inspection_count}")
        if args.slice:
            print(f"Slice: {args.slice}")
        print()
        print(format_human(results))
    
    # Exit code: 0 = all fresh, 1 = stale/needs-inspection found, 2 = error
    if stale_count > 0 or inspection_count > 0:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
