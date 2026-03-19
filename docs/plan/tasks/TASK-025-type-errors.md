# TASK-025: Rich Type Error Messages

## Status: ✅ Complete

## Description

Implement rich, actionable type error messages with source context, suggestions, and helpful diagnostics.

## Specification Reference

- SPEC-003: Type System - Section 9. Error Messages

## Requirements

### Error Types

```rust
/// Rich type error with context and suggestions
#[derive(Debug, Clone)]
pub struct RichTypeError {
    pub code: ErrorCode,
    pub message: String,
    pub span: Span,
    pub primary_label: String,
    pub secondary_labels: Vec<(Span, String)>,
    pub help: Option<String>,
    pub suggestion: Option<Suggestion>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // Type mismatch errors
    E001, // Type mismatch
    E002, // Effect mismatch
    E003, // Cannot unify
    
    // Name resolution errors
    E010, // Undefined variable
    E011, // Undefined capability
    E012, // Undefined policy
    
    // Obligation errors
    E020, // Unfulfilled obligation
    E021, // Prohibition violated
    E022, // Role separation violated
    
    // Safety errors
    E030, // Unauthorized operational action
    E031, // Guard undecidable
    
    // Constraint errors
    E040, // Policy conflict
    E041, // Resource bound exceeded
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub message: String,
    pub replacement: String,
    pub span: Span,
}
```

### Error Formatter

```rust
/// Formats type errors for display
pub struct ErrorFormatter<'a> {
    source: &'a str,
    filename: Option<&'a str>,
}

impl<'a> ErrorFormatter<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source, filename: None }
    }
    
    pub fn with_filename(mut self, filename: &'a str) -> Self {
        self.filename = Some(filename);
        self
    }
    
    /// Format a single error
    pub fn format(&self, error: &RichTypeError) -> String {
        let mut output = String::new();
        
        // Error header
        let code_str = format!("[{:?}]", error.code);
        writeln!(output, "error{}: {}", code_str, error.message).unwrap();
        
        // Location
        let loc = self.format_location(&error.span);
        writeln!(output, "  --> {}", loc).unwrap();
        
        // Source snippet
        writeln!(output, "   |").unwrap();
        if let Some(snippet) = self.get_snippet(&error.span) {
            let line_num = error.span.line;
            writeln!(output, "{:3}| {}", line_num, snippet).unwrap();
            
            // Error indicator
            let padding = " ".repeat(4 + error.span.column - 1);
            let carets = "^".repeat(error.span.end.saturating_sub(error.span.start).max(1));
            writeln!(output, "   |{}{}", padding, carets).unwrap();
        }
        writeln!(output, "   |").unwrap();
        
        // Primary label
        if !error.primary_label.is_empty() {
            writeln!(output, "   = {}", error.primary_label).unwrap();
        }
        
        // Secondary labels
        for (span, label) in &error.secondary_labels {
            let loc = self.format_location(span);
            writeln!(output, "   = note: at {}: {}", loc, label).unwrap();
        }
        
        // Help
        if let Some(help) = &error.help {
            writeln!(output, "   = help: {}", help).unwrap();
        }
        
        // Suggestion
        if let Some(sugg) = &error.suggestion {
            writeln!(output, "   |").unwrap();
            writeln!(output, "   = suggestion: {}", sugg.message).unwrap();
            writeln!(output, "   |").unwrap();
            
            if let Some(snippet) = self.get_snippet(&sugg.span) {
                writeln!(output, "{:3}| {}", sugg.span.line, snippet).unwrap();
                let padding = " ".repeat(4 + sugg.span.column - 1);
                writeln!(output, "   |{}{}", padding, "-".repeat(sugg.replacement.len())).unwrap();
                writeln!(output, "{:3}| {}{}", sugg.span.line, padding, sugg.replacement).unwrap();
            }
        }
        
        output
    }
    
    fn format_location(&self, span: &Span) -> String {
        match self.filename {
            Some(f) => format!("{}:{}:{}", f, span.line, span.column),
            None => format!("{}:{}", span.line, span.column),
        }
    }
    
    fn get_snippet(&self, span: &Span) -> Option<&str> {
        self.source.lines().nth(span.line.saturating_sub(1))
    }
}
```

