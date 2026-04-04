"""Deterministic task worktree derivation, provisioning, and cleanup."""

from __future__ import annotations

import shutil
import subprocess
from dataclasses import dataclass
from enum import Enum
from pathlib import Path

from agent_pipeline.state import TaskManifest, validate_safe_task_id


@dataclass(frozen=True)
class WorktreeAssignment:
    """Deterministic worktree assignment for a task."""

    path: Path
    branch: str


class ProvisioningOutcome(Enum):
    """Provisioning result contract."""

    CREATE = "create"
    REUSE = "reuse"
    BLOCK = "block"


@dataclass(frozen=True)
class ProvisioningResult:
    """Provisioning decision and the deterministic assignment it refers to."""

    outcome: ProvisioningOutcome
    assignment: WorktreeAssignment
    reason: str | None = None


@dataclass(frozen=True)
class WorktreeInfo:
    """Parsed `git worktree list --porcelain` entry."""

    path: Path
    branch: str | None = None


def discover_repo_root(start: Path) -> Path:
    """Walk upward until a repository root is found."""
    current = start.resolve()
    if current.is_file():
        current = current.parent

    for candidate in (current, *current.parents):
        if (candidate / ".git").exists():
            return candidate
        if (candidate / "AGENTS.md").exists() and (candidate / "Cargo.toml").exists():
            return candidate

    raise FileNotFoundError(f"Could not determine repository root from {start}")


def derive_worktree_path(repo_root: Path, task_id: str) -> Path:
    """Derive the deterministic worktree path from the repository root."""
    return repo_root.resolve() / ".worktrees" / task_id


def derive_worktree_branch(task_id: str) -> str:
    """Derive the deterministic worktree branch name."""
    return f"agent-pipeline/{task_id}"


def derive_worktree_assignment(repo_root: Path, task_id: str) -> WorktreeAssignment:
    """Derive the deterministic worktree assignment for a task."""
    return WorktreeAssignment(
        path=derive_worktree_path(repo_root, task_id),
        branch=derive_worktree_branch(task_id),
    )


def _run_git(*args: str, cwd: Path) -> subprocess.CompletedProcess[str]:
    """Run a git command and capture output."""
    return subprocess.run(
        ["git", *args],
        cwd=cwd,
        check=False,
        capture_output=True,
        text=True,
    )


def ensure_worktree_ignored(
    repo_root: Path,
    *,
    run_git=_run_git,
) -> str | None:
    """Return a blocker reason unless `.worktrees/` is git-ignored."""
    result = run_git("check-ignore", ".worktrees/", cwd=repo_root)
    if result.returncode == 0:
        return None

    return "worktree provisioning blocked: repository must git-ignore .worktrees/ before provisioning"


def _list_worktrees(repo_root: Path, *, run_git=_run_git) -> dict[Path, WorktreeInfo]:
    """Return worktrees indexed by path."""
    result = run_git("worktree", "list", "--porcelain", cwd=repo_root)
    if result.returncode != 0:
        return {}

    worktrees: dict[Path, WorktreeInfo] = {}
    current_path: Path | None = None
    current_branch: str | None = None
    for line in result.stdout.splitlines():
        if not line:
            if current_path is not None:
                worktrees[current_path] = WorktreeInfo(path=current_path, branch=current_branch)
            current_path = None
            current_branch = None
            continue

        if line.startswith("worktree "):
            current_path = Path(line.removeprefix("worktree ")).resolve()
            continue

        if line.startswith("branch refs/heads/"):
            current_branch = line.removeprefix("branch refs/heads/")

    if current_path is not None:
        worktrees[current_path] = WorktreeInfo(path=current_path, branch=current_branch)

    return worktrees


