//! Pattern parser for the Ash language.
//!
//! This module provides parsers for Ash patterns used in let bindings,
//! for loops, and match expressions.

use winnow::combinator::alt;
use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::take_while;

use crate::input::ParseInput;
use crate::parse_utils::skip_whitespace_and_comments;
use crate::surface::{Literal, Name, Pattern, VariantPatternPayload};
use crate::token::Span;

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

/// Parse a variant pattern: `Name`, `Name { field: pat, ... }`, or `Name(pat, ...)`
///
/// Examples:
/// - `None` (unit variant)
/// - `Some { value: x }` (record variant)
/// - `RuntimeError(code, msg)` (tuple variant)
fn parse_variant_pattern(input: &mut ParseInput) -> ModalResult<Pattern> {
    let start_pos = input.state.pos;
    let checkpoint = input.clone();

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

    // Check if followed by `{` (record payload), `(` (tuple payload), or not (unit variant)
    skip_whitespace_and_comments(input);

    // Lowercase identifiers cannot be variant patterns with payloads.
    // If the name starts with a lowercase letter and is followed by `{` or `(`,
    // return a parse error (Cut) so alt() does not try other branches with
    // consumed input.
    let is_uppercase_leading = name.chars().next().is_some_and(|c| c.is_ascii_uppercase());

    if !is_uppercase_leading && (input.input.starts_with('{') || input.input.starts_with('(')) {
        *input = checkpoint;
        return Err(winnow::error::ErrMode::Cut(
            winnow::error::ContextError::new(),
        ));
    }

    if input.input.starts_with('{') {
        let fields = match parse_variant_fields(input) {
            Ok(f) => f,
            Err(_) => {
                *input = checkpoint;
                return Err(winnow::error::ErrMode::Backtrack(
                    winnow::error::ContextError::new(),
                ));
            }
        };

        let _span = span_from(&start_pos, &input.state.pos);
        return Ok(Pattern::Variant {
            name: name.into(),
            fields: Some(fields.clone()),
            payload: VariantPatternPayload::Record(fields),
        });
    }

    if input.input.starts_with('(') {
        let items = parse_variant_tuple_items(input)
            .map_err(|_| winnow::error::ErrMode::Cut(winnow::error::ContextError::new()))?;

        let _span = span_from(&start_pos, &input.state.pos);
        return Ok(Pattern::Variant {
            name: name.into(),
            fields: None,
            payload: VariantPatternPayload::Tuple(items),
        });
    }

    // No payload after the identifier: parse as a unit variant only when the
    // identifier is UpperCamelCase (uppercase-leading). Otherwise, it is a
    // variable pattern (e.g. `x`).
    let is_uppercase_leading = name.chars().next().is_some_and(|c| c.is_ascii_uppercase());

    if is_uppercase_leading {
        let _span = span_from(&start_pos, &input.state.pos);
        Ok(Pattern::Variant {
            name: name.into(),
            fields: None,
            payload: VariantPatternPayload::Unit,
        })
    } else {
        // Backtrack and let variable pattern handle it.
        *input = checkpoint;
        Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ))
    }
}

