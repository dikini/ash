#!/usr/bin/env python3
"""Contract tests for TASK-1985's canonical-corpus manifest validator."""

from __future__ import annotations

import json
import hashlib
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
TOOL = REPOSITORY_ROOT / "tools/docs/validate_canonical_corpus.py"
FIXTURES = Path(__file__).with_name("fixtures") / "canonical_corpus"


# PLAN-202 §4.3's compact semantic core.  TASK-1986 may add more narrowly
# scoped rule subjects, but it cannot call the promotion complete without an
# active A1/A2 owner for every one of these programme subjects.
PROMOTION_SUBJECTS = (
    "vocabulary.language-overview",
    "grammar.target",
    "types-effects.target",
    "core-cps.syntax",
    "lowering.surface-to-core",
    "semantics.operational",
    "runtime.observable",
    "conformance.implementation",
)


class CanonicalCorpusValidatorContractTests(unittest.TestCase):
    """Exercise the fail-closed public CLI for the sidecar schema."""

    def run_validator(self, fixture_name: str) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        fixture_root = FIXTURES / fixture_name
        return self.run_validator_at_root(fixture_root)

    def run_validator_at_root(self, fixture_root: Path) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        result = subprocess.run(
            [
                "python3",
                str(TOOL),
                "--root",
                str(fixture_root),
                "--manifest",
                str(fixture_root / "docs/spec/CANONICAL-CORPUS.json"),
                "--format",
                "json",
            ],
            check=False,
            capture_output=True,
            text=True,
        )
        try:
            payload = json.loads(result.stdout)
        except json.JSONDecodeError as error:
            self.fail(f"validator must write a JSON report to stdout: {error}; stderr: {result.stderr}")
        return result, payload

    def run_mutated_valid_fixture(self, mutate: object, extra_args: list[str] | None = None) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        """Run the CLI against a disposable copy of the valid contract fixture."""
        with tempfile.TemporaryDirectory() as temporary_directory:
            fixture_root = Path(temporary_directory) / "fixture"
            shutil.copytree(FIXTURES / "valid", fixture_root)
            manifest_path = fixture_root / "docs/spec/CANONICAL-CORPUS.json"
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            assert callable(mutate)
            mutate(fixture_root, manifest)
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            command = ["python3", str(TOOL), "--root", str(fixture_root), "--manifest", str(manifest_path), "--format", "json"]
            if extra_args:
                command.extend(extra_args)
            result = subprocess.run(command, check=False, capture_output=True, text=True)
            try:
                report = json.loads(result.stdout)
            except json.JSONDecodeError as error:
                self.fail(f"validator must write a JSON report to stdout: {error}; stderr: {result.stderr}")
        return result, report

    def assert_mutation_is_rejected(self, mutate: object, error_kind: str) -> None:
        result, report = self.run_mutated_valid_fixture(mutate)
        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertTrue(any(error.get("kind") == error_kind for error in report["errors"]), f"expected {error_kind!r} error, got {report['errors']!r}")

    def assert_invalid_manifest_case(self, fixture_name: str, error_kind: str) -> None:
        self.assertTrue(TOOL.exists(), f"missing canonical-corpus validator under test: {TOOL}")
        result, report = self.run_validator(fixture_name)

        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["schema"], "canonical-corpus-validation-report/v1")
        errors = report["errors"]
        self.assertIsInstance(errors, list)
        self.assertTrue(
            any(error.get("kind") == error_kind for error in errors),
            f"expected {error_kind!r} error, got {errors!r}",
        )

    def run_promotion_fixture(
        self, mutate: object | None = None
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        """Build a complete TASK-1986 fixture without borrowing repository docs.

        The fixture deliberately models the two workflow-first documents as
        historical A5 records.  Reconciliation may instead retain an A2
        handoff, but either route must use typed supersession and neither
        former document may own a target semantic subject.
        """
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory) / "fixture"
            (root / "docs/spec").mkdir(parents=True)
            (root / "docs/reference").mkdir(parents=True)

            def write(relative: str, heading: str) -> None:
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(f"# {heading}\n\nFixture content.\n", encoding="utf-8")

            nodes: list[dict[str, object]] = []
            traces: list[dict[str, object]] = []
            canonical_ids: list[str] = []
            for index, subject in enumerate(PROMOTION_SUBJECTS, start=1):
                node_id = f"spec.fixture.promoted.{index}"
                trace_id = f"SEM-PROMOTION-{index:02d}"
                relative = f"docs/spec/promoted-{index}.md"
                heading = f"Promoted subject {index}"
                write(relative, heading)
                canonical_ids.append(node_id)
                nodes.append({
                    "id": node_id,
                    "path": relative,
                    "kind": "semantic-rule-set",
                    "authority_level": "A1",
                    "lifecycle": "active",
                    "owner": "language-semantics",
                    "audience": ["human", "agent"],
                    "stability": "alpha",
                    "verified_against": {"git_commit": "fixture-revision", "specs": [], "tasks": [], "code": [], "tests": [], "examples": []},
                    "related": {"explains": [], "superseded_by": None},
                    "refresh_trigger": ["fixture change"],
                    "last_verified": "2026-07-24",
                    "canonical_for": [subject],
                    "supersedes": [],
                    "depends_on": [],
                    "trace_nodes": [trace_id],
                })
                traces.append({"id": trace_id, "kind": "semantic", "document": node_id, "anchor": f"#promoted-subject-{index}"})

            for index, (node_id, relative, heading) in enumerate((
                ("history.fixture.formalization-boundary", "docs/reference/formalization-boundary.md", "Former formalization boundary"),
                ("history.fixture.parser-to-core", "docs/reference/parser-to-core-lowering-contract.md", "Former parser to Core contract"),
            ), start=1):
                trace_id = f"REQ-HISTORICAL-{index:02d}"
                write(relative, heading)
                nodes.append({
                    "id": node_id,
                    "path": relative,
                    "kind": "archive",
                    "authority_level": "A5",
                    "lifecycle": "superseded",
                    "canonical_for": [],
                    "supersedes": [],
                    "depends_on": [],
                    "trace_nodes": [trace_id],
                })
                traces.append({"id": trace_id, "kind": "historical-rationale", "document": node_id, "anchor": f"#{heading.lower().replace(' ', '-')}"})

            write("docs/reference/surface-to-core-handoff.md", "Surface to Core handoff")
            nodes.append({
                "id": "handoff.fixture.surface-to-core",
                "path": "docs/reference/surface-to-core-handoff.md",
                "kind": "handoff-contract",
                "authority_level": "A2",
                "lifecycle": "active",
                "owner": "language-semantics",
                "audience": ["human", "agent"],
                "stability": "alpha",
                "verified_against": {"git_commit": "fixture-revision", "specs": [], "tasks": [], "code": [], "tests": [], "examples": []},
                "related": {"explains": [], "superseded_by": None},
                "refresh_trigger": ["lowering change"],
                "last_verified": "2026-07-24",
                "canonical_for": ["handoff.surface-to-core"],
                "supersedes": [],
                "depends_on": [canonical_ids[4]],
                "trace_nodes": ["LOWER-PROMOTION-HANDOFF"],
            })
            traces.append({"id": "LOWER-PROMOTION-HANDOFF", "kind": "lowering", "document": "handoff.fixture.surface-to-core", "anchor": "#surface-to-core-handoff"})

            write("docs/spec/conformance.md", "Implementation conformance")
            nodes.append({
                "id": "conformance.fixture.implementation",
                "path": "docs/spec/conformance.md",
                "kind": "conformance-case",
                "authority_level": "A3",
                "lifecycle": "active",
                "canonical_for": [],
                "supersedes": [],
                "depends_on": [canonical_ids[7]],
                "trace_nodes": ["CONF-PROMOTION-001"],
            })
            traces.append({"id": "CONF-PROMOTION-001", "kind": "conformance", "document": "conformance.fixture.implementation", "anchor": "#implementation-conformance"})

            manifest: dict[str, object] = {
                "schema": "canonical-corpus/v1",
                "nodes": nodes,
                "trace_nodes": traces,
                "typed_edges": [
                    {"kind": "supersedes", "from": canonical_ids[4], "to": "history.fixture.parser-to-core", "anchor": "#promoted-subject-5"},
                    {"kind": "supersedes", "from": canonical_ids[5], "to": "history.fixture.formalization-boundary", "anchor": "#promoted-subject-6"},
                    {"kind": "tested_by", "from": canonical_ids[7], "to": "conformance.fixture.implementation", "anchor": "#promoted-subject-8"},
                ],
                "default_read_paths": {"human": canonical_ids, "agent": canonical_ids},
            }
            if mutate is not None:
                assert callable(mutate)
                mutate(root, manifest)
            manifest_path = root / "docs/spec/CANONICAL-CORPUS.json"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            result = subprocess.run(
                ["python3", str(TOOL), "--root", str(root), "--manifest", str(manifest_path), "--format", "json", "--require-promotion-completeness"],
                check=False,
                capture_output=True,
                text=True,
            )
            try:
                report = json.loads(result.stdout)
            except json.JSONDecodeError as error:
                self.fail(f"promotion validator must write a JSON report to stdout: {error}; stderr: {result.stderr}")
        return result, report

    def assert_promotion_mutation_is_rejected(self, mutate: object, error_kind: str) -> None:
        result, report = self.run_promotion_fixture(mutate)
        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertTrue(any(error.get("kind") == error_kind for error in report["errors"]), report["errors"])

    def test_valid_manifest_keeps_reference_frontmatter_separate(self) -> None:
        """A valid sidecar augments, rather than repurposes, SPEC-071 metadata."""
        self.assertTrue(TOOL.exists(), f"missing canonical-corpus validator under test: {TOOL}")
        result, report = self.run_validator("valid")

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report, {"schema": "canonical-corpus-validation-report/v1", "errors": []})

    def test_duplicate_canonical_for_owner_across_a1_and_a2_is_rejected(self) -> None:
        """A semantic subject has one owner even when one claim is a handoff contract."""
        self.assert_invalid_manifest_case("duplicate_owner", "duplicate_canonical_owner")

    def test_supersession_cycle_is_rejected(self) -> None:
        """Supersession links form a directed acyclic graph."""
        self.assert_invalid_manifest_case("supersession_cycle", "supersession_cycle")

    def test_typed_edge_source_path_must_exist_under_root(self) -> None:
        """An authority edge cannot refer to a missing or escaping source path."""
        self.assert_invalid_manifest_case("broken_source_path", "invalid_edge_path")

    def test_generated_artifact_hash_must_match_declared_source(self) -> None:
        """Derived packs are stale when their recorded source hash differs from disk."""
        self.assert_invalid_manifest_case("generated_stale", "stale_generated_artifact")

    def test_reference_derivative_cannot_claim_a1_or_a2_authority(self) -> None:
        """SPEC-071 derivatives remain A4 even when included in a canonical sidecar."""
        self.assert_invalid_manifest_case("derivative_authority_leakage", "derivative_authority_leakage")

    def test_controlled_enum_values_are_rejected_when_unknown(self) -> None:
        """The sidecar accepts only declared authority and lifecycle vocabulary."""
        self.assert_invalid_manifest_case("invalid_enum", "invalid_enum")

    def test_normalized_reference_path_cannot_bypass_derivative_authority_rule(self) -> None:
        """`./reference/` normalizes to the derivative root before authority is checked."""
        def mutate(_root: Path, manifest: dict[str, object]) -> None:
            nodes = manifest["nodes"]
            assert isinstance(nodes, list)
            reference = nodes[1]
            assert isinstance(reference, dict)
            reference["path"] = "./reference/grammar-summary.md"
            reference["kind"] = "semantic-rule-set"
            reference["authority_level"] = "A1"
        self.assert_mutation_is_rejected(mutate, "derivative_authority_leakage")

    def test_agent_pack_requires_nonempty_generated_provenance(self) -> None:
        """An agent pack cannot be an A4 authority sink with missing provenance."""
        def mutate(_root: Path, manifest: dict[str, object]) -> None:
            nodes = manifest["nodes"]
            assert isinstance(nodes, list)
            agent_pack = nodes[2]
            assert isinstance(agent_pack, dict)
            agent_pack.pop("generated_from")
        self.assert_mutation_is_rejected(mutate, "missing_generated_provenance")

    def test_agent_pack_rejects_empty_generated_provenance(self) -> None:
        """A present provenance block still needs at least one canonical source."""
        def mutate(_root: Path, manifest: dict[str, object]) -> None:
            nodes = manifest["nodes"]
            assert isinstance(nodes, list)
            agent_pack = nodes[2]
            assert isinstance(agent_pack, dict)
            agent_pack["generated_from"] = {"sources": [], "source_hashes": {}}
        self.assert_mutation_is_rejected(mutate, "missing_generated_provenance")

    def test_agent_pack_rejects_a5_source(self) -> None:
        """Research, plans, audit, and archive nodes never feed current agent guidance."""
        def mutate(root: Path, manifest: dict[str, object]) -> None:
            source_path = root / "docs/spec/research.md"
            source_path.write_text("# Research only\n", encoding="utf-8")
            nodes = manifest["nodes"]
            assert isinstance(nodes, list)
            nodes.append({"id": "note.fixture.research", "path": "docs/spec/research.md", "kind": "audit", "authority_level": "A5", "lifecycle": "active", "owner": "research", "canonical_for": [], "supersedes": [], "depends_on": [], "trace_nodes": []})
            agent_pack = nodes[2]
            assert isinstance(agent_pack, dict)
            agent_pack["generated_from"] = {"sources": ["note.fixture.research"], "source_hashes": {"note.fixture.research": hashlib.sha256(source_path.read_bytes()).hexdigest()}}
        self.assert_mutation_is_rejected(mutate, "forbidden_generated_source")

    def test_canonical_nodes_require_plan_202_metadata(self) -> None:
        """A1/A2 nodes provide all metadata PLAN-202 says they must provide or inherit."""
        required = ("owner", "audience", "stability", "verified_against", "related", "refresh_trigger", "last_verified")
        for field in required:
            with self.subTest(field=field):
                def mutate(_root: Path, manifest: dict[str, object], field: str = field) -> None:
                    nodes = manifest["nodes"]
                    assert isinstance(nodes, list)
                    canonical = nodes[0]
                    assert isinstance(canonical, dict)
                    canonical.pop(field, None)
                self.assert_mutation_is_rejected(mutate, "missing_required_metadata")

    def test_superseded_former_owner_does_not_conflict_with_active_replacement(self) -> None:
        """Supersession permits one historical owner while preserving one active owner."""
        def mutate(_root: Path, manifest: dict[str, object]) -> None:
            nodes = manifest["nodes"]
            assert isinstance(nodes, list)
            current = nodes[0]
            assert isinstance(current, dict)
            current["lifecycle"] = "superseded"
            nodes.append({**current, "id": "spec.fixture.grammar.v2", "lifecycle": "active", "supersedes": ["spec.fixture.grammar"]})
        result, report = self.run_mutated_valid_fixture(mutate)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["errors"], [])

    def test_canonical_subject_requires_an_active_a1_or_a2_owner(self) -> None:
        """A superseded record cannot be the sole owner of a still-indexed semantic subject."""
        def mutate(_root: Path, manifest: dict[str, object]) -> None:
            nodes = manifest["nodes"]
            assert isinstance(nodes, list)
            canonical = nodes[0]
            assert isinstance(canonical, dict)
            canonical["lifecycle"] = "superseded"
        self.assert_mutation_is_rejected(mutate, "missing_active_canonical_owner")

    def test_trace_records_have_independent_stable_trace_ids(self) -> None:
        """Document records refer to stable trace records rather than borrowing document IDs."""
        def mutate(_root: Path, manifest: dict[str, object]) -> None:
            nodes = manifest["nodes"]
            assert isinstance(nodes, list)
            canonical = nodes[0]
            assert isinstance(canonical, dict)
            canonical["trace_nodes"] = ["GRAM-FIXTURE-SURFACE"]
            manifest["trace_nodes"] = [{"id": "GRAM-FIXTURE-SURFACE", "kind": "grammar", "anchor": "#target-grammar", "document": "spec.fixture.grammar"}]
        result, report = self.run_mutated_valid_fixture(mutate)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["errors"], [])

    def test_typed_edge_uses_plan_202_relation_node_ids_and_resolvable_anchor(self) -> None:
        """Traceability edges use the PLAN-202 vocabulary, node IDs, and Markdown anchors."""
        def mutate(_root: Path, manifest: dict[str, object]) -> None:
            manifest["typed_edges"] = [{"kind": "defines", "from": "spec.fixture.grammar", "to": "ref.fixture.grammar-summary", "anchor": "#target-grammar"}]
        result, report = self.run_mutated_valid_fixture(mutate)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["errors"], [])

    def test_invalid_reference_frontmatter_is_reported_when_compatibility_check_is_requested(self) -> None:
        """The canonical validator invokes SPEC-071 validation without changing its schema."""
        def mutate(root: Path, _manifest: dict[str, object]) -> None:
            (root / "reference/grammar-summary.md").write_text("# Missing SPEC-071 frontmatter\n", encoding="utf-8")
        result, report = self.run_mutated_valid_fixture(mutate, ["--check-reference-frontmatter"])
        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertTrue(any(error.get("kind") == "reference_frontmatter_invalid" for error in report["errors"]), report["errors"])

    def test_promotion_complete_core_has_one_active_a1_or_a2_owner_for_each_plan_202_subject(self) -> None:
        """TASK-1986 closes only when the eight §4.3 subjects have one owner each."""
        result, report = self.run_promotion_fixture()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["errors"], [])

    def test_promotion_rejects_missing_required_subject_owner(self) -> None:
        """A manifest cannot declare promotion complete with an unresolved core owner."""
        def mutate(_root: Path, manifest: dict[str, object]) -> None:
            nodes = manifest["nodes"]
            assert isinstance(nodes, list)
            grammar = nodes[1]
            assert isinstance(grammar, dict)
            grammar["canonical_for"] = []
        self.assert_promotion_mutation_is_rejected(mutate, "missing_required_canonical_owner")

    def test_promotion_rejects_workflow_first_documents_as_canonical_target_owners(self) -> None:
        """Former workflow-first boundary documents are historical or typed A2 handoffs, never target owners."""
        def mutate(_root: Path, manifest: dict[str, object]) -> None:
            nodes = manifest["nodes"]
            assert isinstance(nodes, list)
            former_boundary = next(node for node in nodes if isinstance(node, dict) and node["id"] == "history.fixture.formalization-boundary")
            former_boundary.update({"kind": "semantic-rule-set", "authority_level": "A1", "lifecycle": "active", "canonical_for": ["semantics.operational"]})
        self.assert_promotion_mutation_is_rejected(mutate, "former_authority_not_reconciled")

    def test_promotion_default_human_and_agent_paths_exclude_a5_history_and_research(self) -> None:
        """Default reading guidance is productive and cannot route through historical claims."""
        def mutate(_root: Path, manifest: dict[str, object]) -> None:
            paths = manifest["default_read_paths"]
            assert isinstance(paths, dict)
            for audience in ("human", "agent"):
                path = paths[audience]
                assert isinstance(path, list)
                path.append("history.fixture.formalization-boundary")
        self.assert_promotion_mutation_is_rejected(mutate, "forbidden_default_read_path")

    def test_promotion_requires_stable_trace_ids_on_handoff_and_conformance_artifacts(self) -> None:
        """Boundary and conformance records link through trace IDs, not volatile document labels."""
        def mutate(_root: Path, manifest: dict[str, object]) -> None:
            nodes = manifest["nodes"]
            assert isinstance(nodes, list)
            for node in nodes:
                if isinstance(node, dict) and node["id"] in {"handoff.fixture.surface-to-core", "conformance.fixture.implementation"}:
                    node["trace_nodes"] = []
        self.assert_promotion_mutation_is_rejected(mutate, "missing_promotion_traceability")

    def test_incomplete_promotion_fixture_is_rejected(self) -> None:
        """The promotion gate rejects a manifest missing a required subject owner."""

        def mutate(_: Path, manifest: dict[str, object]) -> None:
            nodes = manifest["nodes"]
            assert isinstance(nodes, list)
            first_node = nodes[0]
            assert isinstance(first_node, dict)
            first_node["canonical_for"] = []

        result, report = self.run_promotion_fixture(mutate)
        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertTrue(any(error.get("kind") == "promotion_incomplete" for error in report["errors"]), report["errors"])

    def run_migration_fixture(
        self,
        mutate: object | None = None,
        prepare_root: object | None = None,
        mutate_after_prepare: object | None = None,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        """Build a compact TASK-1987 archive-and-routing fixture.

        The migration gate deliberately covers only sidecar-indexed historical
        nodes.  It proves the new external artifacts are complete without
        pretending the task has classified every repository file itself.
        """
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory) / "fixture"

            def write(relative: str, contents: str) -> Path:
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(contents, encoding="utf-8")
                return path

            current = write("docs/spec/current.md", "# Current contract\n")
            write(
                "docs/legacy/formalization.md",
                "# Former formalization\n\n"
                "> Historical tombstone: this workflow-first source is retained for context only. "
                "Read [the canonical replacement](../spec/current.md).\n",
            )
            pack = write("reference/agents/context-pack.md", "# Current agent context\n")
            write("docs/README.md", "# Documentation\n")
            write(
                "reference/manifests/phase-202-archive.json",
                json.dumps({
                    "schema": "canonical-corpus-archive/v1",
                    "snapshot": {
                        "git_commit": "fixture-revision",
                        "extraction_profile": "phase-202-migration",
                    },
                    "artifacts": [{
                        "node": "history.fixture.formalization",
                        "source_path": "docs/legacy/formalization.md",
                        "disposition": "archive",
                        "original_revision": "fixture-revision",
                        "reason": "workflow-first boundary replaced by the canonical Core contract",
                        "replacement": "spec.fixture.current",
                        "unique_content": "Historical workflow rationale remains available through Git.",
                        "productive_inbound_links": ["docs/README.md"],
                    }],
                }),
            )
            write(
                "reference/manifests/phase-202-redirects.json",
                json.dumps({
                    "schema": "canonical-corpus-redirects/v1",
                    "routes": [{
                        "from": "docs/legacy/formalization.md",
                        "to": "docs/spec/current.md",
                        "kind": "redirect",
                    }],
                }),
            )
            write(
                "docs/plan/audits/TASK-1987-retrieval-quality.json",
                json.dumps({
                    "schema": "canonical-corpus-retrieval-benchmark/v1",
                    "queries": [{
                        "id": "retrieval.fixture.formalization",
                        "before": ["docs/legacy/formalization.md"],
                        "after": ["docs/spec/current.md"],
                        "expected": "spec.fixture.current",
                    }],
                }),
            )
            current_hash = hashlib.sha256(current.read_bytes()).hexdigest()
            manifest: dict[str, object] = {
                "schema": "canonical-corpus/v1",
                "nodes": [
                    {
                        "id": "spec.fixture.current",
                        "path": "docs/spec/current.md",
                        "kind": "semantic-rule-set",
                        "authority_level": "A1",
                        "lifecycle": "active",
                        "owner": "language-semantics",
                        "audience": ["human", "agent"],
                        "stability": "alpha",
                        "verified_against": {"git_commit": "fixture-revision", "specs": [], "tasks": [], "code": [], "tests": [], "examples": []},
                        "related": {"explains": [], "superseded_by": None},
                        "refresh_trigger": ["fixture change"],
                        "last_verified": "2026-07-24",
                        "canonical_for": ["fixture.current"],
                        "supersedes": ["history.fixture.formalization"],
                        "depends_on": [],
                        "trace_nodes": ["SEM-FIXTURE-CURRENT"],
                    },
                    {
                        "id": "history.fixture.formalization",
                        "path": "docs/legacy/formalization.md",
                        "kind": "archive",
                        "authority_level": "A5",
                        "lifecycle": "superseded",
                        "canonical_for": [],
                        "supersedes": [],
                        "depends_on": [],
                        "trace_nodes": ["HIST-FIXTURE-FORMALIZATION"],
                    },
                    {
                        "id": "ref.agents.fixture.context-pack",
                        "path": "reference/agents/context-pack.md",
                        "kind": "agent-pack",
                        "authority_level": "A4",
                        "lifecycle": "generated",
                        "canonical_for": [],
                        "supersedes": [],
                        "depends_on": ["spec.fixture.current"],
                        "trace_nodes": [],
                        "generated_from": {
                            "sources": ["spec.fixture.current"],
                            "source_hashes": {"spec.fixture.current": current_hash},
                        },
                    },
                ],
                "trace_nodes": [
                    {"id": "SEM-FIXTURE-CURRENT", "kind": "semantic", "document": "spec.fixture.current", "anchor": "#current-contract"},
                    {"id": "HIST-FIXTURE-FORMALIZATION", "kind": "historical", "document": "history.fixture.formalization", "anchor": "#former-formalization"},
                ],
                "typed_edges": [{"kind": "supersedes", "from": "spec.fixture.current", "to": "history.fixture.formalization", "anchor": "#current-contract"}],
                "migration": {
                    "archive_manifest": "reference/manifests/phase-202-archive.json",
                    "redirect_map": "reference/manifests/phase-202-redirects.json",
                    "retrieval_benchmark": "docs/plan/audits/TASK-1987-retrieval-quality.json",
                },
            }
            manifest_path = root / "docs/spec/CANONICAL-CORPUS.json"
            if mutate is not None:
                assert callable(mutate)
                mutate(root, manifest)
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            if prepare_root is not None:
                assert callable(prepare_root)
                prepare_root(root)
            if mutate_after_prepare is not None:
                assert callable(mutate_after_prepare)
                mutate_after_prepare(root, manifest)
            result = subprocess.run(
                [
                    "python3", str(TOOL), "--root", str(root), "--manifest", str(manifest_path),
                    "--format", "json", "--require-migration-completeness",
                ],
                check=False,
                capture_output=True,
                text=True,
            )
            try:
                report = json.loads(result.stdout)
            except json.JSONDecodeError as error:
                self.fail(f"validator must write a JSON report to stdout: {error}; stderr: {result.stderr}")
        return result, report

    def assert_migration_mutation_is_rejected(self, mutate: object, error_kind: str) -> None:
        result, report = self.run_migration_fixture(mutate)
        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertTrue(any(error.get("kind") == error_kind for error in report["errors"]), report["errors"])

    def test_migration_complete_fixture_accepts_git_backed_archive_routes(self) -> None:
        """A complete migration preserves history while routing current use to canonical material."""
        result, report = self.run_migration_fixture()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["errors"], [])

    def test_migration_requires_archive_provenance_for_every_sidecar_history_node(self) -> None:
        """Every sidecar-indexed historical record has disposition, revision, rationale, and replacement."""
        def mutate(root: Path, _manifest: dict[str, object]) -> None:
            archive_path = root / "reference/manifests/phase-202-archive.json"
            archive = json.loads(archive_path.read_text(encoding="utf-8"))
            archive["artifacts"][0].pop("original_revision")
            archive_path.write_text(json.dumps(archive), encoding="utf-8")
        self.assert_migration_mutation_is_rejected(mutate, "archive_provenance_incomplete")

    def test_migration_does_not_archive_active_a5_audit_or_evidence_records(self) -> None:
        """A5 is an authority level, not by itself a request to archive an active audit record."""
        def mutate(root: Path, manifest: dict[str, object]) -> None:
            audit_path = root / "docs/plan/audits/current-evidence.md"
            audit_path.parent.mkdir(parents=True, exist_ok=True)
            audit_path.write_text("# Current evidence\n", encoding="utf-8")
            nodes = manifest["nodes"]
            traces = manifest["trace_nodes"]
            assert isinstance(nodes, list)
            assert isinstance(traces, list)
            nodes.append({
                "id": "audit.fixture.current-evidence",
                "path": "docs/plan/audits/current-evidence.md",
                "kind": "evidence",
                "authority_level": "A5",
                "lifecycle": "active",
                "canonical_for": [],
                "supersedes": [],
                "depends_on": [],
                "trace_nodes": ["EVIDENCE-FIXTURE-CURRENT"],
            })
            traces.append({
                "id": "EVIDENCE-FIXTURE-CURRENT",
                "kind": "evidence",
                "document": "audit.fixture.current-evidence",
                "anchor": "#current-evidence",
            })

        result, report = self.run_migration_fixture(mutate)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["errors"], [])

    def test_migration_requires_archived_source_tombstone_with_replacement_route(self) -> None:
        """A retained historical file directs readers to the same canonical replacement as its archive record."""
        def mutate(root: Path, _manifest: dict[str, object]) -> None:
            (root / "docs/legacy/formalization.md").write_text(
                "# Former formalization\n\nRetained for historical context.\n",
                encoding="utf-8",
            )

        self.assert_migration_mutation_is_rejected(mutate, "historical_routing_incomplete")

    def test_migration_requires_a_route_for_productive_inbound_links(self) -> None:
        """A displaced source with productive inbound links redirects to its active canonical replacement."""
        def mutate(root: Path, _manifest: dict[str, object]) -> None:
            redirects_path = root / "reference/manifests/phase-202-redirects.json"
            redirects = json.loads(redirects_path.read_text(encoding="utf-8"))
            redirects["routes"][0]["to"] = "docs/legacy/formalization.md"
            redirects_path.write_text(json.dumps(redirects), encoding="utf-8")
        self.assert_migration_mutation_is_rejected(mutate, "productive_route_not_canonical")

    def test_migration_rejects_a_hand_maintained_snapshot_tree(self) -> None:
        """Archive preservation names a Git snapshot instead of introducing a copied shadow corpus."""
        def mutate(root: Path, _manifest: dict[str, object]) -> None:
            archive_path = root / "reference/manifests/phase-202-archive.json"
            archive = json.loads(archive_path.read_text(encoding="utf-8"))
            archive["materialized_tree"] = "docs/archive"
            archive_path.write_text(json.dumps(archive), encoding="utf-8")
        self.assert_migration_mutation_is_rejected(mutate, "hand_maintained_snapshot")

    def test_migration_rejects_unknown_git_snapshot_when_repository_is_available(self) -> None:
        """Git-backed fixtures reject an archive revision that cannot be resolved in that repository."""
        if shutil.which("git") is None:
            self.skipTest("Git is unavailable; snapshot resolution cannot be exercised")

        def prepare_root(root: Path) -> None:
            for command in (
                ["git", "init", "--quiet"],
                ["git", "config", "user.email", "fixture@example.invalid"],
                ["git", "config", "user.name", "Fixture"],
                ["git", "add", "."],
                ["git", "commit", "--quiet", "-m", "fixture snapshot"],
            ):
                subprocess.run(command, cwd=root, check=True, capture_output=True, text=True)

        def mutate_after_prepare(root: Path, _manifest: dict[str, object]) -> None:
            archive_path = root / "reference/manifests/phase-202-archive.json"
            archive = json.loads(archive_path.read_text(encoding="utf-8"))
            archive["snapshot"]["git_commit"] = "0" * 40
            archive_path.write_text(json.dumps(archive), encoding="utf-8")

        result, report = self.run_migration_fixture(
            prepare_root=prepare_root,
            mutate_after_prepare=mutate_after_prepare,
        )
        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertTrue(
            any(item.get("kind") == "archive_snapshot_unverifiable" for item in report["errors"]),
            report["errors"],
        )

    def test_migration_requires_before_and_after_retrieval_evidence(self) -> None:
        """Migration completion records a deterministic pre/post retrieval query for each replacement route."""
        def mutate(root: Path, _manifest: dict[str, object]) -> None:
            benchmark_path = root / "docs/plan/audits/TASK-1987-retrieval-quality.json"
            benchmark = json.loads(benchmark_path.read_text(encoding="utf-8"))
            benchmark["queries"][0].pop("after")
            benchmark_path.write_text(json.dumps(benchmark), encoding="utf-8")
        self.assert_migration_mutation_is_rejected(mutate, "retrieval_quality_incomplete")


if __name__ == "__main__":
    unittest.main()
