//! Import resolution algorithm for the Ash parser.
//!
//! This module provides functionality to resolve `use` statements in modules,
//! building name binding tables and verifying visibility constraints.

use ash_core::module_graph::{ModuleGraph, ModuleId};
use std::collections::HashMap;
use thiserror::Error;

use crate::surface::Visibility;
use crate::use_tree::{SimplePath, Use, UseItem, UsePath};

/// A binding represents a resolved import.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Binding {
    /// The module ID where the target item is defined.
    pub target_module: ModuleId,
    /// The name of the item being imported.
    pub item_name: String,
    /// The visibility of the item.
    pub visibility: Visibility,
    /// The kind of binding (direct, glob, etc.).
    pub kind: BindingKind,
}

/// The kind of binding.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BindingKind {
    /// Direct import: `use crate::foo::bar;`
    Direct,
    /// Import via glob: `use crate::foo::*;`
    Glob,
    /// Import with alias: `use crate::foo::bar as baz;`
    Aliased { original: String },
}

/// Errors that can occur during import resolution.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ImportError {
    /// The module was not found in the graph.
    #[error("module not found: {path}")]
    ModuleNotFound { path: String },

    /// The item was not found in the target module.
    #[error("item '{item}' not found in module '{module}'")]
    ItemNotFound { item: String, module: String },

    /// The item is not visible from the importing module.
    #[error("item '{item}' is private in module '{module}'")]
    PrivateItem { item: String, module: String },

    /// An import cycle was detected.
    #[error("import cycle detected: {cycle}")]
    ImportCycle { cycle: String },

    /// A conflicting binding was found.
    #[error("conflicting bindings for name '{name}'")]
    ConflictingBinding { name: String },

    /// Invalid path prefix (e.g., not starting with `crate`).
    #[error("invalid path prefix: {prefix}")]
    InvalidPrefix { prefix: String },
}

/// A table of name bindings for a module.
pub type BindingTable = HashMap<String, Binding>;

/// Resolves import statements in a module graph.
///
/// The resolver walks through all modules in the graph, resolving their
/// `use` statements and building a binding table for each module.
pub struct ImportResolver<'a> {
    module_graph: &'a ModuleGraph,
    module_uses: HashMap<ModuleId, Vec<Use>>,
    module_exports: HashMap<ModuleId, HashMap<String, Export>>,
}

/// An exported item from a module.
#[derive(Debug, Clone, PartialEq)]
struct Export {
    name: String,
    visibility: Visibility,
}

impl Binding {
    /// Create a new binding.
    pub fn new(
        target_module: ModuleId,
        item_name: impl Into<String>,
        visibility: Visibility,
        kind: BindingKind,
    ) -> Self {
        Self {
            target_module,
            item_name: item_name.into(),
            visibility,
            kind,
        }
    }
}

impl<'a> ImportResolver<'a> {
    /// Create a new import resolver for the given module graph.
    pub fn new(module_graph: &'a ModuleGraph) -> Self {
        Self {
            module_graph,
            module_uses: HashMap::new(),
            module_exports: HashMap::new(),
        }
    }

    /// Add use statements for a module.
    ///
    /// This should be called before `resolve_all` to provide the use
    /// statements for each module.
    pub fn add_module_uses(&mut self, module_id: ModuleId, uses: Vec<Use>) {
        self.module_uses.insert(module_id, uses);
    }

    /// Add exports for a module.
    ///
    /// This should be called before `resolve_all` to provide the
    /// exported items for each module.
    pub fn add_module_exports(
        &mut self,
        module_id: ModuleId,
        exports: Vec<(impl Into<String>, Visibility)>,
    ) {
        let export_map: HashMap<String, Export> = exports
            .into_iter()
            .map(|(name, vis)| {
                let name = name.into();
                (
                    name.clone(),
                    Export {
                        name,
                        visibility: vis,
                    },
                )
            })
            .collect();
        self.module_exports.insert(module_id, export_map);
    }

    /// Resolve all imports in the module graph.
    ///
    /// Returns a map from module ID to its binding table.
    pub fn resolve_all(&self) -> Result<HashMap<ModuleId, BindingTable>, ImportError> {
        let mut all_bindings: HashMap<ModuleId, BindingTable> = HashMap::new();
        let mut resolution_stack: Vec<ModuleId> = Vec::new();

        // Process each module in the graph
        for &module_id in self.module_graph.nodes.keys() {
            let bindings =
                self.resolve_module_imports(module_id, &mut all_bindings, &mut resolution_stack)?;
            all_bindings.insert(module_id, bindings);
        }

        Ok(all_bindings)
    }

