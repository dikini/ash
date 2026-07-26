#!/usr/bin/env python3
"""Generate the Phase 202 productive-corpus authority inventory.

The tool deliberately accepts a small, controlled subset of YAML frontmatter.
That keeps the inventory reproducible without introducing a new dependency or
pretending to validate arbitrary YAML.  Its output is an audit artifact: it is
written even when the supplied scope or corpus contains conflicts.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any


SCOPE_SCHEMA = "corpus-authority-scope/v1"
INVENTORY_SCHEMA = "corpus-authority-inventory/v1"
CLASSIFICATION_OVERLAY_SCHEMA = "corpus-authority-classification-overlay/v1"
FRONTMATTER_RE = re.compile(r"\A---\s*\n(.*?)\n---\s*(?:\n|\Z)", re.DOTALL)
LINK_RE = re.compile(r"(?<!!)\[[^\]]+\]\(([^)\s]+)(?:\s+\"[^\"]*\")?\)")


def parse_scalar(value: str) -> Any:
    """Parse the scalar/list subset used by corpus frontmatter."""
    value = value.strip()
    if value.startswith(("{", "[")):
        try:
            return json.loads(value)
        except json.JSONDecodeError:
            # The simple bracket-list syntax below is intentionally retained for
            # frontmatter such as ``canonical_for: [grammar.surface]``.
            pass
    if value == "null":
        return None
    if value == "[]":
        return []
    if value.startswith("[") and value.endswith("]"):
        body = value[1:-1].strip()
        if not body:
            return []
        return [part.strip().strip("\"'") for part in body.split(",")]
    return value.strip("\"'")


def parse_frontmatter(text: str) -> dict[str, Any]:
    """Parse the controlled YAML subset used by SPEC-071-style frontmatter.

    The corpus deliberately does not depend on a general YAML parser.  This
    accepts mappings, lists, JSON-style inline values, strings, and ``null``;
    that is enough to retain typed ``verified_against`` and ``related`` data
    without silently flattening it.
    """
    match = FRONTMATTER_RE.match(text)
    if not match:
        return {}
    lines = [line for line in match.group(1).splitlines() if line.strip() and not line.lstrip().startswith("#")]

    def indentation(line: str) -> int:
        return len(line) - len(line.lstrip(" "))

    def parse_block(index: int, indent: int) -> tuple[Any, int]:
        if index >= len(lines) or indentation(lines[index]) < indent:
            return {}, index
        is_list = lines[index].lstrip().startswith("- ")
        result: Any = [] if is_list else {}
        while index < len(lines) and indentation(lines[index]) == indent:
            raw = lines[index].strip()
            if is_list:
                if not raw.startswith("- "):
                    break
                value = raw[2:].strip()
                if value:
                    result.append(parse_scalar(value))
                    index += 1
                else:
                    index += 1
                    nested, index = parse_block(index, indent + 2)
                    result.append(nested)
                continue
            if raw.startswith("- ") or ":" not in raw:
                index += 1
                continue
            key, value = raw.split(":", 1)
            key, value = key.strip(), value.strip()
            index += 1
            if value:
                result[key] = parse_scalar(value)
            elif index < len(lines) and indentation(lines[index]) > indent:
                result[key], index = parse_block(index, indentation(lines[index]))
            else:
                result[key] = None
        return result, index

    parsed, _ = parse_block(0, 0)
    return parsed if isinstance(parsed, dict) else {}


def as_strings(value: Any) -> list[str]:
    if value is None:
        return []
    if isinstance(value, list):
        return [str(item) for item in value]
    return [str(value)]


def relative_path(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def scope_conflict(message: str) -> dict[str, Any]:
    return {"kind": "malformed_scope", "message": message}


def missing_frozen_scope_field(field: str) -> dict[str, Any]:
    return {"kind": "missing_frozen_scope_field", "field": field, "message": f"frozen scope requires {field}"}


def malformed_exclusion(message: str) -> dict[str, Any]:
    return {"kind": "malformed_exclusion", "message": message}


def is_non_empty_string(value: Any) -> bool:
    """Return whether a manifest scalar is a non-empty string."""
    return isinstance(value, str) and bool(value.strip())


def list_of_strings(value: Any) -> bool:
    """Return whether a manifest value is a list of non-empty strings."""
    return isinstance(value, list) and all(is_non_empty_string(item) for item in value)


def is_relative_to_root(root: Path, item: str) -> bool:
    """Return whether the manifest path stays underneath the inventory root."""
    try:
        (root / item).resolve().relative_to(root)
    except ValueError:
        return False
    return True


def existing_path_inside_root(root: Path, item: str, *, file: bool = False) -> bool:
    """Require a real in-root path without accepting an escaping symlink."""
    candidate = root / item
    try:
        resolved = candidate.resolve(strict=True)
        resolved.relative_to(root)
    except (OSError, ValueError):
        return False
    return resolved.is_file() if file else resolved.exists()


def classification_fields_valid(record: dict[str, Any]) -> bool:
    """Return whether a scope-owned artifact classification is complete.

    This intentionally mirrors the minimum classification required of Markdown
    frontmatter.  Scope records are authoritative overlays, not a second,
    weaker metadata format.
    """
    return (
        is_non_empty_string(record.get("id"))
        and is_non_empty_string(record.get("authority_level", record.get("authority")))
        and list_of_strings(record.get("canonical_for"))
        and bool(as_strings(record.get("lifecycle")))
        and bool(as_strings(record.get("status")))
    )


def validate_evidence(root: Path, artifact_path: str, evidence: Any) -> list[dict[str, Any]]:
    """Validate structured document evidence without flattening it."""
    conflicts: list[dict[str, Any]] = []
    if evidence is None:
        return conflicts
    if not isinstance(evidence, dict):
        return [{"kind": "invalid_evidence_path", "path": artifact_path, "message": "verified_against must be a mapping"}]
    for key in ("code", "tests"):
        paths = evidence.get(key, [])
        if not list_of_strings(paths):
            conflicts.append({"kind": "invalid_evidence_path", "path": artifact_path, "message": f"verified_against.{key} must be a list of strings"})
            continue
        for evidence_path in paths:
            if not existing_path_inside_root(root, evidence_path, file=True):
                conflicts.append({"kind": "invalid_evidence_path", "path": evidence_path, "artifact": artifact_path, "message": "evidence path must be an existing in-root file"})
    return conflicts


def load_scope(path: Path, root: Path) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    """Load and validate the frozen scope format without raising to the caller."""
    conflicts: list[dict[str, Any]] = []
    try:
        scope = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        return {}, [scope_conflict(f"cannot read scope: {error}")]
    if not isinstance(scope, dict):
        return {}, [scope_conflict("scope must be a JSON object")]
    if scope.get("schema") != SCOPE_SCHEMA:
        conflicts.append(scope_conflict(f"schema must be {SCOPE_SCHEMA!r}"))
    for field in ("included_roots", "excluded_paths", "semantic_rust_crates"):
        value = scope.get(field)
        if not list_of_strings(value):
            conflicts.append(scope_conflict(f"{field} must be a list of strings"))
    for crate_root in scope.get("semantic_rust_crates", []):
        if not existing_path_inside_root(root, crate_root):
            conflicts.append({"kind": "invalid_semantic_rust_root", "path": crate_root, "message": "semantic Rust root must exist and stay within root"})
    if not isinstance(scope.get("scope_id"), str) or not scope.get("scope_id"):
        conflicts.append(scope_conflict("scope_id must be a non-empty string"))
    for field in ("included_roots", "excluded_paths"):
        for item in scope.get(field, []):
            if not is_relative_to_root(root, item):
                conflicts.append(scope_conflict(f"{field} path escapes root: {item}"))

    if not is_non_empty_string(scope.get("repository_revision")):
        conflicts.append(missing_frozen_scope_field("repository_revision"))

    dirty = scope.get("dirty_worktree")
    if not isinstance(dirty, dict):
        conflicts.append(missing_frozen_scope_field("dirty_worktree"))
    else:
        if not isinstance(dirty.get("qualified"), bool):
            conflicts.append(scope_conflict("dirty_worktree.qualified must be a boolean"))
        if not isinstance(dirty.get("changed_paths"), list) or not list_of_strings(dirty.get("changed_paths")):
            conflicts.append(scope_conflict("dirty_worktree.changed_paths must be a list of strings"))
        elif any(not is_relative_to_root(root, item) for item in dirty["changed_paths"]):
            conflicts.append(scope_conflict("dirty_worktree.changed_paths must stay within root"))
        if not is_non_empty_string(dirty.get("qualification")):
            conflicts.append(scope_conflict("dirty_worktree.qualification must be a non-empty string"))

    productive_roots = scope.get("productive_roots")
    if not list_of_strings(productive_roots) or not productive_roots:
        conflicts.append(missing_frozen_scope_field("productive_roots"))
    elif any(not existing_path_inside_root(root, item) for item in productive_roots):
        conflicts.append(scope_conflict("productive_roots must exist and stay within root"))

    included_roots = scope.get("included_roots", [])
    if isinstance(included_roots, list) and list_of_strings(included_roots):
        if not included_roots:
            conflicts.append(scope_conflict("included_roots must not be empty"))
        for item in included_roots:
            if not existing_path_inside_root(root, item):
                conflicts.append({"kind": "invalid_included_root", "path": item, "message": "included root must exist and stay within root"})

    exclusions = scope.get("exclusions", [])
    if not isinstance(exclusions, list):
        conflicts.append(malformed_exclusion("exclusions must be a list"))
    else:
        seen_exclusions: set[str] = set()
        for index, exclusion in enumerate(exclusions):
            if not isinstance(exclusion, dict):
                conflicts.append(malformed_exclusion(f"exclusions[{index}] must be an object"))
                continue
            path_value = exclusion.get("path")
            if not is_non_empty_string(path_value):
                conflicts.append(malformed_exclusion(f"exclusions[{index}].path must be a non-empty string"))
            elif not is_relative_to_root(root, path_value):
                conflicts.append(malformed_exclusion(f"exclusions[{index}].path escapes root: {path_value}"))
            elif path_value in seen_exclusions:
                conflicts.append(malformed_exclusion(f"duplicate exclusion path: {path_value}"))
            else:
                seen_exclusions.add(path_value)
            if not is_non_empty_string(exclusion.get("reason")):
                conflicts.append(malformed_exclusion(f"exclusions[{index}].reason must be a non-empty string"))

    semantic_rust = scope.get("semantic_rust", [])
    if not isinstance(semantic_rust, list):
        conflicts.append(scope_conflict("semantic_rust must be a list"))
    else:
        for index, record in enumerate(semantic_rust):
            prefix = f"semantic_rust[{index}]"
            if not isinstance(record, dict):
                conflicts.append(scope_conflict(f"{prefix} must be an object"))
                continue
            for field in ("path", "classification"):
                if not is_non_empty_string(record.get(field)):
                    conflicts.append(scope_conflict(f"{prefix}.{field} must be a non-empty string"))
            if is_non_empty_string(record.get("path")) and not is_relative_to_root(root, record["path"]):
                conflicts.append(scope_conflict(f"{prefix}.path escapes root: {record['path']}"))
            for field in ("tests", "canonical_subjects"):
                if not list_of_strings(record.get(field)):
                    conflicts.append(scope_conflict(f"{prefix}.{field} must be a list of strings"))
            if is_non_empty_string(record.get("path")) and not existing_path_inside_root(root, record["path"], file=True):
                conflicts.append({"kind": "invalid_evidence_path", "path": record["path"], "message": f"{prefix}.path must be an existing in-root file"})
            for test_path in record.get("tests", []) if isinstance(record.get("tests"), list) else []:
                if not existing_path_inside_root(root, test_path, file=True):
                    conflicts.append({"kind": "invalid_evidence_path", "path": test_path, "message": f"{prefix}.tests path must be an existing in-root file"})
            missing_evidence = (
                not is_non_empty_string(record.get("symbol"))
                or not isinstance(record.get("executed_test"), dict)
                or not is_non_empty_string(record.get("executed_test", {}).get("command"))
                or not is_non_empty_string(record.get("executed_test", {}).get("result"))
            )
            if missing_evidence:
                conflicts.append({"kind": "missing_semantic_rust_evidence", "index": index, "message": f"{prefix} needs symbol and executed_test command/result evidence"})

    overlay = scope.get("classification_overlay")
    if overlay is not None:
        if not isinstance(overlay, dict) or overlay.get("schema") != CLASSIFICATION_OVERLAY_SCHEMA:
            conflicts.append(scope_conflict(f"classification_overlay.schema must be {CLASSIFICATION_OVERLAY_SCHEMA!r}"))
        entries = overlay.get("entries") if isinstance(overlay, dict) else None
        if not isinstance(entries, list):
            conflicts.append(scope_conflict("classification_overlay.entries must be a list"))
        else:
            seen_overlay_paths: set[str] = set()
            for index, entry in enumerate(entries):
                prefix = f"classification_overlay.entries[{index}]"
                if not isinstance(entry, dict) or not is_non_empty_string(entry.get("path")):
                    conflicts.append(scope_conflict(f"{prefix}.path must be a non-empty string"))
                    continue
                path_value = entry["path"]
                if not is_relative_to_root(root, path_value) or not existing_path_inside_root(root, path_value, file=True):
                    conflicts.append({"kind": "invalid_overlay_path", "path": path_value, "message": f"{prefix}.path must be an existing in-root file"})
                if path_value in seen_overlay_paths:
                    conflicts.append(scope_conflict(f"duplicate classification overlay path: {path_value}"))
                seen_overlay_paths.add(path_value)
                if not classification_fields_valid(entry):
                    conflicts.append({"kind": "malformed_overlay_classification", "path": path_value, "message": f"{prefix} needs id, authority, canonical_for, lifecycle, and status"})
                conflicts.extend(validate_evidence(root, path_value, entry.get("verified_against")))

    data_artifacts = scope.get("data_artifacts", [])
    if not isinstance(data_artifacts, list):
        conflicts.append(scope_conflict("data_artifacts must be a list"))
    else:
        seen_data_paths: set[str] = set()
        for index, record in enumerate(data_artifacts):
            prefix = f"data_artifacts[{index}]"
            if not isinstance(record, dict) or not is_non_empty_string(record.get("path")):
                conflicts.append(scope_conflict(f"{prefix}.path must be a non-empty string"))
                continue
            path_value = record["path"]
            if not is_relative_to_root(root, path_value) or not existing_path_inside_root(root, path_value, file=True):
                conflicts.append({"kind": "invalid_data_artifact_path", "path": path_value, "message": f"{prefix}.path must be an existing in-root file"})
            if path_value in seen_data_paths:
                conflicts.append(scope_conflict(f"duplicate data artifact path: {path_value}"))
            seen_data_paths.add(path_value)
            if not classification_fields_valid(record):
                conflicts.append({"kind": "malformed_data_artifact", "path": path_value, "message": f"{prefix} needs id, authority, canonical_for, lifecycle, and status"})
            conflicts.extend(validate_evidence(root, path_value, record.get("verified_against")))

    known_conflicts = scope.get("known_conflicts", [])
    if not isinstance(known_conflicts, list):
        conflicts.append(scope_conflict("known_conflicts must be a list"))
    else:
        ids: set[str] = set()
        for index, conflict in enumerate(known_conflicts):
            if not isinstance(conflict, dict) or not is_non_empty_string(conflict.get("id")):
                conflicts.append(scope_conflict(f"known_conflicts[{index}].id must be a non-empty string"))
                continue
            # PLAN-202 originally froze four ID-only ledger references.  They
            # remain accepted for the historic fixture; every structured entry
            # uses the complete, subject-free ledger shape below.
            initial_ids = {
                "conflict.docs-readme-spec-index", "conflict.formalization-boundary",
                "conflict.parser-to-core", "conflict.phase-201-handoff",
            }
            if conflict["id"] not in initial_ids:
                required = {"id", "involved_paths", "competing_claims", "evidence", "disposition", "status"}
                valid_shape = set(conflict) == required
                valid_values = (
                    list_of_strings(conflict.get("involved_paths"))
                    and list_of_strings(conflict.get("competing_claims"))
                    and list_of_strings(conflict.get("evidence"))
                    and is_non_empty_string(conflict.get("disposition"))
                    and is_non_empty_string(conflict.get("status"))
                )
                if not valid_shape or not valid_values:
                    conflicts.append({"kind": "malformed_known_conflict", "index": index, "message": "known conflict must use exactly id, involved_paths, competing_claims, evidence, disposition, and status"})
                else:
                    for path_value in [*conflict["involved_paths"], *conflict["evidence"]]:
                        if not is_relative_to_root(root, path_value) or not existing_path_inside_root(root, path_value, file=True):
                            conflicts.append({"kind": "invalid_known_conflict_path", "id": conflict["id"], "path": path_value, "message": "known conflict paths must be existing files within root"})
            if conflict["id"] in ids:
                conflicts.append(scope_conflict(f"duplicate known conflict id: {conflict['id']}"))
            ids.add(conflict["id"])

    expected_subjects = scope.get("expected_canonical_subjects", [])
    if not isinstance(expected_subjects, list):
        conflicts.append(scope_conflict("expected_canonical_subjects must be a list"))
    else:
        for index, expected in enumerate(expected_subjects):
            if isinstance(expected, str) and expected.strip():
                continue
            if isinstance(expected, dict) and is_non_empty_string(expected.get("subject")):
                status = expected.get("status", "required")
                if status not in {"required", "unresolved"}:
                    conflicts.append(scope_conflict(f"expected_canonical_subjects[{index}].status must be required or unresolved"))
                continue
            conflicts.append(scope_conflict(f"expected_canonical_subjects[{index}] must be a subject string or object"))
    return scope, conflicts


def status_claims(metadata: dict[str, Any]) -> list[str]:
    """Return only explicitly structured status metadata.

    Narrative temporal language explains a document but does not constitute an
    auditable status claim.  The merged frontmatter/overlay mapping is the
    sole source of status classifications.
    """
    return sorted({value.lower() for value in as_strings(metadata.get("status"))})


def productive_markdown(root: Path, scope: dict[str, Any]) -> list[Path]:
    paths: set[Path] = set()
    exclusions = set(scope.get("excluded_paths", []))
    exclusions.update(
        exclusion["path"]
        for exclusion in scope.get("exclusions", [])
        if isinstance(exclusion, dict) and is_non_empty_string(exclusion.get("path"))
    )
    for included_root in scope.get("included_roots", []):
        directory = root / included_root
        if directory.is_file() and directory.suffix.lower() == ".md":
            paths.add(directory)
        elif directory.is_dir():
            paths.update(directory.rglob("*.md"))
    return sorted(
        (
            path
            for path in paths
            if not any(
                relative_path(root, path) == exclusion
                or relative_path(root, path).startswith(f"{exclusion}/")
                for exclusion in exclusions
            )
        ),
        key=lambda path: relative_path(root, path),
    )


def artifact_from_metadata(
    root: Path,
    path_text: str,
    data: dict[str, Any],
    *,
    kind: str,
    text: str = "",
    require_overlay: bool = False,
    overlay_present: bool = True,
) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    """Build one inventory row from Markdown or scope-declared metadata."""
    artifact: dict[str, Any] = {
        "id": data.get("id"),
        "path": path_text,
        "kind": data.get("kind", kind),
        "claimed_authority": data.get("authority_level", data.get("authority")),
        "canonical_subjects": sorted(as_strings(data.get("canonical_for"))),
        "lifecycle_claims": sorted(as_strings(data.get("lifecycle"))),
        "status_claims": status_claims(data),
        "conflicts": [],
        "known_conflicts": [],
        "inbound_productive_links": [],
        "current_target_historical": data.get("current_target_historical"),
        "unique_content": as_strings(data.get("unique_content")),
        "proposed_disposition": data.get("proposed_disposition"),
        "verified_against": data.get("verified_against"),
        "related": data.get("related"),
    }
    conflicts: list[dict[str, Any]] = []
    if require_overlay and not overlay_present:
        conflicts.append({"kind": "missing_overlay_classification", "path": path_text, "message": "scoped artifact has no classification overlay entry"})
    else:
        if not artifact["status_claims"]:
            conflicts.append({"kind": "missing_status", "path": artifact["path"], "message": "productive artifact has no status claim"})
        if "current" in artifact["status_claims"] and "historical" in artifact["status_claims"]:
            conflicts.append({"kind": "contradictory_status_claim", "path": artifact["path"], "claims": artifact["status_claims"]})
        if not artifact["claimed_authority"] or not artifact["lifecycle_claims"] or not artifact["canonical_subjects"]:
            conflicts.append({"kind": "unclassified_artifact", "path": artifact["path"], "message": "productive artifact needs authority, lifecycle, and canonical subject classification"})
    conflicts.extend(validate_evidence(root, path_text, artifact["verified_against"]))
    artifact["conflicts"] = conflicts
    return artifact, conflicts


def artifact_for(
    root: Path,
    path: Path,
    *,
    overlay: dict[str, Any] | None = None,
    require_overlay: bool = False,
) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    path_text = relative_path(root, path)

    def unreadable_artifact(conflict: dict[str, Any]) -> tuple[dict[str, Any], list[dict[str, Any]]]:
        return {
            "id": None,
            "path": path_text,
            "kind": "markdown",
            "claimed_authority": None,
            "canonical_subjects": [],
            "lifecycle_claims": [],
            "status_claims": [],
            "conflicts": [conflict],
            "known_conflicts": [],
            "inbound_productive_links": [],
            "current_target_historical": None,
            "unique_content": [],
            "proposed_disposition": None,
            "verified_against": None,
            "related": None,
        }, [conflict]

    try:
        resolved = path.resolve(strict=True)
        resolved.relative_to(root)
    except (OSError, ValueError):
        conflict = {"kind": "escaping_symlink", "path": path_text, "message": "Markdown artifact resolves outside inventory root"}
        return unreadable_artifact(conflict)
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError as error:
        conflict = {"kind": "malformed_utf8", "path": path_text, "message": str(error)}
        return unreadable_artifact(conflict)
    except OSError as error:
        conflict = {"kind": "unreadable_artifact", "path": path_text, "message": str(error)}
        return unreadable_artifact(conflict)
    data = parse_frontmatter(text)
    # Scope classifications deliberately win over incidental frontmatter: the
    # frozen manifest is the audit's declared authority for sparse documents.
    if overlay is not None:
        data = {**data, **{key: value for key, value in overlay.items() if key != "path"}}
    return artifact_from_metadata(
        root,
        path_text,
        data,
        kind="markdown",
        text=text,
        require_overlay=require_overlay,
        overlay_present=overlay is not None,
    )


def add_inbound_links(root: Path, artifacts: list[dict[str, Any]]) -> None:
    by_path = {str(artifact["path"]): artifact for artifact in artifacts}
    inbound: dict[str, set[str]] = defaultdict(set)
    for source in artifacts:
        source_path = root / str(source["path"])
        try:
            resolved_source = source_path.resolve(strict=True)
            resolved_source.relative_to(root)
            text = source_path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError, ValueError):
            continue
        for match in LINK_RE.finditer(text):
            target = match.group(1).split("#", 1)[0]
            if not target or target.startswith(("http://", "https://", "mailto:")):
                continue
            candidate = (source_path.parent / target).resolve()
            try:
                target_path = relative_path(root, candidate)
            except ValueError:
                continue
            if target_path in by_path:
                inbound[target_path].add(str(source["path"]))
    for target_path, sources in inbound.items():
        by_path[target_path]["inbound_productive_links"] = sorted(sources)


def duplicate_owner_conflicts(artifacts: list[dict[str, Any]]) -> list[dict[str, Any]]:
    owners: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for artifact in artifacts:
        active = "active" in artifact["lifecycle_claims"] or "current" in artifact["status_claims"]
        if active and artifact["claimed_authority"] == "A1":
            for subject in artifact["canonical_subjects"]:
                owners[subject].append(artifact)
    conflicts: list[dict[str, Any]] = []
    for subject, candidates in sorted(owners.items()):
        if len(candidates) > 1:
            paths = sorted(str(candidate["path"]) for candidate in candidates)
            conflict = {"kind": "duplicate_canonical_owner", "subject": subject, "paths": paths}
            conflicts.append(conflict)
            for candidate in candidates:
                candidate["conflicts"].append(conflict)
    return conflicts


def duplicate_id_conflicts(artifacts: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Report repeated stable artifact IDs while retaining every artifact."""
    by_id: dict[str, list[dict[str, Any]]] = defaultdict(list)
    conflicts: list[dict[str, Any]] = []
    for artifact in artifacts:
        stable_id = artifact["id"]
        if is_non_empty_string(stable_id):
            by_id[stable_id].append(artifact)
        else:
            conflict = {"kind": "missing_id", "path": artifact["path"], "message": "productive artifact has no stable id"}
            artifact["conflicts"].append(conflict)
            conflicts.append(conflict)
    for stable_id, candidates in sorted(by_id.items()):
        if len(candidates) > 1:
            conflict = {
                "kind": "duplicate_id",
                "id": stable_id,
                "paths": sorted(str(candidate["path"]) for candidate in candidates),
            }
            conflicts.append(conflict)
            for candidate in candidates:
                candidate["conflicts"].append(conflict)
    return conflicts


