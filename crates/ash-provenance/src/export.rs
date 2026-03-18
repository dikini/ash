//! Audit trail export to various formats
//!
//! This module provides exporters for converting trace events and lineage
//! information to different formats for audit and analysis purposes.

use crate::lineage::Lineage;
use crate::trace::TraceEvent;
use serde::{Deserialize, Serialize};
use std::io::Write;

/// Supported export formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    /// JSON format (array of objects).
    Json,
    /// W3C PROV-JSON format.
    ProvJson,
    /// W3C PROV-N notation.
    ProvN,
    /// Neo4j Cypher import format.
    Cypher,
    /// Comma-separated values.
    Csv,
    /// Newline-delimited JSON.
    NdJson,
}

impl ExportFormat {
    /// Get the file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::ProvJson => "prov.json",
            Self::ProvN => "provn",
            Self::Cypher => "cypher",
            Self::Csv => "csv",
            Self::NdJson => "ndjson",
        }
    }

    /// Get the MIME type for this format.
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Json | Self::ProvJson => "application/json",
            Self::ProvN => "text/plain",
            Self::Cypher => "text/plain",
            Self::Csv => "text/csv",
            Self::NdJson => "application/x-ndjson",
        }
    }
}

/// Errors that can occur during export.
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Format error.
    #[error("format error: {0}")]
    Format(String),
}

/// Trait for exporting audit data.
pub trait AuditExporter {
    /// Export trace events to a writer.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or writing fails.
    fn export_traces(
        &self,
        events: &[TraceEvent],
        writer: &mut dyn Write,
    ) -> Result<(), ExportError>;

    /// Export lineage records to a writer.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or writing fails.
    fn export_lineage(
        &self,
        lineages: &[Lineage],
        writer: &mut dyn Write,
    ) -> Result<(), ExportError>;

    /// Get the format this exporter produces.
    fn format(&self) -> ExportFormat;
}

/// JSON exporter for audit data.
#[derive(Debug, Clone, Default)]
pub struct JsonExporter;

impl JsonExporter {
    /// Create a new JSON exporter.
    pub fn new() -> Self {
        Self
    }
}

impl AuditExporter for JsonExporter {
    fn export_traces(
        &self,
        events: &[TraceEvent],
        writer: &mut dyn Write,
    ) -> Result<(), ExportError> {
        let json = serde_json::to_string_pretty(events)?;
        writer.write_all(json.as_bytes())?;
        Ok(())
    }

    fn export_lineage(
        &self,
        lineages: &[Lineage],
        writer: &mut dyn Write,
    ) -> Result<(), ExportError> {
        let json = serde_json::to_string_pretty(lineages)?;
        writer.write_all(json.as_bytes())?;
        Ok(())
    }

    fn format(&self) -> ExportFormat {
        ExportFormat::Json
    }
}

/// Newline-delimited JSON exporter.
#[derive(Debug, Clone, Default)]
pub struct NdJsonExporter;

impl NdJsonExporter {
    /// Create a new NDJSON exporter.
    pub fn new() -> Self {
        Self
    }
}

impl AuditExporter for NdJsonExporter {
    fn export_traces(
        &self,
        events: &[TraceEvent],
        writer: &mut dyn Write,
    ) -> Result<(), ExportError> {
        for event in events {
            let json = serde_json::to_string(event)?;
            writeln!(writer, "{json}")?;
        }
        Ok(())
    }

    fn export_lineage(
        &self,
        lineages: &[Lineage],
        writer: &mut dyn Write,
    ) -> Result<(), ExportError> {
        for lineage in lineages {
            let json = serde_json::to_string(lineage)?;
            writeln!(writer, "{json}")?;
        }
        Ok(())
    }

    fn format(&self) -> ExportFormat {
        ExportFormat::NdJson
    }
}

/// CSV exporter for audit data.
#[derive(Debug, Clone, Default)]
pub struct CsvExporter;

impl CsvExporter {
    /// Create a new CSV exporter.
    pub fn new() -> Self {
        Self
    }
}

