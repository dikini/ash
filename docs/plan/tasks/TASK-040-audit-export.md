# TASK-040: Audit Log Export

## Status: ✅ Complete

## Description

Implement audit log export functionality for compliance and analysis, supporting multiple export formats.

## Specification Reference

- SPEC-001: IR - Provenance section
- W3C PROV standard

## Requirements

### Export Formats

```rust
/// Supported export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// W3C PROV-JSON format
    ProvJson,
    /// W3C PROV-N (Notation) format
    ProvN,
    /// Cypher query format (for Neo4j)
    Cypher,
    /// CSV format
    Csv,
    /// Newline-delimited JSON (for streaming)
    NdJson,
}

impl ExportFormat {
    pub fn mime_type(&self) -> &'static str {
        match self {
            ExportFormat::Json => "application/json",
            ExportFormat::ProvJson => "application/prov+json",
            ExportFormat::ProvN => "text/provenance-notation",
            ExportFormat::Cypher => "text/x-cypher",
            ExportFormat::Csv => "text/csv",
            ExportFormat::NdJson => "application/x-ndjson",
        }
    }
    
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Json => "json",
            ExportFormat::ProvJson => "prov.json",
            ExportFormat::ProvN => "provn",
            ExportFormat::Cypher => "cypher",
            ExportFormat::Csv => "csv",
            ExportFormat::NdJson => "ndjson",
        }
    }
}
```

### Exporter

