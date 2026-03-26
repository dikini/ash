# TASK-283: Fix REPL Multiline Error Detection

## Status: 📝 Planned

## Description

Fix REPL multiline detection which currently turns many real syntax errors into continuation prompts. The logic treats any parse error containing "expected" as incomplete input, but SPEC-011 requires incomplete input to continue while actual errors surface immediately.

## Specification Reference

- SPEC-011: REPL Specification - Section 2.3 (Input Handling)

## Dependencies

- ✅ TASK-079: REPL multiline input detection
- ✅ TASK-083: REPL error display improvements

## Requirements

### Functional Requirements

1. Incomplete input (unclosed brackets, quotes) should continue to next prompt
2. Actual syntax errors should surface immediately with clear error message
3. Detection must distinguish between "need more input" vs "this is wrong"
4. Multi-line constructs should work (multi-line strings, nested blocks)
5. Error recovery should allow continuing after error

### SPEC-011 Input Handling Rules

| Input State | REPL Behavior |
|-------------|---------------|
| Complete, valid | Evaluate and show result |
| Incomplete (unclosed) | Continue to next line with `...` prompt |
| Complete, syntax error | Show error immediately, new prompt |
| Complete, type error | Show error, new prompt |

### Current State (Broken)

**File:** `crates/ash-repl/src/input.rs`

```rust
pub fn is_complete_input(input: &str) -> InputStatus {
    match parse(input) {
        Ok(_) => InputStatus::Complete,
        Err(e) => {
            // WRONG: Any error containing "expected" is treated as incomplete
            if e.message().contains("expected") {
                InputStatus::Incomplete // But this catches real errors too!
            } else {
                InputStatus::Error(e)
            }
        }
    }
}

// Example: This input gives "expected ';'" but is a REAL error
//   let x = 
// User should see error immediately, not be prompted for more input
```

### Target State (Fixed)

```rust
#[derive(Debug, Clone)]
pub enum InputStatus {
    Complete,           // Ready to evaluate
    Incomplete(String), // Need more input (reason)
    Error(ParseError),  // Syntax error - show immediately
}

pub struct InputDetector {
    brace_stack: Vec<char>,  // Track {, [, (
    in_string: bool,         // Track string literals
    string_delim: char,      // ' or "
    escape_next: bool,       // Escape character handling
}

impl InputDetector {
    pub fn new() -> Self {
        Self {
            brace_stack: Vec::new(),
            in_string: false,
            string_delim: '"',
            escape_next: false,
        }
    }
    
    pub fn check(&mut self, input: &str) -> InputStatus {
        // First: structural analysis for incomplete input
        let structural = self.check_structure(input);
        
        match structural {
            StructuralStatus::Unclosed { reason } => {
                return InputStatus::Incomplete(reason);
            }
            StructuralStatus::Balanced => {
                // Structure is complete, try to parse
                match parse(input) {
                    Ok(_) => InputStatus::Complete,
                    Err(e) => {
                        // Check if error indicates structural incompleteness
                        if self.is_incomplete_error(&e) {
                            InputStatus::Incomplete(e.message())
                        } else {
                            InputStatus::Error(e)
                        }
                    }
                }
            }
        }
    }
    
    fn check_structure(&mut self, input: &str) -> StructuralStatus {
        for ch in input.chars() {
            if self.escape_next {
                self.escape_next = false;
                continue;
            }
            
            if ch == '\\' && self.in_string {
                self.escape_next = true;
                continue;
            }
            
            if self.in_string {
                if ch == self.string_delim {
                    self.in_string = false;
                }
                continue;
            }
            
            match ch {
                '"' | '\'' => {
                    self.in_string = true;
                    self.string_delim = ch;
                }
                '{' | '[' | '(' => self.brace_stack.push(ch),
                '}' => {
                    if self.brace_stack.last() != Some(&'{') {
                        return StructuralStatus::Balanced; // Mismatched, let parser report
                    }
                    self.brace_stack.pop();
                }
                ']' => {
                    if self.brace_stack.last() != Some(&'[') {
                        return StructuralStatus::Balanced;
                    }
                    self.brace_stack.pop();
                }
                ')' => {
                    if self.brace_stack.last() != Some(&'(') {
                        return StructuralStatus::Balanced;
                    }
                    self.brace_stack.pop();
                }
                _ => {}
            }
        }
        
        if self.in_string {
            StructuralStatus::Unclosed { 
                reason: format!("unclosed string literal (missing {})", self.string_delim) 
            }
        } else if !self.brace_stack.is_empty() {
            StructuralStatus::Unclosed { 
                reason: format!("unclosed delimiter: {:?}", self.brace_stack) 
            }
        } else {
            StructuralStatus::Balanced
        }
    }
    
    fn is_incomplete_error(&self, error: &ParseError) -> bool {
        // Only treat as incomplete if:
        // 1. Error is at end of input
        // 2. Error suggests missing token (not wrong token)
        // 3. Structure analysis suggests more input needed
        
        let msg = error.message().to_lowercase();
        
        // These patterns suggest incomplete input
        let incomplete_patterns = [
            "unexpected end of file",
            "unexpected eof",
            "unclosed",
            "expected expression",
            "missing",
        ];
        
        // These patterns are actual errors even with "expected"
        let real_error_patterns = [
            "unexpected token",
            "invalid",
            "cannot",
            "mismatched",
        ];
        
        let looks_incomplete = incomplete_patterns.iter().any(|p| msg.contains(p));
        let looks_like_real_error = real_error_patterns.iter().any(|p| msg.contains(p));
        
        looks_incomplete && !looks_like_real_error
    }
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-repl/tests/multiline_error_test.rs`

