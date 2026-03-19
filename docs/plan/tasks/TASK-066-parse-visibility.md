# TASK-066: Parse Visibility Modifiers

## Status: ✅ Complete

## Description

Implement parser for visibility modifiers: pub, pub(crate), pub(super), pub(in path).

## Specification Reference

- SPEC-009: Module System - Section 3 Visibility

## Requirements

### Functional Requirements

1. Parse `pub` → Visibility::Public
2. Parse `pub(crate)` → Visibility::Crate
3. Parse `pub(super)` → Visibility::Super
4. Parse `pub(self)` → Visibility::Self_
5. Parse `pub(in path::to::module)` → Visibility::Restricted
6. Parse absence of visibility → Visibility::Inherited

### Property Requirements

```rust
// Roundtrip for simple visibility
parse_visibility("pub") == Visibility::Public

// All visibility forms parse
parse_visibility("pub(crate)").is_ok()
parse_visibility("pub(super)").is_ok()
parse_visibility("pub(in crate::foo)").is_ok()

// No visibility = inherited
parse_visibility("") == Visibility::Inherited
```

## TDD Steps

### Step 1: Write Tests (Red)

Create parser tests:

```rust
#[test]
fn test_parse_pub() {
    let mut input = new_input("pub");
    assert_eq!(parse_visibility(&mut input).unwrap(), Visibility::Public);
}

#[test]
fn test_parse_pub_crate() {
    let mut input = new_input("pub(crate)");
    assert_eq!(parse_visibility(&mut input).unwrap(), Visibility::Crate);
}

#[test]
fn test_parse_pub_super() {
    let mut input = new_input("pub(super)");
    assert_eq!(parse_visibility(&mut input).unwrap(), Visibility::Super);
}

#[test]
fn test_parse_inherited() {
    let mut input = new_input("");
    assert_eq!(parse_visibility(&mut input).unwrap(), Visibility::Inherited);
}
```

### Step 2: Implement Parser (Green)

```rust
use winnow::combinator::{delimited, opt, preceded};
use winnow::token::literal;

pub fn parse_visibility(input: &mut ParseInput) -> ModalResult<Visibility> {
    if literal("pub").parse_next(input).is_ok() {
        // Check for restricted forms
        let restriction = opt(delimited(
            literal("("),
            alt((
                literal("crate").map(|_| Visibility::Crate),
                literal("super").map(|_| Visibility::Super),
                literal("self").map(|_| Visibility::Self_),
                preceded(literal("in"), parse_module_path)
                    .map(|path| Visibility::Restricted { path: path.into() }),
            )),
            literal(")"),
        )).parse_next(input)?;
        
        return Ok(restriction.unwrap_or(Visibility::Public));
    }
    
    Ok(Visibility::Inherited)
}
```

### Step 3: Refactor (Refactor)

- Extract helper for restricted visibility
- Ensure good error messages

## Completion Checklist

- [ ] All visibility forms parse correctly
- [ ] Tests for each form
- [ ] Error handling for invalid visibility
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

4 hours

## Dependencies

- TASK-065 (Visibility AST)

## Blocked By

- TASK-065

## Blocks

- TASK-067 (Parse module declarations)
