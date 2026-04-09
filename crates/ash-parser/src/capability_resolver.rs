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

    /// Create a resolver with built-in mappings for common capabilities.
    pub fn with_builtin_mappings() -> Self {
        let mut resolver = Self::new();

        // Register built-in io capabilities (both symbolic and qualified forms)
        resolver.register("fs_read", "io", "fs_read");
        resolver.register("io::fs_read", "io", "fs_read");
        resolver.register("fs_write", "io", "fs_write");
        resolver.register("io::fs_write", "io", "fs_write");
        resolver.register("read_file", "io", "read_file");
        resolver.register("io::read_file", "io", "read_file");
        resolver.register("write_file", "io", "write_file");
        resolver.register("io::write_file", "io", "write_file");

        // Register built-in stdio capabilities
        resolver.register("print", "stdio", "print");
        resolver.register("stdio::print", "stdio", "print");
        resolver.register("println", "stdio", "println");
        resolver.register("stdio::println", "stdio", "println");
        resolver.register("read_line", "stdio", "read_line");
        resolver.register("stdio::read_line", "stdio", "read_line");
        resolver.register("prompt", "stdio", "prompt");
        resolver.register("stdio::prompt", "stdio", "prompt");

        // Register built-in http capabilities
        resolver.register("http_get", "http", "get");
        resolver.register("http::get", "http", "get");
        resolver.register("http_post", "http", "post");
        resolver.register("http::post", "http", "post");
        resolver.register("http_put", "http", "put");
        resolver.register("http::put", "http", "put");
        resolver.register("http_delete", "http", "delete");
        resolver.register("http::delete", "http", "delete");

        // Register built-in env capabilities
        resolver.register("get_env", "env", "get_env");
        resolver.register("env::get_env", "env", "get_env");
        resolver.register("set_env", "env", "set_env");
        resolver.register("env::set_env", "env", "set_env");

        resolver
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

    #[test]
    fn test_builtin_mappings() {
        let resolver = CapabilityResolver::with_builtin_mappings();

        // Built-in mappings should resolve
        assert!(resolver.resolve("fs_read").is_some());
        assert!(resolver.resolve("print").is_some());
        assert!(resolver.resolve("http_get").is_some());

        // Unknown capabilities still return None
        assert_eq!(resolver.resolve("unknown_cap"), None);
    }
}
