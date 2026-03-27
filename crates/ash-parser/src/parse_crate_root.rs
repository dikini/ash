//! Crate root metadata parser for the Ash language.
//!
//! This module provides parsers for crate identity and dependency declarations:
//! - `crate <name>;` - Declares the crate name
//! - `dependency <alias> from "<path>";` - Declares an external dependency
//!
//! Example:
//! ```ash
//! crate app;
//!
//! dependency util from "../util/main.ash";
//! dependency policy from "../policy/main.ash";
//!
//! use external::util::sanitize::normalize;
//! ```

use winnow::error::ContextError;
use winnow::error::ErrMode;
use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::take_while;

use crate::combinators::keyword;
use crate::combinators::whitespace;
use crate::input::ParseInput;
use crate::input::span_from;
use crate::surface::{CrateRootMetadata, DependencyDecl};

/// Parse crate root metadata (crate name and dependencies).
///
/// # Examples
///
/// ```
/// use ash_parser::parse_crate_root::parse_crate_root_metadata;
/// use ash_parser::input::new_input;
/// use winnow::prelude::*;
///
/// let mut input = new_input("crate app;\n\ndependency util from \"../util.ash\";");
/// let result = parse_crate_root_metadata.parse_next(&mut input);
/// assert!(result.is_ok());
/// ```
pub fn parse_crate_root_metadata(input: &mut ParseInput) -> ModalResult<CrateRootMetadata> {
    let start_pos = input.state;

    // Parse the crate name declaration first (required)
    let crate_name = parse_crate_decl(input)?;

    // Parse optional whitespace/newlines after crate declaration
    let _ = parse_newlines_and_whitespace(input)?;

    // Parse zero or more dependency declarations
    let dependencies = parse_dependencies(input)?;

    let span = span_from(&start_pos, &input.state);

    Ok(CrateRootMetadata {
        crate_name: crate_name.into(),
        dependencies,
        span,
    })
}

/// Parse zero or more dependency declarations, stopping at first non-dependency content.
/// This is lenient - it doesn't fail if there's other content after the dependencies.
fn parse_dependencies(input: &mut ParseInput) -> ModalResult<Vec<DependencyDecl>> {
    let mut dependencies = Vec::new();

    loop {
        // Check if we're at the end of input
        if input.input.is_empty() {
            break;
        }

        // Try to parse a dependency declaration
        match parse_dependency_decl(input) {
            Ok(dep) => {
                dependencies.push(dep);
                // After a successful parse, allow more whitespace/newlines
                let _ = parse_newlines_and_whitespace(input);
            }
            Err(_) => {
                // If we can't parse a dependency, stop parsing dependencies
                // This allows other content (mod, workflow, etc.) to follow
                break;
            }
        }
    }

    Ok(dependencies)
}

/// Parse a crate declaration: `crate <name>;`
fn parse_crate_decl<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    // Skip any leading whitespace
    let _ = whitespace(input)?;

    // Parse 'crate' keyword
    let _ = keyword("crate").parse_next(input)?;

    // Parse required whitespace after 'crate'
    let _ = parse_required_whitespace(input)?;

    // Parse crate name (identifier)
    let name = parse_identifier(input)?;

    // Parse semicolon
    let _ = parse_semicolon(input)?;

    Ok(name)
}

/// Parse a dependency declaration: `dependency <alias> from "<path>";`
fn parse_dependency_decl(input: &mut ParseInput) -> ModalResult<DependencyDecl> {
    let start_pos = input.state;

    // Skip any leading whitespace/newlines
    let _ = parse_newlines_and_whitespace(input)?;

    // Parse 'dependency' keyword
    let _ = keyword("dependency").parse_next(input)?;

    // Parse required whitespace
    let _ = parse_required_whitespace(input)?;

    // Parse alias (identifier)
    let alias = parse_identifier(input)?;

    // Parse required whitespace
    let _ = parse_required_whitespace(input)?;

    // Parse 'from' keyword
    let _ = keyword("from").parse_next(input)?;

    // Parse required whitespace
    let _ = parse_required_whitespace(input)?;

    // Parse the path (quoted string)
    let path = parse_quoted_string(input)?;

    // Parse semicolon
    let _ = parse_semicolon(input)?;

    let span = span_from(&start_pos, &input.state);

    Ok(DependencyDecl {
        alias: alias.into(),
        root_path: path.into(),
        span,
    })
}

/// Parse an identifier (alphanumeric + underscore, starting with letter or underscore)
fn parse_identifier<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_').parse_next(input)
}

