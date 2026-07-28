#!/usr/bin/env python3
"""Repository contract for the active TASK-2035 semantic-task record."""
from __future__ import annotations

import importlib.util
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
TASK_2035_DOCUMENTATION_VERIFICATION = (
    "python3 -m unittest tools.docs.tests.test_task_2035_semantic_task_record"
)
TASK_2035_RULES = [
    "CONF-SYNTH-SOURCE-WRAPPER-001",
    "OBS-REPL-ENGINE-CLIENT-001",
    "CONF-ENGINE-ONLY-CLIENT-001",
]
TASK_2035_MISSING_CLAUSES = [
    "Realize every selected wrapper, REPL route, and daemon route through Engine; "
    "then realize the remaining target SPEC-077 and SPEC-011 domains before claiming parity."
]
TASK_2035_NON_GOALS = [
    "Source lowering, Engine APIs, test-runner execution, REPL execution, daemon transport, "
    "a general source synthesizer, and Lean implementation."
]
TASK_2035_NEXT_OBLIGATION = (
    "TASK-2038, TASK-2039, and TASK-2042 must implement their named routes with focused tests; "
    "TASK-2041 must establish the same-admitted-program four-client terminal comparison."
)


def task_2035_record() -> dict[str, object]:
    """Return the no-runtime-evidence record required for TASK-2035."""
    return {
        "task": "TASK-2035",
        "task_file": "docs/plan/tasks/TASK-2035-canonical-client-test-contracts.md",
        "coverage_map": "docs/plan/SEMANTIC-RULE-COVERAGE.md#engine-only-client-contracts",
        "canonical_rule_ids": TASK_2035_RULES,
        "implementation": "not_implemented",
        "layers": {
            "type": "partial",
            "core": "partial",
            "cps": "partial",
            "admission_runtime": "not_implemented",
            "verification": "not_implemented",
        },
        "evidence": {
            "status": "none",
            "positive": [],
            "negative": [],
            "mutation": [],
            "parity": {
                "status": "not_applicable",
                "rationale": (
                    "No client route is realized by this documentation task."
                ),
            },
            "proofs": [],
        },
        "parity": "below_spec",
        "missing_spec_clauses": TASK_2035_MISSING_CLAUSES,
        "non_goals": TASK_2035_NON_GOALS,
        "next_obligation": TASK_2035_NEXT_OBLIGATION,
        "verification": [TASK_2035_DOCUMENTATION_VERIFICATION],
    }


class Task2035SemanticTaskRecordTests(unittest.TestCase):
    """Keep the Engine-only client contract active without inventing evidence."""

    def run_validator(
        self, root: Path, manifest: Path
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        """Run the real semantic-task validator and retain its JSON report."""
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

    def test_task_2035_scope_and_documentation_verification_are_controlled(self) -> None:
        """The contract task activates one explicit non-Cargo verification command."""
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory) / "repository"
            shutil.copytree(REPOSITORY_ROOT / "docs", root / "docs")
            manifest = root / "docs/plan/semantic-task-records.json"
            payload = json.loads(manifest.read_text(encoding="utf-8"))
            payload["active_scope"] = {
                "kind": "task-2035-contract",
                "tasks": sorted(TASK_2035_CONTRACT_SCOPE),
            }
            payload["active_tasks"] = sorted(TASK_2035_CONTRACT_SCOPE)
            records = payload["records"]
            assert isinstance(records, list)
            records[:] = [
                record
                for record in records
                if not isinstance(record, dict)
                or record.get("task") not in {"TASK-2035", "TASK-2037", "TASK-2038", "TASK-2039"}
            ]
            records.append(task_2035_record())
            manifest.write_text(
                json.dumps(payload, indent=2) + "\n", encoding="utf-8"
            )

            result, report = self.run_validator(root, manifest)

            self.assertEqual(report.get("schema"), REPORT_SCHEMA)
            self.assertEqual(result.returncode, 0, report.get("errors"))

    def test_checked_in_record_reports_contract_without_runtime_evidence(self) -> None:
        """TASK-2035 is active and says plainly that implementation remains absent."""
        result, report = self.run_validator(REPOSITORY_ROOT, MANIFEST)
        self.assertEqual(report.get("schema"), REPORT_SCHEMA)
        self.assertEqual(result.returncode, 0, report.get("errors"))

        payload = json.loads(MANIFEST.read_text(encoding="utf-8"))
        active_scope = payload["active_scope"]
        self.assertEqual(active_scope["kind"], "task-2039-repl")
        self.assertEqual(set(active_scope["tasks"]), TASK_2039_REPL_SCOPE)
        self.assertEqual(set(payload["active_tasks"]), TASK_2039_REPL_SCOPE)

        task_text = (
            REPOSITORY_ROOT
            / "docs/plan/tasks/TASK-2035-canonical-client-test-contracts.md"
        ).read_text(encoding="utf-8")
        self.assertIn("**Status:** Complete", task_text)

        records = payload["records"]
        self.assertIsInstance(records, list)
        record = next(
            item for item in records if isinstance(item, dict) and item.get("task") == "TASK-2035"
        )
        self.assertEqual(record, task_2035_record())

    def test_task_2035_documentation_command_is_not_a_general_python_escape_hatch(self) -> None:
        """Only TASK-2035 may own its exact documentation verification command."""
        specification = importlib.util.spec_from_file_location(
            "validate_semantic_task_records", TOOL
        )
        self.assertIsNotNone(specification)
        assert specification is not None
        self.assertIsNotNone(specification.loader)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)

        self.assertTrue(
            module.allowed_verification_command(TASK_2035_DOCUMENTATION_VERIFICATION)
        )
        self.assertTrue(
            module.command_matches_task_integration_test(
                TASK_2035_DOCUMENTATION_VERIFICATION, "TASK-2035"
            )
        )
        self.assertFalse(
            module.command_matches_task_integration_test(
                TASK_2035_DOCUMENTATION_VERIFICATION, "TASK-2032"
            )
        )


if __name__ == "__main__":
    unittest.main()
