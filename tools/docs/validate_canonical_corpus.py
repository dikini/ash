#!/usr/bin/env python3
"""Validate the Phase 202 ``canonical-corpus/v1`` authority sidecar.

This validator deliberately does not parse or alter SPEC-071 frontmatter.  The
sidecar is an overlay for canonical authority; typed edges are the boundary
between it and the independent reference-corpus schema.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
from typing import Any


REPORT_SCHEMA = "canonical-corpus-validation-report/v1"
MANIFEST_SCHEMA = "canonical-corpus/v1"
AUTHORITY_LEVELS = {"A0", "A1", "A2", "A3", "A4", "A5"}
LIFECYCLES = {"active", "draft", "generated", "superseded", "archived"}
KINDS = {
    "agent-card", "agent-pack", "archive", "audit", "conformance-case",
    "evidence", "generated", "handoff-contract", "manifest", "plan",
    "reference", "result-schema", "semantic-rule-set", "vocabulary",
}
EDGE_KINDS = {
    "defines", "refines", "requires", "lowers_to", "projects_to",
    "implemented_by", "tested_by", "proved_by", "assumes", "supersedes",
    # The first sidecar used these names before PLAN-202 §9.2 froze the
    # vocabulary.  Keep them readable as legacy path-edge aliases so this
    # validator can be introduced without silently rewriting existing data.
    "depends_on", "derives", "evidence_for", "explains", "implements",
    "projects", "tests",
}
DERIVATIVE_KINDS = {"agent-card", "agent-pack", "generated", "reference"}
CANONICAL_METADATA = {
    "owner", "audience", "stability", "verified_against", "related",
    "refresh_trigger", "last_verified",
}
SPEC071_VERIFIED = {"git_commit", "specs", "tasks", "code", "tests", "examples"}
SPEC071_RELATED = {"depends_on", "explains", "supersedes", "superseded_by", "historical_rationale"}
MARKDOWN_HEADING = re.compile(r"^#{1,6}\s+(.+?)\s*#*\s*$", re.MULTILINE)

# PLAN-202 §4.3's compact semantic core.  These subjects are deliberately
# checked only by the opt-in promotion gate: the initial TASK-1985 sidecar is
# useful governance infrastructure, but is not itself evidence of promotion.
PROMOTION_SUBJECTS = (
    "vocabulary.language-overview",
    "grammar.target",
    "types-effects.target",
    "core-cps.syntax",
    "lowering.surface-to-core",
    "semantics.operational",
    "runtime.observable",
    "conformance.implementation",
)

# These were workflow-first reconciliation documents.  They can remain as
# historical A5 records or A2 handoffs, but must never regain target-semantic
# ownership during promotion.
FORMER_WORKFLOW_FIRST_PATHS = {
    "docs/reference/formalization-boundary.md",
    "docs/reference/parser-to-core-lowering-contract.md",
}


def error(kind: str, message: str, **details: object) -> dict[str, object]:
    """Construct a stable, machine-readable error without leaking tracebacks."""
    return {"kind": kind, "message": message, **details}


def relative_file(root: Path, value: object) -> Path | None:
    """Resolve a repo-relative regular file, rejecting absolute and escaping paths."""
    if not isinstance(value, str) or not value:
        return None
    candidate = Path(value)
    if candidate.is_absolute():
        return None
    resolved = (root / candidate).resolve()
    try:
        resolved.relative_to(root)
    except ValueError:
        return None
    return resolved if resolved.is_file() else None


def normalized_relative_path(root: Path, value: object) -> str | None:
    """Return the normalized repo-relative path for a safe regular file."""
    resolved = relative_file(root, value)
    return resolved.relative_to(root).as_posix() if resolved is not None else None


def is_reference_derivative(root: Path, node: dict[str, object]) -> bool:
    """Classify derivatives using normalized paths, never raw user strings."""
    path = normalized_relative_path(root, node.get("path"))
    return node.get("kind") in DERIVATIVE_KINDS or (path is not None and (path == "reference" or path.startswith("reference/")))


def require_list(value: object) -> list[object] | None:
    return value if isinstance(value, list) else None


def validate_enum(errors: list[dict[str, object]], node: dict[str, object], index: int) -> None:
    allowed = {
        "kind": KINDS,
        "authority_level": AUTHORITY_LEVELS,
        "lifecycle": LIFECYCLES,
    }
    for field, values in allowed.items():
        if node.get(field) not in values:
            errors.append(error(
                "invalid_enum",
                f"node {index} has invalid {field}",
                node=node.get("id"), field=field, value=node.get(field),
            ))


def validate_canonical_metadata(errors: list[dict[str, object]], node: dict[str, object]) -> None:
    """Validate the SPEC-071-shaped metadata inherited by A1/A2 records."""
    if node.get("authority_level") not in {"A1", "A2"}:
        return
    node_id = node.get("id")
    missing = [field for field in sorted(CANONICAL_METADATA) if field not in node]
    if missing:
        errors.append(error("missing_required_metadata", f"{node_id} is missing canonical metadata", node=node_id, fields=missing))
        return
    if not isinstance(node.get("owner"), str) or not node["owner"].strip():
        errors.append(error("missing_required_metadata", f"{node_id}.owner must be non-empty", node=node_id, field="owner"))
    audience = node.get("audience")
    if not isinstance(audience, list) or not audience or not all(item in {"human", "agent"} for item in audience):
        errors.append(error("missing_required_metadata", f"{node_id}.audience must be a non-empty SPEC-071 audience list", node=node_id, field="audience"))
    if not isinstance(node.get("stability"), str) or not node["stability"]:
        errors.append(error("missing_required_metadata", f"{node_id}.stability must be non-empty", node=node_id, field="stability"))
    if not isinstance(node.get("last_verified"), str) or not node["last_verified"]:
        errors.append(error("missing_required_metadata", f"{node_id}.last_verified must be non-empty", node=node_id, field="last_verified"))
    verified = node.get("verified_against")
    if not isinstance(verified, dict):
        errors.append(error("missing_required_metadata", f"{node_id}.verified_against must retain a SPEC-071-shaped evidence mapping", node=node_id, field="verified_against"))
    related = node.get("related")
    if not isinstance(related, dict):
        errors.append(error("missing_required_metadata", f"{node_id}.related must retain a SPEC-071-shaped relationship mapping", node=node_id, field="related"))
    trigger = node.get("refresh_trigger")
    if not isinstance(trigger, list) or not trigger:
        errors.append(error("missing_required_metadata", f"{node_id}.refresh_trigger must be a non-empty list", node=node_id, field="refresh_trigger"))


def markdown_anchor_exists(path: Path, anchor: object) -> bool:
    if not isinstance(anchor, str) or not anchor.startswith("#") or len(anchor) == 1:
        return False
    wanted = anchor[1:].lower()
    # PLAN-202 anchors are stable document anchors.  The repository uses the
    # ordinary Markdown heading form, whose deterministic slug is enough here.
    for match in MARKDOWN_HEADING.finditer(path.read_text(encoding="utf-8")):
        heading = match.group(1).strip().lower()
        slug = re.sub(r"[^a-z0-9 _-]", "", heading)
        slug = re.sub(r"[ _]+", "-", slug).strip("-")
        if wanted == slug:
            return True
    return False


def cycle_nodes(nodes: dict[str, dict[str, object]]) -> set[str]:
    """Return all ids reached by a DFS back edge in the supersession graph."""
    visiting: set[str] = set()
    visited: set[str] = set()
    cyclic: set[str] = set()

    def visit(node_id: str, stack: list[str]) -> None:
        if node_id in visiting:
            cyclic.update(stack[stack.index(node_id):])
            return
        if node_id in visited:
            return
        visiting.add(node_id)
        node = nodes[node_id]
        for target in node.get("supersedes", []):
            if isinstance(target, str) and target in nodes:
                visit(target, [*stack, target])
        visiting.remove(node_id)
        visited.add(node_id)

    for node_id in nodes:
        visit(node_id, [node_id])
    return cyclic


def validate_manifest(root: Path, manifest_path: Path) -> list[dict[str, object]]:
    errors: list[dict[str, object]] = []
    try:
        data = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        return [error("invalid_manifest", "manifest is not readable JSON", detail=str(exc))]
    if not isinstance(data, dict) or data.get("schema") != MANIFEST_SCHEMA:
        return [error("invalid_manifest", f"manifest schema must be {MANIFEST_SCHEMA}")]
    raw_nodes = require_list(data.get("nodes"))
    raw_edges = require_list(data.get("typed_edges"))
    if raw_nodes is None or raw_edges is None:
        return [error("invalid_manifest", "nodes and typed_edges must be lists")]

    nodes: dict[str, dict[str, object]] = {}
    for index, raw_node in enumerate(raw_nodes):
        if not isinstance(raw_node, dict):
            errors.append(error("invalid_manifest", f"node {index} must be an object"))
            continue
        node = raw_node
        node_id = node.get("id")
        if not isinstance(node_id, str) or not node_id:
            errors.append(error("invalid_manifest", f"node {index} must have a non-empty id"))
            continue
        if node_id in nodes:
            errors.append(error("duplicate_id", f"duplicate node id: {node_id}", node=node_id))
            continue
        nodes[node_id] = node
        validate_enum(errors, node, index)
        validate_canonical_metadata(errors, node)
        for list_field in ("canonical_for", "supersedes", "depends_on", "trace_nodes"):
            if require_list(node.get(list_field)) is None:
                errors.append(error("invalid_manifest", f"{node_id}.{list_field} must be a list", node=node_id))
        if relative_file(root, node.get("path")) is None:
            errors.append(error("invalid_node_path", f"{node_id}.path must be an existing in-root file", node=node_id))
        if is_reference_derivative(root, node):
            if node.get("authority_level") != "A4":
                errors.append(error(
                    "derivative_authority_leakage",
                    f"derivative node {node_id} must use authority level A4",
                    node=node_id,
                ))

    owners: dict[str, str] = {}
    indexed_subjects: set[str] = set()
    for node_id, node in nodes.items():
        if node.get("authority_level") not in {"A1", "A2"}:
            continue
        for subject in node.get("canonical_for", []):
            if not isinstance(subject, str) or not subject:
                errors.append(error("invalid_manifest", f"{node_id}.canonical_for contains an invalid subject", node=node_id))
                continue
            indexed_subjects.add(subject)
            if node.get("lifecycle") != "active":
                continue
            prior = owners.setdefault(subject, node_id)
            if prior != node_id:
                errors.append(error(
                    "duplicate_canonical_owner",
                    f"canonical subject {subject} has more than one A1/A2 owner",
                    subject=subject, owners=sorted((prior, node_id)),
                ))

    for subject in sorted(indexed_subjects):
        if subject not in owners:
            errors.append(error(
                "missing_active_canonical_owner",
                f"canonical subject {subject} has no active A1/A2 owner",
                subject=subject,
            ))

    raw_trace_nodes = data.get("trace_nodes", [])
    if not isinstance(raw_trace_nodes, list):
        errors.append(error("invalid_manifest", "trace_nodes must be a list"))
        raw_trace_nodes = []
    traces: dict[str, dict[str, object]] = {}
    for index, raw_trace in enumerate(raw_trace_nodes):
        if not isinstance(raw_trace, dict) or not isinstance(raw_trace.get("id"), str) or not raw_trace["id"]:
            errors.append(error("invalid_trace_node", f"trace node {index} must have a non-empty id"))
            continue
        trace_id = raw_trace["id"]
        if trace_id in traces or trace_id in nodes:
            errors.append(error("duplicate_trace_id", f"trace id must be independent and unique: {trace_id}", trace=trace_id))
            continue
        traces[trace_id] = raw_trace
        document = raw_trace.get("document")
        if not isinstance(document, str) or document not in nodes:
            errors.append(error("unresolved_node_id", f"trace {trace_id}.document references an unknown node", trace=trace_id, field="document", target=document))
            continue
        document_path = relative_file(root, nodes[document].get("path"))
        if document_path is None or not isinstance(raw_trace.get("kind"), str) or not raw_trace["kind"] or not markdown_anchor_exists(document_path, raw_trace.get("anchor")):
            errors.append(error("invalid_trace_node", f"trace {trace_id} must have kind and a resolvable document anchor", trace=trace_id))

    for node_id, node in nodes.items():
        for list_field in ("supersedes", "depends_on"):
            for target in node.get(list_field, []):
                if not isinstance(target, str) or target not in nodes:
                    errors.append(error(
                        "unresolved_node_id",
                        f"{node_id}.{list_field} references an unknown node",
                        node=node_id, field=list_field, target=target,
                    ))
        for trace_id in node.get("trace_nodes", []):
            if not isinstance(trace_id, str) or trace_id not in traces:
                errors.append(error(
                    "unresolved_trace_id",
                    f"{node_id}.trace_nodes references an unknown stable trace record",
                    node=node_id, field="trace_nodes", target=trace_id,
                ))
    cyclic = cycle_nodes(nodes)
    if cyclic:
        errors.append(error("supersession_cycle", "supersedes links must be acyclic", nodes=sorted(cyclic)))

    for index, raw_edge in enumerate(raw_edges):
        if not isinstance(raw_edge, dict):
            errors.append(error("invalid_manifest", f"typed edge {index} must be an object"))
            continue
        if raw_edge.get("kind") not in EDGE_KINDS:
            errors.append(error("invalid_enum", f"typed edge {index} has invalid kind", field="kind", value=raw_edge.get("kind")))
        edge_paths: dict[str, Path] = {}
        for endpoint in ("from", "to"):
            target = raw_edge.get(endpoint)
            if isinstance(target, str) and target in nodes:
                path = relative_file(root, nodes[target].get("path"))
            else:
                path = relative_file(root, target)
            if path is None:
                errors.append(error(
                    "invalid_edge_path" if isinstance(target, str) and "/" in target else "unresolved_node_id",
                    f"typed edge {index}.{endpoint} must be a known node id or existing in-root file",
                    edge=index, endpoint=endpoint, path=target,
                ))
            else:
                edge_paths[endpoint] = path
        if "anchor" in raw_edge and ("from" not in edge_paths or not markdown_anchor_exists(edge_paths["from"], raw_edge.get("anchor"))):
            errors.append(error("invalid_anchor", f"typed edge {index}.anchor must resolve in its source document", edge=index, anchor=raw_edge.get("anchor")))

    for node_id, node in nodes.items():
        generated = node.get("generated_from")
        if node.get("kind") == "agent-pack" and (
            not isinstance(generated, dict)
            or not isinstance(generated.get("sources"), list)
            or not generated["sources"]
        ):
            errors.append(error("missing_generated_provenance", f"agent pack {node_id} requires non-empty generated provenance", node=node_id))
            continue
        if generated is None:
            continue
        if not isinstance(generated, dict):
            errors.append(error("invalid_manifest", f"{node_id}.generated_from must be an object", node=node_id))
            continue
        sources = require_list(generated.get("sources"))
        source_hashes = generated.get("source_hashes")
        if sources is None or not isinstance(source_hashes, dict):
            errors.append(error("invalid_manifest", f"{node_id}.generated_from requires sources and source_hashes", node=node_id))
            continue
        for source_id in sources:
            source = nodes.get(source_id) if isinstance(source_id, str) else None
            declared_hash = source_hashes.get(source_id) if isinstance(source_id, str) else None
            source_path = relative_file(root, source.get("path")) if source else None
            if source_path is None or not isinstance(declared_hash, str):
                errors.append(error("stale_generated_artifact", f"{node_id} has an unresolved generated source", node=node_id, source=source_id))
                continue
            if source.get("authority_level") == "A5":
                errors.append(error("forbidden_generated_source", f"{node_id} cannot derive agent guidance from A5 source {source_id}", node=node_id, source=source_id))
                continue
            actual_hash = hashlib.sha256(source_path.read_bytes()).hexdigest()
            if declared_hash != actual_hash:
                errors.append(error("stale_generated_artifact", f"{node_id} source hash does not match {source_id}", node=node_id, source=source_id))
    return errors


def validate_reference_frontmatter(root: Path, manifest_path: Path) -> list[dict[str, object]]:
    """Run SPEC-071 checks for manifest-listed reference pages without merging schemas."""
    try:
        data = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return []
    raw_nodes = data.get("nodes") if isinstance(data, dict) else None
    if not isinstance(raw_nodes, list):
        return []
    module_path = Path(__file__).resolve().parents[1] / "reference" / "check_frontmatter.py"
    spec = importlib.util.spec_from_file_location("spec071_frontmatter", module_path)
    if spec is None or spec.loader is None:
        return [error("reference_frontmatter_invalid", "unable to load the SPEC-071 frontmatter validator")]
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    reference_nodes = [node for node in raw_nodes if isinstance(node, dict) and node.get("kind") == "reference"]
    known_ids = {str(node.get("id")) for node in reference_nodes if node.get("id")}
    errors: list[dict[str, object]] = []
    for node in reference_nodes:
        path = relative_file(root, node.get("path"))
        if path is None:
            continue  # The primary manifest validation reports this deterministically.
        _page_id, messages = module.validate_file(root, path, known_ids)
        for message in messages:
            errors.append(error("reference_frontmatter_invalid", message, node=node.get("id"), path=path.relative_to(root).as_posix()))
    return errors


def promotion_manifest_data(manifest_path: Path) -> dict[str, object] | None:
    """Load a manifest for the strict promotion-only checks.

    Primary schema errors are reported by ``validate_manifest``.  This helper
    keeps the optional gate fail-closed without replacing that report.
    """
    try:
        data = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    return data if isinstance(data, dict) else None


def is_former_workflow_first_owner(root: Path, node: dict[str, object]) -> bool:
    """Return whether a node is one of the two superseded workflow documents."""
    path = normalized_relative_path(root, node.get("path"))
    node_id = node.get("id")
    return path in FORMER_WORKFLOW_FIRST_PATHS or node_id in {
        "history.fixture.formalization-boundary",
        "history.fixture.parser-to-core",
    }


def validate_promotion_completeness(root: Path, manifest_path: Path) -> list[dict[str, object]]:
    """Enforce the explicit TASK-1986 semantic-promotion closure.

    This is intentionally additive to the general sidecar validator.  It is
    selected by a CLI flag so ordinary inventory/metadata work is not mistaken
    for a completed semantic promotion.
    """
    data = promotion_manifest_data(manifest_path)
    if data is None:
        return [error("promotion_incomplete", "promotion completeness cannot be established from an unreadable manifest")]
    raw_nodes = data.get("nodes")
    if not isinstance(raw_nodes, list):
        return [error("promotion_incomplete", "promotion completeness requires a nodes list")]
    nodes = [node for node in raw_nodes if isinstance(node, dict)]
    node_by_id = {
        node_id: node for node in nodes
        if isinstance((node_id := node.get("id")), str) and node_id
    }
    errors: list[dict[str, object]] = []

    for subject in PROMOTION_SUBJECTS:
        owners = [
            node for node in nodes
            if node.get("authority_level") in {"A1", "A2"}
            and node.get("lifecycle") == "active"
            and isinstance(node.get("canonical_for"), list)
            and subject in node["canonical_for"]
        ]
        if len(owners) != 1:
            errors.append(error(
                "missing_required_canonical_owner",
                f"promotion subject {subject} requires exactly one active A1/A2 owner",
                subject=subject,
                owners=sorted(str(node.get("id")) for node in owners),
            ))

    for node in nodes:
        canonical_for = node.get("canonical_for")
        owns_target_subject = isinstance(canonical_for, list) and any(
            subject in PROMOTION_SUBJECTS for subject in canonical_for
        )
        if (
            owns_target_subject
            and node.get("authority_level") in {"A1", "A2"}
            and node.get("lifecycle") == "active"
            and is_former_workflow_first_owner(root, node)
        ):
            errors.append(error(
                "former_authority_not_reconciled",
                "former workflow-first document cannot own a promoted target semantic subject",
                node=node.get("id"),
            ))

    read_paths = data.get("default_read_paths")
    if not isinstance(read_paths, dict):
        errors.append(error("forbidden_default_read_path", "promotion completeness requires human and agent default read paths"))
    else:
        for audience in ("human", "agent"):
            path_ids = read_paths.get(audience)
            if not isinstance(path_ids, list):
                errors.append(error("forbidden_default_read_path", f"default {audience} read path must be a node-id list", audience=audience))
                continue
            for node_id in path_ids:
                node = node_by_id.get(node_id) if isinstance(node_id, str) else None
                if node is None or node.get("authority_level") == "A5":
                    errors.append(error(
                        "forbidden_default_read_path",
                        f"default {audience} read path cannot include A5 or unknown node {node_id!r}",
                        audience=audience,
                        node=node_id,
                    ))

    declared_trace_ids = {
        trace.get("id") for trace in data.get("trace_nodes", [])
        if isinstance(trace, dict) and isinstance(trace.get("id"), str) and trace["id"]
    }
    for node in nodes:
        if node.get("authority_level") not in {"A2", "A3"}:
            continue
        trace_ids = node.get("trace_nodes")
        if (
            not isinstance(trace_ids, list)
            or not trace_ids
            or any(not isinstance(trace_id, str) or not trace_id or trace_id not in declared_trace_ids for trace_id in trace_ids)
        ):
            errors.append(error(
                "missing_promotion_traceability",
                "A2/A3 promotion artifact requires non-empty stable trace IDs",
                node=node.get("id"),
            ))

    if errors:
        errors.append(error(
            "promotion_incomplete",
            "manifest does not meet TASK-1986 promotion-completeness requirements",
        ))
    return errors


def load_migration_artifact(root: Path, value: object) -> tuple[Path | None, dict[str, object] | None]:
    """Resolve and decode one sidecar-selected migration JSON artifact."""
    path = relative_file(root, value)
    if path is None:
        return None, None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return path, None
    return path, data if isinstance(data, dict) else None


def git_checkout_available(root: Path) -> bool:
    """Return whether ``root`` is a usable Git work tree.

    Migration fixtures intentionally run outside Git, where a descriptive
    revision label is sufficient.  Once the corpus is in a checkout, however,
    the archive claim must be independently verifiable against that checkout.
    """
    try:
        result = subprocess.run(
            ["git", "-C", str(root), "rev-parse", "--is-inside-work-tree"],
            check=False,
            capture_output=True,
            text=True,
        )
    except OSError:
        return False
    return result.returncode == 0 and result.stdout.strip() == "true"


def git_snapshot_contains(root: Path, revision: str, source_path: str) -> bool:
    """Verify a historical source is present in the named Git snapshot."""
    object_result = subprocess.run(
        ["git", "-C", str(root), "rev-parse", "--verify", f"{revision}^{{object}}"],
        check=False,
        capture_output=True,
        text=True,
    )
    if object_result.returncode != 0:
        return False
    tree_result = subprocess.run(
        ["git", "-C", str(root), "cat-file", "-e", f"{revision}:{source_path}"],
        check=False,
        capture_output=True,
        text=True,
    )
    return tree_result.returncode == 0


def retained_source_routes_to_replacement(root: Path, source_path: str, replacement_path: str) -> bool:
    """Check a retained historical page explicitly links readers to its replacement."""
    source = relative_file(root, source_path)
    if source is None:
        # A source removed from the working tree is still preserved through the
        # archive snapshot; only retained pages need a reader-facing tombstone.
        return True
    try:
        contents = source.read_text(encoding="utf-8")
    except OSError:
        return False
    if not re.search(r"\b(?:historical|tombstone)\b", contents, re.IGNORECASE):
        return False
    for target in re.findall(r"\[[^\]]+\]\(([^)]+)\)", contents):
        destination = target.split("#", 1)[0].strip()
        if not destination or re.match(r"[a-z][a-z0-9+.-]*:", destination, re.IGNORECASE):
            continue
        resolved = (source.parent / destination).resolve()
        try:
            if resolved.relative_to(root).as_posix() == replacement_path:
                return True
        except ValueError:
            continue
    return False


def validate_migration_completeness(root: Path, manifest_path: Path) -> list[dict[str, object]]:
    """Enforce TASK-1987's Git-backed historical-corpus migration closure.

    The sidecar remains the complete index of *in-scope* historical records.
    Archive, redirect, and retrieval artifacts are therefore checked against
    its A5 nodes rather than trying to infer authority from every file in the
    repository.
    """
    data = promotion_manifest_data(manifest_path)
    if data is None:
        return [error("migration_incomplete", "migration completeness cannot be established from an unreadable manifest")]
    raw_nodes = data.get("nodes")
    if not isinstance(raw_nodes, list):
        return [error("migration_incomplete", "migration completeness requires a nodes list")]
    nodes = [node for node in raw_nodes if isinstance(node, dict)]
    node_by_id = {
        node_id: node for node in nodes
        if isinstance((node_id := node.get("id")), str) and node_id
    }
    # A5 includes live audit and evidence records.  Migration is concerned
    # only with historical records that have actually left the active corpus.
    history_nodes = [
        node for node in nodes
        if node.get("authority_level") == "A5"
        and node.get("lifecycle") in {"superseded", "archived"}
    ]
    migration = data.get("migration")
    if not isinstance(migration, dict):
        return [error("migration_incomplete", "migration completeness requires a migration artifact mapping")]

    archive_path, archive = load_migration_artifact(root, migration.get("archive_manifest"))
    redirects_path, redirects = load_migration_artifact(root, migration.get("redirect_map"))
    benchmark_path, benchmark = load_migration_artifact(root, migration.get("retrieval_benchmark"))
    errors: list[dict[str, object]] = []
    if archive is None:
        errors.append(error("archive_provenance_incomplete", "migration archive manifest must be readable JSON", path=str(archive_path) if archive_path else migration.get("archive_manifest")))
    if redirects is None:
        errors.append(error("productive_route_not_canonical", "migration redirect map must be readable JSON", path=str(redirects_path) if redirects_path else migration.get("redirect_map")))
    if benchmark is None:
        errors.append(error("retrieval_quality_incomplete", "migration retrieval benchmark must be readable JSON", path=str(benchmark_path) if benchmark_path else migration.get("retrieval_benchmark")))
    if archive is None or redirects is None or benchmark is None:
        errors.append(error("migration_incomplete", "migration artifacts are missing or invalid"))
        return errors

    snapshot = archive.get("snapshot")
    if (
        archive.get("schema") != "canonical-corpus-archive/v1"
        or not isinstance(snapshot, dict)
        or not isinstance(snapshot.get("git_commit"), str)
        or not snapshot["git_commit"].strip()
        or "materialized_tree" in archive
    ):
        errors.append(error(
            "hand_maintained_snapshot",
            "archive preservation must name a Git snapshot and cannot materialize a shadow corpus",
        ))

    snapshot_revision = snapshot.get("git_commit") if isinstance(snapshot, dict) else None
    verify_snapshot = isinstance(snapshot_revision, str) and snapshot_revision.strip()
    if git_checkout_available(root) and not verify_snapshot:
        errors.append(error(
            "archive_snapshot_unverifiable",
            "a Git checkout requires a non-empty archive snapshot git_commit",
        ))

    raw_artifacts = archive.get("artifacts")
    artifacts = raw_artifacts if isinstance(raw_artifacts, list) else []
    artifacts_by_node = {
        artifact.get("node"): artifact
        for artifact in artifacts
        if isinstance(artifact, dict) and isinstance(artifact.get("node"), str) and artifact["node"]
    }
    replacements: dict[str, tuple[dict[str, object], dict[str, object]]] = {}
    required_archive_fields = ("source_path", "disposition", "original_revision", "reason", "replacement")
    for history in history_nodes:
        node_id = history.get("id")
        if not isinstance(node_id, str):
            continue
        artifact = artifacts_by_node.get(node_id)
        history_path = normalized_relative_path(root, history.get("path"))
        if (
            not isinstance(artifact, dict)
            or any(not isinstance(artifact.get(field), str) or not artifact[field].strip() for field in required_archive_fields)
            or artifact.get("source_path") != history_path
        ):
            errors.append(error(
                "archive_provenance_incomplete",
                "every sidecar-indexed A5 record requires archive provenance and a replacement",
                node=node_id,
            ))
            continue
        replacement_id = artifact["replacement"]
        replacement = node_by_id.get(replacement_id)
        if (
            replacement is None
            or replacement.get("authority_level") not in {"A1", "A2"}
            or replacement.get("lifecycle") != "active"
            or not isinstance(replacement.get("canonical_for"), list)
            or not replacement["canonical_for"]
        ):
            errors.append(error(
                "archive_provenance_incomplete",
                "historical archive replacement must name an active A1/A2 canonical owner",
                node=node_id,
                replacement=replacement_id,
            ))
            continue
        replacements[node_id] = (artifact, replacement)

        if git_checkout_available(root) and verify_snapshot and not git_snapshot_contains(
            root, snapshot_revision, artifact["source_path"]
        ):
            errors.append(error(
                "archive_snapshot_unverifiable",
                "archive snapshot must resolve and contain each historical source in this Git checkout",
                node=node_id,
                git_commit=snapshot_revision,
                source=artifact["source_path"],
            ))

    routes = redirects.get("routes") if redirects.get("schema") == "canonical-corpus-redirects/v1" else None
    if not isinstance(routes, list):
        routes = []
    raw_queries = benchmark.get("queries") if benchmark.get("schema") == "canonical-corpus-retrieval-benchmark/v1" else None
    queries = raw_queries if isinstance(raw_queries, list) else []
    for node_id, (artifact, replacement) in replacements.items():
        source_path = artifact["source_path"]
        replacement_path = normalized_relative_path(root, replacement.get("path"))
        if (
            artifact.get("disposition") in {"archive", "migrate", "migrated"}
            and isinstance(replacement_path, str)
            and not retained_source_routes_to_replacement(root, source_path, replacement_path)
        ):
            errors.append(error(
                "historical_routing_incomplete",
                "a retained archived or migrated source must contain historical/tombstone routing to its canonical replacement",
                node=node_id,
                source=source_path,
                replacement=replacement.get("id"),
            ))
        productive_links = artifact.get("productive_inbound_links")
        requires_route = isinstance(productive_links, list) and bool(productive_links)
        matching_routes = [
            route for route in routes
            if isinstance(route, dict)
            and route.get("from") == source_path
            and route.get("to") == replacement_path
            and route.get("kind") == "redirect"
        ]
        if requires_route and not matching_routes:
            errors.append(error(
                "productive_route_not_canonical",
                "a displaced source with productive inbound links must redirect to its active canonical replacement",
                node=node_id,
                source=source_path,
                replacement=replacement.get("id"),
            ))
        matching_queries = [
            query for query in queries
            if isinstance(query, dict)
            and isinstance(query.get("id"), str) and query["id"]
            and query.get("expected") == replacement.get("id")
            and isinstance(query.get("before"), list) and source_path in query["before"]
            and isinstance(query.get("after"), list) and replacement_path in query["after"]
        ]
        if not matching_queries:
            errors.append(error(
                "retrieval_quality_incomplete",
                "every historical replacement requires stable before/after retrieval evidence",
                node=node_id,
                source=source_path,
                replacement=replacement.get("id"),
            ))

    if errors:
        errors.append(error("migration_incomplete", "manifest does not meet TASK-1987 migration-completeness requirements"))
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", default=".", help="repository root")
    parser.add_argument("--manifest", required=True, help="canonical-corpus/v1 JSON sidecar")
    parser.add_argument("--format", choices=("json",), default="json")
    parser.add_argument("--check-reference-frontmatter", action="store_true", help="also report SPEC-071 incompatibilities for manifest-listed reference pages")
    parser.add_argument("--require-promotion-completeness", action="store_true", help="require TASK-1986's promoted semantic core")
    parser.add_argument("--require-migration-completeness", action="store_true", help="require TASK-1987's Git-backed historical migration")
    args = parser.parse_args()
    root = Path(args.root).resolve()
    manifest = Path(args.manifest).resolve()
    errors = validate_manifest(root, manifest)
    if args.check_reference_frontmatter:
        errors.extend(validate_reference_frontmatter(root, manifest))
    if args.require_promotion_completeness:
        errors.extend(validate_promotion_completeness(root, manifest))
    if args.require_migration_completeness:
        errors.extend(validate_migration_completeness(root, manifest))
    print(json.dumps({"schema": REPORT_SCHEMA, "errors": errors}, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
