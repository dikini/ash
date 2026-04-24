//! Error types for type checking
//!
//! Defines errors that can occur during type checking of expressions,
//! including constructor checking errors.

use ash_parser::token::Span;
use thiserror::Error;

/// Error type for constructor checking
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ConstructorError {
    /// Unknown constructor name
    #[error("Unknown constructor: {0}")]
    UnknownConstructor(String, Span),

    /// Missing required field in constructor
    #[error("Missing field '{field}' in constructor '{constructor}'")]
    MissingField {
        /// Name of the constructor
        constructor: String,
        /// Name of the missing field
        field: String,
        /// Source span
        span: Span,
    },

    /// Unknown field provided to constructor
    #[error("Unknown field '{field}' in constructor '{constructor}'")]
    UnknownField {
        /// Name of the constructor
        constructor: String,
        /// Name of the unknown field
        field: String,
        /// Source span
        span: Span,
    },

    /// Type mismatch in field
    #[error(
        "Type mismatch in field '{field}' of constructor '{constructor}': expected {expected}, got {actual}"
    )]
    FieldTypeMismatch {
        /// Name of the constructor
        constructor: String,
        /// Source-facing field label or tuple slot label
        field: String,
        /// Expected type
        expected: String,
        /// Actual type
        actual: String,
        /// Source span
        span: Span,
    },

    /// Type mismatch in positional tuple payload item
    #[error(
        "Type mismatch in positional item {position} of constructor '{constructor}': expected {expected}, got {actual}"
    )]
    TupleFieldTypeMismatch {
        /// Name of the constructor
        constructor: String,
        /// Zero-based tuple position
        position: usize,
        /// Expected type
        expected: String,
        /// Actual type
        actual: String,
        /// Source span
        span: Span,
    },

    /// Wrong number of positional tuple payload items for a constructor.
    #[error(
        "Tuple constructor arity mismatch for '{constructor}': expected {expected}, got {actual}"
    )]
    TupleArityMismatch {
        /// Name of the constructor
        constructor: String,
        /// Expected number of positional items
        expected: usize,
        /// Actual number of positional items
        actual: usize,
        /// Source span
        span: Span,
    },

    /// Match expression does not cover all variants of the scrutinee enum
    #[error("non-exhaustive match on type '{scrutinee_type}': missing {missing}")]
    NonExhaustiveMatch {
        /// Enum (or ADT) type being matched
        scrutinee_type: String,
        /// Human-readable list of missing cases
        missing: String,
        /// Source span
        span: Span,
    },

    /// Unbound variable - variable not found in environment
    #[error("unbound variable: {name}")]
    UnboundVariable {
        /// Name of the variable
        name: String,
        /// Source span
        span: Span,
    },

    /// Type is not iterable (used in for loops)
    #[error("type {ty} is not iterable")]
    NotIterable {
        /// The type that cannot be iterated
        ty: crate::types::Type,
        /// Source span
        span: Span,
    },

    /// Field access requested a field missing from a record-typed base.
    #[error("missing field '{field}' in record")]
    MissingRecordField {
        /// Missing field name.
        field: String,
        /// Source span.
        span: Span,
    },

    /// Field access was applied to a non-record value.
    #[error("cannot access field '{field}' on non-record type {actual}")]
    NotARecord {
        /// Field being requested.
        field: String,
        /// Actual base type encountered.
        actual: crate::types::Type,
        /// Source span.
        span: Span,
    },

    /// Unsupported expression type
    #[error("unsupported expression: {kind}")]
    UnsupportedExpression {
        /// Kind of expression that is unsupported
        kind: String,
        /// Source span
        span: Span,
    },

    /// Unknown type annotation in FnDef parameter or return type
    #[error("unknown type annotation `{name}` in {context}")]
    UnknownTypeAnnotation {
        /// The unresolvable type name
        name: String,
        /// Where the annotation appeared, e.g. "parameter `x`" or "return type"
        context: String,
        /// Source span
        span: Span,
    },

    /// Invalid canonical interface method call
    #[error("invalid interface method call {interface}::{method}: {reason}")]
    InvalidInterfaceMethodCall {
        /// Interface name
        interface: String,
        /// Method name
        method: String,
        /// Human-readable failure reason
        reason: String,
        /// Source span
        span: Span,
    },
}

/// Error type for type environment operations
#[derive(Debug, Clone, PartialEq, Error)]
pub enum TypeEnvError {
    /// Type already defined
    #[error("Type '{0}' is already defined")]
    DuplicateType(String, Span),

    /// Type not found
    #[error("Type '{0}' not found")]
    TypeNotFound(String, Span),

    /// Invalid type definition
    #[error("Invalid type definition: {0}")]
    InvalidDefinition(String, Span),

