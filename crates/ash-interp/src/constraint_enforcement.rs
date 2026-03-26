//! Runtime constraint enforcement for capability grants
//!
//! This module provides enforcement of capability constraints at runtime,
//! checking operations against declared constraints like paths, hosts, and permissions.
//!
//! # Example
//!
//! ```
//! use ash_interp::constraint_enforcement::{ConstraintEnforcer, ConstraintViolation};
//! use ash_parser::surface::{ConstraintBlock, ConstraintField, ConstraintValue};
//! use ash_core::Value;
//! use ash_parser::token::Span;
//!
//! let constraints = ConstraintBlock {
//!     fields: vec![
//!         ConstraintField {
//!             name: "paths".into(),
//!             value: ConstraintValue::Array(vec![ConstraintValue::String("/tmp/*".to_string())]),
//!             span: Span::default(),
//!         },
//!     ],
//!     span: Span::default(),
//! };
//!
//! let args = Value::Record(Box::new(std::collections::HashMap::from([
//!     ("path".to_string(), Value::String("/tmp/data.txt".to_string())),
//! ])));
//!
//! let result = ConstraintEnforcer::check("read", &args, &constraints);
//! assert!(result.is_ok());
//! ```

use ash_core::Value;
use ash_parser::surface::{ConstraintBlock, ConstraintValue};
use std::collections::HashMap;
use std::fmt;

/// Errors that can occur during constraint enforcement
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintViolation {
    /// Required argument missing
    MissingArgument(&'static str),
    /// Constraint format invalid
    InvalidConstraint(&'static str),
    /// Path doesn't match allowed patterns
    PathNotAllowed {
        /// The path that was attempted
        path: String,
        /// The allowed patterns
        allowed: Vec<String>,
    },
    /// Host doesn't match allowed patterns
    HostNotAllowed {
        /// The host that was attempted
        host: String,
        /// The allowed patterns
        allowed: Vec<String>,
    },
    /// Operation requires denied permission
    PermissionDenied {
        /// The operation being performed
        operation: String,
        /// The permission that was denied
        permission: String,
    },
}

impl fmt::Display for ConstraintViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConstraintViolation::MissingArgument(arg) => {
                write!(f, "missing required argument: {}", arg)
            }
            ConstraintViolation::InvalidConstraint(reason) => {
                write!(f, "invalid constraint format: {}", reason)
            }
            ConstraintViolation::PathNotAllowed { path, allowed } => {
                write!(
                    f,
                    "path '{}' not allowed. allowed patterns: {:?}",
                    path, allowed
                )
            }
            ConstraintViolation::HostNotAllowed { host, allowed } => {
                write!(
                    f,
                    "host '{}' not allowed. allowed patterns: {:?}",
                    host, allowed
                )
            }
            ConstraintViolation::PermissionDenied {
                operation,
                permission,
            } => {
                write!(
                    f,
                    "operation '{}' requires '{}' permission, which is denied",
                    operation, permission
                )
            }
        }
    }
}

impl std::error::Error for ConstraintViolation {}

/// Enforces capability constraints at runtime
#[derive(Debug, Clone, Copy, Default)]
pub struct ConstraintEnforcer;

impl ConstraintEnforcer {
    /// Check if an operation satisfies the given constraints
    ///
    /// # Arguments
    /// * `operation` - The operation being performed (e.g., "read", "write", "get")
    /// * `args` - Arguments to the operation, typically containing path/host values
    /// * `constraints` - The constraint block to check against
    ///
    /// # Returns
    /// * `Ok(())` - If all constraints are satisfied
    /// * `Err(ConstraintViolation)` - If any constraint is violated
    pub fn check(
        operation: &str,
        args: &Value,
        constraints: &ConstraintBlock,
    ) -> Result<(), ConstraintViolation> {
        for field in &constraints.fields {
            match field.name.as_ref() {
                "paths" => Self::check_paths(operation, args, &field.value)?,
                "hosts" => Self::check_hosts(operation, args, &field.value)?,
                "ports" => Self::check_ports(operation, args, &field.value)?,
                "read" | "write" | "spawn" | "kill" => {
                    Self::check_permission(operation, &field.name, &field.value)?;
                }
                _ => {} // Unknown constraint, allow
            }
        }
        Ok(())
    }

