# TASK-375: Agent Pipeline Status Effective Stage Agents

## Status: ✅ Complete

## Description

Expose the effective stage-agent mapping in `ash-pipeline status --format json` so runtime configuration is directly observable from the status surface.

## Specification Reference

- `tools/agent-pipeline/src/agent_pipeline/cli.py`: status JSON output
- `tools/agent-pipeline/src/agent_pipeline/agents.py`: default and resolved stage-agent mapping
- `tools/agent-pipeline/README.md`: user-facing configuration contract

## Dependencies

- ✅ TASK-374: Agent Pipeline Configurable Stage Agents

## Requirements

1. Aggregate status JSON output must include the effective stage-agent mapping.
2. The mapping shown must reflect runtime resolution, including partial overrides.
3. Default behavior must remain backward-compatible for text output.
4. Documentation must mention that status JSON exposes the active mapping.

## TDD Steps

1. Add failing CLI tests for default and overridden stage-agent mapping in status JSON.
2. Implement the minimum CLI/status changes to surface the effective mapping.
3. Run the full `tools/agent-pipeline` pytest suite and lint checks.

## Completion Checklist

- [x] Status JSON includes effective stage-agent mapping
- [x] Override-aware mapping is shown correctly
- [x] Text status output remains unchanged unless intentionally documented
- [x] `tools/agent-pipeline/tests` passes
- [x] `ruff check tools/agent-pipeline/src tools/agent-pipeline/tests` passes
- [x] `README.md` updated
- [x] `CHANGELOG.md` updated
