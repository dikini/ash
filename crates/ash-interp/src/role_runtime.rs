//! Runtime role resolution for capability grants
//!
//! This module provides runtime resolution of roles to their capability grants
//! per SPEC-019 and SPEC-024. It maintains a registry of roles and can resolve
//! a workflow's `plays_roles` declarations to effective capability grants.
//!
//! # Example
//!
//! ```
//! use ash_interp::role_runtime::{RoleRegistry, RuntimeCapabilitySet};
//! use ash_parser::surface::{RoleDef, WorkflowDef, RoleRef, CapabilityDecl};
//!
//! let mut registry = RoleRegistry::new();
//!
//! // Register a role with capabilities
//! let role = RoleDef {
//!     name: "ai_agent".into(),
//!     capabilities: vec![
//!         CapabilityDecl { capability: "file".into(), constraints: None, span: ash_parser::token::Span::default() },
//!         CapabilityDecl { capability: "network".into(), constraints: None, span: ash_parser::token::Span::default() },
//!     ],
//!     obligations: vec![],
//!     span: ash_parser::token::Span::default(),
//! };
//! registry.register(role);
//!
//! // Create a workflow that plays the role
//! let workflow = WorkflowDef {
//!     name: "test".into(),
//!     params: vec![],
//!     plays_roles: vec![RoleRef { name: "ai_agent".into(), span: ash_parser::token::Span::default() }],
//!     capabilities: vec![],
//!     body: ash_parser::surface::Workflow::Done { span: ash_parser::token::Span::default() },
//!     contract: None,
//!     span: ash_parser::token::Span::default(),
//! };
//!
//! // Resolve roles to capabilities
//! let caps = registry.resolve_workflow_roles(&workflow).unwrap();
//! assert!(caps.has_capability("file"));
//! assert!(caps.has_capability("network"));
//! ```

use ash_core::Value;
use ash_parser::surface::{CapabilityDecl, ConstraintBlock, RoleDef, WorkflowDef};
use std::collections::HashMap;
use std::fmt;

/// Errors that can occur during role resolution
#[derive(Debug, Clone, PartialEq)]
pub enum RoleError {
    /// Unknown role referenced in workflow
    UnknownRole { name: String },
    /// Incompatible capability grants from multiple roles
    IncompatibleGrants { capability: String },
}

impl fmt::Display for RoleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RoleError::UnknownRole { name } => write!(f, "unknown role: {}", name),
            RoleError::IncompatibleGrants { capability } => {
                write!(f, "incompatible grants for capability: {}", capability)
            }
        }
    }
}

impl std::error::Error for RoleError {}

/// Errors that can occur during capability use checking
#[derive(Debug, Clone, PartialEq)]
pub enum CapabilityError {
    /// Capability not granted
    NotGranted,
    /// Constraint violation
    ConstraintViolation { reason: String },
}

impl fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CapabilityError::NotGranted => write!(f, "capability not granted"),
            CapabilityError::ConstraintViolation { reason } => {
                write!(f, "constraint violation: {}", reason)
            }
        }
    }
}

impl std::error::Error for CapabilityError {}

/// Individual capability grant with constraints and provenance
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityGrant {
    /// The capability name
    pub capability: String,
    /// Optional constraints on the capability
    pub constraints: Option<ConstraintBlock>,
    /// Role names that granted this capability
    pub granted_by: Vec<String>,
}

impl CapabilityGrant {
    /// Create a new capability grant
    pub fn new(capability: String) -> Self {
        Self {
            capability,
            constraints: None,
            granted_by: vec![],
        }
    }

    /// Add a granting role to this capability
    pub fn add_granting_role(&mut self, role: String) {
        if !self.granted_by.contains(&role) {
            self.granted_by.push(role);
        }
    }

    /// Merge another capability declaration into this grant
    pub fn merge(&mut self, decl: &CapabilityDecl) -> Result<(), RoleError> {
        // For now, we simply update constraints if the new decl has them
        // In a more complex implementation, this would check for conflicts
        if decl.constraints.is_some() {
            // If we already have constraints, we might need to merge them
            // For simplicity, we replace with the new constraints
            self.constraints = decl.constraints.clone();
        }
        Ok(())
    }
}

