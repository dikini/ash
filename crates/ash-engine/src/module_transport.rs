//! Canonical, non-authorizing Engine transport for checked module closures.
//!
//! This boundary accepts only already-materialized Core/CPS artifacts and
//! checked public interfaces. It validates their identity and dependency
//! closure, then exposes deterministic metadata to the later TASK-2063
//! linking boundary. It does not admit, execute, or seal anything.

use ash_core::cps::{Atom, ContRef, Term, Value};
use ash_core::module_graph::{MODULE_ARTIFACT_SCHEMA_VERSION, ModuleKey};
use ash_core::module_interface::{
    ModuleInterfaceBindingKind, ModuleInterfaceDefiningIdentity,
    PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION, PublicModuleInterface,
};
use ash_core::module_lowering::{ModuleCoreArtifact, ModuleCpsArtifact, ModuleImportVisibility};
use ash_core::semantic_summary::SourceAnchor;
use ash_typeck::module_core_cps_lowering::LoweredCheckedModuleDefinition;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
struct CheckedLocalCallableEntry {
    name: String,
    kind: ModuleInterfaceBindingKind,
    cps: ModuleCpsArtifact,
    parameter_names: Vec<String>,
}

/// One checked module artifact submitted to the Engine linking boundary.
///
/// The input is deliberately forgeable transport data. It becomes executable
/// only after the Engine validates the complete closure and creates its own
/// sealed linked admission.
#[derive(Debug, Clone, PartialEq)]
pub struct LinkedModuleArtifactInput {
    interface: PublicModuleInterface,
    core: Option<ModuleCoreArtifact>,
    cps: Option<ModuleCpsArtifact>,
    source_anchor: SourceAnchor,
    entry_name: Option<String>,
    parameter_names: Vec<String>,
    local_entries: Vec<CheckedLocalCallableEntry>,
    failure: Option<String>,
}

impl LinkedModuleArtifactInput {
    /// Creates a successful linked-module input.
    #[must_use]
    pub const fn new(
        interface: PublicModuleInterface,
        core: ModuleCoreArtifact,
        cps: ModuleCpsArtifact,
        source_anchor: SourceAnchor,
    ) -> Self {
        Self {
            interface,
            core: Some(core),
            cps: Some(cps),
            source_anchor,
            entry_name: None,
            parameter_names: Vec::new(),
            local_entries: Vec::new(),
            failure: None,
        }
    }

    /// Creates a successful input carrying the checked selected-entry name
    /// and parameter metadata needed by the non-authorizing linked-call
    /// handoff.
    #[must_use]
    pub fn with_entry_metadata(
        interface: PublicModuleInterface,
        core: ModuleCoreArtifact,
        cps: ModuleCpsArtifact,
        source_anchor: SourceAnchor,
        entry_name: impl Into<String>,
        parameter_names: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let entry_name = entry_name.into();
        let parameter_names: Vec<String> = parameter_names.into_iter().map(Into::into).collect();
        Self {
            interface,
            core: Some(core),
            cps: Some(cps.clone()),
            source_anchor,
            entry_name: Some(entry_name.clone()),
            parameter_names: parameter_names.clone(),
            local_entries: vec![CheckedLocalCallableEntry {
                name: entry_name,
                kind: ModuleInterfaceBindingKind::Callable,
                cps,
                parameter_names,
            }],
            failure: None,
        }
    }

    fn with_entry_metadata_and_local_entries(
        interface: PublicModuleInterface,
        core: ModuleCoreArtifact,
        cps: ModuleCpsArtifact,
        source_anchor: SourceAnchor,
        entry_name: impl Into<String>,
        parameter_names: impl IntoIterator<Item = impl Into<String>>,
        local_entries: Vec<CheckedLocalCallableEntry>,
    ) -> Self {
        Self {
            interface,
            core: Some(core),
            cps: Some(cps),
            source_anchor,
            entry_name: Some(entry_name.into()),
            parameter_names: parameter_names.into_iter().map(Into::into).collect(),
            local_entries,
            failure: None,
        }
    }

    /// Creates a failed linked-module result that must reject before admission.
    #[must_use]
    pub fn failed(
        interface: PublicModuleInterface,
        source_anchor: SourceAnchor,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            interface,
            core: None,
            cps: None,
            source_anchor,
            entry_name: None,
            parameter_names: Vec::new(),
            local_entries: Vec::new(),
            failure: Some(reason.into()),
        }
    }

    /// Returns the checked interface carried by this input.
    #[must_use]
    pub const fn interface(&self) -> &PublicModuleInterface {
        &self.interface
    }

    /// Returns the checked Core artifact, when lowering succeeded.
    #[must_use]
    pub const fn core(&self) -> Option<&ModuleCoreArtifact> {
        self.core.as_ref()
    }

    /// Returns the checked CPS artifact, when lowering succeeded.
    #[must_use]
    pub const fn cps(&self) -> Option<&ModuleCpsArtifact> {
        self.cps.as_ref()
    }

    /// Returns the source anchor retained for Engine diagnostics.
    #[must_use]
    pub const fn source_anchor(&self) -> &SourceAnchor {
        &self.source_anchor
    }

    /// Returns the selected checked entry name, when this input came from
    /// entry-oriented lowering.
    #[must_use]
    pub fn entry_name(&self) -> Option<&str> {
        self.entry_name.as_deref()
    }

    /// Returns checker-retained parameter names for the selected entry.
    #[must_use]
    pub fn parameter_names(&self) -> &[String] {
        &self.parameter_names
    }

    /// Returns the checker failure, if this input failed.
    #[must_use]
    pub fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }

    pub(crate) fn into_checked_transport_input(self) -> CheckedModuleArtifactInput {
        CheckedModuleArtifactInput {
            interface: self.interface,
            core: self.core,
            cps: self.cps,
            entry_name: self.entry_name,
            parameter_names: self.parameter_names,
            local_entries: self.local_entries,
            failure: self.failure,
        }
    }
}

/// A forgeable, source-independent checked module closure submitted for
/// Engine linking.
#[derive(Debug, Clone, PartialEq)]
pub struct LinkedModuleClosure {
    root: ModuleKey,
    modules: Vec<LinkedModuleArtifactInput>,
}

impl LinkedModuleClosure {
    /// Creates a candidate closure rooted at `root`.
    #[must_use]
    pub const fn new(root: ModuleKey, modules: Vec<LinkedModuleArtifactInput>) -> Self {
        Self { root, modules }
    }

    /// Returns the requested canonical root identity.
    #[must_use]
    pub const fn root(&self) -> &ModuleKey {
        &self.root
    }

    /// Returns candidate module inputs in caller order.
    #[must_use]
    pub fn modules(&self) -> &[LinkedModuleArtifactInput] {
        &self.modules
    }

    pub(crate) fn into_parts(self) -> (ModuleKey, Vec<LinkedModuleArtifactInput>) {
        (self.root, self.modules)
    }
}

/// Failure while converting checked entry lowering into a forgeable Engine
/// closure. This boundary is non-authorizing; Engine admission validates the
/// resulting closure again before execution.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LinkedModuleClosureBuildError {
    /// No checked module entries were supplied.
    #[error("checked entry lowering is empty")]
    Empty,
    /// Two lowered entries claim the same canonical module.
    #[error("checked entry lowering contains duplicate module {module}")]
    DuplicateLoweredModule {
        /// Duplicated canonical module identity.
        module: ModuleKey,
    },
    /// Two interfaces claim the same canonical module.
    #[error("checked interface closure contains duplicate module {module}")]
    DuplicateInterface {
        /// Duplicated canonical module identity.
        module: ModuleKey,
    },
    /// A lowered module has no matching checked interface.
    #[error("checked module {module} has no matching public interface")]
    MissingInterface {
        /// Module whose interface is absent.
        module: ModuleKey,
    },
    /// A checked interface has no matching lowered module artifact.
    #[error("checked interface {module} has no matching lowered module")]
    UnexpectedInterface {
        /// Module whose interface is not represented by a lowered artifact.
        module: ModuleKey,
    },
    /// A lowered module has no diagnostic source anchor.
    #[error("checked module {module} has no source anchor")]
    MissingSourceAnchor {
        /// Module whose source anchor is absent.
        module: ModuleKey,
    },
    /// An anchor was supplied for a module that was not lowered.
    #[error("source anchor supplied for non-lowered module {module}")]
    UnexpectedSourceAnchor {
        /// Unexpected canonical module identity.
        module: ModuleKey,
    },
    /// Two lowered definitions claim the same canonical module/name pair.
    #[error("checked module {module} has duplicate lowered definition {name:?}")]
    DuplicateLoweredDefinition {
        /// Module containing the duplicate definition.
        module: ModuleKey,
        /// Duplicated definition name.
        name: String,
    },
    /// A module has no selected entry for the local callable closure.
    #[error("checked module {module} has no selected lowered entry")]
    MissingSelectedEntry {
        /// Module missing its selected entry.
        module: ModuleKey,
    },
    /// A selected entry names a module absent from the lowered closure.
    #[error("selected lowered entry names unexpected module {module}")]
    UnexpectedSelectedEntry {
        /// Unexpected selected-entry module.
        module: ModuleKey,
    },
    /// A selected entry names a definition absent from its lowered module.
    #[error("checked module {module} has no lowered definition {name:?}")]
    MissingSelectedDefinition {
        /// Module missing the selected definition.
        module: ModuleKey,
        /// Selected definition name.
        name: String,
    },
    /// The interface and lowered artifacts disagree about canonical identity.
    #[error("checked module {module} lowered artifact mismatch: {fact}")]
    ArtifactMismatch {
        /// Module carrying the mismatch.
        module: ModuleKey,
        /// Mismatched fact category.
        fact: &'static str,
    },
    /// The requested root is absent from the lowered closure.
    #[error("checked entry lowering root {root} is absent")]
    MissingRoot {
        /// Requested canonical root identity.
        root: ModuleKey,
    },
}

