//! Bounded typed runtime channels for process-profile execution.

use std::collections::{HashMap, VecDeque};

use ash_core::runtime::ProcessId;
use ash_core::{SendabilityRejection, Value};
use ash_typeck::Type;
use thiserror::Error;

/// Runtime identity for one in-process channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChannelId(pub ProcessId);

impl ChannelId {
    /// Create a fresh channel identity.
    #[must_use]
    pub fn new() -> Self {
        Self(ProcessId::new())
    }
}

impl Default for ChannelId {
    fn default() -> Self {
        Self::new()
    }
}

/// Structured channel runtime errors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ChannelError {
    /// Channel identity does not exist in the registry.
    #[error("channel {channel_id:?} is not registered")]
    Unknown {
        /// Channel identity.
        channel_id: ChannelId,
    },
    /// Channel is closed and cannot accept new sends.
    #[error("channel {channel_id:?} is closed")]
    Closed {
        /// Channel identity.
        channel_id: ChannelId,
    },
    /// Channel contains no message available for non-blocking receive.
    #[error("channel {channel_id:?} is empty")]
    Empty {
        /// Channel identity.
        channel_id: ChannelId,
    },
    /// Channel has reached its configured bounded capacity.
    #[error("channel {channel_id:?} is full at capacity {capacity}")]
    Full {
        /// Channel identity.
        channel_id: ChannelId,
        /// Configured channel capacity.
        capacity: usize,
    },
    /// Message payload does not match the channel type schema.
    #[error("channel {channel_id:?} expected {expected}, got {actual}")]
    TypeMismatch {
        /// Channel identity.
        channel_id: ChannelId,
        /// Expected payload type.
        expected: String,
        /// Actual payload type.
        actual: String,
    },
    /// Message payload cannot cross process/channel boundaries.
    #[error("channel {channel_id:?} rejected non-sendable payload: {reason}")]
    NonSendable {
        /// Channel identity.
        channel_id: ChannelId,
        /// Structured sendability rejection.
        reason: SendabilityRejection,
    },
    /// Select shape is not supported by the bounded channel runtime.
    #[error("unsupported channel select: {reason}")]
    UnsupportedSelect {
        /// Diagnostic reason.
        reason: String,
    },
}

#[derive(Debug, Clone)]
struct ChannelState {
    payload_type: Type,
    capacity: usize,
    queue: VecDeque<Value>,
    closed: bool,
}

/// Registry of runtime-owned typed channels.
#[derive(Debug, Default, Clone)]
pub struct ChannelRegistry {
    channels: HashMap<ChannelId, ChannelState>,
}

impl ChannelRegistry {
    /// Create an empty channel registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a bounded typed channel and return its identity.
    pub fn create(&mut self, payload_type: Type, capacity: usize) -> ChannelId {
        let id = ChannelId::new();
        self.channels.insert(
            id,
            ChannelState {
                payload_type,
                capacity,
                queue: VecDeque::with_capacity(capacity.min(1024)),
                closed: false,
            },
        );
        id
    }

    /// Send one value into a channel.
    pub fn send(&mut self, channel_id: ChannelId, value: Value) -> Result<(), ChannelError> {
        let state = self
            .channels
            .get_mut(&channel_id)
            .ok_or(ChannelError::Unknown { channel_id })?;
        if state.closed {
            return Err(ChannelError::Closed { channel_id });
        }
        if !state.payload_type.matches(&value) {
            return Err(ChannelError::TypeMismatch {
                channel_id,
                expected: state.payload_type.to_string(),
                actual: channel_value_type_name(&value).to_string(),
            });
        }
        value
            .validate_sendable_for_process_boundary()
            .map_err(|reason| ChannelError::NonSendable { channel_id, reason })?;
        if state.queue.len() >= state.capacity {
            return Err(ChannelError::Full {
                channel_id,
                capacity: state.capacity,
            });
        }

        state.queue.push_back(value);
        Ok(())
    }

    /// Try to receive one value from a channel without blocking.
    pub fn try_receive(&mut self, channel_id: ChannelId) -> Result<Value, ChannelError> {
        let state = self
            .channels
            .get_mut(&channel_id)
            .ok_or(ChannelError::Unknown { channel_id })?;
        state
            .queue
            .pop_front()
            .ok_or(ChannelError::Empty { channel_id })
    }

    /// Close a channel against future sends.
    pub fn close(&mut self, channel_id: ChannelId) -> Result<(), ChannelError> {
        let state = self
            .channels
            .get_mut(&channel_id)
            .ok_or(ChannelError::Unknown { channel_id })?;
        state.closed = true;
        Ok(())
    }

    /// Return the ready channel for supported select shapes.
    pub fn select_ready(
        &self,
        channel_ids: &[ChannelId],
    ) -> Result<Option<ChannelId>, ChannelError> {
        if channel_ids.len() != 1 {
            return Err(ChannelError::UnsupportedSelect {
                reason: "multi-channel select is not supported by the bounded channel runtime yet"
                    .to_string(),
            });
        }

        let channel_id = channel_ids[0];
        let state = self
            .channels
            .get(&channel_id)
            .ok_or(ChannelError::Unknown { channel_id })?;
        Ok((!state.queue.is_empty()).then_some(channel_id))
    }
}

fn channel_value_type_name(value: &Value) -> &'static str {
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
