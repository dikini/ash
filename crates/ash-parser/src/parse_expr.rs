//! Expression parser for the Ash language.
//!
//! This module provides parsers for Ash expressions using precedence climbing.

use winnow::combinator::{alt, delimited, opt, preceded};
use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::{one_of, take_while};

use crate::input::{ParseInput, Position};
use crate::parse_pattern::pattern;
use crate::surface::{BinaryOp, Expr, Literal, Name, UnaryOp};
use crate::token::Span;

/// Parse an expression (entry point).
///
/// This handles the full expression grammar with proper precedence.
pub fn expr(input: &mut ParseInput) -> ModalResult<Expr> {
    // Try if-let first (before other expressions to avoid conflicts with 'if')
    if let Ok(if_let) = parse_if_let_expr(input) {
        return Ok(if_let);
    }
    ternary_expr(input)
}

/// Parse a ternary expression: condition ? then : else
fn ternary_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let _start_pos = input.state;
    let condition = or_expr(input)?;

    // Check for ternary operator
    if opt(preceded(literal_str("?"), or_expr))
        .parse_next(input)?
        .is_some()
    {
        let _then_branch = or_expr(input)?;
        let _ = preceded(literal_str(":"), or_expr).parse_next(input)?;
        // Note: Simplified - ternary not fully implemented in surface AST
        Ok(condition)
    } else {
        Ok(condition)
    }
}

/// Parse an if-let expression: `if let pattern = expr then expr else expr`
///
/// Example: `if let Some { value: x } = opt then { x } else { 0 }`
pub fn parse_if_let_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state;

    // Match "if let"
    let _ = keyword("if").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let _ = keyword("let").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Parse the pattern
    let pat = pattern(input)?;

    skip_whitespace_and_comments(input);
    let _ = literal_str("=").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Parse the expression to match against
    let match_expr = ternary_expr(input)?;

    skip_whitespace_and_comments(input);
    let _ = keyword("then").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Parse then branch (block or expression)
    let then_branch = Box::new(parse_block_or_expr(input)?);

    skip_whitespace_and_comments(input);
    let _ = keyword("else").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Parse else branch (block or expression)
    let else_branch = Box::new(parse_block_or_expr(input)?);

    let span = span_from(&start_pos, &input.state);

    Ok(Expr::IfLet {
        pattern: pat,
        expr: Box::new(match_expr),
        then_branch,
        else_branch,
        span,
    })
}

/// Parse either a block `{ ... }` or a single expression.
///
/// This is used for then/else branches in if-let expressions.
/// A block can contain multiple statements/expressions separated by semicolons.
fn parse_block_or_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    skip_whitespace_and_comments(input);

    if input.input.starts_with("{") {
        // Parse a block with multiple statements
        let _ = literal_str("{").parse_next(input)?;
        skip_whitespace_and_comments(input);

        // Check for empty block
        if input.input.starts_with("}") {
            let _ = literal_str("}").parse_next(input)?;
            return Ok(Expr::Literal(Literal::Null));
        }

        // Parse first expression
        let first = expr(input)?;

        // Check for more expressions (semicolon-separated)
        let mut exprs = vec![first];
        loop {
            skip_whitespace_and_comments(input);
            if input.input.starts_with(";") {
                let _ = input.input.next_slice(1);
                input.state.advance(';');
                skip_whitespace_and_comments(input);

                // If next is }, this was a trailing semicolon
                if input.input.starts_with("}") {
                    break;
                }

                let next = expr(input)?;
                exprs.push(next);
            } else {
                break;
            }
        }

        let _ = literal_str("}").parse_next(input)?;

        // Return the last expression (or the only one)
        Ok(exprs.pop().unwrap_or(Expr::Literal(Literal::Null)))
    } else {
        // Single expression
        expr(input)
    }
}

