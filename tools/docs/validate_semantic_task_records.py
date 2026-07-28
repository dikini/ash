#!/usr/bin/env python3
"""Fail-closed validation for TASK-2028 semantic workflow records.

The checked-in manifest is deliberately the machine-readable authority for
workflow conformance.  This validator checks only the record and its durable
links; it never executes a declared verification command.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import shlex
import string
import subprocess
import sys
from typing import Any


MANIFEST_SCHEMA = "semantic-task-records/v2"
TRACEABILITY_SCHEMA = "semantic-traceability-graph/v2"
REPORT_SCHEMA = "semantic-task-record-validation-report/v1"

LAYER_NAMES = ("type", "core", "cps", "admission_runtime", "verification")
LAYER_STATUSES = {"implemented", "partial", "not_implemented", "not_applicable"}
IMPLEMENTATION_STATUSES = {"implemented", "partial", "not_implemented"}
EVIDENCE_STATUSES = {"proved", "tested", "none"}
PARITY_STATUSES = {"matches_spec", "below_spec"}
SHELL_CONTROL = re.compile(r"[;&|><`$]")
HEADING = re.compile(r"^( {0,3})(#{1,6})[ \t]+(.+?)[ \t]*$", re.MULTILINE)
MARKDOWN_LINK = re.compile(r"\[[^\]]*\]\(([^)]+)\)")
TRACEABILITY_ANCHOR = re.compile(r"^(?!/)(?!.*(?:^|/)\.\.(?:/|$))[A-Za-z0-9._/-]+#[A-Za-z][A-Za-z0-9._-]*$")
TRACEABILITY_FINGERPRINT = re.compile(r"^sha256:[A-Za-z0-9._-]+$")
RUNTIME_REFINEMENT_FIELDS = {
    "status",
    "implementation",
    "implementation_fingerprint",
    "theorem",
    "artifact_hash",
    "anchor",
}

MANIFEST_FIELDS = {"schema", "active_scope", "active_tasks", "records"}
RECORD_FIELDS = {
    "task",
    "task_file",
    "coverage_map",
    "canonical_rule_ids",
    "implementation",
    "layers",
    "evidence",
    "parity",
    "missing_spec_clauses",
    "non_goals",
    "next_obligation",
    "verification",
}
LAYER_FIELDS = set(LAYER_NAMES)
EVIDENCE_FIELDS = {"status", "positive", "negative", "mutation", "parity", "proofs"}
PARITY_FIELDS = {
    "covered": {"status", "evidence"},
    "not_applicable": {"status", "rationale"},
}
ACTIVE_SCOPE_FIELDS = {"kind", "tasks"}
TASK_1988_FOLLOWUPS = {
    "TASK-439",
    "TASK-2001",
    "TASK-2002",
    "TASK-2003",
    "TASK-2004",
    "TASK-2005",
    "TASK-2008",
    "TASK-2013",
    "TASK-2014",
}
TASK_2031_PREREQUISITE_SCOPE = TASK_1988_FOLLOWUPS | {"TASK-2031"}
TASK_2032_INTEGRATION_SCOPE = TASK_2031_PREREQUISITE_SCOPE | {"TASK-2032"}
TASK_2035_CONTRACT_SCOPE = TASK_2032_INTEGRATION_SCOPE | {"TASK-2035"}
TASK_2037_ENGINE_CPS_SCOPE = TASK_2035_CONTRACT_SCOPE | {"TASK-2037"}
TASK_2038_ASH_TEST_SCOPE = TASK_2037_ENGINE_CPS_SCOPE | {"TASK-2038"}
TASK_2039_REPL_SCOPE = TASK_2038_ASH_TEST_SCOPE | {"TASK-2039"}
TASK_2042_DAEMON_SCOPE = TASK_2039_REPL_SCOPE | {"TASK-2042"}
# Closed semantic handoffs remain in the manifest after completion so later
# implementation tasks retain their checked authority boundaries.
# This is deliberately a closed allowlist: all other active records must keep
# the normal in-progress lifecycle.
# TASK-2038, TASK-2039, and TASK-2042 are closed for their selected Engine client routes;
# their remaining partial/below-spec obligations stay owned by TASK-2040 and
# TASK-2041.
CLOSED_SEMANTIC_HANDOFF_TASKS = frozenset(
    {
        "TASK-2031",
        "TASK-2032",
        "TASK-2035",
        "TASK-2037",
        "TASK-2038",
        "TASK-2039",
        "TASK-2042",
    }
)
TASK_2031_DOCUMENTATION_CONTRACT_COMMAND = "python3 -m unittest tools.docs.tests.test_validate_ash_cps_calculus"
TASK_2035_DOCUMENTATION_CONTRACT_COMMAND = (
    "python3 -m unittest tools.docs.tests.test_task_2035_semantic_task_record"
)

# TASK-2028 starts with the smallest command policy needed by its task records.
# Adding a command requires a validator test and TASK-owned evidence.
HELP = "Use --root PATH --manifest PATH to validate a semantic-task records manifest."


def issue(kind: str, message: str, **details: object) -> dict[str, object]:
    """Create one stable, machine-readable validation issue."""
    return {"kind": kind, "message": message, **details}


def nonempty_string(value: object) -> bool:
    """Return whether *value* is a non-blank string."""
    return isinstance(value, str) and bool(value.strip())


def relative_path(root: Path, value: object) -> Path | None:
    """Resolve an existing repository-relative path without traversal escapes."""
    if not nonempty_string(value):
        return None
    candidate = Path(value)
    if candidate.is_absolute() or ".." in candidate.parts:
        return None
    resolved_root = root.resolve()
    resolved = (root / candidate).resolve()
    try:
        resolved.relative_to(resolved_root)
    except ValueError:
        return None
    return resolved


def markdown_slug(heading: str) -> str:
    """Produce the GitHub-style fragment used by the project's Markdown links."""
    text = heading.strip().rstrip("#").strip().lower()
    punctuation = string.punctuation.replace("-", "")
    text = text.translate(str.maketrans("", "", punctuation))
    return re.sub(r"[\s-]+", "-", text).strip("-")


