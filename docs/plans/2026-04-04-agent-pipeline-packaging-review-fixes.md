# Agent Pipeline Packaging Review Fixes Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix the agent-pipeline packaged deployment so its service sandbox, runtime path configuration, queue validation behavior, and helper scripts match the reviewed requirements.

**Architecture:** Keep the existing CLI discovery behavior for normal usage, but make packaged mode explicit through environment variables supplied by the systemd unit and helper scripts. Treat `queue --from-spec` as an atomic validation-then-create operation, and remove clone-location assumptions by deriving paths from script locations.

**Tech Stack:** Python 3.10+, `click`, `pytest`, POSIX shell, systemd user services

---

### Task 1: Add regression coverage for non-mutating `queue --from-spec`

**Files:**

- Modify: `/home/dikini/Projects/ash/tools/agent-pipeline/tests/test_cli.py`
- Modify: `/home/dikini/Projects/ash/tools/agent-pipeline/src/agent_pipeline/cli.py`

**Step 1: Write the failing test**

Add a CLI test asserting that queueing with a missing `--from-spec` path:

- exits non-zero
- leaves no `queue/<task_id>/` bundle behind
- does not poison later retries with a duplicate task id

**Step 2: Run the focused test to verify it fails**

Run: `/home/dikini/Projects/ash/.venv/bin/python -m pytest tools/agent-pipeline/tests/test_cli.py -q`
Expected: FAIL because the current implementation creates the manifest before validating the source file.

**Step 3: Write minimal implementation**

Update `queue()` so `from_spec` is validated before `create_task()` and task bundle creation.

**Step 4: Re-run the focused test**

Run: `/home/dikini/Projects/ash/.venv/bin/python -m pytest tools/agent-pipeline/tests/test_cli.py -q`
Expected: PASS

### Task 2: Make packaged runtime paths explicit and portable

**Files:**

- Modify: `/home/dikini/Projects/ash/tools/agent-pipeline/agent-pipeline.service`
- Modify: `/home/dikini/Projects/ash/tools/agent-pipeline/install.sh`
- Modify: `/home/dikini/Projects/ash/tools/agent-pipeline/vila-integration.sh`
- Modify: `/home/dikini/Projects/ash/tools/agent-pipeline/README.md`

**Step 1: Add failing coverage or direct verification hooks**

Add lightweight tests where practical, and otherwise structure the scripts so their derived paths can be checked deterministically.

**Step 2: Update the systemd unit template**

Make the service consume explicit repo-relative environment values for:

- `AGENT_PIPELINE_WORKSPACE_ROOT`
- `AGENT_PIPELINE_BASE_DIR`
- `PYTHONPATH`

Set the packaged working directory consistently to the tool directory.

**Step 3: Update installer and Vila helper path resolution**

Derive the repo root and tool root from each script’s own location instead of `$HOME/Projects/ash`.

**Step 4: Update packaged-mode documentation**

Document the explicit environment-driven packaged deployment model.

### Task 3: Align the service sandbox with real write behavior

**Files:**

- Modify: `/home/dikini/Projects/ash/tools/agent-pipeline/agent-pipeline.service`
- Review: `/home/dikini/Projects/ash/tools/agent-pipeline/src/agent_pipeline/agents.py`

**Step 1: Verify current write targets**

Confirm that:

- task state writes occur under `tools/agent-pipeline/.agents`
- stage agents run with the repo root as workspace and may edit repository files during `impl`

**Step 2: Apply the minimal unit change**

Relax the service sandbox so it allows repo-root writes needed for implementation stages while preserving explicit writable state under the tool-local `.agents` directory.

**Step 3: Re-check internal consistency**

Ensure the unit’s write permissions match the configured workspace root and base dir.

### Task 4: Final verification and closeout

**Files:**

- Modify: `/home/dikini/Projects/ash/CHANGELOG.md`
- Modify: `/home/dikini/Projects/ash/docs/plan/tasks/TASK-372-agent-pipeline-packaging-review-fixes.md`

**Step 1: Run lint and tests**

Run:

- `cd /home/dikini/Projects/ash/tools/agent-pipeline && /home/dikini/Projects/ash/.venv/bin/python -m ruff check src tests`
- `cd /home/dikini/Projects/ash/tools/agent-pipeline && /home/dikini/Projects/ash/.venv/bin/python -m pytest tests -q`

Expected: all checks pass.

**Step 2: Update changelog**

Add an Unreleased entry summarizing the packaged deployment and queue-validation fixes.

**Step 3: Mark the task file complete**

Check off the completion items in `TASK-372` after verification passes.
