# S57-2 CLI Exit-Immediately Policy Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Update SPEC-005 so `ash run` exits when `main` completes, document abstract exit-code derivation from `main`, and record the change in task tracking and changelog files.

**Architecture:** This is a spec-first documentation task centered on the `ash run` command contract. The implementation tightens command syntax, adds a process-lifecycle subsection, and adds `ash run`-specific exit-code rules while explicitly leaving descendant fate outside the CLI contract.

**Tech Stack:** Markdown specifications, planning docs, changelog policy.

---

### Task 1: Update SPEC-005 `ash run` syntax and process lifecycle

**Files:**

- Modify: `docs/spec/SPEC-005-CLI.md`

**Step 1: Write the failing check**

Define the missing items from `TASK-S57-2`:

- no explicit exit-immediately policy
- no explicit `ash run ... [-- <args>...]` syntax
- no explicit statement that descendants do not extend process lifetime

**Step 2: Run the check to verify it fails**

Run: inspect `docs/spec/SPEC-005-CLI.md`
Expected: current `ash run` section lacks a process-lifecycle rule and argument-separator clarification.

**Step 3: Write the minimal implementation**

Edit `docs/spec/SPEC-005-CLI.md` to:

- update the `ash run` synopsis to include `[-- <args>...]`
- add a process-exit-policy subsection
- state that `ash run` executes `main` and exits immediately on `main` completion
- state that descendants do not extend process lifetime
- note that descendant fate after exit is implementation-defined

**Step 4: Run the check to verify it passes**

Run: inspect `docs/spec/SPEC-005-CLI.md`
Expected: all required lifecycle and syntax points are present.

**Step 5: Commit**

Do not commit yet; combine with later metadata updates.

### Task 2: Update `ash run` exit-code contract and cross-references

**Files:**

- Modify: `docs/spec/SPEC-005-CLI.md`

**Step 1: Write the failing check**

Verify SPEC-005 does not yet define `ash run` exit-code derivation from `main`.

**Step 2: Run the check to verify it fails**

Run: inspect `docs/spec/SPEC-005-CLI.md`
Expected: no `ash run`-specific exit-code mapping tied to `main` completion.

**Step 3: Write the minimal implementation**

Add `ash run` exit-code bullets covering:

- `0` for successful `main` completion with obligations discharged
- `N` for runtime error exit code propagated from `main`
- `1` for bootstrap or verification failure
- descendant failures do not affect process exit code after `main` completion
- cross-reference SPEC-004 and SPEC-021

**Step 4: Run the check to verify it passes**

Run: inspect `docs/spec/SPEC-005-CLI.md`
Expected: exit-code policy is explicit and cross-references are present.

**Step 5: Commit**

Do not commit yet; combine with Task 3 metadata updates.

### Task 3: Update task tracking and changelog

**Files:**

- Modify: `docs/plan/tasks/TASK-S57-2-spec-005-cli-exit-policy.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing check**

Verify the task is still pending and changelog does not yet record completed S57-2 work.

**Step 2: Run the check to verify it fails**

Run: inspect the task file, plan index, and changelog.
Expected: S57-2 is pending and completion is not yet recorded.

**Step 3: Write the minimal implementation**

Update:

- task status and acceptance checklist
- resolved design choices/open questions in the task file
- plan index status for `TASK-S57-2`
- unreleased changelog entry with `(TASK-S57-2)`

**Step 4: Run the check to verify it passes**

Run: inspect the updated metadata files.
Expected: metadata consistently reflects the completed docs work.

**Step 5: Commit**

Prepare a conventional commit message such as: `docs(spec): define cli exit-immediately policy`

### Task 4: Verify the documentation change set

**Files:**

- Verify: `docs/spec/SPEC-005-CLI.md`
- Verify: `docs/plan/tasks/TASK-S57-2-spec-005-cli-exit-policy.md`
- Verify: `docs/plan/PLAN-INDEX.md`
- Verify: `CHANGELOG.md`

**Step 1: Run repo verification**

Run: `cargo test --workspace --quiet`
Expected: baseline remains green.

**Step 2: Run targeted diff/status verification**

Run: `git status --short`
Expected: only intended docs/plans/changelog files changed.

**Step 3: Review against acceptance criteria**

Manually confirm each `TASK-S57-2` acceptance item is satisfied.

**Step 4: Commit**

Do not commit without explicit user instruction.
