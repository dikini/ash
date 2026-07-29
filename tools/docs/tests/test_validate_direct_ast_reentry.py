#!/usr/bin/env python3
"""RED contracts for TASK-2036's staged direct-AST re-entry guard.

The guard is a workflow control during the Phase-205 migration.  It does not
claim that listed migration debt is an approved execution architecture and it
does not exercise Ash runtime behavior.
"""
from __future__ import annotations

import hashlib
import json
import os
import subprocess
import tempfile
import unittest
from contextlib import contextmanager
from pathlib import Path
from typing import Iterator
from unittest.mock import patch


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
TOOL = REPOSITORY_ROOT / "tools/docs/validate_direct_ast_reentry.py"
MANIFEST_RELATIVE_PATH = Path("docs/plan/audits/AUDIT-204-direct-ast-retirement.json")
REPORT_SCHEMA = "direct-ast-reentry-validation-report/v1"
MANIFEST_SCHEMA = "direct-ast-retirement-audit/v1"
# Exact output of `git rev-parse --local-env-vars`: repository-routing
# variables that must not leak from the real hook worktree into a fixture.
GIT_LOCAL_REPOSITORY_ENVIRONMENT_VARIABLES = (
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_CONFIG",
    "GIT_CONFIG_PARAMETERS",
    "GIT_CONFIG_COUNT",
    "GIT_OBJECT_DIRECTORY",
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_GRAFT_FILE",
    "GIT_IMPLICIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_NO_REPLACE_OBJECTS",
    "GIT_REPLACE_REF_BASE",
    "GIT_PREFIX",
    "GIT_SHALLOW_FILE",
    "GIT_COMMON_DIR",
)


def fixture_subprocess_environment() -> dict[str, str]:
    """Copy the environment without Git local repository variables from a hook."""
    environment = os.environ.copy()
    for variable in GIT_LOCAL_REPOSITORY_ENVIRONMENT_VARIABLES:
        environment.pop(variable, None)
    return environment


def manifest_digest(entries: list[dict[str, object]]) -> str:
    """Return TASK-2034's stable-id-sorted manifest digest."""
    payload = json.dumps(
        sorted(entries, key=lambda entry: str(entry["id"])),
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    )
    return f"sha256:{hashlib.sha256(payload.encode('utf-8')).hexdigest()}"


def entry(
    entry_id: str,
    path: str,
    locator: str,
    current_role: str,
    *,
    reachability: str = "run",
    classification: str = "current",
    execution_role: str = "executable",
    disposition: str = "replace",
    owner: str = "TASK-2040",
) -> dict[str, object]:
    """Build one complete audit entry accepted by the TASK-2034 validator."""
    separate_project = classification == "deferred_separate_project"
    return {
        "id": entry_id,
        "path": path,
        "locator": locator,
        "current_role": current_role,
        "reachability": reachability,
        "classification": classification,
        "execution_role": execution_role,
        "target_rule_or_contract": "PLAN-203 Engine-only execution route",
        "disposition": disposition,
        "owner_or_external_handoff": (
            "external:lean-reference-project" if separate_project else owner
        ),
        "consumed_handoff": "AUDIT-204 frozen retirement inventory",
        "produced_handoff": "Phase-205 migration boundary",
        "required_evidence": ["TASK-2036 staged guard test"],
        "rationale": "The entry is migration debt, not approved architecture.",
        "case_id": "",
        "missing_obligation": "",
        "fail_closed_result": "",
        "external_project": "lean-reference-project" if separate_project else "",
        "external_owner": "Lean formalization maintainers" if separate_project else "",
        "external_handoff": (
            "consume canonical target rules in the separate project"
            if separate_project
            else ""
        ),
        "retained_paths": [path] if separate_project else [],
        "prohibited_current_authority": (
            "not a current Ash execution route, differential oracle, or runtime proof"
            if separate_project
            else ""
        ),
    }


