//! Pattern parser for the Ash language.
//!
//! This module provides parsers for Ash patterns used in let bindings,
//! for loops, and match expressions.

use std::collections::HashSet;
use std::sync::OnceLock;

use winnow::combinator::alt;
use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::take_while;

use crate::input::ParseInput;
use crate::surface::{Literal, Name, Pattern};
use crate::token::Span;

/// Static set of Ash keywords for O(1) lookup.
static KEYWORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();

/// Get the set of Ash keywords.
fn get_keywords() -> &'static HashSet<&'static str> {
    KEYWORDS.get_or_init(|| {
        let mut set = HashSet::new();
        set.insert("workflow");
        set.insert("capability");
        set.insert("policy");
        set.insert("role");
        set.insert("observe");
        set.insert("orient");
        set.insert("propose");
        set.insert("decide");
        set.insert("act");
        set.insert("oblige");
        set.insert("check");
        set.insert("let");
        set.insert("if");
        set.insert("then");
        set.insert("else");
        set.insert("for");
        set.insert("do");
        set.insert("par");
        set.insert("with");
        set.insert("maybe");
        set.insert("must");
        set.insert("match");
        set.insert("attempt");
        set.insert("retry");
        set.insert("timeout");
        set.insert("done");
        set.insert("epistemic");
        set.insert("deliberative");
        set.insert("evaluative");
        set.insert("operational");
        set.insert("authority");
        set.insert("obligations");
        set.insert("when");
        set.insert("returns");
        set.insert("where");
        set.insert("permit");
        set.insert("deny");
        set.insert("require_approval");
        set.insert("escalate");
        set.insert("in");
        set.insert("not");
        set.insert("and");
        set.insert("or");
        set.insert("true");
        set.insert("false");
        set.insert("null");
        set
    })
}

/// Parse a pattern (entry point).
///
/// Patterns include:
/// - Variable patterns: `x`
/// - Wildcard: `_`
/// - Tuple patterns: `(a, b, c)`
/// - List patterns: `[a, b, ..rest]`
/// - Record patterns: `{ field: pat, ... }`
/// - Variant patterns: `Some { value: x }` or `None`
/// - Literal patterns: `42`, `"hello"`, `true`
pub fn pattern(input: &mut ParseInput) -> ModalResult<Pattern> {
    skip_whitespace_and_comments(input);
    alt((
        parse_variant_pattern,
        parse_record_pattern,
        parse_wildcard_pattern,
        parse_tuple_pattern,
        parse_list_pattern,
        parse_literal_pattern,
        parse_variable_pattern,
    ))
    .parse_next(input)
}

/// Parse a variant pattern: `Name` or `Name { field: pat, ... }`
///
/// Examples:
/// - `None` (unit variant)
/// - `Some { value: x }` (variant with fields)
/// - `Ok { value: (x, y) }` (nested patterns)
fn parse_variant_pattern(input: &mut ParseInput) -> ModalResult<Pattern> {
    let start_pos = input.state;
    let checkpoint = *input;

    // Try to parse an identifier (variant name)
    let name = match identifier(input) {
        Ok(n) => n,
        Err(_) => {
            *input = checkpoint;
            return Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ));
        }
    };

    // Check if followed by `{` (fields) or not (unit variant)
    skip_whitespace_and_comments(input);

    if input.input.starts_with('{') {
        // Parse the fields block
        let fields = match parse_variant_fields(input) {
            Ok(f) => Some(f),
            Err(_) => {
                *input = checkpoint;
                return Err(winnow::error::ErrMode::Backtrack(
                    winnow::error::ContextError::new(),
                ));
            }
        };

        let _span = span_from(&start_pos, &input.state);
        return Ok(Pattern::Variant {
            name: name.into(),
            fields,
        });
    }

    // No `{` after the identifier: parse as a unit variant only when the
    // identifier is UpperCamelCase (uppercase-leading). Otherwise, it is a
    // variable pattern (e.g. `x`).
    let is_uppercase_leading = name.chars().next().is_some_and(|c| c.is_ascii_uppercase());

    if is_uppercase_leading {
        let _span = span_from(&start_pos, &input.state);
        Ok(Pattern::Variant {
            name: name.into(),
            fields: None,
        })
    } else {
        // Backtrack and let variable pattern handle it.
        *input = checkpoint;
        Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ))
    }
}

