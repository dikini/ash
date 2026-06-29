#!/usr/bin/env python3
"""Validate Ash notes/spec orientation indexes.

The indexes are deliberately human-authored Markdown, but they still need enough
structure for agents and docs gates to trust them. This checker verifies that:

- every Markdown file under docs/notes/ is represented in docs/notes/NOTE-INDEX.md;
- every SPEC-*.md file is represented in docs/spec/SPEC-INDEX.md;
- indexed links resolve and stay inside the expected directory;
- primary topics and tags come from the documented vocabulary;
- status/topic/tag/role cells are present for every indexed document.
"""

from __future__ import annotations

import argparse
import re
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

REPO_ROOT = Path(__file__).resolve().parents[2]
NOTES_DIR = REPO_ROOT / "docs" / "notes"
SPECS_DIR = REPO_ROOT / "docs" / "spec"
NOTE_INDEX = NOTES_DIR / "NOTE-INDEX.md"
SPEC_INDEX = SPECS_DIR / "SPEC-INDEX.md"

NOTE_TOPICS = {
    "ambient-computation",
    "contracts",
    "runtime",
    "workflow",
    "type-system",
    "tooling",
    "memory",
    "general",
}

SPEC_TOPICS = {
    "language-surface",
    "type-system",
    "effect-system",
    "core-ir",
    "runtime",
    "testing",
    "tooling",
    "contracts",
    "general",
}

TAGS = {
    "ambient-monad",
    "authority",
    "contract",
    "core-ir",
    "current-state",
    "deferred",
    "diagnostics",
    "effect-system",
    "evidence",
    "grammar",
    "implemented",
    "lowering",
    "orientation",
    "ownership",
    "references",
    "runtime",
    "semantics",
    "snapshots",
    "surface",
    "syntax",
    "target-state",
    "temporal",
    "testing",
    "tooling",
    "trace",
    "type-system",
    "workflow",
}

MIN_REQUIRED_HEADINGS = {
    "How to use this index",
    "Topic ontology",
    "Tag vocabulary",
    "Read paths",
    "Document table",
}


@dataclass(frozen=True)
class IndexRow:
    source_line: int
    document: str
    status: str
    primary_topic: str
    tags: tuple[str, ...]
    role: str
    read_with: str


def parse_index_table(path: Path) -> list[IndexRow]:
    rows: list[IndexRow] = []
    in_document_table = False
    for line_no, raw_line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        line = raw_line.strip()
        if line == "## Document table":
            in_document_table = True
            continue
        if in_document_table and line.startswith("## "):
            break
        if not in_document_table or not line.startswith("|"):
            continue
        if "---" in line or "Document" in line and "Primary topic" in line:
            continue
        cells = [cell.strip() for cell in line.strip("|").split("|")]
        if len(cells) != 6:
            raise ValueError(f"{path}:{line_no}: expected 6 table cells, got {len(cells)}")
        link_match = re.fullmatch(r"\[([^\]]+)\]\(([^)]+)\)", cells[0])
        if link_match is None:
            raise ValueError(f"{path}:{line_no}: document cell must be a markdown link")
        label, target = link_match.groups()
        if label != target:
            raise ValueError(f"{path}:{line_no}: document link label and target must match")
        tags = tuple(tag.strip() for tag in cells[3].split(",") if tag.strip())
        rows.append(
            IndexRow(
                source_line=line_no,
                document=target,
                status=cells[1],
                primary_topic=cells[2],
                tags=tags,
                role=cells[4],
                read_with=cells[5],
            )
        )
    if not rows:
        raise ValueError(f"{path}: no document table rows found")
    return rows


def markdown_headings(path: Path) -> set[str]:
    headings = set()
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("## "):
            headings.add(line[3:].strip())
    return headings


def expected_docs(directory: Path, pattern: str) -> set[str]:
    return {path.name for path in directory.glob(pattern) if path.name not in {"NOTE-INDEX.md", "SPEC-INDEX.md"}}


