# TASK-284: Fix Proxy Workflow State Dropped on Recursive Execution Paths

## Status: ✅ Complete

## Description

Fix the critical issue where proxy workflow state is dropped on most recursive execution paths. Many branches in the interpreter recurse with `proxy_registry: None` and `suspended_yields: None`, while `Workflow::Yield` requires them. This violates the proxy handoff/resume model in SPEC-023 for workflows that yield under let, if, observe, check, and similar constructs.

## Specification Reference

- SPEC-023: Proxy Workflows Specification
- SPEC-004: Runtime Semantics Specification

## Dependencies

- ✅ TASK-239: Proxy workflow runtime implementation
- ✅ TASK-243: YIELD runtime execution
- ✅ TASK-244: PROXY_RESUME runtime

## Critical File Locations

- `crates/ash-interp/src/execute.rs:207` - recursive call dropping proxy state
- `crates/ash-interp/src/execute.rs:231` - recursive call dropping proxy state
- `crates/ash-interp/src/execute.rs:346` - recursive call dropping proxy state
- `crates/ash-interp/src/execute.rs:589` - recursive call dropping proxy state
- `crates/ash-interp/src/execute.rs:936` - recursive call dropping proxy state

## Requirements

### Functional Requirements

1. All recursive execution paths must preserve `proxy_registry` state
2. All recursive execution paths must preserve `suspended_yields` state
3. Yield expressions within nested contexts (let, if, observe, check) must be resumable
4. Proxy handoff semantics must be maintained across all control flow constructs

### Current State (Broken)

**File:** `crates/ash-interp/src/execute.rs`

```rust
// Many branches pass None for proxy state:
self.execute_steps(
    &then_branch,
    ctx,
    state,
    None, // proxy_registry dropped!
    None, // suspended_yields dropped!
    capability_resolver,
)?;
```

This breaks SPEC-023's proxy resume model because:
- A workflow yields inside an `if` branch
- The proxy state is lost on recursion
- When resumed, the workflow cannot find its suspended yield context

### Target State (Fixed)

```rust
// All branches must thread proxy state through:
self.execute_steps(
    &then_branch,
    ctx,
    state,
    proxy_registry,    // Preserved
    suspended_yields,  // Preserved
    capability_resolver,
)?;
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-interp/tests/proxy_state_propagation_test.rs`

```rust
//! Tests for proxy state propagation through recursive execution paths

use ash_interp::{Interpreter, RuntimeState};
use ash_engine::Engine;

#[test]
fn test_yield_inside_if_preserves_proxy_state() {
    let engine = Engine::builder()
        .with_proxy_registry(test_registry())
        .build();
    
    // Workflow that yields inside an if branch
    let workflow = r#"
        workflow test {
            let condition = true;
            if condition {
                yield to user for approval;
            }
            act log("continued");
        }
    "#;
    
    // First execution: should yield
    let result1 = engine.run(workflow).unwrap();
    assert!(result1.is_suspended());
    let handle = result1.suspend_handle();
    
    // Resume: should continue past the if block
    let result2 = engine.resume(handle, approval()).unwrap();
    assert!(result2.is_complete());
    assert!(logs_contain("continued"));
}

#[test]
fn test_yield_inside_let_preserves_proxy_state() {
    let engine = Engine::builder()
        .with_proxy_registry(test_registry())
        .build();
    
    let workflow = r#"
        workflow test {
            let result = {
                yield to user for input;
                42
            };
            act log(result);
        }
    "#;
    
    let result1 = engine.run(workflow).unwrap();
    assert!(result1.is_suspended());
    
    let result2 = engine.resume(result1.suspend_handle(), value(42)).unwrap();
    assert!(result2.is_complete());
}

#[test]
fn test_nested_yield_resume() {
    let engine = Engine::builder()
        .with_proxy_registry(test_registry())
        .build();
    
    let workflow = r#"
        workflow test {
            if true {
                if true {
                    yield to user;
                }
            }
            act log("deep resume");
        }
    "#;
    
    let result1 = engine.run(workflow).unwrap();
    assert!(result1.is_suspended());
    
    let result2 = engine.resume(result1.suspend_handle(), null()).unwrap();
    assert!(result2.is_complete());
    assert!(logs_contain("deep resume"));
}

#[test]
fn test_yield_in_observe_block() {
    let engine = Engine::builder()
        .with_proxy_registry(test_registry())
        .build();
    
    let workflow = r#"
        workflow test {
            observe {
                yield to user for confirmation;
            } on stream {
                act log("observed");
            }
        }
    "#;
    
    let result1 = engine.run(workflow).unwrap();
    assert!(result1.is_suspended());
    
    let result2 = engine.resume(result1.suspend_handle(), confirmation()).unwrap();
    assert!(result2.is_complete());
}

proptest! {
    #[test]
    fn proxy_state_roundtrip(nesting_depth in 0usize..10) {
        // Property: proxy state must survive arbitrary nesting depth
        // and still be resumable
    }
}
```

