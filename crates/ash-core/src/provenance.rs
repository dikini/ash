//! Provenance tracking for audit trails

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A unique identifier for applications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApplicationId(pub Uuid);

impl ApplicationId {
    pub fn new() -> Self {
        ApplicationId(Uuid::new_v4())
    }
}

impl Default for ApplicationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Provenance information for tracking execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    /// Application instance ID
    pub application_id: ApplicationId,
    /// Parent application (if any)
    pub parent: Option<ApplicationId>,
    /// Lineage of application invocations
    pub lineage: Vec<ApplicationId>,
}

impl Provenance {
    pub fn new() -> Self {
        Provenance {
            application_id: ApplicationId::new(),
            parent: None,
            lineage: vec![],
        }
    }

    pub fn fork(&self) -> Self {
        Provenance {
            application_id: ApplicationId::new(),
            parent: Some(self.application_id),
            lineage: {
                let mut line = self.lineage.clone();
                line.push(self.application_id);
                line
            },
        }
    }
}

impl Default for Provenance {
    fn default() -> Self {
        Self::new()
    }
}

/// Events recorded in the execution trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceEvent {
    /// Observation event
    Obs {
        capability: String,
        timestamp: DateTime<Utc>,
    },
    /// Orientation/analysis event
    Orient {
        expr: String,
        timestamp: DateTime<Utc>,
    },
    /// Decision event
    Decide {
        policy: String,
        decision: Decision,
        timestamp: DateTime<Utc>,
    },
    /// Action execution event
    Act {
        action: String,
        guard: String,
        timestamp: DateTime<Utc>,
    },
    /// Obligation check event
    Oblig {
        role: String,
        satisfied: bool,
        timestamp: DateTime<Utc>,
    },
    /// A thunk was constructed.
    ThunkConstructed {
        mode: String,
        row: Vec<String>,
        timestamp: DateTime<Utc>,
    },
    /// A thunk force operation started.
    ThunkForceStarted {
        mode: String,
        timestamp: DateTime<Utc>,
    },
    /// A thunk body evaluation started.
    ThunkBodyEvaluationStarted {
        mode: String,
        timestamp: DateTime<Utc>,
    },
    /// A thunk body evaluation completed with a terminal outcome.
    ThunkBodyEvaluationCompleted {
        mode: String,
        outcome: String,
        timestamp: DateTime<Utc>,
    },
    /// A thunk force operation completed with a terminal outcome.
    ThunkForceCompleted {
        mode: String,
        outcome: String,
        timestamp: DateTime<Utc>,
    },
    /// Memo table was filled from an evaluated thunk outcome.
    MemoCacheFilled {
        outcome: String,
        timestamp: DateTime<Utc>,
    },
    /// Memo table lookup hit for a thunk.
    MemoCacheHit {
        outcome: String,
        timestamp: DateTime<Utc>,
    },
    /// Memo cached failure replay encountered.
    MemoReplayFailure {
        reason: String,
        timestamp: DateTime<Utc>,
    },
    /// Re-entrant memo force was rejected.
    MemoReentrantRejected { timestamp: DateTime<Utc> },
}

