//! Provenance tracking for capability operations
//!
//! This module provides audit trail tracking for observe, receive, set, and send
//! operations with policy decisions and effect tracking.
//!
//! # Example
//!
//! ```
//! use ash_interp::capability_provenance::{
//!     CapabilityProvenanceTracker, CapabilityProvenanceEvent,
//!     CapabilityEventType, Direction
//! };
//! use ash_core::{Value, Effect};
//! use chrono::Utc;
//!
//! let mut tracker = CapabilityProvenanceTracker::new();
//!
//! tracker.record(CapabilityProvenanceEvent {
//!     event_type: CapabilityEventType::Observed,
//!     direction: Direction::Input,
//!     capability: "sensor".into(),
//!     channel: "temp".into(),
//!     value: Some(Value::Int(25)),
//!     constraints: None,
//!     timestamp: Utc::now(),
//!     effect: Effect::Epistemic,
//!     policy_decisions: vec![],
//! });
//!
//! assert!(tracker.has_event(CapabilityEventType::Observed));
//! ```

use ash_core::{Constraint, Effect, Name, Value};
use chrono::{DateTime, Utc};

/// Type of capability event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityEventType {
    /// Observation event from sampling
    Observed,
    /// Message received from stream
    Received,
    /// Value set on capability
    Set,
    /// Message sent to capability
    Sent,
}

/// Direction of capability operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Input operations: observe, receive
    Input,
    /// Output operations: set, send
    Output,
}

/// A provenance event for capability operations
#[derive(Debug, Clone)]
pub struct CapabilityProvenanceEvent {
    /// The type of capability event
    pub event_type: CapabilityEventType,
    /// Direction of the operation
    pub direction: Direction,
    /// Capability name
    pub capability: Name,
    /// Channel name
    pub channel: Name,
    /// Value being operated on (None for sensitive data)
    pub value: Option<Value>,
    /// Query constraints (for input operations)
    pub constraints: Option<Vec<Constraint>>,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Effect level of the operation
    pub effect: Effect,
    /// Policy decisions applied to this operation
    pub policy_decisions: Vec<PolicyDecision>,
}

/// Policy decision record
#[derive(Debug, Clone)]
pub struct PolicyDecision {
    /// Name of the policy that made the decision
    pub policy_name: String,
    /// The decision outcome (e.g., "Permit", "Deny")
    pub decision: String,
}

/// Provenance tracker for capability operations
#[derive(Debug, Default)]
pub struct CapabilityProvenanceTracker {
    events: Vec<CapabilityProvenanceEvent>,
}

impl CapabilityProvenanceTracker {
    /// Create a new empty tracker
    pub fn new() -> Self {
        Self { events: vec![] }
    }

    /// Record a capability event
    pub fn record(&mut self, event: CapabilityProvenanceEvent) {
        self.events.push(event);
    }

    /// Get all recorded events
    pub fn events(&self) -> &[CapabilityProvenanceEvent] {
        &self.events
    }

    /// Check if tracker has events of a specific type
    pub fn has_event(&self, event_type: CapabilityEventType) -> bool {
        self.events.iter().any(|e| e.event_type == event_type)
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Create an event builder for fluent API
    pub fn event_builder() -> CapabilityEventBuilder {
        CapabilityEventBuilder::new()
    }
}

/// Builder for capability provenance events
#[derive(Debug)]
pub struct CapabilityEventBuilder {
    event_type: Option<CapabilityEventType>,
    direction: Option<Direction>,
    capability: Option<Name>,
    channel: Option<Name>,
    value: Option<Value>,
    constraints: Option<Vec<Constraint>>,
    effect: Option<Effect>,
}

impl CapabilityEventBuilder {
    /// Create a new event builder with all fields unset
    pub fn new() -> Self {
        Self {
            event_type: None,
            direction: None,
            capability: None,
            channel: None,
            value: None,
            constraints: None,
            effect: None,
        }
    }

    /// Set the event type
    pub fn event_type(mut self, event_type: CapabilityEventType) -> Self {
        self.event_type = Some(event_type);
        self
    }

    /// Set the direction
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Set the capability name
    pub fn capability(mut self, capability: Name) -> Self {
        self.capability = Some(capability);
        self
    }

    /// Set the channel name
    pub fn channel(mut self, channel: Name) -> Self {
        self.channel = Some(channel);
        self
    }

    /// Set the value
    pub fn value(mut self, value: Value) -> Self {
        self.value = Some(value);
        self
    }

    /// Set the constraints
    pub fn constraints(mut self, constraints: Vec<Constraint>) -> Self {
        self.constraints = Some(constraints);
        self
    }

    /// Set the effect
    pub fn effect(mut self, effect: Effect) -> Self {
        self.effect = Some(effect);
        self
    }

    /// Build the event, panicking if required fields are missing
    ///
    /// # Panics
    ///
    /// Panics if any of the required fields are not set:
    /// - event_type
    /// - direction
    /// - capability
    /// - channel
    /// - effect
    pub fn build(self) -> CapabilityProvenanceEvent {
        CapabilityProvenanceEvent {
            event_type: self.event_type.expect("event_type required"),
            direction: self.direction.expect("direction required"),
            capability: self.capability.expect("capability required"),
            channel: self.channel.expect("channel required"),
            value: self.value,
            constraints: self.constraints,
            timestamp: Utc::now(),
            effect: self.effect.expect("effect required"),
            policy_decisions: vec![],
        }
    }
}

impl Default for CapabilityEventBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_observe_event() {
        let mut tracker = CapabilityProvenanceTracker::new();

        tracker.record(CapabilityProvenanceEvent {
            event_type: CapabilityEventType::Observed,
            direction: Direction::Input,
            capability: "sensor".into(),
            channel: "temp".into(),
            value: Some(Value::Int(25)),
            constraints: None,
            timestamp: Utc::now(),
            effect: Effect::Epistemic,
            policy_decisions: vec![],
        });

        assert!(tracker.has_event(CapabilityEventType::Observed));
        assert_eq!(tracker.events().len(), 1);
    }

    #[test]
    fn test_record_set_event() {
        let mut tracker = CapabilityProvenanceTracker::new();

        tracker.record(CapabilityProvenanceEvent {
            event_type: CapabilityEventType::Set,
            direction: Direction::Output,
            capability: "hvac".into(),
            channel: "target".into(),
            value: Some(Value::Int(72)),
            constraints: None,
            timestamp: Utc::now(),
            effect: Effect::Operational,
            policy_decisions: vec![],
        });

        assert!(tracker.has_event(CapabilityEventType::Set));
    }

    #[test]
    fn test_builder_pattern() {
        let event = CapabilityEventBuilder::new()
            .event_type(CapabilityEventType::Observed)
            .direction(Direction::Input)
            .capability("sensor".into())
            .channel("temp".into())
            .value(Value::Int(42))
            .effect(Effect::Epistemic)
            .build();

        assert_eq!(event.event_type, CapabilityEventType::Observed);
        assert_eq!(event.direction, Direction::Input);
    }

    #[test]
    fn test_clear_events() {
        let mut tracker = CapabilityProvenanceTracker::new();

        tracker.record(CapabilityProvenanceEvent {
            event_type: CapabilityEventType::Observed,
            direction: Direction::Input,
            capability: "sensor".into(),
            channel: "temp".into(),
            value: None,
            constraints: None,
            timestamp: Utc::now(),
            effect: Effect::Epistemic,
            policy_decisions: vec![],
        });

        assert_eq!(tracker.events().len(), 1);
        tracker.clear();
        assert_eq!(tracker.events().len(), 0);
    }
}