/// Runtime capability grants for a workflow
///
/// Tracks all capabilities granted to a workflow through its played roles.
#[derive(Debug, Clone, Default)]
pub struct RuntimeCapabilitySet {
    /// Map from capability name to grant details
    grants: HashMap<String, CapabilityGrant>,
}

impl RuntimeCapabilitySet {
    /// Create a new empty capability set
    pub fn new() -> Self {
        Self {
            grants: HashMap::new(),
        }
    }

    /// Grant a capability from a declaration
    ///
    /// If the capability is already granted, this will attempt to merge
    /// the new declaration with the existing grant.
    pub fn grant(&mut self, decl: &CapabilityDecl) -> Result<(), RoleError> {
        self.grant_with_role(decl, None)
    }

    /// Grant a capability from a declaration with an optional granting role
    ///
    /// If the capability is already granted, this will attempt to merge
    /// the new declaration with the existing grant and add the role to granted_by.
    pub fn grant_with_role(
        &mut self,
        decl: &CapabilityDecl,
        role_name: Option<&str>,
    ) -> Result<(), RoleError> {
        let name = decl.capability.to_string();

        if let Some(existing) = self.grants.get_mut(&name) {
            // Merge grants
            existing.merge(decl)?;
            if let Some(role) = role_name {
                existing.add_granting_role(role.to_string());
            }
        } else {
            let mut grant = CapabilityGrant {
                capability: name.clone(),
                constraints: decl.constraints.clone(),
                granted_by: vec![],
            };
            if let Some(role) = role_name {
                grant.add_granting_role(role.to_string());
            }
            self.grants.insert(name, grant);
        }

        Ok(())
    }

    /// Grant a capability by name (for simple role authority resolution)
    ///
    /// This is used when resolving role authority (`Vec<Name>`) to capabilities.
    pub fn grant_by_name(&mut self, capability_name: &str, role_name: &str) {
        let name = capability_name.to_string();

        if let Some(existing) = self.grants.get_mut(&name) {
            // Just add the granting role
            existing.add_granting_role(role_name.to_string());
        } else {
            let mut grant = CapabilityGrant::new(name.clone());
            grant.add_granting_role(role_name.to_string());
            self.grants.insert(name, grant);
        }
    }

    /// Check if a capability is granted
    pub fn has_capability(&self, name: &str) -> bool {
        self.grants.contains_key(name)
    }

    /// Get a specific capability grant
    pub fn get_grant(&self, name: &str) -> Option<&CapabilityGrant> {
        self.grants.get(name)
    }

    /// Get all granted capability names
    pub fn granted_capabilities(&self) -> Vec<&String> {
        self.grants.keys().collect()
    }

    /// Check if a capability use is permitted
    ///
    /// This checks:
    /// 1. That the capability is granted
    /// 2. That the operation satisfies any constraints
    ///
    /// # Arguments
    /// * `capability` - The capability name
    /// * `operation` - The operation being performed (e.g., "read", "write")
    /// * `args` - Arguments to the operation for constraint checking
    pub fn check_use(
        &self,
        capability: &str,
        operation: &str,
        args: &Value,
    ) -> Result<(), CapabilityError> {
        let grant = self
            .grants
            .get(capability)
            .ok_or(CapabilityError::NotGranted)?;

        if let Some(constraints) = &grant.constraints {
            // Check operation against constraints
            self.check_constraints(operation, args, constraints)?;
        }

        // Also check any workflow-level constraints that might apply
        // This is a placeholder for more sophisticated constraint checking

        Ok(())
    }

    /// Check constraints against an operation
    fn check_constraints(
        &self,
        operation: &str,
        args: &Value,
        constraints: &ConstraintBlock,
    ) -> Result<(), CapabilityError> {
        use crate::constraint_enforcement::ConstraintEnforcer;

        ConstraintEnforcer::check(operation, args, constraints).map_err(|e| {
            CapabilityError::ConstraintViolation {
                reason: e.to_string(),
            }
        })
    }

    /// Get the number of granted capabilities
    pub fn len(&self) -> usize {
        self.grants.len()
    }

    /// Check if no capabilities are granted
    pub fn is_empty(&self) -> bool {
        self.grants.is_empty()
    }
}

