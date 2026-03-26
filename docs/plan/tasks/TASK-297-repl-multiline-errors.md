# TASK-297: Fix REPL Multiline Error Detection

## Status: 📝 Planned

## Description

Fix the issue where REPL multiline detection turns many real syntax errors into continuation prompts. Treating any parse error containing "expected" as incomplete is too broad. SPEC-011 requires incomplete input to continue, but actual errors to surface immediately.

## Specification Reference

- SPEC-011: REPL Specification

## Dependencies

- ✅ TASK-077: REPL crate
- ✅ TASK-079: REPL multiline input detection
- ✅ TASK-083: REPL error display

## Critical File Locations

- `crates/ash-repl/src/lib.rs:413` - multiline detection logic

## Requirements

### Functional Requirements

1. Incomplete input (trailing operator, unclosed delimiter) should continue
2. Actual syntax errors should surface immediately
3. SPEC-011 multiline rules must be followed
4. Error messages must be clear

### Current State (Broken)

**File:** `crates/ash-repl/src/lib.rs:413`

```rust
fn is_incomplete_input(input: &str, error: &ParseError) -> bool {
    // WRONG: Too broad - any "expected" error triggers continuation
    error.message().contains("expected")  // Line 413
}
```

Problems:
1. Real syntax errors treated as incomplete
2. Users see `...` prompt for errors
3. SPEC-011 compliance broken
4. Poor user experience

