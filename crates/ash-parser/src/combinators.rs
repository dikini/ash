//! Parser combinators for the Ash parser.
//!
//! This module provides basic parser combinators built on top of winnow,
//! specialized for parsing the Ash workflow language.

use winnow::error::ContextError;
use winnow::error::ErrMode;
use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::one_of;
use winnow::token::take_while;

use crate::input::ParseInput;
use crate::input::update_position;

/// Parses zero or more whitespace characters.
///
/// # Examples
///
/// ```
/// use ash_parser::combinators::whitespace;
/// use ash_parser::input::new_input;
/// use winnow::prelude::*;
///
/// let mut input = new_input("   hello");
/// let result = whitespace.parse_next(&mut input);
/// assert!(result.is_ok());
/// ```
pub fn whitespace<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(input)
}

/// Parses a single alphanumeric character.
///
/// # Examples
///
/// ```
/// use ash_parser::combinators::alphanumeric;
/// use ash_parser::input::new_input;
/// use winnow::prelude::*;
///
/// let mut input = new_input("abc");
/// let c = alphanumeric.parse_next(&mut input).unwrap();
/// assert_eq!(c, 'a');
/// ```
pub fn alphanumeric<'a>(input: &mut ParseInput<'a>) -> ModalResult<char> {
    one_of(|c: char| c.is_ascii_alphanumeric()).parse_next(input)
}

/// Consumes optional whitespace before and after the parser.
///
/// # Examples
///
/// ```
/// use ash_parser::combinators::ws;
/// use ash_parser::input::new_input;
/// use ash_parser::input::ParseInput;
/// use winnow::prelude::*;
/// use winnow::token::take_while;
///
/// let mut input = new_input("  hello  ");
/// fn parser<'a>(i: &mut ParseInput<'a>) -> ModalResult<&'a str> {
///     take_while(1.., |c: char| c.is_ascii_alphabetic()).parse_next(i)
/// }
/// let result = ws(parser).parse_next(&mut input);
/// assert!(result.is_ok());
/// ```
#[allow(dead_code)]
pub fn ws<'a, F, O>(mut parser: F) -> impl FnMut(&mut ParseInput<'a>) -> ModalResult<O>
where
    F: FnMut(&mut ParseInput<'a>) -> ModalResult<O>,
{
    move |input: &mut ParseInput<'a>| {
        let _ = whitespace(input)?;
        let output = parser(input)?;
        let _ = whitespace(input)?;
        Ok(output)
    }
}

/// Parses a specific keyword.
///
/// # Examples
///
/// ```
/// use ash_parser::combinators::keyword;
/// use ash_parser::input::new_input;
/// use winnow::prelude::*;
///
/// let mut input = new_input("workflow test");
/// let kw = keyword("workflow").parse_next(&mut input).unwrap();
/// assert_eq!(kw, "workflow");
/// ```
pub fn keyword<'a>(word: &'a str) -> impl Parser<ParseInput<'a>, &'a str, ContextError> {
    move |input: &mut ParseInput<'a>| {
        if input.input.starts_with(word) {
            // Check that the next char is not alphanumeric (to ensure full word match)
            let after = &input.input[word.len()..];
            if after.is_empty() || !after.chars().next().unwrap().is_ascii_alphanumeric() {
                // Advance the input position state
                update_position(&mut input.state, word);
                // Advance the inner stream
                let _ = input.input.next_slice(word.len());
                return Ok(word);
            }
        }
        Err(ErrMode::Backtrack(ContextError::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winnow::combinator::alt;
    use winnow::combinator::opt;
    use winnow::combinator::repeat;

    #[test]
    fn test_whitespace_basic() {
        let mut input = crate::input::new_input("   hello");
        let result = whitespace.parse_next(&mut input).unwrap();
        assert_eq!(result, "   ");
    }

    #[test]
    fn test_whitespace_empty() {
        let mut input = crate::input::new_input("hello");
        let result = whitespace.parse_next(&mut input).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_whitespace_newlines() {
        let mut input = crate::input::new_input("\n\t  hello");
        let result = whitespace.parse_next(&mut input).unwrap();
        assert_eq!(result, "\n\t  ");
    }

    #[test]
    fn test_alphanumeric_success() {
        let mut input = crate::input::new_input("abc");
        let c = alphanumeric.parse_next(&mut input).unwrap();
        assert_eq!(c, 'a');
    }

    #[test]
    fn test_alphanumeric_failure() {
        let mut input = crate::input::new_input("!abc");
        let result = alphanumeric.parse_next(&mut input);
        assert!(result.is_err());
    }

    #[test]
    fn test_keyword_success() {
        let mut input = crate::input::new_input("workflow test");
        let kw = keyword("workflow").parse_next(&mut input).unwrap();
        assert_eq!(kw, "workflow");
    }

    #[test]
    fn test_keyword_with_word_boundary() {
        // "if" should not match "ifx"
        let mut input = crate::input::new_input("ifx");
        let result = keyword("if").parse_next(&mut input);
        assert!(result.is_err());
    }

    #[test]
    fn test_keyword_at_end() {
        let mut input = crate::input::new_input("if");
        let kw = keyword("if").parse_next(&mut input).unwrap();
        assert_eq!(kw, "if");
    }

    #[test]
    fn test_ws_combinator() {
        let mut input = crate::input::new_input("  hello  ");
        let result = ws(|i: &mut ParseInput| keyword("hello").parse_next(i))(&mut input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_combinator_chaining() {
        // Test that we can chain combinators together
        let mut parser = (keyword("let"), whitespace, alphanumeric);

        let mut input = crate::input::new_input("let x = 5");
        let result = parser.parse_next(&mut input);
        assert!(result.is_ok());
        let (kw, _, c) = result.unwrap();
        assert_eq!(kw, "let");
        assert_eq!(c, 'x');
    }

    #[test]
    fn test_alt_combinator() {
        let mut parser = alt((keyword("if"), keyword("let"), keyword("fn")));

        let mut input1 = crate::input::new_input("if x");
        assert_eq!(parser.parse_next(&mut input1).unwrap(), "if");

        let mut input2 = crate::input::new_input("let y");
        assert_eq!(parser.parse_next(&mut input2).unwrap(), "let");

        let mut input3 = crate::input::new_input("fn main");
        assert_eq!(parser.parse_next(&mut input3).unwrap(), "fn");
    }

    #[test]
    fn test_repeat_combinator() {
        // alphanumeric matches both letters AND digits
        let mut input = crate::input::new_input("abc123");
        let result: Result<Vec<char>, _> = repeat(0.., alphanumeric).parse_next(&mut input);
        assert!(result.is_ok());
        let chars = result.unwrap();
        // 'a', 'b', 'c', '1', '2', '3' - all 6 chars are alphanumeric
        assert_eq!(chars.len(), 6);
    }

    #[test]
    fn test_opt_combinator() {
        let mut input = crate::input::new_input("hello");
        let result: Result<Option<&str>, _> = opt(keyword("if")).parse_next(&mut input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }
}
