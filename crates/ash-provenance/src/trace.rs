//! Trace event recording for application execution
//!
//! This module provides types for recording and storing trace events
//! during application execution, enabling comprehensive audit trails.

use ash_core::{ApplicationId, Decision};
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

/// Types of trace events that can be recorded during application execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TraceEvent {
    /// Application execution started.
    ApplicationStarted {
        event_id: EventId,
        application_id: ApplicationId,
        name: String,
        timestamp: DateTime<Utc>,
    },
    /// Application execution completed.
    ApplicationCompleted {
        event_id: EventId,
        application_id: ApplicationId,
        success: bool,
        timestamp: DateTime<Utc>,
    },
    /// Observation of external data.
    Observation {
        event_id: EventId,
        application_id: ApplicationId,
        capability: String,
        value: String,
        timestamp: DateTime<Utc>,
    },
    /// Orientation/analysis of data.
    Orientation {
        event_id: EventId,
        application_id: ApplicationId,
        expression: String,
        result: String,
        timestamp: DateTime<Utc>,
    },
    /// Proposal for action.
    Proposal {
        event_id: EventId,
        application_id: ApplicationId,
        action: String,
        parameters: Vec<(String, String)>,
        timestamp: DateTime<Utc>,
    },
    /// Policy decision.
    Decision {
        event_id: EventId,
        application_id: ApplicationId,
        policy: String,
        decision: Decision,
        reason: Option<String>,
        timestamp: DateTime<Utc>,
    },
    /// Action execution.
    Action {
        event_id: EventId,
        application_id: ApplicationId,
        action: String,
        guard: String,
        timestamp: DateTime<Utc>,
    },
    /// Obligation check.
    ObligationCheck {
        event_id: EventId,
        application_id: ApplicationId,
        role: String,
        satisfied: bool,
        timestamp: DateTime<Utc>,
    },
    /// Error during execution.
    Error {
        event_id: EventId,
        application_id: ApplicationId,
        error: String,
        context: Option<String>,
        timestamp: DateTime<Utc>,
    },
}

impl TraceEvent {
    /// Get the event ID for this event.
    pub fn event_id(&self) -> EventId {
        match self {
            Self::ApplicationStarted { event_id, .. } => *event_id,
            Self::ApplicationCompleted { event_id, .. } => *event_id,
            Self::Observation { event_id, .. } => *event_id,
            Self::Orientation { event_id, .. } => *event_id,
            Self::Proposal { event_id, .. } => *event_id,
            Self::Decision { event_id, .. } => *event_id,
            Self::Action { event_id, .. } => *event_id,
            Self::ObligationCheck { event_id, .. } => *event_id,
            Self::Error { event_id, .. } => *event_id,
        }
    }

    /// Get the application ID for this event.
    pub fn application_id(&self) -> ApplicationId {
        match self {
            Self::ApplicationStarted { application_id, .. } => *application_id,
            Self::ApplicationCompleted { application_id, .. } => *application_id,
            Self::Observation { application_id, .. } => *application_id,
            Self::Orientation { application_id, .. } => *application_id,
            Self::Proposal { application_id, .. } => *application_id,
            Self::Decision { application_id, .. } => *application_id,
            Self::Action { application_id, .. } => *application_id,
            Self::ObligationCheck { application_id, .. } => *application_id,
            Self::Error { application_id, .. } => *application_id,
        }
    }

