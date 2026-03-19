# TASK-109: Obligation Checking with Capabilities

## Status: ✅ Complete

## Description

Implement obligation checking that verifies workflows have required capabilities.

## Specification Reference

- SPEC-017: Capability Integration - Section 3 Obligations

## Requirements

### Functional Requirements

1. Check workflow has required input capabilities
2. Check workflow has required output capabilities
3. Check effect level is sufficient
4. Report missing capabilities

### Property Requirements

```rust
// Missing capability fails
let result = checker.verify(&obligation, &workflow);
assert!(result.is_err());

// Sufficient capabilities pass
let result = checker.verify(&obligation, &complete_workflow);
assert!(result.is_ok());
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_missing_input_capability() {
    let obligation = Obligation::new("check_temp")
        .requires_observe("sensor", "temp");
    
    let workflow = Workflow::new(); // No capabilities
    
    let result = checker.verify(&obligation, &workflow);
    assert!(matches!(result, Err(ObligationError::MissingCapability { .. })));
}

#[test]
fn test_missing_output_capability() {
    let obligation = Obligation::new("control_temp")
        .requires_set("hvac", "target");
    
    let workflow = Workflow::new(); // No set capability
    
    let result = checker.verify(&obligation, &workflow);
    assert!(result.is_err());
}

#[test]
fn test_sufficient_capabilities() {
    let obligation = Obligation::new("control")
        .requires_observe("sensor", "temp")
        .requires_set("hvac", "target");
    
    let workflow = Workflow::new()
        .observes("sensor", "temp")
        .sets("hvac", "target");
    
    let result = checker.verify(&obligation, &workflow);
    assert!(result.is_ok());
}
```

### Step 2: Verify RED

Expected: FAIL - checker not implemented

### Step 3: Implement (Green)

```rust
pub struct ObligationChecker;

impl ObligationChecker {
    pub fn verify(
        &self,
        obligation: &Obligation,
        workflow: &Workflow,
    ) -> Result<(), ObligationError> {
        -- Check input capabilities
        for (cap, channel) in &obligation.required_observes {
            if !workflow.can_observe(cap, channel) {
                return Err(ObligationError::MissingCapability {
                    operation: "observe",
                    capability: format!("{}:{}", cap, channel),
                });
            }
        }
        
        for (cap, channel) in &obligation.required_receives {
            if !workflow.can_receive(cap, channel) {
                return Err(ObligationError::MissingCapability {
                    operation: "receive",
                    capability: format!("{}:{}", cap, channel),
                });
            }
        }
        
        -- Check output capabilities
        for (cap, channel) in &obligation.required_sets {
            if !workflow.can_set(cap, channel) {
                return Err(ObligationError::MissingCapability {
                    operation: "set",
                    capability: format!("{}:{}", cap, channel),
                });
            }
        }
        
        for (cap, channel) in &obligation.required_sends {
            if !workflow.can_send(cap, channel) {
                return Err(ObligationError::MissingCapability {
                    operation: "send",
                    capability: format!("{}:{}", cap, channel),
                });
            }
        }
        
        -- Check effect level
        if workflow.effect() < obligation.min_effect {
            return Err(ObligationError::InsufficientEffect {
                required: obligation.min_effect,
                actual: workflow.effect(),
            });
        }
        
        Ok(())
    }
}
```

### Step 4: Verify GREEN

Expected: PASS

### Step 5: Commit

```bash
git commit -m "feat: obligation checking with capabilities"
```

## Completion Checklist

- [ ] Input capability checking
- [ ] Output capability checking
- [ ] Effect level checking
- [ ] Error reporting
- [ ] Tests pass
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` clean

## Estimated Effort

4 hours

## Dependencies

- TASK-108 (Effect tracking)

## Blocked By

- TASK-108

## Blocks

None
