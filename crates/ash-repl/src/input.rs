//! Input detection for REPL multiline handling
//!
//! This module provides the `InputDetector` which determines whether user input
//! is complete, incomplete (needs more lines), or contains an error.
//!
//! The detection follows SPEC-011 rules:
//! - Complete, valid input -> Evaluate and show result
//! - Incomplete (unclosed brackets, quotes) -> Continue to next line with `...` prompt
//! - Complete, syntax error -> Show error immediately, new prompt

use ash_engine::Engine;

/// The status of user input to the REPL.
#[derive(Debug, Clone)]
pub enum InputStatus {
    /// Input is complete and ready to evaluate.
    Complete,
    /// Input is incomplete and needs more lines.
    Incomplete(String),
    /// Input has a syntax error that should be shown immediately.
    Error(String),
}

/// Internal structural analysis result.
#[derive(Debug)]
enum StructuralStatus {
    /// All delimiters are balanced.
    Balanced,
    /// Input has unclosed delimiters.
    Unclosed {
        /// Human-readable reason for the unclosed state.
        reason: String,
    },
}

/// Detects whether REPL input is complete, incomplete, or erroneous.
///
/// Uses two-phase detection:
/// 1. Structural analysis: checks for balanced braces, brackets, parens, and strings
/// 2. Parse analysis: if structurally balanced, try parsing to detect syntax errors
#[derive(Debug)]
pub struct InputDetector {
    /// Stack of open delimiters: {, [, (
    brace_stack: Vec<char>,
    /// Whether we're currently inside a string literal.
    in_string: bool,
    /// The delimiter for the current string (' or ").
    string_delim: char,
    /// Whether the next character is escaped.
    escape_next: bool,
}

