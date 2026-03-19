# TASK-124: Parse Type Definitions

## Status: ✅ Complete

## Description

Add parser support for type definitions including enums, structs, and type aliases.

## Specification Reference

- SPEC-020: ADT Types - Section 5.1, 5.2

## Requirements

Parse syntax:
```ash
type Status = Pending | Processing { started_at: Time } | Completed;
type Point = { x: Int, y: Int };
type Option<T> = Some { value: T } | None;
pub type Result<T, E> = Ok { value: T } | Err { error: E };
```

## TDD Steps

### Step 1: Create Parser Module (Red)

**File**: `crates/ash-parser/src/parse_type_def.rs` (new)

```rust
//! Parser for type definitions

use ash_core::ast::{TypeDef, TypeBody, VariantDef, Visibility, TypeExpr};
use winnow::{PResult, Parser};
use winnow::combinator::{separated, alt, opt, preceded, terminated};
use winnow::token::literal;

use crate::{Input, lexer::Token, parse_type};

/// Parse a type definition: `type Name = Body;`
pub fn parse_type_def(input: &mut Input) -> PResult<TypeDef> {
    // TODO: Implement
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_enum() {
        let input = "type Status = Pending | Processing | Completed;";
        let result = parse_type_def.parse_peek(Input::new(input));
        assert!(result.is_ok());
        
        let (def, _) = result.unwrap();
        assert_eq!(def.name, "Status");
        match def.body {
            TypeBody::Enum(variants) => {
                assert_eq!(variants.len(), 3);
                assert_eq!(variants[0].name, "Pending");
            }
            _ => panic!("Expected enum"),
        }
    }

    #[test]
    fn test_parse_enum_with_fields() {
        let input = r#"type Result = Success { value: Int } | Failure { error: String };"#;
        let result = parse_type_def.parse_peek(Input::new(input));
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_generic_enum() {
        let input = "type Option<T> = Some { value: T } | None;";
        let result = parse_type_def.parse_peek(Input::new(input));
        assert!(result.is_ok());
        
        let (def, _) = result.unwrap();
        assert_eq!(def.params.len(), 1);
        assert_eq!(def.params[0].0, "T");
    }

    #[test]
    fn test_parse_struct() {
        let input = "type Point = { x: Int, y: Int };";
        let result = parse_type_def.parse_peek(Input::new(input));
        assert!(result.is_ok());
        
        let (def, _) = result.unwrap();
        match def.body {
            TypeBody::Struct(fields) => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "x");
            }
            _ => panic!("Expected struct"),
        }
    }

    #[test]
    fn test_parse_pub_type() {
        let input = "pub type Inner = Int;";
        let result = parse_type_def.parse_peek(Input::new(input));
        assert!(result.is_ok());
        
        let (def, _) = result.unwrap();
        assert!(matches!(def.visibility, Visibility::Public));
    }
}
```

### Step 2: Implement Type Definition Parser (Green)

**File**: `crates/ash-parser/src/parse_type_def.rs`