/// Converts one selected checked entry per canonical module into the Engine's
/// forgeable linked-closure carrier.
///
/// The conversion is atomic: duplicate or missing module/interface/anchor
/// facts reject before a closure is returned. The resulting carrier remains
/// non-authorizing and must still pass [`CheckedModuleTransport::new`] through
/// the Engine admission path.
///
/// # Errors
///
/// Returns [`LinkedModuleClosureBuildError`] when the lowered entries,
/// interfaces, identities, or source-anchor map are incomplete or disagree.
pub fn linked_module_closure_from_checked_entry_lowering(
    root: ModuleKey,
    lowered: Vec<LoweredCheckedModuleDefinition>,
    interfaces: Vec<PublicModuleInterface>,
    source_anchors: &std::collections::BTreeMap<ModuleKey, SourceAnchor>,
) -> Result<LinkedModuleClosure, LinkedModuleClosureBuildError> {
    if lowered.is_empty() {
        return Err(LinkedModuleClosureBuildError::Empty);
    }

    let mut interface_by_key = std::collections::BTreeMap::new();
    for interface in interfaces {
        let key = interface.artifact().key().clone();
        if interface_by_key.insert(key.clone(), interface).is_some() {
            return Err(LinkedModuleClosureBuildError::DuplicateInterface { module: key });
        }
    }

    let mut lowered_keys = std::collections::BTreeSet::new();
    let mut modules = Vec::with_capacity(lowered.len());
    for definition in lowered {
        let key = definition.core().module_artifact().key().clone();
        if !lowered_keys.insert(key.clone()) {
            return Err(LinkedModuleClosureBuildError::DuplicateLoweredModule { module: key });
        }
        let interface = interface_by_key.get(&key).ok_or_else(|| {
            LinkedModuleClosureBuildError::MissingInterface {
                module: key.clone(),
            }
        })?;
        if interface.artifact() != definition.core().module_artifact() {
            return Err(LinkedModuleClosureBuildError::ArtifactMismatch {
                module: key,
                fact: "public interface/Core artifact identity or origin",
            });
        }
        if interface.artifact() != definition.cps().module_artifact() {
            return Err(LinkedModuleClosureBuildError::ArtifactMismatch {
                module: key,
                fact: "public interface/CPS artifact identity or origin",
            });
        }
        let source_anchor = source_anchors.get(&key).cloned().ok_or_else(|| {
            LinkedModuleClosureBuildError::MissingSourceAnchor {
                module: key.clone(),
            }
        })?;
        let local_entries = if definition.is_callable_entry() {
            vec![CheckedLocalCallableEntry {
                name: definition.declaration_name().to_owned(),
                kind: ModuleInterfaceBindingKind::Callable,
                cps: definition.cps().clone(),
                parameter_names: definition.parameter_names().to_vec(),
            }]
        } else {
            Vec::new()
        };
        modules.push(
            LinkedModuleArtifactInput::with_entry_metadata_and_local_entries(
                interface.clone(),
                definition.core().clone(),
                definition.cps().clone(),
                source_anchor,
                definition.declaration_name(),
                definition.parameter_names().iter().cloned(),
                local_entries,
            ),
        );
    }

    if let Some((module, _)) = interface_by_key
        .iter()
        .find(|(module, _)| !lowered_keys.contains(*module))
    {
        return Err(LinkedModuleClosureBuildError::UnexpectedInterface {
            module: module.clone(),
        });
    }

    if !lowered_keys.contains(&root) {
        return Err(LinkedModuleClosureBuildError::MissingRoot { root });
    }
    if let Some((module, _)) = source_anchors
        .iter()
        .find(|(module, _)| !lowered_keys.contains(*module))
    {
        return Err(LinkedModuleClosureBuildError::UnexpectedSourceAnchor {
            module: module.clone(),
        });
    }

    Ok(LinkedModuleClosure::new(root, modules))
}

/// Converts checked definition lowering into one Engine closure.
///
/// The closure carries one selected entry plus checker-lowered local callable
/// siblings per canonical module. A module with no standalone callable body
/// may instead carry the neutral metadata-only entry produced by the
/// typechecker lowering boundary.
/// Same-module aliases and direct local calls can therefore be linked without
/// manufacturing a dependency edge or treating a private helper as a public
/// interface export. Every local callable remains checker-owned lowering data;
/// Engine admission still validates the complete carrier before execution.
///
/// # Errors
///
/// Returns a [`LinkedModuleClosureBuildError`] when definitions, selected entries,
/// interfaces, or source anchors do not form one exact canonical module closure.
#[allow(clippy::result_large_err, clippy::too_many_lines)]
pub fn linked_module_closure_from_checked_definition_lowering(
    root: ModuleKey,
    lowered: Vec<LoweredCheckedModuleDefinition>,
    selected_entries: &std::collections::BTreeMap<ModuleKey, String>,
    interfaces: Vec<PublicModuleInterface>,
    source_anchors: &std::collections::BTreeMap<ModuleKey, SourceAnchor>,
) -> Result<LinkedModuleClosure, LinkedModuleClosureBuildError> {
    if lowered.is_empty() {
        return Err(LinkedModuleClosureBuildError::Empty);
    }

    let mut interface_by_key = std::collections::BTreeMap::new();
    for interface in interfaces {
        let key = interface.artifact().key().clone();
        if interface_by_key.insert(key.clone(), interface).is_some() {
            return Err(LinkedModuleClosureBuildError::DuplicateInterface { module: key });
        }
    }

    let mut definitions_by_module =
        std::collections::BTreeMap::<ModuleKey, Vec<LoweredCheckedModuleDefinition>>::new();
    for definition in lowered {
        let module = definition.core().module_artifact().key().clone();
        let definitions = definitions_by_module.entry(module.clone()).or_default();
        if definitions
            .iter()
            .any(|candidate| candidate.declaration_name() == definition.declaration_name())
        {
            return Err(LinkedModuleClosureBuildError::DuplicateLoweredDefinition {
                module,
                name: definition.declaration_name().to_owned(),
            });
        }
        definitions.push(definition);
    }

    if let Some((module, _)) = selected_entries
        .iter()
        .find(|(module, _)| !definitions_by_module.contains_key(*module))
    {
        return Err(LinkedModuleClosureBuildError::UnexpectedSelectedEntry {
            module: module.clone(),
        });
    }
    if let Some((module, _)) = interface_by_key
        .iter()
        .find(|(module, _)| !definitions_by_module.contains_key(*module))
    {
        return Err(LinkedModuleClosureBuildError::UnexpectedInterface {
            module: module.clone(),
        });
    }
    if !definitions_by_module.contains_key(&root) {
        return Err(LinkedModuleClosureBuildError::MissingRoot { root });
    }
    if let Some((module, _)) = source_anchors
        .iter()
        .find(|(module, _)| !definitions_by_module.contains_key(module))
    {
        return Err(LinkedModuleClosureBuildError::UnexpectedSourceAnchor {
            module: module.clone(),
        });
    }

    let mut modules = Vec::with_capacity(definitions_by_module.len());
    for (module, definitions) in definitions_by_module {
        let interface = interface_by_key.get(&module).ok_or_else(|| {
            LinkedModuleClosureBuildError::MissingInterface {
                module: module.clone(),
            }
        })?;
        let selected_name = selected_entries.get(&module).ok_or_else(|| {
            LinkedModuleClosureBuildError::MissingSelectedEntry {
                module: module.clone(),
            }
        })?;
        if module == root && selected_name.is_empty() {
            return Err(LinkedModuleClosureBuildError::MissingSelectedDefinition {
                module: module.clone(),
                name: selected_name.clone(),
            });
        }
        let selected = if selected_name.is_empty() {
            definitions
                .iter()
                .find(|definition| !definition.is_callable_entry())
                .ok_or_else(
                    || LinkedModuleClosureBuildError::MissingSelectedDefinition {
                        module: module.clone(),
                        name: selected_name.clone(),
                    },
                )?
        } else {
            definitions
                .iter()
                .find(|definition| {
                    definition.is_callable_entry() && definition.declaration_name() == selected_name
                })
                .ok_or_else(
                    || LinkedModuleClosureBuildError::MissingSelectedDefinition {
                        module: module.clone(),
                        name: selected_name.clone(),
                    },
                )?
        };
        for definition in &definitions {
            if interface.artifact() != definition.core().module_artifact()
                || interface.artifact() != definition.cps().module_artifact()
            {
                return Err(LinkedModuleClosureBuildError::ArtifactMismatch {
                    module: module.clone(),
                    fact: "public interface and lowered local callable artifact identity or origin",
                });
            }
        }
        let source_anchor = source_anchors.get(&module).cloned().ok_or_else(|| {
            LinkedModuleClosureBuildError::MissingSourceAnchor {
                module: module.clone(),
            }
        })?;
        let local_entries = definitions
            .iter()
            .filter(|definition| definition.is_callable_entry())
            .map(|definition| CheckedLocalCallableEntry {
                name: definition.declaration_name().to_owned(),
                kind: ModuleInterfaceBindingKind::Callable,
                cps: definition.cps().clone(),
                parameter_names: definition.parameter_names().to_vec(),
            })
            .collect();
        modules.push(
            LinkedModuleArtifactInput::with_entry_metadata_and_local_entries(
                interface.clone(),
                selected.core().clone(),
                selected.cps().clone(),
                source_anchor,
                selected.declaration_name(),
                selected.parameter_names().iter().cloned(),
                local_entries,
            ),
        );
    }

    Ok(LinkedModuleClosure::new(root, modules))
}

