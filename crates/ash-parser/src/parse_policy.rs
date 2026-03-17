//! Policy expression parser for the Ash language.
//!
//! This module provides parsers for policy combinators using precedence climbing.
//! Supports infix operators: &, |, !, >> as well as method chaining.
//!
//! # Grammar
//!
//! ```text
//! policy_expr  ::= policy_or
//!
//! policy_or    ::= policy_and ("|" policy_and)*
//!
//! policy_and   ::= policy_seq ("&" policy_seq)*
//!
//! policy_seq   ::= policy_unary (">>" policy_unary)*
//!
//! policy_unary ::= "!" policy_unary
//!                | "forall" "(" identifier "," expr "," policy_expr ")"
//!                | "exists" "(" identifier "," expr "," policy_expr ")"
//!                | policy_primary
//!
//! policy_primary ::= identifier
//!                  | identifier "(" args ")"
//!                  | "(" policy_expr ")"
//!                  | policy_primary "." method "(" args ")"
//! ```

use winnow::combinator::delimited;
use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::take_while;

use crate::input::{ParseInput, Position};
use crate::parse_expr::expr as parse_expr_value;
use crate::surface::{Expr, Name, PolicyExpr};
use crate::token::Span;

/// Parse a policy expression (entry point).
///
/// # Example
///
/// ```
/// use ash_parser::{new_input, parse_policy_expr};
///
/// let mut input = new_input("policy1 & policy2");
/// let result = parse_policy_expr(&mut input);
/// assert!(result.is_ok());
/// ```
pub fn policy_expr(input: &mut ParseInput) -> ModalResult<PolicyExpr> {
    policy_or(input)
}

/// Parse a policy OR expression: left | right
fn policy_or(input: &mut ParseInput) -> ModalResult<PolicyExpr> {
    let start_pos = input.state;
    let mut exprs = vec![policy_and(input)?];

    while policy_ws(literal_str("|")).parse_next(input).is_ok() {
        exprs.push(policy_and(input)?);
    }

    if exprs.len() == 1 {
        Ok(exprs.into_iter().next().unwrap())
    } else {
        let _span = span_from(&start_pos, &input.state);
        Ok(PolicyExpr::Or(exprs))
    }
}

/// Parse a policy AND expression: left & right
fn policy_and(input: &mut ParseInput) -> ModalResult<PolicyExpr> {
    let start_pos = input.state;
    let mut exprs = vec![policy_seq(input)?];

    while policy_ws(literal_str("&")).parse_next(input).is_ok() {
        exprs.push(policy_seq(input)?);
    }

    if exprs.len() == 1 {
        Ok(exprs.into_iter().next().unwrap())
    } else {
        let _span = span_from(&start_pos, &input.state);
        Ok(PolicyExpr::And(exprs))
    }
}

/// Parse a policy sequential expression: left >> right
fn policy_seq(input: &mut ParseInput) -> ModalResult<PolicyExpr> {
    let start_pos = input.state;
    let mut exprs = vec![policy_unary(input)?];

    while policy_ws(literal_str(">>")).parse_next(input).is_ok() {
        exprs.push(policy_unary(input)?);
    }

    if exprs.len() == 1 {
        Ok(exprs.into_iter().next().unwrap())
    } else {
        let _span = span_from(&start_pos, &input.state);
        Ok(PolicyExpr::Sequential(exprs))
    }
}

/// Parse a policy unary expression: !expr, forall, exists, or primary
fn policy_unary(input: &mut ParseInput) -> ModalResult<PolicyExpr> {
    let start_pos = input.state;

    // Try negation: !expr
    if let Ok(_) = policy_ws(literal_str("!")).parse_next(input) {
        let operand = policy_unary(input)?;
        return Ok(PolicyExpr::Not(Box::new(operand)));
    }

    // Try forall quantifier
    if let Ok(_) = policy_keyword("forall").parse_next(input) {
        return parse_forall(input, &start_pos);
    }

    // Try exists quantifier
    if let Ok(_) = policy_keyword("exists").parse_next(input) {
        return parse_exists(input, &start_pos);
    }

    policy_primary(input)
}

/// Parse a forall expression: forall(var, items, body)
fn parse_forall(input: &mut ParseInput, start_pos: &Position) -> ModalResult<PolicyExpr> {
    let _ = policy_ws(literal_str("(")).parse_next(input)?;
    let var = policy_ws(identifier).parse_next(input)?.into();
    let _ = policy_ws(literal_str(",")).parse_next(input)?;
    let items = Box::new(parse_expr_value(input)?);
    let _ = policy_ws(literal_str(",")).parse_next(input)?;
    let body = Box::new(policy_expr(input)?);
    let _ = policy_ws(literal_str(")")).parse_next(input)?;

    let span = span_from(start_pos, &input.state);
    Ok(PolicyExpr::ForAll {
        var,
        items,
        body,
        span,
    })
}

