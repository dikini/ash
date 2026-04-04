#!/usr/bin/env python3
"""CLI interface for agent pipeline."""

from __future__ import annotations

import json
import os
import re
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import Iterable

import click
from click.core import ParameterSource

from agent_pipeline.agents import AgentSpawner
from agent_pipeline.events import EventLogger
from agent_pipeline.state import Stage, StateManager, TaskManifest
from agent_pipeline.supervisor import StageLogStream, Supervisor
from agent_pipeline.worktrees import remove_task_worktree


DEFAULT_BASE_DIR_NAME = ".agents"


def _discover_workspace_root(start: Path) -> Path | None:
    """Walk upward until a likely Ash workspace root is found."""
    current = start.resolve()
    if current.is_file():
        current = current.parent

    for candidate in (current, *current.parents):
        if (candidate / ".git").exists():
            return candidate
        if (candidate / "AGENTS.md").exists() and (candidate / "Cargo.toml").exists():
            return candidate

    return None


def resolve_workspace_root(configured_root: str | Path | None = None) -> Path:
    """Resolve the workspace root for CLI state operations."""
    if configured_root is None:
        configured_root = os.getenv("AGENT_PIPELINE_WORKSPACE_ROOT")
    if configured_root is None:
        context = click.get_current_context(silent=True)
        if context is not None and context.obj is not None:
            configured_root = context.obj.get("workspace_root")

    if configured_root is not None:
        return Path(configured_root).expanduser().resolve()

    module_root = _discover_workspace_root(Path(__file__))
    if module_root is not None:
        return module_root

    cwd_root = _discover_workspace_root(Path.cwd())
    if cwd_root is not None:
        return cwd_root

    return Path.cwd().resolve()


def resolve_base_dir(
    configured_base_dir: str | Path | None = None,
    workspace_root: str | Path | None = None,
) -> Path:
    """Resolve the state directory for CLI commands."""
    resolved_workspace_root = resolve_workspace_root(workspace_root)
    if configured_base_dir is not None:
        base_dir = Path(configured_base_dir).expanduser()
        if base_dir.is_absolute():
            return base_dir.resolve()
        return (resolved_workspace_root / base_dir).resolve()

    workspace_root_env = os.getenv("AGENT_PIPELINE_WORKSPACE_ROOT")
    if workspace_root is not None or workspace_root_env is not None:
        return resolved_workspace_root / DEFAULT_BASE_DIR_NAME

    configured_base_dir = os.getenv("AGENT_PIPELINE_BASE_DIR")
    if configured_base_dir is None:
        context = click.get_current_context(silent=True)
        if context is not None and context.obj is not None:
            configured_base_dir = context.obj.get("base_dir")
    if configured_base_dir is None:
        return resolved_workspace_root / DEFAULT_BASE_DIR_NAME

    base_dir = Path(configured_base_dir).expanduser()
    if base_dir.is_absolute():
        return base_dir.resolve()
    return (resolved_workspace_root / base_dir).resolve()


def resolve_stage_agents(configured_stage_agents: str | dict[str, str] | None = None) -> dict[str, str] | None:
    """Resolve runtime stage-agent overrides from explicit config or environment."""
    if configured_stage_agents is None:
        context = click.get_current_context(silent=True)
        if context is not None and context.obj is not None:
            configured_stage_agents = context.obj.get("stage_agents")
    if configured_stage_agents is None:
        configured_stage_agents = os.getenv("AGENT_PIPELINE_STAGE_AGENTS")

    if configured_stage_agents is None:
        return None

    if isinstance(configured_stage_agents, dict):
        parsed = configured_stage_agents
    else:
        try:
            parsed = json.loads(configured_stage_agents)
        except json.JSONDecodeError as exc:
            raise ValueError(
                "Invalid AGENT_PIPELINE_STAGE_AGENTS: expected JSON object mapping stage names to agent names"
            ) from exc

    if not isinstance(parsed, dict):
        raise ValueError(
            "Invalid AGENT_PIPELINE_STAGE_AGENTS: expected JSON object mapping stage names to agent names"
        )

    AgentSpawner.resolve_stage_agents(parsed)
    return {str(stage): str(agent) for stage, agent in parsed.items()}


def _load_dashboard_stage_agents(base_dir: Path) -> dict[str, str] | None:
    """Load the persisted runtime stage-agent mapping from the supervisor dashboard."""
    dashboard_path = base_dir / "status" / "dashboard.json"
    if not dashboard_path.exists():
        return None

    try:
        dashboard = json.loads(dashboard_path.read_text())
    except (OSError, json.JSONDecodeError):
        return None

    stage_agents = dashboard.get("stage_agents")
    if not isinstance(stage_agents, dict):
        return None

    if not all(isinstance(stage, str) and isinstance(agent, str) for stage, agent in stage_agents.items()):
        return None

    return stage_agents


