# S57-1 Control-Link Completion Payload Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Update SPEC-004 so spawned workflow completion yields a runtime-internal control-link payload with a distinct terminal-control error contract, then record the change in task tracking and changelog files.

**Architecture:** This is a spec-first documentation change centered in SPEC-004. The implementation adds semantic-domain definitions, a runtime-internal control-authority completion subsection, and a supervisor observation rule, then propagates the change into task/changelog metadata without expanding user-visible syntax.

**Tech Stack:** Markdown specifications, project planning docs, changelog policy.

---

### Task 1: Update SPEC-004 semantic domains and control-authority contract

**Files:**

- Modify: `docs/spec/SPEC-004-SEMANTICS.md`

**Step 1: Write the failing check**

Define a checklist against `TASK-S57-1` acceptance criteria:

- missing `CompletionPayload`
- missing `EffectTrace`
- missing terminal-control error variant
- missing runtime-internal control-authority completion semantics

**Step 2: Run the check to verify the gap exists**

Run: inspect `docs/spec/SPEC-004-SEMANTICS.md`
Expected: current spec mentions control failure boundaries but does not define completion payload semantics.

**Step 3: Write the minimal implementation**

Edit `docs/spec/SPEC-004-SEMANTICS.md` to:

- add `EffectTrace` and `CompletionPayload` to semantic domains
- extend `Error` with terminal-control failure
- add a section defining control authority and terminal completion payload semantics
- state explicitly that completion observation is runtime/supervisor-internal, not surface syntax

**Step 4: Run the check to verify the update**

Run: inspect `docs/spec/SPEC-004-SEMANTICS.md`
Expected: all required definitions and caveats are present.

**Step 5: Commit**

Stage only the SPEC-004 update plus matching changelog/task metadata once later tasks are done.

### Task 2: Add supervisor observation semantics and cross-spec references

**Files:**

- Modify: `docs/spec/SPEC-004-SEMANTICS.md`

**Step 1: Write the failing check**

Verify SPEC-004 does not yet show a supervisor receiving terminal payload and projecting `payload.result`.

**Step 2: Run the check to verify it fails**

Run: inspect `docs/spec/SPEC-004-SEMANTICS.md`
Expected: no normative supervisor completion rule exists yet.

**Step 3: Write the minimal implementation**

Add normative prose/rule text showing:

- `spawn` creates `Instance × ControlAuthority`
- terminal completion seals `CompletionPayload`
- supervisor observes payload through control authority
- supervisor extracts `payload.result`
- cross-reference SPEC-019 for completion/obligation state and SPEC-021 for observable boundaries

**Step 4: Run the check to verify it passes**

Run: inspect `docs/spec/SPEC-004-SEMANTICS.md`
Expected: supervisor observation rule and cross-references are present.

**Step 5: Commit**

Stage with the final metadata updates only after Task 3.

### Task 3: Update task tracking and changelog

**Files:**

- Modify: `docs/plan/tasks/TASK-S57-1-spec-004-control-link-completion.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing check**

Verify the task remains pending and the changelog only records planning, not completion.

**Step 2: Run the check to verify the gap exists**

Run: inspect the task file, plan index, and changelog.
Expected: no completion markers for S57-1 docs work.

**Step 3: Write the minimal implementation**

Update:

- task status to completed and mark acceptance checklist items done
- plan index status for `TASK-S57-1`
- unreleased changelog entry describing the SPEC-004 completion-payload semantics update

**Step 4: Run the check to verify it passes**

Run: inspect the updated metadata files.
Expected: status and changelog reflect the completed task.

**Step 5: Commit**

Prepare a conventional commit message such as: `docs(spec): define control-link completion payload semantics`

### Task 4: Verify the documentation change set

**Files:**

- Verify: `docs/spec/SPEC-004-SEMANTICS.md`
- Verify: `docs/plan/tasks/TASK-S57-1-spec-004-control-link-completion.md`
- Verify: `docs/plan/PLAN-INDEX.md`
- Verify: `CHANGELOG.md`

**Step 1: Run repo verification**

Run: `cargo test --workspace --quiet`
Expected: baseline remains green.

**Step 2: Run targeted diff/status verification**

Run: `git status --short` and inspect the changed files.
Expected: only the intended docs/plans/changelog files changed.

**Step 3: Review against acceptance criteria**

Manually confirm each `TASK-S57-1` acceptance item is satisfied by the spec text.

**Step 4: Commit**

Do not commit without explicit user instruction.
