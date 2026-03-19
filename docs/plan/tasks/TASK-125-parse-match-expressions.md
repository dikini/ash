# TASK-125: Parse Match Expressions

## Status: 🟡 Ready to Start

## Description

Add parser support for match expressions and pattern syntax for ADTs.

## Specification Reference

- SPEC-020: ADT Types - Section 5.3, 5.4

## Requirements

Parse syntax:
```ash
match opt {
    Some { value: x } => x * 2,
    None => 0
}

if let Some { value: x } = opt then {
    act log with x;
}
```

## TDD Steps

### Step 1: Extend Pattern AST (Red)

**File**: `crates/ash-core/src/ast.rs`

Update Pattern enum:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Variable(Name),
    Tuple(Vec<Pattern>),
    Record(Vec<(Name, Pattern)>),
    List(Vec<Pattern>, Option<Name>),
    Wildcard,
    Literal(Value),
    
    /// Variant pattern: Some { value: x } or just Some (unit variant)
    Variant {
        name: Name,
        fields: Option<Vec<(Name, Pattern)>>,  // None for unit variants
    },
}
```

Add MatchArm structure:

```rust
/// Match arm: pattern => expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
}

/// Match expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    // Existing...
    
    /// Match expression
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    
    /// If-let expression (sugar for match)
    IfLet {
        pattern: Pattern,
        expr: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
}
```

### Step 2: Create Parser Module (Red)

**File**: `crates/ash-parser/src/parse_pattern.rs` (extend)

```rust
//! Parser for patterns and match expressions

use ash_core::ast::{Pattern, MatchArm, Expr};
use winnow::{PResult, Parser};

use crate::Input;

/// Parse a pattern
pub fn parse_pattern(input: &mut Input) -> PResult<Pattern> {
    todo!()
}

/// Parse a match expression
pub fn parse_match_expr(input: &mut Input) -> PResult<Expr> {
    todo!()
}

/// Parse if-let expression
pub fn parse_if_let_expr(input: &mut Input) -> PResult<Expr> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_variant_pattern() {
        let input = "Some { value: x }";
        let result = parse_pattern.parse_peek(Input::new(input));
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_unit_variant_pattern() {
        let input = "None";
        let result = parse_pattern.parse_peek(Input::new(input));
        assert!(result.is_ok());
        
        let (pat, _) = result.unwrap();
        match pat {
            Pattern::Variant { name, fields: None } => {
                assert_eq!(name, "None");
            }
            _ => panic!("Expected unit variant pattern"),
        }
    }

    #[test]
    fn test_parse_match_expr() {
        let input = r#"match opt {
            Some { value: x } => x * 2,
            None => 0
        }"#;
        let result = parse_match_expr.parse_peek(Input::new(input));
        assert!(result.is_ok());
        
        let (expr, _) = result.unwrap();
        match expr {
            Expr::Match { arms, .. } => {
                assert_eq!(arms.len(), 2);
            }
            _ => panic!("Expected match expression"),
        }
    }

    #[test]
    fn test_parse_if_let() {
        let input = r#"if let Some { value: x } = opt then {
            x
        } else {
            0
        }"#;
        let result = parse_if_let_expr.parse_peek(Input::new(input));
        assert!(result.is_ok());
    }
}
```

### Step 3: Implement Pattern Parser (Green)

**File**: `crates/ash-parser/src/parse_pattern.rs`

```rust
use winnow::combinator::{alt, separated, opt, delimited, preceded};
use winnow::token::literal;

/// Parse a pattern
pub fn parse_pattern(input: &mut Input) -> PResult<Pattern> {
    alt((
        parse_wildcard,
        parse_literal_pattern,
        parse_tuple_pattern,
        parse_record_pattern,
        parse_variant_pattern,
        parse_variable_pattern,
    )).parse_next(input)
}

fn parse_wildcard(input: &mut Input) -> PResult<Pattern> {
    literal("_").map(|_| Pattern::Wildcard).parse_next(input)
}

fn parse_variable_pattern(input: &mut Input) -> PResult<Pattern> {
    // Variable starts with lowercase
    parse_ident_lowercase.map(|name| Pattern::Variable(name.to_string())).parse_next(input)
}

fn parse_literal_pattern(input: &mut Input) -> PResult<Pattern> {
    // Reuse literal parser from expression parser
    parse_literal.map(Pattern::Literal).parse_next(input)
}

fn parse_tuple_pattern(input: &mut Input) -> PResult<Pattern> {
    let patterns = delimited(
        (literal("("), winnow::combinator::space0),
        separated(0.., parse_pattern, (literal(","), winnow::combinator::space0)),
        (winnow::combinator::space0, literal(")")),
    ).parse_next(input)?;
    
    Ok(Pattern::Tuple(patterns))
}