def get_state_manager(ctx: click.Context | None = None) -> StateManager:
    """Get state manager with default directory."""
    return StateManager(get_base_dir(ctx))


def get_supervisor(ctx: click.Context | None = None) -> Supervisor:
    """Get supervisor with default directory."""
    stage_agents = None
    workspace_root = None
    if ctx is not None and ctx.obj is not None:
        if "stage_agents" in ctx.obj:
            stage_agents = ctx.obj["stage_agents"]
        if "workspace_root" in ctx.obj:
            workspace_root = ctx.obj["workspace_root"]
    else:
        stage_agents = resolve_stage_agents()
        workspace_root = resolve_workspace_root()

    return Supervisor(get_base_dir(ctx), workspace_root=workspace_root, stage_agents=stage_agents)


def get_base_dir(ctx: click.Context | None = None) -> Path:
    """Get the resolved state directory from the Click context."""
    if ctx is not None and ctx.obj is not None and "base_dir" in ctx.obj:
        return ctx.obj["base_dir"]
    return resolve_base_dir()


def _list_tasks(state_mgr: StateManager, subdir: str) -> list[TaskManifest]:
    """List task manifests for a lifecycle directory."""
    tasks_dir = state_mgr.base_dir / subdir
    if not tasks_dir.exists():
        return []

    tasks: list[TaskManifest] = []
    for task_dir in sorted(tasks_dir.iterdir()):
        manifest_path = task_dir / "manifest.json"
        if task_dir.is_dir() and manifest_path.exists():
            tasks.append(state_mgr.load_task_from_path(manifest_path))
    return tasks


@click.group()
@click.option(
    "--workspace-root",
    type=click.Path(path_type=Path, file_okay=False, dir_okay=True),
    envvar="AGENT_PIPELINE_WORKSPACE_ROOT",
    help="Workspace root used to resolve the default state directory.",
)
@click.option(
    "--base-dir",
    type=click.Path(path_type=Path, file_okay=False, dir_okay=True),
    envvar="AGENT_PIPELINE_BASE_DIR",
    help="State directory for task bundles. Defaults to <workspace-root>/.agents.",
)
@click.option(
    "--stage-agents",
    envvar="AGENT_PIPELINE_STAGE_AGENTS",
    help="JSON object mapping stage names to agent names, e.g. '{\"qa\":\"codex\"}'.",
)
@click.version_option(version="0.1.0")
@click.pass_context
def cli(ctx: click.Context, workspace_root: Path | None, base_dir: Path | None, stage_agents: str | None):
    """Agent Pipeline - Multi-agent task orchestrator for Ash."""
    workspace_root_source = ctx.get_parameter_source("workspace_root")
    base_dir_source = ctx.get_parameter_source("base_dir")
    if (
        base_dir_source == ParameterSource.ENVIRONMENT
        and workspace_root_source in {ParameterSource.COMMANDLINE, ParameterSource.ENVIRONMENT}
    ):
        base_dir = None

    resolved_workspace_root = resolve_workspace_root(workspace_root)
    resolved_base_dir = resolve_base_dir(base_dir, resolved_workspace_root)
    try:
        resolved_stage_agents = resolve_stage_agents(stage_agents)
    except ValueError as exc:
        raise click.BadParameter(str(exc), param_hint="--stage-agents") from exc
    ctx.ensure_object(dict)
    ctx.obj["workspace_root"] = resolved_workspace_root
    ctx.obj["workspace_root_explicit"] = workspace_root_source in {ParameterSource.COMMANDLINE, ParameterSource.ENVIRONMENT}
    ctx.obj["base_dir"] = resolved_base_dir
    ctx.obj["stage_agents"] = resolved_stage_agents


@cli.command()
@click.argument("task_id")
@click.option("--notify", help="Notification target (discord:user, telegram:id)")
@click.option("--from-spec", help="Spec file to read task from")
@click.option("--depends-on", "dependencies", multiple=True, help="Prerequisite task id. Repeat for multiple dependencies.")
@click.pass_context
def queue(ctx: click.Context, task_id: str, notify: str | None, from_spec: str | None, dependencies: tuple[str, ...]):
    """Queue a new task for processing."""
    state_mgr = get_state_manager(ctx)

    try:
        _validate_safe_task_id(task_id, context="Task id")
    except ValueError as exc:
        click.echo(str(exc), err=True)
        sys.exit(1)
    
    started_by = notify if notify else "cli"
    
    # Check if task already exists
    try:
        existing_state, _ = state_mgr.find_task(task_id)
        click.echo(f"Error: Task {task_id} already exists in {existing_state}", err=True)
        sys.exit(1)
    except FileNotFoundError:
        pass

    try:
        normalized_dependencies = _normalize_dependencies(task_id, dependencies)
        _validate_dependency_graph(state_mgr, task_id, normalized_dependencies)
    except ValueError as exc:
        click.echo(str(exc), err=True)
        sys.exit(1)

    spec_contents: str | None = None
    if from_spec:
        spec_path = Path(from_spec).expanduser()
        if not spec_path.is_absolute():
            spec_path = resolve_workspace_root(ctx.obj.get("workspace_root") if ctx.obj else None) / spec_path
        if not spec_path.exists():
            click.echo(f"Error: Spec file {from_spec} not found", err=True)
            sys.exit(1)
        spec_contents = spec_path.read_text()
    
    # Create task
    state_mgr.create_task(task_id, started_by=started_by, dependencies=normalized_dependencies)
    
    # Copy spec if provided
    if spec_contents is not None:
        task_dir = state_mgr.base_dir / "queue" / task_id
        task_dir.mkdir(parents=True, exist_ok=True)
        (task_dir / "task.md").write_text(spec_contents)
    
    click.echo(f"Queued {task_id} (started by: {started_by})")


