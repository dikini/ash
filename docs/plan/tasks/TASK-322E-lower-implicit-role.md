# TASK-322E: Update Lowering for Implicit Default Role

## Status: 🔴 Blocking - TASK-322 Sub-task

## Problem

Lowering needs to correctly desugar workflows with `capabilities:` to implicit roles using the new `RoleDef` structure.

## Scope

**This task ONLY updates lowering.** Runtime is already updated (TASK-322D).

## Implementation

### 1. Update Implicit Role Generation

**Current:** May generate `authority:` or old-style role.

**New:** Generate `capabilities:` with proper `CapabilityDecl`.

```rust
// When lowering workflow with capabilities but no explicit role:
// workflow quick_task capabilities: [file @ { paths: ["/tmp/*"] }] { ... }
// 
// Generates:
// role quick_task_default {
//     capabilities: [file @ { paths: ["/tmp/*"] }]
// }
// workflow quick_task plays role(quick_task_default) { ... }

fn generate_implicit_role(
    workflow_name: &Name,
    capabilities: &[CapabilityDecl],
) -> RoleDef {
    RoleDef {
        name: format!("{}_default", workflow_name.as_ref()).into(),
        capabilities: capabilities.to_vec(),  // Copy capability declarations with constraints
        obligations: vec![],
        span: Span::default(),
    }
}
```

### 2. Update Workflow Lowering

Ensure workflow lowering passes capabilities correctly:

```rust
fn lower_workflow_def(workflow: &WorkflowDef) -> Result<CoreWorkflow, LowerError> {
    // If workflow has capabilities but no roles, generate implicit role
    if !workflow.capabilities.is_empty() && workflow.plays_roles.is_empty() {
        let implicit_role = generate_implicit_role(&workflow.name, &workflow.capabilities);
        // Register implicit role, add plays role reference
    }
    
    // ... rest of lowering
}
```

## TDD Steps

### Step 1: Write Lowering Tests (Before Implementation)

```rust
#[test]
fn test_implicit_role_generation_with_constraints() {
    let workflow = WorkflowDef {
        name: "quick_task".into(),
        params: vec![],
        plays_roles: vec![],  // No explicit role
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
                    ],
                    span: Span::default(),
                }),
                span: Span::default(),
            },
        ],
        body: Workflow::Done { span: Span::default() },
        contract: None,
        span: Span::default(),
    };
    
    let (roles, lowered) = lower_workflow(&workflow).unwrap();
    
    // Should generate one implicit role
    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0].name.as_ref(), "quick_task_default");
    
    // Implicit role should have capabilities with constraints
    assert_eq!(roles[0].capabilities.len(), 1);
    assert!(roles[0].capabilities[0].constraints.is_some());
    
    // Lowered workflow should play the implicit role
    assert_eq!(lowered.plays_roles.len(), 1);
    assert_eq!(lowered.plays_roles[0].name.as_ref(), "quick_task_default");
}

#[test]
fn test_explicit_role_no_implicit_generation() {
    let workflow = WorkflowDef {
        name: "explicit".into(),
        params: vec![],
        plays_roles: vec![RoleRef { name: "ai_agent".into(), span: Span::default() }],
        capabilities: vec![],  // No direct capabilities
        body: Workflow::Done { span: Span::default() },
        contract: None,
        span: Span::default(),
    };
    
    let (roles, lowered) = lower_workflow(&workflow).unwrap();
    
    // No implicit role needed
    assert!(roles.is_empty());
    
    // Workflow plays explicit role
    assert_eq!(lowered.plays_roles.len(), 1);
    assert_eq!(lowered.plays_roles[0].name.as_ref(), "ai_agent");
}
```

### Step 2: Verify Tests Fail

```bash
cargo test --package ash-lower test_implicit
# Expected: Fail - lowering doesn't generate proper implicit roles yet
```

### Step 3: Implement Lowering Changes

Update implicit role generation to use new `RoleDef` with `capabilities`.

### Step 4: Verify Tests Pass

```bash
cargo test --package ash-lower test_implicit
# Expected: Pass
```

## Step 5: Code Review Sub-Process

Spawn a code review sub-agent to verify:

```
Review Focus for TASK-322E:
- Implicit roles are generated with correct name pattern
- Capabilities are copied correctly to implicit role
- Constraints are preserved during lowering
- No duplicate implicit role generation
- Explicit roles take precedence (no unnecessary implicit role)
- Error handling for naming conflicts
- Span information preserved for error messages
```

### Review Checklist (Rust-Specific)

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --package ash-lower` clean
- [ ] `cargo test --package ash-lower` all tests pass
- [ ] No unnecessary allocations in lowering
- [ ] Error handling is thorough

### Review Output

Reviewer should provide:
1. **Critical issues** (must fix before merge)
2. **Suggestions** (can be addressed or noted for follow-up)
3. **Approval** or **Request Changes**

## Files to Modify

- `crates/ash-lower/src/` - Update implicit role generation
- `crates/ash-lower/src/` - Update workflow lowering

## Completion Checklist

- [ ] Implicit roles generated with `capabilities:` field
- [ ] Constraints preserved in implicit role capabilities
- [ ] Workflows with explicit roles don't generate implicit roles
- [ ] All lowering tests pass
- [ ] `cargo test --package ash-lower` passes
- [ ] **Code review completed** with no critical issues
- [ ] Review feedback addressed (if any)

**Estimated Hours:** 2-3 (including review)
**Priority:** Blocking
**Blocked By:** TASK-322D
**Blocks:** TASK-322F
