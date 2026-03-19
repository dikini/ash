# TASK-039: Lineage Tracking

## Status: ✅ Complete

## Description

Implement data lineage tracking to trace the origin and transformation of values through workflow execution.

## Specification Reference

- SPEC-004: Operational Semantics - Provenance operations
- SHARO_CORE_LANGUAGE.md - Section 2.1 Provenance

## Requirements

### Lineage Types

```rust
/// Data lineage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lineage {
    /// Unique lineage ID
    pub id: Uuid,
    
    /// Source of the data
    pub source: DataSource,
    
    /// Transformations applied
    pub transformations: Vec<Transformation>,
    
    /// Parent lineages (for merged data)
    pub parents: Vec<Uuid>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Data source types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSource {
    /// External observation
    Observation {
        capability: Box<str>,
        timestamp: DateTime<Utc>,
    },
    /// User input
    UserInput {
        parameter: Box<str>,
    },
    /// Derived from computation
    Computed {
        operation: Box<str>,
        inputs: Vec<Uuid>,
    },
    /// Constant value
    Constant,
}

/// Transformation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transformation {
    /// Operation name
    pub operation: Box<str>,
    /// Operation parameters
    pub parameters: HashMap<Box<str>, Value>,
    /// When the transformation occurred
    pub timestamp: DateTime<Utc>,
}

/// Value with lineage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineagedValue {
    /// The actual value
    pub value: Value,
    /// Lineage information
    pub lineage: Lineage,
}
```

### Lineage Tracker

```rust
/// Tracks data lineage during execution
#[derive(Debug, Default)]
pub struct LineageTracker {
    lineages: HashMap<Uuid, Lineage>,
    variable_lineage: HashMap<Box<str>, Uuid>,
}

impl LineageTracker {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create lineage for observed data
    pub fn observe(&mut self, capability: &str) -> Lineage {
        let lineage = Lineage {
            id: Uuid::new_v4(),
            source: DataSource::Observation {
                capability: capability.into(),
                timestamp: Utc::now(),
            },
            transformations: vec![],
            parents: vec![],
            created_at: Utc::now(),
        };
        
        self.lineages.insert(lineage.id, lineage.clone());
        lineage
    }
    
    /// Create lineage for user input
    pub fn user_input(&mut self, parameter: &str) -> Lineage {
        let lineage = Lineage {
            id: Uuid::new_v4(),
            source: DataSource::UserInput {
                parameter: parameter.into(),
            },
            transformations: vec![],
            parents: vec![],
            created_at: Utc::now(),
        };
        
        self.lineages.insert(lineage.id, lineage.clone());
        lineage
    }
    
    /// Transform a value, recording the transformation
    pub fn transform(
        &mut self,
        input_lineage: &Lineage,
        operation: &str,
        parameters: HashMap<Box<str>, Value>,
    ) -> Lineage {
        let mut new_lineage = Lineage {
            id: Uuid::new_v4(),
            source: DataSource::Computed {
                operation: operation.into(),
                inputs: vec![input_lineage.id],
            },
            transformations: input_lineage.transformations.clone(),
            parents: vec![input_lineage.id],
            created_at: Utc::now(),
        };
        
        new_lineage.transformations.push(Transformation {
            operation: operation.into(),
            parameters,
            timestamp: Utc::now(),
        });
        
        self.lineages.insert(new_lineage.id, new_lineage.clone());
        new_lineage
    }
    
    /// Merge multiple lineages
    pub fn merge(&mut self, parent_lineages: &[&Lineage], operation: &str) -> Lineage {
        let parents: Vec<_> = parent_lineages.iter().map(|l| l.id).collect();
        
        let mut transformations = vec![];
        for parent in parent_lineages {
            transformations.extend(parent.transformations.clone());
        }
        
        let lineage = Lineage {
            id: Uuid::new_v4(),
            source: DataSource::Computed {
                operation: operation.into(),
                inputs: parents.clone(),
            },
            transformations,
            parents,
            created_at: Utc::now(),
        };
        
        self.lineages.insert(lineage.id, lineage.clone());
        lineage
    }
    
    /// Track variable binding
    pub fn bind_variable(&mut self, name: &str, lineage_id: Uuid) {
        self.variable_lineage.insert(name.into(), lineage_id);
    }
    
    /// Get lineage for a variable
    pub fn get_variable_lineage(&self, name: &str) -> Option<&Lineage> {
        self.variable_lineage.get(name)
            .and_then(|id| self.lineages.get(id))
    }
    
    /// Get full lineage history
    pub fn get_full_lineage(&self, id: Uuid) -> Vec<&Lineage> {
        let mut result = vec![];
        let mut to_visit = vec![id];
        let mut visited = HashSet::new();
        
        while let Some(current_id) = to_visit.pop() {
            if visited.contains(&current_id) {
                continue;
            }
            visited.insert(current_id);
            
            if let Some(lineage) = self.lineages.get(&current_id) {
                result.push(lineage);
                to_visit.extend(&lineage.parents);
            }
        }
        
        result
    }
    
    /// Export lineage graph
    pub fn export_graph(&self) -> LineageGraph {
        LineageGraph {
            nodes: self.lineages.values().cloned().collect(),
            edges: self.lineages.values()
                .flat_map(|l| l.parents.iter().map(move |p| (*p, l.id)))
                .collect(),
        }
    }
}

/// Lineage graph structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageGraph {
    pub nodes: Vec<Lineage>,
    pub edges: Vec<(Uuid, Uuid)>,
}

impl LineageGraph {
    /// Export to Graphviz DOT format
    pub fn to_dot(&self) -> String {
        let mut dot = String::new();
        dot.push_str("digraph Lineage {\n");
        
        for node in &self.nodes {
            let label = format!("{:?}", node.source).replace('"', "\\\"");
            dot.push_str(&format!("  \"{}\" [label=\"{}\"];\n", node.id, label));
        }
        
        for (from, to) in &self.edges {
            dot.push_str(&format!("  \"{}\" -> \"{}\";\n", from, to));
        }
        
        dot.push_str("}\n");
        dot
    }
    
    /// Export to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
```

