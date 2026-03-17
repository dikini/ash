//! Error types for the interpreter

use ash_core::{Name, Value};
use thiserror::Error;

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
}

/// Errors that can occur during workflow execution
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ExecError {
    #[error("evaluation error: {0}")]
    Eval(#[from] EvalError),

    #[error("pattern match failed: {pattern} does not match {value}")]
    PatternMatchFailed { pattern: String, value: Value },

    #[error("guard evaluation failed: {guard}")]
    GuardFailed { guard: String },

    #[error("capability not available: {0}")]
    CapabilityNotAvailable(Name),

    #[error("action execution failed: {action} - {reason}")]
    ActionFailed { action: Name, reason: String },

    #[error("policy denied: {policy}")]
    PolicyDenied { policy: Name },

    #[error("workflow execution failed: {0}")]
    ExecutionFailed(String),

    #[error("parallel execution failed: {0}")]
    ParallelFailed(String),

    #[error("for each iteration failed: {0}")]
    ForEachFailed(String),
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

/// Result type for evaluation operations
pub type EvalResult<T> = Result<T, EvalError>;

/// Result type for execution operations
pub type ExecResult<T> = Result<T, ExecError>;

/// Result type for pattern matching operations
pub type PatternResult<T> = Result<T, PatternError>;
