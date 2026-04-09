//! Capability resolution pipeline integration.
//!
//! This module integrates module/import resolution with capability symbol resolution,
//! building a shared `CapabilityResolutionContext` that is passed to lowering and type checking.

use ash_core::module_graph::{ModuleGraph, ModuleId};
use std::collections::HashMap;

use crate::capability_export::{CapabilityExport, CapabilityResolutionContext};
use crate::import_resolver::{BindingTable, ImportResolver};
use crate::surface::Visibility;

/// Pipeline that builds capability resolution context from module/import metadata.
///
/// This is the integration point between:
/// - Module graph construction (which module contains what)
/// - Import resolution (what names are visible where)
/// - Capability export metadata (what (provider, action) targets symbolic names resolve to)
pub struct CapabilityPipeline<'a> {
    /// Module graph for the crate (retained for future use)
    #[allow(dead_code)]
    module_graph: &'a ModuleGraph,
    import_resolver: ImportResolver<'a>,
    /// Per-module capability exports (from capability declarations)
    module_capability_exports: HashMap<ModuleId, Vec<CapabilityExport>>,
}

impl<'a> CapabilityPipeline<'a> {
    /// Create a new capability pipeline for the given module graph.
    pub fn new(module_graph: &'a ModuleGraph) -> Self {
        Self {
            module_graph,
            import_resolver: ImportResolver::new(module_graph),
            module_capability_exports: HashMap::new(),
        }
    }

    /// Register capability exports for a module.
    ///
    /// This should be called during module parsing to register operational
    /// capability declarations with their (provider, action) targets.
    pub fn register_capability_exports(
        &mut self,
        module_id: ModuleId,
        exports: Vec<CapabilityExport>,
    ) {
        self.module_capability_exports.insert(module_id, exports);
    }

    /// Register use statements for a module.
    ///
    /// Delegates to the import resolver.
    pub fn add_module_uses(&mut self, module_id: ModuleId, uses: Vec<crate::use_tree::Use>) {
        self.import_resolver.add_module_uses(module_id, uses);
    }

    /// Build capability resolution contexts for all modules.
    ///
    /// This:
    /// 1. Resolves all imports
    /// 2. Builds per-module capability resolution contexts
    /// 3. Returns a map from module ID to its resolution context
    pub fn build_resolution_contexts(
        mut self,
    ) -> Result<HashMap<ModuleId, CapabilityResolutionContext>, crate::import_resolver::ImportError>
    {
        // First, register capability exports as imports in the import resolver
        self.register_exports_with_resolver();

        // Resolve all imports
        let binding_tables = self.import_resolver.resolve_all()?;

        // Build capability resolution contexts from bindings and exports
        let mut contexts = HashMap::new();
        for (&module_id, bindings) in &binding_tables {
            let context = self.build_context_for_module(module_id, bindings);
            contexts.insert(module_id, context);
        }

        Ok(contexts)
    }

    /// Register capability exports with the import resolver.
    ///
    /// This makes capability symbols visible for import resolution.
    fn register_exports_with_resolver(&mut self) {
        for (&module_id, exports) in &self.module_capability_exports {
            let export_tuples: Vec<(String, Visibility, String, String)> = exports
                .iter()
                .map(|e| {
                    (
                        e.visible_name.to_string(),
                        e.visibility.clone(),
                        e.target_provider.to_string(),
                        e.target_action.to_string(),
                    )
                })
                .collect();

            self.import_resolver
                .add_capability_exports(module_id, export_tuples);
        }
    }

    /// Build a capability resolution context for a single module.
    fn build_context_for_module(
        &self,
        module_id: ModuleId,
        bindings: &BindingTable,
    ) -> CapabilityResolutionContext {
        let mut context = CapabilityResolutionContext::new();

        // Register local capability exports
        if let Some(exports) = self.module_capability_exports.get(&module_id) {
            for export in exports {
                context.register(export);
            }
        }

        // Register imported capability bindings (scoped to this module)
        for (local_name, binding) in bindings {
            if let Some((provider, action)) = &binding.capability_target {
                context.register_import(
                    module_id,             // The module that is importing
                    local_name,            // The local name in the importing module
                    binding.target_module, // The source module
                    &binding.item_name,    // The original name in source
                    (provider.clone().into(), action.clone().into()),
                );
            }
        }

        context
    }
}

