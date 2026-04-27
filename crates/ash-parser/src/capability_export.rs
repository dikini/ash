//! Capability export metadata for module-owned symbolic resolution.
//!
//! This module defines the metadata structures that represent capability declarations
//! as exported symbolic operational targets with canonical `(provider, action)` pairs.
//!
//! The export metadata is used by:
//! - Module resolution to build the capability resolution context
//! - Import resolution to resolve capability symbols across modules
//! - Lowering to convert symbolic capability calls to explicit `(provider, action)` targets

use crate::surface::{
    CapabilityDef, CapabilityImplementationDef, CapabilityImplementationDependency,
    CapabilityImplementationOperation, CapabilityInterfaceDef, CapabilityOperationSig, Name,
    ResourceField, ResourceTypeDef, Visibility,
};

/// Metadata for an exported capability symbol.
///
/// This structure carries the information needed to resolve symbolic operational
/// capability calls to their canonical `(provider, action)` targets.
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityExport {
    /// The symbolic name visible to callers (e.g., "fs_read")
    pub visible_name: Name,
    /// The module where this capability is declared
    pub declaring_module: ModuleId,
    /// Target provider name for operational dispatch (e.g., "io")
    pub target_provider: Name,
    /// Target action name for operational dispatch (e.g., "fs_read")
    pub target_action: Name,
    /// Visibility of the export
    pub visibility: Visibility,
    /// Effect type (observe, set, send, receive, act)
    pub effect: CapabilityEffect,
}

/// Re-export ModuleId from ash_core for consistency.
pub use ash_core::module_graph::ModuleId;

/// Metadata for a Phase 101 module definition export.
///
/// This is intentionally separate from [`CapabilityExport`]. `CapabilityExport`
/// carries legacy direct operational capability provider/action targets; this
/// substrate carries parsed capability interface, capability implementation, and
/// resource type definitions for later Phase 102 semantic processing.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDefinitionExport {
    /// The name visible to importers.
    pub visible_name: Name,
    /// The module where this definition was declared.
    pub declaring_module: ModuleId,
    /// Visibility of the exported definition.
    pub visibility: Visibility,
    /// Parsed metadata for the definition kind.
    pub kind: ModuleDefinitionExportKind,
}

/// Phase 101 definition kinds exported through the module metadata substrate.
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleDefinitionExportKind {
    /// Capability interface metadata.
    CapabilityInterface(CapabilityInterfaceExport),
    /// Capability implementation recipe metadata.
    CapabilityImplementation(CapabilityImplementationExport),
    /// Resource type metadata.
    ResourceType(ResourceTypeExport),
}

/// Export metadata for a capability interface definition.
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityInterfaceExport {
    /// Operation signatures declared by the interface.
    pub operations: Vec<CapabilityOperationSig>,
}

/// Export metadata for a capability implementation recipe definition.
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityImplementationExport {
    /// Target capability interface name.
    pub interface: Name,
    /// Explicit dependencies required by the recipe.
    pub dependencies: Vec<CapabilityImplementationDependency>,
    /// Operation bodies supplied by the recipe.
    pub operations: Vec<CapabilityImplementationOperation>,
}

/// Export metadata for a resource type definition.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceTypeExport {
    /// Fields declared by the resource type.
    pub fields: Vec<ResourceField>,
}

impl ModuleDefinitionExport {
    /// Build export metadata from a capability interface definition.
    pub fn from_capability_interface(def: &CapabilityInterfaceDef, module_id: ModuleId) -> Self {
        Self {
            visible_name: def.name.clone(),
            declaring_module: module_id,
            visibility: def.visibility.clone(),
            kind: ModuleDefinitionExportKind::CapabilityInterface(CapabilityInterfaceExport {
                operations: def.operations.clone(),
            }),
        }
    }

