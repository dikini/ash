#!/usr/bin/env python3
"""Contract tests for the Phase 202 corpus-authority inventory generator."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
TOOL = REPOSITORY_ROOT / "tools/docs/generate_corpus_authority_inventory.py"
FIXTURES = Path(__file__).with_name("fixtures") / "corpus_authority_inventory"


class CorpusAuthorityInventoryContractTests(unittest.TestCase):
    """Exercise the public CLI before its implementation exists."""

    def run_inventory(self, fixture_name: str) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        fixture_root = FIXTURES / fixture_name
        scope = fixture_root / "scope.json"
        return self.run_inventory_at_root(fixture_root, scope)

    def run_inventory_at_root(
        self,
        fixture_root: Path,
        scope: Path,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        with tempfile.TemporaryDirectory() as temporary_directory:
            output = Path(temporary_directory) / "inventory.json"
            result = subprocess.run(
                [
                    "python3",
                    str(TOOL),
                    "--root",
                    str(fixture_root),
                    "--scope",
                    str(scope),
                    "--output",
                    str(output),
                ],
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertTrue(output.exists(), f"generator did not write inventory output: {result.stderr}")
            try:
                payload = json.loads(output.read_text(encoding="utf-8"))
            except json.JSONDecodeError as error:
                self.fail(f"generator wrote malformed inventory JSON: {error}")
        return result, payload

    def assert_invalid_inventory_case(self, fixture_name: str, conflict_kind: str) -> None:
        self.assertTrue(
            TOOL.exists(),
            f"missing inventory generator under test: {TOOL}",
        )
        result, payload = self.run_inventory(fixture_name)

        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertEqual(payload["schema"], "corpus-authority-inventory/v1")
        self.assertIn("artifacts", payload)
        self.assertIn("conflicts", payload)
        conflicts = payload["conflicts"]
        self.assertIsInstance(conflicts, list)
        self.assertTrue(
            any(conflict.get("kind") == conflict_kind for conflict in conflicts),
            f"expected {conflict_kind!r} conflict, got {conflicts!r}",
        )

    def test_duplicate_canonical_subject_ownership_is_a_conflict(self) -> None:
        """Two active A1 owners for one subject must fail the generated audit."""
        self.assert_invalid_inventory_case("duplicate_canonical_owner", "duplicate_canonical_owner")

    def test_current_and_historical_claims_are_a_conflict(self) -> None:
        """Explicit structured status metadata cannot claim current and historical status."""
        self.assert_invalid_inventory_case("contradictory_status_claim", "contradictory_status_claim")

    def test_missing_status_is_an_explicit_conflict(self) -> None:
        """A productive artifact without a status claim cannot be silently classified."""
        self.assert_invalid_inventory_case("missing_status", "missing_status")

    def test_valid_inventory_preserves_frozen_scope_and_audit_evidence(self) -> None:
        """A valid audit must retain the evidence needed by later Phase 202 tasks."""
        self.assertTrue(TOOL.exists(), f"missing inventory generator under test: {TOOL}")
        result, payload = self.run_inventory("complete_valid_inventory")

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(payload["schema"], "corpus-authority-inventory/v1")
        self.assertIn("frozen_scope", payload, "inventory must retain the frozen scope evidence")
        frozen_scope = payload["frozen_scope"]
        self.assertEqual(frozen_scope["schema"], "corpus-authority-scope/v1")
        self.assertEqual(frozen_scope["repository_revision"], "fixture-revision-202")
        self.assertEqual(
            frozen_scope["dirty_worktree"],
            {
                "qualified": True,
                "changed_paths": ["docs/spec/CORE-CPS.md"],
                "qualification": "inventory includes the dirty path as live evidence",
            },
        )
        self.assertEqual(frozen_scope["productive_roots"], ["docs", "reference"])
        self.assertEqual(
            frozen_scope["exclusions"],
            [{"path": "docs/archive", "reason": "historical snapshot outside productive roots"}],
        )

        artifact = next(item for item in payload["artifacts"] if item["id"] == "fixture.core-cps")
        self.assertEqual(artifact["current_target_historical"], "target")
        self.assertEqual(artifact["unique_content"], ["small-step Core/CPS transition relation"])
        self.assertEqual(artifact["proposed_disposition"], "promote")
        self.assertEqual(
            artifact["verified_against"],
            {"code": ["crates/ash-core/src/core.rs"], "tests": ["crates/ash-core/tests/core_step.rs"]},
        )
        self.assertEqual(
            artifact["related"],
            {"explains": ["fixture.runtime-projection"], "superseded_by": None},
        )

        conflict_ids = {conflict["id"] for conflict in payload["known_conflicts"]}
        self.assertEqual(
            conflict_ids,
            {
                "conflict.docs-readme-spec-index",
                "conflict.formalization-boundary",
                "conflict.parser-to-core",
                "conflict.phase-201-handoff",
            },
        )
        self.assertIn(
            {
                "path": "crates/ash-core/src/core.rs",
                "symbol": "ash_core::core::step",
                "tests": ["crates/ash-core/tests/core_step.rs"],
                "executed_test": {
                    "command": "cargo test -p ash-core --test core_step",
                    "result": "passed",
                },
                "classification": "realization_only",
                "canonical_subjects": ["semantics.core-cps.step"],
            },
            payload["semantic_rust"],
        )

    def test_duplicate_stable_id_is_a_conflict(self) -> None:
        """Stable artifact IDs must be unique within one frozen scope."""
        self.assert_invalid_inventory_case("duplicate_stable_id", "duplicate_id")

    def test_missing_expected_canonical_owner_is_a_conflict(self) -> None:
        """Every expected canonical subject needs an owner or explicit unresolved status."""
        self.assert_invalid_inventory_case("missing_canonical_owner", "missing_canonical_owner")

    def test_exclusion_without_a_reason_is_a_conflict(self) -> None:
        """An exclusion cannot be inferred from an unqualified path string."""
        self.assert_invalid_inventory_case("malformed_exclusion", "malformed_exclusion")

    def test_nested_spec_071_evidence_and_related_metadata_are_preserved(self) -> None:
        """Nested SPEC-071-style mappings must remain structured in the audit output."""
        self.assertTrue(TOOL.exists(), f"missing inventory generator under test: {TOOL}")
        result, payload = self.run_inventory("nested_frontmatter_valid")

        self.assertEqual(result.returncode, 0, result.stderr)
        artifact = payload["artifacts"][0]
        self.assertIn("verified_against", artifact)
        self.assertIn("related", artifact)
        self.assertEqual(
            artifact["verified_against"],
            {"code": ["crates/ash-core/src/core.rs"], "tests": ["crates/ash-core/tests/core_step.rs"]},
        )
        self.assertEqual(
            artifact["related"],
            {"explains": ["fixture.runtime-projection"], "superseded_by": None},
        )

    def test_scope_missing_frozen_field_writes_a_conflicted_inventory(self) -> None:
        """A missing revision/qualification/root field cannot be inferred at audit time."""
        self.assert_invalid_inventory_case("missing_frozen_scope_field", "missing_frozen_scope_field")

    def test_nonexistent_included_root_writes_a_conflicted_inventory(self) -> None:
        """Scope roots must exist instead of silently producing an empty inventory."""
        self.assert_invalid_inventory_case("nonexistent_included_root", "invalid_included_root")

    def test_invalid_evidence_path_is_a_conflict(self) -> None:
        """Semantic Rust and document evidence paths must resolve inside the frozen root."""
        self.assert_invalid_inventory_case("invalid_evidence_path", "invalid_evidence_path")

    def test_escaping_markdown_symlink_is_rejected_without_reading_its_target(self) -> None:
        """A symlinked Markdown artifact may not cause the inventory to read outside root."""
        source_root = FIXTURES / "symlink_escape"
        with tempfile.TemporaryDirectory() as temporary_directory:
            fixture_root = Path(temporary_directory) / "fixture"
            shutil.copytree(source_root, fixture_root)
            escape_link = fixture_root / "docs/spec/escape.md"
            try:
                os.symlink("/etc/hosts", escape_link)
            except (NotImplementedError, OSError) as error:
                self.skipTest(f"symlinks unavailable in this environment: {error}")
            result, payload = self.run_inventory_at_root(fixture_root, fixture_root / "scope.json")

        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertTrue(
            any(conflict.get("kind") == "escaping_symlink" for conflict in payload["conflicts"]),
            payload["conflicts"],
        )

    def test_malformed_utf8_document_writes_a_conflicted_inventory(self) -> None:
        """A decode failure must become an audit finding rather than terminate the process."""
        source_root = FIXTURES / "malformed_utf8"
        with tempfile.TemporaryDirectory() as temporary_directory:
            fixture_root = Path(temporary_directory) / "fixture"
            shutil.copytree(source_root, fixture_root)
            (fixture_root / "docs/spec/not-utf8.md").write_bytes(b"---\nstatus: current\n---\n\xff")
            result, payload = self.run_inventory_at_root(fixture_root, fixture_root / "scope.json")

        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertTrue(
            any(conflict.get("kind") == "malformed_utf8" for conflict in payload["conflicts"]),
            payload["conflicts"],
        )

    def test_id_and_status_only_artifact_is_explicitly_unclassified(self) -> None:
        """Sparse metadata cannot be accepted as a productive semantic classification."""
        self.assert_invalid_inventory_case("unclassified_artifact", "unclassified_artifact")

    def test_known_conflict_requires_a_complete_ledger_entry(self) -> None:
        """Known-conflict IDs need enough metadata to serve as actionable audit inputs."""
        self.assert_invalid_inventory_case("malformed_known_conflict", "malformed_known_conflict")

    def test_classification_overlay_supplies_metadata_for_unadorned_markdown(self) -> None:
        """The frozen overlay, not inferred chronology, classifies sparse scoped artifacts."""
        self.assertTrue(TOOL.exists(), f"missing inventory generator under test: {TOOL}")
        result, payload = self.run_inventory("classification_overlay_valid")

        self.assertEqual(result.returncode, 0, result.stderr)
        artifact = next(item for item in payload["artifacts"] if item["path"] == "docs/spec/unadorned.md")
        self.assertEqual(artifact["id"], "fixture.overlay-owned-rule")
        self.assertEqual(artifact["claimed_authority"], "A1")
        self.assertEqual(artifact["canonical_subjects"], ["semantics.core-cps.step"])
        self.assertEqual(artifact["lifecycle_claims"], ["active"])
        self.assertEqual(artifact["status_claims"], ["current"])
        self.assertEqual(artifact["current_target_historical"], "target")
        self.assertEqual(artifact["unique_content"], ["overlay-owned transition rule"])
        self.assertEqual(artifact["proposed_disposition"], "promote")
        self.assertEqual(
            artifact["verified_against"],
            {"code": ["crates/ash-core/src/core.rs"], "tests": ["crates/ash-core/tests/core_step.rs"]},
        )

    def test_scoped_artifact_without_overlay_classification_is_a_conflict(self) -> None:
        """Every productive scoped artifact needs explicit classification or a declared gap."""
        self.assert_invalid_inventory_case("missing_overlay_classification", "missing_overlay_classification")

    def test_declared_data_artifact_is_emitted_and_classified(self) -> None:
        """Canonical JSON/YAML sidecars are part of the corpus, not invisible Markdown adjuncts."""
        self.assertTrue(TOOL.exists(), f"missing inventory generator under test: {TOOL}")
        result, payload = self.run_inventory("data_artifact_valid")

        self.assertEqual(result.returncode, 0, result.stderr)
        matches = [item for item in payload["artifacts"] if item["path"] == "docs/spec/core-rules.json"]
        self.assertEqual(len(matches), 1, payload["artifacts"])
        self.assertEqual(matches[0]["id"], "fixture.core-rule-data")
        self.assertEqual(matches[0]["kind"], "canonical-data")
        self.assertEqual(matches[0]["canonical_subjects"], ["semantics.core-cps.step"])

    def test_structured_known_conflict_is_linked_to_each_affected_artifact(self) -> None:
        """The ledger preserves competing claims and exposes them from affected inventory rows."""
        self.assertTrue(TOOL.exists(), f"missing inventory generator under test: {TOOL}")
        result, payload = self.run_inventory("structured_known_conflict_valid")

        self.assertEqual(result.returncode, 0, result.stderr)
        conflict = next(item for item in payload["known_conflicts"] if item["id"] == "conflict.fixture.grammar")
        self.assertEqual(
            conflict,
            {
                "id": "conflict.fixture.grammar",
                "involved_paths": ["docs/spec/grammar.md", "docs/reference/grammar-contract.md"],
                "competing_claims": ["workflow-first grammar", "target surface grammar"],
                "evidence": ["docs/spec/grammar.md", "docs/reference/grammar-contract.md"],
                "disposition": "unresolved",
                "status": "open",
            },
        )
        affected_paths = set(conflict["involved_paths"])
        affected = [item for item in payload["artifacts"] if item["path"] in affected_paths]
        self.assertEqual(len(affected), 2)
        self.assertTrue(all("conflict.fixture.grammar" in item["known_conflicts"] for item in affected))

    def test_semantic_rust_record_without_symbol_or_executed_test_is_a_conflict(self) -> None:
        """Semantic Rust coverage needs a symbol and executed command/result evidence."""
        self.assert_invalid_inventory_case("missing_semantic_rust_evidence", "missing_semantic_rust_evidence")

    def test_narrative_temporal_words_do_not_override_explicit_status_metadata(self) -> None:
        """Narrative current/historical language is not a machine-readable conflict claim."""
        self.assertTrue(TOOL.exists(), f"missing inventory generator under test: {TOOL}")
        result, payload = self.run_inventory("narrative_status_words_valid")

        self.assertEqual(result.returncode, 0, result.stderr)
        artifact = payload["artifacts"][0]
        self.assertEqual(artifact["status_claims"], ["current"])
        self.assertFalse(any(conflict["kind"] == "contradictory_status_claim" for conflict in payload["conflicts"]))

    def test_known_conflict_links_docs_and_semantic_rust_records(self) -> None:
        """A conflict spanning documentation and realization evidence is navigable from both."""
        self.assertTrue(TOOL.exists(), f"missing inventory generator under test: {TOOL}")
        result, payload = self.run_inventory("structured_doc_rust_conflict_valid")

        self.assertEqual(result.returncode, 0, result.stderr)
        conflict_id = "conflict.fixture.core-realization"
        artifact = next(item for item in payload["artifacts"] if item["path"] == "docs/spec/core.md")
        semantic_rust = next(item for item in payload["semantic_rust"] if item["path"] == "crates/ash-core/src/core.rs")
        self.assertIn(conflict_id, artifact["known_conflicts"])
        self.assertIn("known_conflicts", semantic_rust)
        self.assertIn(conflict_id, semantic_rust["known_conflicts"])


if __name__ == "__main__":
    unittest.main()
