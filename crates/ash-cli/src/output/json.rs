//! JSON output format implementation for Ash CLI
//!
//! Implements SPEC-005 compliant JSON output schema for machine-readable
//! diagnostic and verification information.
//!
//! TASK-298: Fixed to use unified diagnostics array with summary counts

use serde::Serialize;
use std::path::Path;
use std::time::Duration;

/// Schema version for JSON output format
pub const SCHEMA_VERSION: &str = "1.0";

/// Top-level JSON output structure for check command results
/// SPEC-005 compliant schema with diagnostics array and summary
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct JsonOutput {
    /// Schema version identifier
    pub schema_version: &'static str,
    /// Path to the checked file
    pub file: String,
    /// Whether type checking succeeded
    pub success: bool,
    /// Whether strict mode was enabled
    pub strict: bool,
    /// Exit code (0 for success, non-zero for errors)
    pub exit_code: u8,
    /// Unified diagnostics array (errors, warnings, info)
    pub diagnostics: Vec<Diagnostic>,
    /// Summary counts
    pub summary: Summary,
    /// Timing information for various phases
    pub timing: JsonTiming,
}

/// Summary of diagnostics counts
#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub struct Summary {
    /// Number of errors
    pub error_count: usize,
    /// Number of warnings
    pub warning_count: usize,
    /// Number of info messages
    pub info_count: usize,
    /// Total number of diagnostics
    pub total_count: usize,
}

/// Diagnostic severity level
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Unified diagnostic with severity
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Diagnostic {
    /// Severity level
    pub severity: Severity,
    /// Error/warning code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Human-readable message
    pub message: String,
    /// Source location
    pub location: JsonLocation,
    /// Source context (line of code)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Help message with suggestion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

/// Source code location
#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub struct JsonLocation {
    /// File path
    pub file: String,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
}

/// Timing information for various phases
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct JsonTiming {
    /// Time spent parsing (milliseconds)
    pub parse_ms: u64,
    /// Time spent type checking (milliseconds)
    pub typecheck_ms: u64,
    /// Total time (milliseconds)
    pub total_ms: u64,
}