@cli.command()
@click.option("--task", help="Show specific task")
@click.option("--watch", is_flag=True, help="Watch for updates")
@click.option("--format", "output_format", default="text", type=click.Choice(["text", "json"]))
@click.pass_context
def status(ctx: click.Context, task: str | None, watch: bool, output_format: str):
    """Show pipeline status."""
    if task is not None:
        try:
            _validate_safe_task_id(task)
        except ValueError as exc:
            click.echo(str(exc), err=True)
            sys.exit(1)

    state_mgr = get_state_manager(ctx)
    base_dir = get_base_dir(ctx)
    resolved_stage_agent_enums = AgentSpawner.resolve_stage_agents(
        ctx.obj.get("stage_agents") if ctx and ctx.obj else None
    )
    configured_stage_agents = {
        stage.value: agent.value
        for stage, agent in resolved_stage_agent_enums.items()
    }
    stage_agents = _load_dashboard_stage_agents(base_dir) or configured_stage_agents

    if watch:
        # Watch mode - continuous updates
        try:
            while True:
                click.clear()
                _print_status(state_mgr, task, output_format, stage_agents)
                time.sleep(2)
        except KeyboardInterrupt:
            pass
    else:
        _print_status(state_mgr, task, output_format, stage_agents)


def _get_unmet_dependencies(state_mgr: StateManager, manifest: TaskManifest) -> list[str]:
    """Return unresolved prerequisite task ids for a manifest."""
    unmet: list[str] = []
    for dependency in manifest.dependencies:
        try:
            state, dependency_manifest = state_mgr.find_task(dependency)
        except FileNotFoundError:
            unmet.append(dependency)
            continue

        if state != "done" or dependency_manifest.status.value != "complete":
            unmet.append(dependency)

    return unmet


def _feedback_resolution_metadata(task_dir: Path) -> tuple[bool, str | None]:
    """Return whether structured feedback resolution exists and which review artifact it references."""
    resolution_path = task_dir / "feedback-resolution.md"
    if not resolution_path.exists():
        return False, None

    text = resolution_path.read_text(encoding="utf-8")
    for line in text.splitlines():
        if line.startswith("Source review artifact:"):
            return True, line.split(":", 1)[1].strip() or None

    return True, None


SAFE_TASK_ID_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]*$")


def _validate_safe_task_id(task_id: str, *, context: str = "Task id") -> None:
    """Reject task identifiers that could escape lifecycle directories."""
    if not SAFE_TASK_ID_RE.fullmatch(task_id):
        raise ValueError(
            f"{context} must use only letters, numbers, dots, underscores, and hyphens, and may not contain path separators"
        )


def _normalize_dependencies(task_id: str, dependencies: Iterable[str]) -> list[str]:
    """Trim, deduplicate, and validate dependency ids for a queued task."""
    normalized: list[str] = []
    seen: set[str] = set()
    for dependency in dependencies:
        cleaned = dependency.strip()
        if not cleaned or cleaned in seen:
            continue
        _validate_safe_task_id(cleaned, context="Dependency task id")
        if cleaned == task_id:
            raise ValueError(f"Task {task_id} cannot depend on itself")
        normalized.append(cleaned)
        seen.add(cleaned)
    return normalized


def _task_dependency_closes_cycle(
    state_mgr: StateManager,
    task_id: str,
    dependency: str,
    visited: set[str] | None = None,
) -> bool:
    """Return True when adding task_id -> dependency would create a cycle."""
    if dependency == task_id:
        return True
    if visited is None:
        visited = set()
    if dependency in visited:
        return False
    visited.add(dependency)

    try:
        _, dependency_manifest = state_mgr.find_task(dependency)
    except FileNotFoundError:
        return False

    return any(
        _task_dependency_closes_cycle(state_mgr, task_id, child, visited.copy())
        for child in dependency_manifest.dependencies
    )


