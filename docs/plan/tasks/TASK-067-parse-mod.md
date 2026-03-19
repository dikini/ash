# TASK-067: Parse Module Declarations

## Status: ✅ Complete

## Description

Implement parser for module declarations: `mod name;` and `mod name { ... }`.

## Specification Reference

- SPEC-009: Module System - Section 2 Module Declaration

## Requirements

### Functional Requirements

1. Parse `mod name;` as file-based module
2. Parse `mod name { ... }` as inline module
3. Support visibility modifiers: `pub mod name;`
4. Parse definitions inside inline modules

### Property Requirements

```rust
// File-based module
parse_module("mod foo;") -> ModuleDecl { name: "foo", inline: None }

// Inline module
parse_module("mod foo {}") -> ModuleDecl { name: "foo", inline: Some([]) }

// With visibility
parse_module("pub mod foo;") -> visibility: Public
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_parse_file_module() {
    let mut input = new_input("mod foo;");
    let decl = parse_module_decl(&mut input).unwrap();
    assert_eq!(decl.name.as_ref(), "foo");
    assert!(decl.inline.is_none());
}

#[test]
fn test_parse_inline_module() {
    let input = "mod foo { capability c: observe(); }";
    let mut parse_input = new_input(input);
    let decl = parse_module_decl(&mut parse_input).unwrap();
    assert_eq!(decl.name.as_ref(), "foo");
    assert!(decl.inline.is_some());
}

#[test]
fn test_parse_pub_module() {
    let mut input = new_input("pub mod foo;");
    let decl = parse_module_decl(&mut input).unwrap();
    assert_eq!(decl.visibility, Visibility::Public);
}
```

### Step 2: Implement Parser (Green)

```rust
pub fn parse_module_decl(input: &mut ParseInput) -> ModalResult<ModuleDecl> {
    let start = input.state.clone();
    
    let visibility = parse_visibility(input)?;
    skip_whitespace(input);
    
    keyword("mod").parse_next(input)?;
    skip_whitespace(input);
    
    let name = identifier(input)?;
    skip_whitespace(input);
    
    let inline = if literal("{").parse_next(input).is_ok() {
        // Inline module
        let mut defs = Vec::new();
        loop {
            skip_whitespace(input);
            if literal("}").parse_next(input).is_ok() {
                break;
            }
            if input.input.is_empty() {
                return Err(winnow::error::ErrMode::Backtrack(
                    winnow::error::ContextError::new(),
                ));
            }
            defs.push(parse_definition(input)?);
        }
        Some(defs)
    } else {
        literal(";").parse_next(input)?;
        None
    };
    
    Ok(ModuleDecl {
        name: name.into(),
        visibility,
        inline,
        span: span_from(&start, &input.state),
    })
}
```

## Completion Checklist

- [ ] File-based modules parse
- [ ] Inline modules parse
- [ ] Visibility modifiers work
- [ ] Nested definitions parsed
- [ ] Tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

6 hours

## Dependencies

- TASK-065 (Visibility AST)
- TASK-066 (Parse visibility)

## Blocked By

- TASK-066

## Blocks

- TASK-069 (Module resolver)
