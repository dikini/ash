# TASK-064: Module AST Types

## Status: 🟢 Complete

## Description

Implement the ModuleDecl AST type and related structures for the module system.

## Specification Reference

- SPEC-009: Module System - Section 2 Module Declaration

## Requirements

### Functional Requirements

1. `ModuleDecl` struct with: name, visibility, inline content, span
2. Add to surface AST module

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_module_decl_creation() {
    let decl = ModuleDecl {
        name: "foo".into(),
        visibility: Visibility::Inherited,
        inline: None,
        span: Span::default(),
    };
    assert_eq!(decl.name.as_ref(), "foo");
}
```

### Step 2: Implement Types (Green)

Create `crates/ash-parser/src/module.rs`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDecl {
    pub name: Box<str>,
    pub visibility: Visibility,
    pub inline: Option<Vec<Definition>>,
    pub span: Span,
}
```

## Completion Checklist

- [ ] ModuleDecl struct defined
- [ ] Tests pass

## Estimated Effort

4 hours

## Dependencies

None

## Blocked By

Nothing

## Blocks

- TASK-065 (Visibility AST)
- TASK-067 (Parse module declarations)