def _validate_dependency_graph(state_mgr: StateManager, task_id: str, dependencies: list[str]) -> None:
    """Reject new dependency edges that would create cycles against existing manifests."""
    for dependency in dependencies:
        if _task_dependency_closes_cycle(state_mgr, task_id, dependency):
            raise ValueError(f"Adding dependency {dependency} to {task_id} would create a dependency cycle")


def _resolve_task_relative_path(task_dir: Path, relative_path: str) -> Path:
    """Resolve a task-bundle relative path and reject traversal outside the bundle."""
    candidate = (task_dir / relative_path).resolve()
    task_root = task_dir.resolve()
    if candidate != task_root and task_root not in candidate.parents:
        raise ValueError("Review artifact path must stay within the task bundle")
    return candidate


RETRY_STAGE_BY_REVIEW_ARTIFACT = {
    "spec.review": Stage.SPEC_WRITE,
    "plan.review": Stage.PLAN_WRITE,
    "qa.review": Stage.IMPL,
    "validate.review": Stage.IMPL,
}


def _stage_cleanup_paths(task_dir: Path, stage: Stage) -> list[Path]:
    """Return artifact/runtime files that should be cleared for a stage retry."""
    artifact_paths: dict[Stage, list[str]] = {
        Stage.DESIGN: ["design.md"],
        Stage.SPEC_WRITE: ["spec.md"],
        Stage.SPEC_VERIFY: ["spec.verified", "spec.review"],
        Stage.PLAN_WRITE: ["plan.md"],
        Stage.PLAN_VERIFY: ["plan.verified", "plan.review"],
        Stage.IMPL: ["impl.complete", "impl.summary.md", "impl.verification.md"],
        Stage.QA: ["qa.md", "qa.verified", "qa.review"],
        Stage.VALIDATE: ["validated", "validate.review"],
    }
    runtime_paths = [
        f"{stage.value}.stdout.log",
        f"{stage.value}.stderr.log",
        f"{stage.value}.pid",
        f"{stage.value}_prompt.txt",
    ]
    return [task_dir / relative_path for relative_path in [*artifact_paths.get(stage, []), *runtime_paths]]


def _retry_stages_from(start_stage: Stage) -> list[Stage]:
    """Return the restart stage and all downstream stages."""
    stages = list(Stage)
    start_index = stages.index(start_stage)
    return stages[start_index:]


def _review_artifact_basename(review_artifact: str) -> str:
    """Return the canonical review artifact filename from a possibly archived relative path."""
    return Path(review_artifact).name


def _resolve_supported_retry_stage(review_artifact: str) -> Stage:
    """Resolve the restart stage for a supported review artifact path or raise a user-facing error."""
    retry_stage = RETRY_STAGE_BY_REVIEW_ARTIFACT.get(_review_artifact_basename(review_artifact))
    if retry_stage is None:
        raise ValueError(f"Unsupported feedback review artifact for retry: {review_artifact}")
    return retry_stage


def _rewrite_feedback_resolution_review_artifact(
    resolution_path: Path,
    old_review_artifact: str,
    archived_review_artifact: str,
) -> None:
    """Rewrite feedback-resolution metadata to point at the archived review artifact path."""
    text = resolution_path.read_text(encoding="utf-8")
    resolution_path.write_text(
        text.replace(
            f"Source review artifact: {old_review_artifact}",
            f"Source review artifact: {archived_review_artifact}",
            1,
        ),
        encoding="utf-8",
    )


def _worktree_fields(manifest: TaskManifest) -> dict[str, str | None]:
    """Return worktree metadata fields for status surfaces."""
    return {
        "worktree_path": manifest.worktree.path if manifest.worktree is not None else None,
        "worktree_branch": manifest.worktree.branch if manifest.worktree is not None else None,
    }


