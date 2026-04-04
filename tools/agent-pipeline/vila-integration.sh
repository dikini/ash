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

        # Auto-resolve task spec from docs/plan/tasks when possible
        shopt -s nullglob
        SPEC_MATCHES=("$REPO_ROOT"/docs/plan/tasks/"${TASK_ID}"-*.md)
        shopt -u nullglob
        DEPENDS_ON="${4:-}"
        QUEUE_ARGS=(queue "$TASK_ID" --notify "$NOTIFY")
        if [ -n "$DEPENDS_ON" ]; then
            IFS=',' read -r -a DEP_ARRAY <<< "$DEPENDS_ON"
            for dep in "${DEP_ARRAY[@]}"; do
                if [ -n "$dep" ]; then
                    QUEUE_ARGS+=(--depends-on "$dep")
                fi
            done
        fi
        if [ ${#SPEC_MATCHES[@]} -eq 1 ]; then
            QUEUE_ARGS+=(--from-spec "${SPEC_MATCHES[0]}")
        elif [ ${#SPEC_MATCHES[@]} -gt 1 ]; then
            echo "❌ Multiple task spec matches found for $TASK_ID; refusing to queue without an unambiguous --from-spec"
            printf '   %s\n' "${SPEC_MATCHES[@]}"
            exit 1
        else
            echo "❌ No docs/plan/tasks match found for $TASK_ID; refusing to queue without --from-spec"
            exit 1
        fi
        
        # Queue the task
        if python3 "$PIPELINE_DIR/src/ash_pipeline.py" "${QUEUE_ARGS[@]}" 2>&1; then
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

    resolve-feedback)
        # vila-integration.sh resolve-feedback TASK-370 spec.review "summary" "changes" "success condition"
        TASK_ID="$2"
        REVIEW_ARTIFACT="$3"
        SUMMARY="$4"
        CHANGES="$5"
        SUCCESS_CONDITION="$6"

        if [ -z "$TASK_ID" ] || [ -z "$REVIEW_ARTIFACT" ] || [ -z "$SUMMARY" ] || [ -z "$CHANGES" ] || [ -z "$SUCCESS_CONDITION" ]; then
            echo "Error: Task ID, review artifact, summary, changes, and success condition required"
            exit 1
        fi

        python3 "$PIPELINE_DIR/src/ash_pipeline.py" resolve-feedback "$TASK_ID" \
            --review-artifact "$REVIEW_ARTIFACT" \
            --summary "$SUMMARY" \
            --changes "$CHANGES" \
            --success-condition "$SUCCESS_CONDITION" 2>&1
        echo "✅ Feedback resolution saved for $TASK_ID"
        ;;

    retry-feedback)
        # vila-integration.sh retry-feedback TASK-370 [queue|in-progress]
        TASK_ID="$2"
        DESTINATION="${3:-queue}"

        if [ -z "$TASK_ID" ]; then
            echo "Error: Task ID required"
            exit 1
        fi

        python3 "$PIPELINE_DIR/src/ash_pipeline.py" retry-feedback "$TASK_ID" --to "$DESTINATION" 2>&1
        echo "✅ Feedback retry prepared for $TASK_ID via $DESTINATION"
        ;;

    cleanup-worktree)
        # vila-integration.sh cleanup-worktree TASK-370
        TASK_ID="$2"

        if [ -z "$TASK_ID" ]; then
            echo "Error: Task ID required"
            exit 1
        fi

        python3 "$PIPELINE_DIR/src/ash_pipeline.py" cleanup-worktree "$TASK_ID" 2>&1
        echo "✅ Worktree cleanup completed for $TASK_ID"
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

    logs)
        # vila-integration.sh logs TASK-370 [stdout|stderr] [stage] [tail] [follow]
        TASK_ID="$2"
        STREAM="${3:-stdout}"
        STAGE="${4:-}"
        TAIL_LINES="${5:-}"
        FOLLOW_FLAG="${6:-}"

        if [ -z "$TASK_ID" ]; then
            echo "Error: Task ID required"
            exit 1
        fi

        LOG_ARGS=(logs "$TASK_ID" --stream "$STREAM")
        if [ -n "$STAGE" ]; then
            LOG_ARGS+=(--stage "$STAGE")
        fi
        if [ -n "$TAIL_LINES" ]; then
            LOG_ARGS+=(--tail "$TAIL_LINES")
        fi
        if [ "$FOLLOW_FLAG" = "follow" ]; then
            LOG_ARGS+=(--follow)
        fi

        python3 "$PIPELINE_DIR/src/ash_pipeline.py" "${LOG_ARGS[@]}" 2>&1
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
        echo "  $0 queue TASK-370 [discord_username] [dep1,dep2] - Queue a task with optional dependencies"
        echo "  $0 status [TASK-370]                  - Check status"
        echo "  $0 pause TASK-370                     - Pause a task"
        echo "  $0 resume TASK-370                    - Resume a task"
        echo "  $0 abort TASK-370                     - Abort a task"
        echo "  $0 steer TASK-370 'message'           - Send steering message"
        echo "  $0 resolve-feedback TASK-370 spec.review 'summary' 'changes' 'success condition' - Save structured retry guidance"
        echo "  $0 retry-feedback TASK-370 [queue|in-progress] - Release feedback-resolved blocked work"
        echo "  $0 cleanup-worktree TASK-370          - Remove a blocked/done task worktree and clear metadata"
        echo "  $0 events TASK-370                    - Show event history"
        echo "  $0 logs TASK-370 [stdout|stderr] [stage] [tail] [follow] - Show live/persisted stage logs"

        echo ""
        echo "Examples:"
        echo "  $0 queue TASK-370 vila"
        echo "  $0 status"
        echo "  $0 steer TASK-370 'Focus on error handling'"
        ;;
esac
