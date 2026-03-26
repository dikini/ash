//! Capability constraint validation for Ash workflows (TASK-263)
//!
//! This module provides type checking for capability constraints against
//! capability definitions per SPEC-017 and SPEC-024. It validates that:
//! 1. Capability declarations reference known capabilities
//! 2. Constraint fields are valid for the capability type
//! 3. Constraint values match the expected types
//!
//! # Example
//!
//! ```
//! use std::collections::HashMap;
//! use ash_typeck::constraint_checking::ConstraintChecker;
//! use ash_parser::surface::{CapabilityDef, CapabilityDecl, EffectType};
//! use ash_parser::token::Span;
//!
//! // Create capability definitions
//! let mut cap_defs = HashMap::new();
//! cap_defs.insert("file".to_string(), CapabilityDef {
//!     visibility: ash_parser::surface::Visibility::Public,
//!     name: "file".into(),
//!     effect: EffectType::Operational,
//!     params: vec![],
//!     return_type: None,
//!     constraints: vec![],
//!     span: Span::default(),
//! });
//!
//! let checker = ConstraintChecker::new(&cap_defs);
//! ```

use ash_parser::surface::{CapabilityDecl, CapabilityDef, ConstraintBlock, ConstraintValue};
use ash_parser::token::Span;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Error type for constraint checking
#[derive(Debug, Clone, Error, PartialEq)]
pub enum ConstraintCheckError {
    /// Unknown capability referenced
    #[error("Unknown capability: '{name}' does not exist")]
    UnknownCapability { name: String, span: Span },

    /// Invalid constraint field for capability
    #[error("Invalid constraint field '{field}' for capability '{capability}'")]
    InvalidConstraintField {
        capability: String,
        field: String,
        span: Span,
    },

    /// Constraint value type mismatch
    #[error("Constraint type mismatch for field '{field}': expected {expected}, found {found}")]
    ConstraintTypeMismatch {
        field: String,
        expected: String,
        found: String,
    },
}

/// Result type for constraint checking
pub type ConstraintCheckResult<T> = Result<T, ConstraintCheckError>;

/// Type of constraint value expected for a field
#[derive(Debug, Clone, PartialEq)]
pub enum ExpectedConstraintType {
    /// Boolean value
    Bool,
    /// Integer value
    Int,
    /// String value
    String,
    /// Array of values
    Array,
    /// Object with key-value pairs
    Object,
    /// Multiple possible types
    AnyOf(Vec<ExpectedConstraintType>),
}

impl ExpectedConstraintType {
    /// Check if a constraint value matches the expected type
    pub fn matches(&self, value: &ConstraintValue) -> bool {
        match (self, value) {
            (ExpectedConstraintType::Bool, ConstraintValue::Bool(_)) => true,
            (ExpectedConstraintType::Int, ConstraintValue::Int(_)) => true,
            (ExpectedConstraintType::String, ConstraintValue::String(_)) => true,
            (ExpectedConstraintType::Array, ConstraintValue::Array(_)) => true,
            (ExpectedConstraintType::Object, ConstraintValue::Object(_)) => true,
            (ExpectedConstraintType::AnyOf(types), value) => types.iter().any(|t| t.matches(value)),
            _ => false,
        }
    }

    /// Get a human-readable description of the expected type
    pub fn description(&self) -> String {
        match self {
            ExpectedConstraintType::Bool => "bool".to_string(),
            ExpectedConstraintType::Int => "int".to_string(),
            ExpectedConstraintType::String => "string".to_string(),
            ExpectedConstraintType::Array => "array".to_string(),
            ExpectedConstraintType::Object => "object".to_string(),
            ExpectedConstraintType::AnyOf(types) => {
                let descriptions: Vec<_> = types.iter().map(|t| t.description()).collect();
                format!("one of: {}", descriptions.join(", "))
            }
        }
    }
}

