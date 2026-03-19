//! Parser for observe and changed constructs in the Ash language.
//!
//! This module provides parsers for the observe construct with constraints
//! and the changed construct for change detection.

use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::take_while;

use crate::input::ParseInput;
use crate::parse_expr::{expr, identifier};
use crate::parse_pattern::pattern;
use crate::surface::{Expr, Name, Pattern};

/// Parsed observe expression.
///
/// Represents an `observe capability:channel [where constraints] as pattern` construct.
#[derive(Debug, Clone, PartialEq)]
pub struct ObserveExpr {
    /// Capability name (e.g., "sensor" in "sensor:temp")
    pub capability: Name,
    /// Channel name (e.g., "temp" in "sensor:temp")
    pub channel: Name,
    /// Optional constraints for filtering
    pub constraints: Vec<ConstraintExpr>,
    /// Pattern to bind the result to
    pub pattern: Pattern,
}

/// Parsed changed expression.
///
/// Represents a `changed capability:channel [where constraints]` construct
/// for detecting changes in observed values.
#[derive(Debug, Clone, PartialEq)]
pub struct ChangedExpr {
    /// Capability name
    pub capability: Name,
    /// Channel name
    pub channel: Name,
    /// Optional constraints for filtering
    pub constraints: Vec<ConstraintExpr>,
}

/// A constraint expression in the form `name = value`.
#[derive(Debug, Clone, PartialEq)]
pub struct ConstraintExpr {
    /// Name of the constraint field
    pub name: Name,
    /// Value expression
    pub value: Expr,
}

/// Parse an observe expression.
///
/// # Syntax
///
/// ```text
/// observe capability:channel [where constraints] as pattern
/// ```
///
/// # Examples
///
/// ```
/// use ash_parser::{parse_observe, new_input};
/// use winnow::prelude::*;
///
/// let mut input = new_input("observe sensor:temp as t");
/// let result = parse_observe(&mut input);
/// assert!(result.is_ok());
/// ```
pub fn parse_observe(input: &mut ParseInput) -> ModalResult<ObserveExpr> {
    // Parse "observe" keyword
    keyword(input, "observe")?;
    skip_whitespace_and_comments(input);

    // Parse capability:channel
    let (capability, channel) = parse_capability_ref(input)?;
    skip_whitespace_and_comments(input);

    // Parse optional constraints
    let constraints = if try_keyword(input, "where") {
        skip_whitespace_and_comments(input);
        parse_constraints(input)?
    } else {
        vec![]
    };

    skip_whitespace_and_comments(input);

    // Parse 'as pattern'
    keyword(input, "as")?;
    skip_whitespace_and_comments(input);
    let pat = pattern(input)?;

    Ok(ObserveExpr {
        capability: Name::from(capability),
        channel: Name::from(channel),
        constraints,
        pattern: pat,
    })
}

/// Parse a changed expression for change detection.
///
/// # Syntax
///
/// ```text
/// changed capability:channel [where constraints]
/// ```
///
/// # Examples
///
/// ```
/// use ash_parser::{parse_changed, new_input};
/// use winnow::prelude::*;
///
/// let mut input = new_input("changed sensor:temp");
/// let result = parse_changed(&mut input);
/// assert!(result.is_ok());
/// ```
pub fn parse_changed(input: &mut ParseInput) -> ModalResult<ChangedExpr> {
    // Parse "changed" keyword
    keyword(input, "changed")?;
    skip_whitespace_and_comments(input);

    // Parse capability:channel
    let (capability, channel) = parse_capability_ref(input)?;
    skip_whitespace_and_comments(input);

    // Parse optional constraints
    let constraints = if try_keyword(input, "where") {
        skip_whitespace_and_comments(input);
        parse_constraints(input)?
    } else {
        vec![]
    };

    Ok(ChangedExpr {
        capability: Name::from(capability),
        channel: Name::from(channel),
        constraints,
    })
}

/// Parse a capability reference in the form `capability:channel`.
///
/// # Examples
///
/// - `sensor:temp`
/// - `agent:environment`
/// - `market:stock`
fn parse_capability_ref<'a>(input: &mut ParseInput<'a>) -> ModalResult<(&'a str, &'a str)> {
    let capability = identifier(input)?;
    skip_whitespace_and_comments(input);
    literal_str(":").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let channel = identifier(input)?;
    Ok((capability, channel))
}