    /// Build export metadata from a capability implementation definition.
    pub fn from_capability_implementation(
        def: &CapabilityImplementationDef,
        module_id: ModuleId,
    ) -> Self {
        Self {
            visible_name: def.name.clone(),
            declaring_module: module_id,
            visibility: def.visibility.clone(),
            kind: ModuleDefinitionExportKind::CapabilityImplementation(
                CapabilityImplementationExport {
                    interface: def.interface.clone(),
                    dependencies: def.dependencies.clone(),
                    operations: def.operations.clone(),
                },
            ),
        }
    }

    /// Build export metadata from a resource type definition.
    pub fn from_resource_type(def: &ResourceTypeDef, module_id: ModuleId) -> Self {
        Self {
            visible_name: def.name.clone(),
            declaring_module: module_id,
            visibility: def.visibility.clone(),
            kind: ModuleDefinitionExportKind::ResourceType(ResourceTypeExport {
                fields: def.fields.clone(),
            }),
        }
    }

    /// Check if this export is visible from the given module.
    pub fn is_visible_from(&self, from_module: ModuleId) -> bool {
        match &self.visibility {
            Visibility::Public => true,
            Visibility::Crate => true,
            Visibility::Super { .. } => false,
            Visibility::Self_ => self.declaring_module == from_module,
            Visibility::Restricted { .. } => false,
            Visibility::Inherited => self.declaring_module == from_module,
        }
    }
}

/// Collection of Phase 101 definition exports for a module.
#[derive(Debug, Clone, Default)]
pub struct ModuleDefinitionExports {
    exports: Vec<ModuleDefinitionExport>,
}

impl ModuleDefinitionExports {
    /// Create an empty export collection.
    pub fn new() -> Self {
        Self {
            exports: Vec::new(),
        }
    }

    /// Add a definition export.
    pub fn add(&mut self, export: ModuleDefinitionExport) {
        self.exports.push(export);
    }

    /// Find an export by visible name.
    pub fn find_by_name(&self, name: &str) -> Option<&ModuleDefinitionExport> {
        self.exports
            .iter()
            .find(|export| export.visible_name.as_ref() == name)
    }

    /// Get all exports.
    pub fn all(&self) -> &[ModuleDefinitionExport] {
        &self.exports
    }

    /// Check if there are any exports.
    pub fn is_empty(&self) -> bool {
        self.exports.is_empty()
    }

    /// Get the number of exports.
    pub fn len(&self) -> usize {
        self.exports.len()
    }
}

/// Effect classification for capabilities.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CapabilityEffect {
    /// Input observation (epistemic)
    Observe,
    /// State setting (operational)
    Set,
    /// Stream receive (epistemic)
    Receive,
    /// Stream send (operational)
    Send,
    /// Operational action
    Act,
}

impl CapabilityExport {
    /// Create a new capability export from a capability definition.
    ///
    /// Returns `None` if the capability definition does not have explicit
    /// target metadata (i.e., it's not an operational capability with a
    /// defined provider/action pair).
    pub fn from_definition(def: &CapabilityDef, module_id: ModuleId) -> Option<Self> {
        // Only operational capabilities need target metadata for symbolic resolution
        if !matches!(
            def.effect,
            crate::surface::EffectType::Act
                | crate::surface::EffectType::External
                | crate::surface::EffectType::Write
        ) {
            return None;
        }

        // Get the target pair from the definition
        // If no explicit target is defined, return None - do not manufacture
        // a default target as that would recreate bridge behavior
        let (target_provider, target_action) = def.target()?;

        Some(Self {
            visible_name: def.name.clone(),
            declaring_module: module_id,
            target_provider,
            target_action,
            visibility: def.visibility.clone(),
            effect: CapabilityEffect::from_surface(def.effect),
        })
    }

    /// Get the canonical target pair for this capability.
    pub fn target(&self) -> (Name, Name) {
        (self.target_provider.clone(), self.target_action.clone())
    }