### Target State (Fixed)

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum InputStatus {
    Complete,
    Incomplete(IncompleteReason),
    Error(ParseError),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum IncompleteReason {
    UnclosedDelimiter,      // {, [, (
    TrailingOperator,       // +, -, etc
    TrailingBackslash,      // Line continuation
    UnclosedString,         // ", '
    TrailingComma,          // Potential more args
    UnclosedBlockComment,   // /*
}

fn classify_input(input: &str) -> InputStatus {
    // Try to parse as complete input
    match try_parse_complete(input) {
        Ok(_) => InputStatus::Complete,
        Err(e) => {
            // Analyze error to determine if incomplete or actual error
            if let Some(reason) = is_genuinely_incomplete(input, &e) {
                InputStatus::Incomplete(reason)
            } else {
                InputStatus::Error(e)
            }
        }
    }
}

fn is_genuinely_incomplete(input: &str, error: &ParseError) -> Option<IncompleteReason> {
    use regex::Regex;
    
    // Check for unclosed delimiters
    let delimiters = Regex::new(r"[({\[]").unwrap();
    let closes = Regex::new(r"[)}\]]").unwrap();
    let open_count = delimiters.find_iter(input).count();
    let close_count = closes.find_iter(input).count();
    
    if open_count > close_count {
        return Some(IncompleteReason::UnclosedDelimiter);
    }
    
    // Check for trailing operator
    let trimmed = input.trim_end();
    if trimmed.ends_with('+') || trimmed.ends_with('-') || 
       trimmed.ends_with('*') || trimmed.ends_with('/') ||
       trimmed.ends_with('=') || trimmed.ends_with('.') ||
       trimmed.ends_with("->") || trimmed.ends_with("=>") {
        return Some(IncompleteReason::TrailingOperator);
    }
    
    // Check for trailing backslash
    if trimmed.ends_with('\\') {
        return Some(IncompleteReason::TrailingBackslash);
    }
    
    // Check for unclosed string
    let single_quotes = input.chars().filter(|&c| c == '\'').count();
    let double_quotes = input.chars().filter(|&c| c == '"').count();
    
    // Rough check - doesn't handle escapes
    if single_quotes % 2 == 1 {
        return Some(IncompleteReason::UnclosedString);
    }
    if double_quotes % 2 == 1 {
        return Some(IncompleteReason::UnclosedString);
    }
    
    // Check for trailing comma in arg list
    if trimmed.ends_with(',') && 
       (trimmed.contains('(') || trimmed.contains('[')) {
        return Some(IncompleteReason::TrailingComma);
    }
    
    // Check for unclosed block comment
    let block_comment_opens = input.matches("/*").count();
    let block_comment_closes = input.matches("*/").count();
    if block_comment_opens > block_comment_closes {
        return Some(IncompleteReason::UnclosedBlockComment);
    }
    
    // Check error message for specific incomplete patterns
    let msg = error.message();
    
    // These indicate genuine incompleteness
    if msg.contains("unexpected end of file") ||
       msg.contains("unexpected end of input") {
        return Some(IncompleteReason::UnclosedDelimiter);
    }
    
    // Actual syntax errors should not continue
    None
}

fn handle_input_line(&mut self, line: &str) -> ReplResult {
    self.current_input.push_str(line);
    self.current_input.push('\n');
    
    match classify_input(&self.current_input) {
        InputStatus::Complete => {
            let input = std::mem::take(&mut self.current_input);
            self.execute_complete(&input)
        }
        InputStatus::Incomplete(reason) => {
            if self.verbose {
                eprintln!("[Incomplete: {:?}]", reason);
            }
            // Continue to next line
            self.show_continuation_prompt();
            Ok(Value::Null)
        }
        InputStatus::Error(e) => {
            // Surface the error immediately
            self.current_input.clear();
            Err(ReplError::Parse(e))
        }
    }
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-repl/tests/multiline_detection_test.rs`

```rust
//! Tests for REPL multiline detection

use ash_repl::{Repl, InputStatus};

#[test]
fn test_unclosed_brace_continues() {
    let mut repl = Repl::new();
    
    // Unclosed block should continue
    let status = repl.classify_input("workflow test {");
    assert!(matches!(status, InputStatus::Incomplete(_)));
}

#[test]
fn test_trailing_operator_continues() {
    let mut repl = Repl::new();
    
    // Trailing + should continue
    let status = repl.classify_input("1 +");
    assert!(matches!(status, InputStatus::Incomplete(_)));
}

#[test]
fn test_unclosed_paren_continues() {
    let mut repl = Repl::new();
    
    let status = repl.classify_input("foo(");
    assert!(matches!(status, InputStatus::Incomplete(_)));
}

#[test]
fn test_unclosed_string_continues() {
    let mut repl = Repl::new();
    
    let status = repl.classify_input("\"hello");
    assert!(matches!(status, InputStatus::Incomplete(_)));
}

#[test]
fn test_actual_syntax_error_surfaces() {
    let mut repl = Repl::new();
    
    // This is a real syntax error, not incomplete
    let status = repl.classify_input("let x = }");
    
    // Should be Error, not Incomplete
    assert!(matches!(status, InputStatus::Error(_)));
}

#[test]
fn test_invalid_token_error_surfaces() {
    let mut repl = Repl::new();
    
    // Invalid characters
    let status = repl.classify_input("let x = @#$");
    
    // Should error immediately
    assert!(matches!(status, InputStatus::Error(_)));
}

#[test]
fn test_complete_expression_accepted() {
    let mut repl = Repl::new();
    
    let status = repl.classify_input("1 + 2");
    assert!(matches!(status, InputStatus::Complete));
}

#[test]
fn test_complete_workflow_accepted() {
    let mut repl = Repl::new();
    
    let status = repl.classify_input(r#"
        workflow test {
            act print("hello");
        }
    "#);
    assert!(matches!(status, InputStatus::Complete));
}

#[test]
fn test_trailing_comma_in_args_continues() {
    let mut repl = Repl::new();
    
    let status = repl.classify_input("foo(1, 2,");
    assert!(matches!(status, InputStatus::Incomplete(_)));
}

#[test]
fn test_trailing_backslash_continues() {
    let mut repl = Repl::new();
    
    let status = repl.classify_input("let x = \\");
    assert!(matches!(status, InputStatus::Incomplete(_)));
}

#[test]
fn test_type_error_surfaces() {
    let mut repl = Repl::new();
    
    // Type errors (post-parse) should surface
    let status = repl.classify_input("let x: Int = \"string\"");
    
    // If it parses but type checks, it's Complete
    // Type errors are handled at execution, not classification
    assert!(matches!(status, InputStatus::Complete));
}

#[test]
fn test_unclosed_block_comment_continues() {
    let mut repl = Repl::new();
    
    let status = repl.classify_input("/* this is a comment");
    assert!(matches!(status, InputStatus::Incomplete(_)));
}

#[test]
fn test_invalid_identifier_error_surfaces() {
    let mut repl = Repl::new();
    
    // 123foo is not a valid identifier
    let status = repl.classify_input("let 123foo = 1");
    assert!(matches!(status, InputStatus::Error(_)));
}

#[test]
fn test_unexpected_token_error_surfaces() {
    let mut repl = Repl::new();
    
    // } without opening {
    let status = repl.classify_input("}");
    assert!(matches!(status, InputStatus::Error(_)));
}

#[test]
fn test_multiline_block_accepted() {
    let mut repl = Repl::new();
    
    // First line
    let status1 = repl.classify_input("workflow test {");
    assert!(matches!(status1, InputStatus::Incomplete(_)));
    
    // Second line (still incomplete)
    repl.add_line("    act print(\"hello\");");
    let status2 = repl.classify_input(&repl.current_input);
    assert!(matches!(status2, InputStatus::Incomplete(_)));
    
    // Third line (complete)
    repl.add_line("}");
    let status3 = repl.classify_input(&repl.current_input);
    assert!(matches!(status3, InputStatus::Complete));
}

proptest! {
    #[test]
    fn balanced_delimiters_are_complete(input in balanced_delim_strategy()) {
        let repl = Repl::new();
        let status = repl.classify_input(&input);
        // Balanced input should be either Complete or Error, not Incomplete
        assert!(!matches!(status, InputStatus::Incomplete(_)));
    }
}
```

### Step 2: Implement Input Classification

**File:** `crates/ash-repl/src/input_classifier.rs`

```rust
//! Input classification for multiline detection

use ash_parser::{parse, ParseError};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputStatus {
    Complete,
    Incomplete(IncompleteReason),
    Error(ParseError),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IncompleteReason {
    UnclosedDelimiter,
    TrailingOperator,
    TrailingBackslash,
    UnclosedString,
    TrailingComma,
    UnclosedBlockComment,
}

pub struct InputClassifier;

impl InputClassifier {
    pub fn new() -> Self {
        Self
    }
    
    pub fn classify(&self, input: &str) -> InputStatus {
        match parse(input) {
            Ok(_) => InputStatus::Complete,
            Err(e) => {
                if let Some(reason) = self.is_incomplete(input, &e) {
                    InputStatus::Incomplete(reason)
                } else {
                    InputStatus::Error(e)
                }
            }
        }
    }
    
    fn is_incomplete(&self, input: &str, error: &ParseError) -> Option<IncompleteReason> {
        let trimmed = input.trim_end();
        
        // Check structural indicators first
        
        // Unclosed delimiters
        let open_count = input.chars().filter(|c| *c == '(' || *c == '[' || *c == '{').count();
        let close_count = input.chars().filter(|c| *c == ')' || *c == ']' || *c == '}').count();
        if open_count > close_count {
            return Some(IncompleteReason::UnclosedDelimiter);
        }
        
        // Trailing operators indicating more input expected
        if trimmed.ends_with('+') || trimmed.ends_with('-') ||
           trimmed.ends_with('*') || trimmed.ends_with('/') ||
           trimmed.ends_with('%') || trimmed.ends_with('=') ||
           trimmed.ends_with('.') || trimmed.ends_with("->") ||
           trimmed.ends_with("=>") || trimmed.ends_with("&&") ||
           trimmed.ends_with("||") {
            return Some(IncompleteReason::TrailingOperator);
        }
        
        // Line continuation
        if trimmed.ends_with('\\') {
            return Some(IncompleteReason::TrailingBackslash);
        }
        
        // Unclosed strings (rough check)
        let single_quotes = input.matches('\'').count();
        let double_quotes = input.matches('"').count();
        if single_quotes % 2 == 1 || double_quotes % 2 == 1 {
            return Some(IncompleteReason::UnclosedString);
        }
        
        // Trailing comma in call
        if trimmed.ends_with(',') && (input.contains('(') || input.contains('[')) {
            return Some(IncompleteReason::TrailingComma);
        }
        
        // Unclosed block comment
        let opens = input.matches("/*").count();
        let closes = input.matches("*/").count();
        if opens > closes {
            return Some(IncompleteReason::UnclosedBlockComment);
        }
        
        // Check error message
        let msg = error.to_string().to_lowercase();
        
        // These indicate genuine incompleteness
        if msg.contains("unexpected end") ||
           msg.contains("unexpected eof") ||
           msg.contains("unclosed") {
            return Some(IncompleteReason::UnclosedDelimiter);
        }
        
        // Actual errors
        None
    }
}
```

### Step 3: Update REPL Input Handling

**File:** `crates/ash-repl/src/lib.rs`

```rust
use crate::input_classifier::{InputClassifier, InputStatus};

pub struct Repl {
    // ... existing fields ...
    classifier: InputClassifier,
    current_input: String,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            // ...
            classifier: InputClassifier::new(),
            current_input: String::new(),
        }
    }
    
    pub fn handle_line(&mut self, line: &str) -> ReplResult {
        self.current_input.push_str(line);
        self.current_input.push('\n');
        
        match self.classifier.classify(&self.current_input) {
            InputStatus::Complete => {
                let input = std::mem::take(&mut self.current_input);
                self.execute(&input)
            }
            InputStatus::Incomplete(reason) => {
                if self.verbose {
                    eprintln!("  [continuing: {:?}]", reason);
                }
                self.show_continuation_prompt();
                Ok(Value::Null)
            }
            InputStatus::Error(e) => {
                self.current_input.clear();
                Err(ReplError::Parse(e))
            }
        }
    }
    
    fn show_continuation_prompt(&self) {
        print!("... ");
        io::stdout().flush().unwrap();
    }
}
```

## Verification Steps

- [ ] `cargo test -p ash-repl --test multiline_detection_test` passes
- [ ] Real syntax errors surface immediately
- [ ] Incomplete input continues properly
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- Correct multiline detection
- SPEC-011 compliance for REPL input

Required by:
- REPL usability
- Interactive development

## Notes

**Critical Issue**: REPL is unusable for many error cases.

**Risk Assessment**: Medium - affects REPL user experience.

**Implementation Strategy**:
1. First: Create input classifier with clear rules
2. Second: Distinguish incomplete from error
3. Third: Update REPL to use classifier
4. Fourth: Test edge cases
