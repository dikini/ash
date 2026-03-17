//! Name binding with imports for the Ash type checker (TASK-087)
//!
//! This module provides `NameBinder` which integrates import resolution
//! with name resolution, supporting:
//! - Local scope tracking (let bindings, parameters)
//! - Module-level definitions
//! - Import bindings from ImportResolver
//! - Shadowing rules: local > explicit import > glob import > parent module

use ash_core::module_graph::{ModuleGraph, ModuleId};
use ash_parser::import_resolver::{Binding, BindingKind, BindingTable};
use std::collections::HashMap;
use thiserror::Error;

/// Type information for a resolved name
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedName {
    /// The name being resolved
    pub name: String,
    /// The module where the definition is located
    pub module_id: ModuleId,
    /// The kind of resolution
    pub kind: ResolutionKind,
    /// The original binding if from an import
    pub binding: Option<Binding>,
}

/// Kind of name resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionKind {
    /// Local variable binding
    Local,
    /// Module-level definition
    ModuleDefinition,
    /// Explicit import binding
    ExplicitImport,
    /// Glob import binding
    GlobImport,
    /// From parent module
    ParentModule,
}

/// Error during name resolution
#[derive(Debug, Clone, PartialEq, Error)]
pub enum NameError {
    /// Name not found in any scope
    #[error("unresolved name: {name}")]
    Unresolved { name: String },
    /// Name is private and not accessible
    #[error("name '{name}' is private")]
    Private { name: String },
}

/// Local binding information
#[derive(Debug, Clone, PartialEq)]
struct LocalBinding {
    name: String,
    // In a real implementation, this would store type information
}

/// A scope for local bindings
#[derive(Debug, Clone, Default)]
struct LocalScope {
    bindings: HashMap<String, LocalBinding>,
}

/// Name binder that integrates import resolution with name resolution
///
/// The binder maintains a hierarchy of scopes for name resolution:
/// 1. Local scope (let bindings in current block)
/// 2. Current module definitions
/// 3. Explicit import bindings
/// 4. Glob import bindings
/// 5. Parent module (if not found)
#[derive(Debug)]
pub struct NameBinder<'a> {
    /// The module graph for module hierarchy
    module_graph: &'a ModuleGraph,
    /// Import bindings per module (from ImportResolver)
    import_bindings: &'a HashMap<ModuleId, BindingTable>,
    /// Module definitions (name -> module that defines it)
    module_definitions: HashMap<ModuleId, HashMap<String, ModuleId>>,
    /// Local scope stack (innermost last)
    local_scopes: Vec<LocalScope>,
}

impl<'a> NameBinder<'a> {
    /// Create a new name binder
    pub fn new(
        module_graph: &'a ModuleGraph,
        import_bindings: &'a HashMap<ModuleId, BindingTable>,
    ) -> Self {
        Self {
            module_graph,
            import_bindings,
            module_definitions: HashMap::new(),
            local_scopes: vec![LocalScope::default()],
        }
    }

    /// Enter a new local scope
    pub fn enter_scope(&mut self) {
        self.local_scopes.push(LocalScope::default());
    }

    /// Exit the current local scope
    pub fn exit_scope(&mut self) {
        if self.local_scopes.len() > 1 {
            self.local_scopes.pop();
        }
    }

    /// Bind a local variable in the current scope
    pub fn bind_local(&mut self, name: impl Into<String>) {
        let name = name.into();
        if let Some(scope) = self.local_scopes.last_mut() {
            scope
                .bindings
                .insert(name.clone(), LocalBinding { name: name.clone() });
        }
    }

    /// Add a module definition
    pub fn add_module_definition(&mut self, module_id: ModuleId, name: impl Into<String>) {
        let name = name.into();
        let definitions = self.module_definitions.entry(module_id).or_default();
        definitions.insert(name, module_id);
    }

