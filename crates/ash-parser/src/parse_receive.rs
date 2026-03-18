//! Parser for the receive construct in Ash workflows.
//!
//! This module provides parsers for the `receive` expression which allows
//! pattern matching on stream messages with optional guards and timeouts.

use std::time::Duration;

use winnow::combinator::{alt, delimited, opt};
use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::{one_of, take_while};

use crate::input::{ParseInput, Position};
use crate::parse_expr::{expr, identifier};
use crate::parse_workflow::{pattern, workflow};
use crate::surface::{Expr, Name, Pattern, Workflow};
use crate::token::Span;

/// Receive mode: non-blocking, blocking forever, or blocking with timeout
#[derive(Debug, Clone, PartialEq)]
pub enum ReceiveMode {
    /// Non-blocking receive - check for messages and continue immediately
    NonBlocking,
    /// Blocking receive - wait for messages, optionally with timeout
    Blocking(Option<Duration>),
}

impl ReceiveMode {
    /// Returns true if this is a blocking receive mode
    pub fn is_blocking(&self) -> bool {
        matches!(self, ReceiveMode::Blocking(_))
    }

    /// Returns the timeout duration if set
    pub fn timeout(&self) -> Option<Duration> {
        match self {
            ReceiveMode::Blocking(timeout) => *timeout,
            ReceiveMode::NonBlocking => None,
        }
    }
}

/// Stream pattern for matching messages in receive arms
#[derive(Debug, Clone, PartialEq)]
pub enum StreamPatternExpr {
    /// Wildcard pattern: _
    Wildcard,
    /// String literal pattern (for control messages)
    Literal(String),
    /// Binding pattern: capability:channel as pattern
    Binding {
        /// Capability name
        capability: Name,
        /// Channel name
        channel: Name,
        /// Pattern to bind the message
        pattern: Pattern,
    },
}

/// A receive arm: pattern + optional guard + body
#[derive(Debug, Clone, PartialEq)]
pub struct ReceiveArmExpr {
    /// Pattern to match against incoming messages
    pub pattern: StreamPatternExpr,
    /// Optional guard expression
    pub guard: Option<Expr>,
    /// Body workflow to execute when matched
    pub body: Workflow,
    /// Source span
    pub span: Span,
}

/// Parsed receive expression
#[derive(Debug, Clone, PartialEq)]
pub struct ReceiveExpr {
    /// Receive mode (blocking or non-blocking)
    pub mode: ReceiveMode,
    /// Receive arms for matching messages
    pub arms: Vec<ReceiveArmExpr>,
    /// Whether this is a control receive
    pub is_control: bool,
    /// Source span
    pub span: Span,
}

/// Parse a receive expression
///
/// Grammar:
///   receive [control] [wait [DURATION]] { ARM [, ARM]* }
pub fn parse_receive(input: &mut ParseInput) -> ModalResult<ReceiveExpr> {
    let start_pos = input.state;

    // Parse "receive" keyword
    keyword("receive").parse_next(input)?;
    skip_whitespace(input);

    // Check for optional "control" keyword
    let is_control = if keyword("control").parse_next(input).is_ok() {
        skip_whitespace(input);
        true
    } else {
        false
    };

    // Parse mode: "wait" with optional duration, or non-blocking
    let mode = if keyword("wait").parse_next(input).is_ok() {
        skip_whitespace(input);
        let timeout = opt(parse_duration).parse_next(input)?;
        ReceiveMode::Blocking(timeout)
    } else {
        ReceiveMode::NonBlocking
    };

    skip_whitespace(input);

    // Parse arms in braces: { arm1, arm2, ... }
    let arms =
        delimited(literal_str("{"), parse_receive_arms, literal_str("}")).parse_next(input)?;

    let span = span_from(&start_pos, &input.state);

    Ok(ReceiveExpr {
        mode,
        arms,
        is_control,
        span,
    })
}

/// Parse a comma-separated list of receive arms
fn parse_receive_arms(input: &mut ParseInput) -> ModalResult<Vec<ReceiveArmExpr>> {
    let mut arms = Vec::new();

    loop {
        skip_whitespace(input);

        // Check for empty arms or end of list
        if input.input.is_empty() || input.input.starts_with("}") {
            break;
        }

        // Parse a single arm
        let arm = parse_receive_arm(input)?;
        arms.push(arm);

        skip_whitespace(input);

        // Check for comma separator
        if input.input.starts_with(",") {
            let _ = input.input.next_slice(1);
            input.state.advance(',');
        } else {
            // No comma - either end of list or error
            break;
        }
    }

    Ok(arms)
}

