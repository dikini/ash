# TASK-015: Parser Error Recovery

## Status: 🟢 Complete

## Description

Implement sophisticated error recovery for the parser to report multiple errors and continue parsing past common mistakes.

## Specification Reference

- SPEC-002: Surface Language - Section 5. Error Recovery

## Requirements

### Error Recovery Strategies

1. **Statement-Level Recovery**
   - Skip to next semicolon or closing brace
   - Continue parsing subsequent statements

2. **Expression-Level Recovery**
   - Skip to next operator or delimiter
   - Insert placeholder expression

3. **Block-Level Recovery**
   - Synchronize on closing brace
   - Report missing closing delimiter

4. **Definition-Level Recovery**
   - Skip to next top-level definition
   - Continue with subsequent definitions

### Error Types

```rust
#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    UnexpectedToken { found: String, expected: Vec<String> },
    UnclosedDelimiter { opened: char, expected: char, location: Span },
    MissingSemicolon { location: Span },
    InvalidExpression { location: Span, reason: String },
    UnknownKeyword { word: String, suggestion: Option<String> },
    InvalidOperator { op: String },
}

#[derive(Debug, Clone)]
pub struct RichParseError {
    pub kind: ParseErrorKind,
    pub span: Span,
    pub help: Option<String>,
}

impl fmt::Display for RichParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ParseErrorKind::UnexpectedToken { found, expected } => {
                writeln!(f, "error: unexpected token `{}`", found)?;
                if !expected.is_empty() {
                    writeln!(f, "  = expected: {}", expected.join(", "))?;
                }
            }
            ParseErrorKind::UnclosedDelimiter { opened, expected, .. } => {
                writeln!(f, "error: unclosed delimiter `{}`", opened)?;
                writeln!(f, "  = expected: `{}`", expected)?;
            }
            ParseErrorKind::UnknownKeyword { word, suggestion } => {
                writeln!(f, "error: unknown keyword `{}`", word)?;
                if let Some(sugg) = suggestion {
                    writeln!(f, "  = did you mean: `{}`?", sugg)?;
                }
            }
            _ => writeln!(f, "error: {:?}", self.kind)?,
        }
        
        if let Some(help) = &self.help {
            writeln!(f, "  = help: {}", help)?;
        }
        
        Ok(())
    }
}
```

### Recovery Implementation

```rust
/// Parse with error recovery, collecting all errors
pub fn parse_with_recovery<'a>(input: &'a str) -> (Option<Program>, Vec<RichParseError>) {
    let mut errors = Vec::new();
    let parse_input = ParseInput::new(input);
    
    match program(&mut parse_input) {
        Ok(prog) => (Some(prog), errors),
        Err(e) => {
            errors.push(convert_error(e));
            // Attempt recovery
            (None, errors)
        }
    }
}

/// Recovery combinator that collects errors and continues
pub fn recoverable<'a, O, P>(
    parser: P,
    recover: impl Fn(&mut ParseInput<'a>),
) -> impl FnMut(&mut ParseInput<'a>) -> PResult<Option<O>, RichParseError>
where
    P: Parser<ParseInput<'a>, O, RichParseError>,
{
    move |input: &mut ParseInput<'a>| {
        match parser.parse_next(input) {
            Ok(result) => Ok(Some(result)),
            Err(e) => {
                // Apply recovery
                recover(input);
                Err(e)
            }
        }
    }
}

/// Skip to next synchronization point
pub fn skip_to_sync(input: &mut ParseInput<'_>) {
    // Skip until we find a semicolon, closing brace, or keyword
    let sync_tokens = [";", "}", "workflow", "capability", "policy", "role"];
    
    while !input.is_empty() {
        for token in &sync_tokens {
            if input.starts_with(token) {
                return;
            }
        }
        input.next_char();
    }
}

/// Skip to statement boundary
pub fn skip_to_stmt_end(input: &mut ParseInput<'_>) {
    let mut depth = 0;
    
    while !input.is_empty() {
        match input.peek() {
            Some('{') => { depth += 1; input.next_char(); }
            Some('}') if depth > 0 => { depth -= 1; input.next_char(); }
            Some('}') => return,
            Some(';') if depth == 0 => return,
            _ => { input.next_char(); }
        }
    }
}
```

### Smart Suggestions

