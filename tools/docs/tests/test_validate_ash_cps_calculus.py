#!/usr/bin/env python3
"""Contract tests for TASK-1989's machine-readable λAsh-CPS calculus freeze.

The artifact deliberately describes mathematics, not a serialization of Rust
implementation objects.  These tests define the small fail-closed contract the
documentation validator must enforce before the calculus can be promoted.
"""
from __future__ import annotations

import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
TOOL = REPOSITORY_ROOT / "tools/docs/validate_ash_cps_calculus.py"
FIXTURES = Path(__file__).with_name("fixtures") / "ash_cps_calculus"


class AshCpsCalculusContractTests(unittest.TestCase):
    """Exercise the public validator against a complete bounded calculus."""

    def run_artifact(self, artifact: Path) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        self.assertTrue(TOOL.exists(), f"missing TASK-1989 calculus validator: {TOOL}")
        result = subprocess.run(
            ["python3", str(TOOL), "--artifact", str(artifact), "--format", "json"],
            check=False,
            capture_output=True,
            text=True,
        )
        try:
            report = json.loads(result.stdout)
        except json.JSONDecodeError as error:
            self.fail(f"calculus validator must write JSON to stdout: {error}; stderr: {result.stderr}")
        return result, report

    def run_mutation(self, mutate: object) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        with tempfile.TemporaryDirectory() as temporary_directory:
            artifact = Path(temporary_directory) / "ASH-CPS-CALCULUS.json"
            shutil.copyfile(FIXTURES / "well_formed.json", artifact)
            data = json.loads(artifact.read_text(encoding="utf-8"))
            assert callable(mutate)
            mutate(data)
            artifact.write_text(json.dumps(data), encoding="utf-8")
            return self.run_artifact(artifact)

    def assert_mutation_rejected(self, mutate: object, kind: str) -> None:
        result, report = self.run_mutation(mutate)
        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report.get("schema"), "ash-cps-calculus-validation-report/v1")
        errors = report.get("errors")
        self.assertIsInstance(errors, list)
        self.assertTrue(any(error.get("kind") == kind for error in errors if isinstance(error, dict)), errors)

    def test_well_formed_calculus_freeze_is_accepted(self) -> None:
        """The admitted kernel, effect gate, and canonical projections form one artifact."""
        result, report = self.run_artifact(FIXTURES / "well_formed.json")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report, {"schema": "ash-cps-calculus-validation-report/v1", "errors": []})

    def test_rule_ids_are_stable_and_unique(self) -> None:
        """Semantic rule identities cannot be inferred from headings or duplicated."""
        def mutate(data: dict[str, object]) -> None:
            rules = data["rules"]
            assert isinstance(rules, list)
            duplicate = dict(rules[0])
            duplicate["stage"] = "effect"
            rules.append(duplicate)
        self.assert_mutation_rejected(mutate, "duplicate_rule_id")

    def test_staging_places_effect_and_later_features_after_kernel(self) -> None:
        """Raise/Handle are gated; recursion and runtime helpers are not kernel axioms."""
        def mutate(data: dict[str, object]) -> None:
            rules = data["rules"]
            assert isinstance(rules, list)
            rules.append({"id": "SEM-CPS-RAISE-001", "stage": "kernel", "kind": "small-step"})
        self.assert_mutation_rejected(mutate, "invalid_rule_stage")

    def test_return_conflict_has_an_explicit_resolution(self) -> None:
        """PLAN-202's kernel Return and SPEC-098b's no-direct-return claim need a decision."""
        def mutate(data: dict[str, object]) -> None:
            decision = data["return_decision"]
            assert isinstance(decision, dict)
            decision["status"] = "unresolved"
        self.assert_mutation_rejected(mutate, "unresolved_return_decision")

    def test_theorem_ladder_and_admitted_fragment_are_not_implicit(self) -> None:
        """Every theorem has a status and later-only forms stay outside the admitted fragment."""
        def mutate(data: dict[str, object]) -> None:
            ladder = data["theorem_ladder"]
            assert isinstance(ladder, list)
            ladder[0].pop("status")
            admitted = data["admitted_fragment"]
            assert isinstance(admitted, dict)
            admitted["includes"].append("Raise")
        self.assert_mutation_rejected(mutate, "invalid_theorem_ladder")

    def test_examples_require_well_formed_terms_and_terminal_projections(self) -> None:
        """Canonical examples bind a syntax witness to a stable terminal observable."""
        def mutate(data: dict[str, object]) -> None:
            examples = data["examples"]
            assert isinstance(examples, list)
            examples[0].pop("expected_terminal_projection")
        self.assert_mutation_rejected(mutate, "invalid_canonical_example")

    def test_rust_storage_and_helper_behavior_cannot_be_calculus_axioms(self) -> None:
        """Implementation layouts must enter only through named refinement boundaries."""
        def mutate(data: dict[str, object]) -> None:
            trusted_base = data["trusted_base"]
            assert isinstance(trusted_base, dict)
            trusted_base["axioms"].append("Rc<RefCell<Continuation>> layout resolves affine-use")
        self.assert_mutation_rejected(mutate, "rust_helper_axiom")


if __name__ == "__main__":
    unittest.main()
