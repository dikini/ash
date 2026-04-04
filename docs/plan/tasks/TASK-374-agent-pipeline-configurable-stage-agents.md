# TASK-374: Agent Pipeline Configurable Stage Agents

## Status: ✅ Complete

## Description

Make the agents assigned to individual `tools/agent-pipeline` stages configurable while preserving the current default behavior, external orchestration model, and existing prompt/artifact quality contracts.

## Specification Reference

- `tools/agent-pipeline/src/agent_pipeline/agents.py`: current hard-coded stage-agent mapping
- `tools/agent-pipeline/src/agent_pipeline/cli.py`: runtime configuration entry points
- `tools/agent-pipeline/src/agent_pipeline/supervisor.py`: orchestrator construction path
- `tools/agent-pipeline/README.md`: user-facing configuration contract

## Dependencies

- ✅ TASK-371: Agent Pipeline Supervision Fixes
- ✅ TASK-372: Agent Pipeline Packaging Review Fixes
- ✅ TASK-373: Agent Pipeline Quality Contracts

## Requirements

### Functional Requirements

1. Stage-to-agent selection must be configurable at runtime instead of fixed in code.
2. The current stage-agent mapping must remain the default when no override is provided.
3. Partial overrides must be supported without requiring a full mapping definition.
4. Invalid stage names or invalid agent names must fail clearly.
5. CLI, supervisor, and spawner must resolve the same effective mapping.
6. Existing prompt/artifact contracts must remain intact regardless of which supported agent is assigned to a stage.

### Contract Requirements

1. Supported agent names must be explicit and validated against `AgentType`.
2. Stage names must be explicit and validated against `Stage`.
3. Configuration resolution must be documented for users and packaged deployments.
4. Default behavior must stay backward-compatible.

## TDD Steps

1. Add failing spawner tests for default mappings, partial overrides, and invalid override validation.
2. Add failing CLI/supervisor tests for propagating a configured stage-agent mapping.
3. Implement the minimum configuration parsing and mapping resolution needed to satisfy the tests.
4. Run the full `tools/agent-pipeline` pytest suite and lint checks.

## Completion Checklist

- [x] Runtime stage-agent mapping support added
- [x] Default mapping preserved
- [x] Partial overrides supported
- [x] Invalid mappings rejected clearly
- [x] CLI and supervisor use shared mapping resolution
- [x] `tools/agent-pipeline/tests` passes
- [x] `ruff check tools/agent-pipeline/src tools/agent-pipeline/tests` passes
- [x] `README.md` updated
- [x] `CHANGELOG.md` updated