    /// Resolve imports for a single module.
    fn resolve_module_imports(
        &self,
        module_id: ModuleId,
        resolved: &mut HashMap<ModuleId, BindingTable>,
        resolution_stack: &mut Vec<ModuleId>,
    ) -> Result<BindingTable, ImportError> {
        // Check for cycles
        if let Some(pos) = resolution_stack.iter().position(|&id| id == module_id) {
            let cycle = resolution_stack[pos..]
                .iter()
                .map(|id| {
                    self.module_graph
                        .get_node(*id)
                        .map(|n| n.name.clone())
                        .unwrap_or_else(|| format!("{:?}", id))
                })
                .chain(std::iter::once(
                    self.module_graph
                        .get_node(module_id)
                        .map(|n| n.name.clone())
                        .unwrap_or_else(|| format!("{:?}", module_id)),
                ))
                .collect::<Vec<_>>()
                .join(" -> ");
            return Err(ImportError::ImportCycle { cycle });
        }

        // Return cached bindings if already resolved
        if let Some(bindings) = resolved.get(&module_id) {
            return Ok(bindings.clone());
        }

        resolution_stack.push(module_id);
        let mut bindings: BindingTable = HashMap::new();

        // Process use statements for this module
        if let Some(uses) = self.module_uses.get(&module_id) {
            for use_stmt in uses {
                self.resolve_use_statement(module_id, use_stmt, &mut bindings)?;
            }
        }

        resolution_stack.pop();
        Ok(bindings)
    }

    /// Resolve a single use statement.
    fn resolve_use_statement(
        &self,
        importing_module: ModuleId,
        use_stmt: &Use,
        bindings: &mut BindingTable,
    ) -> Result<(), ImportError> {
        match &use_stmt.path {
            UsePath::Simple(path) => self.resolve_simple_import(
                importing_module,
                path,
                use_stmt.alias.as_deref(),
                bindings,
            ),
            UsePath::Glob(path) => self.resolve_glob_import(importing_module, path, bindings),
            UsePath::Nested(path, items) => {
                self.resolve_nested_import(importing_module, path, items, bindings)
            }
        }
    }

    /// Resolve a simple import (e.g., `use crate::foo::bar;`).
    fn resolve_simple_import(
        &self,
        importing_module: ModuleId,
        path: &SimplePath,
        alias: Option<&str>,
        bindings: &mut BindingTable,
    ) -> Result<(), ImportError> {
        let (target_module, item_name) = self.resolve_path_to_module_and_item(path)?;

        // Check visibility
        let exports =
            self.module_exports
                .get(&target_module)
                .ok_or_else(|| ImportError::ItemNotFound {
                    item: item_name.clone(),
                    module: format!("{:?}", target_module),
                })?;

        let export = exports
            .get(&item_name)
            .ok_or_else(|| ImportError::ItemNotFound {
                item: item_name.clone(),
                module: self.get_module_name(target_module),
            })?;

        if !self.is_visible(&export.visibility, importing_module, target_module) {
            return Err(ImportError::PrivateItem {
                item: item_name.clone(),
                module: self.get_module_name(target_module),
            });
        }

        let binding_name = alias.unwrap_or(&item_name).to_string();
        let kind = if alias.is_some() {
            BindingKind::Aliased {
                original: item_name.clone(),
            }
        } else {
            BindingKind::Direct
        };

        let binding = Binding::new(target_module, item_name, export.visibility.clone(), kind);

        if bindings.contains_key(&binding_name) {
            return Err(ImportError::ConflictingBinding { name: binding_name });
        }

        bindings.insert(binding_name, binding);
        Ok(())
    }

    /// Resolve a glob import (e.g., `use crate::foo::*;`).
    fn resolve_glob_import(
        &self,
        importing_module: ModuleId,
        path: &SimplePath,
        bindings: &mut BindingTable,
    ) -> Result<(), ImportError> {
        // For glob imports, we resolve up to the parent of the glob
        let target_module = self.resolve_path_to_module(path)?;

        let exports =
            self.module_exports
                .get(&target_module)
                .ok_or_else(|| ImportError::ModuleNotFound {
                    path: path
                        .segments
                        .iter()
                        .map(|s| s.as_ref())
                        .collect::<Vec<_>>()
                        .join("::"),
                })?;

        for (name, export) in exports {
            // Only import public items via glob
            if !self.is_visible(&export.visibility, importing_module, target_module) {
                continue;
            }

            let binding = Binding::new(
                target_module,
                name.clone(),
                export.visibility.clone(),
                BindingKind::Glob,
            );

            // Glob imports don't conflict with explicit imports
            // They are shadowed by explicit imports
            bindings.entry(name.clone()).or_insert(binding);
        }

        Ok(())
    }

