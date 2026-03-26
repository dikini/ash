# TASK-287: Implement Role Runtime Semantics

## Status: 📝 Planned

## Description

Fix the issue where role runtime semantics are largely stubbed in execution. `Workflow::Check` and `Workflow::Oblig` are no-ops, and output operations hardcode `Role::new("system")` instead of consulting role context. The project has role-enforcement machinery, but it is not wired into the interpreter.

## Specification Reference

- SPEC-019: Role Runtime Semantics Specification
- SPEC-022: Workflow Typing with Obligations
- SPEC-004: Runtime Semantics Specification

## Dependencies

- ✅ TASK-235: SPEC-019 role runtime semantics
- ✅ TASK-236: Role runtime enforcement implementation
- ✅ TASK-219: Align runtime role approval contract
- ✅ TASK-248: Role obligation discharge

## Critical File Locations

- `crates/ash-interp/src/execute.rs:583` - Workflow::Check is no-op
- `crates/ash-interp/src/execute.rs:604` - Workflow::Oblig is no-op
- `crates/ash-interp/src/execute.rs:631` - output hardcodes system role
- `crates/ash-interp/src/execute.rs:657` - output hardcodes system role
- `crates/ash-interp/src/context.rs:88` - role context not consulted

## Requirements

### Functional Requirements

1. `Workflow::Check` must verify the current role is approved for the capability
2. `Workflow::Oblig` must register obligations in the runtime obligation tracker
3. Output operations must use the workflow's current role, not hardcoded "system"
4. Role transitions must be tracked through the execution context
5. Role-based capability filtering must be enforced at runtime

### Current State (Broken)

**File:** `crates/ash-interp/src/execute.rs:583`

```rust
Step::Check { capability } => {
    // NO-OP: Does not actually check role!
    Ok(StepResult::Continue)
}
```

**File:** `crates/ash-interp/src/execute.rs:604`

```rust
Step::Oblig { obligation } => {
    // NO-OP: Does not register obligation!
    Ok(StepResult::Continue)
}
```

**File:** `crates/ash-interp/src/execute.rs:631`

```rust
Step::Output { data, .. } => {
    // Hardcoded role - ignores workflow role context!
    let role = Role::new("system");
    self.perform_output(data, role)?;
    Ok(StepResult::Continue)
}
```

### Target State (Fixed)

```rust
Step::Check { capability } => {
    // FIX: Actually check role approval
    let current_role = ctx.current_role();
    
    if !state.role_registry().is_approved(current_role, capability) {
        return Err(InterpError::RoleError(
            RoleError::NotApproved {
                role: current_role.clone(),
                capability: capability.clone(),
            }
        ));
    }
    
    Ok(StepResult::Continue)
}

Step::Oblig { obligation } => {
    // FIX: Register obligation in runtime tracker
    state.obligation_tracker().register(
        obligation.clone(),
        ctx.current_role(),
    );
    
    Ok(StepResult::Continue)
}

Step::Output { data, .. } => {
    // FIX: Use actual role from context
    let current_role = ctx.current_role();
    self.perform_output(data, current_role)?;
    Ok(StepResult::Continue)
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-interp/tests/role_runtime_semantics_test.rs`

```rust
//! Tests for role runtime semantics

use ash_interp::{Interpreter, RuntimeState};
use ash_engine::{Engine, RoleError};

#[test]
fn test_check_approved_role_succeeds() {
    let engine = Engine::builder()
        .with_role(Role::new("logger")
            .approve_capability(Capability::output()))
        .build();
    
    let workflow = r#"
        workflow test plays role(logger) {
            check output;
            act log("message");
        }
    "#;
    
    let result = engine.run(workflow);
    assert!(result.is_ok());
}

#[test]
fn test_check_unapproved_role_fails() {
    let engine = Engine::builder()
        .with_role(Role::new("logger")
            .approve_capability(Capability::output()))
        .build();
    
    let workflow = r#"
        workflow test plays role(logger) {
            check admin_access;  // Not approved for logger role
        }
    "#;
    
    let result = engine.run(workflow);
    
    assert!(matches!(
        result,
        Err(RoleError::NotApproved {
            role,
            capability,
        }) if role.name() == "logger" && capability.name() == "admin_access"
    ));
}

#[test]
fn test_oblige_registers_obligation() {
    let engine = Engine::builder()
        .with_obligation_tracker(TestObligationTracker::new())
        .build();
    
    let workflow = r#"
        workflow test {
            oblige audit_trail;
            act do_something();
            check audit_trail;
        }
    "#;
    
    let result = engine.run(workflow);
    assert!(result.is_ok());
    
    // Verify obligation was tracked
    let tracker = engine.obligation_tracker();
    assert!(tracker.was_registered("audit_trail"));
    assert!(tracker.was_satisfied("audit_trail"));
}

#[test]
fn test_oblige_without_check_fails() {
    let engine = Engine::builder()
        .with_obligation_tracker(TestObligationTracker::new())
        .build();
    
    let workflow = r#"
        workflow test {
            oblige audit_trail;
            act do_something();
            // Missing check audit_trail!
        }
    "#;
    
    let result = engine.run(workflow);
    
    assert!(matches!(
        result,
        Err(RoleError::UnsatisfiedObligations { .. })
    ));
}

#[test]
fn test_output_uses_workflow_role() {
    let engine = Engine::builder()
        .with_role(Role::new("user_process")
            .approve_capability(Capability::output()))
        .with_output_auditor(TestOutputAuditor::new())
        .build();
    
    let workflow = r#"
        workflow test plays role(user_process) {
            act log("from user_process");
        }
    "#;
    
    let result = engine.run(workflow);
    assert!(result.is_ok());
    
    // Verify output was performed with correct role
    let auditor = engine.output_auditor();
    let records = auditor.records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].role.name(), "user_process");
    assert_ne!(records[0].role.name(), "system");
}

#[test]
fn test_role_transition_during_execution() {
    let engine = Engine::builder()
        .with_role(Role::new("initializer")
            .approve_capability(Capability::read_config()))
        .with_role(Role::new("processor")
            .approve_capability(Capability::write_output()))
        .build();
    
    let workflow = r#"
        workflow test plays role(initializer) {
            check read_config;
            let config = read_config();
            
            assume role(processor);
            
            check write_output;
            write_output(config);
        }
    "#;
    
    let result = engine.run(workflow);
    assert!(result.is_ok());
}

#[test]
fn test_role_isolation_enforcement() {
    let engine = Engine::builder()
        .with_role(Role::new("reader")
            .approve_capability(Capability::read_data()))
        .with_role(Role::new("writer")
            .approve_capability(Capability::write_data()))
        .build();
    
    // Workflow tries to write but only has reader role
    let workflow = r#"
        workflow test plays role(reader) {
            check write_data;  // Should fail - not approved for reader
        }
    "#;
    
    let result = engine.run(workflow);
    assert!(result.is_err());
}

#[test]
fn test_capability_filtering_by_role() {
    let engine = Engine::builder()
        .with_role(Role::new("limited")
            .approve_capability(Capability::read_stream("public")))
        .with_stream_provider(test_stream("public"))
        .with_stream_provider(test_stream("private"))
        .build();
    
    let workflow = r#"
        workflow test plays role(limited) {
            receive {
                on public(data) => act handle(data)
            }
        }
    "#;
    
    let result = engine.run(workflow);
    assert!(result.is_ok());
    
    // Should NOT be able to access private stream
    let workflow_private = r#"
        workflow test plays role(limited) {
            receive {
                on private(data) => act handle(data)
            }
        }
    "#;
    
    let result = engine.run(workflow_private);
    assert!(result.is_err());
}

proptest! {
    #[test]
    fn role_capability_soundness(
        role_capabilities in vec(capability_strategy(), 0..10),
        requested_capability in capability_strategy()
    ) {
        // Property: role can only use approved capabilities
    }
}
```

