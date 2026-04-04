"""State management for agent pipeline tasks."""

from __future__ import annotations

import json
from datetime import datetime
from enum import Enum
from pathlib import Path


MAX_ATTEMPTS = 5
TASK_STATE_DIRS = ("queue", "in-progress", "done", "blocked")


class Stage(Enum):
    """Pipeline stages in order."""
    
    DESIGN = "design"
    SPEC_WRITE = "spec_write"
    SPEC_VERIFY = "spec_verify"
    PLAN_WRITE = "plan_write"
    PLAN_VERIFY = "plan_verify"
    IMPL = "impl"
    QA = "qa"
    VALIDATE = "validate"
    
    def next_stage(self) -> Stage | None:
        """Get next stage in pipeline."""
        stages = list(Stage)
        idx = stages.index(self)
        if idx + 1 < len(stages):
            return stages[idx + 1]
        return None


class TaskStatus(Enum):
    """Task lifecycle status."""
    
    IN_PROGRESS = "in_progress"
    BLOCKED = "blocked"
    COMPLETE = "complete"


class TaskManifest:
    """Task manifest tracking state and progress."""
    
    def __init__(
        self,
        task_id: str,
        current_stage: Stage,
        status: TaskStatus,
        attempts: dict[Stage, int],
        max_attempts: int,
        artifacts: dict[Stage, str],
        blockers: list[str],
        started_by: str,
        created_at: str,
        updated_at: str,
    ):
        self.task_id = task_id
        self.current_stage = current_stage
        self.status = status
        self.attempts = attempts
        self.max_attempts = max_attempts
        self.artifacts = artifacts
        self.blockers = blockers
        self.started_by = started_by
        self.created_at = created_at
        self.updated_at = updated_at
    
    @classmethod
    def create(cls, task_id: str, started_by: str) -> TaskManifest:
        """Create new task manifest."""
        now = datetime.now().isoformat()
        attempts = {stage: 0 for stage in Stage}
        
        return cls(
            task_id=task_id,
            current_stage=Stage.DESIGN,
            status=TaskStatus.IN_PROGRESS,
            attempts=attempts,
            max_attempts=MAX_ATTEMPTS,
            artifacts={},
            blockers=[],
            started_by=started_by,
            created_at=now,
            updated_at=now,
        )
    
    def to_json(self) -> str:
        """Serialize to JSON."""
        data = {
            "task_id": self.task_id,
            "current_stage": self.current_stage.value,
            "status": self.status.value,
            "attempts": {k.value: v for k, v in self.attempts.items()},
            "max_attempts": self.max_attempts,
            "artifacts": {k.value: v for k, v in self.artifacts.items()},
            "blockers": self.blockers,
            "started_by": self.started_by,
            "created_at": self.created_at,
            "updated_at": self.updated_at,
        }
        return json.dumps(data, indent=2)
    
    @classmethod
    def from_json(cls, json_str: str) -> TaskManifest:
        """Deserialize from JSON."""
        data = json.loads(json_str)
        
        return cls(
            task_id=data["task_id"],
            current_stage=Stage(data["current_stage"]),
            status=TaskStatus(data["status"]),
            attempts={Stage(k): v for k, v in data["attempts"].items()},
            max_attempts=data["max_attempts"],
            artifacts={Stage(k): v for k, v in data.get("artifacts", {}).items()},
            blockers=data.get("blockers", []),
            started_by=data["started_by"],
            created_at=data["created_at"],
            updated_at=data["updated_at"],
        )
    
    def increment_attempt(self) -> None:
        """Increment attempt counter for current stage."""
        self.attempts[self.current_stage] += 1
        self.updated_at = datetime.now().isoformat()
    
    def advance_stage(self, artifact_path: str | None = None) -> None:
        """Advance to next stage."""
        if artifact_path:
            self.artifacts[self.current_stage] = artifact_path
        
        next_stage = self.current_stage.next_stage()
        if next_stage:
            self.current_stage = next_stage
        else:
            self.status = TaskStatus.COMPLETE
        
        self.updated_at = datetime.now().isoformat()
    
    def is_blocked(self) -> bool:
        """Check if task is blocked due to max attempts."""
        return self.attempts[self.current_stage] >= self.max_attempts
    
    def should_escalate(self) -> bool:
        """Check if task should escalate to human."""
        return self.is_blocked()
    
    def mark_blocked(self, reason: str) -> None:
        """Mark task as blocked with reason."""
        self.status = TaskStatus.BLOCKED
        self.blockers.append(reason)
        self.updated_at = datetime.now().isoformat()
    
    def mark_complete(self) -> None:
        """Mark task as complete."""
        self.status = TaskStatus.COMPLETE
        self.updated_at = datetime.now().isoformat()


