//! Parser for set statements in the Ash language.
//!
//! This module provides parsers for the set construct for output behaviours.

use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::take_while;

use crate::input::ParseInput;
use crate::parse_expr::{expr, identifier};
use crate::surface::{Expr, Name};

/// Parsed set expression.
///
/// Represents a `set capability:channel = expr` construct
/// for setting output capability values.
#[derive(Debug, Clone, PartialEq)]
pub struct SetExpr {
    /// Capability name (e.g., "hvac" in "hvac:target")
    pub capability: Name,
    /// Channel name (e.g., "target" in "hvac:target")
    pub channel: Name,
    /// Value expression to set
    pub value: Expr,
}

/// Parse a set expression.
///
/// # Syntax
///
/// ```text
/// set capability:channel = expr
/// ```
///
/// # Examples
///
/// ```
/// use ash_parser::{parse_set, new_input};
/// use winnow::prelude::*;
///
/// let mut input = new_input("set hvac:target = 72");
/// let result = parse_set(&mut input);
/// assert!(result.is_ok());
/// ```
pub fn parse_set(input: &mut ParseInput) -> ModalResult<SetExpr> {
    // Parse "set" keyword
    keyword(input, "set")?;
    skip_whitespace_and_comments(input);

    // Parse capability:channel
    let (capability, channel) = parse_capability_ref(input)?;
    skip_whitespace_and_comments(input);

    // Parse "="
    literal_str("=").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Parse value expression
    let value = expr(input)?;

    Ok(SetExpr {
        capability: Name::from(capability),
        channel: Name::from(channel),
        value,
    })
}

/// Parse a capability reference in the form `capability:channel`.
///
/// # Examples
///
/// - `hvac:target`
/// - `light:living_room`
/// - `config:timeout`
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
    fn test_parse_set_simple() {
        let mut input = new_input("set hvac:target = 72");
        let result = parse_set(&mut input);
        assert!(result.is_ok());

        let set = result.unwrap();
        assert_eq!(set.capability, "hvac".into());
        assert_eq!(set.channel, "target".into());
        // Check value is Expr::Literal(Literal::Int(72))
        assert!(matches!(set.value, Expr::Literal(Literal::Int(72))));
    }

    #[test]
    fn test_parse_set_record() {
        // Record-like values can be created using function call syntax
        // e.g., record(80, "warm") - positional arguments
        let mut input = new_input("set light:room = make_record(80, \"warm\")");
        let result = parse_set(&mut input);
        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let set = result.unwrap();
        assert_eq!(set.capability, "light".into());
        assert_eq!(set.channel, "room".into());
        // Value is parsed as a function call
        assert!(
            matches!(set.value, Expr::Call { ref func, ref args, .. } if func.as_ref() == "make_record" && args.len() == 2),
            "Expected Call expression to 'make_record' with 2 args, got {:?}",
            set.value
        );
    }

    #[test]
    fn test_parse_set_expression() {
        // Test with capability and channel names that are definitely not keywords
        // and a simple variable expression
        let mut input = new_input("set hvac:target = myval");
        let result = parse_set(&mut input);
        assert!(
            result.is_ok(),
            "Expected successful parse, got: {:?}",
            result
        );

        let set = result.unwrap();
        assert_eq!(set.capability, "hvac".into());
        assert_eq!(set.channel, "target".into());
        // Check value is a variable expression
        assert!(
            matches!(set.value, Expr::Variable(ref name) if name.as_ref() == "myval"),
            "Expected Variable expression, got {:?}",
            set.value
        );
    }

    #[test]
    fn test_parse_set_with_whitespace() {
        let mut input = new_input("set   hvac:target   =   72");
        let result = parse_set(&mut input);
        assert!(result.is_ok());

        let set = result.unwrap();
        assert_eq!(set.capability, "hvac".into());
        assert_eq!(set.channel, "target".into());
    }

    #[test]
    fn test_parse_set_with_comment() {
        let mut input = new_input("set hvac:target = 72 -- set the temperature");
        let result = parse_set(&mut input);
        assert!(result.is_ok());

        let set = result.unwrap();
        assert_eq!(set.capability, "hvac".into());
        assert_eq!(set.channel, "target".into());
    }

    #[test]
    fn test_parse_capability_ref() {
        let mut input = new_input("hvac:target");
        let result = parse_capability_ref(&mut input);
        assert!(result.is_ok());

        let (cap, chan) = result.unwrap();
        assert_eq!(cap, "hvac");
        assert_eq!(chan, "target");
    }
}
