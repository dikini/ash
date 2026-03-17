//! Ash Parser
//!
//! This crate provides the lexer and parser for the Ash workflow language.

pub mod combinators;
pub mod desugar;
pub mod error;
pub mod error_recovery;
pub mod input;
pub mod lexer;
pub mod lower;
pub mod parse_expr;
pub mod parse_workflow;
pub mod surface;
pub mod token;

pub use combinators::*;
pub use desugar::*;
pub use error::*;
pub use error_recovery::*;
pub use input::*;
pub use lexer::*;
pub use lower::*;
pub use parse_expr::*;
pub use parse_workflow::*;
pub use surface::*;
pub use token::*;

#[cfg(test)]
mod lib_tests {
    // Integration tests for the parser modules

    use super::*;

    #[test]
    fn test_modules_are_public() {
        // Verify all modules are accessible
        let _ = new_input("test");
        let span = Span::new(0, 1, 1, 1);
        let _ = ParseError::new(span, "test error");
    }

    #[test]
    fn test_winnow_integration() {
        use winnow::prelude::*;
        use winnow::token::take_while;

        // Test that winnow parsers work with ParseInput
        let mut input = new_input("hello world");
        let result: PResult<&str> = take_while(1.., |c: char| c.is_ascii_alphabetic())
            .parse_next(&mut input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_end_to_end_basic() {
        // Basic end-to-end test demonstrating parser components working together
        let input_str = "test input";
        let input = new_input(input_str);

        // Verify input tracking
        assert_eq!(input.state.offset, 0);
        assert_eq!(input.state.line, 1);
        assert_eq!(input.state.column, 1);

        // Create a span
        let span = Span::new(0, 4, 1, 1);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 4);

        // Create an error
        let error = ParseError::new(span, "test message")
            .with_expected("something else");
        assert_eq!(error.message, "test message");
        assert_eq!(error.expected.len(), 1);
    }
}