def markdown_sections(text: str) -> dict[str, str]:
    """Return stable fragments and their Markdown sections from *text*."""
    matches = list(HEADING.finditer(text))
    seen: dict[str, int] = {}
    sections: dict[str, str] = {}
    for index, match in enumerate(matches):
        base = markdown_slug(match.group(3))
        if not base:
            continue
        count = seen.get(base, 0)
        seen[base] = count + 1
        fragment = base if count == 0 else f"{base}-{count}"
        level = len(match.group(2))
        end = len(text)
        for later in matches[index + 1 :]:
            if len(later.group(2)) <= level:
                end = later.start()
                break
        sections[fragment] = text[match.start() : end]
    return sections


def string_list(value: object) -> bool:
    """Return whether *value* is a non-empty list of non-blank strings."""
    return (
        isinstance(value, list)
        and bool(value)
        and all(nonempty_string(item) for item in value)
    )


def reject_unknown_fields(
    value: object,
    allowed: set[str],
    kind: str,
    errors: list[dict[str, object]],
    **details: object,
) -> None:
    """Reject fields outside the closed v2 schema object at this level."""
    if not isinstance(value, dict):
        return
    for field in sorted(set(value) - allowed):
        errors.append(
            issue(kind, "schema v2 does not permit this field", field=field, **details)
        )


def validate_parity(value: object, errors: list[dict[str, object]], index: int) -> bool:
    """Accept one closed-schema parity object and reject unowned metadata."""
    if not isinstance(value, dict):
        return False
    status = value.get("status")
    allowed = PARITY_FIELDS.get(status)
    if allowed is None:
        return False
    reject_unknown_fields(value, allowed, "unknown_parity_field", errors, index=index)
    if status == "covered":
        return string_list(value.get("evidence"))
    if status == "not_applicable":
        return nonempty_string(value.get("rationale"))
    return False


def token_has_unsafe_path(token: str) -> bool:
    """Reject absolute or parent-relative path tokens before they reach a shell."""
    path = Path(token)
    return path.is_absolute() or ".." in path.parts or token.startswith("~")


def allowed_verification_command(command: object) -> bool:
    """Recognize the deliberately small, shell-free verification command grammar."""
    if (
        not nonempty_string(command)
        or SHELL_CONTROL.search(command) is not None
        or any(ord(character) < 32 for character in command)
        or any(character in command for character in "'\\\"")
    ):
        return False
    try:
        tokens = shlex.split(command, posix=True)
    except ValueError:
        return False
    if not tokens or any(token_has_unsafe_path(token) for token in tokens):
        return False

    executable = tokens[0]
    if executable == "cargo":
        return (
            len(tokens) == 6
            and tokens[1] == "test"
            and tokens[2] == "-p"
            and nonempty_string(tokens[3])
            and not tokens[3].startswith("-")
            and tokens[4] == "--test"
            and nonempty_string(tokens[5])
            and not tokens[5].startswith("-")
        )
    if executable == "python3":
        return command in {
            TASK_2031_DOCUMENTATION_CONTRACT_COMMAND,
            TASK_2035_DOCUMENTATION_CONTRACT_COMMAND,
        }
    return False


def command_matches_task_integration_test(command: object, task: object) -> bool:
    """Return whether a controlled focused verification target is task-owned."""
    if not allowed_verification_command(command) or not nonempty_string(task):
        return False
    if command == TASK_2031_DOCUMENTATION_CONTRACT_COMMAND:
        return task == "TASK-2031"
    if command == TASK_2035_DOCUMENTATION_CONTRACT_COMMAND:
        return task == "TASK-2035"
    task_number = task.removeprefix("TASK-")
    if task_number == task or not task_number.isdigit():
        return False
    tokens = shlex.split(command, posix=True)
    target = tokens[5]
    return target == f"task_{task_number}" or target.startswith(f"task_{task_number}_")


def load_json(path: Path) -> tuple[object | None, str | None]:
    """Read one UTF-8 JSON file without leaking an exception from the gate."""
    try:
        return json.loads(path.read_text(encoding="utf-8")), None
    except (OSError, json.JSONDecodeError) as error:
        return None, str(error)


def canonical_rules(
    root: Path, errors: list[dict[str, object]]
) -> tuple[set[str], dict[str, dict[str, object]], list[dict[str, object]]]:
    """Load canonical-rule identifiers and edge evidence from traceability."""
    graph_path = root / "docs/spec/SEMANTIC-TRACEABILITY.json"
    graph, load_error = load_json(graph_path)
    if load_error is not None or not isinstance(graph, dict):
        errors.append(
            issue(
                "invalid_traceability_graph",
                "semantic traceability graph must be a readable JSON object",
                path="docs/spec/SEMANTIC-TRACEABILITY.json",
                detail=load_error,
            )
        )
        return set(), {}, []
    if graph.get("schema") != TRACEABILITY_SCHEMA:
        errors.append(
            issue(
                "invalid_traceability_graph",
                "semantic traceability graph has an unsupported schema",
                value=graph.get("schema"),
            )
        )
        return set(), {}, []
    nodes = graph.get("nodes")
    if not isinstance(nodes, list):
        errors.append(
            issue(
                "invalid_traceability_graph",
                "semantic traceability graph nodes must be a list",
            )
        )
        return set(), {}, []
    edges = graph.get("edges")
    if not isinstance(edges, list) or not all(isinstance(edge, dict) for edge in edges):
        errors.append(
            issue(
                "invalid_traceability_graph",
                "semantic traceability graph edges must be a list of objects",
            )
        )
        return set(), {}, []
    nodes_by_id = {
        node["id"]
        : node
        for node in nodes
        if isinstance(node, dict) and isinstance(node.get("id"), str)
    }
    return (
        {
            node_id
            for node_id, node in nodes_by_id.items()
            if node.get("kind") == "canonical-rule"
        },
        nodes_by_id,
        edges,
    )


