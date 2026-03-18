//! Parser for send statements in the Ash language.
//!
//! This module provides parsers for the send construct for output streams.

use winnow::prelude::*;

use crate::input::ParseInput;
use crate::parse_expr::expr;
use crate::parse_utils::{keyword, parse_capability_ref, skip_whitespace_and_comments};
use crate::surface::{Expr, Name};

/// Parsed send expression.
///
/// Represents a `send capability:channel expr` construct
/// for sending values to output streams.
#[derive(Debug, Clone, PartialEq)]
pub struct SendExpr {
    /// Capability name (e.g., "kafka" in "kafka:orders")
    pub capability: Name,
    /// Channel name (e.g., "orders" in "kafka:orders")
    pub channel: Name,
    /// Value expression to send
    pub value: Expr,
}

/// Parse a send expression.
///
/// # Syntax
///
/// ```text
/// send capability:channel expr
/// ```
///
/// # Examples
///
/// ```
/// use ash_parser::{parse_send, new_input};
/// use winnow::prelude::*;
///
/// let mut input = new_input("send kafka:orders order");
/// let result = parse_send(&mut input);
/// assert!(result.is_ok());
/// ```
pub fn parse_send(input: &mut ParseInput) -> ModalResult<SendExpr> {
    // Parse "send" keyword
    keyword(input, "send")?;
    skip_whitespace_and_comments(input);

    // Parse capability:channel
    let (capability, channel) = parse_capability_ref(input)?;
    skip_whitespace_and_comments(input);

    // Parse value expression
    let value = expr(input)?;

    Ok(SendExpr {
        capability: Name::from(capability),
        channel: Name::from(channel),
        value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::new_input;
    use crate::surface::Literal;

    #[test]
    fn test_parse_send_variable() {
        let mut input = new_input("send kafka:orders order");
        let result = parse_send(&mut input);
        assert!(result.is_ok());

        let send = result.unwrap();
        assert_eq!(send.capability, "kafka".into());
        assert_eq!(send.channel, "orders".into());
        // Check value is a variable reference
        match send.value {
            Expr::Variable(name) => assert_eq!(name, "order".into()),
            _ => panic!("Expected variable, got {:?}", send.value),
        }
    }

    #[test]
    fn test_parse_send_string() {
        let mut input = new_input(r#"send alerts:slack "System down""#);
        let result = parse_send(&mut input);
        assert!(result.is_ok());

        let send = result.unwrap();
        assert_eq!(send.capability, "alerts".into());
        assert_eq!(send.channel, "slack".into());
        match send.value {
            Expr::Literal(Literal::String(s)) => assert_eq!(s, "System down".into()),
            _ => panic!("Expected string literal, got {:?}", send.value),
        }
    }

    #[test]
    fn test_parse_send_record() {
        // Using function call syntax for structured values
        let mut input = new_input("send metrics:timings record(op, elapsed)");
        let result = parse_send(&mut input);
        assert!(result.is_ok());

        let send = result.unwrap();
        assert_eq!(send.capability, "metrics".into());
        assert_eq!(send.channel, "timings".into());
        // Value should be a function call
        match send.value {
            Expr::Call { .. } => {}
            _ => panic!("Expected call expression, got {:?}", send.value),
        }
    }

    #[test]
    fn test_parse_send_with_whitespace() {
        let mut input = new_input("send   kafka:orders   order");
        let result = parse_send(&mut input);
        assert!(result.is_ok());

        let send = result.unwrap();
        assert_eq!(send.capability, "kafka".into());
        assert_eq!(send.channel, "orders".into());
    }

    #[test]
    fn test_parse_send_with_comment() {
        let mut input = new_input("send kafka:orders -- the order\norder");
        let result = parse_send(&mut input);
        assert!(result.is_ok());

        let send = result.unwrap();
        assert_eq!(send.capability, "kafka".into());
        assert_eq!(send.channel, "orders".into());
    }

    #[test]
    fn test_parse_send_int_literal() {
        let mut input = new_input("send counter:value 42");
        let result = parse_send(&mut input);
        assert!(result.is_ok());

        let send = result.unwrap();
        assert_eq!(send.capability, "counter".into());
        assert_eq!(send.channel, "value".into());
        match send.value {
            Expr::Literal(Literal::Int(n)) => assert_eq!(n, 42),
            _ => panic!("Expected int literal, got {:?}", send.value),
        }
    }
}