/// Runtime role registry
///
/// Stores role definitions and provides resolution of workflow roles
/// to their effective capability grants.
#[derive(Debug, Clone, Default)]
pub struct RoleRegistry {
    /// Map from role name to role definition
    roles: HashMap<String, RoleDef>,
}

impl RoleRegistry {
    /// Create a new empty role registry
    pub fn new() -> Self {
        Self {
            roles: HashMap::new(),
        }
    }

    /// Register a role definition
    ///
    /// The role will be stored by its name for later resolution.
    pub fn register(&mut self, role: RoleDef) {
        self.roles.insert(role.name.to_string(), role);
    }

    /// Resolve workflow roles to capability grants
    ///
    /// Takes a workflow definition and resolves its `plays_roles` declarations
    /// to a set of effective capability grants.
    ///
    /// # Arguments
    /// * `workflow` - The workflow definition to resolve roles for
    ///
    /// # Returns
    /// * `Ok(RuntimeCapabilitySet)` - The resolved capabilities
    /// * `Err(RoleError)` - If an unknown role is referenced
    pub fn resolve_workflow_roles(
        &self,
        workflow: &WorkflowDef,
    ) -> Result<RuntimeCapabilitySet, RoleError> {
        let mut caps = RuntimeCapabilitySet::new();

        for role_ref in &workflow.plays_roles {
            let role_name = role_ref.name.to_string();
            let role = self
                .roles
                .get(&role_name)
                .ok_or_else(|| RoleError::UnknownRole {
                    name: role_name.clone(),
                })?;

            // Grant capabilities from the role's capabilities
            for cap_decl in &role.capabilities {
                caps.grant_with_role(cap_decl, Some(&role_name))?;
            }
        }

        // Also include any explicit capability declarations from the workflow
        for cap_decl in &workflow.capabilities {
            caps.grant(cap_decl)?;
        }

        Ok(caps)
    }

    /// Get a role definition by name
    pub fn get_role(&self, name: &str) -> Option<&RoleDef> {
        self.roles.get(name)
    }

    /// Check if a role is registered
    pub fn is_registered(&self, name: &str) -> bool {
        self.roles.contains_key(name)
    }

    /// Get the number of registered roles
    pub fn len(&self) -> usize {
        self.roles.len()
    }

    /// Check if no roles are registered
    pub fn is_empty(&self) -> bool {
        self.roles.is_empty()
    }

