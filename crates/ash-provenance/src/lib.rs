//! Ash Provenance - Audit trail and lineage tracking for Ash workflows
//!
//! This crate provides comprehensive provenance tracking including:
//! - Trace event recording for workflow execution
//! - Data lineage tracking for values
//! - Export to multiple audit formats (JSON, CSV, PROV)
//! - Integrity verification using Merkle trees

pub mod audit;
pub mod export;
pub mod integrity;
pub mod lineage;
pub mod trace;

pub use audit::{AuditBackend, AuditError, AuditEvent, AuditLog, CheckResult, FileAuditBackend};
pub use export::{AuditExporter, CsvExporter, ExportFormat, JsonExporter, NdJsonExporter};
pub use integrity::{Hash, MerkleTree, verify_integrity};
pub use lineage::{DataSource, Lineage, LineageTracker, Transformation};
pub use trace::{InMemoryTraceStore, TraceEvent, TraceRecorder, WorkflowTraceSession};

use ash_core::WorkflowId;
use std::sync::Arc;

// Re-export TraceStore so users can use convenience functions
pub use trace::TraceStore;

/// Create a new trace recorder with an in-memory store.
///
/// # Examples
///
/// ```
/// use ash_provenance::create_trace_recorder;
/// use ash_core::WorkflowId;
///
/// let workflow_id = WorkflowId::new();
/// let recorder = create_trace_recorder(workflow_id);
/// ```
pub fn create_trace_recorder(workflow_id: WorkflowId) -> TraceRecorder<InMemoryTraceStore> {
    TraceRecorder::new(workflow_id, InMemoryTraceStore::new())
}

/// Create a new trace recorder with a shared store.
///
/// Useful when multiple recorders need to write to the same backing store.
///
/// # Examples
///
/// ```
/// use ash_provenance::{create_shared_trace_recorder, InMemoryTraceStore};
/// use ash_core::WorkflowId;
/// use std::sync::Arc;
///
/// let store = Arc::new(InMemoryTraceStore::new());
/// let recorder = create_shared_trace_recorder(WorkflowId::new(), store);
/// ```
pub fn create_shared_trace_recorder(
    workflow_id: WorkflowId,
    store: Arc<InMemoryTraceStore>,
) -> TraceRecorder<Arc<InMemoryTraceStore>> {
    TraceRecorder::new_shared(workflow_id, store)
}

/// Create a new lineage tracker.
///
/// # Examples
///
/// ```
/// use ash_provenance::create_lineage_tracker;
///
/// let tracker = create_lineage_tracker();
/// ```
pub fn create_lineage_tracker() -> LineageTracker {
    LineageTracker::new()
}

/// Convenience function to record a workflow start event.
///
/// # Examples
///
/// ```
/// use ash_provenance::{record_workflow_start, create_trace_recorder};
/// use ash_core::WorkflowId;
///
/// let mut recorder = create_trace_recorder(WorkflowId::new());
/// record_workflow_start(&mut recorder, "my_workflow");
/// ```
pub fn record_workflow_start<S: trace::TraceStore>(recorder: &mut TraceRecorder<S>, name: &str) {
    let _ = recorder.record_workflow_started(name);
}

/// Convenience function to record a workflow completion event.
///
/// # Examples
///
/// ```
/// use ash_provenance::{record_workflow_complete, create_trace_recorder};
/// use ash_core::WorkflowId;
///
/// let mut recorder = create_trace_recorder(WorkflowId::new());
/// record_workflow_complete(&mut recorder, true);
/// ```
pub fn record_workflow_complete<S: trace::TraceStore>(
    recorder: &mut TraceRecorder<S>,
    success: bool,
) {
    let _ = recorder.record_workflow_completed(success);
}

/// Convenience function to record an observation event.
///
/// # Examples
///
/// ```
/// use ash_provenance::{record_observation, create_trace_recorder};
/// use ash_core::WorkflowId;
///
/// let mut recorder = create_trace_recorder(WorkflowId::new());
/// record_observation(&mut recorder, "temperature", "25.5");
/// ```
pub fn record_observation<S: trace::TraceStore>(
    recorder: &mut TraceRecorder<S>,
    capability: &str,
    value: &str,
) {
    let _ = recorder.record_observation(capability, value);
}

/// Convenience function to record an action event.
///
/// # Examples
///
/// ```
/// use ash_provenance::{record_action, create_trace_recorder};
/// use ash_core::WorkflowId;
///
/// let mut recorder = create_trace_recorder(WorkflowId::new());
/// record_action(&mut recorder, "send_email", "approved");
/// ```
pub fn record_action<S: trace::TraceStore>(
    recorder: &mut TraceRecorder<S>,
    action: &str,
    guard: &str,
) {
    let _ = recorder.record_action(action, guard);
}

/// Convenience function to record an error event.
///
/// # Examples
///
/// ```
/// use ash_provenance::{record_error, create_trace_recorder};
/// use ash_core::WorkflowId;
///
/// let mut recorder = create_trace_recorder(WorkflowId::new());
/// record_error(&mut recorder, "connection failed");
/// ```
pub fn record_error<S: trace::TraceStore>(recorder: &mut TraceRecorder<S>, error: &str) {
    let _ = recorder.record_error(error, None::<&str>);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_trace_recorder() {
        let workflow_id = WorkflowId::new();
        let recorder = create_trace_recorder(workflow_id);
        assert_eq!(recorder.workflow_id(), workflow_id);
    }

    #[test]
    fn test_create_lineage_tracker() {
        let tracker = create_lineage_tracker();
        // Just verify it can be created
        let _ = tracker;
    }

    #[test]
    fn test_convenience_functions() {
        let workflow_id = WorkflowId::new();
        let mut recorder = create_trace_recorder(workflow_id);

        record_workflow_start(&mut recorder, "test_workflow");
        record_observation(&mut recorder, "sensor", "42");
        record_action(&mut recorder, "notify", "approved");
        record_error(&mut recorder, "timeout");
        record_workflow_complete(&mut recorder, true);

        let events = recorder.store().events();
        assert_eq!(events.len(), 5);
    }
}
