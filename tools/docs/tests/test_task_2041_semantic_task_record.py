#!/usr/bin/env python3
"""RED repository contract for TASK-2041's Engine-only closeout record."""
from __future__ import annotations

import json
import shutil
import subprocess
import tempfile
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

TASK_2041_RECORD = {
    "task": "TASK-2041",
    "task_file": "docs/plan/tasks/TASK-2041-engine-only-closeout-docs-traceability-and-gate.md",
    "coverage_map": "docs/plan/SEMANTIC-RULE-COVERAGE.md#task-2041-engine-only-closeout",
    "canonical_rule_ids": [
        "CONF-ENGINE-ONLY-CLIENT-001",
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
        "positive": ["TEST-TASK-2041-FOUR-CLIENT-PARITY"],
        "negative": [
            "TEST-TASK-2041-ZERO-USE-GATE",
            "TEST-TASK-2041-LEAN-BOUNDARY",
        ],
        "mutation": ["TEST-TASK-2041-DECLARED-CORPUS-PROPERTY"],
        "parity": {
            "status": "covered",
            "evidence": ["TEST-TASK-2041-FOUR-CLIENT-PARITY"],
        },
        "proofs": [],
    },
    "parity": "below_spec",
    "missing_spec_clauses": [
        "The target Core/CPS domains remain partial; TASK-2041 compares only the declared shared source contract across four independent local Engine clients."
    ],
    "non_goals": [
        "A shared Engine service, daemon execution for ash run or REPL, source synthesis, deferred-case implementation, Lean execution, or a runtime refinement proof."
    ],
    "next_obligation": "Later target-rule realization tasks own every remaining partial/below-spec clause.",
    "verification": [
        "cargo test -p ash-cli --test task_2041_engine_only_four_client_parity"
    ],
}


class Task2041SemanticTaskRecordTests(unittest.TestCase):
    """Require an active closeout record without overstating target-spec parity."""

    def test_checked_in_record_activates_the_engine_only_closeout(self) -> None:
        """The closeout owns four local-client parity while target realization stays partial."""
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
        record = next(item for item in payload["records"] if item["task"] == "TASK-2041")
        self.assertEqual(record, TASK_2041_RECORD)

        task = REPOSITORY_ROOT / TASK_2041_RECORD["task_file"]
        task_text = task.read_text(encoding="utf-8")
        self.assertIn("**Status:** Complete", task_text)
        self.assertIn("**Implementation:** partial", task_text)
        self.assertIn("**Evidence:** tested", task_text)
        self.assertIn("**Parity:** below_spec", task_text)

    def test_complete_closeout_handoff_validates_without_overstating_parity(self) -> None:
        """Delivered closeout evidence permits lifecycle closure with its scoped axes intact."""
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory) / "repository"
            shutil.copytree(REPOSITORY_ROOT / "docs", root / "docs")
            manifest = root / "docs/plan/semantic-task-records.json"
            task = root / TASK_2041_RECORD["task_file"]
            task_text = task.read_text(encoding="utf-8")
            self.assertIn("**Status:** Complete", task_text)

            result = subprocess.run(
                [
                    "python3",
                    str(TOOL),
                    "--root",
                    str(root),
                    "--manifest",
                    str(manifest),
                ],
                check=False,
                capture_output=True,
                cwd=root,
                text=True,
            )
            report = json.loads(result.stdout)

            self.assertEqual(report.get("schema"), REPORT_SCHEMA)
            self.assertEqual(result.returncode, 0, report.get("errors"))


if __name__ == "__main__":
    unittest.main()
