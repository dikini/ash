//! Error types for the Ash Engine
//!
//! This module defines the unified error type used by the Engine for all
//! operations including parsing, type checking, and execution.

use thiserror::Error;

/// Terminal classification assigned at the sealed production admission boundary.
///
/// This is intentionally narrower than [`EngineError`]: it records only the
/// two pre-execution outcomes that the checked Core/CPS cutover can classify
/// without inspecting an error message.  Callers outside that boundary should
/// use [`EngineError::production_terminal_classification`] to determine
/// whether an error carries one of these classifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionTerminalClassification {
    /// The source did not produce an Engine-issued production admission.
    MissingAdmission,
    /// Purported checked Core/CPS evidence failed sealed-artifact verification.
    InvalidCheckedCoreCps,
}

impl From<EngineError> for ash_interp::ExecError {
    fn from(err: EngineError) -> Self {
        match err {
            // Preserve distinct error types per SPEC-021
            EngineError::Parse(msg) => Self::Parse(msg),
            EngineError::Type(msg) => Self::Type(msg),
            EngineError::Execution(msg) => Self::ExecutionFailed(msg),
            EngineError::Io(io_err) => Self::Io(io_err.to_string()),
            EngineError::CapabilityNotFound(cap) => Self::CapabilityNotAvailable(cap),
            EngineError::Configuration(msg) => {
                Self::ExecutionFailed(format!("configuration error: {msg}"))
            }
            EngineError::ProductionTerminal { message, .. } => Self::ExecutionFailed(message),
        }
    }
}

/// Errors that can occur during engine operations
///
/// This enum consolidates errors from all stages of workflow processing:
/// - Parsing: Syntax errors in source code
/// - Type checking: Type mismatches and inference failures
/// - Execution: Runtime errors during workflow execution
/// - I/O: File and network operations
/// - Capabilities: Missing or unavailable capabilities
///
/// # Example
///
/// ```
/// use ash_engine::EngineError;
///
/// let err = EngineError::Parse("unexpected token".to_string());
/// assert!(matches!(err, EngineError::Parse(_)));
/// ```
#[derive(Debug, Error)]
pub enum EngineError {
    /// Syntax error during parsing
    #[error("parse error: {0}")]
    Parse(String),

    /// Type checking error
    #[error("type error: {0}")]
    Type(String),

    /// Runtime execution error
    #[error("execution error: {0}")]
    Execution(String),

    /// I/O error (file not found, permission denied, etc.)
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Capability not found or not available
    #[error("capability not found: {0}")]
    CapabilityNotFound(String),

    /// Configuration error (invalid or unsupported configuration)
    #[error("configuration error: {0}")]
    Configuration(String),

    /// A typed production-boundary outcome with its detailed diagnostic.
    ///
    /// This variant is emitted only by a sealed production admission or
    /// checked-CPS driver boundary.  The terminal projection must use its
    /// classification rather than attempting to reconstruct it from the
    /// diagnostic text.
    #[error("{message}")]
    ProductionTerminal {
        /// The canonical production-boundary classification.
        classification: ProductionTerminalClassification,
        /// Detailed Engine diagnostic retained for non-terminal consumers.
        message: String,
    },
}

impl EngineError {
    /// Creates a sealed production-boundary error without discarding its
    /// detailed diagnostic.
    #[must_use]
    pub fn production_terminal(
        classification: ProductionTerminalClassification,
        message: impl Into<String>,
    ) -> Self {
        Self::ProductionTerminal {
            classification,
            message: message.into(),
        }
    }

    /// Returns the typed classification when this error came from a sealed
    /// production boundary.
    #[must_use]
    pub const fn production_terminal_classification(
        &self,
    ) -> Option<ProductionTerminalClassification> {
        match self {
            Self::ProductionTerminal { classification, .. } => Some(*classification),
            Self::Parse(_)
            | Self::Type(_)
            | Self::Execution(_)
            | Self::Io(_)
            | Self::CapabilityNotFound(_)
            | Self::Configuration(_) => None,
        }
    }

