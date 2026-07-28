//! JSON to Ash Value conversion utilities
//!
//! This module provides bidirectional conversion between serde_json::Value
//! and ash_core::Value types.

use ash_core::Value;
use ash_engine::CanonicalTerminalEnvelopeV1;

/// Version of the canonical CLI terminal-observable JSON schema.
pub const CANONICAL_TERMINAL_SCHEMA_VERSION: u64 = 1;

/// Canonical terminal observable emitted by the CLI projection boundary.
///
/// This intentionally carries only language-level terminal information. It
/// excludes trace, session, and runtime-artifact telemetry, which remain
/// separate diagnostic or artifact outputs.
#[derive(Debug, Clone, PartialEq)]
pub enum CanonicalTerminalObservable {
    /// Normal terminal completion.
    Return { value: Value },
    /// Structured terminal trap.
    Trap { reason: String },
    /// Failure before the entry workflow began execution.
    PreEntryFailure { class: String, message: String },
    /// Named result at a deliberately bounded external boundary.
    External { boundary: String, outcome: String },
}

/// Convert a canonical terminal observable to its stable JSON envelope.
///
/// This is distinct from [`value_to_json`]: its value projection represents
/// variants with `constructor` and `fields`, while `_variant` remains solely a
/// legacy compatibility tag for direct value serialization.
#[must_use]
pub fn canonical_terminal_observable_to_json(
    observable: &CanonicalTerminalObservable,
) -> serde_json::Value {
    match observable {
        CanonicalTerminalObservable::Return { value } => serde_json::json!({
            "schema_version": CANONICAL_TERMINAL_SCHEMA_VERSION,
            "kind": "return",
            "value": canonical_value_to_json(value),
        }),
        CanonicalTerminalObservable::Trap { reason } => serde_json::json!({
            "schema_version": CANONICAL_TERMINAL_SCHEMA_VERSION,
            "kind": "trap",
            "reason": reason,
        }),
        CanonicalTerminalObservable::PreEntryFailure { class, message } => serde_json::json!({
            "schema_version": CANONICAL_TERMINAL_SCHEMA_VERSION,
            "kind": "pre_entry_failure",
            "class": class,
            "message": message,
        }),
        CanonicalTerminalObservable::External { boundary, outcome } => serde_json::json!({
            "schema_version": CANONICAL_TERMINAL_SCHEMA_VERSION,
            "kind": "external",
            "boundary": boundary,
            "outcome": outcome,
        }),
    }
}

/// Convert an Engine V1 terminal envelope to the stable CLI observable.
///
/// The projection is mechanical: it adds no execution route, source selector,
/// or client-specific terminal classification.  CLI clients with different
/// transport or presentation boundaries use this function to expose the same
/// Engine result.
#[must_use]
pub fn canonical_terminal_envelope_to_observable(
    envelope: &CanonicalTerminalEnvelopeV1,
) -> CanonicalTerminalObservable {
    match envelope {
        CanonicalTerminalEnvelopeV1::Returned(value) => CanonicalTerminalObservable::Return {
            value: value.clone(),
        },
        CanonicalTerminalEnvelopeV1::Trapped(reason) => CanonicalTerminalObservable::Trap {
            reason: reason.clone(),
        },
        CanonicalTerminalEnvelopeV1::AdmissionRejected => CanonicalTerminalObservable::External {
            boundary: "admission".to_string(),
            outcome: "rejected".to_string(),
        },
        CanonicalTerminalEnvelopeV1::InvalidCheckedArtifact => {
            CanonicalTerminalObservable::PreEntryFailure {
                class: "entry_verification".to_string(),
                message: "checked Core/CPS artifact is invalid".to_string(),
            }
        }
        CanonicalTerminalEnvelopeV1::TimedOut => CanonicalTerminalObservable::External {
            boundary: "execution".to_string(),
            outcome: "timeout".to_string(),
        },
        CanonicalTerminalEnvelopeV1::Cancelled => CanonicalTerminalObservable::External {
            boundary: "execution".to_string(),
            outcome: "cancelled".to_string(),
        },
    }
}

/// Convert an Engine V1 terminal envelope to the stable CLI JSON observation.
///
/// This serializes the shared mechanical observable projection used by both
/// `ash run` and daemon transport.
#[must_use]
pub fn canonical_terminal_envelope_to_json(
    envelope: &CanonicalTerminalEnvelopeV1,
) -> serde_json::Value {
    canonical_terminal_observable_to_json(&canonical_terminal_envelope_to_observable(envelope))
}

fn canonical_value_to_json(value: &Value) -> serde_json::Value {
    match value {
        value if value.is_list() => serde_json::Value::Array(
            value
                .list_to_vec()
                .expect("is_list only returns true for convertible lists")
                .iter()
                .map(canonical_value_to_json)
                .collect(),
        ),
        Value::Record(fields) => serde_json::Value::Object(
            fields
                .iter()
                .map(|(key, value)| (key.clone(), canonical_value_to_json(value)))
                .collect(),
        ),
        Value::Variant { name, fields } => serde_json::json!({
            "constructor": name,
            "fields": fields
                .iter()
                .map(|(key, value)| (key.clone(), canonical_value_to_json(value)))
                .collect::<serde_json::Map<_, _>>(),
        }),
        other => value_to_json(other),
    }
}

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
            Value::list_from_vec(arr.into_iter().map(json_to_value).collect())
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
        value if value.is_list() => serde_json::Value::Array(
            value
                .list_to_vec()
                .expect("is_list only returns true for convertible lists")
                .iter()
                .map(value_to_json)
                .collect(),
        ),
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
                "entry_type".to_string(),
                serde_json::Value::String(inst.addr.entry_type.clone()),
            );
            serde_json::Value::Object(map)
        }
        Value::InstanceAddr(addr) => {
            serde_json::Value::String(format!("InstanceAddr<{}>", addr.entry_type))
        }
        Value::ControlLink(link) => {
            serde_json::Value::String(format!("ControlLink<{:?}>", link.instance_id))
        }
        Value::Float(f) => serde_json::Value::Number(
            serde_json::Number::from_f64(*f).unwrap_or_else(|| serde_json::Number::from(0)),
        ),
        Value::Stream(handle) => serde_json::Value::String(format!("Stream<{}>", handle.id)),
        Value::ProcessHandle(handle) => serde_json::Value::String(format!(
            "P<{:?}:{}>",
            handle.process_id,
            handle.result_type.as_deref().unwrap_or("_")
        )),
        Value::ProcAwaitCapture(handle) => serde_json::Value::String(format!(
            "<proc-await:{:?}:{}>",
            handle.process_id,
            handle.result_type.as_deref().unwrap_or("_")
        )),
        Value::ProcYieldCapture => serde_json::Value::String("<proc-yield>".to_string()),
        Value::ProcParCapture { .. } => serde_json::Value::String("<proc-par>".to_string()),
        Value::ProcScatterCapture { .. } => serde_json::Value::String("<proc-scatter>".to_string()),
        Value::ProcJoinCapture { .. } => serde_json::Value::String("<proc-join>".to_string()),
        Value::ProcGatherCapture { .. } => serde_json::Value::String("<proc-gather>".to_string()),
        Value::Closure { params, .. } => {
            serde_json::Value::String(format!("<closure({} params)>", params.len()))
        }
        Value::ActEnvToken => serde_json::Value::String("<act-env-token>".to_string()),
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
        let items = value.list_to_vec().expect("Expected List");
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::Int(1));
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
