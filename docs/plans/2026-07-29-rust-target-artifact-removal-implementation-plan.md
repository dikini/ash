# Rust Target Artifact Removal Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove all tracked Cargo `target/` artifacts and reject any future attempt to track a Cargo build-output path.

**Architecture:** A repository-wide `target/` rule makes all present and future Cargo output directories ignored. A focused Python policy test inspects Git's index and `git check-ignore`; the pre-commit gate invokes that test so a force-added target artifact cannot be committed.

**Tech Stack:** Git index commands, `.gitignore`, Python standard-library `unittest`, Bash gate orchestration, Markdown task tracking.

---

### Task 1: Establish the tracked-artifact cleanup record

**Files:**

- Create: `docs/plan/tasks/TASK-2043-remove-tracked-rust-target-artifacts.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

**Step 1: Create the task record**

Document the observed inventory (`crates/ash-bench/target/`: 3 files and `crates/ash-fuzz/target/`: 582 files), the global-ignore policy, non-goals, TDD steps, and completion checklist. Set its status to **In progress**.

**Step 2: Add the plan-index row**

Add TASK-2043 after TASK-2042 in the Phase-202 table. State that it removes tracked Cargo build output and makes all nested `target/` directories ignored.

**Step 3: Add the changelog placeholder**

Add an Unreleased `Changed` entry describing the cleanup and enforced no-tracked-target policy, with `(TASK-2043)`.

### Task 2: Write the failing repository-policy test

**Files:**

- Create: `tools/docs/tests/test_no_tracked_rust_target_directories.py`
- Modify: `scripts/check-pre-commit-gate.sh`

**Step 1: Write the failing test**

Create a standard-library unittest that:

```python
tracked = subprocess.run(
    ["git", "ls-files", "-z"], cwd=REPOSITORY_ROOT,
    check=True, capture_output=True,
).stdout.decode().split("\0")
tracked_targets = [path for path in tracked if path and "/target/" in f"/{path}"]
self.assertEqual(tracked_targets, [])
```

Add a second assertion that `git check-ignore -q` succeeds for both `target/.rustc_info.json` and `crates/example/target/.rustc_info.json`, proving the policy is global rather than a single-crate exception.

**Step 2: Run the test to verify it fails**

Run:

```bash
python3 -m unittest tools.docs.tests.test_no_tracked_rust_target_directories
```

Expected: FAIL, listing the current `crates/ash-bench/target/` and `crates/ash-fuzz/target/` paths from the Git index.

**Step 3: Register the guard in pre-commit**

Add the focused unittest next to the existing Python gate-policy test in `scripts/check-pre-commit-gate.sh`. This ensures a force-added target artifact is rejected from the staged index before commit.

### Task 3: Remove target artifacts from Git and install the global ignore rule

**Files:**

- Modify: `.gitignore`
- Remove from Git index: `crates/ash-bench/target/`
- Remove from Git index: `crates/ash-fuzz/target/`

**Step 1: Replace the path-specific ignore rule**

Replace `crates/ash-bench/target/` with:

```gitignore
target/
```

The pattern has no slash and therefore ignores a directory named `target` at any nesting level.

**Step 2: Remove build output from the index without deleting local caches**

Run:

```bash
git rm -r --cached crates/ash-bench/target crates/ash-fuzz/target
```

Do not use a command that removes the local directories. They should remain as ignored Cargo caches after the index change.

**Step 3: Run the test to verify it passes**

Run:

```bash
python3 -m unittest tools.docs.tests.test_no_tracked_rust_target_directories
```

Expected: PASS; no Git-index path lies inside `target/`, and both root and nested examples are ignored.

### Task 4: Complete task metadata and verify the cleanup

**Files:**

- Modify: `docs/plan/tasks/TASK-2043-remove-tracked-rust-target-artifacts.md`
- Modify: `docs/plan/PLAN-INDEX.md`

**Step 1: Record task-owned evidence**

Set TASK-2043 to **Complete** after verification. Record the RED failure, the target-directory inventory, the index-only removal, and the passing guard.

**Step 2: Run focused verification**

Run:

```bash
python3 -m unittest tools.docs.tests.test_no_tracked_rust_target_directories
bash scripts/check-docs-gate.sh
git diff --check
git status --short
```

Expected: the Python guard and documentation gate pass; no tracked `target/` path remains; local target directories are ignored rather than reported as modifications.

**Step 3: Run the normal staged gate when preparing the commit**

Run:

```bash
bash .githooks/pre-commit
```

Expected: the new no-tracked-target guard is executed by the ordinary pre-commit workflow and passes. Commit only after the user authorizes it, using a conventional message such as `build: remove tracked Cargo target artifacts (TASK-2043)`.
