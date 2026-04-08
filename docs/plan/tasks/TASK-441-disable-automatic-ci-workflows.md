# TASK-441: Disable Automatic CI Workflows

## Status: ✅ Complete

## Description

Switch the repository GitHub Actions workflows from automatic execution on `push`, `pull_request`, and scheduled events to manual dispatch only. This task is limited to workflow trigger policy and does not change the jobs, steps, caches, or artifacts produced by the workflows themselves.

## Specification Reference

- Repository workflow policy
- GitHub Actions workflow trigger configuration

## Dependencies

- None

## Requirements

### Functional Requirements

1. Convert each repository workflow under `.github/workflows/` from automatic triggers to `workflow_dispatch` only.
2. Preserve the existing workflow names, jobs, and steps so maintainers can still run the workflows manually.
3. Remove scheduled execution for workflows that currently run on a timer.
4. Update `CHANGELOG.md` and planning surfaces to record the trigger-policy change.

### Non-Functional Requirements

1. Keep the change minimal and limited to trigger policy.
2. Do not alter job logic, cache keys, artifact names, or runtime commands.
3. Keep the YAML valid for GitHub Actions.

## TDD Evidence

### Red

Before this task:
- `ci-fast.yml` ran automatically on `push` and `pull_request`;
- `differential-testing.yml` ran automatically on `push`, `pull_request`, and a daily schedule;
- `lean-reference.yml` ran automatically on `push` and `pull_request`.

### Green

This task is complete when:
- each workflow uses `workflow_dispatch` as its only trigger;
- no workflow auto-runs on `push`, `pull_request`, or `schedule`;
- workflow YAML remains valid.

## Files

- Modify: `.github/workflows/ci-fast.yml`
- Modify: `.github/workflows/differential-testing.yml`
- Modify: `.github/workflows/lean-reference.yml`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

## Completion Checklist

- [x] Added a task record for the workflow-trigger policy change
- [x] Converted all repository workflows to manual dispatch only
- [x] Removed automatic push, pull request, and scheduled triggers
- [x] Updated planning and changelog surfaces
- [x] YAML validation evidence captured

## Notes

This is an operations/workflow-policy change only. It intentionally leaves manual execution available through the GitHub Actions UI or API.