impl JsonOutput {
    /// Create a new JSON output for the given file
    pub fn new(file: &Path) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            file: file.display().to_string(),
            success: true,
            strict: false,
            exit_code: 0,
            diagnostics: Vec::new(),
            summary: Summary::default(),
            timing: JsonTiming {
                parse_ms: 0,
                typecheck_ms: 0,
                total_ms: 0,
            },
        }
    }

    /// Set the strict mode flag
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Set the exit code
    pub fn with_exit_code(mut self, exit_code: u8) -> Self {
        self.exit_code = exit_code;
        self
    }

    /// Add timing information
    pub fn with_timing(mut self, parse: Duration, typecheck: Duration, total: Duration) -> Self {
        self.timing = JsonTiming {
            parse_ms: parse.as_millis() as u64,
            typecheck_ms: typecheck.as_millis() as u64,
            total_ms: total.as_millis() as u64,
        };
        self
    }

    /// Add an error diagnostic
    pub fn with_error(mut self, message: &str, code: &str, location: Option<JsonLocation>) -> Self {
        self.success = false;
        self.diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: Some(code.to_string()),
            message: message.to_string(),
            location: location.unwrap_or_default(),
            context: None,
            help: None,
        });
        self.update_summary();
        self
    }

    /// Add a warning diagnostic
    pub fn with_warning(
        mut self,
        message: &str,
        code: &str,
        location: Option<JsonLocation>,
    ) -> Self {
        self.diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: Some(code.to_string()),
            message: message.to_string(),
            location: location.unwrap_or_default(),
            context: None,
            help: None,
        });
        self.update_summary();
        self
    }

    /// Add an info diagnostic
    pub fn with_info(mut self, message: &str, location: Option<JsonLocation>) -> Self {
        self.diagnostics.push(Diagnostic {
            severity: Severity::Info,
            code: None,
            message: message.to_string(),
            location: location.unwrap_or_default(),
            context: None,
            help: None,
        });
        self.update_summary();
        self
    }

    /// Add an error from a string representation (fallback for simple error handling)
    pub fn with_error_string(mut self, error: &str) -> Self {
        self.success = false;
        // Try to extract a code from the error message, or use a generic one
        let code = if error.contains("Parse") {
            "E0001"
        } else if error.contains("Type") {
            "E0002"
        } else {
            "E9999"
        };
        self.diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: Some(code.to_string()),
            message: error.to_string(),
            location: JsonLocation::default(),
            context: None,
            help: None,
        });
        self.update_summary();
        self
    }

    /// Update summary counts from diagnostics
    fn update_summary(&mut self) {
        self.summary = Summary {
            error_count: self
                .diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Error)
                .count(),
            warning_count: self
                .diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Warning)
                .count(),
            info_count: self
                .diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Info)
                .count(),
            total_count: self.diagnostics.len(),
        };
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl JsonLocation {
    /// Create a new location
    pub fn new(file: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            file: file.into(),
            line,
            column,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_output_default() {
        let output = JsonOutput::new(Path::new("test.ash"));
        assert_eq!(output.schema_version, "1.0");
        assert_eq!(output.file, "test.ash");
        assert!(output.success);
        assert!(!output.strict);
        assert_eq!(output.exit_code, 0);
        assert!(output.diagnostics.is_empty());
        assert_eq!(output.summary.total_count, 0);
    }

    #[test]
    fn test_json_output_with_error() {
        let output = JsonOutput::new(Path::new("test.ash")).with_error(
            "type mismatch",
            "E0032",
            Some(JsonLocation::new("test.ash", 42, 15)),
        );

        assert!(!output.success);
        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(output.diagnostics[0].severity, Severity::Error);
        assert_eq!(output.diagnostics[0].code, Some("E0032".to_string()));
        assert_eq!(output.diagnostics[0].message, "type mismatch");
        assert_eq!(output.diagnostics[0].location.line, 42);
        assert_eq!(output.diagnostics[0].location.column, 15);
        assert_eq!(output.summary.error_count, 1);
        assert_eq!(output.summary.total_count, 1);
    }

    #[test]
    fn test_json_output_with_warning() {
        let output = JsonOutput::new(Path::new("test.ash")).with_warning(
            "unused variable",
            "W0001",
            Some(JsonLocation::new("test.ash", 10, 5)),
        );

        assert!(output.success); // Warnings don't affect success
        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(output.diagnostics[0].severity, Severity::Warning);
        assert_eq!(output.summary.warning_count, 1);
        assert_eq!(output.summary.total_count, 1);
    }

    #[test]
    fn test_json_output_serialization() {
        let output = JsonOutput::new(Path::new("test.ash"))
            .with_strict(true)
            .with_exit_code(0)
            .with_info("compilation started", None);

        let json = output.to_json().unwrap();
        assert!(json.contains("schema_version"));
        assert!(json.contains("1.0"));
        assert!(json.contains("test.ash"));
        assert!(json.contains("diagnostics"));
        assert!(json.contains("summary"));
        assert!(json.contains("error_count"));
        assert!(json.contains("warning_count"));
    }

    #[test]
    fn test_json_output_has_diagnostics_array() {
        let output = JsonOutput::new(Path::new("test.ash"));
        let json = output.to_json().unwrap();
        // diagnostics array should always be present (even if empty)
        assert!(json.contains("diagnostics"));
    }

    #[test]
    fn test_json_summary_counts() {
        let output = JsonOutput::new(Path::new("test.ash"))
            .with_error("error 1", "E0001", None)
            .with_error("error 2", "E0002", None)
            .with_warning("warning 1", "W0001", None)
            .with_info("info 1", None);

        assert_eq!(output.summary.error_count, 2);
        assert_eq!(output.summary.warning_count, 1);
        assert_eq!(output.summary.info_count, 1);
        assert_eq!(output.summary.total_count, 4);
    }

    #[test]
    fn test_json_timing() {
        let output = JsonOutput::new(Path::new("test.ash")).with_timing(
            Duration::from_millis(12),
            Duration::from_millis(45),
            Duration::from_millis(58),
        );

        assert_eq!(output.timing.parse_ms, 12);
        assert_eq!(output.timing.typecheck_ms, 45);
        assert_eq!(output.timing.total_ms, 58);
    }
}
