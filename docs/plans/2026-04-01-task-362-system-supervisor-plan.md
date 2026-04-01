# TASK-362 System Supervisor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the minimal `system_supervisor` placeholder with the canonical ash-std supervisor contract and lock it in with focused regression coverage.

**Architecture:** Keep TASK-362 narrowly scoped to the stdlib-visible supervisor surface. Encode the stable signature and exit-code shaping contract in `std/src/runtime/supervisor.ash`, update tests to reject the old placeholder, and document the runtime-only spawn/completion boundary instead of inventing new surface syntax.

**Tech Stack:** Rust workspace, Ash stdlib `.ash` modules, parser regression tests, focused plan updates.

---

### Task 1: Pin the canonical supervisor surface in tests

**Files:**

- Modify: `crates/ash-parser/tests/stdlib_surface.rs`
- Optional modify: `crates/ash-parser/tests/stdlib_parsing.rs`

**Step 1: Write the failing test**

Change the stdlib surface assertions so they fail if `std/src/runtime/supervisor.ash` still contains the old minimal `ret 0;` placeholder and so they require the canonical supervisor contract markers instead.

**Step 2: Run test to verify it fails**

Run: `cargo test -p ash-parser --quiet --test stdlib_surface runtime_stdlib_surface_is_exposed`
Expected: FAIL because the current supervisor file still exposes the placeholder body.

**Step 3: Add parser-feasible regression coverage**

If the selected supervisor body is valid for the current parser, add a focused parsing regression in `crates/ash-parser/tests/stdlib_parsing.rs` for that exact source shape. If it is not parser-feasible, keep the coverage textual and document the boundary in the task/docs update.

**Step 4: Run parser tests**

Run:

- `cargo test -p ash-parser --quiet --test stdlib_surface runtime_stdlib_surface_is_exposed`
- `cargo test -p ash-parser --quiet --test stdlib_parsing test_runtime_supervisor_workflow_definition_parses`

Expected: The new assertions fail until the supervisor file is updated.

### Task 2: Implement the supervisor surface

**Files:**

- Modify: `std/src/runtime/supervisor.ash`

**Step 1: Write the minimal contract-complete body**

Update `std/src/runtime/supervisor.ash` to:

- import the canonical names it uses
- keep `pub workflow system_supervisor(args: cap Args) -> Int`
- remove the pure placeholder body
- encode the canonical result/exit-code shaping story in the narrowest feasible Ash surface
- document that `main` spawn and terminal-completion observation are runtime-internal semantics for downstream bootstrap work

**Step 2: Run the focused tests**

Run:

- `cargo test -p ash-parser --quiet --test stdlib_surface runtime_stdlib_surface_is_exposed`
- `cargo test -p ash-parser --quiet --test stdlib_parsing test_runtime_supervisor_workflow_definition_parses`

Expected: PASS for the updated canonical surface.

**Step 3: Self-review**

Confirm the file does not introduce `await`, does not change the public signature, and does not overreach into runtime bootstrap implementation.

### Task 3: Task tracking and release notes

Once the supervisor surface and focused regression coverage land, update task tracking and `CHANGELOG.md` to record TASK-362 as complete while keeping the runtime/bootstrap execution boundary explicitly deferred to TASK-363c.
