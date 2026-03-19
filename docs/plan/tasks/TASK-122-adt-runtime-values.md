# TASK-122: ADT Runtime Values

## Status: ✅ Complete

## Description

Add runtime value representations for ADTs to `ash-core` and `ash-interp`. This includes `Value::Variant`, `Value::Struct`, `Value::Tuple` for representing constructed values at runtime.

## Specification Reference

- SPEC-020: ADT Types - Section 7.1

## Requirements

### Functional Requirements

1. Extend `Value` enum with:
   - `Variant` - Enum variant with type name, variant name, and fields
   - `Struct` - Struct value with type name and fields
   - `Tuple` - Anonymous product type (tuple)

2. Implement `Display` for new value types

3. Implement serialization support

4. Update pattern matching in interpreter to work with new value types

### Property Requirements

```rust
// Value roundtrip through serialization
prop_value_roundtrip(v: Value) = {
    let json = serde_json::to_string(&v).unwrap();
    let v2: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v, v2);
}

// Pattern matching reconstructs original
prop_pattern_reconstruct(v: Value) = {
    let pattern = value_to_wildcard_pattern(&v);
    let matched = match_pattern(&pattern, &v);
    assert!(matched.is_ok());
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File**: `crates/ash-core/src/value.rs` (append tests)

```rust
#[cfg(test)]
mod adt_value_tests {
    use super::*;

    #[test]
    fn test_variant_creation() {
        let v = Value::Variant {
            typ: "Option".into(),
            variant: "Some".into(),
            fields: vec![("value".into(), Value::Int(42))],
        };
        
        match &v {
            Value::Variant { typ, variant, fields } => {
                assert_eq!(typ, "Option");
                assert_eq!(variant, "Some");
                assert_eq!(fields.len(), 1);
            }
            _ => panic!("Expected variant"),
        }
    }

    #[test]
    fn test_struct_creation() {
        let s = Value::Struct {
            typ: "Point".into(),
            fields: vec![
                ("x".into(), Value::Int(10)),
                ("y".into(), Value::Int(20)),
            ],
        };
        
        assert_eq!(s.to_string(), "Point { x: 10, y: 20 }");
    }

    #[test]
    fn test_tuple_creation() {
        let t = Value::Tuple(vec![Value::Int(1), Value::String("hello".into())]);
        assert_eq!(t.to_string(), "(1, \"hello\")");
    }

    #[test]
    fn test_variant_serde_roundtrip() {
        let v = Value::Variant {
            typ: "Result".into(),
            variant: "Ok".into(),
            fields: vec![("value".into(), Value::Int(42))],
        };
        
        let json = serde_json::to_string(&v).unwrap();
        let v2: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v, v2);
    }
}
```

### Step 2: Add Value Variants (Green)

**File**: `crates/ash-core/src/value.rs`

Add to `Value` enum:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Value {
    // Existing variants...
    Int(i64),
    String(Box<str>),
    Bool(bool),
    Null,
    Time(DateTime<Utc>),
    Ref(WorkflowId),
    List(Vec<Value>),
    Record(HashMap<Box<str>, Value>),
    Cap(Box<str>),

    /// Tuple value: (1, "hello", true)
    Tuple(Vec<Value>),

    /// Struct value: Point { x: 10, y: 20 }
    Struct {
        #[serde(rename = "struct_type")]
        typ: Box<str>,
        fields: Vec<(Box<str>, Value)>,
    },

    /// Enum variant: Some { value: 42 }
    Variant {
        #[serde(rename = "enum_type")]
        typ: Box<str>,
        variant: Box<str>,
        fields: Vec<(Box<str>, Value)>,
    },
}
```

### Step 3: Update Display (Green)

**File**: `crates/ash-core/src/value.rs`

Add to `Display` impl:

```rust
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Existing cases...
            Value::Int(i) => write!(f, "{}", i),
            Value::String(s) => write!(f, "\"{}\"", s.escape_debug()),
            // ... etc

            Value::Tuple(values) => {
                write!(f, "(")?;
                for (i, v) in values.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }

            Value::Struct { typ, fields } => {
                write!(f, "{} {{", typ)?;
                for (i, (name, value)) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, " {}: {}", name, value)?;
                }
                write!(f, " }}")
            }

            Value::Variant { typ, variant, fields } => {
                write!(f, "{}::{}", typ, variant)?;
                if !fields.is_empty() {
                    write!(f, " {{")?;
                    for (i, (name, value)) in fields.iter().enumerate() {
                        if i > 0 { write!(f, ", ")?; }
                        write!(f, " {}: {}", name, value)?;
                    }
                    write!(f, " }}")?;
                }
                Ok(())
            }
        }
    }
}
```