    /// Resolve a name in the given module context
    ///
    /// Resolution order:
    /// 1. Local scope
    /// 2. Current module definitions
    /// 3. Explicit import bindings
    /// 4. Glob import bindings
    /// 5. Parent module
    pub fn resolve(&self, name: &str, current_module: ModuleId) -> Result<ResolvedName, NameError> {
        // 1. Check local scope (innermost first)
        for scope in self.local_scopes.iter().rev() {
            if let Some(binding) = scope.bindings.get(name) {
                return Ok(ResolvedName {
                    name: binding.name.clone(),
                    module_id: current_module,
                    kind: ResolutionKind::Local,
                    binding: None,
                });
            }
        }

        // 2. Check current module definitions
        if let Some(definitions) = self.module_definitions.get(&current_module)
            && definitions.contains_key(name)
        {
            return Ok(ResolvedName {
                name: name.to_string(),
                module_id: current_module,
                kind: ResolutionKind::ModuleDefinition,
                binding: None,
            });
        }

        // 3. Check explicit imports (Direct and Aliased)
        if let Some(binding_table) = self.import_bindings.get(&current_module)
            && let Some(binding) = binding_table.get(name)
            && matches!(
                binding.kind,
                BindingKind::Direct | BindingKind::Aliased { .. }
            )
        {
            return Ok(ResolvedName {
                name: name.to_string(),
                module_id: binding.target_module,
                kind: ResolutionKind::ExplicitImport,
                binding: Some(binding.clone()),
            });
        }

        // 4. Check glob imports
        if let Some(binding_table) = self.import_bindings.get(&current_module)
            && let Some(binding) = binding_table.get(name)
            && matches!(binding.kind, BindingKind::Glob)
        {
            return Ok(ResolvedName {
                name: name.to_string(),
                module_id: binding.target_module,
                kind: ResolutionKind::GlobImport,
                binding: Some(binding.clone()),
            });
        }

        // 5. Check parent module
        if let Some(parent_id) = self.find_parent_module(current_module)
            && let Some(parent_defs) = self.module_definitions.get(&parent_id)
            && parent_defs.contains_key(name)
        {
            return Ok(ResolvedName {
                name: name.to_string(),
                module_id: parent_id,
                kind: ResolutionKind::ParentModule,
                binding: None,
            });
        }

        // Name not found
        Err(NameError::Unresolved {
            name: name.to_string(),
        })
    }

    /// Find the parent module of a given module
    fn find_parent_module(&self, module_id: ModuleId) -> Option<ModuleId> {
        // Search through all nodes to find one that has module_id as a child
        for (id, node) in &self.module_graph.nodes {
            if node.children.contains(&module_id) {
                return Some(*id);
            }
        }
        None
    }

    /// Check if a name is resolvable (without actually resolving it)
    pub fn can_resolve(&self, name: &str, current_module: ModuleId) -> bool {
        self.resolve(name, current_module).is_ok()
    }

