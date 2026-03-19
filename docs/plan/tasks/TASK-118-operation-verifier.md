# TASK-118: Per-Operation Runtime Verifier

## Status: ✅ Complete

## Description

Implement runtime verification of individual capability operations.

## Specification Reference

- SPEC-018: Capability Runtime Verification Matrix - Section 5

## Requirements

### Functional Requirements

1. Check capability still available at operation time
2. Evaluate dynamic policies
3. Check rate limits
4. Handle approvals and transformations

### Property Requirements

```rust
// Allowed
let result = verifier.verify(&op, &runtime).await;
assert!(matches!(result, OperationResult::Proceed));

// Denied
let result = verifier.verify(&denied_op, &runtime).await;
assert!(matches!(result, OperationResult::Denied { .. }));

// Requires approval
let result = verifier.verify(&restricted_op, &runtime).await;
assert!(matches!(result, OperationResult::RequiresApproval(_)));
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[tokio::test]
async fn test_operation_allowed() {
    let op = CapabilityOperation::observe("sensor", "temp");
    let runtime = RuntimeContext::with_policy(Policy::permit_all());
    
    let result = verifier.verify(&op, &runtime).await;
    assert!(matches!(result, Ok(OperationResult::Proceed)));
}

#[tokio::test]
async fn test_operation_denied() {
    let op = CapabilityOperation::observe("sensor", "temp");
    let runtime = RuntimeContext::with_policy(Policy::deny_all());
    
    let result = verifier.verify(&op, &runtime).await;
    assert!(matches!(result, Ok(OperationResult::Denied { .. })));
}

#[tokio::test]
async fn test_rate_limit_exceeded() {
    let op = CapabilityOperation::send("alerts", "critical");
    let runtime = RuntimeContext::new()
        .with_rate_limit(RateLimit::new(1, Duration::from_secs(60)))
        .exhausted();
    
    let result = verifier.verify(&op, &runtime).await;
    assert!(matches!(result, Err(OperationError::RateLimitExceeded)));
}
```

### Step 2: Verify RED

Expected: FAIL - verifier not implemented

### Step 3: Implement (Green)

```rust
pub struct OperationVerifier;

impl OperationVerifier {
    pub async fn verify(
        &self,
        op: &CapabilityOperation,
        runtime: &RuntimeContext,
    ) -> Result<OperationResult, OperationError> {
        -- 1. Check capability available
        let provider = runtime.capabilities
            .get(&op.capability, &op.channel)
            .ok_or(OperationError::CapabilityUnavailable)?;
        
        -- 2. Check mode
        match op.direction {
            Direction::Output if !provider.supports_output() => {
                return Err(OperationError::ModeNotSupported);
            }
            _ => {}
        }
        
        -- 3. Evaluate dynamic policies
        let ctx = PolicyContext::from_operation(op, runtime);
        match runtime.policy_evaluator.evaluate(&ctx).await? {
            Decision::Permit => {}
            Decision::Deny => return Ok(OperationResult::Denied { policy: ctx.policy }),
            Decision::RequireApproval(role) => {
                return Ok(OperationResult::RequiresApproval(role));
            }
            Decision::Transform(t) => return Ok(OperationResult::Transformed(t)),
        }
        
        -- 4. Check rate limits
        if runtime.rate_limiter.would_exceed(op) {
            return Err(OperationError::RateLimitExceeded);
        }
        
        Ok(OperationResult::Proceed)
    }
}
```

### Step 4: Verify GREEN

Expected: PASS

### Step 5: Commit

```bash
git commit -m "feat: per-operation runtime verifier"
```

## Completion Checklist

- [ ] Availability check
- [ ] Mode check
- [ ] Dynamic policy evaluation
- [ ] Rate limit check
- [ ] Approval handling
- [ ] Tests pass
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` clean

## Estimated Effort

5 hours

## Dependencies

- TASK-110 (Policy evaluation)

## Blocked By

- TASK-110

## Blocks

None
