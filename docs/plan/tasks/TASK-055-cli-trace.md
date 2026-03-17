# TASK-055: ash trace Command

## Status: 🟢 Complete

## Objective

Implement the `ash trace` command for executing workflows with full provenance capture.

## Test Strategy

```rust
#[test]
fn test_trace_creates_output() {
    let temp_dir = TempDir::new().unwrap();
    let trace_file = temp_dir.path().join("trace.json");
    
    TraceCommand::run(&[
        "--output", trace_file.to_str().unwrap(),
        "test_data/workflow.ash"
    ]).unwrap();
    
    assert!(trace_file.exists());
    let trace: serde_json::Value = serde_json::from_reader(
        File::open(&trace_file).unwrap()
    ).unwrap();
    assert!(trace.get("trace_id").is_some());
    assert!(trace.get("events").is_some());
}

#[test]
fn test_trace_formats() {
    // Test JSON format
    // Test PROV-N format
    // Test Cypher format
}
```

## Implementation

```rust
#[derive(Parser)]
pub struct TraceArgs {
    /// Workflow file
    #[arg(required = true)]
    pub workflow: PathBuf,
    
    /// Trace output file
    #[arg(short, long, default_value = "trace.json")]
    pub output: PathBuf,
    
    /// Trace format
    #[arg(short, long, value_enum, default_value = "json")]
    pub format: TraceFormat,
    
    /// Cryptographically sign trace
    #[arg(long)]
    pub sign: bool,
    
    /// Export format
    #[arg(long, value_enum)]
    pub export: Option<ExportFormat>,
}
```

## Completion Criteria

- [ ] Executes workflow with trace capture
- [ ] Multiple trace formats (json, provn, cypher)
- [ ] Exports to W3C PROV, Dublin Core
- [ ] Optional cryptographic signing
- [ ] Tests pass

## Dependencies

- TASK-038: Trace recording
- TASK-040: Audit export

## Estimation

6 hours
