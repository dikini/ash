#!/usr/bin/env python3
"""SPEC-071 pilot validator for the Ash reference corpus.

This is intentionally repo-local and stdlib-only. It validates the controlled
frontmatter subset used by the Phase 124 pilot, repo-relative evidence paths,
internal ref IDs, and local Markdown links. It is not a general YAML parser.
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import Any

REQUIRED_FIELDS = {
    "id",
    "title",
    "kind",
    "audience",
    "authority",
    "status",
    "stability",
    "owner",
    "last_verified",
    "verified_against",
    "related",
    "refresh_trigger",
}
ALLOWED_KIND = {
    "reference",
    "index",
    "status",
    "guide",
    "agent-card",
    "agent-pack",
    "generated",
    "methodology",
    "style-guide",
}
ALLOWED_AUTHORITY = {
    "canonical",
    "canonical-adjacent",
    "derivative",
    "generated",
    "historical-summary",
    "draft",
}
ALLOWED_STATUS = {"current", "partial", "draft", "stale", "superseded", "generated", "unknown"}
ALLOWED_STABILITY = {"alpha", "beta", "stable", "experimental", "historical", "unknown"}
REQUIRED_VERIFIED = {"git_commit", "specs", "tasks", "code", "tests", "examples"}
REQUIRED_RELATED = {"depends_on", "explains", "supersedes", "superseded_by", "historical_rationale"}
PILOT_IDS = {
    "ref.root",
    "ref.index",
    "ref.meta",
    "ref.authority",
    "ref.methodology",
    "ref.style",
    "ref.status.index",
    "ref.language.functions",
    "ref.language.act",
    "ref.language.proc",
    "ref.language.workflow",
    "ref.language.generalized_do",
    "ref.agents.index",
    "ref.agents.context_pack",
    "ref.agents.common_confusions",
    "ref.agents.card.functions",
    "ref.agents.card.act",
    "ref.agents.card.proc",
    "ref.agents.card.workflow",
    "ref.agents.card.generalized_do",
    "ref.status.feature_matrix",
    "ref.status.known_limitations",
    "ref.examples.index",
    "ref.status.drift_report",
    "ref.status.verification_evidence",
}
PILOT_REQUIRED_PATHS = {
    "reference/README.md",
    "reference/INDEX.md",
    "reference/META.md",
    "reference/authority.md",
    "reference/methodology.md",
    "reference/style-guide.md",
    "reference/status/README.md",
    "reference/language/functions.md",
    "reference/language/effects-act.md",
    "reference/language/processes-proc.md",
    "reference/language/workflows.md",
    "reference/language/generalized-do.md",
    "reference/agents/README.md",
    "reference/agents/context-pack-index.md",
    "reference/agents/common-confusions.md",
    "reference/agents/cards/functions.md",
    "reference/agents/cards/act.md",
    "reference/agents/cards/proc.md",
    "reference/agents/cards/workflow.md",
    "reference/agents/cards/generalized-do.md",
    "reference/status/feature-matrix.md",
    "reference/status/known-limitations.md",
    "reference/examples/README.md",
    "reference/status/drift-report.md",
    "reference/status/verification-evidence.md",
}
LINK_RE = re.compile(r"(?<!!)\[[^\]]+\]\(([^)\s]+)(?:\s+\"[^\"]*\")?\)")


def parse_scalar(value: str) -> Any:
    value = value.strip()
    if value == "[]":
        return []
    if value == "null":
        return None
    if value.startswith("[") and value.endswith("]"):
        body = value[1:-1].strip()
        if not body:
            return []
        return [part.strip().strip('"\'') for part in body.split(",")]
    return value.strip('"\'')


def parse_frontmatter(text: str, path: Path) -> tuple[dict[str, Any] | None, list[str]]:
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
                data[key] = parse_scalar(value)
            else:
                data[key] = {}
            continue
        if raw.startswith("  ") and not raw.startswith("    ") and current_key:
            stripped = raw.strip()
            if stripped.startswith("- "):
                if data.get(current_key) == {}:
                    data[current_key] = []
                data.setdefault(current_key, [])
                if not isinstance(data[current_key], list):
                    errors.append(f"{path}: mixed list/mapping at {current_key}")
                    continue
                data[current_key].append(parse_scalar(stripped[2:]))
                continue
            if ":" in stripped:
                subkey, value = stripped.split(":", 1)
                subkey = subkey.strip()
                value = value.strip()
                if not isinstance(data.get(current_key), dict):
                    data[current_key] = {}
                current_subkey = subkey
                data[current_key][subkey] = parse_scalar(value) if value else []
                continue
        if raw.startswith("    []") and current_key and current_subkey:
            container = data.setdefault(current_key, {}).setdefault(current_subkey, [])
            if container not in ([], None):
                errors.append(f"{path}: expected empty list at {current_key}.{current_subkey}")
            else:
                data[current_key][current_subkey] = []
            continue
        if raw.startswith("    - ") and current_key and current_subkey:
            container = data.setdefault(current_key, {}).setdefault(current_subkey, [])
            if not isinstance(container, list):
                errors.append(f"{path}: expected list at {current_key}.{current_subkey}")
                continue
            container.append(parse_scalar(raw.strip()[2:]))
            continue
        errors.append(f"{path}: unsupported frontmatter line: {raw}")
    return data, errors


def iter_markdown(root: Path, pilot: bool) -> list[Path]:
    if pilot:
        return sorted(root / rel for rel in PILOT_REQUIRED_PATHS)
    return sorted((root / "reference").rglob("*.md"))


def validate_links(root: Path, path: Path, text: str) -> list[str]:
    errors: list[str] = []
    for match in LINK_RE.finditer(text):
        target = match.group(1)
        if target.startswith(("http://", "https://", "mailto:", "#")):
            continue
        clean = target.split("#", 1)[0]
        if not clean:
            continue
        candidate = (path.parent / clean).resolve()
        try:
            candidate.relative_to(root.resolve())
        except ValueError:
            errors.append(f"{path}: link escapes repo: {target}")
            continue
        if not candidate.exists():
            errors.append(f"{path}: broken markdown link: {target}")
    return errors


def as_list(value: Any) -> list[str]:
    if value is None:
        return []
    if isinstance(value, list):
        return [str(v) for v in value]
    return [str(value)]


def body_field(text: str, name: str) -> str | None:
    """Return a simple body metadata field used by pilot agent cards."""
    match = re.search(rf"(?m)^{re.escape(name)}:\s*(\S+)\s*$", text)
    return match.group(1) if match else None


def validate_agent_card(root: Path, path: Path, text: str, known_ids: set[str]) -> list[str]:
    """Validate the body-level card link-back metadata required by SPEC-071."""
    rel = path.relative_to(root)
    errors: list[str] = []
    canonical_page = body_field(text, "canonical_page")
    canonical_page_path = body_field(text, "canonical_page_path")
    if canonical_page is None:
        errors.append(f"{rel}: agent-card missing canonical_page")
    elif canonical_page not in known_ids:
        errors.append(f"{rel}: agent-card canonical_page does not resolve: {canonical_page}")
    if canonical_page_path is None:
        errors.append(f"{rel}: agent-card missing canonical_page_path")
        return errors
    target = (path.parent / canonical_page_path).resolve()
    try:
        target.relative_to(root.resolve())
    except ValueError:
        errors.append(f"{rel}: agent-card canonical_page_path escapes repo: {canonical_page_path}")
        return errors
    if not target.exists():
        errors.append(f"{rel}: agent-card canonical_page_path not found: {canonical_page_path}")
        return errors
    target_data, parse_errors = parse_frontmatter(target.read_text(), target.relative_to(root))
    errors.extend(parse_errors)
    if canonical_page and target_data and target_data.get("id") != canonical_page:
        errors.append(
            f"{rel}: canonical_page {canonical_page} does not match "
            f"{target.relative_to(root)} id {target_data.get('id')}"
        )
    return errors


def validate_file(root: Path, path: Path, known_ids: set[str]) -> tuple[str | None, list[str]]:
    text = path.read_text()
    data, errors = parse_frontmatter(text, path.relative_to(root))
    if data is None:
        return None, errors
    rel = path.relative_to(root)
    missing = REQUIRED_FIELDS - data.keys()
    if missing:
        errors.append(f"{rel}: missing required fields: {', '.join(sorted(missing))}")
    page_id = str(data.get("id", ""))
    if not page_id.startswith("ref."):
        errors.append(f"{rel}: id must start with ref.*")
    if data.get("kind") not in ALLOWED_KIND:
        errors.append(f"{rel}: invalid kind {data.get('kind')!r}")
    if data.get("authority") not in ALLOWED_AUTHORITY:
        errors.append(f"{rel}: invalid authority {data.get('authority')!r}")
    if data.get("status") not in ALLOWED_STATUS:
        errors.append(f"{rel}: invalid status {data.get('status')!r}")
    if data.get("stability") not in ALLOWED_STABILITY:
        errors.append(f"{rel}: invalid stability {data.get('stability')!r}")
    audience = data.get("audience")
    if not isinstance(audience, list) or not set(audience).issubset({"human", "agent"}) or not audience:
        errors.append(f"{rel}: audience must be a non-empty list of human/agent")
    verified = data.get("verified_against")
    if not isinstance(verified, dict):
        errors.append(f"{rel}: verified_against must be a mapping")
    else:
        missing_verified = REQUIRED_VERIFIED - verified.keys()
        if missing_verified:
            errors.append(f"{rel}: verified_against missing {', '.join(sorted(missing_verified))}")
        for key in ("specs", "tasks", "code", "examples"):
            for entry in as_list(verified.get(key)):
                candidate = root / entry
                if not candidate.exists():
                    errors.append(f"{rel}: verified_against.{key} path not found: {entry}")
        for entry in as_list(verified.get("tests")):
            if entry.endswith(".rs") or entry.endswith(".py") or "/" in entry:
                if not (root / entry).exists():
                    errors.append(f"{rel}: verified_against.tests path not found: {entry}")
    related = data.get("related")
    if not isinstance(related, dict):
        errors.append(f"{rel}: related must be a mapping")
    else:
        missing_related = REQUIRED_RELATED - related.keys()
        if missing_related:
            errors.append(f"{rel}: related missing {', '.join(sorted(missing_related))}")
        for key in ("depends_on", "explains", "supersedes", "historical_rationale"):
            for entry in as_list(related.get(key)):
                if entry.startswith("ref.") and entry not in known_ids:
                    errors.append(f"{rel}: unresolved related {key} ref id: {entry}")
                elif not entry.startswith("ref.") and ("/" in entry) and not (root / entry).exists():
                    errors.append(f"{rel}: unresolved related {key} path: {entry}")
        superseded_by = related.get("superseded_by")
        if isinstance(superseded_by, str) and superseded_by.startswith("ref.") and superseded_by not in known_ids:
            errors.append(f"{rel}: unresolved superseded_by ref id: {superseded_by}")
    if not as_list(data.get("refresh_trigger")):
        errors.append(f"{rel}: refresh_trigger must not be empty")
    if data.get("kind") == "agent-card":
        errors.extend(validate_agent_card(root, path, text, known_ids))
    errors.extend(validate_links(root, path, text))
    return page_id, errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pilot", action="store_true", help="validate the Phase 124 pilot slice")
    parser.add_argument("--root", default=".", help="repository root")
    args = parser.parse_args()
    root = Path(args.root).resolve()
    paths = iter_markdown(root, args.pilot)
    errors: list[str] = []
    missing = [p for p in paths if not p.exists()]
    for p in missing:
        errors.append(f"missing pilot file: {p.relative_to(root)}")
    paths = [p for p in paths if p.exists()]
    if not paths:
        errors.append("validator covered zero markdown files")
    known_ids: set[str] = set()
    for path in paths:
        data, parse_errors = parse_frontmatter(path.read_text(), path.relative_to(root))
        errors.extend(parse_errors)
        if data and "id" in data:
            known_ids.add(str(data["id"]))
    if args.pilot:
        missing_ids = PILOT_IDS - known_ids
        if missing_ids:
            errors.append("missing pilot ids: " + ", ".join(sorted(missing_ids)))
    checked = 0
    for path in paths:
        _page_id, file_errors = validate_file(root, path, known_ids)
        errors.extend(file_errors)
        checked += 1
    if args.pilot and checked < 20:
        errors.append(f"pilot coverage too small: checked {checked} files")
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        print(f"reference validation failed: checked={checked} errors={len(errors)}", file=sys.stderr)
        return 1
    print(f"reference validation passed: checked={checked} pilot={args.pilot}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
