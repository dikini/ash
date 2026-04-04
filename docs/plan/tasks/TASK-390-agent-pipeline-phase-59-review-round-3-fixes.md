# TASK-390: Agent Pipeline Phase 59 Review Round 3 Fixes

## Status: ✅ Done

## Description

Address the remaining Phase 59 review findings after TASK-389. Ensure the supervisor honors the configured workspace root for worktree provisioning/execution, surface invalid persisted worktree metadata distinctly in aggregate status output, and align README agent-role documentation with the current Hermes-first runtime behavior.

## Specification Reference

- `tools/agent-pipeline/src/agent_pipeline/cli.py`: supervisor construction and aggregate/single-task status surfaces
- `tools/agent-pipeline/src/agent_pipeline/supervisor.py`: repository root selection for provisioning and restart/recovery flows
- `tools/agent-pipeline/src/agent_pipeline/agents.py`: configured workspace root propagation into spawned stages
- `tools/agent-pipeline/README.md`: current runtime/agent-role documentation
- `tools/agent-pipeline/tests/test_cli.py`: status visibility and configured workspace-root behavior
- `tools/agent-pipeline/tests/test_supervisor.py`: configured workspace-root provisioning behavior

## Dependencies

- ✅ TASK-378: Agent Pipeline Worktree Metadata and Provisioning
- ✅ TASK-379: Agent Pipeline Worktree Execution Roots
- ✅ TASK-380: Agent Pipeline Worktree CLI, Status, and Cleanup
- ✅ TASK-381: Agent Pipeline Worktree Recovery and Reuse
- ✅ TASK-382: Phase 59 Closeout
- ✅ TASK-388: Agent Pipeline Phase 59 Review Fixes
- ✅ TASK-389: Agent Pipeline Phase 59 Review Round 2 Fixes

## Requirements

### Functional Requirements

1. The CLI must pass the resolved workspace root into Supervisor construction.
2. Supervisor repository-root discovery must honor the configured workspace root deterministically instead of heuristic rediscovery when one was supplied.
3. Aggregate human-readable status output must surface invalid persisted worktree metadata distinctly from absent metadata.
4. README agent-role/runtime descriptions must match the current Hermes-first implementation.
5. Regression tests must cover configured workspace-root propagation and aggregate invalid-metadata visibility.

### Contract Requirements

1. Workspace-root handling must preserve current external CLI semantics.
2. Aggregate status output must remain readable while adding malformed-state visibility.
3. Docs must not reintroduce stale Codex-default assumptions.

## TDD Steps

1. Add failing tests for supervisor honoring configured workspace root during provisioning.
2. Add failing CLI tests for aggregate invalid worktree metadata visibility.
3. Implement the minimum propagation/status/doc changes needed to satisfy the tests.
4. Run targeted and full relevant verification.

## Completion Checklist

- [x] Supervisor honors configured workspace root for provisioning/execution
- [x] Aggregate status surfaces invalid worktree metadata distinctly
- [x] README reflects Hermes-first runtime behavior accurately
- [x] Relevant `tools/agent-pipeline/tests/test_cli.py` passes
- [x] Relevant `tools/agent-pipeline/tests/test_supervisor.py` passes
- [x] Full agent-pipeline test suite passes
- [x] `CHANGELOG.md` updated
