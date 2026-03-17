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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
