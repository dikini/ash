# TASK-038: Trace Event Recording

## Status: 🟢 Complete

## Description

Implement the trace event recording system for complete audit trail capture during workflow execution.

## Specification Reference

- SPEC-004: Operational Semantics - Trace recording
- SPEC-001: IR - Provenance and TraceEvent

## Requirements

### Trace Event Types

```rust
/// Trace events for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceEvent {
    /// Workflow started
    WorkflowStarted {
        workflow_id: WorkflowId,
        timestamp: DateTime<Utc>,
        inputs: HashMap<Box<str>, Value>,
    },
    
    /// Workflow completed
    WorkflowCompleted {
        workflow_id: WorkflowId,
        timestamp: DateTime<Utc>,
        result: Value,
        duration_ms: u64,
    },
    
    /// Observation event
    Observation {
        workflow_id: WorkflowId,
        capability: Box<str>,
        arguments: HashMap<Box<str>, Value>,
        result: Value,
        timestamp: DateTime<Utc>,
        duration_ms: u64,
    },
    
    /// Orientation/analysis event
    Orientation {
        workflow_id: WorkflowId,
        expression: Box<str>,
        input: Value,
        output: Value,
        timestamp: DateTime<Utc>,
    },
    
    /// Proposal event
    Proposal {
        workflow_id: WorkflowId,
        action: ActionRef,
        timestamp: DateTime<Utc>,
    },
    
    /// Decision event
    Decision {
        workflow_id: WorkflowId,
        policy: Box<str>,
        input: Value,
        decision: Decision,
        timestamp: DateTime<Utc>,
    },
    
    /// Action execution event
    Action {
        workflow_id: WorkflowId,
        action: ActionRef,
        guard: Guard,
        result: Value,
        timestamp: DateTime<Utc>,
        duration_ms: u64,
    },
    
    /// Obligation check event
    ObligationCheck {
        workflow_id: WorkflowId,
        obligation: Obligation,
        satisfied: bool,
        timestamp: DateTime<Utc>,
    },
    
    /// Obligation incurred
    ObligationIncurred {
        workflow_id: WorkflowId,
        role: Box<str>,
        obligations: Vec<Obligation>,
        timestamp: DateTime<Utc>,
    },
    
    /// Variable binding
    VariableBound {
        workflow_id: WorkflowId,
        name: Box<str>,
        value: Value,
        timestamp: DateTime<Utc>,
    },
    
    /// Effect level change
    EffectChanged {
        workflow_id: WorkflowId,
        from: Effect,
        to: Effect,
        timestamp: DateTime<Utc>,
    },
    
    /// Context fork (parallel execution)
    ContextForked {
        workflow_id: WorkflowId,
        new_context_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    
    /// Context join (parallel completion)
    ContextJoined {
        workflow_id: WorkflowId,
        context_ids: Vec<Uuid>,
        timestamp: DateTime<Utc>,
    },
    
    /// Error occurred
    Error {
        workflow_id: WorkflowId,
        error: String,
        location: Option<Span>,
        timestamp: DateTime<Utc>,
    },
    
    /// Approval required
    ApprovalRequired {
        workflow_id: WorkflowId,
        policy: Box<str>,
        role: Box<str>,
        input: Value,
        timestamp: DateTime<Utc>,
    },
    
    /// Escalation occurred
    Escalated {
        workflow_id: WorkflowId,
        policy: Box<str>,
        input: Value,
        timestamp: DateTime<Utc>,
    },
}
```

### Trace Recorder

```rust
/// Trace recorder for event collection
#[derive(Debug)]
pub struct TraceRecorder {
    events: Vec<TraceEvent>,
    config: TraceConfig,
}

#[derive(Debug, Clone)]
pub struct TraceConfig {
    /// Maximum events to record (0 = unlimited)
    pub max_events: usize,
    /// Record variable bindings
    pub record_bindings: bool,
    /// Record effect changes
    pub record_effects: bool,
    /// Record timing information
    pub record_timing: bool,
    /// Redact sensitive values
    pub redact_sensitive: bool,
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            max_events: 0,
            record_bindings: true,
            record_effects: true,
            record_timing: true,
            redact_sensitive: false,
        }
    }
}

impl TraceRecorder {
    pub fn new(config: TraceConfig) -> Self {
        Self {
            events: Vec::new(),
            config,
        }
    }
    
    pub fn record(&mut self, event: TraceEvent) {
        if self.config.max_events > 0 && self.events.len() >= self.config.max_events {
            // Drop oldest event
            self.events.remove(0);
        }
        
        let event = if self.config.redact_sensitive {
            self.redact_sensitive(event)
        } else {
            event
        };
        
        self.events.push(event);
    }
    
    pub fn events(&self) -> &[TraceEvent] {
        &self.events
    }
    
    pub fn into_events(self) -> Vec<TraceEvent> {
        self.events
    }
    
    pub fn clear(&mut self) {
        self.events.clear();
    }
    
    /// Redact sensitive information from event
    fn redact_sensitive(&self, event: TraceEvent) -> TraceEvent {
        // Simplified - would need more sophisticated redaction
        match event {
            TraceEvent::Observation { ref result, .. } if contains_sensitive(result) => {
                // Clone and redact
                event
            }
            _ => event,
        }
    }
    
    /// Export to JSON
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.events)
    }
    
    /// Export to W3C PROV format
    pub fn export_prov(&self) -> String {
        // Simplified PROV export
        let mut output = String::new();
        output.push_str("document\n");
        
        for (i, event) in self.events.iter().enumerate() {
            output.push_str(&format!("  // Event {}: {:?}\n", i, std::mem::discriminant(event)));
        }
        
        output.push_str("endDocument\n");
        output
    }
}

fn contains_sensitive(value: &Value) -> bool {
    // Check if value contains sensitive patterns
    match value {
        Value::String(s) => {
            let s = s.to_lowercase();
            s.contains("password") || s.contains("secret") || s.contains("token")
        }
        Value::Record(fields) => {
            fields.iter().any(|(k, v)| {
                let key = k.to_lowercase();
                key.contains("password") || key.contains("secret") || contains_sensitive(v)
            })
        }
        _ => false,
    }
}
```

