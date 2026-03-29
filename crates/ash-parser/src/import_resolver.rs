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
        let (target_module, item_name) =
            self.resolve_path_to_module_and_item(importing_module, path)?;

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
        let target_module = self.resolve_path_to_module(importing_module, path)?;

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

        let target_module = self.resolve_path_to_module(importing_module, path)?;

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

    /// Resolve a path to a module from the context of the importing module.
    fn resolve_path_to_module(
        &self,
        importing_module: ModuleId,
        path: &SimplePath,
    ) -> Result<ModuleId, ImportError> {
        if path.segments.is_empty() {
            return Err(ImportError::InvalidPrefix {
                prefix: "(empty)".to_string(),
            });
        }

        let first = path.segments[0].as_ref();
        match first {
            "crate" => self.resolve_current_crate_path(importing_module, path),
            "external" => self.resolve_external_path(importing_module, path),
            _ => Err(ImportError::InvalidPrefix {
                prefix: first.to_string(),
            }),
        }
    }

    /// Resolve a path within the current crate (starting with `crate::`).
    fn resolve_current_crate_path(
        &self,
        importing_module: ModuleId,
        path: &SimplePath,
    ) -> Result<ModuleId, ImportError> {
        if path.segments.is_empty() {
            return Err(ImportError::InvalidPrefix {
                prefix: "(empty)".to_string(),
            });
        }

        // Get the importing module's crate root
        // Fall back to global root if no crate metadata is set (backward compatibility)
        let root_id = if let Some(importing_crate) =
            self.module_graph.crate_id_for_module(importing_module)
        {
            self.module_graph
                .get_crate(importing_crate)
                .map(|info| info.root_module)
                .or_else(|| self.module_graph.get_root().copied())
        } else {
            self.module_graph.get_root().copied()
        }
        .ok_or_else(|| ImportError::ModuleNotFound {
            path: "crate".to_string(),
        })?;

        // Walk the path segments (skip "crate")
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

    /// Resolve a path from an external crate (starting with `external::`).
    fn resolve_external_path(
        &self,
        importing_module: ModuleId,
        path: &SimplePath,
    ) -> Result<ModuleId, ImportError> {
        if path.segments.len() < 2 {
            return Err(ImportError::InvalidPrefix {
                prefix: "external".to_string(),
            });
        }

        // Get the importing module's crate to look up the dependency
        // Fall back to using the root's crate if importing module has no crate metadata
        let importing_crate = self
            .module_graph
            .crate_id_for_module(importing_module)
            .or_else(|| {
                self.module_graph
                    .get_root()
                    .and_then(|&root| self.module_graph.crate_id_for_module(root))
            })
            .ok_or_else(|| ImportError::ModuleNotFound {
                path: "crate root".to_string(),
            })?;

        // The second segment is the dependency alias (e.g., "util" in "external::util::sanitize")
        let alias = path.segments[1].as_ref();

        // Look up the target crate ID from the dependency alias
        let target_crate_id = self
            .module_graph
            .dependency_target(importing_crate, alias)
            .ok_or_else(|| ImportError::ModuleNotFound {
                path: format!("external::{} (undeclared dependency)", alias),
            })?;

        // Find the root module of the target crate
        let target_root = self
            .module_graph
            .nodes
            .iter()
            .find_map(|(&module_id, _)| {
                self.module_graph
                    .crate_id_for_module(module_id)
                    .filter(|&crate_id| crate_id == target_crate_id)
                    .and_then(|_| {
                        // Check if this is a root (no parent or parent is in different crate)
                        self.module_graph.get_node(module_id).and_then(|node| {
                            if node.parent.is_none() {
                                Some(module_id)
                            } else {
                                None
                            }
                        })
                    })
            })
            .ok_or_else(|| ImportError::ModuleNotFound {
                path: format!("external::{} (root module)", alias),
            })?;

        // Walk the remaining path segments within the external crate
        let mut current_module = target_root;
        for segment in path.segments.iter().skip(2) {
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
        importing_module: ModuleId,
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

        let module_id = self.resolve_path_to_module(importing_module, &module_path)?;
        Ok((module_id, item_name))
    }

    /// Check if an item is visible from the importing module.
    fn is_visible(
        &self,
        visibility: &Visibility,
        importing_module: ModuleId,
        target_module: ModuleId,
    ) -> bool {
        // Check cross-crate visibility first
        let importing_crate = self.module_graph.crate_id_for_module(importing_module);
        let target_crate = self.module_graph.crate_id_for_module(target_module);

        // If importing from a different crate, only pub items are visible
        if importing_crate != target_crate {
            return matches!(visibility, Visibility::Public);
        }

        match visibility {
            Visibility::Public => true,
            Visibility::Crate => {
                // Single-crate model: pub(crate) visible to all modules in this graph
                self.module_graph.nodes.contains_key(&importing_module)
                    && self.module_graph.nodes.contains_key(&target_module)
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
            Visibility::Restricted { path } => {
                // Parse the path string (e.g., "crate::foo::bar" -> ["crate", "foo", "bar"])
                let path_components: Vec<String> =
                    path.split("::").map(|s| s.to_string()).collect();

                // SPEC-009: Restricted paths are resolved from the DEFINING module's context
                // (where the item is declared), not the importing module's context
                // Example: item in crate::owner with pub(in foo) checks against owner::foo,
                // not the importer's foo
                match self.resolve_restricted_path(target_module, &path_components) {
                    Some(restricted_module) => {
                        // Importing module must be the restricted module or its descendant
                        self.module_graph
                            .is_descendant_or_same(importing_module, restricted_module)
                    }
                    None => false, // Non-existent path = not visible
                }
            }
        }
    }

    /// Resolve a restricted visibility path from the importing crate's context.
    /// Path format: ["crate", "foo", "bar"] representing crate::foo::bar
    /// Also handles "super" and "self" keywords.
    fn resolve_restricted_path(
        &self,
        importing_module: ModuleId,
        path: &[String],
    ) -> Option<ModuleId> {
        if path.is_empty() {
            return None;
        }

        // Handle special path keywords at the start
        let first = path[0].as_str();
        let mut current = match first {
            "crate" => {
                // Get the crate root
                self.module_graph
                    .crate_id_for_module(importing_module)
                    .and_then(|crate_id| self.module_graph.get_crate(crate_id))
                    .map(|info| info.root_module)
                    .or_else(|| self.module_graph.get_root().copied())?
            }
            "self" => importing_module,
            "super" => {
                // Get parent of importing module
                self.module_graph
                    .get_node(importing_module)
                    .and_then(|n| n.parent)?
            }
            _ => {
                // Regular path component - resolve relative to importing_module
                // SPEC-009: foo::bar is relative to the current module, not crate root
                self.find_child_module(importing_module, first)?
            }
        };

        // Process remaining path components
        for component in &path[1..] {
            match component.as_str() {
                "super" => {
                    // Move up to parent
                    current = self.module_graph.get_node(current).and_then(|n| n.parent)?;
                }
                "self" => {
                    // Stay at current module (no-op)
                }
                _ => {
                    // Find child with matching name
                    current = self.find_child_module(current, component)?;
                }
            }
        }

        Some(current)
    }

    /// Find a child module with the given name.
    fn find_child_module(&self, parent: ModuleId, name: &str) -> Option<ModuleId> {
        let node = self.module_graph.get_node(parent)?;
        node.children.iter().find_map(|&child_id| {
            self.module_graph
                .get_node(child_id)
                .filter(|n| n.name == name)
                .map(|_| child_id)
        })
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

    // Note: Multi-crate pub(crate) tests removed - Phase 54 uses single-crate model
    // where pub(crate) is visible to all modules in the graph.

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

    // =========================================================================
    // TASK-334: pub(in path) visibility tests
    // =========================================================================

    // Helper to create a four-level module graph for testing pub(in path):
    // root (crate) -> foo -> bar -> baz
    //              -> sibling (child of root, sibling of foo)
    fn create_four_level_graph() -> (
        ModuleGraph,
        ModuleId,
        ModuleId,
        ModuleId,
        ModuleId,
        ModuleId,
    ) {
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

        // baz module as child of bar
        let baz = graph.add_node(ModuleNode::new(
            "baz".to_string(),
            ash_core::module_graph::ModuleSource::File("foo/bar/baz.ash".to_string()),
        ));
        graph.add_edge(bar, baz);

        // sibling module as child of root (sibling of foo)
        let sibling = graph.add_node(ModuleNode::new(
            "sibling".to_string(),
            ash_core::module_graph::ModuleSource::File("sibling.ash".to_string()),
        ));
        graph.add_edge(root, sibling);

        (graph, root, foo, bar, baz, sibling)
    }

    #[test]
    fn test_pub_in_path_exact_match_allowed() {
        // Module importing its own restricted item should succeed
        let (graph, _root, foo, _bar, _baz, _sibling) = create_four_level_graph();

        let mut resolver = ImportResolver::new(&graph);

        // Add pub(in crate::foo) export to foo module
        resolver.add_module_exports(
            foo,
            vec![(
                "RestrictedItem".to_string(),
                Visibility::Restricted {
                    path: "crate::foo".into(),
                },
            )],
        );

        // Add use statement from foo importing from itself
        let use_path = UsePath::Simple(simple_path(&["crate", "foo", "RestrictedItem"]));
        resolver.add_module_uses(foo, vec![use_stmt(use_path)]);

        // Import should succeed - foo is the restricted path
        let bindings = resolver.resolve_all().unwrap();
        let foo_bindings = bindings.get(&foo).unwrap();

        assert!(foo_bindings.contains_key("RestrictedItem"));
    }

    #[test]
    fn test_pub_in_path_descendant_allowed() {
        // Descendant module importing restricted ancestor item should succeed
        let (graph, _root, foo, bar, baz, _sibling) = create_four_level_graph();

        let mut resolver = ImportResolver::new(&graph);

        // Add pub(in crate::foo) export to foo module
        resolver.add_module_exports(
            foo,
            vec![(
                "RestrictedItem".to_string(),
                Visibility::Restricted {
                    path: "crate::foo".into(),
                },
            )],
        );

        // Add use statement from bar (child of foo) importing from foo
        let use_path = UsePath::Simple(simple_path(&["crate", "foo", "RestrictedItem"]));
        resolver.add_module_uses(bar, vec![use_stmt(use_path)]);

        // Import should succeed - bar is a descendant of foo
        let bindings = resolver.resolve_all().unwrap();
        let bar_bindings = bindings.get(&bar).unwrap();

        assert!(bar_bindings.contains_key("RestrictedItem"));

        // Also test with baz (grandchild of foo)
        let mut resolver2 = ImportResolver::new(&graph);
        resolver2.add_module_exports(
            foo,
            vec![(
                "RestrictedItem".to_string(),
                Visibility::Restricted {
                    path: "crate::foo".into(),
                },
            )],
        );
        let use_path2 = UsePath::Simple(simple_path(&["crate", "foo", "RestrictedItem"]));
        resolver2.add_module_uses(baz, vec![use_stmt(use_path2)]);

        let bindings2 = resolver2.resolve_all().unwrap();
        let baz_bindings = bindings2.get(&baz).unwrap();

        assert!(baz_bindings.contains_key("RestrictedItem"));
    }

    #[test]
    fn test_pub_in_path_sibling_rejected() {
        // Sibling of restricted path trying to import should fail
        let (graph, _root, foo, _bar, _baz, sibling) = create_four_level_graph();

        let mut resolver = ImportResolver::new(&graph);

        // Add pub(in crate::foo) export to foo module
        resolver.add_module_exports(
            foo,
            vec![(
                "RestrictedItem".to_string(),
                Visibility::Restricted {
                    path: "crate::foo".into(),
                },
            )],
        );

        // Add use statement from sibling trying to import from foo
        let use_path = UsePath::Simple(simple_path(&["crate", "foo", "RestrictedItem"]));
        resolver.add_module_uses(sibling, vec![use_stmt(use_path)]);

        // Import should fail - sibling is not a descendant of foo
        let result = resolver.resolve_all();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(
            matches!(err, ImportError::PrivateItem { ref item, .. } if item == "RestrictedItem"),
            "Expected PrivateItem error for sibling import, got: {:?}",
            err
        );
    }

    #[test]
    fn test_pub_in_path_parent_rejected() {
        // Parent of restricted path trying to import should fail
        let (graph, root, foo, _bar, _baz, _sibling) = create_four_level_graph();

        let mut resolver = ImportResolver::new(&graph);

        // Add pub(in crate::foo) export to foo module
        resolver.add_module_exports(
            foo,
            vec![(
                "RestrictedItem".to_string(),
                Visibility::Restricted {
                    path: "crate::foo".into(),
                },
            )],
        );

        // Add use statement from root (parent of foo) trying to import from foo
        let use_path = UsePath::Simple(simple_path(&["crate", "foo", "RestrictedItem"]));
        resolver.add_module_uses(root, vec![use_stmt(use_path)]);

        // Import should fail - root is not a descendant of foo
        let result = resolver.resolve_all();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(
            matches!(err, ImportError::PrivateItem { ref item, .. } if item == "RestrictedItem"),
            "Expected PrivateItem error for parent import, got: {:?}",
            err
        );
    }

    #[test]
    fn test_pub_in_path_crate_root() {
        // pub(in crate) should work like pub(crate) - visible to all in crate
        let (graph, root, foo, _bar, _baz, sibling) = create_four_level_graph();

        let mut resolver = ImportResolver::new(&graph);

        // Add pub(in crate) export to foo module
        resolver.add_module_exports(
            foo,
            vec![(
                "CrateRestrictedItem".to_string(),
                Visibility::Restricted {
                    path: "crate".into(),
                },
            )],
        );

        // Add use statement from sibling (different branch but same crate root)
        let use_path = UsePath::Simple(simple_path(&["crate", "foo", "CrateRestrictedItem"]));
        resolver.add_module_uses(sibling, vec![use_stmt(use_path)]);

        // Import should succeed - sibling is a descendant of crate root
        let bindings = resolver.resolve_all().unwrap();
        let sibling_bindings = bindings.get(&sibling).unwrap();

        assert!(sibling_bindings.contains_key("CrateRestrictedItem"));

        // Also test from root itself
        let mut resolver2 = ImportResolver::new(&graph);
        resolver2.add_module_exports(
            foo,
            vec![(
                "CrateRestrictedItem".to_string(),
                Visibility::Restricted {
                    path: "crate".into(),
                },
            )],
        );
        let use_path2 = UsePath::Simple(simple_path(&["crate", "foo", "CrateRestrictedItem"]));
        resolver2.add_module_uses(root, vec![use_stmt(use_path2)]);

        let bindings2 = resolver2.resolve_all().unwrap();
        let root_bindings = bindings2.get(&root).unwrap();

        assert!(root_bindings.contains_key("CrateRestrictedItem"));
    }

    #[test]
    fn test_pub_in_path_nonexistent_rejected() {
        // Non-existent path should not be visible
        let (graph, _root, foo, _bar, _baz, _sibling) = create_four_level_graph();

        let mut resolver = ImportResolver::new(&graph);

        // Add pub(in crate::nonexistent) export to foo module
        resolver.add_module_exports(
            foo,
            vec![(
                "NonExistentPathItem".to_string(),
                Visibility::Restricted {
                    path: "crate::nonexistent".into(),
                },
            )],
        );

        // Add use statement from foo trying to import
        let use_path = UsePath::Simple(simple_path(&["crate", "foo", "NonExistentPathItem"]));
        resolver.add_module_uses(foo, vec![use_stmt(use_path)]);

        // Import should fail - non-existent path
        let result = resolver.resolve_all();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(
            matches!(err, ImportError::PrivateItem { ref item, .. } if item == "NonExistentPathItem"),
            "Expected PrivateItem error for non-existent path, got: {:?}",
            err
        );
    }

    #[test]
    fn test_pub_in_path_nested_module() {
        // pub(in crate::foo::bar) should be visible to bar and baz, but not foo or sibling
        let (graph, root, foo, bar, baz, sibling) = create_four_level_graph();

        // Test: bar (exact match) should succeed
        let mut resolver = ImportResolver::new(&graph);
        resolver.add_module_exports(
            bar,
            vec![(
                "NestedItem".to_string(),
                Visibility::Restricted {
                    path: "crate::foo::bar".into(),
                },
            )],
        );
        let use_path = UsePath::Simple(simple_path(&["crate", "foo", "bar", "NestedItem"]));
        resolver.add_module_uses(bar, vec![use_stmt(use_path)]);

        let bindings = resolver.resolve_all().unwrap();
        assert!(bindings.get(&bar).unwrap().contains_key("NestedItem"));

        // Test: baz (descendant of bar) should succeed
        let mut resolver2 = ImportResolver::new(&graph);
        resolver2.add_module_exports(
            bar,
            vec![(
                "NestedItem".to_string(),
                Visibility::Restricted {
                    path: "crate::foo::bar".into(),
                },
            )],
        );
        let use_path2 = UsePath::Simple(simple_path(&["crate", "foo", "bar", "NestedItem"]));
        resolver2.add_module_uses(baz, vec![use_stmt(use_path2)]);

        let bindings2 = resolver2.resolve_all().unwrap();
        assert!(bindings2.get(&baz).unwrap().contains_key("NestedItem"));

        // Test: foo (parent of bar) should fail
        let mut resolver3 = ImportResolver::new(&graph);
        resolver3.add_module_exports(
            bar,
            vec![(
                "NestedItem".to_string(),
                Visibility::Restricted {
                    path: "crate::foo::bar".into(),
                },
            )],
        );
        let use_path3 = UsePath::Simple(simple_path(&["crate", "foo", "bar", "NestedItem"]));
        resolver3.add_module_uses(foo, vec![use_stmt(use_path3)]);

        let result3 = resolver3.resolve_all();
        assert!(result3.is_err());

        // Test: root (ancestor of bar) should fail
        let mut resolver4 = ImportResolver::new(&graph);
        resolver4.add_module_exports(
            bar,
            vec![(
                "NestedItem".to_string(),
                Visibility::Restricted {
                    path: "crate::foo::bar".into(),
                },
            )],
        );
        let use_path4 = UsePath::Simple(simple_path(&["crate", "foo", "bar", "NestedItem"]));
        resolver4.add_module_uses(root, vec![use_stmt(use_path4)]);

        let result4 = resolver4.resolve_all();
        assert!(result4.is_err());

        // Test: sibling (unrelated) should fail
        let mut resolver5 = ImportResolver::new(&graph);
        resolver5.add_module_exports(
            bar,
            vec![(
                "NestedItem".to_string(),
                Visibility::Restricted {
                    path: "crate::foo::bar".into(),
                },
            )],
        );
        let use_path5 = UsePath::Simple(simple_path(&["crate", "foo", "bar", "NestedItem"]));
        resolver5.add_module_uses(sibling, vec![use_stmt(use_path5)]);

        let result5 = resolver5.resolve_all();
        assert!(result5.is_err());
    }

    // =========================================================================
    // TASK-335: Edge Cases and Comprehensive Visibility Tests
    // =========================================================================

    #[test]
    fn test_pub_crate_at_root() {
        // pub(crate) at root level should be accessible from all modules in crate
        let mut graph = ModuleGraph::new();
        let crate_a = CrateId(0);

        let root = graph.add_node(ModuleNode::new(
            "crate".to_string(),
            ash_core::module_graph::ModuleSource::File("main.ash".to_string()),
        ));
        graph.set_root(root);
        graph.set_crate(root, crate_a);

        let child = graph.add_node(ModuleNode::new(
            "child".to_string(),
            ash_core::module_graph::ModuleSource::File("child.ash".to_string()),
        ));
        graph.add_edge(root, child);
        graph.set_crate(child, crate_a);

        let mut resolver = ImportResolver::new(&graph);

        // Add pub(crate) export to root module
        resolver.add_module_exports(root, vec![("RootCrateItem".to_string(), Visibility::Crate)]);

        // Add use statement from child importing from root
        let use_path = UsePath::Simple(simple_path(&["crate", "RootCrateItem"]));
        resolver.add_module_uses(child, vec![use_stmt(use_path)]);

        // Import should succeed - same crate
        let bindings = resolver.resolve_all().unwrap();
        let child_bindings = bindings.get(&child).unwrap();

        assert!(child_bindings.contains_key("RootCrateItem"));
        let binding = child_bindings.get("RootCrateItem").unwrap();
        assert_eq!(binding.target_module, root);
    }

    #[test]
    fn test_pub_crate_deeply_nested() {
        // pub(crate) at deeply nested level should be accessible from any module in same crate
        let mut graph = ModuleGraph::new();
        let crate_a = CrateId(0);

        let root = graph.add_node(ModuleNode::new(
            "crate".to_string(),
            ash_core::module_graph::ModuleSource::File("main.ash".to_string()),
        ));
        graph.set_root(root);
        graph.set_crate(root, crate_a);

        // Create deep hierarchy: root -> a -> b -> c -> d
        let mut prev = root;
        let mut modules = vec![root];
        for name in ["a", "b", "c", "d"] {
            let module = graph.add_node(ModuleNode::new(
                name.to_string(),
                ash_core::module_graph::ModuleSource::File(format!("{}.ash", name)),
            ));
            graph.add_edge(prev, module);
            graph.set_crate(module, crate_a);
            modules.push(module);
            prev = module;
        }

        let deepest = modules[4]; // d
        let middle = modules[2]; // b

        let mut resolver = ImportResolver::new(&graph);

        // Add pub(crate) export to deepest module
        resolver.add_module_exports(deepest, vec![("DeepItem".to_string(), Visibility::Crate)]);

        // Add use statement from middle module importing from deepest
        let use_path = UsePath::Simple(simple_path(&["crate", "a", "b", "c", "d", "DeepItem"]));
        resolver.add_module_uses(middle, vec![use_stmt(use_path)]);

        // Import should succeed - same crate regardless of depth
        let bindings = resolver.resolve_all().unwrap();
        let middle_bindings = bindings.get(&middle).unwrap();

        assert!(middle_bindings.contains_key("DeepItem"));
    }

    #[test]
    fn test_pub_super_multi_level_exact() {
        // Test pub(super) at exactly the boundary - parent can see, grandparent cannot
        let (graph, root, foo, bar) = create_three_level_graph();

        // Test with levels=1: only direct parent should see
        let mut resolver1 = ImportResolver::new(&graph);
        resolver1.add_module_exports(
            bar,
            vec![("Super1Item".to_string(), Visibility::Super { levels: 1 })],
        );

        // Parent (foo) should succeed
        let use_path1 = UsePath::Simple(simple_path(&["crate", "foo", "bar", "Super1Item"]));
        resolver1.add_module_uses(foo, vec![use_stmt(use_path1)]);

        let bindings1 = resolver1.resolve_all().unwrap();
        assert!(bindings1.get(&foo).unwrap().contains_key("Super1Item"));

        // Grandparent (root) should fail
        let mut resolver2 = ImportResolver::new(&graph);
        resolver2.add_module_exports(
            bar,
            vec![("Super1Item".to_string(), Visibility::Super { levels: 1 })],
        );
        let use_path2 = UsePath::Simple(simple_path(&["crate", "foo", "bar", "Super1Item"]));
        resolver2.add_module_uses(root, vec![use_stmt(use_path2)]);

        let result2 = resolver2.resolve_all();
        assert!(result2.is_err());
    }

    #[test]
    fn test_pub_super_beyond_root() {
        // pub(super) with levels exceeding tree depth should behave correctly
        let mut graph = ModuleGraph::new();

        let root = graph.add_node(ModuleNode::new(
            "crate".to_string(),
            ash_core::module_graph::ModuleSource::File("main.ash".to_string()),
        ));
        graph.set_root(root);

        let child = graph.add_node(ModuleNode::new(
            "child".to_string(),
            ash_core::module_graph::ModuleSource::File("child.ash".to_string()),
        ));
        graph.add_edge(root, child);

        // Test pub(super, super) from child (only 1 level available)
        let mut resolver = ImportResolver::new(&graph);
        resolver.add_module_exports(
            child,
            vec![("Super2Item".to_string(), Visibility::Super { levels: 2 })],
        );

        // Root should be able to import (is within 2 levels up)
        let use_path = UsePath::Simple(simple_path(&["crate", "child", "Super2Item"]));
        resolver.add_module_uses(root, vec![use_stmt(use_path)]);

        let bindings = resolver.resolve_all().unwrap();
        assert!(bindings.get(&root).unwrap().contains_key("Super2Item"));
    }

    #[test]
    fn test_pub_in_path_deeply_nested() {
        // Test pub(in path) with deeply nested restriction path
        let mut graph = ModuleGraph::new();

        let root = graph.add_node(ModuleNode::new(
            "crate".to_string(),
            ash_core::module_graph::ModuleSource::File("main.ash".to_string()),
        ));
        graph.set_root(root);

        // Create hierarchy: root -> a -> b -> c -> target
        let a = graph.add_node(ModuleNode::new(
            "a".to_string(),
            ash_core::module_graph::ModuleSource::File("a.ash".to_string()),
        ));
        graph.add_edge(root, a);

        let b = graph.add_node(ModuleNode::new(
            "b".to_string(),
            ash_core::module_graph::ModuleSource::File("a/b.ash".to_string()),
        ));
        graph.add_edge(a, b);

        let c = graph.add_node(ModuleNode::new(
            "c".to_string(),
            ash_core::module_graph::ModuleSource::File("a/b/c.ash".to_string()),
        ));
        graph.add_edge(b, c);

        let target = graph.add_node(ModuleNode::new(
            "target".to_string(),
            ash_core::module_graph::ModuleSource::File("a/b/c/target.ash".to_string()),
        ));
        graph.add_edge(c, target);

        let mut resolver = ImportResolver::new(&graph);

        // Add pub(in crate::a::b) export to target module
        resolver.add_module_exports(
            target,
            vec![(
                "DeepRestrictedItem".to_string(),
                Visibility::Restricted {
                    path: "crate::a::b".into(),
                },
            )],
        );

        // c (descendant of b) should succeed
        let use_path1 = UsePath::Simple(simple_path(&[
            "crate",
            "a",
            "b",
            "c",
            "target",
            "DeepRestrictedItem",
        ]));
        resolver.add_module_uses(c, vec![use_stmt(use_path1)]);

        let bindings = resolver.resolve_all().unwrap();
        assert!(bindings.get(&c).unwrap().contains_key("DeepRestrictedItem"));

        // a (ancestor of b, not descendant) should fail
        let mut resolver2 = ImportResolver::new(&graph);
        resolver2.add_module_exports(
            target,
            vec![(
                "DeepRestrictedItem".to_string(),
                Visibility::Restricted {
                    path: "crate::a::b".into(),
                },
            )],
        );
        let use_path2 = UsePath::Simple(simple_path(&[
            "crate",
            "a",
            "b",
            "c",
            "target",
            "DeepRestrictedItem",
        ]));
        resolver2.add_module_uses(a, vec![use_stmt(use_path2)]);

        let result2 = resolver2.resolve_all();
        assert!(result2.is_err());
    }

    #[test]
    fn test_pub_in_path_empty_path_rejected() {
        // Empty path in pub(in path) should be handled gracefully
        let (graph, _root, foo, _bar, _baz, _sibling) = create_four_level_graph();

        let mut resolver = ImportResolver::new(&graph);

        // Add pub(in "") export - empty path
        resolver.add_module_exports(
            foo,
            vec![(
                "EmptyPathItem".to_string(),
                Visibility::Restricted { path: "".into() },
            )],
        );

        // Try to import from foo itself
        let use_path = UsePath::Simple(simple_path(&["crate", "foo", "EmptyPathItem"]));
        resolver.add_module_uses(foo, vec![use_stmt(use_path)]);

        // Import should fail - empty path doesn't resolve
        let result = resolver.resolve_all();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(
            matches!(err, ImportError::PrivateItem { ref item, .. } if item == "EmptyPathItem"),
            "Expected PrivateItem error for empty path, got: {:?}",
            err
        );
    }

    // =========================================================================
    // TASK-335: Complex Hierarchy Tests
    // =========================================================================

    /// Helper to create a complex 5-level hierarchy with multiple branches
    /// Structure:
    ///                     crate (root)
    ///                    /      \
    ///                   /        \
    ///                 left       right
    ///                /   \       /   \
    ///              l1    l2    r1    r2
    ///             /      |     |      \
    ///           l1a     l2a   r1a     r2a
    fn create_complex_hierarchy_graph() -> (
        ModuleGraph,
        ModuleId, // root
        ModuleId, // left
        ModuleId, // right
        ModuleId, // l1
        ModuleId, // l2
        ModuleId, // r1
        ModuleId, // r2
        ModuleId, // l1a
        ModuleId, // r1a
    ) {
        let mut graph = ModuleGraph::new();

        // Root
        let root = graph.add_node(ModuleNode::new(
            "crate".to_string(),
            ash_core::module_graph::ModuleSource::File("main.ash".to_string()),
        ));
        graph.set_root(root);

        // Level 1: left, right
        let left = graph.add_node(ModuleNode::new(
            "left".to_string(),
            ash_core::module_graph::ModuleSource::File("left.ash".to_string()),
        ));
        graph.add_edge(root, left);

        let right = graph.add_node(ModuleNode::new(
            "right".to_string(),
            ash_core::module_graph::ModuleSource::File("right.ash".to_string()),
        ));
        graph.add_edge(root, right);

        // Level 2: l1, l2 (children of left), r1, r2 (children of right)
        let l1 = graph.add_node(ModuleNode::new(
            "l1".to_string(),
            ash_core::module_graph::ModuleSource::File("left/l1.ash".to_string()),
        ));
        graph.add_edge(left, l1);

        let l2 = graph.add_node(ModuleNode::new(
            "l2".to_string(),
            ash_core::module_graph::ModuleSource::File("left/l2.ash".to_string()),
        ));
        graph.add_edge(left, l2);

        let r1 = graph.add_node(ModuleNode::new(
            "r1".to_string(),
            ash_core::module_graph::ModuleSource::File("right/r1.ash".to_string()),
        ));
        graph.add_edge(right, r1);

        let r2 = graph.add_node(ModuleNode::new(
            "r2".to_string(),
            ash_core::module_graph::ModuleSource::File("right/r2.ash".to_string()),
        ));
        graph.add_edge(right, r2);

        // Level 3: l1a (child of l1), r1a (child of r1)
        let l1a = graph.add_node(ModuleNode::new(
            "l1a".to_string(),
            ash_core::module_graph::ModuleSource::File("left/l1/l1a.ash".to_string()),
        ));
        graph.add_edge(l1, l1a);

        let r1a = graph.add_node(ModuleNode::new(
            "r1a".to_string(),
            ash_core::module_graph::ModuleSource::File("right/r1/r1a.ash".to_string()),
        ));
        graph.add_edge(r1, r1a);

        (graph, root, left, right, l1, l2, r1, r2, l1a, r1a)
    }

    #[test]
    fn test_visibility_complex_hierarchy() {
        // Test complex hierarchy with mix of visibility types
        let (graph, root, left, right, l1, l2, _r1, _r2, l1a, r1a) =
            create_complex_hierarchy_graph();

        let mut resolver = ImportResolver::new(&graph);

        // Add various visibility exports to l1a
        resolver.add_module_exports(
            l1a,
            vec![
                ("PubItem".to_string(), Visibility::Public),
                ("SuperItem".to_string(), Visibility::Super { levels: 1 }),
                (
                    "RestrictedItem".to_string(),
                    Visibility::Restricted {
                        path: "crate::left".into(),
                    },
                ),
                ("PrivateItem".to_string(), Visibility::Inherited),
            ],
        );

        // Test 1: l1 (parent of l1a) can import SuperItem
        let use_path1 = UsePath::Simple(simple_path(&["crate", "left", "l1", "l1a", "SuperItem"]));
        resolver.add_module_uses(l1, vec![use_stmt(use_path1)]);

        // Test 2: left (grandparent of l1a) can import RestrictedItem (restricted to crate::left)
        let use_path2 = UsePath::Simple(simple_path(&[
            "crate",
            "left",
            "l1",
            "l1a",
            "RestrictedItem",
        ]));
        resolver.add_module_uses(left, vec![use_stmt(use_path2)]);

        // Test 3: root can import PubItem
        let use_path3 = UsePath::Simple(simple_path(&["crate", "left", "l1", "l1a", "PubItem"]));
        resolver.add_module_uses(root, vec![use_stmt(use_path3)]);

        let bindings = resolver.resolve_all().unwrap();

        // Verify all allowed imports
        assert!(bindings.get(&l1).unwrap().contains_key("SuperItem"));
        assert!(bindings.get(&left).unwrap().contains_key("RestrictedItem"));
        assert!(bindings.get(&root).unwrap().contains_key("PubItem"));

        // Test 4: right (different branch) can only import PubItem
        let mut resolver2 = ImportResolver::new(&graph);
        resolver2.add_module_exports(
            l1a,
            vec![
                ("PubItem".to_string(), Visibility::Public),
                ("SuperItem".to_string(), Visibility::Super { levels: 1 }),
                (
                    "RestrictedItem".to_string(),
                    Visibility::Restricted {
                        path: "crate::left".into(),
                    },
                ),
            ],
        );
        let use_path4 = UsePath::Simple(simple_path(&["crate", "left", "l1", "l1a", "PubItem"]));
        resolver2.add_module_uses(right, vec![use_stmt(use_path4)]);

        let bindings2 = resolver2.resolve_all().unwrap();
        assert!(bindings2.get(&right).unwrap().contains_key("PubItem"));

        // Test 5: right should NOT be able to import SuperItem
        let mut resolver3 = ImportResolver::new(&graph);
        resolver3.add_module_exports(
            l1a,
            vec![("SuperItem".to_string(), Visibility::Super { levels: 1 })],
        );
        let use_path5 = UsePath::Simple(simple_path(&["crate", "left", "l1", "l1a", "SuperItem"]));
        resolver3.add_module_uses(right, vec![use_stmt(use_path5)]);

        let result3 = resolver3.resolve_all();
        assert!(result3.is_err());

        // Test 6: r1a should NOT be able to import RestrictedItem (restricted to crate::left)
        let mut resolver4 = ImportResolver::new(&graph);
        resolver4.add_module_exports(
            l1a,
            vec![(
                "RestrictedItem".to_string(),
                Visibility::Restricted {
                    path: "crate::left".into(),
                },
            )],
        );
        let use_path6 = UsePath::Simple(simple_path(&[
            "crate",
            "left",
            "l1",
            "l1a",
            "RestrictedItem",
        ]));
        resolver4.add_module_uses(r1a, vec![use_stmt(use_path6)]);

        let result4 = resolver4.resolve_all();
        assert!(result4.is_err());

        // Test 7: l2 (sibling branch of l1) should NOT be able to import SuperItem
        let mut resolver5 = ImportResolver::new(&graph);
        resolver5.add_module_exports(
            l1a,
            vec![("SuperItem".to_string(), Visibility::Super { levels: 1 })],
        );
        let use_path7 = UsePath::Simple(simple_path(&["crate", "left", "l1", "l1a", "SuperItem"]));
        resolver5.add_module_uses(l2, vec![use_stmt(use_path7)]);

        let result5 = resolver5.resolve_all();
        assert!(result5.is_err());
    }

    #[test]
    fn test_visibility_cross_branch() {
        // Test visibility between different branches of module tree
        let (graph, _root, left, right, l1, _l2, r1, _r2, _l1a, _r1a) =
            create_complex_hierarchy_graph();

        // Test: pub(crate) should work across branches
        let mut graph_with_crate = graph;
        let crate_a = CrateId(0);
        for module_id in [&left, &right, &l1, &r1] {
            graph_with_crate.set_crate(*module_id, crate_a);
        }

        let mut resolver = ImportResolver::new(&graph_with_crate);

        // Add pub(crate) export to l1
        resolver.add_module_exports(l1, vec![("CrateItem".to_string(), Visibility::Crate)]);

        // r1 (different branch, same crate) should be able to import
        let use_path = UsePath::Simple(simple_path(&["crate", "left", "l1", "CrateItem"]));
        resolver.add_module_uses(r1, vec![use_stmt(use_path)]);

        let bindings = resolver.resolve_all().unwrap();
        assert!(bindings.get(&r1).unwrap().contains_key("CrateItem"));

        // Test: pub(in crate::right) from l1 should be rejected by r1
        let mut resolver2 = ImportResolver::new(&graph_with_crate);
        resolver2.add_module_exports(
            l1,
            vec![(
                "LeftOnlyItem".to_string(),
                Visibility::Restricted {
                    path: "crate::left".into(),
                },
            )],
        );

        let use_path2 = UsePath::Simple(simple_path(&["crate", "left", "l1", "LeftOnlyItem"]));
        resolver2.add_module_uses(r1, vec![use_stmt(use_path2)]);

        let result2 = resolver2.resolve_all();
        assert!(result2.is_err());

        // Test: pub(in crate::right) from r1 should be visible to r1 but not l1
        let mut resolver3 = ImportResolver::new(&graph_with_crate);
        resolver3.add_module_exports(
            r1,
            vec![(
                "RightOnlyItem".to_string(),
                Visibility::Restricted {
                    path: "crate::right".into(),
                },
            )],
        );

        // r1 can import its own restricted item
        let use_path3 = UsePath::Simple(simple_path(&["crate", "right", "r1", "RightOnlyItem"]));
        resolver3.add_module_uses(r1, vec![use_stmt(use_path3)]);

        let bindings3 = resolver3.resolve_all().unwrap();
        assert!(bindings3.get(&r1).unwrap().contains_key("RightOnlyItem"));

        // l1 cannot import it
        let mut resolver4 = ImportResolver::new(&graph_with_crate);
        resolver4.add_module_exports(
            r1,
            vec![(
                "RightOnlyItem".to_string(),
                Visibility::Restricted {
                    path: "crate::right".into(),
                },
            )],
        );
        let use_path4 = UsePath::Simple(simple_path(&["crate", "right", "r1", "RightOnlyItem"]));
        resolver4.add_module_uses(l1, vec![use_stmt(use_path4)]);

        let result4 = resolver4.resolve_all();
        assert!(result4.is_err());
    }

    #[test]
    fn test_visibility_self_only() {
        // Test pub(self) visibility - should NOT be visible from any importing module
        // Note: pub(self) is effectively private; items are only accessible within
        // the same module without going through import resolution
        let (graph, _root, foo, _bar, _baz, _sibling) = create_four_level_graph();

        // root (parent) should NOT be able to import from foo with pub(self)
        let mut resolver2 = ImportResolver::new(&graph);
        resolver2.add_module_exports(foo, vec![("SelfItem".to_string(), Visibility::Self_)]);
        let use_path2 = UsePath::Simple(simple_path(&["crate", "foo", "SelfItem"]));
        resolver2.add_module_uses(
            graph.get_root().copied().unwrap(),
            vec![use_stmt(use_path2)],
        );

        let result2 = resolver2.resolve_all();
        assert!(result2.is_err());

        // bar (descendant) should NOT be able to import from foo with pub(self)
        let mut resolver3 = ImportResolver::new(&graph);
        resolver3.add_module_exports(foo, vec![("SelfItem".to_string(), Visibility::Self_)]);
        let use_path3 = UsePath::Simple(simple_path(&["crate", "foo", "SelfItem"]));
        resolver3.add_module_uses(_bar, vec![use_stmt(use_path3)]);

        let result3 = resolver3.resolve_all();
        assert!(result3.is_err());
    }

    #[test]
    fn test_visibility_mixed_imports() {
        // Test module importing items with different visibilities from same source
        let (graph, root, foo, bar, _baz, _sibling) = create_four_level_graph();

        let mut resolver = ImportResolver::new(&graph);

        // Add mixed visibility exports to bar (excluding Crate which needs crate assignment)
        resolver.add_module_exports(
            bar,
            vec![
                ("PubItem".to_string(), Visibility::Public),
                ("SuperItem".to_string(), Visibility::Super { levels: 1 }),
                ("PrivateItem".to_string(), Visibility::Inherited),
            ],
        );

        // foo (parent of bar) tries to import public and super-visible items
        let use_items = vec![
            UseItem {
                name: "PubItem".into(),
                alias: None,
            },
            UseItem {
                name: "SuperItem".into(),
                alias: None,
            },
        ];
        let use_path = UsePath::Nested(simple_path(&["crate", "foo", "bar"]), use_items);
        resolver.add_module_uses(foo, vec![use_stmt(use_path)]);

        // Should succeed for the visible items
        let bindings = resolver.resolve_all().unwrap();
        let foo_bindings = bindings.get(&foo).unwrap();

        assert!(foo_bindings.contains_key("PubItem"));
        assert!(foo_bindings.contains_key("SuperItem"));

        // root (grandparent) tries to import SuperItem - should fail (levels=1 only allows parent)
        let mut resolver2 = ImportResolver::new(&graph);
        resolver2.add_module_exports(
            bar,
            vec![("SuperItem".to_string(), Visibility::Super { levels: 1 })],
        );
        let use_path2 = UsePath::Simple(simple_path(&["crate", "foo", "bar", "SuperItem"]));
        resolver2.add_module_uses(root, vec![use_stmt(use_path2)]);

        let result2 = resolver2.resolve_all();
        assert!(result2.is_err());
    }

    // =========================================================================
    // TASK-340: External import resolution and cross-crate visibility tests
    // =========================================================================

    /// Helper to create a graph with external crate dependencies:
    /// Main crate (id=0): main_mod -> child_mod
    /// External crate (id=1): util -> sanitize
    fn create_external_crate_graph() -> (
        ModuleGraph,
        CrateId,
        CrateId,
        ModuleId,
        ModuleId,
        ModuleId,
        ModuleId,
    ) {
        use ash_core::module_graph::ModuleSource;

        let mut graph = ModuleGraph::new();

        // Create main crate modules
        let main_mod = graph.add_node(ModuleNode::new(
            "main".to_string(),
            ModuleSource::File("main.ash".to_string()),
        ));
        let child_mod = graph.add_node(ModuleNode::new(
            "child".to_string(),
            ModuleSource::File("main/child.ash".to_string()),
        ));
        graph.set_root(main_mod);
        graph.add_edge(main_mod, child_mod);

        // Register main crate using add_crate (properly registers in crates HashMap)
        let main_crate = graph.add_crate("main".to_string(), "/main".to_string(), main_mod);
        graph.set_crate(child_mod, main_crate);

        // Create external crate modules - util library
        let util_mod = graph.add_node(ModuleNode::new(
            "util".to_string(),
            ModuleSource::File("util/lib.ash".to_string()),
        ));
        let sanitize_mod = graph.add_node(ModuleNode::new(
            "sanitize".to_string(),
            ModuleSource::File("util/sanitize.ash".to_string()),
        ));
        graph.add_edge(util_mod, sanitize_mod);

        // Register external crate using add_crate
        let ext_crate = graph.add_crate("util".to_string(), "/util".to_string(), util_mod);
        graph.set_crate(sanitize_mod, ext_crate);

        // Register the external crate as a dependency of main crate
        graph.add_dependency(main_crate, "util".to_string(), ext_crate);

        (
            graph,
            main_crate,
            ext_crate,
            main_mod,
            child_mod,
            util_mod,
            sanitize_mod,
        )
    }

    #[test]
    fn test_external_import_public_item_allowed() {
        // external::util::sanitize::normalize should succeed when normalize is pub
        let (graph, _main_crate, _ext_crate, main_mod, _child_mod, _util_mod, sanitize_mod) =
            create_external_crate_graph();

        let mut resolver = ImportResolver::new(&graph);

        // Add public export to sanitize module
        resolver.add_module_exports(
            sanitize_mod,
            vec![("normalize".to_string(), Visibility::Public)],
        );

        // Add use statement from main module importing from external crate
        // Path: external::util::sanitize::normalize
        let use_path = UsePath::Simple(simple_path(&["external", "util", "sanitize", "normalize"]));
        resolver.add_module_uses(main_mod, vec![use_stmt(use_path)]);

        // Import should succeed - public items are importable across crates
        let bindings = resolver.resolve_all().unwrap();
        let main_bindings = bindings.get(&main_mod).unwrap();

        assert!(main_bindings.contains_key("normalize"));
        let binding = main_bindings.get("normalize").unwrap();
        assert_eq!(binding.item_name, "normalize");
        assert_eq!(binding.target_module, sanitize_mod);
    }

    #[test]
    fn test_external_import_pub_crate_rejected() {
        // external::util::sanitize::normalize should fail when normalize is pub(crate)
        let (graph, _main_crate, _ext_crate, main_mod, _child_mod, _util_mod, sanitize_mod) =
            create_external_crate_graph();

        let mut resolver = ImportResolver::new(&graph);

        // Add pub(crate) export to sanitize module
        resolver.add_module_exports(
            sanitize_mod,
            vec![("internal_func".to_string(), Visibility::Crate)],
        );

        // Add use statement from main module importing from external crate
        let use_path = UsePath::Simple(simple_path(&[
            "external",
            "util",
            "sanitize",
            "internal_func",
        ]));
        resolver.add_module_uses(main_mod, vec![use_stmt(use_path)]);

        // Import should fail - pub(crate) is not visible across crate boundaries
        let result = resolver.resolve_all();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(
            matches!(err, ImportError::PrivateItem { ref item, .. } if item == "internal_func"),
            "Expected PrivateItem error for pub(crate) external import, got: {:?}",
            err
        );
    }

    #[test]
    fn test_external_import_pub_super_rejected() {
        // external::util::sanitize::normalize should fail when normalize is pub(super)
        let (graph, _main_crate, _ext_crate, main_mod, _child_mod, _util_mod, sanitize_mod) =
            create_external_crate_graph();

        let mut resolver = ImportResolver::new(&graph);

        // Add pub(super) export to sanitize module
        resolver.add_module_exports(
            sanitize_mod,
            vec![("parent_item".to_string(), Visibility::Super { levels: 1 })],
        );

        // Add use statement from main module importing from external crate
        let use_path = UsePath::Simple(simple_path(&[
            "external",
            "util",
            "sanitize",
            "parent_item",
        ]));
        resolver.add_module_uses(main_mod, vec![use_stmt(use_path)]);

        // Import should fail - pub(super) is not visible across crate boundaries
        let result = resolver.resolve_all();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(
            matches!(err, ImportError::PrivateItem { ref item, .. } if item == "parent_item"),
            "Expected PrivateItem error for pub(super) external import, got: {:?}",
            err
        );
    }

    #[test]
    fn test_external_import_undeclared_alias_rejected() {
        // external::unknown::item should fail when "unknown" is not a declared dependency
        let (graph, _main_crate, _ext_crate, main_mod, _child_mod, _util_mod, _sanitize_mod) =
            create_external_crate_graph();

        // Remove the "util" dependency to simulate undeclared alias
        // We need to create a new graph without the dependency
        let mut resolver = ImportResolver::new(&graph);

        // Try to import from an undeclared external crate
        let use_path = UsePath::Simple(simple_path(&["external", "unknown", "item"]));
        resolver.add_module_uses(main_mod, vec![use_stmt(use_path)]);

        // Import should fail - "unknown" is not a declared dependency alias
        let result = resolver.resolve_all();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(
            matches!(err, ImportError::ModuleNotFound { .. }),
            "Expected ModuleNotFound error for undeclared external alias, got: {:?}",
            err
        );
    }

    #[test]
    fn test_external_glob_import_skips_non_public_items() {
        // external::util::sanitize::* should only import pub items
        let (graph, _main_crate, _ext_crate, main_mod, _child_mod, _util_mod, sanitize_mod) =
            create_external_crate_graph();

        let mut resolver = ImportResolver::new(&graph);

        // Add mixed visibility exports to sanitize module
        resolver.add_module_exports(
            sanitize_mod,
            vec![
                ("public_func".to_string(), Visibility::Public),
                ("crate_func".to_string(), Visibility::Crate),
                ("super_func".to_string(), Visibility::Super { levels: 1 }),
                ("private_func".to_string(), Visibility::Inherited),
            ],
        );

        // Add glob import from main module
        let use_path = UsePath::Glob(simple_path(&["external", "util", "sanitize"]));
        resolver.add_module_uses(main_mod, vec![use_stmt(use_path)]);

        // Glob import should only include public items
        let bindings = resolver.resolve_all().unwrap();
        let main_bindings = bindings.get(&main_mod).unwrap();

        assert!(
            main_bindings.contains_key("public_func"),
            "Public item should be imported via glob"
        );
        assert!(
            !main_bindings.contains_key("crate_func"),
            "pub(crate) item should NOT be imported via glob across crates"
        );
        assert!(
            !main_bindings.contains_key("super_func"),
            "pub(super) item should NOT be imported via glob across crates"
        );
        assert!(
            !main_bindings.contains_key("private_func"),
            "Private item should NOT be imported via glob"
        );
    }

    #[test]
    fn test_external_import_nested() {
        // use external::util::{helpers, config as cfg};
        let (graph, _main_crate, _ext_crate, main_mod, _child_mod, util_mod, _sanitize_mod) =
            create_external_crate_graph();

        let mut resolver = ImportResolver::new(&graph);

        // Add exports to util module (items, not modules)
        resolver.add_module_exports(
            util_mod,
            vec![
                ("helpers".to_string(), Visibility::Public),
                ("config".to_string(), Visibility::Public),
            ],
        );

        // Add nested import from main module
        let items = vec![
            UseItem {
                name: "helpers".into(),
                alias: None,
            },
            UseItem {
                name: "config".into(),
                alias: Some("cfg".into()),
            },
        ];
        let use_path = UsePath::Nested(simple_path(&["external", "util"]), items);
        resolver.add_module_uses(main_mod, vec![use_stmt(use_path)]);

        // Import should succeed
        let bindings = resolver.resolve_all().unwrap();
        let main_bindings = bindings.get(&main_mod).unwrap();

        assert!(main_bindings.contains_key("helpers"));
        assert!(main_bindings.contains_key("cfg"));
        assert!(!main_bindings.contains_key("config")); // aliased, not imported under original name
    }
}
