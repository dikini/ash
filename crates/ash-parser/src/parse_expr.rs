//! Expression parser for the Ash language.
//!
//! This module provides parsers for Ash expressions using precedence climbing.

use winnow::combinator::{alt, delimited, opt, preceded};
use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::{one_of, take_while};

use crate::input::{ParseInput, Position};
use crate::surface::{BinaryOp, Expr, Literal, Name, UnaryOp};
use crate::token::Span;

/// Parse an expression (entry point).
///
/// This handles the full expression grammar with proper precedence.
pub fn expr(input: &mut ParseInput) -> ModalResult<Expr> {
    ternary_expr(input)
}

/// Parse a ternary expression: condition ? then : else
fn ternary_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let _start_pos = input.state;
    let condition = or_expr(input)?;

    // Check for ternary operator
    if let Some(_) = opt(preceded(literal_str("?"), or_expr)).parse_next(input)? {
        let _then_branch = or_expr(input)?;
        let _ = preceded(literal_str(":"), or_expr).parse_next(input)?;
        // Note: Simplified - ternary not fully implemented in surface AST
        Ok(condition)
    } else {
        Ok(condition)
    }
}

/// Parse logical OR expressions: left || right
fn or_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state;
    let mut left = and_expr(input)?;

    loop {
        if let Some(_) = opt(literal_str("||")).parse_next(input)? {
            let right = and_expr(input)?;
            let span = span_from(&start_pos, &input.state);
            left = Expr::Binary {
                op: BinaryOp::Or,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        } else if let Some(_) = opt(literal_str("||")).parse_next(input)? {
            let right = and_expr(input)?;
            let span = span_from(&start_pos, &input.state);
            left = Expr::Binary {
                op: BinaryOp::Or,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        } else {
            break;
        }
    }

    Ok(left)
}

/// Parse logical AND expressions: left && right
fn and_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state;
    let mut left = in_expr(input)?;

    loop {
        if let Some(_) = opt(literal_str("&&")).parse_next(input)? {
            let right = in_expr(input)?;
            let span = span_from(&start_pos, &input.state);
            left = Expr::Binary {
                op: BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        } else if let Some(_) = opt(literal_str("&&")).parse_next(input)? {
            let right = in_expr(input)?;
            let span = span_from(&start_pos, &input.state);
            left = Expr::Binary {
                op: BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        } else {
            break;
        }
    }

    Ok(left)
}

/// Parse IN expressions: left in right
fn in_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state;
    let left = comparison_expr(input)?;

    if let Some(_) = opt(keyword("in")).parse_next(input)? {
        let right = comparison_expr(input)?;
        let span = span_from(&start_pos, &input.state);
        Ok(Expr::Binary {
            op: BinaryOp::In,
            left: Box::new(left),
            right: Box::new(right),
            span,
        })
    } else {
        Ok(left)
    }
}

/// Parse comparison expressions: ==, !=, <, >, <=, >=
fn comparison_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state;
    let left = additive_expr(input)?;

    // Try to match comparison operators
    let op = alt((
        literal_str("==").map(|_| BinaryOp::Eq),
        literal_str("!=").map(|_| BinaryOp::Neq),
        literal_str("<=").map(|_| BinaryOp::Leq),
        literal_str(">=").map(|_| BinaryOp::Geq),
        literal_str("<").map(|_| BinaryOp::Lt),
        literal_str(">").map(|_| BinaryOp::Gt),
    ))
    .parse_next(input);

    match op {
        Ok(op) => {
            let right = additive_expr(input)?;
            let span = span_from(&start_pos, &input.state);
            Ok(Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            })
        }
        Err(_) => Ok(left),
    }
}

