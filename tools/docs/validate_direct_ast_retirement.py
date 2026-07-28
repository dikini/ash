#!/usr/bin/env python3
"""Fail-closed validator for TASK-2034's finite direct-AST retirement audit.

The audit records the bounded retirement inventory; it does not assert that a
catalogued implementation has already been migrated or removed.  In
particular, Lean records are a handoff to a separate project and never a
Phase-205 deletion item.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
from typing import Any


MANIFEST_SCHEMA = "direct-ast-retirement-audit/v1"
REPORT_SCHEMA = "direct-ast-retirement-validation-report/v1"

MANIFEST_FIELDS = {"schema", "repository_revision", "entries_sha256", "entries"}
ENTRY_FIELDS = {
    "id",
    "path",
    "locator",
    "current_role",
    "reachability",
    "classification",
    "execution_role",
    "target_rule_or_contract",
    "disposition",
    "owner_or_external_handoff",
    "consumed_handoff",
    "produced_handoff",
    "required_evidence",
    "rationale",
    "case_id",
    "missing_obligation",
    "fail_closed_result",
    "external_project",
    "external_owner",
    "external_handoff",
    "retained_paths",
    "prohibited_current_authority",
}

REACHABILITY = {"run", "daemon", "test", "repl", "differential", "none"}
CLASSIFICATION = {"current", "historical", "deferred_separate_project"}
EXECUTION_ROLE = {"executable", "test-only", "reference-only"}
DISPOSITION = {"replace", "delete", "deferred", "historical", "deferred_separate_project"}
PHASE_205_OWNER = re.compile(r"^TASK-20(?:3[7-9]|4[0-2])$")
SHA256 = re.compile(r"^sha256:[0-9a-f]{64}$")
REVISION = re.compile(r"^[0-9a-f]{40}$")
NONFINITE_PATH_CHARACTER = re.compile(r"[*?\[\]{}]")


def issue(kind: str, message: str, **details: object) -> dict[str, object]:
    """Create one stable, machine-readable validation error."""
    return {"kind": kind, "message": message, **details}


def nonempty(value: object) -> bool:
    """Return whether a required textual field is a non-blank string."""
    return isinstance(value, str) and bool(value.strip())


def entries_digest(entries: list[dict[str, object]]) -> str:
    """Hash entries canonically, independent of their presentation order."""
    ordered = sorted(entries, key=lambda entry: entry["id"])
    payload = json.dumps(ordered, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
    return f"sha256:{hashlib.sha256(payload.encode('utf-8')).hexdigest()}"


def relative_file(root: Path, value: object) -> Path | None:
    """Resolve one repository-relative regular file without traversal or symlinks."""
    path = repository_relative_path(value)
    if path is None:
        return None
    try:
        resolved_root = root.resolve(strict=True)
        resolved = (root / path).resolve(strict=True)
        resolved.relative_to(resolved_root)
    except (OSError, ValueError):
        return None
    return resolved if resolved.is_file() else None


def repository_relative_path(value: object) -> str | None:
    """Return a normalized, safe repository-relative manifest path."""
    if not nonempty(value):
        return None
    assert isinstance(value, str)
    if NONFINITE_PATH_CHARACTER.search(value):
        return None
    candidate = Path(value)
    if candidate.is_absolute() or ".." in candidate.parts:
        return None
    return candidate.as_posix()


def is_lean_path(path: object) -> bool:
    """Identify the preserved Lean reference tree in production and fixtures."""
    if not isinstance(path, str):
        return False
    parts = Path(path).parts
    return parts[:1] == ("lean_reference",) or parts[:2] == ("verification", "lean")


def repository_has_git(root: Path) -> bool:
    """Return whether a root can validate an audit revision against Git history."""
    return (root / ".git").exists()


def revision_exists(root: Path, revision: str) -> bool:
    """Return whether a revision denotes a commit in the local object database."""
    result = subprocess.run(
        ["git", "-C", str(root), "cat-file", "-e", f"{revision}^{{commit}}"],
        check=False,
        capture_output=True,
        text=True,
    )
    return result.returncode == 0


def revision_regular_files(root: Path, revision: str) -> set[str] | None:
    """Return the regular-file names at one frozen revision, or fail closed."""
    result = subprocess.run(
        ["git", "-C", str(root), "ls-tree", "-r", "-z", "--full-tree", revision],
        check=False,
        capture_output=True,
    )
    if result.returncode != 0:
        return None

    paths: set[str] = set()
    for record in result.stdout.split(b"\0"):
        if not record:
            continue
        metadata, separator, raw_path = record.partition(b"\t")
        fields = metadata.split()
        if separator != b"\t" or len(fields) != 3:
            return None
        mode, object_kind, _object_id = fields
        if object_kind == b"blob" and mode in {b"100644", b"100755"}:
            paths.add(raw_path.decode("utf-8", errors="surrogateescape"))
    return paths


def validate_entry(
    root: Path,
    entry: object,
    index: int,
    seen_ids: set[str],
    errors: list[dict[str, object]],
    frozen_regular_files: set[str] | None,
    repository_revision: str | None,
) -> None:
    """Validate one complete, explicit manifest entry."""
    if not isinstance(entry, dict):
        errors.append(issue("invalid_entry", "each audit entry must be an object", index=index))
        return
    unknown = sorted(set(entry) - ENTRY_FIELDS)
    missing = sorted(ENTRY_FIELDS - set(entry))
    if unknown:
        errors.append(issue("unknown_entry_field", "audit entries cannot carry untracked fields", index=index, fields=unknown))
    if missing:
        errors.append(issue("missing_entry_field", "audit entries must contain the complete schema", index=index, fields=missing))
    if unknown or missing:
        return

    entry_id = entry["id"]
    if not nonempty(entry_id):
        errors.append(issue("invalid_entry_id", "audit entry ids must be non-empty", index=index, entry=entry_id))
    elif entry_id in seen_ids:
        errors.append(issue("duplicate_entry_id", "audit entry ids must be unique", entry=entry_id))
    else:
        seen_ids.add(entry_id)

    path = entry["path"]
    if isinstance(path, str) and NONFINITE_PATH_CHARACTER.search(path):
        errors.append(issue("nonfinite_entry_path", "each entry must name one explicit path, not a pattern", entry=entry_id, path=path))
    else:
        entry_path = relative_file(root, path)
        if entry_path is None:
            errors.append(issue("invalid_entry_path", "entry path must be a safe existing repository-relative file", entry=entry_id, path=path))
        else:
            locator = entry["locator"]
            try:
                source = entry_path.read_text(encoding="utf-8")
            except OSError as error:
                errors.append(issue("invalid_entry_path", "entry path must be readable as a text file", entry=entry_id, path=path, detail=str(error)))
            else:
                if not isinstance(locator, str) or not locator or locator not in source:
                    errors.append(issue("missing_entry_locator", "locator must occur exactly in its named file", entry=entry_id, path=path, locator=locator))

    frozen_path = repository_relative_path(path)
    if (
        frozen_regular_files is not None
        and frozen_path is not None
        and frozen_path not in frozen_regular_files
    ):
        errors.append(issue(
            "entry_not_in_repository_revision",
            "entry path must name a regular file at repository_revision",
            entry=entry_id,
            path=path,
            revision=repository_revision,
        ))

    for field, allowed, kind in (
        ("reachability", REACHABILITY, "invalid_reachability"),
        ("classification", CLASSIFICATION, "invalid_classification"),
        ("execution_role", EXECUTION_ROLE, "invalid_execution_role"),
        ("disposition", DISPOSITION, "invalid_disposition"),
    ):
        if entry[field] not in allowed:
            errors.append(issue(kind, "entry field is outside the controlled audit vocabulary", entry=entry_id, field=field, value=entry[field]))

    for field in (
        "locator",
        "current_role",
        "target_rule_or_contract",
        "owner_or_external_handoff",
        "consumed_handoff",
        "produced_handoff",
        "rationale",
    ):
        if not nonempty(entry[field]):
            errors.append(issue("missing_entry_metadata", "entry metadata must be non-empty", entry=entry_id, field=field))
    evidence = entry["required_evidence"]
    if not isinstance(evidence, list) or not evidence or not all(nonempty(item) for item in evidence):
        errors.append(issue("invalid_required_evidence", "required_evidence must be a non-empty string list", entry=entry_id))

    classification = entry["classification"]
    disposition = entry["disposition"]
    owner = entry["owner_or_external_handoff"]
    if classification == "current" and (
        not isinstance(owner, str) or PHASE_205_OWNER.fullmatch(owner) is None
    ):
        errors.append(issue("missing_phase_205_owner", "a current entry needs its exact Phase-205 owner", entry=entry_id, owner=owner))
    if classification == "historical" and disposition != "historical":
        errors.append(issue("invalid_historical_disposition", "historical entries must retain historical disposition", entry=entry_id))
    if classification == "deferred_separate_project" and disposition != "deferred_separate_project":
        errors.append(issue("invalid_separate_project_disposition", "separate-project entries cannot be scheduled for local removal", entry=entry_id))
    if classification == "deferred_separate_project":
        if not isinstance(owner, str) or not owner.startswith("external:"):
            errors.append(issue("invalid_lean_handoff", "separate-project entries require an external handoff", entry=entry_id, owner=owner))
        for field in ("external_project", "external_owner", "external_handoff", "prohibited_current_authority"):
            if not nonempty(entry[field]):
                errors.append(issue("missing_lean_handoff_metadata", "separate-project entries require a complete external handoff", entry=entry_id, field=field))
        retained_paths = entry["retained_paths"]
        if not isinstance(retained_paths, list) or not retained_paths or not all(nonempty(item) for item in retained_paths):
            errors.append(issue("invalid_lean_retained_paths", "separate-project entries must name retained paths", entry=entry_id, field="retained_paths"))

    finite_case_fields = ("case_id", "missing_obligation", "fail_closed_result")
    if disposition == "deferred":
        for field, kind in (
            ("case_id", "missing_deferred_case_id"),
            ("missing_obligation", "missing_deferred_obligation"),
            ("fail_closed_result", "missing_deferred_fail_closed_result"),
        ):
            if not nonempty(entry[field]):
                errors.append(issue(kind, "deferred entries must name their finite failure boundary", entry=entry_id, field=field))
    elif any(not isinstance(entry[field], str) or entry[field].strip() for field in finite_case_fields):
        errors.append(issue(
            "unexpected_finite_case_fields",
            "only finite deferred entries may carry case_id, missing_obligation, or fail_closed_result",
            entry=entry_id,
            disposition=disposition,
        ))

    if is_lean_path(path):
        if classification != "deferred_separate_project" or disposition != "deferred_separate_project":
            errors.append(issue("invalid_lean_disposition", "Lean reference material is deferred to its separate project", entry=entry_id, path=path))


def validate(root: Path, manifest_path: Path) -> list[dict[str, object]]:
    """Validate a frozen manifest and return all fail-closed errors."""
    if not manifest_path.is_file():
        return [issue("missing_manifest", "AUDIT-204 manifest is missing", path=str(manifest_path))]
    try:
        payload: Any = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        return [issue("invalid_manifest", "AUDIT-204 manifest must be readable JSON", detail=str(error))]
    if not isinstance(payload, dict):
        return [issue("invalid_manifest", "AUDIT-204 manifest must be a JSON object")]

    errors: list[dict[str, object]] = []
    unknown = sorted(set(payload) - MANIFEST_FIELDS)
    missing = sorted(MANIFEST_FIELDS - set(payload))
    if unknown:
        errors.append(issue("unknown_manifest_field", "manifest cannot carry untracked fields", fields=unknown))
    if missing:
        errors.append(issue("missing_manifest_field", "manifest must contain the complete schema", fields=missing))
    if payload.get("schema") != MANIFEST_SCHEMA:
        errors.append(issue("invalid_manifest_schema", f"manifest schema must be {MANIFEST_SCHEMA}", value=payload.get("schema")))
    repository_revision = payload.get("repository_revision")
    frozen_regular_files: set[str] | None = None
    if not isinstance(repository_revision, str) or REVISION.fullmatch(repository_revision) is None:
        errors.append(issue("invalid_repository_revision", "repository_revision must be a 40-character lowercase commit id"))
    elif repository_has_git(root):
        if not revision_exists(root, repository_revision):
            errors.append(issue("unknown_repository_revision", "repository_revision must name an existing commit when root has Git metadata", revision=repository_revision))
        else:
            frozen_regular_files = revision_regular_files(root, repository_revision)
            if frozen_regular_files is None:
                errors.append(issue(
                    "repository_revision_inventory_unavailable",
                    "repository_revision regular-file inventory must be readable when root has Git metadata",
                    revision=repository_revision,
                ))
    digest = payload.get("entries_sha256")
    if not isinstance(digest, str) or SHA256.fullmatch(digest) is None:
        errors.append(issue("invalid_entries_digest", "entries_sha256 must be a sha256 digest"))

    entries = payload.get("entries")
    if not isinstance(entries, list):
        errors.append(issue("invalid_entries", "manifest entries must be a list"))
        return errors
    if not entries:
        errors.append(issue("empty_entries", "manifest must contain a finite non-empty inventory"))
    records = [entry for entry in entries if isinstance(entry, dict) and isinstance(entry.get("id"), str)]
    if len(records) == len(entries):
        actual_digest = entries_digest(records)
        if digest != actual_digest:
            errors.append(issue("entries_digest_mismatch", "entries_sha256 must match stable-id-sorted canonical entries", expected=actual_digest, actual=digest))
    seen_ids: set[str] = set()
    for index, entry in enumerate(entries):
        validate_entry(
            root,
            entry,
            index,
            seen_ids,
            errors,
            frozen_regular_files,
            repository_revision if isinstance(repository_revision, str) else None,
        )
    return errors


def main() -> int:
    """Run the audit validator as a JSON-only CLI."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--format", choices=("json",), default="json")
    args = parser.parse_args()
    errors = validate(args.root.resolve(), args.manifest)
    print(json.dumps({"schema": REPORT_SCHEMA, "errors": errors}, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
