//! Error types for the Ash Engine
//!
//! This module defines the unified error type used by the Engine for all
//! operations including parsing, type checking, and execution.

use thiserror::Error;

impl From<EngineError> for ash_interp::ExecError {
    fn from(err: EngineError) -> Self {
        match err {
            EngineError::Parse(msg) => Self::ExecutionFailed(format!("parse error: {msg}")),
            EngineError::Type(msg) => Self::ExecutionFailed(format!("type error: {msg}")),
            EngineError::Execution(msg) => Self::ExecutionFailed(msg),
            EngineError::Io(io_err) => Self::ExecutionFailed(format!("io error: {io_err}")),
            EngineError::CapabilityNotFound(cap) => Self::CapabilityNotAvailable(cap),
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
        let display = format!("{}", err);
        assert!(display.contains("parse error"));
        assert!(display.contains("unexpected token"));
    }

    #[test]
    fn test_type_error_display() {
        let err = EngineError::Type("type mismatch".to_string());
        let display = format!("{}", err);
        assert!(display.contains("type error"));
        assert!(display.contains("type mismatch"));
    }

    #[test]
    fn test_execution_error_display() {
        let err = EngineError::Execution("runtime failed".to_string());
        let display = format!("{}", err);
        assert!(display.contains("execution error"));
        assert!(display.contains("runtime failed"));
    }

    #[test]
    fn test_io_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err = EngineError::Io(io_err);
        let display = format!("{}", err);
        assert!(display.contains("io error"));
        assert!(display.contains("access denied"));
    }

    #[test]
    fn test_capability_not_found_display() {
        let err = EngineError::CapabilityNotFound("stdio:print".to_string());
        let display = format!("{}", err);
        assert!(display.contains("capability not found"));
        assert!(display.contains("stdio:print"));
    }

    // ============================================================
    // From Trait Tests
    // ============================================================

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "test");
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
            let display = format!("{}", err);
            prop_assert!(
                display.contains(&message),
                "Display '{}' should contain message '{}'",
                display,
                message
            );
        }
    }
}
