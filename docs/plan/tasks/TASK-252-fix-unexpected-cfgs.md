# TASK-252: Fix Unexpected_cfgs Warning

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Fix `unexpected_cfgs` warning in ash-typeck.

**Spec Reference:** Cargo configuration, feature flags

**File Locations:**
- `crates/ash-typeck/Cargo.toml`
- `crates/ash-typeck/src/requirements.rs:920`

---

## Background

The audit found:

```bash
warning: unexpected_cfgs
  --> crates/ash-typeck/src/requirements.rs:920
   |
```

Code references `feature = "proptest"` but crate declares no such feature.

---

## Step 1: Investigate Current State

Check Cargo.toml for features:

```bash
grep -A 20 "\[features\]" crates/ash-typeck/Cargo.toml
```

Check code using the feature:

```bash
grep -n "feature.*proptest" crates/ash-typeck/src/requirements.rs
```

---

## Step 2: Determine Fix Strategy

Options:

### Option A: Add the feature to Cargo.toml

```toml
[features]
default = []
proptest = ["dep:proptest"]
```

### Option B: Remove the cfg attribute

If proptest is always available in dev-dependencies, remove the cfg.

### Option C: Fix typo

Maybe it's supposed to be a different feature name.

---

## Step 3: Implement Fix

Most likely: Add feature to Cargo.toml

```toml
# crates/ash-typeck/Cargo.toml
[features]
default = []
proptest = ["proptest"]  # or ["dep:proptest"] in newer cargo

[dependencies]
proptest = { version = "1.0", optional = true }
```

Or if proptest is only for tests:

```toml
[dev-dependencies]
proptest = "1.0"
```

And remove the cfg attribute from code.

---

## Step 4: Verify Fix

```bash
cargo check --package ash-typeck --all-targets --all-features
```

Should produce no warnings.

---

## Step 5: Commit

```bash
git add crates/ash-typeck/Cargo.toml
git add crates/ash-typeck/src/requirements.rs  # if modified
git commit -m "fix: resolve unexpected_cfgs warning in ash-typeck (TASK-252)

- [Add feature to Cargo.toml / Remove cfg from code / Fix typo]
- Feature 'proptest' now properly declared
- No more unexpected_cfgs warnings"
```

---

## Step 6: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-252 implementation"
  context: |
    Files to verify:
    - crates/ash-typeck/Cargo.toml
    - crates/ash-typeck/src/requirements.rs
    
    Requirements:
    1. cargo check --package ash-typeck produces no unexpected_cfgs warning
    2. Feature works as intended if kept
    3. Tests pass with and without feature
    
    Run and report:
    1. cargo check --package ash-typeck --all-targets --all-features
    2. cargo test --package ash-typeck
    3. cargo test --package ash-typeck --features proptest (if feature added)
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] Feature usage identified
- [ ] Fix strategy determined
- [ ] Cargo.toml updated (or cfg removed)
- [ ] Warning eliminated
- [ ] Tests pass
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 2
**Blocked by:** None
**Blocks:** None