    /// Check path constraints against allowed patterns
    ///
    /// Path constraints are only checked for file operations (read/write).
    ///
    /// # Arguments
    /// * `operation` - The operation being performed
    /// * `args` - Arguments containing the path to check
    /// * `allowed_paths` - The constraint value containing allowed path patterns
    pub fn check_paths(
        operation: &str,
        args: &Value,
        allowed_paths: &ConstraintValue,
    ) -> Result<(), ConstraintViolation> {
        // Path constraints only apply to file operations
        if !Self::is_file_operation(operation) {
            return Ok(());
        }

        // Extract path from args
        let path = Self::extract_string_arg(args, "path")
            .ok_or(ConstraintViolation::MissingArgument("path"))?;

        // Parse allowed patterns
        let patterns = Self::extract_string_array(allowed_paths).ok_or(
            ConstraintViolation::InvalidConstraint("paths must be an array of strings"),
        )?;

        // Check if path matches any allowed pattern
        for pattern in &patterns {
            if Self::path_matches(&path, pattern) {
                return Ok(());
            }
        }

        Err(ConstraintViolation::PathNotAllowed {
            path,
            allowed: patterns,
        })
    }

    /// Check host constraints against allowed patterns
    ///
    /// # Arguments
    /// * `operation` - The operation being performed
    /// * `args` - Arguments containing the host to check
    /// * `allowed_hosts` - The constraint value containing allowed host patterns
    pub fn check_hosts(
        operation: &str,
        args: &Value,
        allowed_hosts: &ConstraintValue,
    ) -> Result<(), ConstraintViolation> {
        // Host constraints only apply to network operations
        if !Self::is_network_operation(operation) {
            return Ok(());
        }

        // Extract host from args
        let host = Self::extract_string_arg(args, "host")
            .ok_or(ConstraintViolation::MissingArgument("host"))?;

        // Parse allowed patterns
        let patterns = Self::extract_string_array(allowed_hosts).ok_or(
            ConstraintViolation::InvalidConstraint("hosts must be an array of strings"),
        )?;

        // Check if host matches any allowed pattern
        for pattern in &patterns {
            if Self::host_matches(&host, pattern) {
                return Ok(());
            }
        }

        Err(ConstraintViolation::HostNotAllowed {
            host,
            allowed: patterns,
        })
    }

    /// Check port constraints
    ///
    /// # Arguments
    /// * `operation` - The operation being performed
    /// * `args` - Arguments containing the port to check
    /// * `allowed_ports` - The constraint value containing allowed ports
    pub fn check_ports(
        operation: &str,
        args: &Value,
        allowed_ports: &ConstraintValue,
    ) -> Result<(), ConstraintViolation> {
        // Port constraints only apply to network operations
        if !Self::is_network_operation(operation) {
            return Ok(());
        }

        // Extract port from args - it can be a number or string
        let port = match Self::extract_arg(args, "port") {
            Some(Value::Int(i)) => *i as u16,
            Some(Value::String(s)) => s.parse::<u16>().map_err(|_| {
                ConstraintViolation::InvalidConstraint("port must be a valid number")
            })?,
            None => return Ok(()), // Port is optional
            _ => {
                return Err(ConstraintViolation::InvalidConstraint(
                    "port must be a number",
                ));
            }
        };

        // Parse allowed ports
        let allowed = match allowed_ports {
            ConstraintValue::Array(arr) => arr,
            _ => {
                return Err(ConstraintViolation::InvalidConstraint(
                    "ports must be an array",
                ));
            }
        };

        // Check if port is in allowed list
        for allowed_val in allowed {
            let allowed_port = match allowed_val {
                ConstraintValue::Int(i) => *i as u16,
                ConstraintValue::String(s) => s.parse::<u16>().map_err(|_| {
                    ConstraintViolation::InvalidConstraint("ports must be valid numbers")
                })?,
                _ => continue,
            };

            if port == allowed_port {
                return Ok(());
            }
        }

        // Port not in allowed list - this is a constraint violation
        // For now, we return an error with the port number
        Err(ConstraintViolation::InvalidConstraint(
            "port not in allowed list - permission denied",
        ))
    }

