# TASK-360 RuntimeError Type Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Convert stdlib `RuntimeError` to the canonical single-variant ADT form and align immediate tests and task documentation.

**Architecture:** Keep the change narrowly scoped to the stdlib surface and its direct parser-facing checks. Use TDD to lock in the expected constructor-bearing syntax before changing `std/src/runtime/error.ash`, then update the task document and changelog to reflect the normalized contract.

**Tech Stack:** Rust workspace tests, Ash stdlib `.ash` source files, markdown planning/docs.

---

### Task 1: Add failing parser and surface assertions

**Files:**

- Modify: `crates/ash-parser/tests/stdlib_parsing.rs`
- Modify: `crates/ash-parser/tests/stdlib_surface.rs`

**Step 1: Write the failing test**

Add assertions that require the exact text `pub type RuntimeError = RuntimeError {` and confirm the stdlib no longer uses the plain record-alias form.

**Step 2: Run test to verify it fails**

Run: `cargo test -p ash-parser --test stdlib_parsing --test stdlib_surface --quiet`
Expected: FAIL because `std/src/runtime/error.ash` still contains the record-alias syntax.

**Step 3: Keep the failure focused**

Do not change production files yet; only adjust test expectations until the old syntax is rejected by at least one focused assertion.

**Step 4: Commit**

Commit after the red phase is observed.

### Task 2: Implement the canonical stdlib type definition

**Files:**

- Modify: `std/src/runtime/error.ash`

**Step 1: Write minimal implementation**

Change the definition to:

```ash
pub type RuntimeError = RuntimeError {
    exit_code: Int,
    message: String
};
```

**Step 2: Run focused tests to verify they pass**

Run: `cargo test -p ash-parser --test stdlib_parsing --test stdlib_surface --quiet`
Expected: PASS.

**Step 3: Commit**

Commit the stdlib update once the focused tests pass.

### Task 3: Align task documentation and changelog

**Files:**

- Modify: `docs/plan/tasks/TASK-360-runtime-error-type.md`
- Modify: `CHANGELOG.md`

**Step 1: Update the task file**

Mark the validation gate as resolved against `SPEC-020` and describe the canonical ADT syntax consistently.

**Step 2: Update changelog**

Add an `[Unreleased]` entry for TASK-360 noting that the stdlib `RuntimeError` now uses the canonical single-variant ADT form for entry-point contracts.

**Step 3: Run focused verification**

Run: `cargo test -p ash-parser --test stdlib_parsing --test stdlib_surface --quiet`
Expected: PASS.

### Task 4: Final verification and review

**Files:**

- Review only

**Step 1: Run fresh verification**

Run: `cargo test -p ash-parser --test stdlib_parsing --test stdlib_surface --quiet`
Expected: PASS.

**Step 2: Inspect changed files**

Run: `git status --short`
Expected: only TASK-360 files changed in the worktree.

**Step 3: Request review**

Run a spec-focused review first, then a code-quality review for the TASK-360 diff.
