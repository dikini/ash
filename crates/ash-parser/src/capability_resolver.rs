//! Capability resolution for lowering.
//!
//! This module provides resolution of symbolic capability names to (provider, action) pairs
//! during the lowering phase from surface AST to core IR.

use std::collections::HashMap;

/// Resolved capability target - canonical (provider, action) pair.
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityTarget {
    /// Provider name (e.g., "io", "http", "db")
    pub provider: String,
    /// Action name (e.g., "fs_read", "get", "query")
    pub action: String,
}

impl CapabilityTarget {
    /// Create a new capability target from provider and action names.
    pub fn new(provider: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            action: action.into(),
        }
    }
}

/// Capability resolver that maps symbolic capability names to (provider, action) pairs.
///
/// This is used during lowering to resolve capability names like `fs_read`
/// to their concrete implementations like `io:fs_read`.
#[derive(Debug, Clone, Default)]
pub struct CapabilityResolver {
    /// Maps symbolic capability names to (provider, action) pairs.
    /// For example: "fs_read" -> ("io", "fs_read")
    mappings: HashMap<String, (String, String)>,
}

impl CapabilityResolver {
    /// Create a new empty capability resolver.
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// Register a capability mapping.
    ///
    /// # Arguments
    /// * `capability_name` - The symbolic name (e.g., "fs_read")
    /// * `provider` - The provider name (e.g., "io")
    /// * `action` - The action name (e.g., "fs_read")
    pub fn register(
        &mut self,
        capability_name: impl Into<String>,
        provider: impl Into<String>,
        action: impl Into<String>,
    ) {
        let name = capability_name.into();
        self.mappings.insert(name, (provider.into(), action.into()));
    }

    /// Resolve a symbolic capability name to a (provider, action) pair.
    ///
    /// Returns `None` if the capability name is not explicitly registered.
    /// Per Phase 70 design, symbolic names MUST resolve through explicit
    /// resolver-owned metadata. No fallback or default is provided.
    pub fn resolve(&self, capability_name: &str) -> Option<(String, String)> {
        self.mappings.get(capability_name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explicit_mapping() {
        let mut resolver = CapabilityResolver::new();
        resolver.register("my_cap", "my_provider", "my_action");

        assert_eq!(
            resolver.resolve("my_cap"),
            Some(("my_provider".to_string(), "my_action".to_string()))
        );
    }

    #[test]
    fn test_unregistered_capability_returns_none() {
        let resolver = CapabilityResolver::new();

        // Unregistered capabilities return None - no fallback
        assert_eq!(resolver.resolve("unknown_cap"), None);
        assert_eq!(resolver.resolve("io_fs_read"), None);
    }
}