```rust
//! Tests for REPL multiline error detection

use ash_repl::input::{InputDetector, InputStatus};

#[test]
fn test_complete_expression() {
    let mut detector = InputDetector::new();
    
    assert!(matches!(
        detector.check("42"),
        InputStatus::Complete
    ));
    
    assert!(matches!(
        detector.check("let x = 1 + 2"),
        InputStatus::Complete
    ));
}

#[test]
fn test_unclosed_brace_continues() {
    let mut detector = InputDetector::new();
    
    assert!(matches!(
        detector.check("workflow test {"),
        InputStatus::Incomplete(_)
    ));
    
    assert!(matches!(
        detector.check("if x > 0 {"),
        InputStatus::Incomplete(_)
    ));
}

#[test]
fn test_unclosed_paren_continues() {
    let mut detector = InputDetector::new();
    
    assert!(matches!(
        detector.check("foo("),
        InputStatus::Incomplete(_)
    ));
    
    assert!(matches!(
        detector.check("(1 + 2"),
        InputStatus::Incomplete(_)
    ));
}

#[test]
fn test_unclosed_string_continues() {
    let mut detector = InputDetector::new();
    
    assert!(matches!(
        detector.check(r#""hello"#),
        InputStatus::Incomplete(_)
    ));
    
    assert!(matches!(
        detector.check("'unclosed"),
        InputStatus::Incomplete(_)
    ));
}

#[test]
fn test_syntax_error_surfaces_immediately() {
    let mut detector = InputDetector::new();
    
    // This should be an error, not incomplete
    let result = detector.check("let x = ");
    
    // Should be Error, not Incomplete
    assert!(matches!(result, InputStatus::Error(_)));
}

#[test]
fn test_invalid_token_error_surfaces() {
    let mut detector = InputDetector::new();
    
    // Invalid syntax should error immediately
    let result = detector.check("@#$%^");
    
    assert!(matches!(result, InputStatus::Error(_)));
}

#[test]
fn test_type_error_in_complete_input() {
    let mut detector = InputDetector::new();
    
    // Type errors are in complete input, should not continue
    let result = detector.check(r#""string" + 42"#);
    
    // Structure is complete, should parse then type error
    assert!(matches!(result, InputStatus::Complete | InputStatus::Error(_)));
}

#[test]
fn test_multiline_string() {
    let mut detector = InputDetector::new();
    
    // First line - incomplete string
    assert!(matches!(
        detector.check(r#""line 1"#),
        InputStatus::Incomplete(_)
    ));
    
    // Complete the string
    assert!(matches!(
        detector.check(r#""line 1\nline 2""#),
        InputStatus::Complete
    ));
}

#[test]
fn test_nested_blocks() {
    let mut detector = InputDetector::new();
    
    // Nested unclosed blocks
    assert!(matches!(
        detector.check("workflow test { if x { "),
        InputStatus::Incomplete(_)
    ));
    
    // Close inner
    assert!(matches!(
        detector.check("workflow test { if x { } "),
        InputStatus::Incomplete(_)
    ));
    
    // Close outer
    assert!(matches!(
        detector.check("workflow test { if x { } }"),
        InputStatus::Complete
    ));
}

#[test]
fn test_escaped_quotes_in_string() {
    let mut detector = InputDetector::new();
    
    // Escaped quote should not close string
    assert!(matches!(
        detector.check(r#""hello \"world"#),
        InputStatus::Incomplete(_)
    ));
    
    // Complete with escaped quote
    assert!(matches!(
        detector.check(r#""hello \"world\"""#),
        InputStatus::Complete
    ));
}

#[test]
fn test_real_error_not_incomplete() {
    let mut detector = InputDetector::new();
    
    // These are real syntax errors that should show immediately
    let error_cases = vec![
        "let x =",           // Missing expression
        "if { }",            // Missing condition
        "match x { }",       // Empty match (may be error)
        "foo(",              // Unclosed paren is incomplete, but
        "foo() bar",         // Extra tokens after expression
    ];
    
    for case in error_cases {
        let result = detector.check(case);
        // Should NOT be Incomplete for cases that are clearly errors
        if case == "foo() bar" {
            assert!(!matches!(result, InputStatus::Incomplete(_)), 
                "'{}' should not be incomplete", case);
        }
    }
}
```

