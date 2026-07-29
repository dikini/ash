#!/usr/bin/env python3
"""Repository contract for TASK-2042's closed daemon descriptor handoff."""
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
    "TASK-2001",
    "TASK-2002",
    "TASK-2003",
    "TASK-2004",
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
TASK_2041_CLOSEOUT_SCOPE = TASK_2040_REMOVAL_SCOPE | {"TASK-2041"}

TASK_2042_RECORD = {
    "task": "TASK-2042",
    "task_file": "docs/plan/tasks/TASK-2042-daemon-admitted-request-terminal-envelope-parity.md",
    "coverage_map": (
        "docs/plan/SEMANTIC-RULE-COVERAGE.md#"
        "task-2042-daemon-descriptor-and-terminal-envelope-parity"
    ),
    "canonical_rule_ids": [
        "CONF-ENGINE-ONLY-CLIENT-001",
        "SEM-EFFECT-ADMISSION-001",
        "SEM-EFFECT-TIMEOUT-001",
        "SEM-EFFECT-CANCEL-001",
        "SEM-EFFECT-TERMINAL-001",
    ],
    "implementation": "partial",
    "layers": {
        "type": "not_applicable",
        "core": "not_applicable",
        "cps": "partial",
        "admission_runtime": "partial",
        "verification": "partial",
    },
    "evidence": {
        "status": "tested",
        "positive": ["TEST-TASK-2042-DAEMON-DESCRIPTOR-SUCCESS"],
        "negative": [
            "TEST-TASK-2042-DAEMON-DESCRIPTOR-ADMISSION-REJECTION",
            "TEST-TASK-2042-DAEMON-DESCRIPTOR-PRE-EXECUTION-CLASSIFICATION",
            "TEST-TASK-2042-DAEMON-DESCRIPTOR-RUN-CONTROLS",
        ],
        "mutation": ["TEST-TASK-2042-DAEMON-DESCRIPTOR-MUTATION"],
        "parity": {
            "status": "covered",
            "evidence": ["TEST-TASK-2042-DAEMON-DESCRIPTOR-PARITY"],
        },
        "proofs": [],
    },
    "parity": "below_spec",
    "missing_spec_clauses": [
        "Only `TASK-2035-SHARED-ROUTE-001` is selected. The remaining daemon protocol domain, residual direct-evaluator deletion, and TASK-2041's four-client comparison remain incomplete."
    ],
    "non_goals": [
        "A shared Engine service, cross-process request handles, source synthesis, admission reconstruction, a new daemon language, formatting, or Lean execution."
    ],
    "next_obligation": "Retain the selected daemon descriptor route while TASK-2040 removes residual daemon direct-evaluator calls and TASK-2041 supplies the four-client terminal comparison.",
    "verification": [
        "cargo test -p ash-cli --test task_2042_daemon_admitted_request_terminal_envelope_parity"
    ],
}


class Task2042SemanticTaskRecordTests(unittest.TestCase):
    """Retain the completed daemon route below target-spec parity."""

    def test_checked_in_record_retains_the_completed_daemon_route_evidence(self) -> None:
        """Completion retains focused evidence without taking downstream ownership."""
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
        self.assertEqual(payload["active_scope"]["kind"], "task-2041-engine-only-closeout")
        self.assertEqual(set(payload["active_scope"]["tasks"]), TASK_2041_CLOSEOUT_SCOPE)
        self.assertEqual(set(payload["active_tasks"]), TASK_2041_CLOSEOUT_SCOPE)
        record = next(item for item in payload["records"] if item["task"] == "TASK-2042")
        self.assertEqual(record, TASK_2042_RECORD)

        task = REPOSITORY_ROOT / TASK_2042_RECORD["task_file"]
        task_text = task.read_text(encoding="utf-8")
        self.assertIn("**Status:** Complete", task_text)
        self.assertIn("**Evidence:** tested", task_text)
        self.assertIn("**Parity:** below_spec", task_text)


if __name__ == "__main__":
    unittest.main()
