#!/usr/bin/env python3
"""Validate the evidence-oriented Phase 202 semantic traceability graph.

The graph deliberately records links between independently addressable
specification rules and implementation evidence.  It is not a second source
of semantic authority: a canonical rule remains authoritative at its stable
document anchor.  This tool is intentionally fail-closed so that a missing
edge cannot make a coverage gap look complete.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import sys
from typing import Any


GRAPH_SCHEMA = "semantic-traceability-graph/v1"
REPORT_SCHEMA = "semantic-traceability-validation-report/v1"
SPECIFICATION_COVERAGE_SCHEMA = "semantic-traceability-specification-coverage/v1"
IMPLEMENTATION_COVERAGE_SCHEMA = "semantic-traceability-implementation-coverage/v1"

NODE_PREFIXES = {
    "REQ", "VOCAB", "GRAM", "TYPE", "CORE", "LOWER", "SEM", "OBS", "CONF", "IMPL", "TEST", "PROOF",
}
NODE_KINDS = {"canonical-rule", "implementation", "test", "proof", "model", "disposition"}
EDGE_KINDS = {
    "defines", "refines", "requires", "lowers_to", "projects_to",
    "implemented_by", "tested_by", "proved_by", "assumes", "supersedes",
}
STATUS_FACTS = {
    "specified", "implemented", "tested", "modelled", "proved", "assumed", "deferred",
    "refuted", "not-applicable",
}
KIND_STATUSES = {
    "canonical-rule": {"specified"},
    "implementation": {"implemented", "deferred", "not-applicable"},
    "test": {"tested", "deferred", "not-applicable"},
    "proof": {"proved", "assumed", "deferred", "refuted", "not-applicable"},
    "model": {"modelled", "deferred", "not-applicable"},
    "disposition": {"assumed", "deferred", "refuted", "not-applicable"},
}
PREFIX_FOR_KIND = {
    "implementation": {"IMPL"}, "test": {"TEST"}, "proof": {"PROOF"},
    "model": {"REQ", "GRAM", "TYPE", "LOWER", "SEM", "OBS", "CONF"},
    "disposition": {"REQ", "GRAM", "TYPE", "LOWER", "SEM", "OBS", "CONF"},
}
CANONICAL_PREFIXES = NODE_PREFIXES - {"IMPL", "TEST", "PROOF"}
NODE_ID = re.compile(r"^(?P<prefix>[A-Z]+)-[A-Z0-9]+(?:-[A-Z0-9]+)*$")
ANCHOR = re.compile(r"^(?!/)(?!.*(?:^|/)\.\.(?:/|$))[A-Za-z0-9._/-]+#[A-Za-z][A-Za-z0-9._-]*$")
FINGERPRINT = re.compile(r"^sha256:[A-Za-z0-9._-]+$")


def issue(kind: str, message: str, **details: object) -> dict[str, object]:
    return {"kind": kind, "message": message, **details}


def stable_anchor(value: object) -> bool:
    """Accept a repository-relative path plus stable fragment, never a line number."""
    return isinstance(value, str) and ANCHOR.fullmatch(value) is not None


def node_prefix(node_id: object) -> str | None:
    if not isinstance(node_id, str):
        return None
    match = NODE_ID.fullmatch(node_id)
    return match.group("prefix") if match else None


def node_statuses(node: dict[str, object]) -> list[str] | None:
    statuses = node.get("status")
    if not isinstance(statuses, list) or len(statuses) != 1 or not all(isinstance(value, str) for value in statuses):
        return None
    return statuses


def validate_proof(node: dict[str, object], errors: list[dict[str, object]]) -> None:
    node_id = node.get("id")
    metadata = node.get("proof")
    required = {
        "provider", "tool", "tool_version", "options", "assumptions", "model",
        "implementation_revision", "implementation_fingerprint", "artifact_hash", "outcome",
    }
    if not isinstance(metadata, dict):
        errors.append(issue("invalid_proof_metadata", "proof node requires proof metadata", node=node_id))
        return
    missing = sorted(required - set(metadata))
    invalid = bool(missing)
    for key in ("provider", "tool", "tool_version", "model", "implementation_revision", "artifact_hash"):
        invalid |= not isinstance(metadata.get(key), str) or not metadata.get(key)
    invalid |= not isinstance(metadata.get("options"), list) or not all(isinstance(item, str) for item in metadata.get("options", []))
    invalid |= not isinstance(metadata.get("assumptions"), list) or not all(isinstance(item, str) for item in metadata.get("assumptions", []))
    invalid |= not isinstance(metadata.get("implementation_fingerprint"), str) or FINGERPRINT.fullmatch(metadata.get("implementation_fingerprint", "")) is None
    outcome = metadata.get("outcome")
    invalid |= outcome not in {"verified", "assumed", "deferred", "refuted", "not-applicable"}
    if invalid:
        errors.append(issue("invalid_proof_metadata", "proof metadata is incomplete or malformed", node=node_id, fields=missing))
    statuses = node_statuses(node) or []
    if statuses == ["proved"] and outcome != "verified":
        errors.append(issue("false_proof_status", "a proved status requires a verified proof outcome", node=node_id, outcome=outcome))
    expected = {"assumed": "assumed", "deferred": "deferred", "refuted": "refuted", "not-applicable": "not-applicable"}
    if statuses and statuses[0] in expected and outcome != expected[statuses[0]]:
        errors.append(issue("false_proof_status", "proof status must agree with its recorded outcome", node=node_id, outcome=outcome))


def validate_nodes(payload: dict[str, object], errors: list[dict[str, object]]) -> dict[str, dict[str, object]]:
    raw_nodes = payload.get("nodes")
    if not isinstance(raw_nodes, list):
        errors.append(issue("invalid_schema", "graph.nodes must be a list"))
        return {}
    nodes: dict[str, dict[str, object]] = {}
    for index, node in enumerate(raw_nodes):
        if not isinstance(node, dict):
            errors.append(issue("invalid_node", "node must be an object", index=index))
            continue
        node_id, kind = node.get("id"), node.get("kind")
        prefix = node_prefix(node_id)
        if prefix not in NODE_PREFIXES:
            errors.append(issue("invalid_node_id", "node id must use a stable Phase 202 namespace", index=index, node=node_id))
        elif isinstance(kind, str):
            if kind == "canonical-rule" and prefix not in CANONICAL_PREFIXES:
                errors.append(issue("invalid_node_id", "canonical rules cannot use evidence namespaces", node=node_id))
            elif kind in PREFIX_FOR_KIND and prefix not in PREFIX_FOR_KIND[kind]:
                errors.append(issue("invalid_node_id", "node namespace does not match its kind", node=node_id, kind=kind))
        if not isinstance(node_id, str):
            continue
        if node_id in nodes:
            errors.append(issue("duplicate_node_id", "node ids must be unique", node=node_id))
            continue
        nodes[node_id] = node
        if kind not in NODE_KINDS:
            errors.append(issue("invalid_node_kind", "node kind is not controlled", node=node_id, value=kind))
        statuses = node_statuses(node)
        if statuses is None or statuses[0] not in STATUS_FACTS or kind not in KIND_STATUSES or statuses[0] not in KIND_STATUSES[kind]:
            errors.append(issue("invalid_status_fact", "node status must be one allowed fact for its kind", node=node_id, value=node.get("status")))
        if not stable_anchor(node.get("anchor")):
            errors.append(issue("invalid_node_anchor", "node anchor must be a stable repo-relative fragment", node=node_id, value=node.get("anchor")))
        if kind == "implementation":
            if not isinstance(node.get("public_semantic", False), bool):
                errors.append(issue("invalid_implementation_metadata", "public_semantic must be boolean", node=node_id))
            if not isinstance(node.get("symbol"), str) or not node.get("symbol"):
                errors.append(issue("invalid_implementation_metadata", "implementation requires a stable symbol", node=node_id))
            fingerprint = node.get("source_fingerprint")
            if not isinstance(fingerprint, str) or FINGERPRINT.fullmatch(fingerprint) is None:
                errors.append(issue("invalid_implementation_metadata", "implementation requires a sha256 source fingerprint", node=node_id))
        if kind == "proof":
            validate_proof(node, errors)
    return nodes


def validate_edges(payload: dict[str, object], nodes: dict[str, dict[str, object]], errors: list[dict[str, object]]) -> list[dict[str, object]]:
    raw_edges = payload.get("edges")
    if not isinstance(raw_edges, list):
        errors.append(issue("invalid_schema", "graph.edges must be a list"))
        return []
    edges: list[dict[str, object]] = []
    endpoint_kinds = {"implemented_by": "implementation", "tested_by": "test", "proved_by": "proof"}
    for index, edge in enumerate(raw_edges):
        if not isinstance(edge, dict):
            errors.append(issue("invalid_edge", "edge must be an object", index=index))
            continue
        kind, source, target = edge.get("kind"), edge.get("from"), edge.get("to")
        if kind not in EDGE_KINDS:
            errors.append(issue("invalid_edge_kind", "edge kind is not controlled", index=index, value=kind))
        if not isinstance(source, str) or source not in nodes or not isinstance(target, str) or target not in nodes:
            errors.append(issue("dangling_edge", "edge endpoints must name declared nodes", index=index, source=source, target=target))
        if not stable_anchor(edge.get("anchor")):
            errors.append(issue("invalid_edge_anchor", "edge anchor must be a stable repo-relative fragment", index=index, value=edge.get("anchor")))
        if kind in endpoint_kinds and isinstance(source, str) and isinstance(target, str) and source in nodes and target in nodes:
            if nodes[source].get("kind") != "canonical-rule" or nodes[target].get("kind") != endpoint_kinds[kind]:
                errors.append(issue("invalid_edge_endpoint", "coverage edges must connect canonical rules to matching evidence", index=index, kind=kind, source=source, target=target))
        edges.append(edge)
    return edges


def validate_coverage(nodes: dict[str, dict[str, object]], edges: list[dict[str, object]], errors: list[dict[str, object]]) -> None:
    evidence_edges = {"implemented_by", "tested_by", "proved_by", "assumes"}
    outgoing: dict[str, list[dict[str, object]]] = {node_id: [] for node_id in nodes}
    incoming: dict[str, list[dict[str, object]]] = {node_id: [] for node_id in nodes}
    for edge in edges:
        source, target = edge.get("from"), edge.get("to")
        if isinstance(source, str) and source in outgoing:
            outgoing[source].append(edge)
        if isinstance(target, str) and target in incoming:
            incoming[target].append(edge)
    for node_id, node in nodes.items():
        if node.get("kind") == "canonical-rule":
            owned = any(edge.get("kind") in evidence_edges for edge in outgoing[node_id])
            if not owned:
                errors.append(issue("unowned_canonical_rule", "canonical rule has no implementation, test, proof, or owned disposition", node=node_id))
        if node.get("kind") == "implementation" and node.get("public_semantic") is True:
            owner = any(edge.get("kind") == "implemented_by" and nodes.get(edge.get("from"), {}).get("kind") == "canonical-rule" for edge in incoming[node_id])
            if not owner:
                errors.append(issue("orphan_public_semantic_implementation", "public semantic implementation lacks a canonical owner", node=node_id))


def coverage_reports(nodes: dict[str, dict[str, object]], edges: list[dict[str, object]]) -> tuple[dict[str, object], dict[str, object]]:
    by_source: dict[str, list[dict[str, object]]] = {node_id: [] for node_id in nodes}
    by_target: dict[str, list[dict[str, object]]] = {node_id: [] for node_id in nodes}
    for edge in edges:
        if isinstance(edge.get("from"), str) and edge["from"] in by_source:
            by_source[edge["from"]].append(edge)
        if isinstance(edge.get("to"), str) and edge["to"] in by_target:
            by_target[edge["to"]].append(edge)

    def targets(source: str, kind: str) -> list[str]:
        return sorted(str(edge["to"]) for edge in by_source[source] if edge.get("kind") == kind and isinstance(edge.get("to"), str))

    rules = []
    for node_id in sorted(node_id for node_id, node in nodes.items() if node.get("kind") == "canonical-rule"):
        rules.append({
            "rule": node_id,
            "implementations": targets(node_id, "implemented_by"),
            "tests": targets(node_id, "tested_by"),
            "proofs": targets(node_id, "proved_by"),
            "assumptions": targets(node_id, "assumes"),
        })
    implementations = []
    for node_id in sorted(node_id for node_id, node in nodes.items() if node.get("kind") == "implementation"):
        owners = sorted(str(edge["from"]) for edge in by_target[node_id] if edge.get("kind") == "implemented_by" and isinstance(edge.get("from"), str))
        # A type rule is the direct ownership boundary for a shared Core
        # helper.  Operational rules may still cite that helper as evidence in
        # the specification matrix, but do not turn one implementation into
        # two reverse-coverage owners.  This is intentionally a projection
        # for the implementation report; validation continues to regard every
        # canonical ``implemented_by`` edge as ownership evidence.
        type_owners = [owner for owner in owners if owner.startswith("TYPE-")]
        if type_owners:
            owners = type_owners
        implementations.append({"implementation": node_id, "owners": owners})
    return (
        {"schema": SPECIFICATION_COVERAGE_SCHEMA, "rules": rules},
        {"schema": IMPLEMENTATION_COVERAGE_SCHEMA, "implementations": implementations},
    )


def write_json(path: Path, value: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True, help="repository root used to resolve the graph")
    parser.add_argument("--graph", type=Path, required=True, help="semantic-traceability-graph/v1 JSON file")
    parser.add_argument("--reports-dir", type=Path, help="write deterministic coverage reports here when validation succeeds")
    parser.add_argument("--format", choices=("json",), default="json", help="stdout report format")
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    errors: list[dict[str, object]] = []
    graph_path = args.graph if args.graph.is_absolute() else args.root / args.graph
    try:
        payload: Any = json.loads(graph_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        errors.append(issue("invalid_graph", "unable to read graph JSON", path=str(graph_path), detail=str(exc)))
        payload = {}
    if not isinstance(payload, dict):
        errors.append(issue("invalid_schema", "graph root must be an object"))
        payload = {}
    if payload.get("schema") != GRAPH_SCHEMA:
        errors.append(issue("invalid_schema", "graph schema must be semantic-traceability-graph/v1", value=payload.get("schema")))
    nodes = validate_nodes(payload, errors)
    edges = validate_edges(payload, nodes, errors)
    validate_coverage(nodes, edges, errors)
    errors.sort(key=lambda value: (str(value.get("kind")), str(value.get("node", value.get("index", ""))), str(value.get("message"))))
    report = {"schema": REPORT_SCHEMA, "errors": errors}
    if not errors and args.reports_dir is not None:
        specification, implementation = coverage_reports(nodes, edges)
        write_json(args.reports_dir / "specification-coverage.json", specification)
        write_json(args.reports_dir / "implementation-coverage.json", implementation)
    print(json.dumps(report, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
