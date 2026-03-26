# TASK-243: Implement YIELD Runtime Execution

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Implement runtime execution for `CoreWorkflow::Yield` with continuation capture.

**Spec Reference:** SPEC-023 (Proxy Workflows), Execution semantics

**File Locations:**
- Modify: `crates/ash-interp/src/execute.rs:874`
- Test: `crates/ash-interp/tests/yield_execution_tests.rs` (create)

---

## Background

The audit found YIELD returns an execution error after suspending. With TASK-242's lowering fix, we can now implement proper execution.

Target semantics:
1. Evaluate request expression
2. Capture continuation (the resume clause)
3. Suspend current workflow
4. Route request to role handler
5. On response, restore continuation with result

---

## Step 1: Understand Runtime State

Review current execution context:

```bash
grep -n "struct.*Context" crates/ash-interp/src/context.rs
grep -n "suspend\|resume\|yield" crates/ash-interp/src/execute.rs
```

Check if there's existing continuation/suspension machinery from proxy work.

---

## Step 2: Write Failing Tests

```rust
// crates/ash-interp/tests/yield_execution_tests.rs
use ash_interp::*;
use ash_core::*;

#[test]
fn test_yield_suspends_workflow() {
    // Yield should suspend, not error
}

#[test]
fn test_yield_routes_to_role() {
    // Request should be routed to role's current handler
}

#[test]
fn test_resume_continues_execution() {
    // After response, continuation should execute
}

proptest! {
    #[test]
    fn test_yield_resume_roundtrip(request in arbitrary_json()) {
        // Property: yield then resume = original workflow continues
    }
}
```

---

## Step 3: Design Yield Execution

Implement in `crates/ash-interp/src/execute.rs:874`:

```rust
Workflow::Yield {
    role,
    request,
    expected_response_type: _,
    continuation,
    span,
} => {
    // 1. Evaluate request
    let request_value = eval_expr(request, ctx)?;
    
    // 2. Validate against expected type (if any)
    // TODO: Type validation
    
    // 3. Capture continuation state
    let suspension = Suspension {
        continuation: (**continuation).clone(),
        saved_context: ctx.snapshot(),
        role: role.clone(),
        span: *span,
    };
    
    // 4. Record yield in provenance
    ctx.provenance.record_yield(role, &request_value, span);
    
    // 5. Suspend with routing info
    Err(ExecError::Yielded {
        role: role.clone(),
        request: request_value,
        suspension,
    })
}
```

---

## Step 4: Implement Resume Handling

When workflow is resumed (via `PROXY_RESUME` or similar):

```rust
// In execution engine
pub fn resume_workflow(
    &mut self,
    suspension: Suspension,
    response: Value,
) -> Result<Value, ExecError> {
    // Restore context
    let mut ctx = suspension.saved_context.restore();
    
    // Bind response to continuation pattern
    // Execute continuation
    self.execute_workflow(&suspension.continuation, &mut ctx)
}
```

---

## Step 5: Integration with Proxy System

Ensure yield works with existing proxy infrastructure:

```rust
// Proxy workflow should be able to yield to another role
// Handler lookup by role name
// Mailbox routing
```

---

## Step 6: Run Tests

```bash
cargo test --package ash-interp yield_execution -v
cargo test --package ash-interp proxy  # ensure proxy tests still pass
```

---

## Step 7: Commit

```bash
git add crates/ash-interp/src/execute.rs
git add crates/ash-interp/src/context.rs  # if modified
git add crates/ash-interp/tests/yield_execution_tests.rs
git commit -m "feat: implement YIELD runtime execution (TASK-243)

- Evaluate request expression
- Capture continuation with context snapshot
- Suspend workflow with routing to role
- Resume with response value
- Integration tests for yield/resume cycle
- Works with existing proxy infrastructure"
```

---

## Step 8: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-243 implementation"
  context: |
    Files to verify:
    - crates/ash-interp/src/execute.rs (Yield execution)
    - crates/ash-interp/src/context.rs (Suspension, snapshot)
    - crates/ash-interp/tests/yield_execution_tests.rs
    
    Spec reference: SPEC-023 execution semantics
    Requirements:
    1. Request expression evaluated
    2. Continuation captured
    3. Workflow suspends (not errors)
    4. Can resume with response
    5. Continuation executes after resume
    6. Provenance recorded
    
    Run and report:
    1. cargo test --package ash-interp yield
    2. cargo test --package ash-interp proxy
    3. cargo clippy --package ash-interp --all-targets --all-features -- -D warnings
    4. cargo fmt --check --package ash-interp
    5. Check integration with proxy tests
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] Failing tests written
- [ ] Yield execution implemented
- [ ] Resume handling implemented
- [ ] Proxy integration verified
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 10
**Blocked by:** TASK-242 (Yield lowering)
**Blocks:** TASK-244 (PROXY_RESUME)