impl AuditExporter for CsvExporter {
    fn export_traces(
        &self,
        events: &[TraceEvent],
        writer: &mut dyn Write,
    ) -> Result<(), ExportError> {
        // Write header
        writeln!(writer, "event_id,workflow_id,timestamp,type,data")?;

        for event in events {
            let (event_id, workflow_id, timestamp, type_name, data) = match event {
                TraceEvent::WorkflowStarted {
                    event_id,
                    workflow_id,
                    name,
                    timestamp,
                    ..
                } => (
                    event_id.0.to_string(),
                    workflow_id.0.to_string(),
                    timestamp.to_rfc3339(),
                    "workflow_started",
                    format!("name={}", escape_csv_value(name)),
                ),
                TraceEvent::WorkflowCompleted {
                    event_id,
                    workflow_id,
                    success,
                    timestamp,
                    ..
                } => (
                    event_id.0.to_string(),
                    workflow_id.0.to_string(),
                    timestamp.to_rfc3339(),
                    "workflow_completed",
                    format!("success={success}"),
                ),
                TraceEvent::Observation {
                    event_id,
                    workflow_id,
                    capability,
                    value,
                    timestamp,
                    ..
                } => (
                    event_id.0.to_string(),
                    workflow_id.0.to_string(),
                    timestamp.to_rfc3339(),
                    "observation",
                    format!(
                        "capability={};value={}",
                        escape_csv_value(capability),
                        escape_csv_value(value)
                    ),
                ),
                TraceEvent::Orientation {
                    event_id,
                    workflow_id,
                    expression,
                    result,
                    timestamp,
                    ..
                } => (
                    event_id.0.to_string(),
                    workflow_id.0.to_string(),
                    timestamp.to_rfc3339(),
                    "orientation",
                    format!(
                        "expr={};result={}",
                        escape_csv_value(expression),
                        escape_csv_value(result)
                    ),
                ),
                TraceEvent::Proposal {
                    event_id,
                    workflow_id,
                    action,
                    parameters,
                    timestamp,
                    ..
                } => (
                    event_id.0.to_string(),
                    workflow_id.0.to_string(),
                    timestamp.to_rfc3339(),
                    "proposal",
                    format!(
                        "action={};params={}",
                        escape_csv_value(action),
                        escape_csv_value(&format!("{parameters:?}"))
                    ),
                ),
                TraceEvent::Decision {
                    event_id,
                    workflow_id,
                    policy,
                    decision,
                    timestamp,
                    ..
                } => {
                    let decision_str = match decision {
                        ash_core::Decision::Permit => "permit",
                        ash_core::Decision::Deny => "deny",
                        ash_core::Decision::RequireApproval => "require_approval",
                        ash_core::Decision::Escalate => "escalate",
                    };
                    (
                        event_id.0.to_string(),
                        workflow_id.0.to_string(),
                        timestamp.to_rfc3339(),
                        "decision",
                        format!(
                            "policy={};decision={decision_str}",
                            escape_csv_value(policy)
                        ),
                    )
                }
                TraceEvent::Action {
                    event_id,
                    workflow_id,
                    action,
                    guard,
                    timestamp,
                    ..
                } => (
                    event_id.0.to_string(),
                    workflow_id.0.to_string(),
                    timestamp.to_rfc3339(),
                    "action",
                    format!(
                        "action={};guard={}",
                        escape_csv_value(action),
                        escape_csv_value(guard)
                    ),
                ),
                TraceEvent::ObligationCheck {
                    event_id,
                    workflow_id,
                    role,
                    satisfied,
                    timestamp,
                    ..
                } => (
                    event_id.0.to_string(),
                    workflow_id.0.to_string(),
                    timestamp.to_rfc3339(),
                    "obligation_check",
                    format!("role={};satisfied={satisfied}", escape_csv_value(role)),
                ),
                TraceEvent::Error {
                    event_id,
                    workflow_id,
                    error,
                    context,
                    timestamp,
                    ..
                } => {
                    let ctx_str = context.as_deref().unwrap_or("");
                    (
                        event_id.0.to_string(),
                        workflow_id.0.to_string(),
                        timestamp.to_rfc3339(),
                        "error",
                        format!(
                            "error={};context={}",
                            escape_csv_value(error),
                            escape_csv_value(ctx_str)
                        ),
                    )
                }
            };

            writeln!(
                writer,
                "{},{},{},{},\"{}\"",
                event_id, workflow_id, timestamp, type_name, data
            )?;
        }

        Ok(())
    }

