# TASK-391: Agent Pipeline Phase 59 Review Round 4 Fixes

## Status: ✅ Done

## Description

Address the remaining Phase 59 review findings after TASK-390. Harden stale-worktree recovery against real git registration state, make `cleanup-worktree` robust when only `--base-dir` is supplied outside the repo, and ensure manifest metadata remains consistent when destructive cleanup succeeds but `git worktree prune` fails afterward.

## Specification Reference

- `tools/agent-pipeline/src/agent_pipeline/worktrees.py`: stale-worktree recovery and cleanup semantics
- `tools/agent-pipeline/src/agent_pipeline/cli.py`: cleanup repo-root resolution and manifest-state updates
- `tools/agent-pipeline/src/agent_pipeline/state.py`: persisted worktree metadata updates after cleanup outcomes
- `tools/agent-pipeline/tests/test_worktrees.py`: stale registration and prune-failure coverage
- `tools/agent-pipeline/tests/test_cli.py`: base-dir-only cleanup and post-prune-failure behavior
- `tools/agent-pipeline/README.md`: reproducible verification / cleanup operator guidance

## Dependencies

- ✅ TASK-378: Agent Pipeline Worktree Metadata and Provisioning
- ✅ TASK-379: Agent Pipeline Worktree Execution Roots
- ✅ TASK-380: Agent Pipeline Worktree CLI, Status, and Cleanup
- ✅ TASK-381: Agent Pipeline Worktree Recovery and Reuse
- ✅ TASK-382: Phase 59 Closeout
- ✅ TASK-388: Agent Pipeline Phase 59 Review Fixes
- ✅ TASK-389: Agent Pipeline Phase 59 Review Round 2 Fixes
- ✅ TASK-390: Agent Pipeline Phase 59 Review Round 3 Fixes

## Requirements

### Functional Requirements

1. Stale registered worktrees with missing on-disk directories must recover deterministically instead of relying on a plain `git worktree add` happy path.
2. `cleanup-worktree` must work robustly when only `--base-dir` is supplied and valid persisted worktree metadata already points to the task worktree.
3. If worktree removal succeeds but prune fails, persisted manifest metadata must not continue claiming the removed worktree still exists.
4. Cleanup failures must remain concise and fail closed while keeping persisted state honest.
5. Regression tests must cover stale registered worktrees, base-dir-only cleanup, and prune-failure consistency.

### Contract Requirements

1. Existing Hermes-first runtime and stage contracts must remain unchanged.
2. Worktree cleanup must never regress on containment/path-safety checks.
3. Operator docs must reflect the strengthened cleanup/recovery behavior.

## TDD Steps

1. Add failing tests for stale registered worktree recovery, base-dir-only cleanup, and prune-failure metadata consistency.
2. Implement the minimum worktree/CLI/state changes needed to satisfy the tests.
3. Run targeted and full relevant verification.

## Completion Checklist

- [x] Stale registered worktrees recover deterministically
- [x] Base-dir-only cleanup works outside the repo when persisted metadata is valid
- [x] Prune failure no longer leaves stale manifest worktree metadata behind
- [x] New regression coverage added for all three cases
- [x] Relevant `tools/agent-pipeline/tests/test_worktrees.py` passes
- [x] Relevant `tools/agent-pipeline/tests/test_cli.py` passes
- [x] Full agent-pipeline test suite passes
- [x] `CHANGELOG.md` updated
