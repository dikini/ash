"""Tests for CLI behavior."""

from __future__ import annotations

import json
from pathlib import Path
from unittest.mock import Mock, patch

import pytest
from click.testing import CliRunner

from agent_pipeline.cli import cli, resolve_base_dir, resolve_stage_agents, resolve_workspace_root
from agent_pipeline.state import Stage, StateManager


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


def test_get_supervisor_uses_resolved_workspace_root_from_context(runner: CliRunner, tmp_path: Path) -> None:
    """RED: Supervisor construction should receive the resolved workspace root from the CLI context."""
    from click import Context
    from agent_pipeline.cli import get_supervisor

    workspace_root = tmp_path / "workspace"
    workspace_root.mkdir()

    with patch("agent_pipeline.cli.Supervisor") as mock_supervisor:
        ctx = Context(cli)
        ctx.obj = {
            "workspace_root": workspace_root.resolve(),
            "workspace_root_explicit": True,
            "base_dir": tmp_path / "state",
            "stage_agents": None,
        }
        get_supervisor(ctx)

    assert mock_supervisor.call_args.kwargs["workspace_root"] == workspace_root.resolve()


def test_status_aggregate_text_surfaces_invalid_worktree_metadata(runner: CliRunner, tmp_path: Path) -> None:
    """RED: Aggregate text status should expose malformed persisted worktree metadata distinctly."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-BROKEN", "blocked")
    manifest_path = base_dir / "blocked" / "TASK-BROKEN" / "manifest.json"
    payload = json.loads(manifest_path.read_text())
    payload["worktree"] = {"path": "relative/path", "branch": 42}
    manifest_path.write_text(json.dumps(payload))

    result = runner.invoke(cli, ["--base-dir", str(base_dir), "status"])

    assert result.exit_code == 0
    assert "worktree error: invalid persisted worktree metadata" in result.output


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


def test_queue_accepts_repeated_depends_on_and_persists_manifest_dependencies(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Queue should accept explicit prerequisite task ids."""
    base_dir = tmp_path / "pipeline-state"

    result = runner.invoke(
        cli,
        [
            "--base-dir", str(base_dir),
            "queue", "TASK-DEP",
            "--depends-on", "TASK-100",
            "--depends-on", "TASK-200",
        ],
    )

    assert result.exit_code == 0
    manifest = json.loads((base_dir / "queue" / "TASK-DEP" / "manifest.json").read_text())
    assert manifest["dependencies"] == ["TASK-100", "TASK-200"]


