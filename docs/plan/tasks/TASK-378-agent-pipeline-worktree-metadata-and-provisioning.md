# TASK-378: Agent Pipeline Worktree Metadata and Provisioning

## Status: ✅ Done

## Description

Add per-task worktree metadata and provisioning support to `tools/agent-pipeline` so queued tasks can be assigned a deterministic isolated git worktree before stage execution begins, while preserving the existing `.agents/<state>/<task-id>/` task-bundle artifact model.

## Specification Reference

- `tools/agent-pipeline/src/agent_pipeline/state.py`: task manifest persistence
- `tools/agent-pipeline/src/agent_pipeline/supervisor.py`: task startup/provisioning path
- `tools/agent-pipeline/README.md`: task/workspace operational contract
- `software-development/using-git-worktrees` skill: project-local worktree safety rules

## Dependencies

- ✅ TASK-371: Agent Pipeline Supervision Fixes
- ✅ TASK-377: Ignore Caches and Build Artifacts
- 🟡 TASK-383: Agent Pipeline Task Dependency Gating

## Requirements

### Functional Requirements

1. Each task manifest must be able to persist worktree metadata, including path and branch name.
2. Worktree path derivation must be deterministic: `<repo-root>/.worktrees/<TASK-ID>`.
3. Branch name derivation must be deterministic: `agent-pipeline/<TASK-ID>`.
4. The pipeline must verify that `.worktrees/` is git-ignored before provisioning project-local worktrees.
5. The supervisor must provision or reuse a task worktree before first stage launch.
6. Provisioning must not move or duplicate task-bundle artifacts under `.agents/`.

### Contract Requirements

1. Failed worktree provisioning must surface as a clear blocking error.
2. Existing matching worktrees must be reused when safe instead of recreated blindly.
3. Persisted metadata must survive supervisor restarts.

## TDD Steps

1. Add failing tests for manifest round-trip of worktree metadata.
2. Add failing tests for deterministic worktree/branch derivation and ignore verification.
3. Add failing supervisor tests proving first-stage launch requires successful provisioning.
4. Implement the minimum state/worktree manager/supervisor changes needed to satisfy the tests.
5. Run targeted pytest coverage for state, supervisor, and worktree manager logic.

## Completion Checklist

- [x] Manifest persists worktree metadata
- [x] `.worktrees/` ignore verification enforced
- [x] Deterministic worktree path and branch naming implemented
- [x] Supervisor provisions/reuses task worktree before first launch
- [x] Provisioning errors block task clearly
- [x] `tools/agent-pipeline/tests/test_state.py` passes
- [x] `tools/agent-pipeline/tests/test_supervisor.py` passes
- [x] `tools/agent-pipeline/tests/test_worktrees.py` passes
- [x] `CHANGELOG.md` updated
