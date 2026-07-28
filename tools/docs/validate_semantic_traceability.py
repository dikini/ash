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


GRAPH_SCHEMA = "semantic-traceability-graph/v2"
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
RUNTIME_REFINEMENT_FIELDS = {
    "status",
    "implementation",
    "implementation_fingerprint",
    "theorem",
    "artifact_hash",
    "anchor",
}


def issue(kind: str, message: str, **details: object) -> dict[str, object]:
    return {"kind": kind, "message": message, **details}


def stable_anchor(value: object) -> bool:
    """Accept a repository-relative path plus stable fragment, never a line number."""
    return isinstance(value, str) and ANCHOR.fullmatch(value) is not None


def nonempty_string(value: object) -> bool:
    """Return whether a metadata field is a non-blank string."""
    return isinstance(value, str) and bool(value.strip())


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
    if not nonempty_string(metadata.get("theorem")):
        errors.append(issue(
            "invalid_proof_theorem",
            "proof metadata must state a non-empty theorem",
            node=node_id,
        ))
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


def validate_proof_scopes(nodes: dict[str, dict[str, object]], errors: list[dict[str, object]]) -> None:
    """Require every proof scope to be complete and grounded in its model."""
    for node_id, node in nodes.items():
        if node.get("kind") != "proof":
            continue
        metadata = node.get("proof")
        proof_model_id = metadata.get("model") if isinstance(metadata, dict) else None
        proof_model = nodes.get(proof_model_id) if isinstance(proof_model_id, str) else None
        if proof_model is None:
            errors.append(issue(
                "unknown_proof_model",
                "proof metadata must name an existing model node",
                node=node_id,
                model=proof_model_id,
            ))
        elif proof_model.get("kind") != "model":
            errors.append(issue(
                "invalid_proof_model",
                "proof metadata must name a model node",
                node=node_id,
                model=proof_model_id,
            ))
        scope = metadata.get("scope") if isinstance(metadata, dict) else None
        if not isinstance(scope, dict):
            errors.append(issue(
                "incomplete_proof_scope",
                "proof metadata must declare a scope with its model and covered or excluded rules",
                node=node_id,
            ))
            continue
        model_id = scope.get("model") if isinstance(scope, dict) else None
        if not isinstance(model_id, str) or not model_id or model_id not in nodes:
            errors.append(issue(
                "unknown_proof_scope_model",
                "a declared proof scope must name an existing model node",
                node=node_id,
                model=model_id,
            ))
            continue
        if nodes[model_id].get("kind") != "model":
            errors.append(issue(
                "invalid_proof_scope_model",
                "a declared proof scope must name a model node, not a canonical rule or other evidence",
                node=node_id,
                model=model_id,
            ))
            continue
        if isinstance(proof_model_id, str) and proof_model_id and model_id != proof_model_id:
            errors.append(issue(
                "proof_scope_model_mismatch",
                "a declared proof scope must use the same model as the proof metadata",
                node=node_id,
                model=proof_model_id,
                scope_model=model_id,
            ))
            continue
        scope_has_rule_list = False
        for field in ("proven_rule_ids", "excluded_rule_ids"):
            references = scope.get(field)
            if references is None:
                continue
            if not isinstance(references, list) or not references or not all(isinstance(reference, str) and reference for reference in references):
                errors.append(issue(
                    "invalid_proof_scope_rule",
                    "proof scope rule references must be non-empty lists of stable node identifiers",
                    node=node_id,
                    field=field,
                ))
                continue
            scope_has_rule_list = True
            for reference in references:
                referenced = nodes.get(reference)
                if referenced is None:
                    errors.append(issue(
                        "unknown_proof_scope_rule",
                        "proof scope rule references must name declared graph evidence",
                        node=node_id,
                        field=field,
                        rule=reference,
                    ))
                elif referenced.get("kind") not in {"canonical-rule", "model"}:
                    errors.append(issue(
                        "invalid_proof_scope_rule",
                        "proof scope rule references must name canonical rules or model evidence",
                        node=node_id,
                        field=field,
                        rule=reference,
                    ))
        if not scope_has_rule_list:
            errors.append(issue(
                "incomplete_proof_scope",
                "proof scope must contain at least one non-empty proven_rule_ids or excluded_rule_ids list",
                node=node_id,
            ))