/// Parse additive expressions: +, -
fn additive_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state;
    let mut left = multiplicative_expr(input)?;

    loop {
        let op = alt((
            literal_str("+").map(|_| BinaryOp::Add),
            literal_str("-").map(|_| BinaryOp::Sub),
        ))
        .parse_next(input);

        match op {
            Ok(op) => {
                let right = multiplicative_expr(input)?;
                let span = span_from(&start_pos, &input.state);
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

/// Parse multiplicative expressions: *, /
fn multiplicative_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state;
    let mut left = unary_expr(input)?;

    loop {
        let op = alt((
            literal_str("*").map(|_| BinaryOp::Mul),
            literal_str("/").map(|_| BinaryOp::Div),
        ))
        .parse_next(input);

        match op {
            Ok(op) => {
                let right = unary_expr(input)?;
                let span = span_from(&start_pos, &input.state);
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

/// Parse unary expressions: !, -
fn unary_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state;

    // Try negation first
    if let Some(_) = opt(literal_str("!")).parse_next(input)? {
        let operand = unary_expr(input)?;
        let span = span_from(&start_pos, &input.state);
        return Ok(Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(operand),
            span,
        });
    }

    // Try arithmetic negation (but not if it's followed by a number, that's a literal)
    if let Some(_) = opt(preceded(
        literal_str("-"),
        one_of(|c: char| !c.is_ascii_digit()),
    ))
    .parse_next(input)?
    {
        // This was a minus followed by a non-digit, so it's unary negation
        // We need to backtrack and parse properly
        // For simplicity, just parse the operand
        let operand = primary_expr(input)?;
        let span = span_from(&start_pos, &input.state);
        return Ok(Expr::Unary {
            op: UnaryOp::Neg,
            operand: Box::new(operand),
            span,
        });
    }

    // Try keyword "not"
    if let Some(_) = opt(keyword("not")).parse_next(input)? {
        let operand = unary_expr(input)?;
        let span = span_from(&start_pos, &input.state);
        return Ok(Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(operand),
            span,
        });
    }

    primary_expr(input)
}

/// Parse primary expressions: literals, variables, field access, index access, calls
fn primary_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state;

    // Try parenthesized expression first
    if let Ok(e) = delimited(literal_str("("), expr, literal_str(")")).parse_next(input) {
        return Ok(e);
    }

    // Try literal
    if let Ok(lit) = literal(input) {
        return Ok(Expr::Literal(lit));
    }

    // Try identifier/variable (and potential field access/call)
    let name = identifier(input)?;
    let name_str: Name = name.into();

    // Check for field access or method call
    let mut expr = Expr::Variable(name_str.clone());

    loop {
        // Field access: .field
        if let Some(_) = opt(literal_str(".")).parse_next(input)?
            && let Ok(field) = identifier(input) {
                let span = span_from(&start_pos, &input.state);
                expr = Expr::FieldAccess {
                    base: Box::new(expr),
                    field: field.into(),
                    span,
                };
                continue;
            }

        // Index access: [index]
        if let Some(_) = opt(literal_str("[")).parse_next(input)? {
            let index = self::expr(input)?;
            let _ = literal_str("]").parse_next(input)?;
            let span = span_from(&start_pos, &input.state);
            expr = Expr::IndexAccess {
                base: Box::new(expr),
                index: Box::new(index),
                span,
            };
            continue;
        }

        // Function call: (args)
        if let Some(_) = opt(literal_str("(")).parse_next(input)? {
            let args = if let Ok(_) = literal_str(")").parse_next(input) {
                vec![]
            } else {
                let args = parse_args(input)?;
                let _ = literal_str(")").parse_next(input)?;
                args
            };
            let span = span_from(&start_pos, &input.state);
            expr = Expr::Call {
                func: match &expr {
                    Expr::Variable(n) => n.clone(),
                    _ => "call".into(),
                },
                args,
                span,
            };
            continue;
        }

        break;
    }

    Ok(expr)
}

/// Parse function call arguments
fn parse_args(input: &mut ParseInput) -> ModalResult<Vec<Expr>> {
    let first = expr(input)?;
    let mut args = vec![first];

    loop {
        if let Some(_) = opt(literal_str(",")).parse_next(input)? {
            let arg = expr(input)?;
            args.push(arg);
        } else {
            break;
        }
    }

    Ok(args)
}

/// Parse a literal value.
pub fn literal(input: &mut ParseInput) -> ModalResult<Literal> {
    alt((parse_string, parse_float, parse_int, parse_bool, parse_null)).parse_next(input)
}

/// Parse a string literal.
fn parse_string(input: &mut ParseInput) -> ModalResult<Literal> {
    let _ = literal_str("\"").parse_next(input)?;

    let content = take_while(0.., |c: char| c != '"').parse_next(input)?;

    let _ = literal_str("\"").parse_next(input)?;
    Ok(Literal::String(content.into()))
}

/// Parse an integer literal.
fn parse_int(input: &mut ParseInput) -> ModalResult<Literal> {
    let digits: &str = take_while(1.., |c: char| c.is_ascii_digit()).parse_next(input)?;

    match digits.parse::<i64>() {
        Ok(n) => Ok(Literal::Int(n)),
        Err(_) => Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        )),
    }
}

/// Parse a floating-point literal.
fn parse_float(input: &mut ParseInput) -> ModalResult<Literal> {
    // Simplified float parsing: digits.digits
    let int_part: &str = take_while(1.., |c: char| c.is_ascii_digit()).parse_next(input)?;
    let _ = one_of('.').parse_next(input)?;
    let frac_part: &str = take_while(1.., |c: char| c.is_ascii_digit()).parse_next(input)?;

    let full = format!("{}.{}", int_part, frac_part);
    match full.parse::<f64>() {
        Ok(f) => Ok(Literal::Float(f)),
        Err(_) => Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        )),
    }
}

