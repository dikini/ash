//! Checked-interface-only import resolution for the bounded module path.
//!
//! This resolver accepts only [`FinalizedModuleInterface`] values. It walks
//! canonical [`ModuleKey`] values through public child-module identities and
//! never reads parser resolver state, a filesystem, Engine state, or a raw
//! [`PublicModuleInterface`]. Typed namespace linkage, re-exports, cycles,
//! and binder integration remain outside this bounded TASK-2061 slice.

use std::collections::{BTreeMap, btree_map::Entry};
use std::fmt;

use ash_core::Visibility;
use ash_core::module_graph::ModuleKey;
use ash_core::module_interface::{ModuleInterfaceBinding, ModuleInterfaceDefiningIdentity};
use thiserror::Error;

use crate::module_interface_finalization::FinalizedModuleInterface;

/// Canonical checked interfaces available to one import-resolution pass.
///
/// The store owns only finalizer-issued interface wrappers, indexed by their
/// canonical module keys. Raw Core public-interface carriers have no entry
/// point into this store.
#[derive(Debug)]
pub struct CheckedInterfaceStore {
    interfaces: BTreeMap<ModuleKey, FinalizedModuleInterface>,
}

/// Failure while constructing a checked-interface store.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CheckedInterfaceStoreError {
    /// More than one finalized interface claimed one canonical module key.
    #[error("duplicate finalized interface for module {module}")]
    DuplicateModule {
        /// Canonical module key claimed more than once.
        module: ModuleKey,
    },
}

impl CheckedInterfaceStore {
    /// Creates a canonical index over finalizer-issued interfaces.
    ///
    /// # Errors
    ///
    /// Returns [`CheckedInterfaceStoreError::DuplicateModule`] if multiple
    /// wrappers claim the same canonical module key.
    pub fn new(
        interfaces: Vec<FinalizedModuleInterface>,
    ) -> Result<Self, CheckedInterfaceStoreError> {
        let mut canonical_interfaces = BTreeMap::new();
        for interface in interfaces {
            let module = interface.module_key().clone();
            if let Entry::Vacant(entry) = canonical_interfaces.entry(module.clone()) {
                entry.insert(interface);
            } else {
                return Err(CheckedInterfaceStoreError::DuplicateModule { module });
            }
        }

        Ok(Self {
            interfaces: canonical_interfaces,
        })
    }

    fn interface(&self, module: &ModuleKey) -> Option<&FinalizedModuleInterface> {
        self.interfaces.get(module)
    }
}

/// A canonical, checked-interface import path.
///
/// Its first segment identifies a crate root. Remaining segments are
/// canonical source-visible names: a request traverses every segment except
/// the final explicit-import segment as a public child module.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InterfaceImportPath {
    root: ModuleKey,
    segments: Vec<String>,
}

/// Failure while constructing an [`InterfaceImportPath`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InterfaceImportPathError {
    /// The import path did not name a crate root.
    #[error("interface import path must contain a crate-root segment")]
    EmptyPath,
    /// One source-visible path segment was not canonical.
    #[error("invalid interface import path segment {segment:?}: {reason}")]
    InvalidSegment {
        /// Segment rejected while validating the import path.
        segment: String,
        /// Canonical-key validation diagnostic.
        reason: String,
    },
}

impl InterfaceImportPath {
    /// Builds a canonical import path from root-first source-visible segments.
    ///
    /// # Errors
    ///
    /// Returns [`InterfaceImportPathError`] when the path is empty or cannot
    /// be represented by canonical [`ModuleKey`] segments.
    pub fn new<S>(segments: &[S]) -> Result<Self, InterfaceImportPathError>
    where
        S: AsRef<str>,
    {
        let Some((root_segment, remaining)) = segments.split_first() else {
            return Err(InterfaceImportPathError::EmptyPath);
        };

        let root_spelling = root_segment.as_ref();
        let root = ModuleKey::root(root_spelling).map_err(|error| {
            InterfaceImportPathError::InvalidSegment {
                segment: root_spelling.to_owned(),
                reason: error.to_string(),
            }
        })?;
        let mut validated_key = root.clone();
        let mut validated_segments = Vec::with_capacity(remaining.len());

        for segment in remaining {
            let spelling = segment.as_ref();
            validated_key = validated_key.child(spelling).map_err(|error| {
                InterfaceImportPathError::InvalidSegment {
                    segment: spelling.to_owned(),
                    reason: error.to_string(),
                }
            })?;
            validated_segments.push(spelling.to_owned());
        }

        Ok(Self {
            root,
            segments: validated_segments,
        })
    }

