//! Canonical, non-authorizing Engine transport for checked module closures.
//!
//! This boundary accepts only already-materialized Core/CPS artifacts and
//! checked public interfaces. It validates their identity and dependency
//! closure, then exposes deterministic metadata to the later TASK-2063
//! linking boundary. It does not admit, execute, or seal anything.

use ash_core::module_graph::{MODULE_ARTIFACT_SCHEMA_VERSION, ModuleKey};
use ash_core::module_interface::{
    ModuleInterfaceDefiningIdentity, PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION, PublicModuleInterface,
};
use ash_core::module_lowering::{ModuleCoreArtifact, ModuleCpsArtifact};
use thiserror::Error;

/// One successful or failed checked-module result presented to Engine.
///
/// A failed entry is retained only to let the transport reject the complete
/// closure atomically; it can never become a transport module.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckedModuleArtifactInput {
    interface: PublicModuleInterface,
    core: Option<ModuleCoreArtifact>,
    cps: Option<ModuleCpsArtifact>,
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
    let mut pending = vec![root.clone()];
    while let Some(module_key) = pending.pop() {
        if !reachable.insert(module_key.clone()) {
            continue;
        }
        let Some(module) = index.get(&module_key) else {
            return Err(CheckedModuleTransportError::MissingDependency {
                module: root.clone(),
                dependency: module_key,
            });
        };
        pending.extend(module.interface().artifact().child_keys().iter().cloned());
        pending.extend(
            module
                .interface()
                .dependencies()
                .iter()
                .map(|dependency| dependency.module.clone()),
        );
    }
    Ok(reachable)
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
    if core.imports() != cps.imports() {
        return Err(CheckedModuleTransportError::CoreCpsMismatch {
            module,
            fact: "resolved import snapshot",
        });
    }
    Ok(())
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
        if target_entry.interface().artifact().origin() != import.binding().origin() {
            return Err(CheckedModuleTransportError::ImportOriginMismatch {
                module,
                target: target.clone(),
            });
        }
        if !public_binding_matches(
            index,
            import.binding().defining_identity(),
            import.binding().origin(),
        ) {
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
        ) && !public_binding_matches(index, export.defining_identity(), export.origin())
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
    identity: &ModuleInterfaceDefiningIdentity,
    origin: &ash_core::module_graph::ModuleArtifactOrigin,
) -> bool {
    index.values().any(|entry| {
        entry.interface().bindings().iter().any(|binding| {
            binding.visibility() == ash_core::Visibility::Public
                && binding.defining_identity() == identity
                && binding.origin() == origin
        })
    })
}
