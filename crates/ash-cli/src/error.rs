//! Error types for the Ash CLI with SPEC-005 compliant exit codes.
//!
//! This module defines error types that map to specific exit codes as defined
//! in SPEC-005 Section 4.

use std::process::ExitCode;
use thiserror::Error;

/// CLI error types with exit code mapping per SPEC-005.
#[derive(Debug, Error)]
pub enum CliError {
    /// Parse error - exit code 2
    #[error("parse error: {message}")]
    ParseError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Type error - exit code 3
    #[error("type error: {message}")]
    TypeError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Verification failure - exit code 4
    #[error("verification failed: {message}")]
    VerificationError {
        message: String,
        details: Vec<String>,
    },

    /// Runtime error - exit code 5
    #[error("runtime error: {message}")]
    RuntimeError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// I/O error - exit code 6
    #[error("I/O error: {message}")]
    IoError {
        message: String,
        path: Option<std::path::PathBuf>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Timeout error - exit code 7
    #[error("timeout after {seconds}s")]
    Timeout { seconds: u64 },

    /// Unknown command - exit code 127
    #[error("unknown command: {name}")]
    UnknownCommand { name: String },

    /// General error - exit code 1
    #[error("{message}")]
    General { message: String },
}

impl CliError {
    /// Returns the exit code for this error type per SPEC-005.
    ///
    /// Exit codes:
    /// - 0 = Success
    /// - 1 = General error
    /// - 2 = Parse error
    /// - 3 = Type error
    /// - 4 = Verification failure
    /// - 5 = Runtime error
    /// - 6 = I/O error
    /// - 7 = Timeout
    /// - 127 = Command not found
    pub fn exit_code(&self) -> ExitCode {
        match self {
            CliError::ParseError { .. } => ExitCode::from(2),
            CliError::TypeError { .. } => ExitCode::from(3),
            CliError::VerificationError { .. } => ExitCode::from(4),
            CliError::RuntimeError { .. } => ExitCode::from(5),
            CliError::IoError { .. } => ExitCode::from(6),
            CliError::Timeout { .. } => ExitCode::from(7),
            CliError::UnknownCommand { .. } => ExitCode::from(127),
            CliError::General { .. } => ExitCode::from(1),
        }
    }

    /// Create a parse error from an anyhow error
    pub fn parse<E: std::error::Error + Send + Sync + 'static>(
        message: impl Into<String>,
        source: E,
    ) -> Self {
        CliError::ParseError {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a type error from an anyhow error
    pub fn type_error<E: std::error::Error + Send + Sync + 'static>(
        message: impl Into<String>,
        source: E,
    ) -> Self {
        CliError::TypeError {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a runtime error from an anyhow error
    pub fn runtime<E: std::error::Error + Send + Sync + 'static>(
        message: impl Into<String>,
        source: E,
    ) -> Self {
        CliError::RuntimeError {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create an I/O error
    pub fn io<E: std::error::Error + Send + Sync + 'static>(
        message: impl Into<String>,
        path: Option<std::path::PathBuf>,
        source: E,
    ) -> Self {
        CliError::IoError {
            message: message.into(),
            path,
            source: Some(Box::new(source)),
        }
    }

    /// Create a verification error
    pub fn verification(message: impl Into<String>, details: Vec<String>) -> Self {
        CliError::VerificationError {
            message: message.into(),
            details,
        }
    }

    /// Create a timeout error
    pub fn timeout(seconds: u64) -> Self {
        CliError::Timeout { seconds }
    }

    /// Create an unknown command error
    pub fn unknown_command(name: impl Into<String>) -> Self {
        CliError::UnknownCommand { name: name.into() }
    }

    /// Create a general error
    pub fn general(message: impl Into<String>) -> Self {
        CliError::General {
            message: message.into(),
        }
    }
}

impl From<anyhow::Error> for CliError {
    fn from(err: anyhow::Error) -> Self {
        // Try to classify the error based on its message content
        let msg = err.to_string().to_lowercase();

        if msg.contains("parse error") {
            CliError::ParseError {
                message: err.to_string(),
                source: None,
            }
        } else if msg.contains("type error") {
            CliError::TypeError {
                message: err.to_string(),
                source: None,
            }
        } else if msg.contains("io error") || msg.contains("i/o error") {
            CliError::IoError {
                message: err.to_string(),
                path: None,
                source: None,
            }
        } else if msg.contains("verification") || msg.contains("capability") {
            CliError::VerificationError {
                message: err.to_string(),
                details: vec![],
            }
        } else if msg.contains("runtime") {
            CliError::RuntimeError {
                message: err.to_string(),
                source: None,
            }
        } else if msg.contains("timeout") {
            // Try to extract seconds from message
            CliError::Timeout { seconds: 0 }
        } else {
            CliError::General {
                message: err.to_string(),
            }
        }
    }
}

/// Result type alias for CLI operations
pub type CliResult<T> = Result<T, CliError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_codes() {
        assert_eq!(
            CliError::parse("test", std::io::Error::other("test")).exit_code(),
            ExitCode::from(2)
        );
        assert_eq!(
            CliError::type_error("test", std::io::Error::other("test")).exit_code(),
            ExitCode::from(3)
        );
        assert_eq!(
            CliError::verification("test", vec![]).exit_code(),
            ExitCode::from(4)
        );
        assert_eq!(
            CliError::runtime("test", std::io::Error::other("test")).exit_code(),
            ExitCode::from(5)
        );
        assert_eq!(
            CliError::io("test", None, std::io::Error::other("test")).exit_code(),
            ExitCode::from(6)
        );
        assert_eq!(CliError::timeout(30).exit_code(), ExitCode::from(7));
        assert_eq!(
            CliError::unknown_command("foo").exit_code(),
            ExitCode::from(127)
        );
        assert_eq!(CliError::general("test").exit_code(), ExitCode::from(1));
    }

    #[test]
    fn test_from_anyhow_parse() {
        let anyhow_err = anyhow::anyhow!("parse error: unexpected token");
        let cli_err: CliError = anyhow_err.into();
        assert_eq!(cli_err.exit_code(), ExitCode::from(2));
    }

    #[test]
    fn test_from_anyhow_type() {
        let anyhow_err = anyhow::anyhow!("type error: expected int, got string");
        let cli_err: CliError = anyhow_err.into();
        assert_eq!(cli_err.exit_code(), ExitCode::from(3));
    }
}
