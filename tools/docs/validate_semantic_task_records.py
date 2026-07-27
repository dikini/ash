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


MANIFEST_SCHEMA = "semantic-task-records/v1"
TRACEABILITY_SCHEMA = "semantic-traceability-graph/v1"
REPORT_SCHEMA = "semantic-task-record-validation-report/v1"

LAYER_NAMES = ("type", "core", "cps", "admission_runtime", "verification")
LAYER_STATUSES = {"bounded", "general", "not_applicable"}
DOMAIN_STATUSES = {"bounded", "general"}
SHELL_CONTROL = re.compile(r"[;&|><`$]")
HEADING = re.compile(r"^( {0,3})(#{1,6})[ \t]+(.+?)[ \t]*$", re.MULTILINE)
MARKDOWN_LINK = re.compile(r"\[[^\]]*\]\(([^)]+)\)")

MANIFEST_FIELDS = {"schema", "active_scope", "active_tasks", "records"}
RECORD_FIELDS = {
    "task",
    "task_file",
    "coverage_map",
    "canonical_rule_ids",
    "domain",
    "layers",
    "evidence",
    "non_goals",
    "next_obligation",
    "verification",
}
DOMAIN_FIELDS = {"status", "description"}
LAYER_FIELDS = set(LAYER_NAMES)
EVIDENCE_FIELDS = {"positive", "negative", "mutation", "parity"}
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
# A prerequisite record remains in the manifest after its mathematical handoff
# closes so later implementation tasks retain its checked authority boundary.
# This is deliberately a closed allowlist: all other active records must keep
# the normal in-progress lifecycle.
CLOSED_PREREQUISITE_TASKS = frozenset({"TASK-2031"})
TASK_2031_DOCUMENTATION_CONTRACT_COMMAND = "python3 -m unittest tools.docs.tests.test_validate_ash_cps_calculus"

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
    """Reject fields outside the closed v1 schema object at this level."""
    if not isinstance(value, dict):
        return
    for field in sorted(set(value) - allowed):
        errors.append(
            issue(kind, "schema v1 does not permit this field", field=field, **details)
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
        return command == TASK_2031_DOCUMENTATION_CONTRACT_COMMAND
    return False


def command_matches_task_integration_test(command: object, task: object) -> bool:
    """Return whether a controlled focused verification target is task-owned."""
    if not allowed_verification_command(command) or not nonempty_string(task):
        return False
    if command == TASK_2031_DOCUMENTATION_CONTRACT_COMMAND:
        return task == "TASK-2031"
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


def task_declared_domain(text: str) -> str | None:
    """Read the explicit human-facing semantic domain declaration."""
    match = re.search(
        r"(?im)^\s*(?:\*\*)?Declared domain(?:\*\*)?\s*:\s*(?:\*\*)?\s*"
        r"(bounded|general)\b",
        text,
    )
    return match.group(1).lower() if match else None


def validate_task_file(
    root: Path,
    manifest_path: Path,
    record: dict[str, object],
    errors: list[dict[str, object]],
    index: int,
) -> tuple[Path | None, str | None]:
    """Require a task file that declares the record identity and domain."""
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
    domain = record.get("domain")
    status = domain.get("status") if isinstance(domain, dict) else None
    if task_declared_domain(text) != status:
        errors.append(
            issue("task_domain_mismatch", "task_file domain declaration must match the record", index=index, task=task)
        )
    required_status = "Complete" if task in CLOSED_PREREQUISITE_TASKS else "In progress"
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

    domain = record.get("domain")
    status = domain.get("status") if isinstance(domain, dict) else None
    if (
        status not in DOMAIN_STATUSES
        or re.search(
            rf"(?im)^\s*(?:\*\*)?Domain(?:\*\*)?\s*:\s*(?:\*\*)?\s*{re.escape(str(status))}\b",
            section,
        )
        is None
    ):
        errors.append(
            issue("coverage_domain_mismatch", "coverage_map domain declaration must match the record", index=index)
        )

    evidence = record.get("evidence")
    evidence_matches = isinstance(evidence, dict)
    if isinstance(evidence, dict):
        for evidence_kind in ("positive", "negative", "mutation"):
            values = evidence.get(evidence_kind)
            if not string_list(values) or any(value not in section for value in values):
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


def validate_evidence_traceability(
    record: dict[str, object],
    index: int,
    nodes_by_id: dict[str, dict[str, object]],
    traceability_edges: list[dict[str, object]],
    errors: list[dict[str, object]],
) -> None:
    """Bind each declared test witness to a declared rule and task heading."""
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
        "domain",
        "layers",
        "evidence",
        "non_goals",
        "next_obligation",
        "verification",
    )
    for name in required:
        if name not in record:
            errors.append(issue("missing_required_field", "record is missing a required workflow field", index=index, field=name))
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

    domain = record.get("domain")
    reject_unknown_fields(domain, DOMAIN_FIELDS, "unknown_domain_field", errors, index=index)
    if (
        not isinstance(domain, dict)
        or domain.get("status") not in DOMAIN_STATUSES
        or not nonempty_string(domain.get("description"))
    ):
        errors.append(
            issue(
                "incomplete_domain",
                "domain requires a bounded or general status and non-empty description",
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
    if (
        not isinstance(evidence, dict)
        or not string_list(evidence.get("positive"))
        or not string_list(evidence.get("negative"))
        or not string_list(evidence.get("mutation"))
        or not parity_valid
    ):
        errors.append(
            issue(
                "incomplete_evidence",
                "evidence requires positive, negative, mutation, and parity accountability",
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
    if kind not in {"fixture", "task-1988-followups", "task-2031-prerequisite"} or not string_list(tasks) or len(set(tasks)) != len(tasks):
        errors.append(
            issue("invalid_active_scope", "active_scope must use a controlled kind and unique task list")
        )
        return
    expected_tasks = (
        TASK_1988_FOLLOWUPS if kind == "task-1988-followups"
        else TASK_2031_PREREQUISITE_SCOPE if kind == "task-2031-prerequisite"
        else set(record_tasks)
    )
    if set(tasks) != expected_tasks or (
        kind in {"task-1988-followups", "task-2031-prerequisite"} and set(record_tasks) != expected_tasks
    ):
        errors.append(
            issue(
                "active_scope_task_set_mismatch",
                "active_scope task set does not match its controlled ownership scope",
                active_scope_tasks=sorted(tasks),
                expected_tasks=sorted(expected_tasks),
            )
        )
    if kind == "task-1988-followups":
        for index, record in enumerate(records):
            domain = record.get("domain") if isinstance(record, dict) else None
            if not isinstance(domain, dict) or domain.get("status") != "bounded":
                errors.append(
                    issue(
                        "task_1988_followups_domain_must_be_bounded",
                        "TASK-1988 follow-up records must remain explicitly bounded",
                        index=index,
                        task=record.get("task") if isinstance(record, dict) else None,
                    )
                )
    if kind == "task-2031-prerequisite":
        for index, record in enumerate(records):
            if not isinstance(record, dict):
                continue
            domain = record.get("domain")
            task = record.get("task")
            required_domain = "general" if task == "TASK-2031" else "bounded"
            if not isinstance(domain, dict) or domain.get("status") != required_domain:
                errors.append(
                    issue(
                        "task_2031_prerequisite_domain_mismatch",
                        "TASK-2031 must remain general while every inherited TASK-1988 follow-up remains bounded",
                        index=index,
                        task=task,
                        expected_domain=required_domain,
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