    /// Get the current scope depth
    pub fn scope_depth(&self) -> usize {
        self.local_scopes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::module_graph::{ModuleGraph, ModuleNode, ModuleSource};
    use ash_parser::surface::Visibility;

    // Helper: Create a test module graph with root and child modules
    fn create_test_graph() -> (ModuleGraph, ModuleId, ModuleId) {
        let mut graph = ModuleGraph::new();

        // Root module
        let root = graph.add_node(ModuleNode::new(
            "crate".to_string(),
            ModuleSource::File("main.ash".to_string()),
        ));
        graph.set_root(root);

        // Child module (submodule)
        let child = graph.add_node(ModuleNode::new(
            "foo".to_string(),
            ModuleSource::File("foo.ash".to_string()),
        ));
        graph.add_edge(root, child);

        (graph, root, child)
    }

    // Helper: Create a binding table from explicit imports
    fn create_explicit_binding_table(target_module: ModuleId, name: &str) -> BindingTable {
        let mut table = BindingTable::new();
        table.insert(
            name.to_string(),
            Binding::new(target_module, name, Visibility::Public, BindingKind::Direct),
        );
        table
    }

    // Helper: Create a binding table from glob imports
    fn create_glob_binding_table(target_module: ModuleId, names: Vec<&str>) -> BindingTable {
        let mut table = BindingTable::new();
        for name in names {
            table.insert(
                name.to_string(),
                Binding::new(target_module, name, Visibility::Public, BindingKind::Glob),
            );
        }
        table
    }

    // =========================================================================
    // Local Variable Resolution Tests
    // =========================================================================

    #[test]
    fn test_local_variable_resolution() {
        let (graph, root, _) = create_test_graph();
        let import_bindings: HashMap<ModuleId, BindingTable> = HashMap::new();

        let mut binder = NameBinder::new(&graph, &import_bindings);
        binder.enter_scope();
        binder.bind_local("x");

        let result = binder.resolve("x", root);
        assert!(result.is_ok());

        let resolved = result.unwrap();
        assert_eq!(resolved.name, "x");
        assert_eq!(resolved.kind, ResolutionKind::Local);
        assert_eq!(resolved.module_id, root);
    }

    #[test]
    fn test_local_variable_in_inner_scope() {
        let (graph, root, _) = create_test_graph();
        let import_bindings: HashMap<ModuleId, BindingTable> = HashMap::new();

        let mut binder = NameBinder::new(&graph, &import_bindings);
        binder.bind_local("outer");

        binder.enter_scope();
        binder.bind_local("inner");

        // Should resolve inner variable
        let result = binder.resolve("inner", root);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().kind, ResolutionKind::Local);

        // Should resolve outer variable from inner scope
        let result = binder.resolve("outer", root);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().kind, ResolutionKind::Local);
    }

    #[test]
    fn test_local_variable_not_found_after_exit_scope() {
        let (graph, root, _) = create_test_graph();
        let import_bindings: HashMap<ModuleId, BindingTable> = HashMap::new();

        let mut binder = NameBinder::new(&graph, &import_bindings);
        binder.enter_scope();
        binder.bind_local("scoped_var");
        binder.exit_scope();

        let result = binder.resolve("scoped_var", root);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NameError::Unresolved { .. }));
    }

    // =========================================================================
    // Module Definition Resolution Tests
    // =========================================================================

    #[test]
    fn test_module_definition_resolution() {
        let (graph, root, _) = create_test_graph();
        let import_bindings: HashMap<ModuleId, BindingTable> = HashMap::new();

        let mut binder = NameBinder::new(&graph, &import_bindings);
        binder.add_module_definition(root, "my_function");

        let result = binder.resolve("my_function", root);
        assert!(result.is_ok());

        let resolved = result.unwrap();
        assert_eq!(resolved.name, "my_function");
        assert_eq!(resolved.kind, ResolutionKind::ModuleDefinition);
        assert_eq!(resolved.module_id, root);
    }

    #[test]
    fn test_module_definition_in_child_module() {
        let (graph, root, child) = create_test_graph();
        let import_bindings: HashMap<ModuleId, BindingTable> = HashMap::new();

        let mut binder = NameBinder::new(&graph, &import_bindings);
        binder.add_module_definition(child, "child_fn");

        // Should resolve from child module
        let result = binder.resolve("child_fn", child);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().kind, ResolutionKind::ModuleDefinition);

        // Should NOT resolve from root module (definition is in child)
        let result = binder.resolve("child_fn", root);
        assert!(result.is_err());
    }

    // =========================================================================
    // Import Binding Resolution Tests
    // =========================================================================

    #[test]
    fn test_explicit_import_resolution() {
        let (graph, root, child) = create_test_graph();

        // Create import bindings: root imports "bar" from child
        let binding_table = create_explicit_binding_table(child, "bar");
        let mut import_bindings: HashMap<ModuleId, BindingTable> = HashMap::new();
        import_bindings.insert(root, binding_table);

        let binder = NameBinder::new(&graph, &import_bindings);

        let result = binder.resolve("bar", root);
        assert!(result.is_ok());

        let resolved = result.unwrap();
        assert_eq!(resolved.name, "bar");
        assert_eq!(resolved.kind, ResolutionKind::ExplicitImport);
        assert_eq!(resolved.module_id, child);
    }

    #[test]
    fn test_glob_import_resolution() {
        let (graph, root, child) = create_test_graph();

        // Create glob import bindings
        let binding_table = create_glob_binding_table(child, vec!["item1", "item2"]);
        let mut import_bindings: HashMap<ModuleId, BindingTable> = HashMap::new();
        import_bindings.insert(root, binding_table);

        let binder = NameBinder::new(&graph, &import_bindings);

        let result = binder.resolve("item1", root);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().kind, ResolutionKind::GlobImport);

        let result = binder.resolve("item2", root);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().kind, ResolutionKind::GlobImport);
    }

    // =========================================================================
    // Shadowing Tests
    // =========================================================================

    #[test]
    fn test_local_shadows_import() {
        let (graph, root, child) = create_test_graph();

        // Create import binding for "x"
        let binding_table = create_explicit_binding_table(child, "x");
        let mut import_bindings: HashMap<ModuleId, BindingTable> = HashMap::new();
        import_bindings.insert(root, binding_table);

        let mut binder = NameBinder::new(&graph, &import_bindings);

        // Bind local "x" - should shadow import
        binder.bind_local("x");

        let result = binder.resolve("x", root);
        assert!(result.is_ok());

        let resolved = result.unwrap();
        // Local should win over import
        assert_eq!(resolved.kind, ResolutionKind::Local);
        assert_eq!(resolved.module_id, root);
    }

    #[test]
    fn test_explicit_import_shadows_glob_import() {
        let (graph, root, child) = create_test_graph();

        let mut binding_table = BindingTable::new();
        // Add glob binding
        binding_table.insert(
            "shared".to_string(),
            Binding::new(child, "shared", Visibility::Public, BindingKind::Glob),
        );
        // Add explicit binding for same name
        binding_table.insert(
            "shared".to_string(),
            Binding::new(child, "shared", Visibility::Public, BindingKind::Direct),
        );

        let mut import_bindings: HashMap<ModuleId, BindingTable> = HashMap::new();
        import_bindings.insert(root, binding_table);

        let binder = NameBinder::new(&graph, &import_bindings);

        let result = binder.resolve("shared", root);
        assert!(result.is_ok());

        let resolved = result.unwrap();
        // Explicit import should win over glob
        assert_eq!(resolved.kind, ResolutionKind::ExplicitImport);
    }

    #[test]
    fn test_glob_import_shadows_parent_module() {
        let (graph, root, child) = create_test_graph();

        // Create glob import that brings in same name
        let binding_table = create_glob_binding_table(child, vec!["parent_item"]);
        let mut import_bindings: HashMap<ModuleId, BindingTable> = HashMap::new();
        import_bindings.insert(child, binding_table);

        let mut binder = NameBinder::new(&graph, &import_bindings);

        // Add definition in root (parent)
        binder.add_module_definition(root, "parent_item");

        // Note: We're checking that glob import is found before parent
        // This requires having the definition in root AND glob import in child
        // The glob import should be found first (step 4 vs step 5 in resolve order)
        let result = binder.resolve("parent_item", child);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().kind, ResolutionKind::GlobImport);
    }

    // =========================================================================
    // Parent Module Fallback Tests
    // =========================================================================

    #[test]
    fn test_parent_module_fallback() {
        let (graph, root, child) = create_test_graph();
        let import_bindings: HashMap<ModuleId, BindingTable> = HashMap::new();

        let mut binder = NameBinder::new(&graph, &import_bindings);
        // Add definition only in root
        binder.add_module_definition(root, "root_function");

        // Should resolve from child module via parent fallback
        let result = binder.resolve("root_function", child);
        assert!(result.is_ok());

        let resolved = result.unwrap();
        assert_eq!(resolved.name, "root_function");
        assert_eq!(resolved.kind, ResolutionKind::ParentModule);
        assert_eq!(resolved.module_id, root);
    }

    // =========================================================================
    // Unresolved Name Error Tests
    // =========================================================================

    #[test]
    fn test_unresolved_name_error() {
        let (graph, root, _) = create_test_graph();
        let import_bindings: HashMap<ModuleId, BindingTable> = HashMap::new();

        let binder = NameBinder::new(&graph, &import_bindings);

        let result = binder.resolve("nonexistent", root);
        assert!(result.is_err());

        match result.unwrap_err() {
            NameError::Unresolved { name } => {
                assert_eq!(name, "nonexistent");
            }
            _ => panic!("Expected Unresolved error"),
        }
    }

    #[test]
    fn test_can_resolve_method() {
        let (graph, root, _) = create_test_graph();
        let import_bindings: HashMap<ModuleId, BindingTable> = HashMap::new();

        let mut binder = NameBinder::new(&graph, &import_bindings);
        binder.bind_local("x");

        assert!(binder.can_resolve("x", root));
        assert!(!binder.can_resolve("y", root));
    }

    // =========================================================================
    // Scope Depth Tests
    // =========================================================================

    #[test]
    fn test_scope_depth_tracking() {
        let (graph, _, _) = create_test_graph();
        let import_bindings: HashMap<ModuleId, BindingTable> = HashMap::new();

        let mut binder = NameBinder::new(&graph, &import_bindings);
        assert_eq!(binder.scope_depth(), 1); // Root scope exists

        binder.enter_scope();
        assert_eq!(binder.scope_depth(), 2);

        binder.enter_scope();
        assert_eq!(binder.scope_depth(), 3);

        binder.exit_scope();
        assert_eq!(binder.scope_depth(), 2);

        binder.exit_scope();
        assert_eq!(binder.scope_depth(), 1);

        // Cannot go below 1
        binder.exit_scope();
        assert_eq!(binder.scope_depth(), 1);
    }
}
