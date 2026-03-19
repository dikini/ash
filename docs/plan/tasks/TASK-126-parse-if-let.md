# TASK-126: Parse If-Let Expressions

## Status: ✅ Complete

## Description

Add parser support for if-let expressions as syntactic sugar for match expressions.

## Specification Reference

- SPEC-020: ADT Types - Section 5.4

## Requirements

Parse syntax:
```ash
if let Some { value: x } = opt then {
    act log with x;
} else {
    act log with "No value";
}
```

If-let is sugar for:
```ash
match opt {
    Some { value: x } => { act log with x; },
    _ => { act log with "No value"; }
}
```

## TDD Steps

### Step 1: Write Failing Tests (Red)

**File**: `crates/ash-parser/src/parse_pattern.rs` (extend)

```rust
#[cfg(test)]
mod if_let_tests {
    use super::*;

    #[test]
    fn test_parse_if_let_simple() {
        let input = r#"if let Some { value: x } = opt then {
            x
        } else {
            0
        }"#;
        let result = parse_if_let_expr.parse_peek(Input::new(input));
        assert!(result.is_ok(), "Parse failed: {:?}", result);
    }

    #[test]
    fn test_parse_if_let_unit_variant() {
        let input = r#"if let None = opt then {
            act log with "none";
        } else {
            act log with "some";
        }"#;
        let result = parse_if_let_expr.parse_peek(Input::new(input));
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_if_let_nested() {
        let input = r#"if let Ok { value: Some { value: x } } = result then {
            x
        } else {
            0
        }"#;
        let result = parse_if_let_expr.parse_peek(Input::new(input));
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_if_let_with_block() {
        let input = r#"if let Some { value: x } = opt then {
            let doubled = x * 2;
            doubled
        } else {
            0
        }"#;
        let result = parse_if_let_expr.parse_peek(Input::new(input));
        assert!(result.is_ok());
        
        let (expr, _) = result.unwrap();
        match expr {
            Expr::IfLet { pattern, expr, then_branch, else_branch } => {
                // Verify structure
            }
            _ => panic!("Expected IfLet expression"),
        }
    }
}
```

### Step 2: Implement If-Let Parser (Green)

**File**: `crates/ash-parser/src/parse_pattern.rs`

```rust
/// Parse if-let expression
pub fn parse_if_let_expr(input: &mut Input) -> PResult<Expr> {
    use winnow::combinator::{seq, delimited};
    use winnow::token::literal;
    
    seq! {
        _: literal("if"),
        _: winnow::combinator::space1,
        _: literal("let"),
        _: winnow::combinator::space1,
        pattern: parse_pattern,
        _: winnow::combinator::space1,
        _: literal("="),
        _: winnow::combinator::space1,
        expr: parse_expr,
        _: winnow::combinator::space1,
        _: literal("then"),
        _: winnow::combinator::space0,
        then_branch: parse_block_or_expr,
        _: winnow::combinator::space0,
        _: literal("else"),
        _: winnow::combinator::space0,
        else_branch: parse_block_or_expr,
    }
    .map(|(pattern, expr, then_branch, else_branch)| Expr::IfLet {
        pattern,
        expr: Box::new(expr),
        then_branch: Box::new(then_branch),
        else_branch: Box::new(else_branch),
    })
    .parse_next(input)
}

/// Parse either a block `{ ... }` or a single expression
fn parse_block_or_expr(input: &mut Input) -> PResult<Expr> {
    use winnow::combinator::alt;
    
    alt((
        parse_block_expr,
        preceded(winnow::combinator::space0, parse_expr),
    ))
    .parse_next(input)
}

/// Parse a block expression `{ stmt; ...; expr }`
fn parse_block_expr(input: &mut Input) -> PResult<Expr> {
    use winnow::combinator::delimited;
    use winnow::token::literal;
    
    delimited(
        (literal("{"), winnow::combinator::space0),
        parse_expr,  // Simplified - actual block parsing would handle multiple statements
        (winnow::combinator::space0, literal("}")),
    )
    .parse_next(input)
}
```

### Step 3: Add Expr::IfLet to AST (Green)

**File**: `crates/ash-core/src/ast.rs`

Ensure `Expr` enum has `IfLet` variant:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    // Existing variants...
    
    /// If-let expression (sugar for match)
    IfLet {
        pattern: Pattern,
        expr: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
}
```

### Step 4: Integrate into Expression Parser (Green)

**File**: `crates/ash-parser/src/parse_expr.rs`

Add `if let` to expression parser:

```rust
fn parse_primary_expr(input: &mut Input) -> PResult<Expr> {
    use winnow::combinator::alt;
    
    alt((
        parse_if_let_expr,  // Add this
        parse_match_expr,
        parse_let_expr,
        parse_literal_expr,
        parse_variable_expr,
        // ... others
    ))
    .parse_next(input)
}
```

### Step 5: Run Tests

```bash
cargo test -p ash-parser if_let -- --nocapture
```

### Step 6: Add Property Tests

```rust
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_if_let_roundtrip(
            pattern_str in "[A-Z][a-zA-Z0-9]*\\s*\\{[^}]*\\}",
            expr_str in "[a-z][a-zA-Z0-9]*"
        ) {
            let input = format!("if let {} = {} then {{ 1 }} else {{ 0 }}", pattern_str, expr_str);
            let result = parse_if_let_expr.parse_peek(Input::new(&input));
            prop_assert!(result.is_ok());
        }
    }
}
```

### Step 7: Commit

```bash
git add crates/ash-parser/src/parse_pattern.rs crates/ash-core/src/ast.rs
git commit -m "feat(parser): parse if-let expressions (TASK-126)"
```

## Completion Checklist

- [ ] `parse_if_let_expr` function implemented
- [ ] Handles pattern matching with `if let PAT = EXPR then { ... } else { ... }`
- [ ] `Expr::IfLet` variant in AST
- [ ] `parse_block_or_expr` helper for branches
- [ ] Integration into expression parser
- [ ] Unit tests for simple if-let
- [ ] Unit tests for nested patterns
- [ ] Unit tests for blocks in branches
- [ ] Property tests for roundtrip
- [ ] Error handling for malformed if-let
- [ ] `cargo fmt` and `cargo clippy` pass

## Estimated Effort

3 hours

## Dependencies

- TASK-125 (Parse Match Expressions)

## Blocked By

- TASK-125

## Blocks

- TASK-133 (Match Evaluation - for desugaring)
