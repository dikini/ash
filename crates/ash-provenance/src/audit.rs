//! Audit trail for obligation checks

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Audit event for obligation checks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditEvent {
    /// Name of the obligation being checked
    pub obligation: String,
    /// Workflow instance ID
    pub workflow_id: String,
    /// When the check occurred
    pub timestamp: SystemTime,
    /// Result of the check
    pub result: CheckResult,
    /// Snapshot of relevant workflow state
    pub context: serde_json::Value,
}

/// Result of an obligation check
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status")]
pub enum CheckResult {
    #[serde(rename = "satisfied")]
    Satisfied,
    #[serde(rename = "violated")]
    Violated { reason: String },
}

/// Audit error types
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum AuditError {
    #[error("serialization failed: {0}")]
    Serialization(String),
    #[error("io error: {0}")]
    Io(String),
}

/// Pluggable audit backend
pub trait AuditBackend: Send + Sync {
    /// Record an audit event
    fn record(&mut self, event: AuditEvent) -> Result<(), AuditError>;

    /// Flush any buffered events
    fn flush(&mut self) -> Result<(), AuditError>;
}
/// File-based audit backend (JSON Lines format)
pub struct FileAuditBackend {
    /// Path to the audit log file
    path: std::path::PathBuf,
    file: std::fs::File,
    buffer: Vec<AuditEvent>,
    flush_threshold: usize,
}

impl FileAuditBackend {
    /// Create a new file-based audit backend
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self, std::io::Error> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        Ok(Self {
            path: path.as_ref().to_path_buf(),
            file,
            buffer: Vec::new(),
            flush_threshold: 100, // Flush every 100 events
        })
    }

    /// Get the path to the audit log file
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Set the buffer flush threshold
    #[must_use]
    pub fn with_flush_threshold(mut self, threshold: usize) -> Self {
        self.flush_threshold = threshold;
        self
    }

    /// Flush buffered events to disk
    fn flush_buffer(&mut self) -> Result<(), AuditError> {
        use std::io::Write;

        for event in &self.buffer {
            let line = serde_json::to_string(event)
                .map_err(|e| AuditError::Serialization(e.to_string()))?;
            writeln!(self.file, "{}", line).map_err(|e| AuditError::Io(e.to_string()))?;
        }

        self.file
            .flush()
            .map_err(|e| AuditError::Io(e.to_string()))?;

        self.buffer.clear();
        Ok(())
    }
}

impl AuditBackend for FileAuditBackend {
    fn record(&mut self, event: AuditEvent) -> Result<(), AuditError> {
        self.buffer.push(event);

        if self.buffer.len() >= self.flush_threshold {
            self.flush_buffer()?;
        }

        Ok(())
    }

    fn flush(&mut self) -> Result<(), AuditError> {
        self.flush_buffer()
    }
}

impl Drop for FileAuditBackend {
    fn drop(&mut self) {
        let _ = self.flush_buffer();
    }
}

/// Audit log handle
pub struct AuditLog {
    backend: Box<dyn AuditBackend>,
}

impl AuditLog {
    /// Create a new audit log with the given backend
    pub fn new(backend: Box<dyn AuditBackend>) -> Self {
        Self { backend }
    }

    /// Record an audit event
    pub fn record(&mut self, event: AuditEvent) -> Result<(), AuditError> {
        self.backend.record(event)
    }

    /// Flush the audit log
    pub fn flush(&mut self) -> Result<(), AuditError> {
        self.backend.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::tempdir;

    #[test]
    fn audit_event_serialization() {
        let event = AuditEvent {
            obligation: "audit_trail".into(),
            workflow_id: "wf_abc123".into(),
            timestamp: SystemTime::UNIX_EPOCH,
            result: CheckResult::Satisfied,
            context: serde_json::json!({
                "user": "alice",
                "amount": 100,
            }),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("audit_trail"));
        assert!(json.contains("satisfied"));

        let deserialized: AuditEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.obligation, "audit_trail");
        assert_eq!(deserialized.result, CheckResult::Satisfied);
    }

    #[test]
    fn check_result_violated_serialization() {
        let result = CheckResult::Violated {
            reason: "timeout".into(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("violated"));
        assert!(json.contains("timeout"));

        let deserialized: CheckResult = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized,
            CheckResult::Violated {
                reason: "timeout".into()
            }
        );
    }

