//! Capability read/write type checking for the Ash workflow language
//!
//! This module provides type checking for capability schemas, allowing
//! validation of input (read) and output (write) values against declared types.

use crate::types::Type;
use ash_core::{Name, Value};
use std::collections::HashMap;
use thiserror::Error;

/// Capability schema with read and write types
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilitySchema {
    /// Type returned by observe/receive operations
    pub read: Option<Type>,
    /// Type accepted by set/send operations
    pub write: Option<Type>,
}

impl CapabilitySchema {
    /// Create a read-only schema
    pub fn read_only(ty: Type) -> Self {
        Self {
            read: Some(ty),
            write: None,
        }
    }

    /// Create a write-only schema
    pub fn write_only(ty: Type) -> Self {
        Self {
            read: None,
            write: Some(ty),
        }
    }

    /// Create a bidirectional schema (both read and write)
    pub fn bidirectional(read: Type, write: Type) -> Self {
        Self {
            read: Some(read),
            write: Some(write),
        }
    }

    /// Validate an input value against the read schema
    pub fn validate_input(&self, value: &Value) -> Result<(), CapabilityTypeError> {
        match &self.read {
            Some(schema) => {
                if schema.matches(value) {
                    Ok(())
                } else {
                    Err(CapabilityTypeError::InputMismatch {
                        capability: "unknown".into(),
                        expected: format!("{:?}", schema),
                        actual: format!("{:?}", value),
                    })
                }
            }
            None => Err(CapabilityTypeError::NotReadable("unknown".into())),
        }
    }

    /// Validate an output value against the write schema
    pub fn validate_output(&self, value: &Value) -> Result<(), CapabilityTypeError> {
        match &self.write {
            Some(schema) => {
                if schema.matches(value) {
                    Ok(())
                } else {
                    Err(CapabilityTypeError::OutputMismatch {
                        capability: "unknown".into(),
                        expected: format!("{:?}", schema),
                        actual: format!("{:?}", value),
                    })
                }
            }
            None => Err(CapabilityTypeError::NotWritable("unknown".into())),
        }
    }
}

/// Type error for capability operations
#[derive(Debug, Error, Clone, PartialEq)]
pub enum CapabilityTypeError {
    /// Input type mismatch
    #[error("input type mismatch for '{capability}': expected {expected}, got {actual}")]
    InputMismatch {
        capability: String,
        expected: String,
        actual: String,
    },

    /// Output type mismatch
    #[error("output type mismatch for '{capability}': expected {expected}, got {actual}")]
    OutputMismatch {
        capability: String,
        expected: String,
        actual: String,
    },

    /// Capability is not readable
    #[error("capability '{0}' is not readable")]
    NotReadable(String),

    /// Capability is not writable
    #[error("capability '{0}' is not writable")]
    NotWritable(String),

    /// Unknown capability
    #[error("unknown capability: {0}")]
    UnknownCapability(String),
}

/// Registry of capability schemas
#[derive(Debug, Default)]
pub struct CapabilitySchemaRegistry {
    schemas: HashMap<(Name, Name), CapabilitySchema>,
}

impl CapabilitySchemaRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    /// Register a capability schema for a given capability and channel
    pub fn register(&mut self, capability: Name, channel: Name, schema: CapabilitySchema) {
        self.schemas.insert((capability, channel), schema);
    }

    /// Get a capability schema by capability and channel name
    pub fn get(&self, capability: &Name, channel: &Name) -> Option<&CapabilitySchema> {
        self.schemas.get(&(capability.clone(), channel.clone()))
    }

    /// Validate an input value against a capability's read schema
    pub fn validate_input(
        &self,
        capability: &Name,
        channel: &Name,
        value: &Value,
    ) -> Result<(), CapabilityTypeError> {
        let schema = self.get(capability, channel).ok_or_else(|| {
            CapabilityTypeError::UnknownCapability(format!("{}:{}", capability, channel))
        })?;

        schema.validate_input(value).map_err(|e| match e {
            CapabilityTypeError::InputMismatch {
                expected, actual, ..
            } => CapabilityTypeError::InputMismatch {
                capability: format!("{}:{}", capability, channel),
                expected,
                actual,
            },
            CapabilityTypeError::NotReadable(_) => {
                CapabilityTypeError::NotReadable(format!("{}:{}", capability, channel))
            }
            other => other,
        })
    }

    /// Validate an output value against a capability's write schema
    pub fn validate_output(
        &self,
        capability: &Name,
        channel: &Name,
        value: &Value,
    ) -> Result<(), CapabilityTypeError> {
        let schema = self.get(capability, channel).ok_or_else(|| {
            CapabilityTypeError::UnknownCapability(format!("{}:{}", capability, channel))
        })?;

        schema.validate_output(value).map_err(|e| match e {
            CapabilityTypeError::OutputMismatch {
                expected, actual, ..
            } => CapabilityTypeError::OutputMismatch {
                capability: format!("{}:{}", capability, channel),
                expected,
                actual,
            },
            CapabilityTypeError::NotWritable(_) => {
                CapabilityTypeError::NotWritable(format!("{}:{}", capability, channel))
            }
            other => other,
        })
    }
}