### Step 4: Add Helper Methods (Green)

**File**: `crates/ash-core/src/value.rs`

Add convenience constructors:

```rust
impl Value {
    /// Create a variant value
    pub fn variant(
        typ: impl Into<Box<str>>,
        variant: impl Into<Box<str>>,
        fields: Vec<(impl Into<Box<str>>, Value)>,
    ) -> Self {
        Value::Variant {
            typ: typ.into(),
            variant: variant.into(),
            fields: fields.into_iter().map(|(n, v)| (n.into(), v)).collect(),
        }
    }

    /// Create a struct value
    pub fn struct_(
        typ: impl Into<Box<str>>,
        fields: Vec<(impl Into<Box<str>>, Value)>,
    ) -> Self {
        Value::Struct {
            typ: typ.into(),
            fields: fields.into_iter().map(|(n, v)| (n.into(), v)).collect(),
        }
    }

    /// Create a tuple value
    pub fn tuple(values: Vec<Value>) -> Self {
        Value::Tuple(values)
    }

    /// Get a field from a struct or variant
    pub fn get_field(&self, name: &str) -> Option<&Value> {
        match self {
            Value::Struct { fields, .. } => {
                fields.iter().find(|(n, _)| n == name).map(|(_, v)| v)
            }
            Value::Variant { fields, .. } => {
                fields.iter().find(|(n, _)| n == name).map(|(_, v)| v)
            }
            _ => None,
        }
    }

    /// Get variant name if this is a variant
    pub fn variant_name(&self) -> Option<&str> {
        match self {
            Value::Variant { variant, .. } => Some(variant),
            _ => None,
        }
    }

    /// Check if value is a specific variant
    pub fn is_variant(&self, type_name: &str, variant_name: &str) -> bool {
        matches!(self, Value::Variant { typ, variant, .. } 
            if typ.as_ref() == type_name && variant.as_ref() == variant_name)
    }
}
```

### Step 5: Update Pattern Matching (Green)

**File**: `crates/ash-interp/src/pattern.rs`

Add pattern matching for new value types. First, extend the `Pattern` enum in AST if needed:

```rust
// In crates/ash-core/src/ast.rs - add to Pattern enum:
pub enum Pattern {
    // Existing...
    Variable(Name),
    Tuple(Vec<Pattern>),
    Record(Vec<(Name, Pattern)>),
    List(Vec<Pattern>, Option<Name>),
    Wildcard,
    Literal(Value),
    
    /// Variant pattern: Some { value: x }
    Variant {
        name: Name,
        fields: Vec<(Name, Pattern)>,
    },
}
```

Now add pattern matching logic:

```rust
// In crates/ash-interp/src/pattern.rs

use ash_core::{Pattern, Value};
use std::collections::HashMap;

/// Result of pattern matching
pub type MatchResult = Result<HashMap<String, Value>, MatchError>;

#[derive(Debug, Clone, Error)]
pub enum MatchError {
    #[error("Pattern mismatch: expected {expected}, got {actual}")]
    Mismatch { expected: String, actual: String },
}

/// Match a pattern against a value, returning bindings
pub fn match_pattern(pattern: &Pattern, value: &Value) -> MatchResult {
    let mut bindings = HashMap::new();
    match_pattern_inner(pattern, value, &mut bindings)?;
    Ok(bindings)
}

fn match_pattern_inner(
    pattern: &Pattern,
    value: &Value,
    bindings: &mut HashMap<String, Value>,
) -> Result<(), MatchError> {
    match (pattern, value) {
        // Existing cases...
        (Pattern::Wildcard, _) => Ok(()),
        
        (Pattern::Variable(name), _) => {
            bindings.insert(name.clone(), value.clone());
            Ok(())
        }
        
        (Pattern::Literal(lit), val) => {
            if lit == val {
                Ok(())
            } else {
                Err(MatchError::Mismatch {
                    expected: lit.to_string(),
                    actual: val.to_string(),
                })
            }
        }
        
        // Tuple pattern matches tuple value
        (Pattern::Tuple(patterns), Value::Tuple(values)) => {
            if patterns.len() != values.len() {
                return Err(MatchError::Mismatch {
                    expected: format!("tuple of {} elements", patterns.len()),
                    actual: format!("tuple of {} elements", values.len()),
                });
            }
            for (p, v) in patterns.iter().zip(values.iter()) {
                match_pattern_inner(p, v, bindings)?;
            }
            Ok(())
        }
        
        // Record pattern matches struct value
        (Pattern::Record(field_patterns), Value::Struct { fields, .. }) => {
            for (field_name, field_pattern) in field_patterns {
                match fields.iter().find(|(n, _)| n == field_name) {
                    Some((_, field_value)) => {
                        match_pattern_inner(field_pattern, field_value, bindings)?;
                    }
                    None => {
                        return Err(MatchError::Mismatch {
                            expected: format!("field '{}'", field_name),
                            actual: "missing".to_string(),
                        });
                    }
                }
            }
            Ok(())
        }
        
        // Variant pattern
        (Pattern::Variant { name, fields }, Value::Variant { variant, fields: val_fields, .. }) => {
            if name != variant {
                return Err(MatchError::Mismatch {
                    expected: format!("variant '{}'", name),
                    actual: format!("variant '{}'", variant),
                });
            }
            
            for (field_name, field_pattern) in fields {
                match val_fields.iter().find(|(n, _)| n == field_name) {
                    Some((_, field_value)) => {
                        match_pattern_inner(field_pattern, field_value, bindings)?;
                    }
                    None => {
                        return Err(MatchError::Mismatch {
                            expected: format!("field '{}'", field_name),
                            actual: "missing".to_string(),
                        });
                    }
                }
            }
            Ok(())
        }
        
        // Mismatched types
        _ => Err(MatchError::Mismatch {
            expected: pattern_type_name(pattern),
            actual: value_type_name(value),
        }),
    }
}

fn pattern_type_name(pattern: &Pattern) -> String {
    match pattern {
        Pattern::Variable(_) => "variable".to_string(),
        Pattern::Wildcard => "wildcard".to_string(),
        Pattern::Literal(_) => "literal".to_string(),
        Pattern::Tuple(_) => "tuple".to_string(),
        Pattern::Record(_) => "record".to_string(),
        Pattern::List(_, _) => "list".to_string(),
        Pattern::Variant { name, .. } => format!("variant '{}'", name),
    }
}

fn value_type_name(value: &Value) -> String {
    match value {
        Value::Int(_) => "int".to_string(),
        Value::String(_) => "string".to_string(),
        Value::Bool(_) => "bool".to_string(),
        Value::Null => "null".to_string(),
        Value::Time(_) => "time".to_string(),
        Value::Ref(_) => "ref".to_string(),
        Value::List(_) => "list".to_string(),
        Value::Record(_) => "record".to_string(),
        Value::Cap(_) => "cap".to_string(),
        Value::Tuple(_) => "tuple".to_string(),
        Value::Struct { typ, .. } => format!("struct '{}'", typ),
        Value::Variant { typ, variant, .. } => format!("{}::{}", typ, variant),
    }
}
```

### Step 6: Run Tests and Verify

```bash
cargo test -p ash-core value::adt_value_tests -- --nocapture
cargo test -p ash-interp pattern::tests -- --nocapture
```

### Step 7: Refactor

Review against rust-skills:
- [api-common-traits] Ensure Clone, Debug, PartialEq derived
- [err-thiserror-lib] Use thiserror for MatchError
- [doc-all-public] Document all public methods

## Completion Checklist

- [ ] `Value::Variant`, `Value::Struct`, `Value::Tuple` added
- [ ] `Display` implemented for all new value types
- [ ] `Serialize`/`Deserialize` derived/configured correctly
- [ ] Helper methods (`variant`, `struct_`, `tuple`, `get_field`, etc.)
- [ ] Pattern matching supports new value types
- [ ] `Pattern::Variant` added to AST
- [ ] Property tests for serialization roundtrip
- [ ] Unit tests for display and matching
- [ ] Documentation complete
- [ ] `cargo fmt` and `cargo clippy` pass

## Estimated Effort

5 hours

## Dependencies

- TASK-121 (ADT Core Types)

## Blocked By

- TASK-121

## Blocks

- TASK-124 (Parse ADT Definitions)
- TASK-131 (Constructor Evaluation)
- TASK-132 (Pattern Matching Engine)