    fn export_lineage(
        &self,
        lineages: &[Lineage],
        writer: &mut dyn Write,
    ) -> Result<(), ExportError> {
        // Write header
        writeln!(
            writer,
            "lineage_id,source_type,source_detail,parents,transformations,created_at"
        )?;

        for lineage in lineages {
            let source_type = match &lineage.source {
                crate::lineage::DataSource::Observation { .. } => "observation",
                crate::lineage::DataSource::UserInput { .. } => "user_input",
                crate::lineage::DataSource::Computed { .. } => "computed",
                crate::lineage::DataSource::Constant { .. } => "constant",
            };

            let source_detail = match &lineage.source {
                crate::lineage::DataSource::Observation { capability, .. } => {
                    format!("capability={}", escape_csv_value(capability))
                }
                crate::lineage::DataSource::UserInput { source, .. } => {
                    format!("source={}", escape_csv_value(source))
                }
                crate::lineage::DataSource::Computed { operation, .. } => {
                    format!("operation={}", escape_csv_value(operation))
                }
                crate::lineage::DataSource::Constant { description } => {
                    format!("description={}", escape_csv_value(description))
                }
            };

            let parents = lineage
                .parents
                .iter()
                .map(|p| p.0.to_string())
                .collect::<Vec<_>>()
                .join(";");

            let transformations = lineage
                .transformations
                .iter()
                .map(|t| format!("{}:{}", t.operation, t.description))
                .collect::<Vec<_>>()
                .join(";");

            writeln!(
                writer,
                "{},{},\"{}\",\"{}\",\"{}\",{}",
                lineage.id.0,
                source_type,
                escape_csv_value(&source_detail),
                escape_csv_value(&parents),
                escape_csv_value(&transformations),
                lineage.created_at.to_rfc3339()
            )?;
        }

        Ok(())
    }

    fn format(&self) -> ExportFormat {
        ExportFormat::Csv
    }
}

/// Escape a value for use in CSV.
fn escape_csv_value(value: &str) -> String {
    value.replace('"', "\"\"").replace(['\n', '\r'], " ")
}

/// Factory function to create an exporter for a given format.
pub fn create_exporter(format: ExportFormat) -> Box<dyn AuditExporter> {
    match format {
        ExportFormat::Json => Box::new(JsonExporter::new()),
        ExportFormat::NdJson => Box::new(NdJsonExporter::new()),
        ExportFormat::Csv => Box::new(CsvExporter::new()),
        ExportFormat::ProvJson => Box::new(ProvJsonExporter::new()),
        ExportFormat::ProvN => Box::new(ProvNExporter::new()),
        ExportFormat::Cypher => Box::new(CypherExporter::new()),
    }
}

/// W3C PROV-JSON exporter.
#[derive(Debug, Clone, Default)]
pub struct ProvJsonExporter;

impl ProvJsonExporter {
    /// Create a new PROV-JSON exporter.
    pub fn new() -> Self {
        Self
    }
}

impl AuditExporter for ProvJsonExporter {
    fn export_traces(
        &self,
        events: &[TraceEvent],
        writer: &mut dyn Write,
    ) -> Result<(), ExportError> {
        // Simplified PROV-JSON structure
        let mut doc = serde_json::Map::new();
        let mut entities = Vec::new();
        let mut activities = Vec::new();

        for event in events {
            let activity_id = format!("activity_{}", event.event_id().0);
            let timestamp = event.timestamp().to_rfc3339();

            let activity = serde_json::json!({
                "prov:id": activity_id,
                "prov:startTime": timestamp,
                "type": format!("{:?}", std::mem::discriminant(event)).to_lowercase(),
            });
            activities.push(activity);

            if let TraceEvent::Observation { value, .. } = event {
                let entity_id = format!("entity_{}", event.event_id().0);
                let entity = serde_json::json!({
                    "prov:id": entity_id,
                    "value": value,
                });
                entities.push(entity);
            }
        }

        doc.insert("entity".to_string(), serde_json::Value::Array(entities));
        doc.insert("activity".to_string(), serde_json::Value::Array(activities));

        let json = serde_json::to_string_pretty(&doc)?;
        writer.write_all(json.as_bytes())?;
        Ok(())
    }

    fn export_lineage(
        &self,
        lineages: &[Lineage],
        writer: &mut dyn Write,
    ) -> Result<(), ExportError> {
        let mut doc = serde_json::Map::new();
        let mut entities = Vec::new();
        let mut derivations = Vec::new();

        for lineage in lineages {
            let entity_id = format!("entity_{}", lineage.id.0);
            let entity = serde_json::json!({
                "prov:id": entity_id,
                "source": format!("{:?}", std::mem::discriminant(&lineage.source)).to_lowercase(),
            });
            entities.push(entity);

            for parent in &lineage.parents {
                let derivation = serde_json::json!({
                    "prov:generatedEntity": format!("entity_{}", lineage.id.0),
                    "prov:usedEntity": format!("entity_{}", parent.0),
                });
                derivations.push(derivation);
            }
        }

        doc.insert("entity".to_string(), serde_json::Value::Array(entities));
        doc.insert(
            "wasDerivedFrom".to_string(),
            serde_json::Value::Array(derivations),
        );

        let json = serde_json::to_string_pretty(&doc)?;
        writer.write_all(json.as_bytes())?;
        Ok(())
    }

