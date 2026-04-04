# Agent Pipeline

Multi-agent task orchestrator for the Ash workflow language project.

## Overview

This tool manages a pipeline of specialized agents:

```
design (hermes) → spec_write (hermes) → spec_verify (hermes) →
plan_write (hermes) → plan_verify (hermes) →
impl (hermes) → qa (hermes) → validate (hermes)
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

Task ids are validated as bundle-safe identifiers: letters, numbers, dots, underscores, and hyphens only. Path-style ids or ids containing separators are rejected.

To seed a queue entry from an existing task/spec file:

```bash
ash-pipeline queue TASK-370 --from-spec docs/plan/tasks/TASK-370-example.md
```

This creates a task bundle under `.agents/queue/TASK-370/` with both `manifest.json`
and `task.md`, so later lifecycle moves keep the task description attached.

You can declare prerequisite tasks explicitly and keep dependent work queued until they finish:

```bash
ash-pipeline queue TASK-371 --depends-on TASK-370
ash-pipeline queue TASK-372 --depends-on TASK-370 --depends-on TASK-371
```

Queue order alone does not imply dependency order; explicit `--depends-on` prerequisites do.
A dependency is satisfied only when the prerequisite task reaches the `done` / `complete`
state. Missing or incomplete prerequisite task ids remain visible as unmet dependencies in
status output and keep the dependent task in `queue`.

Dependency ids use the same safe identifier rules as task ids: letters, numbers, dots, underscores, and hyphens only. Self-dependencies, duplicate/whitespace variants, and dependency cycles are rejected at queue time.

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
  "design": "hermes",
  "spec_write": "hermes",
  "spec_verify": "hermes",
  "plan_write": "hermes",
  "plan_verify": "hermes",
  "impl": "hermes",
  "qa": "hermes",
  "validate": "hermes"
}
```

Supported agent names are `codex` and `hermes`. The default runtime now assigns Hermes to every stage so normal operation no longer depends on Codex tokens, but explicit overrides can still reassign selected stages back to Codex when credentials are available. Invalid JSON, unknown stage names, and
unknown agent names fail fast during CLI/supervisor configuration resolution, and CLI usage
surfaces them as concise Click errors instead of Python tracebacks. Reassigning
a stage only changes which supported agent runs it; the existing prompt builders and
artifact/review contracts for each stage remain unchanged.

The packaged systemd service sets both environment variables explicitly so runtime
state stays under `tools/agent-pipeline/.agents/` while stage agents still operate
against the repository workspace.

Current default: all stages run through the local `hermes` CLI. If Codex credits become available again, you can selectively restore Codex to chosen stages with `--stage-agents` or `AGENT_PIPELINE_STAGE_AGENTS`.

### Control tasks

```bash
ash-pipeline pause TASK-370
ash-pipeline resume TASK-370
ash-pipeline abort TASK-370
ash-pipeline steer TASK-370 --message "Focus on error handling"
ash-pipeline resolve-feedback TASK-370 \
  --review-artifact spec.review \
  --summary "Tighten traceability and verification commands" \
  --changes "Add exact verification command\nStrengthen README verification" \
  --success-condition "spec_verify returns VERIFIED"
ash-pipeline retry-feedback TASK-370 --to queue
```

Use `resolve-feedback` when a task is blocked by `spec.review`, `plan.review`, `qa.review`, or `validate.review` and you want to persist an explicit operator interpretation of that feedback inside the task bundle before an explicit retry action. The CLI now rejects other in-bundle files as unsupported review-artifact sources so the saved guidance is always usable by `retry-feedback`.

The command writes `.agents/<state>/<task-id>/feedback-resolution.md` and requires that the referenced review artifact already exists in the same task bundle. Review artifact references must stay inside the task bundle, and only supported retry review artifacts are accepted. Retry prompts then read both:

- the structured `feedback-resolution.md`
- the original review artifact it references

Use `retry-feedback` to explicitly release a blocked, feedback-resolved task back to `queue` or `in-progress`.

Default retry-stage inference:
- `spec.review` → `spec_write`
- `plan.review` → `plan_write`
- `qa.review` → `impl`
- `validate.review` → `impl`

`retry-feedback` archives the referenced review artifact into `retry-history/`, rewrites `feedback-resolution.md` to point at the archived review copy, clears stale downstream artifacts/logs/pid files/prompt files, resets downstream retry counters, and clears blockers before moving the task bundle.

