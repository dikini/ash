# TASK-388: Agent Pipeline Phase 59 Review Fixes

## Status: ✅ Done

## Description

Address the post-closeout review findings for Phase 59 worktree isolation. Tighten worktree cleanup safety so persisted manifest metadata cannot redirect cleanup outside the deterministic task worktree location, make verification instructions reproducible from the repo root, and align task tracking/docs with the actual Phase 59 completion state.

## Specification Reference

- `tools/agent-pipeline/src/agent_pipeline/cli.py`: cleanup-worktree command and status surfaces
- `tools/agent-pipeline/src/agent_pipeline/worktrees.py`: worktree removal safety checks
- `tools/agent-pipeline/src/agent_pipeline/state.py`: persisted worktree metadata validation
- `tools/agent-pipeline/tests/test_cli.py`: cleanup-worktree safety regression coverage
- `tools/agent-pipeline/tests/test_worktrees.py`: worktree cleanup validation coverage
- `docs/plan/tasks/TASK-378-agent-pipeline-worktree-metadata-and-provisioning.md`: task closeout consistency
- `docs/plan/tasks/TASK-382-phase-59-closeout.md`: reproducible verification commands
- `docs/plans/2026-04-04-agent-pipeline-worktree-isolation-plan.md`: Phase 59 plan verification section
- `tools/agent-pipeline/README.md`: operator verification instructions

## Dependencies

- ✅ TASK-378: Agent Pipeline Worktree Metadata and Provisioning
- ✅ TASK-379: Agent Pipeline Worktree Execution Roots
- ✅ TASK-380: Agent Pipeline Worktree CLI, Status, and Cleanup
- ✅ TASK-381: Agent Pipeline Worktree Recovery and Reuse
- ✅ TASK-382: Phase 59 Closeout

## Requirements

### Functional Requirements

1. `cleanup-worktree` must refuse to remove a worktree when persisted worktree metadata does not match the deterministic expected assignment for the task.
2. Worktree removal helpers must hard-enforce that cleanup targets stay under `<repo-root>/.worktrees/<TASK-ID>`.
3. Cleanup safety regressions must be covered by tests that exercise the real validation path rather than only mocking the remover.
4. Phase 59 verification docs must use reproducible commands from the repo root.
5. TASK-378 and related closeout docs must consistently reflect completion state.

### Contract Requirements

1. User-facing cleanup errors must remain concise and specific.
2. Verification instructions must be copy-pasteable from the repo root without hidden environment assumptions.
3. Documentation updates must preserve the current Hermes-first pipeline contract.

## TDD Steps

1. Add failing cleanup-worktree tests for mismatched or out-of-root worktree metadata.
2. Add failing lower-level worktree-removal tests for path validation.
3. Implement the minimum cleanup validation changes needed to satisfy the tests.
4. Update closeout/verification docs to use reproducible commands.
5. Run targeted and full relevant verification.

## Completion Checklist

- [x] cleanup-worktree rejects mismatched persisted worktree metadata
- [x] worktree removal enforces repo-root/.worktrees containment
- [x] cleanup safety regressions are test-covered
- [x] Phase 59 verification docs are reproducible from repo root
- [x] TASK-378/closeout docs are internally consistent
- [x] `tools/agent-pipeline/tests/test_cli.py` passes
- [x] `tools/agent-pipeline/tests/test_worktrees.py` passes
- [x] relevant full agent-pipeline tests pass
- [x] `CHANGELOG.md` updated