/// Policy decision outcomes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    Permit,
    Deny,
    RequireApproval,
    Escalate,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_application_id_new_is_unique() {
        let id1 = ApplicationId::new();
        let id2 = ApplicationId::new();
        assert_ne!(id1, id2, "ApplicationId should be unique");
    }

    #[test]
    fn test_provenance_new() {
        let prov = Provenance::new();
        assert!(prov.parent.is_none());
        assert!(prov.lineage.is_empty());
    }

    #[test]
    fn test_provenance_fork_creates_child() {
        let parent = Provenance::new();
        let parent_id = parent.application_id;
        let child = parent.fork();

        assert_ne!(child.application_id, parent_id);
        assert_eq!(child.parent, Some(parent_id));
        assert_eq!(child.lineage.len(), 1);
        assert_eq!(child.lineage[0], parent_id);
    }

    #[test]
    fn test_provenance_fork_lineage_accumulates() {
        let grandparent = Provenance::new();
        let gp_id = grandparent.application_id;

        let parent = grandparent.fork();
        let p_id = parent.application_id;

        let child = parent.fork();

        assert_eq!(child.lineage.len(), 2);
        assert_eq!(child.lineage[0], gp_id);
        assert_eq!(child.lineage[1], p_id);
        assert_eq!(child.parent, Some(p_id));
    }

    #[test]
    fn test_provenance_default() {
        let prov: Provenance = Default::default();
        assert!(prov.parent.is_none());
        assert!(prov.lineage.is_empty());
    }

    #[test]
    fn test_application_id_default() {
        let id1: ApplicationId = Default::default();
        let id2: ApplicationId = Default::default();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_decision_variants() {
        let decisions = [
            Decision::Permit,
            Decision::Deny,
            Decision::RequireApproval,
            Decision::Escalate,
        ];
        // Just verify they can be constructed
        assert_eq!(decisions.len(), 4);
    }

    #[test]
    fn test_trace_event_construction() {
        let now = Utc::now();
        let events = [
            TraceEvent::Obs {
                capability: "sensor".to_string(),
                timestamp: now,
            },
            TraceEvent::Orient {
                expr: "x > 0".to_string(),
                timestamp: now,
            },
            TraceEvent::Decide {
                policy: "budget".to_string(),
                decision: Decision::Permit,
                timestamp: now,
            },
            TraceEvent::Act {
                action: "notify".to_string(),
                guard: "approved".to_string(),
                timestamp: now,
            },
            TraceEvent::Oblig {
                role: "admin".to_string(),
                satisfied: true,
                timestamp: now,
            },
            TraceEvent::ThunkConstructed {
                mode: "memo".to_string(),
                row: vec!["cap db.read".to_string()],
                timestamp: now,
            },
            TraceEvent::ThunkForceStarted {
                mode: "memo".to_string(),
                timestamp: now,
            },
            TraceEvent::ThunkBodyEvaluationStarted {
                mode: "memo".to_string(),
                timestamp: now,
            },
            TraceEvent::ThunkBodyEvaluationCompleted {
                mode: "memo".to_string(),
                outcome: "success".to_string(),
                timestamp: now,
            },
            TraceEvent::ThunkForceCompleted {
                mode: "memo".to_string(),
                outcome: "success".to_string(),
                timestamp: now,
            },
            TraceEvent::MemoCacheFilled {
                outcome: "success".to_string(),
                timestamp: now,
            },
            TraceEvent::MemoCacheHit {
                outcome: "success".to_string(),
                timestamp: now,
            },
            TraceEvent::MemoReplayFailure {
                reason: "trap".to_string(),
                timestamp: now,
            },
            TraceEvent::MemoReentrantRejected { timestamp: now },
        ];
        assert_eq!(events.len(), 14);
    }

    #[test]
    fn test_provenance_serde_roundtrip() {
        let original = Provenance::new();
        let json = serde_json::to_string(&original).expect("serialize");
        let restored: Provenance = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(original.application_id, restored.application_id);
        assert_eq!(original.parent, restored.parent);
        assert_eq!(original.lineage, restored.lineage);
    }

    #[test]
    fn test_application_id_serde_roundtrip() {
        let original = ApplicationId::new();
        let json = serde_json::to_string(&original).expect("serialize");
        let restored: ApplicationId = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(original, restored);
    }

    #[test]
    fn test_decision_serde_roundtrip() {
        for decision in [
            Decision::Permit,
            Decision::Deny,
            Decision::RequireApproval,
            Decision::Escalate,
        ] {
            let json = serde_json::to_string(&decision).expect("serialize");
            let restored: Decision = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(decision, restored);
        }
    }
}