    /// Check if this export is visible from the given module.
    pub fn is_visible_from(&self, from_module: ModuleId) -> bool {
        match &self.visibility {
            Visibility::Public => true,
            Visibility::Crate => true,         // Simplified: crate-public
            Visibility::Super { .. } => false, // Would need parent relationship
            Visibility::Self_ => self.declaring_module == from_module,
            Visibility::Restricted { .. } => false, // Would need path checking
            Visibility::Inherited => self.declaring_module == from_module,
        }
    }
}

impl CapabilityEffect {
    /// Convert from surface AST effect type.
    fn from_surface(effect: crate::surface::EffectType) -> Self {
        use crate::surface::EffectType;
        match effect {
            EffectType::Observe | EffectType::Read | EffectType::Epistemic => Self::Observe,
            EffectType::Write | EffectType::Operational => Self::Set,
            EffectType::Act | EffectType::External => Self::Act,
            EffectType::Deliberative
            | EffectType::Evaluative
            | EffectType::Analyze
            | EffectType::Decide => {
                // These are not capability effects, default to Act for safety
                Self::Act
            }
        }
    }
}

/// Collection of capability exports for a module.
#[derive(Debug, Clone, Default)]
pub struct CapabilityExports {
    exports: Vec<CapabilityExport>,
}

impl CapabilityExports {
    /// Create an empty export collection.
    pub fn new() -> Self {
        Self {
            exports: Vec::new(),
        }
    }

    /// Add a capability export.
    pub fn add(&mut self, export: CapabilityExport) {
        self.exports.push(export);
    }

    /// Find an export by visible name.
    pub fn find_by_name(&self, name: &str) -> Option<&CapabilityExport> {
        self.exports
            .iter()
            .find(|e| e.visible_name.as_ref() == name)
    }

    /// Get all exports.
    pub fn all(&self) -> &[CapabilityExport] {
        &self.exports
    }

    /// Check if there are any exports.
    pub fn is_empty(&self) -> bool {
        self.exports.is_empty()
    }

    /// Get the number of exports.
    pub fn len(&self) -> usize {
        self.exports.len()
    }
}

/// Resolution context built from capability exports across modules.
///
/// This is the authoritative source for symbolic capability resolution
/// during lowering and type checking.
#[derive(Debug, Clone, Default)]
pub struct CapabilityResolutionContext {
    /// Map from (module_id, visible_name) → (provider, action)
    /// This stores local capability declarations for each module
    local_resolutions: std::collections::HashMap<(ModuleId, String), (Name, Name)>,
    /// Map from (for_module, local_name) → (source_module, provider, action)
    /// This stores imported capability bindings, properly scoped by the importing module
    import_resolutions: std::collections::HashMap<(ModuleId, String), (ModuleId, Name, Name)>,
    /// Map from module name → ModuleId for qualified resolution.
    /// This allows looking up modules by their string name during lowering.
    module_name_to_id: std::collections::HashMap<String, ModuleId>,
}

impl CapabilityResolutionContext {
    /// Create a new empty resolution context.
    pub fn new() -> Self {
        Self {
            local_resolutions: std::collections::HashMap::new(),
            import_resolutions: std::collections::HashMap::new(),
            module_name_to_id: std::collections::HashMap::new(),
        }
    }

    /// Register a module name to ModuleId mapping for qualified resolution.
    ///
    /// This allows qualified capability calls like `module::capability()` to be
    /// resolved by looking up the module name string to get its ModuleId.
    pub fn register_module_name(&mut self, name: &str, module_id: ModuleId) {
        self.module_name_to_id.insert(name.to_string(), module_id);
    }

    /// Register a capability export in the resolution context.
    pub fn register(&mut self, export: &CapabilityExport) {
        let key = (export.declaring_module, export.visible_name.to_string());
        self.local_resolutions.insert(key, export.target());
    }