/// Parse a single receive arm
///
/// Grammar:
///   PATTERN [if EXPR] => WORKFLOW
fn parse_receive_arm(input: &mut ParseInput) -> ModalResult<ReceiveArmExpr> {
    let start_pos = input.state;

    // Parse stream pattern
    let pattern = parse_stream_pattern(input)?;
    skip_whitespace(input);

    // Optional guard: "if" expr
    let guard = if keyword("if").parse_next(input).is_ok() {
        skip_whitespace(input);
        Some(expr(input)?)
    } else {
        None
    };

    skip_whitespace(input);

    // Arrow separator
    literal_str("=>").parse_next(input)?;
    skip_whitespace(input);

    // Body workflow
    let body = workflow(input)?;

    let span = span_from(&start_pos, &input.state);

    Ok(ReceiveArmExpr {
        pattern,
        guard,
        body,
        span,
    })
}

/// Parse a stream pattern
///
/// Grammar:
///   _ | STRING | IDENTIFIER : IDENTIFIER as PATTERN
fn parse_stream_pattern(input: &mut ParseInput) -> ModalResult<StreamPatternExpr> {
    skip_whitespace(input);

    alt((
        parse_wildcard_pattern,
        parse_string_literal_pattern,
        parse_binding_pattern,
    ))
    .parse_next(input)
}

/// Parse a wildcard pattern: _
fn parse_wildcard_pattern(input: &mut ParseInput) -> ModalResult<StreamPatternExpr> {
    literal_str("_").parse_next(input)?;
    Ok(StreamPatternExpr::Wildcard)
}

/// Parse a string literal pattern for control messages
fn parse_string_literal_pattern(input: &mut ParseInput) -> ModalResult<StreamPatternExpr> {
    let _ = literal_str("\"").parse_next(input)?;
    let content: &str = take_while(0.., |c: char| c != '"').parse_next(input)?;
    let _ = literal_str("\"").parse_next(input)?;
    Ok(StreamPatternExpr::Literal(content.to_string()))
}

/// Parse a binding pattern: capability:channel as pattern
fn parse_binding_pattern(input: &mut ParseInput) -> ModalResult<StreamPatternExpr> {
    let capability = identifier(input)?.into();
    literal_str(":").parse_next(input)?;
    let channel = identifier(input)?.into();

    skip_whitespace(input);
    keyword("as").parse_next(input)?;
    skip_whitespace(input);

    let pattern = pattern(input)?;

    Ok(StreamPatternExpr::Binding {
        capability,
        channel,
        pattern,
    })
}

/// Parse a duration: NUMBER [s | m | h]
///
/// Examples: 30s, 5m, 1h
fn parse_duration(input: &mut ParseInput) -> ModalResult<Duration> {
    skip_whitespace(input);

    // Parse number
    let digits: &str = take_while(1.., |c: char| c.is_ascii_digit()).parse_next(input)?;
    let num: u64 = digits
        .parse()
        .map_err(|_| winnow::error::ErrMode::Backtrack(winnow::error::ContextError::new()))?;

    // Parse unit
    let unit = one_of(['s', 'm', 'h']).parse_next(input)?;

    Ok(match unit {
        's' => Duration::from_secs(num),
        'm' => Duration::from_secs(num * 60),
        'h' => Duration::from_secs(num * 60 * 60),
        _ => unreachable!(),
    })
}

