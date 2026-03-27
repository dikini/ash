# TASK-322D: Runtime Constraint Enforcement for Role Capabilities

## Status: 🔴 Blocking - TASK-322 Sub-task

## Problem

Runtime currently grants capabilities by name only without checking constraints. It needs to store and enforce constraints from role definitions.

## Scope

**This task ONLY updates runtime constraint enforcement.** Type checker is already updated (TASK-322C).

## Implementation

### 1. Update `CapabilityGrant` in `crates/ash-interp/src/role_runtime.rs`

**Current:** Stores only capability name and granting roles.

**New:** Also store constraints.

```rust
/// A granted capability at runtime
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityGrant {
    /// The capability name
    name: String,
    /// Roles that granted this capability
    granting_roles: Vec<String>,
    /// Optional constraints from role definition (NEW)
    constraints: Option<ConstraintBlock>,
}
```

### 2. Update `grant_by_capability_decl` Method

**Current:** `grant_by_name` only takes name.

**New:** Method that takes full `CapabilityDecl`:

```rust
/// Grant a capability from a CapabilityDecl (includes constraints)
pub fn grant_by_decl(&mut self, cap_decl: &CapabilityDecl, role_name: &str) {
    let name = cap_decl.capability.as_ref().to_string();

    if let Some(existing) = self.grants.get_mut(&name) {
        existing.add_granting_role(role_name.to_string());
        // TODO: Merge constraints or check compatibility
    } else {
        let mut grant = CapabilityGrant::new(name.clone());
        grant.add_granting_role(role_name.to_string());
        grant.constraints = cap_decl.constraints.clone();  // STORE CONSTRAINTS
        self.grants.insert(name, grant);
    }
}
```

### 3. Update `check_authority` to Enforce Constraints

```rust
/// Check if a capability request satisfies constraints
pub fn check_authority(
    &self,
    capability_name: &str,
    request_args: &HashMap<String, Value>,
) -> Result<(), CapabilityError> {
    let grant = self.grants.get(capability_name)
        .ok_or(CapabilityError::NotGranted)?;
    
    // If there are constraints, check them
    if let Some(constraints) = &grant.constraints {
        self.check_constraints(constraints, request_args)?;
    }
    
    Ok(())
}

/// Check if request args satisfy constraints
fn check_constraints(
    &self,
    constraints: &ConstraintBlock,
    request_args: &HashMap<String, Value>,
) -> Result<(), CapabilityError> {
    for field in &constraints.fields {
        match field.name.as_ref() {
            "paths" => self.check_path_constraint(&field.value, request_args)?,
            "hosts" => self.check_host_constraint(&field.value, request_args)?,
            "read" | "write" => self.check_permission_constraint(&field.value, request_args)?,
            _ => {
                // Unknown constraint field - could warn or fail based on policy
                // For now, skip unknown fields
            }
        }
    }
    Ok(())
}
```

## TDD Steps

### Step 1: Write Runtime Tests (Before Implementation)

```rust
#[test]
fn test_grant_with_constraints_stored() {
    let mut registry = RoleRegistry::new();
    
    let cap_decl = CapabilityDecl {
        capability: "file".into(),
        constraints: Some(ConstraintBlock {
            fields: vec![
                ConstraintField {
                    name: "paths".into(),
                    value: ConstraintValue::Array(vec![
                        ConstraintValue::String("/tmp/*".to_string()),
                    ]),
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        }),
        span: Span::default(),
    };
    
    let mut cap_set = RuntimeCapabilitySet::new();
    cap_set.grant_by_decl(&cap_decl, "ai_agent");
    
    let grant = cap_set.get_grant("file").unwrap();
    assert!(grant.constraints.is_some());
}

#[test]
fn test_constraint_violation_fails() {
    let mut cap_set = RuntimeCapabilitySet::new();
    
    let cap_decl = CapabilityDecl {
        capability: "file".into(),
        constraints: Some(ConstraintBlock {
            fields: vec![
                ConstraintField {
                    name: "paths".into(),
                    value: ConstraintValue::Array(vec![
                        ConstraintValue::String("/tmp/*".to_string()),
                    ]),
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        }),
        span: Span::default(),
    };
    
    cap_set.grant_by_decl(&cap_decl, "ai_agent");
    
    // Request to access /etc/passwd should fail
    let mut args = HashMap::new();
    args.insert("path".to_string(), Value::String("/etc/passwd".to_string()));
    
    let result = cap_set.check_authority("file", &args);
    assert!(matches!(result, Err(CapabilityError::ConstraintViolation { .. })));
}

#[test]
fn test_constraint_satisfied_succeeds() {
    let mut cap_set = RuntimeCapabilitySet::new();
    
    let cap_decl = CapabilityDecl {
        capability: "file".into(),
        constraints: Some(ConstraintBlock {
            fields: vec![
                ConstraintField {
                    name: "paths".into(),
                    value: ConstraintValue::Array(vec![
                        ConstraintValue::String("/tmp/*".to_string()),
                    ]),
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        }),
        span: Span::default(),
    };
    
    cap_set.grant_by_decl(&cap_decl, "ai_agent");
    
    // Request to access /tmp/file.txt should succeed
    let mut args = HashMap::new();
    args.insert("path".to_string(), Value::String("/tmp/file.txt".to_string()));
    
    let result = cap_set.check_authority("file", &args);
    assert!(result.is_ok());
}
```

### Step 2: Verify Tests Fail

```bash
cargo test --package ash-interp test_constraint
# Expected: Fail - constraints not yet enforced
```

### Step 3: Implement Runtime Changes

Add constraint storage and enforcement.

### Step 4: Verify Tests Pass

```bash
cargo test --package ash-interp test_constraint
# Expected: Pass
```

## Step 5: Code Review Sub-Process

Spawn a code review sub-agent to verify:

```
Review Focus for TASK-322D:
- Constraints are stored correctly in CapabilityGrant
- Constraint checking is performed at capability invocation
- Path constraint checking handles wildcards correctly
- Host constraint checking validates against allowed patterns
- Permission constraints work (read/write flags)
- Error messages are clear about what constraint was violated
- No performance issues with constraint checking
- Edge cases: empty constraints, unknown constraint fields
```

### Review Checklist (Rust-Specific)

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --package ash-interp` clean
- [ ] `cargo test --package ash-interp` all tests pass
- [ ] Property tests for constraint enforcement (glob matching, host patterns)
- [ ] No panics on constraint checking
- [ ] Error types are descriptive

### Review Output

Reviewer should provide:
1. **Critical issues** (must fix before merge)
2. **Suggestions** (can be addressed or noted for follow-up)
3. **Approval** or **Request Changes**

## Files to Modify

- `crates/ash-interp/src/role_runtime.rs` - Update `CapabilityGrant`, add constraint checking

## Completion Checklist

- [ ] `CapabilityGrant` stores constraints from role definition
- [ ] `grant_by_decl` method accepts full `CapabilityDecl`
- [ ] `check_authority` enforces constraints against request args
- [ ] Constraint violations return `CapabilityError::ConstraintViolation`
- [ ] All runtime tests pass
- [ ] `cargo test --package ash-interp` passes
- [ ] **Code review completed** with no critical issues
- [ ] Review feedback addressed (if any)

**Estimated Hours:** 3-4 (including review)
**Priority:** Blocking
**Blocked By:** TASK-322C
**Blocks:** TASK-322E
