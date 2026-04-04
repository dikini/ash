# Agent Pipeline Worktree Isolation Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Add per-task git worktree isolation to `tools/agent-pipeline` so each queued task executes against its own repository workspace while the existing `.agents/` task bundle state model remains intact.

**Architecture:** Keep task manifests, prompts, events, and review artifacts under `tools/agent-pipeline/.agents/<state>/<task-id>/`, but provision a dedicated git worktree per task under the repository’s `.worktrees/` directory. Persist worktree metadata in the task manifest, have the supervisor provision/reuse the worktree before first launch, and run Codex/Hermes stages from that worktree root while prompts explicitly distinguish repository workspace paths from task-bundle artifact paths.

**Tech Stack:** Python 3.10+, `click`, `pytest`, `git worktree`, filesystem state under `tools/agent-pipeline/.agents/`, worktrees under `<repo-root>/.worktrees/`

---

## Scope and decisions

- One worktree per task id.
- Default worktree path: `<repo-root>/.worktrees/<TASK-ID>`.
- Default branch name: `agent-pipeline/<TASK-ID>`.
- Worktrees are **kept by default** after success, block, or abort for inspection.
- Cleanup is explicit via a future CLI command rather than automatic.
- Task bundle artifacts stay under `.agents/...`; repository edits happen in the worktree.
- Relative repository references in `task.md` are resolved from the worktree root, not the task bundle directory.
- Because the current pipeline starts queued tasks concurrently, dependency gating must land first so the Phase 59 task bundle can be safely queued with explicit prerequisites.
- Live stage stdout/stderr observability is a separate prerequisite-quality improvement; adding per-stage logs makes dependency/worktree debugging much easier during rollout.

## Preconditions already true in this repo

- `.worktrees/` already exists.
- `.worktrees/` is ignored by git.
- The pipeline already separates task-bundle state from repository working directory configuration.

---

### Task 1: Persist worktree metadata and safe provisioning rules

**Objective:** Add manifest/state support for per-task worktree metadata and a reusable provisioning helper that follows the repository’s worktree conventions safely.

**Files:**
- Create: `tools/agent-pipeline/src/agent_pipeline/worktrees.py`
- Modify: `tools/agent-pipeline/src/agent_pipeline/state.py`
- Modify: `tools/agent-pipeline/src/agent_pipeline/supervisor.py`
- Test: `tools/agent-pipeline/tests/test_state.py`
- Test: `tools/agent-pipeline/tests/test_supervisor.py`
- Create: `tools/agent-pipeline/tests/test_worktrees.py`

**Step 1: Write failing tests**

Add tests that assert:
- `TaskManifest` round-trips new fields for `worktree_path`, `branch_name`, and provisioning status.
- `WorktreeManager` derives `.worktrees/<task-id>` and `agent-pipeline/<task-id>` deterministically.
- project-local worktree provisioning fails if `.worktrees/` is not ignored.
- provisioning creates or reuses the expected worktree path without mutating task artifacts.
- the supervisor provisions worktree metadata before launching the first stage for a task.

**Step 2: Run targeted tests to verify failure**

Run:
- `python -m pytest tools/agent-pipeline/tests/test_state.py -q`
- `python -m pytest tools/agent-pipeline/tests/test_worktrees.py -q`
- `python -m pytest tools/agent-pipeline/tests/test_supervisor.py -q`

Expected: FAIL because manifests do not carry worktree metadata and no worktree manager/provisioning flow exists.

**Step 3: Write minimal implementation**

Implement:
- manifest fields for worktree metadata and safe defaults;
- `WorktreeManager` helpers for path/branch derivation, ignore verification, create/reuse behavior, and explicit error messages;
- supervisor hook that ensures the task’s worktree exists before agent launch.

**Step 4: Run tests to verify pass**

Run the same targeted commands.
Expected: PASS.

**Step 5: Commit**

```bash
git add tools/agent-pipeline/src/agent_pipeline/state.py \
        tools/agent-pipeline/src/agent_pipeline/worktrees.py \
        tools/agent-pipeline/src/agent_pipeline/supervisor.py \
        tools/agent-pipeline/tests/test_state.py \
        tools/agent-pipeline/tests/test_supervisor.py \
        tools/agent-pipeline/tests/test_worktrees.py
git commit -m "feat: provision per-task agent pipeline worktrees"
```

---

### Task 2: Run stage agents from task worktrees with explicit dual-root prompts

**Objective:** Make Codex/Hermes stages execute from the task’s worktree while still reading/writing stage artifacts in the `.agents/` task bundle.

**Files:**
- Modify: `tools/agent-pipeline/src/agent_pipeline/agents.py`
- Modify: `tools/agent-pipeline/src/agent_pipeline/prompt_contracts.py`
- Test: `tools/agent-pipeline/tests/test_agents.py`

**Step 1: Write failing tests**

Add tests that assert:
- Codex launches with `cwd=<task worktree>` and `codex exec -C <task worktree>`.
- Hermes stand-in launches from the same task worktree.
- prompts mention both the repository workspace path and the task bundle path.
- QA prompts review code in the task worktree rather than the shared repository root.
- placeholder task-bundle artifacts remain under `.agents/in-progress/<task-id>/`.

**Step 2: Run targeted tests to verify failure**

Run:
- `python -m pytest tools/agent-pipeline/tests/test_agents.py -q`

Expected: FAIL because agents still use the shared workspace root and prompts only imply a single root.

**Step 3: Write minimal implementation**