def task_links_manifest(task_path: Path, manifest_path: Path, text: str) -> bool:
    """Return whether task prose explicitly links to its checked-in manifest."""
    for match in MARKDOWN_LINK.finditer(text):
        target = match.group(1).split("#", 1)[0]
        if not target:
            continue
        try:
            if (task_path.parent / target).resolve() == manifest_path.resolve():
                return True
        except OSError:
            continue
    return False


def task_links_coverage_map(task_path: Path, root: Path, coverage_map: object, text: str) -> bool:
    """Return whether task prose links to its exact coverage-map fragment."""
    if not nonempty_string(coverage_map):
        return False
    path_text, separator, fragment = coverage_map.partition("#")
    coverage_path = relative_path(root, path_text)
    if not separator or not nonempty_string(fragment) or coverage_path is None:
        return False
    for match in MARKDOWN_LINK.finditer(text):
        target_path, target_separator, target_fragment = match.group(1).partition("#")
        if (
            target_separator
            and target_fragment == fragment
            and target_path
            and (task_path.parent / target_path).resolve() == coverage_path.resolve()
        ):
            return True
    return False


def status_block_matches(text: str, record: dict[str, object]) -> bool:
    """Require the report axes and every declared missing clause in Markdown."""
    evidence = record.get("evidence")
    evidence_status = evidence.get("status") if isinstance(evidence, dict) else None
    for label, value in (
        ("Implementation", record.get("implementation")),
        ("Evidence", evidence_status),
        ("Parity", record.get("parity")),
    ):
        if not isinstance(value, str) or re.search(
            rf"(?m)^\s*\*\*{re.escape(label)}:\*\*\s*{re.escape(value)}\s*$",
            text,
        ) is None:
            return False
    clauses = record.get("missing_spec_clauses")
    if not isinstance(clauses, list):
        return False
    if re.search(r"(?m)^\s*\*\*Missing target-spec clauses:\*\*", text) is None:
        return False
    return all(isinstance(clause, str) and clause in text for clause in clauses)


def validate_task_file(
    root: Path,
    manifest_path: Path,
    record: dict[str, object],
    errors: list[dict[str, object]],
    index: int,
) -> tuple[Path | None, str | None]:
    """Require a task file that declares the record identity and report axes."""
    value = record.get("task_file")
    if not nonempty_string(value):
        errors.append(issue("missing_task_file_link", "record requires a task_file link", index=index))
        return None, None
    path = relative_path(root, value)
    if path is None or path.suffix.lower() != ".md" or not path.is_file():
        errors.append(
            issue(
                "invalid_task_file_link",
                "task_file must name an existing repository-relative Markdown file",
                index=index,
                path=value,
            )
        )
        return None, None
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as error:
        errors.append(
            issue("invalid_task_file_link", "task_file could not be read", index=index, path=value, detail=str(error))
        )
        return None, None

    task = record.get("task")
    heading = re.search(r"(?m)^#\s+(.+?)\s*$", text)
    if not nonempty_string(task) or heading is None or not re.fullmatch(
        rf"{re.escape(task)}:\s+.+", heading.group(1)
    ):
        errors.append(
            issue("task_heading_mismatch", "task_file H1 must identify the record task", index=index, task=task)
        )
    try:
        manifest_relative = manifest_path.resolve().relative_to(root.resolve()).as_posix()
    except ValueError:
        manifest_relative = ""
    if not manifest_relative or not task_links_manifest(path, manifest_path, text):
        errors.append(
            issue("missing_task_manifest_link", "task_file must link to its repository manifest", index=index, task=task)
        )
    if not task_links_coverage_map(path, root, record.get("coverage_map"), text):
        errors.append(
            issue(
                "missing_task_coverage_map_link",
                "task_file must link to its exact coverage-map fragment",
                index=index,
                task=task,
            )
        )
    if not status_block_matches(text, record):
        errors.append(
            issue(
                "task_target_spec_status_block_mismatch",
                "task_file must declare the record implementation, evidence, parity, and missing-clause block",
                index=index,
                task=task,
            )
        )
    required_status = "Complete" if task in CLOSED_SEMANTIC_HANDOFF_TASKS else "In progress"
    status_match = re.search(r"(?m)^\s*\*\*Status:\*\*\s*(In progress|Complete)(?=\s|$)", text)
    observed_status = status_match.group(1) if status_match is not None else None
    if observed_status != required_status:
        errors.append(
            issue(
                "active_task_status_mismatch",
                "task_file must declare the lifecycle status required by its semantic record",
                index=index,
                task=task,
                expected_status=required_status,
                observed_status=observed_status,
            )
        )
    return path, text


def section_links_to_task(section: str, coverage_path: Path, task_path: Path) -> bool:
    """Return whether the coverage section contains a Markdown link to its task."""
    for match in MARKDOWN_LINK.finditer(section):
        target = match.group(1).split("#", 1)[0]
        if target and (coverage_path.parent / target).resolve() == task_path.resolve():
            return True
    return False


