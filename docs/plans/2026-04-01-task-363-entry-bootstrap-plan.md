# Entry Bootstrap Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the canonical entry workflow verification and bootstrap path for Ash by preserving workflow return types, loading runtime stdlib support, validating the `main` contract, and deriving the correct runtime exit result.

**Architecture:** Keep the work centered in the parser and engine crates. First preserve declared workflow return types in parsed `WorkflowDef` metadata, then add small engine helpers for stdlib loading and entry verification, then implement the runtime bootstrap orchestration on top. Avoid speculative syntax and keep CLI rewiring out of scope.

**Tech Stack:** Rust workspace, `ash-parser`, `ash-engine`, Ash stdlib runtime modules, focused parser/engine tests.

---

### Task 1: Preserve workflow declared return types in the parser

**Files:**

- Modify: `crates/ash-parser/src/surface.rs`
- Modify: `crates/ash-parser/src/parse_workflow.rs`
- Modify: `crates/ash-parser/src/lower.rs` (only if plumbing needs adjustment)
- Test: `crates/ash-parser/src/parse_workflow.rs` tests and/or a focused parser test file

**Step 1: Write the failing test**

Add a parser regression that parses a workflow definition like:

```ash
workflow main(args: cap Args) -> Result<(), RuntimeError> { done; }
```

and asserts that the parsed `WorkflowDef` preserves the declared return type as `Some(Type::Constructor { name: "Result", ... })`.

**Step 2: Run test to verify it fails**

Run: `cargo test -p ash-parser parse_workflow::tests::test_workflow_def_with_return_type --quiet`
Expected: FAIL because `WorkflowDef` does not yet store a return type.

**Step 3: Write minimal implementation**

