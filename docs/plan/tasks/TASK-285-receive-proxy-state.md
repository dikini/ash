# TASK-285: Fix Proxy State Dropped in Receive Paths

## Status: 📝 Planned

## Description

Fix the critical issue where receive paths also drop proxy state, preventing proxy handlers from reliably yielding from receive arms. This is a direct spec break because proxy workflows are receive-driven.

## Specification Reference

- SPEC-023: Proxy Workflows Specification
- SPEC-013: Streams and Behaviours Specification
- SPEC-004: Runtime Semantics Specification

## Dependencies

- ✅ TASK-239: Proxy workflow runtime implementation
- ✅ TASK-170: End-to-end receive execution
- TASK-284: Proxy state recursive paths (related)

## Critical File Locations

- `crates/ash-interp/src/execute.rs:682` - receive execution dropping proxy state
- `crates/ash-interp/src/execute_stream.rs:79` - stream execution dropping proxy state
- `crates/ash-interp/src/execute_stream.rs:110` - stream execution dropping proxy state

## Requirements

### Functional Requirements

1. `receive` construct must preserve `proxy_registry` through all execution paths
2. `receive` construct must preserve `suspended_yields` through all execution paths
3. Proxy handlers must be able to yield from within receive arms
4. Stream processing must support suspend/resume semantics

### Current State (Broken)

**File:** `crates/ash-interp/src/execute.rs:682`

```rust
Step::Receive { stream, arms } => {
    // ... setup receive context ...
    
    for arm in arms {
        // Proxy state is dropped here!
        self.execute_steps(
            &arm.body,
            ctx,
            state,
            None, // proxy_registry dropped!
            None, // suspended_yields dropped!
            capability_resolver,
        )?;
    }
}
```

**File:** `crates/ash-interp/src/execute_stream.rs:79`

```rust
// Stream execution also drops proxy state
self.process_stream_event(
    event,
    state,
    None, // proxy_registry dropped!
    None, // suspended_yields dropped!
)?;
```

This breaks SPEC-023 because:
- Proxy workflows are designed to be receive-driven
- A proxy handler cannot yield during stream processing
- The receive/resume cycle breaks proxy state

### Target State (Fixed)

```rust
Step::Receive { stream, arms } => {
    // ... setup receive context ...
    
    for arm in arms {
        // FIX: Preserve proxy state
        self.execute_steps(
            &arm.body,
            ctx,
            state,
            proxy_registry,    // Preserved
            suspended_yields,  // Preserved
            capability_resolver,
        )?;
    }
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-interp/tests/receive_proxy_state_test.rs`

```rust
//! Tests for proxy state preservation in receive paths

use ash_interp::{Interpreter, RuntimeState};
use ash_engine::Engine;

#[test]
fn test_yield_from_receive_arm() {
    let engine = Engine::builder()
        .with_proxy_registry(test_registry())
        .with_stream_provider(test_stream())
        .build();
    
    // Workflow that yields from within a receive arm
    let workflow = r#"
        workflow test {
            receive {
                on event(data) => {
                    yield to user for approval of data;
                    act process(data);
                }
            }
        }
    "#;
    
    // Send event to trigger receive arm
    engine.send_event(test_event("test-data"));
    
    // Should yield from within the receive handler
    let result1 = engine.run(workflow).unwrap();
    assert!(result1.is_suspended());
    assert_eq!(result1.suspend_reason(), "approval");
    
    // Resume should continue within the receive arm
    let result2 = engine.resume(result1.suspend_handle(), approval()).unwrap();
    assert!(result2.is_complete());
    assert!(processed("test-data"));
}

#[test]
fn test_multiple_yields_in_receive() {
    let engine = Engine::builder()
        .with_proxy_registry(test_registry())
        .build();
    
    let workflow = r#"
        workflow test {
            receive {
                on start() => {
                    yield to user for confirmation;
                    act phase1();
                    yield to user for approval;
                    act phase2();
                }
            }
        }
    "#;
    
    engine.send_event(start_event());
    
    // First yield
    let r1 = engine.run(workflow).unwrap();
    assert!(r1.is_suspended());
    
    // Resume to second yield
    let r2 = engine.resume(r1.suspend_handle(), confirmation()).unwrap();
    assert!(r2.is_suspended());
    
    // Resume to completion
    let r3 = engine.resume(r2.suspend_handle(), approval()).unwrap();
    assert!(r3.is_complete());
}

#[test]
fn test_nested_receive_with_yield() {
    let engine = Engine::builder()
        .with_proxy_registry(test_registry())
        .build();
    
    let workflow = r#"
        workflow test {
            receive {
                on outer() => {
                    receive {
                        on inner() => {
                            yield to user;
                        }
                    }
                }
            }
        }
    "#;
    
    engine.send_event(outer_event());
    engine.send_event(inner_event());
    
    let r1 = engine.run(workflow).unwrap();
    assert!(r1.is_suspended());
    
    let r2 = engine.resume(r1.suspend_handle(), null()).unwrap();
    assert!(r2.is_complete());
}

#[test]
fn test_yield_in_observe_with_receive() {
    let engine = Engine::builder()
        .with_proxy_registry(test_registry())
        .build();
    
    let workflow = r#"
        workflow test {
            observe {
                receive {
                    on signal() => {
                        yield to user for decision;
                    }
                }
            } on sensor_stream
        }
    "#;
    
    engine.send_event(signal_event());
    
    let r1 = engine.run(workflow).unwrap();
    assert!(r1.is_suspended());
    
    let r2 = engine.resume(r1.suspend_handle(), decision()).unwrap();
    assert!(r2.is_complete());
}

proptest! {
    #[test]
    fn receive_proxy_state_survives_events(event_count in 1usize..100) {
        // Property: proxy state must survive multiple receive events
        // and still be resumable from any handler
    }
}
```