    /// Get the timestamp for this event.
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::ApplicationStarted { timestamp, .. } => *timestamp,
            Self::ApplicationCompleted { timestamp, .. } => *timestamp,
            Self::Observation { timestamp, .. } => *timestamp,
            Self::Orientation { timestamp, .. } => *timestamp,
            Self::Proposal { timestamp, .. } => *timestamp,
            Self::Decision { timestamp, .. } => *timestamp,
            Self::Action { timestamp, .. } => *timestamp,
            Self::ObligationCheck { timestamp, .. } => *timestamp,
            Self::Error { timestamp, .. } => *timestamp,
        }
    }

    /// Create a application started event.
    pub fn application_started(application_id: ApplicationId, name: impl Into<String>) -> Self {
        Self::ApplicationStarted {
            event_id: EventId::new(),
            application_id,
            name: name.into(),
            timestamp: Utc::now(),
        }
    }

    /// Create a application completed event.
    pub fn application_completed(application_id: ApplicationId, success: bool) -> Self {
        Self::ApplicationCompleted {
            event_id: EventId::new(),
            application_id,
            success,
            timestamp: Utc::now(),
        }
    }

    /// Create an observation event.
    pub fn observation(
        application_id: ApplicationId,
        capability: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self::Observation {
            event_id: EventId::new(),
            application_id,
            capability: capability.into(),
            value: value.into(),
            timestamp: Utc::now(),
        }
    }

    /// Create an orientation event.
    pub fn orientation(
        application_id: ApplicationId,
        expression: impl Into<String>,
        result: impl Into<String>,
    ) -> Self {
        Self::Orientation {
            event_id: EventId::new(),
            application_id,
            expression: expression.into(),
            result: result.into(),
            timestamp: Utc::now(),
        }
    }

    /// Create a proposal event.
    pub fn proposal(
        application_id: ApplicationId,
        action: impl Into<String>,
        parameters: Vec<(String, String)>,
    ) -> Self {
        Self::Proposal {
            event_id: EventId::new(),
            application_id,
            action: action.into(),
            parameters,
            timestamp: Utc::now(),
        }
    }

    /// Create a decision event.
    pub fn decision(
        application_id: ApplicationId,
        policy: impl Into<String>,
        decision: Decision,
        reason: Option<impl Into<String>>,
    ) -> Self {
        Self::Decision {
            event_id: EventId::new(),
            application_id,
            policy: policy.into(),
            decision,
            reason: reason.map(Into::into),
            timestamp: Utc::now(),
        }
    }

    /// Create an action event.
    pub fn action(
        application_id: ApplicationId,
        action: impl Into<String>,
        guard: impl Into<String>,
    ) -> Self {
        Self::Action {
            event_id: EventId::new(),
            application_id,
            action: action.into(),
            guard: guard.into(),
            timestamp: Utc::now(),
        }
    }

    /// Create an obligation check event.
    pub fn obligation_check(
        application_id: ApplicationId,
        role: impl Into<String>,
        satisfied: bool,
    ) -> Self {
        Self::ObligationCheck {
            event_id: EventId::new(),
            application_id,
            role: role.into(),
            satisfied,
            timestamp: Utc::now(),
        }
    }

    /// Create an error event.
    pub fn error(
        application_id: ApplicationId,
        error: impl Into<String>,
        context: Option<impl Into<String>>,
    ) -> Self {
        Self::Error {
            event_id: EventId::new(),
            application_id,
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

    /// Get events for a specific application.
    fn events_for_application(&self, application_id: ApplicationId) -> Vec<TraceEvent>;
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

    fn events_for_application(&self, application_id: ApplicationId) -> Vec<TraceEvent> {
        let events = self.events.read().unwrap();
        events
            .iter()
            .filter(|e| e.application_id() == application_id)
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

    fn events_for_application(&self, application_id: ApplicationId) -> Vec<TraceEvent> {
        (**self).events_for_application(application_id)
    }
}

/// Records trace events for a specific application.
#[derive(Debug, Clone)]
pub struct TraceRecorder<S: TraceStore> {
    application_id: ApplicationId,
    store: S,
}

impl<S: TraceStore> TraceRecorder<S> {
    /// Create a new trace recorder for the given application.
    pub fn new(application_id: ApplicationId, store: S) -> Self {
        Self {
            application_id,
            store,
        }
    }

    /// Get the application ID.
    pub fn application_id(&self) -> ApplicationId {
        self.application_id
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

    /// Record a application started event.
    pub fn record_application_started(
        &mut self,
        name: impl Into<String>,
    ) -> Result<(), TraceStoreError> {
        self.record(TraceEvent::application_started(self.application_id, name))
    }

    /// Record a application completed event.
    pub fn record_application_completed(&mut self, success: bool) -> Result<(), TraceStoreError> {
        self.record(TraceEvent::application_completed(
            self.application_id,
            success,
        ))
    }

    /// Record an observation event.
    pub fn record_observation(
        &mut self,
        capability: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<(), TraceStoreError> {
        self.record(TraceEvent::observation(
            self.application_id,
            capability,
            value,
        ))
    }

    /// Record an orientation event.
    pub fn record_orientation(
        &mut self,
        expression: impl Into<String>,
        result: impl Into<String>,
    ) -> Result<(), TraceStoreError> {
        self.record(TraceEvent::orientation(
            self.application_id,
            expression,
            result,
        ))
    }

    /// Record a proposal event.
    pub fn record_proposal(
        &mut self,
        action: impl Into<String>,
        parameters: Vec<(String, String)>,
    ) -> Result<(), TraceStoreError> {
        self.record(TraceEvent::proposal(
            self.application_id,
            action,
            parameters,
        ))
    }

    /// Record a decision event.
    pub fn record_decision(
        &mut self,
        policy: impl Into<String>,
        decision: Decision,
        reason: Option<impl Into<String>>,
    ) -> Result<(), TraceStoreError> {
        self.record(TraceEvent::decision(
            self.application_id,
            policy,
            decision,
            reason,
        ))
    }

    /// Record an action event.
    pub fn record_action(
        &mut self,
        action: impl Into<String>,
        guard: impl Into<String>,
    ) -> Result<(), TraceStoreError> {
        self.record(TraceEvent::action(self.application_id, action, guard))
    }

    /// Record an obligation check event.
    pub fn record_obligation_check(
        &mut self,
        role: impl Into<String>,
        satisfied: bool,
    ) -> Result<(), TraceStoreError> {
        self.record(TraceEvent::obligation_check(
            self.application_id,
            role,
            satisfied,
        ))
    }

    /// Record an error event.
    pub fn record_error(
        &mut self,
        error: impl Into<String>,
        context: Option<impl Into<String>>,
    ) -> Result<(), TraceStoreError> {
        self.record(TraceEvent::error(self.application_id, error, context))
    }

    /// Get all events for this application.
    pub fn events(&self) -> Vec<TraceEvent> {
        self.store.events_for_application(self.application_id)
    }
}

impl TraceRecorder<Arc<InMemoryTraceStore>> {
    /// Create a new trace recorder with a shared store.
    pub fn new_shared(application_id: ApplicationId, store: Arc<InMemoryTraceStore>) -> Self {
        Self {
            application_id,
            store,
        }
    }
}

/// Wrapper-safe application trace session that guarantees canonical entry/exit framing.
///
/// A session records exactly one application-started event on entry. Terminal success records
/// `ApplicationCompleted { success: true }` as the final event. Terminal failure records
/// `Error` followed by `ApplicationCompleted { success: false }`.
#[derive(Debug)]
pub struct ApplicationTraceSession<S: TraceStore> {
    recorder: TraceRecorder<S>,
}

impl<S: TraceStore> ApplicationTraceSession<S> {
    /// Start a new application trace session by recording the application entry event.
    ///
    /// # Errors
    ///
    /// Returns an error if the start event cannot be stored.
    pub fn start(
        mut recorder: TraceRecorder<S>,
        name: impl Into<String>,
    ) -> Result<Self, TraceStoreError> {
        recorder.record_application_started(name)?;
        Ok(Self { recorder })
    }

    /// Get a mutable recorder reference for accepted runtime progression events.
    pub fn recorder_mut(&mut self) -> &mut TraceRecorder<S> {
        &mut self.recorder
    }

    /// Finish the session successfully, recording terminal completion last.
    ///
    /// # Errors
    ///
    /// Returns an error if the completion event cannot be stored.
    pub fn finish_success(mut self) -> Result<TraceRecorder<S>, TraceStoreError> {
        self.recorder.record_application_completed(true)?;
        Ok(self.recorder)
    }

    /// Finish the session with an error, recording the error before failed completion.
    ///
    /// # Errors
    ///
    /// Returns an error if either the error or completion event cannot be stored.
    pub fn finish_error(
        mut self,
        error: impl Into<String>,
        context: Option<impl Into<String>>,
    ) -> Result<TraceRecorder<S>, TraceStoreError> {
        self.recorder.record_error(error, context)?;
        self.recorder.record_application_completed(false)?;
        Ok(self.recorder)
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
    fn test_trace_event_application_started() {
        let application_id = ApplicationId::new();
        let event = TraceEvent::application_started(application_id, "test");

        assert_eq!(event.application_id(), application_id);
        match &event {
            TraceEvent::ApplicationStarted { name, .. } => assert_eq!(name, "test"),
            _ => panic!("wrong event type"),
        }
    }

    #[test]
    fn test_trace_event_application_completed() {
        let application_id = ApplicationId::new();
        let event = TraceEvent::application_completed(application_id, true);

        match &event {
            TraceEvent::ApplicationCompleted { success, .. } => assert!(success),
            _ => panic!("wrong event type"),
        }
    }

    #[test]
    fn test_trace_event_observation() {
        let application_id = ApplicationId::new();
        let event = TraceEvent::observation(application_id, "sensor", "42.0");

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
        let application_id = ApplicationId::new();
        let event = TraceEvent::decision(
            application_id,
            "budget",
            Decision::Permit,
            Some::<&str>("under_limit"),
        );

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
        let application_id = ApplicationId::new();
        let event = TraceEvent::application_started(application_id, "test");

        store.store(event.clone()).unwrap();
        assert_eq!(store.len(), 1);

        let events = store.events();
        assert_eq!(events.len(), 1);

        let application_events = store.events_for_application(application_id);
        assert_eq!(application_events.len(), 1);

        let other_application = ApplicationId::new();
        let other_events = store.events_for_application(other_application);
        assert!(other_events.is_empty());
    }

    #[test]
    fn test_trace_recorder() {
        let application_id = ApplicationId::new();
        let store = InMemoryTraceStore::new();
        let mut recorder = TraceRecorder::new(application_id, store);

        recorder
            .record_application_started("my_application")
            .unwrap();
        recorder.record_observation("temp", "25.0").unwrap();
        recorder.record_action("cool", "approved").unwrap();
        recorder.record_application_completed(true).unwrap();

        let events = recorder.events();
        assert_eq!(events.len(), 4);
    }

    #[test]
    fn test_trace_recorder_with_shared_store() {
        let store = Arc::new(InMemoryTraceStore::new());
        let application1 = ApplicationId::new();
        let application2 = ApplicationId::new();

        let mut recorder1 = TraceRecorder::new_shared(application1, Arc::clone(&store));
        let mut recorder2 = TraceRecorder::new_shared(application2, Arc::clone(&store));

        recorder1.record_application_started("wf1").unwrap();
        recorder2.record_application_started("wf2").unwrap();

        assert_eq!(store.len(), 2);

        let wf1_events = store.events_for_application(application1);
        assert_eq!(wf1_events.len(), 1);
    }

    #[test]
    fn test_event_id_accessors() {
        let application_id = ApplicationId::new();
        let event = TraceEvent::application_started(application_id, "test");

        let _id = event.event_id();
        let _ts = event.timestamp();
        let _wf = event.application_id();
    }

    #[test]
    fn test_serde_roundtrip() {
        let application_id = ApplicationId::new();
        let original = TraceEvent::observation(application_id, "sensor", "value");

        let json = serde_json::to_string(&original).unwrap();
        let restored: TraceEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(original.event_id(), restored.event_id());
        assert_eq!(original.application_id(), restored.application_id());
    }

    #[test]
    fn test_all_event_variants() {
        let application_id = ApplicationId::new();
        let now = Utc::now();

        let events = vec![
            TraceEvent::ApplicationStarted {
                event_id: EventId::new(),
                application_id,
                name: "test".into(),
                timestamp: now,
            },
            TraceEvent::ApplicationCompleted {
                event_id: EventId::new(),
                application_id,
                success: true,
                timestamp: now,
            },
            TraceEvent::Observation {
                event_id: EventId::new(),
                application_id,
                capability: "cap".into(),
                value: "val".into(),
                timestamp: now,
            },
            TraceEvent::Orientation {
                event_id: EventId::new(),
                application_id,
                expression: "x > 0".into(),
                result: "true".into(),
                timestamp: now,
            },
            TraceEvent::Proposal {
                event_id: EventId::new(),
                application_id,
                action: "send".into(),
                parameters: vec![("to".into(), "user".into())],
                timestamp: now,
            },
            TraceEvent::Decision {
                event_id: EventId::new(),
                application_id,
                policy: "policy".into(),
                decision: Decision::Permit,
                reason: Some("ok".into()),
                timestamp: now,
            },
            TraceEvent::Action {
                event_id: EventId::new(),
                application_id,
                action: "act".into(),
                guard: "guard".into(),
                timestamp: now,
            },
            TraceEvent::ObligationCheck {
                event_id: EventId::new(),
                application_id,
                role: "admin".into(),
                satisfied: true,
                timestamp: now,
            },
            TraceEvent::Error {
                event_id: EventId::new(),
                application_id,
                error: "fail".into(),
                context: Some("ctx".into()),
                timestamp: now,
            },
        ];

        assert_eq!(events.len(), 9);
    }
}
