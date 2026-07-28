//! Trace command for capturing target Ash entry execution provenance.
//!
//! TASK-055: Implement `trace` command with provenance capture.
//! TASK-254: Implement trace flags (--lineage, --verify)

use anyhow::{Context, Result};
use ash_engine::{CanonicalTerminalEnvelopeV1, EngineError};
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
    /// Path to Ash source file.
    #[arg(value_name = "PATH")]
    pub path: String,

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

/// Run a target Ash source file with full provenance tracing.
pub async fn trace(args: &TraceArgs) -> Result<()> {
    let path = Path::new(&args.path);
    let engine = ash_engine::Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .with_custom_provider(
            "time",
            std::sync::Arc::new(ash_engine::providers::TimeProvider::new()),
        )
        .build()
        .context("Failed to build engine")?;
    let mut application = engine.parse_file(path).map_err(classify_engine_error)?;
    engine
        .check(&mut application)
        .map_err(classify_engine_error)?;

    // Execute with tracing
    let trace_result = execute_with_full_trace(&engine, &mut application, path, args).await?;

    // Output trace data
    output_trace(&trace_result, args).await?;

    Ok(())
}

/// Trace result containing execution data
#[derive(Debug, Serialize)]
pub struct TraceResult {
    pub trace_id: String,
    pub application: String,
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

/// Execute a parsed Ash entry with full provenance tracing.
async fn execute_with_full_trace(
    engine: &ash_engine::Engine,
    application: &mut ash_engine::Entry,
    path: &Path,
    args: &TraceArgs,
) -> Result<TraceResult> {
    use ash_core::ApplicationId;
    use ash_provenance::{ApplicationTraceSession, create_trace_recorder};

    // Admission and trace policy both remain Engine-owned.  Reject a route
    // before allocating recorder state or creating a request, so a denied
    // route cannot emit a partial trace document or execute as a side effect.
    let program = match engine.admit_program(application) {
        Ok(program) => program,
        Err(error) if error.production_terminal_classification().is_some() => {
            let terminal = error.canonical_terminal_envelope().expect(
                "a production-terminal classification always has a canonical terminal envelope",
            );
            return trace_terminal_value(terminal).and_then(|_| {
                anyhow::bail!(
                    "a production-terminal classification cannot project a successful trace value"
                )
            });
        }
        Err(error) => return Err(classify_engine_error(error)),
    };
    if !program.permits_trace() {
        let message = program
            .trace_rejection_message()
            .expect("non-traceable admitted programs must carry a trace rejection reason");
        anyhow::bail!("{message}");
    }

    let application_id = ApplicationId::new();
    let recorder = create_trace_recorder(application_id);
    let session = ApplicationTraceSession::start(recorder, "main")?;

    // Initialize lineage tracker if requested
    let lineage_tracker = if args.lineage {
        Some(LineageTracker::new())
    } else {
        None
    };

    // The trace client owns only its recorder lifecycle. It receives the
    // terminal observation from the same opaque admitted-program seam as the
    // other production clients; it never evaluates the parsed entry directly.
    let (request, _cancellation) = engine
        .new_admitted_program_request(&program, None)
        .map_err(classify_engine_error)?;
    let terminal = engine
        .execute_admitted_program(&request)
        .await
        .map_err(classify_engine_error)?;
    let result = trace_terminal_value(terminal);
    let recorder = match &result {
        Ok(_) => session.finish_success()?,
        Err(error) => session.finish_error(format!("{error:?}"), Some("admitted_program"))?,
    };

    let final_value = result?;

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
        trace_id: application_id.0.to_string(),
        application: path.display().to_string(),
        started_at,
        events,
        final_value: final_value.to_string(),
        lineage,
        integrity,
    })
}

/// Mechanically project the Engine-owned terminal envelope into the legacy
/// trace command's value-or-error interface. This is terminal formatting only;
/// all admission and execution authority remains inside `ash_engine`.
fn trace_terminal_value(terminal: CanonicalTerminalEnvelopeV1) -> Result<ash_core::Value> {
    match terminal {
        CanonicalTerminalEnvelopeV1::Returned(value) => Ok(value),
        CanonicalTerminalEnvelopeV1::Trapped(reason) => anyhow::bail!("runtime error: {reason}"),
        CanonicalTerminalEnvelopeV1::AdmissionRejected => anyhow::bail!(
            "runtime error: application execution failed: checked Core/CPS admission rejected: no validated production typed lowering is available"
        ),
        CanonicalTerminalEnvelopeV1::InvalidCheckedArtifact => {
            anyhow::bail!("runtime error: checked Core/CPS artifact is invalid")
        }
        CanonicalTerminalEnvelopeV1::TimedOut => {
            anyhow::bail!("runtime error: admitted program timed out")
        }
        CanonicalTerminalEnvelopeV1::Cancelled => {
            anyhow::bail!("runtime error: admitted program cancelled")
        }
    }
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
    let mut csv = String::from("timestamp,event_type,application_id\n");
    for event in &result.events {
        // Get event type name from the variant
        let type_name = match event {
            ash_provenance::TraceEvent::ApplicationStarted { .. } => "application_started",
            ash_provenance::TraceEvent::ApplicationCompleted { .. } => "application_completed",
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
            event.application_id()
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
        EngineError::ProductionTerminal {
            classification,
            message,
        } => anyhow::anyhow!("production terminal {classification:?}: {message}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_args_parsing() {
        let args = TraceArgs {
            path: "test.ash".to_string(),
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
            application: "main".to_string(),
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
            application: "main".to_string(),
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
            application: "main".to_string(),
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
