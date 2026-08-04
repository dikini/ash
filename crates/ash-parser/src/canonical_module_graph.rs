//! Canonical parser-stage module graph construction.
//!
//! This module consumes parsed module declarations and the shared unit
//! acquisition primitives. It keeps canonical structural identity separate
//! from the compatibility-only resolver route.

use ash_core::module_graph::{ModuleArtifactOrigin, ModuleKey};
use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::module::{ModuleDecl, ModuleSource as ParsedModuleSource, ModuleUnit};
use crate::resolver::{
    CanonicalChildAcquisitionFailure, CanonicalMalformedInlineChild,
    CanonicalRootAcquisitionFailure, Fs, ModuleUnitResolver, ResolveError,
};
use crate::surface::CrateRootMetadata;
use crate::token::Span;

/// The structural state of one canonical module-graph entry.
///
/// This parser-stage state does not imply import binding, a checked interface,
/// or runtime admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalModuleState {
    /// The canonical key has no known structural entry.
    Absent,
    /// A parsed declaration has been observed but its unit is not yet ready.
    Discovered,
    /// The parser has acquired and retained a complete [`ModuleUnit`].
    Parsed,
    /// Structural acquisition failed for this key.
    Failed,
}

/// A compact owned fact retained by a canonical structural diagnostic.
///
/// The graph error exposes source anchors and canonical keys by value, while
/// this wrapper keeps the error enum small enough to propagate without
/// forcing every successful resolver result to carry its largest diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalDiagnosticValue<T>(Box<T>);

impl<T> CanonicalDiagnosticValue<T> {
    /// Returns the retained diagnostic fact.
    #[must_use]
    pub fn value(&self) -> &T {
        &self.0
    }

    /// Consumes the wrapper and returns the retained diagnostic fact.
    #[must_use]
    pub fn into_inner(self) -> T {
        *self.0
    }
}

impl<T> From<T> for CanonicalDiagnosticValue<T> {
    fn from(value: T) -> Self {
        Self(Box::new(value))
    }
}

impl<T> AsRef<T> for CanonicalDiagnosticValue<T> {
    fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

impl<T> std::ops::Deref for CanonicalDiagnosticValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value()
    }
}

impl<T> PartialEq<T> for CanonicalDiagnosticValue<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &T) -> bool {
        self.value() == other
    }
}

impl<T> fmt::Display for CanonicalDiagnosticValue<T>
where
    T: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value().fmt(formatter)
    }
}

/// An anchored structural diagnostic produced while constructing a canonical graph.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CanonicalStructuralDiagnostic {
    /// A parsed declaration's file-backed child could not be acquired.
    #[error("missing child module `{child_key}` (expected at {expected_path})")]
    MissingChild {
        /// The canonical identity derived from the parsed declaration.
        child_key: CanonicalDiagnosticValue<ModuleKey>,
        /// The first source location considered for the child unit.
        expected_path: PathBuf,
    },

    /// A file-backed declaration recursively acquired an active source unit.
    #[error("structural module cycle: {cycle:?}")]
    Cycle {
        /// Canonical keys along the reentrant structural path.
        cycle: Vec<ModuleKey>,
    },

    /// A parsed declaration repeated an already-declared canonical child key.
    #[error("duplicate child module `{child_key}` (first declared at {first_declaration_span:?})")]
    DuplicateChild {
        /// The canonical child identity derived from the parsed declaration.
        child_key: CanonicalDiagnosticValue<ModuleKey>,
        /// Parser-owned anchor of the declaration that first named this child.
        first_declaration_span: CanonicalDiagnosticValue<Span>,
    },

    /// An inline-module header parsed successfully but its body did not.
    #[error("malformed inline child module `{child_key}` at {error_span:?}")]
    MalformedInline {
        /// The canonical identity derived from the parsed inline header.
        child_key: CanonicalDiagnosticValue<ModuleKey>,
        /// Parser-owned span of the malformed body or closing delimiter.
        error_span: CanonicalDiagnosticValue<Span>,
    },
}

