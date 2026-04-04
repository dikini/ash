# TASK-385: Agent Pipeline Feedback Resolution and Retry Guidance

## Status: ✅ Done

## Description

Add a structured feedback-resolution workflow to `tools/agent-pipeline` so blocked or review-failed tasks can carry an explicit operator-authored resolution artifact into the next retry, instead of relying only on ad hoc steering text. This should preserve review provenance, make retry guidance auditable, and support a clean steer/requeue or steer/resume loop without changing the external stage graph.

## Specification Reference

- `tools/agent-pipeline/src/agent_pipeline/cli.py`: operator-facing review resolution commands and status output
- `tools/agent-pipeline/src/agent_pipeline/supervisor.py`: retry/resume handoff behavior and artifact reading surfaces
- `tools/agent-pipeline/src/agent_pipeline/agents.py`: stage prompt inputs for retries after review feedback
- `tools/agent-pipeline/vila-integration.sh`: wrapper command surface
- `tools/agent-pipeline/README.md`: operator feedback-resolution contract

## Dependencies

- ✅ TASK-371: Agent Pipeline Supervision Fixes
- ✅ TASK-373: Agent Pipeline Quality Contracts
- ✅ TASK-383: Agent Pipeline Task Dependency Gating
- ✅ TASK-384: Agent Pipeline Live Stage Logs

## Requirements

### Functional Requirements

1. The task bundle must support a dedicated feedback-resolution artifact, e.g. `feedback-resolution.md` or `review-resolution.md`.
2. Operators must be able to attach structured retry guidance to a task without manually editing bundle files.
3. The resolution artifact must identify the source review artifact it is responding to, e.g. `spec.review`, `plan.review`, `qa.review`, or `validate.review`.
4. Stage retries after review feedback must read both the original review artifact and the structured resolution artifact when present.
5. The workflow must support the common operator pattern of: inspect review → write resolution → requeue or resume.
6. Existing plain `steering.md` guidance must continue to work.

### Contract Requirements

1. Feedback resolution must be stored in the task bundle as canonical workflow state; it must not rely only on transient chat history.
2. Status output should surface whether a task has a feedback-resolution artifact and which review artifact it addresses.
3. Retry guidance must remain auditable across supervisor restarts and task lifecycle moves.
4. The feature must not require automatic queueing; queueing or resuming remains an explicit operator action.

## TDD Steps

1. Add failing CLI tests for creating or updating a structured feedback-resolution artifact for a task.
2. Add failing supervisor or prompt-construction tests proving retries read both review and resolution artifacts.
3. Add failing status tests proving tasks surface feedback-resolution presence and review source.
4. Add failing wrapper tests or documented wrapper expectations for the operator flow.
5. Implement the minimum state/CLI/prompt/wrapper changes needed to satisfy the tests.
6. Run targeted pytest coverage for CLI, supervisor, and prompt behavior.

## Completion Checklist

- [x] Task bundle persists a structured feedback-resolution artifact
- [x] CLI/operator surface can write retry guidance without manual file edits
- [x] Retry prompts consume both review feedback and the operator resolution
- [x] Status output surfaces feedback-resolution presence clearly
- [x] Wrapper/docs expose the flow clearly
- [x] `tools/agent-pipeline/tests/test_cli.py` passes
- [x] `tools/agent-pipeline/tests/test_supervisor.py` passes
- [x] `tools/agent-pipeline/tests/test_agents.py` passes
- [x] `README.md` updated
- [x] `CHANGELOG.md` updated