/// A collision while inserting a checked transport into the canonical cache.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CheckedModuleTransportCacheError {
    /// A canonical root already has a transport entry.
    #[error("duplicate canonical root {root} in checked module transport cache")]
    DuplicateCanonicalRoot {
        /// Canonical root whose existing entry cannot be silently replaced.
        root: ModuleKey,
    },
}

/// One successful or failed checked-module result presented to Engine.
///
/// A failed entry is retained only to let the transport reject the complete
/// closure atomically; it can never become a transport module.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckedModuleArtifactInput {
    interface: PublicModuleInterface,
    core: Option<ModuleCoreArtifact>,
    cps: Option<ModuleCpsArtifact>,
    entry_name: Option<String>,
    parameter_names: Vec<String>,
    local_entries: Vec<CheckedLocalCallableEntry>,
    failure: Option<String>,
}

impl CheckedModuleArtifactInput {
    /// Creates a successful checked-module transport input.
    #[must_use]
    pub const fn new(
        interface: PublicModuleInterface,
        core: ModuleCoreArtifact,
        cps: ModuleCpsArtifact,
    ) -> Self {
        Self {
            interface,
            core: Some(core),
            cps: Some(cps),
            entry_name: None,
            parameter_names: Vec::new(),
            local_entries: Vec::new(),
            failure: None,
        }
    }

    /// Creates a failed checked-module result that must be rejected atomically.
    #[must_use]
    pub fn failed(interface: PublicModuleInterface, reason: impl Into<String>) -> Self {
        Self {
            interface,
            core: None,
            cps: None,
            entry_name: None,
            parameter_names: Vec::new(),
            local_entries: Vec::new(),
            failure: Some(reason.into()),
        }
    }

    /// Returns the checked public interface carried by this result.
    #[must_use]
    pub const fn interface(&self) -> &PublicModuleInterface {
        &self.interface
    }

    /// Returns the Core artifact when the checked result succeeded.
    #[must_use]
    pub const fn core(&self) -> Option<&ModuleCoreArtifact> {
        self.core.as_ref()
    }

    /// Returns the CPS artifact when the checked result succeeded.
    #[must_use]
    pub const fn cps(&self) -> Option<&ModuleCpsArtifact> {
        self.cps.as_ref()
    }

    /// Returns the selected checked entry name, when carried.
    #[must_use]
    pub fn entry_name(&self) -> Option<&str> {
        self.entry_name.as_deref()
    }

    /// Returns checker-retained parameter names for the selected entry.
    #[must_use]
    pub fn parameter_names(&self) -> &[String] {
        &self.parameter_names
    }

    fn local_entry(&self, name: &str) -> Option<&CheckedLocalCallableEntry> {
        self.local_entries.iter().find(|entry| entry.name == name)
    }

    /// Returns the checker failure, if this result is failed.
    #[must_use]
    pub fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }
}

