# TASK-267: Yield Routing by Role

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Implement runtime routing of `yield role(R)` to appropriate role handlers.

**Spec Reference:** SPEC-023

**File Locations:**
- Modify: `crates/ash-interp/src/yield_routing.rs` (or create)
- Modify: `crates/ash-interp/src/execute.rs` (yield execution)
- Test: `crates/ash-interp/tests/yield_routing_tests.rs`

---

## Background

When a workflow executes:
```ash
yield role(ai_assistant) Request { data: x }
resume result : Response { ... }
```

The runtime must:
1. Look up the current handler for role `ai_assistant`
2. Route the request to that handler
3. Suspend the yielding workflow
4. On response, resume with result bound

---

## Step 1: Create Yield Routing Module

Create `crates/ash-interp/src/yield_routing.rs`:

```rust
use ash_core::*;
use std::collections::HashMap;

/// Routes yields to role handlers
pub struct YieldRouter {
    /// Role name -> current handler workflow ID
    handlers: HashMap<String, WorkflowId>,
    /// Pending yields awaiting response
    pending: HashMap<YieldId, PendingYield>,
}

#[derive(Debug, Clone)]
pub struct PendingYield {
    pub yield_id: YieldId,
    pub caller: WorkflowId,
    pub role: String,
    pub request: Value,
    pub continuation: CoreWorkflow,
    pub saved_context: ExecutionContext,
}

impl YieldRouter {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            pending: HashMap::new(),
        }
    }
    
    /// Register a workflow as handler for a role
    pub fn register_handler(&mut self, role: impl Into<String>, workflow: WorkflowId) {
        self.handlers.insert(role.into(), workflow);
    }
    
    /// Route a yield to the appropriate handler
    pub fn route_yield(
        &mut self,
        caller: WorkflowId,
        role: &str,
        request: Value,
        continuation: CoreWorkflow,
        context: ExecutionContext,
    ) -> Result<YieldId, YieldError> {
        let handler = self.handlers.get(role)
            .ok_or(YieldError::NoHandlerForRole(role.to_string()))?;
        
        let yield_id = YieldId::new();
        
        let pending = PendingYield {
            yield_id: yield_id.clone(),
            caller,
            role: role.to_string(),
            request: request.clone(),
            continuation,
            saved_context: context,
        };
        
        self.pending.insert(yield_id.clone(), pending);
        
        // Send to handler
        self.send_to_handler(*handler, yield_id, request)?;
        
        Ok(yield_id)
    }
    
    fn send_to_handler(
        &self,
        handler: WorkflowId,
        yield_id: YieldId,
        request: Value,
    ) -> Result<(), YieldError> {
        // Placeholder: Actual implementation queues to handler's mailbox
        // or directly invokes if handler is ready
        Ok(())
    }
    
    /// Resume a workflow with a response
    pub fn resume_with_response(
        &mut self,
        yield_id: YieldId,
        response: Value,
    ) -> Result<ResumeResult, YieldError> {
        let pending = self.pending.remove(&yield_id)
            .ok_or(YieldError::UnknownYield(yield_id))?;
        
        // Restore context
        let mut ctx = pending.saved_context;
        
        // Execute continuation
        // The continuation is a Match on the response
        let result = match &pending.continuation {
            CoreWorkflow::Match { scrutinee, arms, .. } => {
                // Bind response and match
                ctx.bind("__yield_response".to_string(), response);
                self.execute_match(&mut ctx, arms)
            }
            _ => {
                // Direct continuation
                self.execute_workflow(&pending.continuation, &mut ctx)
            }
        };
        
        Ok(ResumeResult {
            caller: pending.caller,
            value: result,
        })
    }
    
    /// Get handler for a role
    pub fn get_handler(&self, role: &str) -> Option<WorkflowId> {
        self.handlers.get(role).copied()
    }
    
    /// Check if a yield is pending
    pub fn is_pending(&self, yield_id: &YieldId) -> bool {
        self.pending.contains_key(yield_id)
    }
}

#[derive(Debug, Clone)]
pub struct YieldId(uuid::Uuid);

impl YieldId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

#[derive(Debug)]
pub struct ResumeResult {
    pub caller: WorkflowId,
    pub value: Result<Value, ExecError>,
}

#[derive(Debug)]
pub enum YieldError {
    NoHandlerForRole(String),
    UnknownYield(YieldId),
    HandlerBusy,
}
```

---

## Step 2: Write Failing Tests

