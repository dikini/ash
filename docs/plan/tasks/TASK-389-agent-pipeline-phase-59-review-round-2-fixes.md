# TASK-389: Agent Pipeline Phase 59 Review Round 2 Fixes

## Status: ✅ Done

## Description

Address the remaining post-closeout review findings for Phase 59 worktree isolation. Harden persisted task-id validation for on-disk task bundles before any worktree derivation/provisioning, make stale/prunable worktree reuse fail closed or reprovision deterministically instead of launching with a missing cwd, and preserve/report invalid persisted worktree metadata more accurately in CLI/operator surfaces.

## Specification Reference

- `tools/agent-pipeline/src/agent_pipeline/worktrees.py`: worktree derivation, reuse, and removal validation
- `tools/agent-pipeline/src/agent_pipeline/supervisor.py`: queue/restart provisioning and recovery behavior
- `tools/agent-pipeline/src/agent_pipeline/state.py`: persisted manifest/task-id/worktree metadata parsing and serialization
- `tools/agent-pipeline/src/agent_pipeline/cli.py`: cleanup/status handling for invalid worktree metadata
- `tools/agent-pipeline/tests/test_worktrees.py`: stale-reuse and path-validation coverage
- `tools/agent-pipeline/tests/test_supervisor.py`: on-disk malformed task/recovery coverage
- `tools/agent-pipeline/tests/test_state.py`: invalid metadata persistence behavior
- `tools/agent-pipeline/tests/test_cli.py`: cleanup/status behavior for invalid persisted metadata

## Dependencies

- ✅ TASK-378: Agent Pipeline Worktree Metadata and Provisioning
- ✅ TASK-379: Agent Pipeline Worktree Execution Roots
- ✅ TASK-380: Agent Pipeline Worktree CLI, Status, and Cleanup
- ✅ TASK-381: Agent Pipeline Worktree Recovery and Reuse
- ✅ TASK-382: Phase 59 Closeout
- ✅ TASK-388: Agent Pipeline Phase 59 Review Fixes

## Requirements

### Functional Requirements

1. Supervisor/worktree provisioning must reject persisted task ids that are not bundle-safe before deriving any worktree path.
2. Worktree reuse logic must not report `REUSE` when the expected worktree directory is missing on disk.
3. Cleanup/status surfaces must distinguish invalid persisted worktree metadata from absent metadata.
4. Invalid persisted worktree metadata diagnostics must survive load/save cycles where feasible, or fail closed without silently flattening to absent state.
5. Regression tests must cover malformed on-disk task ids, stale git worktree metadata with missing directories, and invalid worktree metadata cleanup/status behavior.

### Contract Requirements

1. All new failures must be concise, deterministic, and fail closed.
2. Existing Hermes-first runtime behavior and stage contracts must remain unchanged.
3. Documentation/tracking must reflect the new hardening work.

## TDD Steps

1. Add failing tests for malformed persisted task ids during provisioning/recovery.
2. Add failing tests for stale/prunable reuse where the directory is missing.
3. Add failing tests for invalid worktree metadata reporting in cleanup/status and persistence behavior.
4. Implement the minimum hardening changes needed to satisfy the tests.
5. Run targeted and full relevant verification.

## Completion Checklist

- [x] Persisted task ids are revalidated before worktree derivation/provisioning
- [x] Missing on-disk worktree directories are not treated as reusable
- [x] Invalid persisted worktree metadata is surfaced distinctly from absent metadata
- [x] New regression coverage added for malformed task ids, stale reuse, and invalid metadata
- [x] Relevant `tools/agent-pipeline/tests/*.py` pass
- [x] Full agent-pipeline test suite passes
- [x] `CHANGELOG.md` updated
