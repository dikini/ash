# TASK-379: Agent Pipeline Worktree Execution Roots

## Status: ✅ Done

## Description

Run agent-pipeline stages from per-task worktree roots instead of the shared repository root, while keeping stage prompts and generated artifacts explicit about the distinction between repository workspace paths and `.agents/` task-bundle paths.

## Specification Reference

- `tools/agent-pipeline/src/agent_pipeline/agents.py`: stage launch cwd / `codex exec -C`
- `tools/agent-pipeline/src/agent_pipeline/prompt_contracts.py`: prompt wording and path contracts
- `tools/agent-pipeline/README.md`: user-facing execution model

## Dependencies

- 🟡 TASK-378: Agent Pipeline Worktree Metadata and Provisioning

## Requirements

### Functional Requirements

1. Codex stages must launch with cwd equal to the task worktree root.
2. Hermes stand-in stages must launch with cwd equal to the task worktree root.
3. `codex exec -C` must point to the task worktree root rather than the shared repository root.
4. Prompts must explicitly distinguish:
   - repository workspace root for code and repo-relative references;
   - task bundle directory for generated artifacts.
5. QA prompts must review implementation directly in the task worktree.
6. Existing stage artifact paths under `.agents/<state>/<task-id>/` must remain unchanged.

### Contract Requirements

1. Relative repository references from `task.md` must be interpreted from the worktree root.
2. Prompts must remove ambiguity about where code edits happen versus where stage artifacts are written.

## TDD Steps

1. Add failing spawner tests for cwd and `-C` using task worktree metadata.
2. Add failing prompt tests asserting explicit dual-root wording.
3. Implement the minimum spawner/prompt updates needed to satisfy the tests.
4. Run targeted pytest coverage for agent launch and prompt generation.

## Completion Checklist

- [x] Codex launches from task worktree root
- [x] Hermes launches from task worktree root
- [x] Prompts explicitly distinguish repo workspace and task bundle roots
- [x] QA prompt reviews code in task worktree
- [x] Artifact paths remain under `.agents/` task bundle
- [x] `tools/agent-pipeline/tests/test_agents.py` passes
- [x] `CHANGELOG.md` updated
