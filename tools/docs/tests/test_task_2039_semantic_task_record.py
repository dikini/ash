#!/usr/bin/env python3
"""Repository contract for TASK-2039's closed REPL Engine-route handoff."""
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
TASK_2065_MODULE_REALIZATION_CLOSEOUT_SCOPE = TASK_2041_CLOSEOUT_SCOPE | {
    "TASK-2057", "TASK-2058", "TASK-2059", "TASK-2060", "TASK-2061", "TASK-2066",
    "TASK-2062", "TASK-2063", "TASK-2067", "TASK-2068", "TASK-2070", "TASK-2071",
    "TASK-2074", "TASK-2075", "TASK-2072", "TASK-2073", "TASK-2069", "TASK-2064", "TASK-2065",
}

TASK_2039_RECORD = {
    "task": "TASK-2039",
    "task_file": "docs/plan/tasks/TASK-2039-repl-canonical-engine-execution.md",
    "coverage_map": (
        "docs/plan/SEMANTIC-RULE-COVERAGE.md#"
        "task-2039-repl-canonical-engine-execution"
    ),
    "canonical_rule_ids": [
        "OBS-REPL-ENGINE-CLIENT-001",
        "CONF-ENGINE-ONLY-CLIENT-001",
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
        "positive": [
            "TEST-TASK-2039-REPL-ENGINE-POSITIVE",
            "TEST-TASK-2039-REPL-MULTILINE",
        ],
        "negative": [
            "TEST-TASK-2039-REPL-ADMISSION-REJECTION",
            "TEST-TASK-2039-REPL-INSPECTION-NO-EVALUATION",
        ],
        "mutation": ["TEST-TASK-2039-REPL-DECLARED-CORPUS-PROPERTY"],
        "parity": {
            "status": "covered",
            "evidence": ["TEST-TASK-2039-REPL-SHARED-ROUTE-PARITY"],
        },
        "proofs": [],
    },
    "parity": "below_spec",
    "missing_spec_clauses": [
        "Only the two exact TASK-2035 REPL source identities are selected. Stored-session "
        "shapes beyond the selected controls, remaining SPEC-011 submission forms, residual "
        "direct-evaluator deletion, daemon and ash run transport, and TASK-2041's four-client "
        "comparison remain incomplete."
    ],
    "non_goals": [
        "A new REPL language, persistent evaluation beyond the specified session state, target "
        "grammar expansion, daemon or ash run transport, or a direct-evaluator compatibility mode.",
        "TASK-2041's four-client same-source-contract terminal comparison.",
    ],
    "next_obligation": "Retain the selected Engine route while TASK-2040 removes residual REPL direct-evaluator calls and TASK-2041 supplies the four-client terminal comparison.",
    "verification": [
        "cargo test -p ash-repl --test task_2039_repl_canonical_engine_execution"
    ],
}


class Task2039SemanticTaskRecordTests(unittest.TestCase):
    """Retain the completed REPL route below target-spec parity."""

    def test_checked_in_record_retains_the_completed_repl_route_evidence(self) -> None:
        """Completion retains the focused route without taking downstream ownership."""
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
        self.assertEqual(payload["active_scope"]["kind"], "task-2065-module-realization-closeout")
        self.assertEqual(set(payload["active_scope"]["tasks"]), TASK_2065_MODULE_REALIZATION_CLOSEOUT_SCOPE)
        self.assertEqual(set(payload["active_tasks"]), TASK_2065_MODULE_REALIZATION_CLOSEOUT_SCOPE)
        record = next(item for item in payload["records"] if item["task"] == "TASK-2039")
        self.assertEqual(record, TASK_2039_RECORD)

        task = REPOSITORY_ROOT / TASK_2039_RECORD["task_file"]
        task_text = task.read_text(encoding="utf-8")
        self.assertIn("**Status:** Complete", task_text)
        self.assertIn("**Evidence:** tested", task_text)
        self.assertIn("**Parity:** below_spec", task_text)


if __name__ == "__main__":
    unittest.main()
