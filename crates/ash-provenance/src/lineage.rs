//! Data lineage tracking for workflow values
//!
//! This module provides types for tracking the origin and transformations
//! of data values throughout workflow execution.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A unique identifier for data values with lineage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LineageId(pub Uuid);

impl LineageId {
    /// Create a new unique lineage ID.
    pub fn new() -> Self {
        LineageId(Uuid::new_v4())
    }
}

impl Default for LineageId {
    fn default() -> Self {
        Self::new()
    }
}

/// The source of a data value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DataSource {
    /// Value came from an external observation.
    Observation {
        /// The capability used for observation.
        capability: String,
        /// Timestamp of the observation.
        timestamp: DateTime<Utc>,
        /// Optional metadata about the observation.
        metadata: Option<HashMap<String, String>>,
    },
    /// Value came from user input.
    UserInput {
        /// Description of the input field/source.
        source: String,
        /// Timestamp of input.
        timestamp: DateTime<Utc>,
    },
    /// Value was computed from other values.
    Computed {
        /// The operation that computed this value.
        operation: String,
        /// IDs of parent values used in computation.
        parents: Vec<LineageId>,
        /// Timestamp of computation.
        timestamp: DateTime<Utc>,
    },
    /// Value is a constant.
    Constant {
        /// Description of the constant.
        description: String,
    },
}

impl DataSource {
    /// Create an observation data source.
    pub fn observation(capability: impl Into<String>) -> Self {
        Self::Observation {
            capability: capability.into(),
            timestamp: Utc::now(),
            metadata: None,
        }
    }

    /// Create an observation data source with metadata.
    pub fn observation_with_metadata(
        capability: impl Into<String>,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self::Observation {
            capability: capability.into(),
            timestamp: Utc::now(),
            metadata: Some(metadata),
        }
    }

    /// Create a user input data source.
    pub fn user_input(source: impl Into<String>) -> Self {
        Self::UserInput {
            source: source.into(),
            timestamp: Utc::now(),
        }
    }

    /// Create a computed data source.
    pub fn computed(operation: impl Into<String>, parents: Vec<LineageId>) -> Self {
        Self::Computed {
            operation: operation.into(),
            parents,
            timestamp: Utc::now(),
        }
    }

    /// Create a constant data source.
    pub fn constant(description: impl Into<String>) -> Self {
        Self::Constant {
            description: description.into(),
        }
    }

    /// Get the timestamp for this source, if applicable.
    pub fn timestamp(&self) -> Option<DateTime<Utc>> {
        match self {
            Self::Observation { timestamp, .. } => Some(*timestamp),
            Self::UserInput { timestamp, .. } => Some(*timestamp),
            Self::Computed { timestamp, .. } => Some(*timestamp),
            Self::Constant { .. } => None,
        }
    }
}

/// A transformation applied to data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transformation {
    /// The operation name.
    pub operation: String,
    /// Description of the transformation.
    pub description: String,
    /// When the transformation occurred.
    pub timestamp: DateTime<Utc>,
    /// Additional metadata about the transformation.
    pub metadata: Option<HashMap<String, String>>,
}

impl Transformation {
    /// Create a new transformation.
    pub fn new(operation: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            description: description.into(),
            timestamp: Utc::now(),
            metadata: None,
        }
    }

    /// Add metadata to the transformation.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let metadata = self.metadata.get_or_insert_with(HashMap::new);
        metadata.insert(key.into(), value.into());
        self
    }
}

/// Lineage information for a data value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lineage {
    /// Unique identifier for this lineage record.
    pub id: LineageId,
    /// The source of the data.
    pub source: DataSource,
    /// Transformations applied to the data.
    pub transformations: Vec<Transformation>,
    /// IDs of parent values (for computed values).
    pub parents: Vec<LineageId>,
    /// When this lineage record was created.
    pub created_at: DateTime<Utc>,
    /// Optional metadata.
    pub metadata: Option<HashMap<String, String>>,
}

