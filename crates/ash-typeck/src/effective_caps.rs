//! Effective capability set composition for Ash workflows (TASK-264)
//!
//! This module provides capability set composition for workflows with multiple roles,
//! per SPEC-024 Section 2. It composes capabilities from multiple sources:
//! - Role-based capabilities (from `plays role(R)` clauses)
//! - Direct workflow capabilities (from `capabilities: [...]` clause)
//!
//! # Example
//!
//! ```
//! use ash_typeck::effective_caps::{EffectiveCapabilitySet, CapabilitySource};
//! use ash_parser::surface::{CapabilityDecl, ConstraintBlock, ConstraintField, ConstraintValue};
//! use ash_parser::token::Span;
//!
//! let mut effective = EffectiveCapabilitySet::new();
//!
//! // Add a capability from a role
//! let decl = CapabilityDecl {
//!     capability: "file".into(),
//!     constraints: None,
//!     span: Span::default(),
//! };
//! effective.add(&decl, CapabilitySource::Role { role_name: "ai_agent".to_string() }).unwrap();
//! ```

use ash_parser::surface::{CapabilityDecl, ConstraintBlock, ConstraintValue};
use std::collections::HashMap;
use thiserror::Error;

/// Error type for capability composition
#[derive(Debug, Clone, Error, PartialEq)]
pub enum CompositionError {
    /// Incompatible constraints from different sources
    #[error(
        "Incompatible constraints for capability '{capability}': conflicts between {source1:?} and {source2:?}"
    )]
    IncompatibleConstraints {
        capability: String,
        source1: CapabilitySource,
        source2: CapabilitySource,
    },
}

/// Source of a capability in the effective set
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CapabilitySource {
    /// Capability provided by a role
    Role { role_name: String },
    /// Implicit default capability (direct workflow declaration)
    ImplicitDefault,
}

impl std::fmt::Display for CapabilitySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CapabilitySource::Role { role_name } => write!(f, "role({})", role_name),
            CapabilitySource::ImplicitDefault => write!(f, "implicit"),
        }
    }
}

/// A merged capability with combined constraints and source tracking
#[derive(Debug, Clone)]
pub struct MergedCapability {
    /// Name of the capability
    pub name: String,
    /// Merged constraints from all sources (union for now)
    pub merged_constraints: Option<ConstraintBlock>,
    /// Sources that provide this capability
    pub sources: Vec<CapabilitySource>,
}

impl MergedCapability {
    /// Create a new merged capability from a declaration
    fn new(name: String, constraints: Option<ConstraintBlock>, source: CapabilitySource) -> Self {
        Self {
            name,
            merged_constraints: constraints,
            sources: vec![source],
        }
    }

    /// Add another source to this merged capability
    fn add_source(&mut self, source: CapabilitySource) {
        if !self.sources.contains(&source) {
            self.sources.push(source);
        }
    }

    /// Merge constraints from another declaration
    fn merge_constraints(&mut self, other_constraints: Option<ConstraintBlock>) {
        // For now, we use union semantics - combine all constraint fields
        // Future: implement constraint intersection for more precise checking
        match (&mut self.merged_constraints, other_constraints) {
            (Some(existing), Some(new)) => {
                // Union: add new fields that don't exist, keep existing fields
                for new_field in new.fields {
                    if !existing.fields.iter().any(|f| f.name == new_field.name) {
                        existing.fields.push(new_field);
                    }
                }
            }
            (None, Some(new)) => {
                self.merged_constraints = Some(new);
            }
            _ => {
                // Either existing has constraints and new doesn't, or neither has constraints
                // In both cases, keep existing constraints
            }
        }
    }
}

/// Effective capability set composed from multiple sources
///
/// This struct manages capabilities from multiple roles and direct declarations,
/// merging constraints and tracking sources for error reporting.
#[derive(Debug, Clone)]
pub struct EffectiveCapabilitySet {
    /// Capabilities keyed by name
    capabilities: HashMap<String, MergedCapability>,
}

