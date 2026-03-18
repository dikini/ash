//! Common parsing utilities for Ash parser
//!
//! This module provides shared helper functions used across multiple
//! parser modules for whitespace handling, keyword parsing, and
//! capability reference parsing.

use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::take_while;

use crate::input::ParseInput;
use crate::parse_expr::identifier;

/// Parse a capability reference in the form `capability:channel`.
///
/// # Examples
///
/// - `sensor:temp`
/// - `kafka:orders`
/// - `config:timeout`
pub fn parse_capability_ref<'a>(input: &mut ParseInput<'a>) -> ModalResult<(&'a str, &'a str)> {
    let capability = identifier(input)?;
    skip_whitespace_and_comments(input);
    literal_str(":").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let channel = identifier(input)?;
    Ok((capability, channel))
}

/// Parse a keyword, ensuring word boundary.
pub fn keyword<'a>(input: &mut ParseInput<'a>, word: &'a str) -> ModalResult<&'a str> {
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
pub fn literal_str<'a>(s: &'a str) -> impl FnMut(&mut ParseInput<'a>) -> ModalResult<&'a str> {
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
pub fn skip_whitespace_and_comments(input: &mut ParseInput) {
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
    fn test_parse_capability_ref() {
        let mut input = new_input("sensor:temp");
        let result = parse_capability_ref(&mut input);
        assert!(result.is_ok());

        let (cap, chan) = result.unwrap();
        assert_eq!(cap, "sensor");
        assert_eq!(chan, "temp");
    }

    #[test]
    fn test_keyword_matching() {
        let mut input = new_input("set ");
        let result = keyword(&mut input, "set");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "set");
    }

    #[test]
    fn test_keyword_rejects_prefix() {
        let mut input = new_input("setting");
        let result = keyword(&mut input, "set");
        assert!(result.is_err());
    }

    #[test]
    fn test_skip_whitespace() {
        let mut input = new_input("   hello");
        skip_whitespace_and_comments(&mut input);
        assert!(input.input.starts_with("hello"));
    }

    #[test]
    fn test_skip_line_comment() {
        let mut input = new_input("-- comment\nhello");
        skip_whitespace_and_comments(&mut input);
        assert!(input.input.starts_with("hello"));
    }

    #[test]
    fn test_skip_block_comment() {
        let mut input = new_input("/* block */hello");
        skip_whitespace_and_comments(&mut input);
        assert!(input.input.starts_with("hello"));
    }
}
