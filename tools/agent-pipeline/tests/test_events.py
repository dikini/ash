"""Tests for event logging module."""

import json
import tempfile
from pathlib import Path
from datetime import datetime

import pytest

from agent_pipeline.events import EventLogger, EventType, Event


class TestEventLogger:
    """Test event logging functionality."""

    @pytest.fixture
    def temp_dir(self):
        """Provide temporary directory for event files."""
        with tempfile.TemporaryDirectory() as tmp:
            yield Path(tmp)

    @pytest.fixture
    def logger(self, temp_dir):
        """Provide EventLogger with temp directory."""
        return EventLogger(temp_dir)

    def test_log_task_created(self, logger, temp_dir):
        """RED: Should log task creation event."""
        logger.log_task_created("TASK-001", started_by="cli")
        
        event_file = temp_dir / "TASK-001.jsonl"
        assert event_file.exists()
        
        lines = event_file.read_text().strip().split("\n")
        assert len(lines) == 1
        
        event = json.loads(lines[0])
        assert event["type"] == "task_created"
        assert event["task_id"] == "TASK-001"
        assert event["started_by"] == "cli"
        assert "timestamp" in event

    def test_log_agent_spawned(self, logger, temp_dir):
        """RED: Should log agent spawn event."""
        logger.log_agent_spawned("TASK-001", agent="codex", stage="design")
        
        event_file = temp_dir / "TASK-001.jsonl"
        lines = event_file.read_text().strip().split("\n")
        
        event = json.loads(lines[0])
        assert event["type"] == "agent_spawned"
        assert event["agent"] == "codex"
        assert event["stage"] == "design"

    def test_log_stage_complete(self, logger, temp_dir):
        """RED: Should log stage completion."""
        logger.log_stage_complete("TASK-001", stage="design", artifact="design.md")
        
        event_file = temp_dir / "TASK-001.jsonl"
        lines = event_file.read_text().strip().split("\n")
        
        event = json.loads(lines[0])
        assert event["type"] == "stage_complete"
        assert event["stage"] == "design"
        assert event["artifact"] == "design.md"

    def test_log_stage_failed(self, logger, temp_dir):
        """RED: Should log stage failure."""
        logger.log_stage_failed("TASK-001", stage="impl", error="test_failure", details="...")
        
        event_file = temp_dir / "TASK-001.jsonl"
        lines = event_file.read_text().strip().split("\n")
        
        event = json.loads(lines[0])
        assert event["type"] == "stage_failed"
        assert event["error"] == "test_failure"
        assert event["details"] == "..."

    def test_log_retry_scheduled(self, logger, temp_dir):
        """RED: Should log retry scheduling."""
        logger.log_retry_scheduled("TASK-001", stage="impl", attempt=2)
        
        event_file = temp_dir / "TASK-001.jsonl"
        lines = event_file.read_text().strip().split("\n")
        
        event = json.loads(lines[0])
        assert event["type"] == "retry_scheduled"
        assert event["attempt"] == 2

    def test_log_blocked(self, logger, temp_dir):
        """RED: Should log blocked status."""
        logger.log_blocked("TASK-001", reason="max retries")
        
        event_file = temp_dir / "TASK-001.jsonl"
        lines = event_file.read_text().strip().split("\n")
        
        event = json.loads(lines[0])
        assert event["type"] == "blocked"
        assert event["reason"] == "max retries"

    def test_log_complete(self, logger, temp_dir):
        """RED: Should log completion."""
        logger.log_complete("TASK-001")
        
        event_file = temp_dir / "TASK-001.jsonl"
        lines = event_file.read_text().strip().split("\n")
        
        event = json.loads(lines[0])
        assert event["type"] == "complete"

    def test_multiple_events_append(self, logger, temp_dir):
        """RED: Should append multiple events to same file."""
        logger.log_task_created("TASK-001", started_by="cli")
        logger.log_agent_spawned("TASK-001", agent="codex", stage="design")
        logger.log_stage_complete("TASK-001", stage="design", artifact="design.md")
        
        event_file = temp_dir / "TASK-001.jsonl"
        lines = event_file.read_text().strip().split("\n")
        
        assert len(lines) == 3
        
        types = [json.loads(line)["type"] for line in lines]
        assert types == ["task_created", "agent_spawned", "stage_complete"]

    def test_get_event_history(self, logger):
        """RED: Should retrieve all events for task."""
        logger.log_task_created("TASK-001", started_by="cli")
        logger.log_agent_spawned("TASK-001", agent="codex", stage="design")
        
        events = logger.get_event_history("TASK-001")
        
        assert len(events) == 2
        assert events[0].type == EventType.TASK_CREATED
        assert events[1].type == EventType.AGENT_SPAWNED

    def test_get_event_history_not_found(self, logger):
        """RED: Should return empty list for non-existent task."""
        events = logger.get_event_history("NONEXISTENT")
        
        assert events == []

    def test_event_timestamp(self, logger, temp_dir):
        """RED: Should include ISO8601 timestamp."""
        before = datetime.now().isoformat()
        logger.log_task_created("TASK-001", started_by="cli")
        after = datetime.now().isoformat()
        
        event_file = temp_dir / "TASK-001.jsonl"
        event = json.loads(event_file.read_text().strip())
        
        assert before <= event["timestamp"] <= after

    def test_event_data_class(self):
        """RED: Event dataclass should store event data."""
        event = Event(
            timestamp="2024-01-01T00:00:00Z",
            type=EventType.TASK_CREATED,
            task_id="TASK-001",
            data={"started_by": "cli"}
        )
        
        assert event.type == EventType.TASK_CREATED
        assert event.task_id == "TASK-001"
        assert event.data["started_by"] == "cli"