    /// Interface already defined
    #[error("Interface '{0}' is already defined")]
    DuplicateInterface(String, Span),

    /// Interface not found
    #[error("Interface '{0}' not found")]
    MissingInterface(String, Span),

    /// Duplicate impl for the same interface and full interface application
    #[error("Impl for interface '{interface}' and type '{ty}' is already defined")]
    DuplicateImpl {
        /// Interface name
        interface: String,
        /// Full interface application
        ty: String,
        /// Source span
        span: Span,
    },

    /// Impl not found for a canonical interface method call
    #[error("No impl found for interface '{interface}' and type '{ty}'")]
    MissingImpl {
        /// Interface name
        interface: String,
        /// Full interface application
        ty: String,
        /// Source span
        span: Span,
    },

    /// Interface method not found
    #[error("Interface '{interface}' does not define method '{method}'")]
    MissingInterfaceMethod {
        /// Interface name
        interface: String,
        /// Method name
        method: String,
        /// Source span
        span: Span,
    },

    /// Overlapping impls for an interface
    #[error("overlapping impls for interface '{interface}'")]
    OverlappingImpls {
        /// Interface name
        interface: String,
        /// Source span
        span: Span,
    },

    /// Recursive interface bound exceeded depth limit
    #[error("recursive interface bound exceeded depth limit")]
    RecursiveBound {
        /// Human-readable failure reason
        message: String,
        /// Source span
        span: Span,
    },

    /// Missing associated type in impl
    #[error("missing associated type '{name}' in impl for interface '{interface}'")]
    MissingAssociatedType {
        /// Interface name
        interface: String,
        /// Associated type name
        name: String,
        /// Source span
        span: Span,
    },

    /// Mismatched projection interface
    #[error("mismatched projection interface: expected '{expected}', found '{found}'")]
    MismatchedProjectionInterface {
        /// Expected interface name
        expected: String,
        /// Found interface name
        found: String,
        /// Source span
        span: Span,
    },

    /// Ambiguous associated type
    #[error("ambiguous associated type '{name}'")]
    AmbiguousAssociatedType {
        /// Associated type name
        name: String,
        /// Source span
        span: Span,
    },
}

/// Error type for exhaustiveness checking
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ExhaustivenessError {
    /// Non-exhaustive pattern match
    #[error("non-exhaustive pattern match for type '{scrutinee_type}'")]
    NonExhaustiveMatch {
        /// Type being matched
        scrutinee_type: String,
        /// Missing patterns
        missing_patterns: Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unknown_constructor_error() {
        let err = ConstructorError::UnknownConstructor("Foo".to_string(), Span::default());
        let msg = format!("{err}");
        assert!(msg.contains("Unknown constructor"));
        assert!(msg.contains("Foo"));
    }

    #[test]
    fn test_missing_field_error() {
        let err = ConstructorError::MissingField {
            constructor: "Some".to_string(),
            field: "value".to_string(),
            span: Span::default(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("Missing field"));
        assert!(msg.contains("Some"));
        assert!(msg.contains("value"));
    }

    #[test]
    fn test_unknown_field_error() {
        let err = ConstructorError::UnknownField {
            constructor: "Point".to_string(),
            field: "z".to_string(),
            span: Span::default(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("Unknown field"));
        assert!(msg.contains("Point"));
        assert!(msg.contains("z"));
    }

    #[test]
    fn test_field_type_mismatch_error() {
        let err = ConstructorError::FieldTypeMismatch {
            constructor: "Some".to_string(),
            field: "value".to_string(),
            expected: "Int".to_string(),
            actual: "String".to_string(),
            span: Span::default(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("Type mismatch"));
        assert!(msg.contains("Some"));
        assert!(msg.contains("value"));
        assert!(msg.contains("Int"));
        assert!(msg.contains("String"));
    }

    #[test]
    fn test_tuple_field_type_mismatch_error() {
        let err = ConstructorError::TupleFieldTypeMismatch {
            constructor: "RuntimeError".to_string(),
            position: 0,
            expected: "Int".to_string(),
            actual: "String".to_string(),
            span: Span::default(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("positional item 0"));
        assert!(msg.contains("RuntimeError"));
        assert!(msg.contains("Int"));
        assert!(msg.contains("String"));
    }

    #[test]
    fn test_duplicate_type_error() {
        let err = TypeEnvError::DuplicateType("Option".to_string(), Span::default());
        let msg = format!("{err}");
        assert!(msg.contains("already defined"));
        assert!(msg.contains("Option"));
    }

    #[test]
    fn test_type_not_found_error() {
        let err = TypeEnvError::TypeNotFound("Unknown".to_string(), Span::default());
        let msg = format!("{err}");
        assert!(msg.contains("not found"));
        assert!(msg.contains("Unknown"));
    }
}