### Step 2: Implement InputDetector

**File:** `crates/ash-repl/src/input.rs`

```rust
use ash_parser::{parse, ParseError};

#[derive(Debug, Clone)]
pub enum InputStatus {
    Complete,
    Incomplete(String),
    Error(ParseError),
}

pub struct InputDetector {
    brace_stack: Vec<char>,
    in_string: bool,
    string_delim: char,
    escape_next: bool,
}

#[derive(Debug)]
enum StructuralStatus {
    Balanced,
    Unclosed { reason: String },
}

impl InputDetector {
    pub fn new() -> Self {
        Self {
            brace_stack: Vec::new(),
            in_string: false,
            string_delim: '"',
            escape_next: false,
        }
    }
    
    pub fn check(&mut self, input: &str) -> InputStatus {
        // Reset state
        self.brace_stack.clear();
        self.in_string = false;
        self.escape_next = false;
        
        // First check structure
        match self.check_structure(input) {
            StructuralStatus::Unclosed { reason } => {
                return InputStatus::Incomplete(reason);
            }
            StructuralStatus::Balanced => {
                // Try to parse
                match parse(input) {
                    Ok(_) => InputStatus::Complete,
                    Err(e) => {
                        if self.is_incomplete_error(&e) {
                            InputStatus::Incomplete(e.to_string())
                        } else {
                            InputStatus::Error(e)
                        }
                    }
                }
            }
        }
    }
    
    fn check_structure(&mut self, input: &str) -> StructuralStatus {
        for ch in input.chars() {
            if self.escape_next {
                self.escape_next = false;
                continue;
            }
            
            if self.in_string {
                if ch == '\\' {
                    self.escape_next = true;
                } else if ch == self.string_delim {
                    self.in_string = false;
                }
                continue;
            }
            
            match ch {
                '"' | '\'' => {
                    self.in_string = true;
                    self.string_delim = ch;
                }
                '{' | '[' | '(' => self.brace_stack.push(ch),
                '}' => {
                    if self.brace_stack.last() != Some(&'{') {
                        return StructuralStatus::Balanced;
                    }
                    self.brace_stack.pop();
                }
                ']' => {
                    if self.brace_stack.last() != Some(&'[') {
                        return StructuralStatus::Balanced;
                    }
                    self.brace_stack.pop();
                }
                ')' => {
                    if self.brace_stack.last() != Some(&'(') {
                        return StructuralStatus::Balanced;
                    }
                    self.brace_stack.pop();
                }
                _ => {}
            }
        }
        
        if self.in_string {
            StructuralStatus::Unclosed { 
                reason: format!("unclosed string (missing '{}')", self.string_delim)
            }
        } else if !self.brace_stack.is_empty() {
            let expected: String = self.brace_stack.iter().map(|c| match c {
                '{' => "}",
                '[' => "]",
                '(' => ")",
                _ => "",
            }).collect();
            StructuralStatus::Unclosed { 
                reason: format!("unclosed delimiter, expecting: {}", expected)
            }
        } else {
            StructuralStatus::Balanced
        }
    }
    
    fn is_incomplete_error(&self, error: &ParseError) -> bool {
        let msg = error.to_string().to_lowercase();
        
        // These suggest we need more input
        let incomplete_indicators = [
            "unexpected end",
            "unexpected eof",
            "unclosed",
        ];
        
        incomplete_indicators.iter().any(|p| msg.contains(p))
    }
}
```

