//! Trace event recording for workflow execution
//!
//! This module provides types for recording and storing trace events
//! during workflow execution, enabling comprehensive audit trails.

use ash_core::{Decision, WorkflowId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// A unique identifier for individual trace events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub Uuid);

impl EventId {
    /// Create a new unique event ID.
    pub fn new() -> Self {
        EventId(Uuid::new_v4())
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

/// Types of trace events that can be recorded during workflow execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TraceEvent {
    /// Workflow execution started.
    WorkflowStarted {
        event_id: EventId,
        workflow_id: WorkflowId,
        name: String,
        timestamp: DateTime<Utc>,
    },
    /// Workflow execution completed.
    WorkflowCompleted {
        event_id: EventId,
        workflow_id: WorkflowId,
        success: bool,
        timestamp: DateTime<Utc>,
    },
    /// Observation of external data.
    Observation {
        event_id: EventId,
        workflow_id: WorkflowId,
        capability: String,
        value: String,
        timestamp: DateTime<Utc>,
    },
    /// Orientation/analysis of data.
    Orientation {
        event_id: EventId,
        workflow_id: WorkflowId,
        expression: String,
        result: String,
        timestamp: DateTime<Utc>,
    },
    /// Proposal for action.
    Proposal {
        event_id: EventId,
        workflow_id: WorkflowId,
        action: String,
        parameters: Vec<(String, String)>,
        timestamp: DateTime<Utc>,
    },
    /// Policy decision.
    Decision {
        event_id: EventId,
        workflow_id: WorkflowId,
        policy: String,
        decision: Decision,
        reason: Option<String>,
        timestamp: DateTime<Utc>,
    },
    /// Action execution.
    Action {
        event_id: EventId,
        workflow_id: WorkflowId,
        action: String,
        guard: String,
        timestamp: DateTime<Utc>,
    },
    /// Obligation check.
    ObligationCheck {
        event_id: EventId,
        workflow_id: WorkflowId,
        role: String,
        satisfied: bool,
        timestamp: DateTime<Utc>,
    },
    /// Error during execution.
    Error {
        event_id: EventId,
        workflow_id: WorkflowId,
        error: String,
        context: Option<String>,
        timestamp: DateTime<Utc>,
    },
}

impl TraceEvent {
    /// Get the event ID for this event.
    pub fn event_id(&self) -> EventId {
        match self {
            Self::WorkflowStarted { event_id, .. } => *event_id,
            Self::WorkflowCompleted { event_id, .. } => *event_id,
            Self::Observation { event_id, .. } => *event_id,
            Self::Orientation { event_id, .. } => *event_id,
            Self::Proposal { event_id, .. } => *event_id,
            Self::Decision { event_id, .. } => *event_id,
            Self::Action { event_id, .. } => *event_id,
            Self::ObligationCheck { event_id, .. } => *event_id,
            Self::Error { event_id, .. } => *event_id,
        }
    }

    /// Get the workflow ID for this event.
    pub fn workflow_id(&self) -> WorkflowId {
        match self {
            Self::WorkflowStarted { workflow_id, .. } => *workflow_id,
            Self::WorkflowCompleted { workflow_id, .. } => *workflow_id,
            Self::Observation { workflow_id, .. } => *workflow_id,
            Self::Orientation { workflow_id, .. } => *workflow_id,
            Self::Proposal { workflow_id, .. } => *workflow_id,
            Self::Decision { workflow_id, .. } => *workflow_id,
            Self::Action { workflow_id, .. } => *workflow_id,
            Self::ObligationCheck { workflow_id, .. } => *workflow_id,
            Self::Error { workflow_id, .. } => *workflow_id,
        }
    }

    /// Get the timestamp for this event.
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::WorkflowStarted { timestamp, .. } => *timestamp,
            Self::WorkflowCompleted { timestamp, .. } => *timestamp,
            Self::Observation { timestamp, .. } => *timestamp,
            Self::Orientation { timestamp, .. } => *timestamp,
            Self::Proposal { timestamp, .. } => *timestamp,
            Self::Decision { timestamp, .. } => *timestamp,
            Self::Action { timestamp, .. } => *timestamp,
            Self::ObligationCheck { timestamp, .. } => *timestamp,
            Self::Error { timestamp, .. } => *timestamp,
        }
    }

    /// Create a workflow started event.
    pub fn workflow_started(workflow_id: WorkflowId, name: impl Into<String>) -> Self {
        Self::WorkflowStarted {
            event_id: EventId::new(),
            workflow_id,
            name: name.into(),
            timestamp: Utc::now(),
        }
    }

    /// Create a workflow completed event.
    pub fn workflow_completed(workflow_id: WorkflowId, success: bool) -> Self {
        Self::WorkflowCompleted {
            event_id: EventId::new(),
            workflow_id,
            success,
            timestamp: Utc::now(),
        }
    }

    /// Create an observation event.
    pub fn observation(
        workflow_id: WorkflowId,
        capability: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self::Observation {
            event_id: EventId::new(),
            workflow_id,
            capability: capability.into(),
            value: value.into(),
            timestamp: Utc::now(),
        }
    }

    /// Create an orientation event.
    pub fn orientation(
        workflow_id: WorkflowId,
        expression: impl Into<String>,
        result: impl Into<String>,
    ) -> Self {
        Self::Orientation {
            event_id: EventId::new(),
            workflow_id,
            expression: expression.into(),
            result: result.into(),
            timestamp: Utc::now(),
        }
    }

    /// Create a proposal event.
    pub fn proposal(
        workflow_id: WorkflowId,
        action: impl Into<String>,
        parameters: Vec<(String, String)>,
    ) -> Self {
        Self::Proposal {
            event_id: EventId::new(),
            workflow_id,
            action: action.into(),
            parameters,
            timestamp: Utc::now(),
        }
    }

    /// Create a decision event.
    pub fn decision(
        workflow_id: WorkflowId,
        policy: impl Into<String>,
        decision: Decision,
        reason: Option<impl Into<String>>,
    ) -> Self {
        Self::Decision {
            event_id: EventId::new(),
            workflow_id,
            policy: policy.into(),
            decision,
            reason: reason.map(Into::into),
            timestamp: Utc::now(),
        }
    }

    /// Create an action event.
    pub fn action(
        workflow_id: WorkflowId,
        action: impl Into<String>,
        guard: impl Into<String>,
    ) -> Self {
        Self::Action {
            event_id: EventId::new(),
            workflow_id,
            action: action.into(),
            guard: guard.into(),
            timestamp: Utc::now(),
        }
    }

    /// Create an obligation check event.
    pub fn obligation_check(
        workflow_id: WorkflowId,
        role: impl Into<String>,
        satisfied: bool,
    ) -> Self {
        Self::ObligationCheck {
            event_id: EventId::new(),
            workflow_id,
            role: role.into(),
            satisfied,
            timestamp: Utc::now(),
        }
    }

    /// Create an error event.
    pub fn error(
        workflow_id: WorkflowId,
        error: impl Into<String>,
        context: Option<impl Into<String>>,
    ) -> Self {
        Self::Error {
            event_id: EventId::new(),
            workflow_id,
            error: error.into(),
            context: context.map(Into::into),
            timestamp: Utc::now(),
        }
    }
}

