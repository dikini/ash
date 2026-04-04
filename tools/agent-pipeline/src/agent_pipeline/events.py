"""Event logging for agent pipeline."""

from __future__ import annotations

import json
from dataclasses import dataclass
from datetime import datetime
from enum import Enum
from pathlib import Path
from typing import Any


class EventType(Enum):
    """Types of pipeline events."""
    
    TASK_CREATED = "task_created"
    AGENT_SPAWNED = "agent_spawned"
    STAGE_COMPLETE = "stage_complete"
    STAGE_FAILED = "stage_failed"
    RETRY_SCHEDULED = "retry_scheduled"
    BLOCKED = "blocked"
    COMPLETE = "complete"


@dataclass
class Event:
    """Single event record."""
    
    timestamp: str
    type: EventType
    task_id: str
    data: dict[str, Any]
    
    def to_json(self) -> str:
        """Serialize to JSON line."""
        data = {
            "timestamp": self.timestamp,
            "type": self.type.value,
            "task_id": self.task_id,
            **self.data
        }
        return json.dumps(data)
    
    @classmethod
    def from_json(cls, json_str: str) -> Event:
        """Deserialize from JSON."""
        data = json.loads(json_str)
        event_type = EventType(data.pop("type"))
        task_id = data.pop("task_id")
        timestamp = data.pop("timestamp")
        return cls(
            timestamp=timestamp,
            type=event_type,
            task_id=task_id,
            data=data
        )


class EventLogger:
    """Append-only event logger."""
    
    def __init__(self, base_dir: Path):
        self.base_dir = Path(base_dir)
        self._ensure_directory()
    
    def _ensure_directory(self) -> None:
        """Create events directory if needed."""
        self.base_dir.mkdir(parents=True, exist_ok=True)
    
    def _now(self) -> str:
        """Get current ISO8601 timestamp."""
        return datetime.now().isoformat()
    
    def _log(self, task_id: str, event_type: EventType, data: dict[str, Any]) -> None:
        """Append event to task's log file."""
        event = Event(
            timestamp=self._now(),
            type=event_type,
            task_id=task_id,
            data=data
        )
        
        event_file = self.base_dir / f"{task_id}.jsonl"
        with open(event_file, "a") as f:
            f.write(event.to_json() + "\n")
    
    def log_task_created(self, task_id: str, started_by: str) -> None:
        """Log task creation."""
        self._log(task_id, EventType.TASK_CREATED, {"started_by": started_by})
    
    def log_agent_spawned(self, task_id: str, agent: str, stage: str) -> None:
        """Log agent spawn."""
        self._log(task_id, EventType.AGENT_SPAWNED, {
            "agent": agent,
            "stage": stage
        })
    
    def log_stage_complete(self, task_id: str, stage: str, artifact: str | None = None) -> None:
        """Log stage completion."""
        data = {"stage": stage}
        if artifact:
            data["artifact"] = artifact
        self._log(task_id, EventType.STAGE_COMPLETE, data)
    
    def log_stage_failed(self, task_id: str, stage: str, error: str, details: str = "") -> None:
        """Log stage failure."""
        self._log(task_id, EventType.STAGE_FAILED, {
            "stage": stage,
            "error": error,
            "details": details
        })
    
    def log_retry_scheduled(self, task_id: str, stage: str, attempt: int) -> None:
        """Log retry scheduling."""
        self._log(task_id, EventType.RETRY_SCHEDULED, {
            "stage": stage,
            "attempt": attempt
        })
    
    def log_blocked(self, task_id: str, reason: str) -> None:
        """Log blocked status."""
        self._log(task_id, EventType.BLOCKED, {"reason": reason})
    
    def log_complete(self, task_id: str) -> None:
        """Log task completion."""
        self._log(task_id, EventType.COMPLETE, {})
    
    def get_event_history(self, task_id: str) -> list[Event]:
        """Get all events for a task."""
        event_file = self.base_dir / f"{task_id}.jsonl"
        
        if not event_file.exists():
            return []
        
        events = []
        with open(event_file) as f:
            for line in f:
                line = line.strip()
                if line:
                    events.append(Event.from_json(line))
        
        return events
