//! Use statement parser for the Ash language.
//!
//! This module provides parsers for Rust-style use statements:
//! - `use crate::foo::bar;` - Simple path
//! - `use crate::foo::bar as baz;` - Simple path with alias
//! - `use crate::foo::*;` - Glob import
//! - `use crate::foo::{a, b, c};` - Nested import
//! - `use crate::foo::{a as x, b};` - Nested import with aliases
//! - `pub use crate::foo::bar;` - With visibility modifier

use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::take_while;

use crate::input::{ParseInput, offset_to_span};
use crate::parse_utils::{parse_notation_pattern_part, skip_whitespace_and_comments};
use crate::parse_visibility::parse_visibility;
use crate::surface::{NotationPatternPart, Visibility};
use crate::token::Span;

use crate::use_tree::{NotationImportSelector, SimplePath, Use, UseItem, UsePath};

/// Parse a use statement.
///
/// # Examples
///
/// ```
/// use ash_parser::parse_use::parse_use;
/// use ash_parser::input::new_input;
/// use winnow::prelude::*;
///
/// let mut input = new_input("use crate::foo::bar;");
/// let result = parse_use.parse_next(&mut input).unwrap();
/// ```
pub fn parse_use(input: &mut ParseInput) -> ModalResult<Use> {
    let start_offset = input.state.source.len() - input.input.len();

    // Parse optional visibility
    let visibility = parse_visibility(input)?;

    // Parse 'use' keyword
    let _ = parse_keyword("use")(input)?;

    // Parse whitespace after 'use'
    let _ = parse_whitespace(input)?;

    // Parse the use path
    let path = parse_use_path(input)?;

    // Parse optional alias for the entire import
    let alias = parse_optional_alias(input)?;

    if matches!(path, UsePath::Notation { .. })
        && (!matches!(visibility, Visibility::Inherited) || alias.is_some())
    {
        return Err(winnow::error::ErrMode::Cut(
            winnow::error::ContextError::new(),
        ));
    }

    // Parse semicolon
    let _ = parse_symbol(";")(input)?;

    let end_offset = input.state.source.len() - input.input.len();
    let span = offset_to_span(input.state.source, start_offset, end_offset);

    Ok(Use {
        visibility,
        path,
        alias,
        span,
    })
}

/// Parse the path component of a use statement.
fn parse_use_path(input: &mut ParseInput) -> ModalResult<UsePath> {
    // Parse the base path (segments before ::* or ::{})
    let base_path = parse_simple_path(input)?;

    // Check for glob or nested
    if input.input.starts_with("::") {
        let _ = parse_symbol("::")(input)?;

        // Check for glob
        if input.input.starts_with('*') {
            let _ = parse_symbol("*")(input)?;
            return Ok(UsePath::Glob(base_path));
        }

        // Check for nested import
        if input.input.starts_with('{') {
            let items = parse_use_items(input)?;
            return Ok(UsePath::Nested(base_path, items));
        }

        // Parentheses distinguish an exact notation selector from `::*`.
        if input.input.starts_with('(') {
            let selector = parse_notation_import_selector(input)?;
            return Ok(UsePath::Notation {
                module: base_path,
                selector,
            });
        }
    }

    Ok(UsePath::Simple(base_path))
}

/// Parse a simple path (e.g., `crate::foo::bar`).
fn parse_simple_path(input: &mut ParseInput) -> ModalResult<SimplePath> {
    let segments = parse_path_segments(input)?;
    Ok(SimplePath { segments })
}

/// Parse path segments separated by `::`.
fn parse_path_segments(input: &mut ParseInput) -> ModalResult<Vec<Box<str>>> {
    let mut segments = Vec::new();

    // First segment
    let first = parse_path_segment(input)?;
    segments.push(first.into());

    // Additional segments
    loop {
        if input.input.starts_with("::") {
            // Look ahead to check if followed by *, {, or (
            let rest = &input.input[2..];
            if rest.starts_with('*') || rest.starts_with('{') || rest.starts_with('(') {
                break;
            }

            let _ = parse_symbol("::")(input)?;
            let segment = parse_path_segment(input)?;
            segments.push(segment.into());
        } else {
            break;
        }
    }

    Ok(segments)
}