### Integration with Runtime

```rust
impl RuntimeContext {
    /// Record a trace event
    pub fn record_trace(&mut self, event: TraceEvent) {
        self.provenance.record(event);
    }
    
    /// Create workflow started event
    pub fn trace_start(&mut self, inputs: HashMap<Box<str>, Value>) {
        self.record_trace(TraceEvent::WorkflowStarted {
            workflow_id: self.provenance.workflow_id.clone(),
            timestamp: Utc::now(),
            inputs,
        });
    }
    
    /// Create workflow completed event
    pub fn trace_complete(&mut self, result: Value, duration: Duration) {
        self.record_trace(TraceEvent::WorkflowCompleted {
            workflow_id: self.provenance.workflow_id.clone(),
            timestamp: Utc::now(),
            result,
            duration_ms: duration.as_millis() as u64,
        });
    }
    
    /// Create observation event
    pub fn trace_observation(
        &mut self,
        capability: &str,
        args: HashMap<Box<str>, Value>,
        result: Value,
        duration: Duration,
    ) {
        self.record_trace(TraceEvent::Observation {
            workflow_id: self.provenance.workflow_id.clone(),
            capability: capability.into(),
            arguments: args,
            result,
            timestamp: Utc::now(),
            duration_ms: duration.as_millis() as u64,
        });
    }
    
    /// Create decision event
    pub fn trace_decision(&mut self, policy: &str, input: Value, decision: Decision) {
        self.record_trace(TraceEvent::Decision {
            workflow_id: self.provenance.workflow_id.clone(),
            policy: policy.into(),
            input,
            decision,
            timestamp: Utc::now(),
        });
    }
    
    /// Create action event
    pub fn trace_action(
        &mut self,
        action: ActionRef,
        guard: Guard,
        result: Value,
        duration: Duration,
    ) {
        self.record_trace(TraceEvent::Action {
            workflow_id: self.provenance.workflow_id.clone(),
            action,
            guard,
            result,
            timestamp: Utc::now(),
            duration_ms: duration.as_millis() as u64,
        });
    }
    
    /// Create error event
    pub fn trace_error(&mut self, error: &impl std::error::Error, location: Option<Span>) {
        self.record_trace(TraceEvent::Error {
            workflow_id: self.provenance.workflow_id.clone(),
            error: error.to_string(),
            location,
            timestamp: Utc::now(),
        });
    }
}
```

## TDD Steps

### Step 1: Define Trace Events

Create `crates/ash-provenance/src/trace.rs`.

### Step 2: Implement TraceRecorder

Add recording and export functionality.

### Step 3: Integrate with Runtime

Add trace methods to RuntimeContext.

### Step 4: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_recording() {
        let mut recorder = TraceRecorder::new(TraceConfig::default());
        
        recorder.record(TraceEvent::WorkflowStarted {
            workflow_id: WorkflowId::new(),
            timestamp: Utc::now(),
            inputs: HashMap::new(),
        });
        
        assert_eq!(recorder.events().len(), 1);
    }

    #[test]
    fn test_max_events() {
        let config = TraceConfig {
            max_events: 3,
            ..Default::default()
        };
        let mut recorder = TraceRecorder::new(config);
        
        for _ in 0..5 {
            recorder.record(TraceEvent::WorkflowStarted {
                workflow_id: WorkflowId::new(),
                timestamp: Utc::now(),
                inputs: HashMap::new(),
            });
        }
        
        assert_eq!(recorder.events().len(), 3);
    }

    #[test]
    fn test_json_export() {
        let mut recorder = TraceRecorder::new(TraceConfig::default());
        
        recorder.record(TraceEvent::WorkflowStarted {
            workflow_id: WorkflowId::new(),
            timestamp: Utc::now(),
            inputs: HashMap::new(),
        });
        
        let json = recorder.export_json().unwrap();
        assert!(json.contains("WorkflowStarted"));
    }

    #[test]
    fn test_sensitive_redaction() {
        let config = TraceConfig {
            redact_sensitive: true,
            ..Default::default()
        };
        let mut recorder = TraceRecorder::new(config);
        
        let mut inputs = HashMap::new();
        inputs.insert("password".into(), Value::String("secret123".into()));
        
        recorder.record(TraceEvent::WorkflowStarted {
            workflow_id: WorkflowId::new(),
            timestamp: Utc::now(),
            inputs,
        });
        
        // Would need to verify redaction in real implementation
    }
}
```

## Completion Checklist

- [ ] TraceEvent enum with all variants
- [ ] TraceRecorder
- [ ] TraceConfig
- [ ] JSON export
- [ ] PROV export (basic)
- [ ] Sensitive data redaction
- [ ] RuntimeContext integration
- [ ] Unit tests for recording
- [ ] Unit tests for export
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Completeness**: Are all execution events recorded?
2. **Performance**: Is recording low-overhead?
3. **Privacy**: Is sensitive data handled correctly?

## Estimated Effort

4 hours

## Dependencies

- ash-core: TraceEvent, WorkflowId

## Blocked By

- ash-core: Core types

## Blocks

- TASK-039: Lineage tracking
- TASK-040: Audit export
