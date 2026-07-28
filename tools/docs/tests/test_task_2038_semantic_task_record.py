#!/usr/bin/env python3
"""Repository contract for TASK-2038's closed Engine-only test-client handoff."""
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

TASK_2038_RECORD = {
    "task": "TASK-2038",
    "task_file": "docs/plan/tasks/TASK-2038-ash-test-canonical-engine-execution.md",
    "coverage_map": (
        "docs/plan/SEMANTIC-RULE-COVERAGE.md#"
        "task-2038-ash-test-canonical-engine-execution"
    ),
    "canonical_rule_ids": [
        "CONF-SYNTH-SOURCE-WRAPPER-001",
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
            "TEST-TASK-2038-SYNTH-WRAPPER-POSITIVE",
            "TEST-TASK-2038-CATALOGUE-PROPERTY",
        ],
        "negative": ["TEST-TASK-2038-DEFERRED-CATALOGUE"],
        "mutation": ["TEST-TASK-2038-MUTATION-NO-FALLBACK"],
        "parity": {
            "status": "covered",
            "evidence": ["TEST-TASK-2038-SHARED-ROUTE-PARITY"],
        },
        "proofs": [],
    },
    "parity": "below_spec",
    "missing_spec_clauses": [
        "Only the two exact TASK-2035 source identities are selected. The remaining "
        "SPEC-077 synthesized-test domain, unselected client routes, residual "
        "direct-evaluator deletion, and TASK-2041's four-client comparison remain "
        "incomplete."
    ],
    "non_goals": [
        "A general source synthesizer, forms absent from the TASK-2035 catalogue, REPL, daemon, or ash run client implementation.",
        "Target grammar expansion or a direct-evaluator compatibility mode.",
        "TASK-2040-owned removal of residual direct test-evaluator and differential material.",
        "TASK-2041's four-client same-admitted-program terminal comparison.",
    ],
    "next_obligation": "Retain the selected Engine route while TASK-2040 removes residual direct test-evaluator material and TASK-2041 supplies the four-client terminal comparison.",
    "verification": [
        "cargo test -p ash-cli --test task_2038_ash_test_canonical_engine_execution"
    ],
}


class Task2038SemanticTaskRecordTests(unittest.TestCase):
    """Retain the completed test-client boundary below target parity."""

    def test_checked_in_record_retains_the_exact_test_client_handoff(self) -> None:
        """Completion retains checked client authority without taking downstream ownership."""
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
        self.assertEqual(payload["active_scope"]["kind"], "task-2039-repl")
        self.assertEqual(
            set(payload["active_scope"]["tasks"]), TASK_2039_REPL_SCOPE
        )
        self.assertEqual(set(payload["active_tasks"]), TASK_2039_REPL_SCOPE)
        record = next(item for item in payload["records"] if item["task"] == "TASK-2038")
        self.assertEqual(record, TASK_2038_RECORD)

        task_text = (
            REPOSITORY_ROOT
            / "docs/plan/tasks/TASK-2038-ash-test-canonical-engine-execution.md"
        ).read_text(encoding="utf-8")
        self.assertIn("**Status:** Complete", task_text)
        self.assertIn("**Evidence:** tested", task_text)
        self.assertIn("**Parity:** below_spec", task_text)


if __name__ == "__main__":
    unittest.main()
