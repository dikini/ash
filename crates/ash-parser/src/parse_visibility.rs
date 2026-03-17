//! Visibility modifier parser for the Ash language.
//!
//! This module provides parsers for Rust-style visibility modifiers:
//! - `pub` → Public
//! - `pub(crate)` → Crate
//! - `pub(super)` → Super
//! - `pub(self)` → Self_
//! - `pub(in path::to::module)` → Restricted
//! - (none) → Inherited

use winnow::combinator::{alt, delimited, opt, preceded};
use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::take_while;

use crate::combinators::keyword;
use crate::input::{ParseInput, update_position};
use crate::surface::Visibility;

/// Parse a visibility modifier.
///
/// Returns `Visibility::Inherited` if no visibility modifier is present.
///
/// # Examples
///
/// ```
/// use ash_parser::parse_visibility::parse_visibility;
/// use ash_parser::input::new_input;
/// use ash_parser::surface::Visibility;
/// use winnow::prelude::*;
///
/// // Parse "pub"
/// let mut input = new_input("pub");
/// let result = parse_visibility.parse_next(&mut input).unwrap();
/// assert!(matches!(result, Visibility::Public));
/// ```
pub fn parse_visibility(input: &mut ParseInput) -> ModalResult<Visibility> {
    // Try to match "pub" keyword
    if opt(keyword("pub")).parse_next(input)?.is_none() {
        return Ok(Visibility::Inherited);
    }

    // Check for restricted visibility: pub(...)
    let restricted = opt(delimited(
        literal_str("("),
        parse_restricted_body,
        literal_str(")"),
    ))
    .parse_next(input)?;

    match restricted {
        Some(vis) => Ok(vis),
        None => Ok(Visibility::Public),
    }
}

/// Parse the body inside `pub(...)`: crate, super, self, or in path
fn parse_restricted_body(input: &mut ParseInput) -> ModalResult<Visibility> {
    alt((
        keyword("crate").map(|_| Visibility::Crate),
        keyword("super").map(|_| Visibility::Super),
        keyword("self").map(|_| Visibility::Self_),
        preceded(keyword("in"), preceded(opt(parse_whitespace), parse_path))
            .map(|path| Visibility::Restricted { path }),
    ))
    .parse_next(input)
}

/// Parse a module path for restricted visibility (e.g., `crate::foo::bar`).
fn parse_path(input: &mut ParseInput) -> ModalResult<Box<str>> {
    // Parse path segments separated by `::`
    let mut path = String::new();

    // First segment
    let first = parse_path_segment(input)?;
    path.push_str(first);

    // Additional segments
    loop {
        if opt(literal_str("::")).parse_next(input)?.is_some() {
            let segment = parse_path_segment(input)?;
            path.push_str("::");
            path.push_str(segment);
        } else {
            break;
        }
    }

    Ok(path.into_boxed_str())
}

/// Parse a single path segment (identifier)
fn parse_path_segment<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_').parse_next(input)
}

/// Parse optional whitespace.
fn parse_whitespace<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(input)
}

/// Parse a string literal token.
fn literal_str<'a>(s: &'a str) -> impl FnMut(&mut ParseInput<'a>) -> ModalResult<&'a str> {
    move |input: &mut ParseInput<'a>| {
        if input.input.starts_with(s) {
            // Update position state
            update_position(&mut input.state, s);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::new_input;

    #[test]
    fn test_parse_pub() {
        let mut input = new_input("pub");
        let result = parse_visibility(&mut input).unwrap();
        assert_eq!(result, Visibility::Public);
    }

    #[test]
    fn test_parse_pub_crate() {
        let mut input = new_input("pub(crate)");
        let result = parse_visibility(&mut input).unwrap();
        assert_eq!(result, Visibility::Crate);
    }

    #[test]
    fn test_parse_pub_super() {
        let mut input = new_input("pub(super)");
        let result = parse_visibility(&mut input).unwrap();
        assert_eq!(result, Visibility::Super);
    }

    #[test]
    fn test_parse_pub_self() {
        let mut input = new_input("pub(self)");
        let result = parse_visibility(&mut input).unwrap();
        assert_eq!(result, Visibility::Self_);
    }

    #[test]
    fn test_parse_pub_in_path() {
        let mut input = new_input("pub(in crate::foo)");
        let result = parse_visibility(&mut input).unwrap();
        assert!(matches!(result, Visibility::Restricted { path } if path.as_ref() == "crate::foo"));
    }

    #[test]
    fn test_parse_inherited_empty() {
        let mut input = new_input("");
        let result = parse_visibility(&mut input).unwrap();
        assert_eq!(result, Visibility::Inherited);
    }

    #[test]
    fn test_parse_inherited_other_keyword() {
        // When there's no visibility modifier, it should return Inherited
        // without consuming input that doesn't start with "pub"
        let mut input = new_input("fn something()");
        let result = parse_visibility(&mut input).unwrap();
        assert_eq!(result, Visibility::Inherited);
    }

    #[test]
    fn test_parse_pub_in_path_complex() {
        let mut input = new_input("pub(in some::module::path)");
        let result = parse_visibility(&mut input).unwrap();
        assert!(
            matches!(result, Visibility::Restricted { path } if path.as_ref() == "some::module::path")
        );
    }
}