Direct `--to in-progress` restore is rejected when task dependencies are still unmet. Use `--to queue` when you want the supervisor to apply normal dependency gating.

Single-task status now surfaces whether a feedback-resolution artifact exists and which review artifact it addresses.

### Run supervisor daemon

```bash
ash-pipeline daemon
```

### View event history

```bash
ash-pipeline events TASK-370
```

### Peek at live stage logs

Running stages now persist stream-specific logs inside the task bundle while the child process is still active:

- `.agents/<state>/<task-id>/<stage>.stdout.log`
- `.agents/<state>/<task-id>/<stage>.stderr.log`

Use the CLI to inspect the current stage by default, or target a specific stage/stream explicitly:

```bash
ash-pipeline logs TASK-370
ash-pipeline logs TASK-370 --stream stderr
ash-pipeline logs TASK-370 --stage impl --tail 50
ash-pipeline logs TASK-370 --follow
ash-pipeline logs TASK-370 --stream stderr --follow
```

`--follow` performs true tail-follow behavior against the persisted stage log file and keeps streaming newly appended chunks until the log goes idle.

If a stage has not started yet and its log file does not exist, the CLI fails with a concise user-facing error instead of a traceback.

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
├── task.md                   # present for tasks queued with --from-spec
├── feedback-resolution.md    # optional structured retry guidance
├── <stage>.stdout.log        # created when that stage launches
└── <stage>.stderr.log        # created when that stage launches
```

Queued manifests may also include a `dependencies` list of prerequisite task ids. Tasks with
unmet dependencies stay in `.agents/queue/` until every prerequisite is present in `.agents/done/`
and marked complete.

### Worktree Contract

Task bundles live under <base-dir>/<state>/<task-id>/.
Task worktrees live under <repo-root>/.worktrees/<TASK-ID>.
Provisioning behavior is create, reuse, or block.
Provisioning is fail-closed when safety checks fail.
Provisioning MUST NOT move or duplicate task-bundle artifacts.

Status surfaces now expose persisted `worktree_path` and `worktree_branch` when available. Use `ash-pipeline cleanup-worktree TASK-XXX` only for `blocked` or `done` tasks; the command refuses queued or in-progress work, reports invalid persisted worktree metadata distinctly from missing metadata, validates that persisted worktree metadata still matches the deterministic `<repo-root>/.worktrees/<TASK-ID>` assignment, clears the manifest's persisted worktree metadata after successful removal, and also clears stale metadata when removal already succeeded but `git worktree prune` fails afterward.

Supervisor/worktree provisioning now also fail closed on unsafe persisted task ids and stale git-worktree reuse records whose on-disk worktree directories are missing.

Verification from the repo root should use `PYTHONPATH=tools/agent-pipeline/src` for pytest-based checks unless the package has been installed into the active environment.

The pipeline now enforces contract-first stage artifacts without changing the external stage graph:

- design prompts require explicit Non-goals, Assumptions, and Traceability sections.
- spec and plan prompts reuse shared document-quality lenses, including Technical Critic, Pedagogical Critic, Style Critic, and a Traceability Matrix.
- impl success now requires `impl.complete`, `impl.summary.md`, and `impl.verification.md` so completion claims are backed by verification evidence.
- qa success/failure is represented by `qa.md` plus either `qa.verified` or `qa.review`.
- validate success/failure is represented by either `validated` or `validate.review`.
- review artifacts are fail-closed: `spec.review`, `plan.review`, `qa.review`, and `validate.review` block the task for human attention instead of consuming more retries.

### Agents

- **Hermes**: all stages by default (`design`, `spec_write`, `spec_verify`, `plan_write`, `plan_verify`, `impl`, `qa`, `validate`)
  - Default runtime path for the current pipeline
  - Uses the local Hermes CLI and stage-specific prompt contracts

- **Codex**: optional per-stage override only
  - Keep available through `--stage-agents` / `AGENT_PIPELINE_STAGE_AGENTS` when credits/providers are available
  - Not the default operator assumption anymore

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
PYTHONPATH=tools/agent-pipeline/src python -m pytest tools/agent-pipeline/tests -q
python -m ruff check tools/agent-pipeline/src tools/agent-pipeline/tests
bash -n tools/agent-pipeline/vila-integration.sh
python -m compileall tools/agent-pipeline/src
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