/// Parse variant fields: `{ field: pat, ... }`
fn parse_variant_fields(input: &mut ParseInput) -> ModalResult<Vec<(Name, Pattern)>> {
    let _ = literal_str("{").parse_next(input)?;

    let mut fields = Vec::new();

    loop {
        skip_whitespace_and_comments(input);

        // Check for end of fields
        if input.input.is_empty() || input.input.starts_with("}") {
            break;
        }

        // Parse field name
        let field_name = identifier(input)?;

        skip_whitespace_and_comments(input);
        let _ = literal_str(":").parse_next(input)?;
        skip_whitespace_and_comments(input);

        // Parse nested pattern
        let field_pattern = pattern(input)?;
        fields.push((field_name.into(), field_pattern));

        skip_whitespace_and_comments(input);

        // Optional comma
        if input.input.starts_with(",") {
            let _ = input.input.next_slice(1);
            input.state.advance(',');
        }
    }

    let _ = literal_str("}").parse_next(input)?;

    Ok(fields)
}

/// Parse a wildcard pattern: `_`
fn parse_wildcard_pattern(input: &mut ParseInput) -> ModalResult<Pattern> {
    let _ = literal_str("_").parse_next(input)?;
    Ok(Pattern::Wildcard)
}

/// Parse a variable pattern: just an identifier
fn parse_variable_pattern(input: &mut ParseInput) -> ModalResult<Pattern> {
    let name = identifier(input)?;
    Ok(Pattern::Variable(name.into()))
}

/// Parse a tuple pattern: `(pat1, pat2, ...)`
fn parse_tuple_pattern(input: &mut ParseInput) -> ModalResult<Pattern> {
    let _ = literal_str("(").parse_next(input)?;
    let patterns = parse_pattern_list(input)?;
    let _ = literal_str(")").parse_next(input)?;
    Ok(Pattern::Tuple(patterns))
}

/// Parse a record pattern: `{ field: pat, ... }`
fn parse_record_pattern(input: &mut ParseInput) -> ModalResult<Pattern> {
    // A record pattern looks like `{ field: pat }`
    // We distinguish from variant pattern by checking if the first field
    // looks like a field binding rather than a variant constructor

    let start_pos = input.state;
    let checkpoint = *input;

    // Must start with `{`
    if literal_str("{").parse_next(input).is_err() {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    let mut fields = Vec::new();

    loop {
        skip_whitespace_and_comments(input);

        if input.input.is_empty() || input.input.starts_with("}") {
            break;
        }

        // Parse field name
        let field_name = match identifier(input) {
            Ok(n) => n,
            Err(_) => {
                *input = checkpoint;
                return Err(winnow::error::ErrMode::Backtrack(
                    winnow::error::ContextError::new(),
                ));
            }
        };

        skip_whitespace_and_comments(input);

        // Must have `:` for record pattern
        if literal_str(":").parse_next(input).is_err() {
            *input = checkpoint;
            return Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ));
        }

        skip_whitespace_and_comments(input);

        // Parse nested pattern
        let field_pattern = match pattern(input) {
            Ok(p) => p,
            Err(_) => {
                *input = checkpoint;
                return Err(winnow::error::ErrMode::Backtrack(
                    winnow::error::ContextError::new(),
                ));
            }
        };

        fields.push((field_name.into(), field_pattern));

        skip_whitespace_and_comments(input);

        // Optional comma
        if input.input.starts_with(",") {
            let _ = input.input.next_slice(1);
            input.state.advance(',');
        }
    }

    if literal_str("}").parse_next(input).is_err() {
        *input = checkpoint;
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    // Success - this is a record pattern
    let _span = span_from(&start_pos, &input.state);
    Ok(Pattern::Record(fields))
}

