"""Agent spawning and management."""

from __future__ import annotations

import os
import signal
import subprocess
from collections.abc import Mapping
from dataclasses import dataclass
from enum import Enum
from pathlib import Path

from agent_pipeline.prompt_contracts import (
    design_contract,
    document_quality_contract,
    implementation_contract,
    qa_contract,
    validate_contract,
)
from agent_pipeline.state import Stage, TaskManifest


class AgentType(Enum):
    """Types of agents that can spawn."""
    
    CODEX = "codex"
    HERMES = "hermes"


@dataclass
class SpawnResult:
    """Result of agent spawn operation."""
    
    success: bool
    exit_code: int
    stdout: str = ""
    stderr: str = ""
    error: str | None = None


@dataclass
class RunningAgent:
    """Handle for an active agent process."""

    task_id: str
    stage: Stage
    agent_type: AgentType
    process: subprocess.Popen[str]
    pid: int
    pid_file: Path
    task_dir: Path
    cwd: Path
    command: tuple[str, ...]
    result: SpawnResult | None = None


class AgentSpawner:
    """Spawns and manages agent processes."""
    
    DEFAULT_STAGE_AGENTS: dict[Stage, AgentType] = {
        Stage.DESIGN: AgentType.CODEX,
        Stage.SPEC_WRITE: AgentType.HERMES,
        Stage.SPEC_VERIFY: AgentType.CODEX,
        Stage.PLAN_WRITE: AgentType.HERMES,
        Stage.PLAN_VERIFY: AgentType.CODEX,
        Stage.IMPL: AgentType.HERMES,
        Stage.QA: AgentType.HERMES,
        Stage.VALIDATE: AgentType.CODEX,
    }
    
    # Default timeout for agent execution (1 hour)
    DEFAULT_TIMEOUT = 3600
    TERMINATE_TIMEOUT = 5
    
    def __init__(
        self,
        base_dir: Path,
        *,
        workspace_root: Path | None = None,
        codex_executable: str = "codex",
        stage_agents: Mapping[Stage | str, AgentType | str] | None = None,
    ):
        self.base_dir = Path(base_dir)
        self.workspace_root = self._resolve_workspace_root(workspace_root)
        self.codex_executable = codex_executable
        self.stage_agents = self.resolve_stage_agents(stage_agents)

    @classmethod
    def resolve_stage_agents(
        cls,
        stage_agents: Mapping[Stage | str, AgentType | str] | None = None,
    ) -> dict[Stage, AgentType]:
        """Resolve the effective stage-agent mapping, applying validated overrides."""
        resolved = dict(cls.DEFAULT_STAGE_AGENTS)
        if stage_agents is None:
            return resolved

        for raw_stage, raw_agent in stage_agents.items():
            stage = cls._normalize_stage(raw_stage)
            agent = cls._normalize_agent(raw_agent)
            resolved[stage] = agent

        return resolved

    @staticmethod
    def _normalize_stage(stage: Stage | str) -> Stage:
        """Normalize a stage override key to a Stage enum."""
        if isinstance(stage, Stage):
            return stage

        try:
            return Stage(stage)
        except ValueError as exc:
            valid_stages = ", ".join(member.value for member in Stage)
            raise ValueError(f"Unknown stage: {stage}. Valid stages: {valid_stages}") from exc

    @staticmethod
    def _normalize_agent(agent: AgentType | str) -> AgentType:
        """Normalize an agent override value to an AgentType enum."""
        if isinstance(agent, AgentType):
            return agent

        try:
            return AgentType(agent)
        except ValueError as exc:
            valid_agents = ", ".join(member.value for member in AgentType)
            raise ValueError(f"Unknown agent: {agent}. Valid agents: {valid_agents}") from exc

    def _resolve_workspace_root(self, configured_root: Path | None) -> Path:
        """Resolve the workspace root from config or repository discovery."""
        if configured_root is not None:
            return Path(configured_root)

        discovered = self._discover_repo_root(self.base_dir)
        if discovered is not None:
            return discovered

        module_root = self._discover_repo_root(Path(__file__).resolve())
        if module_root is not None:
            return module_root

        return self.base_dir

    @staticmethod
    def _discover_repo_root(start: Path) -> Path | None:
        """Walk upward from a path until a repository root is found."""
        current = start.resolve()
        if current.is_file():
            current = current.parent

        for candidate in (current, *current.parents):
            if (candidate / ".git").exists():
                return candidate
            if (candidate / "AGENTS.md").exists() and (candidate / "Cargo.toml").exists():
                return candidate

        return None
    
    def get_agent_type(self, stage: Stage) -> AgentType:
        """Get agent type for stage."""
        return self.stage_agents[stage]
    
    def spawn(self, manifest: TaskManifest, stage: Stage) -> SpawnResult:
        """Blocking compatibility wrapper over the non-blocking lifecycle."""
        try:
            handle = self.launch(manifest, stage)
            return self.reap(handle)
        except FileNotFoundError as e:
            return SpawnResult(success=False, exit_code=-1, error=f"Agent not found: {e}")
        except Exception as e:
            return SpawnResult(success=False, exit_code=-1, error=str(e))

    def launch(self, manifest: TaskManifest, stage: Stage) -> RunningAgent:
        """Launch an agent process and return its running handle immediately."""
        task_dir = self._get_task_dir(manifest)
        task_dir.mkdir(parents=True, exist_ok=True)

        if self.is_agent_running(manifest, stage):
            raise RuntimeError("Agent already running for this stage")

        command, cwd, agent_type = self._prepare_command(manifest, stage)
        process = subprocess.Popen(
            command,
            cwd=cwd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )

        pid_file = self._get_pid_file(manifest, stage)
        pid_file.write_text(str(process.pid))

        return RunningAgent(
            task_id=manifest.task_id,
            stage=stage,
            agent_type=agent_type,
            process=process,
            pid=process.pid,
            pid_file=pid_file,
            task_dir=task_dir,
            cwd=cwd,
            command=tuple(command),
        )

    def poll(self, handle: RunningAgent) -> SpawnResult | None:
        """Poll a running agent and return a result only after it exits."""
        if handle.result is not None:
            return handle.result

        if handle.process.poll() is None:
            return None

        return self._finalize_handle(handle)

    def reap(self, handle: RunningAgent) -> SpawnResult:
        """Wait for agent completion and return the terminal result."""
        if handle.result is not None:
            return handle.result

        try:
            if handle.process.poll() is None:
                handle.process.wait(timeout=self.DEFAULT_TIMEOUT)
        except subprocess.TimeoutExpired:
            self.kill_agent(handle)
            result = SpawnResult(
                success=False,
                exit_code=-1,
                error=f"Timeout after {self.DEFAULT_TIMEOUT}s",
            )
            handle.result = result
            return result

        return self._finalize_handle(handle)
    
    def _get_task_dir(self, manifest: TaskManifest) -> Path:
        """Get task working directory."""
        return self.base_dir / "in-progress" / manifest.task_id

    @staticmethod
    def _build_guidance_context(task_dir: Path) -> str:
        """Return prompt text for optional operator steering guidance."""
        guidance_file = task_dir / "steering.md"
        if not guidance_file.exists():
            return ""

        return f"""

Operator guidance is available at: {guidance_file}
Read and incorporate it before you act.
"""
    
    def _build_codex_prompt(
        self,
        manifest: TaskManifest,
        stage: Stage,
        attempt: int = 0,
        task_dir: Path | None = None,
    ) -> str:
        """Build prompt for codex agent."""
        if task_dir is None:
            task_dir = self._get_task_dir(manifest)

        guidance_context = self._build_guidance_context(task_dir)
        retry_context = ""
        if attempt > 0:
            retry_context = (
                f"\n⚠️  RETRY ATTEMPT {attempt}/5\n"
                "Previous attempt(s) failed. Common issues:\n"
                "- Missing output files\n"
                "- Incomplete analysis\n"
                "- Wrong file format\n\n"
                "Please ensure you create ALL required output files.\n\n"
            )

        prompts = {
            Stage.DESIGN: f"""You are the Designer agent for the Ash workflow language.

Task: {manifest.task_id}
Stage: {stage.value}
{retry_context}
Your job: Create a design document analyzing the problem space.

CRITICAL: You MUST create the output file: {task_dir}/design.md

Read the task file at: {task_dir}/task.md
Write design to: {task_dir}/design.md

Include:
1. Problem statement
2. Analysis approach
3. Success criteria
4. Risk assessment

{design_contract(task_dir)}

Use --yolo mode. Work autonomously.""",
            Stage.SPEC_VERIFY: f"""You are the Spec Verifier agent for the Ash workflow language.

Task: {manifest.task_id}
Stage: {stage.value}
{retry_context}
Your job: Verify the spec document against requirements.

CRITICAL: You MUST create one output file:
- If pass: {task_dir}/spec.verified
- If fail: {task_dir}/spec.review

Read:
- Design: {task_dir}/design.md
- Spec: {task_dir}/spec.md

Check:
1. All Design requirements captured?
2. No contradictions with existing SPECs?
3. Completion criteria measurable?
4. Error cases documented?

{document_quality_contract(task_dir)}

Write the review so the final verdict clearly uses VERIFIED, BLOCKED, or NEEDS_REVISION.

Be pedantic. Find all issues.""",
            Stage.PLAN_VERIFY: f"""You are the Plan Verifier agent.

Task: {manifest.task_id}
Stage: {stage.value}
{retry_context}
Verify the implementation plan against the spec.

CRITICAL: You MUST create one output file:
- If pass: {task_dir}/plan.verified
- If fail: {task_dir}/plan.review

Read:
- Spec: {task_dir}/spec.md
- Plan: {task_dir}/plan.md

Check traceability: Design req → Spec section → Plan task → Test case

{document_quality_contract(task_dir)}

Write the review so the final verdict clearly uses VERIFIED, BLOCKED, or NEEDS_REVISION.
""",
            Stage.VALIDATE: f"""You are the Final Validator agent.

Task: {manifest.task_id}
Stage: {stage.value}
{retry_context}
Perform final validation before marking complete.

{validate_contract(task_dir)}

Check:
1. All stages completed?
2. All artifacts present?
3. Tests passing?
4. Documentation updated?
""",
        }

        prompt = prompts.get(stage, f"Verify {stage.value} for {manifest.task_id}")
        return f"{prompt}{guidance_context}"
    
    def _spawn_codex_for_hermes_task(self, manifest: TaskManifest, stage: Stage, prompt: str) -> tuple[list[str], Path]:
        """Build the temporary codex stand-in command for hermes tasks."""
        cmd = [
            self.codex_executable,
            "exec",
            "--yolo",
            "-C", str(self.workspace_root),
            prompt,
        ]

        return cmd, self.workspace_root
    
    def _build_hermes_prompt(
        self,
        manifest: TaskManifest,
        stage: Stage,
        attempt: int = 0,
        task_dir: Path | None = None,
    ) -> str:
        """Build prompt for hermes agent."""
        if task_dir is None:
            task_dir = self._get_task_dir(manifest)

        guidance_context = self._build_guidance_context(task_dir)
        retry_context = ""
        if attempt > 0:
            retry_context = (
                f"\n⚠️  RETRY ATTEMPT {attempt}/5\n"
                "Previous attempt failed. Ensure you create ALL required output files.\n\n"
            )

        prompts = {
            Stage.SPEC_WRITE: f"""Load /write-docs-workflow skill.

Task: Write SPEC for {manifest.task_id}
{retry_context}CRITICAL: You MUST create the output file: {task_dir}/spec.md

Read design: {task_dir}/design.md
Write spec to: {task_dir}/spec.md

Follow SPEC template. Include all sections.

{document_quality_contract(task_dir)}
""",
            Stage.PLAN_WRITE: f"""Load /plan skill.

Task: Create implementation plan for {manifest.task_id}
{retry_context}CRITICAL: You MUST create the output file: {task_dir}/plan.md

Read spec: {task_dir}/spec.md
Write plan to: {task_dir}/plan.md

Break into actionable tasks with estimates.

{document_quality_contract(task_dir)}
Be explicit about exact file paths, verification commands, and requirement-to-task traceability.
""",
            Stage.IMPL: f"""Load /rust-skills, /test-driven-development, /verification-before-completion skills.

Task: Implement {manifest.task_id}
{retry_context}{implementation_contract(task_dir)}

Read:
- Task: {task_dir}/task.md
- Plan: {task_dir}/plan.md

Implement following TDD:
1. Write failing test
2. Make it pass
3. Refactor

Update CHANGELOG.md
All tests must pass.""",
            Stage.QA: f"""Task: QA for {manifest.task_id}
{retry_context}{qa_contract(task_dir)}

Read:
- Task: {task_dir}/task.md
- Plan: {task_dir}/plan.md
- Read plan: {task_dir}/plan.md
- Read implementation summary: {task_dir}/impl.summary.md
- Read verification evidence: {task_dir}/impl.verification.md
- Review the implementation directly in the repository workspace at: {self.workspace_root}

Run:
- cargo test
- cargo clippy
- cargo fmt --check
- cargo doc

Write QA report to: {task_dir}/qa.md

Report any issues found.""",
        }

        prompt = prompts.get(stage, f"Implement {stage.value} for {manifest.task_id}")
        return f"{prompt}{guidance_context}"
    
    def _prepare_command(self, manifest: TaskManifest, stage: Stage) -> tuple[list[str], Path, AgentType]:
        """Prepare the command, working directory, and agent type for a stage."""
        agent_type = self.get_agent_type(stage)

        if agent_type == AgentType.CODEX:
            cmd, cwd = self._prepare_codex_command(manifest, stage)
        else:
            cmd, cwd = self._prepare_hermes_command(manifest, stage)

        return cmd, cwd, agent_type

    def _prepare_codex_command(self, manifest: TaskManifest, stage: Stage) -> tuple[list[str], Path]:
        """Build the codex command for a stage."""
        task_dir = self._get_task_dir(manifest)
        task_dir.mkdir(parents=True, exist_ok=True)

        # Create default task.md if missing
        task_file = task_dir / "task.md"
        if not task_file.exists():
            task_file.write_text(f"# {manifest.task_id}\n\nTask description not provided.\nPlease create appropriate artifacts for this task.\n")

        # Get attempt number for retry context
        attempt = manifest.attempts.get(stage, 0)

        # Use absolute path for clarity
        abs_task_dir = task_dir.absolute()
        prompt = self._build_codex_prompt(manifest, stage, attempt, abs_task_dir)

        cmd = [
            self.codex_executable,
            "exec",
            "--yolo",
            "-C", str(self.workspace_root),
            prompt,
        ]

        return cmd, self.workspace_root

    def _prepare_hermes_command(self, manifest: TaskManifest, stage: Stage) -> tuple[list[str], Path]:
        """Build the temporary hermes stand-in command."""
        task_dir = self._get_task_dir(manifest)
        task_dir.mkdir(parents=True, exist_ok=True)

        # Get attempt number for retry context
        attempt = manifest.attempts.get(stage, 0)
        abs_task_dir = task_dir.absolute()
        prompt = self._build_hermes_prompt(manifest, stage, attempt, abs_task_dir)

        # Write prompt to file for reference
        prompt_file = task_dir / f"{stage.value}_prompt.txt"
        prompt_file.write_text(prompt)

        # Run hermes via CLI if available, otherwise fail with instructions
        # For now, use codex as a stand-in for hermes tasks
        return self._spawn_codex_for_hermes_task(manifest, stage, prompt)
    
    def _get_pid_file(self, manifest: TaskManifest, stage: Stage) -> Path:
        """Get path to PID file for tracking running agents."""
        task_dir = self._get_task_dir(manifest)
        return task_dir / f"{stage.value}.pid"
    
    def is_agent_running(self, manifest: TaskManifest, stage: Stage) -> bool:
        """Check if an agent is already running for this task/stage."""
        pid_file = self._get_pid_file(manifest, stage)
        
        if not pid_file.exists():
            return False
        
        try:
            pid = int(pid_file.read_text().strip())
            # Check if process exists
            import os
            os.kill(pid, 0)  # Signal 0 checks if process exists
            return True
        except (ValueError, OSError, ProcessLookupError):
            # PID file exists but process is dead
            pid_file.unlink(missing_ok=True)
            return False
    
    def _finalize_handle(self, handle: RunningAgent) -> SpawnResult:
        """Collect process output, clear pid tracking, and build the final result."""
        if handle.result is not None:
            return handle.result

        stdout, stderr = handle.process.communicate()
        handle.pid_file.unlink(missing_ok=True)

        result = SpawnResult(
            success=handle.process.returncode == 0,
            exit_code=handle.process.returncode,
            stdout=stdout,
            stderr=stderr,
            error=None if handle.process.returncode == 0 else f"Exit code {handle.process.returncode}",
        )
        handle.result = result
        return result
    
    def verify_artifact(self, manifest: TaskManifest, stage: Stage) -> bool:
        """Verify that stage artifact exists."""
        task_dir = self._get_task_dir(manifest)
        
        if not task_dir.exists():
            return False
        
        if stage == Stage.IMPL:
            required = ["impl.complete", "impl.summary.md", "impl.verification.md"]
            return all((task_dir / artifact).exists() for artifact in required)

        if stage == Stage.QA:
            report_exists = (task_dir / "qa.md").exists()
            verdict_exists = (task_dir / "qa.verified").exists() or (task_dir / "qa.review").exists()
            return report_exists and verdict_exists

        if stage == Stage.VALIDATE:
            return (task_dir / "validated").exists() or (task_dir / "validate.review").exists()

        artifact_map = {
            Stage.DESIGN: ["design.md"],
            Stage.SPEC_WRITE: ["spec.md"],
            Stage.SPEC_VERIFY: ["spec.verified", "spec.review"],
            Stage.PLAN_WRITE: ["plan.md"],
            Stage.PLAN_VERIFY: ["plan.verified", "plan.review"],
        }

        expected_list = artifact_map.get(stage, [])
        return any((task_dir / expected).exists() for expected in expected_list)
    
    def kill_agent(self, process: RunningAgent | subprocess.Popen[str]) -> None:
        """Kill agent process."""
        proc = process.process if isinstance(process, RunningAgent) else process
        try:
            os.kill(proc.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass  # Already terminated

        try:
            if proc.poll() is None:
                proc.wait(timeout=self.TERMINATE_TIMEOUT)
        except subprocess.TimeoutExpired:
            try:
                os.kill(proc.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            proc.wait()

        if isinstance(process, RunningAgent):
            process.pid_file.unlink(missing_ok=True)
