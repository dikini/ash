//! Provenance tracking for audit trails

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A unique identifier for workflows
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowId(pub Uuid);

impl WorkflowId {
    pub fn new() -> Self {
        WorkflowId(Uuid::new_v4())
    }
}

impl Default for WorkflowId {
    fn default() -> Self {
        Self::new()
    }
}

/// Provenance information for tracking execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    /// Workflow instance ID
    pub workflow_id: WorkflowId,
    /// Parent workflow (if any)
    pub parent: Option<WorkflowId>,
    /// Lineage of workflow invocations
    pub lineage: Vec<WorkflowId>,
}

impl Provenance {
    pub fn new() -> Self {
        Provenance {
            workflow_id: WorkflowId::new(),
            parent: None,
            lineage: vec![],
        }
    }

    pub fn fork(&self) -> Self {
        Provenance {
            workflow_id: WorkflowId::new(),
            parent: Some(self.workflow_id),
            lineage: {
                let mut line = self.lineage.clone();
                line.push(self.workflow_id);
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
    fn test_workflow_id_new_is_unique() {
        let id1 = WorkflowId::new();
        let id2 = WorkflowId::new();
        assert_ne!(id1, id2, "WorkflowId should be unique");
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
        let parent_id = parent.workflow_id;
        let child = parent.fork();

        assert_ne!(child.workflow_id, parent_id);
        assert_eq!(child.parent, Some(parent_id));
        assert_eq!(child.lineage.len(), 1);
        assert_eq!(child.lineage[0], parent_id);
    }

    #[test]
    fn test_provenance_fork_lineage_accumulates() {
        let grandparent = Provenance::new();
        let gp_id = grandparent.workflow_id;

        let parent = grandparent.fork();
        let p_id = parent.workflow_id;

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
    fn test_workflow_id_default() {
        let id1: WorkflowId = Default::default();
        let id2: WorkflowId = Default::default();
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
        ];
        assert_eq!(events.len(), 5);
    }

    #[test]
    fn test_provenance_serde_roundtrip() {
        let original = Provenance::new();
        let json = serde_json::to_string(&original).expect("serialize");
        let restored: Provenance = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(original.workflow_id, restored.workflow_id);
        assert_eq!(original.parent, restored.parent);
        assert_eq!(original.lineage, restored.lineage);
    }

    #[test]
    fn test_workflow_id_serde_roundtrip() {
        let original = WorkflowId::new();
        let json = serde_json::to_string(&original).expect("serialize");
        let restored: WorkflowId = serde_json::from_str(&json).expect("deserialize");

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