/// Extension trait for Type to check if a value matches the type
pub trait TypeMatcher {
    /// Check if a value matches this type
    fn matches(&self, value: &Value) -> bool;
}

impl TypeMatcher for Type {
    fn matches(&self, value: &Value) -> bool {
        match (self, value) {
            // Primitive types - direct matching
            (Type::Int, Value::Int(_)) => true,
            (Type::String, Value::String(_)) => true,
            (Type::Bool, Value::Bool(_)) => true,
            (Type::Null, Value::Null) => true,
            (Type::Time, Value::Time(_)) => true,
            (Type::Ref, Value::Ref(_)) => true,

            // List type - element type must match all elements
            (Type::List(elem_type), Value::List(elements)) => {
                elements.iter().all(|e| elem_type.matches(e))
            }

            // Record type - all fields must be present with matching types
            (Type::Record(fields), Value::Record(record)) => fields
                .iter()
                .all(|(name, ty)| record.get(name.as_ref()).is_some_and(|v| ty.matches(v))),

            // Capability type - match by name (effect not checked at value level)
            (Type::Cap { name, .. }, Value::Cap(cap_name)) => name.as_ref() == cap_name.as_str(),

            // Type variables match any value (during inference)
            (Type::Var(_), _) => true,

            // Function types don't have direct value representations
            (Type::Fun(_, _, _), _) => false,

            // All other combinations don't match
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_read_only_schema() {
        let schema = CapabilitySchema::read_only(Type::Int);

        assert!(schema.read.is_some());
        assert!(schema.write.is_none());

        // Valid input
        assert!(schema.validate_input(&Value::Int(42)).is_ok());

        // Invalid input
        assert!(
            schema
                .validate_input(&Value::String("hello".into()))
                .is_err()
        );

        // Output not allowed (schema has no write type)
        assert!(matches!(
            schema.validate_output(&Value::Int(42)),
            Err(CapabilityTypeError::NotWritable(_))
        ));
    }

    #[test]
    fn test_write_only_schema() {
        let schema = CapabilitySchema::write_only(Type::String);

        assert!(schema.read.is_none());
        assert!(schema.write.is_some());

        // Input not allowed
        assert!(matches!(
            schema.validate_input(&Value::String("test".into())),
            Err(CapabilityTypeError::NotReadable(_))
        ));

        // Valid output
        assert!(
            schema
                .validate_output(&Value::String("test".into()))
                .is_ok()
        );

        // Invalid output type
        assert!(schema.validate_output(&Value::Int(42)).is_err());
    }

    #[test]
    fn test_bidirectional_schema() {
        let schema = CapabilitySchema::bidirectional(Type::Int, Type::Int);

        assert!(schema.read.is_some());
        assert!(schema.write.is_some());

        assert!(schema.validate_input(&Value::Int(42)).is_ok());
        assert!(schema.validate_output(&Value::Int(42)).is_ok());
    }

    #[test]
    fn test_registry_validate_input() {
        let mut registry = CapabilitySchemaRegistry::new();
        registry.register(
            "sensor".to_string(),
            "temp".to_string(),
            CapabilitySchema::read_only(Type::Int),
        );

        // Valid input
        assert!(
            registry
                .validate_input(&"sensor".to_string(), &"temp".to_string(), &Value::Int(25))
                .is_ok()
        );

        // Invalid input type
        assert!(
            registry
                .validate_input(
                    &"sensor".to_string(),
                    &"temp".to_string(),
                    &Value::String("hot".into())
                )
                .is_err()
        );

        // Unknown capability
        assert!(matches!(
            registry.validate_input(&"unknown".to_string(), &"cap".to_string(), &Value::Int(1)),
            Err(CapabilityTypeError::UnknownCapability(_))
        ));
    }

    #[test]
    fn test_registry_validate_output() {
        let mut registry = CapabilitySchemaRegistry::new();
        registry.register(
            "actuator".to_string(),
            "power".to_string(),
            CapabilitySchema::write_only(Type::Bool),
        );

        // Valid output
        assert!(
            registry
                .validate_output(
                    &"actuator".to_string(),
                    &"power".to_string(),
                    &Value::Bool(true)
                )
                .is_ok()
        );

        // Invalid output type
        assert!(
            registry
                .validate_output(
                    &"actuator".to_string(),
                    &"power".to_string(),
                    &Value::Int(1)
                )
                .is_err()
        );

        // Unknown capability
        assert!(matches!(
            registry.validate_output(
                &"unknown".to_string(),
                &"cap".to_string(),
                &Value::Bool(true)
            ),
            Err(CapabilityTypeError::UnknownCapability(_))
        ));
    }

    #[test]
    fn test_record_schema() {
        let schema = CapabilitySchema::read_only(Type::Record(vec![
            (Box::from("value"), Type::Int),
            (Box::from("unit"), Type::String),
        ]));

        let valid = Value::Record(HashMap::from([
            ("value".to_string(), Value::Int(25)),
            ("unit".to_string(), Value::String("celsius".into())),
        ]));

        assert!(schema.validate_input(&valid).is_ok());

        let invalid = Value::Record(HashMap::from([
            ("value".to_string(), Value::String("wrong".into())),
            ("unit".to_string(), Value::String("celsius".into())),
        ]));

        assert!(schema.validate_input(&invalid).is_err());
    }

    #[test]
    fn test_list_schema() {
        let schema = CapabilitySchema::read_only(Type::List(Box::new(Type::Int)));

        // Valid list of ints
        let valid = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert!(schema.validate_input(&valid).is_ok());

        // Invalid - mixed types
        let invalid = Value::List(vec![Value::Int(1), Value::String("two".into())]);
        assert!(schema.validate_input(&invalid).is_err());
    }

    #[test]
    fn test_nested_record_schema() {
        let schema = CapabilitySchema::read_only(Type::Record(vec![
            (
                Box::from("location"),
                Type::Record(vec![
                    (Box::from("lat"), Type::Int),
                    (Box::from("lon"), Type::Int),
                ]),
            ),
            (Box::from("name"), Type::String),
        ]));

        let valid = Value::Record(HashMap::from([
            (
                "location".to_string(),
                Value::Record(HashMap::from([
                    ("lat".to_string(), Value::Int(40)),
                    ("lon".to_string(), Value::Int(-74)),
                ])),
            ),
            ("name".to_string(), Value::String("NYC".into())),
        ]));

        assert!(schema.validate_input(&valid).is_ok());

        // Missing nested field
        let invalid = Value::Record(HashMap::from([
            (
                "location".to_string(),
                Value::Record(HashMap::from([("lat".to_string(), Value::Int(40))])),
            ),
            ("name".to_string(), Value::String("NYC".into())),
        ]));

        assert!(schema.validate_input(&invalid).is_err());
    }

    #[test]
    fn test_type_var_matches_anything() {
        // Type variables should match any value during inference
        let type_var = Type::Var(crate::types::TypeVar(0));

        assert!(type_var.matches(&Value::Int(42)));
        assert!(type_var.matches(&Value::String("hello".into())));
        assert!(type_var.matches(&Value::Bool(true)));
    }

    #[test]
    fn test_capability_value_matching() {
        let cap_type = Type::Cap {
            name: Box::from("FileIO"),
            effect: ash_core::Effect::Operational,
        };

        assert!(cap_type.matches(&Value::Cap("FileIO".to_string())));
        assert!(!cap_type.matches(&Value::Cap("Network".to_string())));
        assert!(!cap_type.matches(&Value::Int(42)));
    }

    #[test]
    fn test_error_messages() {
        let err = CapabilityTypeError::InputMismatch {
            capability: "sensor:temp".to_string(),
            expected: "Int".to_string(),
            actual: "String".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("sensor:temp"));
        assert!(msg.contains("Int"));
        assert!(msg.contains("String"));

        let err = CapabilityTypeError::NotReadable("sensor:temp".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("sensor:temp"));
        assert!(msg.contains("not readable"));

        let err = CapabilityTypeError::NotWritable("actuator:power".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("actuator:power"));
        assert!(msg.contains("not writable"));

        let err = CapabilityTypeError::UnknownCapability("unknown:cap".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("unknown:cap"));
        assert!(msg.contains("unknown capability"));
    }
}
