//! Runtime values

use crate::adt::is_tuple_field_name;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// Opaque affine process handle runtime value.
#[derive(Debug, Clone)]
pub struct ProcessHandle {
    pub handle_id: uuid::Uuid,
    pub process_id: crate::ProcessId,
    pub result_type: Option<String>,
    consumed: Arc<AtomicBool>,
}

impl ProcessHandle {
    #[must_use]
    pub fn new(process_id: crate::ProcessId, result_type: Option<String>) -> Self {
        Self {
            handle_id: uuid::Uuid::new_v4(),
            process_id,
            result_type,
            consumed: Arc::new(AtomicBool::new(false)),
        }
    }

    #[must_use]
    pub fn is_consumed(&self) -> bool {
        self.consumed.load(Ordering::SeqCst)
    }

    pub fn try_consume(&self) -> bool {
        self.consumed
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }
}

impl PartialEq for ProcessHandle {
    fn eq(&self, other: &Self) -> bool {
        self.handle_id == other.handle_id
            && self.process_id == other.process_id
            && self.result_type == other.result_type
            && self.is_consumed() == other.is_consumed()
    }
}

/// Instance address - opaque reference to a workflow instance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstanceAddr {
    pub workflow_type: String,
    pub instance_id: crate::WorkflowId,
}

/// Control link for controlling a spawned instance.
///
/// A control link represents reusable supervision authority while the target instance remains
/// valid. Terminal control operations may invalidate future use for that instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlLink {
    pub instance_id: crate::WorkflowId,
}

/// Instance composite type - returned by spawn, can be split into addr and control
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instance {
    pub addr: InstanceAddr,
    pub control: Option<ControlLink>,
}

/// Runtime values in Ash
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Integer
    Int(i64),
    /// Float (64-bit floating point)
    Float(f64),
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
    /// List of values (boxed to reduce enum size)
    List(Box<Vec<Value>>),
    /// Record (map) (boxed to reduce enum size)
    Record(Box<HashMap<String, Value>>),
    /// Capability reference
    Cap(String),
    /// Variant/Constructor value (ADT)
    /// e.g., `Some { value: 42 }` is `Variant { name: "Some", fields: [("value", Int(42))] }`
    /// and `None` is `Variant { name: "None", fields: [] }`
    Variant {
        /// Constructor name (e.g., "Some", "None", "Ok")
        name: String,
        /// Field values as (name, value) pairs (boxed to reduce enum size)
        fields: Box<Vec<(String, Value)>>,
    },
    /// Instance value - composite of addr and optional control link (boxed to reduce enum size)
    Instance(Box<Instance>),
    /// Instance address value (opaque reference to an instance)
    InstanceAddr(InstanceAddr),
    /// Control link value for controlling spawned instances
    ControlLink(ControlLink),
    /// Stream handle for consuming streaming data
    ///
    /// Streams are used for incremental data sources like chat completions
    /// where data arrives in chunks over time.
    Stream(StreamHandle),
    /// Opaque affine process handle value.
    ProcessHandle(ProcessHandle),
    /// Hidden runtime-only Proc await capture marker.
    ProcAwaitCapture(ProcessHandle),
    /// Hidden runtime-only Proc scheduler yield marker.
    ProcYieldCapture,
    /// Runtime closure value. SPEC-031 §5.2
    /// NOT serializable -- manual serde implementation will error on this variant.
    Closure {
        params: Vec<(String, Option<String>)>,
        body: Box<crate::ast::Expr>,
        env: std::sync::Arc<crate::env_frame::EnvFrame>,
    },
    /// Hidden runtime-only Act environment carrier token.
    ActEnvToken,
}

/// Handle to a stream that can be consumed incrementally
///
/// Stream handles are created by providers (like LLM chat_stream) and
/// consumed by the runtime through the receive construct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamHandle {
    /// Unique identifier for this stream
    pub id: String,
    /// Type of items in the stream (for type checking)
    pub item_type: String,
}