impl Lineage {
    /// Create a new lineage record from a data source.
    pub fn new(source: DataSource) -> Self {
        Self {
            id: LineageId::new(),
            source: source.clone(),
            transformations: Vec::new(),
            parents: match source {
                DataSource::Computed { parents, .. } => parents,
                _ => Vec::new(),
            },
            created_at: Utc::now(),
            metadata: None,
        }
    }

    /// Create a lineage record for an observation.
    pub fn observation(capability: impl Into<String>) -> Self {
        Self::new(DataSource::observation(capability))
    }

    /// Create a lineage record for user input.
    pub fn user_input(source: impl Into<String>) -> Self {
        Self::new(DataSource::user_input(source))
    }

    /// Create a lineage record for a computed value.
    pub fn computed(operation: impl Into<String>, parents: Vec<LineageId>) -> Self {
        Self::new(DataSource::computed(operation, parents))
    }

    /// Create a lineage record for a constant.
    pub fn constant(description: impl Into<String>) -> Self {
        Self::new(DataSource::constant(description))
    }

    /// Add a transformation to this lineage.
    pub fn add_transformation(&mut self, transformation: Transformation) {
        self.transformations.push(transformation);
    }

    /// Create a new transformation and add it.
    pub fn transform(&mut self, operation: impl Into<String>, description: impl Into<String>) {
        self.add_transformation(Transformation::new(operation, description));
    }

    /// Add a parent lineage ID.
    pub fn add_parent(&mut self, parent_id: LineageId) {
        self.parents.push(parent_id);
    }

    /// Add metadata to this lineage.
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let metadata = self.metadata.get_or_insert_with(HashMap::new);
        metadata.insert(key.into(), value.into());
    }

    /// Get the current value type based on transformations.
    pub fn value_type(&self) -> &str {
        self.transformations
            .last()
            .map(|t| t.operation.as_str())
            .unwrap_or_else(|| match &self.source {
                DataSource::Observation { .. } => "observation",
                DataSource::UserInput { .. } => "user_input",
                DataSource::Computed { operation, .. } => operation.as_str(),
                DataSource::Constant { .. } => "constant",
            })
    }

    /// Check if this value derives from a specific parent.
    pub fn derives_from(&self, parent_id: LineageId) -> bool {
        self.parents.contains(&parent_id)
    }
}

/// Tracks lineage for multiple values.
#[derive(Debug, Clone, Default)]
pub struct LineageTracker {
    lineages: HashMap<LineageId, Lineage>,
}

impl LineageTracker {
    /// Create a new lineage tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new lineage record.
    pub fn register(&mut self, lineage: Lineage) -> LineageId {
        let id = lineage.id;
        self.lineages.insert(id, lineage);
        id
    }

    /// Create and register an observation lineage.
    pub fn observe(&mut self, capability: impl Into<String>) -> LineageId {
        let lineage = Lineage::observation(capability);
        self.register(lineage)
    }

    /// Create and register a user input lineage.
    pub fn user_input(&mut self, source: impl Into<String>) -> LineageId {
        let lineage = Lineage::user_input(source);
        self.register(lineage)
    }

    /// Create and register a computed lineage.
    pub fn compute(
        &mut self,
        operation: impl Into<String>,
        parents: Vec<LineageId>,
    ) -> LineageId {
        let lineage = Lineage::computed(operation, parents);
        self.register(lineage)
    }

    /// Create and register a constant lineage.
    pub fn constant(&mut self, description: impl Into<String>) -> LineageId {
        let lineage = Lineage::constant(description);
        self.register(lineage)
    }

    /// Get a lineage record by ID.
    pub fn get(&self, id: LineageId) -> Option<&Lineage> {
        self.lineages.get(&id)
    }

    /// Get a mutable lineage record by ID.
    pub fn get_mut(&mut self, id: LineageId) -> Option<&mut Lineage> {
        self.lineages.get_mut(&id)
    }

    /// Add a transformation to an existing lineage.
    pub fn transform(
        &mut self,
        id: LineageId,
        operation: impl Into<String>,
        description: impl Into<String>,
    ) -> Option<&mut Lineage> {
        #[allow(clippy::manual_inspect)]
        self.lineages.get_mut(&id).map(|lineage| {
            lineage.transform(operation, description);
            lineage
        })
    }