/// Parse a boolean literal.
fn parse_bool(input: &mut ParseInput) -> ModalResult<Literal> {
    alt((
        keyword("true").map(|_| Literal::Bool(true)),
        keyword("false").map(|_| Literal::Bool(false)),
    ))
    .parse_next(input)
}

/// Parse a null literal.
fn parse_null(input: &mut ParseInput) -> ModalResult<Literal> {
    keyword("null").map(|_| Literal::Null).parse_next(input)
}

/// Parse an identifier.
pub fn identifier<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    // Use take_while to match the entire identifier at once
    // First char: letter or underscore, rest: alphanumeric, underscore, or hyphen
    let result: &str = take_while(1.., |c: char| {
        c.is_ascii_alphanumeric() || c == '_' || c == '-'
    })
    .parse_next(input)?;

    // Check that first character is a letter or underscore (not a digit)
    if result.is_empty() || !result.chars().next().unwrap().is_ascii_alphabetic() && !result.starts_with('_') {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    // Check that it's not a keyword
    if is_keyword(result) {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    Ok(result)
}

/// Check if a string is a keyword.
fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "workflow"
            | "capability"
            | "policy"
            | "role"
            | "observe"
            | "orient"
            | "propose"
            | "decide"
            | "act"
            | "oblige"
            | "check"
            | "let"
            | "if"
            | "then"
            | "else"
            | "for"
            | "do"
            | "par"
            | "with"
            | "maybe"
            | "must"
            | "attempt"
            | "retry"
            | "timeout"
            | "done"
            | "epistemic"
            | "deliberative"
            | "evaluative"
            | "operational"
            | "authority"
            | "obligations"
            | "supervises"
            | "when"
            | "returns"
            | "where"
            | "permit"
            | "deny"
            | "require_approval"
            | "escalate"
            | "in"
            | "not"
            | "and"
            | "or"
            | "true"
            | "false"
            | "null"
    )
}

/// Parse a keyword (ensures word boundary).
fn keyword<'a>(word: &'a str) -> impl Parser<ParseInput<'a>, &'a str, winnow::error::ContextError> {
    move |input: &mut ParseInput<'a>| {
        let _start = input.state;

        if input.input.starts_with(word) {
            let after = &input.input[word.len()..];
            if after.is_empty() || !after.chars().next().unwrap().is_ascii_alphanumeric() {
                // Update position state
                for c in word.chars() {
                    input.state.advance(c);
                }
                // Advance the inner stream
                let _ = input.input.next_slice(word.len());
                return Ok(word);
            }
        }
        Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ))
    }
}

/// Whitespace wrapper.
#[allow(dead_code)]
fn ws<'a, F, O>(mut parser: F) -> impl FnMut(&mut ParseInput<'a>) -> ModalResult<O>
where
    F: FnMut(&mut ParseInput<'a>) -> ModalResult<O>,
{
    move |input: &mut ParseInput<'a>| {
        // Skip whitespace and comments
        skip_whitespace_and_comments(input);
        let result = parser(input)?;
        skip_whitespace_and_comments(input);
        Ok(result)
    }
}

/// Parse a string literal token.
fn literal_str<'a>(s: &'a str) -> impl FnMut(&mut ParseInput<'a>) -> ModalResult<&'a str> {
    move |input: &mut ParseInput<'a>| {
        skip_whitespace_and_comments(input);
        if input.input.starts_with(s) {
            // Update position state
            for c in s.chars() {
                input.state.advance(c);
            }
            // Advance the inner stream
            let _ = input.input.next_slice(s.len());
            Ok(s)
        } else {
            Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ))
        }
    }
}

/// Skip whitespace and comments.
fn skip_whitespace_and_comments(input: &mut ParseInput) {
    loop {
        // Skip whitespace
        let _: ModalResult<&str> = take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(input);

        // Check for line comment
        if input.input.starts_with("--") {
            let _: ModalResult<&str> = take_while(0.., |c: char| c != '\n').parse_next(input);
            continue;
        }

        // Check for block comment
        if input.input.starts_with("/*") {
            let _ = input.input.next_slice(2);
            let mut depth = 1;
            while depth > 0 && !input.input.is_empty() {
                if input.input.starts_with("/*") {
                    let _ = input.input.next_slice(2);
                    depth += 1;
                } else if input.input.starts_with("*/") {
                    let _ = input.input.next_slice(2);
                    depth -= 1;
                } else {
                    let _ = input.input.next_token();
                }
            }
            continue;
        }

        break;
    }
}