/// Parse logical OR expressions: left || right
fn or_expr(input: &mut ParseInput) -> ModalResult<Expr> {
    let start_pos = input.state;
    let mut left = and_expr(input)?;

    loop {
        if opt(literal_str("||")).parse_next(input)?.is_some() {
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
        if opt(literal_str("&&")).parse_next(input)?.is_some() {
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

    if opt(keyword("in")).parse_next(input)?.is_some() {
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
    if opt(literal_str("!")).parse_next(input)?.is_some() {
        let operand = unary_expr(input)?;
        let span = span_from(&start_pos, &input.state);
        return Ok(Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(operand),
            span,
        });
    }

    // Try arithmetic negation (but not if it's followed by a number, that's a literal)
    if opt(preceded(
        literal_str("-"),
        one_of(|c: char| !c.is_ascii_digit()),
    ))
    .parse_next(input)?
    .is_some()
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
    if opt(keyword("not")).parse_next(input)?.is_some() {
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

    // Try check obligation expression: check obligation_name
    if keyword("check").parse_next(input).is_ok() {
        skip_whitespace_and_comments(input);
        let obligation = identifier(input)?;
        let span = span_from(&start_pos, &input.state);
        return Ok(Expr::CheckObligation {
            obligation: obligation.into(),
            span,
        });
    }

    // Try identifier/variable (and potential field access/call)
    let name = identifier(input)?;
    let name_str: Name = name.into();

    if parse_inline_constructor_start(input) {
        skip_whitespace_and_comments(input);
        let fields = if literal_str("}").parse_next(input).is_ok() {
            vec![]
        } else {
            parse_constructor_fields(input)?
        };
        let span = span_from(&start_pos, &input.state);
        return Ok(Expr::Constructor {
            name: name_str,
            fields,
            span,
        });
    }

    // Check for field access or method call
    let mut expr = Expr::Variable(name_str.clone());

    loop {
        // Field access: .field
        if opt(literal_str(".")).parse_next(input)?.is_some()
            && let Ok(field) = identifier(input)
        {
            let span = span_from(&start_pos, &input.state);
            expr = Expr::FieldAccess {
                base: Box::new(expr),
                field: field.into(),
                span,
            };
            continue;
        }

        // Index access: [index]
        if opt(literal_str("[")).parse_next(input)?.is_some() {
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
        if opt(literal_str("(")).parse_next(input)?.is_some() {
            let args = if literal_str(")").parse_next(input).is_ok() {
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

fn parse_constructor_fields(input: &mut ParseInput) -> ModalResult<Vec<(Name, Expr)>> {
    let mut fields = vec![parse_constructor_field(input)?];

    loop {
        skip_whitespace_and_comments(input);
        if opt(literal_str(",")).parse_next(input)?.is_some() {
            skip_whitespace_and_comments(input);
            if literal_str("}").parse_next(input).is_ok() {
                break;
            }

            fields.push(parse_constructor_field(input)?);
        } else {
            break;
        }
    }

    skip_whitespace_and_comments(input);
    let _ = literal_str("}").parse_next(input)?;
    Ok(fields)
}

fn parse_constructor_field(input: &mut ParseInput) -> ModalResult<(Name, Expr)> {
    skip_whitespace_and_comments(input);
    let name = identifier(input)?;
    skip_whitespace_and_comments(input);
    let _ = literal_str(":").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let value = expr(input)?;
    Ok((name.into(), value))
}

fn parse_inline_constructor_start(input: &mut ParseInput) -> bool {
    let source = input.input;
    let inline_ws_len = source
        .chars()
        .take_while(|c| matches!(c, ' ' | '\t'))
        .map(char::len_utf8)
        .sum::<usize>();

    let Some(rest) = source.get(inline_ws_len..) else {
        return false;
    };

    if !rest.starts_with('{') {
        return false;
    }

    let consumed = &source[..inline_ws_len + 1];
    for c in consumed.chars() {
        input.state.advance(c);
    }
    let _ = input.input.next_slice(inline_ws_len + 1);
    true
}

/// Parse function call arguments
fn parse_args(input: &mut ParseInput) -> ModalResult<Vec<Expr>> {
    let first = expr(input)?;
    let mut args = vec![first];

    loop {
        if opt(literal_str(",")).parse_next(input)?.is_some() {
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
    alt((
        parse_string,
        parse_float,
        parse_int,
        parse_bool,
        parse_null,
        parse_list,
    ))
    .parse_next(input)
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

/// Parse a list literal: [1, 2, 3] or ["a", "b"]
fn parse_list(input: &mut ParseInput) -> ModalResult<Literal> {
    let _ = literal_str("[").parse_next(input)?;

    // Empty list
    if literal_str("]").parse_next(input).is_ok() {
        return Ok(Literal::List(vec![]));
    }

    // Parse first element
    let first = literal(input)?;
    let mut elements = vec![first];

    // Parse remaining elements
    loop {
        if opt(literal_str(",")).parse_next(input)?.is_some() {
            // Check for trailing comma before ]
            if literal_str("]").parse_next(input).is_ok() {
                break;
            }
            let elem = literal(input)?;
            elements.push(elem);
        } else {
            break;
        }
    }

    let _ = literal_str("]").parse_next(input)?;
    Ok(Literal::List(elements))
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
    if result.is_empty()
        || !result
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
            && !result.starts_with('_')
    {
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
            if after.is_empty()
                || !after
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_alphanumeric())
            {
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
        let _: ModalResult<&str> =
            take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(input);

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
#[allow(clippy::approx_constant)]
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
    fn test_parse_variable_named_supervises() {
        let mut input = test_input("supervises");
        let result = expr(&mut input).unwrap();
        assert!(matches!(result, Expr::Variable(name) if name.as_ref() == "supervises"));
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

    // ============================================================
    // If-Let Expression Tests (TASK-126)
    // ============================================================

    #[test]
    fn test_parse_if_let_simple() {
        // Simple if-let with variant pattern
        let mut input = test_input("if let Some { value: x } = opt then { x } else { 0 }");
        let result = expr(&mut input).unwrap();

        match result {
            Expr::IfLet {
                pattern,
                expr,
                then_branch,
                else_branch,
                ..
            } => {
                // Pattern should be a Variant pattern
                assert!(
                    matches!(pattern, crate::surface::Pattern::Variant { name, .. } if name.as_ref() == "Some")
                );
                // Expression should be variable 'opt'
                assert!(matches!(expr.as_ref(), Expr::Variable(name) if name.as_ref() == "opt"));
                // Then branch should be variable 'x'
                assert!(
                    matches!(then_branch.as_ref(), Expr::Variable(name) if name.as_ref() == "x")
                );
                // Else branch should be literal 0
                assert!(matches!(
                    else_branch.as_ref(),
                    Expr::Literal(Literal::Int(0))
                ));
            }
            _ => panic!("Expected IfLet expression, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_if_let_unit_variant() {
        // Unit variant pattern (just the name without fields)
        let mut input = test_input("if let None = opt then { \"none\" } else { \"some\" }");
        let result = expr(&mut input).unwrap();

        match result {
            Expr::IfLet {
                pattern,
                then_branch,
                else_branch,
                ..
            } => {
                // Unit variants like `None` parse as variant patterns without braces.
                assert!(matches!(
                    pattern,
                    crate::surface::Pattern::Variant { name, fields }
                        if name.as_ref() == "None" && fields.is_none()
                ));
                // Then branch should be string "none"
                assert!(
                    matches!(then_branch.as_ref(), Expr::Literal(Literal::String(s)) if s.as_ref() == "none")
                );
                // Else branch should be string "some"
                assert!(
                    matches!(else_branch.as_ref(), Expr::Literal(Literal::String(s)) if s.as_ref() == "some")
                );
            }
            _ => panic!("Expected IfLet expression, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_if_let_variable_pattern() {
        // Simple variable pattern
        let mut input = test_input("if let x = value then { x } else { 0 }");
        let result = expr(&mut input).unwrap();

        match result {
            Expr::IfLet {
                pattern,
                then_branch,
                else_branch,
                ..
            } => {
                assert!(
                    matches!(pattern, crate::surface::Pattern::Variable(name) if name.as_ref() == "x")
                );
                assert!(
                    matches!(then_branch.as_ref(), Expr::Variable(name) if name.as_ref() == "x")
                );
                assert!(matches!(
                    else_branch.as_ref(),
                    Expr::Literal(Literal::Int(0))
                ));
            }
            _ => panic!("Expected IfLet expression, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_if_let_wildcard_pattern() {
        // Wildcard pattern
        let mut input = test_input("if let _ = value then { 1 } else { 0 }");
        let result = expr(&mut input).unwrap();

        match result {
            Expr::IfLet { pattern, .. } => {
                assert!(matches!(pattern, crate::surface::Pattern::Wildcard));
            }
            _ => panic!("Expected IfLet expression, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_if_let_tuple_pattern() {
        // Tuple pattern
        let mut input = test_input("if let (a, b) = pair then { a } else { b }");
        let result = expr(&mut input).unwrap();

        match result {
            Expr::IfLet {
                pattern,
                then_branch,
                else_branch,
                ..
            } => {
                assert!(matches!(pattern, crate::surface::Pattern::Tuple(pats) if pats.len() == 2));
                assert!(
                    matches!(then_branch.as_ref(), Expr::Variable(name) if name.as_ref() == "a")
                );
                assert!(
                    matches!(else_branch.as_ref(), Expr::Variable(name) if name.as_ref() == "b")
                );
            }
            _ => panic!("Expected IfLet expression, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_if_let_complex_expression() {
        // If-let with complex expression in match position
        let mut input = test_input("if let x = foo() + bar then { x } else { 0 }");
        let result = expr(&mut input).unwrap();

        assert!(matches!(result, Expr::IfLet { .. }));
    }

    #[test]
    fn test_parse_if_let_nested_expressions() {
        // Nested expressions in branches
        let mut input = test_input("if let Some { value: x } = opt then { x + 1 } else { x - 1 }");
        let result = expr(&mut input).unwrap();

        match result {
            Expr::IfLet {
                then_branch,
                else_branch,
                ..
            } => {
                // Both branches should be binary expressions
                assert!(matches!(
                    then_branch.as_ref(),
                    Expr::Binary {
                        op: BinaryOp::Add,
                        ..
                    }
                ));
                assert!(matches!(
                    else_branch.as_ref(),
                    Expr::Binary {
                        op: BinaryOp::Sub,
                        ..
                    }
                ));
            }
            _ => panic!("Expected IfLet expression, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_constructor_expression() {
        let mut input = test_input("Ok { value: 42 }");
        let result = expr(&mut input).unwrap();

        match result {
            Expr::Constructor { name, fields, .. } => {
                assert_eq!(name.as_ref(), "Ok");
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].0.as_ref(), "value");
                assert!(matches!(fields[0].1, Expr::Literal(Literal::Int(42))));
            }
            other => panic!("Expected Constructor expression, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_nested_constructor_expression() {
        let mut input =
            test_input(r#"Err { error: RuntimeError { exit_code: 42, message: "boom" } }"#);
        let result = expr(&mut input).unwrap();

        match result {
            Expr::Constructor { name, fields, .. } => {
                assert_eq!(name.as_ref(), "Err");
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].0.as_ref(), "error");
                match &fields[0].1 {
                    Expr::Constructor { name, fields, .. } => {
                        assert_eq!(name.as_ref(), "RuntimeError");
                        assert_eq!(fields.len(), 2);
                        assert_eq!(fields[0].0.as_ref(), "exit_code");
                        assert!(matches!(fields[0].1, Expr::Literal(Literal::Int(42))));
                        assert_eq!(fields[1].0.as_ref(), "message");
                        assert!(matches!(
                            fields[1].1,
                            Expr::Literal(Literal::String(ref s)) if s.as_ref() == "boom"
                        ));
                    }
                    other => panic!("Expected nested constructor, got {other:?}"),
                }
            }
            other => panic!("Expected Constructor expression, got {other:?}"),
        }
    }
}
