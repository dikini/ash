# TASK-110: Policy Evaluation for Input/Output

## Status: 🔴 Not Started

## Description

Implement policy evaluation for both input and output capabilities.

## Specification Reference

- SPEC-017: Capability Integration - Section 4 Policies

## Requirements

### Functional Requirements

1. Evaluate policies for observe/receive (input)
2. Evaluate policies for set/send (output)
3. Support input transformations (masking)
4. Support output approvals

### Property Requirements

```rust
// Input policy allows
let decision = eval.evaluate_input(&ctx).unwrap();
assert!(matches!(decision, Decision::Permit));

// Output policy denies
let decision = eval.evaluate_output(&ctx).unwrap();
assert!(matches!(decision, Decision::Deny));
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_input_policy_permit() {
    let ctx = CapabilityContext::input("sensor", "temp");
    let decision = policy_eval.evaluate(&ctx).unwrap();
    assert!(matches!(decision, Decision::Permit));
}

#[test]
fn test_input_policy_transform() {
    let ctx = CapabilityContext::input("database", "users");
    let decision = policy_eval.evaluate(&ctx).unwrap();
    
    assert!(matches!(decision, Decision::Transform { .. }));
}

#[test]
fn test_output_policy_deny() {
    let ctx = CapabilityContext::output("hvac", "target", Value::Int(100));
    let decision = policy_eval.evaluate(&ctx).unwrap();
    
    assert!(matches!(decision, Decision::Deny));
}

#[test]
fn test_output_policy_require_approval() {
    let ctx = CapabilityContext::output("hvac", "target", Value::Int(90))
        .with_actor(Role::Operator);
    
    let decision = policy_eval.evaluate(&ctx).unwrap();
    assert!(matches!(decision, Decision::RequireApproval { .. }));
}
```

### Step 2: Verify RED

Expected: FAIL - evaluation not implemented

### Step 3: Implement (Green)

```rust
impl PolicyEvaluator {
    pub fn evaluate_capability(&self, ctx: &CapabilityContext) -> PolicyResult {
        match ctx.direction {
            Direction::Input => self.evaluate_input(ctx),
            Direction::Output => self.evaluate_output(ctx),
        }
    }
    
    fn evaluate_input(&self, ctx: &CapabilityContext) -> PolicyResult {
        -- Check input policies
        for policy in &self.input_policies {
            if policy.matches(ctx) {
                return policy.decision(ctx);
            }
        }
        Ok(Decision::Permit)
    }
    
    fn evaluate_output(&self, ctx: &CapabilityContext) -> PolicyResult {
        -- Check output policies
        for policy in &self.output_policies {
            if policy.matches(ctx) {
                return policy.decision(ctx);
            }
        }
        Ok(Decision::Permit)
    }
}
```

### Step 4: Verify GREEN

Expected: PASS

### Step 5: Commit

```bash
git commit -m "feat: policy evaluation for input/output"
```

## Completion Checklist

- [ ] Input policy evaluation
- [ ] Output policy evaluation
- [ ] Transformation support
- [ ] Approval support
- [ ] Tests pass
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` clean

## Estimated Effort

6 hours

## Dependencies

None

## Blocked By

Nothing

## Blocks

None