/// Get the valid constraint fields for a capability type
fn get_valid_fields_for_capability(cap_name: &str) -> HashSet<&'static str> {
    let mut fields = HashSet::new();

    match cap_name {
        "file" => {
            fields.insert("paths");
            fields.insert("read");
            fields.insert("write");
        }
        "network" => {
            fields.insert("hosts");
            fields.insert("ports");
            fields.insert("protocols");
        }
        "process" => {
            fields.insert("spawn");
            fields.insert("kill");
            fields.insert("signal");
        }
        _ => {
            // Unknown capability - no valid fields
        }
    }

    fields
}

/// Get the expected type for a constraint field of a capability
fn get_expected_type_for_field(cap_name: &str, field: &str) -> Option<ExpectedConstraintType> {
    match cap_name {
        "file" => match field {
            "paths" => Some(ExpectedConstraintType::Array),
            "read" => Some(ExpectedConstraintType::Bool),
            "write" => Some(ExpectedConstraintType::Bool),
            _ => None,
        },
        "network" => match field {
            "hosts" => Some(ExpectedConstraintType::Array),
            "ports" => Some(ExpectedConstraintType::Array),
            "protocols" => Some(ExpectedConstraintType::Array),
            _ => None,
        },
        "process" => match field {
            "spawn" => Some(ExpectedConstraintType::Bool),
            "kill" => Some(ExpectedConstraintType::Bool),
            "signal" => Some(ExpectedConstraintType::Array),
            _ => None,
        },
        _ => None,
    }
}

/// Checker for capability constraints
///
/// Validates capability declarations against their definitions,
/// ensuring constraints match the capability schema.
#[derive(Debug, Clone)]
pub struct ConstraintChecker<'a> {
    /// Map of capability names to their definitions
    capability_defs: &'a HashMap<String, CapabilityDef>,
}

impl<'a> ConstraintChecker<'a> {
    /// Create a new constraint checker with the given capability definitions
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use ash_typeck::constraint_checking::ConstraintChecker;
    /// use ash_parser::surface::CapabilityDef;
    ///
    /// let cap_defs: HashMap<String, CapabilityDef> = HashMap::new();
    /// let checker = ConstraintChecker::new(&cap_defs);
    /// ```
    pub fn new(capability_defs: &'a HashMap<String, CapabilityDef>) -> Self {
        Self { capability_defs }
    }

    /// Validate a capability declaration against its definition
    ///
    /// Checks that:
    /// 1. The capability exists
    /// 2. All constraint fields are valid for this capability type
    /// 3. All constraint values have the correct types
    ///
    /// # Arguments
    ///
    /// * `decl` - The capability declaration to validate
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the declaration is valid, or a `ConstraintCheckError`
    /// if validation fails.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use ash_typeck::constraint_checking::ConstraintChecker;
    /// use ash_parser::surface::{CapabilityDecl, ConstraintBlock, ConstraintField, ConstraintValue};
    /// use ash_parser::token::Span;
    ///
    /// let cap_defs: HashMap<String, ash_parser::surface::CapabilityDef> = HashMap::new();
    /// let checker = ConstraintChecker::new(&cap_defs);
    ///
    /// let decl = CapabilityDecl {
    ///     capability: "file".into(),
    ///     constraints: None,
    ///     span: Span::default(),
    /// };
    /// ```
    pub fn check_capability_decl(&self, decl: &CapabilityDecl) -> ConstraintCheckResult<()> {
        let cap_name = decl.capability.as_ref();

        // Look up the capability definition
        let def = self.lookup_capability(cap_name).ok_or_else(|| {
            ConstraintCheckError::UnknownCapability {
                name: cap_name.to_string(),
                span: decl.span,
            }
        })?;

        // If there are constraints, validate them
        if let Some(constraints) = &decl.constraints {
            self.check_constraints(cap_name, constraints, def)?;
        }

        Ok(())
    }

