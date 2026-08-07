"""Contract tests for the Phase 207 closeout checker."""

from __future__ import annotations

import unittest
from pathlib import Path

from tools.docs.check_phase_207_closeout import (
    REQUIRED_SCANNER_MARKERS,
    _completion_findings,
    audit_repository,
    check_semantic_axes,
    check_scanner_inventory,
)


class Phase207CloseoutCheckerTests(unittest.TestCase):
    """Keep closeout readiness checks fail-closed and scope-aware."""

    def test_missing_semantic_axis_is_reported(self) -> None:
        findings = check_semantic_axes(
            [
                {
                    "task": "TASK-2073",
                    "canonical_rule_ids": ["SEM-MODULE-REALIZATION-003"],
                    "implementation": "partial",
                    "evidence": {"status": "tested"},
                }
            ]
        )

        self.assertTrue(any(finding["code"] == "missing_parity_axis" for finding in findings))

    def test_scanner_inventory_requires_every_audit_marker(self) -> None:
        audit = "\n".join(REQUIRED_SCANNER_MARKERS[:-1])

        findings = check_scanner_inventory(audit)

        self.assertTrue(any(finding["code"] == "missing_scanner_inventory" for finding in findings))

    def test_current_repository_is_auditable_but_not_ready_to_close(self) -> None:
        root = Path(__file__).resolve().parents[3]

        report = audit_repository(root)
        required_report = audit_repository(root, require_complete=True)

        self.assertEqual(report["contract_findings"], [])
        self.assertFalse(report["ready"])
        self.assertTrue(required_report["completion_findings"])

    def test_completion_findings_ignore_historical_partial_handoffs(self) -> None:
        findings = _completion_findings(
            Path("."),
            [
                {
                    "task": "TASK-2057",
                    "canonical_rule_ids": ["SEM-MODULE-REALIZATION-001"],
                    "implementation": "partial",
                    "evidence": {"status": "tested"},
                    "parity": "below_spec",
                },
                {
                    "task": "TASK-2073",
                    "canonical_rule_ids": ["SEM-MODULE-REALIZATION-003"],
                    "implementation": "partial",
                    "evidence": {"status": "tested"},
                    "parity": "below_spec",
                },
            ],
            "status: in progress",
        )

        self.assertEqual(
            {finding.get("task") for finding in findings if finding.get("task")},
            {"TASK-2073"},
        )


if __name__ == "__main__":
    unittest.main()
