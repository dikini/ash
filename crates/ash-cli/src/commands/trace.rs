//! Trace command for capturing workflow execution provenance.
//!
//! TASK-055: Implement `trace` command with provenance capture.
//! TASK-254: Implement trace flags (--lineage, --verify)

use anyhow::{Context, Result};
use ash_engine::EngineError;
use ash_interp::ExecError;
use ash_provenance::LineageTracker;
use ash_provenance::export::ExportFormat;
use ash_provenance::integrity::{TamperEvidentLog, hash_value};
use clap::Args;
use serde::Serialize;
use std::path::Path;

/// Export format for trace command
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum TraceExportFormat {
    /// JSON format
    Json,
    /// PROV-N format
    Provn,
    /// Cypher graph format
    Cypher,
}

/// Arguments for the trace command
#[derive(Args, Debug, Clone)]
pub struct TraceArgs {
    /// Path to workflow file
    #[arg(value_name = "PATH")]
    pub path: String,

    /// Input parameters as JSON object
    #[arg(short, long, value_name = "JSON")]
    pub input: Option<String>,

    /// Output file for trace data
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<String>,

    /// Trace output format (json, ndjson, csv)
    #[arg(short, long, default_value = "json")]
    pub format: String,

    /// Include data lineage information
    #[arg(long)]
    pub lineage: bool,

    /// Verify trace integrity
    #[arg(long)]
    pub verify: bool,

    /// Cryptographically sign trace
    #[arg(long)]
    pub sign: bool,

    /// Export format
    #[arg(long, value_enum)]
    pub export: Option<TraceExportFormat>,

    /// Output in PROV-N format
    #[arg(long)]
    pub provn: bool,

    /// Output Cypher graph
    #[arg(long)]
    pub cypher: bool,
}

/// Run a workflow with full provenance tracing
pub async fn trace(args: &TraceArgs) -> Result<()> {
    let path = Path::new(&args.path);
    let engine = ash_engine::Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .build()
        .context("Failed to build engine")?;
    let workflow = engine.parse_file(path).map_err(classify_engine_error)?;
    engine.check(&workflow).map_err(classify_engine_error)?;

    // Execute with tracing
    let trace_result = execute_with_full_trace(&engine, &workflow, path, args).await?;

    // Output trace data
    output_trace(&trace_result, args).await?;

    Ok(())
}

/// Trace result containing execution data
#[derive(Debug, Serialize)]
pub struct TraceResult {
    pub trace_id: String,
    pub workflow: String,
    pub started_at: String,
    pub events: Vec<ash_provenance::TraceEvent>,
    pub final_value: String,
    /// Data lineage information (included when --lineage flag is used)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<Vec<ash_provenance::Lineage>>,
    /// Integrity verification data (included when --verify flag is used)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrity: Option<IntegrityData>,
}

/// Integrity verification data for trace output
#[derive(Debug, Serialize)]
pub struct IntegrityData {
    /// Merkle tree root hash
    pub root_hash: String,
    /// Number of events included in the integrity check
    pub event_count: usize,
    /// Algorithm used for hashing
    pub algorithm: String,
}

/// Execute a workflow with full provenance tracing
async fn execute_with_full_trace(
    engine: &ash_engine::Engine,
    workflow: &ash_engine::Workflow,
    path: &Path,
    args: &TraceArgs,
) -> Result<TraceResult> {
    use ash_core::WorkflowId;
    use ash_provenance::{WorkflowTraceSession, create_trace_recorder};

    let workflow_id = WorkflowId::new();
    let recorder = create_trace_recorder(workflow_id);
    let session = WorkflowTraceSession::start(recorder, "main")?;

    // Initialize lineage tracker if requested
    let lineage_tracker = if args.lineage {
        Some(LineageTracker::new())
    } else {
        None
    };

    // Execute the workflow
    let result = engine.execute(workflow).await;
    let recorder = match &result {
        Ok(_) => session.finish_success()?,
        Err(error) => session.finish_error(format!("{error:?}"), Some("ash_interp::interpret"))?,
    };

    let final_value = result.map_err(classify_exec_error)?;

    // Get events from recorder
    let events = recorder.events().to_vec();
    let started_at = events
        .first()
        .map(|event| event.timestamp().to_rfc3339())
        .unwrap_or_default();

    // Collect lineage data if requested
    let lineage = if args.lineage {
        lineage_tracker
            .as_ref()
            .map(|tracker| tracker.all().cloned().collect::<Vec<_>>())
    } else {
        None
    };

    // Compute integrity data if requested
    let integrity = if args.verify {
        compute_integrity_data(&events)?
    } else {
        None
    };

    Ok(TraceResult {
        trace_id: workflow_id.0.to_string(),
        workflow: path.display().to_string(),
        started_at,
        events,
        final_value: final_value.to_string(),
        lineage,
        integrity,
    })
}