/// Parse a list pattern: `[pat1, pat2, ..rest]`
fn parse_list_pattern(input: &mut ParseInput) -> ModalResult<Pattern> {
    let _ = literal_str("[").parse_next(input)?;
    let result = parse_list_pattern_inner(input)?;
    let _ = literal_str("]").parse_next(input)?;
    Ok(result)
}

/// Parse the inner content of a list pattern
fn parse_list_pattern_inner(input: &mut ParseInput) -> ModalResult<Pattern> {
    let mut elements = Vec::new();
    let mut rest = None;

    loop {
        skip_whitespace_and_comments(input);

        if input.input.is_empty() || input.input.starts_with("]") {
            break;
        }

        // Check for rest pattern: ..name
        if input.input.starts_with("..") {
            let _ = input.input.next_slice(2);
            input.state.advance('.');
            input.state.advance('.');
            rest = Some(identifier(input)?.into());
            break;
        }

        let pat = pattern(input)?;
        elements.push(pat);

        skip_whitespace_and_comments(input);

        if input.input.starts_with(",") {
            let _ = input.input.next_slice(1);
            input.state.advance(',');
        } else {
            break;
        }
    }

    Ok(Pattern::List { elements, rest })
}

/// Parse a literal pattern
fn parse_literal_pattern(input: &mut ParseInput) -> ModalResult<Pattern> {
    let lit = parse_literal(input)?;
    Ok(Pattern::Literal(lit))
}

/// Parse a comma-separated list of patterns
fn parse_pattern_list(input: &mut ParseInput) -> ModalResult<Vec<Pattern>> {
    let mut patterns = Vec::new();

    loop {
        skip_whitespace_and_comments(input);

        if input.input.is_empty() || input.input.starts_with(")") {
            break;
        }

        let pat = pattern(input)?;
        patterns.push(pat);

        skip_whitespace_and_comments(input);

        if input.input.starts_with(",") {
            let _ = input.input.next_slice(1);
            input.state.advance(',');
        } else {
            break;
        }
    }

    Ok(patterns)
}

/// Parse a literal value.
fn parse_literal(input: &mut ParseInput) -> ModalResult<Literal> {
    alt((
        parse_string_literal,
        parse_float_literal,
        parse_int_literal,
        parse_bool_literal,
        parse_null_literal,
    ))
    .parse_next(input)
}

/// Parse a string literal.
fn parse_string_literal(input: &mut ParseInput) -> ModalResult<Literal> {
    let _ = literal_str("\"").parse_next(input)?;

    let content = take_while(0.., |c: char| c != '"').parse_next(input)?;

    let _ = literal_str("\"").parse_next(input)?;
    Ok(Literal::String(content.into()))
}

/// Parse an integer literal.
fn parse_int_literal(input: &mut ParseInput) -> ModalResult<Literal> {
    let digits: &str = take_while(1.., |c: char| c.is_ascii_digit()).parse_next(input)?;

    match digits.parse::<i64>() {
        Ok(n) => Ok(Literal::Int(n)),
        Err(_) => Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        )),
    }
}

/// Parse a floating-point literal.
fn parse_float_literal(input: &mut ParseInput) -> ModalResult<Literal> {
    let int_part: &str = take_while(1.., |c: char| c.is_ascii_digit()).parse_next(input)?;

    // Check for decimal point
    if !input.input.starts_with('.') {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    let _ = input.input.next_slice(1);
    input.state.advance('.');

    let frac_part: &str = take_while(1.., |c: char| c.is_ascii_digit()).parse_next(input)?;

    let full = format!("{}.{}", int_part, frac_part);
    match full.parse::<f64>() {
        Ok(f) => Ok(Literal::Float(f)),
        Err(_) => Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        )),
    }
}

