# TASK-361 Args Capability Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Verify and complete the canonical stdlib `Args` capability contract for entry-point workflows.

**Architecture:** Keep the change tightly scoped to stdlib-facing tests, the TASK-361 task document, and changelog evidence. Use TDD to prove the import surface, usage-site type form, and explicit `observe Args <index>` invocation before changing any production-facing files.

**Tech Stack:** Rust workspace tests, Ash stdlib `.ash` source files, markdown planning/docs.

---

### Task 1: Add failing regression coverage for the Args contract

**Files:**

- Modify: `crates/ash-parser/tests/stdlib_surface.rs`
- Modify: `crates/ash-typeck/tests/runtime_args_contracts.rs` (new) or nearest existing targeted typecheck test file

**Step 1: Write the failing test**

Add focused assertions/tests that require:

- stdlib exposes `pub capability Args`
- entry workflows can import `use runtime::Args`
- workflow parameters accept `args: cap Args`
- the canonical call form is `observe Args 0`

**Step 2: Run test to verify it fails**

Run: `cargo test -p ash-parser --test stdlib_surface -p ash-typeck runtime_args_contracts --quiet`
Expected: FAIL because the regression barrier is not fully implemented yet.

**Step 3: Keep failure focused**

Do not change stdlib or task docs yet; ensure at least one focused assertion fails for the exact TASK-361 contract.

### Task 2: Implement the minimal Args-surface updates

**Files:**

- Modify: `std/src/runtime/args.ash` only if exact syntax/format needs normalization
- Modify: `crates/ash-parser/tests/stdlib_surface.rs`
- Modify: `crates/ash-typeck/tests/runtime_args_contracts.rs` (new) or chosen existing test file

**Step 1: Write minimal implementation**

Make only the changes needed so tests accept the canonical contract:

```ash
pub capability Args: observe(index: Int) returns Option<String>;
```

and the workflow-facing forms:

```ash
use runtime::Args
workflow main(args: cap Args) -> Result<(), RuntimeError> {
  let first = observe Args 0;
  done;
}
```

**Step 2: Run focused tests to verify they pass**

Run: `cargo test -p ash-parser --test stdlib_surface --quiet`
Run: `cargo test -p ash-typeck runtime_args_contracts --quiet`
Expected: PASS.

### Task 3: Align task documentation and changelog

**Files:**

- Modify: `docs/plan/tasks/TASK-361-args-capability.md`
- Modify: `CHANGELOG.md`

**Step 1: Update the task file**

Mark the validation gate and status consistently with the verified canonical `Args` surface.

**Step 2: Update changelog**

Add an `[Unreleased]` entry for TASK-361 noting that the stdlib `Args` capability contract is now verified for import, `cap Args` workflow parameters, and explicit observation syntax.

**Step 3: Re-run focused verification**

Run: `cargo test -p ash-parser --test stdlib_surface --quiet`
Run: `cargo test -p ash-typeck runtime_args_contracts --quiet`
Expected: PASS.

### Task 4: Final verification and review

**Files:**

- Review only

**Step 1: Run fresh verification**

Run: `cargo test -p ash-parser --test stdlib_surface --quiet`
Run: `cargo test -p ash-typeck runtime_args_contracts --quiet`
Expected: PASS.

**Step 2: Inspect changed files**

Run: `git status --short`
Expected: only TASK-361-related files changed in the worktree.

**Step 3: Review**

Run a spec-focused review first, then a code-quality review for the TASK-361 diff.
