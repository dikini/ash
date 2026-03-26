# TASK-286: Add Capability-Policy Enforcement to Receive

## Status: 📝 Planned

## Description

Fix the compliance gap where `receive` bypasses capability-policy enforcement. Currently, `observe`, `set`, and `send` run policy checks, but `receive` does not. This is a clear compliance gap against the capability matrix/runtime behavior specs.

## Specification Reference

- SPEC-017: Capability Integration Specification
- SPEC-013: Streams and Behaviours Specification
- SPEC-004: Runtime Semantics Specification

## Dependencies

- ✅ TASK-170: End-to-end receive execution
- ✅ TASK-266: Constraint enforcement
- ✅ TASK-110: Policy evaluation for input/output

## Critical File Locations

- `crates/ash-interp/src/execute_stream.rs:44` - receive without policy check
- `crates/ash-interp/src/execute_observe.rs:81` - observe with policy check (reference)
- `crates/ash-interp/src/execute_set.rs:62` - set with policy check (reference)
- `crates/ash-interp/src/exec_send.rs:63` - send with policy check (reference)

## Requirements

### Functional Requirements

1. `receive` must check capability requirements before executing
2. `receive` must enforce policy constraints on stream access
3. `receive` must validate the workflow has the required stream capability
4. `receive` failure should produce appropriate capability errors

### Current State (Broken)

**File:** `crates/ash-interp/src/execute_stream.rs:44`

```rust
// Stream execution without capability check
fn execute_receive(
    &mut self,
    receive: &Receive,
    state: &mut RuntimeState,
) -> Result<ReceiveResult, InterpError> {
    // No capability check here!
    
    let stream = state.get_stream(&receive.stream_name)?;
    
    // Process receive arms...
}
```

**Reference - observe with check (correct):**

```rust
// crates/ash-interp/src/execute_observe.rs:81
fn execute_observe(
    &mut self,
    observe: &Observe,
    state: &mut RuntimeState,
) -> Result<ObserveResult, InterpError> {
    // Capability check BEFORE execution
    self.capability_resolver.check(
        &Capability::ReadStream(observe.stream_name.clone()),
        state.effective_capabilities(),
    )?;
    
    // ... proceed with observe ...
}
```

### Target State (Fixed)

```rust
fn execute_receive(
    &mut self,
    receive: &Receive,
    state: &mut RuntimeState,
) -> Result<ReceiveResult, InterpError> {
    // FIX: Add capability check
    self.capability_resolver.check(
        &Capability::ReceiveStream(receive.stream_name.clone()),
        state.effective_capabilities(),
    )?;
    
    // Also check policy constraints
    self.policy_engine.evaluate(
        &PolicyContext::receive(&receive.stream_name),
        state,
    )?;
    
    let stream = state.get_stream(&receive.stream_name)?;
    
    // Process receive arms...
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-interp/tests/receive_capability_enforcement_test.rs`

