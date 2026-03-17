//! Input handling for the Ash parser.
//!
//! This module provides `ParseInput`, a custom input type for winnow parsers
//! that tracks source offsets, line numbers, and column positions for accurate
//! error reporting and span generation.

use crate::token::Span;
use winnow::stream::Located;
use winnow::stream::Stateful;

/// Position state that tracks line and column information.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Position {
    /// Byte offset from the start of the original input.
    pub offset: usize,
    /// Current line number (1-indexed).
    pub line: usize,
    /// Current column number (1-indexed).
    pub column: usize,
}

impl Position {
    /// Creates a new Position at the start of input.
    pub fn new() -> Self {
        Self {
            offset: 0,
            line: 1,
            column: 1,
        }
    }

    /// Advance position by a character.
    pub fn advance(&mut self, c: char) {
        self.offset += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }
}

/// A custom input type for winnow parsers that tracks source position.
///
/// `ParseInput` wraps a string slice with location tracking and maintains
/// metadata about the current position in the source.
pub type ParseInput<'a> = Stateful<Located<&'a str>, Position>;

/// Creates a new `ParseInput` from a string slice.
///
/// # Examples
///
/// ```
/// use ash_parser::input::new_input;
///
/// let input = new_input("workflow test {}");
/// assert_eq!(input.state.line, 1);
/// assert_eq!(input.state.column, 1);
/// ```
pub fn new_input(input: &str) -> ParseInput<'_> {
    Stateful {
        input: Located::new(input),
        state: Position::new(),
    }
}

/// Creates a `Span` from the current input state.
///
/// # Examples
///
/// ```
/// use ash_parser::input::new_input;
/// use ash_parser::input::current_span;
///
/// let input = new_input("hello");
/// let span = current_span(&input);
/// assert_eq!(span.line, 1);
/// assert_eq!(span.column, 1);
/// ```
pub fn current_span(input: &ParseInput) -> Span {
    Span {
        start: input.state.offset,
        end: input.state.offset,
        line: input.state.line,
        column: input.state.column,
    }
}

/// Creates a `Span` from a starting position to current position.
pub fn span_from(start: &Position, end: &Position) -> Span {
    Span {
        start: start.offset,
        end: end.offset,
        line: start.line,
        column: start.column,
    }
}

/// Update position state after consuming a string.
pub fn update_position(pos: &mut Position, s: &str) {
    for c in s.chars() {
        pos.advance(c);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winnow::stream::Stream;

    #[test]
    fn test_new_input() {
        let input = new_input("hello world");
        assert_eq!(input.state.offset, 0);
        assert_eq!(input.state.line, 1);
        assert_eq!(input.state.column, 1);
    }

    #[test]
    fn test_position_advance() {
        let mut pos = Position::new();
        pos.advance('h');
        assert_eq!(pos.offset, 1);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 2);

        pos.advance('\n');
        assert_eq!(pos.offset, 2);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 1);
    }

    #[test]
    fn test_position_advance_unicode() {
        let mut pos = Position::new();
        pos.advance('é'); // 2 bytes in UTF-8
        assert_eq!(pos.offset, 2);
        assert_eq!(pos.column, 2);
    }

    #[test]
    fn test_current_span() {
        let input = new_input("hello");
        let span = current_span(&input);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 0);
        assert_eq!(span.line, 1);
        assert_eq!(span.column, 1);
    }

    #[test]
    fn test_span_from() {
        let start = Position::new();
        let mut end = Position::new();
        end.offset = 5;
        end.column = 6;

        let span = span_from(&start, &end);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 5);
        assert_eq!(span.line, 1);
        assert_eq!(span.column, 1);
    }

    #[test]
    fn test_update_position() {
        let mut pos = Position::new();
        update_position(&mut pos, "hello\nworld");

        assert_eq!(pos.offset, 11);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 6);
    }

    #[test]
    fn test_parse_input_stream() {
        let mut input = new_input("abc");
        let c = input.next_token();
        assert_eq!(c, Some('a'));
        assert_eq!(input.state.offset, 0); // Position is tracked separately
    }
}
