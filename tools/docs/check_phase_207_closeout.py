#!/usr/bin/env python3
"""Audit Phase 207 closeout evidence without widening semantic scope.

The default mode checks that the closeout inventory is internally auditable.  The
``--require-complete`` mode additionally requires the frozen callable-module
route owners to have complete implementation, evidence, and parity axes. Historical
partial prerequisite handoffs remain auditable but do not inflate the callable-route
completion gate. A partial phase therefore remains reportable without making ordinary
documentation gates fail before closeout is actually ready.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import sys
from typing import Any


REPORT_SCHEMA = "phase-207-closeout-report/v1"
MODULE_RULE_PREFIX = "SEM-MODULE-REALIZATION-"
REQUIRED_SCANNER_MARKERS = (
    "ModuleResolver::parse_module_decls",
    "strip_module_metadata_non_definition_lines",
    "strip_synthesized_metadata_non_definition_lines",
    "import_needs_more_lines",
    "extract_pub_mod_declarations",
    "extract_semicolon_snippets",
    "collect_module_exports",
    "path/string-keyed Engine module caches",
)
ALLOWED_IMPLEMENTATION = {"implemented", "partial", "not_implemented"}
ALLOWED_EVIDENCE = {"proved", "tested", "none"}
ALLOWED_PARITY = {"matches_spec", "below_spec"}
COMPLETION_OWNER_TASKS = frozenset(
    {
        "TASK-2073",
        "TASK-2069",
        "TASK-2063",
        "TASK-2064",
        "TASK-2065",
    }
)


def finding(code: str, message: str, **details: object) -> dict[str, object]:
    """Create one stable closeout finding."""

    return {"code": code, "message": message, **details}


def _is_module_record(record: object) -> bool:
    """Return whether a manifest record owns a Phase 207 module rule."""

    if not isinstance(record, dict):
        return False
    rule_ids = record.get("canonical_rule_ids")
    return isinstance(rule_ids, list) and any(
        isinstance(rule_id, str) and rule_id.startswith(MODULE_RULE_PREFIX)
        for rule_id in rule_ids
    )


def check_semantic_axes(records: list[object]) -> list[dict[str, object]]:
    """Require implementation, evidence, and parity axes on every module rule."""

    findings: list[dict[str, object]] = []
    for index, record in enumerate(records):
        if not _is_module_record(record):
            continue
        assert isinstance(record, dict)
        task = record.get("task", f"record-{index}")
        if record.get("implementation") not in ALLOWED_IMPLEMENTATION:
            findings.append(
                finding(
                    "missing_implementation_axis",
                    "module-rule record must declare an implementation axis",
                    task=task,
                )
            )
        evidence = record.get("evidence")
        if not isinstance(evidence, dict) or evidence.get("status") not in ALLOWED_EVIDENCE:
            findings.append(
                finding(
                    "missing_evidence_axis",
                    "module-rule record must declare an evidence axis",
                    task=task,
                )
            )
        if record.get("parity") not in ALLOWED_PARITY:
            findings.append(
                finding(
                    "missing_parity_axis",
                    "module-rule record must declare a parity axis",
                    task=task,
                )
            )
    return findings


def check_handoff_records(
    root: Path, records: list[object], active_tasks: list[object]
) -> list[dict[str, object]]:
    """Require every active Phase 207 task to have a task and handoff record."""

    record_by_task = {
        record.get("task"): record
        for record in records
        if isinstance(record, dict) and isinstance(record.get("task"), str)
    }
    phase_tasks = {
        record.get("task")
        for record in records
        if _is_module_record(record) and isinstance(record, dict)
    }
    findings: list[dict[str, object]] = []
    for task_value in active_tasks:
        if not isinstance(task_value, str) or task_value not in phase_tasks:
            continue
        record = record_by_task.get(task_value)
        if not isinstance(record, dict):
            findings.append(
                finding(
                    "missing_handoff_record",
                    "active task has no semantic handoff record",
                    task=task_value,
                )
            )
            continue
        task_file_value = record.get("task_file")
        if not isinstance(task_file_value, str):
            findings.append(
                finding(
                    "missing_task_file",
                    "semantic handoff record has no task file",
                    task=task_value,
                )
            )
            continue
        task_file = (root / task_file_value).resolve()
        if not task_file.is_file():
            findings.append(
                finding(
                    "missing_task_file",
                    "semantic handoff record points to a missing task file",
                    task=task_value,
                    path=task_file_value,
                )
            )
            continue
        text = task_file.read_text(encoding="utf-8")
        if "**Status:** Complete" not in text and "## Handoffs" not in text:
            findings.append(
                finding(
                    "missing_handoff_section",
                    "active task file must declare an explicit Handoffs section",
                    task=task_value,
                    path=task_file_value,
                )
            )
        for field in ("next_obligation", "non_goals"):
            if not record.get(field):
                findings.append(
                    finding(
                        "missing_handoff_field",
                        "semantic handoff record must retain its downstream boundary",
                        task=task_value,
                        field=field,
                    )
                )
    return findings


def check_scanner_inventory(audit_text: str) -> list[dict[str, object]]:
    """Require every known scanner seam to appear in the AUDIT-207 inventory."""

    findings: list[dict[str, object]] = []
    for marker in REQUIRED_SCANNER_MARKERS:
        if marker not in audit_text:
            findings.append(
                finding(
                    "missing_scanner_inventory",
                    "known scanner seam is absent from the AUDIT-207 denylist/allowlist",
                    marker=marker,
                )
            )
    if "TASK-2065 must run a repository-wide scanner denylist/allowlist check" not in audit_text:
        findings.append(
            finding(
                "missing_scanner_gate",
                "AUDIT-207 must retain the TASK-2065 scanner closeout gate",
            )
        )
    return findings


def check_reference_claims(root: Path) -> list[dict[str, object]]:
    """Ensure the module reference remains explicitly partial while Phase 207 is open."""

    path = root / "docs/reference/language/library/modules-and-imports.md"
    if not path.is_file():
        return [
            finding(
                "missing_module_reference",
                "the module reference used by Phase 207 is missing",
                path=str(path.relative_to(root)),
            )
        ]
    text = path.read_text(encoding="utf-8")
    frontmatter = re.search(r"(?ms)^---\s*\n(.*?)\n---\s*\n", text)
    status = re.search(r"(?m)^status:\s*(\S+)\s*$", frontmatter.group(1) if frontmatter else "")
    findings: list[dict[str, object]] = []
    if status is None or status.group(1) != "partial":
        findings.append(
            finding(
                "reference_status_overclaims",
                "module reference must remain partial until Phase 207 closeout",
                path=str(path.relative_to(root)),
            )
        )
    if "below_spec" not in text:
        findings.append(
            finding(
                "reference_missing_boundary",
                "module reference must retain an explicit below-spec boundary",
                path=str(path.relative_to(root)),
            )
        )
    return findings


def _completion_findings(
    root: Path, records: list[object], plan_text: str
) -> list[dict[str, object]]:
    """Report blockers owned by the frozen callable-route completion tasks."""

    findings: list[dict[str, object]] = []
    for record in records:
        if not _is_module_record(record):
            continue
        assert isinstance(record, dict)
        task = record.get("task", "unknown-task")
        if task not in COMPLETION_OWNER_TASKS:
            continue
        if record.get("implementation") != "implemented":
            findings.append(
                finding(
                    "incomplete_implementation",
                    "module rule is not fully implemented",
                    task=task,
                    value=record.get("implementation"),
                )
            )
        evidence = record.get("evidence")
        if not isinstance(evidence, dict) or evidence.get("status") not in {"proved", "tested"}:
            findings.append(
                finding(
                    "incomplete_evidence",
                    "module rule lacks tested or proved closeout evidence",
                    task=task,
                    value=evidence.get("status") if isinstance(evidence, dict) else None,
                )
            )
        if record.get("parity") != "matches_spec":
            findings.append(
                finding(
                    "incomplete_parity",
                    "module rule does not yet match the target specification",
                    task=task,
                    value=record.get("parity"),
                )
            )
    if not re.search(r"(?m)^status:\s*complete\s*$", plan_text):
        findings.append(
            finding(
                "phase_still_in_progress",
                "PLAN-207 is not marked complete",
            )
        )
    return findings


def audit_repository(root: Path, require_complete: bool = False) -> dict[str, Any]:
    """Return the closeout report for *root*."""

    manifest_path = root / "docs/plan/semantic-task-records.json"
    plan_path = root / "docs/plan/PLAN-207-COMPLETE-MODULE-REALIZATION.md"
    audit_path = root / "docs/plan/audits/AUDIT-207-module-realization-seams.md"
    payload = json.loads(manifest_path.read_text(encoding="utf-8"))
    records = payload.get("records", [])
    active_tasks = payload.get("active_tasks", [])
    if not isinstance(records, list):
        records = []
    if not isinstance(active_tasks, list):
        active_tasks = []
    contract_findings = [
        *check_semantic_axes(records),
        *check_handoff_records(root, records, active_tasks),
        *check_scanner_inventory(audit_path.read_text(encoding="utf-8")),
        *check_reference_claims(root),
    ]
    completion_findings = _completion_findings(
        root, records, plan_path.read_text(encoding="utf-8")
    )
    return {
        "schema": REPORT_SCHEMA,
        "ready": not contract_findings and not completion_findings,
        "contract_findings": contract_findings,
        "completion_findings": completion_findings,
        "require_complete": require_complete,
    }


def main(argv: list[str] | None = None) -> int:
    """Run the closeout audit and emit one machine-readable report."""

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--require-complete", action="store_true")
    args = parser.parse_args(argv)
    report = audit_repository(args.root.resolve(), args.require_complete)
    print(json.dumps(report, sort_keys=True))
    if report["contract_findings"]:
        return 1
    return 1 if args.require_complete and report["completion_findings"] else 0


if __name__ == "__main__":
    sys.exit(main())
