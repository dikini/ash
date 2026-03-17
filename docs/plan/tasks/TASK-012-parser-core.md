# TASK-012: Parser Core (Winnow)

## Status: 🟢 Complete

## Description

Implement the core parser infrastructure using the `winnow` parser combinator library. This provides the foundation for parsing all Ash surface syntax.

## Specification Reference

- SPEC-002: Surface Language - Section 3. Grammar
- winnow documentation: https://docs.rs/winnow/

## Requirements

### Parser Infrastructure

1. **Combinator Library Setup**
   - Use `winnow` for parser combinators
   - Define custom input type wrapping `&str` with span tracking
   - Implement error recovery mechanisms

2. **Error Types**
   - Rich parse errors with spans
   - Multiple error collection
   - Suggestions for common mistakes

3. **State Management**
   - Parser state for indentation tracking (if needed)
   - Context for better error messages

### Input Type

```rust
use winnow::prelude::*;

/// Parser input with span tracking
#[derive(Debug, Clone, Copy)]
pub struct ParseInput<'a> {
    pub input: &'a str,
    pub start_offset: usize,
    pub line: usize,
    pub column: usize,
}

impl<'a> ParseInput<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            start_offset: 0,
            line: 1,
            column: 1,
        }
    }
    
    pub fn span_from(&self, start: Self) -> Span {
        Span {
            start: start.start_offset,
            end: self.start_offset,
            line: start.line,
            column: start.column,
        }
    }
}

impl<'a> AsRef<str> for ParseInput<'a> {
    fn as_ref(&self) -> &str {
        self.input
    }
}

impl<'a> Stream for ParseInput<'a> {
    type Token = char;
    type Slice = &'a str;
    // ... implementation
}
```

### Core Parsers

```rust
/// Parse a specific keyword
pub fn keyword<'a>(kw: &'static str) -> impl Parser<ParseInput<'a>, &'a str, ParseError> {
    move |input: &mut ParseInput<'a>| {
        let ident = identifier.parse_next(input)?;
        if ident.eq_ignore_ascii_case(kw) {
            Ok(ident)
        } else {
            Err(ErrMode::from_error_kind(input, ErrorKind::Tag))
        }
    }
}

/// Parse an identifier
pub fn identifier<'a>(input: &mut ParseInput<'a>) -> PResult<&'a str, ParseError> {
    take_while(1.., |c: char| c.is_ascii_alphabetic() || c == '_')
        .and_then(take_while(0.., |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-'))
        .parse_next(input)
}

/// Parse whitespace and comments
pub fn ws<'a>(input: &mut ParseInput<'a>) -> PResult<&'a str, ParseError> {
    take_while(0.., |c: char| c.is_whitespace() || c == '-')
        .parse_next(input)
}

/// A combinator that consumes surrounding whitespace
pub fn token<'a, O, P>(parser: P) -> impl Parser<ParseInput<'a>, O, ParseError>
where
    P: Parser<ParseInput<'a>, O, ParseError>,
{
    delimited(ws, parser, ws)
}
```

### Error Type

```rust
#[derive(Debug, Clone)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
    pub expected: Vec<String>,
    pub label: Option<String>,
}

impl ParseError {
    pub fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            expected: vec![],
            label: None,
        }
    }
    
    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.expected.push(expected.into());
        self
    }
    
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "error: {}", self.message)?;
        writeln!(f, "  --> line {}, column {}", self.span.line, self.span.column)?;
        if !self.expected.is_empty() {
            writeln!(f, "  = expected: {}", self.expected.join(", "))?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {}
```

### Recovery Strategies

```rust
/// Recover from error by skipping to next semicolon or closing brace
pub fn recover_to_next_stmt<'a, O, P>(
    parser: P,
) -> impl Parser<ParseInput<'a>, Option<O>, ParseError>
where
    P: Parser<ParseInput<'a>, O, ParseError>,
{
    move |input: &mut ParseInput<'a>| {
        let checkpoint = input.checkpoint();
        match parser.parse_next(input) {
            Ok(result) => Ok(Some(result)),
            Err(_) => {
                input.reset(&checkpoint);
                // Skip until semicolon or closing brace
                let _ = take_till(1.., |c| c == ';' || c == '}').parse_next(input);
                let _ = opt(one_of([';', '}'])).parse_next(input);
                Ok(None)  // Recovery returns None
            }
        }
    }
}
```

## TDD Steps

### Step 1: Set up winnow dependency

Add to `crates/ash-parser/Cargo.toml`:
```toml
[dependencies]
winnow = { workspace = true }
```

### Step 2: Implement ParseInput

Create `crates/ash-parser/src/input.rs` with the custom input type.

### Step 3: Implement Error Types

Create `crates/ash-parser/src/error.rs` with ParseError.

### Step 4: Implement Basic Combinators

Create `crates/ash-parser/src/combinators.rs`:
- `keyword`
- `identifier`
- `ws` (whitespace)
- `token`
- `delimited`
- `separated_list`
- `recover_to_next_stmt`

### Step 5: Write Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_keyword() {
        let input = ParseInput::new("workflow");
        let result = keyword("workflow").parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_identifier() {
        let input = ParseInput::new("foo_bar123");
        let (rem, result) = identifier.parse_peek(input).unwrap();
        assert_eq!(result, "foo_bar123");
        assert!(rem.input.is_empty());
    }

    #[test]
    fn test_parse_delimited() {
        let input = ParseInput::new("( hello )");
        let result = delimited('(', token(identifier), ')').parse(input);
        assert!(result.is_ok());
    }
}
```

## Completion Checklist

- [ ] ParseInput type with span tracking
- [ ] ParseError type with rich diagnostics
- [ ] Basic combinators: keyword, identifier, ws, token
- [ ] Delimited parser for parentheses, braces, brackets
- [ ] Separated list parser
- [ ] Error recovery strategy
- [ ] Unit tests for all combinators
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **winnow integration**: Are we using winnow idiomatically?
2. **Performance**: Are parsers allocation-efficient?
3. **Error messages**: Are they actionable and precise?

## Estimated Effort

8 hours (winnow learning curve + infrastructure)

## Dependencies

- TASK-008: Token definitions (uses Span)
- TASK-011: Surface AST (parsers produce these types)

## Blocked By

- TASK-008: Token definitions
- TASK-011: Surface AST

## Blocks

- TASK-013: Parser workflows (uses these combinators)
- TASK-014: Parser expressions (uses these combinators)