def runtime_refinement_bridge_issue(
    proof: dict[str, object],
    nodes: dict[str, dict[str, object]],
    edges: list[dict[str, object]],
) -> dict[str, object] | None:
    """Return one precise error when a proof lacks a production refinement bridge."""
    proof_id = proof.get("id")
    metadata = proof.get("proof")
    if not isinstance(metadata, dict):
        return issue(
            "invalid_runtime_refinement_bridge",
            "a runtime refinement bridge requires proof metadata",
            proof=proof_id,
        )
    refinement = metadata.get("runtime_refinement")
    if not isinstance(refinement, dict):
        return issue(
            "model_proof_missing_runtime_refinement_bridge",
            "a model proof cannot prove a canonical runtime rule without a verified implementation refinement bridge",
            proof=proof_id,
        )
    if set(refinement) != RUNTIME_REFINEMENT_FIELDS:
        return issue(
            "invalid_runtime_refinement_bridge",
            "runtime refinement metadata must use the complete controlled shape",
            proof=proof_id,
        )

    implementation_id = refinement.get("implementation")
    implementation = nodes.get(implementation_id) if isinstance(implementation_id, str) else None
    if not isinstance(implementation, dict) or implementation.get("kind") != "implementation":
        return issue(
            "runtime_refinement_unknown_implementation",
            "runtime refinement must name a declared implementation node",
            proof=proof_id,
            implementation=implementation_id,
        )
    if (
        refinement.get("status") != "verified"
        or not nonempty_string(refinement.get("theorem"))
        or not isinstance(refinement.get("artifact_hash"), str)
        or FINGERPRINT.fullmatch(refinement["artifact_hash"]) is None
        or not stable_anchor(refinement.get("anchor"))
    ):
        return issue(
            "invalid_runtime_refinement_bridge",
            "runtime refinement requires verified status, theorem, artifact hash, and stable anchor",
            proof=proof_id,
        )
    proof_fingerprint = metadata.get("implementation_fingerprint")
    bridge_fingerprint = refinement.get("implementation_fingerprint")
    implementation_fingerprint = implementation.get("source_fingerprint")
    if (
        not isinstance(bridge_fingerprint, str)
        or FINGERPRINT.fullmatch(bridge_fingerprint) is None
        or bridge_fingerprint != proof_fingerprint
        or bridge_fingerprint != implementation_fingerprint
    ):
        return issue(
            "runtime_refinement_fingerprint_mismatch",
            "runtime refinement, proof metadata, and implementation must name the same source fingerprint",
            proof=proof_id,
            implementation=implementation_id,
        )
    model_id = metadata.get("model")
    model = nodes.get(model_id) if isinstance(model_id, str) else None
    if not isinstance(model, dict) or model.get("kind") != "model":
        return issue(
            "invalid_runtime_refinement_bridge",
            "runtime refinement requires the proof to name a declared model",
            proof=proof_id,
            model=model_id,
        )
    if not any(
        edge.get("kind") == "refines"
        and edge.get("from") == implementation_id
        and edge.get("to") == model_id
        for edge in edges
    ):
        return issue(
            "runtime_refinement_bridge_missing_model_refinement_edge",
            "runtime refinement requires an implementation-to-model refines edge for the proof model",
            proof=proof_id,
            implementation=implementation_id,
            model=model_id,
        )
    return None
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
            elif kind == "proved_by":
                proof_node = nodes[target]
                proof = proof_node.get("proof")
                if node_statuses(proof_node) != ["proved"] or not isinstance(proof, dict) or proof.get("outcome") != "verified":
                    errors.append(issue(
                        "canonical_proved_by_not_verified",
                        "a canonical proved_by edge requires a proved proof with verified outcome",
                        index=index,
                        source=source,
                        target=target,
                    ))
                bridge_error = runtime_refinement_bridge_issue(proof_node, nodes, raw_edges)
                if bridge_error is not None:
                    errors.append({**bridge_error, "index": index, "source": source, "target": target})
                scope = proof.get("scope") if isinstance(proof, dict) else None
                proven_rule_ids = scope.get("proven_rule_ids") if isinstance(scope, dict) else None
                if (
                    not isinstance(proven_rule_ids, list)
                    or not all(isinstance(rule_id, str) and rule_id for rule_id in proven_rule_ids)
                    or source not in proven_rule_ids
                ):
                    errors.append(issue(
                        "proof_scope_mismatch",
                        "a canonical proved_by edge requires its exact source rule in the proof scope",
                        index=index,
                        source=source,
                        target=target,
                    ))
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
    parser.add_argument("--graph", type=Path, required=True, help="semantic-traceability-graph/v2 JSON file")
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
        errors.append(issue("invalid_schema", "graph schema must be semantic-traceability-graph/v2", value=payload.get("schema")))
    nodes = validate_nodes(payload, errors)
    validate_proof_scopes(nodes, errors)
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
