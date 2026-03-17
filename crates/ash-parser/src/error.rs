//! Error types for the Ash parser.
//!
//! This module defines `ParseError`, the error type used throughout the parser
//! for reporting syntax errors with location information and suggestions.

use crate::token::Span;
use std::fmt;

/// A parse error with location information.
///
/// `ParseError` carries information about where an error occurred in the source,
/// what went wrong, and what was expected instead.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    /// The source location of the error.
    pub span: Span,
    /// A human-readable description of the error.
    pub message: String,
    /// A list of what was expected at this position.
    pub expected: Vec<String>,
}

impl ParseError {
    /// Creates a new parse error at the given span with the given message.
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_parser::error::ParseError;
    /// use ash_parser::token::Span;
    ///
    /// let span = Span::new(0, 1, 1, 1);
    /// let error = ParseError::new(span, "unexpected token");
    /// assert_eq!(error.message, "unexpected token");
    /// ```
    pub fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            expected: Vec::new(),
        }
    }

    /// Adds an expected value to the error.
    ///
    /// This can be chained to add multiple expectations.
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_parser::error::ParseError;
    /// use ash_parser::token::Span;
    ///
    /// let span = Span::new(0, 1, 1, 1);
    /// let error = ParseError::new(span, "unexpected token")
    ///     .with_expected("identifier")
    ///     .with_expected("keyword");
    /// assert_eq!(error.expected, vec!["identifier", "keyword"]);
    /// ```
    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.expected.push(expected.into());
        self
    }

    /// Adds multiple expected values to the error.
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_parser::error::ParseError;
    /// use ash_parser::token::Span;
    ///
    /// let span = Span::new(0, 1, 1, 1);
    /// let error = ParseError::new(span, "unexpected token")
    ///     .with_expected_many(&["identifier", "keyword", "literal"]);
    /// assert_eq!(error.expected, vec!["identifier", "keyword", "literal"]);
    /// ```
    pub fn with_expected_many(mut self, expected: &[impl AsRef<str>]) -> Self {
        for e in expected {
            self.expected.push(e.as_ref().to_string());
        }
        self
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "parse error: {}", self.message)?;
        writeln!(
            f,
            "  --> line {}, column {}",
            self.span.line, self.span.column
        )?;
        if !self.expected.is_empty() {
            writeln!(f, "  = expected: {}", self.expected.join(", "))?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_new() {
        let span = Span::new(10, 15, 2, 5);
        let error = ParseError::new(span, "unexpected character");

        assert_eq!(error.span.start, 10);
        assert_eq!(error.span.end, 15);
        assert_eq!(error.span.line, 2);
        assert_eq!(error.span.column, 5);
        assert_eq!(error.message, "unexpected character");
        assert!(error.expected.is_empty());
    }

    #[test]
    fn test_parse_error_with_expected() {
        let span = Span::new(0, 1, 1, 1);
        let error = ParseError::new(span, "unexpected token").with_expected("identifier");

        assert_eq!(error.expected, vec!["identifier"]);
    }

    #[test]
    fn test_parse_error_with_expected_many() {
        let span = Span::new(0, 1, 1, 1);
        let error = ParseError::new(span, "unexpected token")
            .with_expected_many(&["identifier", "keyword"]);

        assert_eq!(error.expected, vec!["identifier", "keyword"]);
    }

    #[test]
    fn test_parse_error_chained_expected() {
        let span = Span::new(0, 1, 1, 1);
        let error = ParseError::new(span, "unexpected token")
            .with_expected("identifier")
            .with_expected("keyword")
            .with_expected("literal");

        assert_eq!(error.expected, vec!["identifier", "keyword", "literal"]);
    }

    #[test]
    fn test_parse_error_display_basic() {
        let span = Span::new(10, 15, 2, 5);
        let error = ParseError::new(span, "unexpected character '}'");

        let display = format!("{}", error);
        assert!(display.contains("parse error: unexpected character '}'"));
        assert!(display.contains("--> line 2, column 5"));
    }

    #[test]
    fn test_parse_error_display_with_expected() {
        let span = Span::new(0, 1, 1, 1);
        let error = ParseError::new(span, "unexpected token")
            .with_expected("identifier")
            .with_expected("keyword");

        let display = format!("{}", error);
        assert!(display.contains("parse error: unexpected token"));
        assert!(display.contains("expected: identifier, keyword"));
    }

    #[test]
    fn test_parse_error_equality() {
        let span1 = Span::new(0, 1, 1, 1);
        let span2 = Span::new(0, 1, 1, 1);
        let span3 = Span::new(1, 2, 1, 2);

        let error1 = ParseError::new(span1, "error");
        let error2 = ParseError::new(span2, "error");
        let error3 = ParseError::new(span3, "error");
        let error4 = ParseError::new(span1, "different error");

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
        assert_ne!(error1, error4);
    }

    #[test]
    fn test_parse_error_clone() {
        let span = Span::new(0, 1, 1, 1);
        let error = ParseError::new(span, "error").with_expected("something");
        let cloned = error.clone();

        assert_eq!(error, cloned);
    }

    #[test]
    fn test_parse_error_error_trait() {
        let span = Span::new(0, 1, 1, 1);
        let error = ParseError::new(span, "test error");

        // Test that it implements std::error::Error
        let _: &dyn std::error::Error = &error;

        // Test description (via Display)
        assert!(format!("{}", error).contains("test error"));
    }
}