    /// Returns the canonical root module key for this path.
    #[must_use]
    pub fn root(&self) -> &ModuleKey {
        &self.root
    }

    /// Returns source-visible segments below the crate root.
    #[must_use]
    pub fn segments(&self) -> &[String] {
        &self.segments
    }
}

impl fmt::Display for InterfaceImportPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.root)?;
        for segment in &self.segments {
            write!(formatter, "::{segment}")?;
        }
        Ok(())
    }
}

/// One named member of a grouped interface import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceImportMember {
    source_name: String,
    local_name: String,
}

impl InterfaceImportMember {
    /// Imports `name` under the same local spelling.
    #[must_use]
    pub fn named(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            source_name: name.clone(),
            local_name: name,
        }
    }

    /// Imports `source_name` under `local_name` without changing identity.
    #[must_use]
    pub fn renamed(source_name: impl Into<String>, local_name: impl Into<String>) -> Self {
        Self {
            source_name: source_name.into(),
            local_name: local_name.into(),
        }
    }
}

/// One bounded request to import from a checked public interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterfaceImportRequest {
    /// Imports one public binding under an explicit local spelling.
    Explicit {
        /// Root-first path ending in the imported public binding.
        path: InterfaceImportPath,
        /// Local spelling selected by the import request.
        local_name: String,
    },
    /// Imports several public bindings atomically from one checked module.
    Group {
        /// Root-first path identifying the source module.
        path: InterfaceImportPath,
        /// Members to resolve before any local binding is published.
        members: Vec<InterfaceImportMember>,
    },
    /// Adds all public source-module bindings as deferred glob candidates.
    Glob {
        /// Root-first path identifying the source module.
        path: InterfaceImportPath,
    },
}

impl InterfaceImportRequest {
    /// Creates an explicit import with a local alias.
    #[must_use]
    pub fn explicit(path: InterfaceImportPath, local_name: impl Into<String>) -> Self {
        Self::Explicit {
            path,
            local_name: local_name.into(),
        }
    }

    /// Creates an atomic grouped import from one module path.
    #[must_use]
    pub fn group(path: InterfaceImportPath, members: Vec<InterfaceImportMember>) -> Self {
        Self::Group { path, members }
    }

    /// Creates a deferred glob import from one module path.
    #[must_use]
    pub fn glob(path: InterfaceImportPath) -> Self {
        Self::Glob { path }
    }
}

/// Import-resolution diagnostic for checked public interfaces.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InterfaceImportDiagnostic {
    /// The canonical path did not resolve a public binding or child module.
    #[error("unresolved public import {name:?} in module {module}")]
    UnresolvedImport {
        /// Checked module in which lookup failed.
        module: ModuleKey,
        /// Source-visible binding or child name that was not public.
        name: String,
    },
    /// A public child identity did not have a finalizer-issued interface.
    #[error("missing finalized checked interface for module {module}")]
    MissingCheckedInterface {
        /// Canonical child module identity missing from the checked store.
        module: ModuleKey,
    },
    /// An explicit local spelling would be published more than once.
    #[error("duplicate explicit local import binding {local_name:?}")]
    DuplicateLocalBinding {
        /// Local spelling claimed by multiple explicit/group requests.
        local_name: String,
    },
    /// Lookup found multiple distinct glob identities for one local spelling.
    #[error("ambiguous glob import binding {local_name:?}")]
    AmbiguousBinding {
        /// Local spelling with multiple distinct glob candidates.
        local_name: String,
    },
    /// Lookup found no explicit binding or glob candidate for a local spelling.
    #[error("unresolved local import binding {local_name:?}")]
    UnresolvedBinding {
        /// Local spelling absent from the import environment.
        local_name: String,
    },
}

