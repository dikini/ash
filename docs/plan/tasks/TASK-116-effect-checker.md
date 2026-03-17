# TASK-116: Effect Compatibility Checker

## Status: 🔴 Not Started

## Description

Implement verification that workflow effect level is within runtime bounds.

## Specification Reference

- SPEC-018: Capability Runtime Verification Matrix - Section 4.4

## Requirements

### Functional Requirements

1. Compute workflow total effect
2. Compare against runtime maximum allowed effect
3. Error if workflow exceeds runtime limit

### Property Requirements

```rust
// Epistemic workflow in Operational context - OK
let result = checker.check(&epistemic_workflow, Effect::Operational);
assert!(result.is_ok());

// Operational workflow in Epistemic context - ERROR
let result = checker.check(&operational_workflow, Effect::Epistemic);
assert!(matches!(result.errors[0], EffectTooHigh { .. }));
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_epistemic_within_bounds() {
    let workflow = Workflow::new() -- Only observes
        .observes("sensor", "temp");
    
    let result = checker.check(&workflow, Effect::Operational);
    assert!(result.is_ok());
}

#[test]
fn test_operational_exceeds_bounds() {
    let workflow = Workflow::new() -- Has set
        .observes("sensor", "temp")
        .sets("hvac", "target");
    
    let result = checker.check(&workflow, Effect::Epistemic);
    assert!(result.has_error(EffectTooHigh {
        workflow_effect: Effect::Operational,
        max_allowed: Effect::Epistemic,
    }));
}

#[test]
fn test_exact_match() {
    let workflow = Workflow::new()
        .sets("hvac", "target");
    
    let result = checker.check(&workflow, Effect::Operational);
    assert!(result.is_ok());
}
```

### Step 2: Verify RED

Expected: FAIL - checker not implemented

### Step 3: Implement (Green)

```rust
pub struct EffectChecker;

impl EffectChecker {
    pub fn check(
        &self,
        workflow: &Workflow,
        max_allowed: Effect,
    ) -> VerificationResult {
        let mut result = VerificationResult::new();
        
        let workflow_effect = workflow.effect();
        
        if workflow_effect > max_allowed {
            result.add_error(VerificationError::EffectTooHigh {
                workflow_effect,
                max_allowed,
            });
        }
        
        result
    }
}

impl Effect {
    pub fn join(self, other: Effect) -> Effect {
        use Effect::*;
        match (self, other) {
            (Operational, _) | (_, Operational) => Operational,
            (Evaluative, _) | (_, Evaluative) => Evaluative,
            (Deliberative, _) | (_, Deliberative) => Deliberative,
            (Epistemic, Epistemic) => Epistemic,
        }
    }
}
```

### Step 4: Verify GREEN

Expected: PASS

### Step 5: Commit

```bash
git commit -m "feat: effect compatibility checker"
```

## Completion Checklist

- [ ] Effect computation
- [ ] Effect comparison
- [ ] Join semantics
- [ ] Error on exceed
- [ ] Tests pass
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` clean

## Estimated Effort

3 hours

## Dependencies

- TASK-108 (Effect tracking)

## Blocked By

- TASK-108

## Blocks

- TASK-119 (Verification aggregator)
