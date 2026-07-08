//! Import resolution algorithm for the Ash parser.
//!
//! This module provides functionality to resolve `use` statements in modules,
//! building name binding tables and verifying visibility constraints.

use ash_core::module_graph::{ModuleGraph, ModuleId};
use std::collections::HashMap;

mod types;
pub use types::{Binding, BindingItemKind, BindingKind, BindingTable, ImportError};

use crate::capability_export::ModuleDefinitionExport;
use crate::surface::Visibility;
use crate::use_tree::{SimplePath, Use, UseItem, UsePath};

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
    item_kind: BindingItemKind,
    /// For operational capabilities: the target (provider, action) pair
    capability_target: Option<(String, String)>,
    /// For Phase 101 capability/resource definitions: parsed module metadata.
    definition_metadata: Option<ModuleDefinitionExport>,
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
                        item_kind: BindingItemKind::Item,
                        capability_target: None,
                        definition_metadata: None,
                    },
                )
            })
            .collect();
        self.module_exports.insert(module_id, export_map);
    }

    /// Add capability exports for a module with target metadata.
    ///
    /// This should be called before `resolve_all` to provide the
    /// exported capability symbols with their (provider, action) targets.
    pub fn add_capability_exports(
        &mut self,
        module_id: ModuleId,
        exports: Vec<(impl Into<String>, Visibility, String, String)>,
    ) {
        let export_map: HashMap<String, Export> = exports
            .into_iter()
            .map(|(name, vis, provider, action)| {
                let name = name.into();
                (
                    name.clone(),
                    Export {
                        name,
                        visibility: vis,
                        item_kind: BindingItemKind::ProviderOperation,
                        capability_target: Some((provider, action)),
                        definition_metadata: None,
                    },
                )
            })
            .collect();
        self.module_exports.insert(module_id, export_map);
    }

    /// Add current resource definition exports for a module.
    ///
    /// This should be called before `resolve_all` to provide exported parser
    /// metadata for resource types.
    pub fn add_definition_exports(
        &mut self,
        module_id: ModuleId,
        exports: Vec<ModuleDefinitionExport>,
    ) {
        let export_map = self.module_exports.entry(module_id).or_default();
        for metadata in exports {
            let name = metadata.visible_name.to_string();
            let item_kind = BindingItemKind::from(&metadata.kind);
            export_map.insert(
                name.clone(),
                Export {
                    name,
                    visibility: metadata.visibility.clone(),
                    item_kind,
                    capability_target: None,
                    definition_metadata: Some(metadata),
                },
            );
        }
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

        let binding = if let Some((provider, action)) = &export.capability_target {
            Binding::with_capability_target(
                target_module,
                item_name,
                export.visibility.clone(),
                kind,
                (provider.clone(), action.clone()),
            )
        } else if let Some(metadata) = &export.definition_metadata {
            Binding::with_definition_metadata(
                target_module,
                item_name,
                export.visibility.clone(),
                kind,
                metadata.clone(),
            )
        } else {
            Binding::with_item_kind(
                target_module,
                item_name,
                export.visibility.clone(),
                kind,
                export.item_kind,
            )
        };

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

            let binding = if let Some((provider, action)) = &export.capability_target {
                Binding::with_capability_target(
                    target_module,
                    name.clone(),
                    export.visibility.clone(),
                    BindingKind::Glob,
                    (provider.clone(), action.clone()),
                )
            } else if let Some(metadata) = &export.definition_metadata {
                Binding::with_definition_metadata(
                    target_module,
                    name.clone(),
                    export.visibility.clone(),
                    BindingKind::Glob,
                    metadata.clone(),
                )
            } else {
                Binding::with_item_kind(
                    target_module,
                    name.clone(),
                    export.visibility.clone(),
                    BindingKind::Glob,
                    export.item_kind,
                )
            };

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

            let binding = if let Some((provider, action)) = &export.capability_target {
                Binding::with_capability_target(
                    target_module,
                    item_name.to_string(),
                    export.visibility.clone(),
                    kind,
                    (provider.clone(), action.clone()),
                )
            } else {
                Binding::with_item_kind(
                    target_module,
                    item_name.to_string(),
                    export.visibility.clone(),
                    kind,
                    export.item_kind,
                )
            };

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
mod tests;
