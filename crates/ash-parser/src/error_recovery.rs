//! Error recovery for the Ash parser.
//!
//! This module provides utilities for recovering from parse errors
//! and continuing to parse the rest of the input.

use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::take_while;

use crate::error::ParseError;
use crate::input::{new_input, ParseInput};
use crate::surface::Workflow;
use crate::token::Span;

/// A recovered parse result containing either a successful value or an error with partial result.
#[derive(Debug, Clone)]
pub enum Recovered<T> {
    /// Successful parse with the value.
    Ok(T),
    /// Parse failed but we recovered; contains partial result and errors.
    Partial(T, Vec<ParseError>),
    /// Complete failure, no partial result available.
    Err(Vec<ParseError>),
}

impl<T> Recovered<T> {
    /// Returns true if this is an Ok variant.
    pub fn is_ok(&self) -> bool {
        matches!(self, Recovered::Ok(_))
    }

    /// Returns true if this is a Partial variant.
    pub fn is_partial(&self) -> bool {
        matches!(self, Recovered::Partial(_, _))
    }

    /// Returns true if this is an Err variant.
    pub fn is_err(&self) -> bool {
        matches!(self, Recovered::Err(_))
    }

    /// Unwraps the value, panicking if this is an Err.
    pub fn unwrap(self) -> T {
        match self {
            Recovered::Ok(v) | Recovered::Partial(v, _) => v,
            Recovered::Err(_) => panic!("called `Recovered::unwrap()` on an `Err` value"),
        }
    }

    /// Returns the value if available, otherwise returns default.
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Recovered::Ok(v) | Recovered::Partial(v, _) => v,
            Recovered::Err(_) => default,
        }
    }

    /// Returns the errors if any.
    pub fn errors(&self) -> Option<&[ParseError]> {
        match self {
            Recovered::Ok(_) => None,
            Recovered::Partial(_, errs) | Recovered::Err(errs) => Some(errs),
        }
    }
}

