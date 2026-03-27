//! Role inclusion type checking for Ash workflows (TASK-262)
//!
//! This module provides type checking for `plays role(R)` clauses in workflows,
//! per SPEC-019 and SPEC-024. It verifies that:
//! 1. Each role referenced in `plays role(R)` exists
//! 2. Capabilities from all included roles are composed
//! 3. Capability conflicts are detected
//!
//! # Example
//!
//! ```
//! use std::collections::HashMap;
//! use ash_typeck::role_checking::{RoleChecker, EffectiveCapabilities};
//! use ash_parser::surface::{RoleDef, CapabilityDecl};
//! use ash_parser::token::Span;
//!
//! // Create role definitions
//! let mut role_defs = HashMap::new();
//! role_defs.insert("ai_agent".to_string(), RoleDef {
//!     name: "ai_agent".into(),
//!     capabilities: vec![
//!         CapabilityDecl { capability: "network".into(), constraints: None, span: Span::default() },
//!         CapabilityDecl { capability: "file".into(), constraints: None, span: Span::default() },
//!     ],
//!     obligations: vec![],
//!     span: Span::default(),
//! });
//!
//! let checker = RoleChecker::new(&role_defs);
//! ```

use ash_parser::surface::{CapabilityDecl, RoleDef, WorkflowDef};
use ash_parser::token::Span;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Error type for role checking
#[derive(Debug, Clone, Error, PartialEq)]
pub enum RoleCheckError {
    /// Role doesn't exist
    #[error("Unknown role: '{name}' does not exist")]
    UnknownRole { name: String, span: Span },
    /// Capability conflict between roles
    #[error("Capability conflict: '{capability}' declared by multiple roles")]
    CapabilityConflict {
        capability: String,
        roles: Vec<String>,
    },
}

/// Result type for role checking
pub type RoleCheckResult<T> = Result<T, RoleCheckError>;

/// Effective capabilities composed from all included roles
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EffectiveCapabilities {
    /// Capabilities from all roles, keyed by capability name
    caps: HashMap<String, CapabilityDecl>,
    /// Roles that contributed to these capabilities
    source_roles: HashMap<String, Vec<String>>,
}

impl EffectiveCapabilities {
    /// Create empty effective capabilities
    pub fn new() -> Self {
        Self {
            caps: HashMap::new(),
            source_roles: HashMap::new(),
        }
    }

    /// Get a capability by name
    pub fn get(&self, name: &str) -> Option<&CapabilityDecl> {
        self.caps.get(name)
    }

    /// Check if a capability exists
    pub fn has(&self, name: &str) -> bool {
        self.caps.contains_key(name)
    }

    /// Get all capability names
    pub fn capability_names(&self) -> impl Iterator<Item = &String> {
        self.caps.keys()
    }

    /// Get the number of capabilities
    pub fn len(&self) -> usize {
        self.caps.len()
    }

    /// Check if there are no capabilities
    pub fn is_empty(&self) -> bool {
        self.caps.is_empty()
    }

    /// Get roles that contributed a capability
    pub fn source_roles(&self, capability: &str) -> Option<&Vec<String>> {
        self.source_roles.get(capability)
    }

    /// Add a capability from a role
    fn add_capability(&mut self, role_name: &str, capability: CapabilityDecl) {
        let cap_name = capability.capability.to_string();
        self.caps.insert(cap_name.clone(), capability);
        self.source_roles
            .entry(cap_name)
            .or_default()
            .push(role_name.to_string());
    }

    /// Merge capabilities from another effective set
    fn merge(&mut self, other: EffectiveCapabilities) -> Result<(), RoleCheckError> {
        for (cap_name, cap) in other.caps {
            if let Some(_existing) = self.caps.get(&cap_name) {
                // Check for conflicts - for now we just detect duplicate names
                // Future: check for constraint compatibility
                let roles = self
                    .source_roles
                    .get(&cap_name)
                    .cloned()
                    .unwrap_or_default();
                return Err(RoleCheckError::CapabilityConflict {
                    capability: cap_name,
                    roles,
                });
            }
            self.caps.insert(cap_name.clone(), cap);
        }
        Ok(())
    }
}