/// Failure preventing publication of a canonical Engine transport closure.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum CheckedModuleTransportError {
    /// No module result was supplied.
    #[error("checked module transport closure is empty")]
    Empty,
    /// Two results claim the same canonical module identity.
    #[error("checked module transport contains duplicate module {module}")]
    DuplicateModule {
        /// Duplicated canonical identity.
        module: ModuleKey,
    },
    /// The requested root is absent from the closure.
    #[error("checked module transport root {root} is absent")]
    MissingRoot {
        /// Requested root identity.
        root: ModuleKey,
    },
    /// A checked result failed before producing Core/CPS artifacts.
    #[error("checked module {module} failed before transport: {reason}")]
    FailedModule {
        /// Failed module identity.
        module: ModuleKey,
        /// Checker failure explanation.
        reason: String,
    },
    /// The public interface and Core artifact disagree about module identity.
    #[error("checked module {module} interface/Core artifact mismatch: {fact}")]
    InterfaceCoreMismatch {
        /// Mismatched module identity.
        module: ModuleKey,
        /// Mismatched fact category.
        fact: &'static str,
    },
    /// The public interface and CPS artifact disagree about module identity.
    #[error("checked module {module} interface/CPS artifact mismatch: {fact}")]
    InterfaceCpsMismatch {
        /// Mismatched module identity.
        module: ModuleKey,
        /// Mismatched fact category.
        fact: &'static str,
    },
    /// The Core and CPS artifacts disagree about module/import provenance.
    #[error("checked module {module} Core/CPS artifact mismatch: {fact}")]
    CoreCpsMismatch {
        /// Mismatched module identity.
        module: ModuleKey,
        /// Mismatched fact category.
        fact: &'static str,
    },
    /// The interface schema is not the current checked schema.
    #[error("checked module {module} has unsupported interface schema {schema_version}")]
    UnsupportedInterfaceSchema {
        /// Module carrying the unsupported schema.
        module: ModuleKey,
        /// Schema version received from the interface.
        schema_version: u32,
    },
    /// The module artifact schema is not the current checked schema.
    #[error("checked module {module} has unsupported artifact schema {schema_version}")]
    UnsupportedArtifactSchema {
        /// Module carrying the unsupported schema.
        module: ModuleKey,
        /// Schema version received from the artifact.
        schema_version: u32,
    },
    /// A declared interface dependency is absent.
    #[error("checked module {module} is missing declared dependency {dependency}")]
    MissingDependency {
        /// Module declaring the dependency.
        module: ModuleKey,
        /// Absent dependency identity.
        dependency: ModuleKey,
    },
    /// A dependency advertises a schema different from the owning interface.
    #[error(
        "checked module {module} dependency {dependency} has schema {actual}, expected {expected}"
    )]
    DependencySchemaMismatch {
        /// Module declaring the dependency.
        module: ModuleKey,
        /// Dependency identity.
        dependency: ModuleKey,
        /// Schema version carried by the dependency edge.
        actual: u32,
        /// Schema version required by the owning interface.
        expected: u32,
    },
    /// A structural child is absent from the closure.
    #[error("checked module {module} is missing structural child {child}")]
    MissingStructuralChild {
        /// Parent module identity.
        module: ModuleKey,
        /// Absent child identity.
        child: ModuleKey,
    },
    /// A structural parent is absent from the closure.
    #[error("checked module {module} is missing structural parent {parent}")]
    MissingStructuralParent {
        /// Child module identity.
        module: ModuleKey,
        /// Absent parent identity.
        parent: ModuleKey,
    },
    /// A child names a parent that does not list it as a structural child.
    #[error("checked module {module} is not listed under structural parent {parent}")]
    StructuralParentChildMismatch {
        /// Child module identity.
        module: ModuleKey,
        /// Claimed parent identity.
        parent: ModuleKey,
    },
    /// An extra result is not reachable from the requested root.
    #[error("checked module {module} is not reachable from root {root}")]
    UnreachableModule {
        /// Requested root identity.
        root: ModuleKey,
        /// Extra module identity.
        module: ModuleKey,
    },
    /// A resolved import names a module outside the supplied closure.
    #[error("checked module {module} import targets missing module {target}")]
    MissingImportTarget {
        /// Importing module identity.
        module: ModuleKey,
        /// Defining module identity.
        target: ModuleKey,
    },
    /// A resolved import targets a module that is not declared as a dependency
    /// by the importing interface.
    #[error("checked module {module} import target {target} is not a declared dependency")]
    UndeclaredImportDependency {
        /// Importing module identity.
        module: ModuleKey,
        /// Defining module identity.
        target: ModuleKey,
    },
    /// A public declaration re-export targets an external module that is not
    /// declared as a dependency by the exporting interface.
    #[error("checked module {module} export {name:?} targets undeclared dependency {target}")]
    UndeclaredExportDependency {
        /// Exporting module identity.
        module: ModuleKey,
        /// Visible exported name.
        name: String,
        /// Defining module identity.
        target: ModuleKey,
    },
    /// A resolved import carries an origin different from its defining module.
    #[error("checked module {module} import target {target} has mismatched origin")]
    ImportOriginMismatch {
        /// Importing module identity.
        module: ModuleKey,
        /// Defining module identity.
        target: ModuleKey,
    },
    /// An exported binding names a module absent from the closure.
    #[error("checked module {module} export {name:?} targets missing module {target}")]
    MissingExportTarget {
        /// Exporting module identity.
        module: ModuleKey,
        /// Visible export name.
        name: String,
        /// Defining module identity.
        target: ModuleKey,
    },
    /// An exported binding carries an origin different from its defining module.
    #[error("checked module {module} export {name:?} has mismatched origin for {target}")]
    ExportOriginMismatch {
        /// Exporting module identity.
        module: ModuleKey,
        /// Visible export name.
        name: String,
        /// Defining module identity.
        target: ModuleKey,
    },
    /// An exported or imported identity is absent from the public target view.
    #[error("checked module {module} binding {name:?} is not an export of {target}")]
    BindingIdentityUnavailable {
        /// Importing or exporting module identity.
        module: ModuleKey,
        /// Local or visible binding name.
        name: String,
        /// Defining module identity.
        target: ModuleKey,
    },
    /// A callable import targets a module without a selected checked entry.
    #[error(
        "checked module {module} callable import {name:?} targets {target} without entry metadata"
    )]
    MissingCallableEntryMetadata {
        /// Importing module identity.
        module: ModuleKey,
        /// Local imported name.
        name: String,
        /// Defining module identity.
        target: Box<ModuleKey>,
    },
    /// A module's selected checked entry is absent from its local callable
    /// closure.
    #[error("checked module {module} selected entry {entry:?} is not a local callable")]
    SelectedEntryUnavailable {
        /// Module carrying the forged or stale selection.
        module: ModuleKey,
        /// Selected declaration name.
        entry: String,
    },
    /// A selected callable's parameter metadata disagrees with its checked
    /// local closure entry.
    #[error("checked module {module} selected entry {entry:?} has mismatched parameter metadata")]
    SelectedEntryParameterMismatch {
        /// Module carrying the forged or stale selection.
        module: ModuleKey,
        /// Selected declaration name.
        entry: String,
    },
    /// A local closure entry claims a non-callable namespace kind.
    #[error("checked module {module} local closure entry {entry:?} is not callable")]
    NonCallableLocalEntry {
        /// Module carrying the forged local entry.
        module: ModuleKey,
        /// Local closure entry name.
        entry: String,
    },
    /// A checked CPS term attempted to invoke an imported metadata namespace.
    #[error("checked module {module} attempted to invoke non-callable import {name:?} ({kind:?})")]
    NonCallableImportInvocation {
        /// Module containing the invalid call.
        module: ModuleKey,
        /// Local imported name used as the callee.
        name: String,
        /// Checked namespace kind carried by the import.
        kind: ModuleInterfaceBindingKind,
    },
    /// A callable import's selected entry does not preserve its defining name.
    #[error(
        "checked module {module} callable import {name:?} targets {target}, but selected entry is {entry:?}"
    )]
    CallableEntryIdentityMismatch {
        /// Importing module identity.
        module: ModuleKey,
        /// Local imported name.
        name: String,
        /// Defining module identity.
        target: Box<ModuleKey>,
        /// Selected target entry name.
        entry: String,
    },
    /// A selected callable entry received the wrong number of arguments.
    #[error(
        "checked module {module} callable import {name:?} targets {target} with arity {expected}, got {actual}"
    )]
    CallableEntryArityMismatch {
        /// Importing module identity.
        module: ModuleKey,
        /// Local imported name.
        name: String,
        /// Defining module identity.
        target: Box<ModuleKey>,
        /// Checked target parameter count.
        expected: usize,
        /// Call-site argument count.
        actual: usize,
    },
    /// A callable import would recursively inline an active module entry.
    #[error("checked module callable linking encountered a cycle at {module}")]
    CallableLinkCycle {
        /// Module whose selected entry is already being linked.
        module: ModuleKey,
    },
    /// The canonical module dependency closure contains a cycle.
    #[error("checked module dependency closure contains a cycle at {module}")]
    ModuleDependencyCycle {
        /// Module revisited while its dependency path is still active.
        module: ModuleKey,
    },
}

/// Atomically validated, canonical-keyed checked module transport.
///
/// This value is deliberately only a metadata carrier. It has no admission,
/// execution, sealing, provider, handler, or terminal-result methods. TASK-2063
/// must validate this closure again before creating its separately sealed
/// linking/admission request.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckedModuleTransport {
    root: ModuleKey,
    modules: Vec<CheckedModuleArtifactInput>,
}

impl CheckedModuleTransport {
    /// Validates and publishes one complete canonical module closure.
    ///
    /// Validation is performed before the result is constructed, so no
    /// partial transport can escape when an entry, dependency, identity,
    /// provenance fact, or closure edge is invalid.
    ///
    /// # Errors
    ///
    /// Returns a [`CheckedModuleTransportError`] when the closure is empty,
    /// incomplete, duplicated, forged, failed, or not reachable from `root`.
    pub fn new(
        root: ModuleKey,
        mut modules: Vec<CheckedModuleArtifactInput>,
    ) -> Result<Self, CheckedModuleTransportError> {
        if modules.is_empty() {
            return Err(CheckedModuleTransportError::Empty);
        }

        modules.sort_by(|left, right| {
            left.interface()
                .artifact()
                .key()
                .cmp(right.interface().artifact().key())
        });
        for pair in modules.windows(2) {
            let left = pair[0].interface().artifact().key();
            let right = pair[1].interface().artifact().key();
            if left == right {
                return Err(CheckedModuleTransportError::DuplicateModule {
                    module: left.clone(),
                });
            }
        }

        let root_present = modules
            .iter()
            .any(|module| module.interface().artifact().key() == &root);
        if !root_present {
            return Err(CheckedModuleTransportError::MissingRoot { root });
        }

        for module in &modules {
            validate_entry(module)?;
        }

        let mut index = std::collections::BTreeMap::new();
        for module in &modules {
            index.insert(module.interface().artifact().key().clone(), module);
        }

        validate_closure_edges(&modules, &index)?;
        let reachable = reachable_modules(&root, &index)?;
        for module in &modules {
            let key = module.interface().artifact().key();
            if !reachable.contains(key) {
                return Err(CheckedModuleTransportError::UnreachableModule {
                    root,
                    module: key.clone(),
                });
            }
        }

        Ok(Self { root, modules })
    }

    /// Returns the source-path-independent cache identity for this transport.
    #[must_use]
    pub fn cache_key(&self) -> String {
        self.root.cache_key()
    }

    /// Returns the canonical root module identity.
    #[must_use]
    pub const fn root(&self) -> &ModuleKey {
        &self.root
    }

    /// Returns successful checked modules in canonical-key order.
    #[must_use]
    pub fn modules(&self) -> &[CheckedModuleArtifactInput] {
        &self.modules
    }

    /// Looks up one transported module by canonical identity.
    #[must_use]
    pub fn module(&self, key: &ModuleKey) -> Option<&CheckedModuleArtifactInput> {
        self.modules
            .binary_search_by(|module| module.interface().artifact().key().cmp(key))
            .ok()
            .map(|index| &self.modules[index])
    }
}

/// Links resolved imported callable entries into one checked CPS term.
///
/// This is a non-authorizing transport operation. It substitutes only
/// callable imports whose defining identity and selected entry metadata agree;
/// Engine admission still validates the resulting term and seals execution.
pub(crate) fn link_checked_module_cps(
    transport: &CheckedModuleTransport,
    module: &ModuleKey,
) -> Result<Term, CheckedModuleTransportError> {
    let mut active = std::collections::BTreeSet::new();
    link_module_term(transport, module, &mut active, None)
}