    /// Register an import alias for a specific module.
    ///
    /// # Arguments
    /// * `for_module` - The module that is importing the capability
    /// * `local_name` - The name used in the importing module
    /// * `source_module` - The module where the capability is defined
    /// * `_source_name` - The original name in the source module (for reference)
    /// * `target` - The (provider, action) target pair
    pub fn register_import(
        &mut self,
        for_module: ModuleId,
        local_name: &str,
        source_module: ModuleId,
        _source_name: &str,
        target: (Name, Name),
    ) {
        let key = (for_module, local_name.to_string());
        self.import_resolutions
            .insert(key, (source_module, target.0, target.1));
    }

    /// Resolve a capability name to its target pair.
    ///
    /// First checks imports for the requesting module, then checks local declarations.
    pub fn resolve(&self, module_id: ModuleId, name: &str) -> Option<(Name, Name)> {
        // First check import resolutions for this specific module
        let import_key = (module_id, name.to_string());
        if let Some((_, provider, action)) = self.import_resolutions.get(&import_key) {
            return Some((provider.clone(), action.clone()));
        }

        // Then check local resolutions (declarations in the same module)
        let local_key = (module_id, name.to_string());
        self.local_resolutions.get(&local_key).cloned()
    }

    /// Resolve a module-qualified capability name.
    pub fn resolve_qualified(
        &self,
        module_id: ModuleId,
        capability_name: &str,
    ) -> Option<(Name, Name)> {
        let key = (module_id, capability_name.to_string());
        self.local_resolutions.get(&key).cloned()
    }

    /// Check if a name is resolvable.
    pub fn can_resolve(&self, module_id: ModuleId, name: &str) -> bool {
        self.resolve(module_id, name).is_some()
    }

    /// Resolve an unqualified capability name for lowering.
    ///
    /// This method is module-scoped - it only looks up capabilities that are
    /// visible in the current module (via imports or local declarations).
    ///
    /// # Arguments
    /// * `current_module` - The module from which to resolve the name
    /// * `name` - The unqualified capability name to resolve
    ///
    /// Returns the (provider, action) pair as strings for lowering, or None if not found.
    pub fn resolve_unqualified(
        &self,
        current_module: ModuleId,
        name: &str,
    ) -> Option<(String, String)> {
        // First check import_resolutions for (current_module, name)
        let import_key = (current_module, name.to_string());
        if let Some((_, provider, action)) = self.import_resolutions.get(&import_key) {
            return Some((provider.to_string(), action.to_string()));
        }

        // Then check local_resolutions for (current_module, name)
        let local_key = (current_module, name.to_string());
        if let Some((provider, action)) = self.local_resolutions.get(&local_key) {
            return Some((provider.to_string(), action.to_string()));
        }

        None
    }

