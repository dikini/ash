#!/usr/bin/env python3
"""CLI interface for agent pipeline."""

from __future__ import annotations

import json
import os
import sys
import time
from pathlib import Path

import click

from agent_pipeline.agents import AgentSpawner
from agent_pipeline.state import StateManager, TaskManifest
from agent_pipeline.supervisor import Supervisor
from agent_pipeline.events import EventLogger


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
        context = click.get_current_context(silent=True)
        if context is not None and context.obj is not None:
            configured_root = context.obj.get("workspace_root")
    if configured_root is None:
        configured_root = os.getenv("AGENT_PIPELINE_WORKSPACE_ROOT")

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
    if configured_base_dir is None:
        context = click.get_current_context(silent=True)
        if context is not None and context.obj is not None:
            configured_base_dir = context.obj.get("base_dir")
    if configured_base_dir is None:
        configured_base_dir = os.getenv("AGENT_PIPELINE_BASE_DIR")

    resolved_workspace_root = resolve_workspace_root(workspace_root)
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
    if ctx is not None and ctx.obj is not None and "stage_agents" in ctx.obj:
        stage_agents = ctx.obj["stage_agents"]
    else:
        stage_agents = resolve_stage_agents()

    return Supervisor(get_base_dir(ctx), stage_agents=stage_agents)


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
    resolved_workspace_root = resolve_workspace_root(workspace_root)
    resolved_base_dir = resolve_base_dir(base_dir, resolved_workspace_root)
    try:
        resolved_stage_agents = resolve_stage_agents(stage_agents)
    except ValueError as exc:
        raise click.BadParameter(str(exc), param_hint="--stage-agents") from exc
    ctx.ensure_object(dict)
    ctx.obj["workspace_root"] = resolved_workspace_root
    ctx.obj["base_dir"] = resolved_base_dir
    ctx.obj["stage_agents"] = resolved_stage_agents


@cli.command()
@click.argument("task_id")
@click.option("--notify", help="Notification target (discord:user, telegram:id)")
@click.option("--from-spec", help="Spec file to read task from")
@click.pass_context
def queue(ctx: click.Context, task_id: str, notify: str | None, from_spec: str | None):
    """Queue a new task for processing."""
    state_mgr = get_state_manager(ctx)
    
    started_by = notify if notify else "cli"
    
    # Check if task already exists
    try:
        existing_state, _ = state_mgr.find_task(task_id)
        click.echo(f"Error: Task {task_id} already exists in {existing_state}", err=True)
        sys.exit(1)
    except FileNotFoundError:
        pass

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
    state_mgr.create_task(task_id, started_by=started_by)
    
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
            _print_task_status(manifest, output_format)
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
                "queue": [_task_to_dict(t) for t in queue_tasks],
                "in_progress": [_task_to_dict(t) for t in in_progress],
                "blocked": [_task_to_dict(t) for t in blocked],
                "done": [_task_to_dict(t) for t in done],
            }
            click.echo(json.dumps(output, indent=2))
        else:
            click.echo("=" * 50)
            click.echo("AGENT PIPELINE STATUS")
            click.echo("=" * 50)
            
            click.echo(f"\nQueue ({len(queue_tasks)} tasks):")
            for t in queue_tasks:
                click.echo(f"  {t.task_id} (by: {t.started_by})")
            
            click.echo(f"\nIn Progress ({len(in_progress)} tasks):")
            for t in in_progress:
                attempt = t.attempts[t.current_stage]
                status_icon = "🟡" if attempt == 0 else "🔁"
                click.echo(f"  {status_icon} {t.task_id}: {t.current_stage.value} (attempt {attempt}/5)")
            
            click.echo(f"\nBlocked ({len(blocked)} tasks):")
            for manifest in blocked:
                blocker = manifest.blockers[-1] if manifest.blockers else "unknown"
                click.echo(f"  🔴 {manifest.task_id}: {blocker}")

            click.echo(f"\nCompleted ({len(done)} tasks):")
            for manifest in done:
                click.echo(f"  ✅ {manifest.task_id}: complete")


def _task_to_dict(manifest: TaskManifest) -> dict:
    """Convert manifest to dict."""
    return {
        "task_id": manifest.task_id,
        "stage": manifest.current_stage.value,
        "status": manifest.status.value,
        "attempts": {k.value: v for k, v in manifest.attempts.items()},
        "started_by": manifest.started_by,
    }


def _print_task_status(manifest: TaskManifest, output_format: str):
    """Print single task status."""
    if output_format == "json":
        click.echo(json.dumps(_task_to_dict(manifest), indent=2))
    else:
        click.echo(f"Task: {manifest.task_id}")
        click.echo(f"Stage: {manifest.current_stage.value}")
        click.echo(f"Status: {manifest.status.value}")
        click.echo(f"Attempts: {manifest.attempts[manifest.current_stage]}/{manifest.max_attempts}")
        click.echo(f"Started by: {manifest.started_by}")
        if manifest.blockers:
            click.echo(f"Blockers: {', '.join(manifest.blockers)}")


@cli.command()
@click.argument("task_id")
@click.pass_context
def pause(ctx: click.Context, task_id: str):
    """Pause a running task."""
    _send_control_command(get_base_dir(ctx), task_id, "pause")
    click.echo(f"Pause requested for {task_id}")


@cli.command()
@click.argument("task_id")
@click.pass_context
def resume(ctx: click.Context, task_id: str):
    """Resume a paused task."""
    _send_control_command(get_base_dir(ctx), task_id, "resume")
    click.echo(f"Resume requested for {task_id}")


@cli.command()
@click.argument("task_id")
@click.pass_context
def abort(ctx: click.Context, task_id: str):
    """Abort a running task."""
    _send_control_command(get_base_dir(ctx), task_id, "abort")
    click.echo(f"Abort requested for {task_id}")


@cli.command()
@click.argument("task_id")
@click.option("--message", required=True, help="Steering message")
@click.pass_context
def steer(ctx: click.Context, task_id: str, message: str):
    """Send steering message to a task."""
    _send_control_command(get_base_dir(ctx), task_id, "steer", message=message)
    click.echo(f"Steering message sent to {task_id}")


def _send_control_command(base_dir: Path, task_id: str, command: str, **kwargs):
    """Send control command to supervisor."""
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


@cli.command()
@click.argument("task_id")
@click.pass_context
def events(ctx: click.Context, task_id: str):
    """Show event history for a task."""
    event_logger = EventLogger(get_base_dir(ctx) / "events")
    events = event_logger.get_event_history(task_id)
    
    if not events:
        click.echo(f"No events found for {task_id}")
        return
    
    click.echo(f"Event history for {task_id}:")
    click.echo("-" * 50)
    
    for event in events:
        click.echo(f"{event.timestamp} | {event.type.value}")
        for key, value in event.data.items():
            click.echo(f"  {key}: {value}")


def main():
    """Entry point."""
    cli()


if __name__ == "__main__":
    main()