def ensure_worktree_ready(
    repo_root: Path,
    manifest: TaskManifest,
    *,
    run_git=_run_git,
) -> ProvisioningResult:
    """Provision or reuse the deterministic worktree for a task, or block with a clear reason."""
    task_id_error = validate_safe_task_id(manifest.task_id)
    if task_id_error is not None:
        return ProvisioningResult(
            outcome=ProvisioningOutcome.BLOCK,
            assignment=derive_worktree_assignment(repo_root, "invalid-task-id"),
            reason=f"worktree provisioning blocked: persisted task id '{manifest.task_id}' is invalid: {task_id_error}",
        )

    assignment = derive_worktree_assignment(repo_root, manifest.task_id)

    metadata_error = manifest.validate_expected_worktree(path=assignment.path, branch=assignment.branch)
    if metadata_error is not None:
        return ProvisioningResult(
            outcome=ProvisioningOutcome.BLOCK,
            assignment=assignment,
            reason=metadata_error,
        )

    ignore_error = ensure_worktree_ignored(repo_root, run_git=run_git)
    if ignore_error is not None:
        return ProvisioningResult(
            outcome=ProvisioningOutcome.BLOCK,
            assignment=assignment,
            reason=ignore_error,
        )

    existing_worktrees = _list_worktrees(repo_root, run_git=run_git)
    existing = existing_worktrees.get(assignment.path.resolve())
    if existing is not None:
        if existing.branch != assignment.branch:
            return ProvisioningResult(
                outcome=ProvisioningOutcome.BLOCK,
                assignment=assignment,
                reason=(
                    "worktree provisioning blocked: existing worktree branch "
                    f"'{existing.branch}' does not match expected '{assignment.branch}'"
                ),
            )
        if assignment.path.exists():
            return ProvisioningResult(outcome=ProvisioningOutcome.REUSE, assignment=assignment)
        prune_result = run_git("worktree", "prune", cwd=repo_root)
        if prune_result.returncode != 0:
            stderr = prune_result.stderr.strip() or prune_result.stdout.strip()
            return ProvisioningResult(
                outcome=ProvisioningOutcome.BLOCK,
                assignment=assignment,
                reason=f"worktree provisioning blocked: {stderr or 'git worktree prune failed'}",
            )

    assignment.path.parent.mkdir(parents=True, exist_ok=True)
    branch_ref = f"refs/heads/{assignment.branch}"
    branch_exists = run_git("show-ref", "--verify", "--quiet", branch_ref, cwd=repo_root).returncode == 0
    if branch_exists:
        create_result = run_git(
            "worktree",
            "add",
            str(assignment.path),
            assignment.branch,
            cwd=repo_root,
        )
    else:
        create_result = run_git(
            "worktree",
            "add",
            "-b",
            assignment.branch,
            str(assignment.path),
            "HEAD",
            cwd=repo_root,
        )

    if create_result.returncode != 0:
        stderr = create_result.stderr.strip() or "git worktree add failed"
        return ProvisioningResult(
            outcome=ProvisioningOutcome.BLOCK,
            assignment=assignment,
            reason=f"worktree provisioning blocked: {stderr}",
        )

    return ProvisioningResult(outcome=ProvisioningOutcome.CREATE, assignment=assignment)


def remove_task_worktree(
    repo_root: Path,
    manifest: TaskManifest,
    *,
    run_git=_run_git,
) -> None:
    """Remove a finished task worktree and prune any stale metadata."""
    if manifest.worktree is None:
        raise FileNotFoundError(f"Task {manifest.task_id} has no tracked worktree metadata")

    repo_root = repo_root.resolve()
    expected_assignment = derive_worktree_assignment(repo_root, manifest.task_id)
    worktrees_root = (repo_root / ".worktrees").resolve()
    worktree_path = Path(manifest.worktree.path).resolve()
    try:
        worktree_path.relative_to(worktrees_root)
    except ValueError as exc:
        raise RuntimeError(
            f"worktree cleanup blocked: persisted worktree path '{worktree_path}' must stay under '{worktrees_root}'"
        ) from exc

    validation_error = manifest.validate_expected_worktree(
        path=expected_assignment.path,
        branch=expected_assignment.branch,
    )
    if validation_error is not None:
        raise RuntimeError(validation_error)

    if worktree_path != expected_assignment.path.resolve():
        raise RuntimeError(
            "worktree cleanup blocked: persisted worktree path "
            f"'{worktree_path}' does not match expected '{expected_assignment.path.resolve()}'"
        )

    result = run_git("worktree", "remove", "--force", str(worktree_path), cwd=repo_root)
    if result.returncode != 0:
        stderr = result.stderr.strip()
        stdout = result.stdout.strip()
        message = stderr or stdout
        if message and ("not a working tree" in message or "is not a working tree" in message):
            shutil.rmtree(worktree_path, ignore_errors=True)
        else:
            raise RuntimeError(message or f"Failed to remove worktree {worktree_path}")
    else:
        shutil.rmtree(worktree_path, ignore_errors=True)

    prune_result = run_git("worktree", "prune", cwd=repo_root)
    if prune_result.returncode != 0:
        stderr = prune_result.stderr.strip() or prune_result.stdout.strip()
        raise RuntimeError(stderr or "Failed to prune git worktree metadata")