/// Parse a non-empty exact notation selector inside parentheses.
fn parse_notation_import_selector(input: &mut ParseInput) -> ModalResult<NotationImportSelector> {
    let _ = parse_symbol("(")(input)?;
    skip_whitespace_and_comments(input);

    let mut parts = Vec::new();
    while !input.input.starts_with(')') {
        if input.input.is_empty() || input.input.starts_with(',') {
            return Err(winnow::error::ErrMode::Cut(
                winnow::error::ContextError::new(),
            ));
        }
        parts.push(parse_notation_pattern_part(input)?);
        skip_whitespace_and_comments(input);
    }

    if parts.is_empty() {
        return Err(winnow::error::ErrMode::Cut(
            winnow::error::ContextError::new(),
        ));
    }
    let _ = parse_symbol(")")(input)?;

    let first_span = notation_part_span(&parts[0]);
    let last_span = notation_part_span(&parts[parts.len() - 1]);
    Ok(NotationImportSelector {
        parts: parts.into_boxed_slice(),
        span: offset_to_span(input.state.source, first_span.start, last_span.end),
    })
}

fn notation_part_span(part: &NotationPatternPart) -> Span {
    match part {
        NotationPatternPart::Hole { span } | NotationPatternPart::Token { span, .. } => *span,
    }
}

/// Parse a single path segment (identifier).
fn parse_path_segment<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_').parse_next(input)
}

/// Parse use items within braces: `{a, b as c, d}` or `{a, b,}` (trailing comma).
fn parse_use_items(input: &mut ParseInput) -> ModalResult<Vec<UseItem>> {
    let _ = parse_symbol("{")(input)?;

    let mut items = Vec::new();
    loop {
        // Skip whitespace
        let _ = parse_whitespace(input)?;

        // Check for closing brace
        if input.input.starts_with('}') {
            let _ = parse_symbol("}")(input)?;
            break;
        }

        // Parse item
        let item = parse_use_item(input)?;
        items.push(item);

        // Skip whitespace
        let _ = parse_whitespace(input)?;

        // Check for comma
        if input.input.starts_with(',') {
            let _ = parse_symbol(",")(input)?;
            // Continue to next item (or trailing comma before })
        } else if input.input.starts_with('}') {
            let _ = parse_symbol("}")(input)?;
            break;
        } else {
            return Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ));
        }
    }

    Ok(items)
}

/// Parse a single use item (e.g., `foo` or `bar as baz`).
fn parse_use_item(input: &mut ParseInput) -> ModalResult<UseItem> {
    // Skip whitespace
    let _ = parse_whitespace(input)?;

    let start_offset = input.state.source.len() - input.input.len();

    let name = parse_path_segment(input)?;

    // Parse optional alias
    let alias = if input.input.starts_with(" as ") || input.input.starts_with("as ") {
        let _ = parse_keyword("as")(input)?;
        let _ = parse_whitespace(input)?;
        let alias_name = parse_path_segment(input)?;
        Some(alias_name.into())
    } else {
        None
    };

    let end_offset = input.state.source.len() - input.input.len();
    let span = offset_to_span(input.state.source, start_offset, end_offset);

    Ok(UseItem {
        name: name.into(),
        alias,
        span,
    })
}

/// Parse optional alias for the entire import: `as name`.
fn parse_optional_alias(input: &mut ParseInput) -> ModalResult<Option<Box<str>>> {
    // Skip whitespace
    let _ = parse_whitespace(input)?;

    if input.input.starts_with(" as ") || input.input.starts_with("as ") {
        let _ = parse_keyword("as")(input)?;
        let _ = parse_whitespace(input)?;
        let name = parse_path_segment(input)?;
        Ok(Some(name.into()))
    } else {
        Ok(None)
    }
}