### Error Builders

```rust
/// Builder for constructing rich type errors
pub struct TypeErrorBuilder {
    code: ErrorCode,
    message: String,
    span: Span,
    primary_label: String,
    secondary_labels: Vec<(Span, String)>,
    help: Option<String>,
    suggestion: Option<Suggestion>,
}

impl TypeErrorBuilder {
    pub fn new(code: ErrorCode) -> Self {
        Self {
            code,
            message: String::new(),
            span: Span::default(),
            primary_label: String::new(),
            secondary_labels: Vec::new(),
            help: None,
            suggestion: None,
        }
    }
    
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = msg.into();
        self
    }
    
    pub fn at(mut self, span: Span) -> Self {
        self.span = span;
        self
    }
    
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.primary_label = label.into();
        self
    }
    
    pub fn secondary(mut self, span: Span, label: impl Into<String>) -> Self {
        self.secondary_labels.push((span, label.into()));
        self
    }
    
    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
    
    pub fn suggest(mut self, message: impl Into<String>, replacement: impl Into<String>, span: Span) -> Self {
        self.suggestion = Some(Suggestion {
            message: message.into(),
            replacement: replacement.into(),
            span,
        });
        self
    }
    
    pub fn build(self) -> RichTypeError {
        RichTypeError {
            code: self.code,
            message: self.message,
            span: self.span,
            primary_label: self.primary_label,
            secondary_labels: self.secondary_labels,
            help: self.help,
            suggestion: self.suggestion,
        }
    }
}
```

### Specific Error Constructors

```rust
/// Create a type mismatch error
pub fn type_mismatch(expected: &Type, actual: &Type, span: Span) -> RichTypeError {
    TypeErrorBuilder::new(ErrorCode::E001)
        .message(format!("type mismatch: expected `{}`, found `{}`", expected, actual))
        .at(span)
        .label(format!("expected `{}`", expected))
        .help("ensure both sides of the assignment have compatible types")
        .build()
}

/// Create an effect mismatch error
pub fn effect_mismatch(expected: Effect, actual: Effect, span: Span) -> RichTypeError {
    let help = if actual > expected {
        format!(
            "this operation has effect `{}` which exceeds the allowed effect `{}`",
            actual, expected
        )
    } else {
        format!("effects do not match")
    };
    
    TypeErrorBuilder::new(ErrorCode::E002)
        .message(format!("effect mismatch"))
        .at(span)
        .label(format!("has effect `{}`", actual))
        .help(help)
        .suggest(
            "add a decision before this action",
            "decide { condition } under policy then {".to_string(),
            span,
        )
        .build()
}

/// Create an undefined variable error
pub fn undefined_variable(name: &str, span: Span, similar: Option<&str>) -> RichTypeError {
    let mut builder = TypeErrorBuilder::new(ErrorCode::E010)
        .message(format!("undefined variable: `{}`", name))
        .at(span)
        .label(format!("not found in this scope"));
    
    if let Some(s) = similar {
        builder = builder
            .help(format!("did you mean `{}`?", s))
            .suggest("try this instead", s.to_string(), span);
    } else {
        builder = builder.help("variables must be declared before use");
    }
    
    builder.build()
}

/// Create an unauthorized action error
pub fn unauthorized_action(action: &ActionRef, span: Span) -> RichTypeError {
    TypeErrorBuilder::new(ErrorCode::E030)
        .message(format!("operational action without authorization"))
        .at(span)
        .label(format!("this action requires a preceding decision"))
        .help("operational actions must be preceded by a `decide` statement")
        .suggest(
            "add a decision before this action",
            format!("decide {{ condition }} under policy then {{\n  act {}(...);\n}}", action.name),
            span,
        )
        .build()
}

/// Create an unfulfilled obligation error
pub fn unfulfilled_obligation(obligation: &Obligation, span: Span) -> RichTypeError {
    TypeErrorBuilder::new(ErrorCode::E020)
        .message(format!("unfulfilled obligation"))
        .at(span)
        .label(format!("this obligation is never discharged"))
        .help(format!("add a `check` statement to discharge this obligation"))
        .build()
}
```

