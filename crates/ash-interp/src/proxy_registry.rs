//! Proxy registry for role-to-proxy mapping
//!
//! Provides bidirectional mapping between role names and proxy workflow instances
//! per SPEC-023 Section 3.2.
//!
//! # Example
//!
//! ```
//! use ash_interp::proxy_registry::ProxyRegistry;
//!
//! let mut registry = ProxyRegistry::new();
//!
//! // Register a role to proxy mapping
//! registry.register("admin".to_string(), "proxy://instance-1".to_string());
//!
//! // Lookup proxy for a role
//! assert_eq!(registry.lookup("admin"), Some(&"proxy://instance-1".to_string()));
//!
//! // Get all roles handled by a proxy
//! let roles = registry.get_roles("proxy://instance-1").unwrap();
//! assert!(roles.contains("admin"));
//!
//! // Unregister a role
//! let proxy = registry.unregister("admin");
//! assert_eq!(proxy, Some("proxy://instance-1".to_string()));
//! ```

use std::collections::{HashMap, HashSet};

/// Role name type alias
pub type RoleName = String;

/// Proxy instance address type alias
pub type InstanceAddr = String;

/// Registry for role-to-proxy bidirectional mapping
///
/// Maintains mappings from roles to proxy instances and vice versa,
/// enabling efficient lookup in both directions.
#[derive(Debug, Clone, Default)]
pub struct ProxyRegistry {
    /// Maps role names to proxy workflow instances
    role_proxies: HashMap<RoleName, InstanceAddr>,
    /// Maps proxy instances to handled roles
    proxy_roles: HashMap<InstanceAddr, HashSet<RoleName>>,
}