def _print_status(
    state_mgr: StateManager,
    task_id: str | None,
    output_format: str,
    stage_agents: dict[str, str],
):
    """Print status output."""
    if task_id:
        # Single task
        try:
            _, manifest = state_mgr.find_task(task_id)
            _print_task_status(state_mgr, manifest, output_format)
        except FileNotFoundError:
            click.echo(f"Task {task_id} not found", err=True)
            sys.exit(1)
    else:
        # All tasks
        queue_tasks = state_mgr.list_queue()
        in_progress = state_mgr.list_in_progress()
        blocked = _list_tasks(state_mgr, "blocked")
        done = _list_tasks(state_mgr, "done")
        
        if output_format == "json":
            output = {
                "stage_agents": stage_agents,
                "queue": [_task_to_dict(state_mgr, manifest, state="queue", bundle_id=bundle_id) for bundle_id, manifest in state_mgr.list_subdir_entries("queue")],
                "in_progress": [_task_to_dict(state_mgr, manifest, state="in-progress", bundle_id=bundle_id) for bundle_id, manifest in state_mgr.list_subdir_entries("in-progress")],
                "blocked": [_task_to_dict(state_mgr, manifest, state="blocked", bundle_id=bundle_id) for bundle_id, manifest in state_mgr.list_subdir_entries("blocked")],
                "done": [_task_to_dict(state_mgr, manifest, state="done", bundle_id=bundle_id) for bundle_id, manifest in state_mgr.list_subdir_entries("done")],
            }
            click.echo(json.dumps(output, indent=2))
            return

        click.echo("=" * 50)
        click.echo("AGENT PIPELINE STATUS")
        click.echo("=" * 50)

        click.echo(f"\nQueue ({len(queue_tasks)} tasks):")
        for bundle_id, t in state_mgr.list_subdir_entries("queue"):
            unmet = _get_unmet_dependencies(state_mgr, t)
            worktree = f" [worktree: {t.worktree.path} @ {t.worktree.branch}]" if t.worktree is not None else ""
            metadata_error = f" [worktree error: {t.worktree_block_reason()}]" if t.worktree_metadata_state().value == "invalid" and t.worktree_block_reason() else ""
            if unmet:
                click.echo(f"  {t.task_id} (by: {t.started_by}) waiting on: {', '.join(unmet)}{worktree}{metadata_error}")
            else:
                click.echo(f"  {t.task_id} (by: {t.started_by}){worktree}{metadata_error}")

        click.echo(f"\nIn Progress ({len(in_progress)} tasks):")
        for bundle_id, t in state_mgr.list_subdir_entries("in-progress"):
            attempt = t.attempts[t.current_stage]
            status_icon = "🟡" if attempt == 0 else "🔁"
            worktree = f" [worktree: {t.worktree.path} @ {t.worktree.branch}]" if t.worktree is not None else ""
            metadata_error = f" [worktree error: {t.worktree_block_reason()}]" if t.worktree_metadata_state().value == "invalid" and t.worktree_block_reason() else ""
            click.echo(f"  {status_icon} {t.task_id}: {t.current_stage.value} (attempt {attempt}/5){worktree}{metadata_error}")

        click.echo(f"\nBlocked ({len(blocked)} tasks):")
        for bundle_id, manifest in state_mgr.list_subdir_entries("blocked"):
            blocker = manifest.blockers[-1] if manifest.blockers else "unknown"
            worktree = f" [worktree: {manifest.worktree.path} @ {manifest.worktree.branch}]" if manifest.worktree is not None else ""
            metadata_error = f" [worktree error: {manifest.worktree_block_reason()}]" if manifest.worktree_metadata_state().value == "invalid" and manifest.worktree_block_reason() else ""
            click.echo(f"  🔴 {manifest.task_id}: {blocker}{worktree}{metadata_error}")

        click.echo(f"\nCompleted ({len(done)} tasks):")
        for bundle_id, manifest in state_mgr.list_subdir_entries("done"):
            worktree = f" [worktree: {manifest.worktree.path} @ {manifest.worktree.branch}]" if manifest.worktree is not None else ""
            metadata_error = f" [worktree error: {manifest.worktree_block_reason()}]" if manifest.worktree_metadata_state().value == "invalid" and manifest.worktree_block_reason() else ""
            click.echo(f"  ✅ {manifest.task_id}: complete{worktree}{metadata_error}")


def _task_to_dict(
    state_mgr: StateManager,
    manifest: TaskManifest,
    *,
    state: str | None = None,
    bundle_id: str | None = None,
) -> dict:
    """Convert manifest to dict."""
    unmet_dependencies = _get_unmet_dependencies(state_mgr, manifest)
    if state is None or bundle_id is None:
        try:
            state, manifest_path = state_mgr.find_task_path(manifest.task_id)
            bundle_id = manifest_path.parent.name
        except FileNotFoundError:
            state = None
            bundle_id = None
    has_feedback_resolution = False
    feedback_review_artifact = None
    if state is not None and bundle_id is not None:
        task_dir = state_mgr.base_dir / state / bundle_id
        has_feedback_resolution, feedback_review_artifact = _feedback_resolution_metadata(task_dir)
    return {
        "task_id": manifest.task_id,
        "stage": manifest.current_stage.value,
        "status": manifest.status.value,
        "attempts": {k.value: v for k, v in manifest.attempts.items()},
        "dependencies": manifest.dependencies,
        "unmet_dependencies": unmet_dependencies,
        "waiting_on_dependencies": bool(unmet_dependencies),
        "has_feedback_resolution": has_feedback_resolution,
        "feedback_review_artifact": feedback_review_artifact,
        "started_by": manifest.started_by,
        "worktree_metadata_state": manifest.worktree_metadata_state().value,
        "worktree_error": manifest.worktree_block_reason(),
        **_worktree_fields(manifest),
    }