def validate_coverage_map(root: Path, record: dict[str, object], errors: list[dict[str, object]], index: int) -> None:
    """Require a coverage-map section summarizing this task's declared layers."""
    value = record.get("coverage_map")
    if not nonempty_string(value):
        errors.append(issue("missing_coverage_map_link", "record requires a coverage_map link", index=index))
        return
    path_text, separator, fragment = value.partition("#")
    path = relative_path(root, path_text)
    if (
        not separator
        or not nonempty_string(fragment)
        or path is None
        or path.suffix.lower() != ".md"
        or not path.is_file()
    ):
        errors.append(
            issue(
                "invalid_coverage_map_link",
                "coverage_map must name an existing repository-relative Markdown file and heading fragment",
                index=index,
                path=value,
            )
        )
        return
    try:
        sections = markdown_sections(path.read_text(encoding="utf-8"))
    except OSError as error:
        errors.append(
            issue(
                "invalid_coverage_map_link",
                "coverage_map could not be read",
                index=index,
                path=value,
                detail=str(error),
            )
        )
        return
    section = sections.get(fragment)
    if section is None:
        errors.append(
            issue(
                "missing_coverage_map_heading",
                "coverage_map fragment must resolve to an existing Markdown heading",
                index=index,
                path=value,
            )
        )
        return
    task = record.get("task")
    if not nonempty_string(task) or task not in section:
        errors.append(
            issue("coverage_heading_missing_task", "coverage_map section must name the record task", index=index, task=task)
        )
    task_path = relative_path(root, record.get("task_file"))
    if task_path is None or not section_links_to_task(section, path, task_path):
        errors.append(
            issue("coverage_task_link_missing", "coverage_map section must link to its task_file", index=index, task=task)
        )
    declared_rules = record.get("canonical_rule_ids")
    if isinstance(declared_rules, list):
        for rule_id in declared_rules:
            if isinstance(rule_id, str) and rule_id not in section:
                errors.append(
                    issue("coverage_heading_missing_rule", "coverage_map section must name each canonical rule", index=index, rule=rule_id)
                )
    layers = record.get("layers")
    if isinstance(layers, dict):
        normalized_section = section.lower().replace("_", "-")
        for layer in LAYER_NAMES:
            status = layers.get(layer)
            if status not in LAYER_STATUSES:
                continue
            label = layer.replace("_", "-")
            expected = str(status).replace("_", "-")
            if re.search(
                rf"\b{re.escape(label)}\s*(?::|=)?\s+{re.escape(expected)}\b",
                normalized_section,
            ) is None:
                errors.append(
                    issue(
                        "coverage_layer_mismatch",
                        "coverage_map section must declare matching layer statuses",
                        index=index,
                        layer=layer,
                        status=status,
                    )
                )

    if not status_block_matches(section, record):
        errors.append(
            issue(
                "coverage_target_spec_status_block_mismatch",
                "coverage_map must declare the record implementation, evidence, parity, and missing-clause block",
                index=index,
            )
        )

    evidence = record.get("evidence")
    evidence_matches = isinstance(evidence, dict)
    if isinstance(evidence, dict):
        evidence_status = evidence.get("status")
        test_values = [evidence.get(kind) for kind in ("positive", "negative", "mutation")]
        proofs = evidence.get("proofs")
        tests_are_lists = all(
            isinstance(values, list) and all(nonempty_string(value) for value in values)
            for values in test_values
        )
        proofs_are_list = isinstance(proofs, list) and all(
            nonempty_string(proof) for proof in proofs
        )
        if evidence_status == "tested":
            evidence_matches &= all(string_list(values) for values in test_values)
            evidence_matches &= proofs == []
        elif evidence_status == "proved":
            evidence_matches &= tests_are_lists and string_list(proofs)
        elif evidence_status == "none":
            evidence_matches &= test_values == [[], [], []] and proofs == []
        else:
            evidence_matches = False
        if not tests_are_lists or not proofs_are_list:
            evidence_matches = False
        for values in (*test_values, proofs if isinstance(proofs, list) else []):
            if isinstance(values, list) and any(value not in section for value in values):
                evidence_matches = False
        parity = evidence.get("parity")
        if not isinstance(parity, dict):
            evidence_matches = False
        elif parity.get("status") == "covered":
            parity_values = parity.get("evidence")
            if not string_list(parity_values) or any(value not in section for value in parity_values):
                evidence_matches = False
        elif parity.get("status") == "not_applicable":
            if re.search(r"(?i)\bnot[ _-]?applicable\b", section) is None:
                evidence_matches = False
        else:
            evidence_matches = False
        if evidence_status == "none" and parity.get("status") != "not_applicable":
            evidence_matches = False
    if not evidence_matches:
        errors.append(
            issue("coverage_evidence_mismatch", "coverage_map evidence summary must match the record", index=index)
        )

    non_goals = record.get("non_goals")
    if not string_list(non_goals) or any(goal not in section for goal in non_goals):
        errors.append(
            issue("coverage_non_goals_missing", "coverage_map must repeat every declared non-goal", index=index)
        )
    next_obligation = record.get("next_obligation")
    if not nonempty_string(next_obligation) or next_obligation not in section:
        errors.append(
            issue("coverage_next_obligation_missing", "coverage_map must repeat the next obligation", index=index)
        )


def declared_evidence_ids(evidence: object) -> list[str]:
    """Collect test witness IDs from every evidence class that owns tests."""
    if not isinstance(evidence, dict):
        return []
    identifiers: list[str] = []
    for category in ("positive", "negative", "mutation"):
        values = evidence.get(category)
        if isinstance(values, list):
            identifiers.extend(value for value in values if isinstance(value, str))
    parity = evidence.get("parity")
    if isinstance(parity, dict) and parity.get("status") == "covered":
        values = parity.get("evidence")
        if isinstance(values, list):
            identifiers.extend(value for value in values if isinstance(value, str))
    return identifiers


