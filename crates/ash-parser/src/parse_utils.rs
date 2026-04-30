//! Common parsing utilities for Ash parser
//!
//! This module provides shared helper functions used across multiple
//! parser modules for whitespace handling, keyword parsing, and
//! capability reference parsing.

use std::collections::HashMap;

use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::take_while;

use crate::input::{ParseInput, offset_to_span};
use crate::token::Span;

const LINE_COMMENT_PREFIXES: [&str; 2] = ["--", "//"];

/// Check if a string is a reserved keyword.
///
/// This is the canonical keyword list for the Ash language, synchronized with
/// the lexer's `lookup_keyword`. All parser modules must use this function
/// (or delegate through `identifier_with_span`) rather than maintaining
/// their own copies.
pub(crate) fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        // Workflow
        "workflow"
        | "capability"
        | "policy"
        | "role"
        // OODA loop
        | "observe"
        | "orient"
        | "propose"
        | "decide"
        | "act"
        // Control flow
        | "oblige"
        | "check"
        | "let"
        | "if"
        | "then"
        | "else"
        | "for"
        | "do"
        | "with"
        // Effect
        | "maybe"
        | "must"
        | "attempt"
        | "retry"
        | "timeout"
        | "done"
        | "ret"
        // Effect levels
        | "epistemic"
        | "deliberative"
        | "evaluative"
        | "operational"
        // Capability
        | "authority"
        | "obligations"
        // Type
        | "when"
        | "returns"
        | "where"
        // Policy
        | "permit"
        | "deny"
        | "require_approval"
        | "escalate"
        // Pure function
        | "fn"
        | "panic"
        | "match"
        | "fail"
        | "with_error"
        // Contract
        | "requires"
        | "ensures"
        // Workflow statements (contextual, but reserved)
        | "set"
        | "send"
        // Operator
        | "in"
        | "not"
        | "and"
        | "or"
        // Literals
        | "true"
        | "false"
        | "null"
    )
}

/// Returns whether `c` may continue an Ash identifier.
///
/// Keep keyword-boundary checks in sync with [`identifier_with_span`] so a
/// contextual keyword parser cannot consume the prefix of a longer legal
/// identifier such as `fail_count` or `with_error-handler`.
pub(crate) fn is_identifier_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

/// Parse an identifier and return it with its source span.
///
/// Canonical implementation: all parser modules delegate here rather than
/// maintaining their own copies. First character must be a letter or
/// underscore; subsequent characters may be alphanumeric, underscore, or
/// hyphen. Keywords are rejected.
pub(crate) fn identifier_with_span<'a>(input: &mut ParseInput<'a>) -> ModalResult<(&'a str, Span)> {
    let start = input.state.source.len() - input.input.len();

    let result: &str = take_while(1.., is_identifier_continue).parse_next(input)?;

    // First character must be a letter or underscore (not a digit)
    let first = result.chars().next();
    if result.is_empty()
        || !first.is_some_and(|c| c.is_ascii_alphabetic()) && !result.starts_with('_')
    {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    // Reject keywords
    if is_keyword(result) {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    let end = start + result.len();
    let span = offset_to_span(input.state.source, start, end);
    input.state.comments.set_last_token(span);
    Ok((result, span))
}

/// Parse an identifier (without span).
pub(crate) fn identifier<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    identifier_with_span(input).map(|(s, _)| s)
}

/// The kind of a comment.
#[derive(Debug, Clone, PartialEq)]
pub enum CommentKind {
    /// `-- ...` or `// ...` style line comment.
    Line,
    /// `/* ... */` style block comment.
    Block,
}

/// A captured comment with its source span.
#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    /// Raw comment text (including delimiters).
    pub text: String,
    /// Kind of comment.
    pub kind: CommentKind,
    /// Source span of the comment.
    pub span: Span,
}

/// Side-table that maps token spans to their leading and trailing comments.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommentTable {
    leading: HashMap<Span, Vec<Comment>>,
    trailing: HashMap<Span, Vec<Comment>>,
    /// The span of the most recently parsed non-comment token.
    pub last_seen_token_span: Option<Span>,
    /// Comments that are pending attachment as leading on the next token.
    pending_leading: Vec<Comment>,
    /// Track whether there has been a newline since `last_seen_token_span`.
    had_newline_since_last_token: bool,
}

impl CommentTable {
    /// Push a leading comment onto the given token span.
    pub fn push_leading(&mut self, span: Span, comment: Comment) {
        if span == Span::default() {
            return;
        }
        self.leading.entry(span).or_default().push(comment);
    }

