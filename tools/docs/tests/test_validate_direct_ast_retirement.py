#!/usr/bin/env python3
"""RED contracts for TASK-2034's frozen direct-AST retirement manifest.

The audit is a finite, revision-bound catalogue.  This suite defines its
machine-readable boundary without claiming that any catalogued item has been
removed or migrated.
"""
from __future__ import annotations

import hashlib
import json
import re
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
TOOL = REPOSITORY_ROOT / "tools/docs/validate_direct_ast_retirement.py"
FIXTURES = Path(__file__).with_name("fixtures") / "direct_ast_retirement"
REPORT_SCHEMA = "direct-ast-retirement-validation-report/v1"
MANIFEST_SCHEMA = "direct-ast-retirement-audit/v1"

MANIFEST_FIELDS = frozenset({"schema", "repository_revision", "entries_sha256", "entries"})
ENTRY_FIELDS = frozenset({
    "id",
    "path",
    "locator",
    "current_role",
    "reachability",
    "classification",
    "execution_role",
    "target_rule_or_contract",
    "disposition",
    "owner_or_external_handoff",
    "consumed_handoff",
    "produced_handoff",
    "required_evidence",
    "rationale",
    "case_id",
    "missing_obligation",
    "fail_closed_result",
    "external_project",
    "external_owner",
    "external_handoff",
    "retained_paths",
    "prohibited_current_authority",
})


