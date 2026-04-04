"""Supervisor for agent pipeline orchestration."""

from __future__ import annotations

import json
import os
import signal
import time
from collections.abc import Mapping
from datetime import datetime
from enum import Enum
from pathlib import Path

from agent_pipeline.state import StateManager, TaskManifest, Stage, TaskStatus, validate_safe_task_id
from agent_pipeline.events import EventLogger
from agent_pipeline.agents import AgentSpawner, AgentType, RunningAgent, SpawnResult
from agent_pipeline.worktrees import (
    ProvisioningOutcome,
    discover_repo_root,
    ensure_worktree_ready,
)


class StageLogStream(Enum):
    """Supported stage log streams."""

    STDOUT = "stdout"
    STDERR = "stderr"


class Supervisor:
    """Main supervision loop for agent pipeline."""
    
    def __init__(
        self,
        base_dir: Path,
        *,
        workspace_root: Path | None = None,
        stage_agents: Mapping[Stage | str, AgentType | str] | None = None,
    ):
        self.base_dir = Path(base_dir)
        self.workspace_root = Path(workspace_root).resolve() if workspace_root is not None else None
        self.state_manager = StateManager(base_dir)
        self.event_logger = EventLogger(base_dir / "events")
        self.agent_spawner = AgentSpawner(base_dir, workspace_root=self.workspace_root, stage_agents=stage_agents)
        
        # Track active tasks and their processes
        self.active_tasks: dict[str, TaskManifest] = {}
        self.active_agents: dict[str, RunningAgent] = {}
        
        # Control state
        self.paused_tasks: set[str] = set()
        self.steering_messages: dict[str, str] = {}
        self.abort_requested: set[str] = set()
    
    def check_queue(self) -> list[TaskManifest]:
        """Check for new tasks in queue."""
        return self.state_manager.list_queue()

    def _discover_repo_root(self) -> Path:
        """Resolve the repository root without assuming the state directory lives inside the repo."""
        if self.workspace_root is not None:
            if not self.workspace_root.exists():
                raise FileNotFoundError(
                    f"Configured workspace root does not exist: {self.workspace_root}"
                )
            if not self.workspace_root.is_dir():
                raise FileNotFoundError(
                    f"Configured workspace root is not a directory: {self.workspace_root}"
                )
            return self.workspace_root
        for candidate in (self.base_dir, Path.cwd(), Path(__file__).resolve()):
            try:
                return discover_repo_root(candidate)
            except FileNotFoundError:
                continue
        raise FileNotFoundError(f"Could not determine repository root from {self.base_dir}")

    def get_unmet_dependencies(self, manifest: TaskManifest) -> list[str]:
        """Return dependency ids that are not yet satisfied by a done/complete task."""
        unmet: list[str] = []
        for dependency in manifest.dependencies:
            try:
                state, dependency_manifest = self.state_manager.find_task(dependency)
            except FileNotFoundError:
                unmet.append(dependency)
                continue

            if state != "done" or dependency_manifest.status != TaskStatus.COMPLETE:
                unmet.append(dependency)

        return unmet
    
    def check_control(self) -> list[dict]:
        """Check for control commands."""
        commands = []
        control_dir = self.base_dir / "control"
        
        for ctrl_file in control_dir.glob("*.json"):
            try:
                data = json.loads(ctrl_file.read_text())
                commands.append(data)
                ctrl_file.unlink()
            except (json.JSONDecodeError, KeyError):
                ctrl_file.unlink()
        
        return commands
    
    def process_control_command(self, command: dict) -> None:
        """Process a control command."""
        cmd = command.get("command")
        task_id = command.get("task_id")
        
        if not cmd or not task_id:
            return
        
        if cmd == "pause":
            self.paused_tasks.add(task_id)
        elif cmd == "resume":
            self.paused_tasks.discard(task_id)
        elif cmd == "abort":
            self.abort_requested.add(task_id)
            self.kill_active_agent(task_id)
            self._abort_task(task_id)
        elif cmd == "steer":
            message = command.get("message", "")
            self.steering_messages[task_id] = message
            self._persist_steering_message(task_id, message)

    @staticmethod
    def _is_pid_running(pid: int) -> bool:
        """Check whether a process id is still alive."""
        try:
            os.kill(pid, 0)
        except OSError:
            return False
        return True

    def _terminate_pid(self, pid: int, timeout: float = 5.0) -> None:
        """Terminate a process id and escalate to SIGKILL if it lingers."""
        try:
            os.kill(pid, signal.SIGTERM)
        except (OSError, ProcessLookupError):
            return

        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if not self._is_pid_running(pid):
                return
            time.sleep(0.1)

        try:
            os.kill(pid, signal.SIGKILL)
        except (OSError, ProcessLookupError):
            return

    def _persist_steering_message(self, task_id: str, message: str) -> None:
        """Persist operator steering alongside the task bundle for future stages."""
        try:
            state, _ = self.state_manager.find_task(task_id)
        except FileNotFoundError:
            return

        if state not in {"queue", "in-progress"}:
            return

        task_dir = self.base_dir / state / task_id
        task_dir.mkdir(parents=True, exist_ok=True)
        guidance_file = task_dir / "steering.md"
        if guidance_file.exists():
            existing = guidance_file.read_text().rstrip()
            guidance_file.write_text(f"{existing}\n\n{message}\n")
        else:
            guidance_file.write_text(f"{message}\n")
    
    def kill_active_agent(self, task_id: str) -> None:
        """Kill active agent for task."""
        handle = self.active_agents.get(task_id)
        if handle is not None:
            self.agent_spawner.kill_agent(handle)
            return

        try:
            state, manifest = self.state_manager.find_task(task_id)
        except FileNotFoundError:
            return

        if state not in {"queue", "in-progress"}:
            return

        pid_file = self.agent_spawner._get_pid_file(manifest, manifest.current_stage)
        if not pid_file.exists():
            return

        try:
            pid = int(pid_file.read_text().strip())
            self._terminate_pid(pid)
        except ValueError:
            pass
        finally:
            pid_file.unlink(missing_ok=True)
    
    def start_task_stage(self, task_id: str) -> bool:
        """Start agent for task's current stage."""
        try:
            state, manifest = self.state_manager.find_task(task_id)
        except FileNotFoundError:
            return False

        if state != "in-progress":
            return False
        
        if task_id in self.paused_tasks:
            return False
        
        if task_id in self.abort_requested:
            self._abort_task(task_id)
            return False

        if task_id in self.active_agents:
            return False

        task_id_error = validate_safe_task_id(manifest.task_id)
        if task_id_error is not None:
            self._block_task(task_id, manifest, f"persisted task id '{manifest.task_id}' is invalid: {task_id_error}")
            return False

        if self.get_unmet_dependencies(manifest):
            return False
        
        stage = manifest.current_stage

        if self.agent_spawner.is_agent_running(manifest, stage):
            return False

        try:
            repo_root = self._discover_repo_root()
        except FileNotFoundError as exc:
            self._block_task(task_id, manifest, str(exc))
            return False

        try:
            provisioning = ensure_worktree_ready(repo_root, manifest)
        except FileNotFoundError as exc:
            self._block_task(task_id, manifest, str(exc))
            return False
        if provisioning.outcome == ProvisioningOutcome.BLOCK:
            self._block_task(task_id, manifest, provisioning.reason or "worktree provisioning blocked")
            return False

        if (
            manifest.worktree is None
            or manifest.worktree.path != str(provisioning.assignment.path.resolve())
            or manifest.worktree.branch != provisioning.assignment.branch
        ):
            manifest.set_worktree(
                path=provisioning.assignment.path,
                branch=provisioning.assignment.branch,
            )
            self.state_manager.save_task(manifest)
        
        agent_type = self.agent_spawner.get_agent_type(stage)
        self.event_logger.log_agent_spawned(task_id, agent_type.value, stage.value)

        try:
            handle = self.agent_spawner.launch(
                manifest,
                stage,
                execution_cwd=provisioning.assignment.path,
            )
        except Exception as exc:
            self._handle_stage_failure(
                task_id,
                manifest,
                SpawnResult(success=False, exit_code=-1, error=str(exc)),
            )
            return False

        self.active_tasks[task_id] = manifest
        self.active_agents[task_id] = handle
        
        return True

    def check_active_tasks(self) -> None:
        """Poll active agents and process any completed work."""
        for task_id, handle in list(self.active_agents.items()):
            result = self.agent_spawner.poll(handle)
            if result is None:
                continue

            self.active_agents.pop(task_id, None)
            manifest = self.active_tasks.pop(task_id, None)
            if manifest is None:
                try:
                    state, manifest = self.state_manager.find_task(task_id)
                except FileNotFoundError:
                    continue
                if state != "in-progress":
                    continue

            review_block_reason = self._review_block_reason(manifest)
            if review_block_reason is not None:
                self._block_task(task_id, manifest, review_block_reason)
                continue

            if result.success:
                self._handle_stage_success(task_id, manifest)
            else:
                self._handle_stage_failure(task_id, manifest, result)
    
    def _handle_stage_success(self, task_id: str, manifest: TaskManifest) -> None:
        """Handle successful stage completion."""
        stage = manifest.current_stage
        
        if not self.agent_spawner.verify_artifact(manifest, stage):
            result = SpawnResult(
                success=False,
                exit_code=-1,
                error=f"Agent exited successfully but {stage.value} artifact is missing"
            )
            self._handle_stage_failure(task_id, manifest, result)
            return
        
        review_block_reason = self._review_block_reason(manifest)
        if review_block_reason is not None:
            self._block_task(task_id, manifest, review_block_reason)
            return
        
        self.event_logger.log_stage_complete(task_id, stage.value)
        manifest.advance_stage()
        
        if manifest.status == TaskStatus.COMPLETE:
            self._complete_task(task_id, manifest)
        else:
            self.state_manager.save_task(manifest)
    
    def _review_block_reason(self, manifest: TaskManifest) -> str | None:
        """Return a blocking reason when a complete review artifact set indicates contract failure."""
        task_dir = self.agent_spawner._get_task_dir(manifest)
        stage = manifest.current_stage

        if stage == Stage.QA:
            if (task_dir / "qa.md").exists() and (task_dir / "qa.review").exists():
                return "qa found issues - see qa.review for details"
            return None

        review_artifacts = {
            Stage.SPEC_VERIFY: "spec.review",
            Stage.PLAN_VERIFY: "plan.review",
            Stage.VALIDATE: "validate.review",
        }
        review_file = review_artifacts.get(stage)
        if review_file and (task_dir / review_file).exists():
            return f"{stage.value} found issues - see {review_file} for details"
        return None

    def _handle_stage_failure(
        self,
        task_id: str,
        manifest: TaskManifest,
        result: SpawnResult
    ) -> None:
        """Handle stage failure with retry logic."""
        stage = manifest.current_stage
        
        self.event_logger.log_stage_failed(
            task_id,
            stage.value,
            result.error or "Unknown error",
            result.stderr
        )
        
        manifest.increment_attempt()
        
        if manifest.should_escalate():
            self._block_task(task_id, manifest, f"max retries exceeded for {stage.value}")
        else:
            self.event_logger.log_retry_scheduled(
                task_id,
                stage.value,
                manifest.attempts[stage]
            )
            self.state_manager.save_task(manifest)
    
    def _block_task(self, task_id: str, manifest: TaskManifest, reason: str) -> None:
        """Block task and notify."""
        manifest.mark_blocked(reason)
        self.state_manager.move_to_blocked(task_id, reason)
        self.event_logger.log_blocked(task_id, reason)
        
        if task_id in self.active_tasks:
            del self.active_tasks[task_id]
        if task_id in self.active_agents:
            del self.active_agents[task_id]
    
    def _complete_task(self, task_id: str, manifest: TaskManifest) -> None:
        """Mark task as complete."""
        manifest.mark_complete()
        self.state_manager.move_to_done(task_id)
        self.event_logger.log_complete(task_id)
        
        if task_id in self.active_tasks:
            del self.active_tasks[task_id]
        if task_id in self.active_agents:
            del self.active_agents[task_id]
    
    def _abort_task(self, task_id: str) -> None:
        """Abort task due to user request."""
        reason = "aborted by user"

        try:
            state, manifest = self.state_manager.find_task(task_id)
        except FileNotFoundError:
            state = None
            manifest = None

        if state == "in-progress":
            self.state_manager.move_to_blocked(task_id, reason)
            self.event_logger.log_blocked(task_id, reason)
        elif state == "queue" and manifest is not None:
            manifest.mark_blocked(reason)
            queue_dir = self.base_dir / "queue" / task_id
            blocked_dir = self.base_dir / "blocked" / task_id
            self.state_manager.save_task(manifest, subdir="queue")
            blocked_dir.parent.mkdir(parents=True, exist_ok=True)
            queue_dir.replace(blocked_dir)
            self.event_logger.log_blocked(task_id, reason)

        self.active_tasks.pop(task_id, None)
        self.active_agents.pop(task_id, None)
        self.paused_tasks.discard(task_id)
        self.steering_messages.pop(task_id, None)
        self.abort_requested.discard(task_id)

    @staticmethod
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
    
    
    def _resolve_stage_log_path(
        self,
        task_id: str,
        stage: Stage | None = None,
        stream: StageLogStream = StageLogStream.STDOUT,
    ) -> Path:
        """Resolve the persisted log path for a task stage stream."""
        state, manifest = self.state_manager.find_task(task_id)
        resolved_stage = manifest.current_stage if stage is None else stage
        task_dir = self.base_dir / state / task_id
        return task_dir / f"{resolved_stage.value}.{stream.value}.log"

    def read_stage_log(
        self,
        task_id: str,
        stage: Stage | None = None,
        stream: StageLogStream = StageLogStream.STDOUT,
        tail: int | None = None,
    ) -> str:
        """Read a persisted stage log for operator inspection."""
        _, manifest = self.state_manager.find_task(task_id)
        resolved_stage = manifest.current_stage if stage is None else stage
        log_path = self._resolve_stage_log_path(task_id, resolved_stage, stream)
        if not log_path.exists():
            raise FileNotFoundError(
                f"No {stream.value} log for {task_id} stage {resolved_stage.value} yet"
            )

        contents = log_path.read_text(encoding="utf-8")
        if tail is None or tail <= 0:
            return contents

        lines = contents.splitlines(keepends=True)
        return "".join(lines[-tail:])

    def follow_stage_log(
        self,
        task_id: str,
        stage: Stage | None = None,
        stream: StageLogStream = StageLogStream.STDOUT,
        *,
        tail: int | None = None,
        poll_interval: float = 0.1,
        idle_timeout: float = 1.0,
    ):
        """Yield newly available log chunks until the file goes idle without dropping bytes after the initial snapshot."""
        _, manifest = self.state_manager.find_task(task_id)
        resolved_stage = manifest.current_stage if stage is None else stage
        log_path = self._resolve_stage_log_path(task_id, resolved_stage, stream)
        if not log_path.exists():
            raise FileNotFoundError(
                f"No {stream.value} log for {task_id} stage {resolved_stage.value} yet"
            )

        with log_path.open("r", encoding="utf-8") as handle:
            if tail is not None and tail > 0:
                full_contents = handle.read()
                lines = full_contents.splitlines(keepends=True)
                initial = "".join(lines[-tail:])
                if initial:
                    yield initial
            else:
                initial = handle.read()
                if initial:
                    yield initial

            idle_started = time.monotonic()
            while True:
                chunk = handle.read()
                if chunk:
                    idle_started = time.monotonic()
                    yield chunk
                    continue

                if time.monotonic() - idle_started >= idle_timeout:
                    break
                time.sleep(poll_interval)

    def emit_status(self) -> None:
        """Write status dashboard."""
        status = {
            "timestamp": datetime.now().isoformat(),
            "active_sessions": len(self.active_tasks),
            "sessions": {
                task_id: {
                    "stage": manifest.current_stage.value,
                    "attempt": manifest.attempts[manifest.current_stage],
                    "started_by": manifest.started_by,
                    "worktree_path": manifest.worktree.path if manifest.worktree is not None else None,
                    "worktree_branch": manifest.worktree.branch if manifest.worktree is not None else None,
                }
                for task_id, manifest in self.active_tasks.items()
            },
            "paused": list(self.paused_tasks),
            "stage_agents": {
                stage.value: agent.value
                for stage, agent in self.agent_spawner.stage_agents.items()
            },
        }

        status_file = self.base_dir / "status" / "dashboard.json"
        status_file.write_text(json.dumps(status, indent=2))
    
    def get_task_status(self, task_id: str) -> dict:
        """Get status for specific task."""
        try:
            state, manifest = self.state_manager.find_task(task_id)
            unmet_dependencies = self.get_unmet_dependencies(manifest)
            task_dir = self.base_dir / state / task_id
            has_feedback_resolution, feedback_review_artifact = self._feedback_resolution_metadata(task_dir)
            return {
                "task_id": manifest.task_id,
                "current_stage": manifest.current_stage.value,
                "status": manifest.status.value,
                "attempts": {k.value: v for k, v in manifest.attempts.items()},
                "blockers": manifest.blockers,
                "dependencies": manifest.dependencies,
                "unmet_dependencies": unmet_dependencies,
                "waiting_on_dependencies": bool(unmet_dependencies),
                "has_feedback_resolution": has_feedback_resolution,
                "feedback_review_artifact": feedback_review_artifact,
                "started_by": manifest.started_by,
                "worktree_path": manifest.worktree.path if manifest.worktree is not None else None,
                "worktree_branch": manifest.worktree.branch if manifest.worktree is not None else None,
                "worktree_metadata_state": manifest.worktree_metadata_state().value,
                "worktree_error": manifest.worktree_block_reason(),
            }
        except FileNotFoundError:
            return {"error": f"Task {task_id} not found"}
    
    def _block_task_bundle(self, bundle_id: str, state: str, manifest: TaskManifest, reason: str) -> None:
        """Block a task bundle using its on-disk directory name when persisted task metadata is unsafe."""
        manifest.mark_blocked(reason)
        self.state_manager.save_task_bundle(bundle_id, manifest, subdir=state)
        self.state_manager.move_task(bundle_id, state, "blocked")
        self.event_logger.log_blocked(bundle_id, reason)

    def run_once(self) -> None:
        """Run one iteration of supervision loop."""
        commands = self.check_control()
        for cmd in commands:
            self.process_control_command(cmd)

        self.check_active_tasks()

        queue_entries = self.state_manager.list_subdir_entries("queue")
        for bundle_id, task in queue_entries:
            task_id_error = validate_safe_task_id(task.task_id)
            if task_id_error is not None:
                self._block_task_bundle(
                    bundle_id,
                    "queue",
                    task,
                    f"persisted task id '{task.task_id}' is invalid: {task_id_error}",
                )
                continue
            if task.task_id in self.abort_requested:
                self._abort_task(task.task_id)
                continue
            if self.get_unmet_dependencies(task):
                continue
            self.state_manager.move_task(bundle_id, "queue", "in-progress")
            self.event_logger.log_task_created(task.task_id, task.started_by)
            self.start_task_stage(task.task_id)
        
        in_progress_entries = self.state_manager.list_subdir_entries("in-progress")
        for bundle_id, manifest in in_progress_entries:
            task_id = manifest.task_id
            if manifest.status.value == "blocked":
                continue
            if manifest.status.value == "complete":
                continue
            task_id_error = validate_safe_task_id(task_id)
            if task_id_error is not None:
                self._block_task_bundle(
                    bundle_id,
                    "in-progress",
                    manifest,
                    f"persisted task id '{task_id}' is invalid: {task_id_error}",
                )
                continue
            if task_id in self.active_agents:
                continue
            if self.agent_spawner.is_agent_running(manifest, manifest.current_stage):
                continue
            self.start_task_stage(task_id)

        self.emit_status()
    
    def run(self, interval: float = 1.0) -> None:
        """Run supervision loop continuously."""
        while True:
            self.run_once()
            time.sleep(interval)
