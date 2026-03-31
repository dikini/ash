# S57-3 Observable Exit Behavior Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Update SPEC-021 so the observable exit contract for `ash run` is tied to `main` completion, descendant fate is explicitly non-observable, and task/changelog metadata record the completed spec work.

**Architecture:** This is a spec-first documentation task. The core change adds a dedicated process-exit observability subsection in SPEC-021, tightens control-authority boundary wording, and records the completed task in plan tracking and changelog files.

**Tech Stack:** Markdown specifications, planning docs, changelog policy.

---

### Task 1: Add process-exit observability rules to SPEC-021

**Files:**

- Modify: `docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md`

**Step 1: Write the failing check**

Define the missing items from `TASK-S57-3`:

- no dedicated process-exit observable subsection
- no explicit trigger tying observable exit to `main` completion
- no explicit observable exit-code mapping tied to the entry workflow

**Step 2: Run the check to verify it fails**

Run: inspect `docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md`
Expected: current Section 2 lacks a dedicated process-exit observability rule.

**Step 3: Write the minimal implementation**

Edit `docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md` to:

- add a new subsection for process-exit observables
- define the observable event as process termination with exit code
- define `main` completion as the trigger
- define exit-code sourcing for `0`, `N`, and `1`
- state that descendant fate after exit is not part of the observable contract

**Step 4: Run the check to verify it passes**

Run: inspect `docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md`
Expected: the process-exit observable contract is explicit and complete.

**Step 5: Commit**

Do not commit yet; combine with metadata updates.

### Task 2: Tighten observable-boundary language and add testability notes

**Files:**

- Modify: `docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md`

**Step 1: Write the failing check**

Verify SPEC-021 does not yet make clear enough that control-authority completion observation is runtime-internal and does not yet provide concrete testable assertions for observable exit behavior.

**Step 2: Run the check to verify it fails**

Run: inspect `docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md`
Expected: no explicit testability bullets and no direct wording that completion observation through `ControlLink` is runtime-internal only.

**Step 3: Write the minimal implementation**

Update SPEC-021 to:

- clarify that `ControlLink` completion observation is runtime-internal and cross-reference SPEC-004
- add testable assertions showing exit code comes from `main`, not descendants
- cross-reference SPEC-005 for CLI process policy

**Step 4: Run the check to verify it passes**

Run: inspect `docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md`
Expected: the internal-vs-observable boundary and testable assertions are explicit.

**Step 5: Commit**

Do not commit yet; combine with Task 3 metadata updates.

### Task 3: Update task tracking and changelog

**Files:**

- Modify: `docs/plan/tasks/TASK-S57-3-spec-021-observable-exit.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing check**

Verify S57-3 is still pending and the changelog does not yet record the completed observable-exit work.

**Step 2: Run the check to verify it fails**

Run: inspect the task file, plan index, and changelog.
Expected: S57-3 is pending and the new spec work is not recorded.

**Step 3: Write the minimal implementation**

Update the files to:

- mark S57-3 complete in the task file
- mark S57-3 complete in the Phase 57A table
- add an Unreleased changelog entry summarizing the observable-exit contract

**Step 4: Run the check to verify it passes**

Run: inspect the task file, plan index, and changelog.
Expected: tracking metadata reflects the completed task.

**Step 5: Commit**

Do not commit; leave changes unstaged for review.

### Task 4: Verify the documentation patch

**Files:**

- Verify: `docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md`
- Verify: `docs/plan/tasks/TASK-S57-3-spec-021-observable-exit.md`
- Verify: `docs/plan/PLAN-INDEX.md`
- Verify: `CHANGELOG.md`

**Step 1: Run documentation verification**

Run: `git diff --check`
Expected: no whitespace or conflict-marker issues.

**Step 2: Re-read requirements**

Check every acceptance criterion in `TASK-S57-3` against the edited files.

**Step 3: Record results**

Note any remaining gap before considering the task complete.
