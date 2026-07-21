//! Ash Provenance - Audit trail and lineage tracking for Ash applications
//!
//! This crate provides comprehensive provenance tracking including:
//! - Trace event recording for application execution
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
pub use trace::{ApplicationTraceSession, InMemoryTraceStore, TraceEvent, TraceRecorder};

use ash_core::ApplicationId;
use std::sync::Arc;

// Re-export TraceStore so users can use convenience functions
pub use trace::TraceStore;

/// Create a new trace recorder with an in-memory store.
///
/// # Examples
///
/// ```
/// use ash_provenance::create_trace_recorder;
/// use ash_core::ApplicationId;
///
/// let application_id = ApplicationId::new();
/// let recorder = create_trace_recorder(application_id);
/// ```
pub fn create_trace_recorder(application_id: ApplicationId) -> TraceRecorder<InMemoryTraceStore> {
    TraceRecorder::new(application_id, InMemoryTraceStore::new())
}

/// Create a new trace recorder with a shared store.
///
/// Useful when multiple recorders need to write to the same backing store.
///
/// # Examples
///
/// ```
/// use ash_provenance::{create_shared_trace_recorder, InMemoryTraceStore};
/// use ash_core::ApplicationId;
/// use std::sync::Arc;
///
/// let store = Arc::new(InMemoryTraceStore::new());
/// let recorder = create_shared_trace_recorder(ApplicationId::new(), store);
/// ```
pub fn create_shared_trace_recorder(
    application_id: ApplicationId,
    store: Arc<InMemoryTraceStore>,
) -> TraceRecorder<Arc<InMemoryTraceStore>> {
    TraceRecorder::new_shared(application_id, store)
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

/// Convenience function to record a application start event.
///
/// # Examples
///
/// ```
/// use ash_provenance::{record_application_start, create_trace_recorder};
/// use ash_core::ApplicationId;
///
/// let mut recorder = create_trace_recorder(ApplicationId::new());
/// record_application_start(&mut recorder, "my_application");
/// ```
pub fn record_application_start<S: trace::TraceStore>(recorder: &mut TraceRecorder<S>, name: &str) {
    let _ = recorder.record_application_started(name);
}

/// Convenience function to record a application completion event.
///
/// # Examples
///
/// ```
/// use ash_provenance::{record_application_complete, create_trace_recorder};
/// use ash_core::ApplicationId;
///
/// let mut recorder = create_trace_recorder(ApplicationId::new());
/// record_application_complete(&mut recorder, true);
/// ```
pub fn record_application_complete<S: trace::TraceStore>(
    recorder: &mut TraceRecorder<S>,
    success: bool,
) {
    let _ = recorder.record_application_completed(success);
}

/// Convenience function to record an observation event.
///
/// # Examples
///
/// ```
/// use ash_provenance::{record_observation, create_trace_recorder};
/// use ash_core::ApplicationId;
///
/// let mut recorder = create_trace_recorder(ApplicationId::new());
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
/// use ash_core::ApplicationId;
///
/// let mut recorder = create_trace_recorder(ApplicationId::new());
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
/// use ash_core::ApplicationId;
///
/// let mut recorder = create_trace_recorder(ApplicationId::new());
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
        let application_id = ApplicationId::new();
        let recorder = create_trace_recorder(application_id);
        assert_eq!(recorder.application_id(), application_id);
    }

    #[test]
    fn test_create_lineage_tracker() {
        let tracker = create_lineage_tracker();
        // Just verify it can be created
        let _ = tracker;
    }

    #[test]
    fn test_convenience_functions() {
        let application_id = ApplicationId::new();
        let mut recorder = create_trace_recorder(application_id);

        record_application_start(&mut recorder, "test_application");
        record_observation(&mut recorder, "sensor", "42");
        record_action(&mut recorder, "notify", "approved");
        record_error(&mut recorder, "timeout");
        record_application_complete(&mut recorder, true);

        let events = recorder.store().events();
        assert_eq!(events.len(), 5);
    }
}