class StateManager:
    """Manages task state files."""
    
    def __init__(self, base_dir: Path):
        self.base_dir = Path(base_dir)
        self._ensure_directories()
    
    def _ensure_directories(self) -> None:
        """Create required directory structure."""
        for subdir in ["queue", "in-progress", "done", "blocked", "control", "events", "status"]:
            (self.base_dir / subdir).mkdir(parents=True, exist_ok=True)

    def _task_dir(self, task_id: str, subdir: str) -> Path:
        """Get directory for a task in a state subdirectory."""
        return self.base_dir / subdir / task_id

    def _manifest_path(self, task_id: str, subdir: str) -> Path:
        """Get manifest path for a task in a state subdirectory."""
        return self._task_dir(task_id, subdir) / "manifest.json"
    
    def create_task(self, task_id: str, started_by: str) -> TaskManifest:
        """Create new task in queue."""
        manifest = TaskManifest.create(task_id, started_by)
        self.save_task(manifest, subdir="queue")
        return manifest
    
    def save_task(self, manifest: TaskManifest, subdir: str = "in-progress") -> None:
        """Save task manifest atomically."""
        task_path = self._manifest_path(manifest.task_id, subdir)
        task_path.parent.mkdir(parents=True, exist_ok=True)
        self._atomic_write(task_path, manifest.to_json())
    
    def _atomic_write(self, path: Path, content: str) -> None:
        """Write file atomically using temp file + rename."""
        # Use temp file in same directory to avoid cross-device rename issues
        temp_path = path.parent / f".{path.name}.tmp"
        temp_path.write_text(content)
        temp_path.replace(path)
    
    def load_task(self, task_id: str, subdir: str = "in-progress") -> TaskManifest:
        """Load task from file."""
        task_path = self._manifest_path(task_id, subdir)
        if not task_path.exists():
            raise FileNotFoundError(f"Task {task_id} not found in {subdir}")
        return self.load_task_from_path(task_path)

    def find_task_path(self, task_id: str) -> tuple[str, Path]:
        """Find manifest path for a task across state directories."""
        for subdir in TASK_STATE_DIRS:
            task_path = self._manifest_path(task_id, subdir)
            if task_path.exists():
                return subdir, task_path
        raise FileNotFoundError(f"Task {task_id} not found")

    def find_task(self, task_id: str) -> tuple[str, TaskManifest]:
        """Find and load a task across state directories."""
        subdir, task_path = self.find_task_path(task_id)
        return subdir, self.load_task_from_path(task_path)
    
    def load_task_from_path(self, path: Path) -> TaskManifest:
        """Load task from specific path."""
        return TaskManifest.from_json(path.read_text())

    def _move_task_directory(self, task_id: str, src_subdir: str, dst_subdir: str) -> Path:
        """Move a whole task directory between lifecycle directories."""
        src = self._task_dir(task_id, src_subdir)
        if not src.exists():
            raise FileNotFoundError(f"Task {task_id} not found in {src_subdir}")

        dst = self._task_dir(task_id, dst_subdir)
        dst.parent.mkdir(parents=True, exist_ok=True)
        src.replace(dst)
        return dst
    
    def move_to_in_progress(self, task_id: str) -> TaskManifest:
        """Move task from queue to in-progress."""
        manifest = self.load_task(task_id, subdir="queue")
        self._move_task_directory(task_id, "queue", "in-progress")
        return manifest
    
    def move_to_done(self, task_id: str) -> None:
        """Move task to done directory."""
        manifest = self.load_task(task_id, subdir="in-progress")
        manifest.mark_complete()
        self.save_task(manifest, subdir="in-progress")
        self._move_task_directory(task_id, "in-progress", "done")
    
    def move_to_blocked(self, task_id: str, reason: str) -> None:
        """Move task to blocked directory."""
        manifest = self.load_task(task_id, subdir="in-progress")
        manifest.mark_blocked(reason)
        self.save_task(manifest, subdir="in-progress")
        self._move_task_directory(task_id, "in-progress", "blocked")
    
    def list_queue(self) -> list[TaskManifest]:
        """List all tasks in queue."""
        return self._list_subdir("queue")
    
    def list_in_progress(self) -> list[TaskManifest]:
        """List all in-progress tasks."""
        return self._list_subdir("in-progress")
    
    def _list_subdir(self, subdir: str) -> list[TaskManifest]:
        """List all tasks in subdirectory."""
        tasks = []
        tasks_dir = self.base_dir / subdir
        for task_dir in sorted(tasks_dir.iterdir()):
            if not task_dir.is_dir():
                continue

            manifest_path = task_dir / "manifest.json"
            if manifest_path.exists():
                tasks.append(self.load_task_from_path(manifest_path))
        return tasks