    /// Resolve a qualified capability name for lowering.
    ///
    /// This method resolves module-qualified capability calls like `module::capability()`
    /// by first looking up the module name to get its ModuleId, then using the
    /// dedicated qualified resolution API.
    ///
    /// # Arguments
    /// * `module_name` - The module name as a string (e.g., "fs")
    /// * `capability_name` - The capability name within that module
    ///
    /// Returns the (provider, action) pair as strings for lowering, or None if not found.
    pub fn resolve_qualified_to_strings(
        &self,
        module_name: &str,
        capability_name: &str,
    ) -> Option<(String, String)> {
        // Look up the module ID from the module name
        let target_module = self.module_name_to_id.get(module_name)?;

        // Use the dedicated qualified resolution API
        self.resolve_qualified(*target_module, capability_name)
            .map(|(provider, action)| (provider.to_string(), action.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_name(s: &str) -> Name {
        s.into()
    }

    #[test]
    fn test_capability_export_from_definition() {
        let def = CapabilityDef {
            visibility: Visibility::Public,
            name: test_name("fs_read"),
            effect: crate::surface::EffectType::Act,
            params: vec![],
            return_type: None,
            constraints: vec![],
            target_provider: Some(test_name("io")),
            target_action: Some(test_name("fs_read")),
            span: crate::token::Span::new(0, 100, 1, 1),
        };

        let module_id = ModuleId(1);
        let export = CapabilityExport::from_definition(&def, module_id);

        assert!(export.is_some());
        let export = export.unwrap();
        assert_eq!(export.visible_name.as_ref(), "fs_read");
        assert_eq!(export.target_provider.as_ref(), "io");
        assert_eq!(export.target_action.as_ref(), "fs_read");
    }

    #[test]
    fn test_capability_export_no_target_returns_none() {
        let def = CapabilityDef {
            visibility: Visibility::Public,
            name: test_name("custom_cap"),
            effect: crate::surface::EffectType::Act,
            params: vec![],
            return_type: None,
            constraints: vec![],
            target_provider: None,
            target_action: None,
            span: crate::token::Span::new(0, 100, 1, 1),
        };

        let module_id = ModuleId(1);
        let export = CapabilityExport::from_definition(&def, module_id);

        // When no explicit target is defined, return None - no default target
        assert!(export.is_none());
    }

    #[test]
    fn test_non_operational_capabilities_not_exported() {
        let def = CapabilityDef {
            visibility: Visibility::Public,
            name: test_name("observe_something"),
            effect: crate::surface::EffectType::Observe,
            params: vec![],
            return_type: None,
            constraints: vec![],
            target_provider: None,
            target_action: None,
            span: crate::token::Span::new(0, 100, 1, 1),
        };

        let module_id = ModuleId(1);
        let export = CapabilityExport::from_definition(&def, module_id);

        // Non-operational capabilities don't need symbolic resolution
        assert!(export.is_none());
    }

    #[test]
    fn test_resolution_context() {
        let mut context = CapabilityResolutionContext::new();
        let module_id = ModuleId(1);

        let export = CapabilityExport {
            visible_name: test_name("fs_read"),
            declaring_module: module_id,
            target_provider: test_name("io"),
            target_action: test_name("fs_read"),
            visibility: Visibility::Public,
            effect: CapabilityEffect::Act,
        };

        context.register(&export);

        assert!(context.can_resolve(module_id, "fs_read"));
        let (provider, action) = context.resolve(module_id, "fs_read").unwrap();
        assert_eq!(provider.as_ref(), "io");
        assert_eq!(action.as_ref(), "fs_read");

        assert!(!context.can_resolve(module_id, "unknown"));
    }

    #[test]
    fn test_import_registration() {
        let mut context = CapabilityResolutionContext::new();
        let source_module = ModuleId(1);
        let importing_module = ModuleId(2);

        // Register import for a specific module
        context.register_import(
            importing_module, // The module that is importing
            "read_file",      // The local name in the importing module
            source_module,    // The source module
            "fs_read",        // The original name in source
            (test_name("io"), test_name("fs_read")),
        );

        // Should resolve in the importing module
        let result = context.resolve(importing_module, "read_file");
        assert!(result.is_some());

        let (provider, action) = result.unwrap();
        assert_eq!(provider.as_ref(), "io");
        assert_eq!(action.as_ref(), "fs_read");

        // Should NOT resolve in a different module
        let other_module = ModuleId(3);
        let result_other = context.resolve(other_module, "read_file");
        assert!(result_other.is_none());
    }

    #[test]
    fn test_resolve_unqualified_different_modules() {
        // TASK-480: Demonstrate that two modules can resolve the same unqualified
        // symbol differently based on their own imports/declarations
        let mut context = CapabilityResolutionContext::new();
        let module_a = ModuleId(1);
        let module_b = ModuleId(2);

        // Module A declares "fs_read" -> (io, fs_read)
        let export_a = CapabilityExport {
            visible_name: test_name("fs_read"),
            declaring_module: module_a,
            target_provider: test_name("io"),
            target_action: test_name("fs_read"),
            visibility: Visibility::Public,
            effect: CapabilityEffect::Act,
        };
        context.register(&export_a);

        // Module B imports "fs_read" as "file_access" -> (storage, read)
        context.register_import(
            module_b,
            "fs_read", // Same unqualified name as in module_a
            module_a,
            "fs_read",
            (test_name("storage"), test_name("read")),
        );

        // Module A resolves "fs_read" -> (io, fs_read)
        let result_a = context.resolve(module_a, "fs_read");
        assert!(result_a.is_some());
        let (provider_a, action_a) = result_a.unwrap();
        assert_eq!(provider_a.as_ref(), "io");
        assert_eq!(action_a.as_ref(), "fs_read");

        // Module B resolves "fs_read" -> (storage, read) - different target!
        let result_b = context.resolve(module_b, "fs_read");
        assert!(result_b.is_some());
        let (provider_b, action_b) = result_b.unwrap();
        assert_eq!(provider_b.as_ref(), "storage");
        assert_eq!(action_b.as_ref(), "read");

        // The same unqualified name resolves to different targets in different modules
        assert_ne!(
            (provider_a.as_ref(), action_a.as_ref()),
            (provider_b.as_ref(), action_b.as_ref())
        );
    }

    #[test]
    fn test_resolve_unqualified_not_cross_module() {
        // TASK-480: Unqualified lookup should NOT find another module's symbol
        // unless explicitly imported
        let mut context = CapabilityResolutionContext::new();
        let module_a = ModuleId(1);
        let module_b = ModuleId(2);

        // Module A declares "secret_cap" with Self visibility
        let export_a = CapabilityExport {
            visible_name: test_name("secret_cap"),
            declaring_module: module_a,
            target_provider: test_name("internal"),
            target_action: test_name("secret"),
            visibility: Visibility::Self_,
            effect: CapabilityEffect::Act,
        };
        context.register(&export_a);

        // Module B does NOT import "secret_cap"

        // Module A can resolve its own capability
        assert!(context.resolve(module_a, "secret_cap").is_some());

        // Module B should NOT be able to resolve module A's unimported capability
        // This tests that unqualified lookup is module-scoped
        let result_b = context.resolve(module_b, "secret_cap");
        assert!(
            result_b.is_none(),
            "Module B should not resolve Module A's unimported capability"
        );
    }

    #[test]
    fn test_resolve_qualified_explicit() {
        // TASK-480: Qualified lookup should resolve through explicit target module
        let mut context = CapabilityResolutionContext::new();
        let module_a = ModuleId(1);
        let module_b = ModuleId(2);

        // Module A declares "public_cap"
        let export_a = CapabilityExport {
            visible_name: test_name("public_cap"),
            declaring_module: module_a,
            target_provider: test_name("provider_a"),
            target_action: test_name("action_a"),
            visibility: Visibility::Public,
            effect: CapabilityEffect::Act,
        };
        context.register(&export_a);

        // Module B declares "public_cap" with same name but different target
        let export_b = CapabilityExport {
            visible_name: test_name("public_cap"),
            declaring_module: module_b,
            target_provider: test_name("provider_b"),
            target_action: test_name("action_b"),
            visibility: Visibility::Public,
            effect: CapabilityEffect::Act,
        };
        context.register(&export_b);

        // Qualified lookup: resolve_qualified should target specific module
        // Looking up "public_cap" in module_a should return provider_a/action_a
        let result_a = context.resolve_qualified(module_a, "public_cap");
        assert!(result_a.is_some());
        let (provider_a, action_a) = result_a.unwrap();
        assert_eq!(provider_a.as_ref(), "provider_a");
        assert_eq!(action_a.as_ref(), "action_a");

        // Looking up "public_cap" in module_b should return provider_b/action_b
        let result_b = context.resolve_qualified(module_b, "public_cap");
        assert!(result_b.is_some());
        let (provider_b, action_b) = result_b.unwrap();
        assert_eq!(provider_b.as_ref(), "provider_b");
        assert_eq!(action_b.as_ref(), "action_b");

        // They should be different
        assert_ne!(
            (provider_a.as_ref(), action_a.as_ref()),
            (provider_b.as_ref(), action_b.as_ref())
        );
    }

    #[test]
    fn test_resolve_for_lowering_module_scoped() {
        // TASK-480: Verify that resolve_unqualified() is properly module-scoped.
        //
        // The resolve_unqualified() method takes a ModuleId parameter and only
        // resolves names that are visible in that specific module (via imports
        // or local declarations).
        let mut context = CapabilityResolutionContext::new();
        let module_a = ModuleId(1);
        let module_b = ModuleId(2);

        // Both modules have "config" capability with different targets
        let export_a = CapabilityExport {
            visible_name: test_name("config"),
            declaring_module: module_a,
            target_provider: test_name("env"),
            target_action: test_name("get"),
            visibility: Visibility::Public,
            effect: CapabilityEffect::Act,
        };
        context.register(&export_a);

        let export_b = CapabilityExport {
            visible_name: test_name("config"),
            declaring_module: module_b,
            target_provider: test_name("file"),
            target_action: test_name("read"),
            visibility: Visibility::Public,
            effect: CapabilityEffect::Act,
        };
        context.register(&export_b);

        // Module A should resolve "config" to (env, get)
        assert_eq!(
            context.resolve_unqualified(module_a, "config"),
            Some(("env".to_string(), "get".to_string())),
            "Module A should resolve 'config' to (env, get)"
        );

        // Module B should resolve "config" to (file, read)
        assert_eq!(
            context.resolve_unqualified(module_b, "config"),
            Some(("file".to_string(), "read".to_string())),
            "Module B should resolve 'config' to (file, read)"
        );
    }

    #[test]
    fn test_resolve_qualified_to_strings() {
        // Phase 72: Test the resolve_qualified_to_strings method that allows
        // qualified capability resolution using module name strings.
        let mut context = CapabilityResolutionContext::new();
        let module_a = ModuleId(1);
        let module_b = ModuleId(2);

        // Register module names for lookup
        context.register_module_name("fs", module_a);
        context.register_module_name("net", module_b);

        // Module "fs" declares "read" capability
        let export_a = CapabilityExport {
            visible_name: test_name("read"),
            declaring_module: module_a,
            target_provider: test_name("io"),
            target_action: test_name("fs_read"),
            visibility: Visibility::Public,
            effect: CapabilityEffect::Act,
        };
        context.register(&export_a);

        // Module "net" declares "read" capability (same name, different target)
        let export_b = CapabilityExport {
            visible_name: test_name("read"),
            declaring_module: module_b,
            target_provider: test_name("socket"),
            target_action: test_name("recv"),
            visibility: Visibility::Public,
            effect: CapabilityEffect::Act,
        };
        context.register(&export_b);

        // Qualified lookup by module name: fs::read should resolve to (io, fs_read)
        let result_a = context.resolve_qualified_to_strings("fs", "read");
        assert!(result_a.is_some());
        let (provider_a, action_a) = result_a.unwrap();
        assert_eq!(provider_a, "io");
        assert_eq!(action_a, "fs_read");

        // Qualified lookup by module name: net::read should resolve to (socket, recv)
        let result_b = context.resolve_qualified_to_strings("net", "read");
        assert!(result_b.is_some());
        let (provider_b, action_b) = result_b.unwrap();
        assert_eq!(provider_b, "socket");
        assert_eq!(action_b, "recv");

        // Unknown module name should return None
        let result_unknown = context.resolve_qualified_to_strings("unknown", "read");
        assert!(result_unknown.is_none());

        // Unknown capability in known module should return None
        let result_unknown_cap = context.resolve_qualified_to_strings("fs", "write");
        assert!(result_unknown_cap.is_none());
    }
}
