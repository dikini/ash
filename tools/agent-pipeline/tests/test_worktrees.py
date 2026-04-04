"""Tests for deterministic worktree derivation and provisioning."""

from __future__ import annotations

import subprocess
import tempfile
from pathlib import Path

import pytest

from agent_pipeline.state import TaskManifest
from agent_pipeline.worktrees import (
    ProvisioningOutcome,
    WorktreeAssignment,
    derive_worktree_assignment,
    derive_worktree_branch,
    derive_worktree_path,
    ensure_worktree_ignored,
    ensure_worktree_ready,
    remove_task_worktree,
)


def _completed(*, returncode: int = 0, stdout: str = "", stderr: str = "") -> subprocess.CompletedProcess[str]:
    return subprocess.CompletedProcess(args=["git"], returncode=returncode, stdout=stdout, stderr=stderr)


class TestWorktrees:
    """Test worktree derivation and provisioning helpers."""

    @pytest.fixture
    def temp_dir(self):
        with tempfile.TemporaryDirectory() as tmp:
            yield Path(tmp)

    def test_derive_and_path_uses_repo_root_not_base_dir(self, temp_dir):
        """RED: Deterministic worktree paths must derive from the repository root, not the task-bundle base dir."""
        repo_root = temp_dir / "repo"
        repo_root.mkdir()
        base_dir = temp_dir / "external-state"
        base_dir.mkdir()

        assignment = derive_worktree_assignment(repo_root, "TASK-378")

        assert derive_worktree_path(repo_root, "TASK-378") == repo_root / ".worktrees" / "TASK-378"
        assert assignment.path == repo_root / ".worktrees" / "TASK-378"
        assert str(base_dir) not in str(assignment.path)

    def test_branch_derivation_is_deterministic(self):
        """RED: Branch names must follow the fixed task-scoped naming contract."""
        assert derive_worktree_branch("TASK-378") == "agent-pipeline/TASK-378"

    def test_ignored_check_passes_when_worktrees_directory_is_gitignored(self, temp_dir):
        """RED: Provisioning should accept repositories that explicitly ignore .worktrees/."""
        repo_root = temp_dir / "repo"
        repo_root.mkdir()

        def run_git(*_args: str, cwd: Path) -> subprocess.CompletedProcess[str]:
            assert cwd == repo_root
            return _completed(stdout=".worktrees/\n")

        assert ensure_worktree_ignored(repo_root, run_git=run_git) is None

    def test_ignored_check_blocks_when_worktrees_directory_is_not_gitignored(self, temp_dir):
        """RED: Provisioning must fail closed when .worktrees/ is not ignored by git."""
        repo_root = temp_dir / "repo"
        repo_root.mkdir()

        def run_git(*_args: str, cwd: Path) -> subprocess.CompletedProcess[str]:
            return _completed(returncode=1, stderr="")

        assert ensure_worktree_ignored(repo_root, run_git=run_git) == (
            "worktree provisioning blocked: repository must git-ignore .worktrees/ before provisioning"
        )

    def test_create_or_reuse_or_block_reuses_matching_existing_worktree(self, temp_dir):
        """RED: Existing matching task worktrees should be reused instead of recreated blindly."""
        repo_root = temp_dir / "repo"
        repo_root.mkdir()
        assignment = WorktreeAssignment(
            path=repo_root / ".worktrees" / "TASK-378",
            branch="agent-pipeline/TASK-378",
        )
        assignment.path.mkdir(parents=True)
        manifest = TaskManifest.create("TASK-378", started_by="cli")
        manifest.set_worktree(path=assignment.path, branch=assignment.branch)

        def run_git(*args: str, cwd: Path) -> subprocess.CompletedProcess[str]:
            if args[:2] == ("check-ignore", ".worktrees/"):
                return _completed(stdout=".worktrees/\n")
            if args[:2] == ("worktree", "list"):
                return _completed(
                    stdout=(
                        f"worktree {assignment.path}\n"
                        "HEAD 0123456789012345678901234567890123456789\n"
                        f"branch refs/heads/{assignment.branch}\n\n"
                    )
                )
            raise AssertionError(f"unexpected git args: {args}")

        result = ensure_worktree_ready(repo_root, manifest, run_git=run_git)

        assert result.outcome == ProvisioningOutcome.REUSE
        assert result.assignment == assignment
        assert result.reason is None

    def test_create_or_reuse_or_block_blocks_mismatched_existing_worktree(self, temp_dir):
        """RED: Existing worktrees with the wrong checked-out branch must block instead of being recreated implicitly."""
        repo_root = temp_dir / "repo"
        repo_root.mkdir()
        expected = repo_root / ".worktrees" / "TASK-378"
        expected.mkdir(parents=True)
        manifest = TaskManifest.create("TASK-378", started_by="cli")
        manifest.set_worktree(path=expected, branch="agent-pipeline/TASK-378")

        def run_git(*args: str, cwd: Path) -> subprocess.CompletedProcess[str]:
            if args[:2] == ("check-ignore", ".worktrees/"):
                return _completed(stdout=".worktrees/\n")
            if args[:2] == ("worktree", "list"):
                return _completed(
                    stdout=(
                        f"worktree {expected}\n"
                        "HEAD 0123456789012345678901234567890123456789\n"
                        "branch refs/heads/agent-pipeline/TASK-999\n\n"
                    )
                )
            raise AssertionError(f"unexpected git args: {args}")

        result = ensure_worktree_ready(repo_root, manifest, run_git=run_git)

        assert result.outcome == ProvisioningOutcome.BLOCK
        assert result.reason == (
            "worktree provisioning blocked: existing worktree branch "
            "'agent-pipeline/TASK-999' does not match expected 'agent-pipeline/TASK-378'"
        )

    def test_create_or_reuse_or_block_creates_missing_worktree_with_deterministic_assignment(self, temp_dir):
        """RED: Missing worktrees should be created with the fixed path and branch naming contract."""
        repo_root = temp_dir / "repo"
        repo_root.mkdir()
        manifest = TaskManifest.create("TASK-378", started_by="cli")
        calls: list[tuple[str, ...]] = []

        def run_git(*args: str, cwd: Path) -> subprocess.CompletedProcess[str]:
            calls.append(args)
            if args[:2] == ("check-ignore", ".worktrees/"):
                return _completed(stdout=".worktrees/\n")
            if args[:2] == ("worktree", "list"):
                return _completed(stdout="")
            if args[:3] == ("show-ref", "--verify", "--quiet"):
                return _completed(returncode=1)
            if args[:3] == ("worktree", "add", "-b"):
                return _completed()
            raise AssertionError(f"unexpected git args: {args}")

        result = ensure_worktree_ready(repo_root, manifest, run_git=run_git)

        assert result.outcome == ProvisioningOutcome.CREATE
        assert result.assignment.path == repo_root / ".worktrees" / "TASK-378"
        assert result.assignment.branch == "agent-pipeline/TASK-378"
        assert (
            "worktree",
            "add",
            "-b",
            "agent-pipeline/TASK-378",
            str(repo_root / ".worktrees" / "TASK-378"),
            "HEAD",
        ) in calls

    def test_remove_task_worktree_rejects_persisted_path_outside_repo_worktrees_root(self, temp_dir):
        """RED: Cleanup must fail closed when persisted worktree metadata points outside <repo>/.worktrees/<TASK-ID>."""
        repo_root = temp_dir / "repo"
        repo_root.mkdir()
        outside = temp_dir / "outside-target"
        outside.mkdir()
        manifest = TaskManifest.create("TASK-378", started_by="cli")
        manifest.set_worktree(path=outside, branch="agent-pipeline/TASK-378")

        with pytest.raises(RuntimeError, match="must stay under"):
            remove_task_worktree(repo_root, manifest)

    def test_remove_task_worktree_rejects_mismatched_persisted_assignment(self, temp_dir):
        """RED: Cleanup must validate the stored assignment against the deterministic task worktree path and branch."""
        repo_root = temp_dir / "repo"
        repo_root.mkdir()
        worktree_path = repo_root / ".worktrees" / "TASK-378"
        worktree_path.mkdir(parents=True)
        manifest = TaskManifest.create("TASK-378", started_by="cli")
        manifest.set_worktree(path=worktree_path, branch="agent-pipeline/TASK-999")

        with pytest.raises(RuntimeError, match="does not match expected"):
            remove_task_worktree(repo_root, manifest)

    def test_create_or_reuse_or_block_recreates_stale_git_worktree_metadata_when_directory_is_missing(self, temp_dir):
        """RED: Stale git worktree entries with missing directories should be pruned before deterministic reprovision."""
        repo_root = temp_dir / "repo"
        repo_root.mkdir()
        expected = repo_root / ".worktrees" / "TASK-378"
        manifest = TaskManifest.create("TASK-378", started_by="cli")
        manifest.set_worktree(path=expected, branch="agent-pipeline/TASK-378")
        calls: list[tuple[str, ...]] = []

        def run_git(*args: str, cwd: Path) -> subprocess.CompletedProcess[str]:
            calls.append(args)
            if args[:2] == ("check-ignore", ".worktrees/"):
                return _completed(stdout=".worktrees/\n")
            if args[:2] == ("worktree", "list"):
                return _completed(
                    stdout=(
                        f"worktree {expected}\n"
                        "HEAD 0123456789012345678901234567890123456789\n"
                        "branch refs/heads/agent-pipeline/TASK-378\n\n"
                    )
                )
            if args[:2] == ("worktree", "prune"):
                return _completed()
            if args[:3] == ("show-ref", "--verify", "--quiet"):
                return _completed(returncode=1)
            if args[:3] == ("worktree", "add", "-b"):
                return _completed()
            raise AssertionError(f"unexpected git args: {args}")

        result = ensure_worktree_ready(repo_root, manifest, run_git=run_git)

        assert result.outcome == ProvisioningOutcome.CREATE
        assert ("worktree", "prune") in calls
        assert ("worktree", "add", "-b", "agent-pipeline/TASK-378", str(expected), "HEAD") in calls

    def test_create_or_reuse_or_block_blocks_when_stale_git_worktree_prune_fails(self, temp_dir):
        """RED: Stale registered worktrees should fail closed if prune cannot clear the missing entry."""
        repo_root = temp_dir / "repo"
        repo_root.mkdir()
        expected = repo_root / ".worktrees" / "TASK-378"
        manifest = TaskManifest.create("TASK-378", started_by="cli")
        manifest.set_worktree(path=expected, branch="agent-pipeline/TASK-378")

        def run_git(*args: str, cwd: Path) -> subprocess.CompletedProcess[str]:
            if args[:2] == ("check-ignore", ".worktrees/"):
                return _completed(stdout=".worktrees/\n")
            if args[:2] == ("worktree", "list"):
                return _completed(
                    stdout=(
                        f"worktree {expected}\n"
                        "HEAD 0123456789012345678901234567890123456789\n"
                        "branch refs/heads/agent-pipeline/TASK-378\n\n"
                    )
                )
            if args[:2] == ("worktree", "prune"):
                return _completed(returncode=1, stderr="prune failed")
            raise AssertionError(f"unexpected git args: {args}")

        result = ensure_worktree_ready(repo_root, manifest, run_git=run_git)

        assert result.outcome == ProvisioningOutcome.BLOCK
        assert result.reason == "worktree provisioning blocked: prune failed"

    def test_create_or_reuse_or_block_rejects_invalid_persisted_task_id_before_deriving_real_assignment(self, temp_dir):
        """RED: On-disk malformed task ids must fail closed before any worktree path escape is possible."""
        repo_root = temp_dir / "repo"
        repo_root.mkdir()
        manifest = TaskManifest.create("TASK-378", started_by="cli")
        manifest.task_id = "../TASK-378"

        result = ensure_worktree_ready(repo_root, manifest)

        assert result.outcome == ProvisioningOutcome.BLOCK
        assert "persisted task id '../TASK-378' is invalid" in (result.reason or "")