impl StreamHandle {
    /// Create a new stream handle
    pub fn new(id: impl Into<String>, item_type: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            item_type: item_type.into(),
        }
    }
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

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_stream(&self) -> Option<&StreamHandle> {
        match self {
            Value::Stream(handle) => Some(handle),
            _ => None,
        }
    }

    pub fn as_process_handle(&self) -> Option<&ProcessHandle> {
        match self {
            Value::ProcessHandle(handle) => Some(handle),
            _ => None,
        }
    }

    /// Create a new variant value with the given name and fields
    pub fn variant(name: impl Into<String>, fields: Vec<(impl Into<String>, Value)>) -> Self {
        Value::Variant {
            name: name.into(),
            fields: Box::new(fields.into_iter().map(|(k, v)| (k.into(), v)).collect()),
        }
    }

    /// Create a new variant value with no fields (unit variant)
    pub fn unit_variant(name: impl Into<String>) -> Self {
        Value::Variant {
            name: name.into(),
            fields: Box::new(vec![]),
        }
    }

    fn variant_fields_are_tuple_payload(fields: &[(String, Value)]) -> bool {
        !fields.is_empty()
            && fields
                .iter()
                .enumerate()
                .all(|(index, (field_name, _))| is_tuple_field_name(field_name, index))
    }
}

impl std::fmt::Display for InstanceAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "InstanceAddr<{}:{:?}>",
            self.workflow_type, self.instance_id
        )
    }
}

impl std::fmt::Display for ControlLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ControlLink<{:?}>", self.instance_id)
    }
}

impl std::fmt::Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Instance {{")?;
        write!(f, " addr: {}", self.addr)?;
        match &self.control {
            Some(ctrl) => write!(f, ", control: Some({})", ctrl)?,
            None => write!(f, ", control: None")?,
        }
        write!(f, " }}")
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
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
            Value::Variant { name, fields } => {
                write!(f, "{}", name)?;
                if !fields.is_empty() {
                    if Self::variant_fields_are_tuple_payload(fields) {
                        write!(f, "(")?;
                        for (i, (_, v)) in fields.iter().enumerate() {
                            if i > 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "{}", v)?;
                        }
                        write!(f, ")")?;
                    } else {
                        write!(f, " {{")?;
                        for (i, (k, v)) in fields.iter().enumerate() {
                            if i > 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "{}: {}", k, v)?;
                        }
                        write!(f, "}}")?;
                    }
                }
                Ok(())
            }
            Value::Instance(instance) => write!(f, "{}", instance),
            Value::InstanceAddr(addr) => write!(f, "{}", addr),
            Value::ControlLink(link) => write!(f, "{}", link),
            Value::Stream(handle) => write!(f, "stream({}: {})", handle.id, handle.item_type),
            Value::ProcessHandle(handle) => write!(f, "P<{}>", handle.process_id.0),
            Value::ProcAwaitCapture(handle) => write!(f, "<proc-await:{}>", handle.process_id.0),
            Value::ProcYieldCapture => write!(f, "<proc-yield>"),
            Value::Closure { params, .. } => {
                write!(f, "<closure({})>", params.len())
            }
            Value::ActEnvToken => write!(f, "<act-env>"),
        }
    }
}

// Manual Serialize/Deserialize because Value::Closure cannot be serialized.
// All other variants delegate to the derived implementations via a helper enum.

