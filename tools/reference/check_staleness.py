#!/usr/bin/env python3
"""Path-based staleness inspector for SPEC-071 reference pages.

The checker is intentionally deterministic and stdlib-only. It compares each
page's verified_against.git_commit with HEAD and reports whether changed paths
intersect the page's declared evidence or path-like refresh triggers.
"""
from __future__ import annotations

import argparse
import fnmatch
import subprocess
import sys
from pathlib import Path
from typing import Any

from check_frontmatter import as_list, parse_frontmatter

PATH_LIST_KEYS = ("specs", "tasks", "code", "tests", "examples")
DECLARED_STALE_STATES = {"stale", "partial", "superseded"}


def run_git_diff(root: Path, baseline: str) -> tuple[list[str], str | None]:
    if baseline == "unknown":
        return [], "unknown baseline"
    try:
        result = subprocess.run(
            ["git", "diff", "--name-only", f"{baseline}..HEAD"],
            cwd=root,
            check=True,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    except subprocess.CalledProcessError as exc:
        message = exc.stderr.strip() or str(exc)
        return [], message
    return [line.strip() for line in result.stdout.splitlines() if line.strip()], None


def iter_markdown(root: Path, path_arg: str) -> list[Path]:
    target = (root / path_arg).resolve()
    if target.is_file():
        return [target]
    return sorted(target.rglob("*.md"))


def is_path_like(value: str) -> bool:
    return "/" in value or value.startswith("reference") or "*" in value


def clean_trigger(value: str) -> str | None:
    text = value.strip()
    for suffix in (" changes", " changed"):
        if text.endswith(suffix):
            text = text[: -len(suffix)].strip()
            break
    if not is_path_like(text):
        return None
    if " " in text:
        return None
    return text


def evidence_patterns(data: dict[str, Any]) -> list[str]:
    patterns: list[str] = []
    verified = data.get("verified_against")
    if isinstance(verified, dict):
        for key in PATH_LIST_KEYS:
            for entry in as_list(verified.get(key)):
                if is_path_like(entry) and " " not in entry:
                    patterns.append(entry)
    for trigger in as_list(data.get("refresh_trigger")):
        cleaned = clean_trigger(trigger)
        if cleaned:
            patterns.append(cleaned)
    return sorted(set(patterns))


def pattern_matches(pattern: str, changed_path: str) -> bool:
    if pattern.endswith("/**"):
        prefix = pattern[:-3].rstrip("/")
        return changed_path == prefix or changed_path.startswith(prefix + "/")
    if any(ch in pattern for ch in "*?[]"):
        return fnmatch.fnmatch(changed_path, pattern)
    return changed_path == pattern or changed_path.startswith(pattern.rstrip("/") + "/")


def classify(data: dict[str, Any], changed_paths: list[str]) -> tuple[str, list[str]]:
    status = str(data.get("status", ""))
    if status in DECLARED_STALE_STATES:
        return status, []
    baseline = str(data.get("verified_against", {}).get("git_commit", "unknown"))
    if baseline == "unknown":
        return "needs-inspection", []
    patterns = evidence_patterns(data)
    relevant = sorted(
        {
            changed
            for changed in changed_paths
            if any(pattern_matches(pattern, changed) for pattern in patterns)
        }
    )
    if relevant:
        return "needs-inspection", relevant
    return "no-relevant-changes", []


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".", help="repository root")
    parser.add_argument("--path", default="reference", help="reference page or directory to inspect")
    parser.add_argument(
        "--fail-on-needs-inspection",
        action="store_true",
        help="return non-zero when any page needs inspection",
    )
    args = parser.parse_args()

    root = Path(args.root).resolve()
    pages = iter_markdown(root, args.path)
    if not pages:
        print(f"staleness inspection failed: no markdown files under {args.path}", file=sys.stderr)
        return 1

    diff_cache: dict[str, tuple[list[str], str | None]] = {}
    errors: list[str] = []
    counts: dict[str, int] = {}

    for page in pages:
        rel = page.relative_to(root)
        data, parse_errors = parse_frontmatter(page.read_text(), rel)
        errors.extend(parse_errors)
        if data is None:
            continue
        verified = data.get("verified_against")
        if not isinstance(verified, dict):
            errors.append(f"{rel}: verified_against must be a mapping")
            continue
        baseline = str(verified.get("git_commit", "unknown"))
        if baseline not in diff_cache:
            diff_cache[baseline] = run_git_diff(root, baseline)
        changed_paths, diff_error = diff_cache[baseline]
        if diff_error and baseline != "unknown":
            errors.append(f"{rel}: cannot diff {baseline}..HEAD: {diff_error}")
            continue
        state, relevant = classify(data, changed_paths)
        counts[state] = counts.get(state, 0) + 1
        detail = f" relevant={','.join(relevant)}" if relevant else ""
        print(f"{rel}: {state}{detail}")

    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    if args.fail_on_needs_inspection and counts.get("needs-inspection", 0):
        return 1
    summary = " ".join(f"{key}={counts[key]}" for key in sorted(counts))
    print(f"staleness inspection complete: checked={len(pages)} {summary}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