    /// Check permission-based constraints
    ///
    /// # Arguments
    /// * `operation` - The operation being performed
    /// * `permission` - The permission name (read/write/spawn/kill)
    /// * `value` - The constraint value (should be a boolean)
    pub fn check_permission(
        operation: &str,
        permission: &str,
        value: &ConstraintValue,
    ) -> Result<(), ConstraintViolation> {
        let allowed = match value {
            ConstraintValue::Bool(b) => *b,
            _ => return Ok(()), // Non-bool value, assume allowed
        };

        // Map operation to permission
        let required_perm = Self::operation_to_permission(operation);

        if required_perm == permission && !allowed {
            return Err(ConstraintViolation::PermissionDenied {
                operation: operation.to_string(),
                permission: permission.to_string(),
            });
        }

        Ok(())
    }

    /// Check if a path matches a glob pattern
    ///
    /// Supports simple glob patterns:
    /// - `*` matches any sequence of characters except `/`
    /// - `**` matches any sequence of characters including `/`
    /// - `/tmp/*` matches `/tmp/data.txt` but not `/tmp/subdir/file.txt`
    /// - `/tmp/**` matches `/tmp/data.txt` and `/tmp/subdir/file.txt`
    ///
    /// # Arguments
    /// * `path` - The path to check
    /// * `pattern` - The glob pattern to match against
    ///
    /// # Returns
    /// * `true` if the path matches the pattern
    pub fn path_matches(path: &str, pattern: &str) -> bool {
        // Handle exact match
        if path == pattern {
            return true;
        }

        // Handle directory prefix patterns like /tmp/*
        if pattern.ends_with("/*") && !pattern.ends_with("/**") {
            let prefix = &pattern[..pattern.len() - 1]; // Keep the trailing slash
            if let Some(stripped) = path.strip_prefix(prefix) {
                // Make sure there's no additional directory separator after the prefix match
                // /tmp/* should match /tmp/file.txt but not /tmp/subdir/file.txt
                return !stripped.contains('/');
            }
            return false;
        }

        // Handle recursive directory patterns like /tmp/**
        if pattern.ends_with("/**") {
            let prefix = &pattern[..pattern.len() - 2]; // Remove /**
            return path.starts_with(prefix);
        }

        // Handle patterns with * in the middle or at the start
        if pattern.contains('*') {
            return Self::glob_matches(path, pattern);
        }

        // Simple prefix check for directories
        if pattern.ends_with('/') {
            return path.starts_with(pattern);
        }

        false
    }

    /// Check if a host matches a pattern
    ///
    /// Supports wildcard patterns:
    /// - `*.example.com` matches `api.example.com` and `www.example.com`
    /// - `example.com` matches only `example.com`
    ///
    /// # Arguments
    /// * `host` - The host to check
    /// * `pattern` - The pattern to match against
    ///
    /// # Returns
    /// * `true` if the host matches the pattern
    #[allow(clippy::collapsible_if)]
    pub fn host_matches(host: &str, pattern: &str) -> bool {
        // Exact match
        if host == pattern {
            return true;
        }

        // Wildcard pattern like *.example.com
        if let Some(suffix) = pattern.strip_prefix("*.") {
            // Host must end with the suffix and have at least one character before it
            // Also ensure the character before the suffix is a dot
            if let Some(pos) = host.rfind(suffix) {
                if pos > 0 && pos + suffix.len() == host.len() {
                    // Check that the character before the match is a dot
                    return host.as_bytes()[pos - 1] == b'.';
                }
            }
        }

        false
    }

    // Helper methods

    fn is_file_operation(operation: &str) -> bool {
        matches!(operation, "read" | "write" | "append" | "delete")
    }

    fn is_network_operation(operation: &str) -> bool {
        matches!(operation, "get" | "post" | "put" | "delete" | "connect")
    }

    fn operation_to_permission(operation: &str) -> &str {
        match operation {
            "read" | "get" => "read",
            "write" | "put" | "post" | "append" => "write",
            "spawn" => "spawn",
            "kill" => "kill",
            _ => operation,
        }
    }

