# TASK-115: Obligation Satisfaction Checker

## Status: ✅ Complete

## Description

Implement verification that runtime obligations satisfy workflow requirements.

## Specification Reference

- SPEC-018: Capability Runtime Verification Matrix - Section 4.2

## Requirements

### Functional Requirements

1. Check required role is granted by obligations
2. Check specific obligations are present
3. Distinguish between required and optional obligations
4. Report which obligations are missing

### Property Requirements

```rust
// Role satisfied
let result = checker.check(&workflow, &obligations);
assert!(result.is_ok());

// Missing required role
let result = checker.check(&needs_operator, &guest_obligations);
assert!(matches!(result.errors[0], RoleMismatch { .. }));
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_role_satisfied() {
    let workflow = Workflow::new().requires_role(Role::Operator);
    let obligations = vec![Obligation::role(Role::Operator)];
    
    let result = checker.check(&workflow, &obligations);
    assert!(result.is_ok());
}

#[test]
fn test_role_mismatch() {
    let workflow = Workflow::new().requires_role(Role::Operator);
    let obligations = vec![Obligation::role(Role::Guest)];
    
    let result = checker.check(&workflow, &obligations);
    assert!(result.has_error(RoleMismatch {
        required: Role::Operator,
        available: vec![Role::Guest],
    }));
}

#[test]
fn test_obligation_present() {
    let workflow = Workflow::new().requires_obligation("maintain_temp");
    let obligations = vec![
        Obligation::new("maintain_temp"),
    ];
    
    let result = checker.check(&workflow, &obligations);
    assert!(result.is_ok());
}

#[test]
fn test_obligation_missing() {
    let workflow = Workflow::new().requires_obligation("maintain_temp");
    let obligations = vec![];
    
    let result = checker.check(&workflow, &obligations);
    assert!(result.has_warning(MissingObligation { .. }));
}
```

### Step 2: Verify RED

Expected: FAIL - checker not implemented

### Step 3: Implement (Green)

```rust
pub struct ObligationChecker;

impl ObligationChecker {
    pub fn check(
        &self,
        workflow: &Workflow,
        obligations: &[Obligation],
    ) -> VerificationResult {
        let mut result = VerificationResult::new();
        
        -- Check role requirement
        if let Some(required_role) = &workflow.required_role {
            let has_role = obligations.iter()
                .any(|o| o.grants_role(required_role));
            
            if !has_role {
                let available: Vec<_> = obligations.iter()
                    .flat_map(|o| o.roles())
                    .collect();
                
                result.add_error(VerificationError::RoleMismatch {
                    required: required_role.clone(),
                    available,
                });
            }
        }
        
        -- Check required obligations
        for required in &workflow.required_obligations {
            let is_present = obligations.iter()
                .any(|o| o.satisfies(required));
            
            if !is_present {
                if required.is_critical() {
                    result.add_error(VerificationError::MissingObligation {
                        obligation: required.clone(),
                    });
                } else {
                    result.add_warning(VerificationWarning::MissingObligation {
                        obligation: required.clone(),
                    });
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
git commit -m "feat: obligation satisfaction checker"
```

## Completion Checklist

- [ ] Role verification
- [ ] Obligation presence check
- [ ] Critical vs optional distinction
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
