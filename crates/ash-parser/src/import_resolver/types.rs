//! Binding and error carriers for import resolution.

use ash_core::module_graph::ModuleId;
use std::collections::HashMap;
use thiserror::Error;

use crate::capability_export::{ModuleDefinitionExport, ModuleDefinitionExportKind};
use crate::surface::Visibility;

/// A binding represents a resolved import.
#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    /// The module ID where the target item is defined.
    pub target_module: ModuleId,
    /// The name of the item being imported.
    pub item_name: String,
    /// The visibility of the item.
    pub visibility: Visibility,
    /// The kind of binding (direct, glob, etc.).
    pub kind: BindingKind,
    /// The kind of item that was imported.
    pub item_kind: BindingItemKind,
    /// For operational capabilities: the target (provider, action) pair
    pub capability_target: Option<(String, String)>,
    /// For Phase 101 capability/resource definitions: parsed module metadata.
    ///
    /// This preserves the parser/module substrate across import resolution
    /// without making the definitions executable or type-checked in Phase 101.
    pub definition_metadata: Option<ModuleDefinitionExport>,
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

/// The kind of exported item a binding refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindingItemKind {
    /// Generic legacy item with no richer parser metadata.
    Item,
    /// Legacy direct capability export carrying provider/action target metadata.
    LegacyCapability,
    /// Phase 101 capability interface definition.
    CapabilityInterface,
    /// Phase 101 capability implementation recipe definition.
    CapabilityImplementation,
    /// Phase 101 resource type definition.
    ResourceType,
}

impl From<&ModuleDefinitionExportKind> for BindingItemKind {
    fn from(kind: &ModuleDefinitionExportKind) -> Self {
        match kind {
            ModuleDefinitionExportKind::CapabilityInterface(_) => Self::CapabilityInterface,
            ModuleDefinitionExportKind::CapabilityImplementation(_) => {
                Self::CapabilityImplementation
            }
            ModuleDefinitionExportKind::ResourceType(_) => Self::ResourceType,
        }
    }
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
            item_kind: BindingItemKind::Item,
            capability_target: None,
            definition_metadata: None,
        }
    }

    /// Create a new binding with an explicit item kind.
    pub fn with_item_kind(
        target_module: ModuleId,
        item_name: impl Into<String>,
        visibility: Visibility,
        kind: BindingKind,
        item_kind: BindingItemKind,
    ) -> Self {
        Self {
            target_module,
            item_name: item_name.into(),
            visibility,
            kind,
            item_kind,
            capability_target: None,
            definition_metadata: None,
        }
    }

    /// Create a new binding with Phase 101 parsed definition metadata.
    pub fn with_definition_metadata(
        target_module: ModuleId,
        item_name: impl Into<String>,
        visibility: Visibility,
        kind: BindingKind,
        metadata: ModuleDefinitionExport,
    ) -> Self {
        let item_kind = BindingItemKind::from(&metadata.kind);
        Self {
            target_module,
            item_name: item_name.into(),
            visibility,
            kind,
            item_kind,
            capability_target: None,
            definition_metadata: Some(metadata),
        }
    }

    /// Create a new binding with capability target metadata.
    pub fn with_capability_target(
        target_module: ModuleId,
        item_name: impl Into<String>,
        visibility: Visibility,
        kind: BindingKind,
        capability_target: (String, String),
    ) -> Self {
        Self {
            target_module,
            item_name: item_name.into(),
            visibility,
            kind,
            item_kind: BindingItemKind::LegacyCapability,
            capability_target: Some(capability_target),
            definition_metadata: None,
        }
    }
}