impl ProxyRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            role_proxies: HashMap::new(),
            proxy_roles: HashMap::new(),
        }
    }

    /// Register a role to proxy mapping
    ///
    /// If the role was already registered to a different proxy, it will be
    /// removed from the old proxy's role set and mapped to the new proxy.
    pub fn register(&mut self, role: RoleName, proxy: InstanceAddr) {
        // Remove from old proxy if role was already registered
        if let Some(old_proxy) = self.role_proxies.get(&role) {
            if let Some(roles) = self.proxy_roles.get_mut(old_proxy) {
                roles.remove(&role);
                // Clean up empty proxy entries
                if roles.is_empty() {
                    self.proxy_roles.remove(old_proxy);
                }
            }
        }

        // Update role -> proxy mapping
        self.role_proxies.insert(role.clone(), proxy.clone());

        // Update proxy -> roles mapping
        self.proxy_roles
            .entry(proxy)
            .or_default()
            .insert(role);
    }

    /// Unregister a role, returning its associated proxy if it existed
    pub fn unregister(&mut self, role: &str) -> Option<InstanceAddr> {
        let proxy = self.role_proxies.remove(role)?;

        if let Some(roles) = self.proxy_roles.get_mut(&proxy) {
            roles.remove(role);
            // Clean up empty proxy entries
            if roles.is_empty() {
                self.proxy_roles.remove(&proxy);
            }
        }

        Some(proxy)
    }

    /// Lookup the proxy instance for a role
    pub fn lookup(&self, role: &str) -> Option<&InstanceAddr> {
        self.role_proxies.get(role)
    }

    /// Get all roles handled by a proxy instance
    pub fn get_roles(&self, proxy: &str) -> Option<&HashSet<RoleName>> {
        self.proxy_roles.get(proxy)
    }

    /// Check if a role is registered
    pub fn is_registered(&self, role: &str) -> bool {
        self.role_proxies.contains_key(role)
    }

    /// Clear all registrations
    pub fn clear(&mut self) {
        self.role_proxies.clear();
        self.proxy_roles.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_lookup() {
        let mut registry = ProxyRegistry::new();

        // Register a role
        registry.register("admin".to_string(), "proxy://instance-1".to_string());

        // Lookup should return the proxy
        assert_eq!(
            registry.lookup("admin"),
            Some(&"proxy://instance-1".to_string())
        );
    }

    #[test]
    fn test_unregister() {
        let mut registry = ProxyRegistry::new();

        // Register and then unregister
        registry.register("admin".to_string(), "proxy://instance-1".to_string());
        let proxy = registry.unregister("admin");

        assert_eq!(proxy, Some("proxy://instance-1".to_string()));
        assert_eq!(registry.lookup("admin"), None);
        assert!(!registry.is_registered("admin"));
    }

    #[test]
    fn test_get_roles_for_proxy() {
        let mut registry = ProxyRegistry::new();

        // Register multiple roles to the same proxy
        registry.register("admin".to_string(), "proxy://instance-1".to_string());
        registry.register("moderator".to_string(), "proxy://instance-1".to_string());
        registry.register("user".to_string(), "proxy://instance-2".to_string());

        // Get roles for proxy-1
        let roles = registry.get_roles("proxy://instance-1").unwrap();
        assert_eq!(roles.len(), 2);
        assert!(roles.contains("admin"));
        assert!(roles.contains("moderator"));
        assert!(!roles.contains("user"));

        // Get roles for proxy-2
        let roles = registry.get_roles("proxy://instance-2").unwrap();
        assert_eq!(roles.len(), 1);
        assert!(roles.contains("user"));
    }

    #[test]
    fn test_is_registered() {
        let mut registry = ProxyRegistry::new();

        assert!(!registry.is_registered("admin"));

        registry.register("admin".to_string(), "proxy://instance-1".to_string());

        assert!(registry.is_registered("admin"));
        assert!(!registry.is_registered("nonexistent"));
    }

    #[test]
    fn test_register_same_role_twice_updates() {
        let mut registry = ProxyRegistry::new();

        // Register role to first proxy
        registry.register("admin".to_string(), "proxy://instance-1".to_string());
        assert_eq!(
            registry.lookup("admin"),
            Some(&"proxy://instance-1".to_string())
        );

        // Register same role to second proxy (should update)
        registry.register("admin".to_string(), "proxy://instance-2".to_string());
        assert_eq!(
            registry.lookup("admin"),
            Some(&"proxy://instance-2".to_string())
        );

        // Old proxy should no longer have this role
        let roles = registry.get_roles("proxy://instance-1");
        assert!(roles.is_none() || !roles.unwrap().contains("admin"));

        // New proxy should have the role
        let roles = registry.get_roles("proxy://instance-2").unwrap();
        assert!(roles.contains("admin"));
    }

    #[test]
    fn test_lookup_nonexistent_role() {
        let registry = ProxyRegistry::new();

        assert_eq!(registry.lookup("nonexistent"), None);
        assert_eq!(registry.get_roles("nonexistent"), None);
    }

    #[test]
    fn test_empty_registry_operations() {
        let mut registry = ProxyRegistry::new();

        // Operations on empty registry should not panic
        assert_eq!(registry.lookup("any"), None);
        assert_eq!(registry.get_roles("any"), None);
        assert!(!registry.is_registered("any"));
        assert_eq!(registry.unregister("any"), None);

        // Clear on empty registry should not panic
        registry.clear();
        assert_eq!(registry.lookup("any"), None);
    }

    #[test]
    fn test_clear() {
        let mut registry = ProxyRegistry::new();

        // Add some mappings
        registry.register("admin".to_string(), "proxy://instance-1".to_string());
        registry.register("user".to_string(), "proxy://instance-2".to_string());

        assert!(registry.is_registered("admin"));
        assert!(registry.is_registered("user"));

        // Clear all
        registry.clear();

        assert!(!registry.is_registered("admin"));
        assert!(!registry.is_registered("user"));
        assert_eq!(registry.lookup("admin"), None);
        assert_eq!(registry.get_roles("proxy://instance-1"), None);
    }

    #[test]
    fn test_unregister_removes_empty_proxy() {
        let mut registry = ProxyRegistry::new();

        // Register single role to proxy
        registry.register("admin".to_string(), "proxy://instance-1".to_string());
        assert!(registry.get_roles("proxy://instance-1").is_some());

        // Unregister the only role
        registry.unregister("admin");

        // Proxy entry should be cleaned up
        assert_eq!(registry.get_roles("proxy://instance-1"), None);
    }

    #[test]
    fn test_multiple_roles_per_proxy() {
        let mut registry = ProxyRegistry::new();

        // Register multiple roles to same proxy
        registry.register("role1".to_string(), "proxy://p1".to_string());
        registry.register("role2".to_string(), "proxy://p1".to_string());
        registry.register("role3".to_string(), "proxy://p1".to_string());

        let roles = registry.get_roles("proxy://p1").unwrap();
        assert_eq!(roles.len(), 3);
        assert!(roles.contains("role1"));
        assert!(roles.contains("role2"));
        assert!(roles.contains("role3"));

        // Unregister one role
        registry.unregister("role2");

        let roles = registry.get_roles("proxy://p1").unwrap();
        assert_eq!(roles.len(), 2);
        assert!(roles.contains("role1"));
        assert!(!roles.contains("role2"));
        assert!(roles.contains("role3"));
    }
}
