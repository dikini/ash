# TASK-250: Run Cargo Fmt

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Format entire workspace with `cargo fmt`.

**Spec Reference:** Project quality gates (PLAN-INDEX.md)

**File Locations:**
- All `.rs` files in workspace

---

## Background

The audit found formatting failures:

```bash
cargo fmt --check
# FAILS
```

Files affected (from audit):
- `crates/ash-interp/src/execute.rs`
- `crates/ash-parser/tests/proxy_parser_tests.rs`
- `crates/ash-core/tests/proxy_ast_tests.rs`

---

## Step 1: Check Current State

```bash
cargo fmt --check 2>&1 | head -50
```

---

## Step 2: Run Formatter

```bash
cargo fmt
```

---

## Step 3: Verify Clean

```bash
cargo fmt --check
```

Expected: No output, exit code 0

---

## Step 4: Review Changes

```bash
git diff --stat
```

Ensure only formatting changes (whitespace, line breaks, etc.)

---

## Step 5: Commit

```bash
git add -A
git commit -m "style: format workspace with cargo fmt (TASK-250)

- Format all Rust source files
- No functional changes
- Satisfies cargo fmt --check gate"
```

---

## Step 6: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-250 implementation"
  context: |
    Requirements:
    1. cargo fmt --check passes
    2. Only formatting changes (no functional changes)
    3. Tests still pass
    
    Run and report:
    1. cargo fmt --check
    2. cargo test --workspace (regression check)
    3. git diff --stat (verify scope)
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] cargo fmt run
- [ ] cargo fmt --check passes
- [ ] Tests pass
- [ ] No functional changes
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 2
**Blocked by:** None
**Blocks:** None