/// Parse an exists expression: exists(var, items, body)
fn parse_exists(input: &mut ParseInput, start_pos: &Position) -> ModalResult<PolicyExpr> {
    let _ = policy_ws(literal_str("(")).parse_next(input)?;
    let var = policy_ws(identifier).parse_next(input)?.into();
    let _ = policy_ws(literal_str(",")).parse_next(input)?;
    let items = Box::new(parse_expr_value(input)?);
    let _ = policy_ws(literal_str(",")).parse_next(input)?;
    let body = Box::new(policy_expr(input)?);
    let _ = policy_ws(literal_str(")")).parse_next(input)?;

    let span = span_from(start_pos, &input.state);
    Ok(PolicyExpr::Exists {
        var,
        items,
        body,
        span,
    })
}

/// Parse a policy primary expression: identifier, call, or parenthesized
fn policy_primary(input: &mut ParseInput) -> ModalResult<PolicyExpr> {
    let start_pos = input.state;

    // Try parenthesized expression first
    if let Ok(expr) = delimited(
        policy_ws(literal_str("(")),
        policy_expr,
        policy_ws(literal_str(")")),
    )
    .parse_next(input)
    {
        return Ok(expr);
    }

    // Parse identifier
    let name = identifier(input)?;
    let name_str: Name = name.into();

    // Check for function call
    if let Ok(_) = policy_ws(literal_str("(")).parse_next(input) {
        let args = if let Ok(_) = literal_str(")").parse_next(input) {
            vec![]
        } else {
            let args = parse_policy_args(input)?;
            let _ = policy_ws(literal_str(")")).parse_next(input)?;
            args
        };
        let span = span_from(&start_pos, &input.state);
        let mut expr = PolicyExpr::Call {
            func: name_str,
            args,
            span,
        };

        // Check for method chain
        expr = parse_method_chain(input, expr, &start_pos)?;
        return Ok(expr);
    }

    // It's a variable reference - check for method chain
    let mut expr = PolicyExpr::Var(name_str);
    expr = parse_method_chain(input, expr, &start_pos)?;
    Ok(expr)
}

/// Parse method chaining: .method(args)
fn parse_method_chain(
    input: &mut ParseInput,
    mut receiver: PolicyExpr,
    start_pos: &Position,
) -> ModalResult<PolicyExpr> {
    loop {
        // Check for method call
        if let Ok(_) = policy_ws(literal_str(".")).parse_next(input)
            && let Ok(method_name) = identifier(input) {
                let method: Name = method_name.to_string().into_boxed_str();

                // Check for arguments
                let args = if let Ok(_) = policy_ws(literal_str("(")).parse_next(input) {
                    if let Ok(_) = literal_str(")").parse_next(input) {
                        vec![]
                    } else {
                        let args = parse_policy_args(input)?;
                        let _ = policy_ws(literal_str(")")).parse_next(input)?;
                        args
                    }
                } else {
                    vec![]
                };

                let span = span_from(start_pos, &input.state);
                receiver = PolicyExpr::MethodCall {
                    receiver: Box::new(receiver),
                    method,
                    args,
                    span,
                };
                continue;
            }
        break;
    }
    Ok(receiver)
}