/// Trait for storing trace events.
///
/// Implementations can provide different storage backends
/// (in-memory, file-based, database, etc.).
pub trait TraceStore: Send + Sync {
    /// Store a trace event.
    ///
    /// # Errors
    ///
    /// Returns an error if the event could not be stored.
    fn store(&self, event: TraceEvent) -> Result<(), TraceStoreError>;

    /// Get all stored events.
    fn events(&self) -> Vec<TraceEvent>;

    /// Get events for a specific workflow.
    fn events_for_workflow(&self, workflow_id: WorkflowId) -> Vec<TraceEvent>;
}

/// Errors that can occur when storing trace events.
#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum TraceStoreError {
    /// The store is at capacity.
    #[error("trace store is at capacity")]
    AtCapacity,
    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(String),
    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(String),
}

/// In-memory storage for trace events.
#[derive(Debug, Default)]
pub struct InMemoryTraceStore {
    events: std::sync::RwLock<Vec<TraceEvent>>,
}

impl Clone for InMemoryTraceStore {
    fn clone(&self) -> Self {
        let events = self.events.read().unwrap();
        Self {
            events: std::sync::RwLock::new(events.clone()),
        }
    }
}

impl InMemoryTraceStore {
    /// Create a new empty in-memory trace store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new store with a specific capacity hint.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: std::sync::RwLock::new(Vec::with_capacity(capacity)),
        }
    }

    /// Clear all events from the store.
    pub fn clear(&self) {
        let mut events = self.events.write().unwrap();
        events.clear();
    }

    /// Get the number of events in the store.
    pub fn len(&self) -> usize {
        let events = self.events.read().unwrap();
        events.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl TraceStore for InMemoryTraceStore {
    fn store(&self, event: TraceEvent) -> Result<(), TraceStoreError> {
        let mut events = self.events.write().unwrap();
        events.push(event);
        Ok(())
    }

    fn events(&self) -> Vec<TraceEvent> {
        let events = self.events.read().unwrap();
        events.clone()
    }

    fn events_for_workflow(&self, workflow_id: WorkflowId) -> Vec<TraceEvent> {
        let events = self.events.read().unwrap();
        events
            .iter()
            .filter(|e| e.workflow_id() == workflow_id)
            .cloned()
            .collect()
    }
}

impl TraceStore for Arc<InMemoryTraceStore> {
    fn store(&self, event: TraceEvent) -> Result<(), TraceStoreError> {
        (**self).store(event)
    }

    fn events(&self) -> Vec<TraceEvent> {
        (**self).events()
    }

    fn events_for_workflow(&self, workflow_id: WorkflowId) -> Vec<TraceEvent> {
        (**self).events_for_workflow(workflow_id)
    }
}

/// Records trace events for a specific workflow.
#[derive(Debug, Clone)]
pub struct TraceRecorder<S: TraceStore> {
    workflow_id: WorkflowId,
    store: S,
}

impl<S: TraceStore> TraceRecorder<S> {
    /// Create a new trace recorder for the given workflow.
    pub fn new(workflow_id: WorkflowId, store: S) -> Self {
        Self {
            workflow_id,
            store,
        }
    }

    /// Get the workflow ID.
    pub fn workflow_id(&self) -> WorkflowId {
        self.workflow_id
    }

    /// Get a reference to the underlying store.
    pub fn store(&self) -> &S {
        &self.store
    }

    /// Record a trace event.
    ///
    /// # Errors
    ///
    /// Returns an error if the event could not be stored.
    pub fn record(&mut self, event: TraceEvent) -> Result<(), TraceStoreError> {
        self.store.store(event)
    }

    /// Record a workflow started event.
    pub fn record_workflow_started(&mut self, name: impl Into<String>) -> Result<(), TraceStoreError> {
        self.record(TraceEvent::workflow_started(self.workflow_id, name))
    }

    /// Record a workflow completed event.
    pub fn record_workflow_completed(&mut self, success: bool) -> Result<(), TraceStoreError> {
        self.record(TraceEvent::workflow_completed(self.workflow_id, success))
    }

    /// Record an observation event.
    pub fn record_observation(
        &mut self,
        capability: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<(), TraceStoreError> {
        self.record(TraceEvent::observation(self.workflow_id, capability, value))
    }

    /// Record an orientation event.
    pub fn record_orientation(
        &mut self,
        expression: impl Into<String>,
        result: impl Into<String>,
    ) -> Result<(), TraceStoreError> {
        self.record(TraceEvent::orientation(self.workflow_id, expression, result))
    }

    /// Record a proposal event.
    pub fn record_proposal(
        &mut self,
        action: impl Into<String>,
        parameters: Vec<(String, String)>,
    ) -> Result<(), TraceStoreError> {
        self.record(TraceEvent::proposal(self.workflow_id, action, parameters))
    }

    /// Record a decision event.
    pub fn record_decision(
        &mut self,
        policy: impl Into<String>,
        decision: Decision,
        reason: Option<impl Into<String>>,
    ) -> Result<(), TraceStoreError> {
        self.record(TraceEvent::decision(self.workflow_id, policy, decision, reason))
    }

    /// Record an action event.
    pub fn record_action(
        &mut self,
        action: impl Into<String>,
        guard: impl Into<String>,
    ) -> Result<(), TraceStoreError> {
        self.record(TraceEvent::action(self.workflow_id, action, guard))
    }

    /// Record an obligation check event.
    pub fn record_obligation_check(
        &mut self,
        role: impl Into<String>,
        satisfied: bool,
    ) -> Result<(), TraceStoreError> {
        self.record(TraceEvent::obligation_check(self.workflow_id, role, satisfied))
    }

    /// Record an error event.
    pub fn record_error(
        &mut self,
        error: impl Into<String>,
        context: Option<impl Into<String>>,
    ) -> Result<(), TraceStoreError> {
        self.record(TraceEvent::error(self.workflow_id, error, context))
    }

    /// Get all events for this workflow.
    pub fn events(&self) -> Vec<TraceEvent> {
        self.store.events_for_workflow(self.workflow_id)
    }
}

impl TraceRecorder<Arc<InMemoryTraceStore>> {
    /// Create a new trace recorder with a shared store.
    pub fn new_shared(workflow_id: WorkflowId, store: Arc<InMemoryTraceStore>) -> Self {
        Self {
            workflow_id,
            store,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_id_unique() {
        let id1 = EventId::new();
        let id2 = EventId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_trace_event_workflow_started() {
        let workflow_id = WorkflowId::new();
        let event = TraceEvent::workflow_started(workflow_id, "test");

        assert_eq!(event.workflow_id(), workflow_id);
        match &event {
            TraceEvent::WorkflowStarted { name, .. } => assert_eq!(name, "test"),
            _ => panic!("wrong event type"),
        }
    }

    #[test]
    fn test_trace_event_workflow_completed() {
        let workflow_id = WorkflowId::new();
        let event = TraceEvent::workflow_completed(workflow_id, true);

        match &event {
            TraceEvent::WorkflowCompleted { success, .. } => assert!(success),
            _ => panic!("wrong event type"),
        }
    }

    #[test]
    fn test_trace_event_observation() {
        let workflow_id = WorkflowId::new();
        let event = TraceEvent::observation(workflow_id, "sensor", "42.0");

        match &event {
            TraceEvent::Observation {
                capability, value, ..
            } => {
                assert_eq!(capability, "sensor");
                assert_eq!(value, "42.0");
            }
            _ => panic!("wrong event type"),
        }
    }

    #[test]
    fn test_trace_event_decision() {
        let workflow_id = WorkflowId::new();
        let event = TraceEvent::decision(workflow_id, "budget", Decision::Permit, Some::<&str>("under_limit"));

        match &event {
            TraceEvent::Decision {
                policy,
                decision,
                reason,
                ..
            } => {
                assert_eq!(policy, "budget");
                assert_eq!(*decision, Decision::Permit);
                assert_eq!(reason.as_deref(), Some("under_limit"));
            }
            _ => panic!("wrong event type"),
        }
    }

    #[test]
    fn test_in_memory_trace_store() {
        let store = InMemoryTraceStore::new();
        let workflow_id = WorkflowId::new();
        let event = TraceEvent::workflow_started(workflow_id, "test");

        store.store(event.clone()).unwrap();
        assert_eq!(store.len(), 1);

        let events = store.events();
        assert_eq!(events.len(), 1);

        let workflow_events = store.events_for_workflow(workflow_id);
        assert_eq!(workflow_events.len(), 1);

        let other_workflow = WorkflowId::new();
        let other_events = store.events_for_workflow(other_workflow);
        assert!(other_events.is_empty());
    }

    #[test]
    fn test_trace_recorder() {
        let workflow_id = WorkflowId::new();
        let store = InMemoryTraceStore::new();
        let mut recorder = TraceRecorder::new(workflow_id, store);

        recorder.record_workflow_started("my_workflow").unwrap();
        recorder.record_observation("temp", "25.0").unwrap();
        recorder.record_action("cool", "approved").unwrap();
        recorder.record_workflow_completed(true).unwrap();

        let events = recorder.events();
        assert_eq!(events.len(), 4);
    }

    #[test]
    fn test_trace_recorder_with_shared_store() {
        let store = Arc::new(InMemoryTraceStore::new());
        let workflow1 = WorkflowId::new();
        let workflow2 = WorkflowId::new();

        let mut recorder1 = TraceRecorder::new_shared(workflow1, Arc::clone(&store));
        let mut recorder2 = TraceRecorder::new_shared(workflow2, Arc::clone(&store));

        recorder1.record_workflow_started("wf1").unwrap();
        recorder2.record_workflow_started("wf2").unwrap();

        assert_eq!(store.len(), 2);

        let wf1_events = store.events_for_workflow(workflow1);
        assert_eq!(wf1_events.len(), 1);
    }

    #[test]
    fn test_event_id_accessors() {
        let workflow_id = WorkflowId::new();
        let event = TraceEvent::workflow_started(workflow_id, "test");

        let _id = event.event_id();
        let _ts = event.timestamp();
        let _wf = event.workflow_id();
    }

    #[test]
    fn test_serde_roundtrip() {
        let workflow_id = WorkflowId::new();
        let original = TraceEvent::observation(workflow_id, "sensor", "value");

        let json = serde_json::to_string(&original).unwrap();
        let restored: TraceEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(original.event_id(), restored.event_id());
        assert_eq!(original.workflow_id(), restored.workflow_id());
    }

    #[test]
    fn test_all_event_variants() {
        let workflow_id = WorkflowId::new();
        let now = Utc::now();

        let events = vec![
            TraceEvent::WorkflowStarted {
                event_id: EventId::new(),
                workflow_id,
                name: "test".into(),
                timestamp: now,
            },
            TraceEvent::WorkflowCompleted {
                event_id: EventId::new(),
                workflow_id,
                success: true,
                timestamp: now,
            },
            TraceEvent::Observation {
                event_id: EventId::new(),
                workflow_id,
                capability: "cap".into(),
                value: "val".into(),
                timestamp: now,
            },
            TraceEvent::Orientation {
                event_id: EventId::new(),
                workflow_id,
                expression: "x > 0".into(),
                result: "true".into(),
                timestamp: now,
            },
            TraceEvent::Proposal {
                event_id: EventId::new(),
                workflow_id,
                action: "send".into(),
                parameters: vec![("to".into(), "user".into())],
                timestamp: now,
            },
            TraceEvent::Decision {
                event_id: EventId::new(),
                workflow_id,
                policy: "policy".into(),
                decision: Decision::Permit,
                reason: Some("ok".into()),
                timestamp: now,
            },
            TraceEvent::Action {
                event_id: EventId::new(),
                workflow_id,
                action: "act".into(),
                guard: "guard".into(),
                timestamp: now,
            },
            TraceEvent::ObligationCheck {
                event_id: EventId::new(),
                workflow_id,
                role: "admin".into(),
                satisfied: true,
                timestamp: now,
            },
            TraceEvent::Error {
                event_id: EventId::new(),
                workflow_id,
                error: "fail".into(),
                context: Some("ctx".into()),
                timestamp: now,
            },
        ];

        assert_eq!(events.len(), 9);
    }
}