    fn extract_arg<'a>(args: &'a Value, key: &str) -> Option<&'a Value> {
        match args {
            Value::Record(map) => map.get(key),
            _ => None,
        }
    }

    fn extract_string_arg(args: &Value, key: &str) -> Option<String> {
        Self::extract_arg(args, key).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
    }

    fn extract_string_array(value: &ConstraintValue) -> Option<Vec<String>> {
        match value {
            ConstraintValue::Array(arr) => {
                let mut result = Vec::new();
                for item in arr {
                    match item {
                        ConstraintValue::String(s) => result.push(s.clone()),
                        _ => return None, // Non-string element
                    }
                }
                Some(result)
            }
            _ => None,
        }
    }

    fn glob_matches(text: &str, pattern: &str) -> bool {
        // Simple glob matching implementation
        // Supports * which matches any sequence of characters
        let mut text_chars = text.chars().peekable();
        let mut pattern_chars = pattern.chars().peekable();

        while let Some(p) = pattern_chars.peek() {
            match p {
                '*' => {
                    pattern_chars.next();
                    if pattern_chars.peek().is_none() {
                        // * at end matches everything remaining
                        return true;
                    }
                    // Try to match the rest of the pattern
                    let remaining_pattern: String = pattern_chars.collect();
                    // Check if remaining pattern appears in text
                    for i in 0..text_chars.clone().count() {
                        let suffix: String = text_chars.clone().skip(i).collect();
                        if Self::glob_matches(&suffix, &remaining_pattern) {
                            return true;
                        }
                    }
                    return false;
                }
                c => {
                    if text_chars.peek() != Some(c) {
                        return false;
                    }
                    text_chars.next();
                    pattern_chars.next();
                }
            }
        }

        text_chars.peek().is_none()
    }
}

/// Helper function to create a Value::Record from key-value pairs
pub fn args_from_pairs(pairs: Vec<(&str, &str)>) -> Value {
    let mut map = HashMap::new();
    for (key, value) in pairs {
        map.insert(key.to_string(), Value::String(value.to_string()));
    }
    Value::Record(Box::new(map))
}