fn link_module_term(
    transport: &CheckedModuleTransport,
    module: &ModuleKey,
    active: &mut std::collections::BTreeSet<(ModuleKey, Option<String>)>,
    answer: Option<&ContRef>,
) -> Result<Term, CheckedModuleTransportError> {
    if !active.insert((module.clone(), None)) {
        return Err(CheckedModuleTransportError::CallableLinkCycle {
            module: module.clone(),
        });
    }
    let result = transport
        .module(module)
        .and_then(CheckedModuleArtifactInput::cps)
        .ok_or_else(|| CheckedModuleTransportError::MissingRoot {
            root: module.clone(),
        })
        .and_then(|cps| link_term(transport, module, cps.cps_program().clone(), active, answer));
    active.remove(&(module.clone(), None));
    result
}

fn link_local_callable_term(
    transport: &CheckedModuleTransport,
    module: &ModuleKey,
    entry: &CheckedLocalCallableEntry,
    active: &mut std::collections::BTreeSet<(ModuleKey, Option<String>)>,
    answer: Option<&ContRef>,
) -> Result<Term, CheckedModuleTransportError> {
    let identity = (module.clone(), Some(entry.name.clone()));
    if !active.insert(identity.clone()) {
        return Err(CheckedModuleTransportError::CallableLinkCycle {
            module: module.clone(),
        });
    }
    let result = link_term(
        transport,
        module,
        entry.cps.cps_program().clone(),
        active,
        answer,
    );
    active.remove(&identity);
    result
}

#[allow(clippy::result_large_err, clippy::too_many_lines)]
fn link_term(
    transport: &CheckedModuleTransport,
    module: &ModuleKey,
    term: Term,
    active: &mut std::collections::BTreeSet<(ModuleKey, Option<String>)>,
    answer: Option<&ContRef>,
) -> Result<Term, CheckedModuleTransportError> {
    Ok(match term {
        Term::LetVal { name, value, body } => Term::LetVal {
            name,
            value: link_value(transport, module, value, active)?,
            body: Box::new(link_term(transport, module, *body, active, answer)?),
        },
        Term::LetPrim {
            name,
            op,
            args,
            body,
        } => Term::LetPrim {
            name,
            op,
            args,
            body: Box::new(link_term(transport, module, *body, active, answer)?),
        },
        Term::LetCont {
            name,
            param,
            cont_body,
            body,
            row,
            multiplicity,
        } => Term::LetCont {
            name,
            param,
            cont_body: Box::new(link_term(transport, module, *cont_body, active, answer)?),
            body: Box::new(link_term(transport, module, *body, active, answer)?),
            row,
            multiplicity,
        },
        Term::LetContCall {
            name,
            cont,
            arg,
            row,
            body,
        } => Term::LetContCall {
            name,
            cont,
            arg,
            row,
            body: Box::new(link_term(transport, module, *body, active, answer)?),
        },
        Term::Jump { cont, arg, row } => Term::Jump {
            cont: linked_answer_continuation(cont, answer),
            arg,
            row,
        },
        Term::JumpValue { cont, arg, row } => Term::JumpValue {
            cont: linked_answer_continuation(cont, answer),
            arg,
            row,
        },
        Term::Call {
            func,
            args,
            cont,
            row,
        } => {
            if let Atom::Var(local_name) = &func {
                if let Some(import) = transport
                    .module(module)
                    .and_then(CheckedModuleArtifactInput::cps)
                    .and_then(|cps| {
                        cps.imports()
                            .iter()
                            .find(|import| import.local_name() == local_name)
                    })
                {
                    if !import.binding().kind().is_runtime_callable() {
                        return Err(CheckedModuleTransportError::NonCallableImportInvocation {
                            module: module.clone(),
                            name: local_name.clone(),
                            kind: import.binding().kind(),
                        });
                    }
                    if import.binding().kind().is_runtime_callable() {
                        let ash_core::module_interface::ModuleInterfaceDefiningIdentity::Declaration(
                            identity,
                        ) = import.binding().defining_identity()
                        else {
                            return Err(CheckedModuleTransportError::BindingIdentityUnavailable {
                                module: module.clone(),
                                name: local_name.clone(),
                                target: module.clone(),
                            });
                        };
                        let target = &identity.module;
                        let target_entry = transport.module(target).ok_or_else(|| {
                            CheckedModuleTransportError::MissingImportTarget {
                                module: module.clone(),
                                target: target.clone(),
                            }
                        })?;
                        if target == module {
                            let local_entry =
                                target_entry.local_entry(&identity.name).ok_or_else(|| {
                                    CheckedModuleTransportError::BindingIdentityUnavailable {
                                        module: module.clone(),
                                        name: local_name.clone(),
                                        target: target.clone(),
                                    }
                                })?;
                            let expected = local_entry.parameter_names.len();
                            if expected != args.len() {
                                return Err(
                                    CheckedModuleTransportError::CallableEntryArityMismatch {
                                        module: module.clone(),
                                        name: local_name.clone(),
                                        target: Box::new(target.clone()),
                                        expected,
                                        actual: args.len(),
                                    },
                                );
                            }
                            let target_term = link_local_callable_term(
                                transport,
                                target,
                                local_entry,
                                active,
                                Some(&cont),
                            )?;
                            bind_callable_parameters(
                                target_term,
                                &local_entry.parameter_names,
                                &args,
                            )
                        } else if let Some(local_entry) = target_entry.local_entry(&identity.name) {
                            // A definition-backed closure may carry several
                            // checked callable bodies for one module. Resolve
                            // an imported target from that per-function
                            // closure before falling back to the selected
                            // module entry; the selected entry remains the
                            // module's standalone transport identity, not a
                            // restriction on other checked local callables.
                            let expected = local_entry.parameter_names.len();
                            if expected != args.len() {
                                return Err(
                                    CheckedModuleTransportError::CallableEntryArityMismatch {
                                        module: module.clone(),
                                        name: local_name.clone(),
                                        target: Box::new(target.clone()),
                                        expected,
                                        actual: args.len(),
                                    },
                                );
                            }
                            let target_term = link_local_callable_term(
                                transport,
                                target,
                                local_entry,
                                active,
                                Some(&cont),
                            )?;
                            bind_callable_parameters(
                                target_term,
                                &local_entry.parameter_names,
                                &args,
                            )
                        } else {
                            let entry_name = target_entry.entry_name().ok_or_else(|| {
                                CheckedModuleTransportError::MissingCallableEntryMetadata {
                                    module: module.clone(),
                                    name: local_name.clone(),
                                    target: Box::new(target.clone()),
                                }
                            })?;
                            if entry_name != identity.name {
                                return Err(
                                    CheckedModuleTransportError::CallableEntryIdentityMismatch {
                                        module: module.clone(),
                                        name: local_name.clone(),
                                        target: Box::new(target.clone()),
                                        entry: entry_name.to_owned(),
                                    },
                                );
                            }
                            let expected = target_entry.parameter_names().len();
                            if expected != args.len() {
                                return Err(
                                    CheckedModuleTransportError::CallableEntryArityMismatch {
                                        module: module.clone(),
                                        name: local_name.clone(),
                                        target: Box::new(target.clone()),
                                        expected,
                                        actual: args.len(),
                                    },
                                );
                            }
                            let target_term =
                                link_module_term(transport, target, active, Some(&cont))?;
                            bind_callable_parameters(
                                target_term,
                                target_entry.parameter_names(),
                                &args,
                            )
                        }
                    } else {
                        Term::Call {
                            func,
                            args,
                            cont,
                            row,
                        }
                    }
                } else if let Some(local_entry) = transport
                    .module(module)
                    .and_then(|entry| entry.local_entry(local_name))
                {
                    let expected = local_entry.parameter_names.len();
                    if expected != args.len() {
                        return Err(CheckedModuleTransportError::CallableEntryArityMismatch {
                            module: module.clone(),
                            name: local_name.clone(),
                            target: Box::new(module.clone()),
                            expected,
                            actual: args.len(),
                        });
                    }
                    let target_term = link_local_callable_term(
                        transport,
                        module,
                        local_entry,
                        active,
                        Some(&cont),
                    )?;
                    bind_callable_parameters(target_term, &local_entry.parameter_names, &args)
                } else {
                    Term::Call {
                        func,
                        args,
                        cont,
                        row,
                    }
                }
            } else {
                Term::Call {
                    func,
                    args,
                    cont,
                    row,
                }
            }
        }
        Term::If {
            cond,
            then_branch,
            else_branch,
            row,
        } => Term::If {
            cond,
            then_branch: Box::new(link_term(transport, module, *then_branch, active, answer)?),
            else_branch: Box::new(link_term(transport, module, *else_branch, active, answer)?),
            row,
        },
        Term::LetRec { name, value, body } => Term::LetRec {
            name,
            value: link_value(transport, module, value, active)?,
            body: Box::new(link_term(transport, module, *body, active, answer)?),
        },
        Term::Match {
            scrutinee,
            arms,
            default,
        } => Term::Match {
            scrutinee,
            arms: arms
                .into_iter()
                .map(|(name, body)| {
                    Ok((
                        name,
                        Box::new(link_term(transport, module, *body, active, answer)?),
                    ))
                })
                .collect::<Result<_, CheckedModuleTransportError>>()?,
            default: default
                .map(|body| link_term(transport, module, *body, active, answer).map(Box::new))
                .transpose()?,
        },
        Term::Raise {
            op,
            args,
            resume,
            row,
        } => Term::Raise {
            op,
            args,
            resume,
            row,
        },
        Term::Handle {
            clause,
            body,
            cont,
            row,
        } => Term::Handle {
            clause,
            body: Box::new(link_term(transport, module, *body, active, answer)?),
            cont,
            row,
        },
        Term::RecordDischarge { discharge, body } => Term::RecordDischarge {
            discharge,
            body: Box::new(link_term(transport, module, *body, active, answer)?),
        },
        Term::Return { value } => {
            let value = link_value(transport, module, value, active)?;
            match answer {
                Some(cont) => Term::JumpValue {
                    cont: cont.clone(),
                    arg: value,
                    row: ash_core::cps::EffectRow::default(),
                },
                None => Term::Return { value },
            }
        }
        Term::Trap { reason } => Term::Trap { reason },
    })
}