/// Parse arguments for policy expressions
fn parse_policy_args(input: &mut ParseInput) -> ModalResult<Vec<Expr>> {
    let first = parse_expr_value(input)?;
    let mut args = vec![first];

    loop {
        if let Ok(_) = policy_ws(literal_str(",")).parse_next(input) {
            let arg = parse_expr_value(input)?;
            args.push(arg);
        } else {
            break;
        }
    }

    Ok(args)
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

/// Parse a keyword (ensures word boundary).
fn policy_keyword<'a>(word: &'a str) -> impl FnMut(&mut ParseInput<'a>) -> ModalResult<&'a str> {
    move |input: &mut ParseInput<'a>| {
        skip_whitespace_and_comments(input);
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

/// Whitespace wrapper for policy parsing.
fn policy_ws<'a, F, O>(mut parser: F) -> impl FnMut(&mut ParseInput<'a>) -> ModalResult<O>
where
    F: FnMut(&mut ParseInput<'a>) -> ModalResult<O>,
{
    move |input: &mut ParseInput<'a>| {
        skip_whitespace_and_comments(input);
        let result = parser(input)?;
        skip_whitespace_and_comments(input);
        Ok(result)
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

/// Parse an identifier.
fn identifier<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::new_input;

    fn test_input(s: &str) -> ParseInput<'_> {
        new_input(s)
    }

    // =========================================================================
    // Parser Tests
    // =========================================================================

    #[test]
    fn test_parse_policy_var() {
        let mut input = test_input("my_policy");
        let result = policy_expr(&mut input).unwrap();
        assert!(matches!(result, PolicyExpr::Var(name) if name.as_ref() == "my_policy"));
    }

    #[test]
    fn test_parse_and_combinator() {
        let mut input = test_input("policy1 & policy2");
        let result = policy_expr(&mut input).unwrap();
        match result {
            PolicyExpr::And(exprs) => {
                assert_eq!(exprs.len(), 2);
            }
            _ => panic!("Expected And, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_or_combinator() {
        let mut input = test_input("primary | fallback");
        let result = policy_expr(&mut input).unwrap();
        match result {
            PolicyExpr::Or(exprs) => {
                assert_eq!(exprs.len(), 2);
            }
            _ => panic!("Expected Or, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_not_combinator() {
        let mut input = test_input("!forbidden_region");
        let result = policy_expr(&mut input).unwrap();
        match result {
            PolicyExpr::Not(inner) => {
                assert!(matches!(inner.as_ref(), PolicyExpr::Var(_)));
            }
            _ => panic!("Expected Not, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_sequential_combinator() {
        let mut input = test_input("check_quota >> check_rate >> process");
        let result = policy_expr(&mut input).unwrap();
        match result {
            PolicyExpr::Sequential(exprs) => {
                assert_eq!(exprs.len(), 3);
            }
            _ => panic!("Expected Sequential, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_method_chain() {
        let mut input = test_input("base.and(other).retry(3)");
        let result = policy_expr(&mut input).unwrap();
        // Should be: MethodCall(MethodCall(Var("base"), "and", [other]), "retry", [3])
        match result {
            PolicyExpr::MethodCall {
                receiver,
                method,
                args,
                ..
            } => {
                assert_eq!(method.as_ref(), "retry");
                assert_eq!(args.len(), 1);
                match receiver.as_ref() {
                    PolicyExpr::MethodCall {
                        receiver: inner_receiver,
                        method: inner_method,
                        ..
                    } => {
                        assert_eq!(inner_method.as_ref(), "and");
                        assert!(matches!(inner_receiver.as_ref(), PolicyExpr::Var(_)));
                    }
                    _ => panic!("Expected nested MethodCall"),
                }
            }
            _ => panic!("Expected MethodCall, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_forall() {
        let mut input = test_input("forall(x, items, policy)");
        let result = policy_expr(&mut input).unwrap();
        match result {
            PolicyExpr::ForAll { var, body, .. } => {
                assert_eq!(var.as_ref(), "x");
                assert!(matches!(body.as_ref(), PolicyExpr::Var(_)));
            }
            _ => panic!("Expected ForAll, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_exists() {
        let mut input = test_input("exists(x, items, policy)");
        let result = policy_expr(&mut input).unwrap();
        match result {
            PolicyExpr::Exists { var, body, .. } => {
                assert_eq!(var.as_ref(), "x");
                assert!(matches!(body.as_ref(), PolicyExpr::Var(_)));
            }
            _ => panic!("Expected Exists, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_complex_nested() {
        let mut input = test_input("(p1 & p2) | (p3 & p4)");
        let result = policy_expr(&mut input).unwrap();
        match result {
            PolicyExpr::Or(exprs) => {
                assert_eq!(exprs.len(), 2);
                assert!(matches!(exprs[0], PolicyExpr::And(_)));
                assert!(matches!(exprs[1], PolicyExpr::And(_)));
            }
            _ => panic!("Expected Or with nested And, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_policy_call() {
        let mut input = test_input("rate_limit(100, 60)");
        let result = policy_expr(&mut input).unwrap();
        match result {
            PolicyExpr::Call { func, args, .. } => {
                assert_eq!(func.as_ref(), "rate_limit");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected Call, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_implies() {
        // Implies is parsed as a function call: implies(a, b)
        let mut input = test_input("implies(a, b)");
        let result = policy_expr(&mut input).unwrap();
        match result {
            PolicyExpr::Call { func, args, .. } => {
                assert_eq!(func.as_ref(), "implies");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected Call for implies, got {:?}", result),
        }
    }

    #[test]
    fn test_operator_precedence() {
        // & should bind tighter than |
        let mut input = test_input("a & b | c");
        let result = policy_expr(&mut input).unwrap();
        // Should be: Or([And([a, b]), c])
        match result {
            PolicyExpr::Or(exprs) => {
                assert_eq!(exprs.len(), 2);
                assert!(matches!(exprs[0], PolicyExpr::And(_)));
                assert!(matches!(exprs[1], PolicyExpr::Var(_)));
            }
            _ => panic!("Expected Or with And as first operand, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_concurrent() {
        // Concurrent is parsed as a function call: concurrent([p1, p2])
        let mut input = test_input("concurrent(p1, p2)");
        let result = policy_expr(&mut input).unwrap();
        match result {
            PolicyExpr::Call { func, args, .. } => {
                assert_eq!(func.as_ref(), "concurrent");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected Call for concurrent, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_with_whitespace() {
        let mut input = test_input("policy1   &   policy2");
        let result = policy_expr(&mut input).unwrap();
        assert!(matches!(result, PolicyExpr::And(_)));
    }

    #[test]
    fn test_parse_with_comments() {
        let mut input = test_input("policy1 -- first policy\n& policy2 -- second policy");
        let result = policy_expr(&mut input).unwrap();
        assert!(matches!(result, PolicyExpr::And(_)));
    }
}
