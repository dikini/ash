# TASK-387: Agent Pipeline Hermes-Only Default Stage Handlers

## Status: ✅ Done

## Description

Switch the default `tools/agent-pipeline` stage-agent mapping so every pipeline stage runs through Hermes instead of Codex. The pipeline should stop depending on Codex tokens for normal operation while preserving the existing stage contracts, artifact expectations, and override surface for future agent reassignment.

## Specification Reference

- `tools/agent-pipeline/src/agent_pipeline/agents.py`: default stage-agent mapping and per-stage command preparation
- `tools/agent-pipeline/src/agent_pipeline/cli.py`: status exposure of effective stage-agent mapping
- `tools/agent-pipeline/src/agent_pipeline/supervisor.py`: runtime stage-agent reporting
- `tools/agent-pipeline/tests/test_agents.py`: default/override mapping and launch behavior
- `tools/agent-pipeline/tests/test_cli.py`: status JSON default mapping coverage
- `tools/agent-pipeline/tests/test_supervisor.py`: supervisor default mapping coverage
- `tools/agent-pipeline/README.md`: documented default mapping and operator guidance
- `tools/agent-pipeline/VILA-INTEGRATION.md`: integration docs for current runtime agent behavior

## Dependencies

- ✅ TASK-383: Agent Pipeline Task Dependency Gating
- ✅ TASK-384: Agent Pipeline Live Stage Logs
- ✅ TASK-385: Agent Pipeline Feedback Resolution and Retry Guidance
- ✅ TASK-386: Agent Pipeline Feedback Retry Helper

## Requirements

### Functional Requirements

1. Default stage-agent mapping must assign Hermes to every pipeline stage.
2. Existing `--stage-agents` / `AGENT_PIPELINE_STAGE_AGENTS` overrides must continue to work for supported agent names.
3. Default runtime launches must no longer require the `codex` executable for any stage.
4. Hermes command preparation must support all stages that now default to Hermes, including design/verify/validate stages.
5. Status surfaces must report the Hermes-only default mapping accurately.
6. Existing artifact/review contracts for each stage must remain unchanged.

### Contract Requirements

1. The change must preserve operator-visible stage names and task lifecycle semantics.
2. The change must remain reversible through explicit stage-agent overrides.
3. Documentation must reflect that Hermes now handles all stages by default.
4. CHANGELOG must record the default-agent shift and any runtime contract implications.

## TDD Steps

1. Add failing tests for Hermes-only default stage-agent mapping in agent, supervisor, and CLI status surfaces.
2. Add failing tests showing default launch paths no longer invoke Codex for default stages.
3. Implement the minimum agent/runtime/doc changes needed to satisfy the tests.
4. Run focused and full relevant pytest coverage plus lint/compile checks.

## Completion Checklist

- [x] Default stage-agent mapping is Hermes-only
- [x] Default launch path no longer depends on Codex for pipeline stages
- [x] Stage-agent overrides still work
- [x] `tools/agent-pipeline/tests/test_agents.py` passes
- [x] `tools/agent-pipeline/tests/test_cli.py` passes
- [x] `tools/agent-pipeline/tests/test_supervisor.py` passes
- [x] `README.md` updated
- [x] `VILA-INTEGRATION.md` updated
- [x] `CHANGELOG.md` updated
