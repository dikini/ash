//! Fully qualified type names for module boundaries.
//!
//! This module provides `QualifiedName` for representing fully qualified type names
//! that preserve module paths, ensuring that types with the same root name in
//! different modules are distinct (SPEC-003 Section 3.3 compliance).

use std::fmt;

/// A fully qualified name for types, functions, or other named entities.
///
/// Qualified names support module boundaries by combining a module path
/// with a base name. This allows disambiguating names that might be
/// defined in different modules.
///
/// Uses `::` as the separator to align with ADT naming conventions.
///
/// # Examples
///
/// ```
/// use ash_typeck::QualifiedName;
///
/// // Root-level name (no module)
/// let int = QualifiedName::root("Int");
/// assert_eq!(int.to_string(), "Int");
///
/// // Qualified name with module path
/// let option = QualifiedName::qualified(
///     vec!["std".to_string(), "option".to_string()],
///     "Option"
/// );
/// assert_eq!(option.to_string(), "std::option::Option");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QualifiedName {
    /// Module path: ["Std", "Maybe"]
    pub module: Vec<String>,
    /// Base name: "Option"
    pub name: String,
}

impl QualifiedName {
    /// Create a root-level name (no module)
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_typeck::QualifiedName;
    ///
    /// let name = QualifiedName::root("Int");
    /// assert!(name.is_root());
    /// assert_eq!(name.to_string(), "Int");
    /// ```
    pub fn root(name: impl Into<String>) -> Self {
        let name = name.into();
        assert!(!name.is_empty(), "name cannot be empty");
        Self {
            module: vec![],
            name,
        }
    }

    /// Create a qualified name
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_typeck::QualifiedName;
    ///
    /// let name = QualifiedName::qualified(
    ///     vec!["std".to_string(), "option".to_string()],
    ///     "Option"
    /// );
    /// assert!(!name.is_root());
    /// assert_eq!(name.to_string(), "std::option::Option");
    /// ```
    pub fn qualified(module: Vec<String>, name: impl Into<String>) -> Self {
        let name = name.into();
        assert!(!name.is_empty(), "name cannot be empty");
        assert!(
            module.iter().all(|m| !m.is_empty()),
            "module path components cannot be empty"
        );
        Self { module, name }
    }

    /// Check if this is a root-level name
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_typeck::QualifiedName;
    ///
    /// assert!(QualifiedName::root("Int").is_root());
    /// assert!(!QualifiedName::qualified(vec!["Std".to_string()], "Int").is_root());
    /// ```
    pub fn is_root(&self) -> bool {
        self.module.is_empty()
    }

    /// Get the full path as a string (std::option::Option)
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_typeck::QualifiedName;
    ///
    /// let root = QualifiedName::root("Int");
    /// assert_eq!(root.display(), "Int");
    ///
    /// let qualified = QualifiedName::qualified(
    ///     vec!["std".to_string(), "option".to_string()],
    ///     "Option"
    /// );
    /// assert_eq!(qualified.display(), "std::option::Option");
    /// ```
    pub fn display(&self) -> String {
        if self.module.is_empty() {
            self.name.clone()
        } else {
            format!("{}::{}", self.module.join("::"), self.name)
        }
    }

    /// Parse from a string with `::` separator
    ///
    /// Supports both qualified names (e.g., "std::option::Option") and
    /// root-level names (e.g., "Int").
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_typeck::QualifiedName;
    ///
    /// let name = QualifiedName::parse("std::option::Option");
    /// assert_eq!(name.module, vec!["std", "option"]);
    /// assert_eq!(name.name, "Option");
    ///
    /// let root = QualifiedName::parse("Int");
    /// assert!(root.is_root());
    /// assert_eq!(root.name, "Int");
    /// ```
    pub fn parse(s: &str) -> Self {
        assert!(!s.is_empty(), "qualified name string cannot be empty");

        // Use :: as the primary separator for ADT naming conventions
        let separator = if s.contains("::") {
            "::"
        } else if s.contains('.') {
            // Also support . separator for backward compatibility
            "."
        } else {
            // No separator found, treat as root name
            return Self::root(s);
        };

        let parts: Vec<_> = s.split(separator).collect();
        // Check for empty components (e.g., "::Foo" or "Foo::" or "Foo::::Bar")
        assert!(
            parts.iter().all(|p| !p.is_empty()),
            "qualified name components cannot be empty: {:?}",
            s
        );
        if parts.len() == 1 {
            Self::root(parts[0])
        } else {
            Self::qualified(
                parts[..parts.len() - 1]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                *parts
                    .last()
                    .expect("parts is non-empty because s is non-empty"),
            )
        }
    }
}

impl fmt::Display for QualifiedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualified_name_root() {
        let name = QualifiedName::root("Option");
        assert!(name.is_root());
        assert_eq!(name.display(), "Option");
    }

    #[test]
    fn qualified_name_qualified() {
        let name =
            QualifiedName::qualified(vec!["std".to_string(), "option".to_string()], "Option");
        assert!(!name.is_root());
        assert_eq!(name.display(), "std::option::Option");
    }

    #[test]
    fn qualified_name_parse_double_colon() {
        // Primary separator is ::
        let name = QualifiedName::parse("std::option::Option");
        assert_eq!(name.module, vec!["std", "option"]);
        assert_eq!(name.name, "Option");
        assert_eq!(name.display(), "std::option::Option");

        let root = QualifiedName::parse("Int");
        assert!(root.is_root());
        assert_eq!(root.name, "Int");
    }

    #[test]
    fn qualified_name_parse_dot_backward_compat() {
        // Dot separator still works for backward compatibility
        let name = QualifiedName::parse("std.option.Option");
        assert_eq!(name.module, vec!["std", "option"]);
        assert_eq!(name.name, "Option");
    }

    #[test]
    fn qualified_name_equality_uses_full_path() {
        // Key SPEC-003 compliance test: a::T and b::T are different types
        let a_t = QualifiedName::parse("a::T");
        let b_t = QualifiedName::parse("b::T");
        let a_t_copy = QualifiedName::parse("a::T");

        assert_eq!(a_t, a_t_copy, "Same qualified name should be equal");
        assert_ne!(a_t, b_t, "Different qualified names should not be equal");
    }

    #[test]
    fn qualified_name_collision_prevention() {
        // std::option::Option and my::option::Option should be different
        let std_option = QualifiedName::parse("std::option::Option");
        let my_option = QualifiedName::parse("my::option::Option");

        assert_eq!(std_option.name, "Option");
        assert_eq!(my_option.name, "Option");
        assert_ne!(std_option.module, my_option.module);
        assert_ne!(std_option, my_option);
    }
}
