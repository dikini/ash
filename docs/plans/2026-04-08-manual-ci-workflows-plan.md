# Manual CI Workflows Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Change all repository GitHub Actions workflows to manual dispatch only.

**Architecture:** This is a configuration-only change. Each workflow keeps its existing jobs and steps, but its `on:` block is narrowed to `workflow_dispatch` so no automatic CI run is created by `push`, `pull_request`, or `schedule`.

**Tech Stack:** GitHub Actions YAML, repository planning docs, changelog policy

---

### Task 1: Record the workflow-policy task

**Files:**
- Create: `docs/plan/tasks/TASK-441-disable-automatic-ci-workflows.md`
- Modify: `docs/plan/PLAN-INDEX.md`

**Step 1: Add the task file**

Create the task record with requirements, files, and completion criteria for switching workflow triggers to manual-only dispatch.

**Step 2: Add the task to the plan index**

Insert `TASK-441` into the active planning index as a completed workflow-policy change.

### Task 2: Convert workflow triggers

**Files:**
- Modify: `.github/workflows/ci-fast.yml`
- Modify: `.github/workflows/differential-testing.yml`
- Modify: `.github/workflows/lean-reference.yml`

**Step 1: Replace automatic triggers**

Change each workflow `on:` block to:

```yaml
on:
  workflow_dispatch:
```

**Step 2: Preserve job behavior**

Do not change workflow names, jobs, steps, caches, or artifacts.

### Task 3: Update policy docs and verify

**Files:**
- Modify: `CHANGELOG.md`

**Step 1: Add changelog entry**

Record that TASK-441 switched the workflows to manual dispatch only.

**Step 2: Validate workflow YAML**

Run a YAML parse check over the workflow files and inspect the resulting diff.
