# Agent Pipeline Supervision Fixes Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix the agent-pipeline supervisor so task execution is non-blocking, state transitions move full task bundles, status lookup includes completed tasks, and agent execution is configurable instead of hard-coded.

**Architecture:** Keep the file-based orchestrator, but normalize task storage around per-task directories with `manifest.json` plus artifacts, split agent launch from result collection, and let the supervisor poll live processes asynchronously. Centralize task lookup/config so CLI and supervisor observe the same task state and runtime paths.

**Tech Stack:** Python 3.10+, `click`, `pytest`, filesystem state under `tools/agent-pipeline/.agents/`

---

### Task 1: Normalize task bundle layout

**Files:**

- Modify: `tools/agent-pipeline/src/agent_pipeline/state.py`
- Test: `tools/agent-pipeline/tests/test_state.py`

**Step 1: Write the failing tests**

Add tests that assert:

- queue creation writes `queue/<task_id>/manifest.json`
- `queue --from-spec` compatible layout places `task.md` alongside the manifest
- `move_to_in_progress()` moves the whole task directory
- `move_to_done()` preserves the task bundle under `done/<task_id>/`
- completed and blocked tasks are discoverable with a shared lookup helper

**Step 2: Run targeted tests to verify they fail**

Run: `python -m pytest tools/agent-pipeline/tests/test_state.py -q`
Expected: FAIL because the current code still stores flat JSON files and lacks shared lookup helpers.

**Step 3: Write minimal implementation**

Refactor `StateManager` to:

- store tasks as `<state>/<task_id>/manifest.json`
- add helpers for task directories, manifest paths, listing, and cross-state lookup
- move directories rather than single JSON files
- preserve compatibility for existing flat manifests if feasible

**Step 4: Run tests to verify they pass**

Run: `python -m pytest tools/agent-pipeline/tests/test_state.py -q`
Expected: PASS

**Step 5: Commit**

`git add tools/agent-pipeline/src/agent_pipeline/state.py tools/agent-pipeline/tests/test_state.py`
`git commit -m "fix: normalize agent pipeline task layout"`

### Task 2: Make agent execution non-blocking

**Files:**

- Modify: `tools/agent-pipeline/src/agent_pipeline/agents.py`
- Test: `tools/agent-pipeline/tests/test_agents.py`

**Step 1: Write the failing tests**

Add tests that assert:

- spawning returns immediately with a live process record
- result collection is performed by a separate poll/reap step
- pid files are created and cleared correctly
- configured working directory/executable path is used instead of a hard-coded repo path

**Step 2: Run targeted tests to verify they fail**

Run: `python -m pytest tools/agent-pipeline/tests/test_agents.py -q`
Expected: FAIL because the current implementation blocks in `communicate()` and hard-codes `/home/dikini/Projects/ash`.

**Step 3: Write minimal implementation**

Refactor `AgentSpawner` to:

- expose a non-blocking launch API
- track process metadata needed for later polling/reaping
- provide a poll/collect API that returns `SpawnResult` only after the process exits
- derive workspace root/executables from configuration

**Step 4: Run tests to verify they pass**

Run: `python -m pytest tools/agent-pipeline/tests/test_agents.py -q`
Expected: PASS

**Step 5: Commit**

`git add tools/agent-pipeline/src/agent_pipeline/agents.py tools/agent-pipeline/tests/test_agents.py`
`git commit -m "fix: decouple agent launch from result collection"`

### Task 3: Repair supervisor process tracking and task control

**Files:**

- Modify: `tools/agent-pipeline/src/agent_pipeline/supervisor.py`
- Test: `tools/agent-pipeline/tests/test_supervisor.py`

**Step 1: Write the failing tests**

Add tests that assert:

- `run_once()` no longer crashes on missing `check_active_tasks()`
- control commands are processed before new launches
- `start_task_stage()` registers active tasks/processes
- `check_active_tasks()` advances, retries, blocks, and completes tasks based on polled results
- abort terminates real active work and moves the task out of `in-progress`

**Step 2: Run targeted tests to verify they fail**

Run: `python -m pytest tools/agent-pipeline/tests/test_supervisor.py -q`
Expected: FAIL because `check_active_tasks()` is missing and active process tracking is incomplete.

**Step 3: Write minimal implementation**

Update `Supervisor` to:

- process control commands before starting new work
- register active manifests + processes
- implement `check_active_tasks()` using the spawner poll/reap API
- make pause/abort/steer semantics operate on persisted task state when needed

**Step 4: Run tests to verify they pass**

Run: `python -m pytest tools/agent-pipeline/tests/test_supervisor.py -q`
Expected: PASS

**Step 5: Commit**

`git add tools/agent-pipeline/src/agent_pipeline/supervisor.py tools/agent-pipeline/tests/test_supervisor.py`
`git commit -m "fix: restore asynchronous supervisor control"`

### Task 4: Unify CLI/status/config behavior

**Files:**

- Modify: `tools/agent-pipeline/src/agent_pipeline/cli.py`
- Create: `tools/agent-pipeline/tests/test_cli.py`
- Modify: `tools/agent-pipeline/README.md`

**Step 1: Write the failing tests**

Add tests that assert:

- queue with `--from-spec` creates a bundle compatible with later state moves
- status finds tasks in queue, in-progress, blocked, and done
- status output includes completed tasks in aggregate output
- CLI can be pointed at the correct base/workspace configuration

**Step 2: Run targeted tests to verify they fail**

Run: `python -m pytest tools/agent-pipeline/tests/test_cli.py -q`
Expected: FAIL because completed tasks are currently invisible and queue layout is inconsistent.

**Step 3: Write minimal implementation**

Refactor CLI helpers to use centralized state/config helpers and update README usage notes to describe task bundle layout and configurable workspace resolution.

**Step 4: Run tests to verify they pass**

Run: `python -m pytest tools/agent-pipeline/tests/test_cli.py -q`
Expected: PASS

**Step 5: Commit**

`git add tools/agent-pipeline/src/agent_pipeline/cli.py tools/agent-pipeline/tests/test_cli.py tools/agent-pipeline/README.md`
`git commit -m "fix: align agent pipeline cli status and queue behavior"`

### Task 5: Final verification, changelog, and closeout

**Files:**

- Modify: `CHANGELOG.md`
- Review: `tools/agent-pipeline/src/agent_pipeline/*.py`
- Review: `tools/agent-pipeline/tests/*.py`

**Step 1: Add changelog entry**

Record the supervision/layout/status fixes under `## [Unreleased]`.

**Step 2: Run focused verification**

Run:

- `python -m pytest tools/agent-pipeline/tests -q`
- `python -m ruff check tools/agent-pipeline/src tools/agent-pipeline/tests`

Expected: all tests pass, no lint errors.

**Step 3: Review against requirements**

Confirm:

- no missing `check_active_tasks()` crash
- active abort works
- completed tasks are visible
- no hard-coded Ash workspace path remains

**Step 4: Commit**

`git add CHANGELOG.md tools/agent-pipeline`
`git commit -m "fix: harden agent pipeline supervision flow"`
