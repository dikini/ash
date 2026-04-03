# Runtime Stdlib Loading Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Complete `TASK-363a` by giving `ash-engine` an honest runtime stdlib registry for the Phase 57 entry path so canonical `use result::...` and `use runtime::...` imports are backed by registered module sources instead of raw preflight shims.

**Architecture:** Keep the work engine-local. Add a small runtime stdlib module table to `Engine`, load the required stdlib sources by canonical module path, validate entry `use` prelude items against that registry, and reuse the existing narrow entry parser/bootstrap path on top.

**Tech Stack:** Rust workspace, `ash-engine`, existing runtime stdlib sources under `std/src`, focused engine integration tests.

---

### Task 1: Add engine-owned runtime stdlib registry

**Files:**

- Modify: `crates/ash-engine/src/lib.rs`
- Modify: `crates/ash-engine/src/entry.rs`
- Test: `crates/ash-engine/tests/entry_verification.rs`

**Step 1: Write the failing test**

Add a test that builds an engine, calls a new runtime stdlib loader, and asserts the engine reports registered modules for `result`, `runtime`, `runtime::error`, and `runtime::args`.

**Step 2: Run test to verify it fails**

Run: `cargo test -p ash-engine loads_registered_runtime_stdlib_modules --quiet`
Expected: FAIL because the engine does not expose a runtime stdlib registry yet.

**Step 3: Write minimal implementation**

Add engine storage for registered runtime stdlib modules plus explicit helpers such as:

- `register_runtime_stdlib_module(...)`
- `load_runtime_stdlib()`
- `has_registered_runtime_module(...)`

Keep the registry source-based and limited to canonical runtime stdlib paths.

**Step 4: Run test to verify it passes**

Run: `cargo test -p ash-engine loads_registered_runtime_stdlib_modules --quiet`
Expected: PASS.

### Task 2: Validate entry imports against the registry

**Files:**

- Modify: `crates/ash-engine/src/entry.rs`
- Modify: `crates/ash-engine/src/lib.rs`
- Test: `crates/ash-engine/tests/entry_verification.rs`

**Step 1: Write the failing test**

Add a test that parses/checks/verifies an entry source with leading imports only after the runtime stdlib registry is populated.

**Step 2: Run test to verify it fails**

Run: `cargo test -p ash-engine parses_checks_and_verifies_entry_source_with_runtime_imports --quiet`
Expected: FAIL because entry import validation is not yet tied to engine-owned registered modules.

**Step 3: Write minimal implementation**

Teach the entry path to parse the leading `use` prelude narrowly and confirm those canonical paths are present in the engine registry before stripping the prelude and parsing the workflow body.

**Step 4: Run test to verify it passes**

Run: `cargo test -p ash-engine parses_checks_and_verifies_entry_source_with_runtime_imports --quiet`
Expected: PASS.

### Task 3: Rewire bootstrap to the engine registry

**Files:**

- Modify: `crates/ash-engine/src/lib.rs`
- Modify: `crates/ash-engine/src/entry.rs`
- Test: `crates/ash-engine/tests/entry_verification.rs`

**Step 1: Write/adjust the failing test**

Ensure the existing bootstrap tests exercise the registry-backed stdlib path rather than the raw preflight helper.

**Step 2: Run tests to verify the old path is insufficient**

Run: `cargo test -p ash-engine --test entry_verification --quiet`
Expected: FAIL or require code changes because bootstrap still relies on the older free-function preflight flow.

**Step 3: Write minimal implementation**

Update `bootstrap_entry_source()` and related entry helpers so runtime stdlib loading happens through the engine registry.

**Step 4: Run tests to verify they pass**

Run: `cargo test -p ash-engine --test entry_verification --quiet`
Expected: PASS.

### Task 4: Update docs and changelog for `TASK-363a`

**Files:**

- Modify: `docs/plan/tasks/TASK-363a-runtime-stdlib-loading.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

**Step 1: Update task bookkeeping**

Mark `TASK-363a` complete only if the engine now owns runtime stdlib registration and the entry path validates imports against that registry.

**Step 2: Add changelog entry**

Record the runtime stdlib registry/import-validation work under `[Unreleased]`.

**Step 3: Run doc diff sanity check**

Run: `git diff --check`
Expected: PASS.

### Task 5: Final focused verification

**Files:**

- No new files

**Step 1: Run targeted verification**

Run:

- `cargo test -p ash-engine --test entry_verification --quiet`
- `cargo test -p ash-engine --quiet`

Expected: PASS.

**Step 2: Run affected workspace verification**

Run:

- `cargo test -p ash-parser --quiet`
- `cargo test -p ash-cli --quiet`
- `cargo fmt --all --check`

Expected: PASS.