/// Parse a boolean literal.
fn parse_bool_literal(input: &mut ParseInput) -> ModalResult<Literal> {
    alt((
        keyword("true").map(|_| Literal::Bool(true)),
        keyword("false").map(|_| Literal::Bool(false)),
    ))
    .parse_next(input)
}

/// Parse a null literal.
fn parse_null_literal(input: &mut ParseInput) -> ModalResult<Literal> {
    keyword("null").map(|_| Literal::Null).parse_next(input)
}

/// Parse an identifier.
fn identifier<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    // Use take_while to match the entire identifier at once
    // First char: letter or underscore, rest: alphanumeric, underscore, or hyphen
    let result: &str = take_while(1.., |c: char| {
        c.is_ascii_alphanumeric() || c == '_' || c == '-'
    })
    .parse_next(input)?;

    // Check that first character is a letter or underscore (not a digit)
    if result.is_empty()
        || !result
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
            && !result.starts_with('_')
    {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    // Check that it's not a keyword
    if is_keyword(result) {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    Ok(result)
}

/// Check if a string is a keyword.
fn is_keyword(s: &str) -> bool {
    get_keywords().contains(s)
}

/// Parse a keyword (ensures word boundary).
fn keyword<'a>(word: &'a str) -> impl Parser<ParseInput<'a>, &'a str, winnow::error::ContextError> {
    move |input: &mut ParseInput<'a>| {
        skip_whitespace_and_comments(input);

        if input.input.starts_with(word) {
            let after = &input.input[word.len()..];
            if after.is_empty() || !after.chars().next().unwrap().is_ascii_alphanumeric() {
                for c in word.chars() {
                    input.state.advance(c);
                }
                let _ = input.input.next_slice(word.len());
                return Ok(word);
            }
        }
        Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ))
    }
}

