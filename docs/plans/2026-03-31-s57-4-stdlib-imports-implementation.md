# TASK-S57-4 Stdlib Import Rules Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Clarify the normative Ash standard-library import rules in SPEC-009 and SPEC-012 and record task completion metadata.

**Architecture:** Keep `::` as the sole path separator, define standard-library modules as compiler-provided root modules, and normatively bind implicit imports to the standard prelude. Update tracking documents in the same change set.

**Tech Stack:** Markdown specifications, task tracking docs, Common Changelog

---

### Task 1: Update SPEC-009 module rules

**Files:**

- Modify: `docs/spec/SPEC-009-MODULES.md`

**Step 1: Write the failing test**

For this documentation task, the failing condition is the missing normative text: SPEC-009 does not currently define the standard-library root namespace or reject dot-style module paths.

**Step 2: Run test to verify it fails**

Inspect `docs/spec/SPEC-009-MODULES.md` and confirm it lacks:

- explicit `::`-only wording
- standard-library root namespace wording
- prelude cross-reference for implicit names

Expected: all three are absent.

**Step 3: Write minimal implementation**

Add normative wording that:

- `::` is the only path separator
- standard-library modules resolve as compiler-provided root modules
- implicit names come from the prelude, with other names requiring explicit imports

**Step 4: Run test to verify it passes**

Re-read the edited section and confirm the new rules and examples are present.

**Step 5: Commit**

Stage the spec update with the rest of the task changes.

### Task 2: Update SPEC-012 import rules

**Files:**

- Modify: `docs/spec/SPEC-012-IMPORTS.md`

**Step 1: Write the failing test**

The failing condition is the missing normative text: SPEC-012 does not currently define standard-library root imports or the exact implicit prelude surface.

**Step 2: Run test to verify it fails**

Inspect `docs/spec/SPEC-012-IMPORTS.md` and confirm it lacks:

- a stdlib-import subsection
- explicit prelude semantics
- invalid examples for legacy dot-style imports

Expected: these clarifications are absent.

**Step 3: Write minimal implementation**

Add normative wording that:

- import paths use `::` only
- stdlib imports use root modules such as `result::Result`
- the prelude implicitly provides `Option`, `Some`, `None`, `Result`, `Ok`, and `Err`
- other stdlib names still require explicit `use`

**Step 4: Run test to verify it passes**

Re-read the edited section and confirm the new rules and examples are present.

**Step 5: Commit**

Stage the import spec update with the rest of the task changes.

### Task 3: Update task tracking and changelog

**Files:**

- Modify: `docs/plan/tasks/TASK-S57-4-spec-009-012-stdlib-imports.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing test**

The failing condition is stale project metadata: TASK-S57-4 is still pending and there is no changelog entry for the spec clarification.

**Step 2: Run test to verify it fails**

Inspect the task file, plan index, and changelog.

Expected:

- task status is pending
- PLAN-INDEX row is pending
- `CHANGELOG.md` has no TASK-S57-4 entry

**Step 3: Write minimal implementation**

Update the task file to record the resolved design and mark completion, update PLAN-INDEX to complete, and add a Common Changelog entry under `Unreleased`.

**Step 4: Run test to verify it passes**

Re-read all three files and confirm they now reflect the completed spec decision.

**Step 5: Commit**

Stage the tracking and changelog updates with the rest of the task changes.
