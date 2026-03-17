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