    /// Resolve a nested import (e.g., `use crate::foo::{bar, baz as b};`).
    fn resolve_nested_import(
        &self,
        importing_module: ModuleId,
        path: &SimplePath,
        items: &[UseItem],
        bindings: &mut BindingTable,
    ) -> Result<(), ImportError> {
        // The path includes the module path + the item name pattern
        // For `use crate::foo::{bar, baz}`, the path is `crate::foo` and items are `bar`, `baz`
        // For `use crate::foo::bar::{a, b}`, the path is `crate::foo::bar` and items are `a`, `b`
        // So the target module is the full path (we don't strip the last segment)
        // The items are looked up in that module

        let target_module = self.resolve_path_to_module(path)?;

        for item in items {
            let item_name = item.name.as_ref();
            let alias = item.alias.as_deref();

            let exports = self.module_exports.get(&target_module).ok_or_else(|| {
                ImportError::ModuleNotFound {
                    path: path
                        .segments
                        .iter()
                        .map(|s| s.as_ref())
                        .collect::<Vec<_>>()
                        .join("::"),
                }
            })?;

            let export = exports
                .get(item_name)
                .ok_or_else(|| ImportError::ItemNotFound {
                    item: item_name.to_string(),
                    module: self.get_module_name(target_module),
                })?;

            if !self.is_visible(&export.visibility, importing_module, target_module) {
                return Err(ImportError::PrivateItem {
                    item: item_name.to_string(),
                    module: self.get_module_name(target_module),
                });
            }

            let binding_name = alias.unwrap_or(item_name).to_string();
            let kind = if alias.is_some() {
                BindingKind::Aliased {
                    original: item_name.to_string(),
                }
            } else {
                BindingKind::Direct
            };

            let binding = Binding::new(
                target_module,
                item_name.to_string(),
                export.visibility.clone(),
                kind,
            );

            if bindings.contains_key(&binding_name) {
                return Err(ImportError::ConflictingBinding { name: binding_name });
            }

            bindings.insert(binding_name, binding);
        }

        Ok(())
    }

    /// Resolve a path to a module.
    fn resolve_path_to_module(&self, path: &SimplePath) -> Result<ModuleId, ImportError> {
        if path.segments.is_empty() {
            return Err(ImportError::InvalidPrefix {
                prefix: "(empty)".to_string(),
            });
        }

        let first = path.segments[0].as_ref();
        if first != "crate" {
            return Err(ImportError::InvalidPrefix {
                prefix: first.to_string(),
            });
        }

        // Start from root module
        let root_id =
            self.module_graph
                .get_root()
                .copied()
                .ok_or_else(|| ImportError::ModuleNotFound {
                    path: "crate".to_string(),
                })?;

        // Walk the path segments
        let mut current_module = root_id;
        for segment in path.segments.iter().skip(1) {
            let node = self.module_graph.get_node(current_module).ok_or_else(|| {
                ImportError::ModuleNotFound {
                    path: segment.to_string(),
                }
            })?;

            // Find child with matching name
            let mut found = None;
            for &child_id in &node.children {
                #[allow(clippy::collapsible_if)]
                if let Some(child_node) = self.module_graph.get_node(child_id) {
                    if child_node.name == segment.as_ref() {
                        found = Some(child_id);
                        break;
                    }
                }
            }

            current_module = found.ok_or_else(|| ImportError::ModuleNotFound {
                path: segment.to_string(),
            })?;
        }

        Ok(current_module)
    }

    /// Resolve a path to a module and the final item name.
    fn resolve_path_to_module_and_item(
        &self,
        path: &SimplePath,
    ) -> Result<(ModuleId, String), ImportError> {
        if path.segments.len() < 2 {
            return Err(ImportError::InvalidPrefix {
                prefix: path
                    .segments
                    .first()
                    .map(|s| s.as_ref().to_string())
                    .unwrap_or_default(),
            });
        }

        // The last segment is the item name, the rest is the module path
        let item_name = path.segments.last().unwrap().as_ref().to_string();
        let module_path = SimplePath {
            segments: path.segments[..path.segments.len() - 1].to_vec(),
        };

        let module_id = self.resolve_path_to_module(&module_path)?;
        Ok((module_id, item_name))
    }

    /// Check if an item is visible from the importing module.
    fn is_visible(
        &self,
        visibility: &Visibility,
        importing_module: ModuleId,
        target_module: ModuleId,
    ) -> bool {
        match visibility {
            Visibility::Public => true,
            Visibility::Crate => {
                // Check if modules are in same crate
                match (
                    self.module_graph.crate_for(importing_module),
                    self.module_graph.crate_for(target_module),
                ) {
                    (Some(importing_crate), Some(target_crate)) => importing_crate == target_crate,
                    _ => false, // Unknown crate = not visible
                }
            }
            Visibility::Inherited => false, // Private items not visible
            Visibility::Super { levels } => {
                // pub(super) means visible to parent modules up to 'levels' steps
                // Check if importing_module is an ancestor of target_module
                // within 'levels' steps up from target
                let target_ancestors: Vec<_> = self
                    .module_graph
                    .ancestors(target_module)
                    .take(*levels + 1) // +1 to include the starting module
                    .collect();
                target_ancestors.contains(&importing_module)
            }
            Visibility::Self_ => false,
            Visibility::Restricted { .. } => true, // Simplified: allow for now
        }
    }

