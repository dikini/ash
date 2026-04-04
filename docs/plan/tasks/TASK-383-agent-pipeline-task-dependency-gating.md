# TASK-383: Agent Pipeline Task Dependency Gating

## Status: ✅ Done

## Description

Add task-level dependency gating to `tools/agent-pipeline` so separately queued tasks can declare prerequisite task ids and remain queued until those dependencies are complete. This prevents dependent work bundles from starting concurrently when they must execute sequentially.

## Specification Reference

- `tools/agent-pipeline/src/agent_pipeline/state.py`: task manifest persistence
- `tools/agent-pipeline/src/agent_pipeline/supervisor.py`: queue admission and task start logic
- `tools/agent-pipeline/src/agent_pipeline/cli.py`: queue/status surfaces
- `tools/agent-pipeline/README.md`: operator-facing scheduling contract

## Dependencies

- ✅ TASK-371: Agent Pipeline Supervision Fixes
- ✅ TASK-376: Agent Pipeline Status Runtime Mapping and CLI Errors

## Requirements

### Functional Requirements

1. Task manifests must support a `dependencies` field containing prerequisite task ids.
2. Tasks with incomplete dependencies must remain in `queue` and must not move to `in-progress`.
3. A dependency is satisfied only when the prerequisite task is in `done` / `complete` state.
4. Aggregate and per-task status must surface when a task is waiting on unmet dependencies.
5. Queueing must allow dependencies to be set explicitly, whether by CLI flag or direct manifest creation path.
6. Tasks without dependencies must keep current behavior.

### Contract Requirements

1. Missing dependency ids must fail clearly or surface as unresolved blockers rather than being silently ignored.
2. Dependency gating must not interfere with normal stage progression inside a single task.
3. The behavior must be deterministic when multiple queued tasks depend on the same prerequisite.
4. Documentation must clearly explain that queue order alone does not imply dependency order; explicit dependencies do.

## TDD Steps

1. Add failing state tests for manifest round-trip of dependency metadata.
2. Add failing supervisor tests proving dependent tasks remain queued until prerequisites are complete.
3. Add failing CLI tests for queueing tasks with dependencies and for status output showing unmet dependencies.
4. Implement the minimum state/supervisor/CLI changes needed to satisfy the tests.
5. Run targeted pytest coverage for state, supervisor, and CLI behavior.

## Completion Checklist

- [x] Manifest persists dependency metadata
- [x] Dependent tasks remain queued until prerequisites complete
- [x] Tasks without dependencies keep current behavior
- [x] Status output shows unmet dependencies clearly
- [x] Queue surface supports explicit dependency input
- [x] `tools/agent-pipeline/tests/test_state.py` passes
- [x] `tools/agent-pipeline/tests/test_supervisor.py` passes
- [x] `tools/agent-pipeline/tests/test_cli.py` passes
- [x] `README.md` updated
- [x] `CHANGELOG.md` updated
