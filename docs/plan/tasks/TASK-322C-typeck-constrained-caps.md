# TASK-322C: Update Type Checker for Constrained Role Capabilities

## Status: 🔴 Blocking - TASK-322 Sub-task

## Problem

Type checker currently synthesizes unconstrained `CapabilityDecl` from bare authority names. It needs to use the actual constrained `CapabilityDecl` from the role definition.

## Scope

**This task ONLY updates type checking.** Parser is already updated (TASK-322B).

## Implementation

### 1. Update `compose_role_capabilities` in `crates/ash-typeck/src/role_checking.rs`

**Current (lines 183-198):**
```rust
fn compose_role_capabilities(&self, role_def: &RoleDef) -> EffectiveCapabilities {
    let mut effective = EffectiveCapabilities::new();
    let role_name = role_def.name.as_ref();

    // Convert authority names to capability declarations
    for authority in &role_def.authority {  // OLD FIELD
        let cap_decl = CapabilityDecl {
            capability: authority.clone(),
            constraints: None,  // NO CONSTRAINTS!
            span: role_def.span,
        };
        effective.add_capability(role_name, cap_decl);
    }

    effective
}
```

**New:**
```rust
fn compose_role_capabilities(&self, role_def: &RoleDef) -> EffectiveCapabilities {
    let mut effective = EffectiveCapabilities::new();
    let role_name = role_def.name.as_ref();

    // Use existing CapabilityDecl with constraints from role
    for cap_decl in &role_def.capabilities {  // NEW FIELD
        effective.add_capability(role_name, cap_decl.clone());
    }

    effective
}
```

### 2. Add Constraint Validation (Optional for this task)

Basic validation that constraint fields exist in capability definition can be added here or deferred to runtime.

## TDD Steps

### Step 1: Write Type Checker Tests (Before Implementation)

```rust
#[test]
fn test_compose_role_with_constrained_capabilities() {
    let mut role_defs = HashMap::new();
    
    // Role with constrained capabilities
    role_defs.insert("ai_agent".to_string(), RoleDef {
        name: "ai_agent".into(),
        capabilities: vec![
            CapabilityDecl {
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
                        ConstraintField {
                            name: "read".into(),
                            value: ConstraintValue::Bool(true),
                            span: Span::default(),
                        },
                    ],
                    span: Span::default(),
                }),
                span: Span::default(),
            },
        ],
        obligations: vec![],
        span: Span::default(),
    });
    
    let checker = RoleChecker::new(&role_defs);
    let effective = checker.compose_role_capabilities(&role_defs["ai_agent"]).unwrap();
    
    assert!(effective.has("file"));
    
    let cap = effective.get("file").unwrap();
    assert!(cap.constraints.is_some());
    
    let constraints = cap.constraints.as_ref().unwrap();
    assert_eq!(constraints.fields.len(), 2);
}

#[test]
fn test_role_capability_constraints_preserved() {
    let mut role_defs = HashMap::new();
    
    role_defs.insert("limited".to_string(), RoleDef {
        name: "limited".into(),
        capabilities: vec![
            CapabilityDecl {
                capability: "network".into(),
                constraints: Some(ConstraintBlock {
                    fields: vec![
                        ConstraintField {
                            name: "hosts".into(),
                            value: ConstraintValue::Array(vec![
                                ConstraintValue::String("*.example.com".to_string()),
                            ]),
                            span: Span::default(),
                        },
                    ],
                    span: Span::default(),
                }),
                span: Span::default(),
            },
        ],
        obligations: vec![],
        span: Span::default(),
    });
    
    let checker = RoleChecker::new(&role_defs);
    let effective = checker.check_workflow_roles(&workflow_with_plays_role("limited")).unwrap();
    
    let network_cap = effective.get("network").unwrap();
    let hosts_field = network_cap.constraints.as_ref().unwrap()
        .fields.iter()
        .find(|f| f.name.as_ref() == "hosts")
        .expect("hosts constraint should exist");
        
    match &hosts_field.value {
        ConstraintValue::Array(arr) => {
            assert_eq!(arr.len(), 1);
            match &arr[0] {
                ConstraintValue::String(s) => assert_eq!(s, "*.example.com"),
                _ => panic!("Expected string value"),
            }
        }
        _ => panic!("Expected array value"),
    }
}
```

### Step 2: Verify Tests Fail

```bash
cargo test --package ash-typeck test_role
# Expected: Compile or test fail - uses old authority field
```

### Step 3: Implement Type Checker Change

Update `compose_role_capabilities` to use `capabilities` field with constraints.

### Step 4: Verify Tests Pass

```bash
cargo test --package ash-typeck test_role
# Expected: Pass
```

## Step 5: Code Review Sub-Process

Spawn a code review sub-agent to verify:

```
Review Focus for TASK-322C:
- Type checker correctly uses new capabilities field
- Constraints are preserved through composition
- No synthesized CapabilityDecl with None constraints
- Error handling for invalid constraints (if validation added)
- Role merging handles constraint conflicts correctly
- Type safety maintained (no unsafe code)
- Performance: no unnecessary cloning
```

### Review Checklist (Rust-Specific)

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --package ash-typeck` clean
- [ ] `cargo test --package ash-typeck` all tests pass
- [ ] Property tests for constraint composition (if proptest available)
- [ ] No unwrap() in production code (use expect with message or proper error handling)
- [ ] Error types are descriptive

### Review Output

Reviewer should provide:
1. **Critical issues** (must fix before merge)
2. **Suggestions** (can be addressed or noted for follow-up)
3. **Approval** or **Request Changes**

## Files to Modify

- `crates/ash-typeck/src/role_checking.rs` - Update `compose_role_capabilities`
- Test files - Update any tests using old `authority` field

## Completion Checklist

- [ ] `compose_role_capabilities` uses `capabilities` field from `RoleDef`
- [ ] Constraints are preserved in `EffectiveCapabilities`
- [ ] All type checker tests pass
- [ ] `cargo test --package ash-typeck` passes
- [ ] **Code review completed** with no critical issues
- [ ] Review feedback addressed (if any)

**Estimated Hours:** 2-3 (including review)
**Priority:** Blocking
**Blocked By:** TASK-322B
**Blocks:** TASK-322D
