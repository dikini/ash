# TASK-065: Visibility AST Types

## Status: ✅ Complete

## Description

Implement the Visibility enum for pub, pub(crate), pub(super), etc.

## Specification Reference

- SPEC-009: Module System - Section 3 Visibility

## Requirements

### Functional Requirements

1. `Visibility` enum with: Inherited, Public, Crate, Super, Self_, Restricted
2. Helper methods: is_pub(), is_visible_in_module()

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_visibility_private() {
    assert!(!Visibility::Inherited.is_pub());
}

#[test]
fn test_visibility_pub() {
    assert!(Visibility::Public.is_pub());
}
```

### Step 2: Implement (Green)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    #[default]
    Inherited,
    Public,
    Crate,
    Super,
    Self_,
    Restricted { path: Box<str> },
}
```

## Completion Checklist

- [ ] Visibility enum defined
- [ ] Helper methods implemented
- [ ] Tests pass

## Estimated Effort

4 hours

## Dependencies

None

## Blocked By

Nothing

## Blocks

- TASK-066 (Parse visibility)
- TASK-070 (Visibility checking)
