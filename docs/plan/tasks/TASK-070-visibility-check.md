# TASK-070: Visibility Checking

## Status: 🟢 Complete

## Description

Implement visibility checking in the type checker to enforce access control.

## Specification Reference

- SPEC-009: Module System - Section 7 Visibility Checking

## Requirements

### Functional Requirements

1. Check if item access is allowed from calling module
2. Support all visibility levels: pub, pub(crate), pub(super), restricted
3. Report visibility errors during type checking

### Property Requirements

```rust
// Public items are always accessible
is_accessible(Visibility::Public, from, owner) == true

// Private items only in same module
is_accessible(Visibility::Inherited, "crate::foo", "crate::foo") == true
is_accessible(Visibility::Inherited, "crate::foo", "crate::bar") == false

// Crate visibility in same crate
is_accessible(Visibility::Crate, "crate::foo", "crate::bar") == true
is_accessible(Visibility::Crate, "external::foo", "crate::bar") == false
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_visibility_check_public() {
    assert!(check_visibility(Visibility::Public, "anywhere", "owner"));
}

#[test]
fn test_visibility_check_private_same() {
    assert!(check_visibility(Visibility::Inherited, "crate::foo", "crate::foo"));
}

#[test]
fn test_visibility_check_private_other() {
    assert!(!check_visibility(Visibility::Inherited, "crate::foo", "crate::bar"));
}

#[test]
fn test_visibility_check_crate() {
    assert!(check_visibility(Visibility::Crate, "crate::foo", "crate::bar"));
    assert!(!check_visibility(Visibility::Crate, "external::foo", "crate::bar"));
}
```

### Step 2: Implement (Green)

```rust
pub fn check_visibility(vis: Visibility, from_module: &str, owner_module: &str) -> bool {
    vis.is_visible_in_module(from_module, owner_module)
}

// In type checker, when resolving names:
if !check_visibility(item.visibility, current_module, item.module) {
    return Err(TypeError::VisibilityViolation { ... });
}
```

## Completion Checklist

- [ ] Visibility check function
- [ ] Integration with type checker
- [ ] Error messages for violations
- [ ] Tests for all visibility levels
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

6 hours

## Dependencies

- TASK-065 (Visibility AST)
- TASK-069 (Module resolver)

## Blocked By

- TASK-069

## Blocks

None (completes Phase 10)
