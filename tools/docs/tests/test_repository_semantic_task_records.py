#!/usr/bin/env python3
"""Repository contract for the active TASK-2031 prerequisite semantic-task records."""
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


class RepositorySemanticTaskRecordTests(unittest.TestCase):
    """Keep the checked-in active records aligned with their declared scope."""

    def test_task_2031_prerequisite_records_validate_as_the_complete_active_scope(self) -> None:
        """TASK-2031 adds one general prerequisite without relaxing inherited bounded records."""
        self.assertTrue(TOOL.exists(), f"missing TASK-2028 validator: {TOOL}")
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
        try:
            report = json.loads(result.stdout)
        except json.JSONDecodeError as error:
            self.fail(
                "semantic-task validator must emit JSON to stdout: "
                f"{error}; stderr: {result.stderr}"
            )

        self.assertEqual(report.get("schema"), REPORT_SCHEMA)
        self.assertEqual(result.returncode, 0, report.get("errors"))

        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        active_scope = manifest["active_scope"]
        self.assertEqual(active_scope["kind"], "task-2031-prerequisite")
        self.assertEqual(set(active_scope["tasks"]), TASK_2031_PREREQUISITE_SCOPE)
        self.assertEqual(len(active_scope["tasks"]), len(TASK_2031_PREREQUISITE_SCOPE))
        self.assertEqual(set(manifest["active_tasks"]), TASK_2031_PREREQUISITE_SCOPE)
        self.assertEqual(len(manifest["active_tasks"]), len(TASK_2031_PREREQUISITE_SCOPE))

        records = manifest["records"]
        self.assertEqual({record["task"] for record in records}, TASK_2031_PREREQUISITE_SCOPE)
        self.assertEqual(len(records), len(TASK_2031_PREREQUISITE_SCOPE))
        domains = {record["task"]: record["domain"]["status"] for record in records}
        self.assertEqual(domains["TASK-2031"], "general")
        self.assertTrue(all(domains[task] == "bounded" for task in TASK_1988_FOLLOWUPS))


if __name__ == "__main__":
    unittest.main()
