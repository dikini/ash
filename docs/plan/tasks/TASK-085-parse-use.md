# TASK-085: Parse Use Statements

## Status: ✅ Complete

## Description

Implement parser for use statements with all forms.

## Specification Reference

- SPEC-012: Import System - Section 2 Import Syntax

## Requirements

### Functional Requirements

1. Parse `use path::to::item;`
2. Parse `use path::to::item as alias;`
3. Parse `use path::*;`
4. Parse `use path::{a, b, c};`
5. Parse `use path::{a as x, b};`
6. Parse `pub use ...` with visibility

### Property Requirements

```rust
// All forms parse correctly
parse_use("use crate::foo::bar;").is_ok()
parse_use("use crate::foo::bar as baz;").is_ok()
parse_use("use crate::foo::*;").is_ok()
parse_use("use crate::foo::{a, b};").is_ok()
parse_use("pub use crate::foo::bar;").is_ok()
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_parse_use_simple() {
    let mut input = new_input("use crate::foo::bar;");
    let use_stmt = parse_use(&mut input).unwrap();
    assert!(matches!(use_stmt.path, UsePath::Simple(_)));
}

#[test]
fn test_parse_use_alias() {
    let mut input = new_input("use crate::foo::bar as baz;");
    let use_stmt = parse_use(&mut input).unwrap();
    assert_eq!(use_stmt.alias.as_ref().map(|s| s.as_ref()), Some("baz"));
}

#[test]
fn test_parse_use_glob() {
    let mut input = new_input("use crate::foo::*;");
    let use_stmt = parse_use(&mut input).unwrap();
    assert!(matches!(use_stmt.path, UsePath::Glob(_)));
}

#[test]
fn test_parse_use_nested() {
    let mut input = new_input("use crate::foo::{a, b as c};");
    let use_stmt = parse_use(&mut input).unwrap();
    if let UsePath::Nested(_, items) = use_stmt.path {
        assert_eq!(items.len(), 2);
        assert_eq!(items[1].alias.as_ref().map(|s| s.as_ref()), Some("c"));
    } else {
        panic!("Expected nested use");
    }
}

#[test]
fn test_parse_pub_use() {
    let mut input = new_input("pub use crate::foo::bar;");
    let use_stmt = parse_use(&mut input).unwrap();
    assert_eq!(use_stmt.visibility, Visibility::Public);
}
```

### Step 2: Implement Parser (Green)

```rust
pub fn parse_use(input: &mut ParseInput) -> ModalResult<Use> {
    let start = input.state.clone();
    
    let visibility = parse_visibility(input)?;
    skip_whitespace(input);
    
    keyword("use").parse_next(input)?;
    skip_whitespace(input);
    
    // Parse the path
    let path = parse_simple_path(input)?;
    
    // Check for glob, nested, or alias
    skip_whitespace(input);
    
    let (use_path, alias) = if literal("::").parse_next(input).is_ok() {
        skip_whitespace(input);
        
        if literal("*").parse_next(input).is_ok() {
            // Glob import
            (UsePath::Glob(path), None)
        } else if literal("{").parse_next(input).is_ok() {
            // Nested import
            let items = parse_use_items(input)?;
            literal("}").parse_next(input)?;
            (UsePath::Nested(path, items), None)
        } else {
            return Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ));
        }
    } else {
        // Simple import - check for alias
        let alias = if keyword("as").parse_next(input).is_ok() {
            skip_whitespace(input);
            Some(identifier(input)?.into())
        } else {
            None
        };
        (UsePath::Simple(path), alias)
    };
    
    skip_whitespace(input);
    literal(";").parse_next(input)?;
    
    Ok(Use {
        visibility,
        path: use_path,
        alias,
        span: span_from(&start, &input.state),
    })
}

fn parse_use_items(input: &mut ParseInput) -> ModalResult<Vec<UseItem>> {
    let mut items = Vec::new();
    
    loop {
        skip_whitespace(input);
        
        if literal("}").parse_peek(input).is_ok() {
            break;
        }
        
        let name = identifier(input)?.into();
        
        skip_whitespace(input);
        let alias = if keyword("as").parse_next(input).is_ok() {
            skip_whitespace(input);
            Some(identifier(input)?.into())
        } else {
            None
        };
        
        items.push(UseItem { name, alias });
        
        skip_whitespace(input);
        if literal(",").parse_next(input).is_err() {
            break;
        }
    }
    
    Ok(items)
}
```

### Step 3: Refactor (Refactor)

- Extract path parsing for reuse
- Ensure good error messages

## Completion Checklist

- [ ] Simple use parses
- [ ] Use with alias parses
- [ ] Glob use parses
- [ ] Nested use parses
- [ ] Pub use parses
- [ ] Tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

4 hours

## Dependencies

- TASK-084 (Use AST)
- TASK-066 (Parse visibility)

## Blocked By

- TASK-084

## Blocks

- TASK-086 (Import resolution)
