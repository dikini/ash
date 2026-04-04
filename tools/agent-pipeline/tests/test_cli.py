"""Tests for CLI behavior."""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from click.testing import CliRunner

from agent_pipeline.cli import cli, resolve_base_dir, resolve_stage_agents, resolve_workspace_root
from agent_pipeline.state import StateManager


@pytest.fixture
def runner() -> CliRunner:
    """Provide a Click test runner."""
    return CliRunner()


def _write_task_bundle(base_dir: Path, task_id: str, state: str) -> None:
    """Create a task bundle in the requested lifecycle state."""
    manager = StateManager(base_dir)
    manager.create_task(task_id, started_by="cli")
    (base_dir / "queue" / task_id / "task.md").write_text(f"# {task_id}\n")

    if state == "queue":
        return
    if state == "in-progress":
        manager.move_to_in_progress(task_id)
        return
    if state == "blocked":
        manager.move_to_in_progress(task_id)
        manager.move_to_blocked(task_id, "waiting on review")
        return
    if state == "done":
        manager.move_to_in_progress(task_id)
        manager.move_to_done(task_id)
        return

    raise ValueError(f"Unsupported state: {state}")


def test_queue_from_spec_creates_bundle_that_survives_later_moves(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Queueing from spec should create a movable task bundle."""
    base_dir = tmp_path / "pipeline-state"
    spec_path = tmp_path / "task-spec.md"
    spec_text = "# TASK-004\n\nDo the work.\n"
    spec_path.write_text(spec_text)

    result = runner.invoke(
        cli,
        ["--base-dir", str(base_dir), "queue", "TASK-004", "--from-spec", str(spec_path)],
    )

    assert result.exit_code == 0
    queued_dir = base_dir / "queue" / "TASK-004"
    assert (queued_dir / "manifest.json").exists()
    assert (queued_dir / "task.md").read_text() == spec_text

    manager = StateManager(base_dir)
    manager.move_to_in_progress("TASK-004")
    manager.move_to_done("TASK-004")

    done_dir = base_dir / "done" / "TASK-004"
    assert (done_dir / "manifest.json").exists()
    assert (done_dir / "task.md").read_text() == spec_text


@pytest.mark.parametrize("state", ["queue", "in-progress", "blocked", "done"])
def test_status_finds_tasks_across_all_lifecycle_directories(
    runner: CliRunner,
    tmp_path: Path,
    state: str,
) -> None:
    """RED: Single-task status should resolve every persisted lifecycle state."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-404", state)

    result = runner.invoke(
        cli,
        ["--base-dir", str(base_dir), "status", "--task", "TASK-404", "--format", "json"],
    )

    assert result.exit_code == 0
    payload = json.loads(result.output)
    assert payload["task_id"] == "TASK-404"
    if state == "blocked":
        assert payload["status"] == "blocked"
    elif state == "done":
        assert payload["status"] == "complete"
    else:
        assert payload["status"] == "in_progress"



def test_status_aggregate_output_includes_completed_tasks(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Aggregate status should report completed task bundles."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-QUEUE", "queue")
    _write_task_bundle(base_dir, "TASK-RUN", "in-progress")
    _write_task_bundle(base_dir, "TASK-BLOCKED", "blocked")
    _write_task_bundle(base_dir, "TASK-DONE", "done")

    result = runner.invoke(cli, ["--base-dir", str(base_dir), "status"])

    assert result.exit_code == 0
    assert "Queue (1 tasks):" in result.output
    assert "In Progress (1 tasks):" in result.output
    assert "Blocked (1 tasks):" in result.output
    assert "Completed (1 tasks):" in result.output
    assert "TASK-DONE" in result.output