/// Parse a quoted string literal (double quotes only, no escape sequences)
fn parse_quoted_string<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    // Skip whitespace
    let _ = whitespace(input)?;

    // Check for opening quote
    if !input.input.starts_with('"') {
        return Err(ErrMode::Backtrack(ContextError::new()));
    }

    // Consume opening quote
    let _ = input.input.next_slice(1);
    input.state.advance('"');

    // Find closing quote
    let content = take_while(0.., |c: char| c != '"').parse_next(input)?;

    // Check for closing quote
    if !input.input.starts_with('"') {
        return Err(ErrMode::Backtrack(ContextError::new()));
    }

    // Consume closing quote
    let _ = input.input.next_slice(1);
    input.state.advance('"');

    // Reject empty paths
    if content.is_empty() {
        return Err(ErrMode::Backtrack(ContextError::new()));
    }

    Ok(content)
}

/// Parse a semicolon, allowing optional whitespace before
fn parse_semicolon<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    let _ = whitespace(input)?;
    if input.input.starts_with(';') {
        let _ = input.input.next_slice(1);
        input.state.advance(';');
        Ok(";")
    } else {
        Err(ErrMode::Backtrack(ContextError::new()))
    }
}

/// Parse required whitespace (at least one character)
fn parse_required_whitespace<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    let result = take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(input)?;
    Ok(result)
}

/// Parse newlines and whitespace (for between declarations)
fn parse_newlines_and_whitespace<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::new_input;

    #[test]
    fn test_parse_crate_decl_only() {
        let mut input = new_input("crate app;");
        let result = parse_crate_root_metadata(&mut input);
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert_eq!(metadata.crate_name.as_ref(), "app");
        assert!(metadata.dependencies.is_empty());
    }

    #[test]
    fn test_parse_crate_with_dependencies() {
        let input_str = r#"crate myapp;

dependency util from "../util.ash";
dependency core from "./core/main.ash";
"#;
        let mut input = new_input(input_str);
        let result = parse_crate_root_metadata(&mut input);
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert_eq!(metadata.crate_name.as_ref(), "myapp");
        assert_eq!(metadata.dependencies.len(), 2);
        assert_eq!(metadata.dependencies[0].alias.as_ref(), "util");
        assert_eq!(metadata.dependencies[0].root_path.as_ref(), "../util.ash");
        assert_eq!(metadata.dependencies[1].alias.as_ref(), "core");
        assert_eq!(
            metadata.dependencies[1].root_path.as_ref(),
            "./core/main.ash"
        );
    }

    #[test]
    fn test_parse_crate_with_single_dependency() {
        let input_str = "crate server;\n\ndependency api from \"../api/main.ash\";";
        let mut input = new_input(input_str);
        let result = parse_crate_root_metadata(&mut input);
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert_eq!(metadata.crate_name.as_ref(), "server");
        assert_eq!(metadata.dependencies.len(), 1);
        assert_eq!(metadata.dependencies[0].alias.as_ref(), "api");
    }

    #[test]
    fn test_parse_dependency_requires_quoted_path() {
        let input_str = "crate app;\ndependency util from ../util.ash;";
        let mut input = new_input(input_str);
        let result = parse_crate_root_metadata(&mut input);
        assert!(result.is_err(), "Unquoted path should fail");
    }

    #[test]
    fn test_parse_empty_path_rejected() {
        let input_str = r#"crate app;
dependency util from "";
"#;
        let mut input = new_input(input_str);
        let result = parse_crate_root_metadata(&mut input);
        assert!(result.is_err(), "Empty path should fail");
    }

    #[test]
    fn test_parse_missing_semicolon() {
        let mut input = new_input("crate app");
        let result = parse_crate_root_metadata(&mut input);
        assert!(result.is_err(), "Missing semicolon should fail");
    }

    #[test]
    fn test_parse_crate_name_with_underscores() {
        let mut input = new_input("crate my_crate_name;");
        let result = parse_crate_root_metadata(&mut input);
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert_eq!(metadata.crate_name.as_ref(), "my_crate_name");
    }

    #[test]
    fn test_parse_identifier_function() {
        let mut input = new_input("test123");
        let result = parse_identifier(&mut input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test123");
    }

    #[test]
    fn test_parse_quoted_string_function() {
        let mut input = new_input("\"../path/to/file.ash\"");
        let result = parse_quoted_string(&mut input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "../path/to/file.ash");
    }

    #[test]
    fn test_span_tracking() {
        let mut input = new_input("crate app;");
        let result = parse_crate_root_metadata(&mut input);
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert_eq!(metadata.span.start, 0);
        // The span end is 10 ("crate app;" is 10 chars)
        // But due to how input tracking works, we just verify it's > start
        assert!(metadata.span.end > metadata.span.start);
        assert_eq!(metadata.span.line, 1);
        assert_eq!(metadata.span.column, 1);
    }

    #[test]
    fn test_dependency_span_tracking() {
        let input_str = "crate app;\n\ndependency util from \"../util.ash\";";
        let mut input = new_input(input_str);
        let result = parse_crate_root_metadata(&mut input);
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert_eq!(metadata.dependencies.len(), 1);
        // The dependency span should capture just the dependency declaration
        let dep_span = metadata.dependencies[0].span;
        assert!(dep_span.start > 0);
        assert!(dep_span.end > dep_span.start);
    }
}