def _print_task_status(state_mgr: StateManager, manifest: TaskManifest, output_format: str):
    """Print single task status."""
    task_payload = _task_to_dict(state_mgr, manifest)
    unmet_dependencies = task_payload["unmet_dependencies"]
    if output_format == "json":
        click.echo(json.dumps(task_payload, indent=2))
    else:
        click.echo(f"Task: {manifest.task_id}")
        click.echo(f"Stage: {manifest.current_stage.value}")
        click.echo(f"Status: {manifest.status.value}")
        click.echo(f"Attempts: {manifest.attempts[manifest.current_stage]}/{manifest.max_attempts}")
        click.echo(f"Started by: {manifest.started_by}")
        click.echo(f"Feedback resolution: {'yes' if task_payload['has_feedback_resolution'] else 'no'}")
        if task_payload["feedback_review_artifact"]:
            click.echo(f"Feedback review artifact: {task_payload['feedback_review_artifact']}")
        if task_payload["worktree_path"]:
            click.echo(f"Worktree: {task_payload['worktree_path']}")
        if task_payload["worktree_branch"]:
            click.echo(f"Worktree branch: {task_payload['worktree_branch']}")
        if task_payload.get("worktree_metadata_state") == "invalid" and task_payload.get("worktree_error"):
            click.echo(f"Worktree metadata error: {task_payload['worktree_error']}")
        if manifest.dependencies:
            click.echo(f"Dependencies: {', '.join(manifest.dependencies)}")
        if unmet_dependencies:
            click.echo(f"Waiting on dependencies: {', '.join(unmet_dependencies)}")
        if manifest.blockers:
            click.echo(f"Blockers: {', '.join(manifest.blockers)}")


@cli.command()
@click.argument("task_id")
@click.pass_context
def pause(ctx: click.Context, task_id: str):
    """Pause a running task."""
    try:
        _validate_safe_task_id(task_id)
    except ValueError as exc:
        click.echo(str(exc), err=True)
        sys.exit(1)
    _send_control_command(get_base_dir(ctx), task_id, "pause")
    click.echo(f"Pause requested for {task_id}")


@cli.command()
@click.argument("task_id")
@click.pass_context
def resume(ctx: click.Context, task_id: str):
    """Resume a paused task."""
    try:
        _validate_safe_task_id(task_id)
    except ValueError as exc:
        click.echo(str(exc), err=True)
        sys.exit(1)
    _send_control_command(get_base_dir(ctx), task_id, "resume")
    click.echo(f"Resume requested for {task_id}")


@cli.command()
@click.argument("task_id")
@click.pass_context
def abort(ctx: click.Context, task_id: str):
    """Abort a running task."""
    try:
        _validate_safe_task_id(task_id)
    except ValueError as exc:
        click.echo(str(exc), err=True)
        sys.exit(1)
    _send_control_command(get_base_dir(ctx), task_id, "abort")
    click.echo(f"Abort requested for {task_id}")


@cli.command()
@click.argument("task_id")
@click.option("--message", required=True, help="Steering message")
@click.pass_context
def steer(ctx: click.Context, task_id: str, message: str):
    """Send steering message to a task."""
    try:
        _validate_safe_task_id(task_id)
    except ValueError as exc:
        click.echo(str(exc), err=True)
        sys.exit(1)
    _send_control_command(get_base_dir(ctx), task_id, "steer", message=message)
    click.echo(f"Steering message sent to {task_id}")


@cli.command("resolve-feedback")
@click.argument("task_id")
@click.option("--review-artifact", required=True, help="Review artifact being addressed, e.g. spec.review")
@click.option("--summary", required=True, help="Short summary of the retry guidance")
@click.option("--changes", required=True, help="Concrete required changes")
@click.option("--success-condition", required=True, help="Condition that means the retry addressed the feedback")
@click.pass_context
def resolve_feedback(
    ctx: click.Context,
    task_id: str,
    review_artifact: str,
    summary: str,
    changes: str,
    success_condition: str,
):
    """Persist structured review-resolution guidance for a task."""
    try:
        _validate_safe_task_id(task_id)
    except ValueError as exc:
        click.echo(str(exc), err=True)
        sys.exit(1)
    state_mgr = get_state_manager(ctx)
    try:
        state, _ = state_mgr.find_task(task_id)
    except FileNotFoundError:
        click.echo(f"Task {task_id} not found", err=True)
        sys.exit(1)

    task_dir = state_mgr.base_dir / state / task_id
    try:
        review_path = _resolve_task_relative_path(task_dir, review_artifact)
        _resolve_supported_retry_stage(review_artifact)
    except ValueError as exc:
        click.echo(str(exc), err=True)
        sys.exit(1)
    if not review_path.exists():
        click.echo(f"Review artifact {review_artifact} not found for {task_id}", err=True)
        sys.exit(1)

    resolution_path = task_dir / "feedback-resolution.md"
    resolution_path.write_text(
        "# Feedback Resolution\n\n"
        f"Source review artifact: {review_artifact}\n\n"
        "## Summary\n"
        f"{summary}\n\n"
        "## Required Changes\n"
        f"{changes}\n\n"
        "## Success Condition\n"
        f"{success_condition}\n",
        encoding="utf-8",
    )
    click.echo(f"Feedback resolution saved for {task_id}")


