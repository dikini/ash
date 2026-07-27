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

from tools.docs.validate_semantic_task_records import markdown_sections


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
TOOL = REPOSITORY_ROOT / "tools/docs/validate_ash_cps_calculus.py"
FIXTURES = Path(__file__).with_name("fixtures") / "ash_cps_calculus"
CANONICAL_ARTIFACT = REPOSITORY_ROOT / "docs/spec/ASH-CPS-CALCULUS.json"
SEMANTIC_TRACEABILITY = REPOSITORY_ROOT / "docs/spec/SEMANTIC-TRACEABILITY.json"


REQUIRED_EFFECT_RULE_IDS = {
    "innermost_lookup": "SEM-EFFECT-LOOKUP-001",
    "record_discharge": "SEM-EFFECT-DISCHARGE-001",
    "missing_discharge": "SEM-EFFECT-MISSDISCHARGE-001",
    "deep_affine_resume": "SEM-EFFECT-RESUME-001",
    "handler_body_trap": "SEM-EFFECT-HANDLERTRAP-001",
    "provider_invocation": "SEM-EFFECT-PROVIDER-001",
    "timeout": "SEM-EFFECT-TIMEOUT-001",
    "cancellation": "SEM-EFFECT-CANCEL-001",
}

REQUIRED_EFFECT_CONFORMANCE_CASES = {
    "normal_return",
    "missing_admission",
    "malformed_unchecked_cps",
    "handler_body_trap",
    "timeout",
    "cancellation",
}

REQUIRED_EFFECT_FORMAL_JUDGMENTS = {
    "configuration_well_formedness",
    "effect_typing",
}

REQUIRED_EFFECT_TRANSITIONS = {
    "frame_lookup",
    "record_discharge",
    "handler_entry_selected_frame_removed",
    "affine_resume_reinstates_handler",
    "affine_resume_reuse_rejected",
    "handled_computation_completion",
    "resumed_tail_completion",
    "abortive_clause_completion",
    "handler_body_trap",
    "provider_external_outcome",
    "timeout_terminalization",
    "cancellation_terminalization",
    "terminalization",
}

REQUIRED_EFFECT_CONFIGURATION_TRANSITIONS = {
    "frame_lookup",
    "raise_dispatch",
    "handler_entry_selected_frame_removed",
    "affine_resume_reinstates_handler",
    "provider_external_outcome",
    "terminalization",
}

REQUIRED_EFFECT_DECLARED_TRANSITIONS = {
    "frame_lookup",
    "record_discharge",
    "raise_dispatch",
    "handler_entry_selected_frame_removed",
    "affine_resume_reinstates_handler",
    "affine_resume_reuse_rejected",
    "handled_computation_completion",
    "resumed_tail_completion",
    "abortive_clause_completion",
    "handler_body_trap",
    "provider_external_outcome",
    "missing_discharge",
    "timeout_terminalization",
    "cancellation_terminalization",
    "terminalization",
}

REQUIRED_EFFECT_ENDPOINT_CONSTRUCTORS = {
    "Lookup",
    "Dispatch",
    "selected",
    "clause",
    "tail",
    "P.success",
    "P.failure",
    "Terminal",
}

REQUIRED_EFFECT_MACHINE_DEFINITIONS = {
    "HandlerFrame": {
        "clauses",
        "done_clause",
        "residual_row",
        "captured_affine_resume",
    },
    "ProviderFrame": {
        "operation_identity",
        "authority",
        "persistent_across_invocation",
        "success_continuation",
        "failure_continuation",
    },
    "captured_affine_resume": {
        "binding",
        "consumption",
        "reinstall_handler_position",
    },
    "operation_clause_matching": {
        "frame_order",
        "clause_order",
        "done_clause",
        "residual_row",
    },
}

REQUIRED_EFFECT_SINGLE_SOURCE_AUTHORITIES = {
    "SEM-EFFECT-LOOKUP-001": "SPEC-099b §5",
    "SEM-EFFECT-RAISE-001": "SPEC-099b §5",
    "SEM-EFFECT-HANDLE-001": "SPEC-099b §5",
    "SEM-EFFECT-DISCHARGE-001": "SPEC-099b §5",
    "SEM-EFFECT-MISSDISCHARGE-001": "SPEC-099b §5",
    "SEM-EFFECT-RESUME-001": "SPEC-099b §5",
    "SEM-EFFECT-HANDLERTRAP-001": "SPEC-099b §6",
    "SEM-EFFECT-ADMISSION-001": "PLAN-203 Admission and client parity",
    "SEM-EFFECT-TIMEOUT-001": "PLAN-203 Admission and client parity",
    "SEM-EFFECT-CANCEL-001": "PLAN-203 Admission and client parity",
}

