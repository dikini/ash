//! LSP diagnostic trait and types for the Ash compiler.
//!
//! This crate defines `AshLspError`, a uniform trait for all Ash compiler errors
//! that can be surfaced as LSP diagnostics, together with lightweight
//! `Diagnostic`, `Range`, `Position`, and `Severity` types.

/// Source span used in diagnostics.
///
/// Mirrors the shape of `ash_parser::token::Span` so conversions are trivial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// Byte offset from the start of the file.
    pub start: usize,
    /// Byte offset of the end of the token.
    pub end: usize,
    /// Line number (1-indexed).
    pub line: usize,
    /// Column number (1-indexed).
    pub column: usize,
}

impl Span {
    /// Creates a new span with the given parameters.
    #[must_use]
    pub const fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }
}

/// Lightweight newtype for diagnostic codes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticCode(pub String);

/// Diagnostic severity levels aligned with LSP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
}

/// Uniform trait for all Ash compiler errors that can be surfaced as LSP diagnostics.
pub trait AshLspError: std::fmt::Display + std::error::Error {
    /// Source location of the error, if available.
    fn span(&self) -> Option<Span>;

    /// Severity of the diagnostic.
    fn severity(&self) -> Severity;

    /// Optional stable diagnostic code.
    fn code(&self) -> Option<DiagnosticCode>;

    /// Human-readable message.
    ///
    /// Defaults to the `Display` representation.
    fn message(&self) -> String {
        self.to_string()
    }
}

/// A simple LSP-style diagnostic representation (no actual lsp-types dependency).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Source range.
    pub range: Range,
    /// Severity level.
    pub severity: Option<Severity>,
    /// Stable diagnostic code.
    pub code: Option<String>,
    /// Source identifier (e.g. `"ash"`).
    pub source: Option<String>,
    /// Human-readable message.
    pub message: String,
}

/// A zero-cost LSP-style range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    /// Start position.
    pub start: Position,
    /// End position.
    pub end: Position,
}

/// A zero-cost LSP-style position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// Zero-based line index.
    pub line: u32,
    /// Zero-based character offset.
    pub character: u32,
}

/// Convert an Ash error to a diagnostic.
///
/// Returns `None` when the error does not carry a source span.
pub fn ash_error_to_diagnostic(err: &dyn AshLspError) -> Option<Diagnostic> {
    let span = err.span()?;
    // Compute end column from byte-width of the span.  This is accurate for
    // single-line spans (the common case).  For multi-line spans the end
    // column is approximate but still strictly better than a 1-char range.
    let byte_width = span.end.saturating_sub(span.start);
    let end_col = span.column.saturating_add(byte_width);
    Some(Diagnostic {
        range: Range {
            start: Position {
                line: span.line.saturating_sub(1) as u32,
                character: span.column.saturating_sub(1) as u32,
            },
            end: Position {
                line: span.line.saturating_sub(1) as u32,
                character: end_col.saturating_sub(1) as u32,
            },
        },
        severity: Some(err.severity()),
        code: err.code().map(|c| c.0),
        source: Some("ash".into()),
        message: err.message(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, thiserror::Error)]
    #[error("test error")]
    struct TestError {
        span: Span,
    }

    impl AshLspError for TestError {
        fn span(&self) -> Option<Span> {
            Some(self.span)
        }
        fn severity(&self) -> Severity {
            Severity::Error
        }
        fn code(&self) -> Option<DiagnosticCode> {
            Some(DiagnosticCode("T001".into()))
        }
    }

    #[test]
    fn test_ash_error_to_diagnostic() {
        let err = TestError {
            span: Span::new(10, 15, 2, 5),
        };
        let diag = ash_error_to_diagnostic(&err).unwrap();
        assert_eq!(diag.range.start.line, 1);
        assert_eq!(diag.range.start.character, 4);
        // byte_width = 15 - 10 = 5, end_col = 5 + 5 = 10, end character = 9
        assert_eq!(diag.range.end.character, 9);
        assert_eq!(diag.severity, Some(Severity::Error));
        assert_eq!(diag.code, Some("T001".into()));
        assert_eq!(diag.message, "test error");
        assert_eq!(diag.source, Some("ash".into()));
    }

    #[test]
    fn test_ash_error_to_diagnostic_zero_width_span() {
        let err = TestError {
            span: Span::default(),
        };
        let diag = ash_error_to_diagnostic(&err).unwrap();
        // Zero-width span: start == end == 0, so end_col = 0 + 0 = 0.
        // Both positions are (0, 0) after 1-indexed → 0-indexed with underflow guard.
        assert_eq!(diag.range.start.line, 0);
        assert_eq!(diag.range.end.line, 0);
    }
}