fn bind_callable_parameters(mut term: Term, parameter_names: &[String], args: &[Atom]) -> Term {
    for (parameter, argument) in parameter_names.iter().zip(args.iter()).rev() {
        term = Term::LetVal {
            name: parameter.clone(),
            value: Value::Atom(argument.clone()),
            body: Box::new(term),
        };
    }
    term
}

fn link_value(
    transport: &CheckedModuleTransport,
    module: &ModuleKey,
    value: Value,
    active: &mut std::collections::BTreeSet<(ModuleKey, Option<String>)>,
) -> Result<Value, CheckedModuleTransportError> {
    Ok(match value {
        Value::Lam {
            params,
            cont,
            body,
            captured_env,
            rec_binding,
            row,
        } => Value::Lam {
            params,
            cont,
            body: Box::new(link_term(transport, module, *body, active, None)?),
            captured_env,
            rec_binding,
            row,
        },
        Value::Cont {
            param,
            body,
            captured_env,
            captured_chain,
            consumed,
            row,
            multiplicity,
        } => Value::Cont {
            param,
            body: Box::new(link_term(transport, module, *body, active, None)?),
            captured_env,
            captured_chain,
            consumed,
            row,
            multiplicity,
        },
        Value::Record { fields } => Value::Record {
            fields: fields
                .into_iter()
                .map(|(name, value)| Ok((name, link_value(transport, module, value, active)?)))
                .collect::<Result<_, CheckedModuleTransportError>>()?,
        },
        Value::Tuple { elems } => Value::Tuple {
            elems: elems
                .into_iter()
                .map(|value| link_value(transport, module, value, active))
                .collect::<Result<_, CheckedModuleTransportError>>()?,
        },
        other => other,
    })
}

fn linked_answer_continuation(cont: ContRef, answer: Option<&ContRef>) -> ContRef {
    if matches!(&cont, ContRef::Label(label) if label == "__answer") {
        answer.cloned().unwrap_or(cont)
    } else {
        cont
    }
}

/// Canonical-keyed cache for already validated checked module transports.
///
/// This cache is deliberately separate from the legacy source loader cache:
/// it stores only a complete non-authorizing transport and indexes it by
/// [`ModuleKey`]. A duplicate root is rejected rather than overwritten, so a
/// changed or forged artifact cannot be silently substituted for an existing
/// checked closure. Source paths and display strings never participate in the
/// key or lookup.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CheckedModuleTransportCache {
    entries: std::collections::BTreeMap<ModuleKey, CheckedModuleTransport>,
}

impl CheckedModuleTransportCache {
    /// Inserts one complete checked transport without replacing an existing root.
    ///
    /// # Errors
    ///
    /// Returns [`CheckedModuleTransportCacheError::DuplicateCanonicalRoot`] when
    /// the canonical root already has an entry.
    pub fn insert(
        &mut self,
        transport: CheckedModuleTransport,
    ) -> Result<(), CheckedModuleTransportCacheError> {
        let root = transport.root().clone();
        if self.entries.contains_key(&root) {
            return Err(CheckedModuleTransportCacheError::DuplicateCanonicalRoot { root });
        }
        self.entries.insert(root, transport);
        Ok(())
    }

    /// Looks up a checked transport by canonical module identity.
    #[must_use]
    pub fn get(&self, root: &ModuleKey) -> Option<&CheckedModuleTransport> {
        self.entries.get(root)
    }