impl Default for InputDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl InputDetector {
    /// Create a new `InputDetector` with fresh state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            brace_stack: Vec::new(),
            in_string: false,
            string_delim: '"',
            escape_next: false,
        }
    }

    /// Check the status of the input.
    ///
    /// Resets internal state before checking, so each call is independent.
    ///
    /// # Arguments
    ///
    /// * `input` - The input string to check.
    ///
    /// # Returns
    ///
    /// An `InputStatus` indicating whether the input is complete, incomplete, or has an error.
    pub fn check(&mut self, input: &str) -> InputStatus {
        // Reset state for fresh analysis
        self.reset();

        // Empty or whitespace-only input is considered complete
        if input.trim().is_empty() {
            return InputStatus::Complete;
        }

        // Phase 1: Structural analysis (unclosed delimiters/strings)
        match self.check_structure(input) {
            StructuralStatus::Unclosed { reason } => {
                // Check if it's a structural issue (delimiters, strings) or trailing operator
                // Structural issues are definitely incomplete
                // Trailing operators need verification from the parser
                if self.has_unclosed_delimiters_or_strings() {
                    return InputStatus::Incomplete(reason);
                }
                // For trailing operators, we need to check with the parser
                // Fall through to parse analysis
            }
            StructuralStatus::Balanced => {}
        }

        // Phase 2: Parse analysis
        Self::check_parse(input)
    }

    /// Returns true if there are unclosed delimiters or strings.
    const fn has_unclosed_delimiters_or_strings(&self) -> bool {
        self.in_string || !self.brace_stack.is_empty()
    }

    /// Reset the detector's internal state.
    fn reset(&mut self) {
        self.brace_stack.clear();
        self.in_string = false;
        self.string_delim = '"';
        self.escape_next = false;
    }

    /// Analyze the structure of the input for unclosed delimiters.
    ///
    /// Tracks:
    /// - Brace pairs: `{` and `}`
    /// - Bracket pairs: `[` and `]`
    /// - Paren pairs: `(` and `)`
    /// - String delimiters: `"` and `'`, respecting escape sequences
    fn check_structure(&mut self, input: &str) -> StructuralStatus {
        for ch in input.chars() {
            if self.handle_escape(ch) {
                continue;
            }

            if self.handle_string_delimiter(ch) {
                continue;
            }

            self.handle_delimiter(ch);
        }

        self.determine_structural_status(input)
    }

    /// Handle escape character processing within strings.
    ///
    /// When inside a string, a backslash escapes the next character.
    /// This means \" doesn't close a double-quoted string, and \\' doesn't
    /// close a single-quoted string.
    ///
    /// Returns `true` if the character was consumed as part of escape handling.
    const fn handle_escape(&mut self, ch: char) -> bool {
        // If previous char was a backslash, this char is escaped
        if self.escape_next {
            self.escape_next = false;
            return true; // Consumed as escaped character
        }

        // If we're in a string and see a backslash, the next char is escaped
        if self.in_string && ch == '\\' {
            self.escape_next = true;
            return true; // Consumed the backslash
        }

        false // Not an escape sequence
    }

    /// Handle string delimiter processing.
    ///
    /// Returns `true` if we were inside a string (whether we closed it or not).
    const fn handle_string_delimiter(&mut self, ch: char) -> bool {
        if !self.in_string {
            if ch == '"' || ch == '\'' {
                self.in_string = true;
                self.string_delim = ch;
                return true;
            }
            return false;
        }

        // We're in a string - check if this closes it
        if ch == self.string_delim {
            self.in_string = false;
        }
        true
    }

    /// Handle brace, bracket, and paren delimiters.
    fn handle_delimiter(&mut self, ch: char) {
        match ch {
            '{' | '[' | '(' => self.brace_stack.push(ch),
            '}' => {
                if self.brace_stack.last() == Some(&'{') {
                    self.brace_stack.pop();
                }
                // Mismatched delimiters are treated as balanced to let parser report the error
            }
            ']' => {
                if self.brace_stack.last() == Some(&'[') {
                    self.brace_stack.pop();
                }
            }
            ')' => {
                if self.brace_stack.last() == Some(&'(') {
                    self.brace_stack.pop();
                }
            }
            _ => {}
        }
    }

    /// Determine the final structural status based on accumulated state.
    fn determine_structural_status(&self, input: &str) -> StructuralStatus {
        if self.in_string {
            return StructuralStatus::Unclosed {
                reason: format!("unclosed string literal (missing '{}')", self.string_delim),
            };
        }

        if !self.brace_stack.is_empty() {
            let expected: String = self
                .brace_stack
                .iter()
                .map(|c| match c {
                    '{' => "}",
                    '[' => "]",
                    '(' => ")",
                    _ => "",
                })
                .collect();
            return StructuralStatus::Unclosed {
                reason: format!("unclosed delimiter, expecting: {expected}"),
            };
        }

        // Check for trailing operators that indicate more input is expected
        if let Some(reason) = Self::check_trailing_operator(input) {
            return StructuralStatus::Unclosed { reason };
        }

        StructuralStatus::Balanced
    }

    /// Check if input ends with a trailing operator (indicating continuation).
    fn check_trailing_operator(input: &str) -> Option<String> {
        let trimmed = input.trim_end();

        // Check for trailing operators that suggest more input is coming
        if trimmed.ends_with('+')
            || trimmed.ends_with('-')
            || trimmed.ends_with('*')
            || trimmed.ends_with('/')
            || trimmed.ends_with('%')
            || trimmed.ends_with('=')
            || trimmed.ends_with('.')
            || trimmed.ends_with("->")
            || trimmed.ends_with("=>")
            || trimmed.ends_with("&&")
            || trimmed.ends_with("||")
        {
            return Some("trailing operator, expecting more input".to_string());
        }

        // Check for trailing comma in function calls or arrays
        if trimmed.ends_with(',') {
            return Some("trailing comma, expecting more arguments".to_string());
        }

        None
    }

    /// Try parsing the input to detect syntax errors.
    ///
    /// For expressions (not workflow definitions), wraps them in a workflow
    /// context like the REPL does during evaluation.
    fn check_parse(input: &str) -> InputStatus {
        let engine = Engine::default();
        let trimmed = input.trim();

        // Check for trailing operators before parsing
        let has_trailing_operator = Self::check_trailing_operator(input).is_some();

        // First, try parsing as-is (for workflow definitions, module declarations, etc.)
        match engine.parse(trimmed) {
            Ok(_) => InputStatus::Complete,
            Err(ash_engine::EngineError::Parse(_err_msg)) => {
                // Try wrapping in a workflow as the REPL does for expressions
                let wrapped = format!("workflow __repl__ {{ ret {trimmed}; }}");
                match engine.parse(&wrapped) {
                    Ok(_) => InputStatus::Complete,
                    Err(ash_engine::EngineError::Parse(msg)) => {
                        if Self::is_incomplete_error(&msg) {
                            return InputStatus::Incomplete(msg);
                        }
                        // If we have a trailing operator but the parse error doesn't indicate
                        // incompleteness, check if it's an expression context
                        if has_trailing_operator && Self::looks_like_expression_prefix(trimmed) {
                            return InputStatus::Incomplete(
                                "trailing operator, expecting more input".to_string(),
                            );
                        }
                        InputStatus::Error(msg)
                    }
                    Err(other) => InputStatus::Error(other.to_string()),
                }
            }
            Err(other) => InputStatus::Error(other.to_string()),
        }
    }

    /// Check if input looks like an expression prefix (not a statement).
    fn looks_like_expression_prefix(input: &str) -> bool {
        let trimmed = input.trim();

        // If it starts with 'let', it's a statement, not an expression
        if trimmed.starts_with("let ") {
            return false;
        }

        // If it starts with 'workflow', it's a definition
        if trimmed.starts_with("workflow ") {
            return false;
        }

        // Otherwise, assume it could be an expression
        true
    }

    /// Determine if a parse error indicates incomplete input vs a real syntax error.
    ///
    /// Incomplete errors suggest the user needs to provide more input.
    /// Real errors suggest the user has made a mistake that won't be fixed by more input.
    fn is_incomplete_error(msg: &str) -> bool {
        let msg_lower = msg.to_lowercase();

        // These patterns suggest we need more input
        let incomplete_patterns = [
            "unexpected end of input",
            "unexpected eof",
            "unexpected end of file",
            "unclosed",
            "unterminated",
        ];

        // These patterns are actual errors even if they contain "expected"
        let real_error_patterns = [
            "unexpected token",
            "invalid",
            "cannot",
            "mismatched",
            "found",
        ];

        let looks_incomplete = incomplete_patterns.iter().any(|p| msg_lower.contains(p));
        let looks_like_real_error = real_error_patterns.iter().any(|p| msg_lower.contains(p));

        // Only treat as incomplete if it looks incomplete AND doesn't look like a real error
        looks_incomplete && !looks_like_real_error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_detector_new() {
        let detector = InputDetector::new();
        assert!(detector.brace_stack.is_empty());
        assert!(!detector.in_string);
        assert!(!detector.escape_next);
    }

    #[test]
    fn test_empty_input_is_complete() {
        let mut detector = InputDetector::new();
        assert!(matches!(detector.check(""), InputStatus::Complete));
    }

    #[test]
    fn test_whitespace_input_is_complete() {
        let mut detector = InputDetector::new();
        assert!(matches!(detector.check("   "), InputStatus::Complete));
        assert!(matches!(detector.check("\n\t  "), InputStatus::Complete));
    }

    #[test]
    fn test_unclosed_brace_incomplete() {
        let mut detector = InputDetector::new();
        let result = detector.check("{");
        assert!(matches!(result, InputStatus::Incomplete(_)));
    }

    #[test]
    fn test_unclosed_paren_incomplete() {
        let mut detector = InputDetector::new();
        let result = detector.check("(");
        assert!(matches!(result, InputStatus::Incomplete(_)));
    }

    #[test]
    fn test_unclosed_bracket_incomplete() {
        let mut detector = InputDetector::new();
        let result = detector.check("[");
        assert!(matches!(result, InputStatus::Incomplete(_)));
    }

    #[test]
    fn test_unclosed_string_incomplete() {
        let mut detector = InputDetector::new();
        let result = detector.check("\"hello");
        assert!(matches!(result, InputStatus::Incomplete(_)));
    }

    #[test]
    fn test_escaped_quote_not_closing() {
        let mut detector = InputDetector::new();
        let result = detector.check("\"hello\\\"");
        assert!(matches!(result, InputStatus::Incomplete(_)));
    }

    #[test]
    fn test_state_reset_between_checks() {
        let mut detector = InputDetector::new();

        // First check with unclosed brace
        detector.check("{");
        assert!(!detector.brace_stack.is_empty());

        // After second check, state should be reset
        detector.check("42");
        // Stack should be reset at the start of check()
    }
}
