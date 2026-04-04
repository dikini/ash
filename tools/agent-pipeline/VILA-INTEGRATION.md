# Vila Integration Guide

This document describes how to integrate the agent pipeline with **Vila**, your Hermes Discord gateway bot.

## Architecture

```
Discord (you) 
    → Vila (Hermes gateway)
    → Hermes Agent
    → vila-integration.sh
    → agent-pipeline daemon (systemd)
    → spawns Hermes-driven stage agents by default
```

The pipeline runs independently as a systemd service. Vila interacts with it via the CLI wrapper.

Current default runtime policy: every stage is handled by Hermes unless an explicit stage-agent override is configured.

## Setup

### 1. Install the Pipeline Service

```bash
cd /path/to/your/clone/tools/agent-pipeline
./install.sh
```

This creates:

- `~/.local/bin/ash-pipeline` - CLI wrapper
- `~/.config/systemd/user/agent-pipeline.service` - Systemd service

### 2. Start the Supervisor

```bash
# Start now and enable on boot
systemctl --user enable --now agent-pipeline

# Check status
systemctl --user status agent-pipeline

# View logs
journalctl --user -u agent-pipeline -f
```

### 3. Make Integration Available to Hermes

The `vila-integration.sh` script should be callable by Hermes. Add to your Hermes skills or just call directly via terminal tool.

## Usage from Discord (via Vila)

### Natural Language Examples

When talking to Vila on Discord, you can say:

**Queue a task:**

```
You: "Vila, please queue TASK-370 in the agent pipeline"
Vila: [runs vila-integration.sh queue TASK-370 vila]
Vila: "✅ TASK-370 queued successfully! It's now in the design stage."
```

Important: queue now fails closed unless the wrapper can auto-resolve exactly one matching task spec from `docs/plan/tasks/TASK-370-*.md`. If there are zero or multiple matches, Vila should report the error instead of queueing a placeholder task.

**Check status:**

```
You: "How is TASK-370 progressing?"
Vila: [runs vila-integration.sh status TASK-370]
Vila: "📊 TASK-370 is in the impl stage (attempt 2/5). 
       Last event: Stage failed, retry scheduled."
```

**Steer a task:**

```
You: "Tell TASK-370 to focus on error handling"
Vila: [runs vila-integration.sh steer TASK-370 "Focus on error handling"]
Vila: "✅ Steering message sent to TASK-370"
```

**List all active tasks:**

```
You: "What's in the pipeline?"
Vila: [runs vila-integration.sh status]
Vila: "📊 Pipeline Status:
       Queue: 2 tasks (TASK-371, TASK-372)
       In Progress: 1 task (TASK-370: impl, attempt 2/5)"
```

### Direct Commands

If Vila supports slash commands or command prefix:

```
/pipeline queue TASK-370
/pipeline status
/pipeline status TASK-370
/pipeline pause TASK-370
/pipeline resume TASK-370
/pipeline abort TASK-370
/pipeline steer TASK-370 "message"
/pipeline resolve-feedback TASK-370 spec.review "summary" "changes" "success condition"
/pipeline retry-feedback TASK-370 queue
/pipeline logs TASK-370
/pipeline events TASK-370
```

## How It Works

1. **You message Vila** on Discord
2. **Vila (Hermes)** parses your intent
3. **Hermes** runs `vila-integration.sh` via terminal tool
4. **The script** calls `ash-pipeline` CLI
5. **CLI** writes to `.agents/queue/` directory
6. **Supervisor daemon** (running via systemd) picks up the task
7. **Supervisor** spawns the configured stage agent (Hermes by default)
8. **Agents** write artifacts to `.agents/in-progress/TASK-XXX/`
9. **You** can check status anytime via Vila

For review-blocked tasks, the normal operator flow is now:
1. inspect the blocking review artifact
2. run `resolve-feedback`
3. run `retry-feedback`
4. re-check status/events/logs

## Non-Blocking Operation

The key advantage: **the pipeline supervisor runs independently**.

- You can chat with Vila normally
- Tasks progress in the background
- Vila only interacts when you ask
- No Discord polling needed in the pipeline

## Monitoring

### From terminal (on sharo)

```bash
# Watch live status
ash-pipeline status --watch

# Check specific task
ash-pipeline status --task TASK-370

# View event history
ash-pipeline events TASK-370
```

### From Discord (via Vila)

```
"Vila, check pipeline status"
"Vila, show me TASK-370 events"
```

## Troubleshooting

**Supervisor not running:**

```bash
systemctl --user start agent-pipeline
```

**Clear stuck tasks:**

```bash
# From terminal
ash-pipeline abort TASK-XXX

# Or from Discord
"Vila, abort TASK-XXX"
```

**View logs:**

```bash
journalctl --user -u agent-pipeline -n 100 --no-pager
ash-pipeline logs TASK-370
ash-pipeline logs TASK-370 --stream stderr --follow
```

## Security Notes

- The systemd service runs as your user (not root)
- It has limited filesystem access to the repository root plus the tool-local `.agents/` state directory
- Network access is unrestricted (needed for Hermes/tool-driven stage execution and optional external providers)
- Discord token is not stored in the pipeline (it's in Vila/Hermes)

## Integration with Ash Dogfooding

When Ash is mature enough, this Python orchestrator should be rewritten as an Ash workflow:

```ash
workflow PipelineSupervisor {
  receive {
    QueueTask { task_id, started_by } => {
      spawn TaskExecutor with task_id
    }
    AgentComplete { task_id, result } => {
      if result == Success {
        advance_stage(task_id)
      } else if attempts[task_id] >= 5 {
        notify_human(task_id)
      } else {
        schedule_retry(task_id)
      }
    }
  }
}
```

Until then, the Python implementation provides:

- Immediate usability
- Proper state management
- Retry logic
- Human escalation
- Discord integration