Add `return_type: Option<Type>` to `WorkflowDef` and parse the optional `-> Type` segment in `workflow_def()`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p ash-parser parse_workflow::tests::test_workflow_def_with_return_type --quiet`
Expected: PASS.

**Step 5: Run nearby parser checks**

Run: `cargo test -p ash-parser parse_workflow --quiet`
Expected: PASS.

### Task 2: Implement canonical entry signature verification (`TASK-364`)

**Files:**

- Modify: `crates/ash-engine/src/lib.rs` or create `crates/ash-engine/src/entry.rs`
- Modify: `crates/ash-engine/src/error.rs` if a focused error type is needed
- Test: `crates/ash-engine/src/check.rs` or a new `crates/ash-engine/tests/entry_verification.rs`

**Step 1: Write the failing tests**

Add tests for:

- valid `workflow main() -> Result<(), RuntimeError> { done; }`
- valid `workflow main(args: cap Args) -> Result<(), RuntimeError> { done; }`
- wrong return type rejected
- non-capability parameter rejected
- missing `main` rejected

**Step 2: Run tests to verify they fail**

Run: `cargo test -p ash-engine entry_verification --quiet`
Expected: FAIL because the verifier does not exist yet.

**Step 3: Write minimal implementation**

Add a focused helper that validates the parsed surface workflow definition against the exact `SPEC-022` entry contract.

**Step 4: Run tests to verify they pass**

Run: `cargo test -p ash-engine entry_verification --quiet`
Expected: PASS.

**Step 5: Self-review**

Confirm the verifier only applies to the designated entry workflow path and does not change ordinary workflow typing semantics.

### Task 3: Add runtime stdlib-loading support needed by entry verification (`TASK-363a` prerequisite slice)

**Files:**

- Modify: `crates/ash-engine/src/lib.rs` or create a focused helper module
- Read/consume: `std/src/runtime/mod.ash`, `std/src/runtime/error.ash`, `std/src/runtime/args.ash`, `std/src/runtime/supervisor.ash`
- Test: `crates/ash-engine/tests/entry_verification.rs` or a new focused stdlib-loading test file

**Step 1: Write the failing test**

Add a test that builds an engine, loads the runtime stdlib through the new helper, and then successfully parses/checks an entry workflow using `RuntimeError` and `Args` imports.

**Step 2: Run test to verify it fails**

Run: `cargo test -p ash-engine stdlib_loading --quiet`
Expected: FAIL because the helper does not exist yet.

**Step 3: Write minimal implementation**

Add a helper that reads the required stdlib runtime source files and loads them into the engine’s entry-aware verification path.

**Step 4: Run test to verify it passes**

Run: `cargo test -p ash-engine stdlib_loading --quiet`
Expected: PASS.

### Task 4: Implement runtime entry verification (`TASK-363b`)

**Files:**

- Modify: `crates/ash-engine/src/lib.rs` or `crates/ash-engine/src/entry.rs`
- Test: `crates/ash-engine/tests/entry_verification.rs`

**Step 1: Write the failing integration test**

Add a test that:

- creates an engine
- loads the runtime stdlib helper
- parses/checks an entry source
- calls the runtime entry verification helper

and expects a clean success for a valid `main`, plus a specific failure for an invalid one.

**Step 2: Run test to verify it fails**

Run: `cargo test -p ash-engine runtime_entry_verification --quiet`
Expected: FAIL because the runtime-facing helper does not exist yet.

**Step 3: Write minimal implementation**

Compose stdlib loading, parse/check, and `TASK-364` signature validation into a single runtime-facing verifier.

**Step 4: Run test to verify it passes**

Run: `cargo test -p ash-engine runtime_entry_verification --quiet`
Expected: PASS.

### Task 5: Implement bootstrap execution (`TASK-363c`)

**Files:**

- Modify: `crates/ash-engine/src/lib.rs` or `crates/ash-engine/src/entry.rs`
- Test: `crates/ash-engine/tests/entry_bootstrap.rs`

**Step 1: Write the failing bootstrap tests**

Add tests for:

- valid `main` that succeeds → bootstrap returns `0`
- valid `main` returning `Err(RuntimeError { exit_code: 42, ... })` → bootstrap returns `42`
- missing `main` → bootstrap returns a pre-entry error

**Step 2: Run tests to verify they fail**

Run: `cargo test -p ash-engine entry_bootstrap --quiet`
Expected: FAIL because the bootstrap helper does not exist yet.

**Step 3: Write minimal implementation**

Add a bootstrap helper that loads stdlib, parses/checks the entry file, verifies `main`, executes it, and maps the outcome into the required exit-result classes.

**Step 4: Run tests to verify they pass**

Run: `cargo test -p ash-engine entry_bootstrap --quiet`
Expected: PASS.

### Task 6: Update tracking docs and changelog

**Files:**

- Modify: `docs/plan/tasks/TASK-363a-runtime-stdlib-loading.md`
- Modify: `docs/plan/tasks/TASK-363b-runtime-main-verification.md`
- Modify: `docs/plan/tasks/TASK-363c-runtime-bootstrap-execution.md`
- Modify: `docs/plan/tasks/TASK-364-main-verification.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

**Step 1: Update task status/docs**

Mark the completed task statuses accurately and note that the implementation uses the existing engine-centered entry path.

**Step 2: Add changelog entry**

Record the entry bootstrap and verification work in `CHANGELOG.md` under `[Unreleased]`.

**Step 3: Run doc diff sanity check**

Run: `git diff --check`
Expected: PASS.

### Task 7: Final focused verification

**Files:**

- No new files

**Step 1: Run affected crate tests**

Run:

- `cargo test -p ash-parser parse_workflow --quiet`
- `cargo test -p ash-engine entry_verification --quiet`
- `cargo test -p ash-engine stdlib_loading --quiet`
- `cargo test -p ash-engine runtime_entry_verification --quiet`
- `cargo test -p ash-engine entry_bootstrap --quiet`

Expected: PASS.

**Step 2: Run format/build checks**

Run:

- `cargo fmt --all --check`
- `cargo build -q`

Expected: PASS.

**Step 3: Commit**

Run:

- `git add ...`
- `git commit -m "feat(runtime): implement entry bootstrap verification"`

Expected: clean commit containing code, tests, docs, and changelog updates.