class DirectAstReentryGuardContractTests(unittest.TestCase):
    """Exercise the staged-only, manifest-scoped TASK-2036 command-line guard."""

    def test_fixture_git_strips_hook_git_environment_before_subprocess(self) -> None:
        """Temporary fixture Git commands cannot inherit the real worktree routing."""
        hook_variables = {
            variable: f"/real/worktree/{variable.lower()}"
            for variable in GIT_LOCAL_REPOSITORY_ENVIRONMENT_VARIABLES
        }
        captured: dict[str, object] = {}

        def mocked_run(*arguments: object, **keywords: object) -> subprocess.CompletedProcess[str]:
            captured["env"] = keywords.get("env")
            return subprocess.CompletedProcess(arguments[0], 0, stdout="fixture output\n", stderr="")

        with patch.dict(
            os.environ,
            {
                **hook_variables,
                "TASK_2036_PRESERVED_ENVIRONMENT": "preserved",
                "GIT_PAGER": "cat",
            },
            clear=False,
        ), patch(f"{__name__}.subprocess.run", side_effect=mocked_run):
            self.assertEqual(self.git(Path("/temporary/fixture"), "status", "--short"), "fixture output")

        environment = captured.get("env")
        self.assertIsInstance(environment, dict)
        assert isinstance(environment, dict)
        for variable in GIT_LOCAL_REPOSITORY_ENVIRONMENT_VARIABLES:
            self.assertNotIn(variable, environment)
        self.assertEqual(environment.get("TASK_2036_PRESERVED_ENVIRONMENT"), "preserved")
        self.assertEqual(environment.get("GIT_PAGER"), "cat")

    def test_run_guard_strips_hook_git_environment_before_subprocess(self) -> None:
        """The launched guard cannot inherit the hook's real-worktree Git routing."""
        hook_variables = {
            variable: f"/real/worktree/{variable.lower()}"
            for variable in GIT_LOCAL_REPOSITORY_ENVIRONMENT_VARIABLES
        }
        captured: dict[str, object] = {}
        report = json.dumps(
            {
                "schema": REPORT_SCHEMA,
                "findings": [],
                "manifest_errors": [],
            }
        )

        def mocked_run(*arguments: object, **keywords: object) -> subprocess.CompletedProcess[str]:
            captured["env"] = keywords.get("env")
            return subprocess.CompletedProcess(arguments[0], 0, stdout=report, stderr="")

        with patch.dict(
            os.environ,
            {
                **hook_variables,
                "TASK_2036_PRESERVED_ENVIRONMENT": "preserved",
                "GIT_PAGER": "cat",
            },
            clear=False,
        ), patch(f"{__name__}.subprocess.run", side_effect=mocked_run):
            result, actual_report = self.run_guard(Path("/temporary/fixture"))

        self.assertEqual(result.returncode, 0)
        self.assertEqual(actual_report["schema"], REPORT_SCHEMA)
        environment = captured.get("env")
        self.assertIsInstance(environment, dict)
        assert isinstance(environment, dict)
        for variable in GIT_LOCAL_REPOSITORY_ENVIRONMENT_VARIABLES:
            self.assertNotIn(variable, environment)
        self.assertEqual(environment.get("TASK_2036_PRESERVED_ENVIRONMENT"), "preserved")
        self.assertEqual(environment.get("GIT_PAGER"), "cat")

    def git(self, root: Path, *arguments: str) -> str:
        """Run Git in a temporary fixture repository and return stdout."""
        result = subprocess.run(
            ["git", "-C", str(root), *arguments],
            check=False,
            capture_output=True,
            text=True,
            env=fixture_subprocess_environment(),
        )
        self.assertEqual(
            result.returncode,
            0,
            f"git {' '.join(arguments)} failed: {result.stderr}",
        )
        return result.stdout.strip()

    def test_fixture_repository_disables_commit_signing_before_fixture_commits(self) -> None:
        """Fixture commits explicitly opt out of any inherited signing policy."""
        commands: list[tuple[str, ...]] = []

        def fixture_git(_root: Path, *arguments: str) -> str:
            commands.append(arguments)
            if arguments == ("rev-parse", "HEAD"):
                return "0" * 40
            return ""

        with patch.object(self, "git", side_effect=fixture_git):
            with self.repository():
                pass

        signing_configuration = ("config", "commit.gpgsign", "false")
        first_fixture_commit = ("commit", "--quiet", "-m", "fixture source inventory")
        self.assertIn(signing_configuration, commands)
        self.assertLess(
            commands.index(signing_configuration),
            commands.index(first_fixture_commit),
            "fixture signing must be disabled before its first commit",
        )

    def write(self, root: Path, relative_path: str, contents: str) -> Path:
        """Write one fixture file below the repository root."""
        path = root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(contents, encoding="utf-8")
        return path

    def audit_entries(self) -> list[dict[str, object]]:
        """Return representative audited paths for every guarded category."""
        return [
            entry(
                "AUDIT-204-AST-001",
                "crates/ash-interp/src/eval.rs",
                "// audited direct AST evaluator",
                "legacy direct AST evaluator",
                reachability="differential",
            ),
            entry(
                "AUDIT-204-CPS-001",
                "crates/ash-interp/src/cps/mod.rs",
                "// audited non-Engine CPS executor",
                "non-Engine checked CPS executor",
            ),
            entry(
                "AUDIT-204-DIFF-TEST-001",
                "crates/ash-engine/tests/differential.rs",
                "// audited differential oracle",
                "Rust differential test",
                reachability="differential",
                execution_role="test-only",
                disposition="delete",
            ),
            entry(
                "AUDIT-204-TEST-EXEC-001",
                "crates/ash-cli/src/test_runner/synthesized/contract.rs",
                "// audited test-runner local evaluator",
                "client-local test runner evaluator",
                reachability="test",
                execution_role="test-only",
            ),
            entry(
                "AUDIT-204-REPL-EXEC-001",
                "crates/ash-repl/src/lib.rs",
                "// audited REPL local evaluator",
                "client-local REPL evaluator",
                reachability="repl",
            ),
            entry(
                "AUDIT-204-LEAN-001",
                "lean_reference/Ash.lean",
                "/-! audited separate Lean project -/",
                "Lean reference implementation",
                reachability="none",
                classification="deferred_separate_project",
                execution_role="reference-only",
                disposition="deferred_separate_project",
            ),
            entry(
                "AUDIT-204-LEAN-DOC-001",
                "docs/history/lean-reference.md",
                "# Historical Lean Reference",
                "historical Lean reference documentation",
                reachability="none",
                classification="historical",
                execution_role="reference-only",
                disposition="historical",
                owner="TASK-2041",
            ),
        ]

    def source_contents(self) -> dict[str, str]:
        """Return the exact base tree required by the miniature frozen audit."""
        return {
            "crates/ash-interp/src/eval.rs": "// audited direct AST evaluator\n",
            "crates/ash-interp/src/cps/mod.rs": "// audited non-Engine CPS executor\n",
            "crates/ash-engine/tests/differential.rs": "// audited differential oracle\n",
            "crates/ash-cli/src/test_runner/synthesized/contract.rs": "// audited test-runner local evaluator\n",
            "crates/ash-repl/src/lib.rs": "// audited REPL local evaluator\n",
            "lean_reference/Ash.lean": "/-! audited separate Lean project -/\n",
            "docs/history/lean-reference.md": "# Historical Lean Reference\n",
        }

    @contextmanager
    def repository(self, *, retain_deleted_rust_entry: bool = False) -> Iterator[Path]:
        """Yield a repository whose manifest is valid at a frozen ancestor commit."""
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory) / "guard-fixture"
            root.mkdir()
            self.git(root, "init", "--quiet")
            self.git(root, "config", "user.email", "task-2036@example.invalid")
            self.git(root, "config", "user.name", "TASK-2036 fixture")
            self.git(root, "config", "commit.gpgsign", "false")

            for path, contents in self.source_contents().items():
                self.write(root, path, contents)
            self.write(
                root,
                str(MANIFEST_RELATIVE_PATH),
                json.dumps(
                    {
                        "schema": MANIFEST_SCHEMA,
                        "repository_revision": "0" * 40,
                        "entries_sha256": manifest_digest(self.audit_entries()),
                        "entries": self.audit_entries(),
                    },
                    indent=2,
                    sort_keys=True,
                )
                + "\n",
            )
            self.git(root, "add", ".")
            self.git(root, "commit", "--quiet", "-m", "fixture source inventory")
            frozen_revision = self.git(root, "rev-parse", "HEAD")

            manifest = {
                "schema": MANIFEST_SCHEMA,
                "repository_revision": frozen_revision,
                "entries_sha256": manifest_digest(self.audit_entries()),
                "entries": self.audit_entries(),
            }
            self.write(
                root,
                str(MANIFEST_RELATIVE_PATH),
                json.dumps(manifest, indent=2, sort_keys=True) + "\n",
            )
            self.git(root, "add", str(MANIFEST_RELATIVE_PATH))
            self.git(root, "commit", "--quiet", "-m", "freeze audit manifest")
            if not retain_deleted_rust_entry:
                self.git(root, "rm", "--", "crates/ash-engine/tests/differential.rs")
                self.git(root, "commit", "--quiet", "-m", "retire differential test")
            yield root

    def stage_append(self, root: Path, relative_path: str, addition: str) -> None:
        """Append an added line and place the exact diff in the index."""
        path = root / relative_path
        path.write_text(path.read_text(encoding="utf-8") + addition, encoding="utf-8")
        self.git(root, "add", relative_path)

    def stage_new(self, root: Path, relative_path: str, contents: str) -> None:
        """Create and stage a new file, retaining its path in the staged diff."""
        self.write(root, relative_path, contents)
        self.git(root, "add", relative_path)

    def stage_private_differential_test_move(
        self, root: Path, source_path: str, target_path: str
    ) -> None:
        """Stage a retired test resurrection beside a private relocation target."""
        source = root / source_path
        target = root / target_path
        source.parent.mkdir(parents=True, exist_ok=True)
        target.parent.mkdir(parents=True, exist_ok=True)
        source_contents = "// resurrected retired differential oracle\n"
        source.write_text(source_contents, encoding="utf-8")
        target.write_text(
            source_contents + "let harness = DifferentialHarness::new();\n",
            encoding="utf-8",
        )
        self.git(root, "add", source_path)
        self.git(root, "add", target_path)

    def rewrite_manifest(self, root: Path, mutate: object) -> None:
        """Mutate and stage the fixture manifest while retaining a correct digest."""
        manifest_path = root / MANIFEST_RELATIVE_PATH
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        self.assertTrue(callable(mutate))
        mutate(manifest)
        manifest["entries_sha256"] = manifest_digest(manifest["entries"])
        manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        self.git(root, "add", str(MANIFEST_RELATIVE_PATH))

    def run_guard(self, root: Path) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        """Run the wished-for staged guard and require a JSON-only report."""
        self.assertTrue(TOOL.is_file(), "missing TASK-2036 guard under test")
        manifest_path = root / MANIFEST_RELATIVE_PATH
        result = subprocess.run(
            [
                "python3",
                str(TOOL),
                "--root",
                str(root),
                "--manifest",
                str(manifest_path),
                "--staged",
                "--format",
                "json",
            ],
            check=False,
            capture_output=True,
            text=True,
            env=fixture_subprocess_environment(),
        )
        try:
            report = json.loads(result.stdout)
        except json.JSONDecodeError as error:
            self.fail(f"guard must emit a JSON report on stdout: {error}; stderr: {result.stderr}")
        self.assertEqual(report.get("schema"), REPORT_SCHEMA)
        self.assertIsInstance(report.get("findings"), list)
        self.assertIsInstance(report.get("manifest_errors"), list)
        return result, report

    def finding(
        self,
        report: dict[str, object],
        *,
        kind: str,
        category: str,
        path: str,
    ) -> dict[str, object]:
        """Return one required structured finding or fail with the full report."""
        findings = report["findings"]
        assert isinstance(findings, list)
        for finding in findings:
            if isinstance(finding, dict) and all(
                finding.get(key) == value
                for key, value in (("kind", kind), ("category", category), ("path", path))
            ):
                return finding
        self.fail(f"missing {kind} finding for {path}: {report}")

    def test_current_listed_rust_delete_entry_fails_closed_without_a_staged_addition(self) -> None:
        """After cutover, a listed Rust delete entry cannot remain present at all."""
        with self.repository(retain_deleted_rust_entry=True) as root:
            path = "crates/ash-engine/tests/differential.rs"
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="current_listed_rust_use",
            category="differential_oracle",
            path=path,
        )
        self.assertEqual(finding.get("manifest_id"), "AUDIT-204-DIFF-TEST-001")
        self.assertEqual(finding.get("location"), "manifest-listed")

    def test_listed_path_direct_ast_evaluator_reentry_still_fails_closed(self) -> None:
        """An audit ID owns prior debt but cannot exempt a new evaluator addition."""
        with self.repository() as root:
            path = "crates/ash-interp/src/eval.rs"
            self.stage_append(root, path, "let _ = eval_expr(expression);\n")
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="unlisted_reentry",
            category="direct_ast_evaluator",
            path=path,
        )
        self.assertEqual(finding.get("manifest_id"), "AUDIT-204-AST-001")
        self.assertEqual(finding.get("location"), "manifest-listed")

    def test_unlisted_direct_ast_evaluator_fails_closed_at_unknown_location(self) -> None:
        """New AST evaluator code below an audited source root has no allowlist entry."""
        with self.repository() as root:
            path = "crates/ash-new/src/evaluator.rs"
            self.stage_new(root, path, "pub fn run(expr: Expr) { eval_expr(expr); }\n")
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="unlisted_reentry",
            category="direct_ast_evaluator",
            path=path,
        )
        self.assertIsNone(finding.get("manifest_id"))
        self.assertEqual(finding.get("location"), "unknown")

    def test_unlisted_async_direct_ast_evaluator_fails_closed(self) -> None:
        """An async spelling of direct AST evaluation cannot bypass the guard."""
        with self.repository() as root:
            path = "crates/ash-new/src/async_evaluator.rs"
            self.stage_new(root, path, "async fn run(expr: Expr) { eval_expr_async(expr).await; }\n")
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="unlisted_reentry",
            category="direct_ast_evaluator",
            path=path,
        )
        self.assertIsNone(finding.get("manifest_id"))
        self.assertEqual(finding.get("location"), "unknown")

    def test_unlisted_direct_ast_function_pointer_fails_closed(self) -> None:
        """Taking the direct evaluator as a value is execution reachability too."""
        with self.repository() as root:
            path = "crates/ash-new/src/evaluator_pointer.rs"
            self.stage_new(root, path, "fn select() { let evaluator = eval_expr; }\n")
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="unlisted_reentry",
            category="direct_ast_evaluator",
            path=path,
        )
        self.assertIsNone(finding.get("manifest_id"))
        self.assertEqual(finding.get("location"), "unknown")

    def test_unlisted_public_non_engine_cps_executor_fails_closed(self) -> None:
        """A public CPS executor outside ash-engine cannot re-enter under a new crate."""
        with self.repository() as root:
            path = "crates/ash-cps/src/lib.rs"
            self.stage_new(
                root,
                path,
                "pub fn execute_checked_cps(program: CheckedCps) -> Terminal { todo!() }\n",
            )
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="unlisted_reentry",
            category="public_non_engine_cps_executor",
            path=path,
        )
        self.assertIsNone(finding.get("manifest_id"))
        self.assertEqual(finding.get("location"), "unknown")

    def test_unlisted_public_terminal_cps_executor_fails_closed(self) -> None:
        """A terminal-evaluation spelling is also a public non-Engine CPS route."""
        with self.repository() as root:
            path = "crates/ash-cps/src/terminal.rs"
            self.stage_new(
                root,
                path,
                "pub fn eval_checked_terminal(program: CheckedCps) -> Terminal { todo!() }\n",
            )
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="unlisted_reentry",
            category="public_non_engine_cps_executor",
            path=path,
        )
        self.assertIsNone(finding.get("manifest_id"))
        self.assertEqual(finding.get("location"), "unknown")

    def test_unlisted_public_checked_cps_executor_fails_closed(self) -> None:
        """A public eval_checked API outside the Engine is another CPS executor route."""
        with self.repository() as root:
            path = "crates/ash-cps/src/checked.rs"
            self.stage_new(
                root,
                path,
                "pub fn eval_checked(program: CheckedCps) -> Terminal { todo!() }\n",
            )
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="unlisted_reentry",
            category="public_non_engine_cps_executor",
            path=path,
        )
        self.assertIsNone(finding.get("manifest_id"))
        self.assertEqual(finding.get("location"), "unknown")

    def test_unlisted_public_unchecked_cps_executor_fails_closed(self) -> None:
        """A public eval_unchecked API outside the Engine is another CPS executor route."""
        with self.repository() as root:
            path = "crates/ash-cps/src/unchecked.rs"
            self.stage_new(
                root,
                path,
                "pub fn eval_unchecked(program: Cps) -> Terminal { todo!() }\n",
            )
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="unlisted_reentry",
            category="public_non_engine_cps_executor",
            path=path,
        )
        self.assertIsNone(finding.get("manifest_id"))
        self.assertEqual(finding.get("location"), "unknown")

    def test_unlisted_public_terminal_cps_reexport_fails_closed(self) -> None:
        """Re-exporting a terminal CPS API is public executor reachability too."""
        with self.repository() as root:
            path = "crates/ash-cps/src/lib.rs"
            self.stage_new(root, path, "pub use ash_interp::cps::eval_checked_terminal;\n")
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="unlisted_reentry",
            category="public_non_engine_cps_executor",
            path=path,
        )
        self.assertIsNone(finding.get("manifest_id"))
        self.assertEqual(finding.get("location"), "unknown")

    def test_unlisted_differential_oracle_fails_closed(self) -> None:
        """A new differential oracle below an audited test root remains prohibited."""
        with self.repository() as root:
            path = "crates/ash-engine/tests/new_differential_oracle.rs"
            self.stage_new(
                root,
                path,
                "fn differential_oracle(program: Program) { legacy_eval(program); }\n",
            )
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="unlisted_reentry",
            category="differential_oracle",
            path=path,
        )
        self.assertIsNone(finding.get("manifest_id"))
        self.assertEqual(finding.get("location"), "unknown")

    def test_unlisted_differential_harness_marker_fails_closed(self) -> None:
        """A comparison harness is prohibited even without the exact oracle name."""
        with self.repository() as root:
            path = "crates/ash-engine/tests/new_differential_harness.rs"
            self.stage_new(root, path, "let harness = DifferentialHarness::new();\n")
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="unlisted_reentry",
            category="differential_oracle",
            path=path,
        )
        self.assertIsNone(finding.get("manifest_id"))
        self.assertEqual(finding.get("location"), "unknown")

    def test_private_differential_test_relocation_fails_closed_after_cutover(self) -> None:
        """A deleted differential test cannot be reintroduced under an Engine-private path."""
        with self.repository() as root:
            source_path = "crates/ash-engine/tests/differential.rs"
            target_path = "crates/ash-engine/src/differential/tests/differential.rs"
            self.stage_private_differential_test_move(root, source_path, target_path)
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="unlisted_reentry",
            category="differential_oracle",
            path=target_path,
        )
        self.assertIsNone(finding.get("manifest_id"))
        self.assertEqual(finding.get("location"), "unknown")

    def test_private_engine_differential_addition_without_same_change_audit_move_fails_closed(self) -> None:
        """A private test path alone is never a reusable differential-oracle exception."""
        with self.repository() as root:
            path = "crates/ash-engine/src/differential/tests/new_harness.rs"
            self.stage_new(root, path, "let harness = DifferentialHarness::new();\n")
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="unlisted_reentry",
            category="differential_oracle",
            path=path,
        )
        self.assertIsNone(finding.get("manifest_id"))
        self.assertEqual(finding.get("location"), "unknown")

    def test_unlisted_test_runner_local_evaluation_fails_closed(self) -> None:
        """A test-runner helper cannot create its own local execution route."""
        with self.repository() as root:
            path = "crates/ash-cli/src/test_runner/engine_bypass.rs"
            self.stage_new(root, path, "fn execute_case(expr: Expr) { eval_expr(expr); }\n")
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="unlisted_reentry",
            category="client_local_evaluation",
            path=path,
        )
        self.assertEqual(finding.get("client"), "test_runner")
        self.assertIsNone(finding.get("manifest_id"))
        self.assertEqual(finding.get("location"), "unknown")

    def test_unlisted_repl_local_evaluation_fails_closed(self) -> None:
        """A REPL command cannot create a second evaluator route."""
        with self.repository() as root:
            path = "crates/ash-repl/src/commands.rs"
            self.stage_new(root, path, "fn submit(expr: Expr) { eval_expr(expr); }\n")
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="unlisted_reentry",
            category="client_local_evaluation",
            path=path,
        )
        self.assertEqual(finding.get("client"), "repl")
        self.assertIsNone(finding.get("manifest_id"))
        self.assertEqual(finding.get("location"), "unknown")

    def test_unlisted_repl_checked_cps_evaluation_fails_closed(self) -> None:
        """A checked-CPS spelling cannot create a local REPL evaluator route."""
        with self.repository() as root:
            path = "crates/ash-repl/src/checked_eval.rs"
            self.stage_new(
                root,
                path,
                "fn submit(program: CheckedCps) { let _ = eval_checked_terminal(program); }\n",
            )
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="unlisted_reentry",
            category="client_local_evaluation",
            path=path,
        )
        self.assertEqual(finding.get("client"), "repl")
        self.assertIsNone(finding.get("manifest_id"))
        self.assertEqual(finding.get("location"), "unknown")

    def test_unlisted_repl_eval_checked_call_fails_closed(self) -> None:
        """A REPL local eval_checked call cannot bypass admitted Engine execution."""
        with self.repository() as root:
            path = "crates/ash-repl/src/eval_checked.rs"
            self.stage_new(
                root,
                path,
                "fn submit(program: CheckedCps) { let _ = eval_checked(program); }\n",
            )
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="unlisted_reentry",
            category="client_local_evaluation",
            path=path,
        )
        self.assertEqual(finding.get("client"), "repl")
        self.assertIsNone(finding.get("manifest_id"))
        self.assertEqual(finding.get("location"), "unknown")

    def test_unlisted_run_client_local_evaluation_fails_closed(self) -> None:
        """A run-client command cannot bypass admitted Engine execution."""
        with self.repository() as root:
            path = "crates/ash-cli/src/commands/run/local_eval.rs"
            self.stage_new(root, path, "fn execute(expr: Expr) { eval_expr(expr); }\n")
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="unlisted_reentry",
            category="client_local_evaluation",
            path=path,
        )
        self.assertEqual(finding.get("client"), "run")
        self.assertIsNone(finding.get("manifest_id"))
        self.assertEqual(finding.get("location"), "unknown")

    def test_unlisted_daemon_client_local_evaluation_fails_closed(self) -> None:
        """A daemon request handler cannot create another local evaluator route."""
        with self.repository() as root:
            path = "crates/ash-daemon/src/request_eval.rs"
            self.stage_new(root, path, "fn execute(expr: Expr) { eval_expr(expr); }\n")
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="unlisted_reentry",
            category="client_local_evaluation",
            path=path,
        )
        self.assertEqual(finding.get("client"), "daemon")
        self.assertIsNone(finding.get("manifest_id"))
        self.assertEqual(finding.get("location"), "unknown")

    def test_listed_deferred_lean_material_is_visible_only_as_separate_project_debt(self) -> None:
        """A listed Lean file can be changed only under its deferred-project handoff."""
        with self.repository() as root:
            path = "lean_reference/Ash.lean"
            self.stage_append(root, path, "/- preserved separate-project proof note -/\n")
            result, report = self.run_guard(root)

        self.assertEqual(result.returncode, 0, result.stderr)
        finding = self.finding(
            report,
            kind="listed_migration_debt",
            category="lean_separate_project",
            path=path,
        )
        self.assertEqual(finding.get("manifest_id"), "AUDIT-204-LEAN-001")
        self.assertEqual(finding.get("location"), "manifest-listed")

    def test_historical_lean_document_relabelled_as_current_authority_fails_closed(self) -> None:
        """Historical prose cannot become current Ash execution authority again."""
        with self.repository() as root:
            path = "docs/history/lean-reference.md"
            self.stage_append(
                root,
                path,
                "\nThe Lean reference interpreter is the current Ash execution authority.\n",
            )
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="stale_current_ash_authority",
            category="lean_authority",
            path=path,
        )
        self.assertEqual(finding.get("manifest_id"), "AUDIT-204-LEAN-DOC-001")
        self.assertEqual(finding.get("location"), "manifest-listed")

    def test_historical_lean_document_reverse_order_current_route_wording_fails_closed(self) -> None:
        """Current-route authority wording is forbidden regardless of word order."""
        with self.repository() as root:
            path = "docs/history/lean-reference.md"
            self.stage_append(
                root,
                path,
                "\nThe current Ash execution route is the Lean reference interpreter.\n",
            )
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="stale_current_ash_authority",
            category="lean_authority",
            path=path,
        )
        self.assertEqual(finding.get("manifest_id"), "AUDIT-204-LEAN-DOC-001")
        self.assertEqual(finding.get("location"), "manifest-listed")

    def test_historical_lean_document_current_differential_oracle_authority_fails_closed(self) -> None:
        """Lean cannot become the current Ash differential oracle through new prose."""
        with self.repository() as root:
            path = "docs/history/lean-reference.md"
            self.stage_append(
                root,
                path,
                "\nThe Lean reference interpreter is the current Ash differential oracle.\n",
            )
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="stale_current_ash_authority",
            category="lean_authority",
            path=path,
        )
        self.assertEqual(finding.get("manifest_id"), "AUDIT-204-LEAN-DOC-001")
        self.assertEqual(finding.get("location"), "manifest-listed")

    def test_historical_lean_document_current_conformance_authority_fails_closed(self) -> None:
        """Lean cannot be relabelled as current Ash conformance authority."""
        with self.repository() as root:
            path = "docs/history/lean-reference.md"
            self.stage_append(
                root,
                path,
                "\nLean is the current Ash conformance authority.\n",
            )
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="stale_current_ash_authority",
            category="lean_authority",
            path=path,
        )
        self.assertEqual(finding.get("manifest_id"), "AUDIT-204-LEAN-DOC-001")
        self.assertEqual(finding.get("location"), "manifest-listed")

    def test_historical_lean_document_current_proof_evidence_fails_closed(self) -> None:
        """Lean cannot be relabelled as current Ash proof evidence."""
        with self.repository() as root:
            path = "docs/history/lean-reference.md"
            self.stage_append(
                root,
                path,
                "\nLean provides current Ash proof evidence.\n",
            )
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="stale_current_ash_authority",
            category="lean_authority",
            path=path,
        )
        self.assertEqual(finding.get("manifest_id"), "AUDIT-204-LEAN-DOC-001")
        self.assertEqual(finding.get("location"), "manifest-listed")

    def test_historical_lean_document_current_runtime_refinement_proof_fails_closed(
        self,
    ) -> None:
        """Lean cannot be relabelled as the current Ash runtime refinement proof."""
        with self.repository() as root:
            path = "docs/history/lean-reference.md"
            self.stage_append(
                root,
                path,
                "\nLean provides the current Ash runtime refinement proof.\n",
            )
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        finding = self.finding(
            report,
            kind="stale_current_ash_authority",
            category="lean_authority",
            path=path,
        )
        self.assertEqual(finding.get("manifest_id"), "AUDIT-204-LEAN-DOC-001")
        self.assertEqual(finding.get("location"), "manifest-listed")

    def test_duplicate_allowlist_id_cannot_replace_the_frozen_manifest(self) -> None:
        """A repeated ID changes the staged audit and must fail before revalidation."""
        with self.repository() as root:
            def duplicate_id(manifest: object) -> None:
                assert isinstance(manifest, dict)
                entries = manifest["entries"]
                assert isinstance(entries, list)
                assert isinstance(entries[1], dict)
                entries[1]["id"] = "AUDIT-204-AST-001"

            self.rewrite_manifest(root, duplicate_id)
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        manifest_errors = report["manifest_errors"]
        assert isinstance(manifest_errors, list)
        self.assertTrue(
            any(
                isinstance(error, dict) and error.get("kind") == "frozen_manifest_modified"
                for error in manifest_errors
            ),
            report,
        )

    def test_stale_allowlist_entry_cannot_replace_the_frozen_manifest(self) -> None:
        """A staged stale locator is a manifest rewrite, never a new allowlist boundary."""
        with self.repository() as root:
            def stale_locator(manifest: object) -> None:
                assert isinstance(manifest, dict)
                entries = manifest["entries"]
                assert isinstance(entries, list)
                assert isinstance(entries[0], dict)
                entries[0]["locator"] = "// stale audit locator"

            self.rewrite_manifest(root, stale_locator)
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        manifest_errors = report["manifest_errors"]
        assert isinstance(manifest_errors, list)
        self.assertTrue(
            any(
                isinstance(error, dict) and error.get("kind") == "frozen_manifest_modified"
                for error in manifest_errors
            ),
            report,
        )

    def test_removing_a_current_manifest_entry_cannot_weaken_the_frozen_allowlist(self) -> None:
        """A digest-correct deletion of migration debt remains a forbidden audit rewrite."""
        with self.repository() as root:
            def remove_current_entry(manifest: object) -> None:
                assert isinstance(manifest, dict)
                entries = manifest["entries"]
                assert isinstance(entries, list)
                entries[:] = [
                    item
                    for item in entries
                    if not (isinstance(item, dict) and item.get("id") == "AUDIT-204-AST-001")
                ]

            self.rewrite_manifest(root, remove_current_entry)
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        manifest_errors = report["manifest_errors"]
        assert isinstance(manifest_errors, list)
        self.assertTrue(
            any(
                isinstance(error, dict) and error.get("kind") == "frozen_manifest_modified"
                for error in manifest_errors
            ),
            report,
        )

    def test_staged_removal_of_listed_delete_entry_keeps_the_frozen_audit_valid(self) -> None:
        """Phase-205 deletion may remove a listed delete item without locator validation."""
        with self.repository(retain_deleted_rust_entry=True) as root:
            path = "crates/ash-engine/tests/differential.rs"
            self.git(root, "rm", "--", path)
            result, report = self.run_guard(root)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["manifest_errors"], [], report)
        self.assertEqual(report["findings"], [])

    def test_nonlegacy_staged_change_under_a_scanned_root_passes(self) -> None:
        """Ordinary code under a scanned root is not rejected by a wildcard rule."""
        with self.repository() as root:
            self.stage_new(
                root,
                "crates/ash-new/src/value.rs",
                "pub fn normalize(value: i64) -> i64 { value }\n",
            )
            result, report = self.run_guard(root)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["findings"], [])
        self.assertEqual(report["manifest_errors"], [])

    def test_historical_prose_mentioning_a_cps_api_is_not_runtime_reentry(self) -> None:
        """Rust API markers are checked in Rust sources, not ordinary history prose."""
        with self.repository() as root:
            self.stage_new(
                root,
                "docs/history/cps-api-notes.md",
                "Historical note: `eval_checked_terminal` was a retired Rust API symbol.\n",
            )
            result, report = self.run_guard(root)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["manifest_errors"], [], report)
        self.assertEqual(report["findings"], [])

    def test_historical_prose_mentioning_differential_oracle_is_not_runtime_reentry(self) -> None:
        """Historical differential terminology alone does not introduce an evaluator route."""
        with self.repository() as root:
            self.stage_new(
                root,
                "docs/history/differential-notes.md",
                "Historical note: the differential oracle was retired from the prior design.\n",
            )
            result, report = self.run_guard(root)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["manifest_errors"], [], report)
        self.assertEqual(report["findings"], [])

    def test_unstaged_legacy_text_is_outside_the_staged_diff_view(self) -> None:
        """Only additions placed in the Git index are gate input."""
        with self.repository() as root:
            self.write(
                root,
                "crates/ash-new/src/not_staged.rs",
                "pub fn run(expr: Expr) { eval_expr(expr); }\n",
            )
            result, report = self.run_guard(root)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["findings"], [])

    def test_staged_snapshot_ignores_unstaged_manifest_and_locator_corruption(self) -> None:
        """The guard must read its audit boundary from the index, never the worktree."""
        with self.repository() as root:
            reentry_path = "crates/ash-new/src/evaluator.rs"
            self.stage_new(
                root,
                reentry_path,
                "pub fn run(expr: Expr) { eval_expr(expr); }\n",
            )

            # These edits deliberately remain unstaged.  A staged-only guard must
            # use the indexed audit and indexed source locator instead.
            (root / MANIFEST_RELATIVE_PATH).write_text("{invalid working manifest", encoding="utf-8")
            self.write(
                root,
                "crates/ash-interp/src/eval.rs",
                "// unstaged source no longer contains the audit locator\n",
            )
            result, report = self.run_guard(root)

        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(report["manifest_errors"], [], report)
        finding = self.finding(
            report,
            kind="unlisted_reentry",
            category="direct_ast_evaluator",
            path=reentry_path,
        )
        self.assertIsNone(finding.get("manifest_id"))
        self.assertEqual(finding.get("location"), "unknown")


if __name__ == "__main__":
    unittest.main()
