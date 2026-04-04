"""Tests for agent spawning module."""

import tempfile
from pathlib import Path
from unittest.mock import MagicMock, Mock, patch

import pytest

from agent_pipeline.agents import AgentSpawner, AgentType, RunningAgent, SpawnResult
from agent_pipeline.state import TaskManifest, Stage


class TestAgentSpawner:
    """Test agent spawning functionality."""

    @pytest.fixture
    def temp_dir(self):
        """Provide temporary directory."""
        with tempfile.TemporaryDirectory() as tmp:
            yield Path(tmp)

    @pytest.fixture
    def spawner(self, temp_dir):
        """Provide AgentSpawner."""
        workspace_root = temp_dir / "workspace"
        workspace_root.mkdir()
        return AgentSpawner(
            temp_dir,
            workspace_root=workspace_root,
            codex_executable="custom-codex",
        )

    @pytest.fixture
    def sample_manifest(self):
        """Provide sample task manifest."""
        return TaskManifest.create("TASK-001", started_by="cli")

    def test_get_agent_type_for_stage(self, spawner):
        """RED: Should map stages to agent types."""
        assert spawner.get_agent_type(Stage.DESIGN) == AgentType.CODEX
        assert spawner.get_agent_type(Stage.SPEC_WRITE) == AgentType.HERMES
        assert spawner.get_agent_type(Stage.SPEC_VERIFY) == AgentType.CODEX
        assert spawner.get_agent_type(Stage.PLAN_WRITE) == AgentType.HERMES
        assert spawner.get_agent_type(Stage.PLAN_VERIFY) == AgentType.CODEX
        assert spawner.get_agent_type(Stage.IMPL) == AgentType.HERMES
        assert spawner.get_agent_type(Stage.QA) == AgentType.HERMES
        assert spawner.get_agent_type(Stage.VALIDATE) == AgentType.CODEX

    def test_partial_stage_agent_override_replaces_selected_stages_only(self, temp_dir):
        """RED: Runtime overrides should replace selected stages while preserving defaults elsewhere."""
        workspace_root = temp_dir / "workspace"
        workspace_root.mkdir()
        spawner = AgentSpawner(
            temp_dir,
            workspace_root=workspace_root,
            stage_agents={"qa": "codex", "design": "hermes"},
        )

        assert spawner.get_agent_type(Stage.QA) == AgentType.CODEX
        assert spawner.get_agent_type(Stage.DESIGN) == AgentType.HERMES
        assert spawner.get_agent_type(Stage.SPEC_WRITE) == AgentType.HERMES
        assert spawner.get_agent_type(Stage.VALIDATE) == AgentType.CODEX

    def test_invalid_stage_agent_override_rejects_unknown_stage(self, temp_dir):
        """RED: Unknown stage names should fail clearly."""
        workspace_root = temp_dir / "workspace"
        workspace_root.mkdir()

        with pytest.raises(ValueError, match="Unknown stage"):
            AgentSpawner(temp_dir, workspace_root=workspace_root, stage_agents={"bogus": "codex"})

    def test_invalid_stage_agent_override_rejects_unknown_agent(self, temp_dir):
        """RED: Unknown agent names should fail clearly."""
        workspace_root = temp_dir / "workspace"
        workspace_root.mkdir()

        with pytest.raises(ValueError, match="Unknown agent"):
            AgentSpawner(temp_dir, workspace_root=workspace_root, stage_agents={"qa": "bogus"})

    def test_build_codex_prompt_for_design(self, spawner, sample_manifest, temp_dir):
        """RED: Should build appropriate codex prompt."""
        # Create task directory structure
        task_dir = temp_dir / "in-progress" / "TASK-001"
        task_dir.mkdir(parents=True)
        (task_dir / "task.md").write_text("# Design task")
        
        prompt = spawner._build_codex_prompt(sample_manifest, Stage.DESIGN)
        
        assert "codex" in prompt.lower() or "yolo" in prompt.lower()
        assert "TASK-001" in prompt
        assert "design" in prompt.lower()

    def test_build_hermes_prompt_for_impl(self, spawner, sample_manifest, temp_dir):
        """RED: Should build appropriate hermes prompt."""
        sample_manifest.current_stage = Stage.IMPL
        
        task_dir = temp_dir / "in-progress" / "TASK-001"
        task_dir.mkdir(parents=True)
        
        prompt = spawner._build_hermes_prompt(sample_manifest, Stage.IMPL)
        
        assert "TASK-001" in prompt
        assert "implement" in prompt.lower()
        assert "rust" in prompt.lower() or "test" in prompt.lower()

    def test_design_prompt_includes_contract_sections_and_verdict_vocabulary(self, spawner, sample_manifest, temp_dir):
        """RED: Design prompts should include reusable contract language and traceability sections."""
        task_dir = temp_dir / "in-progress" / "TASK-001"
        task_dir.mkdir(parents=True)
        (task_dir / "task.md").write_text("# Design task")

        prompt = spawner._build_codex_prompt(sample_manifest, Stage.DESIGN)

        assert "Non-goals" in prompt
        assert "Assumptions" in prompt
        assert "Traceability" in prompt
        assert "VERIFIED" in prompt
        assert "BLOCKED" in prompt
        assert "NEEDS_REVISION" in prompt

    def test_spec_and_plan_prompts_require_traceability_and_doc_quality_lenses(self, spawner, sample_manifest, temp_dir):
        """RED: Spec/plan prompts should reuse document-quality and traceability templates."""
        task_dir = temp_dir / "in-progress" / "TASK-001"
        task_dir.mkdir(parents=True)

        spec_prompt = spawner._build_hermes_prompt(sample_manifest, Stage.SPEC_WRITE)
        plan_prompt = spawner._build_hermes_prompt(sample_manifest, Stage.PLAN_WRITE)

        assert "Technical Critic" in spec_prompt
        assert "Pedagogical Critic" in spec_prompt
        assert "Style Critic" in spec_prompt
        assert "Traceability Matrix" in spec_prompt
        assert "exact file paths" in plan_prompt.lower()
        assert "verification commands" in plan_prompt.lower()
        assert "requirement-to-task traceability" in plan_prompt.lower()

    def test_impl_prompt_requires_evidence_artifacts(self, spawner, sample_manifest, temp_dir):
        """RED: Implementation prompt should require summary and verification evidence artifacts."""
        task_dir = temp_dir / "in-progress" / "TASK-001"
        task_dir.mkdir(parents=True)

        prompt = spawner._build_hermes_prompt(sample_manifest, Stage.IMPL)

        assert "impl.summary.md" in prompt
        assert "impl.verification.md" in prompt
        assert "evidence before claims" in prompt.lower()

    def test_qa_and_validate_prompts_support_fail_closed_artifacts(self, spawner, sample_manifest, temp_dir):
        """RED: QA and validate prompts should use pass/fail artifacts and explicit verdict sections."""
        task_dir = temp_dir / "in-progress" / "TASK-001"
        task_dir.mkdir(parents=True)

        qa_prompt = spawner._build_hermes_prompt(sample_manifest, Stage.QA)
        validate_prompt = spawner._build_codex_prompt(sample_manifest, Stage.VALIDATE)

        assert "qa.verified" in qa_prompt
        assert "qa.review" in qa_prompt
        assert "Spec compliance" in qa_prompt
        assert "Code quality" in qa_prompt
        assert "Verification evidence" in qa_prompt
        assert "validate.review" in validate_prompt
        assert "VERIFIED" in validate_prompt
        assert "BLOCKED" in validate_prompt

    def test_qa_prompt_reads_workspace_and_impl_evidence_not_nonexistent_impl_dir(self, spawner, sample_manifest, temp_dir):
        """RED: QA should consume workspace code plus impl evidence artifacts, not a synthetic impl/ directory."""
        task_dir = temp_dir / "in-progress" / "TASK-001"
        task_dir.mkdir(parents=True)

        qa_prompt = spawner._build_hermes_prompt(sample_manifest, Stage.QA)

        assert f"Read plan: {task_dir}/plan.md" in qa_prompt
        assert f"Read implementation summary: {task_dir}/impl.summary.md" in qa_prompt
        assert f"Read verification evidence: {task_dir}/impl.verification.md" in qa_prompt
        assert "/impl/" not in qa_prompt

    @patch("subprocess.Popen")
    def test_launch_returns_handle_immediately(self, mock_popen, spawner, sample_manifest, temp_dir):
        """RED: Should launch non-blocking and write a pid file."""
        mock_process = MagicMock()
        mock_process.pid = 4321
        mock_process.poll.return_value = None
        mock_popen.return_value = mock_process

        task_dir = temp_dir / "in-progress" / "TASK-001"
        task_dir.mkdir(parents=True)
        (task_dir / "task.md").write_text("# Task")

        handle = spawner.launch(sample_manifest, Stage.DESIGN)

        assert isinstance(handle, RunningAgent)
        assert handle.process is mock_process
        assert handle.pid == 4321
        assert handle.pid_file.read_text().strip() == "4321"
        assert handle.cwd == temp_dir / "workspace"
        mock_popen.assert_called_once()
        call_args = mock_popen.call_args
        assert call_args.kwargs["cwd"] == temp_dir / "workspace"
        assert call_args.args[0][0] == "custom-codex"

    @patch("subprocess.Popen")
    def test_poll_returns_none_while_process_is_running(self, mock_popen, spawner, sample_manifest, temp_dir):
        """RED: Should leave pid files in place until the process exits."""
        mock_process = MagicMock()
        mock_process.pid = 4321
        mock_process.poll.return_value = None
        mock_popen.return_value = mock_process

        task_dir = temp_dir / "in-progress" / "TASK-001"
        task_dir.mkdir(parents=True)
        (task_dir / "task.md").write_text("# Task")

        handle = spawner.launch(sample_manifest, Stage.DESIGN)

        assert spawner.poll(handle) is None
        assert handle.pid_file.exists()

    @patch("subprocess.Popen")
    def test_poll_reaps_successful_process(self, mock_popen, spawner, sample_manifest, temp_dir):
        """RED: Should return a SpawnResult only after exit and clear pid files."""
        mock_process = MagicMock()
        mock_process.pid = 4321
        mock_process.poll.return_value = 0
        mock_process.communicate.return_value = ("ok", "")
        mock_process.returncode = 0
        mock_popen.return_value = mock_process

        task_dir = temp_dir / "in-progress" / "TASK-001"
        task_dir.mkdir(parents=True)
        (task_dir / "task.md").write_text("# Task")
        (task_dir / "design.md").write_text("artifact")

        handle = spawner.launch(sample_manifest, Stage.DESIGN)
        result = spawner.poll(handle)

        assert result == SpawnResult(success=True, exit_code=0, stdout="ok", stderr="", error=None)
        assert handle.pid_file.exists() is False

    @patch("subprocess.Popen")
    def test_reap_waits_for_process_exit(self, mock_popen, spawner, sample_manifest, temp_dir):
        """RED: Should support explicit reap after a non-blocking launch."""
        mock_process = MagicMock()
        mock_process.pid = 4321
        mock_process.wait.return_value = 0
        mock_process.communicate.return_value = ("done", "")
        mock_process.returncode = 0
        mock_popen.return_value = mock_process

        task_dir = temp_dir / "in-progress" / "TASK-001"
        task_dir.mkdir(parents=True)
        (task_dir / "task.md").write_text("# Task")
        (task_dir / "design.md").write_text("artifact")

        handle = spawner.launch(sample_manifest, Stage.DESIGN)
        result = spawner.reap(handle)

        assert result.success is True
        assert result.stdout == "done"
        assert handle.pid_file.exists() is False

    @patch("subprocess.Popen")
    def test_poll_reaps_failure_process(self, mock_popen, spawner, sample_manifest, temp_dir):
        """RED: Should surface exit failures from the reap path."""
        mock_process = MagicMock()
        mock_process.pid = 4321
        mock_process.poll.return_value = 1
        mock_process.communicate.return_value = ("", "Error occurred")
        mock_process.returncode = 1
        mock_popen.return_value = mock_process

        task_dir = temp_dir / "in-progress" / "TASK-001"
        task_dir.mkdir(parents=True)
        (task_dir / "task.md").write_text("# Task")

        handle = spawner.launch(sample_manifest, Stage.DESIGN)
        result = spawner.poll(handle)

        assert result.success is False
        assert result.exit_code == 1
        assert "exit code" in result.error.lower()
        assert result.stderr == "Error occurred"

    @patch("subprocess.Popen")
    def test_launch_uses_configured_workspace_root_and_executable(self, mock_popen, sample_manifest, temp_dir):
        """RED: Should use configured runtime paths instead of a hard-coded repo path."""
        workspace_root = temp_dir / "custom-workspace"
        workspace_root.mkdir()
        spawner = AgentSpawner(
            temp_dir,
            workspace_root=workspace_root,
            codex_executable="alt-codex",
        )
        mock_process = MagicMock()
        mock_process.pid = 555
        mock_process.poll.return_value = None
        mock_popen.return_value = mock_process

        task_dir = temp_dir / "in-progress" / "TASK-001"
        task_dir.mkdir(parents=True)
        (task_dir / "task.md").write_text("# Task")

        spawner.launch(sample_manifest, Stage.DESIGN)

        call_args = mock_popen.call_args
        assert call_args.kwargs["cwd"] == workspace_root
        assert call_args.args[0][0] == "alt-codex"
        assert str(workspace_root) in call_args.args[0]

    @patch("subprocess.Popen")
    def test_spawn_wrapper_reuses_launch_and_reap(self, mock_popen, spawner, sample_manifest, temp_dir):
        """RED: Legacy blocking spawn should still work through the new lifecycle."""
        mock_process = MagicMock()
        mock_process.pid = 4321
        mock_process.wait.return_value = 0
        mock_process.communicate.return_value = ("done", "")
        mock_process.returncode = 0
        mock_popen.return_value = mock_process

        task_dir = temp_dir / "in-progress" / "TASK-001"
        task_dir.mkdir(parents=True)
        (task_dir / "task.md").write_text("# Task")
        (task_dir / "design.md").write_text("artifact")

        result = spawner.spawn(sample_manifest, Stage.DESIGN)

        assert result.success is True
        assert result.stdout == "done"

    def test_spawn_result_dataclass(self):
        """RED: SpawnResult should store result data."""
        result = SpawnResult(
            success=True,
            exit_code=0,
            stdout="Output",
            stderr="",
            error=None
        )
        
        assert result.success is True
        assert result.exit_code == 0
        assert result.stdout == "Output"

    def test_spawn_result_failure(self):
        """RED: SpawnResult should store failure data."""
        result = SpawnResult(
            success=False,
            exit_code=1,
            stdout="",
            stderr="Error",
            error="Process failed"
        )
        
        assert result.success is False
        assert result.error == "Process failed"

    def test_verify_artifact_exists(self, spawner, sample_manifest, temp_dir):
        """RED: Should verify artifact file exists."""
        task_dir = temp_dir / "in-progress" / "TASK-001"
        task_dir.mkdir(parents=True)

        (task_dir / "design.md").write_text("Done")
        
        assert spawner.verify_artifact(sample_manifest, Stage.DESIGN) is True

    def test_verify_artifact_missing(self, spawner, sample_manifest, temp_dir):
        """RED: Should report missing artifact."""
        task_dir = temp_dir / "in-progress" / "TASK-001"
        task_dir.mkdir(parents=True)
        
        assert spawner.verify_artifact(sample_manifest, Stage.DESIGN) is False

    def test_verify_artifact_requires_impl_evidence_bundle(self, spawner, sample_manifest, temp_dir):
        """RED: Impl success should require summary and verification evidence, not only a marker."""
        task_dir = temp_dir / "in-progress" / "TASK-001"
        task_dir.mkdir(parents=True)

        assert spawner.verify_artifact(sample_manifest, Stage.IMPL) is False

        (task_dir / "impl.complete").write_text("done")
        assert spawner.verify_artifact(sample_manifest, Stage.IMPL) is False

        (task_dir / "impl.summary.md").write_text("summary")
        assert spawner.verify_artifact(sample_manifest, Stage.IMPL) is False

        (task_dir / "impl.verification.md").write_text("verification")
        assert spawner.verify_artifact(sample_manifest, Stage.IMPL) is True

    def test_verify_artifact_accepts_qa_review_or_verified_with_report(self, spawner, sample_manifest, temp_dir):
        """RED: QA should require both qa.md and a pass/fail verdict artifact."""
        task_dir = temp_dir / "in-progress" / "TASK-001"
        task_dir.mkdir(parents=True)

        assert spawner.verify_artifact(sample_manifest, Stage.QA) is False

        (task_dir / "qa.md").write_text("report")
        assert spawner.verify_artifact(sample_manifest, Stage.QA) is False

        (task_dir / "qa.review").write_text("needs work")
        assert spawner.verify_artifact(sample_manifest, Stage.QA) is True

    def test_verify_artifact_accepts_validate_review(self, spawner, sample_manifest, temp_dir):
        """RED: Validate should accept either a success or fail-closed artifact."""
        task_dir = temp_dir / "in-progress" / "TASK-001"
        task_dir.mkdir(parents=True)

        (task_dir / "validate.review").write_text("blocked")

        assert spawner.verify_artifact(sample_manifest, Stage.VALIDATE) is True

    @patch("subprocess.Popen")
    def test_kill_agent_removes_pid_file(self, mock_popen, spawner, sample_manifest, temp_dir):
        """RED: Should kill a running agent and clean its pid file."""
        mock_process = Mock()
        mock_process.pid = 12345
        mock_process.poll.return_value = None
        mock_popen.return_value = mock_process

        task_dir = temp_dir / "in-progress" / "TASK-001"
        task_dir.mkdir(parents=True)
        (task_dir / "task.md").write_text("# Task")

        handle = spawner.launch(sample_manifest, Stage.DESIGN)

        with patch("os.kill") as mock_kill:
            spawner.kill_agent(handle)
            mock_kill.assert_called_once_with(12345, pytest.approx(15))  # SIGTERM
        assert handle.pid_file.exists() is False
