# TASK-392: Agent Pipeline Phase 59 Review Round 5 Fixes

## Status: ✅ Done

## Description

Address the final known Phase 59 review findings after TASK-391. Fail closed when a configured workspace root is missing instead of letting the supervisor crash, make base-dir-only cleanup derive the repo root safely for malformed absolute worktree paths, and align README cleanup wording with the implemented prune-failure state-honesty behavior.

## Specification Reference

- `tools/agent-pipeline/src/agent_pipeline/supervisor.py`: configured workspace-root handling
- `tools/agent-pipeline/src/agent_pipeline/worktrees.py`: provisioning subprocess cwd behavior
- `tools/agent-pipeline/src/agent_pipeline/cli.py`: cleanup repo-root derivation and error handling
- `tools/agent-pipeline/tests/test_supervisor.py`: missing configured workspace-root fail-closed coverage
- `tools/agent-pipeline/tests/test_cli.py`: malformed absolute worktree path cleanup coverage
- `tools/agent-pipeline/README.md`: cleanup semantics wording

## Dependencies

- ✅ TASK-378: Agent Pipeline Worktree Metadata and Provisioning
- ✅ TASK-379: Agent Pipeline Worktree Execution Roots
- ✅ TASK-380: Agent Pipeline Worktree CLI, Status, and Cleanup
- ✅ TASK-381: Agent Pipeline Worktree Recovery and Reuse
- ✅ TASK-382: Phase 59 Closeout
- ✅ TASK-388: Agent Pipeline Phase 59 Review Fixes
- ✅ TASK-389: Agent Pipeline Phase 59 Review Round 2 Fixes
- ✅ TASK-390: Agent Pipeline Phase 59 Review Round 3 Fixes
- ✅ TASK-391: Agent Pipeline Phase 59 Review Round 4 Fixes

## Requirements

### Functional Requirements

1. Supervisor must block tasks cleanly when a configured workspace root is missing or unusable instead of crashing.
2. Base-dir-only cleanup must fail closed with a concise user-facing error for malformed but absolute persisted worktree paths instead of raising `IndexError`.
3. README must mention that cleanup also clears metadata when removal succeeded but prune failed afterward.
4. Regression tests must cover missing configured workspace roots and malformed absolute worktree cleanup paths.

### Contract Requirements

1. Existing Hermes-first runtime and Phase 59 worktree contracts must remain unchanged.
2. New failures must be concise, deterministic, and fail closed.
3. No path-safety regressions may be introduced.

## TDD Steps

1. Add failing tests for missing configured workspace-root fail-closed behavior.
2. Add failing tests for malformed absolute worktree paths in base-dir-only cleanup.
3. Implement the minimum supervisor/CLI/doc changes needed to satisfy the tests.
4. Run targeted and full relevant verification.

## Completion Checklist

- [x] Missing configured workspace roots fail closed without crashing supervisor
- [x] Base-dir-only cleanup rejects malformed absolute worktree paths cleanly
- [x] README cleanup semantics match implementation
- [x] Relevant `tools/agent-pipeline/tests/test_supervisor.py` passes
- [x] Relevant `tools/agent-pipeline/tests/test_cli.py` passes
- [x] Full agent-pipeline test suite passes
- [x] `CHANGELOG.md` updated