fn parse_record_pattern(input: &mut Input) -> PResult<Pattern> {
    let fields = delimited(
        (literal("{"), winnow::combinator::space0),
        separated(0.., parse_field_pattern, (literal(","), winnow::combinator::space0)),
        (winnow::combinator::space0, literal("}")),
    ).parse_next(input)?;
    
    Ok(Pattern::Record(fields))
}

fn parse_field_pattern(input: &mut Input) -> PResult<(String, Pattern)> {
    let name = parse_ident_lowercase.parse_next(input)?;
    
    // Shorthand: `{ x }` is sugar for `{ x: x }`
    let pattern = opt(preceded(
        (winnow::combinator::space0, literal(":"), winnow::combinator::space0),
        parse_pattern
    )).parse_next(input)?;
    
    let pattern = match pattern {
        Some(p) => p,
        None => Pattern::Variable(name.to_string()),
    };
    
    Ok((name.to_string(), pattern))
}

fn parse_variant_pattern(input: &mut Input) -> PResult<Pattern> {
    // Variant name starts with uppercase
    let name = parse_ident_uppercase.parse_next(input)?;
    
    // Optional fields
    let fields = opt(delimited(
        (literal("{"), winnow::combinator::space0),
        separated(0.., parse_field_pattern, (literal(","), winnow::combinator::space0)),
        (winnow::combinator::space0, literal("}")),
    )).parse_next(input)?;
    
    Ok(Pattern::Variant {
        name: name.to_string(),
        fields,
    })
}
```

### Step 4: Implement Match Expression Parser (Green)

```rust
/// Parse match expression
pub fn parse_match_expr(input: &mut Input) -> PResult<Expr> {
    literal("match").parse_next(input)?;
    winnow::combinator::space1.parse_next(input)?;
    
    let scrutinee = parse_expr.parse_next(input)?;
    
    winnow::combinator::space0.parse_next(input)?;
    let arms = delimited(
        (literal("{"), winnow::combinator::space0),
        separated(1.., parse_match_arm, (literal(","), winnow::combinator::opt(winnow::combinator::space0))),
        (winnow::combinator::space0, literal("}")),
    ).parse_next(input)?;
    
    Ok(Expr::Match {
        scrutinee: Box::new(scrutinee),
        arms,
    })
}

fn parse_match_arm(input: &mut Input) -> PResult<MatchArm> {
    let pattern = parse_pattern.parse_next(input)?;
    
    winnow::combinator::space0.parse_next(input)?;
    literal("=>").parse_next(input)?;
    winnow::combinator::space0.parse_next(input)?;
    
    let body = parse_expr.parse_next(input)?;
    
    Ok(MatchArm { pattern, body })
}
```

### Step 5: Implement If-Let Parser (Green)

```rust
/// Parse if-let expression
pub fn parse_if_let_expr(input: &mut Input) -> PResult<Expr> {
    literal("if").parse_next(input)?;
    winnow::combinator::space1.parse_next(input)?;
    literal("let").parse_next(input)?;
    winnow::combinator::space1.parse_next(input)?;
    
    let pattern = parse_pattern.parse_next(input)?;
    
    winnow::combinator::space1.parse_next(input)?;
    literal("=").parse_next(input)?;
    winnow::combinator::space1.parse_next(input)?;
    
    let expr = parse_expr.parse_next(input)?;
    
    winnow::combinator::space1.parse_next(input)?;
    literal("then").parse_next(input)?;
    
    let then_branch = parse_block_or_expr.parse_next(input)?;
    
    winnow::combinator::space0.parse_next(input)?;
    literal("else").parse_next(input)?;
    
    let else_branch = parse_block_or_expr.parse_next(input)?;
    
    Ok(Expr::IfLet {
        pattern,
        expr: Box::new(expr),
        then_branch: Box::new(then_branch),
        else_branch: Box::new(else_branch),
    })
}

fn parse_block_or_expr(input: &mut Input) -> PResult<Expr> {
    alt((
        parse_block_expr,
        preceded(winnow::combinator::space0, parse_expr),
    )).parse_next(input)
}
```

### Step 6: Run Tests

```bash
cargo test -p ash-parser parse_pattern -- --nocapture
```

## Completion Checklist

- [ ] Pattern parsing for all pattern types
- [ ] Variant patterns with fields
- [ ] Unit variant patterns
- [ ] Tuple patterns
- [ ] Record patterns with shorthand
- [ ] Match expression parsing
- [ ] If-let expression parsing
- [ ] Integration into expression parser
- [ ] Unit tests for all pattern types
- [ ] Error handling for malformed patterns
- [ ] `cargo fmt` and `cargo clippy` pass

## Estimated Effort

5 hours

## Dependencies

- TASK-124 (Parse Type Definitions)

## Blocked By

- TASK-124

## Blocks

- TASK-128 (Pattern Typing)
- TASK-132 (Pattern Matching Engine)
