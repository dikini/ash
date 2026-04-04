# TASK-371: Agent Pipeline Supervision Fixes

## Status: ✅ Complete

## Description

Fix the Python agent-pipeline orchestrator under `tools/agent-pipeline/` so it supervises child agents asynchronously, preserves task artifacts across state transitions, exposes completed tasks through status APIs, and removes hard-coded workspace assumptions.

## Requirements

1. `Supervisor.run_once()` must not crash because `check_active_tasks()` is missing.
2. Agent launch must be non-blocking so pause/abort/steer and queued work can still be processed while a stage is running.
3. Task state transitions must move the full task bundle (`manifest.json`, `task.md`, stage artifacts), not just a flat manifest file.
4. Abort must terminate the actual live agent process and persist the task out of `in-progress`.
5. Status lookups must find tasks in queue, in-progress, blocked, and done.
6. Agent execution must derive workspace/executable configuration without a hard-coded `/home/dikini/Projects/ash` path.

## TDD Steps

1. Add failing state-layout tests for task bundle creation, moves, and cross-state lookup.
2. Add failing agent-spawner tests for non-blocking launch, result polling, and configurable execution roots.
3. Add failing supervisor tests for active process tracking, async completion polling, and abort semantics.
4. Add failing CLI tests for queue bundle layout and done-task status visibility.
5. Implement the smallest changes to make each focused test set pass.
6. Run the full `tools/agent-pipeline` pytest suite and lint checks.

## Completion Checklist

- [x] Task bundles live under per-task directories with `manifest.json`
- [x] `check_active_tasks()` exists and supervises live processes
- [x] Abort kills real running agents and updates persisted task state
- [x] Completed tasks appear in supervisor and CLI status responses
- [x] Hard-coded workspace path removed from agent execution
- [x] `tools/agent-pipeline/tests` passes
- [x] `ruff check tools/agent-pipeline/src tools/agent-pipeline/tests` passes
- [x] `CHANGELOG.md` updated