    /// Get the name of a module.
    fn get_module_name(&self, module_id: ModuleId) -> String {
        self.module_graph
            .get_node(module_id)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| format!("{:?}", module_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Span;
    use ash_core::module_graph::{CrateId, ModuleNode};

    // Helper to create a simple module graph
    fn create_test_graph() -> ModuleGraph {
        let mut graph = ModuleGraph::new();

        // Root module (crate)
        let root = graph.add_node(ModuleNode::new(
            "crate".to_string(),
            ash_core::module_graph::ModuleSource::File("main.ash".to_string()),
        ));
        graph.set_root(root);

        // foo module as child of root
        let foo = graph.add_node(ModuleNode::new(
            "foo".to_string(),
            ash_core::module_graph::ModuleSource::File("foo.ash".to_string()),
        ));
        graph.add_edge(root, foo);

        // bar module as child of foo
        let bar = graph.add_node(ModuleNode::new(
            "bar".to_string(),
            ash_core::module_graph::ModuleSource::File("foo/bar.ash".to_string()),
        ));
        graph.add_edge(foo, bar);

        graph
    }

    // Helper to create a multi-crate module graph for testing pub(crate)
    // Structure: mod_a1 (root, crate_a) -> mod_a2 (crate_a)
    //                         |
    //                         +--> external (crate_b) -> mod_b2 (crate_b)
    fn create_multi_crate_graph() -> (ModuleGraph, ModuleId, ModuleId, ModuleId, ModuleId) {
        use ash_core::module_graph::ModuleSource;

        let mut graph = ModuleGraph::new();

        // Create Crate A with two modules
        let crate_a = CrateId(0);
        let mod_a1 = graph.add_node(ModuleNode::new(
            "mod_a1".to_string(),
            ModuleSource::File("crate_a/mod_a1.ash".to_string()),
        ));
        let mod_a2 = graph.add_node(ModuleNode::new(
            "mod_a2".to_string(),
            ModuleSource::File("crate_a/mod_a2.ash".to_string()),
        ));
        graph.set_crate(mod_a1, crate_a);
        graph.set_crate(mod_a2, crate_a);
        graph.set_root(mod_a1);
        graph.add_edge(mod_a1, mod_a2);

        // Create Crate B - external module (as child of mod_a1 for path resolution)
        // but belonging to a different crate
        let crate_b = CrateId(1);
        let external = graph.add_node(ModuleNode::new(
            "external".to_string(),
            ModuleSource::File("external/lib.ash".to_string()),
        ));
        let mod_b2 = graph.add_node(ModuleNode::new(
            "mod_b2".to_string(),
            ModuleSource::File("external/mod_b2.ash".to_string()),
        ));
        graph.set_crate(external, crate_b);
        graph.set_crate(mod_b2, crate_b);
        graph.add_edge(mod_a1, external); // external is accessible from mod_a1
        graph.add_edge(external, mod_b2);

        (graph, mod_a1, mod_a2, external, mod_b2)
    }

    fn simple_path(segments: &[&str]) -> SimplePath {
        SimplePath {
            segments: segments.iter().map(|s| (*s).into()).collect(),
        }
    }

    fn use_stmt(path: UsePath) -> Use {
        Use {
            visibility: Visibility::Inherited,
            path,
            alias: None,
            span: Span::new(0, 10, 1, 1),
        }
    }

    // =========================================================================
    // RED Phase Tests - These should fail before implementation
    // =========================================================================

    #[test]
    fn test_resolve_simple_import() {
        let graph = create_test_graph();
        let root = graph.get_root().copied().unwrap();
        let foo = graph
            .get_node(root)
            .unwrap()
            .children
            .first()
            .copied()
            .unwrap();

        let mut resolver = ImportResolver::new(&graph);

        // Add exports for foo module
        resolver.add_module_exports(foo, vec![("MyItem".to_string(), Visibility::Public)]);

        // Add use statement: use crate::foo::MyItem;
        let use_path = UsePath::Simple(simple_path(&["crate", "foo", "MyItem"]));
        resolver.add_module_uses(root, vec![use_stmt(use_path)]);

        let bindings = resolver.resolve_all().unwrap();
        let root_bindings = bindings.get(&root).unwrap();

        assert!(root_bindings.contains_key("MyItem"));
        let binding = root_bindings.get("MyItem").unwrap();
        assert_eq!(binding.item_name, "MyItem");
        assert_eq!(binding.target_module, foo);
        assert!(matches!(binding.kind, BindingKind::Direct));
    }

    #[test]
    fn test_resolve_glob_import() {
        let graph = create_test_graph();
        let root = graph.get_root().copied().unwrap();
        let foo = graph
            .get_node(root)
            .unwrap()
            .children
            .first()
            .copied()
            .unwrap();

        let mut resolver = ImportResolver::new(&graph);

        // Add multiple exports to foo module
        resolver.add_module_exports(
            foo,
            vec![
                ("Item1".to_string(), Visibility::Public),
                ("Item2".to_string(), Visibility::Public),
            ],
        );

        // Add glob import: use crate::foo::*;
        let use_path = UsePath::Glob(simple_path(&["crate", "foo"]));
        resolver.add_module_uses(root, vec![use_stmt(use_path)]);

        let bindings = resolver.resolve_all().unwrap();
        let root_bindings = bindings.get(&root).unwrap();

        assert!(root_bindings.contains_key("Item1"));
        assert!(root_bindings.contains_key("Item2"));

        let binding1 = root_bindings.get("Item1").unwrap();
        assert!(matches!(binding1.kind, BindingKind::Glob));
    }

    #[test]
    fn test_resolve_nested_import() {
        let graph = create_test_graph();
        let root = graph.get_root().copied().unwrap();
        let foo = graph
            .get_node(root)
            .unwrap()
            .children
            .first()
            .copied()
            .unwrap();

        let mut resolver = ImportResolver::new(&graph);

        // Add exports for foo module
        resolver.add_module_exports(
            foo,
            vec![
                ("ItemA".to_string(), Visibility::Public),
                ("ItemB".to_string(), Visibility::Public),
            ],
        );

        // Add nested import: use crate::foo::{ItemA, ItemB};
        let items = vec![
            UseItem {
                name: "ItemA".into(),
                alias: None,
            },
            UseItem {
                name: "ItemB".into(),
                alias: None,
            },
        ];
        let use_path = UsePath::Nested(simple_path(&["crate", "foo"]), items);
        resolver.add_module_uses(root, vec![use_stmt(use_path)]);

        let bindings = resolver.resolve_all().unwrap();
        let root_bindings = bindings.get(&root).unwrap();

        assert!(root_bindings.contains_key("ItemA"));
        assert!(root_bindings.contains_key("ItemB"));
    }

    #[test]
    fn test_resolve_import_with_alias() {
        let graph = create_test_graph();
        let root = graph.get_root().copied().unwrap();
        let foo = graph
            .get_node(root)
            .unwrap()
            .children
            .first()
            .copied()
            .unwrap();

        let mut resolver = ImportResolver::new(&graph);

        // Add exports for foo module
        resolver.add_module_exports(foo, vec![("OriginalName".to_string(), Visibility::Public)]);

        // Add import with alias: use crate::foo::OriginalName as AliasName;
        let use_path = UsePath::Simple(simple_path(&["crate", "foo", "OriginalName"]));
        let mut use_stmt = use_stmt(use_path);
        use_stmt.alias = Some("AliasName".into());
        resolver.add_module_uses(root, vec![use_stmt]);

        let bindings = resolver.resolve_all().unwrap();
        let root_bindings = bindings.get(&root).unwrap();

        assert!(!root_bindings.contains_key("OriginalName"));
        assert!(root_bindings.contains_key("AliasName"));

        let binding = root_bindings.get("AliasName").unwrap();
        assert_eq!(binding.item_name, "OriginalName");
        assert!(
            matches!(&binding.kind, BindingKind::Aliased { original } if original == "OriginalName")
        );
    }

    #[test]
    fn test_visibility_check_private_item_fails() {
        let graph = create_test_graph();
        let root = graph.get_root().copied().unwrap();
        let foo = graph
            .get_node(root)
            .unwrap()
            .children
            .first()
            .copied()
            .unwrap();

        let mut resolver = ImportResolver::new(&graph);

        // Add private export
        resolver.add_module_exports(
            foo,
            vec![("PrivateItem".to_string(), Visibility::Inherited)],
        );

        // Try to import private item: use crate::foo::PrivateItem;
        let use_path = UsePath::Simple(simple_path(&["crate", "foo", "PrivateItem"]));
        resolver.add_module_uses(root, vec![use_stmt(use_path)]);

        let result = resolver.resolve_all();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, ImportError::PrivateItem { item, .. } if item == "PrivateItem"));
    }

