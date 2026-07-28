#!/usr/bin/env python3
"""Repository contract for TASK-2040's activated Engine-only removal record."""
from __future__ import annotations

import json
import subprocess
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
TOOL = REPOSITORY_ROOT / "tools/docs/validate_semantic_task_records.py"
MANIFEST = REPOSITORY_ROOT / "docs/plan/semantic-task-records.json"
REPORT_SCHEMA = "semantic-task-record-validation-report/v1"
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
TASK_2040_REMOVAL_SCOPE = TASK_2042_DAEMON_SCOPE | {"TASK-2040"}

TASK_2040_RECORD = {
    "task": "TASK-2040",
    "task_file": "docs/plan/tasks/TASK-2040-remove-direct-ast-and-differential.md",
    "coverage_map": (
        "docs/plan/SEMANTIC-RULE-COVERAGE.md#"
        "task-2040-engine-only-removal"
    ),
    "canonical_rule_ids": [
        "CONF-ENGINE-ONLY-CLIENT-001",
        "SEM-TARGET-CORE-CPS-001",
        "OBS-TARGET-PROJECTION-001",
        "CONF-IMPLEMENTATION-001",
    ],
    "implementation": "partial",
    "layers": {
        "type": "partial",
        "core": "partial",
        "cps": "partial",
        "admission_runtime": "partial",
        "verification": "partial",
    },
    "evidence": {
        "status": "tested",
        "positive": ["TEST-TASK-2040-ENGINE-TERMINAL-POSITIVE"],
        "negative": [
            "TEST-TASK-2040-MANIFEST-REMOVAL",
            "TEST-TASK-2040-EXTERNAL-API-ABSENCE",
            "TEST-TASK-2040-REPLACEMENT-LEAN-CONTROLS",
        ],
        "mutation": ["TEST-TASK-2040-DECLARED-CONTRACT-ENGINE-PROPERTY"],
        "parity": {
            "status": "not_applicable",
            "rationale": "TASK-2041 owns the four-client normalized-terminal comparison.",
        },
        "proofs": [],
    },
    "parity": "below_spec",
    "missing_spec_clauses": [
        "The target Core/CPS domains and TASK-2041's four-client comparison remain incomplete."
    ],
    "non_goals": [
        "Lean implementation or deletion, a direct-evaluator compatibility route, source synthesis, a new execution domain, or TASK-2041's four-client parity proof."
    ],
    "next_obligation": "TASK-2041 validates the zero-use state, documentation and traceability, and four-client parity.",
    "verification": [
        "cargo test -p ash-engine --test task_2040_engine_only_removal",
    ],
}


class Task2040SemanticTaskRecordTests(unittest.TestCase):
    """Retain the activated removal task below target-spec parity."""

    def test_checked_in_record_records_tested_removal_evidence(self) -> None:
        """The completed removal controls are recorded without claiming target-spec parity."""
        result = subprocess.run(
            [
                "python3",
                str(TOOL),
                "--root",
                str(REPOSITORY_ROOT),
                "--manifest",
                str(MANIFEST),
            ],
            check=False,
            capture_output=True,
            cwd=REPOSITORY_ROOT,
            text=True,
        )
        report = json.loads(result.stdout)
        self.assertEqual(report.get("schema"), REPORT_SCHEMA)
        self.assertEqual(result.returncode, 0, report.get("errors"))

        payload = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.assertEqual(payload["active_scope"]["kind"], "task-2040-engine-only-removal")
        self.assertEqual(set(payload["active_scope"]["tasks"]), TASK_2040_REMOVAL_SCOPE)
        self.assertEqual(set(payload["active_tasks"]), TASK_2040_REMOVAL_SCOPE)
        record = next(item for item in payload["records"] if item["task"] == "TASK-2040")
        self.assertEqual(record, TASK_2040_RECORD)

        task = REPOSITORY_ROOT / TASK_2040_RECORD["task_file"]
        task_text = task.read_text(encoding="utf-8")
        self.assertIn("**Status:** Complete", task_text)
        self.assertIn("**Evidence:** tested", task_text)
        self.assertIn("**Parity:** below_spec", task_text)


if __name__ == "__main__":
    unittest.main()