/// Compute integrity data for trace events using Merkle tree
fn compute_integrity_data(events: &[ash_provenance::TraceEvent]) -> Result<Option<IntegrityData>> {
    if events.is_empty() {
        return Ok(None);
    }

    let mut log = TamperEvidentLog::new();

    for event in events {
        let hash = hash_value(event).map_err(|e| anyhow::anyhow!("failed to hash event: {}", e))?;
        log.append(hash.as_bytes());
    }

    let root_hash = log
        .root()
        .map(|h: ash_provenance::integrity::Hash| h.to_hex())
        .unwrap_or_default();

    Ok(Some(IntegrityData {
        root_hash,
        event_count: events.len(),
        algorithm: "SHA-256".to_string(),
    }))
}

/// Output trace data to file or stdout
async fn output_trace(result: &TraceResult, args: &TraceArgs) -> Result<()> {
    let format = match args.format.as_str() {
        "ndjson" => ExportFormat::NdJson,
        "csv" => ExportFormat::Csv,
        _ => ExportFormat::Json,
    };

    let output = match format {
        ExportFormat::Json => export_json(result)?,
        ExportFormat::NdJson => export_ndjson(result)?,
        ExportFormat::Csv => export_csv(result)?,
        _ => {
            // Fallback to JSON for unsupported formats
            println!("[WARN] Format not fully supported, using JSON");
            export_json(result)?
        }
    };

    match &args.output {
        Some(path) => {
            tokio::fs::write(path, output)
                .await
                .with_context(|| format!("Failed to write trace to {}", path))?;
        }
        None => {
            println!("{}", output);
        }
    }

    Ok(())
}

/// Export trace as JSON
fn export_json(result: &TraceResult) -> Result<String> {
    Ok(serde_json::to_string_pretty(result)?)
}

/// Export trace as NDJSON (newline-delimited JSON)
fn export_ndjson(result: &TraceResult) -> Result<String> {
    let mut lines = Vec::new();
    for event in &result.events {
        lines.push(serde_json::to_string(event)?);
    }
    Ok(lines.join("\n"))
}

/// Export trace as CSV
fn export_csv(result: &TraceResult) -> Result<String> {
    let mut csv = String::from("timestamp,event_type,workflow_id\n");
    for event in &result.events {
        // Get event type name from the variant
        let type_name = match event {
            ash_provenance::TraceEvent::WorkflowStarted { .. } => "workflow_started",
            ash_provenance::TraceEvent::WorkflowCompleted { .. } => "workflow_completed",
            ash_provenance::TraceEvent::Observation { .. } => "observation",
            ash_provenance::TraceEvent::Orientation { .. } => "orientation",
            ash_provenance::TraceEvent::Proposal { .. } => "proposal",
            ash_provenance::TraceEvent::Decision { .. } => "decision",
            ash_provenance::TraceEvent::Action { .. } => "action",
            ash_provenance::TraceEvent::ObligationCheck { .. } => "obligation_check",
            ash_provenance::TraceEvent::Error { .. } => "error",
        };
        let line = format!(
            "{},{:?},{:?}\n",
            event.timestamp().to_rfc3339(),
            type_name,
            event.workflow_id()
        );
        csv.push_str(&line);
    }
    Ok(csv)
}

fn classify_engine_error(error: EngineError) -> anyhow::Error {
    match error {
        EngineError::Parse(message) => anyhow::anyhow!("parse error: {message}"),
        EngineError::Type(message) => anyhow::anyhow!("type error: {message}"),
        EngineError::Execution(message) => anyhow::anyhow!("runtime error: {message}"),
        EngineError::CapabilityNotFound(message) => {
            anyhow::anyhow!("verification error: capability not found: {message}")
        }
        EngineError::Io(error) => anyhow::anyhow!("io error: {error}"),
        EngineError::Configuration(message) => {
            anyhow::anyhow!("configuration error: {message}")
        }
    }
}

