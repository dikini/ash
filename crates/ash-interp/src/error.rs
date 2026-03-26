//! Error types for the interpreter

use ash_core::{Name, Value};
use thiserror::Error;

use crate::capability_policy::Role;

/// Errors that can occur during expression evaluation
#[derive(Debug, Error, Clone, PartialEq)]
pub enum EvalError {
    #[error("undefined variable: {0}")]
    UndefinedVariable(Name),

    #[error("type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("invalid binary operation: {op} on {left} and {right}")]
    InvalidBinaryOp {
        op: String,
        left: String,
        right: String,
    },

    #[error("invalid unary operation: {op} on {operand}")]
    InvalidUnaryOp { op: String, operand: String },

    #[error("field not found: {field} in {value}")]
    FieldNotFound { field: Name, value: Value },

    #[error("index out of bounds: {index} in list of length {len}")]
    IndexOutOfBounds { index: i64, len: usize },

    #[error("invalid index type: expected integer, got {0}")]
    InvalidIndexType(String),

    #[error("execution failed: {0}")]
    ExecutionFailed(String),

    #[error("unknown function: {0}")]
    UnknownFunction(Name),

    #[error("wrong number of arguments: expected {expected}, got {actual}")]
    WrongArity { expected: usize, actual: usize },

    #[error("division by zero")]
    DivisionByZero,

    #[error("not implemented: {0}")]
    NotImplemented(String),

    #[error("non-exhaustive match: no arm matched value {value}")]
    NonExhaustiveMatch { value: String },
}

/// Errors that can occur during workflow execution
#[derive(Debug, Clone, PartialEq)]
pub enum ExecError {
    Eval(EvalError),
    PatternMatchFailed {
        pattern: String,
        value: Value,
    },
    GuardFailed {
        guard: String,
    },
    CapabilityNotAvailable(Name),
    ActionFailed {
        action: Name,
        reason: String,
    },
    PolicyDenied {
        policy: Name,
    },
    /// The operation is paused until the explicitly named approval role acts.
    RequiresApproval {
        role: Role,
        operation: String,
        capability: Name,
    },
    ExecutionFailed(String),
    ParallelFailed(String),
    ForEachFailed(String),
    TypeMismatch {
        provider: String,
        expected: String,
        actual: String,
        path: Option<String>,
    },
    ValidationFailed(String),
    MailboxFull {
        limit: usize,
    },
    /// Workflow yielded to a proxy role and is awaiting response
    YieldSuspended {
        /// Target role that should handle the yield
        role: String,
        /// The request value sent to the proxy (boxed to reduce error size)
        request: Box<Value>,
        /// Expected response type for validation
        expected_response_type: String,
        /// Correlation ID for matching yield/resume pairs
        correlation_id: String,
        /// Proxy instance address
        proxy_addr: String,
    },
}

impl std::error::Error for ExecError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Eval(e) => Some(e),
            _ => None,
        }
    }
}

impl From<EvalError> for ExecError {
    fn from(err: EvalError) -> Self {
        Self::Eval(err)
    }
}

impl From<ash_core::MailboxOverflowError> for ExecError {
    fn from(err: ash_core::MailboxOverflowError) -> Self {
        Self::MailboxFull { limit: err.limit() }
    }
}

impl std::fmt::Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eval(e) => write!(f, "evaluation error: {e}"),
            Self::PatternMatchFailed { pattern, value } => {
                write!(f, "pattern match failed: {pattern} does not match {value}")
            }
            Self::GuardFailed { guard } => write!(f, "guard evaluation failed: {guard}"),
            Self::CapabilityNotAvailable(name) => write!(f, "capability not available: {name}"),
            Self::ActionFailed { action, reason } => {
                write!(f, "action execution failed: {action} - {reason}")
            }
            Self::PolicyDenied { policy } => write!(f, "policy denied: {policy}"),
            Self::RequiresApproval {
                role,
                operation,
                capability,
            } => write!(
                f,
                "approval required: role '{}' must approve {} on {}",
                role.as_ref(),
                operation,
                capability
            ),
            Self::ExecutionFailed(msg) => write!(f, "workflow execution failed: {msg}"),
            Self::ParallelFailed(msg) => write!(f, "parallel execution failed: {msg}"),
            Self::ForEachFailed(msg) => write!(f, "for each iteration failed: {msg}"),
            Self::TypeMismatch {
                provider,
                expected,
                actual,
                path,
            } => {
                if let Some(p) = path {
                    write!(
                        f,
                        "type mismatch in provider '{provider}' at {p}: expected {expected}, got {actual}"
                    )
                } else {
                    write!(
                        f,
                        "type mismatch in provider '{provider}': expected {expected}, got {actual}"
                    )
                }
            }
            Self::ValidationFailed(msg) => write!(f, "validation failed: {msg}"),
            Self::MailboxFull { limit } => {
                write!(f, "mailbox full: limit of {limit} entries exceeded")
            }
            Self::YieldSuspended {
                role,
                request,
                expected_response_type,
                correlation_id,
                proxy_addr,
            } => write!(
                f,
                "workflow yielded to role '{}' with request {:?} (expected response: {}) at proxy {} with correlation_id={}",
                role, request, expected_response_type, proxy_addr, correlation_id
            ),
        }
    }
}

impl ExecError {
    /// Create a type mismatch error for a provider
    pub fn type_mismatch(
        provider: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::TypeMismatch {
            provider: provider.into(),
            expected: expected.into(),
            actual: actual.into(),
            path: None,
        }
    }

    /// Add a path to the type mismatch error
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        if let Self::TypeMismatch { path: p, .. } = &mut self {
            *p = Some(path.into());
        }
        self
    }
}

/// Errors that can occur during pattern matching
#[derive(Debug, Error, Clone, PartialEq)]
pub enum PatternError {
    #[error("pattern match failed: expected {expected}, got {actual}")]
    MatchFailed { expected: String, actual: String },

    #[error("list length mismatch: expected at least {expected}, got {actual}")]
    ListLengthMismatch { expected: usize, actual: usize },

    #[error("record field missing: {0}")]
    FieldMissing(Name),

    #[error("cannot match against non-record value: {0}")]
    NotARecord(Value),
}

/// Validation errors for provider value validation
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ValidationError {
    #[error("invalid value: {0}")]
    InvalidValue(String),

    #[error("value out of range: {0}")]
    OutOfRange(String),

    #[error("value format error: {0}")]
    FormatError(String),
}

impl From<ValidationError> for ExecError {
    fn from(err: ValidationError) -> Self {
        ExecError::ValidationFailed(err.to_string())
    }
}

/// Result type for evaluation operations
pub type EvalResult<T> = Result<T, EvalError>;

/// Result type for execution operations
pub type ExecResult<T> = Result<T, ExecError>;

/// Result type for pattern matching operations
pub type PatternResult<T> = Result<T, PatternError>;

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;
