# TASK-014: Expression Parser

## Status: ✅ Complete

## Description

Implement parsers for expressions with correct operator precedence and associativity.

## Specification Reference

- SPEC-002: Surface Language - Section 3.6 Expressions

## Requirements

### Operator Precedence (Highest to Lowest)

1. Primary: literals, variables, field access, index access, calls, parentheses
2. Unary: `-`, `not`, `#` (length), `empty?`
3. Multiplicative: `*`, `/`
4. Additive: `+`, `-`
5. Comparison: `=`, `!=`, `<`, `>`, `<=`, `>=`
6. Membership: `in`
7. Logical AND: `and`
8. Logical OR: `or`
9. Ternary: `? :`

### Expression Parsers

```rust
/// Entry point for expression parsing
pub fn expr<'a>(input: &mut ParseInput<'a>) -> PResult<Expr, ParseError> {
    ternary_expr.parse_next(input)
}

/// Ternary: <or_expr> (? <expr> : <expr>)?
pub fn ternary_expr<'a>(input: &mut ParseInput<'a>) -> PResult<Expr, ParseError> {
    let start = *input;
    let condition = or_expr.parse_next(input)?;
    
    if let Ok(_) = token('?').parse_next(input) {
        let then_expr = expr.map(Box::new).parse_next(input)?;
        token(':').parse_next(input)?;
        let else_expr = expr.map(Box::new).parse_next(input)?;
        
        let span = input.span_from(start);
        return Ok(Expr::Ternary {
            condition: Box::new(condition),
            then_expr,
            else_expr,
            span,
        });
    }
    
    Ok(condition)
}

/// OR: <and_expr> (or <and_expr>)*
pub fn or_expr<'a>(input: &mut ParseInput<'a>) -> PResult<Expr, ParseError> {
    let start = *input;
    let mut left = and_expr.parse_next(input)?;
    
    while let Ok(_) = keyword("or").parse_next(input) {
        let right = and_expr.parse_next(input)?;
        let span = input.span_from(start);
        left = Expr::Binary {
            op: BinaryOp::Or,
            left: Box::new(left),
            right: Box::new(right),
            span,
        };
    }
    
    Ok(left)
}

/// AND: <in_expr> (and <in_expr>)*
pub fn and_expr<'a>(input: &mut ParseInput<'a>) -> PResult<Expr, ParseError> {
    let start = *input;
    let mut left = in_expr.parse_next(input)?;
    
    while let Ok(_) = keyword("and").parse_next(input) {
        let right = in_expr.parse_next(input)?;
        let span = input.span_from(start);
        left = Expr::Binary {
            op: BinaryOp::And,
            left: Box::new(left),
            right: Box::new(right),
            span,
        };
    }
    
    Ok(left)
}

/// IN: <comparison> (in <comparison>)?
pub fn in_expr<'a>(input: &mut ParseInput<'a>) -> PResult<Expr, ParseError> {
    let start = *input;
    let left = comparison_expr.parse_next(input)?;
    
    if let Ok(_) = keyword("in").parse_next(input) {
        let right = comparison_expr.parse_next(input)?;
        let span = input.span_from(start);
        return Ok(Expr::Binary {
            op: BinaryOp::In,
            left: Box::new(left),
            right: Box::new(right),
            span,
        });
    }
    
    Ok(left)
}

/// Comparison: <additive> ((=|!=|<|>|<=|>=) <additive>)?
pub fn comparison_expr<'a>(input: &mut ParseInput<'a>) -> PResult<Expr, ParseError> {
    let start = *input;
    let left = additive_expr.parse_next(input)?;
    
    let op = alt((
        token("!=").map(|_| BinaryOp::Neq),
        token("<=").map(|_| BinaryOp::Leq),
        token(">=").map(|_| BinaryOp::Geq),
        token('=').map(|_| BinaryOp::Eq),
        token('<').map(|_| BinaryOp::Lt),
        token('>').map(|_| BinaryOp::Gt),
    )).parse_next(input);
    
    if let Ok(op) = op {
        let right = additive_expr.parse_next(input)?;
        let span = input.span_from(start);
        return Ok(Expr::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
            span,
        });
    }
    
    Ok(left)
}

/// Additive: <multiplicative> ((+|-) <multiplicative>)*
pub fn additive_expr<'a>(input: &mut ParseInput<'a>) -> PResult<Expr, ParseError> {
    let start = *input;
    let mut left = multiplicative_expr.parse_next(input)?;
    
    loop {
        let op = alt((
            token('+').map(|_| BinaryOp::Add),
            token('-').map(|_| BinaryOp::Sub),
        )).parse_next(input);
        
        match op {
            Ok(op) => {
                let right = multiplicative_expr.parse_next(input)?;
                let span = input.span_from(start);
                left = Expr::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                    span,
                };
            }
            Err(_) => break,
        }
    }
    
    Ok(left)
}

/// Multiplicative: <unary> ((*|/) <unary>)*
pub fn multiplicative_expr<'a>(input: &mut ParseInput<'a>) -> PResult<Expr, ParseError> {
    let start = *input;
    let mut left = unary_expr.parse_next(input)?;
    
    loop {
        let op = alt((
            token('*').map(|_| BinaryOp::Mul),
            token('/').map(|_| BinaryOp::Div),
        )).parse_next(input);
        
        match op {
            Ok(op) => {
                let right = unary_expr.parse_next(input)?;
                let span = input.span_from(start);
                left = Expr::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                    span,
                };
            }
            Err(_) => break,
        }
    }
    
    Ok(left)
}

/// Unary: (-|not|#|empty?) <unary> | <postfix>
pub fn unary_expr<'a>(input: &mut ParseInput<'a>) -> PResult<Expr, ParseError> {
    let start = *input;
    
    let op = alt((
        token('-').map(|_| UnaryOp::Neg),
        keyword("not").map(|_| UnaryOp::Not),
        token('#').map(|_| UnaryOp::Len),
        keyword("empty?").map(|_| UnaryOp::Empty),
    )).parse_next(input);
    
    if let Ok(op) = op {
        let operand = unary_expr.map(Box::new).parse_next(input)?;
        let span = input.span_from(start);
        return Ok(Expr::Unary { op, operand, span });
    }
    
    postfix_expr.parse_next(input)
}

/// Postfix: <primary> (.field | [<expr>] | (<args>))*
pub fn postfix_expr<'a>(input: &mut ParseInput<'a>) -> PResult<Expr, ParseError> {
    let start = *input;
    let mut expr = primary_expr.parse_next(input)?;
    
    loop {
        let postfix = alt((
            // Field access: .field
            preceded(token('.'), identifier).map(|f| Postfix::Field(f.into())),
            // Index access: [<expr>]
            delimited(token('['), expr, token(']')).map(|e| Postfix::Index(Box::new(e))),
            // Function call: (<args>)
            delimited(
                token('('),
                separated_list(0.., expr, token(',')),
                token(')'),
            ).map(|a| Postfix::Call(a)),
        )).parse_next(input);
        
        match postfix {
            Ok(post) => {
                let span = input.span_from(start);
                expr = match post {
                    Postfix::Field(field) => Expr::FieldAccess {
                        base: Box::new(expr),
                        field,
                        span,
                    },
                    Postfix::Index(index) => Expr::IndexAccess {
                        base: Box::new(expr),
                        index,
                        span,
                    },
                    Postfix::Call(args) => {
                        // Need to extract function name from expr
                        // This requires expr to be a variable
                        match expr {
                            Expr::Variable(name) => Expr::Call {
                                func: name,
                                args,
                                span,
                            },
                            _ => return Err(ErrMode::from_error_kind(input, ErrorKind::Verify)),
                        }
                    }
                };
            }
            Err(_) => break,
        }
    }
    
    Ok(expr)
}

/// Primary: literals, variables, $input, (<expr>)
pub fn primary_expr<'a>(input: &mut ParseInput<'a>) -> PResult<Expr, ParseError> {
    let start = *input;
    
    alt((
        literal_expr,
        input_ref_expr,
        identifier_expr,
        delimited(token('('), expr, token(')')),
    )).parse_next(input)
}

pub fn literal_expr<'a>(input: &mut ParseInput<'a>) -> PResult<Expr, ParseError> {
    let start = *input;
    let lit = literal.parse_next(input)?;
    let span = input.span_from(start);
    Ok(Expr::Literal(lit))
}

pub fn input_ref_expr<'a>(input: &mut ParseInput<'a>) -> PResult<Expr, ParseError> {
    let start = *input;
    token('$').parse_next(input)?;
    let name = identifier.parse_next(input)?;
    let span = input.span_from(start);
    Ok(Expr::InputRef(name.into()))
}

pub fn identifier_expr<'a>(input: &mut ParseInput<'a>) -> PResult<Expr, ParseError> {
    let start = *input;
    let name = identifier.parse_next(input)?;
    let span = input.span_from(start);
    Ok(Expr::Variable(name.into()))
}
```