/// Recover from a parse error by skipping to the next statement boundary.
///
/// This combinator attempts to parse with the given parser. If parsing fails,
/// it records the error and advances the input to a known recovery point
/// (like the next semicolon or closing brace).
pub fn recover_to_next_stmt<'a, F, O>(
    mut parser: F,
) -> impl FnMut(&mut ParseInput<'a>) -> PResult<O> + 'a
where
    F: FnMut(&mut ParseInput<'a>) -> PResult<O> + 'a,
{
    move |input: &mut ParseInput<'a>| {
        // Try the parser first
        match parser(input) {
            Ok(result) => Ok(result),
            Err(e) => {
                // Recovery: skip to next statement boundary
                skip_to_stmt_boundary(input);
                Err(e)
            }
        }
    }
}

/// Skip input to the next statement boundary.
///
/// Statement boundaries are: semicolons, closing braces, or keywords that
/// start new statements.
fn skip_to_stmt_boundary(input: &mut ParseInput) {
    let stmt_start_keywords = [
        "workflow", "observe", "orient", "propose", "decide", "act",
        "let", "if", "for", "par", "with", "maybe", "must", "done",
        "check",
    ];

    loop {
        if input.input.is_empty() {
            break;
        }

        // Check for statement boundary characters
        if input.input.starts_with(";")
            || input.input.starts_with("}")
            || input.input.starts_with("{")
        {
            break;
        }

        // Check for statement-starting keywords
        let is_keyword_boundary = stmt_start_keywords.iter().any(|kw| {
            input.input.starts_with(kw) && {
                let after = &input.input[kw.len()..];
                after.is_empty()
                    || !after.chars().next().unwrap().is_ascii_alphanumeric()
            }
        });

        if is_keyword_boundary {
            break;
        }

        // Skip comment
        if input.input.starts_with("--") {
            let _: PResult<&str> = take_while(0.., |c: char| c != '\n').parse_next(input);
            continue;
        }

        // Skip block comment
        if input.input.starts_with("/*") {
            let _ = input.input.next_slice(2);
            while !input.input.is_empty() && !input.input.starts_with("*/") {
                let _ = input.input.next_token();
            }
            let _ = input.input.next_slice(2);
            continue;
        }

        // Skip one character
        if let Some(c) = input.input.next_token() {
            input.state.advance(c);
        } else {
            break;
        }
    }
}

/// Parse with error recovery, collecting all errors.
///
/// This function attempts to parse a complete workflow, but if errors occur,
/// it tries to recover and continue parsing, collecting all errors.
pub fn parse_with_recovery(input: &str) -> Recovered<Workflow> {
    let mut input = new_input(input);
    let mut errors = Vec::new();

    // Try to parse as a workflow definition first
    if let Ok(def) = crate::parse_workflow::workflow_def(&mut input) {
        return Recovered::Ok(def.body);
    }

    // Try to parse as a workflow body
    match crate::parse_workflow::workflow(&mut input) {
        Ok(workflow) => {
            if errors.is_empty() {
                Recovered::Ok(workflow)
            } else {
                Recovered::Partial(workflow, errors)
            }
        }
        Err(e) => {
            // Record the error
            let span = current_span(&input);
            errors.push(ParseError::new(span, format!("Parse error: {:?}", e)));

            // Try to recover and continue
            skip_to_stmt_boundary(&mut input);

            if input.input.is_empty() {
                Recovered::Err(errors)
            } else {
                // Try to parse remaining as a workflow
                match crate::parse_workflow::workflow(&mut input) {
                    Ok(workflow) => Recovered::Partial(workflow, errors),
                    Err(_) => Recovered::Err(errors),
                }
            }
        }
    }
}

/// Create a span from the current input position.
fn current_span(input: &ParseInput) -> Span {
    Span {
        start: input.state.offset,
        end: input.state.offset,
        line: input.state.line,
        column: input.state.column,
    }
}

/// Create a recovery error at the current position.
pub fn recovery_error(input: &ParseInput, message: impl Into<String>) -> ParseError {
    ParseError::new(current_span(input), message)
}

/// Attempt to parse, recovering on failure and returning (result, errors).
///
/// This is a higher-level recovery function that wraps any parser and
/// returns both the result and any errors that occurred.
pub fn try_recover<'a, F, O>(mut input: ParseInput<'a>, mut parser: F) -> (Option<O>, Vec<ParseError>)
where
    F: FnMut(&mut ParseInput<'a>) -> PResult<O>,
{
    let mut errors = Vec::new();

    match parser(&mut input) {
        Ok(result) => (Some(result), errors),
        Err(e) => {
            errors.push(recovery_error(&input, "Parse failed"));
            skip_to_stmt_boundary(&mut input);
            (None, errors)
        }
    }
}

/// Synchronize to a safe parsing point.
///
/// This function skips input until it finds a token that can reliably
/// start a new statement, helping the parser resynchronize after errors.
pub fn synchronize(input: &mut ParseInput) {
    skip_to_stmt_boundary(input);

    // Skip any semicolons
    while input.input.starts_with(";") {
        let _ = input.input.next_slice(1);
        input.state.advance(';');
        skip_whitespace_and_comments(input);
    }
}

/// Skip whitespace and comments.
fn skip_whitespace_and_comments(input: &mut ParseInput) {
    loop {
        // Skip whitespace
        let _: PResult<&str> = take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(input);

        // Check for line comment
        if input.input.starts_with("--") {
            let _: PResult<&str> = take_while(0.., |c: char| c != '\n').parse_next(input);
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
    use crate::surface::Workflow;

    #[test]
    fn test_recovered_ok() {
        let r = Recovered::Ok(Workflow::Done { span: Span::default() });
        assert!(r.is_ok());
        assert!(!r.is_err());
    }

    #[test]
    fn test_recovered_partial() {
        let r = Recovered::Partial(
            Workflow::Done { span: Span::default() },
            vec![ParseError::new(Span::default(), "test error")],
        );
        assert!(r.is_partial());
        assert!(!r.is_ok());
        assert!(!r.is_err());
        assert!(r.errors().is_some());
    }

    #[test]
    fn test_recovered_err() {
        let r: Recovered<Workflow> = Recovered::Err(vec![ParseError::new(Span::default(), "error")]);
        assert!(r.is_err());
        assert!(!r.is_ok());
        assert!(r.errors().is_some());
    }

    #[test]
    fn test_recovered_unwrap() {
        let r = Recovered::Ok(42);
        assert_eq!(r.unwrap(), 42);

        let r = Recovered::Partial(42, vec![]);
        assert_eq!(r.unwrap(), 42);
    }

    #[test]
    fn test_recovered_unwrap_or() {
        let r: Recovered<i32> = Recovered::Err(vec![]);
        assert_eq!(r.unwrap_or(99), 99);
    }

    #[test]
    fn test_skip_to_stmt_boundary_semicolon() {
        let mut input = new_input("error token; valid_stmt");
        skip_to_stmt_boundary(&mut input);
        assert!(input.input.starts_with(";"));
    }

    #[test]
    fn test_skip_to_stmt_boundary_brace() {
        let mut input = new_input("error token} valid");
        skip_to_stmt_boundary(&mut input);
        assert!(input.input.starts_with("}"));
    }

    #[test]
    fn test_skip_to_stmt_boundary_keyword() {
        let mut input = new_input("error token done");
        skip_to_stmt_boundary(&mut input);
        assert!(input.input.starts_with("done"));
    }

    #[test]
    fn test_synchronize() {
        let mut input = new_input(";  ; let x = 1");
        synchronize(&mut input);
        assert!(input.input.starts_with("let"));
    }

    #[test]
    fn test_recovery_error() {
        let input = new_input("test");
        let err = recovery_error(&input, "test error");
        assert_eq!(err.message, "test error");
        assert_eq!(err.span.line, 1);
        assert_eq!(err.span.column, 1);
    }

    #[test]
    fn test_try_recover_success() {
        let input = new_input("done");
        let (result, errors) = try_recover(input, crate::parse_workflow::workflow);
        assert!(result.is_some());
        assert!(errors.is_empty());
    }

    #[test]
    fn test_parse_with_recovery_empty() {
        let result = parse_with_recovery("");
        // Empty input should either succeed with Done or fail gracefully
        assert!(!result.is_ok()); // No workflow to parse
    }

    #[test]
    fn test_parse_with_recovery_valid() {
        let result = parse_with_recovery("workflow test { done }");
        assert!(result.is_ok());
    }
}