def missing_canonical_owner_conflicts(scope: dict[str, Any], artifacts: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Ensure every required expected subject has an active A1 current owner."""
    owners = {
        subject
        for artifact in artifacts
        if artifact["claimed_authority"] == "A1"
        and ("active" in artifact["lifecycle_claims"] or "current" in artifact["status_claims"])
        for subject in artifact["canonical_subjects"]
    }
    conflicts: list[dict[str, Any]] = []
    for expected in scope.get("expected_canonical_subjects", []):
        if isinstance(expected, str):
            subject, status = expected, "required"
        elif isinstance(expected, dict):
            subject, status = expected["subject"], expected.get("status", "required")
        else:
            continue
        if status != "unresolved" and subject not in owners:
            conflicts.append({"kind": "missing_canonical_owner", "subject": subject})
    return conflicts


def link_known_conflicts(
    scope: dict[str, Any],
    artifacts: list[dict[str, Any]],
    semantic_rust: list[dict[str, Any]],
) -> None:
    """Expose each structured ledger conflict from each affected inventory row."""
    by_path = {str(artifact["path"]): artifact for artifact in artifacts}
    rust_by_path = {str(record["path"]): record for record in semantic_rust if is_non_empty_string(record.get("path"))}
    for conflict in scope.get("known_conflicts", []):
        if not isinstance(conflict, dict):
            continue
        conflict_id = conflict.get("id")
        involved_paths = conflict.get("involved_paths")
        if not is_non_empty_string(conflict_id) or not list_of_strings(involved_paths):
            continue
        for path_text in involved_paths:
            artifact = by_path.get(path_text)
            if artifact is not None:
                artifact["known_conflicts"].append(conflict_id)
            semantic_record = rust_by_path.get(path_text)
            if semantic_record is not None:
                semantic_record.setdefault("known_conflicts", []).append(conflict_id)
    for artifact in artifacts:
        artifact["known_conflicts"].sort()
    for record in semantic_rust:
        if "known_conflicts" in record:
            record["known_conflicts"].sort()


def write_output(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", required=True, help="repository or fixture root")
    parser.add_argument("--scope", required=True, help="scope manifest JSON path")
    parser.add_argument("--output", required=True, help="inventory JSON output path")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    output = Path(args.output).resolve()
    scope, conflicts = load_scope(Path(args.scope).resolve(), root)
    artifacts: list[dict[str, Any]] = []
    semantic_rust = [
        dict(record)
        for record in scope.get("semantic_rust", [])
        if isinstance(record, dict)
    ]
    if root.is_dir():
        overlay_entries = (
            {
                entry["path"]: entry
                for entry in scope.get("classification_overlay", {}).get("entries", [])
                if isinstance(entry, dict) and is_non_empty_string(entry.get("path"))
            }
            if isinstance(scope.get("classification_overlay"), dict)
            else {}
        )
        require_overlay = "classification_overlay" in scope
        for path in productive_markdown(root, scope):
            path_text = relative_path(root, path)
            artifact, artifact_conflicts = artifact_for(
                root,
                path,
                overlay=overlay_entries.get(path_text),
                require_overlay=require_overlay,
            )
            artifacts.append(artifact)
            conflicts.extend(artifact_conflicts)
        for record in scope.get("data_artifacts", []) if isinstance(scope.get("data_artifacts"), list) else []:
            if not isinstance(record, dict) or not is_non_empty_string(record.get("path")):
                continue
            path_text = record["path"]
            if any(artifact["path"] == path_text for artifact in artifacts):
                continue
            artifact, artifact_conflicts = artifact_from_metadata(
                root,
                path_text,
                record,
                kind="data",
            )
            artifacts.append(artifact)
            conflicts.extend(artifact_conflicts)
        add_inbound_links(root, artifacts)
        link_known_conflicts(scope, artifacts, semantic_rust)
        conflicts.extend(duplicate_owner_conflicts(artifacts))
        conflicts.extend(duplicate_id_conflicts(artifacts))
        conflicts.extend(missing_canonical_owner_conflicts(scope, artifacts))
    elif not root.is_dir():
        conflicts.append({"kind": "invalid_root", "message": f"root is not a directory: {root}"})

    payload = {
        "schema": INVENTORY_SCHEMA,
        "scope_id": scope.get("scope_id"),
        "frozen_scope": scope,
        "artifacts": artifacts,
        "semantic_rust": semantic_rust,
        "known_conflicts": scope.get("known_conflicts", []),
        "conflicts": conflicts,
    }
    write_output(output, payload)
    if conflicts:
        print(f"corpus authority inventory failed: artifacts={len(artifacts)} conflicts={len(conflicts)}", file=sys.stderr)
        return 1
    print(f"corpus authority inventory passed: artifacts={len(artifacts)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