/// Errors produced by canonical parser module-graph construction.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CanonicalModuleGraphError {
    /// A structural error anchored at the parsed parent declaration.
    #[error("structural error at {parent_key}:{declaration_span:?}: {diagnostic}")]
    Structural {
        /// Canonical identity of the declaration's parent module.
        parent_key: CanonicalDiagnosticValue<ModuleKey>,
        /// Parser-owned declaration anchor in the parent source.
        declaration_span: CanonicalDiagnosticValue<Span>,
        /// The structural failure observed at this declaration.
        diagnostic: CanonicalStructuralDiagnostic,
        /// Canonical entries whose structural transition failed atomically.
        failed_keys: Box<[ModuleKey]>,
    },

    /// Root-unit acquisition failed before any canonical graph could be returned.
    #[error("failed to acquire root module `{root_key}` from {root_path}: {source}")]
    RootAcquisition {
        /// Canonical root identity requested by the caller.
        root_key: ModuleKey,
        /// Root source path supplied by the caller.
        root_path: PathBuf,
        /// The underlying source-acquisition failure.
        #[source]
        source: Box<ResolveError>,
    },

    /// A non-missing child acquisition error prevented graph construction.
    #[error("failed to acquire child of `{parent_key}` at {declaration_span:?}: {source}")]
    ChildAcquisition {
        /// Canonical identity of the child declaration's parent.
        parent_key: ModuleKey,
        /// Parser-owned declaration anchor in the parent source.
        declaration_span: Span,
        /// The underlying source-acquisition failure.
        #[source]
        source: Box<ResolveError>,
    },
}

impl CanonicalModuleGraphError {
    /// Returns the canonical structural entries whose transition failed.
    ///
    /// Only anchored structural rejections retain failed graph states. Source
    /// acquisition errors do not publish graph state and therefore report no
    /// failed canonical entries.
    #[must_use]
    pub fn failed_keys(&self) -> &[ModuleKey] {
        match self {
            Self::Structural { failed_keys, .. } => failed_keys,
            Self::RootAcquisition { .. } | Self::ChildAcquisition { .. } => &[],
        }
    }

    /// Returns the retained failure state for one canonical entry.
    ///
    /// The report is intentionally attached to the error rather than a
    /// partially constructed graph, so no failed entry exposes a usable unit.
    #[must_use]
    pub fn failed_state(&self, key: &ModuleKey) -> Option<CanonicalModuleState> {
        self.failed_keys()
            .contains(key)
            .then_some(CanonicalModuleState::Failed)
    }

    fn structural(
        parent_key: ModuleKey,
        declaration_span: Span,
        diagnostic: CanonicalStructuralDiagnostic,
    ) -> Self {
        let failed_keys = Self::failed_keys_for_diagnostic(&diagnostic);
        Self::Structural {
            parent_key: parent_key.into(),
            declaration_span: declaration_span.into(),
            diagnostic,
            failed_keys,
        }
    }

    fn failed_keys_for_diagnostic(diagnostic: &CanonicalStructuralDiagnostic) -> Box<[ModuleKey]> {
        match diagnostic {
            CanonicalStructuralDiagnostic::MissingChild { child_key, .. }
            | CanonicalStructuralDiagnostic::DuplicateChild { child_key, .. }
            | CanonicalStructuralDiagnostic::MalformedInline { child_key, .. } => {
                vec![child_key.value().clone()].into_boxed_slice()
            }
            CanonicalStructuralDiagnostic::Cycle { cycle } => {
                let mut failed_keys = Vec::with_capacity(cycle.len());
                for key in cycle {
                    if !failed_keys.contains(key) {
                        failed_keys.push(key.clone());
                    }
                }
                failed_keys.into_boxed_slice()
            }
        }
    }
}

/// A canonical-keyed, parser-stage module graph containing acquired module units.
///
/// The graph records only parsed structural declarations and source-acquired
/// [`ModuleUnit`] values. It is intentionally not an import, type-checking,
/// lowering, or runtime graph.
#[derive(Debug)]
pub struct CanonicalModuleGraph {
    root_key: ModuleKey,
    root_crate_metadata: Option<CrateRootMetadata>,
    children: BTreeMap<ModuleKey, Vec<ModuleKey>>,
    module_units: BTreeMap<ModuleKey, ModuleUnit>,
    states: BTreeMap<ModuleKey, CanonicalModuleState>,
}

impl CanonicalModuleGraph {
    /// Returns the canonical identity of this graph's root module.
    #[must_use]
    pub fn root_key(&self) -> &ModuleKey {
        &self.root_key
    }

    /// Returns the parser-owned crate preamble from the root source, if any.
    ///
    /// This provenance handoff does not resolve dependencies or bind imports.
    #[must_use]
    pub fn root_crate_metadata(&self) -> Option<&CrateRootMetadata> {
        self.root_crate_metadata.as_ref()
    }

    /// Returns direct structural children for `key` in canonical-key order.
    #[must_use]
    pub fn children(&self, key: &ModuleKey) -> Option<&[ModuleKey]> {
        self.children.get(key).map(Vec::as_slice)
    }