def declared_proof_ids(evidence: object) -> list[str]:
    """Collect proof witness IDs declared by a proved record."""
    if not isinstance(evidence, dict):
        return []
    proofs = evidence.get("proofs")
    return [proof for proof in proofs if isinstance(proof, str)] if isinstance(proofs, list) else []


def valid_verified_proof_evidence(
    proof_node: dict[str, object],
    nodes_by_id: dict[str, dict[str, object]],
    traceability_edges: list[dict[str, object]],
) -> bool:
    """Require proof evidence to reach the named production implementation."""
    if proof_node.get("status") != ["proved"]:
        return False
    proof = proof_node.get("proof")
    if not isinstance(proof, dict) or proof.get("outcome") != "verified":
        return False
    if not nonempty_string(proof.get("theorem")):
        return False
    model_id = proof.get("model")
    scope = proof.get("scope")
    if (
        not isinstance(model_id, str)
        or nodes_by_id.get(model_id, {}).get("kind") != "model"
        or not isinstance(scope, dict)
        or scope.get("model") != model_id
    ):
        return False
    proven_rule_ids = scope.get("proven_rule_ids")
    if not string_list(proven_rule_ids):
        return False
    refinement = proof.get("runtime_refinement")
    if not isinstance(refinement, dict) or set(refinement) != RUNTIME_REFINEMENT_FIELDS:
        return False
    implementation_id = refinement.get("implementation")
    implementation = nodes_by_id.get(implementation_id) if isinstance(implementation_id, str) else None
    if not isinstance(implementation, dict) or implementation.get("kind") != "implementation":
        return False
    bridge_fingerprint = refinement.get("implementation_fingerprint")
    if (
        refinement.get("status") != "verified"
        or not isinstance(bridge_fingerprint, str)
        or TRACEABILITY_FINGERPRINT.fullmatch(bridge_fingerprint) is None
        or bridge_fingerprint != proof.get("implementation_fingerprint")
        or bridge_fingerprint != implementation.get("source_fingerprint")
        or not nonempty_string(refinement.get("theorem"))
        or not isinstance(refinement.get("artifact_hash"), str)
        or TRACEABILITY_FINGERPRINT.fullmatch(refinement["artifact_hash"]) is None
        or not isinstance(refinement.get("anchor"), str)
        or TRACEABILITY_ANCHOR.fullmatch(refinement["anchor"]) is None
    ):
        return False
    return any(
        edge.get("kind") == "refines"
        and edge.get("from") == implementation_id
        and edge.get("to") == model_id
        for edge in traceability_edges
    )


def validate_evidence_traceability(
    record: dict[str, object],
    index: int,
    nodes_by_id: dict[str, dict[str, object]],
    traceability_edges: list[dict[str, object]],
    errors: list[dict[str, object]],
) -> None:
    """Bind declared test and proof witnesses to rules and task headings."""
    task_file = record.get("task_file")
    rules = record.get("canonical_rule_ids")
    declared_rules = set(rules) if string_list(rules) else set()
    for evidence_id in declared_evidence_ids(record.get("evidence")):
        node = nodes_by_id.get(evidence_id)
        if node is None:
            errors.append(
                issue(
                    "unknown_evidence_node",
                    "declared evidence must name a traceability node",
                    index=index,
                    evidence=evidence_id,
                )
            )
            continue
        if node.get("kind") != "test":
            errors.append(
                issue(
                    "evidence_node_not_test",
                    "declared evidence must name a traceability test node",
                    index=index,
                    evidence=evidence_id,
                )
            )
            continue
        ownership_edges = [
            edge
            for edge in traceability_edges
            if edge.get("kind") == "tested_by"
            and edge.get("from") in declared_rules
            and edge.get("to") == evidence_id
        ]
        if not ownership_edges:
            errors.append(
                issue(
                    "missing_evidence_tested_by_edge",
                    "declared evidence requires a tested_by edge from a record canonical rule",
                    index=index,
                    evidence=evidence_id,
                )
            )
            continue
        if not any(
            isinstance(edge.get("anchor"), str)
            and isinstance(task_file, str)
            and edge["anchor"].startswith(f"{task_file}#")
            for edge in ownership_edges
        ):
            errors.append(
                issue(
                    "evidence_task_traceability_anchor_mismatch",
                    "evidence tested_by edge must anchor in the record task_file",
                    index=index,
                    evidence=evidence_id,
                )
            )

    for proof_id in declared_proof_ids(record.get("evidence")):
        node = nodes_by_id.get(proof_id)
        if node is None:
            errors.append(
                issue(
                    "unknown_proof_evidence_node",
                    "declared proof evidence must name a traceability node",
                    index=index,
                    evidence=proof_id,
                )
            )
            continue
        if node.get("kind") != "proof":
            errors.append(
                issue(
                    "evidence_node_not_proof",
                    "declared proof evidence must name a traceability proof node",
                    index=index,
                    evidence=proof_id,
                )
            )
            continue
        if not valid_verified_proof_evidence(node, nodes_by_id, traceability_edges):
            errors.append(
                issue(
                    "proved_evidence_not_verified",
                    "proved evidence requires a verified proof with a production refinement bridge",
                    index=index,
                    evidence=proof_id,
                )
            )
            continue
        ownership_edges = [
            edge
            for edge in traceability_edges
            if edge.get("kind") == "proved_by"
            and edge.get("from") in declared_rules
            and edge.get("to") == proof_id
        ]
        if not ownership_edges:
            errors.append(
                issue(
                    "missing_evidence_proved_by_edge",
                    "declared proof evidence requires a proved_by edge from a record canonical rule",
                    index=index,
                    evidence=proof_id,
                )
            )
            continue
        proof = node.get("proof")
        scope = proof.get("scope") if isinstance(proof, dict) else None
        proven_rule_ids = scope.get("proven_rule_ids") if isinstance(scope, dict) else None
        if not isinstance(proven_rule_ids, list) or not any(
            edge.get("from") in proven_rule_ids for edge in ownership_edges
        ):
            errors.append(
                issue(
                    "proved_evidence_not_verified",
                    "proved evidence requires a canonical proved_by edge within the proof scope",
                    index=index,
                    evidence=proof_id,
                )
            )
            continue
        if not any(
            isinstance(edge.get("anchor"), str)
            and isinstance(task_file, str)
            and edge["anchor"].startswith(f"{task_file}#")
            for edge in ownership_edges
        ):
            errors.append(
                issue(
                    "evidence_task_traceability_anchor_mismatch",
                    "evidence proved_by edge must anchor in the record task_file",
                    index=index,
                    evidence=proof_id,
                )
            )
            continue