## TDD Steps

### Step 1: Implement Primary Expressions

Start with literals, variables, and input refs.

### Step 2: Implement Postfix Expressions

Add field access, index access, and calls.

### Step 3: Implement Unary and Binary Operators

Work from high to low precedence.

### Step 4: Implement Ternary

Add the conditional operator.

### Step 5: Add Precedence Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precedence_add_mul() {
        // a + b * c should parse as a + (b * c)
        let input = "a + b * c";
        let (_, expr) = expr.parse_peek(ParseInput::new(input)).unwrap();
        
        match expr {
            Expr::Binary { op: BinaryOp::Add, left, right } => {
                assert!(matches!(left.as_ref(), Expr::Variable(_)));
                assert!(matches!(right.as_ref(), Expr::Binary { op: BinaryOp::Mul, .. }));
            }
            _ => panic!("Expected Add at top level"),
        }
    }

    #[test]
    fn test_precedence_and_or() {
        // a and b or c should parse as (a and b) or c
        let input = "a and b or c";
        let (_, expr) = expr.parse_peek(ParseInput::new(input)).unwrap();
        
        match expr {
            Expr::Binary { op: BinaryOp::Or, .. } => {
                // OR at top level is correct
            }
            _ => panic!("Expected Or at top level"),
        }
    }

    #[test]
    fn test_field_access() {
        let input = "foo.bar.baz";
        let (_, expr) = expr.parse_peek(ParseInput::new(input)).unwrap();
        assert!(matches!(expr, Expr::FieldAccess { .. }));
    }

    #[test]
    fn test_function_call() {
        let input = "foo(a, b, c)";
        let (_, expr) = expr.parse_peek(ParseInput::new(input)).unwrap();
        assert!(matches!(expr, Expr::Call { .. }));
    }
}
```

## Completion Checklist

- [ ] All expression types parse correctly
- [ ] Operator precedence matches SPEC-002
- [ ] Left-to-right associativity for binary ops
- [ ] Right-to-right associativity for ternary
- [ ] Comprehensive precedence tests
- [ ] Edge case tests (empty expressions, nested, etc.)
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Precedence**: Does `a + b * c` parse as `a + (b * c)`?
2. **Associativity**: Does `a - b - c` parse as `(a - b) - c`?
3. **Left recursion**: Are all left-recursive rules converted to loops?

## Estimated Effort

6 hours

## Dependencies

- TASK-012: Parser core (uses combinators)

## Blocked By

- TASK-012: Parser core

## Blocks

- TASK-013: Workflow parser (expressions used in workflows)