    /// Returns the actual source-acquired unit retained for `key`.
    #[must_use]
    pub fn module_unit(&self, key: &ModuleKey) -> Option<&ModuleUnit> {
        self.module_units.get(key)
    }

    /// Iterates over canonical module keys and their parser-owned units in
    /// canonical-key order.
    ///
    /// This is a read-only structural handoff. Consumers receive no source
    /// acquisition authority beyond the already retained typed units.
    pub fn module_units(&self) -> impl Iterator<Item = (&ModuleKey, &ModuleUnit)> {
        self.module_units.iter()
    }

    /// Returns the parser-stage structural state recorded for `key`.
    #[must_use]
    pub fn state(&self, key: &ModuleKey) -> Option<CanonicalModuleState> {
        self.states.get(key).copied()
    }

    /// Returns the parser-stage structural state for `key`, or [`CanonicalModuleState::Absent`].
    ///
    /// This is a read-only query: absent keys do not create graph entries or
    /// expose any partially acquired module state.
    #[must_use]
    pub fn state_or_absent(&self, key: &ModuleKey) -> CanonicalModuleState {
        self.state(key).unwrap_or(CanonicalModuleState::Absent)
    }
}

/// Builds a canonical parser graph from parsed module declarations.
///
/// Every child is acquired through [`ModuleUnitResolver`] from an AST
/// [`ModuleDecl`]. Source spelling and paths only guide acquisition; graph
/// membership and topology are keyed exclusively by [`ModuleKey`].
pub struct CanonicalModuleGraphResolver {
    unit_resolver: ModuleUnitResolver,
}

impl Default for CanonicalModuleGraphResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl CanonicalModuleGraphResolver {
    /// Creates a graph resolver backed by the real filesystem.
    #[must_use]
    pub fn new() -> Self {
        Self {
            unit_resolver: ModuleUnitResolver::new(),
        }
    }

    /// Creates a graph resolver with a testable filesystem implementation.
    ///
    /// This constructor only injects source acquisition. Structural graph
    /// identity and topology continue to be derived exclusively from parsed
    /// [`ModuleDecl`] values and canonical [`ModuleKey`] values.
    #[must_use]
    pub fn with_fs(fs: Box<dyn Fs>) -> Self {
        Self {
            unit_resolver: ModuleUnitResolver::with_fs(fs),
        }
    }

    /// Resolves `root_path` into an all-or-nothing canonical parser graph.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalModuleGraphError::Structural`] for a missing,
    /// duplicate, or reentrant file-backed child. Other source and parse
    /// failures retain their underlying [`ResolveError`] as acquisition
    /// errors.
    pub fn resolve_root(
        &self,
        root_key: ModuleKey,
        root_path: impl AsRef<Path>,
    ) -> Result<CanonicalModuleGraph, CanonicalModuleGraphError> {
        let root_path = root_path.as_ref().to_path_buf();
        let root_acquisition = self
            .unit_resolver
            .acquire_root_for_canonical_graph(root_key.clone(), &root_path)
            .map_err(|source| Self::root_acquisition_error(&root_key, &root_path, source))?;
        let mut builder = CanonicalModuleGraphBuilder::new(
            &self.unit_resolver,
            root_key.clone(),
            root_acquisition.crate_metadata,
        );
        builder.resolve_unit(root_key, root_path, root_acquisition.module_unit)?;
        Ok(builder.finish())
    }

    fn root_acquisition_error(
        root_key: &ModuleKey,
        root_path: &Path,
        source: CanonicalRootAcquisitionFailure,
    ) -> CanonicalModuleGraphError {
        match source {
            CanonicalRootAcquisitionFailure::MalformedInline {
                module_name,
                declaration_span,
                error_span,
            } => match root_key.child(&module_name) {
                Ok(child_key) => CanonicalModuleGraphError::structural(
                    root_key.clone(),
                    declaration_span,
                    CanonicalStructuralDiagnostic::MalformedInline {
                        child_key: child_key.into(),
                        error_span: error_span.into(),
                    },
                ),
                Err(error) => CanonicalModuleGraphError::RootAcquisition {
                    root_key: root_key.clone(),
                    root_path: root_path.to_path_buf(),
                    source: Box::new(ResolveError::InvalidModuleUnitIdentity {
                        module_name: module_name.into(),
                        parent_path: root_path.to_path_buf(),
                        declaration_span,
                        message: error.to_string(),
                    }),
                },
            },
            CanonicalRootAcquisitionFailure::Resolve(source) => match source {
                ResolveError::DuplicateModuleDeclarationWithSpans {
                    module_name,
                    first_declaration_span,
                    declaration_span,
                    ..
                } => match root_key.child(&module_name) {
                    Ok(child_key) => CanonicalModuleGraphError::structural(
                        root_key.clone(),
                        declaration_span,
                        CanonicalStructuralDiagnostic::DuplicateChild {
                            child_key: child_key.into(),
                            first_declaration_span: first_declaration_span.into(),
                        },
                    ),
                    Err(error) => CanonicalModuleGraphError::RootAcquisition {
                        root_key: root_key.clone(),
                        root_path: root_path.to_path_buf(),
                        source: Box::new(ResolveError::InvalidModuleUnitIdentity {
                            module_name,
                            parent_path: root_path.to_path_buf(),
                            declaration_span,
                            message: error.to_string(),
                        }),
                    },
                },
                source => CanonicalModuleGraphError::RootAcquisition {
                    root_key: root_key.clone(),
                    root_path: root_path.to_path_buf(),
                    source: Box::new(source),
                },
            },
        }
    }
}