### Step 3: Update REPL Loop

**File:** `crates/ash-repl/src/repl.rs`

```rust
impl Repl {
    pub fn run(&mut self) -> Result<(), ReplError> {
        let mut input_buffer = String::new();
        let mut detector = InputDetector::new();
        
        loop {
            let prompt = if input_buffer.is_empty() { ">>> " } else { "... " };
            print!("{}", prompt);
            io::stdout().flush()?;
            
            let mut line = String::new();
            if io::stdin().read_line(&mut line)? == 0 {
                // EOF
                break;
            }
            
            if input_buffer.is_empty() {
                input_buffer = line;
            } else {
                input_buffer.push_str(&line);
            }
            
            // Check if input is complete
            match detector.check(&input_buffer) {
                InputStatus::Complete => {
                    let input = input_buffer.trim();
                    if input == ":quit" || input == ":q" {
                        break;
                    }
                    
                    match self.session.evaluate(input) {
                        Ok(result) => self.display_result(&result),
                        Err(e) => self.display_error(&e),
                    }
                    
                    input_buffer.clear();
                }
                InputStatus::Incomplete(reason) => {
                    // Continue to next line
                    if self.session.verbose {
                        eprintln!("[incomplete: {}]", reason);
                    }
                }
                InputStatus::Error(e) => {
                    // Show error and reset
                    self.display_parse_error(&e);
                    input_buffer.clear();
                }
            }
        }
        
        Ok(())
    }
}
```

## Verification Steps

- [ ] `cargo test -p ash-repl --test multiline_error_test` passes
- [ ] `cargo test -p ash-repl` all tests pass
- [ ] Manual REPL testing with various inputs
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- Correct multiline detection
- Immediate error surfacing
- SPEC-011 compliance

Required by:
- REPL user experience

## Notes

**User Impact**: Current behavior is frustrating - users get stuck in "..." prompts for real errors.

**Design Decision**: Two-phase detection - structural first, then parser error classification.

**Edge Cases**:
- Unicode in strings
- Multi-line comments
- Heredocs (if supported)
- Copy-paste with multiple complete expressions
