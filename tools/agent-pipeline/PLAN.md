# Agent Pipeline Orchestrator Plan

## Goal

Build a Python-based task orchestrator for the multi-agent pipeline (design → spec → verify → plan → verify → impl → qa → validate) that manages state via files, spawns agents (Hermes by default, with optional Codex overrides), and supports local supervision.

## Location

`tools/agent-pipeline/` - separate subtree from main Ash project

## Core Components

### 1. State Machine (`state.py`)

**Responsibilities:**
- Define task manifest schema (JSON)
- Track stage transitions
- Manage attempt counters (max 5 per stage)
- Handle blocking/escalation

**Task Manifest Schema:**
```json
{
  "task_id": "TASK-370",
  "current_stage": "design",
  "status": "in_progress|blocked|complete",
  "attempts": {"design": 1, "spec_write": 0, ...},
  "max_attempts": 5,
  "artifacts": {"design": "path/to/design.md"},
  "blockers": [],
  "started_by": "cli|discord:user|cron",
  "created_at": "ISO8601",
  "updated_at": "ISO8601"
}
```

**Stages:**
1. `design` (hermes)
2. `spec_write` (hermes)
3. `spec_verify` (hermes)
4. `plan_write` (hermes)
5. `plan_verify` (hermes)
6. `impl` (hermes)
7. `qa` (hermes)
8. `validate` (hermes)

### 2. Agent Spawner (`agents.py`)

**Responsibilities:**
- Spawn Hermes stage sessions by default
- Allow optional Codex overrides for selected stages when explicitly configured
- Manage agent lifecycle (start, monitor, timeout)
- Capture stdout/stderr to event log

**Codex Integration:**
```python
subprocess.run([
    "codex", "--yolo",
    "--prompt", crafted_prompt,
    "--cwd", task_dir
], timeout=3600)
```

**Hermes Integration:**
- For now: generate shell command that runs hermes with task context
- Future: direct API integration

### 3. Supervisor (`supervisor.py`)

**Responsibilities:**
- Poll `queue/` directory for new tasks
- Check `control/` for steering commands (pause, abort, steer)
- Monitor active agent processes
- Handle stage transitions on completion
- Write status to `status/dashboard.json`

**Control Commands:**
- `pause`: Stop after current stage
- `abort`: Terminate immediately
- `steer`: Add guidance message to task context

### 4. CLI Interface (`cli.py`)

**Commands:**
- `ash-pipeline queue <task_id> [--notify <target>]` - Add task to queue
- `ash-pipeline status [--task <id>] [--watch]` - Show status
- `ash-pipeline pause <task_id>` - Pause task
- `ash-pipeline abort <task_id>` - Abort task
- `ash-pipeline steer <task_id> --message "..."` - Add guidance
- `ash-pipeline daemon` - Run supervisor loop

### 5. Event Logger (`events.py`)

**Responsibilities:**
- Append-only event log (JSONL format)
- Events: task_created, agent_spawned, stage_complete, stage_failed, retry_scheduled, blocked, completed

## Directory Structure

```
tools/agent-pipeline/
├── README.md
├── PLAN.md
├── pyproject.toml
├── src/
│   ├── agent_pipeline/
│   │   ├── __init__.py
│   │   ├── state.py      # Task manifest management
│   │   ├── agents.py     # Agent spawning
│   │   ├── supervisor.py # Main supervision loop
│   │   ├── events.py     # Event logging
│   │   └── cli.py        # Command interface
│   └── ash_pipeline.py   # Entry point
├── tests/
│   ├── test_state.py
│   ├── test_agents.py
│   ├── test_supervisor.py
│   └── test_cli.py
└── .agents/              # Runtime data (created at runtime)
    ├── queue/
    ├── in-progress/
    ├── done/
    ├── blocked/
    ├── control/
    ├── events/
    └── status/
```

## Test Strategy

### Unit Tests (pytest)

1. **test_state.py** - Task manifest CRUD, stage transitions, attempt tracking
2. **test_agents.py** - Mock agent spawning, timeout handling
3. **test_supervisor.py** - Queue polling, control command processing
4. **test_cli.py** - Command parsing, integration with core modules

### Integration Test

- Create a mock task that runs through all stages with mock agents
- Verify state transitions
- Verify retry logic
- Verify blocking after max attempts

## Implementation Order

1. **State management** - Core data structures and file I/O
2. **Event logging** - Simple append-only logger
3. **Agent spawning** - Codex wrapper with mocks
4. **Supervisor loop** - Polling and state transitions
5. **CLI** - Command interface
6. **Integration test** - End-to-end validation

## Files Likely to Change

- `src/agent_pipeline/state.py` - Core state machine
- `src/agent_pipeline/agents.py` - Agent integration
- `src/agent_pipeline/supervisor.py` - Orchestration logic
- `src/agent_pipeline/cli.py` - User interface
- Tests for each module

## Risks

1. **Codex --yolo reliability** - May fail or hang; need timeouts and retries
2. **Hermes subagent delegation** - Not directly available in Python; may need subprocess approach
3. **File locking** - Concurrent access to state files; use atomic writes
4. **State corruption** - Power loss during write; use write-to-temp-then-rename

## Open Questions

1. Should we use SQLite instead of JSON files for state? (start with files, migrate if needed)
2. How to handle agent output streaming? (capture to file, tail for progress)
3. Should supervisor be a daemon or one-shot per task? (daemon with polling)

## Verification Steps

1. Create test task via CLI
2. Verify task appears in queue/
3. Run supervisor, verify task moves through stages
4. Verify artifacts created in done/
5. Verify events log has complete history
6. Test pause/abort/steer commands
7. Test max retry blocking