def sorted_entry_digest(entries: object) -> str:
    """Return the canonical digest required by the frozen finite inventory."""
    assert isinstance(entries, list)
    ordered = sorted(entries, key=lambda entry: entry["id"])
    payload = json.dumps(ordered, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
    return f"sha256:{hashlib.sha256(payload.encode("utf-8")).hexdigest()}"


class DirectAstRetirementManifestContractTests(unittest.TestCase):
    """Exercise the fail-closed public CLI for AUDIT-204."""

    def run_validator(self, root: Path, manifest: Path) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        self.assertTrue(TOOL.exists(), f"missing TASK-2034 validator under test: {TOOL}")
        result = subprocess.run(
            [
                "python3", str(TOOL), "--root", str(root), "--manifest", str(manifest),
                "--format", "json",
            ],
            check=False,
            capture_output=True,
            text=True,
        )
        try:
            report = json.loads(result.stdout)
        except json.JSONDecodeError as error:
            self.fail(f"validator must write JSON to stdout: {error}; stderr: {result.stderr}")
        return result, report

    def run_fixture(self, name: str) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        root = FIXTURES / name
        return self.run_validator(root, root / "AUDIT-204-direct-ast-retirement.json")

    def run_mutation(
        self, mutate: object, *, recompute_digest: bool = True
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory) / "fixture"
            shutil.copytree(FIXTURES / "valid", root)
            manifest_path = root / "AUDIT-204-direct-ast-retirement.json"
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            assert callable(mutate)
            mutate(manifest)
            if recompute_digest:
                entries = manifest["entries"]
                manifest["entries_sha256"] = sorted_entry_digest(entries)
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            return self.run_validator(root, manifest_path)

    def run_production_manifest_mutation(
        self, mutate: object, *, recompute_digest: bool = True
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        """Mutate the frozen production inventory without changing its checked-in copy."""
        production_manifest = REPOSITORY_ROOT / "docs/plan/audits/AUDIT-204-direct-ast-retirement.json"
        self.assertTrue(production_manifest.is_file(), "TASK-2034 real audit manifest must exist")
        with tempfile.TemporaryDirectory() as temporary_directory:
            manifest_path = Path(temporary_directory) / production_manifest.name
            manifest = json.loads(production_manifest.read_text(encoding="utf-8"))
            assert callable(mutate)
            mutate(manifest)
            if recompute_digest:
                entries = manifest["entries"]
                manifest["entries_sha256"] = sorted_entry_digest(entries)
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            return self.run_validator(REPOSITORY_ROOT, manifest_path)

    def assert_rejected(self, mutate: object, kind: str, *, recompute_digest: bool = True) -> None:
        result, report = self.run_mutation(mutate, recompute_digest=recompute_digest)
        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report.get("schema"), REPORT_SCHEMA)
        errors = report.get("errors")
        self.assertIsInstance(errors, list)
        self.assertTrue(any(error.get("kind") == kind for error in errors if isinstance(error, dict)), errors)

    def assert_production_lean_handoff_field_rejected(
        self, path: str, field: str, kind: str
    ) -> None:
        """Require every retained Lean record to preserve its named external handoff field."""
        entry_id = ""

        def clear_handoff_field(manifest: dict[str, object]) -> None:
            nonlocal entry_id
            entries = manifest["entries"]
            assert isinstance(entries, list)
            entry = next(entry for entry in entries if entry["path"] == path)
            entry_id = entry["id"]
            entry[field] = [] if field == "retained_paths" else ""

        result, report = self.run_production_manifest_mutation(clear_handoff_field)
        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report.get("schema"), REPORT_SCHEMA)
        errors = report.get("errors")
        self.assertIsInstance(errors, list)
        self.assertTrue(
            any(
                error.get("kind") == kind
                and error.get("entry") == entry_id
                and error.get("field") == field
                for error in errors
                if isinstance(error, dict)
            ),
            errors,
        )

    def test_valid_fixture_covers_every_finite_classification_and_disposition(self) -> None:
        """A complete finite inventory is revision-bound and accepted."""
        fixture = FIXTURES / "valid" / "AUDIT-204-direct-ast-retirement.json"
        manifest = json.loads(fixture.read_text(encoding="utf-8"))
        self.assertEqual(set(manifest), MANIFEST_FIELDS)
        entries = manifest["entries"]
        self.assertIsInstance(entries, list)
        self.assertTrue(entries)
        self.assertTrue(all(set(entry) == ENTRY_FIELDS for entry in entries if isinstance(entry, dict)))
        self.assertEqual(manifest["entries_sha256"], sorted_entry_digest(entries))
        self.assertEqual(
            {entry["classification"] for entry in entries},
            {"current", "historical", "deferred_separate_project"},
        )
        self.assertEqual(
            {entry["disposition"] for entry in entries},
            {"replace", "delete", "deferred", "historical", "deferred_separate_project"},
        )
        result, report = self.run_fixture("valid")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report, {"schema": REPORT_SCHEMA, "errors": []})

    def test_digest_is_deterministic_over_entries_sorted_by_stable_id(self) -> None:
        """Presentation order cannot change the signed finite inventory."""
        def reverse_entries(manifest: dict[str, object]) -> None:
            entries = manifest["entries"]
            assert isinstance(entries, list)
            entries.reverse()

        result, report = self.run_mutation(reverse_entries)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report, {"schema": REPORT_SCHEMA, "errors": []})

    def test_duplicate_ids_and_nonfinite_paths_are_rejected(self) -> None:
        """Every record names one explicit repository item, never a generated class."""
        def duplicate_id(manifest: dict[str, object]) -> None:
            entries = manifest["entries"]
            assert isinstance(entries, list)
            entries[1]["id"] = entries[0]["id"]

        def glob_path(manifest: dict[str, object]) -> None:
            entries = manifest["entries"]
            assert isinstance(entries, list)
            entries[0]["path"] = "crates/ash-interp/src/**/*.rs"

        self.assert_rejected(duplicate_id, "duplicate_entry_id")
        self.assert_rejected(glob_path, "nonfinite_entry_path")

    def test_paths_must_be_existing_root_relative_files(self) -> None:
        """An audit cannot point outside its frozen repository or at a phantom item."""
        def bad_path(manifest: dict[str, object]) -> None:
            entries = manifest["entries"]
            assert isinstance(entries, list)
            entries[0]["path"] = "../outside.rs"

        self.assert_rejected(bad_path, "invalid_entry_path")

    def test_locator_must_exist_in_the_named_file(self) -> None:
        """A catalogue locator is a checked anchor, not unverified narrative."""
        def missing_locator(manifest: dict[str, object]) -> None:
            entries = manifest["entries"]
            assert isinstance(entries, list)
            entries[0]["locator"] = "this locator does not exist"

        self.assert_rejected(missing_locator, "missing_entry_locator")

    def test_schema_digest_and_unknown_fields_fail_closed(self) -> None:
        """Digest metadata and the entry schema are not advisory narrative."""
        def malformed_digest(manifest: dict[str, object]) -> None:
            manifest["entries_sha256"] = "not-a-sha256"

        def unknown_entry_field(manifest: dict[str, object]) -> None:
            entries = manifest["entries"]
            assert isinstance(entries, list)
            entries[0]["untracked"] = "must not be silently accepted"

        def unknown_manifest_field(manifest: dict[str, object]) -> None:
            manifest["untracked"] = True

        self.assert_rejected(malformed_digest, "invalid_entries_digest", recompute_digest=False)
        self.assert_rejected(unknown_entry_field, "unknown_entry_field")
        self.assert_rejected(unknown_manifest_field, "unknown_manifest_field")

    def test_repository_revision_must_be_an_existing_commit_when_root_has_git(self) -> None:
        """A production audit cannot claim a syntactically valid but unknown revision."""
        real_manifest = REPOSITORY_ROOT / "docs/plan/audits/AUDIT-204-direct-ast-retirement.json"
        self.assertTrue(real_manifest.is_file(), "TASK-2034 real audit manifest must exist")
        with tempfile.TemporaryDirectory() as temporary_directory:
            manifest_path = Path(temporary_directory) / "AUDIT-204-direct-ast-retirement.json"
            manifest = json.loads(real_manifest.read_text(encoding="utf-8"))
            manifest["repository_revision"] = "f" * 40
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            result, report = self.run_validator(REPOSITORY_ROOT, manifest_path)
        self.assertNotEqual(result.returncode, 0, result.stderr)
        errors = report.get("errors")
        self.assertIsInstance(errors, list)
        self.assertTrue(
            any(error.get("kind") == "unknown_repository_revision" for error in errors if isinstance(error, dict)),
            errors,
        )

    def test_real_manifest_entries_are_present_at_its_frozen_repository_revision(self) -> None:
        """AUDIT-204 cannot catalogue a present-day path absent from its frozen commit."""
        real_manifest = REPOSITORY_ROOT / "docs/plan/audits/AUDIT-204-direct-ast-retirement.json"
        manifest = json.loads(real_manifest.read_text(encoding="utf-8"))
        revision = manifest["repository_revision"]
        entries = manifest["entries"]
        self.assertIsInstance(revision, str)
        self.assertIsInstance(entries, list)

        result, report = self.run_validator(REPOSITORY_ROOT, real_manifest)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report, {"schema": REPORT_SCHEMA, "errors": []})

        for entry in entries:
            self.assertIsInstance(entry, dict)
            entry_id = entry["id"]
            path = entry["path"]
            self.assertIsInstance(entry_id, str)
            self.assertIsInstance(path, str)
            with self.subTest(entry=entry_id, path=path):
                frozen_path = subprocess.run(
                    ["git", "-C", str(REPOSITORY_ROOT), "cat-file", "-e", f"{revision}:{path}"],
                    check=False,
                    capture_output=True,
                    text=True,
                )
                self.assertEqual(
                    frozen_path.returncode,
                    0,
                    f"AUDIT-204 entry {entry_id} names {path!r}, which is absent from {revision}: "
                    f"{frozen_path.stderr}",
                )

    def test_production_manifest_rejects_current_path_absent_from_frozen_revision(self) -> None:
        """Recomputing the digest cannot authorize a path introduced after the audit commit."""
        mutated_entry_id = ""

        def replace_path_with_post_revision_file(manifest: dict[str, object]) -> None:
            nonlocal mutated_entry_id
            entries = manifest["entries"]
            assert isinstance(entries, list)
            entry = next(entry for entry in entries if entry["path"] != "Cargo.lock")
            mutated_entry_id = entry["id"]
            entry["path"] = "Cargo.lock"
            entry["locator"] = "# This file is automatically @generated by Cargo."

        result, report = self.run_production_manifest_mutation(replace_path_with_post_revision_file)
        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report.get("schema"), REPORT_SCHEMA)
        errors = report.get("errors")
        self.assertIsInstance(errors, list)
        self.assertTrue(
            any(
                error.get("kind") == "entry_not_in_repository_revision"
                and error.get("entry") == mutated_entry_id
                and error.get("path") == "Cargo.lock"
                for error in errors
                if isinstance(error, dict)
            ),
            errors,
        )

    def test_real_manifest_enumerates_finite_retirement_paths(self) -> None:
        """The audited corpus and independently executable clients are not implicit classes."""
        real_manifest = REPOSITORY_ROOT / "docs/plan/audits/AUDIT-204-direct-ast-retirement.json"
        manifest = json.loads(real_manifest.read_text(encoding="utf-8"))
        entries = manifest["entries"]
        self.assertIsInstance(entries, list)
        paths = {entry["path"] for entry in entries if isinstance(entry, dict)}

        corpus_root = REPOSITORY_ROOT / "tests/differential/corpus"
        corpus_paths = {
            path.relative_to(REPOSITORY_ROOT).as_posix()
            for path in corpus_root.rglob("*")
            if path.is_file()
        }
        self.assertEqual(corpus_paths, paths & corpus_paths)

        required_paths = {
            "crates/ash-cli/src/test_runner/executor.rs",
            "crates/ash-cli/src/test_runner/property.rs",
            "crates/ash-cli/src/test_runner/synthesized/law.rs",
            "crates/ash-cli/src/test_runner/synthesized/smallworld.rs",
            "crates/ash-cli/src/test_runner/synthesized/tests.rs",
            "crates/ash-cli/src/commands/daemon.rs",
            "crates/ash-interp/tests/task_1591_cps_ir.rs",
            "crates/ash-interp/tests/task_1592_cps_ir.rs",
            "crates/ash-interp/tests/task_1593_cps_ir.rs",
            "crates/ash-interp/tests/task_1595_cps_ir.rs",
            "crates/ash-interp/tests/task_1596_cps_ir.rs",
            "crates/ash-interp/tests/task_1616_cps_ir_speculative_fixtures.rs",
            "crates/ash-interp/tests/task_1616b_cps_ir_correctness_fixes.rs",
            "crates/ash-interp/tests/task_1663_cps_runtime_scaffold.rs",
            "crates/ash-interp/tests/task_1664_cps_force_runtime.rs",
            "crates/ash-interp/tests/task_1672_cps_thunk_trace_observability.rs",
            "crates/ash-interp/tests/task_1682_cps_multishot_runtime.rs",
            "crates/ash-interp/tests/task_1683_cps_multishot_validation.rs",
            "crates/ash-interp/tests/task_1858_1859_handler_provider_semantics.rs",
            "crates/ash-interp/tests/task_1993_frame_ordered_dispatch.rs",
            "crates/ash-interp/tests/task_1932_host_boundary_cross_boundary_fixtures.rs",
            "docs/plan/tasks/TASK-052-fuzzing.md",
            "docs/plans/2026-07-26-task-2014-run-wide-cooperative-control-implementation-plan.md",
        }
        self.assertTrue(required_paths <= paths, sorted(required_paths - paths))

    def test_real_manifest_enumerates_every_direct_ast_eval_expr_reference(self) -> None:
        """Every Rust direct-AST eval_expr reference is one explicit finite audit record."""
        real_manifest = REPOSITORY_ROOT / "docs/plan/audits/AUDIT-204-direct-ast-retirement.json"
        manifest = json.loads(real_manifest.read_text(encoding="utf-8"))
        entries = manifest["entries"]
        self.assertIsInstance(entries, list)
        paths = {entry["path"] for entry in entries if isinstance(entry, dict)}

        eval_expr_paths = {
            path.relative_to(REPOSITORY_ROOT).as_posix()
            for path in (REPOSITORY_ROOT / "crates").rglob("*.rs")
            if re.search(r"\beval_expr\b", path.read_text(encoding="utf-8"))
        }
        self.assertTrue(eval_expr_paths, "the direct-AST eval_expr audit set must not be implicit")
        self.assertEqual(eval_expr_paths, paths & eval_expr_paths)

    def test_current_executable_entries_require_a_phase_205_owner(self) -> None:
        """A current executable item cannot be audited without its migration owner."""
        def remove_owner(manifest: dict[str, object]) -> None:
            entries = manifest["entries"]
            assert isinstance(entries, list)
            entry = next(entry for entry in entries if entry["execution_role"] == "executable")
            entry["owner_or_external_handoff"] = ""

        self.assert_rejected(remove_owner, "missing_phase_205_owner")

    def test_current_executable_entries_reject_a_phase_204_owner(self) -> None:
        """Phase-204 audit and guard tasks cannot own Phase-205 execution migration."""
        def assign_phase_204_owner(manifest: dict[str, object]) -> None:
            entries = manifest["entries"]
            assert isinstance(entries, list)
            entry = next(entry for entry in entries if entry["execution_role"] == "executable")
            entry["owner_or_external_handoff"] = "TASK-2036"

        self.assert_rejected(assign_phase_204_owner, "missing_phase_205_owner")

    def test_every_current_entry_requires_a_phase_205_owner(self) -> None:
        """Current test and reference records cannot be left with Phase-204 or no owner."""
        def assign_phase_204_owner_to_test(manifest: dict[str, object]) -> None:
            entries = manifest["entries"]
            assert isinstance(entries, list)
            entry = next(
                entry
                for entry in entries
                if entry["classification"] == "current" and entry["execution_role"] == "test-only"
            )
            entry["owner_or_external_handoff"] = "TASK-2036"

        def remove_reference_owner(manifest: dict[str, object]) -> None:
            entries = manifest["entries"]
            assert isinstance(entries, list)
            entry = next(
                entry
                for entry in entries
                if entry["classification"] == "current" and entry["execution_role"] == "reference-only"
            )
            entry["owner_or_external_handoff"] = "unowned"

        self.assert_rejected(assign_phase_204_owner_to_test, "missing_phase_205_owner")
        self.assert_rejected(remove_reference_owner, "missing_phase_205_owner")

    def test_deferred_cases_name_their_finite_failure_boundary(self) -> None:
        """Deferred is a finite named case with an exact missing obligation and result."""
        def remove_missing_obligation(manifest: dict[str, object]) -> None:
            entries = manifest["entries"]
            assert isinstance(entries, list)
            entry = next(entry for entry in entries if entry["disposition"] == "deferred")
            entry["missing_obligation"] = ""

        self.assert_rejected(remove_missing_obligation, "missing_deferred_obligation")

    def test_lean_handoff_is_not_a_deferred_test_case(self) -> None:
        """Separate Lean work names its external handoff, not a fabricated test case."""
        def clear_case_only_fields(manifest: dict[str, object]) -> None:
            entries = manifest["entries"]
            assert isinstance(entries, list)
            entry = next(entry for entry in entries if entry["path"].startswith("verification/lean/"))
            entry["case_id"] = ""
            entry["missing_obligation"] = ""
            entry["fail_closed_result"] = ""

        result, report = self.run_mutation(clear_case_only_fields)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report, {"schema": REPORT_SCHEMA, "errors": []})

    def test_only_finite_deferred_entries_may_carry_case_fields(self) -> None:
        """Case metadata cannot leak into a current migration or a Lean handoff."""
        def add_case_to_current_entry(manifest: dict[str, object]) -> None:
            entries = manifest["entries"]
            assert isinstance(entries, list)
            entry = next(entry for entry in entries if entry["disposition"] == "replace")
            entry["case_id"] = "unexpected-current-case"

        def add_case_to_lean_entry(manifest: dict[str, object]) -> None:
            entries = manifest["entries"]
            assert isinstance(entries, list)
            entry = next(entry for entry in entries if entry["path"].startswith("verification/lean/"))
            entry["fail_closed_result"] = "unexpected Lean case result"

        self.assert_rejected(add_case_to_current_entry, "unexpected_finite_case_fields")
        self.assert_rejected(add_case_to_lean_entry, "unexpected_finite_case_fields")

    def test_enums_and_lean_separate_project_handoff_are_enforced(self) -> None:
        """Lean remains deferred separate work and no enum may silently widen scope."""
        def invalid_enum(manifest: dict[str, object]) -> None:
            entries = manifest["entries"]
            assert isinstance(entries, list)
            entries[0]["execution_role"] = "client-local-evaluator"

        def invalid_lean_disposition(manifest: dict[str, object]) -> None:
            entries = manifest["entries"]
            assert isinstance(entries, list)
            entry = next(entry for entry in entries if entry["path"].startswith("verification/lean/"))
            entry["disposition"] = "delete"

        self.assert_rejected(invalid_enum, "invalid_execution_role")
        self.assert_rejected(invalid_lean_disposition, "invalid_lean_disposition")

    def test_lean_workflow_handoff_metadata_is_fail_closed(self) -> None:
        """The retained Lean CI workflow cannot omit any external handoff metadata."""
        for field, kind in (
            ("external_project", "missing_lean_handoff_metadata"),
            ("external_owner", "missing_lean_handoff_metadata"),
            ("external_handoff", "missing_lean_handoff_metadata"),
            ("retained_paths", "invalid_lean_retained_paths"),
            ("prohibited_current_authority", "missing_lean_handoff_metadata"),
        ):
            with self.subTest(field=field):
                self.assert_production_lean_handoff_field_rejected(
                    ".github/workflows/lean-reference.yml", field, kind
                )

    def test_lean_planning_handoff_metadata_is_fail_closed(self) -> None:
        """Retained Lean planning material cannot omit any external handoff metadata."""
        for field, kind in (
            ("external_project", "missing_lean_handoff_metadata"),
            ("external_owner", "missing_lean_handoff_metadata"),
            ("external_handoff", "missing_lean_handoff_metadata"),
            ("retained_paths", "invalid_lean_retained_paths"),
            ("prohibited_current_authority", "missing_lean_handoff_metadata"),
        ):
            with self.subTest(field=field):
                self.assert_production_lean_handoff_field_rejected(
                    "docs/plan/LEAN_IMPLEMENTATION_EFFORT.md", field, kind
                )


if __name__ == "__main__":
    unittest.main()
