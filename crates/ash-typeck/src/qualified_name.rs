//! Fully qualified type names for module boundaries.

use std::fmt;

/// A fully qualified name for types, functions, or other named entities.
///
/// Qualified names support module boundaries by combining a module path
/// with a base name. This allows disambiguating names that might be
/// defined in different modules.
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
///     vec!["Std".to_string(), "Maybe".to_string()],
///     "Option"
/// );
/// assert_eq!(option.to_string(), "Std.Maybe.Option");
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
        Self {
            module: vec![],
            name: name.into(),
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
    ///     vec!["Std".to_string(), "Maybe".to_string()],
    ///     "Option"
    /// );
    /// assert!(!name.is_root());
    /// assert_eq!(name.to_string(), "Std.Maybe.Option");
    /// ```
    pub fn qualified(module: Vec<String>, name: impl Into<String>) -> Self {
        Self {
            module,
            name: name.into(),
        }
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

    /// Get the full path as a string (Std.Maybe.Option)
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
    ///     vec!["Std".to_string(), "Maybe".to_string()],
    ///     "Option"
    /// );
    /// assert_eq!(qualified.display(), "Std.Maybe.Option");
    /// ```
    pub fn display(&self) -> String {
        if self.module.is_empty() {
            self.name.clone()
        } else {
            format!("{}.{}", self.module.join("."), self.name)
        }
    }

    /// Parse from a dotted string
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_typeck::QualifiedName;
    ///
    /// let name = QualifiedName::parse("Std.Maybe.Option");
    /// assert_eq!(name.module, vec!["Std", "Maybe"]);
    /// assert_eq!(name.name, "Option");
    ///
    /// let root = QualifiedName::parse("Int");
    /// assert!(root.is_root());
    /// assert_eq!(root.name, "Int");
    /// ```
    pub fn parse(s: &str) -> Self {
        let parts: Vec<_> = s.split('.').collect();
        if parts.len() == 1 {
            Self::root(parts[0])
        } else {
            Self::qualified(
                parts[..parts.len() - 1].iter().map(|s| s.to_string()).collect(),
                *parts.last().unwrap(),
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
            QualifiedName::qualified(vec!["Std".to_string(), "Maybe".to_string()], "Option");
        assert!(!name.is_root());
        assert_eq!(name.display(), "Std.Maybe.Option");
    }

    #[test]
    fn qualified_name_parse() {
        let name = QualifiedName::parse("Std.Maybe.Option");
        assert_eq!(name.module, vec!["Std", "Maybe"]);
        assert_eq!(name.name, "Option");

        let root = QualifiedName::parse("Int");
        assert!(root.is_root());
        assert_eq!(root.name, "Int");
    }
}