### Step 2: Implement Check Execution

**File:** `crates/ash-interp/src/execute.rs`

```rust
impl Interpreter {
    fn execute_check(
        &mut self,
        capability: &Capability,
        ctx: &Context,
        state: &RuntimeState,
    ) -> Result<StepResult, InterpError> {
        let current_role = ctx.current_role();
        
        // Check if role is approved for this capability
        if !state.role_registry().is_approved(current_role, capability) {
            return Err(InterpError::RoleError(
                RoleError::NotApproved {
                    role: current_role.clone(),
                    capability: capability.clone(),
                }
            ));
        }
        
        // Record the check for audit trail
        state.audit_trail().record_role_check(
            current_role,
            capability,
            ctx.current_span(),
        );
        
        Ok(StepResult::Continue)
    }
}
```

### Step 3: Implement Oblig Execution

**File:** `crates/ash-interp/src/execute.rs`

```rust
impl Interpreter {
    fn execute_oblige(
        &mut self,
        obligation: &Obligation,
        ctx: &Context,
        state: &mut RuntimeState,
    ) -> Result<StepResult, InterpError> {
        let current_role = ctx.current_role();
        
        // Register obligation with current role context
        state.obligation_tracker_mut().register(
            obligation.clone(),
            ObligationContext {
                declared_by: current_role.clone(),
                span: ctx.current_span(),
            },
        );
        
        Ok(StepResult::Continue)
    }
}
```

### Step 4: Fix Role Context in Output

**File:** `crates/ash-interp/src/execute.rs`

```rust
impl Interpreter {
    fn execute_output(
        &mut self,
        data: &Expr,
        ctx: &mut Context,
        state: &mut RuntimeState,
    ) -> Result<StepResult, InterpError> {
        let value = self.eval_expr(data, ctx, state)?;
        
        // FIX: Use actual role from context instead of hardcoded "system"
        let current_role = ctx.current_role();
        
        // Verify role has output capability
        self.execute_check(&Capability::output(), ctx, state)?;
        
        // Perform output with role attribution
        state.output().write(
            OutputRecord::new(value)
                .with_role(current_role.clone())
                .with_timestamp(now()),
        )?;
        
        Ok(StepResult::Continue)
    }
}
```

### Step 5: Update Context with Role Tracking

**File:** `crates/ash-interp/src/context.rs`

```rust
pub struct Context {
    // ... existing fields ...
    role_stack: Vec<Role>,
}

impl Context {
    pub fn current_role(&self) -> &Role {
        // Return innermost role from stack, or default
        self.role_stack.last()
            .unwrap_or(&Role::default())
    }
    
    pub fn push_role(&mut self, role: Role) {
        self.role_stack.push(role);
    }
    
    pub fn pop_role(&mut self) -> Option<Role> {
        self.role_stack.pop()
    }
}
```

## Verification Steps

- [ ] `cargo test -p ash-interp --test role_runtime_semantics_test` passes
- [ ] `cargo test -p ash-engine` passes (integration)
- [ ] Role isolation tests verify security boundaries
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- Working role runtime semantics
- SPEC-019 compliance

Required by:
- Full role-based access control

## Notes

**Security Impact**: This is a security-critical fix. Currently, role-based access control is not enforced at runtime.

**Architecture Note**: The role context must be threaded through all execution paths, similar to proxy state in TASK-284.

**Migration Path**: Existing workflows that rely on implicit "system" role may need explicit role declarations.