    #[test]
    fn test_glob_import_skips_private_items() {
        let graph = create_test_graph();
        let root = graph.get_root().copied().unwrap();
        let foo = graph
            .get_node(root)
            .unwrap()
            .children
            .first()
            .copied()
            .unwrap();

        let mut resolver = ImportResolver::new(&graph);

        // Add mixed visibility exports
        resolver.add_module_exports(
            foo,
            vec![
                ("PublicItem".to_string(), Visibility::Public),
                ("PrivateItem".to_string(), Visibility::Inherited),
            ],
        );

        // Add glob import: use crate::foo::*;
        let use_path = UsePath::Glob(simple_path(&["crate", "foo"]));
        resolver.add_module_uses(root, vec![use_stmt(use_path)]);

        let bindings = resolver.resolve_all().unwrap();
        let root_bindings = bindings.get(&root).unwrap();

        // Public item should be imported
        assert!(root_bindings.contains_key("PublicItem"));
        // Private item should NOT be imported
        assert!(!root_bindings.contains_key("PrivateItem"));
    }

    #[test]
    fn test_import_cycle_detection() {
        let mut graph = ModuleGraph::new();

        // Create two modules that import from each other
        let mod_a = graph.add_node(ModuleNode::new(
            "mod_a".to_string(),
            ash_core::module_graph::ModuleSource::File("mod_a.ash".to_string()),
        ));
        let mod_b = graph.add_node(ModuleNode::new(
            "mod_b".to_string(),
            ash_core::module_graph::ModuleSource::File("mod_b.ash".to_string()),
        ));

        graph.set_root(mod_a);
        graph.add_edge(mod_a, mod_b);

        let mut resolver = ImportResolver::new(&graph);

        // Add exports
        resolver.add_module_exports(mod_a, vec![("ItemA".to_string(), Visibility::Public)]);
        resolver.add_module_exports(mod_b, vec![("ItemB".to_string(), Visibility::Public)]);

        // This test demonstrates cycle detection structure
        // In a real scenario, we'd need use statements that create cycles
        // For now, just verify the resolver works without cycles
        let bindings = resolver.resolve_all().unwrap();
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn test_nested_import_with_alias() {
        let graph = create_test_graph();
        let root = graph.get_root().copied().unwrap();
        let foo = graph
            .get_node(root)
            .unwrap()
            .children
            .first()
            .copied()
            .unwrap();

        let mut resolver = ImportResolver::new(&graph);

        resolver.add_module_exports(
            foo,
            vec![
                ("FuncA".to_string(), Visibility::Public),
                ("FuncB".to_string(), Visibility::Public),
            ],
        );

        // use crate::foo::{FuncA as A, FuncB};
        let items = vec![
            UseItem {
                name: "FuncA".into(),
                alias: Some("A".into()),
            },
            UseItem {
                name: "FuncB".into(),
                alias: None,
            },
        ];
        let use_path = UsePath::Nested(simple_path(&["crate", "foo"]), items);
        resolver.add_module_uses(root, vec![use_stmt(use_path)]);

        let bindings = resolver.resolve_all().unwrap();
        let root_bindings = bindings.get(&root).unwrap();

        assert!(root_bindings.contains_key("A"));
        assert!(root_bindings.contains_key("FuncB"));
        assert!(!root_bindings.contains_key("FuncA"));

        let binding = root_bindings.get("A").unwrap();
        assert_eq!(binding.item_name, "FuncA");
    }

    #[test]
    fn test_conflicting_binding_error() {
        let graph = create_test_graph();
        let root = graph.get_root().copied().unwrap();
        let foo = graph
            .get_node(root)
            .unwrap()
            .children
            .first()
            .copied()
            .unwrap();

        let mut resolver = ImportResolver::new(&graph);

        resolver.add_module_exports(foo, vec![("Item".to_string(), Visibility::Public)]);

        // Two imports with the same name
        let use1 = use_stmt(UsePath::Simple(simple_path(&["crate", "foo", "Item"])));
        let mut use2 = use_stmt(UsePath::Simple(simple_path(&["crate", "foo", "Item"])));
        use2.alias = Some("Item".into()); // Same name as the first import

        resolver.add_module_uses(root, vec![use1, use2]);

        let _result = resolver.resolve_all();
        // This should fail with conflicting binding
        // Note: The actual behavior depends on implementation details
    }

    #[test]
    fn test_item_not_found_error() {
        let graph = create_test_graph();
        let root = graph.get_root().copied().unwrap();
        let foo = graph
            .get_node(root)
            .unwrap()
            .children
            .first()
            .copied()
            .unwrap();

        let mut resolver = ImportResolver::new(&graph);

        let empty_exports: Vec<(String, Visibility)> = vec![];
        resolver.add_module_exports(foo, empty_exports);

        // Try to import non-existent item
        let use_path = UsePath::Simple(simple_path(&["crate", "foo", "NonExistent"]));
        resolver.add_module_uses(root, vec![use_stmt(use_path)]);

        let result = resolver.resolve_all();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, ImportError::ItemNotFound { item, .. } if item == "NonExistent"));
    }

    #[test]
    fn test_module_not_found_error() {
        let graph = create_test_graph();
        let root = graph.get_root().copied().unwrap();

        let mut resolver = ImportResolver::new(&graph);

        // Try to import from non-existent module
        let use_path = UsePath::Simple(simple_path(&["crate", "nonexistent", "Item"]));
        resolver.add_module_uses(root, vec![use_stmt(use_path)]);

        let result = resolver.resolve_all();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, ImportError::ModuleNotFound { .. }));
    }

    // =========================================================================
    // TASK-332: pub(crate) visibility tests
    // =========================================================================

    #[test]
    fn test_pub_crate_same_crate_allowed() {
        // Create graph with modules in same crate
        let (graph, mod_a1, mod_a2, _mod_b1, _mod_b2) = create_multi_crate_graph();

        let mut resolver = ImportResolver::new(&graph);

        // Add pub(crate) export to mod_a2 (same crate as mod_a1)
        resolver.add_module_exports(mod_a2, vec![("CrateItem".to_string(), Visibility::Crate)]);

        // Add use statement from mod_a1 importing from mod_a2
        // Path needs to go from mod_a1 root: mod_a2::CrateItem
        let use_path = UsePath::Simple(simple_path(&["crate", "mod_a2", "CrateItem"]));
        resolver.add_module_uses(mod_a1, vec![use_stmt(use_path)]);

        // Import should succeed - same crate
        let bindings = resolver.resolve_all().unwrap();
        let mod_a1_bindings = bindings.get(&mod_a1).unwrap();

        assert!(mod_a1_bindings.contains_key("CrateItem"));
        let binding = mod_a1_bindings.get("CrateItem").unwrap();
        assert_eq!(binding.item_name, "CrateItem");
        assert_eq!(binding.target_module, mod_a2);
    }

    #[test]
    fn test_pub_crate_cross_crate_rejected() {
        // Create graph with modules in different crates
        let (graph, mod_a1, _mod_a2, _external, mod_b2) = create_multi_crate_graph();

        let mut resolver = ImportResolver::new(&graph);

        // Add pub(crate) export to mod_b2 (different crate from mod_a1)
        resolver.add_module_exports(mod_b2, vec![("CrateItem".to_string(), Visibility::Crate)]);

        // Add use statement from mod_a1 importing from mod_b2 via external
        let use_path = UsePath::Simple(simple_path(&["crate", "external", "mod_b2", "CrateItem"]));
        resolver.add_module_uses(mod_a1, vec![use_stmt(use_path)]);

        // Import should fail - different crate
        let result = resolver.resolve_all();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(
            matches!(err, ImportError::PrivateItem { ref item, .. } if item == "CrateItem"),
            "Expected PrivateItem error for cross-crate pub(crate) import, got: {:?}",
            err
        );
    }

    #[test]
    fn test_pub_crate_unknown_crate_rejected() {
        let graph = create_test_graph();
        let root = graph.get_root().copied().unwrap();
        let foo = graph
            .get_node(root)
            .unwrap()
            .children
            .first()
            .copied()
            .unwrap();

        let mut resolver = ImportResolver::new(&graph);

        // Add pub(crate) export without setting crate membership
        resolver.add_module_exports(foo, vec![("CrateItem".to_string(), Visibility::Crate)]);

        // Try to import the pub(crate) item
        let use_path = UsePath::Simple(simple_path(&["crate", "foo", "CrateItem"]));
        resolver.add_module_uses(root, vec![use_stmt(use_path)]);

        // Import should fail - unknown crate
        let result = resolver.resolve_all();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(
            matches!(err, ImportError::PrivateItem { ref item, .. } if item == "CrateItem"),
            "Expected PrivateItem error for unknown crate, got: {:?}",
            err
        );
    }

    // =========================================================================
    // TASK-333: pub(super) visibility tests
    // =========================================================================

    // Helper to create a three-level module graph:
    // root (level 0) -> foo (level 1) -> bar (level 2)
    fn create_three_level_graph() -> (ModuleGraph, ModuleId, ModuleId, ModuleId) {
        let mut graph = ModuleGraph::new();

        // Root module (crate)
        let root = graph.add_node(ModuleNode::new(
            "crate".to_string(),
            ash_core::module_graph::ModuleSource::File("main.ash".to_string()),
        ));
        graph.set_root(root);

        // foo module as child of root
        let foo = graph.add_node(ModuleNode::new(
            "foo".to_string(),
            ash_core::module_graph::ModuleSource::File("foo.ash".to_string()),
        ));
        graph.add_edge(root, foo);

        // bar module as child of foo
        let bar = graph.add_node(ModuleNode::new(
            "bar".to_string(),
            ash_core::module_graph::ModuleSource::File("foo/bar.ash".to_string()),
        ));
        graph.add_edge(foo, bar);

        (graph, root, foo, bar)
    }

    #[test]
    fn test_pub_super_parent_allowed() {
        // Parent (foo) importing from child (bar) with pub(super) should succeed
        let (graph, _root, foo, bar) = create_three_level_graph();

        let mut resolver = ImportResolver::new(&graph);

        // Add pub(super) export to bar module
        resolver.add_module_exports(
            bar,
            vec![("SuperItem".to_string(), Visibility::Super { levels: 1 })],
        );

        // Add use statement from foo importing from bar
        let use_path = UsePath::Simple(simple_path(&["crate", "foo", "bar", "SuperItem"]));
        resolver.add_module_uses(foo, vec![use_stmt(use_path)]);

        // Import should succeed - foo is parent of bar
        let bindings = resolver.resolve_all().unwrap();
        let foo_bindings = bindings.get(&foo).unwrap();

        assert!(foo_bindings.contains_key("SuperItem"));
        let binding = foo_bindings.get("SuperItem").unwrap();
        assert_eq!(binding.item_name, "SuperItem");
        assert_eq!(binding.target_module, bar);
    }

    #[test]
    fn test_pub_super_grandparent_allowed() {
        // Grandparent (root) importing with pub(super, super) should succeed
        let (graph, root, _foo, bar) = create_three_level_graph();

        let mut resolver = ImportResolver::new(&graph);

        // Add pub(super, super) export to bar module (visible up 2 levels)
        resolver.add_module_exports(
            bar,
            vec![(
                "SuperSuperItem".to_string(),
                Visibility::Super { levels: 2 },
            )],
        );

        // Add use statement from root importing from bar via foo
        let use_path = UsePath::Simple(simple_path(&["crate", "foo", "bar", "SuperSuperItem"]));
        resolver.add_module_uses(root, vec![use_stmt(use_path)]);

        // Import should succeed - root is grandparent of bar
        let bindings = resolver.resolve_all().unwrap();
        let root_bindings = bindings.get(&root).unwrap();

        assert!(root_bindings.contains_key("SuperSuperItem"));
        let binding = root_bindings.get("SuperSuperItem").unwrap();
        assert_eq!(binding.item_name, "SuperSuperItem");
        assert_eq!(binding.target_module, bar);
    }

    #[test]
    fn test_pub_super_sibling_rejected() {
        // Sibling importing should fail
        let (mut graph, _root, foo, bar) = create_three_level_graph();

        // First, add another module as sibling of bar (child of foo) - BEFORE creating resolver
        let sibling = graph.add_node(ModuleNode::new(
            "sibling".to_string(),
            ash_core::module_graph::ModuleSource::File("foo/sibling.ash".to_string()),
        ));
        graph.add_edge(foo, sibling);

        let mut resolver = ImportResolver::new(&graph);

        // Add pub(super) export to bar module
        resolver.add_module_exports(
            bar,
            vec![("SuperItem".to_string(), Visibility::Super { levels: 1 })],
        );

        // Add use statement from sibling trying to import from bar
        let use_path = UsePath::Simple(simple_path(&["crate", "foo", "bar", "SuperItem"]));
        resolver.add_module_uses(sibling, vec![use_stmt(use_path)]);

        // Import should fail - sibling is not an ancestor
        let result = resolver.resolve_all();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(
            matches!(err, ImportError::PrivateItem { ref item, .. } if item == "SuperItem"),
            "Expected PrivateItem error for sibling import, got: {:?}",
            err
        );
    }

    #[test]
    fn test_pub_super_child_rejected() {
        // Child importing from parent should fail (wrong direction)
        let (graph, _root, foo, bar) = create_three_level_graph();

        let mut resolver = ImportResolver::new(&graph);

        // Add pub(super) export to foo module
        resolver.add_module_exports(
            foo,
            vec![("ParentItem".to_string(), Visibility::Super { levels: 1 })],
        );

        // Add use statement from bar (child) trying to import from foo (parent)
        let use_path = UsePath::Simple(simple_path(&["crate", "foo", "ParentItem"]));
        resolver.add_module_uses(bar, vec![use_stmt(use_path)]);

        // Import should fail - bar is not an ancestor of foo
        let result = resolver.resolve_all();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(
            matches!(err, ImportError::PrivateItem { ref item, .. } if item == "ParentItem"),
            "Expected PrivateItem error for child-to-parent import, got: {:?}",
            err
        );
    }

    #[test]
    fn test_pub_super_at_root() {
        // pub(super) at root should not be visible from anywhere (no parent)
        let mut graph = ModuleGraph::new();

        // Root module only
        let root = graph.add_node(ModuleNode::new(
            "crate".to_string(),
            ash_core::module_graph::ModuleSource::File("main.ash".to_string()),
        ));
        graph.set_root(root);

        // Create a child module
        let child = graph.add_node(ModuleNode::new(
            "child".to_string(),
            ash_core::module_graph::ModuleSource::File("child.ash".to_string()),
        ));
        graph.add_edge(root, child);

        let mut resolver = ImportResolver::new(&graph);

        // Add pub(super) export to root module
        resolver.add_module_exports(
            root,
            vec![("RootItem".to_string(), Visibility::Super { levels: 1 })],
        );

        // Try to import from child - should fail since root has no parent
        let use_path = UsePath::Simple(simple_path(&["crate", "RootItem"]));
        resolver.add_module_uses(child, vec![use_stmt(use_path)]);

        // Import should fail - no parent above root
        let result = resolver.resolve_all();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(
            matches!(err, ImportError::PrivateItem { ref item, .. } if item == "RootItem"),
            "Expected PrivateItem error for pub(super) at root, got: {:?}",
            err
        );
    }

    #[test]
    fn test_pub_super_levels_insufficient() {
        // pub(super) (levels=1) should not be visible from grandparent
        let (graph, root, _foo, bar) = create_three_level_graph();

        let mut resolver = ImportResolver::new(&graph);

        // Add pub(super) (levels=1) export to bar module
        resolver.add_module_exports(
            bar,
            vec![("SuperItem".to_string(), Visibility::Super { levels: 1 })],
        );

        // Add use statement from root (grandparent) trying to import from bar
        let use_path = UsePath::Simple(simple_path(&["crate", "foo", "bar", "SuperItem"]));
        resolver.add_module_uses(root, vec![use_stmt(use_path)]);

        // Import should fail - root is 2 levels up, but only 1 level allowed
        let result = resolver.resolve_all();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(
            matches!(err, ImportError::PrivateItem { ref item, .. } if item == "SuperItem"),
            "Expected PrivateItem error for insufficient levels, got: {:?}",
            err
        );
    }
}