/// Imported public bindings available to later type-checker-owned consumers.
///
/// Explicit bindings shadow glob candidates. Glob identity conflicts remain
/// deferred until [`Self::lookup`], so adding imports cannot publish a
/// premature winner.
#[derive(Debug, Default)]
pub struct InterfaceImportEnvironment {
    explicit_bindings: BTreeMap<String, ModuleInterfaceBinding>,
    glob_candidates:
        BTreeMap<String, BTreeMap<ModuleInterfaceDefiningIdentity, ModuleInterfaceBinding>>,
}

impl InterfaceImportEnvironment {
    /// Creates an empty checked-interface import environment.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolves one local spelling under explicit-over-glob precedence.
    ///
    /// # Errors
    ///
    /// Returns [`InterfaceImportDiagnostic::UnresolvedBinding`] when no
    /// imported binding has this local spelling, or
    /// [`InterfaceImportDiagnostic::AmbiguousBinding`] for distinct glob
    /// identities with no explicit winner.
    pub fn lookup(
        &self,
        local_name: &str,
    ) -> Result<&ModuleInterfaceBinding, InterfaceImportDiagnostic> {
        if let Some(binding) = self.explicit_bindings.get(local_name) {
            return Ok(binding);
        }

        let Some(candidates) = self.glob_candidates.get(local_name) else {
            return Err(InterfaceImportDiagnostic::UnresolvedBinding {
                local_name: local_name.to_owned(),
            });
        };
        if candidates.len() != 1 {
            return Err(InterfaceImportDiagnostic::AmbiguousBinding {
                local_name: local_name.to_owned(),
            });
        }

        Ok(candidates
            .values()
            .next()
            .expect("a single checked glob candidate has one binding"))
    }

    fn contains_explicit(&self, local_name: &str) -> bool {
        self.explicit_bindings.contains_key(local_name)
    }

    fn insert_explicit(&mut self, local_name: String, binding: ModuleInterfaceBinding) {
        self.explicit_bindings.insert(local_name, binding);
    }

    fn insert_glob_candidate(&mut self, binding: ModuleInterfaceBinding) {
        let local_name = binding.visible_name().to_owned();
        let identity = binding.defining_identity().clone();
        self.glob_candidates
            .entry(local_name)
            .or_default()
            .entry(identity)
            .or_insert(binding);
    }
}

/// Resolves bounded interface imports from finalizer-issued checked interfaces.
#[derive(Debug)]
pub struct InterfaceImportResolver {
    store: CheckedInterfaceStore,
}

impl InterfaceImportResolver {
    /// Creates a resolver over the supplied canonical checked-interface store.
    #[must_use]
    pub fn new(store: CheckedInterfaceStore) -> Self {
        Self { store }
    }

