# TASK-132: Pattern Matching Engine

## Status: 🟡 Ready

## Description

Implement the core pattern matching engine for matching values against patterns.

## Specification Reference

- SPEC-020: ADT Types - Section 7.2

## Requirements

Match patterns against values and extract bindings:
```rust
match value {
    Some { value: x } => ...  -- Extract x from Some variant
    None => ...
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File**: `crates/ash-interp/src/pattern.rs` (new)

```rust
//! Pattern matching engine

use ash_core::ast::Pattern;
use ash_core::value::Value;
use std::collections::HashMap;
use crate::{EvalError, EvalResult};

/// Result of pattern matching: bindings or failure
pub type MatchBindings = HashMap<String, Value>;

/// Match a pattern against a value
pub fn match_pattern(
    pattern: &Pattern,
    value: &Value,
) -> EvalResult<MatchBindings> {
    let mut bindings = HashMap::new();
    match_pattern_inner(pattern, value, &mut bindings)?;
    Ok(bindings)
}

fn match_pattern_inner(
    pattern: &Pattern,
    value: &Value,
    bindings: &mut MatchBindings,
) -> EvalResult<()> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::ast::Pattern;
    use ash_core::value::Value;

    #[test]
    fn test_match_wildcard() {
        let pattern = Pattern::Wildcard;
        let value = Value::Int(42);
        
        let result = match_pattern(&pattern, &value);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_match_variable() {
        let pattern = Pattern::Variable("x".to_string());
        let value = Value::Int(42);
        
        let result = match_pattern(&pattern, &value);
        assert!(result.is_ok());
        
        let bindings = result.unwrap();
        assert_eq!(bindings.get("x"), Some(&Value::Int(42)));
    }

    #[test]
    fn test_match_variant() {
        let pattern = Pattern::Variant {
            name: "Some".to_string(),
            fields: Some(vec![("value".to_string(), Pattern::Variable("x".to_string()))]),
        };
        let value = Value::Variant {
            type_name: "Option".into(),
            variant_name: "Some".into(),
            fields: {
                let mut fields = HashMap::new();
                fields.insert("value".into(), Value::Int(42));
                fields
            },
        };
        
        let result = match_pattern(&pattern, &value);
        assert!(result.is_ok());
        
        let bindings = result.unwrap();
        assert_eq!(bindings.get("x"), Some(&Value::Int(42)));
    }

    #[test]
    fn test_match_variant_mismatch() {
        let pattern = Pattern::Variant {
            name: "Some".to_string(),
            fields: None,
        };
        let value = Value::Variant {
            type_name: "Option".into(),
            variant_name: "None".into(),
            fields: HashMap::new(),
        };
        
        let result = match_pattern(&pattern, &value);
        assert!(result.is_err());
    }

    #[test]
    fn test_match_literal() {
        let pattern = Pattern::Literal(Value::Int(42));
        let value = Value::Int(42);
        
        let result = match_pattern(&pattern, &value);
        assert!(result.is_ok());
    }

    #[test]
    fn test_match_literal_mismatch() {
        let pattern = Pattern::Literal(Value::Int(42));
        let value = Value::Int(43);
        
        let result = match_pattern(&pattern, &value);
        assert!(result.is_err());
    }
}
```

### Step 2: Implement Pattern Matching (Green)

```rust
fn match_pattern_inner(
    pattern: &Pattern,
    value: &Value,
    bindings: &mut MatchBindings,
) -> EvalResult<()> {
    match (pattern, value) {
        // Wildcard matches anything
        (Pattern::Wildcard, _) => Ok(()),
        
        // Variable binds the value
        (Pattern::Variable(name), val) => {
            bindings.insert(name.clone(), val.clone());
            Ok(())
        }
        
        // Literal must match exactly
        (Pattern::Literal(lit), val) => {
            if lit == val {
                Ok(())
            } else {
                Err(EvalError::PatternMismatch {
                    pattern: pattern.to_string(),
                    value: val.to_string(),
                })
            }
        }
        
        // Variant pattern
        (
            Pattern::Variant { name, fields },
            Value::Variant { variant_name, fields: val_fields, .. }
        ) => {
            if name != variant_name {
                return Err(EvalError::PatternMismatch {
                    pattern: format!("variant {}", name),
                    value: format!("variant {}", variant_name),
                });
            }
            
            // Match fields
            if let Some(field_patterns) = fields {
                for (field_name, field_pattern) in field_patterns {
                    let field_value = val_fields.get(field_name.as_str())
                        .ok_or_else(|| EvalError::MissingField {
                            field: field_name.clone(),
                        })?;
                    match_pattern_inner(field_pattern, field_value, bindings)?;
                }
            }
            
            Ok(())
        }
        
        // Struct pattern (similar to variant)
        (
            Pattern::Record(field_patterns),
            Value::Struct { fields: val_fields, .. }
        ) => {
            for (field_name, field_pattern) in field_patterns {
                let field_value = val_fields.get(field_name.as_str())
                    .ok_or_else(|| EvalError::MissingField {
                        field: field_name.clone(),
                    })?;
                match_pattern_inner(field_pattern, field_value, bindings)?;
            }
            Ok(())
        }
        
        // Tuple pattern
        (Pattern::Tuple(patterns), Value::Tuple(values)) => {
            if patterns.len() != values.len() {
                return Err(EvalError::PatternArityMismatch {
                    expected: patterns.len(),
                    actual: values.len(),
                });
            }
            for (pat, val) in patterns.iter().zip(values.iter()) {
                match_pattern_inner(pat, val, bindings)?;
            }
            Ok(())
        }
        
        // List pattern (simplified)
        (Pattern::List(patterns, tail), Value::List(values)) => {
            if patterns.len() > values.len() {
                return Err(EvalError::PatternArityMismatch {
                    expected: patterns.len(),
                    actual: values.len(),
                });
            }
            for (pat, val) in patterns.iter().zip(values.iter()) {
                match_pattern_inner(pat, val, bindings)?;
            }
            if let Some(tail_name) = tail {
                let tail_values: Vec<_> = values[patterns.len()..].to_vec();
                bindings.insert(tail_name.clone(), Value::List(tail_values));
            }
            Ok(())
        }
        
        // Mismatched types
        _ => Err(EvalError::PatternMismatch {
            pattern: pattern_type_name(pattern),
            value: value_type_name(value),
        }),
    }
}