### JSON Output

```rust
/// Error format for machine-readable output
#[derive(Debug, Serialize)]
pub struct JsonError {
    pub code: String,
    pub message: String,
    pub file: Option<String>,
    pub line: usize,
    pub column: usize,
    pub labels: Vec<JsonLabel>,
    pub help: Option<String>,
    pub suggestion: Option<JsonSuggestion>,
}

#[derive(Debug, Serialize)]
pub struct JsonLabel {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Serialize)]
pub struct JsonSuggestion {
    pub message: String,
    pub replacement: String,
}

impl RichTypeError {
    pub fn to_json(&self, filename: Option<&str>) -> JsonError {
        JsonError {
            code: format!("{:?}", self.code),
            message: self.message.clone(),
            file: filename.map(|s| s.to_string()),
            line: self.span.line,
            column: self.span.column,
            labels: vec![JsonLabel {
                message: self.primary_label.clone(),
                line: self.span.line,
                column: self.span.column,
            }],
            help: self.help.clone(),
            suggestion: self.suggestion.as_ref().map(|s| JsonSuggestion {
                message: s.message.clone(),
                replacement: s.replacement.clone(),
            }),
        }
    }
}
```

## TDD Steps

### Step 1: Define Error Types

Create `crates/ash-typeck/src/errors.rs` with RichTypeError.

### Step 2: Implement ErrorFormatter

Add formatting with source context and pretty printing.

### Step 3: Implement ErrorBuilder

Add builder pattern for constructing errors.

### Step 4: Implement Specific Constructors

Add constructors for each error type.

### Step 5: Add JSON Output

Add serialization for machine-readable output.

### Step 6: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_formatting() {
        let source = r#"workflow test {
  act delete_file();
}"#;
        
        let error = unauthorized_action(
            &ActionRef { name: "delete_file".into(), args: vec![] },
            Span { line: 2, column: 3, start: 20, end: 35 },
        );
        
        let formatter = ErrorFormatter::new(source);
        let formatted = formatter.format(&error);
        
        assert!(formatted.contains("error"));
        assert!(formatted.contains("unauthorized"));
        assert!(formatted.contains("--> 2:3"));
    }

    #[test]
    fn test_type_mismatch_error() {
        let error = type_mismatch(
            &Type::Int,
            &Type::String,
            Span::default(),
        );
        
        assert_eq!(error.code, ErrorCode::E001);
        assert!(error.message.contains("type mismatch"));
    }

    #[test]
    fn test_json_output() {
        let error = undefined_variable("foo", Span { line: 5, column: 10, start: 100, end: 103 }, None);
        let json = error.to_json(Some("test.ash"));
        
        assert_eq!(json.code, "E010");
        assert_eq!(json.file, Some("test.ash".to_string()));
        assert_eq!(json.line, 5);
    }
}
```

## Completion Checklist

- [ ] RichTypeError with all context fields
- [ ] ErrorCode enum with all error types
- [ ] ErrorFormatter with pretty printing
- [ ] TypeErrorBuilder for construction
- [ ] Specific error constructors for common cases
- [ ] JSON output format
- [ ] Source snippet extraction
- [ ] Suggestion formatting
- [ ] Unit tests for formatting
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Clarity**: Are error messages understandable to users?
2. **Actionability**: Do errors include helpful suggestions?
3. **Consistency**: Is error formatting consistent across all errors?

## Estimated Effort

6 hours

## Dependencies

- All previous type system tasks (produces errors from them)

## Blocked By

- TASK-018 through TASK-024

## Blocks

- TASK-053: CLI check (uses error formatting)
