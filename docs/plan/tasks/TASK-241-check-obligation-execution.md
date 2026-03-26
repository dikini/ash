# TASK-241: Implement Workflow::CheckObligation Execution

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Implement runtime execution for `Workflow::CheckObligation` to complete SPEC-022 obligation lifecycle.

**Spec Reference:** SPEC-022 (Workflow Typing), Section on obligation checking

**File Locations:**
- Modify: `crates/ash-interp/src/execute.rs:867`
- Test: `crates/ash-interp/tests/obligation_execution_tests.rs` (extend)

---

## Background

The audit found that `Workflow::CheckObligation` exists but execution hard-fails. This must work with TASK-240's obligation tracking.

Current code:
```rust
Workflow::CheckObligation { name, .. } => {
    Err(ExecError::ExecutionFailed(format!(
        "CHECK_OBLIGATION '{name}' not yet implemented in interpreter"
    )))
}
```

Target semantics:
- `CheckObligation { name }` returns `Value::Bool(true)` if obligation exists
- Returns `Value::Bool(false)` if obligation not found
- Does NOT discharge the obligation (separate operation)
- Used in guards and conditions

---

## Step 1: Write Failing Tests

Extend existing test file:

```rust
// Add to crates/ash-interp/tests/obligation_execution_tests.rs

proptest! {
    #[test]
    fn test_check_obligation_finds_active(name in "[a-z_][a-z0-9_]*") {
        // After oblige, check should return true
    }
    
    #[test]
    fn test_check_obligation_missing_returns_false(name in "[a-z_][a-z0-9_]*") {
        // Check without oblige should return false
    }
    
    #[test]
    fn test_check_obligation_not_linear(name in "[a-z_][a-z0-9_]*") {
        // Multiple checks should all return true (read-only)
    }
}
```

---

## Step 2: Implement CheckObligation Execution

Modify `execute_workflow` in `crates/ash-interp/src/execute.rs:867`:

```rust
Workflow::CheckObligation { name, span } => {
    let found = ctx.obligations.contains_key(name);
    
    // Record provenance for audit
    ctx.provenance.record_check_obligation(name, found, span);
    
    Ok(Value::Bool(found))
}
```

---

## Step 3: Add Discharge Operation (if needed)

If SPEC-022 requires obligation discharge (removal), add:

```rust
// May be a separate Workflow variant or internal operation
Workflow::DischargeObligation { name, span } => {
    match ctx.obligations.remove(name) {
        Some(_) => {
            ctx.provenance.record_discharge(name, span);
            Ok(Value::Null)
        }
        None => Err(ExecError::ObligationNotFound {
            name: name.clone(),
            span: *span,
        }),
    }
}
```

Check SPEC-022 for discharge semantics.

---

## Step 4: Integration Test

Test full lifecycle:

```rust
#[test]
fn test_obligation_lifecycle() {
    // 1. Workflow obliges
    // 2. Check confirms
    // 3. Discharge removes
    // 4. Check returns false
}
```

---

## Step 5: Run Tests

```bash
cargo test --package ash-interp obligation_execution_tests -v
```

Expected: All tests pass

---

## Step 6: Commit

```bash
git add crates/ash-interp/src/execute.rs
git add crates/ash-interp/tests/obligation_execution_tests.rs
git commit -m "feat: implement Workflow::CheckObligation execution (TASK-241)

- Implement check that returns bool based on obligation presence
- Add provenance recording for obligation checks
- Integration tests for full obligation lifecycle
- Optional discharge operation if required by SPEC-022"
```

---

## Step 7: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-241 implementation"
  context: |
    Files to verify:
    - crates/ash-interp/src/execute.rs (CheckObligation execution)
    - crates/ash-interp/tests/obligation_execution_tests.rs
    
    Spec reference: SPEC-022 obligation checking section
    Requirements:
    1. CheckObligation returns Bool(true) if obligation exists
    2. CheckObligation returns Bool(false) if not found
    3. Does not modify obligation set (read-only)
    4. Provenance recorded
    
    Run and report:
    1. cargo test --package ash-interp obligation
    2. cargo clippy --package ash-interp --all-targets --all-features -- -D warnings
    3. cargo fmt --check --package ash-interp
    4. Check spec compliance
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] Failing tests written
- [ ] CheckObligation execution implemented
- [ ] Optional discharge implemented (if needed)
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 6
**Blocked by:** TASK-240 (Oblige execution)
**Blocks:** None (completes obligation pair)