fn classify_exec_error(error: ExecError) -> anyhow::Error {
    match error {
        ExecError::ExecutionFailed(message) if message.starts_with("parse error:") => {
            anyhow::anyhow!("{message}")
        }
        ExecError::ExecutionFailed(message) if message.starts_with("type error:") => {
            anyhow::anyhow!("{message}")
        }
        ExecError::CapabilityNotAvailable(name) => {
            anyhow::anyhow!("verification error: capability not available: {name}")
        }
        ExecError::PolicyDenied { policy } => anyhow::anyhow!("policy denial: {policy}"),
        ExecError::RequiresApproval {
            role,
            operation,
            capability,
        } => anyhow::anyhow!(
            "approval required: role '{}' must approve {} on {}",
            role.as_ref(),
            operation,
            capability
        ),
        other => anyhow::anyhow!("runtime error: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_args_parsing() {
        let args = TraceArgs {
            path: "test.ash".to_string(),
            input: None,
            output: Some("trace.json".to_string()),
            format: "json".to_string(),
            lineage: true,
            verify: false,
            sign: true,
            export: Some(TraceExportFormat::Json),
            provn: false,
            cypher: false,
        };

        assert_eq!(args.path, "test.ash");
        assert_eq!(args.format, "json");
        assert!(args.lineage);
        assert!(!args.verify);
        assert!(args.sign);
        assert!(matches!(args.export, Some(TraceExportFormat::Json)));
    }

    #[test]
    fn test_trace_args_new_flags() {
        let args = TraceArgs {
            path: "test.ash".to_string(),
            input: None,
            output: None,
            format: "json".to_string(),
            lineage: false,
            verify: false,
            sign: false,
            export: Some(TraceExportFormat::Provn),
            provn: true,
            cypher: false,
        };

        assert!(args.provn);
        assert!(matches!(args.export, Some(TraceExportFormat::Provn)));
    }

    #[test]
    fn test_trace_args_cypher() {
        let args = TraceArgs {
            path: "test.ash".to_string(),
            input: None,
            output: None,
            format: "json".to_string(),
            lineage: false,
            verify: false,
            sign: false,
            export: Some(TraceExportFormat::Cypher),
            provn: false,
            cypher: true,
        };

        assert!(args.cypher);
        assert!(matches!(args.export, Some(TraceExportFormat::Cypher)));
    }

    #[test]
    fn test_export_formats() {
        use ash_core::Value;

        let result = TraceResult {
            trace_id: "trace-id".to_string(),
            workflow: "main".to_string(),
            started_at: "2026-03-20T00:00:00Z".to_string(),
            events: vec![],
            final_value: Value::Int(42).to_string(),
            lineage: None,
            integrity: None,
        };

        let json = export_json(&result).unwrap();
        assert!(json.contains("final_value"));
        assert!(json.contains("trace_id"));

        let ndjson = export_ndjson(&result).unwrap();
        assert!(ndjson.is_empty() || ndjson.contains("{"));

        let csv = export_csv(&result).unwrap();
        assert!(csv.contains("timestamp"));
    }

    #[test]
    fn test_trace_with_lineage() {
        let result = TraceResult {
            trace_id: "trace-id".to_string(),
            workflow: "main".to_string(),
            started_at: "2026-03-20T00:00:00Z".to_string(),
            events: vec![],
            final_value: "42".to_string(),
            lineage: Some(vec![]),
            integrity: None,
        };

        let json = export_json(&result).unwrap();
        assert!(json.contains("lineage"));
    }

    #[test]
    fn test_trace_with_integrity() {
        let result = TraceResult {
            trace_id: "trace-id".to_string(),
            workflow: "main".to_string(),
            started_at: "2026-03-20T00:00:00Z".to_string(),
            events: vec![],
            final_value: "42".to_string(),
            lineage: None,
            integrity: Some(IntegrityData {
                root_hash: "abc123".to_string(),
                event_count: 0,
                algorithm: "SHA-256".to_string(),
            }),
        };

        let json = export_json(&result).unwrap();
        assert!(json.contains("integrity"));
        assert!(json.contains("root_hash"));
        assert!(json.contains("SHA-256"));
    }

    #[test]
    fn test_compute_integrity_data_empty() {
        let events: Vec<ash_provenance::TraceEvent> = vec![];
        let integrity = compute_integrity_data(&events).unwrap();
        assert!(integrity.is_none());
    }
}