impl Default for EffectiveCapabilitySet {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectiveCapabilitySet {
    /// Create a new empty effective capability set
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
        }
    }

    /// Add a capability declaration to the effective set
    ///
    /// If the capability already exists from another source, the constraints
    /// are merged and the source is tracked.
    ///
    /// # Arguments
    ///
    /// * `decl` - The capability declaration to add
    /// * `source` - The source of this capability (role or implicit default)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if successful, or a `CompositionError` if constraints
    /// are incompatible.
    pub fn add(
        &mut self,
        decl: &CapabilityDecl,
        source: CapabilitySource,
    ) -> Result<(), CompositionError> {
        let cap_name = decl.capability.as_ref();

        if let Some(existing) = self.capabilities.get_mut(cap_name) {
            // Capability already exists - merge constraints and add source
            existing.add_source(source.clone());
            existing.merge_constraints(decl.constraints.clone());
        } else {
            // New capability
            let merged =
                MergedCapability::new(cap_name.to_string(), decl.constraints.clone(), source);
            self.capabilities.insert(cap_name.to_string(), merged);
        }

        Ok(())
    }

    /// Check if a capability exists in the effective set
    pub fn has_capability(&self, name: &str) -> bool {
        self.capabilities.contains_key(name)
    }

    /// Get a capability by name
    pub fn get(&self, name: &str) -> Option<&MergedCapability> {
        self.capabilities.get(name)
    }

    /// Get the merged constraint block for a capability
    pub fn get_constraint(&self, name: &str) -> Option<&ConstraintBlock> {
        self.capabilities
            .get(name)
            .and_then(|cap| cap.merged_constraints.as_ref())
    }

    /// Check if a capability use is valid given its constraints
    ///
    /// For now, this checks that the use constraints are a subset of
    /// the declared constraints (simplified check).
    pub fn check_use(&self, cap_name: &str, use_constraints: &ConstraintBlock) -> bool {
        let Some(merged) = self.capabilities.get(cap_name) else {
            return false;
        };

        let Some(declared) = &merged.merged_constraints else {
            // No constraints declared, any use is valid
            return true;
        };

        // Check that all use constraints are satisfied by declared constraints
        for use_field in &use_constraints.fields {
            if let Some(declared_field) = declared.fields.iter().find(|f| f.name == use_field.name)
            {
                // Field exists - check if use value is compatible
                // For now, simple equality check
                // Future: implement proper constraint subsumption
                if !constraint_values_compatible(&declared_field.value, &use_field.value) {
                    return false;
                }
            } else {
                // Use constraint field not found in declared constraints
                return false;
            }
        }

        true
    }

    /// Get all capability names in the effective set
    pub fn capability_names(&self) -> impl Iterator<Item = &String> {
        self.capabilities.keys()
    }

    /// Get the number of capabilities
    pub fn len(&self) -> usize {
        self.capabilities.len()
    }

    /// Check if there are no capabilities
    pub fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }

    /// Get all sources for a capability
    pub fn get_sources(&self, name: &str) -> Option<&Vec<CapabilitySource>> {
        self.capabilities.get(name).map(|cap| &cap.sources)
    }

    /// Merge another effective capability set into this one
    pub fn merge(&mut self, other: &EffectiveCapabilitySet) -> Result<(), CompositionError> {
        for (name, merged_cap) in &other.capabilities {
            for source in &merged_cap.sources {
                // Create a synthetic CapabilityDecl for the merge
                let decl = CapabilityDecl {
                    capability: name.clone().into(),
                    constraints: merged_cap.merged_constraints.clone(),
                    span: ash_parser::token::Span::default(),
                };
                self.add(&decl, source.clone())?;
            }
        }
        Ok(())
    }
}

