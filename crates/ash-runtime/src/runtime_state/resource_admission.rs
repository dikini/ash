//! Runtime resource admission and split/join policy data models.

use ash_core::runtime::{ProcessId, ResourceInstance};

/// Runtime resource split/join policy violation with the offending resource metadata attached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceSplitJoinViolation {
    /// Process that attempted the split/join operation.
    pub process_id: ProcessId,
    /// Operation being checked, such as `par`, `scatter`, `join`, or `gather`.
    pub operation: &'static str,
    /// Resource whose policy rejected the operation.
    pub resource: ResourceInstance,
    /// Human-readable rejection reason.
    pub reason: String,
}

impl ResourceSplitJoinViolation {
    pub(super) fn new(
        process_id: ProcessId,
        operation: &'static str,
        resource: ResourceInstance,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            process_id,
            operation,
            resource,
            reason: reason.into(),
        }
    }

    /// Render policy evidence notes suitable for an operational failure carrier.
    #[must_use]
    pub fn evidence_notes(&self) -> Vec<String> {
        vec![
            format!("resource policy violation during proc::{}", self.operation),
            format!("process: {:?}", self.process_id),
            format!("resource id: {:?}", self.resource.id),
            format!("resource type: {:?}", self.resource.type_id),
            format!("resource owner: {:?}", self.resource.owner),
            format!("resource lifecycle: {:?}", self.resource.lifecycle),
            format!(
                "resource split/join policy: {:?}",
                self.resource.split_join_policy
            ),
            self.reason.clone(),
        ]
    }

    /// Render policy provenance suitable for an operational failure carrier.
    #[must_use]
    pub fn evidence_provenance(&self) -> Vec<String> {
        vec![format!(
            "resource provenance: {:?}",
            self.resource.provenance
        )]
    }
}

impl std::fmt::Display for ResourceSplitJoinViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "resource {:?} of type {:?} with policy {:?} rejected proc::{} for process {:?}: {}",
            self.resource.id,
            self.resource.type_id,
            self.resource.split_join_policy,
            self.operation,
            self.process_id,
            self.reason
        )
    }
}

impl std::error::Error for ResourceSplitJoinViolation {}