    /// Returns the number of cached canonical closures.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the cache has no canonical closure entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

fn validate_closure_edges(
    modules: &[CheckedModuleArtifactInput],
    index: &std::collections::BTreeMap<ModuleKey, &CheckedModuleArtifactInput>,
) -> Result<(), CheckedModuleTransportError> {
    for module in modules {
        let key = module.interface().artifact().key();
        for dependency in module.interface().dependencies() {
            if !index.contains_key(&dependency.module) {
                return Err(CheckedModuleTransportError::MissingDependency {
                    module: key.clone(),
                    dependency: dependency.module.clone(),
                });
            }
            if dependency.schema_version != module.interface().schema_version() {
                return Err(CheckedModuleTransportError::DependencySchemaMismatch {
                    module: key.clone(),
                    dependency: dependency.module.clone(),
                    actual: dependency.schema_version,
                    expected: module.interface().schema_version(),
                });
            }
        }
        for child in module.interface().artifact().child_keys() {
            if !index.contains_key(child) {
                return Err(CheckedModuleTransportError::MissingStructuralChild {
                    module: key.clone(),
                    child: child.clone(),
                });
            }
        }
        if let Some(parent) = module.interface().artifact().structural_parent() {
            let Some(parent_entry) = index.get(parent) else {
                return Err(CheckedModuleTransportError::MissingStructuralParent {
                    module: key.clone(),
                    parent: parent.clone(),
                });
            };
            if !parent_entry
                .interface()
                .artifact()
                .child_keys()
                .contains(key)
            {
                return Err(CheckedModuleTransportError::StructuralParentChildMismatch {
                    module: key.clone(),
                    parent: parent.clone(),
                });
            }
        }
        validate_export_bindings(module, index)?;
        validate_import_targets(module, index)?;
    }
    Ok(())
}

fn reachable_modules(
    root: &ModuleKey,
    index: &std::collections::BTreeMap<ModuleKey, &CheckedModuleArtifactInput>,
) -> Result<std::collections::BTreeSet<ModuleKey>, CheckedModuleTransportError> {
    let mut reachable = std::collections::BTreeSet::new();
    let mut active = std::collections::BTreeSet::new();
    visit_reachable_module(root, root, index, &mut reachable, &mut active)?;
    Ok(reachable)
}

fn visit_reachable_module(
    root: &ModuleKey,
    module_key: &ModuleKey,
    index: &std::collections::BTreeMap<ModuleKey, &CheckedModuleArtifactInput>,
    reachable: &mut std::collections::BTreeSet<ModuleKey>,
    active: &mut std::collections::BTreeSet<ModuleKey>,
) -> Result<(), CheckedModuleTransportError> {
    if active.contains(module_key) {
        return Err(CheckedModuleTransportError::ModuleDependencyCycle {
            module: module_key.clone(),
        });
    }
    if !reachable.insert(module_key.clone()) {
        return Ok(());
    }
    let Some(module) = index.get(module_key) else {
        return Err(CheckedModuleTransportError::MissingDependency {
            module: root.clone(),
            dependency: module_key.clone(),
        });
    };

    active.insert(module_key.clone());
    for child in module.interface().artifact().child_keys() {
        visit_reachable_module(root, child, index, reachable, active)?;
    }
    for dependency in module.interface().dependencies() {
        visit_reachable_module(root, &dependency.module, index, reachable, active)?;
    }
    active.remove(module_key);
    Ok(())
}

fn validate_entry(entry: &CheckedModuleArtifactInput) -> Result<(), CheckedModuleTransportError> {
    let module = entry.interface().artifact().key().clone();
    if entry.interface().schema_version() != PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION {
        return Err(CheckedModuleTransportError::UnsupportedInterfaceSchema {
            module,
            schema_version: entry.interface().schema_version(),
        });
    }
    if entry.interface().artifact().schema_version() != MODULE_ARTIFACT_SCHEMA_VERSION {
        return Err(CheckedModuleTransportError::UnsupportedArtifactSchema {
            module,
            schema_version: entry.interface().artifact().schema_version(),
        });
    }
    let Some(core) = entry.core() else {
        return Err(CheckedModuleTransportError::FailedModule {
            module,
            reason: entry
                .failure()
                .unwrap_or("unknown checker failure")
                .to_owned(),
        });
    };
    let Some(cps) = entry.cps() else {
        return Err(CheckedModuleTransportError::FailedModule {
            module,
            reason: entry.failure().unwrap_or("missing CPS artifact").to_owned(),
        });
    };

    if entry.interface().artifact() != core.module_artifact() {
        return Err(CheckedModuleTransportError::InterfaceCoreMismatch {
            module,
            fact: "module artifact identity or origin",
        });
    }
    if entry.interface().artifact() != cps.module_artifact() {
        return Err(CheckedModuleTransportError::InterfaceCpsMismatch {
            module,
            fact: "module artifact identity or origin",
        });
    }
    if core.module_artifact() != cps.module_artifact() {
        return Err(CheckedModuleTransportError::CoreCpsMismatch {
            module,
            fact: "module artifact identity or origin",
        });
    }
    if core.interface_schema_version() != entry.interface().schema_version() {
        return Err(CheckedModuleTransportError::InterfaceCoreMismatch {
            module,
            fact: "public-interface schema version",
        });
    }
    if cps.interface_schema_version() != entry.interface().schema_version() {
        return Err(CheckedModuleTransportError::InterfaceCpsMismatch {
            module,
            fact: "public-interface schema version",
        });
    }
    if core.dependencies() != entry.interface().dependencies() {
        return Err(CheckedModuleTransportError::InterfaceCoreMismatch {
            module,
            fact: "public-interface dependency snapshot",
        });
    }
    if cps.dependencies() != entry.interface().dependencies() {
        return Err(CheckedModuleTransportError::InterfaceCpsMismatch {
            module,
            fact: "public-interface dependency snapshot",
        });
    }
    if core.imports() != cps.imports() {
        return Err(CheckedModuleTransportError::CoreCpsMismatch {
            module,
            fact: "resolved import snapshot",
        });
    }
    for local_entry in &entry.local_entries {
        if local_entry.kind != ModuleInterfaceBindingKind::Callable {
            return Err(CheckedModuleTransportError::NonCallableLocalEntry {
                module: entry.interface().artifact().key().clone(),
                entry: local_entry.name.clone(),
            });
        }
        if local_entry.cps.module_artifact() != entry.interface().artifact()
            || local_entry.cps.interface_schema_version() != entry.interface().schema_version()
            || local_entry.cps.dependencies() != entry.interface().dependencies()
            || local_entry.cps.imports() != core.imports()
        {
            return Err(CheckedModuleTransportError::CoreCpsMismatch {
                module: entry.interface().artifact().key().clone(),
                fact: "checked local callable artifact metadata",
            });
        }
    }
    validate_selected_entry_metadata(entry)
}

fn validate_selected_entry_metadata(
    entry: &CheckedModuleArtifactInput,
) -> Result<(), CheckedModuleTransportError> {
    let Some(selected_name) = entry.entry_name().filter(|name| !name.is_empty()) else {
        return Ok(());
    };
    if let Some(local_entry) = entry.local_entries.iter().find(|local_entry| {
        local_entry.name == selected_name
            && local_entry.kind == ModuleInterfaceBindingKind::Callable
    }) {
        if local_entry.parameter_names != entry.parameter_names {
            return Err(
                CheckedModuleTransportError::SelectedEntryParameterMismatch {
                    module: entry.interface().artifact().key().clone(),
                    entry: selected_name.to_owned(),
                },
            );
        }
        return Ok(());
    }
    Err(CheckedModuleTransportError::SelectedEntryUnavailable {
        module: entry.interface().artifact().key().clone(),
        entry: selected_name.to_owned(),
    })
}

fn validate_import_targets(
    entry: &CheckedModuleArtifactInput,
    index: &std::collections::BTreeMap<ModuleKey, &CheckedModuleArtifactInput>,
) -> Result<(), CheckedModuleTransportError> {
    let Some(core) = entry.core() else {
        return Ok(());
    };
    let module = entry.interface().artifact().key().clone();
    for import in core.imports() {
        let target = match import.binding().defining_identity() {
            ModuleInterfaceDefiningIdentity::ChildModule(target) => target,
            ModuleInterfaceDefiningIdentity::Declaration(identity) => &identity.module,
        };
        let Some(target_entry) = index.get(target) else {
            return Err(CheckedModuleTransportError::MissingImportTarget {
                module,
                target: target.clone(),
            });
        };
        if target != &module
            && !entry
                .interface()
                .dependencies()
                .iter()
                .any(|dependency| dependency.module == *target)
        {
            return Err(CheckedModuleTransportError::UndeclaredImportDependency {
                module,
                target: target.clone(),
            });
        }
        if target_entry.interface().artifact().origin() != import.binding().origin() {
            return Err(CheckedModuleTransportError::ImportOriginMismatch {
                module,
                target: target.clone(),
            });
        }
        let binding_available = match import.binding().defining_identity() {
            ModuleInterfaceDefiningIdentity::ChildModule(_) => {
                public_structural_child_matches(index, target, import.binding().origin())
            }
            ModuleInterfaceDefiningIdentity::Declaration(identity) if target == &module => {
                public_binding_matches(
                    index,
                    target,
                    import.binding().defining_identity(),
                    import.binding().origin(),
                ) || index.get(&module).is_some_and(|entry| {
                    local_binding_matches(entry, identity, import.binding().origin())
                })
            }
            ModuleInterfaceDefiningIdentity::Declaration(identity) => {
                public_binding_matches(
                    index,
                    target,
                    import.binding().defining_identity(),
                    import.binding().origin(),
                ) || match import.visibility() {
                    ModuleImportVisibility::Crate if module.crate_root() == target.crate_root() => {
                        // `pub(crate)` imports are checked by the canonical
                        // Type route but intentionally do not enter the
                        // target's public interface. Admit only the callable
                        // artifact that the sealed closure carries; metadata
                        // non-callable metadata namespaces never
                        // use this escape.
                        crate_visible_callable_matches(
                            index,
                            target,
                            identity,
                            import.binding().origin(),
                        )
                    }
                    ModuleImportVisibility::Super { levels }
                        if module.crate_root() == target.crate_root() =>
                    {
                        super_visible_callable_matches(
                            index,
                            &module,
                            target,
                            identity,
                            *levels,
                            import.binding().origin(),
                        )
                    }
                    ModuleImportVisibility::Restricted { path }
                        if module.crate_root() == target.crate_root() =>
                    {
                        restricted_visible_callable_matches(
                            index,
                            &module,
                            target,
                            identity,
                            path,
                            import.binding().origin(),
                        )
                    }
                    _ => false,
                }
            }
        };
        if !binding_available {
            return Err(CheckedModuleTransportError::BindingIdentityUnavailable {
                module,
                name: import.local_name().to_owned(),
                target: target.clone(),
            });
        }
    }
    Ok(())
}

fn validate_export_bindings(
    entry: &CheckedModuleArtifactInput,
    index: &std::collections::BTreeMap<ModuleKey, &CheckedModuleArtifactInput>,
) -> Result<(), CheckedModuleTransportError> {
    let module = entry.interface().artifact().key().clone();
    for export in entry.interface().bindings() {
        let target = match export.defining_identity() {
            ModuleInterfaceDefiningIdentity::ChildModule(target) => target,
            ModuleInterfaceDefiningIdentity::Declaration(identity) => &identity.module,
        };
        let Some(target_entry) = index.get(target) else {
            return Err(CheckedModuleTransportError::MissingExportTarget {
                module,
                name: export.visible_name().to_owned(),
                target: target.clone(),
            });
        };
        let target_is_structural_child = matches!(
            export.defining_identity(),
            ModuleInterfaceDefiningIdentity::ChildModule(_)
        );
        if target != &module
            && !target_is_structural_child
            && !entry
                .interface()
                .dependencies()
                .iter()
                .any(|dependency| dependency.module == *target)
        {
            return Err(CheckedModuleTransportError::UndeclaredExportDependency {
                module,
                name: export.visible_name().to_owned(),
                target: target.clone(),
            });
        }
        if target_entry.interface().artifact().origin() != export.origin() {
            return Err(CheckedModuleTransportError::ExportOriginMismatch {
                module,
                name: export.visible_name().to_owned(),
                target: target.clone(),
            });
        }
        if !matches!(export.visibility(), ash_core::Visibility::Public) {
            return Err(CheckedModuleTransportError::BindingIdentityUnavailable {
                module,
                name: export.visible_name().to_owned(),
                target: target.clone(),
            });
        }
        if matches!(
            export.defining_identity(),
            ModuleInterfaceDefiningIdentity::Declaration(_)
        ) && !public_binding_matches(index, target, export.defining_identity(), export.origin())
        {
            return Err(CheckedModuleTransportError::BindingIdentityUnavailable {
                module,
                name: export.visible_name().to_owned(),
                target: target.clone(),
            });
        }
    }
    Ok(())
}

fn public_binding_matches(
    index: &std::collections::BTreeMap<ModuleKey, &CheckedModuleArtifactInput>,
    defining_module: &ModuleKey,
    identity: &ModuleInterfaceDefiningIdentity,
    origin: &ash_core::module_graph::ModuleArtifactOrigin,
) -> bool {
    index.get(defining_module).is_some_and(|entry| {
        entry.interface().bindings().iter().any(|binding| {
            binding.visibility() == ash_core::Visibility::Public
                && binding.defining_identity() == identity
                && binding.origin() == origin
        })
    })
}

fn crate_visible_callable_matches(
    index: &std::collections::BTreeMap<ModuleKey, &CheckedModuleArtifactInput>,
    defining_module: &ModuleKey,
    identity: &ash_core::module_interface::ModuleInterfaceDeclarationIdentity,
    origin: &ash_core::module_graph::ModuleArtifactOrigin,
) -> bool {
    if identity.kind != ModuleInterfaceBindingKind::Callable {
        return false;
    }
    index
        .get(defining_module)
        .is_some_and(|entry| local_binding_matches(entry, identity, origin))
}

fn super_visible_callable_matches(
    index: &std::collections::BTreeMap<ModuleKey, &CheckedModuleArtifactInput>,
    importing_module: &ModuleKey,
    defining_module: &ModuleKey,
    identity: &ash_core::module_interface::ModuleInterfaceDeclarationIdentity,
    levels: usize,
    origin: &ash_core::module_graph::ModuleArtifactOrigin,
) -> bool {
    if identity.kind != ModuleInterfaceBindingKind::Callable || levels == 0 {
        return false;
    }
    let mut scope = defining_module.clone();
    for _ in 0..levels {
        let Some(parent) = scope.parent() else {
            return false;
        };
        scope = parent;
    }
    is_descendant_or_same(importing_module, &scope)
        && index
            .get(defining_module)
            .is_some_and(|entry| local_binding_matches(entry, identity, origin))
}

fn restricted_visible_callable_matches(
    index: &std::collections::BTreeMap<ModuleKey, &CheckedModuleArtifactInput>,
    importing_module: &ModuleKey,
    defining_module: &ModuleKey,
    identity: &ash_core::module_interface::ModuleInterfaceDeclarationIdentity,
    path: &str,
    origin: &ash_core::module_graph::ModuleArtifactOrigin,
) -> bool {
    if identity.kind != ModuleInterfaceBindingKind::Callable {
        return false;
    }

    let mut segments = path.split("::");
    if segments.next() != Some("crate") {
        return false;
    }
    let mut region = defining_module.crate_root();
    for segment in segments {
        if segment.is_empty() {
            return false;
        }
        let Ok(next) = region.child(segment) else {
            return false;
        };
        region = next;
    }

    index.contains_key(&region)
        && is_descendant_or_same(defining_module, &region)
        && is_descendant_or_same(importing_module, &region)
        && index
            .get(defining_module)
            .is_some_and(|entry| local_binding_matches(entry, identity, origin))
}

fn is_descendant_or_same(module: &ModuleKey, ancestor: &ModuleKey) -> bool {
    let mut current = Some(module.clone());
    while let Some(candidate) = current {
        if &candidate == ancestor {
            return true;
        }
        current = candidate.parent();
    }
    false
}

fn public_structural_child_matches(
    index: &std::collections::BTreeMap<ModuleKey, &CheckedModuleArtifactInput>,
    child: &ModuleKey,
    origin: &ash_core::module_graph::ModuleArtifactOrigin,
) -> bool {
    let Some(parent) = index
        .get(child)
        .and_then(|entry| entry.interface().artifact().structural_parent())
    else {
        return false;
    };
    index.get(parent).is_some_and(|entry| {
        entry.interface().bindings().iter().any(|binding| {
            binding.visibility() == ash_core::Visibility::Public
                && binding.origin() == origin
                && matches!(
                    binding.defining_identity(),
                    ModuleInterfaceDefiningIdentity::ChildModule(target) if target == child
                )
        })
    })
}

fn local_binding_matches(
    entry: &CheckedModuleArtifactInput,
    identity: &ash_core::module_interface::ModuleInterfaceDeclarationIdentity,
    origin: &ash_core::module_graph::ModuleArtifactOrigin,
) -> bool {
    entry.interface().artifact().origin() == origin
        && identity.kind == ModuleInterfaceBindingKind::Callable
        && (entry
            .entry_name()
            .is_some_and(|entry_name| entry_name == identity.name)
            || entry
                .local_entry(&identity.name)
                .is_some_and(|local_entry| local_entry.kind == identity.kind))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::core_ash::{CoreAtom, CoreExpr};
    use ash_core::core_ash_typecheck::{CoreTypeCheckEnv, type_check_core_program};
    use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};
    use ash_core::cps::{Atom as CpsAtom, ContRef, EffectRow, Term as CpsTerm};
    use ash_core::module_graph::{ModuleArtifact, ModuleArtifactOrigin, ModuleKey};
    use ash_core::module_interface::{
        ModuleInterfaceBindingKind, ModuleInterfaceDeclarationIdentity,
        PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION,
    };