    /// Push a trailing comment onto the given token span.
    pub fn push_trailing(&mut self, span: Span, comment: Comment) {
        if span == Span::default() {
            return;
        }
        self.trailing.entry(span).or_default().push(comment);
    }

    /// Record that a token with the given span has just been parsed.
    /// Flushes any pending leading comments to this span.
    pub fn set_last_token(&mut self, span: Span) {
        if span == Span::default() {
            return;
        }
        let pending: Vec<_> = self.pending_leading.drain(..).collect();
        for comment in pending {
            self.push_leading(span, comment);
        }
        self.last_seen_token_span = Some(span);
        self.had_newline_since_last_token = false;
    }

    /// Mark that a newline has been seen while skipping whitespace.
    pub(crate) fn mark_newline(&mut self) {
        self.had_newline_since_last_token = true;
    }

    /// Push a comment that was encountered while skipping whitespace.
    /// Uses the heuristic from SPEC-039 §4.4:
    /// - If on the same line as the preceding token, attach as trailing.
    /// - Otherwise, queue as leading for the next token.
    #[allow(clippy::collapsible_if)]
    pub(crate) fn push_skipped_comment(&mut self, comment: Comment) {
        if let Some(last) = self.last_seen_token_span {
            if !self.had_newline_since_last_token {
                self.push_trailing(last, comment);
                return;
            }
        }
        self.pending_leading.push(comment);
    }

    /// Flush any pending leading comments as trailing on the given span.
    /// Used for EOF comments.
    pub fn flush_pending_leading_to_trailing(&mut self, span: Span) {
        if span == Span::default() {
            self.pending_leading.clear();
            return;
        }
        let pending: Vec<_> = self.pending_leading.drain(..).collect();
        for comment in pending {
            self.push_trailing(span, comment);
        }
    }

    /// Returns the leading comments for a given span.
    pub fn leading(&self, span: Span) -> &[Comment] {
        self.leading.get(&span).map_or(&[], |v| v.as_slice())
    }

    /// Returns the trailing comments for a given span.
    pub fn trailing(&self, span: Span) -> &[Comment] {
        self.trailing.get(&span).map_or(&[], |v| v.as_slice())
    }

    /// Total number of comments stored in the table.
    pub fn total_count(&self) -> usize {
        self.leading.values().map(|v| v.len()).sum::<usize>()
            + self.trailing.values().map(|v| v.len()).sum::<usize>()
            + self.pending_leading.len()
    }
}

/// Parse a capability reference in the form `capability:channel`.
///
/// # Examples
///
/// - `sensor:temp`
/// - `kafka:orders`
/// - `config:timeout`
pub fn parse_capability_ref<'a>(input: &mut ParseInput<'a>) -> ModalResult<(&'a str, &'a str)> {
    let capability = identifier_with_span(input)?.0;
    skip_whitespace_and_comments(input);
    literal_str(":").parse_next(input)?;
    skip_whitespace_and_comments(input);
    let channel = identifier_with_span(input)?.0;
    Ok((capability, channel))
}

/// Parse a keyword, ensuring word boundary.
pub fn keyword<'a>(input: &mut ParseInput<'a>, word: &'a str) -> ModalResult<&'a str> {
    if input.input.starts_with(word) {
        let after = &input.input[word.len()..];
        if after.is_empty() || !after.chars().next().unwrap().is_ascii_alphanumeric() {
            let start = crate::input::current_span(input);
            // Update position state
            for c in word.chars() {
                input.state.pos.advance(c);
            }
            // Advance the inner stream
            let _ = input.input.next_slice(word.len());
            let end = crate::input::current_span(input);
            let span = Span::new(start.start, end.start, start.line, start.column);
            input.state.comments.set_last_token(span);
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
            let start = crate::input::current_span(input);
            // Update position state
            for c in s.chars() {
                input.state.pos.advance(c);
            }
            // Advance the inner stream
            let _ = input.input.next_slice(s.len());
            let end = crate::input::current_span(input);
            let span = Span::new(start.start, end.start, start.line, start.column);
            input.state.comments.set_last_token(span);
            Ok(s)
        } else {
            Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ))
        }
    }
}