def validate_record(
    root: Path,
    manifest_path: Path,
    record: object,
    index: int,
    rule_ids: set[str],
    nodes_by_id: dict[str, dict[str, object]],
    traceability_edges: list[dict[str, object]],
    errors: list[dict[str, object]],
) -> None:
    """Validate one semantic task workflow record."""
    if not isinstance(record, dict):
        errors.append(issue("invalid_record", "record must be an object", index=index))
        return

    reject_unknown_fields(record, RECORD_FIELDS, "unknown_record_field", errors, index=index)

    required = (
        "task",
        "canonical_rule_ids",
        "implementation",
        "layers",
        "evidence",
        "parity",
        "missing_spec_clauses",
        "non_goals",
        "next_obligation",
        "verification",
    )
    for name in required:
        if name not in record:
            errors.append(issue("missing_required_field", "record is missing a required workflow field", index=index, field=name))
    evidence_for_axes = record.get("evidence")
    if (
        "implementation" not in record
        or "parity" not in record
        or "missing_spec_clauses" not in record
        or not isinstance(evidence_for_axes, dict)
        or "status" not in evidence_for_axes
    ):
        errors.append(
            issue(
                "missing_target_spec_status_axes",
                "record must declare implementation, evidence.status, parity, and missing_spec_clauses",
                index=index,
            )
        )
    validate_task_file(root, manifest_path, record, errors, index)
    validate_coverage_map(root, record, errors, index)
    validate_evidence_traceability(record, index, nodes_by_id, traceability_edges, errors)

    if not nonempty_string(record.get("task")):
        errors.append(issue("invalid_task", "task must be a non-blank identifier", index=index))

    declared_rules = record.get("canonical_rule_ids")
    task_file = record.get("task_file")
    if not string_list(declared_rules):
        errors.append(
            issue(
                "invalid_canonical_rule_ids",
                "canonical_rule_ids must be a non-empty list of rule identifiers",
                index=index,
            )
        )
    else:
        for rule_id in declared_rules:
            if rule_id not in rule_ids:
                errors.append(
                    issue(
                        "unknown_canonical_rule",
                        "canonical_rule_ids must resolve to canonical-rule traceability nodes",
                        index=index,
                        rule=rule_id,
                    )
                )
                continue
            rule_edges = [edge for edge in traceability_edges if edge.get("from") == rule_id]
            if not rule_edges:
                errors.append(
                    issue(
                        "missing_task_traceability_edge",
                        "each canonical rule requires task-file anchored traceability evidence",
                        index=index,
                        rule=rule_id,
                    )
                )
            elif not any(
                isinstance(edge.get("anchor"), str)
                and isinstance(task_file, str)
                and edge["anchor"].startswith(f"{task_file}#")
                for edge in rule_edges
            ):
                errors.append(
                    issue(
                        "task_traceability_anchor_mismatch",
                        "canonical-rule traceability evidence must anchor in the record task_file",
                        index=index,
                        rule=rule_id,
                    )
                )

    if isinstance(task_file, str):
        task_path = relative_path(root, task_file)
        if task_path is not None and task_path.is_file():
            try:
                task_fragments = set(markdown_sections(task_path.read_text(encoding="utf-8")))
            except OSError:
                task_fragments = set()
            prefix = f"{task_file}#"
            for edge in traceability_edges:
                anchor = edge.get("anchor")
                if isinstance(anchor, str) and anchor.startswith(prefix):
                    fragment = anchor.removeprefix(prefix)
                    if fragment not in task_fragments:
                        errors.append(
                            issue(
                                "missing_task_traceability_anchor_heading",
                                "task-file traceability anchor must resolve to a Markdown heading",
                                index=index,
                                anchor=anchor,
                            )
                        )

    implementation = record.get("implementation")
    if implementation not in IMPLEMENTATION_STATUSES:
        errors.append(
            issue(
                "invalid_implementation_status",
                "implementation must state implemented, partial, or not_implemented",
                index=index,
                value=implementation,
            )
        )

    parity_status = record.get("parity")
    if parity_status == "exceeds_spec":
        errors.append(
            issue(
                "exceeds_spec_requires_spec_update",
                "behavior beyond the target specification requires a specification update before implementation",
                index=index,
            )
        )
    elif parity_status not in PARITY_STATUSES:
        errors.append(
            issue(
                "invalid_parity_status",
                "parity must state matches_spec or below_spec",
                index=index,
                value=parity_status,
            )
        )
    if implementation == "implemented" and parity_status == "below_spec":
        errors.append(
            issue(
                "implemented_below_spec",
                "implemented status cannot report target-spec parity below_spec",
                index=index,
            )
        )

    missing_spec_clauses = record.get("missing_spec_clauses")
    missing_clauses_valid = isinstance(missing_spec_clauses, list) and all(
        nonempty_string(clause) for clause in missing_spec_clauses
    )
    if not missing_clauses_valid or (
        (implementation == "partial" or parity_status == "below_spec")
        and not missing_spec_clauses
    ):
        errors.append(
            issue(
                "invalid_missing_spec_clauses",
                "partial or below-spec records require non-empty missing_spec_clauses",
                index=index,
            )
        )
    if implementation == "implemented" and missing_spec_clauses:
        errors.append(
            issue(
                "implemented_with_missing_target_spec_clauses",
                "implemented status cannot retain missing target-spec clauses",
                index=index,
            )
        )

    layers = record.get("layers")
    reject_unknown_fields(layers, LAYER_FIELDS, "unknown_layers_field", errors, index=index)
    if not isinstance(layers, dict) or any(layers.get(name) not in LAYER_STATUSES for name in LAYER_NAMES):
        errors.append(
            issue(
                "incomplete_layers",
                "layers must declare every controlled semantic workflow layer",
                index=index,
            )
        )

    evidence = record.get("evidence")
    reject_unknown_fields(evidence, EVIDENCE_FIELDS, "unknown_evidence_field", errors, index=index)
    parity_valid = (
        validate_parity(evidence.get("parity"), errors, index)
        if isinstance(evidence, dict)
        else False
    )
    evidence_status = evidence.get("status") if isinstance(evidence, dict) else None
    if evidence_status not in EVIDENCE_STATUSES:
        errors.append(
            issue(
                "invalid_evidence_status",
                "evidence.status must state proved, tested, or none",
                index=index,
                value=evidence_status,
            )
        )
    if implementation == "implemented" and evidence_status == "none":
        errors.append(
            issue(
                "implemented_without_evidence",
                "implemented records require proved or tested evidence",
                index=index,
            )
        )
    if parity_status == "matches_spec" and implementation != "implemented":
        errors.append(
            issue(
                "matches_spec_without_implementation",
                "matches_spec parity requires implementation to be implemented",
                index=index,
            )
        )
    evidence_valid = isinstance(evidence, dict) and evidence_status in EVIDENCE_STATUSES
    if isinstance(evidence, dict):
        test_values = [evidence.get(kind) for kind in ("positive", "negative", "mutation")]
        proofs = evidence.get("proofs")
        test_lists_are_valid = all(
            isinstance(values, list) and all(nonempty_string(value) for value in values)
            for values in test_values
        )
        proofs_are_valid = isinstance(proofs, list) and all(
            nonempty_string(proof) for proof in proofs
        )
        evidence_valid &= test_lists_are_valid and proofs_are_valid and parity_valid
        if evidence_status == "tested":
            evidence_valid &= all(string_list(values) for values in test_values) and proofs == []
        elif evidence_status == "proved":
            evidence_valid &= string_list(proofs)
        elif evidence_status == "none":
            parity = evidence.get("parity")
            evidence_valid &= (
                test_values == [[], [], []]
                and proofs == []
                and isinstance(parity, dict)
                and parity.get("status") == "not_applicable"
            )
    if not evidence_valid:
        errors.append(
            issue(
                "incomplete_evidence",
                "evidence status must own the matching test or proof identifiers",
                index=index,
            )
        )

    if not string_list(record.get("non_goals")):
        errors.append(issue("incomplete_non_goals", "non_goals must be a non-empty string list", index=index))
    if not nonempty_string(record.get("next_obligation")):
        errors.append(issue("incomplete_next_obligation", "next_obligation must be non-blank", index=index))

    verification = record.get("verification")
    if not string_list(verification):
        errors.append(
            issue(
                "invalid_verification_commands",
                "verification must be a non-empty list of commands",
                index=index,
            )
        )
    else:
        if any(not allowed_verification_command(command) for command in verification):
            errors.append(
                issue(
                    "unsafe_verification_command",
                    "verification commands must use the controlled direct-command grammar",
                    index=index,
                )
            )
        if not any(
            command_matches_task_integration_test(command, record.get("task"))
            for command in verification
        ):
            errors.append(
                issue(
                    "missing_task_owned_integration_test",
                    "verification requires one focused Cargo integration target for this task",
                    index=index,
                    task=record.get("task"),
                )
            )


