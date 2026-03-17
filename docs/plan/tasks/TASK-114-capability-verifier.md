# TASK-114: Capability Availability Verifier

## Status: 🔴 Not Started

## Description

Implement verification that runtime provides all capabilities required by workflow.

## Specification Reference

- SPEC-018: Capability Runtime Verification Matrix - Section 4.1

## Requirements

### Functional Requirements

1. Check all required `observes` are available and observable
2. Check all required `receives` are available and receivable
3. Check all required `sets` are available and settable
4. Check all required `sends` are available and sendable
5. Distinguish between missing and wrong-mode (read-only vs writable)

### Property Requirements

```rust
// All present
let result = verifier.verify(&workflow, &registry);
assert!(result.is_ok());

// Missing capability
let result = verifier.verify(&needs_missing, &registry);
assert!(matches!(result.errors[0], MissingCapability { .. }));

// Read-only when write needed
let result = verifier.verify(&needs_set, &readonly_registry);
assert!(matches!(result.errors[0], NotSettable { .. }));
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_all_capabilities_present() {
    let workflow = Workflow::new()
        .observes("sensor", "temp")
        .sets("hvac", "target");
    
    let registry = CapabilityRegistry::new()
        .with("sensor", "temp", BehaviourProvider)
        .with("hvac", "target", SettableProvider);
    
    let result = verifier.verify(&workflow, &registry);
    assert!(result.is_ok());
}

#[test]
fn test_missing_observe() {
    let workflow = Workflow::new().observes("sensor", "temp");
    let registry = CapabilityRegistry::new(); // Empty
    
    let result = verifier.verify(&workflow, &registry);
    assert!(result.has_error(VerificationError::MissingCapability {
        operation: "observe",
        capability: "sensor:temp",
    }));
}

#[test]
fn test_not_settable() {
    let workflow = Workflow::new().sets("sensor", "temp");
    
    -- Registry has provider but not settable
    let registry = CapabilityRegistry::new()
        .with_readonly("sensor", "temp");
    
    let result = verifier.verify(&workflow, &registry);
    assert!(result.has_error(VerificationError::NotSettable { .. }));
}
```

### Step 2: Verify RED

Expected: FAIL - verifier not implemented

### Step 3: Implement (Green)

```rust
pub struct CapabilityVerifier;

impl CapabilityVerifier {
    pub fn verify(
        &self,
        workflow: &Workflow,
        registry: &CapabilityRegistry,
    ) -> VerificationResult {
        let mut result = VerificationResult::new();
        
        -- Check observes
        for (cap, chan) in &workflow.required_observes {
            match registry.get(cap, chan) {
                None => result.add_error(VerificationError::MissingCapability {
                    operation: "observe",
                    capability: format!("{}:{}", cap, chan),
                }),
                Some(p) if !p.is_observable() => result.add_error(
                    VerificationError::NotObservable { ... }
                ),
                _ => {}
            }
        }
        
        -- Check receives
        for (cap, chan) in &workflow.required_receives {
            match registry.get(cap, chan) {
                None => result.add_error(VerificationError::MissingCapability {
                    operation: "receive",
                    capability: format!("{}:{}", cap, chan),
                }),
                Some(p) if !p.is_receivable() => result.add_error(
                    VerificationError::NotReceivable { ... }
                ),
                _ => {}
            }
        }
        
        -- Check sets
        for (cap, chan) in &workflow.required_sets {
            match registry.get(cap, chan) {
                None => result.add_error(VerificationError::MissingCapability {
                    operation: "set",
                    capability: format!("{}:{}", cap, chan),
                }),
                Some(p) if !p.is_settable() => result.add_error(
                    VerificationError::NotSettable { ... }
                ),
                _ => {}
            }
        }
        
        -- Check sends
        for (cap, chan) in &workflow.required_sends {
            match registry.get(cap, chan) {
                None => result.add_error(VerificationError::MissingCapability {
                    operation: "send",
                    capability: format!("{}:{}", cap, chan),
                }),
                Some(p) if !p.is_sendable() => result.add_error(
                    VerificationError::NotSendable { ... }
                ),
                _ => {}
            }
        }
        
        result
    }
}
```

### Step 4: Verify GREEN

Expected: PASS

### Step 5: Commit

```bash
git commit -m "feat: capability availability verifier"
```

## Completion Checklist

- [ ] Observe verification
- [ ] Receive verification
- [ ] Set verification
- [ ] Send verification
- [ ] Missing vs wrong-mode distinction
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

- TASK-119 (Verification aggregator)
