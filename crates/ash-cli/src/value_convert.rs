//! JSON to Ash Value conversion utilities
//!
//! This module provides bidirectional conversion between serde_json::Value
//! and ash_core::Value types.

use ash_core::Value;

/// Convert a JSON value to an Ash Value
///
/// # Examples
///
/// ```
/// use ash_cli::value_convert::json_to_value;
/// use ash_core::Value;
///
/// let json = serde_json::json!(42);
/// let value = json_to_value(json);
/// assert_eq!(value, Value::Int(42));
/// ```
pub fn json_to_value(json: serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(u) = n.as_u64() {
                // Handle large unsigned values by converting to i64
                Value::Int(i64::try_from(u).unwrap_or(i64::MAX))
            } else {
                // For non-integer numbers, we still store as Int by truncating
                // since Ash Value doesn't have a Float variant
                Value::Int(n.as_f64().map(|f| f as i64).unwrap_or(0))
            }
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => {
            Value::List(Box::new(arr.into_iter().map(json_to_value).collect()))
        }
        serde_json::Value::Object(obj) => {
            let fields: std::collections::HashMap<String, Value> = obj
                .into_iter()
                .map(|(k, v)| (k, json_to_value(v)))
                .collect();
            Value::Record(Box::new(fields))
        }
    }
}

/// Convert an Ash Value to a JSON value
///
/// # Examples
///
/// ```
/// use ash_cli::value_convert::value_to_json;
/// use ash_core::Value;
///
/// let value = Value::Int(42);
/// let json = value_to_json(&value);
/// assert_eq!(json, serde_json::json!(42));
/// ```
pub fn value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(i) => serde_json::Value::Number((*i).into()),
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::List(items) => serde_json::Value::Array(items.iter().map(value_to_json).collect()),
        Value::Record(fields) => {
            let map: serde_json::Map<String, serde_json::Value> = fields
                .iter()
                .map(|(k, v)| (k.clone(), value_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
        Value::Time(t) => serde_json::Value::String(t.to_rfc3339()),
        Value::Ref(r) => serde_json::Value::String(format!("&{r}")),
        Value::Cap(c) => serde_json::Value::String(format!("cap:{c}")),
        Value::Variant { name, fields } => {
            let mut map = serde_json::Map::new();
            map.insert(
                "_variant".to_string(),
                serde_json::Value::String(name.clone()),
            );
            for (k, v) in fields.iter() {
                map.insert(k.clone(), value_to_json(v));
            }
            serde_json::Value::Object(map)
        }
        Value::Instance(inst) => {
            let mut map = serde_json::Map::new();
            map.insert(
                "workflow_type".to_string(),
                serde_json::Value::String(inst.addr.workflow_type.clone()),
            );
            serde_json::Value::Object(map)
        }
        Value::InstanceAddr(addr) => {
            serde_json::Value::String(format!("InstanceAddr<{}>", addr.workflow_type))
        }
        Value::ControlLink(link) => {
            serde_json::Value::String(format!("ControlLink<{:?}>", link.instance_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_null() {
        assert_eq!(json_to_value(serde_json::Value::Null), Value::Null);
    }

    #[test]
    fn test_json_bool() {
        assert_eq!(json_to_value(serde_json::json!(true)), Value::Bool(true));
        assert_eq!(json_to_value(serde_json::json!(false)), Value::Bool(false));
    }

    #[test]
    fn test_json_int() {
        assert_eq!(json_to_value(serde_json::json!(42)), Value::Int(42));
        assert_eq!(json_to_value(serde_json::json!(-100)), Value::Int(-100));
    }

    #[test]
    fn test_json_string() {
        assert_eq!(
            json_to_value(serde_json::json!("hello")),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_json_array() {
        let json = serde_json::json!([1, 2, 3]);
        let value = json_to_value(json);
        match value {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::Int(1));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_json_object() {
        let json = serde_json::json!({"a": 1, "b": "test"});
        let value = json_to_value(json);
        match value {
            Value::Record(fields) => {
                assert_eq!(fields.get("a"), Some(&Value::Int(1)));
                assert_eq!(fields.get("b"), Some(&Value::String("test".to_string())));
            }
            _ => panic!("Expected Record"),
        }
    }

    #[test]
    fn test_value_null_to_json() {
        assert_eq!(value_to_json(&Value::Null), serde_json::Value::Null);
    }

    #[test]
    fn test_value_int_to_json() {
        assert_eq!(value_to_json(&Value::Int(42)), serde_json::json!(42));
    }

    #[test]
    fn test_roundtrip() {
        let original = serde_json::json!({
            "name": "test",
            "count": 42,
            "active": true,
            "items": [1, 2, 3],
            "nested": {"a": 1}
        });

        let value = json_to_value(original.clone());
        let back = value_to_json(&value);

        assert_eq!(original, back);
    }
}
