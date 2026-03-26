# TASK-304: Role Semantics Integration Tests

## Status: 📝 Planned

## Description

Create integration tests for role runtime semantics (TASK-287) that verify role context is correctly established by `Oblig` and inherited through nested workflows. The implementation exists but needs E2E coverage.

## Current Implementation (from TASK-287)

```rust
// Workflow::Oblig establishes role context
Workflow::Oblig { role, workflow: inner } => {
    let ctx = ctx.with_role_context(RoleContext::new(role.clone()));
    execute_workflow_inner(inner, ctx, ...).await
}

// Workflow::Check validates against active role
Workflow::Check { obligation, continuation } => {
    require_active_role(&ctx, &obligation.role)?;
    // check condition...
}

// Set/Send use active role
let actor = active_actor(&ctx); // from role_context or "system" default
```

## Test Scenarios

### Scenario 1: Basic Oblig/Check Flow
```rust
// Role established by Oblig is available for Check
workflow test {
    oblige reviewer {
        check reviewer { condition: true }
        ret "ok"
    }
}

// Should succeed - check uses same role
```

### Scenario 2: Role Mismatch
```rust
workflow test {
    oblige reviewer {
        check approver { condition: true }  // Different role!
        ret "ok"
    }
}

// Should fail - active role "reviewer" doesn't match obligation role "approver"
```

### Scenario 3: Nested Oblig
```rust
workflow test {
    oblige admin {
        oblige operator {  // Nested - inner role shadows outer?
            check operator { condition: true }
            ret "ok"
        }
    }
}

// Verify correct role is active in nested context
```

### Scenario 4: Role in Set/Send Operations
```rust
workflow test {
    oblige admin {
        set config { value: "setting" }  // Should use "admin" role
        ret done
    }
}

// Verify Set operation uses "admin" role for policy evaluation
```

### Scenario 5: Role Persists Through Sequential
```rust
workflow test {
    oblige reviewer {
        let x = 1
        check reviewer { condition: true }  // Role still active
        ret "ok"
    }
}

// Role context persists across sequential composition
```

## Test Requirements

1. **Success Cases**: Valid oblige/check flows complete successfully
2. **Failure Cases**: Role mismatches produce clear errors
3. **Nested Cases**: Role shadowing/inheritance works correctly
4. **Integration**: Tests use actual interpreter, not mocked

## Files to Create

- `crates/ash-interp/tests/role_semantics_e2e_test.rs`

## Completion Checklist

- [ ] Basic oblige/check flow test
- [ ] Role mismatch error test
- [ ] Nested oblige role handling test
- [ ] Role used in set/send operations test
- [ ] Role persistence through sequential test
- [ ] All E2E tests pass
- [ ] `cargo test -p ash-interp --test role_semantics_e2e_test` passes
- [ ] `cargo clippy` clean