def test_workspace_configuration_resolves_stable_base_dir_outside_caller_cwd(
    runner: CliRunner,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """RED: Workspace-configured state should not depend on the invocation cwd."""
    workspace_root = tmp_path / "workspace"
    workspace_root.mkdir()
    spec_path = tmp_path / "task-spec.md"
    spec_path.write_text("# TASK-WS\n")
    monkeypatch.setenv("AGENT_PIPELINE_WORKSPACE_ROOT", str(workspace_root))

    with runner.isolated_filesystem(temp_dir=str(tmp_path)):
        result = runner.invoke(
            cli,
            ["queue", "TASK-WS", "--from-spec", str(spec_path)],
            env={"AGENT_PIPELINE_WORKSPACE_ROOT": str(workspace_root)},
        )

        assert result.exit_code == 0
        assert resolve_workspace_root() == workspace_root
        assert resolve_base_dir() == workspace_root / ".agents"
        assert (workspace_root / ".agents" / "queue" / "TASK-WS" / "manifest.json").exists()
        assert not Path(".agents").exists()


def test_queue_rejects_duplicate_task_ids_in_other_lifecycle_directories(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Queue should not silently shadow an existing task in another lifecycle directory."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-DUP", "done")

    result = runner.invoke(cli, ["--base-dir", str(base_dir), "queue", "TASK-DUP"])

    assert result.exit_code != 0
    assert "already exists in done" in result.output


def test_queue_from_spec_missing_input_does_not_create_task_state(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Invalid --from-spec input must not leave behind a poisoned queued bundle."""
    base_dir = tmp_path / "pipeline-state"
    missing_spec = tmp_path / "missing-task.md"

    result = runner.invoke(
        cli,
        ["--base-dir", str(base_dir), "queue", "TASK-MISSING", "--from-spec", str(missing_spec)],
    )

    assert result.exit_code != 0
    assert "not found" in result.output
    assert not (base_dir / "queue" / "TASK-MISSING").exists()

    retry_spec = tmp_path / "retry-task.md"
    retry_spec.write_text("# TASK-MISSING\n")
    retry = runner.invoke(
        cli,
        ["--base-dir", str(base_dir), "queue", "TASK-MISSING", "--from-spec", str(retry_spec)],
    )

    assert retry.exit_code == 0


def test_queue_from_spec_resolves_repo_relative_paths_against_workspace_root(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Relative --from-spec paths should resolve from the configured workspace root."""
    workspace_root = tmp_path / "workspace"
    workspace_root.mkdir()
    spec_path = workspace_root / "docs" / "plan" / "tasks" / "TASK-REL.md"
    spec_path.parent.mkdir(parents=True)
    spec_text = "# TASK-REL\n\nRelative path spec.\n"
    spec_path.write_text(spec_text)

    outside_cwd = tmp_path / "outside"
    outside_cwd.mkdir()

    with runner.isolated_filesystem(temp_dir=str(outside_cwd)):
        result = runner.invoke(
            cli,
            [
                "--workspace-root",
                str(workspace_root),
                "queue",
                "TASK-REL",
                "--from-spec",
                "docs/plan/tasks/TASK-REL.md",
            ],
        )

    assert result.exit_code == 0
    queued_bundle = workspace_root / ".agents" / "queue" / "TASK-REL"
    assert (queued_bundle / "task.md").read_text() == spec_text


def test_queue_from_missing_spec_does_not_create_task_bundle(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Invalid --from-spec input should not leave queued task state behind."""
    base_dir = tmp_path / "pipeline-state"
    missing_spec = tmp_path / "missing-spec.md"

    result = runner.invoke(
        cli,
        ["--base-dir", str(base_dir), "queue", "TASK-MISSING", "--from-spec", str(missing_spec)],
    )

    assert result.exit_code != 0
    assert "not found" in result.output
    assert not (base_dir / "queue" / "TASK-MISSING").exists()

    retry_result = runner.invoke(cli, ["--base-dir", str(base_dir), "queue", "TASK-MISSING"])
    assert retry_result.exit_code == 0


def test_resolve_stage_agents_from_json_env(monkeypatch: pytest.MonkeyPatch) -> None:
    """RED: Stage-agent overrides should parse from the shared environment surface."""
    monkeypatch.setenv("AGENT_PIPELINE_STAGE_AGENTS", '{"qa":"codex","design":"hermes"}')

    resolved = resolve_stage_agents()

    assert resolved == {"qa": "codex", "design": "hermes"}


def test_resolve_stage_agents_rejects_invalid_json(monkeypatch: pytest.MonkeyPatch) -> None:
    """RED: Invalid stage-agent JSON should fail clearly."""
    monkeypatch.setenv("AGENT_PIPELINE_STAGE_AGENTS", '{not-json}')

    with pytest.raises(ValueError, match="Invalid AGENT_PIPELINE_STAGE_AGENTS"):
        resolve_stage_agents()


def test_status_json_includes_effective_default_stage_agent_mapping(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Aggregate status JSON should expose the active default stage-agent mapping."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-JSON", "queue")

    result = runner.invoke(cli, ["--base-dir", str(base_dir), "status", "--format", "json"])

    assert result.exit_code == 0
    payload = json.loads(result.output)
    assert payload["stage_agents"] == {
        "design": "codex",
        "spec_write": "hermes",
        "spec_verify": "codex",
        "plan_write": "hermes",
        "plan_verify": "codex",
        "impl": "hermes",
        "qa": "hermes",
        "validate": "codex",
    }


def test_status_json_includes_effective_overridden_stage_agent_mapping(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Aggregate status JSON should show runtime stage-agent overrides."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-JSON", "queue")

    result = runner.invoke(
        cli,
        [
            "--base-dir",
            str(base_dir),
            "--stage-agents",
            '{"qa":"codex","design":"hermes"}',
            "status",
            "--format",
            "json",
        ],
    )

    assert result.exit_code == 0
    payload = json.loads(result.output)
    assert payload["stage_agents"]["qa"] == "codex"
    assert payload["stage_agents"]["design"] == "hermes"
    assert payload["stage_agents"]["spec_write"] == "hermes"


def test_status_json_prefers_runtime_dashboard_stage_agent_mapping(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Status JSON should report the mapping persisted by the running supervisor when available."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-JSON", "queue")
    status_dir = base_dir / "status"
    status_dir.mkdir(parents=True, exist_ok=True)
    (status_dir / "dashboard.json").write_text(
        json.dumps(
            {
                "timestamp": "2026-04-04T00:00:00",
                "active_sessions": 0,
                "sessions": {},
                "paused": [],
                "stage_agents": {
                    "design": "codex",
                    "spec_write": "hermes",
                    "spec_verify": "codex",
                    "plan_write": "hermes",
                    "plan_verify": "codex",
                    "impl": "hermes",
                    "qa": "codex",
                    "validate": "codex",
                },
            }
        )
    )

    result = runner.invoke(cli, ["--base-dir", str(base_dir), "status", "--format", "json"])

    assert result.exit_code == 0
    payload = json.loads(result.output)
    assert payload["stage_agents"]["qa"] == "codex"


def test_cli_stage_agent_validation_failure_is_user_facing(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Invalid stage-agent overrides should fail cleanly without a traceback."""
    base_dir = tmp_path / "pipeline-state"

    result = runner.invoke(
        cli,
        [
            "--base-dir",
            str(base_dir),
            "--stage-agents",
            '{"bogus":"codex"}',
            "status",
            "--format",
            "json",
        ],
    )

    assert result.exit_code != 0
    assert "Unknown stage" in result.output
    assert "Traceback" not in result.output
