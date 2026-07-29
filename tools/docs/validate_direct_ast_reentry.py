#!/usr/bin/env python3
"""Validate staged changes against TASK-2034's direct-AST retirement audit."""
from __future__ import annotations

import argparse
from contextlib import contextmanager
import io
import json
import os
from pathlib import Path
from pathlib import PurePosixPath
import re
import subprocess
import sys
import tarfile
import tempfile
from typing import Any, Iterator

try:
    from .validate_direct_ast_retirement import validate as validate_manifest
except ImportError:  # Direct execution places this file's directory on sys.path.
    from validate_direct_ast_retirement import validate as validate_manifest


REPORT_SCHEMA = "direct-ast-reentry-validation-report/v1"
FROZEN_MANIFEST_PATH = "docs/plan/audits/AUDIT-204-direct-ast-retirement.json"
REVISION = re.compile(r"^[0-9a-f]{40}$")

SCANNED_ROOTS = (
    ".github/workflows/",
    "tests/differential/",
    "crates/",
    "docs/",
    "scripts/",
    "lean_reference/",
)
DIRECT_AST_EVALUATOR = re.compile(r"\beval_expr(?:_async)?\b")
PUBLIC_FUNCTION = re.compile(
    r"\bpub(?:\s*\([^)]*\))?\s+(?:async\s+)?fn\s+([A-Za-z0-9_]+)\s*\("
)
TERMINAL_CPS_APIS = frozenset(
    {
        "execute_checked_cps",
        "execute_cps",
        "eval_checked",
        "eval_unchecked",
        "eval_checked_cps",
        "eval_cps",
        "execute_checked_terminal",
        "execute_terminal",
        "eval_checked_terminal",
        "eval_terminal",
    }
)
TERMINAL_CPS_API = re.compile(
    r"\b(?:" + "|".join(sorted(TERMINAL_CPS_APIS)) + r")\b"
)
PUBLIC_REEXPORT = re.compile(r"\bpub\s+use\b")
DIFFERENTIAL_ORACLE = re.compile(
    r"\b(?:differential[ _-]?(?:oracle|harness)|DifferentialHarness)\b",
    re.IGNORECASE,
)
RETIRED_DIFFERENTIAL_MODULE = re.compile(r"\bmod\s+differential\s*;")
LEAN_CURRENT_AUTHORITY = re.compile(
    r"(?:\blean\b[^\n]*\b(?:is|remains|acts\s+as|serves\s+as|(?<!not\s)provides)\s+"
    r"(?:the\s+)?current\s+ash\s+(?:"
    r"(?:execution|runtime|conformance)\s+(?:authority|route)|"
    r"proof\s+(?:evidence|authority)|runtime\s+refinement\s+proof|"
    r"differential\s+oracle)\b|"
    r"\bcurrent\s+ash\s+(?:(?:execution|runtime)\s+route|conformance\s+authority|"
    r"proof\s+(?:evidence|authority)|runtime\s+refinement\s+proof|differential\s+oracle)\s+"
    r"is\s+(?:the\s+)?lean\s+reference\s+interpreter\b)",
    re.IGNORECASE,
)
HUNK_HEADER = re.compile(r"^@@ -[^ ]+ \+(\d+)(?:,\d+)? @@")


def report_error(kind: str, message: str, **details: object) -> dict[str, object]:
    """Create one machine-readable failure for the report's manifest_errors field."""
    return {"kind": kind, "message": message, **details}


def manifest_entries(manifest_path: Path) -> list[dict[str, object]]:
    """Load the manifest entries after the TASK-2034 validator has accepted it."""
    payload: Any = json.loads(manifest_path.read_text(encoding="utf-8"))
    entries = payload["entries"]
    assert isinstance(entries, list)
    return [entry for entry in entries if isinstance(entry, dict)]