### Step 2: Fix Receive Execution

**File:** `crates/ash-interp/src/execute.rs`

```rust
impl Interpreter {
    fn execute_receive(
        &mut self,
        arms: &[ReceiveArm],
        ctx: &mut Context,
        state: &mut RuntimeState,
        proxy_registry: Option<&ProxyRegistry>,  // Thread through
        suspended_yields: Option<&SuspendedYields>, // Thread through
        capability_resolver: &CapabilityResolver,
    ) -> Result<StepResult, InterpError> {
        // ... match event to arm ...
        
        // FIX: Pass proxy state to arm execution
        self.execute_steps(
            &matched_arm.body,
            ctx,
            state,
            proxy_registry,      // WAS: None
            suspended_yields,    // WAS: None
            capability_resolver,
        )
    }
}
```

### Step 3: Fix Stream Execution

**File:** `crates/ash-interp/src/execute_stream.rs`

```rust
impl StreamExecutor {
    fn process_event(
        &mut self,
        event: StreamEvent,
        state: &mut RuntimeState,
        proxy_registry: Option<&ProxyRegistry>,  // Add parameter
        suspended_yields: Option<&SuspendedYields>, // Add parameter
    ) -> Result<EventResult, InterpError> {
        // ... process event ...
        
        // FIX: Pass proxy state
        self.execute_handler(
            handler,
            event,
            state,
            proxy_registry,      // WAS: None
            suspended_yields,    // WAS: None
        )
    }
}
```

### Step 4: Add Stream State Container

**File:** `crates/ash-interp/src/stream_state.rs`

```rust
/// Carries proxy state through stream processing
pub struct StreamContext<'a> {
    proxy_registry: Option<&'a ProxyRegistry>,
    suspended_yields: Option<&'a SuspendedYields>,
    capability_resolver: &'a CapabilityResolver,
}

impl<'a> StreamContext<'a> {
    pub fn new(
        proxy_registry: Option<&'a ProxyRegistry>,
        suspended_yields: Option<&'a SuspendedYields>,
        capability_resolver: &'a CapabilityResolver,
    ) -> Self {
        Self {
            proxy_registry,
            suspended_yields,
            capability_resolver,
        }
    }
    
    pub fn proxy_registry(&self) -> Option<&ProxyRegistry> {
        self.proxy_registry
    }
    
    pub fn suspended_yields(&self) -> Option<&SuspendedYields> {
        self.suspended_yields
    }
}
```

## Verification Steps

- [ ] `cargo test -p ash-interp --test receive_proxy_state_test` passes
- [ ] `cargo test -p ash-engine` passes (integration tests)
- [ ] Stream tests verify proxy state preservation
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- Working proxy yields from receive handlers
- SPEC-023 compliance for receive-driven proxy workflows

Required by:
- Full proxy workflow feature completion

## Notes

**Critical Issue**: This is a SPEC-023 compliance violation. Proxy workflows cannot function as designed without this fix.

**Relationship to TASK-284**: This task specifically focuses on receive/stream paths, while TASK-284 handles general recursive execution paths. Both are required for complete proxy state preservation.

**Testing Challenge**: Stream/receive tests require async event injection. Use test doubles for stream providers.
