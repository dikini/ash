"""Tests for supervisor module."""

import json
import tempfile
import threading
import time
from pathlib import Path
from unittest.mock import Mock, patch

import pytest

from agent_pipeline.agents import AgentType, SpawnResult
from agent_pipeline.supervisor import Supervisor, StageLogStream
from agent_pipeline.state import Stage, TaskManifest
from agent_pipeline.worktrees import (
    ProvisioningOutcome,
    ProvisioningResult,
    WorktreeAssignment,
)


class TestSupervisor:
    """Test supervisor functionality."""

    @pytest.fixture
    def temp_dir(self):
        """Provide temporary directory."""
        with tempfile.TemporaryDirectory() as tmp:
            yield Path(tmp)

    @pytest.fixture
    def supervisor(self, temp_dir):
        """Provide Supervisor instance."""
        return Supervisor(temp_dir)

    def _write_task(self, supervisor, manifest, subdir="in-progress"):
        """Persist a task bundle for tests."""
        supervisor.state_manager.save_task(manifest, subdir=subdir)
        task_dir = supervisor.base_dir / subdir / manifest.task_id
        task_dir.mkdir(parents=True, exist_ok=True)
        (task_dir / "task.md").write_text(f"# {manifest.task_id}\n")
        return task_dir

    def test_check_queue_finds_new_tasks(self, supervisor):
        """RED: Should find tasks in queue directory."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        self._write_task(supervisor, manifest, subdir="queue")

        tasks = supervisor.check_queue()

        assert len(tasks) == 1
        assert tasks[0].task_id == "TASK-001"

    def test_check_control_commands(self, supervisor, temp_dir):
        """RED: Should read control commands and delete the file."""
        control_file = temp_dir / "control" / "pause_TASK-001.json"
        control_file.write_text(json.dumps({
            "command": "pause",
            "task_id": "TASK-001",
            "timestamp": "2024-01-01T00:00:00Z",
        }))

        commands = supervisor.check_control()

        assert commands == [{
            "command": "pause",
            "task_id": "TASK-001",
            "timestamp": "2024-01-01T00:00:00Z",
        }]
        assert control_file.exists() is False

    def test_supervisor_uses_configured_stage_agent_mapping(self, temp_dir):
        """RED: Supervisor should pass through runtime stage-agent overrides to its spawner."""
        supervisor = Supervisor(temp_dir, stage_agents={"qa": "codex"})

        assert supervisor.agent_spawner.get_agent_type(Stage.QA) == AgentType.CODEX
        assert supervisor.agent_spawner.get_agent_type(Stage.DESIGN) == AgentType.HERMES

    def test_supervisor_uses_configured_workspace_root_for_repo_discovery(self, temp_dir):
        """RED: Supervisor provisioning should honor the configured workspace root instead of heuristic rediscovery."""
        workspace_root = temp_dir / "configured-workspace"
        workspace_root.mkdir()
        supervisor = Supervisor(temp_dir, workspace_root=workspace_root)

        assert supervisor._discover_repo_root() == workspace_root.resolve()
        assert supervisor.agent_spawner.workspace_root == workspace_root.resolve()

    def test_missing_configured_workspace_root_blocks_task_instead_of_crashing(self, temp_dir):
        """RED: Missing configured workspace roots should fail closed with a blocker instead of crashing the supervisor."""
        supervisor = Supervisor(temp_dir, workspace_root=temp_dir / "missing-workspace")
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        self._write_task(supervisor, manifest)

        assert supervisor.start_task_stage("TASK-001") is False

        state, blocked_manifest = supervisor.state_manager.find_task("TASK-001")
        assert state == "blocked"
        assert blocked_manifest.blockers[-1] == f"Configured workspace root does not exist: {(temp_dir / 'missing-workspace').resolve()}"

    def test_run_once_no_longer_crashes_and_registers_running_task(self, supervisor, temp_dir):
        """RED: Daemon iteration should stay responsive with a live child process."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        self._write_task(supervisor, manifest, subdir="queue")
        handle = Mock(name="running-handle")

        with patch.object(supervisor.agent_spawner, "launch", return_value=handle) as mock_launch:
            supervisor.run_once()

        mock_launch.assert_called_once()
        assert supervisor.active_tasks["TASK-001"].task_id == "TASK-001"
        assert supervisor.active_agents["TASK-001"] is handle

        dashboard = json.loads((temp_dir / "status" / "dashboard.json").read_text())
        assert dashboard["active_sessions"] == 1
        assert dashboard["sessions"]["TASK-001"]["stage"] == "design"

    def test_run_once_processes_control_before_launching_new_work(self, supervisor):
        """RED: Pause must be applied before the supervisor starts a queued stage."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        self._write_task(supervisor, manifest, subdir="queue")
        command = {"command": "pause", "task_id": "TASK-001"}

        with patch.object(supervisor, "check_control", return_value=[command]), \
             patch.object(supervisor.agent_spawner, "launch") as mock_launch:
            supervisor.run_once()

        mock_launch.assert_not_called()
        assert "TASK-001" in supervisor.paused_tasks
        state, paused_manifest = supervisor.state_manager.find_task("TASK-001")
        assert state == "in-progress"
        assert paused_manifest.current_stage == Stage.DESIGN

    def test_run_once_keeps_dependent_task_queued_until_dependency_is_done(self, supervisor):
        """RED: Queued tasks with unmet dependencies must remain queued until prerequisites complete."""
        base = TaskManifest.create("TASK-BASE", started_by="cli")
        dependent = TaskManifest.create("TASK-DEP", started_by="cli", dependencies=["TASK-BASE"])
        self._write_task(supervisor, base, subdir="queue")
        self._write_task(supervisor, dependent, subdir="queue")

        with patch.object(supervisor.agent_spawner, "launch", return_value=Mock(name="handle")) as mock_launch, \
             patch.object(supervisor.agent_spawner, "poll", return_value=None):
            supervisor.run_once()
            assert mock_launch.call_count == 1

            state_base, _ = supervisor.state_manager.find_task("TASK-BASE")
            state_dep, _ = supervisor.state_manager.find_task("TASK-DEP")
            assert state_base == "in-progress"
            assert state_dep == "queue"

            supervisor.state_manager.move_to_done("TASK-BASE")
            supervisor.run_once()
            assert mock_launch.call_count == 2

        state_dep, _ = supervisor.state_manager.find_task("TASK-DEP")
        assert state_dep == "in-progress"

    def test_run_once_keeps_missing_dependency_task_queued(self, supervisor):
        """RED: Missing dependency ids should remain visible as unresolved queue gating, not be ignored."""
        dependent = TaskManifest.create("TASK-DEP", started_by="cli", dependencies=["TASK-MISSING"])
        self._write_task(supervisor, dependent, subdir="queue")

        with patch.object(supervisor.agent_spawner, "launch") as mock_launch:
            supervisor.run_once()

        mock_launch.assert_not_called()
        state_dep, _ = supervisor.state_manager.find_task("TASK-DEP")
        assert state_dep == "queue"

    def test_run_once_admits_multiple_dependents_deterministically_after_shared_dependency_completes(self, supervisor):
        """RED: Shared dependency release should remain deterministic for multiple queued dependents."""
        base = TaskManifest.create("TASK-BASE", started_by="cli")
        task_b = TaskManifest.create("TASK-B", started_by="cli", dependencies=["TASK-BASE"])
        task_a = TaskManifest.create("TASK-A", started_by="cli", dependencies=["TASK-BASE"])
        self._write_task(supervisor, base, subdir="queue")
        self._write_task(supervisor, task_b, subdir="queue")
        self._write_task(supervisor, task_a, subdir="queue")

        with patch.object(supervisor.agent_spawner, "launch", return_value=Mock(name="handle")) as mock_launch, \
             patch.object(supervisor.agent_spawner, "poll", return_value=None):
            supervisor.run_once()
            supervisor.state_manager.move_to_done("TASK-BASE")
            supervisor.run_once()

        launched_task_ids = [call.args[0].task_id for call in mock_launch.call_args_list]
        assert launched_task_ids == ["TASK-BASE", "TASK-A", "TASK-B"]

    def test_start_task_stage_registers_manifest_and_handle_without_blocking(self, supervisor):
        """RED: Starting a stage should cache manifest and running handle immediately."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        self._write_task(supervisor, manifest)
        handle = Mock(name="running-handle")

        with patch.object(supervisor.agent_spawner, "launch", return_value=handle) as mock_launch, \
             patch.object(supervisor.agent_spawner, "poll") as mock_poll:
            assert supervisor.start_task_stage("TASK-001") is True

        mock_launch.assert_called_once()
        mock_poll.assert_not_called()
        assert supervisor.active_tasks["TASK-001"].task_id == "TASK-001"
        assert supervisor.active_agents["TASK-001"] is handle

    def test_worktree_and_launch_provisions_before_first_stage_launch(self, supervisor, temp_dir):
        """RED: The supervisor must provision or reuse the deterministic worktree before launching a stage."""
        manifest = TaskManifest.create("TASK-378", started_by="cli")
        self._write_task(supervisor, manifest)
        assignment = WorktreeAssignment(
            path=temp_dir / "repo" / ".worktrees" / "TASK-378",
            branch="agent-pipeline/TASK-378",
        )
        assignment.path.mkdir(parents=True)
        handle = Mock(name="running-handle")

        with patch("agent_pipeline.supervisor.discover_repo_root", return_value=temp_dir / "repo"), \
             patch("agent_pipeline.supervisor.ensure_worktree_ready", return_value=ProvisioningResult(
                 outcome=ProvisioningOutcome.CREATE,
                 assignment=assignment,
             )) as mock_provision, \
             patch.object(supervisor.agent_spawner, "launch", return_value=handle) as mock_launch:
            assert supervisor.start_task_stage("TASK-378") is True

        mock_provision.assert_called_once()
        launched_manifest = mock_launch.call_args.args[0]
        assert launched_manifest.task_id == "TASK-378"
        assert mock_launch.call_args.args[1] == Stage.DESIGN
        assert mock_launch.call_args.kwargs["execution_cwd"] == assignment.path
        persisted = supervisor.state_manager.load_task("TASK-378")
        assert persisted.worktree is not None
        assert persisted.worktree.path == str(assignment.path)
        assert persisted.worktree.branch == assignment.branch

    def test_base_dir_and_worktree_derive_repo_local_assignment(self, temp_dir):
        """RED: Overridden state directories must not change repo-local worktree derivation."""
        repo_root = temp_dir / "repo"
        repo_root.mkdir()
        base_dir = temp_dir / "external-state"
        base_dir.mkdir()
        supervisor = Supervisor(base_dir)
        manifest = TaskManifest.create("TASK-378", started_by="cli")
        (base_dir / "in-progress" / "TASK-378").mkdir(parents=True, exist_ok=True)
        self._write_task(supervisor, manifest)
        assignment = WorktreeAssignment(
            path=repo_root / ".worktrees" / "TASK-378",
            branch="agent-pipeline/TASK-378",
        )

        with patch("agent_pipeline.supervisor.discover_repo_root", return_value=repo_root), \
             patch("agent_pipeline.supervisor.ensure_worktree_ready", return_value=ProvisioningResult(
                 outcome=ProvisioningOutcome.REUSE,
                 assignment=assignment,
             )), \
             patch.object(supervisor.agent_spawner, "launch", return_value=Mock()) as mock_launch:
            assert supervisor.start_task_stage("TASK-378") is True

        assert str(base_dir) not in str(mock_launch.call_args.kwargs["execution_cwd"])
        assert mock_launch.call_args.kwargs["execution_cwd"] == assignment.path

    def test_malformed_metadata_and_block_prevents_launch_with_clear_reason(self, supervisor):
        """RED: Malformed persisted worktree metadata must block the task before launch with a supervisor-visible reason."""
        manifest = TaskManifest.create("TASK-378", started_by="cli")
        task_dir = self._write_task(supervisor, manifest)
        manifest_path = task_dir / "manifest.json"
        payload = json.loads(manifest_path.read_text())
        payload["worktree"] = {"path": "relative/path", "branch": 7}
        manifest_path.write_text(json.dumps(payload, indent=2))

        with patch("agent_pipeline.supervisor.discover_repo_root", return_value=supervisor.base_dir.parent), \
             patch.object(supervisor.agent_spawner, "launch") as mock_launch:
            assert supervisor.start_task_stage("TASK-378") is False

        mock_launch.assert_not_called()
        state, blocked_manifest = supervisor.state_manager.find_task("TASK-378")
        assert state == "blocked"
        assert blocked_manifest.blockers[-1] == (
            "invalid persisted worktree metadata: worktree.path must be an absolute path string; "
            "worktree.branch must be a string"
        )

    def test_ignore_and_block_prevents_launch_without_retry_consumption(self, supervisor, temp_dir):
        """RED: Ignore verification failures must block the task instead of scheduling a retry."""
        manifest = TaskManifest.create("TASK-378", started_by="cli")
        self._write_task(supervisor, manifest)
        assignment = WorktreeAssignment(
            path=temp_dir / "repo" / ".worktrees" / "TASK-378",
            branch="agent-pipeline/TASK-378",
        )

        with patch("agent_pipeline.supervisor.discover_repo_root", return_value=temp_dir / "repo"), \
             patch("agent_pipeline.supervisor.ensure_worktree_ready", return_value=ProvisioningResult(
                 outcome=ProvisioningOutcome.BLOCK,
                 assignment=assignment,
                 reason="worktree provisioning blocked: repository must git-ignore .worktrees/ before provisioning",
             )), \
             patch.object(supervisor.agent_spawner, "launch") as mock_launch:
            assert supervisor.start_task_stage("TASK-378") is False

        mock_launch.assert_not_called()
        state, blocked_manifest = supervisor.state_manager.find_task("TASK-378")
        assert state == "blocked"
        assert blocked_manifest.attempts[Stage.DESIGN] == 0
        assert blocked_manifest.blockers[-1] == (
            "worktree provisioning blocked: repository must git-ignore .worktrees/ before provisioning"
        )

    def test_provisioning_and_block_surfaces_clear_supervisor_reason(self, supervisor, temp_dir):
        """RED: Provisioning blockers must surface directly instead of being treated as retryable spawn failures."""
        manifest = TaskManifest.create("TASK-378", started_by="cli")
        self._write_task(supervisor, manifest)
        assignment = WorktreeAssignment(
            path=temp_dir / "repo" / ".worktrees" / "TASK-378",
            branch="agent-pipeline/TASK-378",
        )

        with patch("agent_pipeline.supervisor.discover_repo_root", return_value=temp_dir / "repo"), \
             patch("agent_pipeline.supervisor.ensure_worktree_ready", return_value=ProvisioningResult(
                 outcome=ProvisioningOutcome.BLOCK,
                 assignment=assignment,
                 reason="worktree provisioning blocked: existing worktree branch mismatch",
             )), \
             patch.object(supervisor.agent_spawner, "launch") as mock_launch:
            assert supervisor.start_task_stage("TASK-378") is False

        mock_launch.assert_not_called()
        state, blocked_manifest = supervisor.state_manager.find_task("TASK-378")
        assert state == "blocked"
        assert blocked_manifest.blockers[-1] == "worktree provisioning blocked: existing worktree branch mismatch"

    def test_cwd_and_task_bundle_launches_from_worktree_without_moving_bundle(self, supervisor, temp_dir):
        """RED: Stage execution should use the provisioned worktree as cwd while task artifacts stay in the task bundle."""
        manifest = TaskManifest.create("TASK-378", started_by="cli")
        task_dir = self._write_task(supervisor, manifest)
        assignment = WorktreeAssignment(
            path=temp_dir / "repo" / ".worktrees" / "TASK-378",
            branch="agent-pipeline/TASK-378",
        )
        assignment.path.mkdir(parents=True)

        with patch("agent_pipeline.supervisor.discover_repo_root", return_value=temp_dir / "repo"), \
             patch("agent_pipeline.supervisor.ensure_worktree_ready", return_value=ProvisioningResult(
                 outcome=ProvisioningOutcome.REUSE,
                 assignment=assignment,
             )), \
             patch.object(supervisor.agent_spawner, "launch", return_value=Mock()) as mock_launch:
            assert supervisor.start_task_stage("TASK-378") is True

        assert mock_launch.call_args.kwargs["execution_cwd"] == assignment.path
        assert (task_dir / "manifest.json").exists()
        assert (task_dir / "task.md").exists()

    def test_task_bundle_artifacts_remain_under_agents_not_worktree(self, supervisor, temp_dir):
        """RED: Provisioning must not move or duplicate task-bundle artifacts into the worktree."""
        manifest = TaskManifest.create("TASK-378", started_by="cli")
        task_dir = self._write_task(supervisor, manifest)
        for name in ("design.md", "spec.md", "plan.md", "steering.md"):
            (task_dir / name).write_text(name)
        assignment = WorktreeAssignment(
            path=temp_dir / "repo" / ".worktrees" / "TASK-378",
            branch="agent-pipeline/TASK-378",
        )
        assignment.path.mkdir(parents=True)

        with patch("agent_pipeline.supervisor.discover_repo_root", return_value=temp_dir / "repo"), \
             patch("agent_pipeline.supervisor.ensure_worktree_ready", return_value=ProvisioningResult(
                 outcome=ProvisioningOutcome.REUSE,
                 assignment=assignment,
             )), \
             patch.object(supervisor.agent_spawner, "launch", return_value=Mock()):
            assert supervisor.start_task_stage("TASK-378") is True

        for name in ("manifest.json", "task.md", "design.md", "spec.md", "plan.md", "steering.md"):
            assert (task_dir / name).exists()
            assert not (assignment.path / name).exists()

    def test_start_task_stage_refuses_in_progress_task_with_unmet_dependencies(self, supervisor):
        """RED: Dependency gating should still apply before starting an in-progress task stage."""
        manifest = TaskManifest.create("TASK-DEP", started_by="cli", dependencies=["TASK-BASE"])
        self._write_task(supervisor, manifest)

        with patch.object(supervisor.agent_spawner, "launch") as mock_launch:
            assert supervisor.start_task_stage("TASK-DEP") is False

        mock_launch.assert_not_called()

    def test_run_once_keeps_in_progress_task_idle_until_dependencies_are_satisfied(self, supervisor):
        """RED: The in-progress restart sweep must not bypass dependency gating."""
        manifest = TaskManifest.create("TASK-DEP", started_by="cli", dependencies=["TASK-BASE"])
        self._write_task(supervisor, manifest)

        with patch.object(supervisor.agent_spawner, "launch") as mock_launch, \
             patch.object(supervisor.agent_spawner, "poll", return_value=None):
            supervisor.run_once()

        mock_launch.assert_not_called()

    def test_check_active_tasks_advances_after_polled_success(self, supervisor):
        """RED: Completed work should advance the manifest after polling."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        self._write_task(supervisor, manifest)
        handle = Mock(name="running-handle")
        supervisor.active_tasks[manifest.task_id] = manifest
        supervisor.active_agents[manifest.task_id] = handle

        with patch.object(supervisor.agent_spawner, "poll", return_value=SpawnResult(success=True, exit_code=0)), \
             patch.object(supervisor.agent_spawner, "verify_artifact", return_value=True):
            supervisor.check_active_tasks()

        updated = supervisor.state_manager.load_task("TASK-001")
        assert updated.current_stage == Stage.SPEC_WRITE
        assert "TASK-001" not in supervisor.active_tasks
        assert "TASK-001" not in supervisor.active_agents

    def test_check_active_tasks_retries_after_polled_failure(self, supervisor):
        """RED: Failed work should increment attempts and leave the task in progress."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        self._write_task(supervisor, manifest)
        handle = Mock(name="running-handle")
        supervisor.active_tasks[manifest.task_id] = manifest
        supervisor.active_agents[manifest.task_id] = handle

        with patch.object(
            supervisor.agent_spawner,
            "poll",
            return_value=SpawnResult(success=False, exit_code=1, error="boom", stderr="trace"),
        ):
            supervisor.check_active_tasks()

        updated = supervisor.state_manager.load_task("TASK-001")
        assert updated.current_stage == Stage.DESIGN
        assert updated.attempts[Stage.DESIGN] == 1
        assert "TASK-001" not in supervisor.active_tasks
        assert "TASK-001" not in supervisor.active_agents

    def test_check_active_tasks_blocks_after_max_retries(self, supervisor):
        """RED: Max retries should move the task to blocked once polling reports failure."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        for _ in range(manifest.max_attempts - 1):
            manifest.increment_attempt()
        self._write_task(supervisor, manifest)
        handle = Mock(name="running-handle")
        supervisor.active_tasks[manifest.task_id] = manifest
        supervisor.active_agents[manifest.task_id] = handle

        with patch.object(
            supervisor.agent_spawner,
            "poll",
            return_value=SpawnResult(success=False, exit_code=1, error="boom", stderr="trace"),
        ):
            supervisor.check_active_tasks()

        state, blocked_manifest = supervisor.state_manager.find_task("TASK-001")
        assert state == "blocked"
        assert blocked_manifest.blockers[-1] == "max retries exceeded for design"

    def test_check_active_tasks_completes_final_stage(self, supervisor):
        """RED: Final-stage success should move the task bundle to done."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        manifest.current_stage = Stage.VALIDATE
        self._write_task(supervisor, manifest)
        handle = Mock(name="running-handle")
        supervisor.active_tasks[manifest.task_id] = manifest
        supervisor.active_agents[manifest.task_id] = handle

        with patch.object(supervisor.agent_spawner, "poll", return_value=SpawnResult(success=True, exit_code=0)), \
             patch.object(supervisor.agent_spawner, "verify_artifact", return_value=True):
            supervisor.check_active_tasks()

        state, done_manifest = supervisor.state_manager.find_task("TASK-001")
        assert state == "done"
        assert done_manifest.status.value == "complete"

    def test_abort_terminates_uncached_process_and_moves_task_out_of_in_progress(self, supervisor):
        """RED: Abort must kill real work even when the handle was not cached in memory."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        manifest.current_stage = Stage.IMPL
        task_dir = self._write_task(supervisor, manifest)
        pid_file = task_dir / "impl.pid"
        pid_file.write_text("4321")

        with patch.object(supervisor, "_terminate_pid") as mock_terminate:
            supervisor.process_control_command({"command": "abort", "task_id": "TASK-001"})

        mock_terminate.assert_called_once_with(4321)
        assert pid_file.exists() is False
        state, blocked_manifest = supervisor.state_manager.find_task("TASK-001")
        assert state == "blocked"
        assert blocked_manifest.blockers[-1] == "aborted by user"

    def test_steer_persists_guidance_for_future_stages(self, supervisor):
        """RED: Steering should be written into the task bundle instead of staying in memory only."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        task_dir = self._write_task(supervisor, manifest)

        supervisor.process_control_command({
            "command": "steer",
            "task_id": "TASK-001",
            "message": "Focus on error handling",
        })

        assert (task_dir / "steering.md").read_text() == "Focus on error handling\n"

    def test_check_active_tasks_blocks_verifier_review_findings(self, supervisor):
        """RED: Verifier review artifacts should block instead of retrying the same review stage."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        manifest.current_stage = Stage.SPEC_VERIFY
        task_dir = self._write_task(supervisor, manifest)
        (task_dir / "spec.review").write_text("needs revision")
        handle = Mock(name="running-handle")
        supervisor.active_tasks[manifest.task_id] = manifest
        supervisor.active_agents[manifest.task_id] = handle

        with patch.object(supervisor.agent_spawner, "poll", return_value=SpawnResult(success=True, exit_code=0)), \
             patch.object(supervisor.agent_spawner, "verify_artifact", return_value=True):
            supervisor.check_active_tasks()

        state, blocked_manifest = supervisor.state_manager.find_task("TASK-001")
        assert state == "blocked"
        assert blocked_manifest.blockers[-1] == "spec_verify found issues - see spec.review for details"

    def test_check_active_tasks_blocks_qa_review_findings(self, supervisor):
        """RED: QA review artifacts should block instead of consuming retries."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        manifest.current_stage = Stage.QA
        task_dir = self._write_task(supervisor, manifest)
        (task_dir / "qa.md").write_text("report")
        (task_dir / "qa.review").write_text("needs revision")
        handle = Mock(name="running-handle")
        supervisor.active_tasks[manifest.task_id] = manifest
        supervisor.active_agents[manifest.task_id] = handle

        with patch.object(supervisor.agent_spawner, "poll", return_value=SpawnResult(success=True, exit_code=0)), \
             patch.object(supervisor.agent_spawner, "verify_artifact", return_value=True):
            supervisor.check_active_tasks()

        state, blocked_manifest = supervisor.state_manager.find_task("TASK-001")
        assert state == "blocked"
        assert blocked_manifest.blockers[-1] == "qa found issues - see qa.review for details"

    def test_check_active_tasks_blocks_validate_review_findings(self, supervisor):
        """RED: Validate review artifacts should block instead of retrying validate."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        manifest.current_stage = Stage.VALIDATE
        task_dir = self._write_task(supervisor, manifest)
        (task_dir / "validate.review").write_text("blocked")
        handle = Mock(name="running-handle")
        supervisor.active_tasks[manifest.task_id] = manifest
        supervisor.active_agents[manifest.task_id] = handle

        with patch.object(supervisor.agent_spawner, "poll", return_value=SpawnResult(success=True, exit_code=0)), \
             patch.object(supervisor.agent_spawner, "verify_artifact", return_value=True):
            supervisor.check_active_tasks()

        state, blocked_manifest = supervisor.state_manager.find_task("TASK-001")
        assert state == "blocked"
        assert blocked_manifest.blockers[-1] == "validate found issues - see validate.review for details"

    def test_check_active_tasks_blocks_review_findings_even_on_nonzero_exit(self, supervisor):
        """RED: Review artifacts should fail-close even when the reviewer exits non-zero."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        manifest.current_stage = Stage.QA
        task_dir = self._write_task(supervisor, manifest)
        (task_dir / "qa.md").write_text("report")
        (task_dir / "qa.review").write_text("needs revision")
        handle = Mock(name="running-handle")
        supervisor.active_tasks[manifest.task_id] = manifest
        supervisor.active_agents[manifest.task_id] = handle

        with patch.object(
            supervisor.agent_spawner,
            "poll",
            return_value=SpawnResult(success=False, exit_code=1, error="review failed", stderr="details"),
        ), patch.object(supervisor.agent_spawner, "verify_artifact", return_value=True):
            supervisor.check_active_tasks()

        state, blocked_manifest = supervisor.state_manager.find_task("TASK-001")
        assert state == "blocked"
        assert blocked_manifest.blockers[-1] == "qa found issues - see qa.review for details"
        assert blocked_manifest.attempts[Stage.QA] == 0

    def test_check_active_tasks_does_not_block_on_bare_qa_review_without_report(self, supervisor):
        """RED: Bare qa.review without qa.md should remain a retryable QA failure, not a blocking contract failure."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        manifest.current_stage = Stage.QA
        task_dir = self._write_task(supervisor, manifest)
        (task_dir / "qa.review").write_text("needs revision")
        handle = Mock(name="running-handle")
        supervisor.active_tasks[manifest.task_id] = manifest
        supervisor.active_agents[manifest.task_id] = handle

        with patch.object(
            supervisor.agent_spawner,
            "poll",
            return_value=SpawnResult(success=False, exit_code=1, error="review failed", stderr="details"),
        ), patch.object(supervisor.agent_spawner, "verify_artifact", return_value=False):
            supervisor.check_active_tasks()

        state, updated_manifest = supervisor.state_manager.find_task("TASK-001")
        assert state == "in-progress"
        assert updated_manifest.attempts[Stage.QA] == 1

    def test_get_task_status_reads_persisted_manifest(self, supervisor):
        """RED: Status should reflect persisted task metadata."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        manifest.current_stage = Stage.IMPL
        manifest.increment_attempt()
        self._write_task(supervisor, manifest)

        status = supervisor.get_task_status("TASK-001")

        assert status["task_id"] == "TASK-001"
        assert status["current_stage"] == "impl"
        assert status["attempts"]["impl"] == 1

    def test_get_task_status_surfaces_feedback_resolution_metadata(self, supervisor):
        """RED: Status should expose whether a task has structured review-resolution guidance."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        task_dir = self._write_task(supervisor, manifest, subdir="blocked")
        (task_dir / "feedback-resolution.md").write_text(
            "# Feedback Resolution\n\nSource review artifact: spec.review\n"
        )

        status = supervisor.get_task_status("TASK-001")

        assert status["has_feedback_resolution"] is True
        assert status["feedback_review_artifact"] == "spec.review"

    def test_get_task_status_surfaces_worktree_metadata(self, supervisor, temp_dir):
        """RED: Supervisor status should expose persisted worktree path and branch."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        manifest.set_worktree(
            path=temp_dir / "repo" / ".worktrees" / "TASK-001",
            branch="agent-pipeline/TASK-001",
        )
        self._write_task(supervisor, manifest, subdir="done")

        status = supervisor.get_task_status("TASK-001")

        assert status["worktree_path"] == str(temp_dir / "repo" / ".worktrees" / "TASK-001")
        assert status["worktree_branch"] == "agent-pipeline/TASK-001"

    def test_run_once_restart_reuses_persisted_worktree_metadata_without_rewriting_manifest(self, temp_dir):
        """RED: Restart recovery should reuse persisted worktree metadata instead of mutating it again."""
        supervisor = Supervisor(temp_dir)
        manifest = TaskManifest.create("TASK-RESTART", started_by="cli")
        assignment = WorktreeAssignment(
            path=temp_dir / "repo" / ".worktrees" / "TASK-RESTART",
            branch="agent-pipeline/TASK-RESTART",
        )
        assignment.path.mkdir(parents=True)
        manifest.set_worktree(path=assignment.path, branch=assignment.branch)
        self._write_task(supervisor, manifest)
        original_updated_at = supervisor.state_manager.load_task("TASK-RESTART").updated_at

        with patch("agent_pipeline.supervisor.discover_repo_root", return_value=temp_dir / "repo"), \
             patch("agent_pipeline.supervisor.ensure_worktree_ready", return_value=ProvisioningResult(
                 outcome=ProvisioningOutcome.REUSE,
                 assignment=assignment,
             )), \
             patch.object(supervisor.agent_spawner, "launch", return_value=Mock()):
            assert supervisor.start_task_stage("TASK-RESTART") is True

        persisted = supervisor.state_manager.load_task("TASK-RESTART")
        assert persisted.worktree is not None
        assert persisted.worktree.path == str(assignment.path)
        assert persisted.worktree.branch == assignment.branch
        assert persisted.updated_at == original_updated_at

    def test_start_task_stage_blocks_invalid_persisted_task_id_before_worktree_derivation(self, temp_dir):
        """RED: Restart/provisioning should fail closed when on-disk task ids are not bundle-safe."""
        supervisor = Supervisor(temp_dir)
        task_dir = temp_dir / "in-progress" / "bad-task"
        task_dir.mkdir(parents=True)
        payload = {
            "task_id": "../TASK-BAD",
            "current_stage": "design",
            "status": "in_progress",
            "attempts": {stage.value: 0 for stage in Stage},
            "max_attempts": 5,
            "artifacts": {},
            "blockers": [],
            "dependencies": [],
            "started_by": "cli",
            "created_at": "2026-04-04T00:00:00",
            "updated_at": "2026-04-04T00:00:00",
        }
        (task_dir / "manifest.json").write_text(json.dumps(payload))
        (task_dir / "task.md").write_text("# bad task\n")

        supervisor.run_once()

        blocked_manifest = supervisor.state_manager.load_task_from_path(temp_dir / "blocked" / "bad-task" / "manifest.json")
        assert blocked_manifest.task_id == "../TASK-BAD"
        assert "persisted task id '../TASK-BAD' is invalid" in blocked_manifest.blockers[-1]

    def test_emit_status_includes_active_worktree_metadata(self, supervisor, temp_dir):
        """RED: Dashboard status should preserve active session worktree visibility across restarts."""
        manifest = TaskManifest.create("TASK-STATUS", started_by="cli")
        manifest.set_worktree(
            path=temp_dir / "repo" / ".worktrees" / "TASK-STATUS",
            branch="agent-pipeline/TASK-STATUS",
        )
        supervisor.active_tasks[manifest.task_id] = manifest

        supervisor.emit_status()

        dashboard = json.loads((temp_dir / "status" / "dashboard.json").read_text())
        assert dashboard["sessions"]["TASK-STATUS"]["worktree_path"] == str(temp_dir / "repo" / ".worktrees" / "TASK-STATUS")
        assert dashboard["sessions"]["TASK-STATUS"]["worktree_branch"] == "agent-pipeline/TASK-STATUS"

    def test_read_stage_log_returns_current_stage_stdout_for_in_progress_task(self, supervisor):
        """RED: Operators should be able to inspect current-stage logs while work is still active."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        manifest.current_stage = Stage.DESIGN
        task_dir = self._write_task(supervisor, manifest)
        (task_dir / "design.stdout.log").write_text("line 1\nline 2\n")

        log_output = supervisor.read_stage_log("TASK-001")

        assert log_output == "line 1\nline 2\n"

    def test_read_stage_log_supports_explicit_stage_stream_and_tail(self, supervisor):
        """RED: Log peeking should support stream selection and last-N-line tails."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        manifest.current_stage = Stage.IMPL
        task_dir = self._write_task(supervisor, manifest)
        (task_dir / "impl.stderr.log").write_text("a\nb\nc\n")

        log_output = supervisor.read_stage_log(
            "TASK-001",
            stage=Stage.IMPL,
            stream=StageLogStream.STDERR,
            tail=2,
        )

        assert log_output == "b\nc\n"

    def test_read_stage_log_raises_for_missing_stage_log(self, supervisor):
        """RED: Stages that have not started yet should fail with a concise missing-log error."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        manifest.current_stage = Stage.DESIGN
        self._write_task(supervisor, manifest)

        with pytest.raises(FileNotFoundError, match="No stdout log for TASK-001 stage design yet"):
            supervisor.read_stage_log("TASK-001")

    def test_follow_stage_log_streams_appended_chunks_until_idle_timeout(self, supervisor):
        """RED: Follow mode should yield initial content and later appends from a live log file."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        manifest.current_stage = Stage.DESIGN
        task_dir = self._write_task(supervisor, manifest)
        log_path = task_dir / "design.stdout.log"
        log_path.write_text("start\n")

        def append_later() -> None:
            time.sleep(0.02)
            with log_path.open("a", encoding="utf-8") as handle:
                handle.write("next\n")
                handle.flush()

        writer = threading.Thread(target=append_later)
        writer.start()
        try:
            chunks = list(
                supervisor.follow_stage_log(
                    "TASK-001",
                    poll_interval=0.01,
                    idle_timeout=0.08,
                )
            )
        finally:
            writer.join()

        assert chunks == ["start\n", "next\n"]

    def test_follow_stage_log_with_tail_does_not_skip_bytes_appended_between_snapshot_and_follow(self, supervisor):
        """RED: --follow --tail must not lose bytes written after the initial tail snapshot but before follow resumes."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        manifest.current_stage = Stage.DESIGN
        task_dir = self._write_task(supervisor, manifest)
        log_path = task_dir / "design.stdout.log"
        log_path.write_text("one\ntwo\n")

        stream = supervisor.follow_stage_log(
            "TASK-001",
            tail=1,
            poll_interval=0.01,
            idle_timeout=0.05,
        )

        first_chunk = next(stream)
        with log_path.open("a", encoding="utf-8") as handle:
            handle.write("three\n")
            handle.flush()
        remaining_chunks = list(stream)

        assert first_chunk == "two\n"
        assert remaining_chunks == ["three\n"]