/// Parse a string literal token.
fn literal_str<'a>(s: &'a str) -> impl FnMut(&mut ParseInput<'a>) -> ModalResult<&'a str> {
    move |input: &mut ParseInput<'a>| {
        skip_whitespace_and_comments(input);
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

/// Skip whitespace and comments.
fn skip_whitespace_and_comments(input: &mut ParseInput) {
    loop {
        // Skip whitespace
        let _: ModalResult<&str> =
            take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(input);

        // Check for line comment
        if input.input.starts_with("--") {
            let _: ModalResult<&str> = take_while(0.., |c: char| c != '\n').parse_next(input);
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

/// Create a span from start position to current position.
fn span_from(start: &crate::input::Position, end: &crate::input::Position) -> Span {
    Span {
        start: start.offset,
        end: end.offset,
        line: start.line,
        column: start.column,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::new_input;

    fn test_input(s: &str) -> ParseInput<'_> {
        new_input(s)
    }

    #[test]
    fn test_parse_variable_pattern() {
        let mut input = test_input("x");
        let result = pattern(&mut input).unwrap();
        assert!(matches!(result, Pattern::Variable(name) if name.as_ref() == "x"));
    }

    #[test]
    fn test_parse_wildcard_pattern() {
        let mut input = test_input("_");
        let result = pattern(&mut input).unwrap();
        assert!(matches!(result, Pattern::Wildcard));
    }

    #[test]
    fn test_parse_tuple_pattern() {
        let mut input = test_input("(x, y, z)");
        let result = pattern(&mut input).unwrap();
        match result {
            Pattern::Tuple(patterns) => {
                assert_eq!(patterns.len(), 3);
            }
            _ => panic!("Expected Tuple pattern"),
        }
    }

    #[test]
    fn test_parse_list_pattern() {
        let mut input = test_input("[a, b, c]");
        let result = pattern(&mut input).unwrap();
        match result {
            Pattern::List { elements, rest } => {
                assert_eq!(elements.len(), 3);
                assert!(rest.is_none());
            }
            _ => panic!("Expected List pattern"),
        }
    }

    #[test]
    fn test_parse_list_pattern_with_rest() {
        let mut input = test_input("[head, ..tail]");
        let result = pattern(&mut input).unwrap();
        match result {
            Pattern::List { elements, rest } => {
                assert_eq!(elements.len(), 1);
                assert!(rest.is_some());
                assert_eq!(rest.unwrap().as_ref(), "tail");
            }
            _ => panic!("Expected List pattern"),
        }
    }

    #[test]
    fn test_parse_record_pattern() {
        let mut input = test_input("{ x: a, y: b }");
        let result = pattern(&mut input).unwrap();
        match result {
            Pattern::Record(fields) => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0.as_ref(), "x");
                assert_eq!(fields[1].0.as_ref(), "y");
            }
            _ => panic!("Expected Record pattern"),
        }
    }

    #[test]
    fn test_parse_variable_pattern_named_supervises() {
        let mut input = test_input("supervises");
        let result = pattern(&mut input).unwrap();
        assert!(matches!(result, Pattern::Variable(name) if name.as_ref() == "supervises"));
    }

    #[test]
    fn test_parse_variant_pattern_unit() {
        let mut input = test_input("None");
        let result = pattern(&mut input).unwrap();
        match result {
            Pattern::Variant { name, fields } => {
                assert_eq!(name.as_ref(), "None");
                assert!(fields.is_none());
            }
            _ => panic!("Expected Variant pattern for unit variant"),
        }
    }

    #[test]
    fn test_parse_variant_pattern_with_fields() {
        let mut input = test_input("Some { value: x }");
        let result = pattern(&mut input).unwrap();
        match result {
            Pattern::Variant { name, fields } => {
                assert_eq!(name.as_ref(), "Some");
                assert!(fields.is_some());
                let fields = fields.unwrap();
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].0.as_ref(), "value");
                assert!(matches!(&fields[0].1, Pattern::Variable(v) if v.as_ref() == "x"));
            }
            _ => panic!("Expected Variant pattern, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_variant_pattern_multiple_fields() {
        let mut input = test_input("Ok { value: x, error: e }");
        let result = pattern(&mut input).unwrap();
        match result {
            Pattern::Variant { name, fields } => {
                assert_eq!(name.as_ref(), "Ok");
                assert!(fields.is_some());
                let fields = fields.unwrap();
                assert_eq!(fields.len(), 2);
            }
            _ => panic!("Expected Variant pattern"),
        }
    }

    #[test]
    fn test_parse_literal_pattern_int() {
        let mut input = test_input("42");
        let result = pattern(&mut input).unwrap();
        match result {
            Pattern::Literal(Literal::Int(42)) => {}
            _ => panic!("Expected Int literal pattern"),
        }
    }

    #[test]
    fn test_parse_literal_pattern_string() {
        let mut input = test_input("\"hello\"");
        let result = pattern(&mut input).unwrap();
        match result {
            Pattern::Literal(Literal::String(s)) => {
                assert_eq!(s.as_ref(), "hello");
            }
            _ => panic!("Expected String literal pattern"),
        }
    }

    #[test]
    fn test_parse_literal_pattern_bool() {
        let mut input = test_input("true");
        let result = pattern(&mut input).unwrap();
        match result {
            Pattern::Literal(Literal::Bool(true)) => {}
            _ => panic!("Expected Bool literal pattern"),
        }
    }

    #[test]
    fn test_parse_nested_variant_pattern() {
        let mut input = test_input("Some { value: (x, y) }");
        let result = pattern(&mut input).unwrap();
        match result {
            Pattern::Variant { name, fields } => {
                assert_eq!(name.as_ref(), "Some");
                let fields = fields.unwrap();
                assert_eq!(fields.len(), 1);
                assert!(matches!(&fields[0].1, Pattern::Tuple(_)));
            }
            _ => panic!("Expected Variant pattern with nested tuple"),
        }
    }
}
