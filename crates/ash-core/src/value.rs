//! Runtime values

use crate::adt::is_tuple_field_name;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// Convert an effect level string to a numeric rank for comparison.
/// Pure = 0, Act = 1, Proc = 2, Workflow = 3.
pub fn effect_level_rank(level: &str) -> u8 {
    match level {
        "Pure" => 0,
        "Act" => 1,
        "Proc" => 2,
        "Workflow" => 3,
        _ => 0, // Default to Pure for unknown
    }
}

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

/// Structured reason a runtime value cannot cross a process boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendabilityRejection {
    /// Runtime closures capture handler/environment state and are not process payloads.
    Closure,
    /// External references are borrowed resource authority, not owned transferable data.
    BorrowedResource,
    /// Capability references carry authority and must not be copied across process boundaries.
    Capability,
    /// Workflow instance values carry live workflow authority.
    WorkflowInstance,
    /// Workflow instance addresses are runtime-local references.
    WorkflowInstanceAddress,
    /// Control links are reusable supervision authority.
    ControlLink,
    /// Stream handles carry live consumer state.
    StreamHandle,
    /// A process handle was already consumed by an affine operation.
    ConsumedProcessHandle { process_id: crate::ProcessId },
    /// Hidden runtime marker or handler frame carrier.
    RuntimeToken(&'static str),
    /// A nested payload failed validation at the reported field path.
    AtPath {
        path: String,
        reason: Box<SendabilityRejection>,
    },
}

impl SendabilityRejection {
    /// Attach a field path to a nested sendability rejection.
    #[must_use]
    pub fn at_path(path: impl Into<String>, reason: SendabilityRejection) -> SendabilityRejection {
        let path = path.into();
        match reason {
            SendabilityRejection::AtPath {
                path: nested_path,
                reason,
            } => SendabilityRejection::AtPath {
                path: format!("{path}.{nested_path}"),
                reason,
            },
            reason => SendabilityRejection::AtPath {
                path,
                reason: Box::new(reason),
            },
        }
    }
}

impl std::fmt::Display for SendabilityRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SendabilityRejection::Closure => {
                write!(f, "closure captures cannot cross process boundaries")
            }
            SendabilityRejection::BorrowedResource => {
                write!(
                    f,
                    "borrowed resource references cannot cross process boundaries"
                )
            }
            SendabilityRejection::Capability => {
                write!(f, "capabilities cannot cross process boundaries")
            }
            SendabilityRejection::WorkflowInstance => {
                write!(f, "workflow instances cannot cross process boundaries")
            }
            SendabilityRejection::WorkflowInstanceAddress => {
                write!(
                    f,
                    "workflow instance addresses cannot cross process boundaries"
                )
            }
            SendabilityRejection::ControlLink => {
                write!(f, "control links cannot cross process boundaries")
            }
            SendabilityRejection::StreamHandle => {
                write!(f, "stream handles cannot cross process boundaries")
            }
            SendabilityRejection::ConsumedProcessHandle { process_id } => {
                write!(
                    f,
                    "process handle for {} was already consumed",
                    process_id.0
                )
            }
            SendabilityRejection::RuntimeToken(token) => {
                write!(f, "runtime token `{token}` cannot cross process boundaries")
            }
            SendabilityRejection::AtPath { path, reason } => write!(f, "{path}: {reason}"),
        }
    }
}

impl std::error::Error for SendabilityRejection {}