def lexical_manifest_path(root: Path, manifest_path: Path) -> str:
    """Return the canonical manifest path without resolving or following input paths."""
    root_path = Path(os.path.abspath(str(root)))
    candidate = manifest_path
    if candidate.is_absolute():
        try:
            candidate = candidate.relative_to(root_path)
        except ValueError as error:
            raise RuntimeError("manifest must be below the repository root") from error
    parts = PurePosixPath(str(candidate)).parts
    if not parts or any(part in {"", ".", ".."} for part in parts):
        raise RuntimeError("manifest must be a lexical repository-relative path")
    relative = "/".join(parts)
    if relative != FROZEN_MANIFEST_PATH:
        raise RuntimeError("manifest must name the TASK-2034 audit")
    return relative


def staged_manifest_modified(root: Path, manifest_path: str) -> bool:
    """Return whether the index attempts to alter the immutable audit boundary."""
    result = subprocess.run(
        [
            "git",
            "-C",
            str(root),
            "diff",
            "--cached",
            "--quiet",
            "--no-ext-diff",
            "--no-renames",
            "--no-textconv",
            "--",
            manifest_path,
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode == 1:
        return True
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip())
    return False


def frozen_manifest_bytes(root: Path, manifest_path: str) -> bytes:
    """Read one regular manifest blob from HEAD without consulting index or worktree."""
    tree = subprocess.run(
        ["git", "-C", str(root), "ls-tree", "-z", "HEAD", "--", manifest_path],
        check=False,
        capture_output=True,
    )
    if tree.returncode != 0:
        raise RuntimeError(tree.stderr.decode("utf-8", errors="replace").strip())
    records = [record for record in tree.stdout.split(b"\0") if record]
    if len(records) != 1:
        raise RuntimeError("the frozen manifest must be one tracked regular file")
    metadata, separator, raw_path = records[0].partition(b"\t")
    fields = metadata.split()
    if (
        separator != b"\t"
        or len(fields) != 3
        or fields[0] not in {b"100644", b"100755"}
        or fields[1] != b"blob"
        or raw_path.decode("utf-8", errors="surrogateescape") != manifest_path
    ):
        raise RuntimeError("the frozen manifest must be one tracked regular file")
    blob = subprocess.run(
        ["git", "-C", str(root), "show", f"HEAD:{manifest_path}"],
        check=False,
        capture_output=True,
    )
    if blob.returncode != 0:
        raise RuntimeError(blob.stderr.decode("utf-8", errors="replace").strip())
    return blob.stdout


def frozen_manifest_revision(manifest: bytes) -> str:
    """Read the revision named by the immutable manifest before materializing it."""
    try:
        payload: Any = json.loads(manifest.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise RuntimeError("the frozen manifest must be readable JSON") from error
    if not isinstance(payload, dict) or not isinstance(payload.get("repository_revision"), str):
        raise RuntimeError("the frozen manifest must name a repository revision")
    revision = payload["repository_revision"]
    if REVISION.fullmatch(revision) is None:
        raise RuntimeError("the frozen manifest repository revision is invalid")
    return revision


def archive_relative_path(name: str) -> Path:
    """Return a safe archive member path without trusting archive extraction helpers."""
    parts = PurePosixPath(name).parts
    if not parts or any(part in {"", ".", ".."} for part in parts):
        raise RuntimeError("revision archive contains an unsafe path")
    path = PurePosixPath(name)
    if path.is_absolute():
        raise RuntimeError("revision archive contains an unsafe path")
    return Path(*parts)


def materialize_revision(root: Path, revision: str, snapshot: Path) -> None:
    """Copy only regular files and directories from a revision into a disposable root."""
    archive = subprocess.run(
        ["git", "-C", str(root), "archive", "--format=tar", revision],
        check=False,
        capture_output=True,
    )
    if archive.returncode != 0:
        raise RuntimeError(archive.stderr.decode("utf-8", errors="replace").strip())
    try:
        source = tarfile.open(fileobj=io.BytesIO(archive.stdout), mode="r:")
    except tarfile.TarError as error:
        raise RuntimeError("repository revision archive is unreadable") from error
    with source:
        for member in source:
            target = snapshot / archive_relative_path(member.name)
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True)
                continue
            if not member.isfile() or target.exists():
                raise RuntimeError("revision archive contains an unsafe member")
            contents = source.extractfile(member)
            if contents is None:
                raise RuntimeError("revision archive member is unreadable")
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(contents.read())
            target.chmod(member.mode & 0o777)


@contextmanager
def frozen_validation_snapshot(
    root: Path,
    manifest_path: str,
    manifest: bytes,
) -> Iterator[tuple[Path, Path]]:
    """Materialize the manifest's frozen revision and insert its immutable HEAD blob."""
    revision = frozen_manifest_revision(manifest)
    with tempfile.TemporaryDirectory(prefix=".task-2036-index-", dir=root) as directory:
        snapshot = Path(directory)
        git_dir = subprocess.run(
            ["git", "-C", str(root), "rev-parse", "--absolute-git-dir"],
            check=False,
            capture_output=True,
            text=True,
        )
        if git_dir.returncode != 0:
            raise RuntimeError(git_dir.stderr.strip())
        materialize_revision(root, revision, snapshot)
        snapshot_manifest = snapshot / Path(*PurePosixPath(manifest_path).parts)
        snapshot_manifest.parent.mkdir(parents=True, exist_ok=True)
        snapshot_manifest.write_bytes(manifest)
        (snapshot / ".git").write_text(
            f"gitdir: {git_dir.stdout.strip()}\n",
            encoding="utf-8",
        )
        yield snapshot, snapshot_manifest


def scanned_roots(entries: list[dict[str, object]]) -> tuple[str, ...]:
    """Derive the controlled scan roots from paths declared in the audit."""
    paths = [entry.get("path") for entry in entries]
    return tuple(
        root for root in SCANNED_ROOTS if any(isinstance(path, str) and path.startswith(root) for path in paths)
    )


def is_scanned_path(path: str, roots: tuple[str, ...]) -> bool:
    """Return whether the path is below one root declared by the audit."""
    return any(path.startswith(root) for root in roots)


def staged_paths(root: Path) -> list[str]:
    """Return paths in the Git index, without consulting unstaged file contents."""
    result = subprocess.run(
        [
            "git",
            "-C",
            str(root),
            "diff",
            "--cached",
            "--name-only",
            "-z",
            "--no-ext-diff",
            "--no-renames",
            "--no-textconv",
            "--",
        ],
        check=False,
        capture_output=True,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr.decode("utf-8", errors="replace").strip())
    return sorted(
        path.decode("utf-8", errors="surrogateescape")
        for path in result.stdout.split(b"\0")
        if path
    )


def staged_added_lines(root: Path, path: str) -> list[tuple[int, str]]:
    """Return only added staged lines for one path and their new-file line numbers."""
    result = subprocess.run(
        [
            "git",
            "-C",
            str(root),
            "diff",
            "--cached",
            "--no-ext-diff",
            "--no-renames",
            "--no-textconv",
            "--no-color",
            "--unified=0",
            "--",
            path,
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip())

    added: list[tuple[int, str]] = []
    new_line: int | None = None
    for line in result.stdout.splitlines():
        header = HUNK_HEADER.match(line)
        if header:
            new_line = int(header.group(1))
            continue
        if new_line is None:
            continue
        if line.startswith("+"):
            added.append((new_line, line[1:]))
            new_line += 1
        elif line.startswith("-") or line.startswith("\\"):
            continue
        else:
            new_line += 1
    return added


def entry_by_path(entries: list[dict[str, object]]) -> dict[str, dict[str, object]]:
    """Choose a deterministic audit owner for each listed path."""
    selected: dict[str, dict[str, object]] = {}
    for entry in sorted(entries, key=lambda candidate: str(candidate.get("id", ""))):
        path = entry.get("path")
        if isinstance(path, str) and path not in selected:
            selected[path] = entry
    return selected


def listed_category(entry: dict[str, object]) -> str:
    """Classify an audited path without treating its debt as approved architecture."""
    if entry.get("classification") == "deferred_separate_project":
        return "lean_separate_project"
    role = str(entry.get("current_role", "")).lower()
    if "cps" in role:
        return "public_non_engine_cps_executor"
    if "differential" in role:
        return "differential_oracle"
    if "test runner" in role or "repl" in role or "client-local" in role:
        return "client_local_evaluation"
    if "ast" in role or "evaluator" in role or "oracle" in role:
        return "direct_ast_evaluator"
    return "listed_migration_debt"


def client_for_path(path: str) -> str | None:
    """Name a client route that must not execute locally."""
    if path.startswith("crates/ash-cli/src/test_runner/"):
        return "test_runner"
    if path.startswith("crates/ash-repl/"):
        return "repl"
    if path.startswith("crates/ash-daemon/"):
        return "daemon"
    if path.startswith("crates/ash-cli/"):
        return "run"
    return None


def is_rust_source(path: str) -> bool:
    """Limit raw evaluator and CPS API recognition to Rust source additions."""
    return path.endswith(".rs")


def prohibited_category(path: str, line: str) -> tuple[str, str | None] | None:
    """Recognize explicit prohibited re-entry markers before any debt classification."""
    if LEAN_CURRENT_AUTHORITY.search(line):
        return ("lean_authority", None)
    if not is_rust_source(path):
        return None
    if DIFFERENTIAL_ORACLE.search(line):
        return ("differential_oracle", None)
    client = client_for_path(path)
    if DIRECT_AST_EVALUATOR.search(line):
        return ("client_local_evaluation", client) if client else ("direct_ast_evaluator", None)
    if TERMINAL_CPS_API.search(line):
        if client:
            return ("client_local_evaluation", client)
        public = PUBLIC_FUNCTION.search(line)
        if not path.startswith("crates/ash-engine/") and public is not None:
            if public.group(1) in TERMINAL_CPS_APIS:
                return ("public_non_engine_cps_executor", None)
        if not path.startswith("crates/ash-engine/") and PUBLIC_REEXPORT.search(line):
            return ("public_non_engine_cps_executor", None)
    return None


def finding(
    *,
    kind: str,
    category: str,
    path: str,
    line: int,
    location: str,
    manifest_id: str | None,
    client: str | None = None,
) -> dict[str, object]:
    """Create one deterministic re-entry finding."""
    result: dict[str, object] = {
        "kind": kind,
        "category": category,
        "path": path,
        "line": line,
        "location": location,
        "manifest_id": manifest_id,
    }
    if client is not None:
        result["client"] = client
    return result


def indexed_contents(root: Path, path: str) -> str | None:
    """Read one path from the staged tree, returning None when it is absent."""
    result = subprocess.run(
        ["git", "-C", str(root), "show", f":{path}"],
        check=False,
        capture_output=True,
    )
    if result.returncode == 128:
        return None
    if result.returncode != 0:
        raise RuntimeError(result.stderr.decode("utf-8", errors="replace").strip())
    return result.stdout.decode("utf-8", errors="replace")


def current_rust_delete_findings(
    root: Path, entries: list[dict[str, object]]
) -> list[dict[str, object]]:
    """Reject residual Rust paths that the frozen audit retired by deletion."""
    findings: list[dict[str, object]] = []
    for entry in entries:
        path = entry.get("path")
        if (
            entry.get("classification") != "current"
            or entry.get("disposition") != "delete"
            or not isinstance(path, str)
            or not path.endswith(".rs")
        ):
            continue
        contents = indexed_contents(root, path)
        if contents is None:
            continue
        if path == "crates/ash-engine/src/lib.rs":
            match = RETIRED_DIFFERENTIAL_MODULE.search(contents)
            if match is None:
                continue
            line = contents.count("\n", 0, match.start()) + 1
        else:
            line = 1
        findings.append(
            finding(
                kind="current_listed_rust_use",
                category=listed_category(entry),
                path=path,
                line=line,
                location="manifest-listed",
                manifest_id=str(entry["id"]),
            )
        )
    return findings


def validate_staged(root: Path, manifest_path: Path) -> dict[str, object]:
    """Validate the manifest, then inspect only staged additions in declared roots."""
    try:
        frozen_path = lexical_manifest_path(root, manifest_path)
        if staged_manifest_modified(root, frozen_path):
            return {
                "schema": REPORT_SCHEMA,
                "findings": [],
                "manifest_errors": [
                    report_error(
                        "frozen_manifest_modified",
                        "the TASK-2034 audit manifest must remain identical to HEAD",
                        path=frozen_path,
                    )
                ],
            }
        manifest = frozen_manifest_bytes(root, frozen_path)
        with frozen_validation_snapshot(root, frozen_path, manifest) as (
            snapshot,
            snapshot_manifest,
        ):
            manifest_errors = validate_manifest(snapshot, snapshot_manifest)
            if manifest_errors:
                return {
                    "schema": REPORT_SCHEMA,
                    "findings": [],
                    "manifest_errors": manifest_errors,
                }
            entries = manifest_entries(snapshot_manifest)
        roots = scanned_roots(entries)
        audited_paths = entry_by_path(entries)
        findings = current_rust_delete_findings(root, entries)
        for path in staged_paths(root):
            if not is_scanned_path(path, roots):
                continue
            listed = audited_paths.get(path)
            for line_number, line in staged_added_lines(root, path):
                prohibited = prohibited_category(path, line)
                if prohibited is not None and prohibited[0] == "lean_authority":
                    findings.append(
                        finding(
                            kind="stale_current_ash_authority",
                            category="lean_authority",
                            path=path,
                            line=line_number,
                            location="manifest-listed" if listed else "unknown",
                            manifest_id=str(listed["id"]) if listed else None,
                        )
                    )
                elif prohibited is not None:
                    category, client = prohibited
                    findings.append(
                        finding(
                            kind="unlisted_reentry",
                            category=category,
                            path=path,
                            line=line_number,
                            location="manifest-listed" if listed else "unknown",
                            manifest_id=str(listed["id"]) if listed else None,
                            client=client,
                        )
                    )
                elif listed is not None:
                    findings.append(
                        finding(
                            kind="listed_migration_debt",
                            category=listed_category(listed),
                            path=path,
                            line=line_number,
                            location="manifest-listed",
                            manifest_id=str(listed["id"]),
                        )
                    )
    except (OSError, RuntimeError, ValueError, KeyError, json.JSONDecodeError, tarfile.TarError) as error:
        return {
            "schema": REPORT_SCHEMA,
            "findings": [],
            "manifest_errors": [
                report_error(
                    "frozen_manifest_unavailable",
                    "the frozen manifest and its repository revision must be readable",
                    detail=str(error),
                )
            ],
        }

    findings.sort(
        key=lambda item: (
            str(item["path"]),
            int(item["line"]),
            str(item["kind"]),
            str(item["category"]),
            str(item.get("manifest_id") or ""),
        )
    )
    return {"schema": REPORT_SCHEMA, "findings": findings, "manifest_errors": []}


def main() -> int:
    """Run the staged guard and emit its report as JSON only on stdout."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--staged", action="store_true", required=True)
    parser.add_argument("--format", choices=("json",), default="json")
    args = parser.parse_args()

    report = validate_staged(args.root.resolve(), args.manifest)
    print(json.dumps(report, sort_keys=True))
    manifest_errors = report["manifest_errors"]
    findings = report["findings"]
    assert isinstance(manifest_errors, list)
    assert isinstance(findings, list)
    rejected = any(
        isinstance(item, dict) and item.get("kind") != "listed_migration_debt"
        for item in findings
    )
    return 1 if manifest_errors or rejected else 0


if __name__ == "__main__":
    sys.exit(main())