impl Serialize for Value {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Value::Closure { .. }
            | Value::ActEnvToken
            | Value::ProcessHandle(_)
            | Value::ProcAwaitCapture(_)
            | Value::ProcYieldCapture => Err(serde::ser::Error::custom(
                "runtime-only value cannot be serialized",
            )),
            Value::Int(v) => {
                serde::Serialize::serialize(&serde_helper::SerializableValue::Int(*v), serializer)
            }
            Value::Float(v) => {
                serde::Serialize::serialize(&serde_helper::SerializableValue::Float(*v), serializer)
            }
            Value::String(v) => serde::Serialize::serialize(
                &serde_helper::SerializableValue::String(v.clone()),
                serializer,
            ),
            Value::Bool(v) => {
                serde::Serialize::serialize(&serde_helper::SerializableValue::Bool(*v), serializer)
            }
            Value::Null => {
                serde::Serialize::serialize(&serde_helper::SerializableValue::Null, serializer)
            }
            Value::Time(v) => {
                serde::Serialize::serialize(&serde_helper::SerializableValue::Time(*v), serializer)
            }
            Value::Ref(v) => serde::Serialize::serialize(
                &serde_helper::SerializableValue::Ref(v.clone()),
                serializer,
            ),
            Value::List(v) => serde::Serialize::serialize(
                &serde_helper::SerializableValue::List(v.clone()),
                serializer,
            ),
            Value::Record(v) => serde::Serialize::serialize(
                &serde_helper::SerializableValue::Record(v.clone()),
                serializer,
            ),
            Value::Cap(v) => serde::Serialize::serialize(
                &serde_helper::SerializableValue::Cap(v.clone()),
                serializer,
            ),
            Value::Variant { name, fields } => serde::Serialize::serialize(
                &serde_helper::SerializableValue::Variant {
                    name: name.clone(),
                    fields: fields.clone(),
                },
                serializer,
            ),
            Value::Instance(v) => serde::Serialize::serialize(
                &serde_helper::SerializableValue::Instance(v.clone()),
                serializer,
            ),
            Value::InstanceAddr(v) => serde::Serialize::serialize(
                &serde_helper::SerializableValue::InstanceAddr(v.clone()),
                serializer,
            ),
            Value::ControlLink(v) => serde::Serialize::serialize(
                &serde_helper::SerializableValue::ControlLink(v.clone()),
                serializer,
            ),
            Value::Stream(v) => serde::Serialize::serialize(
                &serde_helper::SerializableValue::Stream(v.clone()),
                serializer,
            ),
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let sv = serde_helper::SerializableValue::deserialize(deserializer)?;
        Ok(match sv {
            serde_helper::SerializableValue::Int(v) => Value::Int(v),
            serde_helper::SerializableValue::Float(v) => Value::Float(v),
            serde_helper::SerializableValue::String(v) => Value::String(v),
            serde_helper::SerializableValue::Bool(v) => Value::Bool(v),
            serde_helper::SerializableValue::Null => Value::Null,
            serde_helper::SerializableValue::Time(v) => Value::Time(v),
            serde_helper::SerializableValue::Ref(v) => Value::Ref(v),
            serde_helper::SerializableValue::List(v) => Value::List(v),
            serde_helper::SerializableValue::Record(v) => Value::Record(v),
            serde_helper::SerializableValue::Cap(v) => Value::Cap(v),
            serde_helper::SerializableValue::Variant { name, fields } => {
                Value::Variant { name, fields }
            }
            serde_helper::SerializableValue::Instance(v) => Value::Instance(v),
            serde_helper::SerializableValue::InstanceAddr(v) => Value::InstanceAddr(v),
            serde_helper::SerializableValue::ControlLink(v) => Value::ControlLink(v),
            serde_helper::SerializableValue::Stream(v) => Value::Stream(v),
        })
    }
}

mod serde_helper {
    use super::*;
    #[allow(clippy::box_collection)]
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum SerializableValue {
        Int(i64),
        Float(f64),
        String(String),
        Bool(bool),
        Null,
        Time(chrono::DateTime<chrono::Utc>),
        Ref(String),
        List(Box<Vec<Value>>),
        Record(Box<HashMap<String, Value>>),
        Cap(String),
        Variant {
            name: String,
            fields: Box<Vec<(String, Value)>>,
        },
        Instance(Box<Instance>),
        InstanceAddr(InstanceAddr),
        ControlLink(ControlLink),
        Stream(StreamHandle),
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
                    prop::collection::vec(inner.clone(), 0..8)
                        .prop_map(|v| Value::List(Box::new(v))),
                    prop::collection::hash_map("[a-z]+".prop_map(String::from), inner, 0..8)
                        .prop_map(|m| Value::Record(Box::new(m))),
                ]
            },
        )
    }

    #[test]
    fn value_enum_shape_remains_closed_to_runtime_only_types() {
        fn classify(value: Value) -> &'static str {
            match value {
                Value::Int(_) => "Int",
                Value::Float(_) => "Float",
                Value::String(_) => "String",
                Value::Bool(_) => "Bool",
                Value::Null => "Null",
                Value::Time(_) => "Time",
                Value::Ref(_) => "Ref",
                Value::List(_) => "List",
                Value::Record(_) => "Record",
                Value::Cap(_) => "Cap",
                Value::Variant { .. } => "Variant",
                Value::Instance(_) => "Instance",
                Value::InstanceAddr(_) => "InstanceAddr",
                Value::ControlLink(_) => "ControlLink",
                Value::Stream(_) => "Stream",
                Value::ProcessHandle(_) => "ProcessHandle",
                Value::ProcAwaitCapture(_) => "ProcAwaitCapture",
                Value::ProcYieldCapture => "ProcYieldCapture",
                Value::Closure { .. } => "Closure",
                Value::ActEnvToken => "ActEnvToken",
            }
        }

        assert_eq!(classify(Value::Null), "Null");
        assert_eq!(classify(Value::ActEnvToken), "ActEnvToken");
        assert_eq!(
            classify(Value::ProcessHandle(ProcessHandle::new(
                crate::ProcessId::new(),
                Some("Int".to_string())
            ))),
            "ProcessHandle"
        );
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
