# TASK-381: Agent Pipeline Worktree Recovery and Reuse

## Status: ✅ Done

## Description

Harden the agent-pipeline worktree flow so supervisor restarts, reused task ids, and pre-existing worktree directories behave predictably without duplicating branches/worktrees or losing operator visibility into task workspace state.

## Specification Reference

- `tools/agent-pipeline/src/agent_pipeline/supervisor.py`: restart and task lifecycle behavior
- `tools/agent-pipeline/src/agent_pipeline/worktrees.py`: reuse/recovery logic
- `tools/agent-pipeline/src/agent_pipeline/cli.py`: cleanup/status visibility

## Dependencies

- 🟡 TASK-378: Agent Pipeline Worktree Metadata and Provisioning
- 🟡 TASK-379: Agent Pipeline Worktree Execution Roots
- 🟡 TASK-380: Agent Pipeline Worktree CLI, Status, and Cleanup

## Requirements

### Functional Requirements

1. Supervisor restart must reuse persisted worktree metadata instead of creating duplicate worktrees.
2. Existing matching worktree directories/branches must be reused safely when they correspond to the task metadata.
3. Mismatched pre-existing worktree/branch state must fail clearly instead of being silently overwritten.
4. Status output must preserve worktree metadata across queue, in-progress, blocked, and done transitions.
5. Cleanup must clear or update persisted metadata consistently after removal.

### Contract Requirements

1. Recovery behavior must be deterministic and test-covered.
2. Reuse logic must be explicit about which states are accepted versus blocked.

## TDD Steps

1. Add failing supervisor/worktree tests for restart reuse and duplicate-avoidance behavior.
2. Add failing CLI/status tests for metadata persistence across lifecycle transitions.
3. Implement the minimum recovery/reuse logic needed to satisfy the tests.
4. Run targeted pytest coverage for supervisor, worktree manager, and CLI persistence behavior.

## Completion Checklist

- [x] Restart reuses persisted worktree metadata safely
- [x] Duplicate worktree creation avoided
- [x] Mismatched existing state fails clearly
- [x] Metadata persists across task lifecycle transitions
- [x] Cleanup updates persisted metadata consistently
- [x] `tools/agent-pipeline/tests/test_supervisor.py` passes
- [x] `tools/agent-pipeline/tests/test_cli.py` passes
- [x] `CHANGELOG.md` updated