```rust
use ash_core::ast::{TypeDef, TypeBody, VariantDef, Visibility, TypeExpr};
use winnow::{PResult, Parser};
use winnow::combinator::{separated, alt, opt, preceded, terminated, delimited};
use winnow::token::{literal, take_while};

use crate::{Input, lexer::Token};

/// Parse visibility: `pub` or nothing
fn parse_visibility(input: &mut Input) -> PResult<Visibility> {
    use winnow::combinator::dispatch;
    
    dispatch! { literal;
        "pub" => Visibility::Public,
        _ => Visibility::Private,
    }
    .parse_next(input)
}

/// Parse type parameters: `<T, U>` or nothing
fn parse_type_params(input: &mut Input) -> PResult<Vec<Box<str>>> {
    use winnow::combinator::cut_err;
    
    let params = delimited(
        (literal("<"), winnow::combinator::space0),
        separated(
            1..,
            parse_type_ident,
            (literal(","), winnow::combinator::space0)
        ),
        (winnow::combinator::space0, literal(">")),
    )
    .parse_next(input)?;
    
    Ok(params)
}

/// Parse a type identifier (uppercase start)
fn parse_type_ident(input: &mut Input) -> PResult<Box<str>> {
    take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_')
        .verify(|s: &str| s.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false))
        .map(|s: &str| s.into())
        .parse_next(input)
}

/// Parse a type definition
pub fn parse_type_def(input: &mut Input) -> PResult<TypeDef> {
    let visibility = opt(parse_visibility).map(|v| v.unwrap_or(Visibility::Private)).parse_next(input)?;
    
    literal("type").parse_next(input)?;
    winnow::combinator::space1.parse_next(input)?;
    
    let name = parse_type_ident.parse_next(input)?;
    let params = opt(parse_type_params).map(|p| p.unwrap_or_default()).parse_next(input)?;
    
    winnow::combinator::space0.parse_next(input)?;
    literal("=").parse_next(input)?;
    winnow::combinator::space0.parse_next(input)?;
    
    let body = parse_type_body.parse_next(input)?;
    
    winnow::combinator::space0.parse_next(input)?;
    literal(";").parse_next(input)?;
    
    Ok(TypeDef {
        name: name.to_string(),
        params,
        body,
        visibility,
    })
}

/// Parse the body of a type definition
fn parse_type_body(input: &mut Input) -> PResult<TypeBody> {
    alt((
        parse_enum_body,
        parse_struct_body,
        parse_alias_body,
    )).parse_next(input)
}

/// Parse enum body: `Variant1 | Variant2 { field: Type }`
fn parse_enum_body(input: &mut Input) -> PResult<TypeBody> {
    let variants = separated(
        1..,
        parse_variant,
        (winnow::combinator::space0, literal("|"), winnow::combinator::space0)
    ).parse_next(input)?;
    
    Ok(TypeBody::Enum(variants))
}

/// Parse a single variant
fn parse_variant(input: &mut Input) -> PResult<VariantDef> {
    let name = parse_type_ident.parse_next(input)?;
    
    let fields = opt(delimited(
        (literal("{"), winnow::combinator::space0),
        separated(
            0..,
            parse_field,
            (literal(","), winnow::combinator::space0)
        ),
        (winnow::combinator::space0, literal("}")),
    )).map(|f| f.unwrap_or_default()).parse_next(input)?;
    
    Ok(VariantDef {
        name: name.to_string(),
        fields,
    })
}

/// Parse struct body: `{ field: Type, ... }`
fn parse_struct_body(input: &mut Input) -> PResult<TypeBody> {
    let fields = delimited(
        (literal("{"), winnow::combinator::space0),
        separated(
            0..,
            parse_field,
            (literal(","), winnow::combinator::space0)
        ),
        (winnow::combinator::space0, literal("}")),
    ).parse_next(input)?;
    
    Ok(TypeBody::Struct(fields))
}

/// Parse a field: `name: Type`
fn parse_field(input: &mut Input) -> PResult<(String, TypeExpr)> {
    let name = take_while(1.., |c: char| c.is_ascii_lowercase() || c == '_')
        .parse_next(input)?;
    
    winnow::combinator::space0.parse_next(input)?;
    literal(":").parse_next(input)?;
    winnow::combinator::space0.parse_next(input)?;
    
    let ty = parse_type_expr.parse_next(input)?;
    
    Ok((name.to_string(), ty))
}

/// Parse type expression
fn parse_type_expr(input: &mut Input) -> PResult<TypeExpr> {
    alt((
        parse_constructor_expr,
        parse_named_type,
    )).parse_next(input)
}

/// Parse named type: `Int`, `String`, `T`
fn parse_named_type(input: &mut Input) -> PResult<TypeExpr> {
    parse_type_ident.map(|name| TypeExpr::Named(name.to_string())).parse_next(input)
}

/// Parse constructor: `Option<T>`, `Result<Int, String>`
fn parse_constructor_expr(input: &mut Input) -> PResult<TypeExpr> {
    let name = parse_type_ident.parse_next(input)?;
    
    let args = opt(delimited(
        (literal("<"), winnow::combinator::space0),
        separated(
            1..,
            parse_type_expr,
            (literal(","), winnow::combinator::space0)
        ),
        (winnow::combinator::space0, literal(">")),
    )).map(|a| a.unwrap_or_default()).parse_next(input)?;
    
    Ok(TypeExpr::Constructor {
        name: name.to_string(),
        args,
    })
}

/// Parse alias body: just a type expression
fn parse_alias_body(input: &mut Input) -> PResult<TypeBody> {
    parse_type_expr.map(TypeBody::Alias).parse_next(input)
}
```

### Step 3: Integrate into Module Parser

**File**: `crates/ash-parser/src/lib.rs`

Add module:
```rust
pub mod parse_type_def;
```

Update module parsing to include type definitions:

```rust
// In parse_module.rs or similar
use crate::parse_type_def::parse_type_def;

fn parse_module_item(input: &mut Input) -> PResult<Definition> {
    alt((
        parse_type_def.map(Definition::TypeDef),
        parse_workflow.map(Definition::Workflow),
        // ... etc
    )).parse_next(input)
}
```

### Step 4: Run Tests

```bash
cargo test -p ash-parser parse_type_def -- --nocapture
```

### Step 5: Add Property Tests

```rust
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_parse_roundtrip(def in arbitrary_type_def()) {
            let source = type_def_to_string(&def);
            let parsed = parse_type_def.parse_peek(Input::new(&source));
            assert!(parsed.is_ok());
            
            let (result, _) = parsed.unwrap();
            assert_eq!(result.name, def.name);
        }
    }
}
```

## Completion Checklist

- [ ] `parse_type_def` function implemented
- [ ] Handles `pub` visibility
- [ ] Handles type parameters `<T, U>`
- [ ] Parses enum definitions with variants and fields
- [ ] Parses struct definitions
- [ ] Parses type aliases
- [ ] Integrated into module parser
- [ ] Unit tests for all syntax variants
- [ ] Property tests for roundtrip parsing
- [ ] Error handling for malformed input
- [ ] `cargo fmt` and `cargo clippy` pass

## Estimated Effort

6 hours

## Dependencies

- TASK-121 (ADT Core Types)

## Blocked By

- TASK-121

## Blocks

- TASK-125 (Parse Match Expressions)
- TASK-127 (Constructor Typing)