/// Private, atomic construction state for [`CanonicalModuleGraphResolver`].
struct CanonicalModuleGraphBuilder<'a> {
    unit_resolver: &'a ModuleUnitResolver,
    graph: CanonicalModuleGraph,
    // A transient acquisition guard keyed by physical source provenance. It is
    // not graph state, a lookup/cache key, or a substitute for `ModuleKey`.
    active_file_sources: BTreeMap<PathBuf, ModuleKey>,
    active_keys: Vec<ModuleKey>,
}

impl<'a> CanonicalModuleGraphBuilder<'a> {
    fn new(
        unit_resolver: &'a ModuleUnitResolver,
        root_key: ModuleKey,
        root_crate_metadata: Option<CrateRootMetadata>,
    ) -> Self {
        Self {
            unit_resolver,
            graph: CanonicalModuleGraph {
                root_key,
                root_crate_metadata,
                children: BTreeMap::new(),
                module_units: BTreeMap::new(),
                states: BTreeMap::new(),
            },
            active_file_sources: BTreeMap::new(),
            active_keys: Vec::new(),
        }
    }

    fn resolve_unit(
        &mut self,
        key: ModuleKey,
        source_path: PathBuf,
        unit: ModuleUnit,
    ) -> Result<(), CanonicalModuleGraphError> {
        self.graph
            .states
            .insert(key.clone(), CanonicalModuleState::Discovered);
        let is_file_origin = matches!(unit.artifact().origin(), ModuleArtifactOrigin::File(_));
        if is_file_origin {
            self.active_file_sources
                .insert(source_path.clone(), key.clone());
        }
        self.active_keys.push(key.clone());

        for declaration in unit.body().module_decls() {
            if matches!(declaration.source, ParsedModuleSource::File) {
                let candidate_path = self
                    .unit_resolver
                    .resolve_child_path(&source_path, declaration)
                    .map_err(|source| {
                        Self::child_acquisition_error(
                            &key,
                            declaration,
                            CanonicalChildAcquisitionFailure::Resolve(source),
                        )
                    })?;
                if let Some(reentrant_key) = self.active_file_sources.get(&candidate_path) {
                    return Err(CanonicalModuleGraphError::structural(
                        key.clone(),
                        declaration.span,
                        CanonicalStructuralDiagnostic::Cycle {
                            cycle: self.cycle_for(reentrant_key),
                        },
                    ));
                }
            }

            let child_unit = self
                .unit_resolver
                .acquire_child_for_canonical_graph(&key, &source_path, declaration)
                .map_err(|source| Self::child_acquisition_error(&key, declaration, source))?;
            let child_key = child_unit.artifact().key().clone();
            let child_source_path = child_unit
                .source_path()
                .map(PathBuf::from)
                .unwrap_or_else(|| source_path.clone());

            self.resolve_unit(child_key, child_source_path, child_unit)?;
        }

        self.active_keys.pop();
        if is_file_origin {
            self.active_file_sources.remove(&source_path);
        }
        self.graph
            .children
            .insert(key.clone(), unit.artifact().child_keys().to_vec());
        self.graph.module_units.insert(key.clone(), unit);
        self.graph.states.insert(key, CanonicalModuleState::Parsed);
        Ok(())
    }