### Integration with Runtime

```rust
impl RuntimeContext {
    /// Create a value with observed lineage
    pub fn observe_lineaged(&mut self, capability: &str, value: Value) -> LineagedValue {
        let lineage = self.provenance.lineage.observe(capability);
        
        LineagedValue { value, lineage }
    }
    
    /// Transform a value with lineage
    pub fn transform_lineaged(
        &mut self,
        input: &LineagedValue,
        operation: &str,
        parameters: HashMap<Box<str>, Value>,
        new_value: Value,
    ) -> LineagedValue {
        let lineage = self.provenance.lineage.transform(
            &input.lineage,
            operation,
            parameters,
        );
        
        LineagedValue {
            value: new_value,
            lineage,
        }
    }
    
    /// Bind a lineaged value to a variable
    pub fn bind_lineaged(&mut self, name: &str, value: &LineagedValue) {
        self.provenance.lineage.bind_variable(name, value.lineage.id);
    }
}
```

## TDD Steps

### Step 1: Define Lineage Types

Create `crates/ash-provenance/src/lineage.rs`.

### Step 2: Implement LineageTracker

Add tracking and graph export.

### Step 3: Add Graph Export

Add DOT and JSON export.

### Step 4: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observe_lineage() {
        let mut tracker = LineageTracker::new();
        let lineage = tracker.observe("read_file");
        
        assert!(matches!(lineage.source, DataSource::Observation { .. }));
        assert!(tracker.lineages.contains_key(&lineage.id));
    }

    #[test]
    fn test_transform_lineage() {
        let mut tracker = LineageTracker::new();
        let input = tracker.observe("read_file");
        
        let output = tracker.transform(&input, "parse_json", HashMap::new());
        
        assert_eq!(output.parents, vec![input.id]);
        assert_eq!(output.transformations.len(), 1);
    }

    #[test]
    fn test_merge_lineage() {
        let mut tracker = LineageTracker::new();
        let input1 = tracker.observe("source1");
        let input2 = tracker.observe("source2");
        
        let merged = tracker.merge(&[&input1, &input2], "combine");
        
        assert_eq!(merged.parents.len(), 2);
    }

    #[test]
    fn test_full_lineage_history() {
        let mut tracker = LineageTracker::new();
        let input = tracker.observe("source");
        let transformed = tracker.transform(&input, "op1", HashMap::new());
        let final_val = tracker.transform(&transformed, "op2", HashMap::new());
        
        let history = tracker.get_full_lineage(final_val.id);
        
        assert_eq!(history.len(), 3);
    }

    #[test]
    fn test_export_dot() {
        let mut tracker = LineageTracker::new();
        let input = tracker.observe("source");
        tracker.transform(&input, "op", HashMap::new());
        
        let graph = tracker.export_graph();
        let dot = graph.to_dot();
        
        assert!(dot.contains("digraph Lineage"));
        assert!(dot.contains("->"));
    }
}
```

## Completion Checklist

- [ ] Lineage struct
- [ ] DataSource enum
- [ ] Transformation record
- [ ] LineageTracker
- [ ] LineageGraph
- [ ] DOT export
- [ ] JSON export
- [ ] Variable binding tracking
- [ ] Full history retrieval
- [ ] Unit tests for lineage
- [ ] Unit tests for graph export
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Completeness**: Is full lineage tracked?
2. **Performance**: Is lineage tracking efficient?
3. **Export quality**: Are exports useful?

## Estimated Effort

4 hours

## Dependencies

- ash-core: Value, Provenance

## Blocked By

- ash-core: Core types

## Blocks

- TASK-040: Audit export