    /// Check if a lineage exists.
    pub fn contains(&self, id: LineageId) -> bool {
        self.lineages.contains_key(&id)
    }

    /// Get the number of tracked lineages.
    pub fn len(&self) -> usize {
        self.lineages.len()
    }

    /// Check if the tracker is empty.
    pub fn is_empty(&self) -> bool {
        self.lineages.is_empty()
    }

    /// Get all lineage IDs.
    pub fn ids(&self) -> impl Iterator<Item = LineageId> + '_ {
        self.lineages.keys().copied()
    }

    /// Get all lineages.
    pub fn all(&self) -> impl Iterator<Item = &Lineage> {
        self.lineages.values()
    }

    /// Get the ancestry (parents, grandparents, etc.) for a lineage.
    pub fn ancestry(&self, id: LineageId) -> Vec<LineageId> {
        let mut result = Vec::new();
        let mut to_visit = vec![id];
        let mut visited = std::collections::HashSet::new();

        while let Some(current) = to_visit.pop() {
            if !visited.insert(current) {
                continue;
            }

            if let Some(lineage) = self.lineages.get(&current) {
                for parent in &lineage.parents {
                    result.push(*parent);
                    to_visit.push(*parent);
                }
            }
        }

        result
    }

    /// Get all descendants of a lineage (values that have this as an ancestor).
    pub fn descendants(&self, id: LineageId) -> Vec<LineageId> {
        self.lineages
            .values()
            .filter(|lineage| lineage.derives_from(id))
            .map(|lineage| lineage.id)
            .collect()
    }

    /// Find lineages by source type.
    pub fn by_source(&self, source_type: &str) -> Vec<&Lineage> {
        self.lineages
            .values()
            .filter(|lineage| match &lineage.source {
                DataSource::Observation { .. } if source_type == "observation" => true,
                DataSource::UserInput { .. } if source_type == "user_input" => true,
                DataSource::Computed { .. } if source_type == "computed" => true,
                DataSource::Constant { .. } if source_type == "constant" => true,
                _ => false,
            })
            .collect()
    }

    /// Create a new computed value by combining multiple existing lineages.
    pub fn combine(
        &mut self,
        operation: impl Into<String>,
        inputs: Vec<LineageId>,
    ) -> LineageId {
        self.compute(operation, inputs)
    }

    /// Clear all tracked lineages.
    pub fn clear(&mut self) {
        self.lineages.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lineage_id_unique() {
        let id1 = LineageId::new();
        let id2 = LineageId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_lineage_observation() {
        let lineage = Lineage::observation("temperature_sensor");
        match &lineage.source {
            DataSource::Observation { capability, .. } => {
                assert_eq!(capability, "temperature_sensor");
            }
            _ => panic!("wrong source type"),
        }
        assert!(lineage.parents.is_empty());
    }

    #[test]
    fn test_lineage_computed() {
        let parent1 = LineageId::new();
        let parent2 = LineageId::new();
        let lineage = Lineage::computed("sum", vec![parent1, parent2]);

        assert_eq!(lineage.parents.len(), 2);
        assert!(lineage.derives_from(parent1));
        assert!(lineage.derives_from(parent2));
    }

    #[test]
    fn test_transformation() {
        let transform = Transformation::new("filter", "remove nulls")
            .with_metadata("threshold", "0.5");

        assert_eq!(transform.operation, "filter");
        assert_eq!(transform.description, "remove nulls");
        assert_eq!(
            transform.metadata.as_ref().unwrap().get("threshold"),
            Some(&"0.5".to_string())
        );
    }

    #[test]
    fn test_lineage_transformations() {
        let mut lineage = Lineage::observation("data");
        lineage.transform("parse", "parse json");
        lineage.transform("validate", "check schema");

        assert_eq!(lineage.transformations.len(), 2);
        assert_eq!(lineage.value_type(), "validate");
    }

    #[test]
    fn test_lineage_tracker_register() {
        let mut tracker = LineageTracker::new();
        let lineage = Lineage::constant("pi");
        let id = tracker.register(lineage);

        assert!(tracker.contains(id));
        assert_eq!(tracker.len(), 1);
    }

    #[test]
    fn test_lineage_tracker_observe() {
        let mut tracker = LineageTracker::new();
        let id = tracker.observe("sensor");

        let lineage = tracker.get(id).unwrap();
        match &lineage.source {
            DataSource::Observation { capability, .. } => assert_eq!(capability, "sensor"),
            _ => panic!("wrong source"),
        }
    }

    #[test]
    fn test_lineage_tracker_compute() {
        let mut tracker = LineageTracker::new();
        let input1 = tracker.observe("a");
        let input2 = tracker.observe("b");

        let result = tracker.compute("add", vec![input1, input2]);

        let lineage = tracker.get(result).unwrap();
        assert_eq!(lineage.parents.len(), 2);
    }

    #[test]
    fn test_lineage_tracker_transform() {
        let mut tracker = LineageTracker::new();
        let id = tracker.observe("data");

        tracker.transform(id, "format", "convert to csv");

        let lineage = tracker.get(id).unwrap();
        assert_eq!(lineage.transformations.len(), 1);
    }

    #[test]
    fn test_lineage_tracker_ancestry() {
        let mut tracker = LineageTracker::new();
        let grandparent = tracker.observe("source");
        let parent = tracker.compute("map", vec![grandparent]);
        let child = tracker.compute("reduce", vec![parent]);

        let ancestry = tracker.ancestry(child);
        assert!(ancestry.contains(&parent));
        assert!(ancestry.contains(&grandparent));
    }

    #[test]
    fn test_lineage_tracker_descendants() {
        let mut tracker = LineageTracker::new();
        let parent = tracker.observe("source");
        let child1 = tracker.compute("map", vec![parent]);
        let child2 = tracker.compute("filter", vec![parent]);

        let descendants = tracker.descendants(parent);
        assert_eq!(descendants.len(), 2);
        assert!(descendants.contains(&child1));
        assert!(descendants.contains(&child2));
    }

    #[test]
    fn test_lineage_tracker_by_source() {
        let mut tracker = LineageTracker::new();
        tracker.observe("sensor");
        tracker.user_input("config");
        tracker.constant("pi");

        let observations = tracker.by_source("observation");
        assert_eq!(observations.len(), 1);

        let inputs = tracker.by_source("user_input");
        assert_eq!(inputs.len(), 1);

        let constants = tracker.by_source("constant");
        assert_eq!(constants.len(), 1);
    }

    #[test]
    fn test_lineage_tracker_combine() {
        let mut tracker = LineageTracker::new();
        let a = tracker.observe("a");
        let b = tracker.observe("b");
        let c = tracker.observe("c");

        let combined = tracker.combine("merge", vec![a, b, c]);
        let lineage = tracker.get(combined).unwrap();

        assert_eq!(lineage.parents.len(), 3);
    }

    #[test]
    fn test_lineage_serde_roundtrip() {
        let lineage = Lineage::observation("test");
        let json = serde_json::to_string(&lineage).unwrap();
        let restored: Lineage = serde_json::from_str(&json).unwrap();

        assert_eq!(lineage.id, restored.id);
        assert_eq!(lineage.parents, restored.parents);
    }

    #[test]
    fn test_data_source_timestamps() {
        let obs = DataSource::observation("test");
        assert!(obs.timestamp().is_some());

        let input = DataSource::user_input("field");
        assert!(input.timestamp().is_some());

        let computed = DataSource::computed("add", vec![]);
        assert!(computed.timestamp().is_some());

        let constant = DataSource::constant("pi");
        assert!(constant.timestamp().is_none());
    }

    #[test]
    fn test_lineage_metadata() {
        let mut lineage = Lineage::observation("test");
        lineage.add_metadata("key", "value");
        lineage.add_metadata("foo", "bar");

        let metadata = lineage.metadata.as_ref().unwrap();
        assert_eq!(metadata.get("key"), Some(&"value".to_string()));
        assert_eq!(metadata.get("foo"), Some(&"bar".to_string()));
    }
}