    /// Check constraints against a capability definition
    fn check_constraints(
        &self,
        cap_name: &str,
        constraints: &ConstraintBlock,
        _def: &CapabilityDef,
    ) -> ConstraintCheckResult<()> {
        for field in &constraints.fields {
            let field_name = field.name.as_ref();

            // Check if the field is valid for this capability type
            if !self.is_valid_constraint_field(cap_name, field_name) {
                return Err(ConstraintCheckError::InvalidConstraintField {
                    capability: cap_name.to_string(),
                    field: field_name.to_string(),
                    span: field.span,
                });
            }

            // Check the constraint value type
            self.check_constraint_value(cap_name, field_name, &field.value)?;
        }

        Ok(())
    }

    /// Check if a field name is valid for a capability type
    fn is_valid_constraint_field(&self, cap_name: &str, field: &str) -> bool {
        let valid_fields = get_valid_fields_for_capability(cap_name);
        valid_fields.contains(field)
    }

    /// Check that a constraint value has the expected type for its field
    #[allow(clippy::collapsible_if)]
    fn check_constraint_value(
        &self,
        cap_name: &str,
        field_name: &str,
        value: &ConstraintValue,
    ) -> ConstraintCheckResult<()> {
        if let Some(expected_type) = get_expected_type_for_field(cap_name, field_name) {
            if !expected_type.matches(value) {
                return Err(ConstraintCheckError::ConstraintTypeMismatch {
                    field: field_name.to_string(),
                    expected: expected_type.description(),
                    found: get_constraint_value_type(value),
                });
            }
        }

        // For arrays, check that all items have valid types for the field
        if let ConstraintValue::Array(arr) = value {
            for (idx, item) in arr.iter().enumerate() {
                // Check that the array item is a valid primitive type
                if !is_valid_array_item(item) {
                    return Err(ConstraintCheckError::ConstraintTypeMismatch {
                        field: format!("{}[{}]", field_name, idx),
                        expected: "string or int".to_string(),
                        found: get_constraint_value_type(item),
                    });
                }
            }
        }

        Ok(())
    }

    /// Look up a capability definition by name
    fn lookup_capability(&self, name: &str) -> Option<&CapabilityDef> {
        self.capability_defs.get(name)
    }

    /// Check if a capability exists
    pub fn has_capability(&self, name: &str) -> bool {
        self.capability_defs.contains_key(name)
    }

    /// Get all available capability names
    pub fn available_capabilities(&self) -> impl Iterator<Item = &String> {
        self.capability_defs.keys()
    }
}

/// Get a string description of a constraint value's type
fn get_constraint_value_type(value: &ConstraintValue) -> String {
    match value {
        ConstraintValue::Bool(_) => "bool".to_string(),
        ConstraintValue::Int(_) => "int".to_string(),
        ConstraintValue::String(_) => "string".to_string(),
        ConstraintValue::Array(arr) => {
            if arr.is_empty() {
                "empty array".to_string()
            } else {
                format!("array of {}", get_constraint_value_type(&arr[0]))
            }
        }
        ConstraintValue::Object(_) => "object".to_string(),
    }
}