REQUIRED_EFFECT_MULTI_SOURCE_AUTHORITIES = {
    "SEM-EFFECT-PROVIDER-001": {
        "provider_boundary": "SPEC-099b §5",
        "run_control_outcomes": "PLAN-203 Admission and client parity",
    },
    "SEM-EFFECT-TERMINAL-001": {
        "kernel_return": "λAsh-CPS₀ kernel Return",
        "structured_trap": "SPEC-099b §6",
        "external_projection": "OBS-TARGET-PROJECTION-001",
        "run_control": "PLAN-203 Admission and client parity",
    },
}


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

    def run_canonical_mutation(self, mutate: object) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        """Mutate TASK-2031's complete contract without weakening the kernel fixture."""
        with tempfile.TemporaryDirectory() as temporary_directory:
            artifact = Path(temporary_directory) / "ASH-CPS-CALCULUS.json"
            shutil.copyfile(CANONICAL_ARTIFACT, artifact)
            data = json.loads(artifact.read_text(encoding="utf-8"))
            assert callable(mutate)
            mutate(data)
            artifact.write_text(json.dumps(data), encoding="utf-8")
            return self.run_artifact(artifact)

    def run_raw_canonical_mutation(self, mutate: object) -> subprocess.CompletedProcess[str]:
        """Run a malformed TASK-2031 contract without pre-parsing the report."""
        with tempfile.TemporaryDirectory() as temporary_directory:
            artifact = Path(temporary_directory) / "ASH-CPS-CALCULUS.json"
            shutil.copyfile(CANONICAL_ARTIFACT, artifact)
            data = json.loads(artifact.read_text(encoding="utf-8"))
            assert callable(mutate)
            mutate(data)
            artifact.write_text(json.dumps(data), encoding="utf-8")
            return subprocess.run(
                ["python3", str(TOOL), "--artifact", str(artifact), "--format", "json"],
                check=False,
                capture_output=True,
                text=True,
            )

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

    def test_effect_extension_has_complete_rule_indexed_correspondence_contract(self) -> None:
        """TASK-2031 makes effectful CPS a conservative, non-authorizing refinement target."""
        result, report = self.run_artifact(CANONICAL_ARTIFACT)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report, {"schema": "ash-cps-calculus-validation-report/v1", "errors": []})

        artifact = json.loads(CANONICAL_ARTIFACT.read_text(encoding="utf-8"))
        correspondence = artifact.get("effect_correspondence")
        self.assertIsInstance(
            correspondence,
            dict,
            "TASK-2031 requires a machine-checkable effect_correspondence contract, not a prose gate",
        )
        assert isinstance(correspondence, dict)
        self.assertEqual(correspondence.get("status"), "complete")
        self.assertEqual(correspondence.get("calculus"), "lambda-Ash-Effect")
        self.assertEqual(correspondence.get("conservative_extension_of"), "lambda-Ash-CPS0")

        configuration = correspondence.get("configuration")
        self.assertIsInstance(configuration, dict)
        assert isinstance(configuration, dict)
        self.assertTrue(
            {
                "term",
                "value_environment",
                "continuation_store",
                "affine_continuation_consumption",
                "ordered_handler_provider_frames",
                "discharge_record",
                "residual_closed_rows",
                "external_outcome",
            }.issubset(configuration.get("components", [])),
            configuration,
        )

        syntax = correspondence.get("syntax")
        self.assertIsInstance(syntax, dict)
        assert isinstance(syntax, dict)
        self.assertTrue(
            {
                "Raise",
                "Handle",
                "RecordDischarge",
                "HandlerFrame",
                "ProviderFrame",
                "AffineResume",
                "ExternalOutcome",
            }.issubset(syntax.get("forms", [])),
            syntax,
        )

        rules = artifact.get("rules")
        self.assertIsInstance(rules, list)
        assert isinstance(rules, list)
        rule_ids = {rule.get("id") for rule in rules if isinstance(rule, dict)}
        self.assertTrue(set(REQUIRED_EFFECT_RULE_IDS.values()).issubset(rule_ids), rule_ids)

        rule_index = correspondence.get("rule_index")
        self.assertIsInstance(rule_index, dict)
        assert isinstance(rule_index, dict)
        self.assertEqual(
            {name: rule_index.get(name) for name in REQUIRED_EFFECT_RULE_IDS},
            REQUIRED_EFFECT_RULE_IDS,
        )
        mapping = correspondence.get("mapping")
        self.assertIsInstance(mapping, dict)
        assert isinstance(mapping, dict)
        for rule_id in REQUIRED_EFFECT_RULE_IDS.values():
            row = mapping.get(rule_id)
            self.assertIsInstance(row, dict, rule_id)
            assert isinstance(row, dict)
            self.assertTrue(
                all(isinstance(row.get(field), str) and row[field].strip() for field in (
                    "cps_artifact",
                    "target_operational",
                    "engine_view",
                    "terminal_projection",
                )),
                row,
            )

        conformance = correspondence.get("conformance_obligations")
        self.assertIsInstance(conformance, list)
        assert isinstance(conformance, list)
        cases = {case.get("case") for case in conformance if isinstance(case, dict)}
        self.assertTrue(REQUIRED_EFFECT_CONFORMANCE_CASES.issubset(cases), cases)
        for case in conformance:
            if isinstance(case, dict) and case.get("case") in REQUIRED_EFFECT_CONFORMANCE_CASES:
                self.assertTrue(set(case.get("rule_ids", [])).issubset(rule_ids), case)
                self.assertIsInstance(case.get("terminal_outcome"), str, case)

        authority = correspondence.get("authority_boundary")
        self.assertIsInstance(authority, dict)
        assert isinstance(authority, dict)
        self.assertTrue(authority.get("rows_are_requirements_only"), authority)
        self.assertEqual(authority.get("frame_installation"), "separately_authorized_admission_only")
        self.assertTrue(authority.get("no_second_execution_route"), authority)

        verus = correspondence.get("verus_candidates")
        self.assertIsInstance(verus, list)
        assert isinstance(verus, list)
        self.assertTrue(verus, "TASK-2031 must name selected high-value Verus candidates")
        self.assertTrue(all(candidate.get("status") == "deferred" for candidate in verus if isinstance(candidate, dict)), verus)

    def test_effect_extension_declares_formal_relations_authority_and_effect_only_coverage(self) -> None:
        """TASK-2031's complete extension is a formal, correctly sourced non-kernel contract."""
        artifact = json.loads(CANONICAL_ARTIFACT.read_text(encoding="utf-8"))
        correspondence = artifact["effect_correspondence"]
        assert isinstance(correspondence, dict)

        formal_relations = correspondence.get("formal_relations")
        self.assertIsInstance(
            formal_relations,
            dict,
            "the effect extension must machine-record its well-formedness, typing, and transition relations",
        )
        assert isinstance(formal_relations, dict)
        for relation_group, required_relations in (
            ("judgments", REQUIRED_EFFECT_FORMAL_JUDGMENTS),
            ("transitions", REQUIRED_EFFECT_TRANSITIONS),
        ):
            relations = formal_relations.get(relation_group)
            self.assertIsInstance(relations, dict, relation_group)
            assert isinstance(relations, dict)
            self.assertTrue(required_relations.issubset(relations), relations)
            for relation_name in required_relations:
                relation = relations[relation_name]
                self.assertIsInstance(relation, dict, relation_name)
                assert isinstance(relation, dict)
                self.assertIsInstance(relation.get("notation"), str, relation)
                self.assertTrue(relation["notation"].strip(), relation)
                self.assertTrue(relation.get("rule_ids"), relation)

        coverage = correspondence.get("effect_extension_coverage")
        self.assertEqual(
            coverage,
            {
                "status": "complete",
                "separate_from_admitted_kernel_fragment": True,
                "kernel_fragment_excludes_effect_forms": True,
            },
            "the frozen kernel fragment must not be presented as the complete effect extension",
        )

        mapping = correspondence["mapping"]
        assert isinstance(mapping, dict)
        self.assertEqual(
            {
                rule_id: mapping[rule_id].get("target_authority")
                for rule_id in REQUIRED_EFFECT_SINGLE_SOURCE_AUTHORITIES
            },
            REQUIRED_EFFECT_SINGLE_SOURCE_AUTHORITIES,
        )
        for rule_id, expected_authorities in REQUIRED_EFFECT_MULTI_SOURCE_AUTHORITIES.items():
            row = mapping.get(rule_id)
            self.assertIsInstance(row, dict, rule_id)
            assert isinstance(row, dict)
            self.assertNotIn(
                "target_authority",
                row,
                "provider and terminal rules must not flatten their distinct source authorities",
            )
            self.assertEqual(row.get("target_authorities"), expected_authorities, row)

    def test_effect_configuration_requires_a_discharge_record_and_validator_rejects_its_omission(self) -> None:
        """The effect configuration carries discharge evidence independently of residual rows."""
        artifact = json.loads(CANONICAL_ARTIFACT.read_text(encoding="utf-8"))
        correspondence = artifact["effect_correspondence"]
        assert isinstance(correspondence, dict)
        configuration = correspondence["configuration"]
        assert isinstance(configuration, dict)
        self.assertIn("discharge_record", configuration.get("components", []), configuration)

        def remove_discharge_record(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            configuration = correspondence["configuration"]
            assert isinstance(configuration, dict)
            components = configuration["components"]
            assert isinstance(components, list)
            components.remove("discharge_record")

        result, report = self.run_canonical_mutation(remove_discharge_record)
        self.assertNotEqual(result.returncode, 0, report)

    def test_handler_completion_routes_distinguish_done_from_abortive_clause_results(self) -> None:
        """SPEC-099b §5 routes only handled computation and resumed-tail completion through done."""
        artifact = json.loads(CANONICAL_ARTIFACT.read_text(encoding="utf-8"))
        correspondence = artifact["effect_correspondence"]
        assert isinstance(correspondence, dict)
        formal_relations = correspondence["formal_relations"]
        assert isinstance(formal_relations, dict)
        transitions = formal_relations["transitions"]
        assert isinstance(transitions, dict)

        for relation_name in ("handled_computation_completion", "resumed_tail_completion"):
            relation = transitions.get(relation_name)
            self.assertIsInstance(relation, dict, relation_name)
            assert isinstance(relation, dict)
            self.assertEqual(relation.get("completion_route"), "done_once", relation)
            self.assertIn("done", relation.get("notation", ""), relation)

        abortive_clause = transitions.get("abortive_clause_completion")
        self.assertIsInstance(abortive_clause, dict, "abortive operation-clause completion needs its own relation")
        assert isinstance(abortive_clause, dict)
        self.assertEqual(abortive_clause.get("completion_route"), "handler_result_directly", abortive_clause)
        self.assertNotIn("done", abortive_clause.get("notation", ""), abortive_clause)

        handler_body_done_claims = [
            relation_name
            for relation_name, relation in transitions.items()
            if isinstance(relation, dict)
            and "handler-body" in str(relation.get("notation", ""))
            and "done" in str(relation.get("notation", ""))
        ]
        self.assertEqual(handler_body_done_claims, [], transitions)

    def test_effect_contract_validator_rejects_collapsed_handler_completion_routes(self) -> None:
        """Completion-route metadata is fail-closed rather than a prose-only semantic claim."""
        def collapse_abortive_clause_into_done(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            formal_relations = correspondence["formal_relations"]
            assert isinstance(formal_relations, dict)
            transitions = formal_relations["transitions"]
            assert isinstance(transitions, dict)
            abortive_clause = transitions.get("abortive_clause_completion")
            if not isinstance(abortive_clause, dict):
                abortive_clause = {}
                transitions["abortive_clause_completion"] = abortive_clause
            abortive_clause["completion_route"] = "done_once"
            abortive_clause["notation"] = "handler-body Return(v) →E done(v) exactly-once"
            abortive_clause["rule_ids"] = ["SEM-EFFECT-HANDLE-001"]

        result, report = self.run_canonical_mutation(collapse_abortive_clause_into_done)
        self.assertNotEqual(result.returncode, 0, report)

    def test_effect_contract_has_machine_definitions_and_configuration_transition_endpoints(self) -> None:
        """Required effect objects cannot be represented only by names in relation notation."""
        artifact = json.loads(CANONICAL_ARTIFACT.read_text(encoding="utf-8"))
        correspondence = artifact["effect_correspondence"]
        assert isinstance(correspondence, dict)
        definitions = correspondence.get("formal_definitions")
        self.assertIsInstance(definitions, dict)
        assert isinstance(definitions, dict)
        for definition_name, required_fields in REQUIRED_EFFECT_MACHINE_DEFINITIONS.items():
            definition = definitions.get(definition_name)
            self.assertIsInstance(definition, dict, definition_name)
            assert isinstance(definition, dict)
            self.assertTrue(required_fields.issubset(definition), definition)

        provider = definitions["ProviderFrame"]
        assert isinstance(provider, dict)
        self.assertIs(provider.get("persistent_across_invocation"), True, provider)

        formal_relations = correspondence["formal_relations"]
        assert isinstance(formal_relations, dict)
        transitions = formal_relations["transitions"]
        assert isinstance(transitions, dict)
        for transition_name in REQUIRED_EFFECT_CONFIGURATION_TRANSITIONS:
            transition = transitions.get(transition_name)
            self.assertIsInstance(transition, dict, transition_name)
            assert isinstance(transition, dict)
            self.assertEqual(transition.get("relation_kind"), "configuration-to-configuration", transition)
            self.assertIsInstance(transition.get("source_configuration"), str, transition)
            self.assertTrue(transition["source_configuration"].strip(), transition)
            self.assertIsInstance(transition.get("target_configuration"), str, transition)
            self.assertTrue(transition["target_configuration"].strip(), transition)

    def test_effect_contract_validator_rejects_opaque_machine_definitions_and_non_configuration_steps(self) -> None:
        """λAsh-Effect needs formal frames and real configuration-to-configuration transitions."""
        def erase_machine_definitions(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            correspondence["formal_definitions"] = {}

        def erase_provider_step_target(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            formal_relations = correspondence["formal_relations"]
            assert isinstance(formal_relations, dict)
            transitions = formal_relations["transitions"]
            assert isinstance(transitions, dict)
            provider = transitions["provider_external_outcome"]
            assert isinstance(provider, dict)
            provider.pop("target_configuration", None)

        rejected: list[tuple[str, int, list[object]]] = []
        for mutate, expected_kind in (
            (erase_machine_definitions, "incomplete_effect_machine_definitions"),
            (erase_provider_step_target, "invalid_effect_configuration_transition"),
        ):
            result, report = self.run_canonical_mutation(mutate)
            errors = report.get("errors", [])
            is_rejected = result.returncode != 0 and any(
                isinstance(error, dict) and error.get("kind") == expected_kind for error in errors
            )
            if not is_rejected:
                rejected.append((expected_kind, result.returncode, errors if isinstance(errors, list) else [errors]))
        self.assertEqual(rejected, [])

    def test_every_effect_transition_has_vocabulary_closed_configuration_endpoints(self) -> None:
        """Notation may abbreviate state, but its endpoint constructors must be declared machine vocabulary."""
        artifact = json.loads(CANONICAL_ARTIFACT.read_text(encoding="utf-8"))
        correspondence = artifact["effect_correspondence"]
        assert isinstance(correspondence, dict)
        vocabulary = correspondence.get("endpoint_vocabulary")
        self.assertIsInstance(vocabulary, list)
        assert isinstance(vocabulary, list)
        self.assertTrue(REQUIRED_EFFECT_ENDPOINT_CONSTRUCTORS.issubset(vocabulary), vocabulary)

        formal_relations = correspondence["formal_relations"]
        assert isinstance(formal_relations, dict)
        transitions = formal_relations["transitions"]
        assert isinstance(transitions, dict)
        self.assertTrue(REQUIRED_EFFECT_DECLARED_TRANSITIONS.issubset(transitions), transitions)
        for transition_name in REQUIRED_EFFECT_DECLARED_TRANSITIONS:
            transition = transitions[transition_name]
            self.assertIsInstance(transition, dict, transition_name)
            assert isinstance(transition, dict)
            self.assertEqual(transition.get("relation_kind"), "configuration-to-configuration", transition)
            for endpoint in ("source_configuration", "target_configuration"):
                self.assertIsInstance(transition.get(endpoint), str, (transition_name, endpoint))
                self.assertTrue(transition[endpoint].strip(), (transition_name, endpoint))
            referenced = transition.get("endpoint_vocabulary")
            self.assertIsInstance(referenced, list, transition_name)
            assert isinstance(referenced, list)
            self.assertTrue(set(referenced).issubset(vocabulary), (transition_name, referenced, vocabulary))

    def test_effect_validator_rejects_unknown_endpoint_vocabulary_and_missing_nonprimary_transition_endpoints(self) -> None:
        """Coverage applies to all declared transitions, not merely the initial lookup/provider subset."""
        def introduce_unknown_constructor(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            formal_relations = correspondence["formal_relations"]
            assert isinstance(formal_relations, dict)
            transitions = formal_relations["transitions"]
            assert isinstance(transitions, dict)
            lookup = transitions["frame_lookup"]
            assert isinstance(lookup, dict)
            lookup["endpoint_vocabulary"] = ["UnknownConstructor"]

        def erase_reuse_trap_target(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            formal_relations = correspondence["formal_relations"]
            assert isinstance(formal_relations, dict)
            transitions = formal_relations["transitions"]
            assert isinstance(transitions, dict)
            reuse_trap = transitions["affine_resume_reuse_rejected"]
            assert isinstance(reuse_trap, dict)
            reuse_trap.pop("target_configuration", None)

        for mutate, expected_kind in (
            (introduce_unknown_constructor, "unknown_effect_endpoint_vocabulary"),
            (erase_reuse_trap_target, "invalid_effect_configuration_transition"),
        ):
            with self.subTest(mutate=mutate.__name__):
                result, report = self.run_canonical_mutation(mutate)
                self.assertNotEqual(result.returncode, 0, report)
                errors = report.get("errors", [])
                self.assertTrue(
                    any(isinstance(error, dict) and error.get("kind") == expected_kind for error in errors),
                    errors,
                )

    def test_effect_reduction_declares_a_single_raise_successor_chain(self) -> None:
        """Raise cannot compete with lookup/dispatch as two unrelated small steps from one configuration."""
        artifact = json.loads(CANONICAL_ARTIFACT.read_text(encoding="utf-8"))
        correspondence = artifact["effect_correspondence"]
        assert isinstance(correspondence, dict)
        determinism = correspondence.get("reduction_determinism")
        self.assertIsInstance(determinism, dict)
        assert isinstance(determinism, dict)
        self.assertEqual(determinism.get("strategy"), "chained-raise-lookup-dispatch")
        self.assertEqual(determinism.get("chain"), ["frame_lookup", "raise_dispatch"])
        self.assertIs(determinism.get("raise_dispatch_single_next_configuration"), True)
        formal_relations = correspondence["formal_relations"]
        assert isinstance(formal_relations, dict)
        transitions = formal_relations["transitions"]
        assert isinstance(transitions, dict)
        lookup, dispatch = transitions["frame_lookup"], transitions["raise_dispatch"]
        assert isinstance(lookup, dict) and isinstance(dispatch, dict)
        self.assertEqual(dispatch.get("source_configuration"), lookup.get("target_configuration"))

    def test_effect_validator_rejects_competing_raise_step(self) -> None:
        """Mutating dispatch back to a second Raise step must fail the determinism contract."""
        def make_dispatch_compete_with_lookup(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            formal_relations = correspondence["formal_relations"]
            assert isinstance(formal_relations, dict)
            transitions = formal_relations["transitions"]
            assert isinstance(transitions, dict)
            lookup = transitions["frame_lookup"]
            dispatch = transitions["raise_dispatch"]
            assert isinstance(lookup, dict) and isinstance(dispatch, dict)
            dispatch["source_configuration"] = lookup["source_configuration"]

        result, report = self.run_canonical_mutation(make_dispatch_compete_with_lookup)
        self.assertNotEqual(result.returncode, 0, report)
        errors = report.get("errors", [])
        self.assertTrue(
            any(isinstance(error, dict) and error.get("kind") == "nondeterministic_effect_reduction" for error in errors),
            errors,
        )

    def test_provider_transition_preserves_captured_resume_and_distinguishes_success_from_failure(self) -> None:
        """Provider outcomes retain r, frame identity, and state rather than replacing execution with a terminal shortcut."""
        artifact = json.loads(CANONICAL_ARTIFACT.read_text(encoding="utf-8"))
        correspondence = artifact["effect_correspondence"]
        assert isinstance(correspondence, dict)
        formal_relations = correspondence["formal_relations"]
        assert isinstance(formal_relations, dict)
        transitions = formal_relations["transitions"]
        assert isinstance(transitions, dict)
        provider = transitions["provider_external_outcome"]
        assert isinstance(provider, dict)
        self.assertEqual(
            provider.get("invocation_bindings"),
            {"arguments": "a*", "captured_resume": "r", "provider_frame": "P"},
        )
        self.assertEqual(provider.get("frame_persistence"), "preserved")
        outcomes = provider.get("outcomes")
        self.assertEqual(
            outcomes,
            {
                "success": {"target": "r(value)", "retains_configuration_state": True},
                "failure": {"target": "ExternalOutcome(ξ)", "retains_configuration_state": True},
            },
        )

    def test_effect_validator_rejects_provider_success_that_loses_captured_resume(self) -> None:
        """Provider success cannot discard the captured CPS continuation r."""
        def discard_provider_resume(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            formal_relations = correspondence["formal_relations"]
            assert isinstance(formal_relations, dict)
            transitions = formal_relations["transitions"]
            assert isinstance(transitions, dict)
            provider = transitions["provider_external_outcome"]
            assert isinstance(provider, dict)
            provider["target_configuration"] = "⟨P.success(a*), η, κ, α, Fpre · P · Fpost, δ, ρ, ξ⟩"

        result, report = self.run_canonical_mutation(discard_provider_resume)
        self.assertNotEqual(result.returncode, 0, report)
        errors = report.get("errors", [])
        self.assertTrue(
            any(isinstance(error, dict) and error.get("kind") == "lost_provider_resume" for error in errors),
            errors,
        )

    def test_global_effect_determinism_is_explicitly_disjoint_or_narrowed_to_raise_dispatch(self) -> None:
        """Generic terminal rules may not overlap handler, trap, timeout, or cancellation steps under a global claim."""
        artifact = json.loads(CANONICAL_ARTIFACT.read_text(encoding="utf-8"))
        correspondence = artifact["effect_correspondence"]
        assert isinstance(correspondence, dict)
        determinism = correspondence["reduction_determinism"]
        assert isinstance(determinism, dict)
        globally_deterministic = determinism.get("single_next_configuration_per_state")
        if globally_deterministic is True:
            disjointness = determinism.get("disjointness")
            self.assertIsInstance(disjointness, dict)
            assert isinstance(disjointness, dict)
            self.assertEqual(disjointness.get("method"), "explicit-state-phase-or-mutually-exclusive-premises")
            protected = disjointness.get("protected_transition_pairs")
            self.assertIsInstance(protected, list)
            assert isinstance(protected, list)
            pairs = {frozenset(pair) for pair in protected if isinstance(pair, list)}
            for transition in (
                "handled_computation_completion",
                "resumed_tail_completion",
                "abortive_clause_completion",
                "handler_body_trap",
                "timeout_terminalization",
                "cancellation_terminalization",
            ):
                self.assertIn(frozenset((transition, "terminalization")), pairs)
        else:
            self.assertIs(globally_deterministic, False)
            self.assertEqual(determinism.get("scope"), "raise-lookup-dispatch-only")
            self.assertIs(determinism.get("raise_dispatch_single_next_configuration"), True)

    def test_effect_validator_rejects_overlapping_terminal_source_under_a_global_claim(self) -> None:
        """Handler completion, traps, and external outcomes must not silently share terminalization's source state."""
        def overlap_terminalization_with(transition_name: str) -> object:
            def mutate(data: dict[str, object]) -> None:
                correspondence = data["effect_correspondence"]
                assert isinstance(correspondence, dict)
                determinism = correspondence["reduction_determinism"]
                assert isinstance(determinism, dict)
                determinism["single_next_configuration_per_state"] = True
                formal_relations = correspondence["formal_relations"]
                assert isinstance(formal_relations, dict)
                transitions = formal_relations["transitions"]
                assert isinstance(transitions, dict)
                terminalization = transitions["terminalization"]
                other = transitions[transition_name]
                assert isinstance(terminalization, dict) and isinstance(other, dict)
                terminalization["source_configuration"] = other["source_configuration"]
                terminal_vocabulary = terminalization["endpoint_vocabulary"]
                other_vocabulary = other["endpoint_vocabulary"]
                assert isinstance(terminal_vocabulary, list) and isinstance(other_vocabulary, list)
                terminalization["endpoint_vocabulary"] = sorted(set(terminal_vocabulary) | set(other_vocabulary))
            return mutate

        for transition_name in (
            "handled_computation_completion",
            "handler_body_trap",
            "timeout_terminalization",
            "cancellation_terminalization",
        ):
            with self.subTest(transition=transition_name):
                result, report = self.run_canonical_mutation(overlap_terminalization_with(transition_name))
                self.assertNotEqual(result.returncode, 0, report)
                errors = report.get("errors", [])
                self.assertTrue(
                    any(isinstance(error, dict) and error.get("kind") == "overlapping_effect_transition" for error in errors),
                    errors,
                )

    def test_nonterminal_effect_outcomes_have_terminal_envelope_successor_chains(self) -> None:
        """Handler and missing-discharge outcomes reach the terminal envelope through declared CPS steps only."""
        artifact = json.loads(CANONICAL_ARTIFACT.read_text(encoding="utf-8"))
        correspondence = artifact["effect_correspondence"]
        assert isinstance(correspondence, dict)
        chains = correspondence.get("terminal_successor_chains")
        self.assertEqual(
            chains,
            {
                "done": ["done_terminalization", "terminalization"],
                "handler-result": ["handler_result_terminalization", "terminalization"],
                "MissingDischarge": ["missing_discharge_terminalization", "terminalization"],
            },
        )
        authority = correspondence["authority_boundary"]
        assert isinstance(authority, dict)
        self.assertTrue(authority.get("no_second_execution_route"), authority)
        formal_relations = correspondence["formal_relations"]
        assert isinstance(formal_relations, dict)
        transitions = formal_relations["transitions"]
        assert isinstance(transitions, dict)
        for chain in chains.values():
            for transition_name in chain:
                transition = transitions.get(transition_name)
                self.assertIsInstance(transition, dict, transition_name)
                assert isinstance(transition, dict)
                self.assertEqual(transition.get("relation_kind"), "configuration-to-configuration", transition)

    def test_effect_validator_rejects_a_missing_nonterminal_terminal_chain_step(self) -> None:
        """Deleting a done-to-envelope successor must not leave a nonterminal result stranded."""
        def delete_done_successor(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            formal_relations = correspondence["formal_relations"]
            assert isinstance(formal_relations, dict)
            transitions = formal_relations["transitions"]
            assert isinstance(transitions, dict)
            transitions.pop("done_terminalization", None)

        result, report = self.run_canonical_mutation(delete_done_successor)
        self.assertNotEqual(result.returncode, 0, report)
        errors = report.get("errors", [])
        self.assertTrue(
            any(isinstance(error, dict) and error.get("kind") == "incomplete_effect_terminal_successor_chain" for error in errors),
            errors,
        )

    def test_effect_validator_checks_endpoint_text_against_declared_vocabulary(self) -> None:
        """A transition cannot hide an undeclared constructor behind unchanged vocabulary metadata."""
        def replace_target_with_undeclared_constructor(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            formal_relations = correspondence["formal_relations"]
            assert isinstance(formal_relations, dict)
            transitions = formal_relations["transitions"]
            assert isinstance(transitions, dict)
            discharge = transitions["record_discharge"]
            assert isinstance(discharge, dict)
            discharge["target_configuration"] = "⟨UndeclaredEndpoint(d), η, κ, α, F, δ, ρ, ξ⟩"

        result, report = self.run_canonical_mutation(replace_target_with_undeclared_constructor)
        self.assertNotEqual(result.returncode, 0, report)
        errors = report.get("errors", [])
        self.assertTrue(
            any(isinstance(error, dict) and error.get("kind") == "undeclared_effect_endpoint_constructor" for error in errors),
            errors,
        )

    def test_completion_and_terminal_steps_use_an_acyclic_phase_graph(self) -> None:
        """Terminal projection states cannot re-enter generic handler completion after done/result conversion."""
        artifact = json.loads(CANONICAL_ARTIFACT.read_text(encoding="utf-8"))
        correspondence = artifact["effect_correspondence"]
        assert isinstance(correspondence, dict)
        formal_relations = correspondence["formal_relations"]
        assert isinstance(formal_relations, dict)
        transitions = formal_relations["transitions"]
        assert isinstance(transitions, dict)
        completion_steps = {
            "handled_computation_completion",
            "resumed_tail_completion",
            "abortive_clause_completion",
            "done_terminalization",
            "handler_result_terminalization",
            "missing_discharge_terminalization",
            "terminalization",
        }
        phase_edges: dict[str, set[str]] = {}
        for transition_name in completion_steps:
            transition = transitions.get(transition_name)
            self.assertIsInstance(transition, dict, transition_name)
            assert isinstance(transition, dict)
            phase = transition.get("state_phase")
            self.assertIsInstance(phase, dict, transition_name)
            assert isinstance(phase, dict)
            source, target = phase.get("source"), phase.get("target")
            self.assertIsInstance(source, str, transition_name)
            self.assertIsInstance(target, str, transition_name)
            assert isinstance(source, str) and isinstance(target, str)
            self.assertTrue(source.strip() and target.strip() and source != target, transition)
            phase_edges.setdefault(source, set()).add(target)

        terminal_phase = transitions["terminalization"]["state_phase"]
        assert isinstance(terminal_phase, dict)
        terminal_source = terminal_phase["source"]
        self.assertIsInstance(terminal_source, str)
        assert isinstance(terminal_source, str)
        for transition_name in ("handled_computation_completion", "resumed_tail_completion", "abortive_clause_completion"):
            phase = transitions[transition_name]["state_phase"]
            assert isinstance(phase, dict)
            self.assertNotEqual(phase.get("source"), terminal_source, transition_name)

        visiting: set[str] = set()
        visited: set[str] = set()

        def visit(phase: str) -> bool:
            if phase in visiting:
                return True
            if phase in visited:
                return False
            visiting.add(phase)
            cycle = any(visit(target) for target in phase_edges.get(phase, set()))
            visiting.remove(phase)
            visited.add(phase)
            return cycle

        self.assertFalse(any(visit(phase) for phase in phase_edges), phase_edges)

    def test_effect_validator_rejects_generic_return_reentry_and_completion_cycles(self) -> None:
        """Reintroducing a generic Return completion source or a phase cycle must reject."""
        def restore_generic_return_source(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            transitions = correspondence["formal_relations"]["transitions"]
            assert isinstance(transitions, dict)
            handled = transitions["handled_computation_completion"]
            assert isinstance(handled, dict)
            handled["source_configuration"] = "⟨Return(v), η, κ, α, F, δ, ρ, ξ⟩"
            handled["state_phase"] = {"source": "generic-return", "target": "done"}

        def create_done_cycle(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            transitions = correspondence["formal_relations"]["transitions"]
            assert isinstance(transitions, dict)
            handled = transitions["handled_computation_completion"]
            done = transitions["done_terminalization"]
            assert isinstance(handled, dict) and isinstance(done, dict)
            handled["state_phase"] = {"source": "handled-return", "target": "done"}
            done["state_phase"] = {"source": "done", "target": "handled-return"}

        for mutate, expected_kind in (
            (restore_generic_return_source, "invalid_effect_completion_phase"),
            (create_done_cycle, "cyclic_effect_transition_graph"),
        ):
            with self.subTest(mutate=mutate.__name__):
                result, report = self.run_canonical_mutation(mutate)
                self.assertNotEqual(result.returncode, 0, report)
                errors = report.get("errors", [])
                self.assertTrue(
                    any(isinstance(error, dict) and error.get("kind") == expected_kind for error in errors),
                    errors,
                )

    def test_handler_body_trap_is_phase_separated_from_terminal_trap_projection(self) -> None:
        """A handler-body trap must be a tagged phase step or declared auxiliary relation, never Trap → Trap."""
        artifact = json.loads(CANONICAL_ARTIFACT.read_text(encoding="utf-8"))
        correspondence = artifact["effect_correspondence"]
        assert isinstance(correspondence, dict)
        transitions = correspondence["formal_relations"]["transitions"]
        assert isinstance(transitions, dict)
        trap = transitions["handler_body_trap"]
        terminalization = transitions["terminalization"]
        assert isinstance(trap, dict) and isinstance(terminalization, dict)
        if trap.get("relation_kind") == "auxiliary-handler-trap":
            self.assertIs(trap.get("excluded_from_config_step_terminalization"), True, trap)
        else:
            self.assertEqual(trap.get("relation_kind"), "configuration-to-configuration", trap)
            trap_phase = trap.get("state_phase")
            terminal_phase = terminalization.get("state_phase")
            self.assertIsInstance(trap_phase, dict)
            self.assertIsInstance(terminal_phase, dict)
            assert isinstance(trap_phase, dict) and isinstance(terminal_phase, dict)
            self.assertNotEqual(trap_phase.get("source"), terminal_phase.get("source"), trap)
            self.assertNotEqual(trap_phase.get("source"), trap_phase.get("target"), trap)

    def test_effect_validator_rejects_generic_handler_body_trap_self_loop(self) -> None:
        """Restoring an untagged Trap → Trap step must not silently overlap terminalization."""
        def restore_generic_trap_self_loop(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            transitions = correspondence["formal_relations"]["transitions"]
            assert isinstance(transitions, dict)
            trap = transitions["handler_body_trap"]
            assert isinstance(trap, dict)
            trap["relation_kind"] = "configuration-to-configuration"
            trap["source_configuration"] = "⟨Trap(reason), η, κ, α, F, δ, ρ, ξ⟩"
            trap["target_configuration"] = "⟨Trap(reason), η, κ, α, F, δ, ρ, ξ⟩"
            trap["state_phase"] = {"source": "generic-trap", "target": "generic-trap"}

        result, report = self.run_canonical_mutation(restore_generic_trap_self_loop)
        self.assertNotEqual(result.returncode, 0, report)
        errors = report.get("errors", [])
        self.assertTrue(
            any(isinstance(error, dict) and error.get("kind") == "invalid_handler_body_trap_transition" for error in errors),
            errors,
        )

    def test_effect_determinism_theorem_is_scoped_to_the_declared_raise_dispatch_route(self) -> None:
        """The theorem ladder cannot claim full effect-fragment determinism without a matching global proof."""
        artifact = json.loads(CANONICAL_ARTIFACT.read_text(encoding="utf-8"))
        theorem = next(theorem for theorem in artifact["theorem_ladder"] if theorem["id"] == "THM-EFFECT-DET-001")
        assert isinstance(theorem, dict)
        self.assertEqual(theorem.get("scope"), "raise-lookup-dispatch-only")
        self.assertNotIn("effect fragment is deterministic", theorem.get("claim", "").lower())

    def test_effect_validator_rejects_a_broadened_effect_determinism_theorem(self) -> None:
        """A local deterministic route cannot be relabelled as an effect-fragment theorem."""
        def broaden_theorem(data: dict[str, object]) -> None:
            ladder = data["theorem_ladder"]
            assert isinstance(ladder, list)
            theorem = next(item for item in ladder if item["id"] == "THM-EFFECT-DET-001")
            assert isinstance(theorem, dict)
            theorem["scope"] = "effect-fragment"
            theorem["claim"] = "the effect fragment is deterministic relative to a fixed provider oracle"

        result, report = self.run_canonical_mutation(broaden_theorem)
        self.assertNotEqual(result.returncode, 0, report)
        errors = report.get("errors", [])
        self.assertTrue(
            any(isinstance(error, dict) and error.get("kind") == "effect_determinism_theorem_scope_mismatch" for error in errors),
            errors,
        )

    def test_provider_outcomes_distinguish_cps_resumption_from_continuous_terminal_paths(self) -> None:
        """Provider success resumes r(value); only failure, timeout, and cancellation take external terminal paths."""
        artifact = json.loads(CANONICAL_ARTIFACT.read_text(encoding="utf-8"))
        correspondence = artifact["effect_correspondence"]
        assert isinstance(correspondence, dict)
        chains = correspondence.get("provider_outcome_routes")
        self.assertEqual(
            chains,
            {
                "success": {
                    "outcome_kind": "success",
                    "disposition": "cps-resumption",
                    "transitions": ["provider_success_resume"],
                    "resumption_target": "r(value)",
                    "terminalizing": False,
                },
                "generic_failure": {
                    "outcome_kind": "generic_failure",
                    "disposition": "external-terminal",
                    "transitions": ["provider_failure_terminalization", "generic_failure_terminalization"],
                    "terminal_configuration": "Terminal(ExternalOutcome(ξ))",
                    "terminalizing": True,
                },
                "timeout": {
                    "outcome_kind": "timeout",
                    "disposition": "external-terminal",
                    "transitions": ["timeout_terminalization", "timeout_external_terminalization"],
                    "terminal_configuration": "Terminal(ExternalOutcome(timeout))",
                    "terminalizing": True,
                },
                "cancellation": {
                    "outcome_kind": "cancellation",
                    "disposition": "external-terminal",
                    "transitions": ["cancellation_terminalization", "cancellation_external_terminalization"],
                    "terminal_configuration": "Terminal(ExternalOutcome(cancelled))",
                    "terminalizing": True,
                },
            },
        )
        authority = correspondence["authority_boundary"]
        assert isinstance(authority, dict)
        self.assertTrue(authority.get("no_second_execution_route"), authority)
        transitions = correspondence["formal_relations"]["transitions"]
        assert isinstance(transitions, dict)
        success = chains["success"]
        assert isinstance(success, dict)
        success_transition = transitions["provider_success_resume"]
        assert isinstance(success_transition, dict)
        self.assertEqual(success_transition.get("outcome_branch"), "success")
        self.assertEqual(success_transition.get("target_configuration"), "⟨r(value), η, κ, α, Fpre · P · Fpost, δ, ρ, ξ⟩")
        self.assertEqual(success_transition.get("state_phase", {}).get("target"), "provider-success-resume")
        self.assertNotIn("provider_success_terminalization", transitions)
        self.assertNotIn("identity_continuation_bounded_domain", success)
        for outcome_kind, chain in chains.items():
            assert isinstance(chain, dict)
            if chain["terminalizing"] is False:
                continue
            for transition_name in chain["transitions"]:
                transition = transitions.get(transition_name)
                self.assertIsInstance(transition, dict, (outcome_kind, transition_name))
                assert isinstance(transition, dict)
                self.assertEqual(transition.get("relation_kind"), "configuration-to-configuration", transition)
                phase = transition.get("state_phase")
                self.assertIsInstance(phase, dict, transition_name)
                assert isinstance(phase, dict)
                self.assertTrue(all(isinstance(phase.get(key), str) and phase[key].strip() for key in ("source", "target")), phase)
            first = transitions[chain["transitions"][0]]
            second = transitions[chain["transitions"][1]]
            assert isinstance(first, dict) and isinstance(second, dict)
            self.assertEqual(first.get("target_configuration"), second.get("source_configuration"), outcome_kind)
            self.assertEqual(first.get("outcome_kind"), outcome_kind, first)
            self.assertEqual(second.get("outcome_kind"), outcome_kind, second)
            self.assertIn("Terminal(ExternalOutcome", second.get("target_configuration", ""), second)

    def test_effect_validator_rejects_a_missing_or_discontinuous_generic_provider_failure_terminal_path(self) -> None:
        """Generic provider failure cannot be left as a stuck ExternalOutcome branch."""
        def remove_generic_failure_terminalization(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            transitions = correspondence["formal_relations"]["transitions"]
            assert isinstance(transitions, dict)
            transitions.pop("provider_failure_terminalization", None)

        def mismatch_generic_failure_endpoint(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            transitions = correspondence["formal_relations"]["transitions"]
            assert isinstance(transitions, dict)
            failure = transitions["provider_failure_terminalization"]
            assert isinstance(failure, dict)
            failure["target_configuration"] = "⟨TerminalReady(ExternalOutcome(other)), η, κ, α, Fpre · P · Fpost, δ, ρ, other⟩"

        for mutate, expected_kind in (
            (remove_generic_failure_terminalization, "incomplete_provider_outcome_terminal_chain"),
            (mismatch_generic_failure_endpoint, "discontinuous_provider_outcome_terminal_chain"),
        ):
            with self.subTest(mutate=mutate.__name__):
                result, report = self.run_canonical_mutation(mutate)
                self.assertNotEqual(result.returncode, 0, report)
                errors = report.get("errors", [])
                self.assertTrue(
                    any(isinstance(error, dict) and error.get("kind") == expected_kind for error in errors),
                    errors,
                )

    def test_external_outcome_has_one_canonical_terminal_projection_owner_and_shape(self) -> None:
        """External outcomes project once as Terminal(ExternalOutcome(...)), either generically or provider-specifically."""
        artifact = json.loads(CANONICAL_ARTIFACT.read_text(encoding="utf-8"))
        correspondence = artifact["effect_correspondence"]
        assert isinstance(correspondence, dict)
        projection = correspondence.get("external_projection_contract")
        self.assertIsInstance(projection, dict)
        assert isinstance(projection, dict)
        self.assertEqual(projection.get("terminal_shape"), "Terminal(ExternalOutcome(ξ))")
        owner = projection.get("owner")
        self.assertIn(owner, {"generic-terminalization", "provider-specific-terminalization"})
        transitions = correspondence["formal_relations"]["transitions"]
        assert isinstance(transitions, dict)
        provider_routes = correspondence["provider_outcome_routes"]
        assert isinstance(provider_routes, dict)

        if owner == "generic-terminalization":
            self.assertEqual(projection.get("owner_transition"), "terminalization")
            generic = transitions["terminalization"]
            assert isinstance(generic, dict)
            self.assertIn("TerminalReady(ExternalOutcome(ξ))", generic.get("source_configuration", ""), generic)
            self.assertIn("Terminal(ExternalOutcome(ξ))", generic.get("target_configuration", ""), generic)
            for route_name in ("generic_failure", "timeout", "cancellation"):
                route = provider_routes[route_name]
                assert isinstance(route, dict)
                self.assertEqual(route.get("transitions", [])[-1], "terminalization", route)
        else:
            generic = transitions["terminalization"]
            assert isinstance(generic, dict)
            self.assertNotIn("ExternalOutcome", generic.get("source_configuration", ""), generic)
            owned = projection.get("owner_transitions")
            self.assertIsInstance(owned, list)
            assert isinstance(owned, list)
            self.assertEqual(set(owned), {
                "generic_failure_terminalization",
                "timeout_external_terminalization",
                "cancellation_external_terminalization",
            })
            for transition_name in owned:
                transition = transitions.get(transition_name)
                self.assertIsInstance(transition, dict, transition_name)
                assert isinstance(transition, dict)
                self.assertIn("Terminal(ExternalOutcome", transition.get("target_configuration", ""), transition)

    def test_effect_validator_rejects_duplicate_or_lossy_external_terminal_projection(self) -> None:
        """Two canonical owners or Terminal(ξ) would create conflicting/lossy external observations."""
        def create_duplicate_external_owner(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            transitions = correspondence["formal_relations"]["transitions"]
            assert isinstance(transitions, dict)
            terminalization = transitions["terminalization"]
            assert isinstance(terminalization, dict)
            terminalization["source_configuration"] = "⟨TerminalReady(ExternalOutcome(ξ)), η, κ, α, F, δ, ρ, ξ⟩"
            terminalization["target_configuration"] = "⟨Terminal(ExternalOutcome(ξ)), η, κ, α, F, δ, ρ, ξ⟩"
            terminalization["endpoint_vocabulary"] = ["TerminalReady", "Return", "Trap", "ExternalOutcome", "Terminal"]
            transitions["provider_duplicate_external_projection"] = {
                "relation_kind": "configuration-to-configuration",
                "source_configuration": "⟨TerminalReady(ExternalOutcome(ξ)), η, κ, α, Fpre · P · Fpost, δ, ρ, ξ⟩",
                "target_configuration": "⟨Terminal(ExternalOutcome(ξ)), η, κ, α, Fpre · P · Fpost, δ, ρ, ξ⟩",
                "endpoint_vocabulary": ["TerminalReady", "ExternalOutcome", "Terminal"],
                "state_phase": {"source": "provider-failure-ready", "target": "terminal-envelope"},
                "outcome_kind": "generic_failure",
                "rule_ids": ["SEM-EFFECT-PROVIDER-001", "SEM-EFFECT-TERMINAL-001"],
            }

        def erase_external_outcome_wrapper(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            projection = correspondence.get("external_projection_contract")
            owner_transition = projection.get("owner_transition") if isinstance(projection, dict) else "terminalization"
            assert isinstance(owner_transition, str)
            transitions = correspondence["formal_relations"]["transitions"]
            assert isinstance(transitions, dict)
            owner = transitions[owner_transition]
            assert isinstance(owner, dict)
            owner["target_configuration"] = "⟨Terminal(ξ), η, κ, α, F, δ, ρ, ξ⟩"

        for mutate, expected_kind in (
            (create_duplicate_external_owner, "duplicate_external_terminal_projection"),
            (erase_external_outcome_wrapper, "noncanonical_external_terminal_projection"),
        ):
            with self.subTest(mutate=mutate.__name__):
                result, report = self.run_canonical_mutation(mutate)
                self.assertNotEqual(result.returncode, 0, report)
                errors = report.get("errors", [])
                self.assertTrue(
                    any(isinstance(error, dict) and error.get("kind") == expected_kind for error in errors),
                    errors,
                )

    def test_task_2031_lookup_candidate_is_a_scoped_correspondence_bridge_not_the_limited_lookup_proof(self) -> None:
        """The remaining Task-2031 lookup obligation is explicit about the proof boundary it does not claim."""
        artifact = json.loads(CANONICAL_ARTIFACT.read_text(encoding="utf-8"))
        correspondence = artifact["effect_correspondence"]
        assert isinstance(correspondence, dict)
        candidates = correspondence["verus_candidates"]
        assert isinstance(candidates, list)
        lookup_candidates = [
            candidate
            for candidate in candidates
            if isinstance(candidate, dict) and candidate.get("rule_id") == "SEM-EFFECT-LOOKUP-001"
        ]
        self.assertEqual(len(lookup_candidates), 1, candidates)
        candidate = lookup_candidates[0]
        self.assertEqual(candidate.get("scope"), "TASK-2031 λAsh-Effect correspondence")
        self.assertEqual(candidate.get("candidate_kind"), "correspondence-bridge")
        self.assertEqual(candidate.get("status"), "deferred")
        self.assertEqual(candidate.get("disposition"), "deferred-unproved")
        self.assertEqual(candidate.get("distinct_from_proof"), "PROOF-CPS-FRAME-LOOKUP-001")

    def test_effect_normal_return_is_a_handle_done_witness_not_a_raw_kernel_return(self) -> None:
        """Normal effect completion enters the selected Handle's done clause exactly once."""
        artifact = json.loads(CANONICAL_ARTIFACT.read_text(encoding="utf-8"))
        examples = artifact["examples"]
        assert isinstance(examples, list)
        normal_return = next(example for example in examples if example.get("id") == "EX-CPS-EFFECT-NORMAL-RETURN-001")
        assert isinstance(normal_return, dict)
        term = normal_return.get("term")
        self.assertIsInstance(term, dict)
        assert isinstance(term, dict)
        self.assertEqual(term.get("form"), "Handle", normal_return)
        self.assertTrue(term.get("done_clause"), normal_return)
        self.assertTrue(term.get("done_clause_once"), normal_return)
        self.assertIn("SEM-EFFECT-HANDLE-001", normal_return.get("rule_ids", []), normal_return)

    def test_effect_contract_validator_rejects_missing_relations_stale_authority_and_raw_done_witness(self) -> None:
        """The validator rejects omissions and stale source phrases instead of accepting prose-shaped rows."""
        def remove_formal_relations(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            correspondence["formal_relations"] = {}

        def restore_stale_section_four_phrase(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            mapping = correspondence["mapping"]
            assert isinstance(mapping, dict)
            row = mapping["SEM-EFFECT-LOOKUP-001"]
            assert isinstance(row, dict)
            row["target_operational"] = "SPEC-099b §4 stale lookup mapping"

        def replace_done_witness_with_raw_return(data: dict[str, object]) -> None:
            examples = data["examples"]
            assert isinstance(examples, list)
            normal_return = next(example for example in examples if example.get("id") == "EX-CPS-EFFECT-NORMAL-RETURN-001")
            assert isinstance(normal_return, dict)
            normal_return["term"] = {"form": "Return", "value": "handler-normal-result"}

        rejected: list[tuple[str, int, list[object]]] = []
        for mutate, expected_kind in (
            (remove_formal_relations, "incomplete_effect_formal_relations"),
            (restore_stale_section_four_phrase, "incorrect_effect_mapping_authority"),
            (replace_done_witness_with_raw_return, "invalid_effect_normal_return_witness"),
        ):
            result, report = self.run_canonical_mutation(mutate)
            errors = report.get("errors", [])
            is_rejected = result.returncode != 0 and any(
                isinstance(error, dict) and error.get("kind") == expected_kind for error in errors
            )
            if not is_rejected:
                rejected.append((expected_kind, result.returncode, errors if isinstance(errors, list) else [errors]))
        self.assertEqual(rejected, [])

    def test_effect_contract_shape_errors_always_return_a_json_validation_report(self) -> None:
        """Malformed JSON shapes reject through the report contract rather than a Python traceback."""
        def nested_configuration_components(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            configuration = correspondence["configuration"]
            assert isinstance(configuration, dict)
            configuration["components"] = [[]]

        def object_syntax_forms(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            syntax = correspondence["syntax"]
            assert isinstance(syntax, dict)
            syntax["forms"] = [{}]

        def list_obligation_case(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            obligations = correspondence["conformance_obligations"]
            assert isinstance(obligations, list)
            first_obligation = obligations[0]
            assert isinstance(first_obligation, dict)
            first_obligation["case"] = []

        for mutate in (nested_configuration_components, object_syntax_forms, list_obligation_case):
            with self.subTest(mutate=mutate.__name__):
                result = self.run_raw_canonical_mutation(mutate)
                self.assertNotEqual(result.returncode, 0, result.stderr)
                self.assertNotIn("Traceback", result.stderr)
                report = json.loads(result.stdout)
                self.assertEqual(report.get("schema"), "ash-cps-calculus-validation-report/v1")
                errors = report.get("errors")
                self.assertIsInstance(errors, list)
                self.assertTrue(errors, report)

    def test_cli_rejects_a_list_valued_rule_stage_with_a_json_error_report(self) -> None:
        """A malformed rule-stage shape must fail closed at the public CLI boundary."""
        def list_rule_stage(data: dict[str, object]) -> None:
            rules = data["rules"]
            assert isinstance(rules, list)
            first_rule = rules[0]
            assert isinstance(first_rule, dict)
            first_rule["stage"] = []

        result = self.run_raw_canonical_mutation(list_rule_stage)
        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertNotIn("Traceback", result.stderr)
        report = json.loads(result.stdout)
        self.assertEqual(report.get("schema"), "ash-cps-calculus-validation-report/v1")
        errors = report.get("errors")
        self.assertIsInstance(errors, list)
        self.assertTrue(errors, report)

    def test_task_2031_traceability_uses_resolving_anchors_and_scopes_the_lookup_bridge_candidate(self) -> None:
        """TASK-2031 must distinguish its correspondence bridge from the narrower lookup proof."""
        traceability = json.loads(SEMANTIC_TRACEABILITY.read_text(encoding="utf-8"))
        nodes = traceability.get("nodes", [])
        edges = traceability.get("edges", [])
        self.assertIsInstance(nodes, list)
        self.assertIsInstance(edges, list)
        anchors = [
            item.get("anchor")
            for item in [*nodes, *edges]
            if isinstance(item, dict) and isinstance(item.get("anchor"), str)
        ]
        self.assertFalse(
            "docs/spec/ASH-CPS-CALCULUS.md#gated-effect-extension" in anchors,
            "traceability still uses the nonexistent #gated-effect-extension anchor",
        )

        calculus_text = (REPOSITORY_ROOT / "docs/spec/ASH-CPS-CALCULUS.md").read_text(encoding="utf-8")
        for heading in (
            "## Judgments and kernel rules",
            "## Effect extension",
            "## Admitted fragment and exclusions",
        ):
            self.assertIn(heading, calculus_text)

        sections = markdown_sections(calculus_text)
        effect_fragment = next(
            (
                fragment
                for fragment, section in sections.items()
                if section.splitlines()[0] == "## Effect extension"
            ),
            None,
        )
        self.assertIsNotNone(effect_fragment)
        assert effect_fragment is not None
        canonical_anchor = f"docs/spec/ASH-CPS-CALCULUS.md#{effect_fragment}"
        self.assertIn(canonical_anchor, anchors)
        for anchor in anchors:
            assert isinstance(anchor, str)
            path, fragment = anchor.split("#", maxsplit=1)
            if path == "docs/spec/ASH-CPS-CALCULUS.md":
                self.assertIn(fragment, sections, anchor)

        artifact = json.loads(CANONICAL_ARTIFACT.read_text(encoding="utf-8"))
        correspondence = artifact["effect_correspondence"]
        assert isinstance(correspondence, dict)
        candidates = correspondence["verus_candidates"]
        assert isinstance(candidates, list)
        for candidate in candidates:
            self.assertIsInstance(candidate, dict, candidates)
            assert isinstance(candidate, dict)
            self.assertEqual(candidate.get("scope"), "TASK-2031 λAsh-Effect correspondence")
            self.assertEqual(candidate.get("disposition"), "deferred-unproved")
        lookup_candidates = [
            candidate
            for candidate in candidates
            if candidate.get("rule_id") == "SEM-EFFECT-LOOKUP-001"
        ]
        self.assertEqual(len(lookup_candidates), 1, candidates)
        lookup_candidate = lookup_candidates[0]
        self.assertEqual(lookup_candidate.get("candidate_kind"), "correspondence-bridge")
        self.assertEqual(lookup_candidate.get("distinct_from_proof"), "PROOF-CPS-FRAME-LOOKUP-001")

    def test_effect_correspondence_rejects_an_incomplete_rule_mapping(self) -> None:
        """Each new effect rule needs all four correspondence columns."""
        def mutate(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            mapping = correspondence["mapping"]
            assert isinstance(mapping, dict)
            row = mapping["SEM-EFFECT-LOOKUP-001"]
            assert isinstance(row, dict)
            row.pop("terminal_projection")

        result, report = self.run_canonical_mutation(mutate)
        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertTrue(
            any(error.get("kind") == "incomplete_effect_correspondence_mapping" for error in report.get("errors", [])),
            report,
        )

    def test_effect_correspondence_rejects_a_mismapped_rule_index(self) -> None:
        """Stable correspondence names may not silently point at a different rule."""
        def mutate(data: dict[str, object]) -> None:
            correspondence = data["effect_correspondence"]
            assert isinstance(correspondence, dict)
            rule_index = correspondence["rule_index"]
            assert isinstance(rule_index, dict)
            rule_index["timeout"] = "SEM-EFFECT-CANCEL-001"

        result, report = self.run_canonical_mutation(mutate)
        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertTrue(
            any(error.get("kind") == "mis_mapped_effect_correspondence_rule" for error in report.get("errors", [])),
            report,
        )


if __name__ == "__main__":
    unittest.main()
