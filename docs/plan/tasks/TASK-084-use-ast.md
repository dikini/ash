# TASK-084: Use Statement AST Types

## Status: 🟢 Complete

## Description

Implement the Use statement AST type for import declarations.

## Specification Reference

- SPEC-012: Import System - Section 6 Grammar

## Requirements

### Functional Requirements

1. `Use` struct with:
   - `visibility: Visibility`
   - `path: UsePath` - the import path
   - `alias: Option<Box<str>>` - optional rename
   - `span: Span`

2. `UsePath` enum:
   - `Simple(SimplePath)` - single item
   - `Glob(SimplePath)` - `path::*`
   - `Nested(SimplePath, Vec<UseItem>)` - `path::{a, b}`

3. `UseItem` struct:
   - `name: Box<str>`
   - `alias: Option<Box<str>>`

### Property Requirements

```rust
// Simple use
Use { path: Simple(path), alias: None, .. }

// Use with alias
Use { path: Simple(path), alias: Some("alias"), .. }

// Glob use
Use { path: Glob(path), .. }

// Nested use
Use { path: Nested(path, items), .. }
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_use_simple() {
    let use_stmt = Use {
        visibility: Visibility::Inherited,
        path: UsePath::Simple(simple_path!("crate", "foo", "bar")),
        alias: None,
        span: Span::default(),
    };
    assert!(matches!(use_stmt.path, UsePath::Simple(_)));
}

#[test]
fn test_use_glob() {
    let use_stmt = Use {
        visibility: Visibility::Public,
        path: UsePath::Glob(simple_path!("crate", "foo")),
        alias: None,
        span: Span::default(),
    };
    assert!(matches!(use_stmt.path, UsePath::Glob(_)));
}

#[test]
fn test_use_nested() {
    let items = vec![
        UseItem { name: "a".into(), alias: None },
        UseItem { name: "b".into(), alias: Some("c".into()) },
    ];
    let use_stmt = Use {
        visibility: Visibility::Inherited,
        path: UsePath::Nested(simple_path!("crate", "foo"), items),
        alias: None,
        span: Span::default(),
    };
    assert!(matches!(use_stmt.path, UsePath::Nested(_, _)));
}
```

### Step 2: Implement Types (Green)

```rust
/// A use statement: `use path;` or `pub use path::*;`
#[derive(Debug, Clone, PartialEq)]
pub struct Use {
    pub visibility: Visibility,
    pub path: UsePath,
    pub alias: Option<Box<str>>,
    pub span: Span,
}

/// Different forms of use paths
#[derive(Debug, Clone, PartialEq)]
pub enum UsePath {
    /// `crate::foo::bar`
    Simple(SimplePath),
    /// `crate::foo::*`
    Glob(SimplePath),
    /// `crate::foo::{bar, baz as b}`
    Nested(SimplePath, Vec<UseItem>),
}

/// An item in a nested use: `bar` or `bar as baz`
#[derive(Debug, Clone, PartialEq)]
pub struct UseItem {
    pub name: Box<str>,
    pub alias: Option<Box<str>>,
}

/// Simple path without wildcards or nesting: `crate::foo::bar`
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SimplePath {
    pub segments: Vec<Box<str>>,
}
```

### Step 3: Refactor (Refactor)

- Ensure SimplePath has helper methods
- Add constructors for common cases

## Completion Checklist

- [ ] Use struct defined
- [ ] UsePath enum defined
- [ ] UseItem struct defined
- [ ] SimplePath struct defined
- [ ] Tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

3 hours

## Dependencies

- TASK-065 (Visibility AST)

## Blocked By

- TASK-065

## Blocks

- TASK-085 (Parse use statements)
- TASK-086 (Import resolution)
