# TASK-249: Fix Clippy Warnings

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Fix all clippy warnings to satisfy `-D warnings` gate.

**Spec Reference:** Project quality gates (PLAN-INDEX.md)

**File Locations:**
- Multiple files across workspace (see audit report)
- Primary: `crates/ash-core/tests/proxy_ast_tests.rs:175`
- Primary: `crates/ash-parser/src/lower.rs:20`
- Primary: `crates/ash-parser/src/parse_module.rs:16`

---

## Background

The audit found clippy warnings that fail `-D warnings`:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
# FAILS
```

This blocks the project's own quality standard.

---

## Step 1: Get Full Warning List

Run clippy and capture all warnings:

```bash
cargo clippy --workspace --all-targets --all-features 2>&1 | tee clippy_output.txt
```

Categorize by type:
- Unused imports
- Unused variables
- Needless borrows
- Clone on Copy types
- etc.

---

## Step 2: Fix Warnings Systematically

### Pattern 1: Unused Imports

```rust
// Before
use ash_core::workflow::Workflow;
use ash_core::value::Value;  // unused

// After
use ash_core::workflow::Workflow;
```

### Pattern 2: Unused Variables

```rust
// Before
let result = some_operation();
// never use result

// After
some_operation();

// Or if needed:
let _result = some_operation();
```

### Pattern 3: Needless Borrow

```rust
// Before
let x = &&y;
if x == &&z { }

// After
if y == z { }
```

### Pattern 4: Clone on Copy

```rust
// Before
let x = y.clone();  // y is Copy

// After
let x = y;
```

---

## Step 3: Fix Each File

### File 1: `crates/ash-core/tests/proxy_ast_tests.rs:175`

```bash
cargo clippy --package ash-core --tests 2>&1 | grep proxy_ast_tests
```

Fix specific warning at line 175.

### File 2: `crates/ash-parser/src/lower.rs:20`

```bash
cargo clippy --package ash-parser 2>&1 | grep lower.rs
```

Fix specific warning at line 20.

### File 3: `crates/ash-parser/src/parse_module.rs:16`

```bash
cargo clippy --package ash-parser 2>&1 | grep parse_module.rs
```

Fix specific warning at line 16.

---

## Step 4: Verify Clean

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Expected: No warnings, exit code 0

---

## Step 5: Commit

```bash
git add -A
git commit -m "chore: fix all clippy warnings (TASK-249)

- Remove unused imports
- Fix unused variables
- Fix needless borrows
- Fix clone-on-Copy types
- All files pass clippy -D warnings

Fixes warnings in:
- ash-core/tests/proxy_ast_tests.rs
- ash-parser/src/lower.rs
- ash-parser/src/parse_module.rs
- [other files as needed]"
```

---

## Step 6: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-249 implementation"
  context: |
    Files to verify: All modified files
    
    Requirements:
    1. cargo clippy --workspace --all-targets --all-features -- -D warnings passes
    2. No warnings in any crate
    3. No functional changes (only style fixes)
    
    Run and report:
    1. cargo clippy --workspace --all-targets --all-features -- -D warnings
    2. Check exit code is 0
    3. Verify no functional changes (tests still pass)
    4. cargo test --workspace (regression check)
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] Full warning list captured
- [ ] Warnings categorized
- [ ] All warnings fixed
- [ ] Clippy -D warnings passes
- [ ] Tests still pass
- [ ] No functional changes
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 4
**Blocked by:** None
**Blocks:** None (can parallel with other fixes)