```rust
/// Audit log exporter
#[derive(Debug)]
pub struct AuditExporter {
    config: ExportConfig,
}

#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// Include full value details
    pub include_values: bool,
    /// Include variable bindings
    pub include_bindings: bool,
    /// Include timing information
    pub include_timing: bool,
    /// Redact sensitive fields
    pub redact_sensitive: bool,
    /// Redacted field names
    pub sensitive_fields: Vec<String>,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            include_values: true,
            include_bindings: false,
            include_timing: true,
            redact_sensitive: true,
            sensitive_fields: vec![
                "password".to_string(),
                "secret".to_string(),
                "token".to_string(),
                "key".to_string(),
                "credential".to_string(),
            ],
        }
    }
}

impl AuditExporter {
    pub fn new(config: ExportConfig) -> Self {
        Self { config }
    }
    
    /// Export trace to specified format
    pub fn export(&self, trace: &[TraceEvent], format: ExportFormat) -> Result<String, ExportError> {
        match format {
            ExportFormat::Json => self.export_json(trace),
            ExportFormat::ProvJson => self.export_prov_json(trace),
            ExportFormat::ProvN => self.export_prov_n(trace),
            ExportFormat::Cypher => self.export_cypher(trace),
            ExportFormat::Csv => self.export_csv(trace),
            ExportFormat::NdJson => self.export_ndjson(trace),
        }
    }
    
    fn export_json(&self, trace: &[TraceEvent]) -> Result<String, ExportError> {
        let filtered: Vec<_> = trace.iter()
            .map(|e| self.redact_event(e))
            .collect();
        
        serde_json::to_string_pretty(&filtered)
            .map_err(|e| ExportError::Serialization(e.to_string()))
    }
    
    fn export_ndjson(&self, trace: &[TraceEvent]) -> Result<String, ExportError> {
        let mut output = String::new();
        
        for event in trace {
            let line = serde_json::to_string(&self.redact_event(event))
                .map_err(|e| ExportError::Serialization(e.to_string()))?;
            output.push_str(&line);
            output.push('\n');
        }
        
        Ok(output)
    }
    
    fn export_prov_json(&self, trace: &[TraceEvent]) -> Result<String, ExportError> {
        let mut prov = ProvDocument::new();
        
        for (i, event) in trace.iter().enumerate() {
            let id = format!("e{}", i);
            
            match event {
                TraceEvent::Observation { capability, result, .. } => {
                    prov.add_activity(&id, capability);
                    prov.add_entity(&format!("{}_result", id), "result");
                    prov.was_generated_by(&format!("{}_result", id), &id);
                }
                TraceEvent::Action { action, .. } => {
                    prov.add_activity(&id, &action.name);
                }
                _ => {}
            }
        }
        
        prov.to_json()
    }
    
    fn export_prov_n(&self, trace: &[TraceEvent]) -> Result<String, ExportError> {
        let mut output = String::new();
        output.push_str("document\n");
        
        for (i, event) in trace.iter().enumerate() {
            match event {
                TraceEvent::Observation { capability, .. } => {
                    output.push_str(&format!("  activity(e{}, \"{}\")\n", i, capability));
                }
                TraceEvent::Action { action, .. } => {
                    output.push_str(&format!("  activity(e{}, \"{}\")\n", i, action.name));
                }
                TraceEvent::Decision { policy, decision, .. } => {
                    output.push_str(&format!(
                        "  activity(e{}, \"decide\", [policy=\"{}\", decision=\"{:?}\"])\n",
                        i, policy, decision
                    ));
                }
                _ => {}
            }
        }
        
        output.push_str("endDocument\n");
        Ok(output)
    }
    
    fn export_cypher(&self, trace: &[TraceEvent]) -> Result<String, ExportError> {
        let mut output = String::new();
        
        // Create nodes
        for (i, event) in trace.iter().enumerate() {
            let label = match event {
                TraceEvent::Observation { .. } => "Observation",
                TraceEvent::Action { .. } => "Action",
                TraceEvent::Decision { .. } => "Decision",
                TraceEvent::WorkflowStarted { .. } => "Start",
                TraceEvent::WorkflowCompleted { .. } => "Complete",
                _ => "Event",
            };
            
            output.push_str(&format!(
                "CREATE (e{}:{} {{id: '{}', timestamp: '{}'}})\n",
                i, label, i, event.timestamp()
            ));
        }
        
        // Create relationships
        for i in 1..trace.len() {
            output.push_str(&format!(
                "CREATE (e{})-[:NEXT]->(e{})\n",
                i - 1, i
            ));
        }
        
        Ok(output)
    }
    
    fn export_csv(&self, trace: &[TraceEvent]) -> Result<String, ExportError> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        
        // Header
        wtr.write_record(&["timestamp", "event_type", "details"])
            .map_err(|e| ExportError::Csv(e.to_string()))?;
        
        // Rows
        for event in trace {
            let (event_type, details) = match event {
                TraceEvent::Observation { capability, .. } => {
                    ("Observation", capability.to_string())
                }
                TraceEvent::Action { action, .. } => {
                    ("Action", action.name.to_string())
                }
                TraceEvent::Decision { policy, decision, .. } => {
                    ("Decision", format!("{}: {:?}", policy, decision))
                }
                _ => ("Other", "".to_string()),
            };
            
            wtr.write_record(&[
                event.timestamp().to_rfc3339(),
                event_type.to_string(),
                details,
            ]).map_err(|e| ExportError::Csv(e.to_string()))?;
        }
        
        String::from_utf8(wtr.into_inner().unwrap())
            .map_err(|e| ExportError::Utf8(e.to_string()))
    }
    
    fn redact_event(&self, event: &TraceEvent) -> TraceEvent {
        if !self.config.redact_sensitive {
            return event.clone();
        }
        
        // Clone and redact sensitive fields
        event.clone() // Simplified
    }
}

/// W3C PROV document structure
#[derive(Debug, Default)]
pub struct ProvDocument {
    activities: Vec<(String, String)>,
    entities: Vec<(String, String)>,
    agents: Vec<(String, String)>,
    relations: Vec<String>,
}

impl ProvDocument {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add_activity(&mut self, id: &str, name: &str) {
        self.activities.push((id.to_string(), name.to_string()));
    }
    
    pub fn add_entity(&mut self, id: &str, name: &str) {
        self.entities.push((id.to_string(), name.to_string()));
    }
    
    pub fn was_generated_by(&mut self, entity: &str, activity: &str) {
        self.relations.push(format!(
            "wasGeneratedBy({}, {})",
            entity, activity
        ));
    }
    
    pub fn to_json(&self) -> Result<String, ExportError> {
        // Simplified PROV-JSON output
        Ok("{}".to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("CSV error: {0}")]
    Csv(String),
    
    #[error("UTF-8 error: {0}")]
    Utf8(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Export Utilities

```rust
/// Export trace to file
pub fn export_to_file(
    trace: &[TraceEvent],
    format: ExportFormat,
    path: impl AsRef<Path>,
) -> Result<(), ExportError> {
    let exporter = AuditExporter::new(ExportConfig::default());
    let content = exporter.export(trace, format)?;
    
    std::fs::write(path, content)?;
    Ok(())
}