@cli.command("retry-feedback")
@click.argument("task_id")
@click.option("--to", "destination", type=click.Choice(["queue", "in-progress"]), default="queue", show_default=True)
@click.pass_context
def retry_feedback(ctx: click.Context, task_id: str, destination: str):
    """Release a blocked feedback-resolved task back to queue or in-progress."""
    try:
        _validate_safe_task_id(task_id)
    except ValueError as exc:
        click.echo(str(exc), err=True)
        sys.exit(1)
    state_mgr = get_state_manager(ctx)
    try:
        state, manifest = state_mgr.find_task(task_id)
    except FileNotFoundError:
        click.echo(f"Task {task_id} not found", err=True)
        sys.exit(1)

    if state != "blocked":
        click.echo(f"Task {task_id} is in {state}, not blocked", err=True)
        sys.exit(1)

    task_dir = state_mgr.base_dir / state / task_id
    has_feedback_resolution, review_artifact = _feedback_resolution_metadata(task_dir)
    if not has_feedback_resolution:
        click.echo(f"Task {task_id} has no feedback-resolution.md", err=True)
        sys.exit(1)
    if not review_artifact:
        click.echo(f"Task {task_id} feedback-resolution.md does not name a source review artifact", err=True)
        sys.exit(1)

    try:
        configured_review_path = _resolve_task_relative_path(task_dir, review_artifact)
        retry_stage = _resolve_supported_retry_stage(review_artifact)
    except ValueError as exc:
        click.echo(str(exc), err=True)
        sys.exit(1)

    unmet_dependencies = _get_unmet_dependencies(state_mgr, manifest)
    if destination == "in-progress" and unmet_dependencies:
        click.echo(
            f"Cannot move {task_id} directly to in-progress while dependencies are unmet: {', '.join(unmet_dependencies)}",
            err=True,
        )
        sys.exit(1)

    resolution_path = task_dir / "feedback-resolution.md"
    current_review_path = task_dir / _review_artifact_basename(review_artifact)
    review_path = current_review_path if current_review_path.exists() else configured_review_path
    if not review_path.exists():
        click.echo(f"Review artifact {review_artifact} not found for {task_id}", err=True)
        sys.exit(1)

    archive_dir = task_dir / "retry-history" / datetime.now().strftime("%Y%m%dT%H%M%S")
    archive_dir.mkdir(parents=True, exist_ok=True)
    archived_review_path = archive_dir / review_path.name
    if archived_review_path.exists():
        archived_review_path = archive_dir / f"{datetime.now().strftime('%H%M%S%f')}-{review_path.name}"
    review_path.replace(archived_review_path)
    archived_review_artifact = archived_review_path.relative_to(task_dir).as_posix()
    _rewrite_feedback_resolution_review_artifact(resolution_path, review_artifact, archived_review_artifact)

    for stage in _retry_stages_from(retry_stage):
        manifest.attempts[stage] = 0
        for path in _stage_cleanup_paths(task_dir, stage):
            if path == archived_review_path:
                continue
            path.unlink(missing_ok=True)

    manifest.current_stage = retry_stage
    manifest.status = manifest.status.IN_PROGRESS
    manifest.blockers = []
    manifest.updated_at = datetime.now().isoformat()
    state_mgr.save_task(manifest, subdir="blocked")
    state_mgr.move_task(task_id, "blocked", destination)
    click.echo(f"Retried {task_id} via {destination} at stage {retry_stage.value}")


@cli.command()
@click.argument("task_id")
@click.option("--stage", "stage_name", type=click.Choice([stage.value for stage in Stage]))
@click.option("--stream", default="stdout", type=click.Choice([stream.value for stream in StageLogStream]))
@click.option("--tail", type=int, help="Show only the last N lines")
@click.option("--follow", is_flag=True, help="Follow the log until it goes idle")
@click.pass_context
def logs(ctx: click.Context, task_id: str, stage_name: str | None, stream: str, tail: int | None, follow: bool):
    """Show live or persisted stdout/stderr for a task stage."""
    try:
        _validate_safe_task_id(task_id)
    except ValueError as exc:
        click.echo(str(exc), err=True)
        sys.exit(1)
    supervisor = get_supervisor(ctx)
    resolved_stage = Stage(stage_name) if stage_name is not None else None
    resolved_stream = StageLogStream(stream)

    try:
        if follow:
            for chunk in supervisor.follow_stage_log(
                task_id,
                stage=resolved_stage,
                stream=resolved_stream,
                tail=tail,
            ):
                click.echo(chunk, nl=False)
            return

        click.echo(
            supervisor.read_stage_log(task_id, stage=resolved_stage, stream=resolved_stream, tail=tail),
            nl=False,
        )
    except FileNotFoundError as exc:
        click.echo(str(exc), err=True)
        sys.exit(1)