    /// Unregister a role
    pub fn unregister(&mut self, name: &str) -> Option<RoleDef> {
        self.roles.remove(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_parser::surface::{ConstraintBlock, ConstraintField, ConstraintValue, RoleRef};
    use ash_parser::token::Span;

    fn test_span() -> Span {
        Span::default()
    }

    fn create_test_role(name: &str, capabilities: Vec<&str>) -> RoleDef {
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

    fn create_test_workflow(name: &str, plays_roles: Vec<&str>) -> WorkflowDef {
        WorkflowDef {
            name: name.into(),
            params: vec![],
            plays_roles: plays_roles
                .into_iter()
                .map(|r| RoleRef {
                    name: r.into(),
                    span: test_span(),
                })
                .collect(),
            capabilities: vec![],
            body: ash_parser::surface::Workflow::Done { span: test_span() },
            contract: None,
            span: test_span(),
        }
    }

    #[test]
    fn test_role_registry_new() {
        let registry = RoleRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_and_get_role() {
        let mut registry = RoleRegistry::new();
        let role = create_test_role("admin", vec!["file", "network"]);

        registry.register(role.clone());

        assert_eq!(registry.len(), 1);
        assert!(registry.is_registered("admin"));

        let retrieved = registry.get_role("admin").unwrap();
        assert_eq!(retrieved.name, "admin".into());
        assert_eq!(retrieved.capabilities.len(), 2);
    }

    #[test]
    fn test_resolve_single_role() {
        let mut registry = RoleRegistry::new();
        registry.register(create_test_role("ai_agent", vec!["file", "process"]));

        let workflow = create_test_workflow("test", vec!["ai_agent"]);
        let caps = registry.resolve_workflow_roles(&workflow).unwrap();

        assert!(caps.has_capability("file"));
        assert!(caps.has_capability("process"));
        assert!(!caps.has_capability("network"));
    }

    #[test]
    fn test_resolve_unknown_role() {
        let registry = RoleRegistry::new();
        let workflow = create_test_workflow("test", vec!["nonexistent"]);

        let result = registry.resolve_workflow_roles(&workflow);
        assert!(result.is_err());

        match result {
            Err(RoleError::UnknownRole { name }) => assert_eq!(name, "nonexistent"),
            _ => panic!("Expected UnknownRole error"),
        }
    }

    #[test]
    fn test_resolve_multiple_roles() {
        let mut registry = RoleRegistry::new();
        registry.register(create_test_role("file_user", vec!["file"]));
        registry.register(create_test_role("net_user", vec!["network"]));

        let workflow = create_test_workflow("test", vec!["file_user", "net_user"]);
        let caps = registry.resolve_workflow_roles(&workflow).unwrap();

        assert!(caps.has_capability("file"));
        assert!(caps.has_capability("network"));
        assert_eq!(caps.len(), 2);
    }

    #[test]
    fn test_capability_use_check_granted() {
        let mut registry = RoleRegistry::new();
        registry.register(create_test_role("file_user", vec!["file"]));

        let workflow = create_test_workflow("test", vec!["file_user"]);
        let caps = registry.resolve_workflow_roles(&workflow).unwrap();

        // Should succeed for granted capability
        assert!(caps.check_use("file", "read", &Value::Null).is_ok());
    }

    #[test]
    fn test_capability_use_check_not_granted() {
        let mut registry = RoleRegistry::new();
        registry.register(create_test_role("file_user", vec!["file"]));

        let workflow = create_test_workflow("test", vec!["file_user"]);
        let caps = registry.resolve_workflow_roles(&workflow).unwrap();

        // Should fail for ungranted capability
        let result = caps.check_use("network", "get", &Value::Null);
        assert!(result.is_err());

        match result {
            Err(CapabilityError::NotGranted) => (),
            _ => panic!("Expected NotGranted error"),
        }
    }

    #[test]
    fn test_capability_grant_by_name_tracks_role() {
        let mut caps = RuntimeCapabilitySet::new();
        caps.grant_by_name("file", "admin");
        caps.grant_by_name("file", "user");

        let grant = caps.get_grant("file").unwrap();
        assert_eq!(grant.granted_by.len(), 2);
        assert!(grant.granted_by.contains(&"admin".to_string()));
        assert!(grant.granted_by.contains(&"user".to_string()));
    }

    #[test]
    fn test_grant_with_constraints() {
        let mut caps = RuntimeCapabilitySet::new();

        let constraint_block = ConstraintBlock {
            fields: vec![ConstraintField {
                name: "paths".into(),
                value: ConstraintValue::Array(vec![ConstraintValue::String("/tmp/*".to_string())]),
                span: test_span(),
            }],
            span: test_span(),
        };

        let decl = CapabilityDecl {
            capability: "file".into(),
            constraints: Some(constraint_block.clone()),
            span: test_span(),
        };

        caps.grant(&decl).unwrap();

        let grant = caps.get_grant("file").unwrap();
        assert!(grant.constraints.is_some());
    }

    #[test]
    fn test_workflow_with_explicit_capabilities() {
        let mut registry = RoleRegistry::new();
        registry.register(create_test_role("base", vec!["file"]));

        let mut workflow = create_test_workflow("test", vec!["base"]);
        workflow.capabilities.push(CapabilityDecl {
            capability: "network".into(),
            constraints: None,
            span: test_span(),
        });

        let caps = registry.resolve_workflow_roles(&workflow).unwrap();

        assert!(caps.has_capability("file")); // From role
        assert!(caps.has_capability("network")); // From explicit declaration
    }

    #[test]
    fn test_error_display() {
        let err = RoleError::UnknownRole {
            name: "test".to_string(),
        };
        assert_eq!(err.to_string(), "unknown role: test");

        let err = RoleError::IncompatibleGrants {
            capability: "file".to_string(),
        };
        assert_eq!(err.to_string(), "incompatible grants for capability: file");

        let err = CapabilityError::NotGranted;
        assert_eq!(err.to_string(), "capability not granted");

        let err = CapabilityError::ConstraintViolation {
            reason: "path not allowed".to_string(),
        };
        assert_eq!(err.to_string(), "constraint violation: path not allowed");
    }
}