    fn child_acquisition_error(
        parent_key: &ModuleKey,
        declaration: &ModuleDecl,
        source: CanonicalChildAcquisitionFailure,
    ) -> CanonicalModuleGraphError {
        match source {
            CanonicalChildAcquisitionFailure::MalformedInline(detail) => {
                let CanonicalMalformedInlineChild {
                    module_name,
                    source_path,
                    declaration_span,
                    error_span,
                    ..
                } = *detail;
                let nested_parent_key = match parent_key.child(&declaration.name) {
                    Ok(child_key) => child_key,
                    Err(error) => {
                        return CanonicalModuleGraphError::ChildAcquisition {
                            parent_key: parent_key.clone(),
                            declaration_span: declaration.span,
                            source: Box::new(ResolveError::InvalidModuleUnitIdentity {
                                module_name: declaration.name.to_string(),
                                parent_path: source_path.clone(),
                                declaration_span: declaration.span,
                                message: error.to_string(),
                            }),
                        };
                    }
                };
                match nested_parent_key.child(&module_name) {
                    Ok(child_key) => CanonicalModuleGraphError::structural(
                        nested_parent_key,
                        declaration_span,
                        CanonicalStructuralDiagnostic::MalformedInline {
                            child_key: child_key.into(),
                            error_span: error_span.into(),
                        },
                    ),
                    Err(error) => CanonicalModuleGraphError::ChildAcquisition {
                        parent_key: parent_key.clone(),
                        declaration_span,
                        source: Box::new(ResolveError::InvalidModuleUnitIdentity {
                            module_name: module_name.into(),
                            parent_path: source_path,
                            declaration_span,
                            message: error.to_string(),
                        }),
                    },
                }
            }
            CanonicalChildAcquisitionFailure::Resolve(ResolveError::ModuleUnitNotFound {
                parent_path,
                expected_path,
                ..
            }) => match parent_key.child(&declaration.name) {
                Ok(child_key) => CanonicalModuleGraphError::structural(
                    parent_key.clone(),
                    declaration.span,
                    CanonicalStructuralDiagnostic::MissingChild {
                        child_key: child_key.into(),
                        expected_path,
                    },
                ),
                Err(error) => CanonicalModuleGraphError::ChildAcquisition {
                    parent_key: parent_key.clone(),
                    declaration_span: declaration.span,
                    source: Box::new(ResolveError::InvalidModuleUnitIdentity {
                        module_name: declaration.name.to_string(),
                        parent_path,
                        declaration_span: declaration.span,
                        message: error.to_string(),
                    }),
                },
            },
            CanonicalChildAcquisitionFailure::Resolve(
                ResolveError::DuplicateModuleDeclarationWithSpans {
                    module_name,
                    path,
                    first_declaration_span,
                    declaration_span,
                    ..
                },
            ) => {
                let duplicate_parent_key = match parent_key.child(&declaration.name) {
                    Ok(child_key) => child_key,
                    Err(error) => {
                        return CanonicalModuleGraphError::ChildAcquisition {
                            parent_key: parent_key.clone(),
                            declaration_span: declaration.span,
                            source: Box::new(ResolveError::InvalidModuleUnitIdentity {
                                module_name: declaration.name.to_string(),
                                parent_path: path,
                                declaration_span: declaration.span,
                                message: error.to_string(),
                            }),
                        };
                    }
                };
                match duplicate_parent_key.child(&module_name) {
                    Ok(child_key) => CanonicalModuleGraphError::structural(
                        duplicate_parent_key,
                        declaration_span,
                        CanonicalStructuralDiagnostic::DuplicateChild {
                            child_key: child_key.into(),
                            first_declaration_span: first_declaration_span.into(),
                        },
                    ),
                    Err(error) => CanonicalModuleGraphError::ChildAcquisition {
                        parent_key: parent_key.clone(),
                        declaration_span,
                        source: Box::new(ResolveError::InvalidModuleUnitIdentity {
                            module_name,
                            parent_path: path,
                            declaration_span,
                            message: error.to_string(),
                        }),
                    },
                }
            }
            CanonicalChildAcquisitionFailure::Resolve(source) => {
                CanonicalModuleGraphError::ChildAcquisition {
                    parent_key: parent_key.clone(),
                    declaration_span: declaration.span,
                    source: Box::new(source),
                }
            }
        }
    }

    fn cycle_for(&self, reentrant_key: &ModuleKey) -> Vec<ModuleKey> {
        let mut cycle = self
            .active_keys
            .iter()
            .skip_while(|key| *key != reentrant_key)
            .cloned()
            .collect::<Vec<_>>();
        cycle.push(reentrant_key.clone());
        cycle
    }

    fn finish(self) -> CanonicalModuleGraph {
        self.graph
    }
}
