//! Error formatting and display for the REPL.

use colored::Colorize;

/// Format an error with context and highlighting.
///
/// # Arguments
///
/// * `source` - The source code that caused the error.
/// * `error` - The error message.
/// * `line_num` - The line number where the error occurred (1-indexed).
///
/// # Returns
///
/// A formatted error string with line numbers and highlighting.
#[must_use]
pub fn format_error(source: &str, error: &str, line_num: Option<usize>) -> String {
    let mut output = String::new();

    // Error header
    output.push_str(&format!("{}\n", "Error:".red().bold()));
    output.push_str(&format!("  {error}\n"));

    if let Some(line) = line_num {
        output.push('\n');

        // Show context lines
        let lines: Vec<&str> = source.lines().collect();
        let start = line.saturating_sub(2);
        let end = (line + 1).min(lines.len());

        for (i, line_content) in lines.iter().enumerate().take(end).skip(start) {
            let line_number = i + 1;
            let is_error_line = line_number == line;

            if is_error_line {
                output.push_str(&format!(
                    "{} | {}\n",
                    line_number.to_string().red().bold(),
                    line_content
                ));
                // Add caret underline
                let caret = "^".repeat(line_content.len().max(1));
                output.push_str(&format!(
                    "{} | {}\n",
                    " ".repeat(line_number.to_string().len()),
                    caret.red().bold()
                ));
            } else {
                output.push_str(&format!(
                    "{} | {}\n",
                    line_number.to_string().dimmed(),
                    line_content
                ));
            }
        }
    }

    output
}

/// Format a type error with context.
///
/// # Arguments
///
/// * `expr` - The expression that caused the type error.
/// * `expected` - The expected type.
/// * `found` - The actual type found.
///
/// # Returns
///
/// A formatted type error string.
#[must_use]
#[allow(dead_code)]
pub fn format_type_error(expr: &str, expected: &str, found: &str) -> String {
    format!(
        "{}\n  {}\n\nExpected: {}\nFound: {}\n",
        "Type Error:".red().bold(),
        format!("Type mismatch in expression: {}", expr),
        expected.green(),
        found.red()
    )
}

/// Suggest fixes for common errors.
///
/// # Arguments
///
/// * `error` - The error message to analyze.
///
/// # Returns
///
/// An optional suggestion string if a common error pattern is recognized.
#[must_use]
pub fn suggest_fix(error: &str) -> Option<String> {
    let error_lower = error.to_lowercase();

    if error_lower.contains("unexpected end of input") {
        Some("Did you forget to close a brace or parenthesis?".to_string())
    } else if error_lower.contains("unterminated string") {
        Some("Check that all string literals are properly closed with quotes.".to_string())
    } else if error_lower.contains("unknown keyword") {
        Some("Check for typos in keywords.".to_string())
    } else if error_lower.contains("expected") && error_lower.contains("found") {
        Some("Check the syntax matches the expected pattern.".to_string())
    } else if error_lower.contains("parse") {
        Some("Verify the syntax is correct.".to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_error_with_line() {
        let source = "1 +\n2 + 3";
        let error = "unexpected end of input";
        let formatted = format_error(source, error, Some(1));

        assert!(formatted.contains("Error:"));
        assert!(formatted.contains("unexpected end of input"));
        assert!(formatted.contains("1"));
        assert!(formatted.contains("|"));
    }

    #[test]
    fn test_format_error_without_line() {
        let formatted = format_error("", "something went wrong", None);
        assert!(formatted.contains("Error:"));
        assert!(formatted.contains("something went wrong"));
        assert!(!formatted.contains("|"));
    }

    #[test]
    fn test_format_error_with_context() {
        let source = "line 1\nline 2\nline 3\nline 4";
        let error = "test error";
        let formatted = format_error(source, error, Some(2));

        // Should show context lines around line 2
        assert!(formatted.contains("1 |"));
        assert!(formatted.contains("2 |"));
        assert!(formatted.contains("3 |"));
    }

    #[test]
    fn test_format_type_error() {
        let formatted = format_type_error("x + 1", "Int", "String");
        assert!(formatted.contains("Type Error:"));
        assert!(formatted.contains("Expected:"));
        assert!(formatted.contains("Found:"));
        assert!(formatted.contains("Int"));
        assert!(formatted.contains("String"));
    }

    #[test]
    fn test_suggest_fix_unclosed() {
        let suggestion = suggest_fix("unexpected end of input");
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("brace"));
    }

    #[test]
    fn test_suggest_fix_unterminated_string() {
        let suggestion = suggest_fix("unterminated string literal");
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("quotes"));
    }

    #[test]
    fn test_suggest_fix_unknown_keyword() {
        let suggestion = suggest_fix("unknown keyword: wrkflow");
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("typos"));
    }

    #[test]
    fn test_suggest_fix_none() {
        let suggestion = suggest_fix("some random error that is not recognized");
        assert!(suggestion.is_none());
    }

    #[test]
    fn test_suggest_fix_case_insensitive() {
        let suggestion = suggest_fix("UNEXPECTED END OF INPUT");
        assert!(suggestion.is_some());
    }
}