Update the spawner/prompt builders so:
- stage cwd comes from manifest worktree metadata;
- Codex/Hermes `-C`/cwd both target the worktree root;
- prompts explicitly state:
  - repository workspace root for code and repo-relative references;
  - task bundle directory for generated artifacts.

**Step 4: Run tests to verify pass**

Run:
- `python -m pytest tools/agent-pipeline/tests/test_agents.py -q`

Expected: PASS.

**Step 5: Commit**

```bash
git add tools/agent-pipeline/src/agent_pipeline/agents.py \
        tools/agent-pipeline/src/agent_pipeline/prompt_contracts.py \
        tools/agent-pipeline/tests/test_agents.py
git commit -m "feat: execute agent pipeline stages from task worktrees"
```

---

### Task 3: Add worktree-aware CLI/status/cleanup surfaces

**Objective:** Expose worktree metadata through CLI/status, add explicit cleanup controls, and document the operational contract.

**Files:**
- Modify: `tools/agent-pipeline/src/agent_pipeline/cli.py`
- Modify: `tools/agent-pipeline/vila-integration.sh`
- Modify: `tools/agent-pipeline/README.md`
- Test: `tools/agent-pipeline/tests/test_cli.py`

**Step 1: Write failing tests**

Add tests that assert:
- single-task and aggregate status output include worktree path/branch when available;
- JSON status includes worktree metadata fields;
- `cleanup-worktree TASK-XXX` refuses to remove worktrees for in-progress tasks;
- `cleanup-worktree` succeeds for done/blocked tasks and updates persisted state;
- CLI errors for missing/nonexistent worktrees are concise and user-facing.

**Step 2: Run targeted tests to verify failure**

Run:
- `python -m pytest tools/agent-pipeline/tests/test_cli.py -q`

Expected: FAIL because CLI surfaces do not expose worktree metadata or cleanup behavior.

**Step 3: Write minimal implementation**

Implement:
- worktree metadata in text/json status output;
- `cleanup-worktree` CLI command with safety checks;
- wrapper support for the new command if needed;
- README updates covering path conventions, cleanup expectations, and dual-root execution behavior.

**Step 4: Run tests to verify pass**

Run:
- `python -m pytest tools/agent-pipeline/tests/test_cli.py -q`

Expected: PASS.

**Step 5: Commit**

```bash
git add tools/agent-pipeline/src/agent_pipeline/cli.py \
        tools/agent-pipeline/vila-integration.sh \
        tools/agent-pipeline/README.md \
        tools/agent-pipeline/tests/test_cli.py
git commit -m "feat: add worktree-aware agent pipeline cli surfaces"
```

---

### Task 4: End-to-end recovery, verification, and closeout

**Objective:** Prove the full worktree flow survives restarts and common edge cases, then document and close out the phase.

**Files:**
- Modify: `tools/agent-pipeline/tests/test_supervisor.py`
- Modify: `tools/agent-pipeline/tests/test_cli.py`
- Modify: `tools/agent-pipeline/tests/test_agents.py`
- Modify: `CHANGELOG.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/plan/tasks/TASK-382-phase-59-closeout.md`

**Step 1: Add failing integration/recovery tests**

Add tests that assert:
- supervisor restart reuses persisted worktree metadata instead of creating duplicates;
- existing matching worktree directories are reused cleanly;
- cleanup only removes finished task worktrees;
- status remains correct across queue → in-progress → blocked/done with worktree metadata intact.

**Step 2: Run targeted tests to verify failure**

Run:
- `python -m pytest tools/agent-pipeline/tests/test_supervisor.py -q`
- `python -m pytest tools/agent-pipeline/tests/test_cli.py -q`
- `python -m pytest tools/agent-pipeline/tests/test_agents.py -q`

Expected: FAIL until recovery/cleanup/status edge cases are implemented.

**Step 3: Implement minimal fixes**

Patch the smallest missing pieces for restart safety, reuse semantics, and cleanup state updates.

**Step 4: Run full verification**

Run:
- `PYTHONPATH=tools/agent-pipeline/src python -m pytest tools/agent-pipeline/tests -q`
- `python -m ruff check tools/agent-pipeline/src tools/agent-pipeline/tests`
- `bash -n tools/agent-pipeline/vila-integration.sh`
- `python -m compileall tools/agent-pipeline/src`

Expected: all tests pass, no lint errors, shell syntax clean.

**Step 5: Update docs and closeout artifacts**

Update:
- `CHANGELOG.md`
- `docs/plan/PLAN-INDEX.md`
- `docs/plan/tasks/TASK-382-phase-59-closeout.md`

**Step 6: Commit**

```bash
git add tools/agent-pipeline/tests \
        CHANGELOG.md \
        docs/plan/PLAN-INDEX.md \
        docs/plan/tasks/TASK-382-phase-59-closeout.md
git commit -m "feat: verify and close out agent pipeline worktree isolation"
```

---

## Execution order

1. TASK-383 — task dependency gating
2. TASK-384 — live stdout/stderr capture and operator log peeking
3. TASK-378 — worktree metadata + provisioning
4. TASK-379 — worktree execution roots + prompts
5. TASK-380 — CLI/status/cleanup surfaces
6. TASK-381 — end-to-end verification support
7. TASK-382 — phase closeout

## Verification gate for the bundle

Before marking the bundle complete, verify:
- task manifests persist worktree metadata correctly;
- no stage runs from the shared repo root when worktree metadata is present;
- prompts clearly distinguish repo workspace from task bundle paths;
- cleanup is explicit and safe;
- worktree state survives supervisor restarts;
- `tools/agent-pipeline/tests` passes.