/// Checker for workflow role inclusions
#[derive(Debug, Clone)]
pub struct RoleChecker<'a> {
    /// Map of role names to their definitions
    role_defs: &'a HashMap<String, RoleDef>,
}

impl<'a> RoleChecker<'a> {
    /// Create a new role checker with the given role definitions
    pub fn new(role_defs: &'a HashMap<String, RoleDef>) -> Self {
        Self { role_defs }
    }

    /// Check workflow role inclusions and return effective capabilities
    pub fn check_workflow_roles(
        &self,
        workflow: &WorkflowDef,
    ) -> RoleCheckResult<EffectiveCapabilities> {
        let mut effective = EffectiveCapabilities::new();
        let mut seen_roles: HashSet<String> = HashSet::new();

        for role_ref in &workflow.plays_roles {
            let role_name = role_ref.name.as_ref();

            // Check for duplicate role references
            if !seen_roles.insert(role_name.to_string()) {
                // Duplicate role - still valid, just skip
                continue;
            }

            // Look up the role definition
            let role_def =
                self.lookup_role(role_name)
                    .ok_or_else(|| RoleCheckError::UnknownRole {
                        name: role_name.to_string(),
                        span: role_ref.span,
                    })?;

            // Compose capabilities from this role
            let role_caps = self.compose_role_capabilities(role_def);
            effective.merge(role_caps)?;
        }

        Ok(effective)
    }

    /// Look up a role by name
    fn lookup_role(&self, name: &str) -> Option<&RoleDef> {
        self.role_defs.get(name)
    }

    /// Compose capabilities from a single role
    fn compose_role_capabilities(&self, role_def: &RoleDef) -> EffectiveCapabilities {
        let mut effective = EffectiveCapabilities::new();
        let role_name = role_def.name.as_ref();

        // Use existing CapabilityDecl with constraints from role
        for cap_decl in &role_def.capabilities {
            effective.add_capability(role_name, cap_decl.clone());
        }

        effective
    }

    /// Check if a role exists
    pub fn has_role(&self, name: &str) -> bool {
        self.role_defs.contains_key(name)
    }