/// Result of the capability pipeline containing resolution contexts for all modules.
pub struct CapabilityPipelineResult {
    /// Resolution context per module
    pub contexts: HashMap<ModuleId, CapabilityResolutionContext>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability_export::{CapabilityEffect, ModuleId as ExportModuleId};
    use ash_core::module_graph::ModuleNode;

    fn test_name(s: &str) -> crate::surface::Name {
        s.into()
    }

    #[test]
    fn test_pipeline_builds_context_from_exports() {
        let mut graph = ModuleGraph::new();

        let root = graph.add_node(ModuleNode::new(
            "crate".to_string(),
            ash_core::module_graph::ModuleSource::File("main.ash".to_string()),
        ));
        graph.set_root(root);

        let io_mod = graph.add_node(ModuleNode::new(
            "io".to_string(),
            ash_core::module_graph::ModuleSource::File("io.ash".to_string()),
        ));
        graph.add_edge(root, io_mod);

        let mut pipeline = CapabilityPipeline::new(&graph);

        // Register capability export for io module
        let export = CapabilityExport {
            visible_name: test_name("fs_read"),
            declaring_module: ExportModuleId(io_mod.0),
            target_provider: test_name("io"),
            target_action: test_name("fs_read"),
            visibility: Visibility::Public,
            effect: CapabilityEffect::Act,
        };
        pipeline.register_capability_exports(io_mod, vec![export]);

        // Build contexts
        let contexts = pipeline.build_resolution_contexts().unwrap();

        // io module should have the capability in its context
        let io_context = contexts.get(&io_mod).unwrap();
        let (provider, action) = io_context
            .resolve(ExportModuleId(io_mod.0), "fs_read")
            .unwrap();
        assert_eq!(provider.as_ref(), "io");
        assert_eq!(action.as_ref(), "fs_read");
    }

    #[test]
    fn test_pipeline_resolves_imported_capabilities() {
        let mut graph = ModuleGraph::new();

        let root = graph.add_node(ModuleNode::new(
            "crate".to_string(),
            ash_core::module_graph::ModuleSource::File("main.ash".to_string()),
        ));
        graph.set_root(root);

        let io_mod = graph.add_node(ModuleNode::new(
            "io".to_string(),
            ash_core::module_graph::ModuleSource::File("io.ash".to_string()),
        ));
        graph.add_edge(root, io_mod);

        let mut pipeline = CapabilityPipeline::new(&graph);

        // Register capability export
        let export = CapabilityExport {
            visible_name: test_name("fs_read"),
            declaring_module: ExportModuleId(io_mod.0),
            target_provider: test_name("io"),
            target_action: test_name("fs_read"),
            visibility: Visibility::Public,
            effect: CapabilityEffect::Act,
        };
        pipeline.register_capability_exports(io_mod, vec![export]);

        // Add use statement in root importing from io
        let use_path = crate::use_tree::UsePath::Simple(crate::use_tree::SimplePath {
            segments: vec!["crate".into(), "io".into(), "fs_read".into()],
        });
        let use_stmt = crate::use_tree::Use {
            path: use_path,
            alias: None,
            visibility: Visibility::Inherited,
            span: crate::token::Span::new(0, 10, 1, 1),
        };
        pipeline.add_module_uses(root, vec![use_stmt]);

        // Build contexts
        let contexts = pipeline.build_resolution_contexts().unwrap();

        // Root module should be able to resolve the imported capability
        let root_context = contexts.get(&root).unwrap();
        let (provider, action) = root_context
            .resolve(ExportModuleId(root.0), "fs_read")
            .unwrap();
        assert_eq!(provider.as_ref(), "io");
        assert_eq!(action.as_ref(), "fs_read");
    }
}