def validate_active_scope(
    payload: dict[str, object], records: list[object], record_tasks: list[str], errors: list[dict[str, object]]
) -> None:
    """Check explicit ownership of the active semantic task set."""
    if "active_scope" not in payload:
        errors.append(issue("missing_active_scope", "manifest must declare an active_scope"))
        return
    scope = payload.get("active_scope")
    if not isinstance(scope, dict):
        errors.append(issue("invalid_active_scope", "active_scope must be an object"))
        return
    reject_unknown_fields(scope, ACTIVE_SCOPE_FIELDS, "unknown_active_scope_field", errors)
    kind = scope.get("kind")
    tasks = scope.get("tasks")
    if kind not in {
        "fixture",
        "task-1988-followups",
        "task-2031-prerequisite",
        "task-2032-integration",
        "task-2035-contract",
        "task-2037-engine-cps",
        "task-2038-ash-test",
        "task-2039-repl",
        "task-2042-daemon",
    } or not string_list(tasks) or len(set(tasks)) != len(tasks):
        errors.append(
            issue("invalid_active_scope", "active_scope must use a controlled kind and unique task list")
        )
        return
    expected_tasks = (
        TASK_1988_FOLLOWUPS if kind == "task-1988-followups"
        else TASK_2031_PREREQUISITE_SCOPE if kind == "task-2031-prerequisite"
        else TASK_2032_INTEGRATION_SCOPE if kind == "task-2032-integration"
        else TASK_2035_CONTRACT_SCOPE if kind == "task-2035-contract"
        else TASK_2037_ENGINE_CPS_SCOPE if kind == "task-2037-engine-cps"
        else TASK_2038_ASH_TEST_SCOPE if kind == "task-2038-ash-test"
        else TASK_2039_REPL_SCOPE if kind == "task-2039-repl"
        else TASK_2042_DAEMON_SCOPE if kind == "task-2042-daemon"
        else set(record_tasks)
    )
    if set(tasks) != expected_tasks or (
        kind in {
            "task-1988-followups",
            "task-2031-prerequisite",
            "task-2032-integration",
            "task-2035-contract",
            "task-2037-engine-cps",
            "task-2038-ash-test",
            "task-2039-repl",
            "task-2042-daemon",
        }
        and set(record_tasks) != expected_tasks
    ):
        errors.append(
            issue(
                "active_scope_task_set_mismatch",
                "active_scope task set does not match its controlled ownership scope",
                active_scope_tasks=sorted(tasks),
                expected_tasks=sorted(expected_tasks),
            )
        )