/// Parse a keyword (ensures word boundary)
fn keyword<'a>(word: &'a str) -> impl Parser<ParseInput<'a>, &'a str, winnow::error::ContextError> {
    move |input: &mut ParseInput<'a>| {
        skip_whitespace(input);

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

/// Parse a string literal token
fn literal_str<'a>(s: &'a str) -> impl FnMut(&mut ParseInput<'a>) -> ModalResult<&'a str> {
    move |input: &mut ParseInput<'a>| {
        skip_whitespace(input);
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

/// Skip whitespace and comments
fn skip_whitespace(input: &mut ParseInput) {
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

/// Create a span from start position to current position
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
    use crate::input::new_input;

    #[test]
    fn test_parse_receive_simple() {
        let input = "receive { sensor:temp as t => done }";
        let result = parse_receive(&mut new_input(input));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let receive = result.unwrap();
        assert!(matches!(receive.mode, ReceiveMode::NonBlocking));
        assert_eq!(receive.arms.len(), 1);
        assert!(!receive.is_control);
    }

    #[test]
    fn test_parse_receive_wait() {
        let input = "receive wait { sensor:temp as t => done }";
        let result = parse_receive(&mut new_input(input));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let receive = result.unwrap();
        assert!(matches!(receive.mode, ReceiveMode::Blocking(None)));
    }

    #[test]
    fn test_parse_receive_wait_timeout() {
        let input = "receive wait 30s { _ => act heartbeat() }";
        let result = parse_receive(&mut new_input(input));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let receive = result.unwrap();
        assert!(matches!(receive.mode, ReceiveMode::Blocking(Some(_))));
        assert_eq!(receive.mode.timeout(), Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_parse_receive_wait_timeout_minutes() {
        let input = "receive wait 5m { _ => done }";
        let result = parse_receive(&mut new_input(input));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let receive = result.unwrap();
        assert_eq!(receive.mode.timeout(), Some(Duration::from_secs(5 * 60)));
    }

    #[test]
    fn test_parse_receive_wait_timeout_hours() {
        let input = "receive wait 1h { _ => done }";
        let result = parse_receive(&mut new_input(input));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let receive = result.unwrap();
        assert_eq!(receive.mode.timeout(), Some(Duration::from_secs(60 * 60)));
    }

    #[test]
    fn test_parse_receive_with_guard() {
        let input = r#"receive { 
            sensor:temp as t if t > 100 => act alert()
        }"#;
        let result = parse_receive(&mut new_input(input));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let receive = result.unwrap();
        assert!(receive.arms[0].guard.is_some());
    }

    #[test]
    fn test_parse_receive_control() {
        // Control receive with string literal patterns for control messages
        let input = r#"receive control { 
            "shutdown" => done,
            _ => done
        }"#;
        let result = parse_receive(&mut new_input(input));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        assert!(result.unwrap().is_control);
    }

    #[test]
    fn test_parse_receive_multiple_arms() {
        let input = r#"receive {
            sensor:temp as t if t > 100 => act alert(),
            sensor:temp as t => act log(t),
            _ => act skip()
        }"#;
        let result = parse_receive(&mut new_input(input));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        assert_eq!(result.unwrap().arms.len(), 3);
    }

    #[test]
    fn test_parse_duration() {
        // Test seconds
        let mut input = new_input("30s");
        let result = parse_duration(&mut input);
        assert_eq!(result.unwrap(), Duration::from_secs(30));

        // Test minutes
        let mut input = new_input("5m");
        let result = parse_duration(&mut input);
        assert_eq!(result.unwrap(), Duration::from_secs(5 * 60));

        // Test hours
        let mut input = new_input("1h");
        let result = parse_duration(&mut input);
        assert_eq!(result.unwrap(), Duration::from_secs(60 * 60));
    }

    #[test]
    fn test_parse_stream_pattern_wildcard() {
        let mut input = new_input("_");
        let result = parse_stream_pattern(&mut input);
        assert!(matches!(result.unwrap(), StreamPatternExpr::Wildcard));
    }

    #[test]
    fn test_parse_stream_pattern_literal() {
        let mut input = new_input("\"shutdown\"");
        let result = parse_stream_pattern(&mut input);
        assert!(matches!(result.unwrap(), StreamPatternExpr::Literal(s) if s == "shutdown"));
    }

    #[test]
    fn test_parse_stream_pattern_binding() {
        let mut input = new_input("sensor:temp as t");
        let result = parse_stream_pattern(&mut input);
        assert!(matches!(
            result.unwrap(),
            StreamPatternExpr::Binding {
                capability,
                channel,
                ..
            } if capability.as_ref() == "sensor" && channel.as_ref() == "temp"
        ));
    }

    #[test]
    fn test_parse_receive_arm_with_guard() {
        let input = "sensor:temp as t if t > 100 => done";
        let result = parse_receive_arm(&mut new_input(input));
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let arm = result.unwrap();
        assert!(arm.guard.is_some());
    }

    #[test]
    fn test_receive_mode_is_blocking() {
        let blocking = ReceiveMode::Blocking(None);
        let blocking_with_timeout = ReceiveMode::Blocking(Some(Duration::from_secs(10)));
        let non_blocking = ReceiveMode::NonBlocking;

        assert!(blocking.is_blocking());
        assert!(blocking_with_timeout.is_blocking());
        assert!(!non_blocking.is_blocking());
    }

    #[test]
    fn test_receive_mode_timeout() {
        let blocking = ReceiveMode::Blocking(Some(Duration::from_secs(30)));
        let blocking_no_timeout = ReceiveMode::Blocking(None);
        let non_blocking = ReceiveMode::NonBlocking;

        assert_eq!(blocking.timeout(), Some(Duration::from_secs(30)));
        assert_eq!(blocking_no_timeout.timeout(), None);
        assert_eq!(non_blocking.timeout(), None);
    }
}
