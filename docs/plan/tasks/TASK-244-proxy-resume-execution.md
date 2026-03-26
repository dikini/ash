# TASK-244: Implement PROXY_RESUME Runtime

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Implement `PROXY_RESUME` runtime operation to complete proxy/yield end-to-end execution.

**Spec Reference:** SPEC-023 (Proxy Workflows), Resume semantics

**File Locations:**
- Modify: `crates/ash-interp/src/execute.rs` (add resume handling)
- Modify: `crates/ash-interp/src/lib.rs` (public API)
- Test: `crates/ash-interp/tests/proxy_resume_tests.rs` (create)

---

## Background

The audit found `PROXY_RESUME` is explicitly unimplemented. With TASK-243's yield suspension, we need the corresponding resume mechanism.

Target semantics:
1. External agent or handler completes processing
2. Calls resume with response value
3. Suspended workflow continues with response bound
4. Continuation executes to completion or next yield

---

## Step 1: Understand Suspension State

Review TASK-243's Suspension structure:

```bash
grep -n "struct Suspension" crates/ash-interp/src/
grep -n "Yielded" crates/ash-interp/src/error.rs
```

Expected:
```rust
pub struct Suspension {
    pub continuation: CoreWorkflow,
    pub saved_context: ContextSnapshot,
    pub role: String,
    pub span: Span,
}

pub enum ExecError {
    // ...
    Yielded {
        role: String,
        request: Value,
        suspension: Suspension,
    },
}
```

---

## Step 2: Write Failing Tests

```rust
// crates/ash-interp/tests/proxy_resume_tests.rs
use ash_interp::*;
use ash_core::*;

#[test]
fn test_proxy_resume_continues_workflow() {
    // 1. Start workflow that yields
    // 2. Get suspension
    // 3. Resume with response
    // 4. Workflow completes
}

#[test]
fn test_proxy_resume_binds_response() {
    // Response value should be bound in continuation scope
}

#[test]
fn test_proxy_resume_multiple_yields() {
    // Workflow with multiple yields should resume sequentially
}

proptest! {
    #[test]
    fn test_resume_response_roundtrip(response in arbitrary_value()) {
        // Property: resumed response equals what continuation receives
    }
}
```

---

## Step 3: Design Resume API

Add public resume method:

```rust
// crates/ash-interp/src/lib.rs or engine.rs
impl Interpreter {
    /// Resume a suspended workflow with a response
    pub fn resume(
        &mut self,
        suspension: Suspension,
        response: Value,
    ) -> Result<WorkflowResult, ExecError> {
        // Restore context from snapshot
        let mut ctx = suspension.saved_context.restore();
        
        // Bind response to a special variable or pattern
        // The continuation expects the response via pattern matching
        
        // Execute continuation
        let result = self.execute_workflow(&suspension.continuation, &mut ctx)?;
        
        Ok(WorkflowResult {
            value: result,
            context: ctx,
        })
    }
}
```

---

## Step 4: Implement Response Binding

The continuation is a workflow that pattern-matches on response. Ensure the response is available:

```rust
// Option 1: Store in context bindings
ctx.bindings.insert("__yield_response".to_string(), response);

// Option 2: Pass directly if continuation expects it
// The continuation should be a Match on the response
```

Check SPEC-023 for the expected pattern:
```ash
yield role(ai) Request { ... }
resume result : Response {
    Response { data } => { ... }  // Pattern match on response
}
```

The continuation IS the resume clause body, with `result` bound to the response.

---

## Step 5: Implement Resume Execution

```rust
// In Interpreter
pub fn resume_with_response(
    &mut self,
    suspension: Suspension,
    response: Value,
) -> Result<Value, ExecError> {
    let mut ctx = suspension.saved_context.restore();
    
    // Record resume in provenance
    ctx.provenance.record_resume(&suspension.role, &response);
    
    // The continuation should already expect the response
    // If continuation is a Match, execute it
    match &suspension.continuation {
        Workflow::Match { scrutinee, arms, .. } => {
            // scrutinee should reference the response binding
            // Execute match
            self.execute_match(response, arms, &mut ctx)
        }
        _ => {
            // Direct execution
            self.execute_workflow(&suspension.continuation, &mut ctx)
        }
    }
}
```

---

## Step 6: Integration with Role Handler

When a role handler processes a yield:

```rust
// Handler receives Yielded error
// Processes request
// Calls resume_with_response when done
impl RoleHandler {
    fn handle_yield(&self, suspension: Suspension, request: Value) {
        // Process...
        let response = self.process(request);
        
        // Resume original workflow
        self.interpreter.resume_with_response(suspension, response);
    }
}
```

---

## Step 7: Run Tests

```bash
cargo test --package ash-interp proxy_resume -v
cargo test --package ash-interp yield  # ensure yield tests still pass
```

---

## Step 8: Commit

```bash
git add crates/ash-interp/src/execute.rs
git add crates/ash-interp/src/lib.rs
git add crates/ash-interp/tests/proxy_resume_tests.rs
git commit -m "feat: implement PROXY_RESUME runtime execution (TASK-244)

- Add public resume API to Interpreter
- Restore context from suspension snapshot
- Bind response to continuation pattern
- Execute continuation after resume
- Integration with role handler lifecycle
- Property tests for resume roundtrip"
```

---

## Step 9: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-244 implementation"
  context: |
    Files to verify:
    - crates/ash-interp/src/execute.rs (resume handling)
    - crates/ash-interp/src/lib.rs (public resume API)
    - crates/ash-interp/tests/proxy_resume_tests.rs
    
    Spec reference: SPEC-023 resume semantics
    Requirements:
    1. Resume accepts suspension and response
    2. Context restored from snapshot
    3. Response bound to continuation
    4. Continuation executes correctly
    5. Multiple yields/resumes work sequentially
    6. Provenance recorded
    
    Run and report:
    1. cargo test --package ash-interp proxy_resume
    2. cargo test --package ash-interp yield
    3. cargo clippy --package ash-interp --all-targets --all-features -- -D warnings
    4. cargo fmt --check --package ash-interp
    5. End-to-end yield+resume test
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] Failing tests written
- [ ] Resume API implemented
- [ ] Response binding works
- [ ] Continuation executes correctly
- [ ] Integration with role handlers
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 8
**Blocked by:** TASK-243 (YIELD execution)
**Blocks:** None (completes proxy/yield implementation)