/// Check if a constraint value is a valid array item (string or int)
fn is_valid_array_item(value: &ConstraintValue) -> bool {
    matches!(value, ConstraintValue::String(_) | ConstraintValue::Int(_))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_parser::surface::{ConstraintField, Visibility};

    fn test_span() -> Span {
        Span::new(0, 0, 1, 1)
    }

    fn create_capability_def(name: &str) -> CapabilityDef {
        CapabilityDef {
            visibility: Visibility::Public,
            name: name.into(),
            effect: ash_parser::surface::EffectType::Operational,
            params: vec![],
            return_type: None,
            constraints: vec![],
            span: test_span(),
        }
    }

    fn create_capability_decl(name: &str, constraints: Option<ConstraintBlock>) -> CapabilityDecl {
        CapabilityDecl {
            capability: name.into(),
            constraints,
            span: test_span(),
        }
    }

    fn create_constraint_block(fields: Vec<(&str, ConstraintValue)>) -> ConstraintBlock {
        let fields = fields
            .into_iter()
            .map(|(name, value)| ConstraintField {
                name: name.into(),
                value,
                span: test_span(),
            })
            .collect();

        ConstraintBlock {
            fields,
            span: test_span(),
        }
    }

    #[test]
    fn test_valid_file_constraints() {
        let mut cap_defs = HashMap::new();
        cap_defs.insert("file".to_string(), create_capability_def("file"));

        let checker = ConstraintChecker::new(&cap_defs);

        // Valid file constraints
        let decl = create_capability_decl(
            "file",
            Some(create_constraint_block(vec![
                (
                    "paths",
                    ConstraintValue::Array(vec![ConstraintValue::String("/tmp/*".to_string())]),
                ),
                ("read", ConstraintValue::Bool(true)),
                ("write", ConstraintValue::Bool(false)),
            ])),
        );

        assert!(checker.check_capability_decl(&decl).is_ok());
    }

    #[test]
    fn test_invalid_constraint_field() {
        let mut cap_defs = HashMap::new();
        cap_defs.insert("file".to_string(), create_capability_def("file"));

        let checker = ConstraintChecker::new(&cap_defs);

        // Invalid field for file capability
        let decl = create_capability_decl(
            "file",
            Some(create_constraint_block(vec![(
                "invalid_field",
                ConstraintValue::Bool(true),
            )])),
        );

        let result = checker.check_capability_decl(&decl);
        assert!(result.is_err());

        match result.unwrap_err() {
            ConstraintCheckError::InvalidConstraintField {
                capability, field, ..
            } => {
                assert_eq!(capability, "file");
                assert_eq!(field, "invalid_field");
            }
            _ => panic!("Expected InvalidConstraintField error"),
        }
    }

    #[test]
    fn test_constraint_type_mismatch() {
        let mut cap_defs = HashMap::new();
        cap_defs.insert("file".to_string(), create_capability_def("file"));

        let checker = ConstraintChecker::new(&cap_defs);

        // Type mismatch: read should be bool, not string
        let decl = create_capability_decl(
            "file",
            Some(create_constraint_block(vec![(
                "read",
                ConstraintValue::String("true".to_string()),
            )])),
        );

        let result = checker.check_capability_decl(&decl);
        assert!(result.is_err());

        match result.unwrap_err() {
            ConstraintCheckError::ConstraintTypeMismatch {
                field,
                expected,
                found,
            } => {
                assert_eq!(field, "read");
                assert_eq!(expected, "bool");
                assert_eq!(found, "string");
            }
            _ => panic!("Expected ConstraintTypeMismatch error"),
        }
    }

    #[test]
    fn test_unknown_capability() {
        let cap_defs: HashMap<String, CapabilityDef> = HashMap::new();
        let checker = ConstraintChecker::new(&cap_defs);

        let decl = create_capability_decl("unknown_cap", None);

        let result = checker.check_capability_decl(&decl);
        assert!(result.is_err());

        match result.unwrap_err() {
            ConstraintCheckError::UnknownCapability { name, .. } => {
                assert_eq!(name, "unknown_cap");
            }
            _ => panic!("Expected UnknownCapability error"),
        }
    }

    #[test]
    fn test_valid_network_constraints() {
        let mut cap_defs = HashMap::new();
        cap_defs.insert("network".to_string(), create_capability_def("network"));

        let checker = ConstraintChecker::new(&cap_defs);

        let decl = create_capability_decl(
            "network",
            Some(create_constraint_block(vec![
                (
                    "hosts",
                    ConstraintValue::Array(vec![ConstraintValue::String(
                        "*.example.com".to_string(),
                    )]),
                ),
                (
                    "ports",
                    ConstraintValue::Array(vec![ConstraintValue::Int(443)]),
                ),
                (
                    "protocols",
                    ConstraintValue::Array(vec![ConstraintValue::String("https".to_string())]),
                ),
            ])),
        );

        assert!(checker.check_capability_decl(&decl).is_ok());
    }

    #[test]
    fn test_valid_process_constraints() {
        let mut cap_defs = HashMap::new();
        cap_defs.insert("process".to_string(), create_capability_def("process"));

        let checker = ConstraintChecker::new(&cap_defs);

        let decl = create_capability_decl(
            "process",
            Some(create_constraint_block(vec![
                ("spawn", ConstraintValue::Bool(true)),
                ("kill", ConstraintValue::Bool(false)),
                (
                    "signal",
                    ConstraintValue::Array(vec![ConstraintValue::Int(9), ConstraintValue::Int(15)]),
                ),
            ])),
        );

        assert!(checker.check_capability_decl(&decl).is_ok());
    }

    #[test]
    fn test_no_constraints_is_valid() {
        let mut cap_defs = HashMap::new();
        cap_defs.insert("file".to_string(), create_capability_def("file"));

        let checker = ConstraintChecker::new(&cap_defs);

        // Capability with no constraints should be valid
        let decl = create_capability_decl("file", None);

        assert!(checker.check_capability_decl(&decl).is_ok());
    }

    #[test]
    fn test_empty_constraints_is_valid() {
        let mut cap_defs = HashMap::new();
        cap_defs.insert("file".to_string(), create_capability_def("file"));

        let checker = ConstraintChecker::new(&cap_defs);

        // Capability with empty constraint block should be valid
        let decl = create_capability_decl("file", Some(create_constraint_block(vec![])));

        assert!(checker.check_capability_decl(&decl).is_ok());
    }

    #[test]
    fn test_has_capability() {
        let mut cap_defs = HashMap::new();
        cap_defs.insert("file".to_string(), create_capability_def("file"));

        let checker = ConstraintChecker::new(&cap_defs);

        assert!(checker.has_capability("file"));
        assert!(!checker.has_capability("unknown"));
    }

    #[test]
    fn test_available_capabilities() {
        let mut cap_defs = HashMap::new();
        cap_defs.insert("file".to_string(), create_capability_def("file"));
        cap_defs.insert("network".to_string(), create_capability_def("network"));

        let checker = ConstraintChecker::new(&cap_defs);
        let caps: Vec<_> = checker.available_capabilities().collect();

        assert_eq!(caps.len(), 2);
        assert!(caps.contains(&&"file".to_string()));
        assert!(caps.contains(&&"network".to_string()));
    }

    #[test]
    fn test_expected_constraint_type_matches() {
        assert!(ExpectedConstraintType::Bool.matches(&ConstraintValue::Bool(true)));
        assert!(ExpectedConstraintType::Int.matches(&ConstraintValue::Int(42)));
        assert!(
            ExpectedConstraintType::String.matches(&ConstraintValue::String("test".to_string()))
        );
        assert!(ExpectedConstraintType::Array.matches(&ConstraintValue::Array(vec![])));
        assert!(ExpectedConstraintType::Object.matches(&ConstraintValue::Object(vec![])));

        // Mismatches
        assert!(!ExpectedConstraintType::Bool.matches(&ConstraintValue::Int(1)));
        assert!(!ExpectedConstraintType::Int.matches(&ConstraintValue::String("42".to_string())));
    }

    #[test]
    fn test_any_of_type_matches() {
        let any_type = ExpectedConstraintType::AnyOf(vec![
            ExpectedConstraintType::Bool,
            ExpectedConstraintType::Int,
        ]);

        assert!(any_type.matches(&ConstraintValue::Bool(true)));
        assert!(any_type.matches(&ConstraintValue::Int(42)));
        assert!(!any_type.matches(&ConstraintValue::String("test".to_string())));
    }

    #[test]
    fn test_type_descriptions() {
        assert_eq!(ExpectedConstraintType::Bool.description(), "bool");
        assert_eq!(ExpectedConstraintType::Int.description(), "int");
        assert_eq!(ExpectedConstraintType::String.description(), "string");
        assert_eq!(ExpectedConstraintType::Array.description(), "array");
        assert_eq!(ExpectedConstraintType::Object.description(), "object");

        let any_type = ExpectedConstraintType::AnyOf(vec![
            ExpectedConstraintType::Bool,
            ExpectedConstraintType::Int,
        ]);
        assert_eq!(any_type.description(), "one of: bool, int");
    }
}