    #[test]
    fn file_backend_writes_json_lines() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");

        {
            let mut backend = FileAuditBackend::new(&path)
                .unwrap()
                .with_flush_threshold(1); // Flush immediately

            let event = AuditEvent {
                obligation: "respond_by_deadline".into(),
                workflow_id: "wf_def456".into(),
                timestamp: SystemTime::UNIX_EPOCH,
                result: CheckResult::Violated {
                    reason: "timeout".into(),
                },
                context: serde_json::json!({
                    "deadline": "2024-01-15T10:30:00Z",
                }),
            };

            backend.record(event).unwrap();
        } // Drop flushes

        // Read and verify
        let mut file = std::fs::File::open(&path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        assert!(contents.contains("respond_by_deadline"));
        assert!(contents.contains("violated"));
        assert!(contents.contains("timeout"));
        assert!(contents.ends_with("\n")); // JSON Lines format
    }

    #[test]
    fn file_backend_buffers_until_threshold() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");

        let mut backend = FileAuditBackend::new(&path)
            .unwrap()
            .with_flush_threshold(3);

        // Write 2 events (below threshold)
        for i in 0..2 {
            backend
                .record(AuditEvent {
                    obligation: format!("obligation_{}", i),
                    workflow_id: format!("wf_{}", i),
                    timestamp: SystemTime::UNIX_EPOCH,
                    result: CheckResult::Satisfied,
                    context: serde_json::Value::Null,
                })
                .unwrap();
        }

        // File should be empty (buffered)
        let metadata = std::fs::metadata(&path).unwrap();
        assert_eq!(metadata.len(), 0);

        // Write 1 more (hits threshold)
        backend
            .record(AuditEvent {
                obligation: "obligation_2".into(),
                workflow_id: "wf_2".into(),
                timestamp: SystemTime::UNIX_EPOCH,
                result: CheckResult::Satisfied,
                context: serde_json::Value::Null,
            })
            .unwrap();

        // Now file has content
        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents.lines().count(), 3);
    }

    #[test]
    fn file_backend_flush_on_drop() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");

        {
            let mut backend = FileAuditBackend::new(&path)
                .unwrap()
                .with_flush_threshold(10); // High threshold

            // Write 2 events (below threshold, should not flush)
            for i in 0..2 {
                backend
                    .record(AuditEvent {
                        obligation: format!("obligation_{}", i),
                        workflow_id: format!("wf_{}", i),
                        timestamp: SystemTime::UNIX_EPOCH,
                        result: CheckResult::Satisfied,
                        context: serde_json::Value::Null,
                    })
                    .unwrap();
            }

            // File should be empty before drop
            let metadata = std::fs::metadata(&path).unwrap();
            assert_eq!(metadata.len(), 0);
        } // Drop should flush

        // After drop, file should have content
        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents.lines().count(), 2);
    }

    #[test]
    fn audit_log_wrapper() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");

        {
            let backend = FileAuditBackend::new(&path)
                .unwrap()
                .with_flush_threshold(1);
            let mut log = AuditLog::new(Box::new(backend));

            let event = AuditEvent {
                obligation: "test_obligation".into(),
                workflow_id: "wf_test".into(),
                timestamp: SystemTime::UNIX_EPOCH,
                result: CheckResult::Satisfied,
                context: serde_json::json!({"key": "value"}),
            };

            log.record(event).unwrap();
        }

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("test_obligation"));
        assert!(contents.contains("wf_test"));
    }

    #[test]
    fn audit_log_flush_explicit() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");

        let backend = FileAuditBackend::new(&path)
            .unwrap()
            .with_flush_threshold(10);
        let mut log = AuditLog::new(Box::new(backend));

        log.record(AuditEvent {
            obligation: "test".into(),
            workflow_id: "wf_1".into(),
            timestamp: SystemTime::UNIX_EPOCH,
            result: CheckResult::Satisfied,
            context: serde_json::Value::Null,
        })
        .unwrap();

        // Not flushed yet
        let metadata = std::fs::metadata(&path).unwrap();
        assert_eq!(metadata.len(), 0);

        // Explicit flush
        log.flush().unwrap();

        // Now flushed
        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents.lines().count(), 1);
    }
}
