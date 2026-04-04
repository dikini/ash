# Agent Pipeline

Multi-agent task orchestrator for the Ash workflow language project.

## Overview

This tool manages a pipeline of specialized agents:

```
design (codex) → spec_write (hermes) → spec_verify (codex) →
plan_write (hermes) → plan_verify (codex) →
impl (hermes) → qa (hermes) → validate (codex)
```

Each stage can retry up to 5 times before escalating to human review.

## Installation

```bash
cd tools/agent-pipeline
pip install -e .
```

### Install the packaged supervisor service

```bash
cd tools/agent-pipeline
./install.sh
```

The installer renders the systemd unit and CLI wrapper using the current clone path,
so the packaged deployment does not depend on the repository living under a fixed
`~/Projects/ash` location.

## Usage

### Queue a task

```bash
ash-pipeline queue TASK-370 --notify discord:user#1234
```

To seed a queue entry from an existing task/spec file:

```bash
ash-pipeline queue TASK-370 --from-spec docs/plan/tasks/TASK-370-example.md
```

This creates a task bundle under `.agents/queue/TASK-370/` with both `manifest.json`
and `task.md`, so later lifecycle moves keep the task description attached.

If the `--from-spec` path is invalid, the command exits without creating any queued
task state.

### Check status

```bash
ash-pipeline status
ash-pipeline status --task TASK-370
ash-pipeline status --watch
```

Aggregate status reports queued, in-progress, blocked, and completed task bundles. JSON output includes the effective `stage_agents` mapping, and when a supervisor is running it prefers the mapping persisted in `status/dashboard.json` so the status surface reflects the runtime agent selection actually in use.

### Configure workspace and state paths

By default, the CLI resolves the Ash workspace root from the repository and stores task
state under `<workspace-root>/.agents/`, instead of depending on the shell's current
working directory.

You can override this with either global flags or environment variables:

```bash
ash-pipeline --workspace-root /path/to/ash status
ash-pipeline --base-dir /tmp/ash-agent-state status
ash-pipeline --stage-agents '{"qa":"codex","design":"hermes"}' status

AGENT_PIPELINE_WORKSPACE_ROOT=/path/to/ash ash-pipeline status
AGENT_PIPELINE_BASE_DIR=/tmp/ash-agent-state ash-pipeline status
AGENT_PIPELINE_STAGE_AGENTS='{"qa":"codex","design":"hermes"}' ash-pipeline status
```

`AGENT_PIPELINE_STAGE_AGENTS` and `--stage-agents` accept a JSON object that maps stage
names to supported agent names. Partial overrides are allowed; unspecified stages keep
using the defaults.

Default stage-agent mapping:

```json
{
  "design": "codex",
  "spec_write": "hermes",
  "spec_verify": "codex",
  "plan_write": "hermes",
  "plan_verify": "codex",
  "impl": "hermes",
  "qa": "hermes",
  "validate": "codex"
}
```

Supported agent names are `codex` and `hermes`. Invalid JSON, unknown stage names, and
unknown agent names fail fast during CLI/supervisor configuration resolution, and CLI usage
surfaces them as concise Click errors instead of Python tracebacks. Reassigning
a stage only changes which supported agent runs it; the existing prompt builders and
artifact/review contracts for each stage remain unchanged.

The packaged systemd service sets both environment variables explicitly so runtime
state stays under `tools/agent-pipeline/.agents/` while stage agents still operate
against the repository workspace.

### Control tasks

```bash
ash-pipeline pause TASK-370
ash-pipeline resume TASK-370
ash-pipeline abort TASK-370
ash-pipeline steer TASK-370 --message "Focus on error handling"
```

### Run supervisor daemon

```bash
ash-pipeline daemon
```

### View event history

```bash
ash-pipeline events TASK-370
```

## Architecture

### State Management

Tasks are stored as JSON files in `.agents/`:

```
.agents/
├── queue/           # Pending task bundles
├── in-progress/     # Currently executing task bundles
├── done/            # Completed task bundles
├── blocked/         # Blocked task bundles (max retries exceeded)
├── control/         # Control commands (pause, abort, steer)
├── events/          # Event logs (JSONL)
└── status/          # Status dashboard
```

Each lifecycle directory stores one folder per task:

```
.agents/<state>/<task-id>/
├── manifest.json
└── task.md          # present for tasks queued with --from-spec
```

The pipeline now enforces contract-first stage artifacts without changing the external stage graph:

- design prompts require explicit Non-goals, Assumptions, and Traceability sections.
- spec and plan prompts reuse shared document-quality lenses, including Technical Critic, Pedagogical Critic, Style Critic, and a Traceability Matrix.
- impl success now requires `impl.complete`, `impl.summary.md`, and `impl.verification.md` so completion claims are backed by verification evidence.
- qa success/failure is represented by `qa.md` plus either `qa.verified` or `qa.review`.
- validate success/failure is represented by either `validated` or `validate.review`.
- review artifacts are fail-closed: `spec.review`, `plan.review`, `qa.review`, and `validate.review` block the task for human attention instead of consuming more retries.

### Agents

- **Codex** (`--yolo` mode): Design, verification, validation
  - Pedantic, thorough, good for catching spec issues
  
- **Hermes**: Spec writing, planning, implementation, QA
  - Direct work via subagent delegation

### Retry Logic

Each stage can retry up to 5 times:

1. Agent spawns for stage
2. If successful → advance to next stage
3. If failed → increment attempt, retry
4. After 5 failures → block task, notify human

### Control Commands

Tasks can be controlled via files in `.agents/control/`:

- `pause`: Stop after current stage
- `resume`: Continue from pause
- `abort`: Terminate immediately
- `steer`: Add guidance message to task context

## Testing

```bash
pytest tests/ -v
```

## Design Decisions

1. **File-based state**: No database required, observable, survives restarts
2. **Atomic writes**: Temp file + rename prevents corruption
3. **Append-only events**: Complete audit trail
4. **Local-only**: No network exposure, works on isolated systems
5. **Python implementation**: Practical until Ash can self-host

## Future: Dogfooding

When Ash has sufficient capabilities, this orchestrator should be rewritten as an Ash workflow:

```ash
workflow PipelineSupervisor {
  receive {
    QueueTask { task_id } => spawn TaskExecutor with task_id
    AgentComplete { task_id, result } => handle_completion(task_id, result)
  }
}
```

See `docs/ideas/minimal-core/MCE-009-TEST-WORKFLOWS.md` for the Ash Bowl of example workflows.