```rust
use strsim::{levenshtein, normalized_levenshtein};

const KEYWORDS: &[&str] = &[
    "workflow", "capability", "policy", "role",
    "observe", "orient", "propose", "decide", "act",
    "let", "if", "then", "else", "for", "do", "par",
    "with", "maybe", "must", "done",
];

pub fn suggest_keyword(word: &str) -> Option<String> {
    KEYWORDS
        .iter()
        .map(|&kw| (kw, normalized_levenshtein(word, kw)))
        .filter(|(_, sim)| *sim > 0.6)
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(kw, _)| kw.to_string())
}

pub fn check_unknown_keyword(word: &str) -> Option<RichParseError> {
    if KEYWORDS.contains(&word) {
        return None;
    }
    
    let suggestion = suggest_keyword(word);
    
    Some(RichParseError {
        kind: ParseErrorKind::UnknownKeyword {
            word: word.to_string(),
            suggestion,
        },
        span: Span::default(),  // populated by caller
        help: suggestion.as_ref().map(|s| format!("did you mean `{}`?", s)),
    })
}
```

### Error Reporting Format

```rust
pub fn format_error(input: &str, error: &RichParseError) -> String {
    let mut output = String::new();
    
    // Error message
    output.push_str(&format!("error: {:?}\n", error.kind));
    
    // Location
    let line_num = error.span.line;
    let col = error.span.column;
    
    // Get line content
    let lines: Vec<&str> = input.lines().collect();
    if let Some(line) = lines.get(line_num.saturating_sub(1)) {
        output.push_str(&format!("  --> {}:{}\n", line_num, col));
        output.push_str(&format!("   |\n"));
        output.push_str(&format!("{:3}| {}\n", line_num, line));
        
        // Error indicator
        let spaces = " ".repeat(col.saturating_sub(1) + 4);
        let carets = "^".repeat(error.span.end.saturating_sub(error.span.start).max(1));
        output.push_str(&format!("{}  | {}{}\n", spaces, spaces, carets));
    }
    
    // Help message
    if let Some(help) = &error.help {
        output.push_str(&format!("   = help: {}\n", help));
    }
    
    output
}
```

## TDD Steps

### Step 1: Create Error Types

Create `crates/ash-parser/src/error.rs` with RichParseError and ParseErrorKind.

### Step 2: Implement Basic Recovery

Implement `skip_to_sync` and `skip_to_stmt_end` functions.

### Step 3: Add Suggestion Logic

Add `suggest_keyword` using levenshtein distance (add `strsim` dependency).

### Step 4: Integrate Recovery into Parser

Modify parsers to use `recoverable` combinator.

### Step 5: Write Recovery Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_missing_semicolon() {
        let input = r#"
            workflow test {
                observe foo
                act bar
            }
        "#;
        
        let (prog, errors) = parse_with_recovery(input);
        
        // Should still produce a program
        assert!(prog.is_some());
        // Should have at least one error
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_recovery_unknown_keyword_suggestion() {
        let input = "workflo test { done }";
        let (_, errors) = parse_with_recovery(input);
        
        assert!(!errors.is_empty());
        // Check that suggestion is provided
        if let ParseErrorKind::UnknownKeyword { suggestion: Some(sugg), .. } = &errors[0].kind {
            assert_eq!(sugg, "workflow");
        }
    }

    #[test]
    fn test_multiple_errors_collected() {
        let input = r#"
            workflow test {
                obserb foo
                actt bar
                decidee { true } then { done }
            }
        "#;
        
        let (_, errors) = parse_with_recovery(input);
        
        // Should collect multiple errors
        assert!(errors.len() >= 3);
    }
}
```

## Completion Checklist

- [ ] RichParseError type with all error kinds
- [ ] Error recovery strategies implemented
- [ ] Keyword suggestion using levenshtein distance
- [ ] Multiple error collection
- [ ] Pretty error formatting with context
- [ ] Recovery tests for common mistakes
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Coverage**: Can we recover from all common syntax errors?
2. **Suggestions**: Are keyword suggestions helpful?
3. **Performance**: Does error recovery significantly slow parsing?

## Estimated Effort

6 hours

## Dependencies

- TASK-012: Parser core
- TASK-013: Workflow parser
- TASK-014: Expression parser

## Blocked By

- TASK-012, TASK-013, TASK-014

## Blocks

- TASK-025: Type errors (builds on error infrastructure)