/// Skip whitespace and comments, recording any comments into the input state's
/// comment table.
pub fn skip_whitespace_and_comments(input: &mut ParseInput) {
    loop {
        // Skip whitespace
        let _ws_start = input.state.pos;
        let ws: ModalResult<&str> =
            take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(input);
        if let Ok(ws_text) = ws {
            for c in ws_text.chars() {
                if c == '\n' {
                    input.state.comments.mark_newline();
                }
                input.state.pos.advance(c);
            }
        }

        // Check for line comment
        if consume_line_comment(input) {
            continue;
        }

        // Check for block comment
        if input.input.starts_with("/*") {
            let start = crate::input::current_span(input);
            let mut text = String::new();
            text.push_str("/*");
            input.state.pos.advance('/');
            input.state.pos.advance('*');
            let _ = input.input.next_slice(2);
            let mut depth = 1;
            while depth > 0 && !input.input.is_empty() {
                if input.input.starts_with("/*") {
                    text.push_str("/*");
                    let _ = input.input.next_slice(2);
                    input.state.pos.advance('/');
                    input.state.pos.advance('*');
                    depth += 1;
                } else if input.input.starts_with("*/") {
                    text.push_str("*/");
                    let _ = input.input.next_slice(2);
                    input.state.pos.advance('*');
                    input.state.pos.advance('/');
                    depth -= 1;
                } else {
                    let c = input.input.next_token().unwrap_or('\0');
                    text.push(c);
                    input.state.pos.advance(c);
                }
            }
            let end = crate::input::current_span(input);
            let span = Span::new(start.start, end.start, start.line, start.column);
            input.state.comments.push_skipped_comment(Comment {
                text,
                kind: CommentKind::Block,
                span,
            });
            continue;
        }

        break;
    }
}

/// Skip horizontal whitespace (spaces/tabs), line comments, and block comments,
/// but NOT newlines. Used before statement boundary checks where the newline
/// itself is the delimiter.
pub fn skip_horizontal_ws_and_comments(input: &mut ParseInput) {
    loop {
        // Skip horizontal whitespace only (spaces and tabs, not newlines)
        let ws: ModalResult<&str> =
            take_while(0.., |c: char| c == ' ' || c == '\t').parse_next(input);
        if let Ok(ws_text) = ws {
            for c in ws_text.chars() {
                input.state.pos.advance(c);
            }
        }

        // Check for line comment: consume up to (but not including) the newline
        if consume_line_comment(input) {
            continue;
        }

        // Check for block comment (skip it, may span multiple lines)
        if input.input.starts_with("/*") {
            let start = crate::input::current_span(input);
            let mut text = String::new();
            text.push_str("/*");
            input.state.pos.advance('/');
            input.state.pos.advance('*');
            let _ = input.input.next_slice(2);
            let mut depth = 1;
            while depth > 0 && !input.input.is_empty() {
                if input.input.starts_with("/*") {
                    text.push_str("/*");
                    let _ = input.input.next_slice(2);
                    input.state.pos.advance('/');
                    input.state.pos.advance('*');
                    depth += 1;
                } else if input.input.starts_with("*/") {
                    text.push_str("*/");
                    let _ = input.input.next_slice(2);
                    input.state.pos.advance('*');
                    input.state.pos.advance('/');
                    depth -= 1;
                } else {
                    let c = input.input.next_token().unwrap_or('\0');
                    text.push(c);
                    input.state.pos.advance(c);
                }
            }
            let end = crate::input::current_span(input);
            let span = Span::new(start.start, end.start, start.line, start.column);
            input.state.comments.push_skipped_comment(Comment {
                text,
                kind: CommentKind::Block,
                span,
            });
            continue;
        }

        break;
    }
}