def test_queue_normalizes_duplicate_and_whitespace_dependencies(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Dependency inputs should be trimmed and deduplicated before they reach the manifest."""
    base_dir = tmp_path / "pipeline-state"

    result = runner.invoke(
        cli,
        [
            "--base-dir", str(base_dir),
            "queue", "TASK-DEP",
            "--depends-on", " TASK-100 ",
            "--depends-on", "TASK-100",
            "--depends-on", "TASK-200 ",
        ],
    )

    assert result.exit_code == 0
    manifest = json.loads((base_dir / "queue" / "TASK-DEP" / "manifest.json").read_text())
    assert manifest["dependencies"] == ["TASK-100", "TASK-200"]


def test_queue_rejects_self_dependency(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Tasks should fail fast when asked to depend on themselves."""
    base_dir = tmp_path / "pipeline-state"

    result = runner.invoke(
        cli,
        ["--base-dir", str(base_dir), "queue", "TASK-SELF", "--depends-on", "TASK-SELF"],
    )

    assert result.exit_code != 0
    assert "Task TASK-SELF cannot depend on itself" in result.output


def test_queue_rejects_transitive_dependency_cycles(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Queue should reject dependency edges that would create a cycle against existing task manifests."""
    base_dir = tmp_path / "pipeline-state"
    manager = StateManager(base_dir)
    manager.create_task("TASK-BASE", started_by="cli", dependencies=["TASK-NEW"])

    result = runner.invoke(
        cli,
        ["--base-dir", str(base_dir), "queue", "TASK-NEW", "--depends-on", "TASK-BASE"],
    )

    assert result.exit_code != 0
    assert "would create a dependency cycle" in result.output


def test_queue_rejects_dependency_ids_with_path_traversal_characters(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Dependency ids must be safe task identifiers, not filesystem paths."""
    base_dir = tmp_path / "pipeline-state"

    result = runner.invoke(
        cli,
        ["--base-dir", str(base_dir), "queue", "TASK-NEW", "--depends-on", "../TASK-BASE"],
    )

    assert result.exit_code != 0
    assert "Dependency task id must use only letters, numbers, dots, underscores, and hyphens" in result.output


def test_queue_rejects_task_ids_with_path_traversal_characters(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Top-level task ids should also reject filesystem path syntax."""
    base_dir = tmp_path / "pipeline-state"

    result = runner.invoke(
        cli,
        ["--base-dir", str(base_dir), "queue", "../TASK-NEW"],
    )

    assert result.exit_code != 0
    assert "Task id must use only letters, numbers, dots, underscores, and hyphens" in result.output


@pytest.mark.parametrize(
    ("command", "args"),
    [
        ("status", ["--task", "../TASK-NEW"]),
        ("pause", ["../TASK-NEW"]),
        ("resume", ["../TASK-NEW"]),
        ("abort", ["../TASK-NEW"]),
        ("steer", ["../TASK-NEW", "--message", "focus"]),
        ("resolve-feedback", [
            "../TASK-NEW",
            "--review-artifact", "spec.review",
            "--summary", "summary",
            "--changes", "changes",
            "--success-condition", "done",
        ]),
        ("retry-feedback", ["../TASK-NEW"]),
        ("logs", ["../TASK-NEW"]),
        ("events", ["../TASK-NEW"]),
    ],
)
def test_non_queue_commands_reject_task_ids_with_path_traversal_characters(
    runner: CliRunner,
    tmp_path: Path,
    command: str,
    args: list[str],
) -> None:
    """RED: All filesystem-touching CLI entrypoints should reject unsafe task ids, not just queue."""
    base_dir = tmp_path / "pipeline-state"

    result = runner.invoke(
        cli,
        ["--base-dir", str(base_dir), command, *args],
    )

    assert result.exit_code != 0
    assert "Task id must use only letters, numbers, dots, underscores, and hyphens" in result.output


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


def test_resolve_stage_agents_defaults_remain_none_without_override(monkeypatch: pytest.MonkeyPatch) -> None:
    """RED: Default Hermes-only mapping should come from AgentSpawner defaults, not an implicit CLI override payload."""
    monkeypatch.delenv("AGENT_PIPELINE_STAGE_AGENTS", raising=False)

    assert resolve_stage_agents() is None


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
        "design": "hermes",
        "spec_write": "hermes",
        "spec_verify": "hermes",
        "plan_write": "hermes",
        "plan_verify": "hermes",
        "impl": "hermes",
        "qa": "hermes",
        "validate": "hermes",
    }


def test_status_task_json_includes_dependency_wait_fields_for_queued_task(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Per-task status should surface unmet dependency gating."""
    base_dir = tmp_path / "pipeline-state"
    manager = StateManager(base_dir)
    manager.create_task("TASK-DEP", started_by="cli", dependencies=["TASK-BASE"])

    result = runner.invoke(
        cli,
        ["--base-dir", str(base_dir), "status", "--task", "TASK-DEP", "--format", "json"],
    )

    assert result.exit_code == 0
    payload = json.loads(result.output)
    assert payload["dependencies"] == ["TASK-BASE"]
    assert payload["unmet_dependencies"] == ["TASK-BASE"]
    assert payload["waiting_on_dependencies"] is True


def test_status_aggregate_text_surfaces_unmet_dependencies_for_queued_tasks(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Aggregate text status should show when queued work is waiting on dependencies."""
    base_dir = tmp_path / "pipeline-state"
    manager = StateManager(base_dir)
    manager.create_task("TASK-DEP", started_by="cli", dependencies=["TASK-BASE"])

    result = runner.invoke(cli, ["--base-dir", str(base_dir), "status"])

    assert result.exit_code == 0
    assert "TASK-DEP" in result.output
    assert "waiting on: TASK-BASE" in result.output


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
                    "design": "hermes",
                    "spec_write": "hermes",
                    "spec_verify": "hermes",
                    "plan_write": "hermes",
                    "plan_verify": "hermes",
                    "impl": "hermes",
                    "qa": "codex",
                    "validate": "hermes",
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


def test_logs_command_reads_current_stage_stdout_for_running_task(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: CLI should expose current-stage stdout while a task is still in progress."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-LOG", "in-progress")
    task_dir = base_dir / "in-progress" / "TASK-LOG"
    (task_dir / "design.stdout.log").write_text("live output\n")

    result = runner.invoke(cli, ["--base-dir", str(base_dir), "logs", "TASK-LOG"])

    assert result.exit_code == 0
    assert result.output == "live output\n"


def test_logs_command_tails_last_n_lines_of_requested_stream(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: CLI should support bounded tail output for live stderr peeking."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-LOG", "in-progress")
    task_dir = base_dir / "in-progress" / "TASK-LOG"
    manager = StateManager(base_dir)
    manifest = manager.load_task("TASK-LOG", subdir="in-progress")
    manifest.current_stage = Stage.IMPL
    manager.save_task(manifest, subdir="in-progress")
    (task_dir / "impl.stderr.log").write_text("one\ntwo\nthree\n")

    result = runner.invoke(
        cli,
        ["--base-dir", str(base_dir), "logs", "TASK-LOG", "--stream", "stderr", "--tail", "2"],
    )

    assert result.exit_code == 0
    assert result.output == "two\nthree\n"


def test_logs_command_reports_missing_log_for_stage_that_has_not_started(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Missing logs should fail with concise user-facing output."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-NOLOG", "queue")

    result = runner.invoke(
        cli,
        ["--base-dir", str(base_dir), "logs", "TASK-NOLOG", "--stage", "design", "--stream", "stdout"],
    )

    assert result.exit_code != 0
    assert "No stdout log for TASK-NOLOG stage design yet" in result.output
    assert "Traceback" not in result.output


def test_logs_command_follow_streams_chunks_from_supervisor(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: --follow should stream chunks instead of doing a single snapshot read."""
    base_dir = tmp_path / "pipeline-state"
    fake_supervisor = Mock()
    fake_supervisor.follow_stage_log.return_value = iter(["hello\n", "world\n"])

    with patch("agent_pipeline.cli.get_supervisor", return_value=fake_supervisor):
        result = runner.invoke(
            cli,
            ["--base-dir", str(base_dir), "logs", "TASK-LOG", "--follow"],
        )

    assert result.exit_code == 0
    assert result.output == "hello\nworld\n"
    fake_supervisor.follow_stage_log.assert_called_once()
    fake_supervisor.read_stage_log.assert_not_called()


def test_resolve_feedback_command_writes_structured_resolution_artifact(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Operators should be able to persist structured retry guidance without manual file edits."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-RESOLVE", "blocked")
    task_dir = base_dir / "blocked" / "TASK-RESOLVE"
    (task_dir / "spec.review").write_text("needs revision\n")

    result = runner.invoke(
        cli,
        [
            "--base-dir", str(base_dir),
            "resolve-feedback", "TASK-RESOLVE",
            "--review-artifact", "spec.review",
            "--summary", "Tighten traceability and verification commands",
            "--changes", "1. Add exact command\n2. Strengthen README verification",
            "--success-condition", "spec_verify should be able to return VERIFIED",
        ],
    )

    assert result.exit_code == 0
    resolution = (task_dir / "feedback-resolution.md").read_text()
    assert "Source review artifact: spec.review" in resolution
    assert "Tighten traceability and verification commands" in resolution
    assert "1. Add exact command" in resolution
    assert "spec_verify should be able to return VERIFIED" in resolution


def test_resolve_feedback_requires_existing_review_artifact(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Structured retry guidance must point at a review artifact that actually exists in the task bundle."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-RESOLVE", "blocked")

    result = runner.invoke(
        cli,
        [
            "--base-dir", str(base_dir),
            "resolve-feedback", "TASK-RESOLVE",
            "--review-artifact", "spec.review",
            "--summary", "Tighten traceability",
            "--changes", "Add exact verification command",
            "--success-condition", "Verifier returns VERIFIED",
        ],
    )

    assert result.exit_code != 0
    assert "Review artifact spec.review not found for TASK-RESOLVE" in result.output


def test_resolve_feedback_rejects_review_artifact_path_outside_task_bundle(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Review artifact references must stay inside the task bundle and not allow traversal."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-RESOLVE", "blocked")
    outside_review = tmp_path / "outside.review"
    outside_review.write_text("wrong place\n")

    result = runner.invoke(
        cli,
        [
            "--base-dir", str(base_dir),
            "resolve-feedback", "TASK-RESOLVE",
            "--review-artifact", "../outside.review",
            "--summary", "Tighten traceability",
            "--changes", "Add exact verification command",
            "--success-condition", "Verifier returns VERIFIED",
        ],
    )

    assert result.exit_code != 0
    assert "must stay within the task bundle" in result.output


def test_resolve_feedback_rejects_unsupported_in_bundle_artifact(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: resolve-feedback should reject existing in-bundle files that are not supported review artifacts."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-RESOLVE", "blocked")
    task_dir = base_dir / "blocked" / "TASK-RESOLVE"
    (task_dir / "task.md").write_text("# TASK-RESOLVE\n")

    result = runner.invoke(
        cli,
        [
            "--base-dir", str(base_dir),
            "resolve-feedback", "TASK-RESOLVE",
            "--review-artifact", "task.md",
            "--summary", "Tighten traceability",
            "--changes", "Add exact verification command",
            "--success-condition", "Verifier returns VERIFIED",
        ],
    )

    assert result.exit_code != 0
    assert "Unsupported feedback review artifact for retry: task.md" in result.output


def test_resolve_feedback_writes_utf8_encoded_resolution_file(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: feedback-resolution.md should be written as UTF-8 because later readers decode it explicitly as UTF-8."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-RESOLVE", "blocked")
    task_dir = base_dir / "blocked" / "TASK-RESOLVE"
    (task_dir / "spec.review").write_text("needs revision\n")

    original_write_text = Path.write_text
    encodings: list[str | None] = []

    def recording_write_text(self: Path, data: str, encoding=None, errors=None, newline=None):
        if self.name == "feedback-resolution.md":
            encodings.append(encoding)
        return original_write_text(self, data, encoding=encoding, errors=errors, newline=newline)

    with patch.object(Path, "write_text", new=recording_write_text):
        result = runner.invoke(
            cli,
            [
                "--base-dir", str(base_dir),
                "resolve-feedback", "TASK-RESOLVE",
                "--review-artifact", "spec.review",
                "--summary", "Résumé guidance",
                "--changes", "Añadir verificación exacta",
                "--success-condition", "Verifier returns VERIFIED",
            ],
        )

    assert result.exit_code == 0
    assert encodings == ["utf-8"]


def test_status_json_surfaces_feedback_resolution_presence_and_review_source(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Status should show whether structured feedback resolution exists for a task."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-RESOLVE", "blocked")
    task_dir = base_dir / "blocked" / "TASK-RESOLVE"
    (task_dir / "feedback-resolution.md").write_text(
        "# Feedback Resolution\n\nSource review artifact: spec.review\n"
    )

    result = runner.invoke(
        cli,
        ["--base-dir", str(base_dir), "status", "--task", "TASK-RESOLVE", "--format", "json"],
    )

    assert result.exit_code == 0
    payload = json.loads(result.output)
    assert payload["has_feedback_resolution"] is True
    assert payload["feedback_review_artifact"] == "spec.review"


def test_status_text_surfaces_feedback_resolution_presence_and_review_source(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Human-readable status should surface review-resolution metadata too."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-RESOLVE", "blocked")
    task_dir = base_dir / "blocked" / "TASK-RESOLVE"
    (task_dir / "feedback-resolution.md").write_text(
        "# Feedback Resolution\n\nSource review artifact: plan.review\n"
    )

    result = runner.invoke(cli, ["--base-dir", str(base_dir), "status", "--task", "TASK-RESOLVE"])

    assert result.exit_code == 0
    assert "Feedback resolution: yes" in result.output
    assert "Feedback review artifact: plan.review" in result.output


def test_status_task_json_includes_worktree_path_and_branch(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Per-task status JSON should surface persisted worktree metadata."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-WT", "blocked")
    manager = StateManager(base_dir)
    manifest = manager.load_task("TASK-WT", subdir="blocked")
    worktree_path = tmp_path / "repo" / ".worktrees" / "TASK-WT"
    manifest.set_worktree(path=worktree_path, branch="agent-pipeline/TASK-WT")
    manager.save_task(manifest, subdir="blocked")

    result = runner.invoke(
        cli,
        ["--base-dir", str(base_dir), "status", "--task", "TASK-WT", "--format", "json"],
    )

    assert result.exit_code == 0
    payload = json.loads(result.output)
    assert payload["worktree_path"] == str(worktree_path)
    assert payload["worktree_branch"] == "agent-pipeline/TASK-WT"


def test_status_text_includes_worktree_metadata_for_finished_tasks(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Human-readable status should show worktree path and branch when persisted."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-WT", "done")
    manager = StateManager(base_dir)
    manifest = manager.load_task("TASK-WT", subdir="done")
    worktree_path = tmp_path / "repo" / ".worktrees" / "TASK-WT"
    manifest.set_worktree(path=worktree_path, branch="agent-pipeline/TASK-WT")
    manager.save_task(manifest, subdir="done")

    result = runner.invoke(cli, ["--base-dir", str(base_dir), "status", "--task", "TASK-WT"])

    assert result.exit_code == 0
    assert f"Worktree: {worktree_path}" in result.output
    assert "Worktree branch: agent-pipeline/TASK-WT" in result.output


@pytest.mark.parametrize("state", ["queue", "in-progress"])
def test_cleanup_worktree_refuses_active_tasks(
    runner: CliRunner,
    tmp_path: Path,
    state: str,
) -> None:
    """RED: cleanup-worktree must refuse queued or active tasks."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-CLEAN", state)
    manager = StateManager(base_dir)
    manifest = manager.load_task("TASK-CLEAN", subdir=state)
    worktree_path = tmp_path / "repo" / ".worktrees" / "TASK-CLEAN"
    manifest.set_worktree(path=worktree_path, branch="agent-pipeline/TASK-CLEAN")
    manager.save_task(manifest, subdir=state)

    result = runner.invoke(cli, ["--base-dir", str(base_dir), "cleanup-worktree", "TASK-CLEAN"])

    assert result.exit_code != 0
    assert f"Refusing to clean up worktree for TASK-CLEAN while task is in {state}" in result.output


@pytest.mark.parametrize("state", ["blocked", "done"])
def test_cleanup_worktree_removes_finished_task_worktree_and_clears_manifest_metadata(
    runner: CliRunner,
    tmp_path: Path,
    state: str,
) -> None:
    """RED: cleanup-worktree should remove blocked/done worktrees and clear persisted metadata."""
    base_dir = tmp_path / "pipeline-state"
    repo_root = tmp_path / "repo"
    repo_root.mkdir()
    _write_task_bundle(base_dir, "TASK-CLEAN", state)
    manager = StateManager(base_dir)
    manifest = manager.load_task("TASK-CLEAN", subdir=state)
    worktree_path = repo_root / ".worktrees" / "TASK-CLEAN"
    worktree_path.mkdir(parents=True)
    manifest.set_worktree(path=worktree_path, branch="agent-pipeline/TASK-CLEAN")
    manager.save_task(manifest, subdir=state)

    with patch("agent_pipeline.cli.remove_task_worktree", return_value=None) as mock_remove:
        result = runner.invoke(
            cli,
            [
                "--workspace-root",
                str(repo_root),
                "--base-dir",
                str(base_dir),
                "cleanup-worktree",
                "TASK-CLEAN",
            ],
        )

    assert result.exit_code == 0
    mock_remove.assert_called_once()
    updated = manager.load_task("TASK-CLEAN", subdir=state)
    assert updated.worktree is None
    assert f"Cleaned up worktree for TASK-CLEAN: {worktree_path}" in result.output


def test_cleanup_worktree_rejects_missing_worktree_metadata(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: cleanup-worktree should fail clearly when a finished task has no tracked worktree."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-CLEAN", "done")

    result = runner.invoke(cli, ["--base-dir", str(base_dir), "cleanup-worktree", "TASK-CLEAN"])

    assert result.exit_code != 0
    assert "Task TASK-CLEAN has no tracked worktree metadata" in result.output


def test_cleanup_worktree_rejects_metadata_pointing_outside_expected_task_worktree(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: cleanup-worktree must fail closed when persisted worktree metadata points outside the deterministic task worktree."""
    base_dir = tmp_path / "pipeline-state"
    repo_root = tmp_path / "repo"
    repo_root.mkdir()
    outside = tmp_path / "outside-target"
    outside.mkdir()
    _write_task_bundle(base_dir, "TASK-CLEAN", "done")
    manager = StateManager(base_dir)
    manifest = manager.load_task("TASK-CLEAN", subdir="done")
    manifest.set_worktree(path=outside, branch="agent-pipeline/TASK-CLEAN")
    manager.save_task(manifest, subdir="done")

    result = runner.invoke(
        cli,
        ["--workspace-root", str(repo_root), "--base-dir", str(base_dir), "cleanup-worktree", "TASK-CLEAN"],
    )

    assert result.exit_code != 0
    assert "must stay under" in result.output


def test_cleanup_worktree_surfaces_invalid_persisted_worktree_metadata_error(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: cleanup-worktree should report invalid persisted worktree metadata distinctly from missing metadata."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-CLEAN", "done")
    manifest_path = base_dir / "done" / "TASK-CLEAN" / "manifest.json"
    payload = json.loads(manifest_path.read_text())
    payload["worktree"] = {"path": "relative/path", "branch": 42}
    manifest_path.write_text(json.dumps(payload))

    result = runner.invoke(cli, ["--base-dir", str(base_dir), "cleanup-worktree", "TASK-CLEAN"])

    assert result.exit_code != 0
    assert "invalid persisted worktree metadata" in result.output


def test_cleanup_worktree_uses_manifest_worktree_path_when_only_base_dir_is_supplied(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: cleanup-worktree should remain robust when only --base-dir is supplied and cwd is outside the repo."""
    base_dir = tmp_path / "pipeline-state"
    repo_root = tmp_path / "repo"
    repo_root.mkdir()
    _write_task_bundle(base_dir, "TASK-CLEAN", "done")
    manager = StateManager(base_dir)
    manifest = manager.load_task("TASK-CLEAN", subdir="done")
    worktree_path = repo_root / ".worktrees" / "TASK-CLEAN"
    worktree_path.mkdir(parents=True)
    manifest.set_worktree(path=worktree_path, branch="agent-pipeline/TASK-CLEAN")
    manager.save_task(manifest, subdir="done")

    outside_dir = tmp_path / "outside"
    outside_dir.mkdir()
    with patch("agent_pipeline.cli.remove_task_worktree", return_value=None) as mock_remove:
        with runner.isolated_filesystem(temp_dir=str(outside_dir)):
            result = runner.invoke(cli, ["--base-dir", str(base_dir), "cleanup-worktree", "TASK-CLEAN"])

    assert result.exit_code == 0
    assert mock_remove.call_args.args[0] == repo_root.resolve()


def test_cleanup_worktree_rejects_malformed_absolute_path_when_only_base_dir_is_supplied(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Base-dir-only cleanup should fail closed for malformed absolute persisted worktree paths."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-CLEAN", "done")
    manifest_path = base_dir / "done" / "TASK-CLEAN" / "manifest.json"
    payload = json.loads(manifest_path.read_text())
    payload["worktree"] = {"path": "/tmp", "branch": "agent-pipeline/TASK-CLEAN"}
    manifest_path.write_text(json.dumps(payload))

    result = runner.invoke(cli, ["--base-dir", str(base_dir), "cleanup-worktree", "TASK-CLEAN"])

    assert result.exit_code != 0
    assert "does not match expected '<repo-root>/.worktrees/TASK-CLEAN' structure" in result.output


def test_cleanup_worktree_clears_manifest_metadata_when_prune_fails_after_removal(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: If removal already happened, prune failure should not leave stale tracked worktree metadata behind."""
    base_dir = tmp_path / "pipeline-state"
    repo_root = tmp_path / "repo"
    repo_root.mkdir()
    _write_task_bundle(base_dir, "TASK-CLEAN", "done")
    manager = StateManager(base_dir)
    manifest = manager.load_task("TASK-CLEAN", subdir="done")
    worktree_path = repo_root / ".worktrees" / "TASK-CLEAN"
    worktree_path.mkdir(parents=True)
    manifest.set_worktree(path=worktree_path, branch="agent-pipeline/TASK-CLEAN")
    manager.save_task(manifest, subdir="done")

    def remove_then_fail(_repo_root, task_manifest):
        Path(task_manifest.worktree.path).rmdir()
        raise RuntimeError("Failed to prune git worktree metadata")

    with patch("agent_pipeline.cli.remove_task_worktree", side_effect=remove_then_fail):
        result = runner.invoke(
            cli,
            ["--workspace-root", str(repo_root), "--base-dir", str(base_dir), "cleanup-worktree", "TASK-CLEAN"],
        )

    assert result.exit_code != 0
    updated = manager.load_task("TASK-CLEAN", subdir="done")
    assert updated.worktree is None
    assert "Failed to prune git worktree metadata" in result.output


def test_retry_feedback_requeues_blocked_task_at_stage_inferred_from_review_artifact(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Feedback-resolved blocked tasks should be explicitly releasable back to queue at the correct producer stage."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-RETRY", "blocked")
    task_dir = base_dir / "blocked" / "TASK-RETRY"
    (task_dir / "spec.review").write_text("needs revision\n")
    (task_dir / "feedback-resolution.md").write_text(
        "# Feedback Resolution\n\nSource review artifact: spec.review\n"
    )
    (task_dir / "spec.verified").write_text("stale\n")
    (task_dir / "plan.md").write_text("stale downstream\n")
    (task_dir / "spec_write.stdout.log").write_text("old log\n")
    manager = StateManager(base_dir)
    manifest = manager.load_task("TASK-RETRY", subdir="blocked")
    manifest.current_stage = Stage.SPEC_VERIFY
    manifest.attempts[Stage.SPEC_WRITE] = 2
    manifest.attempts[Stage.SPEC_VERIFY] = 1
    manager.save_task(manifest, subdir="blocked")

    result = runner.invoke(cli, ["--base-dir", str(base_dir), "retry-feedback", "TASK-RETRY"])

    assert result.exit_code == 0
    queued_dir = base_dir / "queue" / "TASK-RETRY"
    assert queued_dir.exists()
    updated = manager.load_task("TASK-RETRY", subdir="queue")
    assert updated.current_stage == Stage.SPEC_WRITE
    assert updated.status.value == "in_progress"
    assert updated.blockers == []
    assert updated.attempts[Stage.SPEC_WRITE] == 0
    assert updated.attempts[Stage.SPEC_VERIFY] == 0
    assert (queued_dir / "spec.review").exists() is False
    assert (queued_dir / "spec.verified").exists() is False
    assert (queued_dir / "plan.md").exists() is False
    assert (queued_dir / "spec_write.stdout.log").exists() is False
    resolution = (queued_dir / "feedback-resolution.md").read_text()
    assert "Source review artifact: retry-history/" in resolution
    archived_reviews = list((queued_dir / "retry-history").rglob("spec.review"))
    assert len(archived_reviews) == 1


def test_retry_feedback_requires_blocked_task_with_feedback_resolution(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Retry helper should reject blocked tasks that are missing feedback resolution guidance."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-RETRY", "blocked")
    task_dir = base_dir / "blocked" / "TASK-RETRY"
    (task_dir / "spec.review").write_text("needs revision\n")

    result = runner.invoke(cli, ["--base-dir", str(base_dir), "retry-feedback", "TASK-RETRY"])

    assert result.exit_code != 0
    assert "has no feedback-resolution.md" in result.output


def test_retry_feedback_rejects_non_blocked_tasks(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Retry helper must only operate on blocked task bundles."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-RETRY", "queue")

    result = runner.invoke(cli, ["--base-dir", str(base_dir), "retry-feedback", "TASK-RETRY"])

    assert result.exit_code != 0
    assert "is in queue, not blocked" in result.output


def test_retry_feedback_refuses_direct_in_progress_restore_when_dependencies_unmet(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: Direct in-progress restore must still respect dependency gating."""
    base_dir = tmp_path / "pipeline-state"
    manager = StateManager(base_dir)
    manager.create_task("TASK-RETRY", started_by="cli", dependencies=["TASK-BASE"])
    manager.move_to_in_progress("TASK-RETRY")
    manager.move_to_blocked("TASK-RETRY", "waiting on review")
    task_dir = base_dir / "blocked" / "TASK-RETRY"
    (task_dir / "qa.review").write_text("needs fixes\n")
    (task_dir / "feedback-resolution.md").write_text(
        "# Feedback Resolution\n\nSource review artifact: qa.review\n"
    )

    result = runner.invoke(
        cli,
        ["--base-dir", str(base_dir), "retry-feedback", "TASK-RETRY", "--to", "in-progress"],
    )

    assert result.exit_code != 0
    assert "Cannot move TASK-RETRY directly to in-progress while dependencies are unmet: TASK-BASE" in result.output


def test_retry_feedback_supports_second_cycle_when_resolution_references_archived_review_path(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: A second retry should still infer the restart stage and preserve the newest review when feedback-resolution points at an archived path."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-RETRY", "blocked")
    task_dir = base_dir / "blocked" / "TASK-RETRY"
    retry_history_dir = task_dir / "retry-history" / "20260404T120000"
    retry_history_dir.mkdir(parents=True)
    (retry_history_dir / "spec.review").write_text("older review\n")
    (task_dir / "spec.review").write_text("newer review\n")
    (task_dir / "feedback-resolution.md").write_text(
        "# Feedback Resolution\n\nSource review artifact: retry-history/20260404T120000/spec.review\n"
    )
    manager = StateManager(base_dir)
    manifest = manager.load_task("TASK-RETRY", subdir="blocked")
    manifest.current_stage = Stage.SPEC_VERIFY
    manager.save_task(manifest, subdir="blocked")

    result = runner.invoke(cli, ["--base-dir", str(base_dir), "retry-feedback", "TASK-RETRY"])

    assert result.exit_code == 0
    queued_dir = base_dir / "queue" / "TASK-RETRY"
    updated = manager.load_task("TASK-RETRY", subdir="queue")
    assert updated.current_stage == Stage.SPEC_WRITE
    resolution = (queued_dir / "feedback-resolution.md").read_text()
    assert "Source review artifact: retry-history/" in resolution
    archived_reviews = sorted(path.read_text() for path in (queued_dir / "retry-history").rglob("spec.review"))
    assert archived_reviews == ["newer review\n", "older review\n"]


def test_retry_feedback_rejects_review_artifact_path_outside_task_bundle(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: retry-feedback must reject feedback-resolution entries that escape the task bundle."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-RETRY", "blocked")
    outside_review = tmp_path / "outside.review"
    outside_review.write_text("wrong place\n")
    task_dir = base_dir / "blocked" / "TASK-RETRY"
    (task_dir / "feedback-resolution.md").write_text(
        "# Feedback Resolution\n\nSource review artifact: ../outside.review\n"
    )

    result = runner.invoke(cli, ["--base-dir", str(base_dir), "retry-feedback", "TASK-RETRY"])

    assert result.exit_code != 0
    assert "must stay within the task bundle" in result.output


def test_retry_feedback_updates_manifest_timestamp_when_releasing_task(
    runner: CliRunner,
    tmp_path: Path,
) -> None:
    """RED: retry-feedback should refresh updated_at when it mutates manifest state."""
    base_dir = tmp_path / "pipeline-state"
    _write_task_bundle(base_dir, "TASK-RETRY", "blocked")
    task_dir = base_dir / "blocked" / "TASK-RETRY"
    (task_dir / "spec.review").write_text("needs revision\n")
    (task_dir / "feedback-resolution.md").write_text(
        "# Feedback Resolution\n\nSource review artifact: spec.review\n"
    )
    manager = StateManager(base_dir)
    manifest = manager.load_task("TASK-RETRY", subdir="blocked")
    manifest.current_stage = Stage.SPEC_VERIFY
    old_updated_at = manifest.updated_at
    manager.save_task(manifest, subdir="blocked")

    result = runner.invoke(cli, ["--base-dir", str(base_dir), "retry-feedback", "TASK-RETRY"])

    assert result.exit_code == 0
    updated = manager.load_task("TASK-RETRY", subdir="queue")
    assert updated.updated_at != old_updated_at