/// Instance address - opaque reference to a workflow instance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstanceAddr {
    pub entry_type: String,
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
    /// Hidden runtime-only Proc two-child admission marker.
    ProcParCapture { left: Box<Value>, right: Box<Value> },
    /// Hidden runtime-only Proc scatter admission marker.
    ProcScatterCapture {
        items: Box<Vec<Value>>,
        mapper: Box<Value>,
    },
    /// Hidden runtime-only Proc two-handle wait-for-all observation marker.
    ProcJoinCapture {
        left: ProcessHandle,
        right: ProcessHandle,
    },
    /// Hidden runtime-only Proc ordered handle-list wait-for-all observation marker.
    ProcGatherCapture { handles: Box<Vec<ProcessHandle>> },
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

    /// Return true when this value may be transferred across a process boundary.
    ///
    /// This is stricter than Rust's `Send`: Ash process payloads must be owned data or affine
    /// process handles, not captured closures, borrowed resources, handler frames, or live runtime
    /// authority.
    #[must_use]
    pub fn is_sendable_across_process_boundary(&self) -> bool {
        self.validate_sendable_for_process_boundary().is_ok()
    }

    /// Validate that this value may be transferred across a process boundary.
    pub fn validate_sendable_for_process_boundary(&self) -> Result<(), SendabilityRejection> {
        match self {
            Value::Int(_)
            | Value::Float(_)
            | Value::String(_)
            | Value::Bool(_)
            | Value::Null
            | Value::Time(_) => Ok(()),
            Value::Record(fields) => {
                let mut names: Vec<_> = fields.keys().collect();
                names.sort();

                for name in names {
                    if let Some(value) = fields.get(name) {
                        value
                            .validate_sendable_for_process_boundary()
                            .map_err(|reason| SendabilityRejection::at_path(name, reason))?;
                    }
                }

                Ok(())
            }
            Value::Variant { fields, .. } => {
                for (name, value) in fields.iter() {
                    value
                        .validate_sendable_for_process_boundary()
                        .map_err(|reason| SendabilityRejection::at_path(name, reason))?;
                }

                Ok(())
            }
            Value::ProcessHandle(handle) if !handle.is_consumed() => Ok(()),
            Value::ProcessHandle(handle) => Err(SendabilityRejection::ConsumedProcessHandle {
                process_id: handle.process_id,
            }),
            Value::Ref(_) => Err(SendabilityRejection::BorrowedResource),
            Value::Cap(_) => Err(SendabilityRejection::Capability),
            Value::Instance(_) => Err(SendabilityRejection::WorkflowInstance),
            Value::InstanceAddr(_) => Err(SendabilityRejection::WorkflowInstanceAddress),
            Value::ControlLink(_) => Err(SendabilityRejection::ControlLink),
            Value::Stream(_) => Err(SendabilityRejection::StreamHandle),
            Value::ProcAwaitCapture(_) => Err(SendabilityRejection::RuntimeToken("proc-await")),
            Value::ProcYieldCapture => Err(SendabilityRejection::RuntimeToken("proc-yield")),
            Value::ProcParCapture { .. } => Err(SendabilityRejection::RuntimeToken("proc-par")),
            Value::ProcScatterCapture { .. } => {
                Err(SendabilityRejection::RuntimeToken("proc-scatter"))
            }
            Value::ProcJoinCapture { .. } => Err(SendabilityRejection::RuntimeToken("proc-join")),
            Value::ProcGatherCapture { .. } => {
                Err(SendabilityRejection::RuntimeToken("proc-gather"))
            }
            Value::Closure { .. } => Err(SendabilityRejection::Closure),
            Value::ActEnvToken => Err(SendabilityRejection::RuntimeToken("act-env")),
        }
    }

    /// Return true if this value is pure (no effect level).
    /// Pure values: Int, Float, String, Bool, Null, Time, pure Record, pure Variant, pure Closure.
    pub fn is_pure(&self) -> bool {
        match self {
            Value::Int(_)
            | Value::Float(_)
            | Value::String(_)
            | Value::Bool(_)
            | Value::Null
            | Value::Time(_) => true,
            Value::Record(fields) => fields.values().all(|v| v.is_pure()),
            Value::Variant { fields, .. } => fields.iter().all(|(_, v)| v.is_pure()),
            Value::Closure { env, .. } => {
                // Closure is pure if all captured bindings are pure
                env.all_bindings().all(|(_, v)| v.is_pure())
            }
            // Everything else is effectful
            _ => false,
        }
    }

    /// Return the effect level of this value as a string.
    pub fn effect_level(&self) -> String {
        match self {
            Value::Int(_)
            | Value::Float(_)
            | Value::String(_)
            | Value::Bool(_)
            | Value::Null
            | Value::Time(_) => "Pure".to_string(),
            Value::Record(fields) => {
                let max_effect = fields
                    .values()
                    .map(|v| v.effect_level())
                    .max_by_key(|e| crate::value::effect_level_rank(e));
                max_effect.unwrap_or_else(|| "Pure".to_string())
            }
            Value::Variant { fields, .. } => {
                let max_effect = fields
                    .iter()
                    .map(|(_, v)| v.effect_level())
                    .max_by_key(|e| crate::value::effect_level_rank(e));
                max_effect.unwrap_or_else(|| "Pure".to_string())
            }
            Value::Cap(_) => "Act".to_string(),
            Value::Closure { .. } => {
                if self.is_pure() {
                    "Pure".to_string()
                } else {
                    "Act".to_string()
                }
            }
            Value::ProcessHandle(_)
            | Value::ProcAwaitCapture(_)
            | Value::ProcYieldCapture
            | Value::ProcParCapture { .. }
            | Value::ProcScatterCapture { .. }
            | Value::ProcJoinCapture { .. }
            | Value::ProcGatherCapture { .. } => "Proc".to_string(),
            Value::Instance(_) | Value::InstanceAddr(_) | Value::ControlLink(_) => {
                "Workflow".to_string()
            }
            Value::Stream(_) => "Act".to_string(),
            Value::ActEnvToken => "Act".to_string(),
            Value::Ref(_) => "Pure".to_string(),
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

    /// Create the canonical empty list runtime value.
    #[must_use]
    pub fn list_nil() -> Self {
        Self::unit_variant("Nil")
    }

    /// Create the canonical non-empty list runtime value.
    #[must_use]
    pub fn list_cons(head: Value, tail: Value) -> Self {
        Self::variant("Cons", vec![("head", head), ("tail", tail)])
    }

    /// Convert a vector into the canonical nested `Cons`/`Nil` runtime list.
    #[must_use]
    pub fn list_from_vec(values: Vec<Value>) -> Self {
        values
            .into_iter()
            .rev()
            .fold(Self::list_nil(), |tail, head| Self::list_cons(head, tail))
    }

    /// Convert a canonical nested `Cons`/`Nil` runtime list into a vector.
    #[must_use]
    pub fn is_list(&self) -> bool {
        match self {
            Value::Variant { name, fields } if name == "Nil" => fields.is_empty(),
            Value::Variant { name, fields } if name == "Cons" => {
                let Some(tail) = fields
                    .iter()
                    .find(|(field, _)| field == "tail")
                    .map(|(_, v)| v)
                else {
                    return false;
                };
                tail.is_list()
            }
            _ => false,
        }
    }

    /// Convert a canonical nested `Cons`/`Nil` runtime list into a vector.
    #[must_use]
    pub fn list_to_vec(&self) -> Option<Vec<Value>> {
        let mut result = Vec::new();
        let mut current = self;
        loop {
            match current {
                Value::Variant { name, fields } if name == "Nil" && fields.is_empty() => {
                    return Some(result);
                }
                Value::Variant { name, fields } if name == "Cons" => {
                    let head = fields
                        .iter()
                        .find(|(field, _)| field == "head")
                        .map(|(_, value)| value)?;
                    let tail = fields
                        .iter()
                        .find(|(field, _)| field == "tail")
                        .map(|(_, value)| value)?;
                    result.push(head.clone());
                    current = tail;
                }
                _ => return None,
            }
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
            self.entry_type, self.instance_id
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
            list if list.is_list() => {
                write!(f, "[")?;
                for (i, v) in list
                    .list_to_vec()
                    .expect("is_list only returns true for convertible lists")
                    .iter()
                    .enumerate()
                {
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
            Value::ProcParCapture { .. } => write!(f, "<proc-par>"),
            Value::ProcScatterCapture { .. } => write!(f, "<proc-scatter>"),
            Value::ProcJoinCapture { .. } => write!(f, "<proc-join>"),
            Value::ProcGatherCapture { .. } => write!(f, "<proc-gather>"),
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
            | Value::ProcYieldCapture
            | Value::ProcParCapture { .. }
            | Value::ProcScatterCapture { .. }
            | Value::ProcJoinCapture { .. }
            | Value::ProcGatherCapture { .. } => Err(serde::ser::Error::custom(
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
            Value::Record(v) => serde::Serialize::serialize(
                &serde_helper::SerializableValue::Record(v.clone()),
                serializer,
            ),
            Value::Cap(v) => serde::Serialize::serialize(
                &serde_helper::SerializableValue::Cap(v.clone()),
                serializer,
            ),
            Value::Variant { .. } if self.is_list() => serde::Serialize::serialize(
                &serde_helper::SerializableValue::SerializedList(Box::new(
                    self.list_to_vec()
                        .expect("is_list only returns true for convertible lists"),
                )),
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
            serde_helper::SerializableValue::SerializedList(v) => Value::list_from_vec(*v),
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
        #[serde(rename = "List")]
        SerializedList(Box<Vec<Value>>),
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
                    prop::collection::vec(inner.clone(), 0..8).prop_map(Value::list_from_vec),
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
                Value::ProcParCapture { .. } => "ProcParCapture",
                Value::ProcScatterCapture { .. } => "ProcScatterCapture",
                Value::ProcJoinCapture { .. } => "ProcJoinCapture",
                Value::ProcGatherCapture { .. } => "ProcGatherCapture",
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
