# TASK-251: Fix Rustdoc Warnings

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Fix all rustdoc warnings to satisfy documentation quality gate.

**Spec Reference:** Project quality gates (PLAN-INDEX.md)

**File Locations:**
- `crates/ash-typeck/src/requirements.rs:920`
- `crates/ash-interp/src/execute.rs:916`
- `crates/ash-parser/src/surface.rs:534`

---

## Background

The audit found rustdoc warnings:

```bash
cargo doc --workspace --no-deps
# Emits warnings
```

Types of warnings:
- Broken intra-doc links
- Invalid code blocks
- Invalid HTML tags

---

## Step 1: Get Full Warning List

```bash
cargo doc --workspace --no-deps 2>&1 | grep -E "warning|error" | head -50
```

---

## Step 2: Fix Each Warning Type

### Broken Intra-Doc Links

```rust
// Before: broken link
/// See [Workflow::execute] for details.

// After: working link  
/// See [`Workflow::execute`](crate::workflow::Workflow::execute) for details.
```

### Invalid Code Blocks

```rust
// Before: invalid language
/// ```invalid_lang
/// code
/// ```

// After: valid language
/// ```rust
/// code
/// ```
```

### Invalid HTML

```rust
// Before: invalid tag
/// <invalid>content</invalid>

// After: proper formatting
/// **content**
```

---

## Step 3: Fix Specific Files

### File 1: `crates/ash-typeck/src/requirements.rs:920`

```bash
cargo doc --package ash-typeck 2>&1 | grep requirements.rs
```

Fix warning at line 920.

### File 2: `crates/ash-interp/src/execute.rs:916`

```bash
cargo doc --package ash-interp 2>&1 | grep execute.rs
```

Fix warning at line 916.

### File 3: `crates/ash-parser/src/surface.rs:534`

```bash
cargo doc --package ash-parser 2>&1 | grep surface.rs
```

Fix warning at line 534.

---

## Step 4: Verify Clean

```bash
cargo doc --workspace --no-deps 2>&1 | grep -i warning
```

Expected: No warnings

---

## Step 5: Commit

```bash
git add -A
git commit -m "docs: fix rustdoc warnings (TASK-251)

- Fix broken intra-doc links
- Fix invalid code block languages
- Fix invalid HTML tags
- All documentation builds without warnings

Files fixed:
- ash-typeck/src/requirements.rs
- ash-interp/src/execute.rs
- ash-parser/src/surface.rs"
```

---

## Step 6: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-251 implementation"
  context: |
    Files to verify: All modified documentation
    
    Requirements:
    1. cargo doc --workspace --no-deps produces no warnings
    2. Links work correctly
    3. Code blocks render properly
    4. HTML renders correctly
    
    Run and report:
    1. cargo doc --workspace --no-deps
    2. Check for warnings
    3. Verify docs render (open target/doc/index.html check)
    4. cargo test --workspace (regression check)
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] All warnings identified
- [ ] Broken links fixed
- [ ] Invalid code blocks fixed
- [ ] Invalid HTML fixed
- [ ] cargo doc produces no warnings
- [ ] Tests pass
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 6
**Blocked by:** None
**Blocks:** None
