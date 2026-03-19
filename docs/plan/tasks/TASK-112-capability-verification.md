# TASK-112: Capability Declaration Verification

## Status: ✅ Complete

## Description

Implement compile-time verification that workflows only use declared capabilities.

## Specification Reference

- SPEC-017: Capability Integration - Section 6 Capability Safety

## Requirements

### Functional Requirements

1. Verify observe uses declared observes
2. Verify receive uses declared receives
3. Verify set uses declared sets
4. Verify send uses declared sends
5. Report undeclared capability errors

### Property Requirements

```rust
// Undeclared observe fails
let result = verify(&workflow);
assert!(matches!(result, Err(UndeclaredCapability { operation: "observe", .. })));

// Declared operations pass
let result = verify(&declared_workflow);
assert!(result.is_ok());
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_undeclared_observe() {
    let workflow = Workflow::parse(r#"
        workflow bad { observe sensor:temp as t }
    "#);
    
    let result = verify_capabilities(&workflow);
    assert!(matches!(result, Err(CapabilityError::NotDeclared { .. })));
}

#[test]
fn test_undeclared_set() {
    let workflow = Workflow::parse(r#"
        workflow bad { set hvac:target = 72 }
    "#);
    
    let result = verify_capabilities(&workflow);
    assert!(result.is_err());
}

#[test]
fn test_declared_ok() {
    let workflow = Workflow::parse(r#"
        workflow good observes sensor:temp, sets hvac:target {
            observe sensor:temp as t;
            set hvac:target = 72
        }
    "#);
    
    let result = verify_capabilities(&workflow);
    assert!(result.is_ok());
}
```

### Step 2: Verify RED

Expected: FAIL - verification not implemented

### Step 3: Implement (Green)

```rust
pub fn verify_capabilities(workflow: &Workflow) -> Result<(), CapabilityError> {
    for operation in workflow.all_operations() {
        let allowed = match operation {
            Operation::Observe { cap, chan } => 
                workflow.capabilities.can_observe(cap, chan),
            Operation::Receive { cap, chan } => 
                workflow.capabilities.can_receive(cap, chan),
            Operation::Set { cap, chan } => 
                workflow.capabilities.can_set(cap, chan),
            Operation::Send { cap, chan } => 
                workflow.capabilities.can_send(cap, chan),
            _ => true,
        };
        
        if !allowed {
            return Err(CapabilityError::NotDeclared {
                operation: operation.name(),
                capability: operation.capability(),
            });
        }
    }
    
    Ok(())
}
```

### Step 4: Verify GREEN

Expected: PASS

### Step 5: Commit

```bash
git commit -m "feat: capability declaration verification"
```

## Completion Checklist

- [ ] Observe verification
- [ ] Receive verification
- [ ] Set verification
- [ ] Send verification
- [ ] Error messages
- [ ] Tests pass
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` clean

## Estimated Effort

4 hours

## Dependencies

None

## Blocked By

Nothing

## Blocks

None
