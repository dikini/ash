"""Tests for state management module."""

import json
import tempfile
from pathlib import Path

import pytest

from agent_pipeline.state import (
    TaskManifest,
    Stage,
    TaskStatus,
    StateManager,
    MAX_ATTEMPTS,
)


class TestTaskManifest:
    """Test TaskManifest data class."""

    def test_create_minimal_manifest(self):
        """RED: Should create minimal valid manifest."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        
        assert manifest.task_id == "TASK-001"
        assert manifest.current_stage == Stage.DESIGN
        assert manifest.status == TaskStatus.IN_PROGRESS
        assert manifest.started_by == "cli"
        assert manifest.attempts[Stage.DESIGN] == 0
        assert manifest.max_attempts == MAX_ATTEMPTS

    def test_manifest_serialization(self):
        """RED: Should serialize to and from JSON."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        
        json_str = manifest.to_json()
        restored = TaskManifest.from_json(json_str)
        
        assert restored.task_id == manifest.task_id
        assert restored.current_stage == manifest.current_stage
        assert restored.status == manifest.status

    def test_increment_attempt(self):
        """RED: Should increment attempt counter for current stage."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        
        manifest.increment_attempt()
        assert manifest.attempts[Stage.DESIGN] == 1
        
        manifest.increment_attempt()
        assert manifest.attempts[Stage.DESIGN] == 2

    def test_advance_stage(self):
        """RED: Should advance to next stage and reset attempt counter."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        
        manifest.advance_stage()
        assert manifest.current_stage == Stage.SPEC_WRITE
        
        manifest.advance_stage()
        assert manifest.current_stage == Stage.SPEC_VERIFY

    def test_advance_stage_records_artifact(self):
        """RED: Should record artifact when advancing stage."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        
        manifest.advance_stage(artifact_path="design.md")
        assert manifest.artifacts[Stage.DESIGN] == "design.md"

    def test_is_blocked_max_attempts(self):
        """RED: Should report blocked when max attempts reached."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        
        for _ in range(MAX_ATTEMPTS):
            manifest.increment_attempt()
        
        assert manifest.is_blocked()
        assert manifest.should_escalate()

    def test_is_blocked_not_reached(self):
        """RED: Should not report blocked before max attempts."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        
        for _ in range(MAX_ATTEMPTS - 1):
            manifest.increment_attempt()
        
        assert not manifest.is_blocked()
        assert not manifest.should_escalate()

    def test_mark_blocked(self):
        """RED: Should mark task as blocked with reason."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        
        manifest.mark_blocked("test failure")
        
        assert manifest.status == TaskStatus.BLOCKED
        assert manifest.blockers == ["test failure"]

    def test_mark_complete(self):
        """RED: Should mark task as complete."""
        manifest = TaskManifest.create("TASK-001", started_by="cli")
        
        manifest.mark_complete()
        
        assert manifest.status == TaskStatus.COMPLETE