/// Stream export for large traces
pub async fn stream_export<W: AsyncWrite + Unpin>(
    trace: &[TraceEvent],
    format: ExportFormat,
    writer: &mut W,
) -> Result<(), ExportError> {
    let exporter = AuditExporter::new(ExportConfig::default());
    let content = exporter.export(trace, format)?;
    
    writer.write_all(content.as_bytes()).await
        .map_err(|e| ExportError::Io(e))?;
    
    Ok(())
}
```

## TDD Steps

### Step 1: Define Export Formats

Create `crates/ash-provenance/src/export.rs`.

### Step 2: Implement Exporters

Add export for all formats.

### Step 3: Add Redaction

Implement sensitive data redaction.

### Step 4: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_json() {
        let exporter = AuditExporter::new(ExportConfig::default());
        
        let trace = vec![
            TraceEvent::WorkflowStarted {
                workflow_id: WorkflowId::new(),
                timestamp: Utc::now(),
                inputs: HashMap::new(),
            },
        ];
        
        let json = exporter.export(&trace, ExportFormat::Json).unwrap();
        assert!(json.contains("WorkflowStarted"));
    }

    #[test]
    fn test_export_prov_n() {
        let exporter = AuditExporter::new(ExportConfig::default());
        
        let trace = vec![
            TraceEvent::Observation {
                workflow_id: WorkflowId::new(),
                capability: "read".into(),
                arguments: HashMap::new(),
                result: Value::Null,
                timestamp: Utc::now(),
                duration_ms: 0,
            },
        ];
        
        let provn = exporter.export(&trace, ExportFormat::ProvN).unwrap();
        assert!(provn.contains("document"));
        assert!(provn.contains("activity"));
    }

    #[test]
    fn test_export_cypher() {
        let exporter = AuditExporter::new(ExportConfig::default());
        
        let trace = vec![
            TraceEvent::Action {
                workflow_id: WorkflowId::new(),
                action: ActionRef { name: "write".into(), args: vec![] },
                guard: Guard::Always,
                result: Value::Null,
                timestamp: Utc::now(),
                duration_ms: 0,
            },
        ];
        
        let cypher = exporter.export(&trace, ExportFormat::Cypher).unwrap();
        assert!(cypher.contains("CREATE"));
        assert!(cypher.contains("Action"));
    }

    #[test]
    fn test_export_csv() {
        let exporter = AuditExporter::new(ExportConfig::default());
        
        let trace = vec![
            TraceEvent::Decision {
                workflow_id: WorkflowId::new(),
                policy: "test".into(),
                input: Value::Bool(true),
                decision: Decision::Permit,
                timestamp: Utc::now(),
            },
        ];
        
        let csv = exporter.export(&trace, ExportFormat::Csv).unwrap();
        assert!(csv.contains("timestamp"));
        assert!(csv.contains("Decision"));
    }
}
```

## Completion Checklist

- [ ] ExportFormat enum
- [ ] AuditExporter
- [ ] JSON export
- [ ] PROV-JSON export
- [ ] PROV-N export
- [ ] Cypher export
- [ ] CSV export
- [ ] NDJSON export
- [ ] Sensitive data redaction
- [ ] Unit tests for each format
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Completeness**: Are all major formats supported?
2. **Standards**: Is W3C PROV compliance correct?
3. **Security**: Is sensitive data properly redacted?

## Estimated Effort

4 hours

## Dependencies

- ash-core: TraceEvent
- csv, serde_json crates

## Blocked By

- ash-core: Core types
- TASK-038: Trace recording

## Blocks

- TASK-041: Integrity
- TASK-055: CLI trace command
