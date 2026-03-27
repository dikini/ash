# TASK-322A: Update RoleDef AST to Use CapabilityDecl

## Status: 🔴 Blocking - TASK-322 Sub-task

## Problem

`RoleDef` currently stores `authority: Vec<Name>` (bare capability names) but SPEC-024 requires `capabilities: Vec<CapabilityDecl>` (capability names with optional constraints).

## Scope

**This task ONLY changes the AST type.** No parser or logic changes yet.

## Implementation

### 1. Update `RoleDef` struct in `crates/ash-parser/src/surface.rs`

**Current:**
```rust
pub struct RoleDef {
    pub name: Name,
    pub authority: Vec<Name>,  // <-- Change this
    pub obligations: Vec<Name>,
    pub span: Span,
}
```

**New:**
```rust
pub struct RoleDef {
    pub name: Name,
    /// Capabilities granted to this role (replaces authority)
    pub capabilities: Vec<CapabilityDecl>,
    pub obligations: Vec<Name>,
    pub span: Span,
}
```

## TDD Steps

### Step 1: Write Tests (Before Implementation)

```rust
#[test]
fn test_role_def_with_capability_decl() {
    let role = RoleDef {
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
                    ],
                    span: Span::default(),
                }),
                span: Span::default(),
            },
        ],
        obligations: vec![],
        span: Span::default(),
    };

    assert_eq!(role.capabilities.len(), 1);
    assert_eq!(role.capabilities[0].capability.as_ref(), "file");
    assert!(role.capabilities[0].constraints.is_some());
}
```

### Step 2: Verify Tests Fail

```bash
cargo test --package ash-parser test_role_def_with_capability_decl
# Expected: Compile error - authority field doesn't exist
```

### Step 3: Implement AST Change

Change `authority: Vec<Name>` to `capabilities: Vec<CapabilityDecl>` in `RoleDef`.

### Step 4: Fix Compilation Errors in Dependent Code

**Temporarily** update call sites to use new field name (even if logic is wrong):

```rust
// In parse_module.rs, temporarily:
RoleDef {
    name: name.into(),
    capabilities: Vec::new(), // placeholder - will be fixed in TASK-322B
    obligations,
    span: ...,
}
```

### Step 5: Verify Tests Pass

```bash
cargo test --package ash-parser test_role_def_with_capability_decl
# Expected: Pass
```

## Step 6: Code Review Sub-Process

Spawn a code review sub-agent to verify:

```
Review Focus for TASK-322A:
- Check that RoleDef struct change is minimal and correct
- Verify CapabilityDecl is the right type (from surface.rs, not redefined)
- Ensure no logic changes in this task (only AST)
- Check that temporary placeholders in dependent code are clearly marked
- Verify tests cover both constrained and unconstrained capability declarations
- Check for any breaking changes to public API
```

### Review Checklist (Rust-Specific)

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --package ash-parser` clean
- [ ] `cargo test --package ash-parser` all tests pass
- [ ] No new warnings in production code
- [ ] Public API changes documented (if any)

### Review Output

Reviewer should provide:
1. **Critical issues** (must fix before merge)
2. **Suggestions** (can be addressed or noted for follow-up)
3. **Approval** or **Request Changes**

## Files to Modify

- `crates/ash-parser/src/surface.rs` - Change `RoleDef` struct
- All files that construct `RoleDef` - Update field name (temporary placeholder values)

## Completion Checklist

- [ ] `RoleDef` has `capabilities: Vec<CapabilityDecl>` field
- [ ] Old `authority` field removed
- [ ] All compilation errors in dependent code fixed (with placeholders)
- [ ] Tests for new field structure pass
- [ ] `cargo check --package ash-parser` passes
- [ ] **Code review completed** with no critical issues
- [ ] Review feedback addressed (if any)

**Estimated Hours:** 1-2 (including review)
**Priority:** Blocking (foundation for TASK-322B)
**Blocked By:** None
**Blocks:** TASK-322B