def validate_index(
    *,
    index_path: Path,
    directory: Path,
    pattern: str,
    allowed_topics: set[str],
    require_tag_vocab: bool,
) -> list[str]:
    errors: list[str] = []
    if not index_path.exists():
        return [f"missing index: {index_path}"]

    missing_headings = MIN_REQUIRED_HEADINGS - markdown_headings(index_path)
    for heading in sorted(missing_headings):
        errors.append(f"{index_path}: missing required heading '## {heading}'")

    try:
        rows = parse_index_table(index_path)
    except ValueError as exc:
        return [str(exc)]

    seen = {row.document for row in rows}
    expected = expected_docs(directory, pattern)
    for missing in sorted(expected - seen):
        errors.append(f"{index_path}: missing row for {missing}")
    for extra in sorted(seen - expected):
        errors.append(f"{index_path}: row for non-target file {extra}")

    duplicate_counts: dict[str, int] = {}
    for row in rows:
        duplicate_counts[row.document] = duplicate_counts.get(row.document, 0) + 1
    for document, count in sorted(duplicate_counts.items()):
        if count > 1:
            errors.append(f"{index_path}: duplicate row for {document}")

    for row in rows:
        line_label = f"{index_path}:{row.source_line}"
        target = (index_path.parent / row.document).resolve()
        try:
            target.relative_to(directory.resolve())
        except ValueError:
            errors.append(f"{line_label}: link escapes expected directory: {row.document}")
            continue
        if not target.exists():
            errors.append(f"{line_label}: link target does not exist: {row.document}")
        if not row.status or row.status == "—":
            errors.append(f"{line_label}: status must be present")
        if row.primary_topic not in allowed_topics:
            errors.append(f"{line_label}: unknown primary topic '{row.primary_topic}'")
        if not row.tags:
            errors.append(f"{line_label}: tags must be non-empty")
        if require_tag_vocab:
            for tag in row.tags:
                if tag not in TAGS:
                    errors.append(f"{line_label}: unknown tag '{tag}'")
        if not row.role or row.role == "—":
            errors.append(f"{line_label}: role must be present")
    return errors


def run_self_test() -> list[str]:
    """Exercise the table parser with a malformed temporary index."""
    with tempfile.TemporaryDirectory() as raw_dir:
        base = Path(raw_dir)
        docs_dir = base / "docs" / "notes"
        docs_dir.mkdir(parents=True)
        (docs_dir / "NOTE-001-EXAMPLE.md").write_text("# NOTE-001: Example\n", encoding="utf-8")
        index = docs_dir / "NOTE-INDEX.md"
        index.write_text(
            "# Index\n\n"
            "## How to use this index\n\n"
            "## Topic ontology\n\n"
            "## Tag vocabulary\n\n"
            "## Read paths\n\n"
            "## Document table\n\n"
            "| Document | Status | Primary topic | Tags | Role | Read with |\n"
            "|---|---|---|---|---|---|\n"
            "| [NOTE-001-EXAMPLE.md](NOTE-001-EXAMPLE.md) | active | invalid-topic | orientation | design note | — |\n",
            encoding="utf-8",
        )
        old_root = globals()["REPO_ROOT"]
        try:
            errors = validate_index(
                index_path=index,
                directory=docs_dir,
                pattern="*.md",
                allowed_topics=NOTE_TOPICS,
                require_tag_vocab=True,
            )
        finally:
            globals()["REPO_ROOT"] = old_root
        if not any("unknown primary topic" in error for error in errors):
            return ["self-test failed: invalid topic was not detected"]
    return []


def main(argv: Iterable[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true", help="run parser self-test before real validation")
    parser.add_argument(
        "--allow-extension-tags",
        action="store_true",
        help="allow tags outside the controlled vocabulary",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    errors: list[str] = []
    if args.self_test:
        errors.extend(run_self_test())
    errors.extend(
        validate_index(
            index_path=NOTE_INDEX,
            directory=NOTES_DIR,
            pattern="*.md",
            allowed_topics=NOTE_TOPICS,
            require_tag_vocab=not args.allow_extension_tags,
        )
    )
    errors.extend(
        validate_index(
            index_path=SPEC_INDEX,
            directory=SPECS_DIR,
            pattern="SPEC-*.md",
            allowed_topics=SPEC_TOPICS,
            require_tag_vocab=not args.allow_extension_tags,
        )
    )

    if errors:
        print("orientation-index-check: FAILED", file=sys.stderr)
        for error in errors:
            print(f"  {error}", file=sys.stderr)
        return 1
    print("orientation-index-check: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