fn consume_line_comment(input: &mut ParseInput) -> bool {
    let Some(prefix) = LINE_COMMENT_PREFIXES
        .iter()
        .find(|prefix| input.input.starts_with(**prefix))
        .copied()
    else {
        return false;
    };

    let start = crate::input::current_span(input);
    let mut text = String::from(prefix);
    for c in prefix.chars() {
        input.state.pos.advance(c);
    }
    let _ = input.input.next_slice(prefix.len());
    let rest: ModalResult<&str> = take_while(0.., |c: char| c != '\n').parse_next(input);
    if let Ok(r) = rest {
        text.push_str(r);
        for c in r.chars() {
            input.state.pos.advance(c);
        }
    }
    let end = crate::input::current_span(input);
    let span = Span::new(start.start, end.start, start.line, start.column);
    input.state.comments.push_skipped_comment(Comment {
        text,
        kind: CommentKind::Line,
        span,
    });

    true
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
        assert_eq!(input.state.comments.total_count(), 1);
    }

    #[test]
    fn test_skip_block_comment() {
        let mut input = new_input("/* block */hello");
        skip_whitespace_and_comments(&mut input);
        assert!(input.input.starts_with("hello"));
        assert_eq!(input.state.comments.total_count(), 1);
    }

    // Classification matrix from SPEC-039 §4.4.1

    #[test]
    fn test_comment_table_simple_trailing() {
        // "1 -- trailing\n" - trailing on 1
        let mut input = new_input("1 -- trailing\n");
        // Simulate parsing token '1' at span (0,1,1,1)
        let _: ModalResult<&str> =
            take_while(1.., |c: char| c.is_ascii_digit()).parse_next(&mut input);
        input.state.comments.set_last_token(Span::new(0, 1, 1, 1));
        skip_whitespace_and_comments(&mut input);
        let span = Span::new(0, 1, 1, 1);
        assert_eq!(input.state.comments.trailing(span).len(), 1);
        assert_eq!(input.state.comments.trailing(span)[0].text, "-- trailing");
    }

    #[test]
    fn test_comment_table_simple_leading() {
        // "-- leading\n1" - leading on 1
        let mut input = new_input("-- leading\n1");
        skip_whitespace_and_comments(&mut input);
        // Now parse token '1'
        let span = Span::new(11, 12, 2, 1);
        input.state.comments.set_last_token(span);
        assert_eq!(input.state.comments.leading(span).len(), 1);
        assert_eq!(input.state.comments.leading(span)[0].text, "-- leading");
    }

    #[test]
    fn test_comment_table_blank_line_separator() {
        // "1\n\n-- leading\n2"
        let mut input = new_input("1\n\n-- leading\n2");
        let _: ModalResult<&str> =
            take_while(1.., |c: char| c.is_ascii_digit()).parse_next(&mut input);
        input.state.comments.set_last_token(Span::new(0, 1, 1, 1));
        skip_whitespace_and_comments(&mut input);
        let span = Span::new(14, 15, 4, 1);
        input.state.comments.set_last_token(span);
        assert_eq!(input.state.comments.leading(span).len(), 1);
    }
    #[test]
    fn test_comment_table_consecutive_same_line() {
        // "1 -- a -- b\n" - line comment runs to end of line, so this is one comment
        let mut input = new_input("1 -- a -- b\n");
        let _: ModalResult<&str> =
            take_while(1.., |c: char| c.is_ascii_digit()).parse_next(&mut input);
        input.state.comments.set_last_token(Span::new(0, 1, 1, 1));
        skip_whitespace_and_comments(&mut input);
        let span = Span::new(0, 1, 1, 1);
        let trailing = input.state.comments.trailing(span);
        assert_eq!(trailing.len(), 1);
        assert_eq!(trailing[0].text, "-- a -- b");
    }

    #[test]
    fn test_comment_table_consecutive_multiline() {
        // "-- a\n-- b\n1"
        let mut input = new_input("-- a\n-- b\n1");
        skip_whitespace_and_comments(&mut input);
        let span = Span::new(10, 11, 3, 1);
        input.state.comments.set_last_token(span);
        assert_eq!(input.state.comments.leading(span).len(), 2);
    }

    #[test]
    fn test_comment_table_eof_trailing() {
        // "1 -- eof"
        let mut input = new_input("1 -- eof");
        let _: ModalResult<&str> =
            take_while(1.., |c: char| c.is_ascii_digit()).parse_next(&mut input);
        input.state.comments.set_last_token(Span::new(0, 1, 1, 1));
        skip_whitespace_and_comments(&mut input);
        let span = Span::new(0, 1, 1, 1);
        input.state.comments.flush_pending_leading_to_trailing(span);
        assert_eq!(input.state.comments.trailing(span).len(), 1);
        assert_eq!(input.state.comments.trailing(span)[0].text, "-- eof");
    }

    #[test]
    fn test_comment_table_eof_leading_only_no_prior_token() {
        // "-- eof\n"
        let mut input = new_input("-- eof\n");
        skip_whitespace_and_comments(&mut input);
        // No token to attach to; pending should remain
        assert_eq!(input.state.comments.pending_leading.len(), 1);
        // flush with default span should drop
        input
            .state
            .comments
            .flush_pending_leading_to_trailing(Span::default());
        assert_eq!(input.state.comments.total_count(), 0);
    }

    #[test]
    fn test_comment_table_comment_before_first_token() {
        // "-- header\nmodule M"
        let mut input = new_input("-- header\nmodule M");
        skip_whitespace_and_comments(&mut input);
        let span = Span::new(10, 16, 2, 1);
        input.state.comments.set_last_token(span);
        assert_eq!(input.state.comments.leading(span).len(), 1);
        assert_eq!(input.state.comments.leading(span)[0].text, "-- header");
    }
}