class TestStateManager:
    """Test StateManager file operations."""

    @pytest.fixture
    def temp_dir(self):
        """Provide temporary directory for state files."""
        with tempfile.TemporaryDirectory() as tmp:
            yield Path(tmp)

    @pytest.fixture
    def manager(self, temp_dir):
        """Provide StateManager with temp directory."""
        return StateManager(temp_dir)

    def test_create_task(self, manager, temp_dir):
        """RED: Should create task file in queue directory."""
        manager.create_task("TASK-001", started_by="cli")
        
        queue_file = temp_dir / "queue" / "TASK-001" / "manifest.json"
        assert queue_file.exists()
        
        data = json.loads(queue_file.read_text())
        assert data["task_id"] == "TASK-001"
        assert data["started_by"] == "cli"

    def test_load_task(self, manager, temp_dir):
        """RED: Should load task from file."""
        manager.create_task("TASK-001", started_by="cli")
        
        loaded = manager.load_task("TASK-001", subdir="queue")
        
        assert loaded.task_id == "TASK-001"
        assert loaded.started_by == "cli"

    def test_load_task_not_found(self, manager):
        """RED: Should raise error for non-existent task."""
        with pytest.raises(FileNotFoundError):
            manager.load_task("NONEXISTENT")

    def test_save_task(self, manager, temp_dir):
        """RED: Should save task updates atomically."""
        manifest = manager.create_task("TASK-001", started_by="cli")
        
        manifest.increment_attempt()
        manager.save_task(manifest)
        
        loaded = manager.load_task("TASK-001")
        assert loaded.attempts[Stage.DESIGN] == 1

    def test_move_to_in_progress(self, manager, temp_dir):
        """RED: Should move task from queue to in-progress."""
        manager.create_task("TASK-001", started_by="cli")
        
        manager.move_to_in_progress("TASK-001")
        
        assert not (temp_dir / "queue" / "TASK-001").exists()
        assert (temp_dir / "in-progress" / "TASK-001" / "manifest.json").exists()

    def test_move_to_done(self, manager, temp_dir):
        """RED: Should move task to done directory."""
        manager.create_task("TASK-001", started_by="cli")
        manager.move_to_in_progress("TASK-001")
        
        manager.move_to_done("TASK-001")
        
        assert not (temp_dir / "in-progress" / "TASK-001").exists()
        assert (temp_dir / "done" / "TASK-001" / "manifest.json").exists()

    def test_move_to_blocked(self, manager, temp_dir):
        """RED: Should move task to blocked directory."""
        manager.create_task("TASK-001", started_by="cli")
        manager.move_to_in_progress("TASK-001")
        
        manager.move_to_blocked("TASK-001", "max retries exceeded")
        
        assert not (temp_dir / "in-progress" / "TASK-001").exists()
        assert (temp_dir / "blocked" / "TASK-001" / "manifest.json").exists()
        
        loaded = manager.load_task_from_path(temp_dir / "blocked" / "TASK-001" / "manifest.json")
        assert loaded.status == TaskStatus.BLOCKED
        assert loaded.blockers == ["max retries exceeded"]

    def test_find_task_across_subdirectories(self, manager):
        """RED: Should find task regardless of lifecycle directory."""
        manager.create_task("TASK-001", started_by="cli")

        location, manifest = manager.find_task("TASK-001")
        assert location == "queue"
        assert manifest.task_id == "TASK-001"

        manager.move_to_in_progress("TASK-001")
        location, manifest = manager.find_task("TASK-001")
        assert location == "in-progress"
        assert manifest.task_id == "TASK-001"

        manager.move_to_blocked("TASK-001", "waiting on review")
        location, manifest = manager.find_task("TASK-001")
        assert location == "blocked"
        assert manifest.blockers == ["waiting on review"]

    def test_find_task_path_returns_manifest_location(self, manager, temp_dir):
        """RED: Should expose manifest path for a task in any state directory."""
        manager.create_task("TASK-001", started_by="cli")

        location, manifest_path = manager.find_task_path("TASK-001")

        assert location == "queue"
        assert manifest_path == temp_dir / "queue" / "TASK-001" / "manifest.json"

    def test_list_queue(self, manager):
        """RED: Should list all tasks in queue."""
        manager.create_task("TASK-001", started_by="cli")
        manager.create_task("TASK-002", started_by="discord:user")
        
        tasks = manager.list_queue()
        
        assert len(tasks) == 2
        assert {t.task_id for t in tasks} == {"TASK-001", "TASK-002"}

    def test_list_in_progress(self, manager):
        """RED: Should list all in-progress tasks."""
        manager.create_task("TASK-001", started_by="cli")
        manager.move_to_in_progress("TASK-001")
        
        tasks = manager.list_in_progress()
        
        assert len(tasks) == 1
        assert tasks[0].task_id == "TASK-001"

    def test_atomic_write(self, manager, temp_dir):
        """RED: Should use atomic write to prevent corruption."""
        manifest = manager.create_task("TASK-001", started_by="cli")
        
        # Save should use temp file + rename pattern
        manifest.increment_attempt()
        manager.save_task(manifest, subdir="queue")
        
        # File should exist and be valid JSON
        task_file = temp_dir / "queue" / "TASK-001" / "manifest.json"
        assert task_file.exists()
        data = json.loads(task_file.read_text())
        assert data["attempts"]["design"] == 1
