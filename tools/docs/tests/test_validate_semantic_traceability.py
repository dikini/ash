#!/usr/bin/env python3
"""Contract tests for TASK-1990's semantic traceability graph gate.

The graph is deliberately evidence-oriented rather than a second semantic
authority: canonical rules own the specification side, while implementation,
test, and proof nodes provide independently addressable evidence.  The public
tool must emit a validation report to stdout and two deterministic coverage
reports when requested.
"""
from __future__ import annotations

import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
TOOL = REPOSITORY_ROOT / "tools/docs/validate_semantic_traceability.py"
FIXTURES = Path(__file__).with_name("fixtures") / "semantic_traceability_graph"


class SemanticTraceabilityContractTests(unittest.TestCase):
    """Exercise the fail-closed graph validator and reproducible report CLI."""

    def run_graph(
        self, graph: Path, root: Path, reports_dir: Path | None = None
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        self.assertTrue(TOOL.exists(), f"missing TASK-1990 traceability validator: {TOOL}")
        command = [
            "python3", str(TOOL), "--root", str(root), "--graph", str(graph), "--format", "json",
        ]
        if reports_dir is not None:
            command.extend(["--reports-dir", str(reports_dir)])
        result = subprocess.run(command, check=False, capture_output=True, text=True)
        try:
            report = json.loads(result.stdout)
        except json.JSONDecodeError as error:
            self.fail(f"traceability validator must write JSON to stdout: {error}; stderr: {result.stderr}")
        return result, report

    def run_fixture(self, name: str, reports_dir: Path | None = None) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        root = FIXTURES / name
        return self.run_graph(root / "SEMANTIC-TRACEABILITY.json", root, reports_dir)

    def run_mutation(self, mutate: object) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory) / "fixture"
            shutil.copytree(FIXTURES / "valid", root)
            graph = root / "SEMANTIC-TRACEABILITY.json"
            payload = json.loads(graph.read_text(encoding="utf-8"))
            assert callable(mutate)
            mutate(payload)
            graph.write_text(json.dumps(payload), encoding="utf-8")
            return self.run_graph(graph, root)

    def assert_mutation_rejected(self, mutate: object, kind: str) -> None:
        result, report = self.run_mutation(mutate)
        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report.get("schema"), "semantic-traceability-validation-report/v1")
        errors = report.get("errors")
        self.assertIsInstance(errors, list)
        self.assertTrue(any(error.get("kind") == kind for error in errors if isinstance(error, dict)), errors)

    def test_complete_pilot_graph_is_accepted_and_generates_bidirectional_reports(self) -> None:
        """A canonical calculus rule and two pilot targets retain all coverage directions."""
        with tempfile.TemporaryDirectory() as temporary_directory:
            reports_dir = Path(temporary_directory) / "reports"
            result, report = self.run_fixture("valid", reports_dir)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(report, {"schema": "semantic-traceability-validation-report/v1", "errors": []})

            specification = json.loads((reports_dir / "specification-coverage.json").read_text(encoding="utf-8"))
            implementation = json.loads((reports_dir / "implementation-coverage.json").read_text(encoding="utf-8"))

        self.assertEqual(specification["schema"], "semantic-traceability-specification-coverage/v1")
        self.assertEqual(implementation["schema"], "semantic-traceability-implementation-coverage/v1")
        self.assertEqual(
            {entry["rule"] for entry in specification["rules"]},
            {"SEM-CPS-LETVAL-001", "TYPE-ROW-NORMALIZE-001", "SEM-EFFECT-LOOKUP-001"},
        )
        by_implementation = {entry["implementation"]: entry for entry in implementation["implementations"]}
        self.assertEqual(by_implementation["IMPL-CORE-NORMALIZE-ROW"]["owners"], ["TYPE-ROW-NORMALIZE-001"])
        self.assertEqual(by_implementation["IMPL-CPS-FRAME-LOOKUP"]["owners"], ["SEM-EFFECT-LOOKUP-001"])

    def test_reports_are_byte_reproducible_for_the_same_graph(self) -> None:
        """A stable graph yields stable coverage artifacts, not traversal-order output."""
        with tempfile.TemporaryDirectory() as temporary_directory:
            first, second = Path(temporary_directory) / "first", Path(temporary_directory) / "second"
            first_result, _ = self.run_fixture("valid", first)
            second_result, _ = self.run_fixture("valid", second)
            self.assertEqual(first_result.returncode, 0, first_result.stderr)
            self.assertEqual(second_result.returncode, 0, second_result.stderr)
            for name in ("specification-coverage.json", "implementation-coverage.json"):
                self.assertEqual((first / name).read_bytes(), (second / name).read_bytes(), name)

    def test_node_namespaces_and_distinct_status_facts_are_validated(self) -> None:
        """Stable REQ..PROOF IDs and coverage facts are not free-form labels."""
        def mutate(payload: dict[str, object]) -> None:
            nodes = payload["nodes"]
            assert isinstance(nodes, list)
            nodes[0]["id"] = "RULE-LETVAL-001"
        self.assert_mutation_rejected(mutate, "invalid_node_id")

        def invalid_status(payload: dict[str, object]) -> None:
            nodes = payload["nodes"]
            assert isinstance(nodes, list)
            nodes[0]["status"] = ["covered"]
        self.assert_mutation_rejected(invalid_status, "invalid_status_fact")

    def test_controlled_edges_must_have_existing_nodes_and_stable_anchors(self) -> None:
        """Links cannot silently point at nonexistent graph records or line-number anchors."""
        def dangling(payload: dict[str, object]) -> None:
            edges = payload["edges"]
            assert isinstance(edges, list)
            edges[0]["to"] = "IMPL-NOT-DECLARED"
        self.assert_mutation_rejected(dangling, "dangling_edge")

        def invalid_kind(payload: dict[str, object]) -> None:
            edges = payload["edges"]
            assert isinstance(edges, list)
            edges[0]["kind"] = "covers"
        self.assert_mutation_rejected(invalid_kind, "invalid_edge_kind")

        def unstable_anchor(payload: dict[str, object]) -> None:
            edges = payload["edges"]
            assert isinstance(edges, list)
            edges[0]["anchor"] = "docs/spec/ASH-CPS-CALCULUS.md:42"
        self.assert_mutation_rejected(unstable_anchor, "invalid_edge_anchor")

    def test_proved_status_requires_complete_successful_proof_metadata(self) -> None:
        """A failed tool run cannot be represented as proof merely by setting `proved`."""
        def false_proof(payload: dict[str, object]) -> None:
            nodes = payload["nodes"]
            assert isinstance(nodes, list)
            proof = next(node for node in nodes if node["id"] == "PROOF-ROW-NORMALIZE-001")
            assert isinstance(proof, dict)
            metadata = proof["proof"]
            assert isinstance(metadata, dict)
            metadata["outcome"] = "failed"
        self.assert_mutation_rejected(false_proof, "false_proof_status")

        def missing_fingerprint(payload: dict[str, object]) -> None:
            nodes = payload["nodes"]
            assert isinstance(nodes, list)
            proof = next(node for node in nodes if node["id"] == "PROOF-ROW-NORMALIZE-001")
            assert isinstance(proof, dict)
            metadata = proof["proof"]
            assert isinstance(metadata, dict)
            metadata.pop("implementation_fingerprint")
        self.assert_mutation_rejected(missing_fingerprint, "invalid_proof_metadata")

    def test_effect_rule_cannot_be_proved_by_a_proof_outside_its_declared_scope(self) -> None:
        """A λAsh-CPS0 frame-lookup model does not prove λAsh-Effect lookup correspondence."""
        def misapply_limited_cps_proof(payload: dict[str, object]) -> None:
            nodes = payload["nodes"]
            assert isinstance(nodes, list)
            proof = next(node for node in nodes if node["id"] == "PROOF-CPS-FRAME-LOOKUP-001")
            assert isinstance(proof, dict)
            proof["status"] = ["proved"]
            metadata = proof["proof"]
            assert isinstance(metadata, dict)
            metadata["outcome"] = "verified"
            metadata["scope"] = {
                "calculus": "lambda-Ash-CPS0",
                "proven_rule_ids": ["SEM-CPS-FRAME-LOOKUP-001"],
                "excluded_rule_ids": ["SEM-EFFECT-LOOKUP-001"],
            }

        self.assert_mutation_rejected(misapply_limited_cps_proof, "proof_scope_mismatch")

    def test_proof_scope_model_must_name_a_declared_model_even_without_a_proof_edge(self) -> None:
        """Scope metadata is evidence itself; it cannot cite a made-up rule or non-model node."""
        def remove_proof_edge(payload: dict[str, object]) -> None:
            edges = payload["edges"]
            assert isinstance(edges, list)
            payload["edges"] = [edge for edge in edges if edge.get("kind") != "proved_by"]

        def cite_unknown_scope_model(payload: dict[str, object]) -> None:
            nodes = payload["nodes"]
            assert isinstance(nodes, list)
            proof = next(node for node in nodes if node["id"] == "PROOF-CPS-FRAME-LOOKUP-001")
            assert isinstance(proof, dict)
            metadata = proof["proof"]
            assert isinstance(metadata, dict)
            metadata["scope"] = {"model": "SEM-FICTITIOUS-FRAME-LOOKUP-MODEL-001"}

        def cite_canonical_rule_as_scope_model(payload: dict[str, object]) -> None:
            nodes = payload["nodes"]
            assert isinstance(nodes, list)
            proof = next(node for node in nodes if node["id"] == "PROOF-CPS-FRAME-LOOKUP-001")
            assert isinstance(proof, dict)
            metadata = proof["proof"]
            assert isinstance(metadata, dict)
            metadata["scope"] = {"model": "SEM-EFFECT-LOOKUP-001"}

        for mutate, expected_kind in (
            (lambda payload: (remove_proof_edge(payload), cite_unknown_scope_model(payload)), "unknown_proof_scope_model"),
            (lambda payload: (remove_proof_edge(payload), cite_canonical_rule_as_scope_model(payload)), "invalid_proof_scope_model"),
        ):
            with self.subTest(expected_kind=expected_kind):
                self.assert_mutation_rejected(mutate, expected_kind)

    def test_proof_scope_model_must_match_the_proof_model(self) -> None:
        """A scope cannot silently substitute a different valid model for the one the proof executed."""
        def substitute_different_existing_model(payload: dict[str, object]) -> None:
            nodes = payload["nodes"]
            assert isinstance(nodes, list)
            nodes.extend((
                {
                    "id": "SEM-CPS-FRAME-LOOKUP-MODEL-001",
                    "kind": "model",
                    "status": ["modelled"],
                    "anchor": "proofs/cps-frame-lookup.rs#declared-model",
                },
                {
                    "id": "SEM-CPS-OTHER-FRAME-LOOKUP-MODEL-001",
                    "kind": "model",
                    "status": ["modelled"],
                    "anchor": "proofs/cps-frame-lookup.rs#different-model",
                },
            ))
            proof = next(node for node in nodes if node["id"] == "PROOF-CPS-FRAME-LOOKUP-001")
            assert isinstance(proof, dict)
            metadata = proof["proof"]
            assert isinstance(metadata, dict)
            metadata["model"] = "SEM-CPS-FRAME-LOOKUP-MODEL-001"
            metadata["scope"] = {"model": "SEM-CPS-OTHER-FRAME-LOOKUP-MODEL-001"}

        self.assert_mutation_rejected(substitute_different_existing_model, "proof_scope_model_mismatch")

    def test_proof_scope_rule_ids_resolve_and_model_level_scope_is_valid(self) -> None:
        """Rule lists cannot contain phantom IDs, while a declared model may scope a proof without rule claims."""
        graph = REPOSITORY_ROOT / "docs/spec/SEMANTIC-TRACEABILITY.json"
        result, report = self.run_graph(graph, REPOSITORY_ROOT)
        self.assertEqual(result.returncode, 0, report)
        self.assertEqual(report, {"schema": "semantic-traceability-validation-report/v1", "errors": []})

        def use_model_level_scope(payload: dict[str, object]) -> None:
            nodes = payload["nodes"]
            assert isinstance(nodes, list)
            nodes.append({
                "id": "SEM-CPS-FRAME-LOOKUP-MODEL-001",
                "kind": "model",
                "status": ["modelled"],
                "anchor": "proofs/cps-frame-lookup.rs#model",
            })
            proof = next(node for node in nodes if node["id"] == "PROOF-CPS-FRAME-LOOKUP-001")
            assert isinstance(proof, dict)
            metadata = proof["proof"]
            assert isinstance(metadata, dict)
            metadata["model"] = "SEM-CPS-FRAME-LOOKUP-MODEL-001"
            metadata["scope"] = {"model": "SEM-CPS-FRAME-LOOKUP-MODEL-001"}

        result, report = self.run_mutation(use_model_level_scope)
        self.assertEqual(result.returncode, 0, report)

    def test_canonical_rule_without_evidence_or_owned_gap_is_rejected(self) -> None:
        """A specified SEM/TYPE rule must expose evidence or a visible owned disposition."""
        def mutate(payload: dict[str, object]) -> None:
            edges = payload["edges"]
            assert isinstance(edges, list)
            payload["edges"] = [edge for edge in edges if edge["from"] != "SEM-CPS-LETVAL-001"]
        self.assert_mutation_rejected(mutate, "unowned_canonical_rule")

    def test_public_semantic_implementation_without_canonical_owner_is_rejected(self) -> None:
        """Public semantic Rust APIs cannot disappear from the reverse coverage matrix."""
        def mutate(payload: dict[str, object]) -> None:
            edges = payload["edges"]
            assert isinstance(edges, list)
            payload["edges"] = [edge for edge in edges if edge["to"] != "IMPL-CPS-FRAME-LOOKUP"]
        self.assert_mutation_rejected(mutate, "orphan_public_semantic_implementation")


if __name__ == "__main__":
    unittest.main()