/// Helper function to create a simple constraint block from fields
pub fn constraint_block_from_fields(fields: Vec<(&str, ConstraintValue)>) -> ConstraintBlock {
    use ash_parser::surface::ConstraintField;
    use ash_parser::token::Span;

    ConstraintBlock {
        fields: fields
            .into_iter()
            .map(|(name, value)| ConstraintField {
                name: name.into(),
                value,
                span: Span::default(),
            })
            .collect(),
        span: Span::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_matches_exact() {
        assert!(ConstraintEnforcer::path_matches(
            "/tmp/data.txt",
            "/tmp/data.txt"
        ));
        assert!(!ConstraintEnforcer::path_matches(
            "/tmp/other.txt",
            "/tmp/data.txt"
        ));
    }

    #[test]
    fn test_path_matches_glob() {
        // Simple glob patterns
        assert!(ConstraintEnforcer::path_matches("/tmp/data.txt", "/tmp/*"));
        assert!(ConstraintEnforcer::path_matches("/tmp/file.log", "/tmp/*"));
        assert!(!ConstraintEnforcer::path_matches("/etc/passwd", "/tmp/*"));
        assert!(!ConstraintEnforcer::path_matches(
            "/tmp/subdir/file.txt",
            "/tmp/*"
        ));

        // Recursive glob patterns
        assert!(ConstraintEnforcer::path_matches("/tmp/data.txt", "/tmp/**"));
        assert!(ConstraintEnforcer::path_matches(
            "/tmp/subdir/file.txt",
            "/tmp/**"
        ));
        assert!(ConstraintEnforcer::path_matches(
            "/tmp/a/b/c/d.txt",
            "/tmp/**"
        ));
        assert!(!ConstraintEnforcer::path_matches(
            "/var/tmp/file.txt",
            "/tmp/**"
        ));
    }

    #[test]
    fn test_host_matches_exact() {
        assert!(ConstraintEnforcer::host_matches(
            "example.com",
            "example.com"
        ));
        assert!(!ConstraintEnforcer::host_matches(
            "api.example.com",
            "example.com"
        ));
    }

    #[test]
    fn test_host_matches_wildcard() {
        assert!(ConstraintEnforcer::host_matches(
            "api.example.com",
            "*.example.com"
        ));
        assert!(ConstraintEnforcer::host_matches(
            "www.example.com",
            "*.example.com"
        ));
        assert!(ConstraintEnforcer::host_matches(
            "deep.sub.example.com",
            "*.example.com"
        ));
        assert!(!ConstraintEnforcer::host_matches(
            "example.com",
            "*.example.com"
        ));
        assert!(!ConstraintEnforcer::host_matches(
            "other.com",
            "*.example.com"
        ));
        assert!(!ConstraintEnforcer::host_matches(
            "notexample.com",
            "*.example.com"
        ));
    }

    #[test]
    fn test_extract_string_array() {
        let arr = ConstraintValue::Array(vec![
            ConstraintValue::String("/tmp/*".to_string()),
            ConstraintValue::String("/var/log/*".to_string()),
        ]);

        let result = ConstraintEnforcer::extract_string_array(&arr);
        assert_eq!(
            result,
            Some(vec!["/tmp/*".to_string(), "/var/log/*".to_string()])
        );
    }

    #[test]
    fn test_extract_string_array_rejects_non_strings() {
        let arr = ConstraintValue::Array(vec![
            ConstraintValue::String("/tmp/*".to_string()),
            ConstraintValue::Int(42),
        ]);

        let result = ConstraintEnforcer::extract_string_array(&arr);
        assert_eq!(result, None);
    }

    #[test]
    fn test_args_from_pairs() {
        let args = args_from_pairs(vec![("path", "/tmp/data.txt"), ("host", "example.com")]);

        match &args {
            Value::Record(map) => {
                assert_eq!(
                    map.get("path"),
                    Some(&Value::String("/tmp/data.txt".to_string()))
                );
                assert_eq!(
                    map.get("host"),
                    Some(&Value::String("example.com".to_string()))
                );
            }
            _ => panic!("Expected Record"),
        }
    }

    #[test]
    fn test_constraint_block_from_fields() {
        let block = constraint_block_from_fields(vec![(
            "paths",
            ConstraintValue::Array(vec![ConstraintValue::String("/tmp/*".to_string())]),
        )]);

        assert_eq!(block.fields.len(), 1);
        assert_eq!(block.fields[0].name.as_ref(), "paths");
    }

    #[test]
    fn test_check_permission_allowed() {
        let result =
            ConstraintEnforcer::check_permission("read", "read", &ConstraintValue::Bool(true));
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_permission_denied() {
        let result =
            ConstraintEnforcer::check_permission("read", "read", &ConstraintValue::Bool(false));
        assert!(result.is_err());

        match result.unwrap_err() {
            ConstraintViolation::PermissionDenied {
                operation,
                permission,
            } => {
                assert_eq!(operation, "read");
                assert_eq!(permission, "read");
            }
            _ => panic!("Expected PermissionDenied"),
        }
    }

    #[test]
    fn test_check_permission_unrelated() {
        // Write permission doesn't affect read operation
        let result =
            ConstraintEnforcer::check_permission("read", "write", &ConstraintValue::Bool(false));
        assert!(result.is_ok());
    }

    #[test]
    fn test_operation_to_permission_mapping() {
        assert_eq!(ConstraintEnforcer::operation_to_permission("read"), "read");
        assert_eq!(ConstraintEnforcer::operation_to_permission("get"), "read");
        assert_eq!(
            ConstraintEnforcer::operation_to_permission("write"),
            "write"
        );
        assert_eq!(ConstraintEnforcer::operation_to_permission("put"), "write");
        assert_eq!(ConstraintEnforcer::operation_to_permission("post"), "write");
        assert_eq!(
            ConstraintEnforcer::operation_to_permission("spawn"),
            "spawn"
        );
        assert_eq!(ConstraintEnforcer::operation_to_permission("kill"), "kill");
        assert_eq!(
            ConstraintEnforcer::operation_to_permission("custom"),
            "custom"
        );
    }

    #[test]
    fn test_display_error_messages() {
        let err = ConstraintViolation::MissingArgument("path");
        assert_eq!(err.to_string(), "missing required argument: path");

        let err = ConstraintViolation::InvalidConstraint("paths must be array");
        assert_eq!(
            err.to_string(),
            "invalid constraint format: paths must be array"
        );

        let err = ConstraintViolation::PathNotAllowed {
            path: "/etc/passwd".to_string(),
            allowed: vec!["/tmp/*".to_string()],
        };
        assert!(err.to_string().contains("'/etc/passwd' not allowed"));

        let err = ConstraintViolation::HostNotAllowed {
            host: "evil.com".to_string(),
            allowed: vec!["*.example.com".to_string()],
        };
        assert!(err.to_string().contains("'evil.com' not allowed"));

        let err = ConstraintViolation::PermissionDenied {
            operation: "write".to_string(),
            permission: "write".to_string(),
        };
        assert!(err.to_string().contains("'write' permission"));
    }
}
