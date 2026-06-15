#!/usr/bin/env python3
"""Reference staleness checker for the Ash reference corpus.

The checker reads SPEC-071-style frontmatter, compares each page's
``verified_against.git_commit`` against ``HEAD``, and reports pages whose
``refresh_trigger`` paths changed after that verification commit.

Exit codes:
  0: all checked pages are fresh
  1: at least one checked page is stale or needs inspection
  2: command/setup error
"""
from __future__ import annotations

import argparse
import fnmatch
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

LEGACY_SLICE_PATTERNS: dict[str, list[str]] = {
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


def parse_scalar(value: str) -> Any:
    value = value.strip()
    if value == "[]":
        return []
    if value == "null":
        return None
    if value.startswith("[") and value.endswith("]"):
        body = value[1:-1].strip()
        return [part.strip().strip("\"'") for part in body.split(",")] if body else []
    return value.strip("\"'")


def parse_frontmatter(text: str, path: Path) -> tuple[dict[str, Any] | None, list[str]]:
    """Parse the subset of YAML frontmatter used by reference pages."""
    if not text.startswith("---\n"):
        return None, [f"{path}: missing frontmatter fence"]
    end = text.find("\n---\n", 4)
    if end == -1:
        return None, [f"{path}: missing closing frontmatter fence"]

    data: dict[str, Any] = {}
    current_key: str | None = None
    current_subkey: str | None = None

    for raw in text[4:end].splitlines():
        if not raw.strip():
            continue
        if not raw.startswith(" ") and ":" in raw:
            key, value = raw.split(":", 1)
            current_key = key.strip()
            current_subkey = None
            data[current_key] = parse_scalar(value) if value.strip() else {}
            continue
        if raw.startswith("  ") and not raw.startswith("    ") and current_key:
            stripped = raw.strip()
            if stripped.startswith("- "):
                if not isinstance(data.get(current_key), list):
                    data[current_key] = []
                data[current_key].append(stripped[2:].strip().strip("\"'"))
                continue
            if ":" in stripped:
                subkey, value = stripped.split(":", 1)
                current_subkey = subkey.strip()
                if not isinstance(data.get(current_key), dict):
                    data[current_key] = {}
                data[current_key][current_subkey] = parse_scalar(value) if value.strip() else []
                continue
        if raw.startswith("    - ") and current_key and current_subkey:
            parent = data.setdefault(current_key, {})
            if isinstance(parent, dict):
                items = parent.setdefault(current_subkey, [])
                if isinstance(items, list):
                    items.append(raw.strip()[2:].strip().strip("\"'"))
    return data, []


def git(root: Path, args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", *args],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )


def git_changed_files(root: Path, since_commit: str) -> tuple[set[str] | None, str | None]:
    result = git(root, ["diff", "--name-only", f"{since_commit}..HEAD"])
    if result.returncode != 0:
        return None, result.stderr.strip() or result.stdout.strip() or "git diff failed"
    return set(result.stdout.splitlines()), None


def commit_exists(root: Path, git_commit: str) -> bool:
    return git(root, ["cat-file", "-e", f"{git_commit}^{{commit}}"]).returncode == 0


def normalize_trigger(trigger: str) -> str:
    trigger = trigger.strip().lstrip("/")
    for suffix in (" changes", " changed", " updates", " update"):
        if trigger.endswith(suffix):
            trigger = trigger[: -len(suffix)].strip()
    return trigger


def matches_refresh_trigger(changed_files: set[str], triggers: list[str]) -> tuple[list[str], list[str]]:
    matched_triggers: list[str] = []
    matched_files: list[str] = []
    for raw_trigger in triggers:
        trigger = normalize_trigger(str(raw_trigger))
        if not trigger:
            continue
        for changed in sorted(changed_files):
            candidate = changed.lstrip("/")
            if fnmatch.fnmatch(candidate, trigger) or candidate == trigger:
                matched_triggers.append(str(raw_trigger))
                matched_files.append(candidate)
                break
    return matched_triggers, matched_files


def page_result(root: Path, path: Path, data: dict[str, Any] | None) -> dict[str, Any]:
    rel = str(path.relative_to(root))
    result: dict[str, Any] = {
        "path": rel,
        "id": data.get("id") if isinstance(data, dict) else None,
        "slice": data.get("slice") if isinstance(data, dict) else None,
        "status": "unknown",
        "reason": "",
        "last_verified_commit": None,
        "matched_triggers": [],
        "changed_files": [],
    }
    if data is None:
        result["status"] = "needs-frontmatter"
        result["reason"] = "missing or invalid frontmatter"
        return result

    verified = data.get("verified_against")
    if not isinstance(verified, dict):
        result["status"] = "needs-inspection"
        result["reason"] = "verified_against is not a mapping"
        return result
    git_commit = verified.get("git_commit")
    if not git_commit:
        result["status"] = "needs-inspection"
        result["reason"] = "missing verified_against.git_commit"
        return result
    result["last_verified_commit"] = git_commit
    if not commit_exists(root, str(git_commit)):
        result["status"] = "needs-inspection"
        result["reason"] = f"commit {git_commit} not found in repo"
        return result

    triggers = data.get("refresh_trigger", [])
    if isinstance(triggers, str):
        triggers = [triggers]
    if not isinstance(triggers, list):
        result["status"] = "needs-inspection"
        result["reason"] = "refresh_trigger is not a list or string"
        return result

    changed_files, error = git_changed_files(root, str(git_commit))
    if changed_files is None:
        result["status"] = "needs-inspection"
        result["reason"] = f"git diff inconclusive: {error}"
        return result
    matched_triggers, matched_files = matches_refresh_trigger(changed_files, [str(t) for t in triggers])
    result["matched_triggers"] = matched_triggers
    result["changed_files"] = matched_files
    if matched_triggers:
        result["status"] = "stale"
        result["reason"] = "refresh_trigger matched changed files"
    else:
        result["status"] = "fresh"
        result["reason"] = "no refresh_trigger changes since last verification"
    return result


def load_pages(root: Path) -> list[tuple[Path, dict[str, Any] | None]]:
    pages = []
    for path in sorted((root / "reference").rglob("*.md")):
        data, _ = parse_frontmatter(path.read_text(), path)
        pages.append((path, data))
    return pages


def page_in_slice(root: Path, path: Path, data: dict[str, Any] | None, slice_name: str) -> bool:
    if isinstance(data, dict) and data.get("slice") == slice_name:
        return True
    rel = str(path.relative_to(root))
    return any(fnmatch.fnmatch(rel, pattern) for pattern in LEGACY_SLICE_PATTERNS.get(slice_name, []))


def format_human(results: list[dict[str, Any]]) -> str:
    lines = [f"{'Path':<60} {'Status':<18} {'Reason'}", "-" * 120]
    for row in results:
        path = row["path"]
        if len(path) > 58:
            path = "..." + path[-55:]
        lines.append(f"{path:<60} {row['status']:<18} {row.get('reason', '')}")
        if row.get("changed_files"):
            lines.append(f"{'':<60} {'':<18} changed: {', '.join(row['changed_files'])}")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description="Check reference corpus staleness")
    parser.add_argument("--slice", help="filter by frontmatter slice name, e.g. reference-slice-3")
    parser.add_argument("--all", action="store_true", help="check all reference pages (default)")
    parser.add_argument("--json", action="store_true", help="output JSON")
    parser.add_argument("--root", default=".", help="repository root")
    args = parser.parse_args()

    if args.all and args.slice:
        print("ERROR: use either --all or --slice, not both", file=sys.stderr)
        return 2

    root = Path(args.root).resolve()
    reference_dir = root / "reference"
    if not reference_dir.exists():
        print(f"ERROR: reference/ directory not found at {reference_dir}", file=sys.stderr)
        return 2

    head = git(root, ["rev-parse", "HEAD"])
    if head.returncode != 0:
        print("ERROR: could not determine HEAD commit", file=sys.stderr)
        return 2

    pages = load_pages(root)
    if args.slice:
        pages = [(path, data) for path, data in pages if page_in_slice(root, path, data, args.slice)]
        if not pages:
            print(f"WARNING: no pages match slice '{args.slice}'", file=sys.stderr)

    results = [page_result(root, path, data) for path, data in pages]
    fresh = sum(1 for row in results if row["status"] == "fresh")
    stale = sum(1 for row in results if row["status"] == "stale")
    needs_frontmatter = sum(1 for row in results if row["status"] == "needs-frontmatter")
    needs_inspection = sum(1 for row in results if row["status"] == "needs-inspection")

    summary = {
        "head_commit": head.stdout.strip(),
        "slice": args.slice,
        "checked": len(results),
        "fresh": fresh,
        "stale": stale,
        "needs_frontmatter": needs_frontmatter,
        "needs_inspection": needs_inspection,
        "results": results,
    }
    if args.json:
        print(json.dumps(summary, indent=2))
    else:
        scope = args.slice if args.slice else "all"
        print(
            f"Staleness check: scope={scope} checked={len(results)} fresh={fresh} "
            f"stale={stale} needs_frontmatter={needs_frontmatter} needs_inspection={needs_inspection}"
        )
        print()
        print(format_human(results))

    return 1 if stale or needs_frontmatter or needs_inspection else 0


if __name__ == "__main__":
    raise SystemExit(main())