### Step 2: Audit All Recursive Paths

**File:** `crates/ash-interp/src/execute.rs`

Audit and document every recursive call:

```rust
// Create audit list:
// Line 207: execute_steps in if-then - FIX NEEDED
// Line 231: execute_steps in if-else - FIX NEEDED
// Line 346: execute_steps in match arms - FIX NEEDED
// Line 589: execute_steps in let binding - FIX NEEDED
// Line 936: execute_steps in check block - FIX NEEDED
```

### Step 3: Fix State Propagation

**File:** `crates/ash-interp/src/execute.rs`

```rust
impl Interpreter {
    fn execute_step(
        &mut self,
        step: &Step,
        ctx: &mut Context,
        state: &mut RuntimeState,
        proxy_registry: Option<&ProxyRegistry>,  // Thread through
        suspended_yields: Option<&SuspendedYields>, // Thread through
        capability_resolver: &CapabilityResolver,
    ) -> Result<StepResult, InterpError> {
        match step {
            Step::If { condition, then_branch, else_branch } => {
                let eval = self.eval_expr(condition, ctx, state)?;
                if eval.is_truthy() {
                    // FIX: Pass proxy state instead of None
                    self.execute_steps(
                        then_branch,
                        ctx,
                        state,
                        proxy_registry,      // WAS: None
                        suspended_yields,    // WAS: None
                        capability_resolver,
                    )
                } else if let Some(else_branch) = else_branch {
                    self.execute_steps(
                        else_branch,
                        ctx,
                        state,
                        proxy_registry,      // WAS: None
                        suspended_yields,    // WAS: None
                        capability_resolver,
                    )
                } else {
                    Ok(StepResult::Continue)
                }
            }
            // ... fix all other branches
        }
    }
}
```

### Step 4: Add Compile-Time Guards

**File:** `crates/ash-interp/src/execute.rs`

Add a type-safe wrapper to prevent accidental state dropping:

```rust
/// Ensures proxy state is always threaded through recursive calls
struct ProxyState<'a> {
    registry: Option<&'a ProxyRegistry>,
    suspended: Option<&'a SuspendedYields>,
}

impl<'a> ProxyState<'a> {
    fn new(
        registry: Option<&'a ProxyRegistry>,
        suspended: Option<&'a SuspendedYields>,
    ) -> Self {
        Self { registry, suspended }
    }
    
    fn for_recursion(&self) -> (Option<&'a ProxyRegistry>, Option<&'a SuspendedYields>) {
        (self.registry, self.suspended)
    }
}
```

## Verification Steps

- [ ] `cargo test -p ash-interp --test proxy_state_propagation_test` passes
- [ ] `cargo test -p ash-engine` passes (integration tests)
- [ ] New tests verify yield/resume in nested contexts
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- Reliable proxy state propagation through all execution paths
- SPEC-023 compliance for nested yield/resume

Required by:
- TASK-285: Receive paths proxy state fix (related)

## Notes

**Critical Issue**: This is a fundamental SPEC-023 violation. Nested yields are effectively broken without this fix.

**Risk Assessment**: High - affects all proxy workflow usage with control flow.

**Implementation Strategy**: 
1. First pass: mechanical fix of all `None` values
2. Second pass: add regression tests
3. Third pass: add type-safe wrapper to prevent future regressions
