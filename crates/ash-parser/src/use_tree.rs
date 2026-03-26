//! AST types for `use` statements in the Ash parser.
//!
//! This module defines the types representing use/import declarations,
//! supporting various forms like simple paths, globs, and nested imports.

use crate::surface::Visibility;
use crate::token::Span;

/// A simple path like `crate::foo::bar`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimplePath {
    /// The segments of the path (e.g., `["crate", "foo", "bar"]`).
    pub segments: Vec<Box<str>>,
}

/// The path component of a use statement.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UsePath {
    /// A simple path: `crate::foo::bar`
    Simple(SimplePath),
    /// A glob import: `crate::foo::*`
    Glob(SimplePath),
    /// A nested import: `crate::foo::{bar, baz}`
    Nested(SimplePath, Vec<UseItem>),
}

/// An item in a nested use statement.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UseItem {
    /// The name being imported.
    pub name: Box<str>,
    /// Optional alias: `as alias_name`
    pub alias: Option<Box<str>>,
}

/// A complete use statement.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Use {
    /// Visibility modifier (e.g., `pub`, `pub(crate)`).
    pub visibility: Visibility,
    /// The import path.
    pub path: UsePath,
    /// Optional alias for the entire import.
    pub alias: Option<Box<str>>,
    /// Source span of the use statement.
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Construction Tests - RED phase (tests should fail initially)
    // =========================================================================

    #[test]
    fn test_simple_use_construction() {
        let path = SimplePath {
            segments: vec!["crate".into(), "foo".into(), "bar".into()],
        };
        let use_stmt = Use {
            visibility: Visibility::Inherited,
            path: UsePath::Simple(path),
            alias: None,
            span: Span::new(0, 20, 1, 1),
        };

        assert!(matches!(use_stmt.visibility, Visibility::Inherited));
        assert!(use_stmt.alias.is_none());
        assert_eq!(use_stmt.span.start, 0);
        assert_eq!(use_stmt.span.end, 20);
    }

    #[test]
    fn test_use_with_alias() {
        let path = SimplePath {
            segments: vec!["crate".into(), "foo".into()],
        };
        let use_stmt = Use {
            visibility: Visibility::Public,
            path: UsePath::Simple(path),
            alias: Some("my_foo".into()),
            span: Span::new(0, 25, 1, 1),
        };

        assert!(matches!(use_stmt.visibility, Visibility::Public));
        assert_eq!(use_stmt.alias, Some("my_foo".into()));
    }

    #[test]
    fn test_glob_import() {
        let path = SimplePath {
            segments: vec!["crate".into(), "foo".into()],
        };
        let use_stmt = Use {
            visibility: Visibility::Inherited,
            path: UsePath::Glob(path),
            alias: None,
            span: Span::new(0, 15, 1, 1),
        };

        match &use_stmt.path {
            UsePath::Glob(p) => {
                assert_eq!(p.segments.len(), 2);
                assert_eq!(p.segments[0].as_ref(), "crate");
                assert_eq!(p.segments[1].as_ref(), "foo");
            }
            _ => panic!("Expected Glob path"),
        }
    }

    #[test]
    fn test_nested_import() {
        let path = SimplePath {
            segments: vec!["crate".into(), "foo".into()],
        };
        let items = vec![
            UseItem {
                name: "bar".into(),
                alias: None,
            },
            UseItem {
                name: "baz".into(),
                alias: Some("my_baz".into()),
            },
        ];
        let use_stmt = Use {
            visibility: Visibility::Crate,
            path: UsePath::Nested(path, items),
            alias: None,
            span: Span::new(0, 30, 1, 1),
        };

        match &use_stmt.path {
            UsePath::Nested(p, items) => {
                assert_eq!(p.segments.len(), 2);
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].name.as_ref(), "bar");
                assert!(items[0].alias.is_none());
                assert_eq!(items[1].name.as_ref(), "baz");
                assert_eq!(items[1].alias, Some("my_baz".into()));
            }
            _ => panic!("Expected Nested path"),
        }
    }

    #[test]
    fn test_all_visibility_variants() {
        let visibilities = vec![
            Visibility::Inherited,
            Visibility::Public,
            Visibility::Crate,
            Visibility::Super { levels: 1 },
            Visibility::Self_,
            Visibility::Restricted {
                path: "crate::foo".into(),
            },
        ];

        for (i, visibility) in visibilities.into_iter().enumerate() {
            let path = SimplePath {
                segments: vec!["crate".into(), "module".into()],
            };
            let use_stmt = Use {
                visibility,
                path: UsePath::Simple(path),
                alias: None,
                span: Span::new(i * 10, i * 10 + 5, 1, 1),
            };

            // Verify the visibility was set correctly
            match (i, &use_stmt.visibility) {
                (0, Visibility::Inherited) => {}
                (1, Visibility::Public) => {}
                (2, Visibility::Crate) => {}
                (3, Visibility::Super { levels: 1 }) => {}
                (4, Visibility::Self_) => {}
                (5, Visibility::Restricted { path }) => {
                    assert_eq!(path.as_ref(), "crate::foo");
                }
                _ => panic!("Unexpected visibility variant at index {}", i),
            }
        }
    }

    // =========================================================================
    // Additional Edge Case Tests
    // =========================================================================

    #[test]
    fn test_single_segment_path() {
        let path = SimplePath {
            segments: vec!["foo".into()],
        };
        let use_stmt = Use {
            visibility: Visibility::Inherited,
            path: UsePath::Simple(path),
            alias: None,
            span: Span::new(0, 10, 1, 1),
        };

        match &use_stmt.path {
            UsePath::Simple(p) => {
                assert_eq!(p.segments.len(), 1);
                assert_eq!(p.segments[0].as_ref(), "foo");
            }
            _ => panic!("Expected Simple path"),
        }
    }

    #[test]
    fn test_empty_nested_items() {
        let path = SimplePath {
            segments: vec!["crate".into()],
        };
        let items: Vec<UseItem> = vec![];
        let use_stmt = Use {
            visibility: Visibility::Inherited,
            path: UsePath::Nested(path, items),
            alias: None,
            span: Span::new(0, 15, 1, 1),
        };

        match &use_stmt.path {
            UsePath::Nested(_, items) => {
                assert!(items.is_empty());
            }
            _ => panic!("Expected Nested path"),
        }
    }

    #[test]
    fn test_use_item_with_alias_only() {
        let item = UseItem {
            name: "foo".into(),
            alias: Some("bar".into()),
        };

        assert_eq!(item.name.as_ref(), "foo");
        assert_eq!(item.alias, Some("bar".into()));
    }

    #[test]
    fn test_use_item_without_alias() {
        let item = UseItem {
            name: "foo".into(),
            alias: None,
        };

        assert_eq!(item.name.as_ref(), "foo");
        assert!(item.alias.is_none());
    }

    #[test]
    fn test_glob_with_long_path() {
        let path = SimplePath {
            segments: vec![
                "crate".into(),
                "a".into(),
                "b".into(),
                "c".into(),
                "d".into(),
            ],
        };
        let use_stmt = Use {
            visibility: Visibility::Inherited,
            path: UsePath::Glob(path),
            alias: None,
            span: Span::new(0, 50, 1, 1),
        };

        match &use_stmt.path {
            UsePath::Glob(p) => {
                assert_eq!(p.segments.len(), 5);
            }
            _ => panic!("Expected Glob path"),
        }
    }

    #[test]
    fn test_use_clone() {
        let path = SimplePath {
            segments: vec!["crate".into(), "foo".into()],
        };
        let use_stmt = Use {
            visibility: Visibility::Public,
            path: UsePath::Simple(path),
            alias: Some("alias".into()),
            span: Span::new(0, 20, 1, 1),
        };

        let cloned = use_stmt.clone();
        assert_eq!(cloned.visibility, use_stmt.visibility);
        assert_eq!(cloned.alias, use_stmt.alias);
        assert_eq!(cloned.span, use_stmt.span);
    }

    #[test]
    fn test_simple_path_equality() {
        let path1 = SimplePath {
            segments: vec!["a".into(), "b".into()],
        };
        let path2 = SimplePath {
            segments: vec!["a".into(), "b".into()],
        };
        let path3 = SimplePath {
            segments: vec!["a".into(), "c".into()],
        };

        assert_eq!(path1, path2);
        assert_ne!(path1, path3);
    }

    #[test]
    fn test_use_path_variants_distinct() {
        let simple = UsePath::Simple(SimplePath {
            segments: vec!["foo".into()],
        });
        let glob = UsePath::Glob(SimplePath {
            segments: vec!["foo".into()],
        });
        let nested = UsePath::Nested(
            SimplePath {
                segments: vec!["foo".into()],
            },
            vec![],
        );

        assert_ne!(simple, glob);
        assert_ne!(simple, nested);
        assert_ne!(glob, nested);
    }
}