    /// Get all available role names
    pub fn available_roles(&self) -> impl Iterator<Item = &String> {
        self.role_defs.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_parser::surface::RoleRef;

    fn test_span() -> Span {
        Span::new(0, 0, 1, 1)
    }

    fn create_role_def(name: &str, capabilities: Vec<&str>) -> RoleDef {
        RoleDef {
            name: name.into(),
            capabilities: capabilities
                .into_iter()
                .map(|cap| CapabilityDecl {
                    capability: cap.into(),
                    constraints: None,
                    span: test_span(),
                })
                .collect(),
            obligations: vec![],
            span: test_span(),
        }
    }

    fn create_workflow_def_with_roles(role_names: Vec<&str>) -> WorkflowDef {
        let plays_roles: Vec<RoleRef> = role_names
            .into_iter()
            .map(|name| RoleRef {
                name: name.into(),
                span: test_span(),
            })
            .collect();

        WorkflowDef {
            name: "test_workflow".into(),
            params: vec![],
            plays_roles,
            capabilities: vec![],
            body: ash_parser::surface::Workflow::Done { span: test_span() },
            contract: None,
            span: test_span(),
        }
    }

    #[test]
    fn test_valid_role_inclusion() {
        let mut role_defs = HashMap::new();
        role_defs.insert(
            "ai_agent".to_string(),
            create_role_def("ai_agent", vec!["network", "file"]),
        );

        let checker = RoleChecker::new(&role_defs);
        let workflow = create_workflow_def_with_roles(vec!["ai_agent"]);

        let result = checker.check_workflow_roles(&workflow);
        assert!(result.is_ok());

        let effective = result.unwrap();
        assert!(effective.has("network"));
        assert!(effective.has("file"));
        assert_eq!(effective.len(), 2);
    }

    #[test]
    fn test_unknown_role_error() {
        let role_defs = HashMap::new();
        let checker = RoleChecker::new(&role_defs);
        let workflow = create_workflow_def_with_roles(vec!["unknown_role"]);

        let result = checker.check_workflow_roles(&workflow);
        assert!(result.is_err());

        match result.unwrap_err() {
            RoleCheckError::UnknownRole { name, .. } => {
                assert_eq!(name, "unknown_role");
            }
            _ => panic!("Expected UnknownRole error"),
        }
    }

    #[test]
    fn test_multiple_role_capabilities_composed() {
        let mut role_defs = HashMap::new();
        role_defs.insert(
            "ai_agent".to_string(),
            create_role_def("ai_agent", vec!["network", "file"]),
        );
        role_defs.insert(
            "network_client".to_string(),
            create_role_def("network_client", vec!["http", "websocket"]),
        );

        let checker = RoleChecker::new(&role_defs);
        let workflow = create_workflow_def_with_roles(vec!["ai_agent", "network_client"]);

        let result = checker.check_workflow_roles(&workflow);
        assert!(result.is_ok());

        let effective = result.unwrap();
        assert!(effective.has("network"));
        assert!(effective.has("file"));
        assert!(effective.has("http"));
        assert!(effective.has("websocket"));
        assert_eq!(effective.len(), 4);
    }

    #[test]
    fn test_role_inclusion_commutativity() {
        let mut role_defs = HashMap::new();
        role_defs.insert(
            "role_a".to_string(),
            create_role_def("role_a", vec!["cap_a"]),
        );
        role_defs.insert(
            "role_b".to_string(),
            create_role_def("role_b", vec!["cap_b"]),
        );

        let checker = RoleChecker::new(&role_defs);

        // Order 1: role_a, role_b
        let workflow1 = create_workflow_def_with_roles(vec!["role_a", "role_b"]);
        let result1 = checker.check_workflow_roles(&workflow1).unwrap();

        // Order 2: role_b, role_a
        let workflow2 = create_workflow_def_with_roles(vec!["role_b", "role_a"]);
        let result2 = checker.check_workflow_roles(&workflow2).unwrap();

        // Should have same capabilities regardless of order
        assert_eq!(result1.len(), result2.len());
        assert!(result1.has("cap_a"));
        assert!(result1.has("cap_b"));
        assert!(result2.has("cap_a"));
        assert!(result2.has("cap_b"));
    }

    #[test]
    fn test_empty_role_inclusion() {
        let role_defs = HashMap::new();
        let checker = RoleChecker::new(&role_defs);
        let workflow = create_workflow_def_with_roles(vec![]);

        let result = checker.check_workflow_roles(&workflow);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_duplicate_role_reference() {
        let mut role_defs = HashMap::new();
        role_defs.insert(
            "ai_agent".to_string(),
            create_role_def("ai_agent", vec!["network"]),
        );

        let checker = RoleChecker::new(&role_defs);
        // Duplicate role reference should be handled gracefully
        let workflow = create_workflow_def_with_roles(vec!["ai_agent", "ai_agent"]);

        let result = checker.check_workflow_roles(&workflow);
        assert!(result.is_ok());

        // Should only count capabilities once
        let effective = result.unwrap();
        assert_eq!(effective.len(), 1);
    }

    #[test]
    fn test_lookup_role() {
        let mut role_defs = HashMap::new();
        let role_def = create_role_def("test_role", vec![]);
        role_defs.insert("test_role".to_string(), role_def.clone());

        let checker = RoleChecker::new(&role_defs);

        assert!(checker.lookup_role("test_role").is_some());
        assert!(checker.lookup_role("nonexistent").is_none());
    }

    #[test]
    fn test_has_role() {
        let mut role_defs = HashMap::new();
        role_defs.insert(
            "existing_role".to_string(),
            create_role_def("existing_role", vec![]),
        );

        let checker = RoleChecker::new(&role_defs);

        assert!(checker.has_role("existing_role"));
        assert!(!checker.has_role("nonexistent_role"));
    }

    #[test]
    fn test_available_roles() {
        let mut role_defs = HashMap::new();
        role_defs.insert("role1".to_string(), create_role_def("role1", vec![]));
        role_defs.insert("role2".to_string(), create_role_def("role2", vec![]));

        let checker = RoleChecker::new(&role_defs);
        let roles: Vec<_> = checker.available_roles().collect();

        assert_eq!(roles.len(), 2);
        assert!(roles.contains(&&"role1".to_string()));
        assert!(roles.contains(&&"role2".to_string()));
    }
}
