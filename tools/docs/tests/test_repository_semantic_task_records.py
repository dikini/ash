#!/usr/bin/env python3
"""Repository contract for the active TASK-2032 integration semantic-task records."""
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


class RepositorySemanticTaskRecordTests(unittest.TestCase):
    """Keep the checked-in active records aligned with their declared scope."""

    def run_validator(self, root: Path, manifest: Path) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        """Run the semantic-task validator and preserve its JSON report contract."""
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
        try:
            report = json.loads(result.stdout)
        except json.JSONDecodeError as error:
            self.fail(
                "semantic-task validator must emit JSON to stdout: "
                f"{error}; stderr: {result.stderr}"
            )
        return result, report

    def test_task_2032_handoff_remains_in_the_active_repl_scope(self) -> None:
        """Later client routes extend, rather than replace, the checked integration handoff."""
        self.assertTrue(TOOL.exists(), f"missing TASK-2028 validator: {TOOL}")
        result, report = self.run_validator(REPOSITORY_ROOT, MANIFEST)

        self.assertEqual(report.get("schema"), REPORT_SCHEMA)
        self.assertEqual(result.returncode, 0, report.get("errors"))

        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        active_scope = manifest["active_scope"]
        self.assertEqual(active_scope["kind"], "task-2039-repl")
        self.assertEqual(set(active_scope["tasks"]), TASK_2039_REPL_SCOPE)
        self.assertEqual(len(active_scope["tasks"]), len(TASK_2039_REPL_SCOPE))
        self.assertEqual(set(manifest["active_tasks"]), TASK_2039_REPL_SCOPE)
        self.assertEqual(len(manifest["active_tasks"]), len(TASK_2039_REPL_SCOPE))

        records = manifest["records"]
        self.assertEqual({record["task"] for record in records}, TASK_2039_REPL_SCOPE)
        self.assertEqual(len(records), len(TASK_2039_REPL_SCOPE))
        self.assertTrue(TASK_2032_INTEGRATION_SCOPE.issubset(set(manifest["active_tasks"])))

    def test_closed_task_2031_prerequisite_and_task_2032_integration_validate(self) -> None:
        """The complete bounded integration owner remains in the declared active scope."""
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory) / "repository"
            shutil.copytree(REPOSITORY_ROOT / "docs", root / "docs")
            manifest = root / "docs/plan/semantic-task-records.json"
            task_file = root / "docs/plan/tasks/TASK-2031-lambda-ash-effect-correspondence.md"
            self.assertIn("**Status:** Complete", task_file.read_text(encoding="utf-8"))

            manifest_data = json.loads(manifest.read_text(encoding="utf-8"))
            records = manifest_data["records"]
            assert isinstance(records, list)
            inherited_task_files = {
                record["task"]: root / record["task_file"]
                for record in records
                if isinstance(record, dict) and record.get("task") in TASK_1988_FOLLOWUPS
            }
            self.assertEqual(set(inherited_task_files), TASK_1988_FOLLOWUPS)
            for inherited_task, inherited_file in inherited_task_files.items():
                self.assertIn("**Status:** In progress", inherited_file.read_text(encoding="utf-8"), inherited_task)

            task_2032_file = root / "docs/plan/tasks/TASK-2032-shared-engine-execution-seam-and-client-parity.md"
            self.assertIn("**Status:** Complete", task_2032_file.read_text(encoding="utf-8"))

            result, report = self.run_validator(root, manifest)
            self.assertEqual(report.get("schema"), REPORT_SCHEMA)
            self.assertEqual(result.returncode, 0, report.get("errors"))

            inherited_file = inherited_task_files["TASK-2001"]
            inherited_file.write_text(
                inherited_file.read_text(encoding="utf-8").replace(
                    "**Status:** In progress", "**Status:** Complete", 1
                ),
                encoding="utf-8",
            )
            result, report = self.run_validator(root, manifest)
            self.assertNotEqual(result.returncode, 0, report)
            self.assertTrue(
                any(
                    error.get("kind") == "active_task_status_mismatch" and error.get("task") == "TASK-2001"
                    for error in report.get("errors", [])
                    if isinstance(error, dict)
                ),
                report,
            )

            inherited_file.write_text(
                inherited_file.read_text(encoding="utf-8").replace(
                    "**Status:** Complete", "**Status:** In progress", 1
                ),
                encoding="utf-8",
            )
            task_2032_file.write_text(
                task_2032_file.read_text(encoding="utf-8").replace(
                    "**Status:** Complete", "**Status:** In progress", 1
                ),
                encoding="utf-8",
            )
            result, report = self.run_validator(root, manifest)
            self.assertNotEqual(result.returncode, 0, report)
            self.assertTrue(
                any(
                    error.get("kind") == "active_task_status_mismatch" and error.get("task") == "TASK-2032"
                    for error in report.get("errors", [])
                    if isinstance(error, dict)
                ),
                report,
            )

            task_file.write_text(
                task_file.read_text(encoding="utf-8").replace(
                    "**Status:** Complete", "**Status:** In progress", 1
                ),
                encoding="utf-8",
            )
            result, report = self.run_validator(root, manifest)
            self.assertNotEqual(result.returncode, 0, report)
            self.assertTrue(
                any(
                    error.get("kind") == "active_task_status_mismatch" and error.get("task") == "TASK-2031"
                    for error in report.get("errors", [])
                    if isinstance(error, dict)
                ),
                report,
            )


if __name__ == "__main__":
    unittest.main()
