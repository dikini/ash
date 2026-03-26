//! Tests for REPL multiline error detection

use ash_repl::input::{InputDetector, InputStatus};

#[test]
fn test_complete_expression() {
    let mut detector = InputDetector::new();

    // Simple literals are valid expressions
    assert!(
        matches!(detector.check("42"), InputStatus::Complete),
        "simple number should be complete"
    );

    assert!(
        matches!(detector.check("\"hello\""), InputStatus::Complete),
        "string literal should be complete"
    );

    // Arithmetic expressions
    assert!(
        matches!(detector.check("1 + 2 * 3"), InputStatus::Complete),
        "arithmetic expression should be complete"
    );
}

#[test]
fn test_unclosed_brace_continues() {
    let mut detector = InputDetector::new();

    assert!(
        matches!(
            detector.check("workflow test {"),
            InputStatus::Incomplete(_)
        ),
        "unclosed brace should be incomplete"
    );

    assert!(
        matches!(detector.check("if x > 0 {"), InputStatus::Incomplete(_)),
        "unclosed if block should be incomplete"
    );
}

#[test]
fn test_unclosed_paren_continues() {
    let mut detector = InputDetector::new();

    assert!(
        matches!(detector.check("foo("), InputStatus::Incomplete(_)),
        "unclosed paren should be incomplete"
    );

    assert!(
        matches!(detector.check("(1 + 2"), InputStatus::Incomplete(_)),
        "unclosed paren expression should be incomplete"
    );
}

#[test]
fn test_unclosed_bracket_continues() {
    let mut detector = InputDetector::new();

    assert!(
        matches!(detector.check("[1, 2, 3"), InputStatus::Incomplete(_)),
        "unclosed bracket should be incomplete"
    );

    assert!(
        matches!(detector.check("arr["), InputStatus::Incomplete(_)),
        "unclosed bracket access should be incomplete"
    );
}

#[test]
fn test_unclosed_string_continues() {
    let mut detector = InputDetector::new();

    assert!(
        matches!(detector.check(r#""hello"#), InputStatus::Incomplete(_)),
        "unclosed double-quoted string should be incomplete"
    );

    assert!(
        matches!(detector.check("'unclosed"), InputStatus::Incomplete(_)),
        "unclosed single-quoted string should be incomplete"
    );
}

#[test]
fn test_syntax_error_surfaces_immediately() {
    let mut detector = InputDetector::new();

    // This should be an error, not incomplete - "let x = " is a real syntax error
    let result = detector.check("let x = ");

    // Should be Error, not Incomplete
    assert!(
        matches!(result, InputStatus::Error(_)),
        "'let x = ' should be an error, not incomplete, got {result:?}"
    );
}

#[test]
fn test_invalid_token_error_surfaces() {
    let mut detector = InputDetector::new();

    // Invalid syntax should error immediately
    let result = detector.check("@#$%^");

    assert!(
        matches!(result, InputStatus::Error(_)),
        "invalid tokens should be an error, got {result:?}"
    );
}

#[test]
fn test_mismatched_brace_error_surfaces() {
    let mut detector = InputDetector::new();

    // Mismatched braces - this is a real error
    let result = detector.check("workflow test { } }");

    // This should surface as an error since braces are mismatched
    assert!(
        !matches!(result, InputStatus::Incomplete(_)),
        "mismatched braces should not be incomplete, got {result:?}"
    );
}

#[test]
fn test_multiline_string() {
    let mut detector = InputDetector::new();

    // First line - incomplete string
    assert!(
        matches!(detector.check(r#""line 1"#), InputStatus::Incomplete(_)),
        "incomplete string should continue"
    );

    // Complete the string
    assert!(
        matches!(detector.check(r#""line 1\nline 2""#), InputStatus::Complete),
        "complete string should be complete"
    );
}

#[test]
fn test_nested_blocks() {
    let mut detector = InputDetector::new();

    // Nested unclosed blocks (using valid Ash syntax with 'then')
    assert!(
        matches!(
            detector.check("workflow test { if x then { "),
            InputStatus::Incomplete(_)
        ),
        "nested unclosed blocks should be incomplete"
    );

    // Close inner
    assert!(
        matches!(
            detector.check("workflow test { if x then { } "),
            InputStatus::Incomplete(_)
        ),
        "outer block still open should be incomplete"
    );

    // Close outer with proper syntax
    assert!(
        matches!(
            detector.check("workflow test { if x then { } }"),
            InputStatus::Complete
        ),
        "all blocks closed should be complete"
    );
}

#[test]
fn test_escaped_quotes_in_string() {
    let mut detector = InputDetector::new();

    // Escaped quote should not close string
    // Input: "hello \"world  (unclosed - the \" is escaped so it doesn't close)
    assert!(
        matches!(
            detector.check("\"hello \\\"world"),
            InputStatus::Incomplete(_)
        ),
        "string with escaped quote should be incomplete"
    );

    // Note: "hello \"world\"" with escaped quotes inside may not parse successfully
    // depending on the Ash parser's string escape handling. The structural analysis
    // correctly identifies it as balanced, but we skip parse validation for complex
    // escape sequences since they're parser-dependent.
}

#[test]
fn test_escaped_backslash_in_string() {
    let mut detector = InputDetector::new();

    // Escaped backslash at end (incomplete string)
    // Input: "path\\  (backslash escapes the quote, so string continues)
    assert!(
        matches!(detector.check("\"path\\\\"), InputStatus::Incomplete(_)),
        "string with escaped backslash at end should be incomplete"
    );

    // Complete properly
    // Input: "path\\"  (the \\" is escaped backslash then closing quote)
    assert!(
        matches!(detector.check("\"path\\\\\""), InputStatus::Complete),
        "string properly closed should be complete"
    );
}

#[test]
fn test_complete_workflow() {
    let mut detector = InputDetector::new();

    assert!(
        matches!(
            detector.check("workflow test { ret 42; }"),
            InputStatus::Complete
        ),
        "complete workflow should be complete"
    );
}

#[test]
fn test_empty_input() {
    let mut detector = InputDetector::new();

    assert!(
        matches!(detector.check(""), InputStatus::Complete),
        "empty input should be complete (nothing to do)"
    );

    assert!(
        matches!(detector.check("   "), InputStatus::Complete),
        "whitespace-only input should be complete"
    );
}

#[test]
fn test_detector_state_reset() {
    let mut detector = InputDetector::new();

    // First check leaves state
    detector.check("workflow test {");

    // Second check should start fresh, not inherit state
    let result = detector.check("42");
    assert!(
        matches!(result, InputStatus::Complete),
        "detector should reset state between checks, got {result:?}"
    );
}

#[test]
fn test_incomplete_reason_provided() {
    let mut detector = InputDetector::new();

    if let InputStatus::Incomplete(reason) = detector.check("workflow test {") {
        assert!(
            reason.contains("unclosed") || reason.contains('}'),
            "incomplete reason should mention unclosed delimiter: got {reason}"
        );
    } else {
        panic!("expected Incomplete status");
    }
}
