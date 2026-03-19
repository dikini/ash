# TASK-117: Static Policy Validator

## Status: ✅ Complete

## Description

Implement static validation of policies against workflow capability usage.

## Specification Reference

- SPEC-018: Capability Runtime Verification Matrix - Section 4.3

## Requirements

### Functional Requirements

1. Check for static policy conflicts
2. Identify operations that will always be denied
3. Identify operations that require approval
4. Warn about potential issues

### Property Requirements

```rust
// Always denied
let result = validator.validate(&workflow, &deny_all_policies);
assert!(result.has_error(PolicyConflict { .. }));

// Requires approval
let result = validator.validate(&workflow, &approval_policies);
assert!(result.has_warning(RequiresApproval { .. }));
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_always_denied() {
    let workflow = Workflow::new().observes("sensor", "temp");
    let policies = vec![Policy::deny_all()];
    
    let result = validator.validate(&workflow, &policies);
    assert!(result.has_error(PolicyConflict {
        reason: "All observations denied".into(),
    }));
}

#[test]
fn test_requires_approval() {
    let workflow = Workflow::new().sets("hvac", "target");
    let policies = vec![Policy::require_approval(Role::Admin)];
    
    let result = validator.validate(&workflow, &policies);
    assert!(result.has_warning(RequiresApproval {
        role: Role::Admin,
        operations: vec!["set hvac:target"],
    }));
}
```

### Step 2: Verify RED

Expected: FAIL - validator not implemented

### Step 3: Implement (Green)

```rust
pub struct StaticPolicyValidator;

impl StaticPolicyValidator {
    pub fn validate(
        &self,
        workflow: &Workflow,
        policies: &[Policy],
    ) -> VerificationResult {
        let mut result = VerificationResult::new();
        
        for op in workflow.all_capability_operations() {
            for policy in policies {
                if !policy.applies_to(&op) {
                    continue;
                }
                
                match policy.decision_type() {
                    DecisionType::AlwaysDeny => {
                        result.add_error(VerificationError::PolicyConflict {
                            policy: policy.name(),
                            reason: format!("{:?} always denied", op),
                        });
                    }
                    DecisionType::RequiresApproval(role) => {
                        result.add_warning(VerificationWarning::RequiresApproval {
                            role,
                            operation: op.to_string(),
                        });
                    }
                    _ => {}
                }
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
git commit -m "feat: static policy validator"
```

## Completion Checklist

- [ ] Static conflict detection
- [ ] Always-denied detection
- [ ] Approval requirement detection
- [ ] Error/warning classification
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