    /// Returns the classification assigned by a sealed production boundary.
    ///
    /// # Panics
    ///
    /// Panics when called for an error that did not originate at such a
    /// boundary.  General Engine consumers should use
    /// [`Self::production_terminal_classification`] instead.
    #[must_use]
    pub const fn classification(&self) -> ProductionTerminalClassification {
        self.production_terminal_classification().expect(
            "EngineError::classification requires an error from a sealed production boundary",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ============================================================
    // Variant Construction Tests
    // ============================================================

    #[test]
    fn test_parse_error_construction() {
        let err = EngineError::Parse("unexpected '}'".to_string());
        assert!(matches!(err, EngineError::Parse(_)));
    }

    #[test]
    fn test_type_error_construction() {
        let err = EngineError::Type("expected Int, got String".to_string());
        assert!(matches!(err, EngineError::Type(_)));
    }

    #[test]
    fn test_execution_error_construction() {
        let err = EngineError::Execution("division by zero".to_string());
        assert!(matches!(err, EngineError::Execution(_)));
    }

    #[test]
    fn test_io_error_construction() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file.txt");
        let err = EngineError::Io(io_err);
        assert!(matches!(err, EngineError::Io(_)));
    }

    #[test]
    fn test_capability_not_found_construction() {
        let err = EngineError::CapabilityNotFound("fs:read".to_string());
        assert!(matches!(err, EngineError::CapabilityNotFound(_)));
    }

    // ============================================================
    // Display Format Tests
    // ============================================================

    #[test]
    fn test_parse_error_display() {
        let err = EngineError::Parse("unexpected token".to_string());
        let display = format!("{err}");
        assert!(display.contains("parse error"));
        assert!(display.contains("unexpected token"));
    }

    #[test]
    fn test_type_error_display() {
        let err = EngineError::Type("type mismatch".to_string());
        let display = format!("{err}");
        assert!(display.contains("type error"));
        assert!(display.contains("type mismatch"));
    }

    #[test]
    fn test_execution_error_display() {
        let err = EngineError::Execution("runtime failed".to_string());
        let display = format!("{err}");
        assert!(display.contains("execution error"));
        assert!(display.contains("runtime failed"));
    }

    #[test]
    fn test_io_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err = EngineError::Io(io_err);
        let display = format!("{err}");
        assert!(display.contains("io error"));
        assert!(display.contains("access denied"));
    }

    #[test]
    fn test_capability_not_found_display() {
        let err = EngineError::CapabilityNotFound("stdio:print".to_string());
        let display = format!("{err}");
        assert!(display.contains("capability not found"));
        assert!(display.contains("stdio:print"));
    }

    // ============================================================
    // From Trait Tests
    // ============================================================

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::other("test");
        let err: EngineError = io_err.into();
        assert!(matches!(err, EngineError::Io(_)));
    }

    // ============================================================
    // Property-Based Tests
    // ============================================================

    proptest! {
        /// Property: Parse errors preserve their message content
        #[test]
        fn prop_parse_error_preserves_message(message in r"[a-zA-Z0-9_: \-]{0,200}") {
            let err = EngineError::Parse(message.clone());
            if let EngineError::Parse(found) = err {
                prop_assert_eq!(found, message);
            } else {
                prop_assert!(false, "Should be Parse variant");
            }
        }

        /// Property: Type errors preserve their message content
        #[test]
        fn prop_type_error_preserves_message(message in r"[a-zA-Z0-9_:<> \-]{0,200}") {
            let err = EngineError::Type(message.clone());
            if let EngineError::Type(found) = err {
                prop_assert_eq!(found, message);
            } else {
                prop_assert!(false, "Should be Type variant");
            }
        }

        /// Property: Execution errors preserve their message content
        #[test]
        fn prop_execution_error_preserves_message(message in r"[a-zA-Z0-9_: \-]{0,200}") {
            let err = EngineError::Execution(message.clone());
            if let EngineError::Execution(found) = err {
                prop_assert_eq!(found, message);
            } else {
                prop_assert!(false, "Should be Execution variant");
            }
        }

        /// Property: CapabilityNotFound errors preserve the capability name
        #[test]
        fn prop_capability_error_preserves_name(name in "[a-z][a-z0-9_:]{1,50}") {
            let err = EngineError::CapabilityNotFound(name.clone());
            if let EngineError::CapabilityNotFound(found) = err {
                prop_assert_eq!(found, name);
            } else {
                prop_assert!(false, "Should be CapabilityNotFound variant");
            }
        }

        /// Property: Error display contains the error message
        #[test]
        fn prop_error_display_contains_message(message in "[a-zA-Z0-9_ ]{1,100}") {
            let err = EngineError::Parse(message.clone());
            let display = format!("{err}");
            prop_assert!(
                display.contains(&message),
                "Display '{}' should contain message '{}'",
                display,
                message
            );
        }
    }
}
