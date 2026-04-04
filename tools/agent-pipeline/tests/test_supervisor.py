"""Tests for supervisor module."""

import json
import tempfile
from pathlib import Path
from unittest.mock import Mock, patch

import pytest

from agent_pipeline.agents import AgentType, SpawnResult
from agent_pipeline.state import Stage, TaskManifest
from agent_pipeline.supervisor import Supervisor


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
        assert supervisor.agent_spawner.get_agent_type(Stage.DESIGN) == AgentType.CODEX

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