```rust
//! Tests for capability-policy enforcement in receive

use ash_interp::{Interpreter, RuntimeState};
use ash_engine::{Engine, CapabilityError};

#[test]
fn test_receive_without_capability_fails() {
    let engine = Engine::builder()
        // No receive_stream capability granted
        .with_capabilities(vec![Capability::stdio()])
        .build();
    
    let workflow = r#"
        workflow test {
            receive {
                on event(data) => act log(data)
            }
        }
    "#;
    
    let result = engine.run(workflow);
    
    assert!(matches!(
        result,
        Err(CapabilityError::MissingCapability { 
            required: Capability::ReceiveStream(_),
            ..
        })
    ));
}

#[test]
fn test_receive_with_capability_succeeds() {
    let engine = Engine::builder()
        .with_capabilities(vec![
            Capability::receive_stream("events"),
        ])
        .with_stream_provider(test_stream("events"))
        .build();
    
    let workflow = r#"
        workflow test {
            receive {
                on event(data) => act log(data)
            }
        }
    "#;
    
    engine.send_event(test_event("hello"));
    
    let result = engine.run(workflow);
    assert!(result.is_ok());
    assert!(logs_contain("hello"));
}

#[test]
fn test_receive_policy_violation_blocked() {
    let engine = Engine::builder()
        .with_capabilities(vec![
            Capability::receive_stream("sensitive"),
        ])
        .with_policy(Policy::deny_sensitive_streams())
        .build();
    
    let workflow = r#"
        workflow test {
            receive {
                on event(data) => act log(data)
            }
        }
    "#;
    
    let result = engine.run(workflow);
    
    assert!(matches!(
        result,
        Err(CapabilityError::PolicyViolation { .. })
    ));
}

#[test]
fn test_receive_constraint_enforcement() {
    let engine = Engine::builder()
        .with_capabilities(vec![
            Capability::receive_stream("events")
                .with_constraint(Constraint::max_rate(100)),
        ])
        .with_stream_provider(test_stream("events"))
        .build();
    
    let workflow = r#"
        workflow test {
            receive {
                on event(data) => act log(data)
            }
        }
    "#;
    
    // Send events exceeding rate limit
    for _ in 0..200 {
        engine.send_event(test_event("flood"));
    }
    
    let result = engine.run(workflow);
    
    assert!(matches!(
        result,
        Err(CapabilityError::ConstraintViolation { 
            constraint: Constraint::MaxRate(_),
            ..
        })
    ));
}

#[test]
fn test_receive_capabilities_composed_with_roles() {
    let engine = Engine::builder()
        .with_role(Role::new("stream_consumer")
            .with_capability(Capability::receive_stream("orders")))
        .with_stream_provider(test_stream("orders"))
        .build();
    
    let workflow = r#"
        workflow test plays role(stream_consumer) {
            receive {
                on order(data) => act process_order(data)
            }
        }
    "#;
    
    engine.send_event(order_event());
    
    let result = engine.run(workflow);
    assert!(result.is_ok());
}

proptest! {
    #[test]
    fn receive_capability_soundness(
        has_capability in any::<bool>(),
        stream_name in "[a-z]+"
    ) {
        // Property: receive succeeds iff capability is present
    }
}
```

### Step 2: Add Capability Check

**File:** `crates/ash-interp/src/execute_stream.rs`

```rust
impl StreamExecutor {
    fn execute_receive(
        &mut self,
        receive: &Receive,
        state: &mut RuntimeState,
        capability_resolver: &CapabilityResolver,
    ) -> Result<ReceiveResult, InterpError> {
        // NEW: Capability check
        let required_cap = Capability::ReceiveStream(receive.stream_name.clone());
        
        if !capability_resolver.check(&required_cap, &state.effective_capabilities())? {
            return Err(InterpError::CapabilityError(
                CapabilityError::MissingCapability {
                    required: required_cap,
                    available: state.effective_capabilities().clone(),
                    operation: "receive".to_string(),
                }
            ));
        }
        
        // NEW: Policy evaluation
        if let Some(policy_engine) = &self.policy_engine {
            policy_engine.evaluate_receive(
                &receive.stream_name,
                state,
            )?;
        }
        
        // Continue with stream processing
        let stream = state.get_stream(&receive.stream_name)?;
        
        // ...
    }
}
```

### Step 3: Add Policy Context for Receive

**File:** `crates/ash-policy/src/context.rs`

```rust
impl PolicyContext {
    pub fn receive(stream_name: &str) -> Self {
        Self {
            operation: Operation::Receive,
            resource: Resource::Stream(stream_name.to_string()),
            constraints: vec![],
        }
    }
}
```

### Step 4: Update Capability Enum

**File:** `crates/ash-core/src/capability.rs`

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Capability {
    // ... existing variants ...
    
    /// Receive from a stream (if no name, any stream)
    ReceiveStream(Option<String>),
}

impl Capability {
    pub fn receive_stream(name: impl Into<String>) -> Self {
        Self::ReceiveStream(Some(name.into()))
    }
    
    /// Check if this capability covers receiving from the given stream
    pub fn covers_receive(&self, stream_name: &str) -> bool {
        matches!(self, 
            Self::ReceiveStream(None) | 
            Self::ReceiveStream(Some(name)) if name == stream_name
        )
    }
}
```

## Verification Steps

- [ ] `cargo test -p ash-interp --test receive_capability_enforcement_test` passes
- [ ] `cargo test -p ash-policy` passes
- [ ] Capability denial tests verify proper error messages
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- Complete capability matrix compliance
- Secure receive execution

Required by:
- Full capability enforcement across all operations

## Notes

**Compliance Gap**: SPEC-017 defines a capability matrix. `receive` is currently an unchecked operation, which violates the security model.

**Capability Design Decision**: 
- `ReceiveStream(None)` = can receive from any stream
- `ReceiveStream(Some(name))` = can only receive from named stream

**Policy Integration**: Policy evaluation happens after capability check but before stream access, allowing for dynamic constraints (rate limiting, time-based access, etc.).
