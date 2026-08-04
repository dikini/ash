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
    CLOSED_SEMANTIC_HANDOFF_TASKS,
    TASK_2075_TWO_TIER_MODULE_COLLECTION_SCOPE,
    TASK_2074_CANONICAL_EXPANDED_MODULE_GRAPH_SCOPE,
    TASK_2071_MODULE_NAMESPACE_CONTRACT_SCOPE,
    TASK_2070_SCOPED_SELF_SIMPLE_FUNCTION_ALIASES_SCOPE,
    TASK_2068_FINAL_INTERFACES_PARSED_IMPORTS_BINDER_SCOPE,
    TASK_2067_CANONICAL_MODULE_GRAPH_SCOPE,
    TASK_2063_ENGINE_LINKED_MODULE_ADMISSION_SCOPE,
    TASK_2062_MODULE_AWARE_CORE_CPS_LOWERING_SCOPE,
    TASK_2041_ENGINE_ONLY_CLOSEOUT_SCOPE,
    TASK_2040_ENGINE_ONLY_REMOVAL_SCOPE,
    TASK_2066_TYPEENV_MODULE_UNIT_INTERFACE_FINALIZATION_SCOPE,
    TASK_2061_INTERFACE_IMPORT_RESOLUTION_SCOPE,
    TASK_2060_CHECKED_MODULE_INTERFACE_SCOPE,
    TASK_2059_FILE_INLINE_MODULE_UNIT_PARITY_SCOPE,
    TASK_2058_CANONICAL_MODULE_IDENTITY_SCOPE,
    TASK_2057_MODULE_DISCOVERY_SCOPE,
    TASK_2039_REPL_SCOPE,
    TASK_2037_ENGINE_CPS_SCOPE,
    TASK_2032_INTEGRATION_SCOPE,
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
        """Create the smallest repository that owns one active semantic task."""
        task_file = root / "docs/plan/tasks/TASK-9001-example.md"
        task_file.parent.mkdir(parents=True)
        task_file.write_text(
            "# TASK-9001: Example semantic workflow record\n\n"
            "This fixture is linked from its machine-readable record.\n\n"
            "**Status:** In progress\n\n"
            "**Semantic task record:** "
            "[TASK-9001](../semantic-task-records.json)\n\n"
            "**Semantic coverage map:** "
            "[TASK-9001 workflow record](../SEMANTIC-RULE-COVERAGE.md#"
            "task-9001-workflow-record)\n\n"
            "**Implementation:** partial\n"
            "**Evidence:** tested\n"
            "**Parity:** below_spec\n"
            "**Missing target-spec clauses:**\n"
            "- Calls with parameters are not admitted.\n\n"
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
            "**Implementation:** partial\n"
            "**Evidence:** tested\n"
            "**Parity:** below_spec\n"
            "**Missing target-spec clauses:**\n"
            "- Calls with parameters are not admitted.\n"
            "**Layers:** type partial; core partial; cps partial; "
            "admission-runtime partial; verification partial.\n"
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
                    "schema": "semantic-traceability-graph/v2",
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
        """Return a valid target-spec record with all four evidence classes."""
        return {
            "schema": "semantic-task-records/v2",
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
                    "implementation": "partial",
                    "parity": "below_spec",
                    "missing_spec_clauses": ["Calls with parameters are not admitted."],
                    "layers": {
                        "type": "partial",
                        "core": "partial",
                        "cps": "partial",
                        "admission_runtime": "partial",
                        "verification": "partial",
                    },
                    "evidence": {
                        "status": "tested",
                        "proofs": [],
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

    def assert_mutation_accepted(self, mutate: Any) -> None:
        """Require a synthetic v2 record to pass the fail-closed validator."""
        result, report = self.run_mutation(mutate)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report, {"schema": REPORT_SCHEMA, "errors": []})

    @staticmethod
    def declare_target_spec_status(
        payload: dict[str, Any],
        *,
        implementation: str = "partial",
        evidence: str = "tested",
        parity: str = "below_spec",
    ) -> None:
        """Declare the three target-spec report axes on the fixture record."""
        record = payload["records"][0]
        assert isinstance(record, dict)
        record["implementation"] = implementation
        record["parity"] = parity
        record_evidence = record["evidence"]
        assert isinstance(record_evidence, dict)
        record_evidence["status"] = evidence

    @staticmethod
    def replace_status_block(
        payload: dict[str, Any], root: Path, *, implementation: str, evidence: str
    ) -> None:
        """Synchronize the fixture's task and coverage report-axis blocks."""
        task_file = root / payload["records"][0]["task_file"]
        coverage_map = root / "docs/plan/SEMANTIC-RULE-COVERAGE.md"
        for path in (task_file, coverage_map):
            path.write_text(
                path.read_text(encoding="utf-8")
                .replace("**Implementation:** partial", f"**Implementation:** {implementation}")
                .replace("**Evidence:** tested", f"**Evidence:** {evidence}"),
                encoding="utf-8",
            )

    def declare_proved_evidence(
        self, payload: dict[str, Any], root: Path, *, include_edge: bool
    ) -> None:
        """Turn the fixture into a proved record with one checked proof witness."""
        record = payload["records"][0]
        assert isinstance(record, dict)
        record["evidence"] = {
            "status": "proved",
            "proofs": ["PROOF-9001-CALL-001"],
            "positive": [],
            "negative": [],
            "mutation": [],
            "parity": {
                "status": "not_applicable",
                "rationale": "This proof fixture has no reference interpreter pair.",
            },
        }
        self.replace_status_block(payload, root, implementation="partial", evidence="proved")
        coverage_map = root / "docs/plan/SEMANTIC-RULE-COVERAGE.md"
        coverage_map.write_text(
            coverage_map.read_text(encoding="utf-8").replace(
                "- **Positive:** `TEST-9001-POSITIVE`\n"
                "- **Negative:** `TEST-9001-NEGATIVE`\n"
                "- **Mutation:** `TEST-9001-MUTATION`\n",
                "- **Proof:** `PROOF-9001-CALL-001`\n",
            ),
            encoding="utf-8",
        )
        traceability, graph = self.traceability_graph(root)
        nodes = graph["nodes"]
        edges = graph["edges"]
        assert isinstance(nodes, list) and isinstance(edges, list)
        nodes.extend((
            {
                "id": "IMPL-9001-CALL-REFINEMENT",
                "kind": "implementation",
                "status": ["implemented"],
                "public_semantic": False,
                "symbol": "ash_engine::fixture::call_refinement",
                "source_fingerprint": "sha256:fixture-call-refinement",
                "anchor": "tools/docs/tests/test_validate_semantic_task_records.py#semantic-task-record-contract-tests",
            },
            {
                "id": "SEM-CPS-CALL-MODEL-001",
                "kind": "model",
                "status": ["modelled"],
                "anchor": "tools/docs/tests/test_validate_semantic_task_records.py#semantic-task-record-contract-tests",
            },
            {
                "id": "PROOF-9001-CALL-001",
                "kind": "proof",
                "status": ["proved"],
                "anchor": "tools/docs/tests/test_validate_semantic_task_records.py#semantic-task-record-contract-tests",
                "proof": {
                    "provider": "fixture",
                    "tool": "fixture",
                    "tool_version": "1",
                    "options": [],
                    "assumptions": [],
                    "model": "SEM-CPS-CALL-MODEL-001",
                    "implementation_revision": "fixture",
                    "implementation_fingerprint": "sha256:fixture-call-refinement",
                    "artifact_hash": "sha256:fixture-call-proof",
                    "outcome": "verified",
                    "theorem": (
                        "The call-refinement implementation preserves the "
                        "declared local-call result."
                    ),
                    "scope": {
                        "model": "SEM-CPS-CALL-MODEL-001",
                        "proven_rule_ids": ["SEM-CPS-CALL-001"],
                    },
                    "runtime_refinement": {
                        "status": "verified",
                        "implementation": "IMPL-9001-CALL-REFINEMENT",
                        "implementation_fingerprint": "sha256:fixture-call-refinement",
                        "theorem": (
                            "The call-refinement implementation refines the "
                            "declared local-call model."
                        ),
                        "artifact_hash": "sha256:fixture-call-runtime-refinement",
                        "anchor": (
                            "tools/docs/tests/"
                            "test_validate_semantic_task_records.py#"
                            "semantic-task-record-contract-tests"
                        ),
                    },
                },
            },
        ))
        edges.append({
            "kind": "refines",
            "from": "PROOF-9001-CALL-001",
            "to": "SEM-CPS-CALL-MODEL-001",
            "anchor": "tools/docs/tests/test_validate_semantic_task_records.py#semantic-task-record-contract-tests",
        })
        edges.append({
            "kind": "refines",
            "from": "IMPL-9001-CALL-REFINEMENT",
            "to": "SEM-CPS-CALL-MODEL-001",
            "anchor": "tools/docs/tests/test_validate_semantic_task_records.py#semantic-task-record-contract-tests",
        })
        if include_edge:
            edges.append({
                "kind": "proved_by",
                "from": "SEM-CPS-CALL-001",
                "to": "PROOF-9001-CALL-001",
                "anchor": "docs/plan/tasks/TASK-9001-example.md#evidence",
            })
        self.write_traceability_graph(traceability, graph)

    def test_partial_target_spec_record_with_canonical_links_is_accepted(self) -> None:
        """A partial target-spec task record with canonical links is valid."""
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory) / "fixture"
            manifest = self.write_valid_fixture(root)
            result, report = self.run_validator(root, manifest)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report, {"schema": REPORT_SCHEMA, "errors": []})

    def test_target_spec_report_axes_are_required(self) -> None:
        """A feature report must separate implementation, evidence, and parity."""
        def omit_axes(_payload: dict[str, Any], _root: Path) -> None:
            record = _payload["records"][0]
            assert isinstance(record, dict)
            record.pop("implementation")
            record.pop("parity")
            record.pop("missing_spec_clauses")
            evidence = record["evidence"]
            assert isinstance(evidence, dict)
            evidence.pop("status")

        self.assert_mutation_rejected(omit_axes, "missing_target_spec_status_axes")

    def test_retired_implementation_statuses_are_rejected(self) -> None:
        """`bounded` and `general` cannot describe target-spec implementation."""
        for status in ("bounded", "general"):
            with self.subTest(status=status):
                def declare_retired_status(
                    payload: dict[str, Any], _root: Path, status: str = status
                ) -> None:
                    self.declare_target_spec_status(
                        payload, implementation=status
                    )

                self.assert_mutation_rejected(
                    declare_retired_status, "invalid_implementation_status"
                )

    def test_target_spec_axes_reject_inconsistent_claims(self) -> None:
        """Implementation and parity claims require the evidence they assert."""
        def implementation_without_evidence(
            payload: dict[str, Any], _root: Path
        ) -> None:
            self.declare_target_spec_status(
                payload,
                implementation="implemented",
                evidence="none",
                parity="matches_spec",
            )

        def matching_parity_without_implementation(
            payload: dict[str, Any], _root: Path
        ) -> None:
            self.declare_target_spec_status(
                payload,
                implementation="partial",
                evidence="tested",
                parity="matches_spec",
            )

        for mutate, kind in (
            (implementation_without_evidence, "implemented_without_evidence"),
            (matching_parity_without_implementation, "matches_spec_without_implementation"),
        ):
            with self.subTest(kind=kind):
                self.assert_mutation_rejected(mutate, kind)

    def test_implemented_record_cannot_report_below_spec_parity(self) -> None:
        """Implementation status is not complete while target-spec clauses remain missing."""
        def implemented_below_spec(
            payload: dict[str, Any], root: Path
        ) -> None:
            self.declare_target_spec_status(
                payload,
                implementation="implemented",
                evidence="tested",
                parity="below_spec",
            )
            self.replace_status_block(
                payload, root, implementation="implemented", evidence="tested"
            )

        self.assert_mutation_rejected(
            implemented_below_spec, "implemented_below_spec"
        )

    def test_implemented_matching_record_cannot_report_missing_spec_clauses(self) -> None:
        """Target-spec parity has no missing clauses when implementation is complete."""
        def implemented_with_missing_target_spec_clauses(
            payload: dict[str, Any], root: Path
        ) -> None:
            self.declare_target_spec_status(
                payload,
                implementation="implemented",
                evidence="tested",
                parity="matches_spec",
            )
            self.replace_status_block(
                payload, root, implementation="implemented", evidence="tested"
            )
            task_file = root / payload["records"][0]["task_file"]
            coverage_map = root / "docs/plan/SEMANTIC-RULE-COVERAGE.md"
            for path in (task_file, coverage_map):
                path.write_text(
                    path.read_text(encoding="utf-8").replace(
                        "**Parity:** below_spec", "**Parity:** matches_spec"
                    ),
                    encoding="utf-8",
                )

        self.assert_mutation_rejected(
            implemented_with_missing_target_spec_clauses,
            "implemented_with_missing_target_spec_clauses",
        )

    def test_exceeds_spec_parity_requires_a_specification_update(self) -> None:
        """A target-spec status cannot authorize behavior beyond that target."""
        def exceeds_spec(payload: dict[str, Any], root: Path) -> None:
            self.declare_target_spec_status(
                payload,
                implementation="implemented",
                evidence="tested",
                parity="exceeds_spec",
            )
            self.replace_status_block(
                payload, root, implementation="implemented", evidence="tested"
            )

        self.assert_mutation_rejected(
            exceeds_spec, "exceeds_spec_requires_spec_update"
        )

    def test_none_evidence_is_valid_for_a_not_implemented_below_spec_record(self) -> None:
        """Missing implementation evidence is an honest report state, not malformed data."""
        def declare_no_evidence(payload: dict[str, Any], root: Path) -> None:
            record = payload["records"][0]
            assert isinstance(record, dict)
            record["implementation"] = "not_implemented"
            record["parity"] = "below_spec"
            record["evidence"] = {
                "status": "none",
                "proofs": [],
                "positive": [],
                "negative": [],
                "mutation": [],
                "parity": {
                    "status": "not_applicable",
                    "rationale": "No implementation evidence exists.",
                },
            }
            self.replace_status_block(
                payload, root, implementation="not_implemented", evidence="none"
            )

        self.assert_mutation_accepted(declare_no_evidence)

    def test_proved_evidence_requires_a_canonical_proof_edge(self) -> None:
        """A proved status names a proof node owned by the declared canonical rule."""
        self.assert_mutation_accepted(
            lambda payload, root: self.declare_proved_evidence(
                payload, root, include_edge=True
            )
        )
        self.assert_mutation_rejected(
            lambda payload, root: self.declare_proved_evidence(
                payload, root, include_edge=False
            ),
            "missing_evidence_proved_by_edge",
        )

    def test_proved_evidence_rejects_a_deferred_proof(self) -> None:
        """A proof edge alone cannot make a deferred proof evidence for a proved record."""
        def defer_proof(payload: dict[str, Any], root: Path) -> None:
            self.declare_proved_evidence(payload, root, include_edge=True)
            traceability, graph = self.traceability_graph(root)
            nodes = graph["nodes"]
            assert isinstance(nodes, list)
            proof = next(node for node in nodes if node["id"] == "PROOF-9001-CALL-001")
            assert isinstance(proof, dict)
            proof["status"] = ["deferred"]
            metadata = proof["proof"]
            assert isinstance(metadata, dict)
            metadata["outcome"] = "deferred"
            self.write_traceability_graph(traceability, graph)

        self.assert_mutation_rejected(
            defer_proof, "proved_evidence_not_verified"
        )

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

    def test_task_file_links_its_record_and_declares_its_target_spec_status(self) -> None:
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

        def change_implementation_status(payload: dict[str, Any], root: Path) -> None:
            task_file = root / payload["records"][0]["task_file"]
            task_file.write_text(
                task_file.read_text(encoding="utf-8").replace(
                    "**Implementation:** partial",
                    "**Implementation:** implemented",
                ),
                encoding="utf-8",
            )

        for mutate, kind in (
            (remove_manifest_link, "missing_task_manifest_link"),
            (replace_manifest_link_with_prose, "missing_task_manifest_link"),
            (change_implementation_status, "task_target_spec_status_block_mismatch"),
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
                "SEM-CPS-CALL-001 type partial core partial cps partial "
                "admission-runtime partial verification partial",
                "sem-cps-call-001-type-partial-core-partial-cps-partial-"
                "admission-runtime-partial-verification-partial",
            )

        def replace_canonical_rule(payload: dict[str, Any], root: Path) -> None:
            replace_heading(
                payload,
                root,
                "TASK-9001 SEM-CPS-JUMP-001 type partial core partial cps partial "
                "admission-runtime partial verification partial",
                "task-9001-sem-cps-jump-001-type-partial-core-partial-cps-partial-"
                "admission-runtime-partial-verification-partial",
            )

        def change_layer_status(payload: dict[str, Any], root: Path) -> None:
            replace_heading(
                payload,
                root,
                "TASK-9001 SEM-CPS-CALL-001 type partial core partial cps implemented "
                "admission-runtime partial verification partial",
                "task-9001-sem-cps-call-001-type-partial-core-partial-cps-implemented-"
                "admission-runtime-partial-verification-partial",
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
            ("**Implementation:** partial", "**Implementation:** implemented", "coverage_target_spec_status_block_mismatch"),
            ("core partial", "core implemented", "coverage_layer_mismatch"),
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

    def test_task_2057_complete_status_requires_the_closed_handoff_allowlist(self) -> None:
        """TASK-2057 may close while active; an arbitrary active task may not."""

        def close_task_2057(payload: dict[str, Any], root: Path) -> None:
            record = payload["records"][0]
            assert isinstance(record, dict)
            old_relative_path = record["task_file"]
            assert isinstance(old_relative_path, str)
            old_task_file = root / old_relative_path
            new_relative_path = "docs/plan/tasks/TASK-2057-module-discovery.md"
            new_task_file = root / new_relative_path
            old_task_file.rename(new_task_file)

            record["task"] = "TASK-2057"
            record["task_file"] = new_relative_path
            coverage_map = record["coverage_map"]
            assert isinstance(coverage_map, str)
            record["coverage_map"] = coverage_map.replace("task-9001", "task-2057")
            record["verification"] = [
                "cargo test -p ash-parser --test task_2057_module_discovery"
            ]
            payload["active_scope"] = {"kind": "fixture", "tasks": ["TASK-2057"]}
            payload["active_tasks"] = ["TASK-2057"]

            new_task_file.write_text(
                new_task_file.read_text(encoding="utf-8")
                .replace("TASK-9001", "TASK-2057")
                .replace("task-9001-workflow-record", "task-2057-workflow-record")
                .replace("**Status:** In progress", "**Status:** Complete"),
                encoding="utf-8",
            )
            coverage_path = root / "docs/plan/SEMANTIC-RULE-COVERAGE.md"
            coverage_path.write_text(
                coverage_path.read_text(encoding="utf-8")
                .replace("TASK-9001-example.md", "TASK-2057-module-discovery.md")
                .replace("TASK-9001", "TASK-2057"),
                encoding="utf-8",
            )
            traceability_path = root / "docs/spec/SEMANTIC-TRACEABILITY.json"
            traceability_path.write_text(
                traceability_path.read_text(encoding="utf-8")
                .replace("TASK-9001-example.md", "TASK-2057-module-discovery.md"),
                encoding="utf-8",
            )

        def close_unlisted_task(payload: dict[str, Any], root: Path) -> None:
            task_file = root / payload["records"][0]["task_file"]
            task_file.write_text(
                task_file.read_text(encoding="utf-8").replace(
                    "**Status:** In progress", "**Status:** Complete"
                ),
                encoding="utf-8",
            )

        self.assertIn("TASK-2057", CLOSED_SEMANTIC_HANDOFF_TASKS)
        self.assert_mutation_accepted(close_task_2057)
        self.assert_mutation_rejected(
            close_unlisted_task, "active_task_status_mismatch"
        )

    def test_task_2058_complete_status_requires_the_closed_handoff_allowlist(self) -> None:
        """TASK-2058 may close while active; an arbitrary active task may not."""

        def close_task_2058(payload: dict[str, Any], root: Path) -> None:
            record = payload["records"][0]
            assert isinstance(record, dict)
            old_relative_path = record["task_file"]
            assert isinstance(old_relative_path, str)
            old_task_file = root / old_relative_path
            new_relative_path = (
                "docs/plan/tasks/TASK-2058-canonical-module-identity-and-artifacts.md"
            )
            new_task_file = root / new_relative_path
            old_task_file.rename(new_task_file)

            record["task"] = "TASK-2058"
            record["task_file"] = new_relative_path
            record["coverage_map"] = (
                "docs/plan/SEMANTIC-RULE-COVERAGE.md#"
                "task-2058-canonical-module-identity-and-artifacts"
            )
            record["verification"] = [
                "cargo test -p ash-core --test task_2058_canonical_module_identity"
            ]
            payload["active_scope"] = {"kind": "fixture", "tasks": ["TASK-2058"]}
            payload["active_tasks"] = ["TASK-2058"]

            new_task_file.write_text(
                new_task_file.read_text(encoding="utf-8")
                .replace("TASK-9001", "TASK-2058")
                .replace(
                    "task-9001-workflow-record",
                    "task-2058-canonical-module-identity-and-artifacts",
                )
                .replace("**Status:** In progress", "**Status:** Complete"),
                encoding="utf-8",
            )
            coverage_path = root / "docs/plan/SEMANTIC-RULE-COVERAGE.md"
            coverage_path.write_text(
                coverage_path.read_text(encoding="utf-8")
                .replace("TASK-9001-example.md", "TASK-2058-canonical-module-identity-and-artifacts.md")
                .replace(
                    "TASK-9001 workflow record",
                    "TASK-2058 canonical module identity and artifacts",
                )
                .replace("TASK-9001", "TASK-2058"),
                encoding="utf-8",
            )
            traceability_path = root / "docs/spec/SEMANTIC-TRACEABILITY.json"
            traceability_path.write_text(
                traceability_path.read_text(encoding="utf-8").replace(
                    "TASK-9001-example.md",
                    "TASK-2058-canonical-module-identity-and-artifacts.md",
                ),
                encoding="utf-8",
            )

        def close_unlisted_task(payload: dict[str, Any], root: Path) -> None:
            task_file = root / payload["records"][0]["task_file"]
            task_file.write_text(
                task_file.read_text(encoding="utf-8").replace(
                    "**Status:** In progress", "**Status:** Complete"
                ),
                encoding="utf-8",
            )

        self.assertIn("TASK-2058", CLOSED_SEMANTIC_HANDOFF_TASKS)
        self.assert_mutation_accepted(close_task_2058)
        self.assert_mutation_rejected(
            close_unlisted_task, "active_task_status_mismatch"
        )

    def test_task_2059_complete_status_requires_the_closed_handoff_allowlist(self) -> None:
        """TASK-2059 may close only when its module-unit handoff is allowlisted."""

        def configure_task_2059_scope(payload: dict[str, Any], root: Path) -> None:
            tasks = sorted(TASK_2059_FILE_INLINE_MODULE_UNIT_PARITY_SCOPE)
            records = payload["records"]
            assert isinstance(records, list) and len(records) == 1
            fixture_record = records[0]
            assert isinstance(fixture_record, dict)
            fixture_task_file = root / fixture_record["task_file"]
            fixture_task_text = fixture_task_file.read_text(encoding="utf-8")
            coverage_path = root / "docs/plan/SEMANTIC-RULE-COVERAGE.md"
            coverage_template = coverage_path.read_text(encoding="utf-8")
            traceability_path, traceability = self.traceability_graph(root)
            fixture_edges = traceability["edges"]
            assert isinstance(fixture_edges, list)

            scoped_records: list[dict[str, Any]] = []
            coverage_sections: list[str] = []
            scoped_edges: list[dict[str, Any]] = []
            for task in tasks:
                if task == "TASK-2059":
                    filename = "TASK-2059-file-inline-module-unit-parity.md"
                    title = "TASK-2059 file inline module unit parity"
                else:
                    filename = f"{task}-fixture.md"
                    title = f"{task} fixture"
                fragment = title.lower().replace(" ", "-")
                relative_task_file = f"docs/plan/tasks/{filename}"
                status = "Complete" if task in CLOSED_SEMANTIC_HANDOFF_TASKS else "In progress"
                if task == "TASK-2059":
                    status = "Complete"

                task_file = root / relative_task_file
                task_file.write_text(
                    fixture_task_text.replace("TASK-9001", task)
                    .replace("task-9001-workflow-record", fragment)
                    .replace("**Status:** In progress", f"**Status:** {status}"),
                    encoding="utf-8",
                )
                coverage_sections.append(
                    coverage_template.removeprefix("# Semantic Rule Coverage Map\n\n")
                    .replace("TASK-9001 workflow record", title)
                    .replace("TASK-9001-example.md", filename)
                    .replace("TASK-9001", task)
                )

                record = dict(fixture_record)
                record["task"] = task
                record["task_file"] = relative_task_file
                record["coverage_map"] = (
                    "docs/plan/SEMANTIC-RULE-COVERAGE.md#" f"{fragment}"
                )
                task_number = task.removeprefix("TASK-")
                record["verification"] = [
                    "cargo test -p ash-parser --test "
                    f"task_{task_number}_fixture"
                ]
                if task == "TASK-2059":
                    record["verification"] = [
                        "cargo test -p ash-parser --test "
                        "task_2059_file_inline_module_unit_parity"
                    ]
                scoped_records.append(record)

                for edge in fixture_edges:
                    assert isinstance(edge, dict)
                    scoped_edge = dict(edge)
                    scoped_edge["anchor"] = f"{relative_task_file}#evidence"
                    scoped_edges.append(scoped_edge)

            coverage_path.write_text(
                "# Semantic Rule Coverage Map\n\n" + "\n".join(coverage_sections),
                encoding="utf-8",
            )
            traceability["edges"] = scoped_edges
            self.write_traceability_graph(traceability_path, traceability)
            payload["records"] = scoped_records
            payload["active_scope"] = {
                "kind": "task-2059-file-inline-module-unit-parity",
                "tasks": tasks,
            }
            payload["active_tasks"] = tasks

        def close_nonallowlisted_task(payload: dict[str, Any], root: Path) -> None:
            configure_task_2059_scope(payload, root)
            task_file = root / "docs/plan/tasks/TASK-2001-fixture.md"
            task_file.write_text(
                task_file.read_text(encoding="utf-8").replace(
                    "**Status:** In progress", "**Status:** Complete"
                ),
                encoding="utf-8",
            )

        self.assertIn("TASK-2059", CLOSED_SEMANTIC_HANDOFF_TASKS)
        self.assert_mutation_accepted(configure_task_2059_scope)
        self.assert_mutation_rejected(
            close_nonallowlisted_task, "active_task_status_mismatch"
        )

    def test_task_2060_complete_status_requires_the_closed_handoff_allowlist(self) -> None:
        """TASK-2060 may close only when its checked-interface handoff is allowlisted."""

        def configure_task_2060_scope(payload: dict[str, Any], root: Path) -> None:
            tasks = sorted(TASK_2060_CHECKED_MODULE_INTERFACE_SCOPE)
            records = payload["records"]
            assert isinstance(records, list) and len(records) == 1
            fixture_record = records[0]
            assert isinstance(fixture_record, dict)
            fixture_task_file = root / fixture_record["task_file"]
            fixture_task_text = fixture_task_file.read_text(encoding="utf-8")
            coverage_path = root / "docs/plan/SEMANTIC-RULE-COVERAGE.md"
            coverage_template = coverage_path.read_text(encoding="utf-8")
            traceability_path, traceability = self.traceability_graph(root)
            fixture_edges = traceability["edges"]
            assert isinstance(fixture_edges, list)

            scoped_records: list[dict[str, Any]] = []
            coverage_sections: list[str] = []
            scoped_edges: list[dict[str, Any]] = []
            for task in tasks:
                if task == "TASK-2060":
                    filename = "TASK-2060-checked-module-interface-and-export-closure.md"
                    title = "TASK-2060 checked module interface and export closure"
                else:
                    filename = f"{task}-fixture.md"
                    title = f"{task} fixture"
                fragment = title.lower().replace(" ", "-")
                relative_task_file = f"docs/plan/tasks/{filename}"
                status = "Complete" if task in CLOSED_SEMANTIC_HANDOFF_TASKS else "In progress"
                if task == "TASK-2060":
                    status = "Complete"

                task_file = root / relative_task_file
                task_file.write_text(
                    fixture_task_text.replace("TASK-9001", task)
                    .replace("task-9001-workflow-record", fragment)
                    .replace("**Status:** In progress", f"**Status:** {status}"),
                    encoding="utf-8",
                )
                coverage_sections.append(
                    coverage_template.removeprefix("# Semantic Rule Coverage Map\n\n")
                    .replace("TASK-9001 workflow record", title)
                    .replace("TASK-9001-example.md", filename)
                    .replace("TASK-9001", task)
                )

                record = dict(fixture_record)
                record["task"] = task
                record["task_file"] = relative_task_file
                record["coverage_map"] = (
                    "docs/plan/SEMANTIC-RULE-COVERAGE.md#" f"{fragment}"
                )
                task_number = task.removeprefix("TASK-")
                record["verification"] = [
                    "cargo test -p ash-parser --test "
                    f"task_{task_number}_fixture"
                ]
                if task == "TASK-2060":
                    record["verification"] = [
                        "cargo test -p ash-core --test "
                        "task_2060_public_module_interface"
                    ]
                scoped_records.append(record)

                for edge in fixture_edges:
                    assert isinstance(edge, dict)
                    scoped_edge = dict(edge)
                    scoped_edge["anchor"] = f"{relative_task_file}#evidence"
                    scoped_edges.append(scoped_edge)

            coverage_path.write_text(
                "# Semantic Rule Coverage Map\n\n" + "\n".join(coverage_sections),
                encoding="utf-8",
            )
            traceability["edges"] = scoped_edges
            self.write_traceability_graph(traceability_path, traceability)
            payload["records"] = scoped_records
            payload["active_scope"] = {
                "kind": "task-2060-checked-module-interface",
                "tasks": tasks,
            }
            payload["active_tasks"] = tasks

        def close_nonallowlisted_task(payload: dict[str, Any], root: Path) -> None:
            configure_task_2060_scope(payload, root)
            task_file = root / "docs/plan/tasks/TASK-2001-fixture.md"
            task_file.write_text(
                task_file.read_text(encoding="utf-8").replace(
                    "**Status:** In progress", "**Status:** Complete"
                ),
                encoding="utf-8",
            )

        self.assertIn("TASK-2060", CLOSED_SEMANTIC_HANDOFF_TASKS)
        self.assert_mutation_accepted(configure_task_2060_scope)
        self.assert_mutation_rejected(
            close_nonallowlisted_task, "active_task_status_mismatch"
        )

    def test_retired_domain_field_is_rejected(self) -> None:
        """Records cannot reintroduce a second status vocabulary through domain."""
        for status in ("bounded", "general"):
            with self.subTest(status=status):
                def reintroduce_domain(
                    payload: dict[str, Any], _root: Path, status: str = status
                ) -> None:
                    payload["records"][0]["domain"] = {"status": status}

                self.assert_mutation_rejected(
                    reintroduce_domain, "unknown_record_field"
                )

    def test_v2_schema_rejects_unknown_fields_at_every_controlled_level(self) -> None:
        """Schema v2 must fail closed instead of silently ignoring governance data."""
        def add_root_field(payload: dict[str, Any], _root: Path) -> None:
            payload["unexpected"] = True

        def add_record_field(payload: dict[str, Any], _root: Path) -> None:
            payload["records"][0]["unexpected"] = True

        def add_layers_field(payload: dict[str, Any], _root: Path) -> None:
            payload["records"][0]["layers"]["unexpected"] = "partial"

        def add_evidence_field(payload: dict[str, Any], _root: Path) -> None:
            payload["records"][0]["evidence"]["unexpected"] = ["TEST-UNEXPECTED"]

        def add_parity_field(payload: dict[str, Any], _root: Path) -> None:
            payload["records"][0]["evidence"]["parity"]["unexpected"] = True

        for mutate, kind in (
            (add_root_field, "unknown_manifest_field"),
            (add_record_field, "unknown_record_field"),
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
        self.assertTrue(allowed_verification_command("cargo test -p ash-parser --lib"))
        self.assertFalse(
            command_matches_task_integration_test(
                "cargo test -p ash-parser --lib", "TASK-2074"
            )
        )
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

    def test_task_2075_red_collector_target_is_deferred_until_green(self) -> None:
        """The RED collector command is valid for future GREEN, but is not required to pass yet."""
        command = "python3 -m unittest tools.docs.tests.test_task_2071_module_namespace_contract"
        self.assertTrue(allowed_verification_command(command))
        self.assertTrue(command_matches_task_integration_test(command, "TASK-2071"))
        self.assertTrue(command_matches_task_integration_test(command, "TASK-2075"))
        self.assertFalse(command_matches_task_integration_test(command, "TASK-2072"))

        red_command = (
            "cargo test -p ash-typeck --test task_2075_two_tier_module_collection"
        )
        self.assertTrue(allowed_verification_command(red_command))
        self.assertTrue(command_matches_task_integration_test(red_command, "TASK-2075"))
        self.assertFalse(command_matches_task_integration_test(red_command, "TASK-2074"))

        manifest = json.loads(
            (REPOSITORY_ROOT / "docs/plan/semantic-task-records.json").read_text()
        )
        collection_record = next(
            record for record in manifest["records"] if record["task"] == "TASK-2075"
        )
        self.assertNotIn(red_command, collection_record["verification"])
        self.assertIn(
            "cargo test -p ash-parser --test task_2075_collection_visibility_carriers",
            collection_record["verification"],
        )
        task = (
            REPOSITORY_ROOT
            / "docs/plan/tasks/TASK-2075-two-tier-complete-module-collection.md"
        ).read_text()
        self.assertIn("intentionally excluded from the manifest's required-success", task)
        self.assertIn("verification:", task)
        self.assertIn("only after the production collector module exists", task)

    def test_task_2031_scope_owns_the_exact_task_set_without_a_domain_status_policy(self) -> None:
        """Ownership scopes do not reintroduce a second feature-status vocabulary."""
        tasks = sorted(TASK_2031_PREREQUISITE_SCOPE)
        records = [
            {"task": task, "implementation": "partial"}
            for task in tasks
        ]
        payload = {"active_scope": {"kind": "task-2031-prerequisite", "tasks": tasks}}
        errors: list[dict[str, object]] = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertEqual(errors, [])

        payload["active_scope"]["tasks"] = tasks[1:]
        errors = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertTrue(
            any(error.get("kind") == "active_scope_task_set_mismatch" for error in errors),
            errors,
        )

    def test_task_2032_integration_scope_owns_the_exact_task_set_without_a_domain_status_policy(self) -> None:
        """Integration ownership is independent from a record's implementation axis."""
        tasks = sorted(TASK_2032_INTEGRATION_SCOPE)
        records = [
            {
                "task": task,
                "implementation": "partial",
            }
            for task in tasks
        ]
        payload = {"active_scope": {"kind": "task-2032-integration", "tasks": tasks}}
        errors: list[dict[str, object]] = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertEqual(errors, [])

        payload["active_scope"]["tasks"] = tasks[:-1]
        errors = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertTrue(
            any(error.get("kind") == "active_scope_task_set_mismatch" for error in errors),
            errors,
        )

    def test_task_2037_engine_cps_scope_owns_the_exact_task_set(self) -> None:
        """The Engine executor handoff extends the controlled active scope by one task."""
        tasks = sorted(TASK_2037_ENGINE_CPS_SCOPE)
        records = [
            {
                "task": task,
                "implementation": "partial",
            }
            for task in tasks
        ]
        payload = {"active_scope": {"kind": "task-2037-engine-cps", "tasks": tasks}}
        errors: list[dict[str, object]] = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertEqual(errors, [])

        payload["active_scope"]["tasks"] = tasks[:-1]
        errors = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertTrue(
            any(error.get("kind") == "active_scope_task_set_mismatch" for error in errors),
            errors,
        )

    def test_task_2039_repl_scope_owns_the_exact_task_set(self) -> None:
        """The REPL client route adds its active task without discarding prior handoffs."""
        tasks = sorted(TASK_2039_REPL_SCOPE)
        records = [
            {
                "task": task,
                "implementation": "partial",
            }
            for task in tasks
        ]
        payload = {"active_scope": {"kind": "task-2039-repl", "tasks": tasks}}
        errors: list[dict[str, object]] = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertEqual(errors, [])

        payload["active_scope"]["tasks"] = tasks[:-1]
        errors = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertTrue(
            any(error.get("kind") == "active_scope_task_set_mismatch" for error in errors),
            errors,
        )

    def test_task_2040_engine_only_removal_scope_owns_the_exact_task_set(self) -> None:
        """The removal task extends the controlled scope without dropping client handoffs."""
        tasks = sorted(TASK_2040_ENGINE_ONLY_REMOVAL_SCOPE)
        records = [
            {
                "task": task,
                "implementation": "partial",
            }
            for task in tasks
        ]
        payload = {
            "active_scope": {
                "kind": "task-2040-engine-only-removal",
                "tasks": tasks,
            }
        }
        errors: list[dict[str, object]] = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertEqual(errors, [])

        payload["active_scope"]["tasks"] = tasks[:-1]
        errors = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertTrue(
            any(error.get("kind") == "active_scope_task_set_mismatch" for error in errors),
            errors,
        )

    def test_task_2057_module_discovery_scope_requires_the_full_closed_handoff_chain(self) -> None:
        """Module discovery extends the closed TASK-2041 handoff with TASK-2057 only."""
        tasks = sorted(TASK_2057_MODULE_DISCOVERY_SCOPE)
        self.assertEqual(
            set(tasks), TASK_2041_ENGINE_ONLY_CLOSEOUT_SCOPE | {"TASK-2057"}
        )
        records = [
            {
                "task": task,
                "implementation": "partial",
            }
            for task in tasks
        ]
        payload = {
            "active_scope": {
                "kind": "task-2057-module-discovery",
                "tasks": tasks,
            }
        }
        errors: list[dict[str, object]] = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertEqual(errors, [])

        payload["active_scope"]["tasks"] = [
            task for task in tasks if task != "TASK-2057"
        ]
        errors = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertTrue(
            any(error.get("kind") == "active_scope_task_set_mismatch" for error in errors),
            errors,
        )

    def test_task_2058_canonical_module_identity_scope_requires_the_full_discovery_chain(self) -> None:
        """Canonical identity extends the declared TASK-2057 scope with TASK-2058 only."""
        tasks = sorted(TASK_2058_CANONICAL_MODULE_IDENTITY_SCOPE)
        self.assertEqual(
            set(tasks), TASK_2057_MODULE_DISCOVERY_SCOPE | {"TASK-2058"}
        )
        records = [
            {
                "task": task,
                "implementation": "not_implemented" if task == "TASK-2058" else "partial",
            }
            for task in tasks
        ]
        payload = {
            "active_scope": {
                "kind": "task-2058-canonical-module-identity",
                "tasks": tasks,
            }
        }
        errors: list[dict[str, object]] = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertEqual(errors, [])

        payload["active_scope"]["tasks"] = [
            task for task in tasks if task != "TASK-2058"
        ]
        errors = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertTrue(
            any(error.get("kind") == "active_scope_task_set_mismatch" for error in errors),
            errors,
        )

    def test_task_2059_file_inline_module_unit_parity_scope_requires_the_full_identity_chain(self) -> None:
        """Module-unit parity extends the TASK-2058 scope with TASK-2059 only."""
        tasks = sorted(TASK_2059_FILE_INLINE_MODULE_UNIT_PARITY_SCOPE)
        self.assertEqual(
            set(tasks), TASK_2058_CANONICAL_MODULE_IDENTITY_SCOPE | {"TASK-2059"}
        )
        records = [
            {
                "task": task,
                "implementation": "not_implemented" if task == "TASK-2059" else "partial",
            }
            for task in tasks
        ]
        payload = {
            "active_scope": {
                "kind": "task-2059-file-inline-module-unit-parity",
                "tasks": tasks,
            }
        }
        errors: list[dict[str, object]] = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertEqual(errors, [])

        payload["active_scope"]["tasks"] = [
            task for task in tasks if task != "TASK-2059"
        ]
        errors = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertTrue(
            any(error.get("kind") == "active_scope_task_set_mismatch" for error in errors),
            errors,
        )

    def test_task_2060_checked_module_interface_scope_requires_the_full_module_unit_chain(self) -> None:
        """Checked interfaces extend module-unit parity with TASK-2060 only."""
        tasks = sorted(TASK_2060_CHECKED_MODULE_INTERFACE_SCOPE)
        self.assertEqual(
            set(tasks), TASK_2059_FILE_INLINE_MODULE_UNIT_PARITY_SCOPE | {"TASK-2060"}
        )
        records = [
            {
                "task": task,
                "implementation": "not_implemented" if task == "TASK-2060" else "partial",
            }
            for task in tasks
        ]
        payload = {
            "active_scope": {
                "kind": "task-2060-checked-module-interface",
                "tasks": tasks,
            }
        }
        errors: list[dict[str, object]] = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertEqual(errors, [])

        payload["active_scope"]["tasks"] = [
            task for task in tasks if task != "TASK-2060"
        ]
        errors = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertTrue(
            any(error.get("kind") == "active_scope_task_set_mismatch" for error in errors),
            errors,
        )

    def test_task_2061_interface_import_resolution_scope_requires_the_full_checked_interface_chain(self) -> None:
        """Interface binding extends checked interfaces with TASK-2061 only."""
        tasks = sorted(TASK_2061_INTERFACE_IMPORT_RESOLUTION_SCOPE)
        self.assertEqual(
            set(tasks), TASK_2060_CHECKED_MODULE_INTERFACE_SCOPE | {"TASK-2061"}
        )
        self.assertIn("TASK-2061", CLOSED_SEMANTIC_HANDOFF_TASKS)
        records = [
            {
                "task": task,
                "implementation": "not_implemented" if task == "TASK-2061" else "partial",
            }
            for task in tasks
        ]
        payload = {
            "active_scope": {
                "kind": "task-2061-interface-import-resolution",
                "tasks": tasks,
            }
        }
        errors: list[dict[str, object]] = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertEqual(errors, [])

        payload["active_scope"]["tasks"] = [
            task for task in tasks if task != "TASK-2061"
        ]
        errors = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertTrue(
            any(error.get("kind") == "active_scope_task_set_mismatch" for error in errors),
            errors,
        )

    def test_task_2066_typeenv_finalization_scope_requires_the_full_import_prerequisite_chain(self) -> None:
        """TypeEnv finalization extends the import prerequisite chain with TASK-2066 only."""
        tasks = sorted(TASK_2066_TYPEENV_MODULE_UNIT_INTERFACE_FINALIZATION_SCOPE)
        self.assertEqual(
            set(tasks), TASK_2061_INTERFACE_IMPORT_RESOLUTION_SCOPE | {"TASK-2066"}
        )
        self.assertIn("TASK-2066", CLOSED_SEMANTIC_HANDOFF_TASKS)
        records = [
            {
                "task": task,
                "implementation": "not_implemented" if task == "TASK-2066" else "partial",
            }
            for task in tasks
        ]
        payload = {
            "active_scope": {
                "kind": "task-2066-typeenv-module-unit-interface-finalization",
                "tasks": tasks,
            }
        }
        errors: list[dict[str, object]] = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertEqual(errors, [])

        payload["active_scope"]["tasks"] = [
            task for task in tasks if task != "TASK-2066"
        ]
        errors = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertTrue(
            any(error.get("kind") == "active_scope_task_set_mismatch" for error in errors),
            errors,
        )

    def test_task_2062_module_aware_core_cps_lowering_scope_requires_the_full_checked_chain(self) -> None:
        """Module lowering extends the checked-interface chain with TASK-2062 only."""
        tasks = sorted(TASK_2062_MODULE_AWARE_CORE_CPS_LOWERING_SCOPE)
        self.assertEqual(
            set(tasks), TASK_2066_TYPEENV_MODULE_UNIT_INTERFACE_FINALIZATION_SCOPE | {"TASK-2062"}
        )
        self.assertIn("TASK-2062", CLOSED_SEMANTIC_HANDOFF_TASKS)
        records = [
            {
                "task": task,
                "implementation": "partial",
            }
            for task in tasks
        ]
        payload = {
            "active_scope": {
                "kind": "task-2062-module-aware-core-cps-lowering",
                "tasks": tasks,
            }
        }
        errors: list[dict[str, object]] = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertEqual(errors, [])

        payload["active_scope"]["tasks"] = [
            task for task in tasks if task != "TASK-2062"
        ]
        errors = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertTrue(
            any(error.get("kind") == "active_scope_task_set_mismatch" for error in errors),
            errors,
        )

    def test_task_2063_engine_linked_module_admission_scope_requires_the_full_checked_chain(self) -> None:
        """Linked admission extends the checked Core/CPS chain with TASK-2063 only."""
        tasks = sorted(TASK_2063_ENGINE_LINKED_MODULE_ADMISSION_SCOPE)
        self.assertEqual(
            set(tasks), TASK_2062_MODULE_AWARE_CORE_CPS_LOWERING_SCOPE | {"TASK-2063"}
        )
        self.assertNotIn("TASK-2063", CLOSED_SEMANTIC_HANDOFF_TASKS)
        records = [
            {
                "task": task,
                "implementation": "not_implemented" if task == "TASK-2063" else "partial",
            }
            for task in tasks
        ]
        payload = {
            "active_scope": {
                "kind": "task-2063-engine-linked-module-admission",
                "tasks": tasks,
            }
        }
        errors: list[dict[str, object]] = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertEqual(errors, [])

        payload["active_scope"]["tasks"] = [
            task for task in tasks if task != "TASK-2063"
        ]
        errors = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertTrue(
            any(error.get("kind") == "active_scope_task_set_mismatch" for error in errors),
            errors,
        )

    def test_task_2067_canonical_module_graph_scope_requires_its_owned_rules_and_full_chain(self) -> None:
        """TASK-2067 extends linked admission and owns both structural module rules."""
        tasks = sorted(TASK_2063_ENGINE_LINKED_MODULE_ADMISSION_SCOPE | {"TASK-2067"})
        task_2067_record = {
            "task": "TASK-2067",
            "canonical_rule_ids": ["MOD-REAL-001", "MOD-REAL-002"],
            "implementation": "not_implemented",
            "layers": {
                "type": "not_implemented",
                "core": "not_implemented",
                "cps": "not_applicable",
                "admission_runtime": "not_applicable",
                "verification": "not_implemented",
            },
            "evidence": {"status": "none"},
            "parity": "below_spec",
            "missing_spec_clauses": ["Canonical structural graph remains unimplemented."],
            "non_goals": ["Import binding and runtime admission."],
            "next_obligation": "Build the canonical structural graph.",
        }
        self.assertEqual(
            set(task_2067_record["canonical_rule_ids"]),
            {"MOD-REAL-001", "MOD-REAL-002"},
        )
        records = [
            {
                "task": task,
                "implementation": "not_implemented" if task in {"TASK-2063", "TASK-2067"} else "partial",
            }
            for task in tasks
        ]
        payload = {
            "active_scope": {
                "kind": "task-2067-canonical-module-graph-and-structural-diagnostics",
                "tasks": tasks,
            }
        }
        errors: list[dict[str, object]] = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertEqual(errors, [])

        payload["active_scope"]["tasks"] = [
            task for task in tasks if task != "TASK-2067"
        ]
        errors = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertTrue(
            any(error.get("kind") == "active_scope_task_set_mismatch" for error in errors),
            errors,
        )

    def test_task_2068_final_interfaces_scope_requires_its_owned_rules_and_full_chain(self) -> None:
        """TASK-2068 extends canonical module graphs with final interface binding."""
        tasks = sorted(TASK_2068_FINAL_INTERFACES_PARSED_IMPORTS_BINDER_SCOPE)
        self.assertEqual(
            set(tasks), TASK_2067_CANONICAL_MODULE_GRAPH_SCOPE | {"TASK-2068"}
        )
        task_2068_record = {
            "task": "TASK-2068",
            "canonical_rule_ids": [
                "SEM-MODULE-REALIZATION-003",
                "SEM-MODULE-REALIZATION-004",
            ],
            "implementation": "not_implemented",
            "layers": {
                "type": "not_implemented",
                "core": "not_implemented",
                "cps": "not_applicable",
                "admission_runtime": "not_applicable",
                "verification": "not_implemented",
            },
            "evidence": {"status": "none"},
            "parity": "below_spec",
            "missing_spec_clauses": [
                "Final public interfaces and parsed import binding remain unimplemented."
            ],
            "non_goals": ["Definition lowering and runtime admission."],
            "next_obligation": "Build final interfaces and parsed-import binder facts.",
        }
        self.assertEqual(
            set(task_2068_record["canonical_rule_ids"]),
            {"SEM-MODULE-REALIZATION-003", "SEM-MODULE-REALIZATION-004"},
        )
        self.assertEqual(task_2068_record["implementation"], "not_implemented")
        self.assertEqual(task_2068_record["evidence"]["status"], "none")
        self.assertEqual(task_2068_record["parity"], "below_spec")
        records = [
            {
                "task": task,
                "implementation": "not_implemented"
                if task in {"TASK-2063", "TASK-2067", "TASK-2068"}
                else "partial",
            }
            for task in tasks
        ]
        payload = {
            "active_scope": {
                "kind": "task-2068-final-interfaces-parsed-imports-and-binder-integration",
                "tasks": tasks,
            }
        }
        errors: list[dict[str, object]] = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertEqual(errors, [])

        payload["active_scope"]["tasks"] = [
            task for task in tasks if task != "TASK-2068"
        ]
        errors = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertTrue(
            any(error.get("kind") == "active_scope_task_set_mismatch" for error in errors),
            errors,
        )

    def test_task_2070_closed_handoff_scope_is_exact_and_excludes_planned_successors(self) -> None:
        """TASK-2070 closes its bounded handoff without activating TASK-2071."""
        tasks = sorted(TASK_2070_SCOPED_SELF_SIMPLE_FUNCTION_ALIASES_SCOPE)
        self.assertEqual(
            set(tasks),
            TASK_2068_FINAL_INTERFACES_PARSED_IMPORTS_BINDER_SCOPE | {"TASK-2070"},
        )
        self.assertIn("TASK-2070", CLOSED_SEMANTIC_HANDOFF_TASKS)
        self.assertNotIn("TASK-2071", tasks)

        records = [{"task": task, "implementation": "partial"} for task in tasks]
        payload = {
            "active_scope": {
                "kind": "task-2070-scoped-self-simple-function-aliases",
                "tasks": tasks,
            }
        }
        errors: list[dict[str, object]] = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertEqual(errors, [])

        payload["active_scope"]["tasks"] = [
            task for task in tasks if task != "TASK-2070"
        ]
        errors = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertTrue(
            any(error.get("kind") == "active_scope_task_set_mismatch" for error in errors),
            errors,
        )

    def test_task_2071_contract_scope_is_exact_and_excludes_planned_implementation(self) -> None:
        """The closed TASK-2071 scope snapshot excludes its then-unactivated successors."""
        tasks = sorted(TASK_2071_MODULE_NAMESPACE_CONTRACT_SCOPE)
        self.assertEqual(
            set(tasks),
            TASK_2070_SCOPED_SELF_SIMPLE_FUNCTION_ALIASES_SCOPE | {"TASK-2071"},
        )
        self.assertIn("TASK-2071", CLOSED_SEMANTIC_HANDOFF_TASKS)
        self.assertNotIn("TASK-2074", tasks)
        self.assertNotIn("TASK-2075", tasks)

        records = [
            {
                "task": task,
                "implementation": "not_implemented" if task == "TASK-2071" else "partial",
            }
            for task in tasks
        ]
        payload = {
            "active_scope": {
                "kind": "task-2071-module-namespace-contract",
                "tasks": tasks,
            }
        }
        errors: list[dict[str, object]] = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertEqual(errors, [])

        payload["active_scope"]["tasks"] = [
            task for task in tasks if task != "TASK-2071"
        ]
        errors = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertTrue(
            any(error.get("kind") == "active_scope_task_set_mismatch" for error in errors),
            errors,
        )

    def test_task_2074_closed_scope_snapshot_excludes_its_then_unactivated_successor(self) -> None:
        """The closed TASK-2074 scope remains the snapshot from before TASK-2075 activation."""
        tasks = sorted(TASK_2074_CANONICAL_EXPANDED_MODULE_GRAPH_SCOPE)
        self.assertEqual(
            set(tasks),
            TASK_2071_MODULE_NAMESPACE_CONTRACT_SCOPE | {"TASK-2074"},
        )
        self.assertIn("TASK-2074", CLOSED_SEMANTIC_HANDOFF_TASKS)
        self.assertNotIn("TASK-2075", tasks)

        records = [
            {
                "task": task,
                "implementation": "not_implemented"
                if task in {"TASK-2063", "TASK-2071"}
                else "partial",
            }
            for task in tasks
        ]
        payload = {
            "active_scope": {
                "kind": "task-2074-canonical-expanded-module-graph",
                "tasks": tasks,
            }
        }
        errors: list[dict[str, object]] = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertEqual(errors, [])

        payload["active_scope"]["tasks"] = [
            task for task in tasks if task != "TASK-2074"
        ]
        errors = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertTrue(
            any(error.get("kind") == "active_scope_task_set_mismatch" for error in errors),
            errors,
        )

    def test_task_2075_active_scope_is_exact_after_visibility_carrier_evidence(self) -> None:
        """TASK-2075 stays the sole active collection owner after its first tested slice."""
        tasks = sorted(TASK_2075_TWO_TIER_MODULE_COLLECTION_SCOPE)
        self.assertEqual(
            set(tasks),
            TASK_2074_CANONICAL_EXPANDED_MODULE_GRAPH_SCOPE | {"TASK-2075"},
        )
        self.assertNotIn("TASK-2075", CLOSED_SEMANTIC_HANDOFF_TASKS)
        self.assertNotIn("TASK-2072", tasks)
        self.assertNotIn("TASK-2073", tasks)

        records = [
            {
                "task": task,
                "implementation": "not_implemented"
                if task in {"TASK-2063", "TASK-2071"}
                else "partial",
            }
            for task in tasks
        ]
        payload = {
            "active_scope": {
                "kind": "task-2075-two-tier-complete-module-collection",
                "tasks": tasks,
            }
        }
        errors: list[dict[str, object]] = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertEqual(errors, [])

        payload["active_scope"]["tasks"] = [
            task for task in tasks if task != "TASK-2075"
        ]
        errors = []
        validate_active_scope(payload, records, tasks, errors)
        self.assertTrue(
            any(error.get("kind") == "active_scope_task_set_mismatch" for error in errors),
            errors,
        )

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