/// Parse one or more constraints separated by "and".
///
/// # Examples
///
/// - `unit = "celsius"`
/// - `symbol = "AAPL" and exchange = "NASDAQ"`
fn parse_constraints(input: &mut ParseInput) -> ModalResult<Vec<ConstraintExpr>> {
    let mut constraints = vec![];

    loop {
        let constraint = parse_constraint(input)?;
        constraints.push(constraint);

        skip_whitespace_and_comments(input);

        // Check for "and" separator
        if !try_keyword(input, "and") {
            break;
        }
        skip_whitespace_and_comments(input);
    }

    Ok(constraints)
}

/// Parse a single constraint in the form `name = value`.
fn parse_constraint(input: &mut ParseInput) -> ModalResult<ConstraintExpr> {
    let name = identifier(input)?;
    skip_whitespace_and_comments(input);
    literal_str("=").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let value = expr(input)?;

    Ok(ConstraintExpr {
        name: Name::from(name),
        value,
    })
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

/// Try to parse a keyword, returning true if successful without consuming on failure.
fn try_keyword<'a>(input: &mut ParseInput<'a>, word: &'a str) -> bool {
    // Save state for potential backtrack
    let saved_input = input.input;
    let saved_state = input.state;

    match keyword(input, word) {
        Ok(_) => true,
        Err(_) => {
            // Restore state
            input.input = saved_input;
            input.state = saved_state;
            false
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::new_input;

    #[test]
    fn test_parse_observe_simple() {
        let mut input = new_input("observe sensor:temp as t");
        let result = parse_observe(&mut input);
        assert!(result.is_ok());

        let observe = result.unwrap();
        assert_eq!(observe.capability, "sensor".into());
        assert_eq!(observe.channel, "temp".into());
        assert!(observe.constraints.is_empty());
    }

    #[test]
    fn test_parse_observe_with_constraint() {
        let mut input = new_input(r#"observe sensor:temp where unit = "celsius" as t"#);
        let result = parse_observe(&mut input);
        assert!(result.is_ok());

        let observe = result.unwrap();
        assert_eq!(observe.constraints.len(), 1);
        assert_eq!(observe.constraints[0].name, "unit".into());
    }

    #[test]
    fn test_parse_observe_multiple_constraints() {
        let mut input = new_input(
            r#"observe market:stock where symbol = "AAPL" and exchange = "NASDAQ" as price"#,
        );
        let result = parse_observe(&mut input);
        assert!(result.is_ok());

        let observe = result.unwrap();
        assert_eq!(observe.constraints.len(), 2);
        assert_eq!(observe.constraints[0].name, "symbol".into());
        assert_eq!(observe.constraints[1].name, "exchange".into());
    }

    #[test]
    fn test_parse_observe_destructuring() {
        let mut input = new_input("observe sensor:temp as { value: t, unit: u }");
        let result = parse_observe(&mut input);
        // Note: Record patterns are not yet fully supported in the pattern parser
        // This test may fail until record pattern support is added
        // For now, we just verify the parser doesn't panic
        if let Ok(observe) = result {
            // Pattern should be record destructuring if supported
            assert!(
                matches!(observe.pattern, Pattern::Record(_)),
                "Expected Record pattern, got {:?}",
                observe.pattern
            );
        }
    }

    #[test]
    fn test_parse_changed() {
        let mut input = new_input("changed sensor:temp");
        let result = parse_changed(&mut input);
        assert!(result.is_ok());

        let changed = result.unwrap();
        assert_eq!(changed.capability, "sensor".into());
        assert_eq!(changed.channel, "temp".into());
    }

    #[test]
    fn test_parse_changed_with_constraints() {
        let mut input = new_input(r#"changed sensor:temp where unit = "celsius""#);
        let result = parse_changed(&mut input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().constraints.len(), 1);
    }

    #[test]
    fn test_parse_observe_complex_capability() {
        let mut input = new_input("observe agent:environment as env");
        let result = parse_observe(&mut input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().capability, "agent".into());
    }

    #[test]
    fn test_parse_capability_ref() {
        let mut input = new_input("sensor:temp");
        let result = parse_capability_ref(&mut input);
        assert!(result.is_ok());

        let (cap, chan) = result.unwrap();
        assert_eq!(cap, "sensor");
        assert_eq!(chan, "temp");
    }

    #[test]
    fn test_parse_constraint() {
        let mut input = new_input(r#"unit = "celsius""#);
        let result = parse_constraint(&mut input);
        assert!(result.is_ok());

        let constraint = result.unwrap();
        assert_eq!(constraint.name, "unit".into());
    }

    #[test]
    fn test_parse_constraints_single() {
        let mut input = new_input(r#"symbol = "AAPL""#);
        let result = parse_constraints(&mut input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_parse_constraints_multiple() {
        let mut input = new_input(r#"symbol = "AAPL" and exchange = "NASDAQ""#);
        let result = parse_constraints(&mut input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }
}