/// Parse a keyword with word boundary check.
fn parse_keyword<'a>(word: &'a str) -> impl FnMut(&mut ParseInput<'a>) -> ModalResult<&'a str> {
    move |input: &mut ParseInput<'a>| {
        // Skip whitespace
        let _ = parse_whitespace(input)?;

        if input.input.starts_with(word) {
            let after = &input.input[word.len()..];
            if after.is_empty()
                || after
                    .chars()
                    .next()
                    .map(|c| !c.is_ascii_alphanumeric() && c != '_')
                    .unwrap_or(true)
            {
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

/// Parse a symbol (literal string).
fn parse_symbol<'a>(s: &'a str) -> impl FnMut(&mut ParseInput<'a>) -> ModalResult<&'a str> {
    move |input: &mut ParseInput<'a>| {
        // Skip whitespace
        let _ = parse_whitespace(input)?;

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

/// Parse optional whitespace.
fn parse_whitespace<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::new_input;
    use crate::surface::Visibility;
    use crate::token::Span;

    // =========================================================================
    // RED Phase: Tests should FAIL initially
    // =========================================================================

    #[test]
    fn test_parse_simple_use() {
        let mut input = new_input("use crate::foo::bar;");
        let result = parse_use(&mut input).unwrap();

        assert!(matches!(result.visibility, Visibility::Inherited));
        assert!(result.alias.is_none());

        match result.path {
            UsePath::Simple(path) => {
                assert_eq!(path.segments.len(), 3);
                assert_eq!(path.segments[0].as_ref(), "crate");
                assert_eq!(path.segments[1].as_ref(), "foo");
                assert_eq!(path.segments[2].as_ref(), "bar");
            }
            _ => panic!("Expected Simple path, got {:?}", result.path),
        }
    }

    #[test]
    fn test_parse_use_with_alias() {
        let mut input = new_input("use crate::foo::bar as baz;");
        let result = parse_use(&mut input).unwrap();

        assert!(matches!(result.visibility, Visibility::Inherited));
        assert_eq!(result.alias, Some("baz".into()));

        match result.path {
            UsePath::Simple(path) => {
                assert_eq!(path.segments.len(), 3);
                assert_eq!(path.segments[2].as_ref(), "bar");
            }
            _ => panic!("Expected Simple path"),
        }
    }

    #[test]
    fn test_parse_glob_import() {
        let mut input = new_input("use crate::foo::*;");
        let result = parse_use(&mut input).unwrap();

        assert!(matches!(result.visibility, Visibility::Inherited));

        match result.path {
            UsePath::Glob(path) => {
                assert_eq!(path.segments.len(), 2);
                assert_eq!(path.segments[0].as_ref(), "crate");
                assert_eq!(path.segments[1].as_ref(), "foo");
            }
            _ => panic!("Expected Glob path, got {:?}", result.path),
        }
    }

    #[test]
    fn test_parse_nested_import() {
        let mut input = new_input("use crate::foo::{a, b};");
        let result = parse_use(&mut input).unwrap();

        match result.path {
            UsePath::Nested(path, items) => {
                assert_eq!(path.segments.len(), 2);
                assert_eq!(path.segments[0].as_ref(), "crate");
                assert_eq!(path.segments[1].as_ref(), "foo");
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].name.as_ref(), "a");
                assert_eq!(items[1].name.as_ref(), "b");
            }
            _ => panic!("Expected Nested path"),
        }
    }

    #[test]
    fn test_parse_nested_with_aliases() {
        let mut input = new_input("use crate::foo::{a as x, b};");
        let result = parse_use(&mut input).unwrap();

        match result.path {
            UsePath::Nested(path, items) => {
                assert_eq!(path.segments.len(), 2);
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].name.as_ref(), "a");
                assert_eq!(items[0].alias, Some("x".into()));
                assert_eq!(items[1].name.as_ref(), "b");
                assert!(items[1].alias.is_none());
            }
            _ => panic!("Expected Nested path"),
        }
    }

    #[test]
    fn parses_group_members_with_distinct_exact_source_spans() {
        let source = "use crate::api::{first, second as local_second};";
        let mut input = new_input(source);
        let result = parse_use(&mut input).expect("the grouped use fixture parses");

        match result.path {
            UsePath::Nested(base, items) => {
                assert_eq!(base.segments, vec!["crate".into(), "api".into()]);
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].name.as_ref(), "first");
                assert_eq!(items[0].alias, None);
                assert_eq!(items[0].span, Span::new(17, 22, 1, 18));
                assert_eq!(items[1].name.as_ref(), "second");
                assert_eq!(items[1].alias.as_deref(), Some("local_second"));
                assert_eq!(items[1].span, Span::new(24, 46, 1, 25));
            }
            other => panic!("expected nested crate::api import, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_pub_use() {
        let mut input = new_input("pub use crate::foo::bar;");
        let result = parse_use(&mut input).unwrap();

        assert!(matches!(result.visibility, Visibility::Public));

        match result.path {
            UsePath::Simple(path) => {
                assert_eq!(path.segments.len(), 3);
                assert_eq!(path.segments[0].as_ref(), "crate");
            }
            _ => panic!("Expected Simple path"),
        }
    }

    // =========================================================================
    // Additional Edge Case Tests
    // =========================================================================

    #[test]
    fn test_parse_single_segment() {
        let mut input = new_input("use std;");
        let result = parse_use(&mut input).unwrap();

        match result.path {
            UsePath::Simple(path) => {
                assert_eq!(path.segments.len(), 1);
                assert_eq!(path.segments[0].as_ref(), "std");
            }
            _ => panic!("Expected Simple path"),
        }
    }

    #[test]
    fn test_parse_nested_single_item() {
        let mut input = new_input("use std::{io};");
        let result = parse_use(&mut input).unwrap();

        match result.path {
            UsePath::Nested(path, items) => {
                assert_eq!(path.segments.len(), 1);
                assert_eq!(path.segments[0].as_ref(), "std");
                assert_eq!(items.len(), 1);
                assert_eq!(items[0].name.as_ref(), "io");
            }
            _ => panic!("Expected Nested path"),
        }
    }

    #[test]
    fn test_parse_pub_crate_use() {
        let mut input = new_input("pub(crate) use crate::foo;");
        let result = parse_use(&mut input).unwrap();

        assert!(matches!(result.visibility, Visibility::Crate));
    }

    // =========================================================================
    // TASK-340: External import parsing tests
    // =========================================================================

    #[test]
    fn test_parse_external_simple_import() {
        let mut input = new_input("use external::util::item;");
        let result = parse_use(&mut input).unwrap();

        assert!(matches!(result.visibility, Visibility::Inherited));
        assert!(result.alias.is_none());

        match result.path {
            UsePath::Simple(path) => {
                assert_eq!(path.segments.len(), 3);
                assert_eq!(path.segments[0].as_ref(), "external");
                assert_eq!(path.segments[1].as_ref(), "util");
                assert_eq!(path.segments[2].as_ref(), "item");
            }
            _ => panic!("Expected Simple path, got {:?}", result.path),
        }
    }

    #[test]
    fn test_parse_external_nested_import_with_aliases() {
        let mut input = new_input("use external::util::{a, b as c};");
        let result = parse_use(&mut input).unwrap();

        match result.path {
            UsePath::Nested(path, items) => {
                assert_eq!(path.segments.len(), 2);
                assert_eq!(path.segments[0].as_ref(), "external");
                assert_eq!(path.segments[1].as_ref(), "util");
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].name.as_ref(), "a");
                assert!(items[0].alias.is_none());
                assert_eq!(items[1].name.as_ref(), "b");
                assert_eq!(items[1].alias, Some("c".into()));
            }
            _ => panic!("Expected Nested path, got {:?}", result.path),
        }
    }

    #[test]
    fn test_parse_external_glob_import() {
        let mut input = new_input("use external::util::*;");
        let result = parse_use(&mut input).unwrap();

        match result.path {
            UsePath::Glob(path) => {
                assert_eq!(path.segments.len(), 2);
                assert_eq!(path.segments[0].as_ref(), "external");
                assert_eq!(path.segments[1].as_ref(), "util");
            }
            _ => panic!("Expected Glob path, got {:?}", result.path),
        }
    }

    #[test]
    fn test_parse_external_import_with_alias() {
        let mut input = new_input("use external::util::helpers as h;");
        let result = parse_use(&mut input).unwrap();

        assert_eq!(result.alias, Some("h".into()));

        match result.path {
            UsePath::Simple(path) => {
                assert_eq!(path.segments.len(), 3);
                assert_eq!(path.segments[0].as_ref(), "external");
                assert_eq!(path.segments[1].as_ref(), "util");
                assert_eq!(path.segments[2].as_ref(), "helpers");
            }
            _ => panic!("Expected Simple path, got {:?}", result.path),
        }
    }
}