def validate(root: Path, manifest_path: Path) -> list[dict[str, object]]:
    """Return all deterministic errors for one semantic-task records manifest."""
    errors: list[dict[str, object]] = []
    payload, load_error = load_json(manifest_path)
    if load_error is not None or not isinstance(payload, dict):
        return [
            issue(
                "invalid_manifest",
                "semantic-task records manifest must be a readable JSON object",
                path=str(manifest_path),
                detail=load_error,
            )
        ]
    reject_unknown_fields(payload, MANIFEST_FIELDS, "unknown_manifest_field", errors)
    if payload.get("schema") != MANIFEST_SCHEMA:
        errors.append(
            issue(
                "invalid_schema",
                "semantic-task records manifest has an unsupported schema",
                value=payload.get("schema"),
            )
        )
    records = payload.get("records")
    if not isinstance(records, list) or not records:
        errors.append(issue("invalid_schema", "semantic-task records manifest requires a non-empty records list"))
        records = []
    rule_ids, nodes_by_id, traceability_edges = canonical_rules(root, errors)
    record_tasks: list[str] = []
    for index, record in enumerate(records):
        validate_record(
            root,
            manifest_path,
            record,
            index,
            rule_ids,
            nodes_by_id,
            traceability_edges,
            errors,
        )
        if isinstance(record, dict) and nonempty_string(record.get("task")):
            record_tasks.append(record["task"])
    seen_tasks: set[str] = set()
    for task in record_tasks:
        if task in seen_tasks:
            errors.append(issue("duplicate_task", "every task may have only one semantic workflow record", task=task))
        seen_tasks.add(task)

    validate_active_scope(payload, records, record_tasks, errors)

    active_tasks = payload.get("active_tasks")
    if "active_tasks" not in payload:
        errors.append(issue("missing_active_tasks", "manifest must declare the exact active task set"))
    elif not string_list(active_tasks) or len(set(active_tasks)) != len(active_tasks):
        errors.append(issue("invalid_active_tasks", "active_tasks must be a unique non-empty task list"))
    elif set(active_tasks) != set(record_tasks):
        errors.append(
            issue(
                "active_task_set_mismatch",
                "active_tasks must equal the set of record task identifiers",
                active_tasks=sorted(active_tasks),
                record_tasks=sorted(set(record_tasks)),
            )
        )
    errors.sort(key=lambda value: json.dumps(value, sort_keys=True))
    return errors


def run_self_test() -> list[dict[str, object]]:
    """Run the isolated validator contract suite without contaminating stdout."""
    root = Path(__file__).resolve().parents[2]
    result = subprocess.run(
        [
            sys.executable,
            "-m",
            "unittest",
            "tools.docs.tests.test_validate_semantic_task_records",
        ],
        cwd=root,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode == 0:
        return []
    return [
        issue(
            "self_test_failed",
            "semantic-task record validator contract suite failed",
            detail=result.stderr.strip() or result.stdout.strip(),
        )
    ]


def parse_args(
    argv: list[str],
) -> tuple[argparse.Namespace | None, list[dict[str, object]], bool]:
    """Parse the narrow CLI while preserving JSON reports for bad arguments."""
    if argv == ["--help"]:
        return None, [], True
    parser = argparse.ArgumentParser(description=__doc__, add_help=False)
    parser.add_argument("--root", type=Path)
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--self-test", action="store_true")
    try:
        args, unknown = parser.parse_known_args(argv)
    except SystemExit:
        return None, [issue("invalid_arguments", "unable to parse validator arguments")], False
    if unknown:
        return None, [issue("invalid_arguments", "unrecognized validator arguments", arguments=unknown)], False
    if args.self_test:
        if args.root is not None or args.manifest is not None:
            return None, [issue("invalid_arguments", "--self-test cannot be combined with --root or --manifest")], False
        return args, [], False
    if args.root is None or args.manifest is None:
        return None, [issue("invalid_arguments", "--root and --manifest are required")], False
    return args, [], False


def main(argv: list[str]) -> int:
    """Emit the stable JSON report and return non-zero on any validation error."""
    args, errors, help_requested = parse_args(argv)
    if help_requested:
        print(json.dumps({"schema": REPORT_SCHEMA, "errors": [], "help": HELP}, sort_keys=True))
        return 0
    if not errors and args is not None:
        if args.self_test:
            errors = run_self_test()
        else:
            root = args.root.resolve()
            manifest_path = args.manifest if args.manifest.is_absolute() else root / args.manifest
            if not root.is_dir():
                errors = [issue("invalid_root", "--root must name an existing directory", path=str(root))]
            else:
                errors = validate(root, manifest_path)
    report = {"schema": REPORT_SCHEMA, "errors": errors}
    print(json.dumps(report, sort_keys=True))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