fn pattern_type_name(pattern: &Pattern) -> String {
    match pattern {
        Pattern::Wildcard => "wildcard".to_string(),
        Pattern::Variable(_) => "variable".to_string(),
        Pattern::Literal(_) => "literal".to_string(),
        Pattern::Variant { name, .. } => format!("variant {}", name),
        Pattern::Struct { .. } => "struct".to_string(),
        Pattern::Tuple(_) => "tuple".to_string(),
        Pattern::List(_, _) => "list".to_string(),
        Pattern::Record(_) => "record".to_string(),
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
        Value::Struct { type_name, .. } => format!("struct {}", type_name),
        Value::Variant { variant_name, .. } => format!("variant {}", variant_name),
    }
}
```

### Step 3: Add EvalError Variants

**File**: `crates/ash-interp/src/error.rs`

```rust
#[derive(Debug, Clone, Error)]
pub enum EvalError {
    // Existing...
    
    #[error("Pattern mismatch: expected {pattern}, got {value}")]
    PatternMismatch { pattern: String, value: String },
    
    #[error("Pattern arity mismatch: expected {expected}, got {actual}")]
    PatternArityMismatch { expected: usize, actual: usize },
    
    #[error("Missing field in pattern: {field}")]
    MissingField { field: String },
}
```

### Step 4: Run Tests

```bash
cargo test -p ash-interp pattern -- --nocapture
```

### Step 5: Add Property Tests

```rust
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_wildcard_always_matches(v in arbitrary_value()) {
            let result = match_pattern(&Pattern::Wildcard, &v);
            prop_assert!(result.is_ok());
            prop_assert!(result.unwrap().is_empty());
        }
        
        #[test]
        fn prop_variable_always_matches_and_binds(v in arbitrary_value()) {
            let result = match_pattern(&Pattern::Variable("x".to_string()), &v);
            prop_assert!(result.is_ok());
            
            let bindings = result.unwrap();
            prop_assert_eq!(bindings.get("x"), Some(&v));
        }
        
        #[test]
        fn prop_literal_matches_itself(v in arbitrary_value()) {
            let pattern = value_to_literal_pattern(&v);
            let result = match_pattern(&pattern, &v);
            prop_assert!(result.is_ok());
        }
    }
}
```

### Step 6: Commit

```bash
git add crates/ash-interp/src/pattern.rs crates/ash-interp/src/error.rs
git commit -m "feat(interp): pattern matching engine (TASK-132)"
```

## Completion Checklist

- [ ] `match_pattern` function
- [ ] `MatchBindings` type alias
- [ ] Pattern::Wildcard matching
- [ ] Pattern::Variable binding
- [ ] Pattern::Literal matching
- [ ] Pattern::Variant matching
- [ ] Pattern::Struct matching
- [ ] Pattern::Tuple matching
- [ ] Pattern::List matching
- [ ] PatternMismatch error
- [ ] PatternArityMismatch error
- [ ] MissingField error
- [ ] Unit tests for all pattern types
- [ ] Property tests for wildcard/variable
- [ ] Documentation comments
- [ ] `cargo fmt` and `cargo clippy` pass

## Estimated Effort

6 hours

## Dependencies

- TASK-128 (Type Check Patterns)
- TASK-131 (Constructor Evaluation)

## Blocked By

- TASK-128
- TASK-131

## Blocks

- TASK-133 (Match Evaluation)