    fn format(&self) -> ExportFormat {
        ExportFormat::ProvJson
    }
}

/// W3C PROV-N notation exporter.
#[derive(Debug, Clone, Default)]
pub struct ProvNExporter;

impl ProvNExporter {
    /// Create a new PROV-N exporter.
    pub fn new() -> Self {
        Self
    }
}

impl AuditExporter for ProvNExporter {
    fn export_traces(
        &self,
        events: &[TraceEvent],
        writer: &mut dyn Write,
    ) -> Result<(), ExportError> {
        writeln!(writer, "document")?;

        for event in events {
            let activity_id = format!("activity_{}", event.event_id().0);
            let timestamp = event.timestamp().to_rfc3339();
            writeln!(writer, "  activity({activity_id}, {timestamp}, -)")?;
        }

        writeln!(writer, "endDocument")?;
        Ok(())
    }

    fn export_lineage(
        &self,
        lineages: &[Lineage],
        writer: &mut dyn Write,
    ) -> Result<(), ExportError> {
        writeln!(writer, "document")?;

        for lineage in lineages {
            let entity_id = format!("entity_{}", lineage.id.0);
            writeln!(writer, "  entity({entity_id})")?;

            for parent in &lineage.parents {
                let parent_id = format!("entity_{}", parent.0);
                writeln!(writer, "  wasDerivedFrom({entity_id}, {parent_id})")?;
            }
        }

        writeln!(writer, "endDocument")?;
        Ok(())
    }

    fn format(&self) -> ExportFormat {
        ExportFormat::ProvN
    }
}

/// Neo4j Cypher exporter for graph import.
#[derive(Debug, Clone, Default)]
pub struct CypherExporter;

impl CypherExporter {
    /// Create a new Cypher exporter.
    pub fn new() -> Self {
        Self
    }
}

impl AuditExporter for CypherExporter {
    fn export_traces(
        &self,
        events: &[TraceEvent],
        writer: &mut dyn Write,
    ) -> Result<(), ExportError> {
        for event in events {
            let event_id = event.event_id().0.to_string();
            let type_name = format!("{:?}", std::mem::discriminant(event))
                .split("::")
                .last()
                .unwrap_or("Event")
                .to_lowercase();

            writeln!(
                writer,
                "CREATE (e:Event {{id: '{}', type: '{}', timestamp: '{}'}})",
                event_id,
                type_name,
                event.timestamp().to_rfc3339()
            )?;
        }

        Ok(())
    }

    fn export_lineage(
        &self,
        lineages: &[Lineage],
        writer: &mut dyn Write,
    ) -> Result<(), ExportError> {
        // Create nodes
        for lineage in lineages {
            writeln!(
                writer,
                "CREATE (n:Lineage {{id: '{}', created_at: '{}'}})",
                lineage.id.0,
                lineage.created_at.to_rfc3339()
            )?;
        }

        // Create relationships
        for lineage in lineages {
            for parent in &lineage.parents {
                writeln!(
                    writer,
                    "MATCH (child:Lineage {{id: '{}'}}), (parent:Lineage {{id: '{}'}})",
                    lineage.id.0, parent.0
                )?;
                writeln!(writer, "CREATE (child)-[:DERIVED_FROM]->(parent)")?;
            }
        }

        Ok(())
    }

