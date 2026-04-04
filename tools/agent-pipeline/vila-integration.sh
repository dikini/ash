#!/bin/bash
# Integration script for Vila (Hermes) to interact with agent pipeline
# Usage: vila-integration.sh <command> [args]

set -e

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PIPELINE_DIR="$SCRIPT_DIR"
REPO_ROOT="$(cd -- "$PIPELINE_DIR/../.." && pwd)"
STATE_DIR="$PIPELINE_DIR/.agents"
export PYTHONPATH="$PIPELINE_DIR/src"
export AGENT_PIPELINE_WORKSPACE_ROOT="$REPO_ROOT"
export AGENT_PIPELINE_BASE_DIR="$STATE_DIR"

# Always run from pipeline directory so relative paths work
cd "$PIPELINE_DIR"

case "$1" in
    queue)
        # vila-integration.sh queue TASK-370 [discord_username]
        TASK_ID="$2"
        NOTIFY="${3:-discord:vila}"
        
        if [ -z "$TASK_ID" ]; then
            echo "Error: Task ID required"
            exit 1
        fi
        
        # Check if supervisor is running
        if ! systemctl --user is-active --quiet agent-pipeline 2>/dev/null; then
            echo "⚠️  Agent pipeline supervisor is not running."
            echo "   Start it with: systemctl --user start agent-pipeline"
            exit 1
        fi
        
        # Queue the task
        if python3 "$PIPELINE_DIR/src/ash_pipeline.py" queue "$TASK_ID" --notify "$NOTIFY" 2>&1; then
            echo "✅ **$TASK_ID** queued successfully!"
            echo ""
            echo "Current status:"
            python3 "$PIPELINE_DIR/src/ash_pipeline.py" status --task "$TASK_ID" 2>&1 | head -10
        else
            echo "❌ Failed to queue $TASK_ID"
        fi
        ;;
        
    status)
        # vila-integration.sh status [TASK-370]
        TASK_ID="$2"
        
        if [ -n "$TASK_ID" ]; then
            python3 "$PIPELINE_DIR/src/ash_pipeline.py" status --task "$TASK_ID" 2>&1
        else
            python3 "$PIPELINE_DIR/src/ash_pipeline.py" status 2>&1
        fi
        ;;
        
    pause|resume|abort)
        # vila-integration.sh pause TASK-370
        COMMAND="$1"
        TASK_ID="$2"
        
        if [ -z "$TASK_ID" ]; then
            echo "Error: Task ID required"
            exit 1
        fi
        
        python3 "$PIPELINE_DIR/src/ash_pipeline.py" "$COMMAND" "$TASK_ID" 2>&1
        echo "✅ $COMMAND command sent for $TASK_ID"
        ;;
        
    steer)
        # vila-integration.sh steer TASK-370 "message"
        TASK_ID="$2"
        MESSAGE="$3"
        
        if [ -z "$TASK_ID" ] || [ -z "$MESSAGE" ]; then
            echo "Error: Task ID and message required"
            exit 1
        fi
        
        python3 "$PIPELINE_DIR/src/ash_pipeline.py" steer "$TASK_ID" --message "$MESSAGE" 2>&1
        echo "✅ Steering message sent to $TASK_ID"
        ;;
        
    events)
        # vila-integration.sh events TASK-370
        TASK_ID="$2"
        
        if [ -z "$TASK_ID" ]; then
            echo "Error: Task ID required"
            exit 1
        fi
        
        python3 "$PIPELINE_DIR/src/ash_pipeline.py" events "$TASK_ID" 2>&1 | tail -20
        ;;
        
    supervisor-status)
        # Check if supervisor is running
        if systemctl --user is-active --quiet agent-pipeline 2>/dev/null; then
            echo "✅ Agent pipeline supervisor is running"
            systemctl --user status agent-pipeline --no-pager 2>&1 | head -5
        else
            echo "❌ Agent pipeline supervisor is not running"
            echo "   Start with: systemctl --user start agent-pipeline"
        fi
        ;;
        
    *)
        echo "Agent Pipeline Integration for Vila"
        echo ""
        echo "Usage:"
        echo "  $0 queue TASK-370 [discord_username]  - Queue a task"
        echo "  $0 status [TASK-370]                  - Check status"
        echo "  $0 pause TASK-370                     - Pause a task"
        echo "  $0 resume TASK-370                    - Resume a task"
        echo "  $0 abort TASK-370                     - Abort a task"
        echo "  $0 steer TASK-370 'message'           - Send steering message"
        echo "  $0 events TASK-370                    - Show event history"
        echo "  $0 supervisor-status                  - Check if supervisor is running"
        echo ""
        echo "Examples:"
        echo "  $0 queue TASK-370 vila"
        echo "  $0 status"
        echo "  $0 steer TASK-370 'Focus on error handling'"
        ;;
esac