    fn checked_core() -> ash_core::core_ash_typecheck::TypedCoreProgram {
        let raw = RawCoreProgram::new(CoreExpr::Atom(CoreAtom::LitInt(1)));
        let validated = validate_core_program(raw).expect("test Core validates");
        type_check_core_program(validated, &CoreTypeCheckEnv::default())
            .expect("test Core type-checks")
    }

    #[test]
    fn local_callable_identity_rejects_a_forged_non_callable_namespace_kind() {
        let module = ModuleKey::root("local_identity").expect("test module key is canonical");
        let origin = ModuleArtifactOrigin::File("fixtures/local_identity.ash".to_owned());
        let artifact = ModuleArtifact::new(module.clone(), origin.clone(), None, Vec::new())
            .expect("test artifact is canonical");
        let core = ModuleCoreArtifact::new_with_interface_metadata(
            artifact.clone(),
            Vec::new(),
            PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION,
            Vec::new(),
            checked_core(),
        );
        let cps = ModuleCpsArtifact::from_core_artifact(
            &core,
            CpsTerm::Jump {
                cont: ContRef::Label("__answer".to_owned()),
                arg: CpsAtom::Int(1),
                row: EffectRow::default(),
            },
        );
        let interface = PublicModuleInterface::new(artifact, Vec::new())
            .expect("test interface is export-closed");
        let entry = CheckedModuleArtifactInput {
            interface,
            core: Some(core),
            cps: Some(cps.clone()),
            entry_name: Some("entry".to_owned()),
            parameter_names: Vec::new(),
            local_entries: vec![CheckedLocalCallableEntry {
                name: "entry".to_owned(),
                kind: ModuleInterfaceBindingKind::Callable,
                cps,
                parameter_names: Vec::new(),
            }],
            failure: None,
        };
        let forged = ModuleInterfaceDeclarationIdentity::new(
            module.clone(),
            "entry",
            ModuleInterfaceBindingKind::Evidence,
        );

        assert!(
            !local_binding_matches(&entry, &forged, &origin),
            "a non-callable namespace identity must not match a local callable entry by name"
        );

        let mut forged_parameters = entry.clone();
        forged_parameters.parameter_names = vec!["forged_parameter".to_owned()];
        assert!(
            matches!(
                validate_entry(&forged_parameters),
                Err(CheckedModuleTransportError::SelectedEntryParameterMismatch {
                    module: ref selected_module,
                    entry: ref selected_entry,
                }) if selected_module == &module && selected_entry == "entry"
            ),
            "selected parameter metadata must match the checker-lowered local callable"
        );

        let mut missing_local_entry = entry.clone();
        missing_local_entry.parameter_names.clear();
        missing_local_entry.local_entries.clear();
        assert!(
            matches!(
                validate_entry(&missing_local_entry),
                Err(CheckedModuleTransportError::SelectedEntryUnavailable {
                    module: ref selected_module,
                    entry: ref selected_entry,
                }) if selected_module == &module && selected_entry == "entry"
            ),
            "selected entry metadata must not pass without a checked local callable closure"
        );

        let mut non_callable_local_entry = entry.clone();
        non_callable_local_entry.entry_name = None;
        non_callable_local_entry.local_entries[0].kind = ModuleInterfaceBindingKind::Evidence;
        assert!(
            validate_entry(&non_callable_local_entry).is_err(),
            "a local closure entry with a non-callable namespace kind must reject before transport"
        );

        let mut forged_selection = entry;
        forged_selection.entry_name = Some("forged".to_owned());
        assert!(
            matches!(
                validate_entry(&forged_selection),
                Err(CheckedModuleTransportError::SelectedEntryUnavailable {
                    module: ref selected_module,
                    entry: ref selected_entry,
                }) if selected_module == &module && selected_entry == "forged"
            ),
            "selected entry metadata must name a checker-lowered local callable"
        );
    }
}
