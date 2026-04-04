# TASK-376: Agent Pipeline Status Runtime Mapping and CLI Errors

## Status: ✅ Done

## Description

Fix the `tools/agent-pipeline` status/config surfaces so `ash-pipeline status --format json` reports the stage-agent mapping actually used by the running supervisor, and invalid stage-agent overrides fail with concise user-facing CLI errors instead of Python tracebacks.

## Specification Reference

- `tools/agent-pipeline/src/agent_pipeline/cli.py`: status JSON and CLI validation surface
- `tools/agent-pipeline/src/agent_pipeline/supervisor.py`: persisted runtime dashboard
- `tools/agent-pipeline/src/agent_pipeline/agents.py`: stage-agent validation rules
- `tools/agent-pipeline/README.md`: user-facing configuration/status contract

## Dependencies

- ✅ TASK-374: Agent Pipeline Configurable Stage Agents
- ✅ TASK-375: Agent Pipeline Status Effective Stage Agents

## Requirements

1. Status JSON must prefer the supervisor’s persisted effective stage-agent mapping when available.
2. The supervisor dashboard must persist the effective stage-agent mapping it is actually using.
3. Invalid stage-agent overrides must fail through a concise Click/user-facing error path, not an uncaught traceback.
4. Default behavior must remain backward-compatible when no dashboard/runtime mapping is present.

## TDD Steps

1. Add failing CLI tests for runtime dashboard stage-agent mapping visibility.
2. Add failing CLI tests that assert invalid `--stage-agents` input exits cleanly without a traceback.
3. Implement the minimum dashboard + CLI error-handling changes to satisfy the tests.
4. Run the full `tools/agent-pipeline` pytest suite and lint checks.

## Completion Checklist

- [x] Supervisor dashboard persists effective stage-agent mapping
- [x] Status JSON prefers runtime mapping when available
- [x] Invalid stage-agent overrides fail cleanly at the CLI surface
- [x] `tools/agent-pipeline/tests` passes
- [x] `ruff check tools/agent-pipeline/src tools/agent-pipeline/tests` passes
- [x] `README.md` updated
- [x] `CHANGELOG.md` updated