def _send_control_command(base_dir: Path, task_id: str, command: str, **kwargs):
    """Send control command to supervisor."""
    _validate_safe_task_id(task_id)
    control_dir = base_dir / "control"
    control_dir.mkdir(parents=True, exist_ok=True)
    
    control_file = control_dir / f"{command}_{task_id}.json"
    data = {
        "command": command,
        "task_id": task_id,
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ"),
        **kwargs
    }
    control_file.write_text(json.dumps(data))


@cli.command()
@click.option("--interval", default=1.0, help="Polling interval in seconds")
@click.pass_context
def daemon(ctx: click.Context, interval: float):
    """Run supervisor daemon."""
    base_dir = get_base_dir(ctx)
    click.echo("Starting agent pipeline supervisor...")
    click.echo(f"Base directory: {base_dir}")
    click.echo(f"Polling interval: {interval}s")
    click.echo("Press Ctrl+C to stop")
    
    supervisor = get_supervisor(ctx)
    
    try:
        supervisor.run(interval=interval)
    except KeyboardInterrupt:
        click.echo("\nStopping supervisor...")


def _derive_cleanup_repo_root_from_worktree_path(task_id: str, worktree_path: str) -> Path:
    """Derive the repo root from persisted worktree metadata for base-dir-only cleanup."""
    resolved = Path(worktree_path).resolve()
    if resolved.parent.name != ".worktrees" or resolved.name != task_id:
        raise RuntimeError(
            "worktree cleanup blocked: persisted worktree path "
            f"'{resolved}' does not match expected '<repo-root>/.worktrees/{task_id}' structure"
        )
    return resolved.parents[1]


@cli.command("cleanup-worktree")
@click.argument("task_id")
@click.pass_context
def cleanup_worktree(ctx: click.Context, task_id: str):
    """Remove a finished task's provisioned worktree and clear persisted metadata."""
    try:
        _validate_safe_task_id(task_id)
    except ValueError as exc:
        click.echo(str(exc), err=True)
        sys.exit(1)

    state_mgr = get_state_manager(ctx)
    try:
        state, manifest = state_mgr.find_task(task_id)
    except FileNotFoundError:
        click.echo(f"Task {task_id} not found", err=True)
        sys.exit(1)

    if state in {"queue", "in-progress"}:
        click.echo(f"Refusing to clean up worktree for {task_id} while task is in {state}", err=True)
        sys.exit(1)

    if manifest.worktree_metadata_state().value == "invalid":
        click.echo(manifest.worktree_block_reason() or f"Task {task_id} has invalid tracked worktree metadata", err=True)
        sys.exit(1)

    if manifest.worktree is None:
        click.echo(f"Task {task_id} has no tracked worktree metadata", err=True)
        sys.exit(1)

    worktree_path = manifest.worktree.path
    configured_workspace_root = ctx.obj.get("workspace_root") if ctx.obj else None
    workspace_root_explicit = bool(ctx.obj.get("workspace_root_explicit")) if ctx.obj else False
    workspace_root = resolve_workspace_root(configured_workspace_root)
    try:
        repo_root = (
            workspace_root
            if workspace_root_explicit
            else _derive_cleanup_repo_root_from_worktree_path(task_id, manifest.worktree.path)
        )
        remove_task_worktree(repo_root, manifest)
    except (FileNotFoundError, RuntimeError) as exc:
        if "prune" in str(exc).lower() and not Path(worktree_path).exists():
            manifest.clear_worktree()
            state_mgr.save_task(manifest, subdir=state)
        click.echo(str(exc), err=True)
        sys.exit(1)

    manifest.clear_worktree()
    state_mgr.save_task(manifest, subdir=state)
    click.echo(f"Cleaned up worktree for {task_id}: {worktree_path}")


@cli.command()
@click.argument("task_id")
@click.pass_context
def events(ctx: click.Context, task_id: str):
    """Show event history for a task."""
    try:
        _validate_safe_task_id(task_id)
    except ValueError as exc:
        click.echo(str(exc), err=True)
        sys.exit(1)
    event_logger = EventLogger(get_base_dir(ctx) / "events")
    events = event_logger.get_event_history(task_id)
    
    if not events:
        click.echo(f"No events for {task_id}")
        return
    
    for event in events:
        click.echo(f"{event.timestamp} | {event.event_type}")
        for key, value in event.data.items():
            click.echo(f"  {key}: {value}")


def main():
    """Entry point."""
    cli()


if __name__ == "__main__":
    main()
