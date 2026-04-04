# TASK-380: Agent Pipeline Worktree CLI, Status, and Cleanup

## Status: ✅ Done

## Description

Expose worktree metadata through the agent-pipeline CLI/status surfaces and add explicit, safe worktree cleanup commands so operators can inspect and manage task workspaces without relying on ad hoc git commands.

## Specification Reference

- `tools/agent-pipeline/src/agent_pipeline/cli.py`: status and operator commands
- `tools/agent-pipeline/vila-integration.sh`: operator wrapper command surface
- `tools/agent-pipeline/README.md`: worktree operational guidance

## Dependencies

- 🟡 TASK-378: Agent Pipeline Worktree Metadata and Provisioning
- 🟡 TASK-379: Agent Pipeline Worktree Execution Roots

## Requirements

### Functional Requirements

1. Single-task and aggregate status output must expose worktree path and branch when available.
2. JSON status output must include worktree metadata fields.
3. The CLI must provide an explicit cleanup command for removing finished-task worktrees.
4. Cleanup must refuse to remove worktrees for tasks still in queue or in progress.
5. Cleanup must update persisted task metadata/state consistently.
6. The operator wrapper should expose the cleanup flow or document the direct CLI command clearly.

### Contract Requirements

1. Cleanup failures must surface concise user-facing errors.
2. Status output must remain readable and backward-compatible when no worktree metadata is present.

## TDD Steps

1. Add failing CLI tests for worktree metadata in text and JSON status output.
2. Add failing CLI tests for safe cleanup refusal on active tasks.
3. Add failing CLI tests for successful cleanup on blocked/done tasks.
4. Implement the minimum CLI/wrapper/README changes needed to satisfy the tests.
5. Run targeted pytest coverage for CLI behavior.

## Completion Checklist

- [x] Status text output includes worktree metadata
- [x] Status JSON includes worktree metadata
- [x] Cleanup command exists and is safe for active tasks
- [x] Cleanup works for blocked/done tasks
- [x] Wrapper/docs expose cleanup flow clearly
- [x] `tools/agent-pipeline/tests/test_cli.py` passes
- [x] `README.md` updated
- [x] `CHANGELOG.md` updated
