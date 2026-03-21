//! Error types for type checking
//!
//! Defines errors that can occur during type checking of expressions,
//! including constructor checking errors.

use thiserror::Error;

/// Error type for constructor checking
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ConstructorError {
    /// Unknown constructor name
    #[error("Unknown constructor: {0}")]
    UnknownConstructor(String),

    /// Missing required field in constructor
    #[error("Missing field '{field}' in constructor '{constructor}'")]
    MissingField {
        /// Name of the constructor
        constructor: String,
        /// Name of the missing field
        field: String,
    },

    /// Unknown field provided to constructor
    #[error("Unknown field '{field}' in constructor '{constructor}'")]
    UnknownField {
        /// Name of the constructor
        constructor: String,
        /// Name of the unknown field
        field: String,
    },

    /// Type mismatch in field
    #[error(
        "Type mismatch in field '{field}' of constructor '{constructor}': expected {expected}, got {actual}"
    )]
    FieldTypeMismatch {
        /// Name of the constructor
        constructor: String,
        /// Name of the field
        field: String,
        /// Expected type
        expected: String,
        /// Actual type
        actual: String,
    },

    /// Match expression does not cover all variants of the scrutinee enum
    #[error("non-exhaustive match on type '{scrutinee_type}': missing {missing}")]
    NonExhaustiveMatch {
        /// Enum (or ADT) type being matched
        scrutinee_type: String,
        /// Human-readable list of missing cases
        missing: String,
    },
}

/// Error type for type environment operations
#[derive(Debug, Clone, PartialEq, Error)]
pub enum TypeEnvError {
    /// Type already defined
    #[error("Type '{0}' is already defined")]
    DuplicateType(String),

    /// Type not found
    #[error("Type '{0}' not found")]
    TypeNotFound(String),

    /// Invalid type definition
    #[error("Invalid type definition: {0}")]
    InvalidDefinition(String),
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
        let err = ConstructorError::UnknownConstructor("Foo".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("Unknown constructor"));
        assert!(msg.contains("Foo"));
    }

    #[test]
    fn test_missing_field_error() {
        let err = ConstructorError::MissingField {
            constructor: "Some".to_string(),
            field: "value".to_string(),
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
        };
        let msg = format!("{err}");
        assert!(msg.contains("Type mismatch"));
        assert!(msg.contains("Some"));
        assert!(msg.contains("value"));
        assert!(msg.contains("Int"));
        assert!(msg.contains("String"));
    }

    #[test]
    fn test_duplicate_type_error() {
        let err = TypeEnvError::DuplicateType("Option".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("already defined"));
        assert!(msg.contains("Option"));
    }

    #[test]
    fn test_type_not_found_error() {
        let err = TypeEnvError::TypeNotFound("Unknown".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("not found"));
        assert!(msg.contains("Unknown"));
    }
}