```rust
// crates/ash-interp/tests/yield_routing_tests.rs
use ash_interp::yield_routing::*;
use ash_core::*;

#[test]
fn test_register_and_route() {
    let mut router = YieldRouter::new();
    
    let handler_id = WorkflowId::new();
    router.register_handler("ai_assistant", handler_id);
    
    let caller_id = WorkflowId::new();
    let request = json!({ "data": 42 });
    let continuation = CoreWorkflow::Done;
    let context = ExecutionContext::new();
    
    let yield_id = router.route_yield(
        caller_id,
        "ai_assistant",
        request,
        continuation,
        context,
    ).unwrap();
    
    assert!(router.is_pending(&yield_id));
}

#[test]
fn test_route_to_unknown_role_fails() {
    let mut router = YieldRouter::new();
    // No handler registered
    
    let result = router.route_yield(
        WorkflowId::new(),
        "unknown_role",
        json!({}),
        CoreWorkflow::Done,
        ExecutionContext::new(),
    );
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), YieldError::NoHandlerForRole(_)));
}

#[test]
fn test_resume_continues_execution() {
    let mut router = YieldRouter::new();
    
    // Setup yield
    let handler_id = WorkflowId::new();
    router.register_handler("ai", handler_id);
    
    let caller_id = WorkflowId::new();
    let yield_id = router.route_yield(
        caller_id,
        "ai",
        json!({}),
        CoreWorkflow::Done,
        ExecutionContext::new(),
    ).unwrap();
    
    // Resume
    let response = json!({ "result": "success" });
    let result = router.resume_with_response(yield_id, response);
    
    assert!(result.is_ok());
    let resume_result = result.unwrap();
    assert_eq!(resume_result.caller, caller_id);
}

#[test]
fn test_resume_unknown_yield_fails() {
    let mut router = YieldRouter::new();
    
    let result = router.resume_with_response(
        YieldId::new(),  // Never routed
        json!({}),
    );
    
    assert!(result.is_err());
}

proptest! {
    #[test]
    fn test_yield_id_unique(calls in 1..100usize) {
        // Each yield should get unique ID
    }
}
```

---

## Step 3: Integrate into Yield Execution

```rust
// crates/ash-interp/src/execute.rs
Workflow::Yield {
    role,
    request,
    continuation,
    span,
} => {
    // Evaluate request
    let request_value = eval_expr(request, ctx)?;
    
    // Route to handler
    let yield_id = ctx.yield_router.route_yield(
        ctx.current_workflow_id(),
        role,
        request_value,
        (**continuation).clone(),
        ctx.snapshot(),
    ).map_err(|e| ExecError::YieldFailed(e))?;
    
    // Suspend
    Err(ExecError::Yielded { yield_id })
}
```

---

## Step 4: Integrate into Resume

```rust
// When handler responds
pub fn handle_handler_response(
    &mut self,
    yield_id: YieldId,
    response: Value,
) -> Result<(), ExecError> {
    let resume_result = self.yield_router
        .resume_with_response(yield_id, response)
        .map_err(|e| ExecError::ResumeFailed(e))?;
    
    // Schedule resumed workflow
    self.schedule_workflow(resume_result.caller);
    
    Ok(())
}
```

---

## Step 5: Run Tests

```bash
cargo test --package ash-interp yield_routing -v
```

---

## Step 6: Commit

```bash
git add crates/ash-interp/src/yield_routing.rs
git add crates/ash-interp/tests/yield_routing_tests.rs
git add crates/ash-interp/src/execute.rs
git commit -m "feat: yield routing by role (TASK-267)

- Add YieldRouter for role-based yield routing
- Register workflows as role handlers
- Route yields to current handler
- Track pending yields
- Resume with response continues execution
- Integration with execute.rs yield/resume
- Error handling for unknown roles/yields
- Tests for routing and resumption"
```

---

## Step 7: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-267 implementation"
  context: |
    Files to verify:
    - crates/ash-interp/src/yield_routing.rs
    - crates/ash-interp/tests/yield_routing_tests.rs
    - crates/ash-interp/src/execute.rs
    
    Spec reference: SPEC-023
    Requirements:
    1. Handlers register for roles
    2. Yields routed to registered handler
    3. Unknown roles produce error
    4. Yielding workflow suspended
    5. Resume continues with response
    6. Unknown yield produces error
    7. Integration with execute loop
    
    Run and report:
    1. cargo test --package ash-interp yield_routing
    2. cargo clippy --package ash-interp --all-targets --all-features -- -D warnings
    3. cargo fmt --check --package ash-interp
    4. Test full yield-resume cycle
    5. Test multiple concurrent yields
    6. Test role handler switch
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] YieldRouter created
- [ ] Failing tests written
- [ ] Handler registration
- [ ] Yield routing
- [ ] Pending yield tracking
- [ ] Resume with response
- [ ] Execute integration
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 10
**Blocked by:** TASK-266
**Blocks:** Phase 46.4 (Agent Harness - optional)

**Note:** This completes the core Phase 46 implementation. Phase 46.4 (Agent Harness) is optional and depends on project priorities.
