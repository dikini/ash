# TASK-240: Implement Workflow::Oblige Execution

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Implement runtime execution for `Workflow::Oblige` to satisfy SPEC-022 contract requirements.

**Spec Reference:** SPEC-022 (Workflow Typing), Section on obligation tracking

**File Locations:**
- Modify: `crates/ash-interp/src/execute.rs:856`
- Test: `crates/ash-interp/tests/obligation_execution_tests.rs` (create)

---

## Background

The audit found that `Workflow::Oblige` exists in AST and is type-checked, but execution hard-fails with `"not yet implemented in interpreter"`. This is a direct contract break.

Current code:
```rust
Workflow::Oblige { name, .. } => {
    Err(ExecError::ExecutionFailed(format!(
        "OBLIGE '{name}' not yet implemented in interpreter"
    )))
}
```

Target semantics:
- `Oblige { name, capability }` adds a named obligation to the runtime context
- The obligation tracks which capability created it (for auditing)
- Subsequent `CheckObligation` can verify/discharge the obligation

---

## Step 1: Write Failing Test

Create test file with property tests for obligation tracking:

```rust
// crates/ash-interp/tests/obligation_execution_tests.rs
use ash_interp::*;
use ash_core::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_oblige_adds_obligation(name in "[a-z_][a-z0-9_]*") {
        // Workflow that obliges then checks should find the obligation
    }
    
    #[test]
    fn test_oblige_without_check_fails_ensures(name in "[a-z_][a-z0-9_]*") {
        // Workflow with `ensures: O(name)` that never obliges should fail
    }
}
```

**Verification:** Test should fail with "not yet implemented"

---

## Step 2: Design Obligation Tracking in Context

Review `RuntimeContext` structure:

```bash
grep -n "struct RuntimeContext" crates/ash-interp/src/context.rs
```

Add obligation tracking:

```rust
// In RuntimeContext or ExecutionContext
pub struct ActiveObligation {
    pub name: String,
    pub capability: CapabilityRef,
    pub created_at: Span,
}

pub struct RuntimeContext {
    // ... existing fields ...
    pub obligations: HashMap<String, ActiveObligation>,
}
```

---

## Step 3: Implement Oblige Execution

Modify `execute_workflow` in `crates/ash-interp/src/execute.rs:856`:

```rust
Workflow::Oblige { name, capability, span } => {
    let obligation = ActiveObligation {
        name: name.clone(),
        capability: capability.clone(),
        created_at: *span,
    };
    
    // Check if already obligated (linear obligation violation)
    if ctx.obligations.contains_key(name) {
        return Err(ExecError::LinearObligationViolation {
            name: name.clone(),
            span: *span,
        });
    }
    
    ctx.obligations.insert(name.clone(), obligation);
    
    // Record provenance
    ctx.provenance.record_oblige(name, capability, span);
    
    Ok(Value::Null)
}
```

---

## Step 4: Run Tests

```bash
cargo test --package ash-interp obligation_execution_tests -v
```

Expected: Tests pass

---

## Step 5: Commit

```bash
git add crates/ash-interp/src/execute.rs
git add crates/ash-interp/tests/obligation_execution_tests.rs
git commit -m "feat: implement Workflow::Oblige execution (TASK-240)

- Add ActiveObligation tracking to RuntimeContext
- Implement obligation insertion with linearity check
- Add provenance recording for audit trail
- Property tests for obligation lifecycle"
```

---

## Step 6: Codex Verification (REQUIRED)

Spawn codex sub-agent to verify implementation:

```
delegate_task to codex:
  goal: "Verify TASK-240 implementation"
  context: |
    Files to verify:
    - crates/ash-interp/src/execute.rs (Oblige execution)
    - crates/ash-interp/tests/obligation_execution_tests.rs
    - Any modified context files
    
    Spec reference: SPEC-022 obligation tracking section
    Requirements:
    1. Oblige adds obligation to context
    2. Linearity check prevents duplicate obligations
    3. Provenance recorded
    4. Returns Value::Null
    
    Run and report:
    1. cargo test --package ash-interp obligation
    2. cargo clippy --package ash-interp --all-targets --all-features -- -D warnings
    3. cargo fmt --check --package ash-interp
    4. Check for spec compliance
    5. Check for code quality issues
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

Do not mark task complete until VERIFIED.

---

## Completion Checklist

- [ ] Failing tests written
- [ ] Oblige execution implemented
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 6
**Blocked by:** None
**Blocks:** TASK-241 (CheckObligation)
