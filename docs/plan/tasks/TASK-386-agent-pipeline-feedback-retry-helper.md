# TASK-386: Agent Pipeline Feedback Retry Helper

## Status: ✅ Done

## Description

Add a native retry helper to `tools/agent-pipeline` so operators can explicitly release a review-blocked task back into queue or in-progress after writing `feedback-resolution.md`, without manually editing manifests or moving task bundles. The helper should preserve review provenance, clean stale downstream artifacts that would otherwise re-block the task, and reset the manifest to the correct producer stage for the referenced review artifact.

## Specification Reference

- `tools/agent-pipeline/src/agent_pipeline/cli.py`: operator-facing retry helper command and validation
- `tools/agent-pipeline/src/agent_pipeline/state.py`: blocked-task restore/move helpers
- `tools/agent-pipeline/src/agent_pipeline/agents.py`: retry prompt consumption of archived review artifacts through `feedback-resolution.md`
- `tools/agent-pipeline/vila-integration.sh`: wrapper command surface
- `tools/agent-pipeline/README.md`: operator retry-helper contract

## Dependencies

- ✅ TASK-371: Agent Pipeline Supervision Fixes
- ✅ TASK-383: Agent Pipeline Task Dependency Gating
- ✅ TASK-384: Agent Pipeline Live Stage Logs
- ✅ TASK-385: Agent Pipeline Feedback Resolution and Retry Guidance

## Requirements

### Functional Requirements

1. The CLI must provide an explicit helper command for retrying a blocked task that already has `feedback-resolution.md`.
2. The helper must reject tasks that are not currently in `blocked`.
3. The helper must reject blocked tasks that do not yet have `feedback-resolution.md`.
4. The helper must infer the restart stage from the referenced review artifact by default:
   - `spec.review` → `spec_write`
   - `plan.review` → `plan_write`
   - `qa.review` → `impl`
   - `validate.review` → `impl`
5. The helper must support an explicit destination of either `queue` or `in-progress`.
6. The helper must preserve the referenced review artifact by archiving it inside the task bundle and rewriting `feedback-resolution.md` to point at the archived path.
7. The helper must remove stale stage artifacts, logs, pid files, and prompt files from the restart stage onward so fail-closed review artifacts cannot immediately re-block the retry.
8. The helper must reset manifest blockers and retry counters from the restart stage onward while preserving upstream history.
9. Direct restore to `in-progress` must fail when task dependencies are still unmet.

### Contract Requirements

1. The feature must remain operator-explicit; no automatic queueing or resuming is allowed.
2. Retry provenance must remain auditable within the task bundle.
3. Existing `feedback-resolution.md` prompt consumption must continue to work after archiving the original review artifact.
4. Status and queue behavior for unrelated tasks must remain unchanged.

## TDD Steps

1. Add failing state tests for restoring blocked task bundles back to queue/in-progress.
2. Add failing CLI tests for successful retry-helper release, missing resolution errors, non-blocked task errors, and dependency-gated `in-progress` refusal.
3. Add failing CLI/prompt tests proving archived review artifacts remain referenced after retry preparation.
4. Implement the minimum state/CLI/wrapper/docs changes needed to satisfy the tests.
5. Run targeted pytest coverage for state, CLI, supervisor, and prompt behavior.

## Completion Checklist

- [x] Blocked task bundles can be restored natively without manual directory moves
- [x] Retry helper requires `feedback-resolution.md`
- [x] Retry helper archives the referenced review artifact and rewrites the resolution file
- [x] Retry helper clears stale downstream artifacts/logs safely
- [x] Retry helper resets stage/blocker state correctly
- [x] Direct `in-progress` restore respects dependency gating
- [x] `tools/agent-pipeline/tests/test_state.py` passes
- [x] `tools/agent-pipeline/tests/test_cli.py` passes
- [x] `tools/agent-pipeline/tests/test_agents.py` passes
- [x] `README.md` updated
- [x] `CHANGELOG.md` updated
