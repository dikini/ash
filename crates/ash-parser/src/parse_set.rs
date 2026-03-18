//! Parser for set statements in the Ash language.
//!
//! This module provides parsers for the set construct for output behaviours.

use winnow::prelude::*;

use crate::input::ParseInput;
use crate::parse_expr::expr;
use crate::parse_utils::{
    keyword, literal_str, parse_capability_ref, skip_whitespace_and_comments,
};
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
    }

    #[test]
    fn test_parse_set_record() {
        // Using function call syntax for structured values
        let mut input = new_input("set light:room = brightness(80)");
        let result = parse_set(&mut input);
        assert!(result.is_ok());

        let set = result.unwrap();
        assert_eq!(set.capability, "light".into());
        assert_eq!(set.channel, "room".into());
    }

    #[test]
    fn test_parse_set_int_literal() {
        // Test with an integer literal expression
        let mut input = new_input("set hvac:target = 72");
        let result = parse_set(&mut input);
        assert!(result.is_ok(), "Parse failed: {:?}", result);

        let set = result.unwrap();
        assert_eq!(set.capability, "hvac".into());
        assert_eq!(set.channel, "target".into());
        match set.value {
            Expr::Literal(Literal::Int(n)) => assert_eq!(n, 72),
            _ => panic!("Expected int literal, got {:?}", set.value),
        }
    }

    #[test]
    fn test_parse_set_function_call() {
        // Test with a function call expression for structured values
        let mut input = new_input("set light:room = brightness(80)");
        let result = parse_set(&mut input);
        assert!(result.is_ok(), "Parse failed: {:?}", result);

        let set = result.unwrap();
        assert_eq!(set.capability, "light".into());
        assert_eq!(set.channel, "room".into());
        // Value should be a function call
        match set.value {
            Expr::Call { .. } => {}
            _ => panic!("Expected call expression, got {:?}", set.value),
        }
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
        let mut input = new_input("set hvac:target = -- target temperature\n72");
        let result = parse_set(&mut input);
        assert!(result.is_ok());

        let set = result.unwrap();
        assert_eq!(set.capability, "hvac".into());
        assert_eq!(set.channel, "target".into());
    }

    #[test]
    fn test_parse_set_string_value() {
        let mut input = new_input(r#"set config:name = "production""#);
        let result = parse_set(&mut input);
        assert!(result.is_ok());

        let set = result.unwrap();
        assert_eq!(set.capability, "config".into());
        assert_eq!(set.channel, "name".into());
        match set.value {
            Expr::Literal(Literal::String(s)) => assert_eq!(s, "production".into()),
            _ => panic!("Expected string literal, got {:?}", set.value),
        }
    }
}