    fn format(&self) -> ExportFormat {
        ExportFormat::Cypher
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::{Decision, WorkflowId};

    fn create_test_events() -> Vec<TraceEvent> {
        let workflow_id = WorkflowId::new();
        vec![
            TraceEvent::workflow_started(workflow_id, "test"),
            TraceEvent::observation(workflow_id, "sensor", "42.0"),
            TraceEvent::decision(workflow_id, "policy", Decision::Permit, None::<&str>),
            TraceEvent::workflow_completed(workflow_id, true),
        ]
    }

    fn create_test_lineages() -> Vec<Lineage> {
        let parent = Lineage::observation("source");
        let parent_id = parent.id;
        let child = Lineage::computed("map", vec![parent_id]);

        vec![parent, child]
    }

    #[test]
    fn test_json_exporter() {
        let exporter = JsonExporter::new();
        let events = create_test_events();

        let mut output = Vec::new();
        exporter.export_traces(&events, &mut output).unwrap();

        let json = String::from_utf8(output).unwrap();
        assert!(json.contains("workflow_started"));
        assert!(json.contains("observation"));

        assert_eq!(exporter.format(), ExportFormat::Json);
    }

    #[test]
    fn test_ndjson_exporter() {
        let exporter = NdJsonExporter::new();
        let events = create_test_events();

        let mut output = Vec::new();
        exporter.export_traces(&events, &mut output).unwrap();

        let json = String::from_utf8(output).unwrap();
        let lines: Vec<_> = json.lines().collect();
        assert_eq!(lines.len(), 4);

        assert_eq!(exporter.format(), ExportFormat::NdJson);
    }

    #[test]
    fn test_csv_exporter() {
        let exporter = CsvExporter::new();
        let events = create_test_events();

        let mut output = Vec::new();
        exporter.export_traces(&events, &mut output).unwrap();

        let csv = String::from_utf8(output).unwrap();
        let lines: Vec<_> = csv.lines().collect();
        assert_eq!(lines.len(), 5); // header + 4 events
        assert!(lines[0].contains("event_id,workflow_id"));

        assert_eq!(exporter.format(), ExportFormat::Csv);
    }

    #[test]
    fn test_csv_exporter_lineage() {
        let exporter = CsvExporter::new();
        let lineages = create_test_lineages();

        let mut output = Vec::new();
        exporter.export_lineage(&lineages, &mut output).unwrap();

        let csv = String::from_utf8(output).unwrap();
        let lines: Vec<_> = csv.lines().collect();
        assert_eq!(lines.len(), 3); // header + 2 lineages
    }

    #[test]
    fn test_prov_json_exporter() {
        let exporter = ProvJsonExporter::new();
        let events = create_test_events();

        let mut output = Vec::new();
        exporter.export_traces(&events, &mut output).unwrap();

        let json = String::from_utf8(output).unwrap();
        assert!(json.contains("activity"));

        assert_eq!(exporter.format(), ExportFormat::ProvJson);
    }

    #[test]
    fn test_prov_n_exporter() {
        let exporter = ProvNExporter::new();
        let events = create_test_events();

        let mut output = Vec::new();
        exporter.export_traces(&events, &mut output).unwrap();

        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("document"));
        assert!(text.contains("endDocument"));

        assert_eq!(exporter.format(), ExportFormat::ProvN);
    }

    #[test]
    fn test_cypher_exporter() {
        let exporter = CypherExporter::new();
        let events = create_test_events();

        let mut output = Vec::new();
        exporter.export_traces(&events, &mut output).unwrap();

        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("CREATE"));
        assert!(text.contains("Event"));

        assert_eq!(exporter.format(), ExportFormat::Cypher);
    }

    #[test]
    fn test_export_format_extension() {
        assert_eq!(ExportFormat::Json.extension(), "json");
        assert_eq!(ExportFormat::ProvJson.extension(), "prov.json");
        assert_eq!(ExportFormat::ProvN.extension(), "provn");
        assert_eq!(ExportFormat::Csv.extension(), "csv");
        assert_eq!(ExportFormat::NdJson.extension(), "ndjson");
        assert_eq!(ExportFormat::Cypher.extension(), "cypher");
    }

    #[test]
    fn test_export_format_mime_type() {
        assert_eq!(ExportFormat::Json.mime_type(), "application/json");
        assert_eq!(ExportFormat::Csv.mime_type(), "text/csv");
        assert_eq!(ExportFormat::NdJson.mime_type(), "application/x-ndjson");
    }

    #[test]
    fn test_create_exporter() {
        let json_exporter = create_exporter(ExportFormat::Json);
        assert_eq!(json_exporter.format(), ExportFormat::Json);

        let csv_exporter = create_exporter(ExportFormat::Csv);
        assert_eq!(csv_exporter.format(), ExportFormat::Csv);
    }

    #[test]
    fn test_escape_csv_value() {
        assert_eq!(escape_csv_value("hello"), "hello");
        assert_eq!(escape_csv_value("he\"llo"), "he\"\"llo");
        assert_eq!(escape_csv_value("hello\nworld"), "hello world");
    }

    #[test]
    fn test_serde_roundtrip() {
        let format = ExportFormat::Json;
        let json = serde_json::to_string(&format).unwrap();
        let restored: ExportFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(format, restored);
    }
}
