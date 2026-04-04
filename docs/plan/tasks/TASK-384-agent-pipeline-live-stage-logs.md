# TASK-384: Agent Pipeline Live Stage Logs

## Status: ✅ Done

## Description

Add live stdout/stderr capture for running agent-pipeline stages and expose a CLI/operator surface for peeking at those logs while a task is still in progress. This closes the current observability gap where process output is only available after the child process exits.

## Specification Reference

- `tools/agent-pipeline/src/agent_pipeline/agents.py`: child process launch and output capture
- `tools/agent-pipeline/src/agent_pipeline/supervisor.py`: active task supervision
- `tools/agent-pipeline/src/agent_pipeline/cli.py`: operator-facing status/log commands
- `tools/agent-pipeline/vila-integration.sh`: wrapper command surface
- `tools/agent-pipeline/README.md`: user-facing observability contract

## Dependencies

- ✅ TASK-371: Agent Pipeline Supervision Fixes
- ✅ TASK-376: Agent Pipeline Status Runtime Mapping and CLI Errors

## Requirements

### Functional Requirements

1. Running stages must stream or tee stdout to a persistent file under the task bundle.
2. Running stages must stream or tee stderr to a persistent file under the task bundle.
3. Log file names must be deterministic per stage, e.g. `<task_dir>/<stage>.stdout.log` and `<task_dir>/<stage>.stderr.log`.
4. The CLI must expose a command to read or tail live stage logs for a task.
5. The wrapper script should expose the same operator flow or clearly document the direct CLI usage.
6. Existing post-exit result handling must continue to work.

### Contract Requirements

1. Log capture must not block the supervisor or child process lifecycle.
2. Operators must be able to inspect logs while the process is still running.
3. Missing logs for stages that have not started must fail with concise user-facing output.
4. Documentation must explain where logs live and how to inspect them.

## TDD Steps

1. Add failing agent-spawner tests for teeing child stdout/stderr into deterministic log files.
2. Add failing supervisor/CLI tests for reading logs while a stage is still active.
3. Add failing CLI tests for user-facing errors when logs do not yet exist.
4. Implement the minimum agent launch, log persistence, and CLI surface needed to satisfy the tests.
5. Run targeted pytest coverage for agents, supervisor, and CLI behavior.

## Completion Checklist

- [x] Stage stdout is captured to persistent log files
- [x] Stage stderr is captured to persistent log files
- [x] Operators can inspect logs while stages are running
- [x] CLI log/tail surface exists and is usable
- [x] Wrapper/docs expose the flow clearly
- [x] Existing result handling remains intact
- [x] `tools/agent-pipeline/tests/test_agents.py` passes
- [x] `tools/agent-pipeline/tests/test_supervisor.py` passes
- [x] `tools/agent-pipeline/tests/test_cli.py` passes
- [x] `README.md` updated
- [x] `CHANGELOG.md` updated
