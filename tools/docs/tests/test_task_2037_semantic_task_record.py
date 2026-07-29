#!/usr/bin/env python3
"""Repository contract for the active TASK-2037 semantic-task record."""
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

TASK_2037_RECORD = {
    "task": "TASK-2037",
    "task_file": "docs/plan/tasks/TASK-2037-engine-owned-cps-executor-and-runtime-crate-rename.md",
    "coverage_map": "docs/plan/SEMANTIC-RULE-COVERAGE.md#task-2037-engine-owned-cps-executor-boundary",
    "canonical_rule_ids": [
        "SEM-TARGET-CORE-CPS-001",
        "SEM-EFFECT-ADMISSION-001",
        "OBS-TARGET-PROJECTION-001",
        "CONF-ENGINE-ONLY-CLIENT-001",
        "SEM-CPS-TRAP-001",
        "SEM-EFFECT-TIMEOUT-001",
        "SEM-EFFECT-CANCEL-001",
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
        "positive": [
            "TEST-TASK-2037-ENGINE-OWNED-CPS-POSITIVE",
            "TEST-TASK-2037-ENGINE-OWNED-CPS-TRAP",
            "TEST-TASK-2037-ENGINE-OWNED-CPS-TIMEOUT",
            "TEST-TASK-2037-ENGINE-OWNED-CPS-CANCELLATION",
        ],
        "negative": [
            "TEST-TASK-2037-ENGINE-OWNED-CPS-NEGATIVE",
        ],
        "mutation": [
            "TEST-TASK-2037-ENGINE-OWNED-CPS-MUTATION",
        ],
        "parity": {
            "status": "not_applicable",
            "rationale": "No client route or reference-executor comparison is performed by this prerequisite boundary task.",
        },
        "proofs": [],
    },
    "parity": "below_spec",
    "missing_spec_clauses": [
        "Selected client routes, full target Core/CPS domains, deletion of direct-AST and differential material, and TASK-2041's four-client terminal comparison remain incomplete."
    ],
    "non_goals": [
        "Test-runner, REPL, daemon, or ash run client-route implementation.",
        "Deletion of direct-AST evaluation, the Rust differential stack, or Lean material.",
        "Renaming ash-interp while TASK-2040-owned AST material remains.",
        "Transferring TASK-2040 deletion ownership when retained audit-listed differential tests move into Engine-private test modules.",
    ],
    "next_obligation": "TASK-2038, TASK-2039, TASK-2042, and TASK-2040 must consume the Engine-private executor boundary; TASK-2041 must prove API absence and four-client normalized-terminal parity.",
    "verification": [
        "cargo test -p ash-engine --test task_2037_engine_owned_cps_executor"
    ],
}


class Task2037SemanticTaskRecordTests(unittest.TestCase):
    """Keep the prerequisite executor boundary explicitly below target parity."""

    def test_checked_in_record_activates_exact_scope_without_client_parity(self) -> None:
        """TASK-2037 owns the private boundary, not a runnable client route."""
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
        self.assertEqual(
            payload["active_scope"]["kind"], "task-2041-engine-only-closeout"
        )
        self.assertEqual(
            set(payload["active_scope"]["tasks"]), TASK_2041_CLOSEOUT_SCOPE
        )
        self.assertEqual(set(payload["active_tasks"]), TASK_2041_CLOSEOUT_SCOPE)
        record = next(item for item in payload["records"] if item["task"] == "TASK-2037")
        self.assertEqual(record, TASK_2037_RECORD)

        task = REPOSITORY_ROOT / TASK_2037_RECORD["task_file"]
        task_text = task.read_text(encoding="utf-8")
        self.assertIn("**Status:** Complete", task_text)
        self.assertIn("**Evidence:** tested", task_text)
        self.assertIn("**Parity:** below_spec", task_text)


if __name__ == "__main__":
    unittest.main()
