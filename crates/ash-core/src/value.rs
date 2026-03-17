//! Runtime values

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Runtime values in Ash
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// Integer
    Int(i64),
    /// String
    String(String),
    /// Boolean
    Bool(bool),
    /// Null
    Null,
    /// Timestamp
    Time(chrono::DateTime<chrono::Utc>),
    /// Reference to external resource
    Ref(String),
    /// List of values
    List(Vec<Value>),
    /// Record (map)
    Record(HashMap<String, Value>),
    /// Capability reference
    Cap(String),
}

impl Value {
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::Time(t) => write!(f, "{}", t),
            Value::Ref(r) => write!(f, "&{}", r),
            Value::List(l) => {
                write!(f, "[")?;
                for (i, v) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Record(r) => {
                write!(f, "{{")?;
                for (i, (k, v)) in r.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Cap(c) => write!(f, "cap({})", c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// Generate arbitrary Value values covering all variants
    fn arb_value() -> impl Strategy<Value = Value> {
        let leaf = prop_oneof![
            any::<i64>().prop_map(Value::Int),
            any::<bool>().prop_map(Value::Bool),
            "[a-zA-Z0-9_]*".prop_map(Value::String),
            Just(Value::Null),
            // Timestamps within reasonable range (year 2000-2100)
            (0i64..4102444800i64).prop_map(|secs| {
                Value::Time(chrono::DateTime::from_timestamp(secs, 0).unwrap_or(chrono::Utc::now()))
            }),
            "[a-zA-Z_][a-zA-Z0-9_]*".prop_map(Value::Ref),
            "[a-zA-Z_][a-zA-Z0-9_]*".prop_map(Value::Cap),
        ];

        leaf.prop_recursive(
            4,  // Depth
            64, // Max size
            8,  // Items per collection
            |inner| {
                prop_oneof![
                    prop::collection::vec(inner.clone(), 0..8).prop_map(Value::List),
                    prop::collection::hash_map("[a-z]+".prop_map(String::from), inner, 0..8)
                        .prop_map(Value::Record),
                ]
            },
        )
    }

    // Serde Roundtrip Tests
    proptest! {
        #[test]
        fn test_serde_roundtrip(v in arb_value()) {
            let serialized = serde_json::to_string(&v).expect("serialization should succeed");
            let deserialized: Value = serde_json::from_str(&serialized).expect("deserialization should succeed");
            prop_assert_eq!(v, deserialized);
        }
    }

    // Display Format Tests
    #[test]
    fn test_display_null() {
        let v = Value::Null;
        assert_eq!(format!("{}", v), "null");
    }

    proptest! {
        #[test]
        fn test_display_int(i in any::<i64>()) {
            let v = Value::Int(i);
            prop_assert_eq!(format!("{}", v), format!("{}", i));
        }

        #[test]
        fn test_display_bool(b in any::<bool>()) {
            let v = Value::Bool(b);
            prop_assert_eq!(format!("{}", v), format!("{}", b));
        }

        #[test]
        fn test_display_string(s in "[a-zA-Z0-9_]*") {
            let v = Value::String(s.clone());
            prop_assert_eq!(format!("{}", v), format!("\"{}\"", s));
        }
    }

    // Accessor Method Tests
    proptest! {
        #[test]
        fn test_as_int_returns_some_for_int(i in any::<i64>()) {
            let v = Value::Int(i);
            prop_assert_eq!(v.as_int(), Some(i));
        }

        #[test]
        fn test_as_int_returns_none_for_non_int(v in arb_value()) {
            prop_assume!(!matches!(v, Value::Int(_)));
            prop_assert_eq!(v.as_int(), None);
        }

        #[test]
        fn test_as_string_returns_some_for_string(s in "[a-zA-Z0-9_]*") {
            let v = Value::String(s.clone());
            prop_assert_eq!(v.as_string(), Some(s.as_str()));
        }

        #[test]
        fn test_as_string_returns_none_for_non_string(v in arb_value()) {
            prop_assume!(!matches!(v, Value::String(_)));
            prop_assert_eq!(v.as_string(), None);
        }

        #[test]
        fn test_as_bool_returns_some_for_bool(b in any::<bool>()) {
            let v = Value::Bool(b);
            prop_assert_eq!(v.as_bool(), Some(b));
        }

        #[test]
        fn test_as_bool_returns_none_for_non_bool(v in arb_value()) {
            prop_assume!(!matches!(v, Value::Bool(_)));
            prop_assert_eq!(v.as_bool(), None);
        }
    }

    // Equality Tests
    proptest! {
        #[test]
        fn test_equality_reflexive(v in arb_value()) {
            prop_assert_eq!(v.clone(), v);
        }

        #[test]
        fn test_equality_identical_values_are_equal(v in arb_value()) {
            let v2 = v.clone();
            prop_assert_eq!(v, v2);
        }

        #[test]
        fn test_equality_different_ints_not_equal(i1 in any::<i64>(), i2 in any::<i64>()) {
            prop_assume!(i1 != i2);
            let v1 = Value::Int(i1);
            let v2 = Value::Int(i2);
            prop_assert_ne!(v1, v2);
        }

        #[test]
        fn test_equality_different_bools_not_equal(b1 in any::<bool>(), b2 in any::<bool>()) {
            prop_assume!(b1 != b2);
            let v1 = Value::Bool(b1);
            let v2 = Value::Bool(b2);
            prop_assert_ne!(v1, v2);
        }

        #[test]
        fn test_equality_different_strings_not_equal(s1 in "[a-zA-Z0-9_]*", s2 in "[a-zA-Z0-9_]*") {
            prop_assume!(s1 != s2);
            let v1 = Value::String(s1);
            let v2 = Value::String(s2);
            prop_assert_ne!(v1, v2);
        }

        #[test]
        fn test_equality_different_types_not_equal(v1 in arb_value(), v2 in arb_value()) {
            // Check if the discriminants are different (different variant types)
            let disc1 = std::mem::discriminant(&v1);
            let disc2 = std::mem::discriminant(&v2);
            prop_assume!(disc1 != disc2);
            prop_assert_ne!(v1, v2);
        }
    }
}
