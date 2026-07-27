#!/usr/bin/env python3
"""Contract tests for TASK-2028 semantic-task workflow records.

The manifest is deliberately the machine-readable authority for workflow
conformance.  These tests use a tiny synthetic repository so that the
validator contract is independent of whichever active task records happen to
be checked in at the time it is run.
"""
from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from typing import Any

from tools.docs.validate_semantic_task_records import (
    TASK_2031_PREREQUISITE_SCOPE,
    allowed_verification_command,
    command_matches_task_integration_test,
    validate_active_scope,
)


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
TOOL = REPOSITORY_ROOT / "tools/docs/validate_semantic_task_records.py"
REPORT_SCHEMA = "semantic-task-record-validation-report/v1"


class SemanticTaskRecordContractTests(unittest.TestCase):
    """Exercise the fail-closed semantic-task record validator CLI."""

    def write_valid_fixture(self, root: Path) -> Path:
        """Create the smallest repository that owns one bounded semantic task."""
        task_file = root / "docs/plan/tasks/TASK-9001-example.md"
        task_file.parent.mkdir(parents=True)
        task_file.write_text(
            "# TASK-9001: Example semantic workflow record\n\n"
            "This bounded fixture is linked from its machine-readable record.\n\n"
            "**Status:** In progress\n\n"
            "**Semantic task record:** "
            "[TASK-9001](../semantic-task-records.json)\n\n"
            "**Semantic coverage map:** "
            "[TASK-9001 workflow record](../SEMANTIC-RULE-COVERAGE.md#"
            "task-9001-workflow-record)\n\n"
            "**Declared domain:** bounded\n\n"
            "## Evidence\n\n"
            "The task-owned traceability edges resolve to this heading.\n",
            encoding="utf-8",
        )

        coverage_map = root / "docs/plan/SEMANTIC-RULE-COVERAGE.md"
        coverage_map.parent.mkdir(parents=True, exist_ok=True)
        coverage_map.write_text(
            "# Semantic Rule Coverage Map\n\n"
            "## TASK-9001 workflow record\n\n"
            "**Task:** [TASK-9001](tasks/TASK-9001-example.md)\n"
            "**Canonical rules:** `SEM-CPS-CALL-001`, `SEM-CPS-JUMP-001`\n"
            "**Domain:** bounded\n"
            "**Layers:** type bounded; core bounded; cps bounded; "
            "admission-runtime bounded; verification bounded.\n"
            "**Evidence:**\n"
            "- **Positive:** `TEST-9001-POSITIVE`\n"
            "- **Negative:** `TEST-9001-NEGATIVE`\n"
            "- **Mutation:** `TEST-9001-MUTATION`\n"
            "- **Parity:** not applicable; this fixture has no reference interpreter pair.\n"
            "**Non-goals:** General call lowering.\n"
            "**Next obligation:** Admit parameterized local calls.\n",
            encoding="utf-8",
        )

        traceability = root / "docs/spec/SEMANTIC-TRACEABILITY.json"
        traceability.parent.mkdir(parents=True, exist_ok=True)
        traceability.write_text(
            json.dumps(
                {
                    "schema": "semantic-traceability-graph/v1",
                    "nodes": [
                        {
                            "id": "SEM-CPS-CALL-001",
                            "kind": "canonical-rule",
                            "status": ["specified"],
                            "anchor": "docs/spec/CANONICAL-CORE.md#calls",
                        },
                        {
                            "id": "SEM-CPS-JUMP-001",
                            "kind": "canonical-rule",
                            "status": ["specified"],
                            "anchor": "docs/spec/CANONICAL-CORE.md#jumps",
                        },
                        {
                            "id": "TEST-9001-POSITIVE",
                            "kind": "test",
                            "status": ["tested"],
                            "anchor": "tools/docs/tests/test_validate_semantic_task_records.py#semantic-task-record-contract-tests",
                        },
                        {
                            "id": "TEST-9001-NEGATIVE",
                            "kind": "test",
                            "status": ["tested"],
                            "anchor": "tools/docs/tests/test_validate_semantic_task_records.py#semantic-task-record-contract-tests",
                        },
                        {
                            "id": "TEST-9001-MUTATION",
                            "kind": "test",
                            "status": ["tested"],
                            "anchor": "tools/docs/tests/test_validate_semantic_task_records.py#semantic-task-record-contract-tests",
                        },
                        {
                            "id": "TEST-9001-PARITY",
                            "kind": "test",
                            "status": ["tested"],
                            "anchor": "tools/docs/tests/test_validate_semantic_task_records.py#semantic-task-record-contract-tests",
                        },
                    ],
                    "edges": [
                        {
                            "kind": "tested_by",
                            "from": "SEM-CPS-CALL-001",
                            "to": "TEST-9001-POSITIVE",
                            "anchor": "docs/plan/tasks/TASK-9001-example.md#evidence",
                        },
                        {
                            "kind": "tested_by",
                            "from": "SEM-CPS-CALL-001",
                            "to": "TEST-9001-NEGATIVE",
                            "anchor": "docs/plan/tasks/TASK-9001-example.md#evidence",
                        },
                        {
                            "kind": "tested_by",
                            "from": "SEM-CPS-JUMP-001",
                            "to": "TEST-9001-MUTATION",
                            "anchor": "docs/plan/tasks/TASK-9001-example.md#evidence",
                        },
                        {
                            "kind": "tested_by",
                            "from": "SEM-CPS-JUMP-001",
                            "to": "TEST-9001-PARITY",
                            "anchor": "docs/plan/tasks/TASK-9001-example.md#evidence",
                        },
                    ],
                }
            ),
            encoding="utf-8",
        )

        manifest = root / "docs/plan/semantic-task-records.json"
        manifest.write_text(json.dumps(self.valid_manifest()), encoding="utf-8")
        return manifest

    @staticmethod
    def valid_manifest() -> dict[str, Any]:
        """Return a complete bounded record with all four evidence classes."""
        return {
            "schema": "semantic-task-records/v1",
            "active_scope": {
                "kind": "fixture",
                "tasks": ["TASK-9001"],
            },
            "active_tasks": ["TASK-9001"],
            "records": [
                {
                    "task": "TASK-9001",
                    "task_file": "docs/plan/tasks/TASK-9001-example.md",
                    "coverage_map": (
                        "docs/plan/SEMANTIC-RULE-COVERAGE.md#"
                        "task-9001-workflow-record"
                    ),
                    "canonical_rule_ids": [
                        "SEM-CPS-CALL-001",
                        "SEM-CPS-JUMP-001",
                    ],
                    "domain": {
                        "status": "bounded",
                        "description": "One closed local-call fixture.",
                    },
                    "layers": {
                        "type": "bounded",
                        "core": "bounded",
                        "cps": "bounded",
                        "admission_runtime": "bounded",
                        "verification": "bounded",
                    },
                    "evidence": {
                        "positive": ["TEST-9001-POSITIVE"],
                        "negative": ["TEST-9001-NEGATIVE"],
                        "mutation": ["TEST-9001-MUTATION"],
                        "parity": {
                            "status": "not_applicable",
                            "rationale": "The fixture has no reference interpreter pair.",
                        },
                    },
                    "non_goals": ["General call lowering."],
                    "next_obligation": "Admit parameterized local calls.",
                    "verification": [
                        "cargo test -p ash-engine --test task_9001_example"
                    ],
                }
            ],
        }

    @staticmethod
    def traceability_graph(root: Path) -> tuple[Path, dict[str, Any]]:
        """Load the synthetic traceability graph for one evidence mutation."""
        traceability = root / "docs/spec/SEMANTIC-TRACEABILITY.json"
        graph = json.loads(traceability.read_text(encoding="utf-8"))
        assert isinstance(graph, dict)
        return traceability, graph

    @staticmethod
    def write_traceability_graph(path: Path, graph: dict[str, Any]) -> None:
        """Persist one deliberately-mutated synthetic traceability graph."""
        path.write_text(json.dumps(graph), encoding="utf-8")

    def declare_covered_parity(self, payload: dict[str, Any], root: Path) -> None:
        """Turn the fixture's no-reference parity case into an owned parity check."""
        evidence = payload["records"][0]["evidence"]
        assert isinstance(evidence, dict)
        evidence["parity"] = {
            "status": "covered",
            "evidence": ["TEST-9001-PARITY"],
        }
        coverage_map = root / "docs/plan/SEMANTIC-RULE-COVERAGE.md"
        coverage_map.write_text(
            coverage_map.read_text(encoding="utf-8").replace(
                "- **Parity:** not applicable; this fixture has no reference interpreter pair.\n",
                "- **Parity:** `TEST-9001-PARITY`\n",
            ),
            encoding="utf-8",
        )

    def declared_evidence_id(
        self, payload: dict[str, Any], category: str
    ) -> str:
        """Return one active evidence ID, including a covered parity declaration."""
        evidence = payload["records"][0]["evidence"]
        assert isinstance(evidence, dict)
        if category == "parity":
            parity = evidence["parity"]
            assert isinstance(parity, dict)
            values = parity["evidence"]
        else:
            values = evidence[category]
        assert isinstance(values, list) and len(values) == 1
        assert isinstance(values[0], str)
        return values[0]

    def replace_declared_evidence_id(
        self,
        payload: dict[str, Any],
        root: Path,
        category: str,
        replacement: str,
    ) -> str:
        """Replace one declaration and its coverage-map mention without graph edits."""
        original = self.declared_evidence_id(payload, category)
        evidence = payload["records"][0]["evidence"]
        assert isinstance(evidence, dict)
        if category == "parity":
            parity = evidence["parity"]
            assert isinstance(parity, dict)
            parity["evidence"] = [replacement]
        else:
            evidence[category] = [replacement]
        coverage_map = root / "docs/plan/SEMANTIC-RULE-COVERAGE.md"
        coverage_map.write_text(
            coverage_map.read_text(encoding="utf-8").replace(original, replacement),
            encoding="utf-8",
        )
        return original

    def run_validator(
        self, root: Path, manifest: Path
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, Any]]:
        self.assertTrue(
            TOOL.exists(), f"missing TASK-2028 semantic-task validator: {TOOL}"
        )
        result = subprocess.run(
            ["python3", str(TOOL), "--root", str(root), "--manifest", str(manifest)],
            check=False,
            capture_output=True,
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

    def run_raw_validator(
        self, *arguments: str
    ) -> subprocess.CompletedProcess[str]:
        """Run the CLI without assuming it received a normal validation request."""
        return subprocess.run(
            ["python3", str(TOOL), *arguments],
            check=False,
            capture_output=True,
            text=True,
        )

    def run_mutation(
        self, mutate: Any
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, Any]]:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory) / "fixture"
            manifest = self.write_valid_fixture(root)
            payload = json.loads(manifest.read_text(encoding="utf-8"))
            mutate(payload, root)
            manifest.write_text(json.dumps(payload), encoding="utf-8")
            return self.run_validator(root, manifest)

    def assert_mutation_rejected(self, mutate: Any, kind: str) -> None:
        result, report = self.run_mutation(mutate)
        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report.get("schema"), REPORT_SCHEMA)
        errors = report.get("errors")
        self.assertIsInstance(errors, list)
        self.assertTrue(
            any(
                isinstance(error, dict) and error.get("kind") == kind
                for error in errors
            ),
            errors,
        )

    def test_complete_bounded_record_with_canonical_links_is_accepted(self) -> None:
        """A task may declare one bounded slice with explicit workflow evidence."""
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory) / "fixture"
            manifest = self.write_valid_fixture(root)
            result, report = self.run_validator(root, manifest)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report, {"schema": REPORT_SCHEMA, "errors": []})

    def test_required_workflow_fields_reject_when_missing(self) -> None:
        """A record cannot omit layer, evidence, scope, or next-step accountability."""
        for field in ("layers", "evidence", "non_goals", "next_obligation"):
            with self.subTest(field=field):
                def remove_required_field(payload: dict[str, Any], _root: Path) -> None:
                    records = payload["records"]
                    assert isinstance(records, list)
                    records[0].pop(field)

                self.assert_mutation_rejected(
                    remove_required_field, "missing_required_field"
                )

    def test_unknown_canonical_rule_is_rejected(self) -> None:
        """Rule IDs must resolve to canonical-rule nodes in traceability."""
        def replace_rule(payload: dict[str, Any], _root: Path) -> None:
            records = payload["records"]
            assert isinstance(records, list)
            records[0]["canonical_rule_ids"] = ["SEM-NOT-DECLARED-001"]

        self.assert_mutation_rejected(replace_rule, "unknown_canonical_rule")

    def test_task_and_coverage_links_are_required(self) -> None:
        """The record must name both its human task file and its coverage-map row."""
        for field, kind in (
            ("task_file", "missing_task_file_link"),
            ("coverage_map", "missing_coverage_map_link"),
        ):
            with self.subTest(field=field):
                def remove_link(payload: dict[str, Any], _root: Path) -> None:
                    records = payload["records"]
                    assert isinstance(records, list)
                    records[0].pop(field)

                self.assert_mutation_rejected(remove_link, kind)

    def test_task_file_links_its_record_and_declares_its_bounded_identity(self) -> None:
        """Human task prose must not drift away from the machine-owned record."""
        def remove_manifest_link(payload: dict[str, Any], root: Path) -> None:
            task_file = root / payload["records"][0]["task_file"]
            task_file.write_text(
                task_file.read_text(encoding="utf-8").replace(
                    "**Semantic task record:** "
                    "[TASK-9001](../semantic-task-records.json)\n\n",
                    "",
                ),
                encoding="utf-8",
            )

        def replace_manifest_link_with_prose(
            payload: dict[str, Any], root: Path
        ) -> None:
            task_file = root / payload["records"][0]["task_file"]
            task_file.write_text(
                task_file.read_text(encoding="utf-8").replace(
                    "[TASK-9001](../semantic-task-records.json)",
                    "docs/plan/semantic-task-records.json",
                ),
                encoding="utf-8",
            )

        def change_declared_domain(payload: dict[str, Any], root: Path) -> None:
            task_file = root / payload["records"][0]["task_file"]
            task_file.write_text(
                task_file.read_text(encoding="utf-8").replace(
                    "**Declared domain:** bounded",
                    "**Declared domain:** general",
                ),
                encoding="utf-8",
            )

        for mutate, kind in (
            (remove_manifest_link, "missing_task_manifest_link"),
            (replace_manifest_link_with_prose, "missing_task_manifest_link"),
            (change_declared_domain, "task_domain_mismatch"),
        ):
            with self.subTest(kind=kind):
                self.assert_mutation_rejected(mutate, kind)

    def test_task_file_links_its_exact_coverage_map_fragment(self) -> None:
        """Task prose must let reviewers navigate directly to its owned map row."""
        def remove_coverage_map_link(payload: dict[str, Any], root: Path) -> None:
            task_file = root / payload["records"][0]["task_file"]
            task_file.write_text(
                task_file.read_text(encoding="utf-8").replace(
                    "**Semantic coverage map:** "
                    "[TASK-9001 workflow record](../SEMANTIC-RULE-COVERAGE.md#"
                    "task-9001-workflow-record)\n\n",
                    "",
                ),
                encoding="utf-8",
            )

        self.assert_mutation_rejected(
            remove_coverage_map_link, "missing_task_coverage_map_link"
        )

    def test_coverage_heading_declares_task_rules_and_matching_layer_statuses(self) -> None:
        """The map heading is a reviewable summary, not merely an arbitrary anchor."""
        def replace_heading(
            payload: dict[str, Any],
            root: Path,
            heading: str,
            fragment: str,
        ) -> None:
            coverage_map = root / "docs/plan/SEMANTIC-RULE-COVERAGE.md"
            coverage_map.write_text(
                "# Semantic Rule Coverage Map\n\n## " + heading + "\n",
                encoding="utf-8",
            )
            payload["records"][0]["coverage_map"] = (
                "docs/plan/SEMANTIC-RULE-COVERAGE.md#" + fragment
            )

        def remove_task_id(payload: dict[str, Any], root: Path) -> None:
            replace_heading(
                payload,
                root,
                "SEM-CPS-CALL-001 type bounded core bounded cps bounded "
                "admission-runtime bounded verification bounded",
                "sem-cps-call-001-type-bounded-core-bounded-cps-bounded-"
                "admission-runtime-bounded-verification-bounded",
            )

        def replace_canonical_rule(payload: dict[str, Any], root: Path) -> None:
            replace_heading(
                payload,
                root,
                "TASK-9001 SEM-CPS-JUMP-001 type bounded core bounded cps bounded "
                "admission-runtime bounded verification bounded",
                "task-9001-sem-cps-jump-001-type-bounded-core-bounded-cps-bounded-"
                "admission-runtime-bounded-verification-bounded",
            )

        def change_layer_status(payload: dict[str, Any], root: Path) -> None:
            replace_heading(
                payload,
                root,
                "TASK-9001 SEM-CPS-CALL-001 type bounded core bounded cps general "
                "admission-runtime bounded verification bounded",
                "task-9001-sem-cps-call-001-type-bounded-core-bounded-cps-general-"
                "admission-runtime-bounded-verification-bounded",
            )

        for mutate, kind in (
            (remove_task_id, "coverage_heading_missing_task"),
            (replace_canonical_rule, "coverage_heading_missing_rule"),
            (change_layer_status, "coverage_layer_mismatch"),
        ):
            with self.subTest(kind=kind):
                self.assert_mutation_rejected(mutate, kind)

    def test_coverage_section_requires_the_complete_task_workflow_summary(self) -> None:
        """The selected map row must expose reviewable task-owned workflow facts."""
        def replace_coverage_text(
            payload: dict[str, Any], root: Path, old: str, new: str
        ) -> None:
            coverage_map = root / "docs/plan/SEMANTIC-RULE-COVERAGE.md"
            text = coverage_map.read_text(encoding="utf-8")
            self.assertIn(old, text)
            coverage_map.write_text(text.replace(old, new), encoding="utf-8")

        mutations: tuple[tuple[str, str, str], ...] = (
            ("**Task:** [TASK-9001](tasks/TASK-9001-example.md)", "**Task:** TASK-9001", "coverage_task_link_missing"),
            ("**Domain:** bounded", "**Domain:** general", "coverage_domain_mismatch"),
            ("core bounded", "core general", "coverage_layer_mismatch"),
            ("- **Positive:** `TEST-9001-POSITIVE`\n", "", "coverage_evidence_mismatch"),
            ("- **Negative:** `TEST-9001-NEGATIVE`\n", "", "coverage_evidence_mismatch"),
            ("- **Mutation:** `TEST-9001-MUTATION`\n", "", "coverage_evidence_mismatch"),
            (
                "- **Parity:** not applicable; this fixture has no reference interpreter pair.\n",
                "",
                "coverage_evidence_mismatch",
            ),
            ("**Non-goals:** General call lowering.\n", "", "coverage_non_goals_missing"),
            (
                "**Next obligation:** Admit parameterized local calls.\n",
                "",
                "coverage_next_obligation_missing",
            ),
        )
        for old, new, kind in mutations:
            with self.subTest(kind=kind, old=old):
                def mutate(payload: dict[str, Any], root: Path) -> None:
                    replace_coverage_text(payload, root, old, new)

                self.assert_mutation_rejected(mutate, kind)

    def test_task_identity_duplicates_and_active_task_set_are_checked(self) -> None:
        """Records are one-to-one with task files and the declared active task set."""
        def disagree_with_task_heading(payload: dict[str, Any], root: Path) -> None:
            task_file = root / payload["records"][0]["task_file"]
            task_file.write_text(
                task_file.read_text(encoding="utf-8").replace(
                    "# TASK-9001:", "# TASK-9002:"
                ),
                encoding="utf-8",
            )

        def duplicate_task(payload: dict[str, Any], _root: Path) -> None:
            records = payload["records"]
            assert isinstance(records, list)
            records.append(dict(records[0]))

        def omit_declared_active_set(payload: dict[str, Any], _root: Path) -> None:
            payload.pop("active_tasks")

        def add_unrecorded_active_task(payload: dict[str, Any], _root: Path) -> None:
            payload["active_tasks"] = ["TASK-9001", "TASK-9002"]

        for mutate, kind in (
            (disagree_with_task_heading, "task_heading_mismatch"),
            (duplicate_task, "duplicate_task"),
            (omit_declared_active_set, "missing_active_tasks"),
            (add_unrecorded_active_task, "active_task_set_mismatch"),
        ):
            with self.subTest(kind=kind):
                self.assert_mutation_rejected(mutate, kind)

    def test_active_scope_requires_exact_headings_and_in_progress_task_status(self) -> None:
        """Active task records cannot outlive their explicit workflow ownership."""
        def remove_active_scope(payload: dict[str, Any], _root: Path) -> None:
            payload.pop("active_scope")

        def use_incomplete_task_1988_scope(
            payload: dict[str, Any], _root: Path
        ) -> None:
            payload["active_scope"] = {
                "kind": "task-1988-followups",
                "tasks": [
                    "TASK-2001",
                    "TASK-2002",
                    "TASK-2003",
                    "TASK-2004",
                    "TASK-2005",
                    "TASK-2008",
                    "TASK-2013",
                    "TASK-2014",
                ],
            }

        def close_active_task(payload: dict[str, Any], root: Path) -> None:
            task_file = root / payload["records"][0]["task_file"]
            task_file.write_text(
                task_file.read_text(encoding="utf-8").replace(
                    "**Status:** In progress", "**Status:** Complete"
                ),
                encoding="utf-8",
            )

        def relax_task_heading(payload: dict[str, Any], root: Path) -> None:
            task_file = root / payload["records"][0]["task_file"]
            task_file.write_text(
                task_file.read_text(encoding="utf-8").replace(
                    "# TASK-9001: Example semantic workflow record",
                    "# TASK-9001 Example semantic workflow record",
                ),
                encoding="utf-8",
            )

        for mutate, kind in (
            (remove_active_scope, "missing_active_scope"),
            (use_incomplete_task_1988_scope, "active_scope_task_set_mismatch"),
            (close_active_task, "active_task_status_mismatch"),
            (relax_task_heading, "task_heading_mismatch"),
        ):
            with self.subTest(kind=kind):
                self.assert_mutation_rejected(mutate, kind)

    def test_task_1988_followups_scope_rejects_general_domain_records(self) -> None:
        """TASK-1988 follow-ups stay explicitly bounded across machine and prose records."""
        def declare_general_domain(payload: dict[str, Any], root: Path) -> None:
            payload["active_scope"] = {
                "kind": "task-1988-followups",
                "tasks": [
                    "TASK-439",
                    "TASK-2001",
                    "TASK-2002",
                    "TASK-2003",
                    "TASK-2004",
                    "TASK-2005",
                    "TASK-2008",
                    "TASK-2013",
                    "TASK-2014",
                ],
            }
            payload["records"][0]["domain"]["status"] = "general"

            task_file = root / payload["records"][0]["task_file"]
            task_file.write_text(
                task_file.read_text(encoding="utf-8").replace(
                    "**Declared domain:** bounded",
                    "**Declared domain:** general",
                ),
                encoding="utf-8",
            )

            coverage_map = root / "docs/plan/SEMANTIC-RULE-COVERAGE.md"
            coverage_map.write_text(
                coverage_map.read_text(encoding="utf-8").replace(
                    "**Domain:** bounded",
                    "**Domain:** general",
                ),
                encoding="utf-8",
            )

        self.assert_mutation_rejected(
            declare_general_domain,
            "task_1988_followups_domain_must_be_bounded",
        )

    def test_v1_schema_rejects_unknown_fields_at_every_controlled_level(self) -> None:
        """Schema v1 must fail closed instead of silently ignoring governance data."""
        def add_root_field(payload: dict[str, Any], _root: Path) -> None:
            payload["unexpected"] = True

        def add_record_field(payload: dict[str, Any], _root: Path) -> None:
            payload["records"][0]["unexpected"] = True

        def add_domain_field(payload: dict[str, Any], _root: Path) -> None:
            payload["records"][0]["domain"]["unexpected"] = True

        def add_layers_field(payload: dict[str, Any], _root: Path) -> None:
            payload["records"][0]["layers"]["unexpected"] = "bounded"

        def add_evidence_field(payload: dict[str, Any], _root: Path) -> None:
            payload["records"][0]["evidence"]["unexpected"] = ["TEST-UNEXPECTED"]

        def add_parity_field(payload: dict[str, Any], _root: Path) -> None:
            payload["records"][0]["evidence"]["parity"]["unexpected"] = True

        for mutate, kind in (
            (add_root_field, "unknown_manifest_field"),
            (add_record_field, "unknown_record_field"),
            (add_domain_field, "unknown_domain_field"),
            (add_layers_field, "unknown_layers_field"),
            (add_evidence_field, "unknown_evidence_field"),
            (add_parity_field, "unknown_parity_field"),
        ):
            with self.subTest(kind=kind):
                self.assert_mutation_rejected(mutate, kind)

    def test_traceability_requires_task_anchored_evidence_for_every_rule(self) -> None:
        """Canonical-rule evidence must remain owned by the record's task file."""
        def remove_jump_rule_edge(payload: dict[str, Any], root: Path) -> None:
            traceability = root / "docs/spec/SEMANTIC-TRACEABILITY.json"
            graph = json.loads(traceability.read_text(encoding="utf-8"))
            graph["edges"] = [
                edge
                for edge in graph["edges"]
                if edge["from"] != "SEM-CPS-JUMP-001"
            ]
            traceability.write_text(json.dumps(graph), encoding="utf-8")

        def point_edge_at_a_dangling_task_anchor(
            payload: dict[str, Any], root: Path
        ) -> None:
            traceability = root / "docs/spec/SEMANTIC-TRACEABILITY.json"
            graph = json.loads(traceability.read_text(encoding="utf-8"))
            for edge in graph["edges"]:
                if edge["from"] == "SEM-CPS-CALL-001":
                    edge["anchor"] = "docs/plan/tasks/TASK-9999-missing.md#evidence"
            traceability.write_text(json.dumps(graph), encoding="utf-8")

        for mutate, kind in (
            (remove_jump_rule_edge, "missing_task_traceability_edge"),
            (point_edge_at_a_dangling_task_anchor, "task_traceability_anchor_mismatch"),
        ):
            with self.subTest(kind=kind):
                self.assert_mutation_rejected(mutate, kind)

    def test_every_declared_evidence_id_requires_a_test_traceability_node(self) -> None:
        """All evidence classes must name test nodes, including covered parity evidence."""
        for category in ("positive", "negative", "mutation", "parity"):
            with self.subTest(category=category, mutation="missing-node"):
                def replace_with_missing_node(
                    payload: dict[str, Any], root: Path, category: str = category
                ) -> None:
                    if category == "parity":
                        self.declare_covered_parity(payload, root)
                    self.replace_declared_evidence_id(
                        payload,
                        root,
                        category,
                        f"TEST-9001-MISSING-{category.upper()}",
                    )

                self.assert_mutation_rejected(
                    replace_with_missing_node, "unknown_evidence_node"
                )

            with self.subTest(category=category, mutation="wrong-node-kind"):
                def make_declared_node_non_test(
                    payload: dict[str, Any], root: Path, category: str = category
                ) -> None:
                    if category == "parity":
                        self.declare_covered_parity(payload, root)
                    evidence_id = self.declared_evidence_id(payload, category)
                    traceability, graph = self.traceability_graph(root)
                    nodes = graph["nodes"]
                    assert isinstance(nodes, list)
                    for node in nodes:
                        if isinstance(node, dict) and node.get("id") == evidence_id:
                            node["kind"] = "canonical-rule"
                            break
                    else:
                        self.fail(f"fixture graph lacks {evidence_id}")
                    self.write_traceability_graph(traceability, graph)

                self.assert_mutation_rejected(
                    make_declared_node_non_test, "evidence_node_not_test"
                )

    def test_every_declared_evidence_id_requires_a_task_anchored_tested_by_edge(self) -> None:
        """Evidence must be a test owned by one declared rule and this task heading."""
        def mutate_evidence_edge(
            payload: dict[str, Any],
            root: Path,
            category: str,
            change: Any,
        ) -> None:
            if category == "parity":
                self.declare_covered_parity(payload, root)
            evidence_id = self.declared_evidence_id(payload, category)
            traceability, graph = self.traceability_graph(root)
            edges = graph["edges"]
            assert isinstance(edges, list)
            matching_edges = [
                edge
                for edge in edges
                if isinstance(edge, dict) and edge.get("to") == evidence_id
            ]
            self.assertEqual(len(matching_edges), 1, evidence_id)
            change(matching_edges[0], graph, root)
            self.write_traceability_graph(traceability, graph)

        def wrong_edge_kind(edge: dict[str, Any], _graph: dict[str, Any], _root: Path) -> None:
            edge["kind"] = "implements"

        def wrong_target(edge: dict[str, Any], _graph: dict[str, Any], _root: Path) -> None:
            edge["to"] = "TEST-9001-UNRELATED"

        def wrong_rule(edge: dict[str, Any], graph: dict[str, Any], _root: Path) -> None:
            nodes = graph["nodes"]
            assert isinstance(nodes, list)
            nodes.append(
                {
                    "id": "SEM-OTHER-001",
                    "kind": "canonical-rule",
                    "status": ["specified"],
                    "anchor": "docs/spec/CANONICAL-CORE.md#other",
                }
            )
            edge["from"] = "SEM-OTHER-001"

        def wrong_task_file_anchor(
            edge: dict[str, Any], _graph: dict[str, Any], root: Path
        ) -> None:
            other_task = root / "docs/plan/tasks/TASK-9002-other.md"
            other_task.write_text(
                "# TASK-9002: Other record\n\n## Evidence\n",
                encoding="utf-8",
            )
            edge["anchor"] = "docs/plan/tasks/TASK-9002-other.md#evidence"

        mutations: tuple[tuple[str, Any, str], ...] = (
            ("wrong-edge-kind", wrong_edge_kind, "missing_evidence_tested_by_edge"),
            ("wrong-target", wrong_target, "missing_evidence_tested_by_edge"),
            ("wrong-rule", wrong_rule, "missing_evidence_tested_by_edge"),
            (
                "wrong-task-file-anchor",
                wrong_task_file_anchor,
                "evidence_task_traceability_anchor_mismatch",
            ),
        )
        for category in ("positive", "negative", "mutation", "parity"):
            for mutation, change, kind in mutations:
                with self.subTest(category=category, mutation=mutation):
                    def mutate(
                        payload: dict[str, Any],
                        root: Path,
                        category: str = category,
                        change: Any = change,
                    ) -> None:
                        mutate_evidence_edge(payload, root, category, change)

                    self.assert_mutation_rejected(mutate, kind)

    def test_task_traceability_anchor_fragment_must_resolve_to_a_heading(self) -> None:
        """A task-owned traceability edge cannot name a nonexistent task fragment."""
        def point_edge_at_missing_task_heading(
            payload: dict[str, Any], root: Path
        ) -> None:
            traceability = root / "docs/spec/SEMANTIC-TRACEABILITY.json"
            graph = json.loads(traceability.read_text(encoding="utf-8"))
            graph["edges"][0]["anchor"] = (
                f"{payload['records'][0]['task_file']}#missing-evidence"
            )
            traceability.write_text(json.dumps(graph), encoding="utf-8")

        self.assert_mutation_rejected(
            point_edge_at_missing_task_heading,
            "missing_task_traceability_anchor_heading",
        )

    def test_task_traceability_anchor_requires_its_evidence_heading(self) -> None:
        """Existing task anchors fail when their target heading is removed."""
        def remove_evidence_heading(payload: dict[str, Any], root: Path) -> None:
            task_file = root / payload["records"][0]["task_file"]
            task_file.write_text(
                task_file.read_text(encoding="utf-8").replace("## Evidence\n\n", ""),
                encoding="utf-8",
            )

        self.assert_mutation_rejected(
            remove_evidence_heading,
            "missing_task_traceability_anchor_heading",
        )

    def test_verification_requires_a_matching_task_owned_integration_target(self) -> None:
        """Documentation checks cannot replace the task's Cargo integration evidence."""
        for command in (
            "python3 tools/docs/validate_semantic_task_records.py --self-test",
            "cargo test -p ash-engine --test task_9002_example",
        ):
            with self.subTest(command=command):
                def replace_verification(
                    payload: dict[str, Any], _root: Path
                ) -> None:
                    payload["records"][0]["verification"] = [command]

                self.assert_mutation_rejected(
                    replace_verification, "missing_task_owned_integration_test"
                )

    def test_verification_commands_are_narrow_and_shell_safe(self) -> None:
        """A record's focused Cargo command remains shell-free and directly executable."""
        allowed = (
            "cargo test -p ash-engine --test task_9001_example",
        )
        for command in allowed:
            with self.subTest(allowed=command):
                def set_allowed_command(payload: dict[str, Any], _root: Path) -> None:
                    records = payload["records"]
                    assert isinstance(records, list)
                    records[0]["verification"] = [command]

                with tempfile.TemporaryDirectory() as temporary_directory:
                    root = Path(temporary_directory) / "fixture"
                    manifest = self.write_valid_fixture(root)
                    payload = json.loads(manifest.read_text(encoding="utf-8"))
                    set_allowed_command(payload, root)
                    manifest.write_text(json.dumps(payload), encoding="utf-8")
                    result, report = self.run_validator(root, manifest)

                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(report, {"schema": REPORT_SCHEMA, "errors": []})

        rejected = (
            "cargo check --workspace",
            "cargo test -p ash-engine --no-run --test task_9001_example",
            "cargo test -p ash-engine",
            "cargo test -p ash-engine --test",
            "cargo test -p ash-engine --test task_9001_example hidden_filter",
            "cargo test -p ash-engine --test task_9001_example && touch escaped",
            "cargo test -p ash-engine --test task_9001_example > output.txt",
            "python3 tools/docs/validate_semantic_task_records.py $(pwd)",
            "python3 tools/docs/arbitrary.py",
            "python3 tools/docs/validate_semantic_task_records.py",
            "bash scripts/check-docs-gate.sh; true",
            "bash scripts/arbitrary.sh",
            "bash scripts/check-docs-gate.sh unexpected",
            "bash /tmp/check-docs-gate.sh",
            "bash scripts/../check-docs-gate.sh",
            "/bin/bash scripts/check-docs-gate.sh",
            "cargo test -p ash-engine\n--test task_9001_example",
        )
        for command in rejected:
            with self.subTest(rejected=command):
                def set_unsafe_command(payload: dict[str, Any], _root: Path) -> None:
                    records = payload["records"]
                    assert isinstance(records, list)
                    records[0]["verification"] = [command]

                self.assert_mutation_rejected(
                    set_unsafe_command, "unsafe_verification_command"
                )

    def test_task_2031_documentation_contract_target_is_narrowly_allowlisted(self) -> None:
        """TASK-2031 may verify its validator handoff without opening a general docs escape."""
        command = "python3 -m unittest tools.docs.tests.test_validate_ash_cps_calculus"
        self.assertTrue(allowed_verification_command(command))
        self.assertTrue(command_matches_task_integration_test(command, "TASK-2031"))
        self.assertFalse(command_matches_task_integration_test(command, "TASK-2014"))
        self.assertFalse(allowed_verification_command("python3 -m unittest tools.docs.tests.test_validate_semantic_task_records"))

    def test_task_2031_scope_keeps_inherited_records_bounded(self) -> None:
        """The general TASK-2031 handoff cannot relax pre-existing bounded records."""
        tasks = sorted(TASK_2031_PREREQUISITE_SCOPE)
        records = [
            {"task": task, "domain": {"status": "general" if task == "TASK-2031" else "bounded"}}
            for task in tasks
        ]
        payload = {"active_scope": {"kind": "task-2031-prerequisite", "tasks": tasks}}
        errors: list[dict[str, object]] = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertEqual(errors, [])

        records[0]["domain"] = {"status": "general"}
        errors = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertTrue(any(error.get("kind") == "task_2031_prerequisite_domain_mismatch" for error in errors), errors)

    def test_help_emits_a_stable_json_report(self) -> None:
        """Even help output must preserve the validator's machine-readable stdout contract."""
        result = self.run_raw_validator("--help")
        try:
            report = json.loads(result.stdout)
        except json.JSONDecodeError as error:
            self.fail(
                "--help must emit only a JSON semantic-task validation report: "
                f"{error}; stderr: {result.stderr}; stdout: {result.stdout!r}"
            )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report.get("schema"), REPORT_SCHEMA)
        self.assertEqual(report.get("errors"), [])
        self.assertIsInstance(report.get("help"), str)
        self.assertTrue(report["help"].strip())


if __name__ == "__main__":
    unittest.main()