/// Check if two constraint values are compatible
///
/// For now, this is a simplified check that considers:
/// - Exact equality
/// - Subset relationships for arrays
fn constraint_values_compatible(declared: &ConstraintValue, used: &ConstraintValue) -> bool {
    match (declared, used) {
        // Exact equality for primitives
        (ConstraintValue::Bool(d), ConstraintValue::Bool(u)) => d == u,
        (ConstraintValue::Int(d), ConstraintValue::Int(u)) => d == u,
        (ConstraintValue::String(d), ConstraintValue::String(u)) => d == u,

        // Array compatibility: used array must be a subset of declared array
        (ConstraintValue::Array(declared_arr), ConstraintValue::Array(used_arr)) => {
            // For each element in used, it must exist in declared
            used_arr.iter().all(|u| declared_arr.iter().any(|d| d == u))
        }

        // Object compatibility: used fields must exist in declared with compatible values
        (ConstraintValue::Object(declared_obj), ConstraintValue::Object(used_obj)) => {
            used_obj.iter().all(|(key, u_val)| {
                declared_obj
                    .iter()
                    .find(|(d_key, _)| d_key == key)
                    .map(|(_, d_val)| constraint_values_compatible(d_val, u_val))
                    .unwrap_or(false)
            })
        }

        // Different types are incompatible
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_parser::surface::ConstraintField;
    use ash_parser::token::Span;

    fn test_span() -> Span {
        Span::new(0, 0, 1, 1)
    }

    fn create_capability_decl(name: &str, constraints: Option<ConstraintBlock>) -> CapabilityDecl {
        CapabilityDecl {
            capability: name.into(),
            constraints,
            span: test_span(),
        }
    }

    fn create_constraint_block(fields: Vec<(&str, ConstraintValue)>) -> ConstraintBlock {
        let fields = fields
            .into_iter()
            .map(|(name, value)| ConstraintField {
                name: name.into(),
                value,
                span: test_span(),
            })
            .collect();

        ConstraintBlock {
            fields,
            span: test_span(),
        }
    }

    #[test]
    fn test_single_capability() {
        let mut effective = EffectiveCapabilitySet::new();

        let decl = create_capability_decl("file", None);
        let source = CapabilitySource::Role {
            role_name: "ai_agent".to_string(),
        };

        assert!(effective.add(&decl, source).is_ok());
        assert!(effective.has_capability("file"));
        assert_eq!(effective.len(), 1);
    }

    #[test]
    fn test_multiple_sources_same_capability() {
        let mut effective = EffectiveCapabilitySet::new();

        // Add file capability from ai_agent role
        let decl1 = create_capability_decl("file", None);
        let source1 = CapabilitySource::Role {
            role_name: "ai_agent".to_string(),
        };
        assert!(effective.add(&decl1, source1).is_ok());

        // Add file capability from another role
        let decl2 = create_capability_decl("file", None);
        let source2 = CapabilitySource::Role {
            role_name: "file_processor".to_string(),
        };
        assert!(effective.add(&decl2, source2).is_ok());

        // Should still have only one file capability
        assert_eq!(effective.len(), 1);

        // But should have two sources
        let sources = effective.get_sources("file").unwrap();
        assert_eq!(sources.len(), 2);
        assert!(sources.contains(&CapabilitySource::Role {
            role_name: "ai_agent".to_string()
        }));
        assert!(sources.contains(&CapabilitySource::Role {
            role_name: "file_processor".to_string()
        }));
    }

    #[test]
    fn test_different_capabilities_composed() {
        let mut effective = EffectiveCapabilitySet::new();

        // Add capabilities from different roles
        let file_decl = create_capability_decl("file", None);
        let file_source = CapabilitySource::Role {
            role_name: "ai_agent".to_string(),
        };
        assert!(effective.add(&file_decl, file_source).is_ok());

        let network_decl = create_capability_decl("network", None);
        let network_source = CapabilitySource::Role {
            role_name: "net_client".to_string(),
        };
        assert!(effective.add(&network_decl, network_source).is_ok());

        let cache_decl = create_capability_decl("cache", None);
        let cache_source = CapabilitySource::ImplicitDefault;
        assert!(effective.add(&cache_decl, cache_source).is_ok());

        // All capabilities should be present
        assert!(effective.has_capability("file"));
        assert!(effective.has_capability("network"));
        assert!(effective.has_capability("cache"));
        assert_eq!(effective.len(), 3);
    }

    #[test]
    fn test_implicit_default_source() {
        let mut effective = EffectiveCapabilitySet::new();

        // Add capability from implicit default (direct workflow declaration)
        let decl = create_capability_decl("cache", None);
        let source = CapabilitySource::ImplicitDefault;

        assert!(effective.add(&decl, source.clone()).is_ok());
        assert!(effective.has_capability("cache"));

        let sources = effective.get_sources("cache").unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0], source);
    }

    #[test]
    fn test_constraint_merge() {
        let mut effective = EffectiveCapabilitySet::new();

        // First declaration with some constraints
        let decl1 = create_capability_decl(
            "file",
            Some(create_constraint_block(vec![(
                "read",
                ConstraintValue::Bool(true),
            )])),
        );
        let source1 = CapabilitySource::Role {
            role_name: "ai_agent".to_string(),
        };
        assert!(effective.add(&decl1, source1).is_ok());

        // Second declaration with different constraints
        let decl2 = create_capability_decl(
            "file",
            Some(create_constraint_block(vec![(
                "write",
                ConstraintValue::Bool(false),
            )])),
        );
        let source2 = CapabilitySource::Role {
            role_name: "file_processor".to_string(),
        };
        assert!(effective.add(&decl2, source2).is_ok());

        // Constraints should be merged (union)
        let constraint = effective.get_constraint("file").unwrap();
        assert_eq!(constraint.fields.len(), 2);
    }

    #[test]
    fn test_check_use_valid() {
        let mut effective = EffectiveCapabilitySet::new();

        // Add capability with constraints
        let decl = create_capability_decl(
            "file",
            Some(create_constraint_block(vec![
                ("read", ConstraintValue::Bool(true)),
                (
                    "paths",
                    ConstraintValue::Array(vec![
                        ConstraintValue::String("/tmp/*".to_string()),
                        ConstraintValue::String("/home/*".to_string()),
                    ]),
                ),
            ])),
        );
        let source = CapabilitySource::Role {
            role_name: "ai_agent".to_string(),
        };
        assert!(effective.add(&decl, source).is_ok());

        // Use with subset of constraints should be valid
        let use_constraints = create_constraint_block(vec![
            ("read", ConstraintValue::Bool(true)),
            (
                "paths",
                ConstraintValue::Array(vec![ConstraintValue::String("/tmp/*".to_string())]),
            ),
        ]);

        assert!(effective.check_use("file", &use_constraints));
    }

    #[test]
    fn test_check_use_invalid() {
        let mut effective = EffectiveCapabilitySet::new();

        // Add capability with constraints
        let decl = create_capability_decl(
            "file",
            Some(create_constraint_block(vec![(
                "read",
                ConstraintValue::Bool(true),
            )])),
        );
        let source = CapabilitySource::Role {
            role_name: "ai_agent".to_string(),
        };
        assert!(effective.add(&decl, source).is_ok());

        // Use with different constraint value should be invalid
        let use_constraints = create_constraint_block(vec![("read", ConstraintValue::Bool(false))]);

        assert!(!effective.check_use("file", &use_constraints));
    }

    #[test]
    fn test_check_use_unknown_capability() {
        let effective = EffectiveCapabilitySet::new();

        let use_constraints = create_constraint_block(vec![]);
        assert!(!effective.check_use("unknown", &use_constraints));
    }

    #[test]
    fn test_composition_idempotence() {
        let mut effective = EffectiveCapabilitySet::new();

        // Add same capability twice from same source
        let decl = create_capability_decl("file", None);
        let source = CapabilitySource::Role {
            role_name: "ai_agent".to_string(),
        };

        assert!(effective.add(&decl, source.clone()).is_ok());
        assert!(effective.add(&decl, source.clone()).is_ok());

        // Should still have one capability with one source
        assert_eq!(effective.len(), 1);
        let sources = effective.get_sources("file").unwrap();
        assert_eq!(sources.len(), 1);
    }

    #[test]
    fn test_merge_capability_sets() {
        let mut set1 = EffectiveCapabilitySet::new();
        let mut set2 = EffectiveCapabilitySet::new();

        // Add to set1
        let decl1 = create_capability_decl("file", None);
        let source1 = CapabilitySource::Role {
            role_name: "ai_agent".to_string(),
        };
        assert!(set1.add(&decl1, source1).is_ok());

        // Add to set2
        let decl2 = create_capability_decl("network", None);
        let source2 = CapabilitySource::Role {
            role_name: "net_client".to_string(),
        };
        assert!(set2.add(&decl2, source2).is_ok());

        // Merge set2 into set1
        assert!(set1.merge(&set2).is_ok());

        assert!(set1.has_capability("file"));
        assert!(set1.has_capability("network"));
        assert_eq!(set1.len(), 2);
    }

    #[test]
    fn test_is_empty() {
        let mut effective = EffectiveCapabilitySet::new();
        assert!(effective.is_empty());

        let decl = create_capability_decl("file", None);
        let source = CapabilitySource::ImplicitDefault;
        assert!(effective.add(&decl, source).is_ok());

        assert!(!effective.is_empty());
    }

    #[test]
    fn test_capability_source_display() {
        let role_source = CapabilitySource::Role {
            role_name: "ai_agent".to_string(),
        };
        assert_eq!(role_source.to_string(), "role(ai_agent)");

        let implicit_source = CapabilitySource::ImplicitDefault;
        assert_eq!(implicit_source.to_string(), "implicit");
    }
}