/// Create a span from start position to current position.
fn span_from(start: &Position, end: &Position) -> Span {
    Span {
        start: start.offset,
        end: end.offset,
        line: start.line,
        column: start.column,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_input(s: &str) -> ParseInput<'_> {
        crate::input::new_input(s)
    }

    #[test]
    fn test_parse_int_literal() {
        let mut input = test_input("42");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::Literal(Literal::Int(42))));
    }

    #[test]
    fn test_parse_float_literal() {
        let mut input = test_input("3.14");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::Literal(Literal::Float(f)) if (f - 3.14).abs() < 0.001));
    }

    #[test]
    fn test_parse_string_literal() {
        let mut input = test_input("\"hello world\"");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::Literal(Literal::String(s)) if s.as_ref() == "hello world"));
    }

    #[test]
    fn test_parse_bool_literal() {
        let mut input = test_input("true");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::Literal(Literal::Bool(true))));

        let mut input = test_input("false");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::Literal(Literal::Bool(false))));
    }

    #[test]
    fn test_parse_null_literal() {
        let mut input = test_input("null");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::Literal(Literal::Null)));
    }

    #[test]
    fn test_parse_variable() {
        let mut input = test_input("my_variable");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::Variable(name) if name.as_ref() == "my_variable"));
    }

    #[test]
    fn test_parse_binary_addition() {
        let mut input = test_input("1 + 2");
        let result = expr(&mut input).unwrap();
        assert!(matches!(
            result,
            Expr::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_binary_multiplication() {
        let mut input = test_input("3 * 4");
        let result = expr(&mut input).unwrap();
        assert!(matches!(
            result,
            Expr::Binary {
                op: BinaryOp::Mul,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_precedence() {
        // Multiplication has higher precedence than addition
        let mut input = test_input("1 + 2 * 3");
        let result = expr(&mut input).unwrap();

        // Should be: 1 + (2 * 3), not (1 + 2) * 3
        match result {
            Expr::Binary {
                op: BinaryOp::Add,
                left,
                right,
                ..
            } => {
                assert!(matches!(left.as_ref(), Expr::Literal(Literal::Int(1))));
                assert!(matches!(
                    right.as_ref(),
                    Expr::Binary {
                        op: BinaryOp::Mul,
                        ..
                    }
                ));
            }
            _ => panic!("Expected Add expression"),
        }
    }

    #[test]
    fn test_parse_comparison() {
        let mut input = test_input("x > 5");
        let result = expr(&mut input).unwrap();
        assert!(matches!(
            result,
            Expr::Binary {
                op: BinaryOp::Gt,
                ..
            }
        ));

        let mut input = test_input("a == b");
        let result = expr(&mut input).unwrap();
        assert!(matches!(
            result,
            Expr::Binary {
                op: BinaryOp::Eq,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_logical_and() {
        let mut input = test_input("a && b");
        let result = expr(&mut input).unwrap();
        assert!(matches!(
            result,
            Expr::Binary {
                op: BinaryOp::And,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_logical_or() {
        let mut input = test_input("a || b");
        let result = expr(&mut input).unwrap();
        assert!(matches!(
            result,
            Expr::Binary {
                op: BinaryOp::Or,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_field_access() {
        let mut input = test_input("obj.field");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::FieldAccess { .. }));
    }

    #[test]
    fn test_parse_function_call() {
        let mut input = test_input("foo()");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::Call { func, .. } if func.as_ref() == "foo"));
    }

    #[test]
    fn test_parse_function_call_with_args() {
        let mut input = test_input("foo(1, 2, 3)");
        let result = expr(&mut input).unwrap();
        match result {
            Expr::Call { func, args, .. } => {
                assert_eq!(func.as_ref(), "foo");
                assert_eq!(args.len(), 3);
            }
            _ => panic!("Expected Call expression"),
        }
    }

    #[test]
    fn test_parse_parenthesized() {
        let mut input = test_input("(1 + 2) * 3");
        let result = expr(&mut input).unwrap();

        // Should be: (1 + 2) * 3
        match result {
            Expr::Binary {
                op: BinaryOp::Mul,
                left,
                ..
            } => {
                assert!(matches!(
                    left.as_ref(),
                    Expr::Binary {
                        op: BinaryOp::Add,
                        ..
                    }
                ));
            }
            _ => panic!("Expected Mul expression"),
        }
    }

    #[test]
    fn test_parse_complex_expression() {
        let mut input = test_input("a + b * c - d / e");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::Binary { .. }));
    }

    #[test]
    fn test_parse_in_expression() {
        let mut input = test_input("x in list");
        let result = expr(&mut input).unwrap();
        assert!(matches!(
            result,
            Expr::Binary {
                op: BinaryOp::In,
                ..
            }
        ));
    }
}