fn parse_variant_tuple_items(input: &mut ParseInput) -> ModalResult<Vec<Pattern>> {
    let _ = literal_str("(").parse_next(input)?;
    let checkpoint = input.clone();
    let items = match parse_pattern_list(input) {
        Ok(items) => items,
        Err(err) => {
            *input = checkpoint;
            return Err(err);
        }
    };
    if literal_str(")").parse_next(input).is_err() {
        *input = checkpoint;
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    if input.input.starts_with(':') {
        *input = checkpoint;
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    Ok(items)
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

        // Handle `..` (rest pattern — ignore remaining fields)
        if input.input.starts_with("..") {
            let _ = input.input.next_slice(2);
            input.state.advance('.');
            input.state.advance('.');
            skip_whitespace_and_comments(input);
            // Optional trailing comma after `..`
            if input.input.starts_with(",") {
                let _ = input.input.next_slice(1);
                input.state.advance(',');
            }
            skip_whitespace_and_comments(input);
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
    let (name, span) = crate::parse_utils::identifier_with_span(input)?;
    Ok(Pattern::Variable {
        name: name.into(),
        span,
    })
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

    let start_pos = input.state.pos;
    let checkpoint = input.clone();

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

        // Handle `..` (rest pattern — ignore remaining fields)
        if input.input.starts_with("..") {
            let _ = input.input.next_slice(2);
            input.state.advance('.');
            input.state.advance('.');
            skip_whitespace_and_comments(input);
            if input.input.starts_with(",") {
                let _ = input.input.next_slice(1);
                input.state.advance(',');
            }
            skip_whitespace_and_comments(input);
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

        // Check for shorthand syntax: `field` (no colon) means `field: field`
        // This is only valid when followed by `,` or `}`
        if !input.input.starts_with(':') {
            // Shorthand: field name becomes both the field and the variable
            if input.input.starts_with(",") || input.input.starts_with("}") {
                let field_pattern = Pattern::Variable {
                    name: field_name.into(),
                    span: span_from(&start_pos, &input.state.pos),
                };
                fields.push((field_name.into(), field_pattern));
                
                // Optional comma
                if input.input.starts_with(",") {
                    let _ = input.input.next_slice(1);
                    input.state.advance(',');
                }
                continue;
            } else {
                // Not shorthand and not followed by colon — invalid
                *input = checkpoint;
                return Err(winnow::error::ErrMode::Backtrack(
                    winnow::error::ContextError::new(),
                ));
            }
        }

        // Must have `:` for explicit record pattern
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
    let _span = span_from(&start_pos, &input.state.pos);
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
        Ok(f) => Ok(Literal::Float(ordered_float::OrderedFloat(f))),
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
    crate::parse_utils::identifier_with_span(input).map(|(s, _)| s)
}

/// Parse a keyword (ensures word boundary).
fn keyword<'a>(word: &'a str) -> impl Parser<ParseInput<'a>, &'a str, winnow::error::ContextError> {
    move |input: &mut ParseInput<'a>| {
        skip_whitespace_and_comments(input);

        if input.input.starts_with(word) {
            let after = &input.input[word.len()..];
            if after
                .chars()
                .next()
                .is_none_or(|c| !c.is_ascii_alphanumeric())
            {
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
        assert!(matches!(result, Pattern::Variable { name, .. } if name.as_ref() == "x"));
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
        assert!(matches!(result, Pattern::Variable { name, .. } if name.as_ref() == "supervises"));
    }

    #[test]
    fn test_parse_variant_pattern_unit() {
        let mut input = test_input("None");
        let result = pattern(&mut input).unwrap();
        match result {
            Pattern::Variant {
                name,
                fields,
                payload,
            } => {
                assert_eq!(name.as_ref(), "None");
                assert!(fields.is_none());
                assert!(matches!(payload, VariantPatternPayload::Unit));
            }
            _ => panic!("Expected Variant pattern for unit variant"),
        }
    }

    #[test]
    fn test_parse_variant_pattern_with_fields() {
        let mut input = test_input("Some { value: x }");
        let result = pattern(&mut input).unwrap();
        match result {
            Pattern::Variant {
                name,
                fields,
                payload,
            } => {
                assert_eq!(name.as_ref(), "Some");
                assert!(fields.is_some());
                let fields = fields.unwrap();
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].0.as_ref(), "value");
                assert!(
                    matches!(&fields[0].1, Pattern::Variable { name: v, .. } if v.as_ref() == "x")
                );
                assert!(
                    matches!(payload, VariantPatternPayload::Record(items) if items.len() == 1)
                );
            }
            _ => panic!("Expected Variant pattern, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_variant_pattern_multiple_fields() {
        let mut input = test_input("Ok { value: x, error: e }");
        let result = pattern(&mut input).unwrap();
        match result {
            Pattern::Variant {
                name,
                fields,
                payload,
            } => {
                assert_eq!(name.as_ref(), "Ok");
                assert!(fields.is_some());
                let fields = fields.unwrap();
                assert_eq!(fields.len(), 2);
                assert!(
                    matches!(payload, VariantPatternPayload::Record(items) if items.len() == 2)
                );
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
            Pattern::Variant {
                name,
                fields,
                payload,
            } => {
                assert_eq!(name.as_ref(), "Some");
                let fields = fields.unwrap();
                assert_eq!(fields.len(), 1);
                assert!(matches!(&fields[0].1, Pattern::Tuple(_)));
                assert!(
                    matches!(payload, VariantPatternPayload::Record(items) if items.len() == 1)
                );
            }
            _ => panic!("Expected Variant pattern with nested tuple"),
        }
    }

    #[test]
    fn test_lowercase_tuple_variant_pattern_rejected() {
        // `foo(bar)` should not parse as a variant pattern
        let mut input = test_input("foo(bar)");
        let result = pattern(&mut input);
        assert!(
            result.is_err(),
            "Expected lowercase tuple variant pattern to be rejected, got: {result:?}"
        );
    }

    #[test]
    fn test_lowercase_record_variant_pattern_rejected() {
        // `foo { x: y }` should not parse as a variant pattern
        let mut input = test_input("foo { x: y }");
        let result = pattern(&mut input);
        assert!(
            result.is_err(),
            "Expected lowercase record variant pattern to be rejected, got: {result:?}"
        );
    }

    #[test]
    fn test_uppercase_tuple_variant_pattern_accepted() {
        let mut input = test_input("Foo(bar)");
        let result = pattern(&mut input).unwrap();
        match result {
            Pattern::Variant {
                name,
                payload: VariantPatternPayload::Tuple(items),
                ..
            } => {
                assert_eq!(name.as_ref(), "Foo");
                assert_eq!(items.len(), 1);
            }
            other => panic!("Expected uppercase tuple variant pattern, got {other:?}"),
        }
    }

    #[test]
    fn test_uppercase_record_variant_pattern_accepted() {
        let mut input = test_input("Foo { x: y }");
        let result = pattern(&mut input).unwrap();
        match result {
            Pattern::Variant {
                name,
                payload: VariantPatternPayload::Record(fields),
                ..
            } => {
                assert_eq!(name.as_ref(), "Foo");
                assert_eq!(fields.len(), 1);
            }
            other => panic!("Expected uppercase record variant pattern, got {other:?}"),
        }
    }
}
