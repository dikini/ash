//! Pattern matching for destructuring values
//!
//! Patterns allow extracting and binding values from complex data structures.

use ash_core::{Name, Pattern, Value};
use std::collections::HashMap;

use crate::error::PatternError;

/// Match a pattern against a value, returning bound variables on success
///
/// # Arguments
/// * `pattern` - The pattern to match
/// * `value` - The value to match against
///
/// # Returns
/// `Ok(HashMap)` with variable bindings if the match succeeds,
/// `Err(PatternError)` if the pattern cannot match the value.
///
/// # Examples
/// ```
/// use ash_core::{Pattern, Value};
/// use ash_interp::pattern::match_pattern;
/// use std::collections::HashMap;
///
/// let pattern = Pattern::Variable("x".to_string());
/// let value = Value::Int(42);
/// let bindings = match_pattern(&pattern, &value).unwrap();
/// assert_eq!(bindings.get("x"), Some(&Value::Int(42)));
/// ```
pub fn match_pattern(
    pattern: &Pattern,
    value: &Value,
) -> Result<HashMap<Name, Value>, PatternError> {
    let mut bindings = HashMap::new();
    match_pattern_recursive(pattern, value, &mut bindings)?;
    Ok(bindings)
}

fn match_pattern_recursive(
    pattern: &Pattern,
    value: &Value,
    bindings: &mut HashMap<Name, Value>,
) -> Result<(), PatternError> {
    match pattern {
        Pattern::Wildcard => {
            // Wildcard matches anything, binds nothing
            Ok(())
        }

        Pattern::Variable(name) => {
            // Variable matches anything and binds the value
            bindings.insert(name.clone(), value.clone());
            Ok(())
        }

        Pattern::Literal(expected) => {
            // Literal only matches equal values
            if expected == value {
                Ok(())
            } else {
                Err(PatternError::MatchFailed {
                    expected: format!("{:?}", expected),
                    actual: format!("{:?}", value),
                })
            }
        }

        Pattern::Tuple(patterns) => {
            // Tuple pattern matches list of same length
            match value {
                Value::List(values) if values.len() == patterns.len() => {
                    for (p, v) in patterns.iter().zip(values.iter()) {
                        match_pattern_recursive(p, v, bindings)?;
                    }
                    Ok(())
                }
                _ => Err(PatternError::MatchFailed {
                    expected: format!("tuple of {} elements", patterns.len()),
                    actual: format!("{:?}", value),
                }),
            }
        }

        Pattern::Record(field_patterns) => {
            // Record pattern matches record with matching fields
            match value {
                Value::Record(fields) => {
                    for (field_name, field_pattern) in field_patterns {
                        match fields.get(field_name) {
                            Some(field_value) => {
                                match_pattern_recursive(field_pattern, field_value, bindings)?;
                            }
                            None => {
                                return Err(PatternError::FieldMissing(field_name.clone()));
                            }
                        }
                    }
                    Ok(())
                }
                _ => Err(PatternError::NotARecord(value.clone())),
            }
        }

        Pattern::List(prefix_patterns, rest_binding) => {
            // List pattern matches list with at least prefix_patterns.len() elements
            match value {
                Value::List(values) => {
                    if values.len() < prefix_patterns.len() {
                        return Err(PatternError::ListLengthMismatch {
                            expected: prefix_patterns.len(),
                            actual: values.len(),
                        });
                    }

                    // Match prefix elements
                    for (p, v) in prefix_patterns.iter().zip(values.iter()) {
                        match_pattern_recursive(p, v, bindings)?;
                    }

                    // Bind rest if specified
                    if let Some(rest_name) = rest_binding {
                        let rest_values: Vec<Value> = values[prefix_patterns.len()..].to_vec();
                        bindings.insert(rest_name.clone(), Value::List(Box::new(rest_values)));
                    }

                    Ok(())
                }
                _ => Err(PatternError::MatchFailed {
                    expected: "list".to_string(),
                    actual: format!("{:?}", value),
                }),
            }
        }

        Pattern::Variant { name, fields } => {
            // Variant pattern matching - check value is a variant with matching name
            match value {
                Value::Variant {
                    name: variant_name,
                    fields: variant_fields,
                } => {
                    // Check variant name matches
                    if variant_name != name {
                        return Err(PatternError::MatchFailed {
                            expected: format!("variant {}", name),
                            actual: format!("variant {}", variant_name),
                        });
                    }

                    // Handle field patterns
                    match fields {
                        None => {
                            // Pattern expects unit variant (no fields)
                            if !variant_fields.is_empty() {
                                return Err(PatternError::MatchFailed {
                                    expected: format!("unit variant {}", name),
                                    actual: format!(
                                        "variant {} with {} fields",
                                        name,
                                        variant_fields.len()
                                    ),
                                });
                            }
                            Ok(())
                        }
                        Some(field_patterns) => {
                            // Pattern expects variant with specific fields
                            // Match each field pattern against the corresponding value field
                            for (field_name, field_pattern) in field_patterns {
                                let field_value = variant_fields
                                    .iter()
                                    .find(|(k, _)| k == field_name.as_str())
                                    .map(|(_, v)| v)
                                    .ok_or_else(|| {
                                        PatternError::FieldMissing(field_name.clone())
                                    })?;
                                match_pattern_recursive(field_pattern, field_value, bindings)?;
                            }
                            Ok(())
                        }
                    }
                }
                _ => Err(PatternError::MatchFailed {
                    expected: format!("variant {}", name),
                    actual: format!("{:?}", value),
                }),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_wildcard() {
        let pattern = Pattern::Wildcard;
        assert!(match_pattern(&pattern, &Value::Int(42)).is_ok());
        assert!(match_pattern(&pattern, &Value::Null).is_ok());
    }

    #[test]
    fn test_match_variable() {
        let pattern = Pattern::Variable("x".to_string());
        let bindings = match_pattern(&pattern, &Value::Int(42)).unwrap();
        assert_eq!(bindings.get("x"), Some(&Value::Int(42)));
    }

    #[test]
    fn test_match_literal_success() {
        let pattern = Pattern::Literal(Value::Int(42));
        let bindings = match_pattern(&pattern, &Value::Int(42)).unwrap();
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_match_literal_failure() {
        let pattern = Pattern::Literal(Value::Int(42));
        assert!(match_pattern(&pattern, &Value::Int(43)).is_err());
    }

    #[test]
    fn test_match_tuple() {
        let pattern = Pattern::Tuple(vec![
            Pattern::Variable("x".to_string()),
            Pattern::Variable("y".to_string()),
        ]);
        let value = Value::List(Box::new(vec![Value::Int(1), Value::Int(2)]));
        let bindings = match_pattern(&pattern, &value).unwrap();

        assert_eq!(bindings.get("x"), Some(&Value::Int(1)));
        assert_eq!(bindings.get("y"), Some(&Value::Int(2)));
    }

    #[test]
    fn test_match_tuple_wrong_length() {
        let pattern = Pattern::Tuple(vec![Pattern::Variable("x".to_string())]);
        let value = Value::List(Box::new(vec![Value::Int(1), Value::Int(2)]));
        assert!(match_pattern(&pattern, &value).is_err());
    }

    #[test]
    fn test_match_tuple_not_list() {
        let pattern = Pattern::Tuple(vec![Pattern::Variable("x".to_string())]);
        assert!(match_pattern(&pattern, &Value::Int(42)).is_err());
    }

    #[test]
    fn test_match_record() {
        let pattern = Pattern::Record(vec![
            ("name".to_string(), Pattern::Variable("n".to_string())),
            ("age".to_string(), Pattern::Variable("a".to_string())),
        ]);

        let mut fields = HashMap::new();
        fields.insert("name".to_string(), Value::String("Alice".to_string()));
        fields.insert("age".to_string(), Value::Int(30));
        let value = Value::Record(Box::new(fields));

        let bindings = match_pattern(&pattern, &value).unwrap();
        assert_eq!(bindings.get("n"), Some(&Value::String("Alice".to_string())));
        assert_eq!(bindings.get("a"), Some(&Value::Int(30)));
    }

    #[test]
    fn test_match_record_missing_field() {
        let pattern = Pattern::Record(vec![(
            "missing".to_string(),
            Pattern::Variable("x".to_string()),
        )]);

        let mut fields = HashMap::new();
        fields.insert("present".to_string(), Value::Int(1));
        let value = Value::Record(Box::new(fields));

        assert!(match_pattern(&pattern, &value).is_err());
    }

    #[test]
    fn test_match_record_not_record() {
        let pattern = Pattern::Record(vec![]);
        assert!(match_pattern(&pattern, &Value::Int(42)).is_err());
    }

    #[test]
    fn test_match_list_exact() {
        let pattern = Pattern::List(
            vec![
                Pattern::Variable("a".to_string()),
                Pattern::Variable("b".to_string()),
            ],
            None,
        );
        let value = Value::List(Box::new(vec![Value::Int(1), Value::Int(2)]));
        let bindings = match_pattern(&pattern, &value).unwrap();

        assert_eq!(bindings.get("a"), Some(&Value::Int(1)));
        assert_eq!(bindings.get("b"), Some(&Value::Int(2)));
    }

    #[test]
    fn test_match_list_with_rest() {
        let pattern = Pattern::List(
            vec![Pattern::Variable("head".to_string())],
            Some("tail".to_string()),
        );
        let value = Value::List(Box::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
        let bindings = match_pattern(&pattern, &value).unwrap();

        assert_eq!(bindings.get("head"), Some(&Value::Int(1)));
        assert_eq!(
            bindings.get("tail"),
            Some(&Value::List(Box::new(vec![Value::Int(2), Value::Int(3)])))
        );
    }

    #[test]
    fn test_match_list_too_short() {
        let pattern = Pattern::List(
            vec![
                Pattern::Variable("a".to_string()),
                Pattern::Variable("b".to_string()),
            ],
            None,
        );
        let value = Value::List(Box::new(vec![Value::Int(1)]));
        assert!(match_pattern(&pattern, &value).is_err());
    }

    #[test]
    fn test_match_list_not_list() {
        let pattern = Pattern::List(vec![], None);
        assert!(match_pattern(&pattern, &Value::Int(42)).is_err());
    }

    #[test]
    fn test_nested_pattern() {
        let pattern = Pattern::Tuple(vec![
            Pattern::Record(vec![(
                "x".to_string(),
                Pattern::Variable("inner".to_string()),
            )]),
            Pattern::Wildcard,
        ]);

        let mut fields = HashMap::new();
        fields.insert("x".to_string(), Value::Int(42));
        let value = Value::List(Box::new(vec![Value::Record(Box::new(fields)), Value::Null]));

        let bindings = match_pattern(&pattern, &value).unwrap();
        assert_eq!(bindings.get("inner"), Some(&Value::Int(42)));
    }

    #[test]
    fn test_match_empty_list() {
        let pattern = Pattern::List(vec![], None);
        let value = Value::List(Box::default());
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_match_empty_list_with_rest() {
        let pattern = Pattern::List(vec![], Some("rest".to_string()));
        let value = Value::List(Box::new(vec![Value::Int(1), Value::Int(2)]));
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert_eq!(
            bindings.get("rest"),
            Some(&Value::List(Box::new(vec![Value::Int(1), Value::Int(2)])))
        );
    }

    #[test]
    fn test_match_empty_record() {
        let pattern = Pattern::Record(vec![]);
        let value = Value::Record(Box::default());
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert!(bindings.is_empty());
    }

    // ============================================================
    // TASK-132: Pattern Matching Engine - Variant Tests
    // ============================================================

    #[test]
    fn test_match_variant_unit() {
        // None matches Pattern::Variant { name: "None", fields: None }
        let pattern = Pattern::Variant {
            name: "None".to_string(),
            fields: None,
        };
        let value = Value::Variant {
            name: "None".to_string(),
            fields: Box::new(vec![]),
        };
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_match_variant_with_fields() {
        // Some { value: 42 } matches Pattern::Variant { name: "Some", fields: [("value", Variable("x"))] }
        let pattern = Pattern::Variant {
            name: "Some".to_string(),
            fields: Some(vec![(
                "value".to_string(),
                Pattern::Variable("x".to_string()),
            )]),
        };
        let value = Value::Variant {
            name: "Some".to_string(),
            fields: Box::new(vec![("value".to_string(), Value::Int(42))]),
        };
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert_eq!(bindings.get("x"), Some(&Value::Int(42)));
    }

    #[test]
    fn test_match_variant_wrong_name() {
        // Pattern expects "Some", value is "None"
        let pattern = Pattern::Variant {
            name: "Some".to_string(),
            fields: None,
        };
        let value = Value::Variant {
            name: "None".to_string(),
            fields: Box::new(vec![]),
        };
        assert!(match_pattern(&pattern, &value).is_err());
    }

    #[test]
    fn test_match_variant_unit_vs_fields() {
        // Pattern expects unit variant, value has fields
        let pattern = Pattern::Variant {
            name: "Some".to_string(),
            fields: None,
        };
        let value = Value::Variant {
            name: "Some".to_string(),
            fields: Box::new(vec![("value".to_string(), Value::Int(42))]),
        };
        assert!(match_pattern(&pattern, &value).is_err());
    }

    #[test]
    fn test_match_variant_missing_field() {
        // Pattern expects field "value", value doesn't have it
        let pattern = Pattern::Variant {
            name: "Point".to_string(),
            fields: Some(vec![(
                "value".to_string(),
                Pattern::Variable("x".to_string()),
            )]),
        };
        let value = Value::Variant {
            name: "Point".to_string(),
            fields: Box::new(vec![("x".to_string(), Value::Int(1))]),
        };
        assert!(match_pattern(&pattern, &value).is_err());
    }

    #[test]
    fn test_match_variant_not_variant() {
        // Pattern expects variant, value is Int
        let pattern = Pattern::Variant {
            name: "Some".to_string(),
            fields: None,
        };
        assert!(match_pattern(&pattern, &Value::Int(42)).is_err());
    }

    #[test]
    fn test_match_variant_nested() {
        // Nested variant pattern: Some { value: (x, y) }
        let pattern = Pattern::Variant {
            name: "Some".to_string(),
            fields: Some(vec![(
                "value".to_string(),
                Pattern::Tuple(vec![
                    Pattern::Variable("x".to_string()),
                    Pattern::Variable("y".to_string()),
                ]),
            )]),
        };
        let value = Value::Variant {
            name: "Some".to_string(),
            fields: Box::new(vec![(
                "value".to_string(),
                Value::List(Box::new(vec![Value::Int(1), Value::Int(2)])),
            )]),
        };
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert_eq!(bindings.get("x"), Some(&Value::Int(1)));
        assert_eq!(bindings.get("y"), Some(&Value::Int(2)));
    }

    #[test]
    fn test_match_variant_multiple_fields() {
        // Point { x: a, y: b }
        let pattern = Pattern::Variant {
            name: "Point".to_string(),
            fields: Some(vec![
                ("x".to_string(), Pattern::Variable("a".to_string())),
                ("y".to_string(), Pattern::Variable("b".to_string())),
            ]),
        };
        let value = Value::Variant {
            name: "Point".to_string(),
            fields: Box::new(vec![
                ("x".to_string(), Value::Int(10)),
                ("y".to_string(), Value::Int(20)),
            ]),
        };
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert_eq!(bindings.get("a"), Some(&Value::Int(10)));
        assert_eq!(bindings.get("b"), Some(&Value::Int(20)));
    }

    #[test]
    fn test_match_variant_with_literal() {
        // Some { value: 42 } - literal pattern in variant field
        let pattern = Pattern::Variant {
            name: "Some".to_string(),
            fields: Some(vec![(
                "value".to_string(),
                Pattern::Literal(Value::Int(42)),
            )]),
        };
        let value = Value::Variant {
            name: "Some".to_string(),
            fields: Box::new(vec![("value".to_string(), Value::Int(42))]),
        };
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert!(bindings.is_empty());

        // Wrong literal value should fail
        let value2 = Value::Variant {
            name: "Some".to_string(),
            fields: Box::new(vec![("value".to_string(), Value::Int(99))]),
        };
        assert!(match_pattern(&pattern, &value2).is_err());
    }

    #[test]
    fn test_match_variant_with_wildcard() {
        // Some { value: _ } - wildcard in variant field
        let pattern = Pattern::Variant {
            name: "Some".to_string(),
            fields: Some(vec![("value".to_string(), Pattern::Wildcard)]),
        };
        let value = Value::Variant {
            name: "Some".to_string(),
            fields: Box::new(vec![("value".to_string(), Value::Int(42))]),
        };
        let bindings = match_pattern(&pattern, &value).unwrap();
        assert!(bindings.is_empty());
    }
}