    /// Resolves requests against only checked public interface bindings.
    ///
    /// Grouped requests are atomic: no group member is published if any member
    /// is unresolved or duplicates an explicit local spelling.
    ///
    /// # Errors
    ///
    /// Returns [`InterfaceImportDiagnostic`] when a public path cannot be
    /// resolved, a checked child interface is absent, or an explicit local
    /// spelling conflicts.
    pub fn resolve(
        &self,
        environment: &mut InterfaceImportEnvironment,
        requests: &[InterfaceImportRequest],
    ) -> Result<(), InterfaceImportDiagnostic> {
        for request in requests {
            match request {
                InterfaceImportRequest::Explicit { path, local_name } => {
                    let binding = self.resolve_binding_path(path)?;
                    if environment.contains_explicit(local_name) {
                        return Err(InterfaceImportDiagnostic::DuplicateLocalBinding {
                            local_name: local_name.clone(),
                        });
                    }
                    environment.insert_explicit(local_name.clone(), binding.clone());
                }
                InterfaceImportRequest::Group { path, members } => {
                    self.resolve_group(environment, path, members)?;
                }
                InterfaceImportRequest::Glob { path } => {
                    let interface = self.resolve_module_path(path)?;
                    for binding in interface.interface().bindings() {
                        if binding.visibility() == Visibility::Public {
                            environment.insert_glob_candidate(binding.clone());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn resolve_group(
        &self,
        environment: &mut InterfaceImportEnvironment,
        path: &InterfaceImportPath,
        members: &[InterfaceImportMember],
    ) -> Result<(), InterfaceImportDiagnostic> {
        let interface = self.resolve_module_path(path)?;
        let mut staged = Vec::with_capacity(members.len());
        let mut staged_names = BTreeMap::new();

        for member in members {
            let binding = find_public_binding(interface, &member.source_name).ok_or_else(|| {
                InterfaceImportDiagnostic::UnresolvedImport {
                    module: interface.module_key().clone(),
                    name: member.source_name.clone(),
                }
            })?;
            if environment.contains_explicit(&member.local_name)
                || staged_names.insert(member.local_name.clone(), ()).is_some()
            {
                return Err(InterfaceImportDiagnostic::DuplicateLocalBinding {
                    local_name: member.local_name.clone(),
                });
            }
            staged.push((member.local_name.clone(), binding.clone()));
        }

        for (local_name, binding) in staged {
            environment.insert_explicit(local_name, binding);
        }

        Ok(())
    }

    fn resolve_binding_path(
        &self,
        path: &InterfaceImportPath,
    ) -> Result<&ModuleInterfaceBinding, InterfaceImportDiagnostic> {
        let Some((binding_name, module_segments)) = path.segments.split_last() else {
            return Err(InterfaceImportDiagnostic::UnresolvedImport {
                module: path.root.clone(),
                name: path.root.to_string(),
            });
        };
        let interface = self.resolve_module_segments(path.root(), module_segments)?;

        find_public_binding(interface, binding_name).ok_or_else(|| {
            InterfaceImportDiagnostic::UnresolvedImport {
                module: interface.module_key().clone(),
                name: binding_name.clone(),
            }
        })
    }

    fn resolve_module_path(
        &self,
        path: &InterfaceImportPath,
    ) -> Result<&FinalizedModuleInterface, InterfaceImportDiagnostic> {
        self.resolve_module_segments(path.root(), path.segments())
    }

    fn resolve_module_segments(
        &self,
        root: &ModuleKey,
        segments: &[String],
    ) -> Result<&FinalizedModuleInterface, InterfaceImportDiagnostic> {
        let mut module = root.clone();
        let mut interface = self.checked_interface(&module)?;

        for segment in segments {
            let binding = find_public_binding(interface, segment).ok_or_else(|| {
                InterfaceImportDiagnostic::UnresolvedImport {
                    module: interface.module_key().clone(),
                    name: segment.clone(),
                }
            })?;
            let ModuleInterfaceDefiningIdentity::ChildModule(child) = binding.defining_identity()
            else {
                return Err(InterfaceImportDiagnostic::UnresolvedImport {
                    module: interface.module_key().clone(),
                    name: segment.clone(),
                });
            };
            module = child.clone();
            interface = self.checked_interface(&module)?;
        }

        Ok(interface)
    }

    fn checked_interface(
        &self,
        module: &ModuleKey,
    ) -> Result<&FinalizedModuleInterface, InterfaceImportDiagnostic> {
        self.store.interface(module).ok_or_else(|| {
            InterfaceImportDiagnostic::MissingCheckedInterface {
                module: module.clone(),
            }
        })
    }
}

fn find_public_binding<'a>(
    interface: &'a FinalizedModuleInterface,
    visible_name: &str,
) -> Option<&'a ModuleInterfaceBinding> {
    interface.interface().bindings().iter().find(|binding| {
        binding.visible_name() == visible_name && binding.visibility() == Visibility::Public
    })
}
