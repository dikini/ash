//! Parser for send statements in the Ash language.
//!
//! This module provides parsers for the send construct for output streams.

use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::take_while;

use crate::input::ParseInput;
use crate::parse_expr::{expr, identifier};
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

/// Parse a capability reference in the form `capability:channel`.
///
/// # Examples
///
/// - `kafka:orders`
/// - `alerts:slack`
/// - `metrics:timings`
fn parse_capability_ref<'a>(input: &mut ParseInput<'a>) -> ModalResult<(&'a str, &'a str)> {
    let capability = identifier(input)?;
    skip_whitespace_and_comments(input);
    literal_str(":").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let channel = identifier(input)?;
    Ok((capability, channel))
}

/// Parse a keyword, ensuring word boundary.
fn keyword<'a>(input: &mut ParseInput<'a>, word: &'a str) -> ModalResult<&'a str> {
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
        // Check value is a variable expression
        assert!(
            matches!(send.value, Expr::Variable(ref name) if name.as_ref() == "order"),
            "Expected Variable expression, got {:?}",
            send.value
        );
    }

    #[test]
    fn test_parse_send_string() {
        let mut input = new_input(r#"send alerts:slack "System down""#);
        let result = parse_send(&mut input);
        assert!(result.is_ok());

        let send = result.unwrap();
        assert_eq!(send.capability, "alerts".into());
        assert_eq!(send.channel, "slack".into());
        // Check value is a string literal
        assert!(
            matches!(send.value, Expr::Literal(Literal::String(ref s)) if s.as_ref() == "System down"),
            "Expected String literal, got {:?}",
            send.value
        );
    }

    #[test]
    fn test_parse_send_record() {
        // Record-like values are created using function call syntax
        // since record literal syntax is not yet supported by the expression parser
        let mut input = new_input("send metrics:timings record(op, elapsed)");
        let result = parse_send(&mut input);
        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let send = result.unwrap();
        assert_eq!(send.capability, "metrics".into());
        assert_eq!(send.channel, "timings".into());
        // Value is a function call expression for record creation
        assert!(
            matches!(send.value, Expr::Call { ref func, ref args, .. } if func.as_ref() == "record" && args.len() == 2),
            "Expected Call expression to 'record' with 2 args, got {:?}",
            send.value
        );
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
        let mut input = new_input("send kafka:orders order -- send to kafka");
        let result = parse_send(&mut input);
        assert!(result.is_ok());

        let send = result.unwrap();
        assert_eq!(send.capability, "kafka".into());
        assert_eq!(send.channel, "orders".into());
    }

    #[test]
    fn test_parse_capability_ref() {
        let mut input = new_input("kafka:orders");
        let result = parse_capability_ref(&mut input);
        assert!(result.is_ok());

        let (cap, chan) = result.unwrap();
        assert_eq!(cap, "kafka");
        assert_eq!(chan, "orders");
    }
}
