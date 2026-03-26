//! JSON output format implementation for Ash CLI
//!
//! Implements SPEC-005 compliant JSON output schema for machine-readable
//! diagnostic and verification information.

use serde::Serialize;
use std::path::Path;
use std::time::Duration;

/// Schema version for JSON output format
pub const SCHEMA_VERSION: &str = "1.0";

/// Top-level JSON output structure for check command results
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
    /// List of errors (empty if no errors)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub errors: Vec<JsonDiagnostic>,
    /// List of warnings (empty if no warnings)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub warnings: Vec<JsonDiagnostic>,
    /// Verification results (None if not checked)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<JsonVerification>,
    /// Timing information for various phases
    pub timing: JsonTiming,
}

/// Diagnostic message (error or warning)
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct JsonDiagnostic {
    /// Severity level: "error" or "warning"
    pub severity: &'static str,
    /// Error/warning code (e.g., "E0032", "W001")
    pub code: String,
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

/// Verification results
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct JsonVerification {
    /// Total number of obligations
    pub obligations: usize,
    /// Number of satisfied obligations
    pub satisfied: usize,
    /// List of pending (unsatisfied) obligation names
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub pending: Vec<String>,
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
            errors: Vec::new(),
            warnings: Vec::new(),
            verification: None,
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
        self.errors.push(JsonDiagnostic {
            severity: "error",
            code: code.to_string(),
            message: message.to_string(),
            location: location.unwrap_or_default(),
            context: None,
            help: None,
        });
        self
    }

    /// Add a warning diagnostic
    pub fn with_warning(
        mut self,
        message: &str,
        code: &str,
        location: Option<JsonLocation>,
    ) -> Self {
        self.warnings.push(JsonDiagnostic {
            severity: "warning",
            code: code.to_string(),
            message: message.to_string(),
            location: location.unwrap_or_default(),
            context: None,
            help: None,
        });
        self
    }

    /// Set verification results
    pub fn with_verification(
        mut self,
        obligations: usize,
        satisfied: usize,
        pending: Vec<String>,
    ) -> Self {
        self.verification = Some(JsonVerification {
            obligations,
            satisfied,
            pending,
        });
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
        self.errors.push(JsonDiagnostic {
            severity: "error",
            code: code.to_string(),
            message: error.to_string(),
            location: JsonLocation::default(),
            context: None,
            help: None,
        });
        self
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
        assert!(output.errors.is_empty());
        assert!(output.warnings.is_empty());
    }

    #[test]
    fn test_json_output_with_error() {
        let output = JsonOutput::new(Path::new("test.ash")).with_error(
            "type mismatch",
            "E0032",
            Some(JsonLocation::new("test.ash", 42, 15)),
        );

        assert!(!output.success);
        assert_eq!(output.errors.len(), 1);
        assert_eq!(output.errors[0].severity, "error");
        assert_eq!(output.errors[0].code, "E0032");
        assert_eq!(output.errors[0].message, "type mismatch");
        assert_eq!(output.errors[0].location.line, 42);
        assert_eq!(output.errors[0].location.column, 15);
    }

    #[test]
    fn test_json_output_serialization() {
        let output = JsonOutput::new(Path::new("test.ash"))
            .with_strict(true)
            .with_exit_code(0)
            .with_verification(5, 4, vec!["audit_trail".to_string()]);

        let json = output.to_json().unwrap();
        assert!(json.contains("schema_version"));
        assert!(json.contains("1.0"));
        assert!(json.contains("test.ash"));
        assert!(json.contains("verification"));
        assert!(json.contains("obligations"));
    }

    #[test]
    fn test_json_output_skips_empty_arrays() {
        let output = JsonOutput::new(Path::new("test.ash"));
        let json = output.to_json().unwrap();
        // Empty errors and warnings should be skipped
        assert!(!json.contains("errors"));
        assert!(!json.contains("warnings"));
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
