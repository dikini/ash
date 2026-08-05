//! Name-view-only resolution for parsed module imports.
//!
//! This is the TASK-2072 binding boundary.  It consumes parser-owned `use`
//! syntax and the import-facing [`CanonicalProvisionalNameView`] only.  The
//! graph is used for already-acquired structural keys and artifact origins;
//! declaration lookup never falls back to source or checker facts.

use std::collections::{BTreeMap, BTreeSet};

use ash_core::module_graph::{ModuleArtifactOrigin, ModuleKey};
use ash_parser::surface::{
    NormalizedNotationPatternKey, Visibility, normalized_notation_pattern_key,
};
use ash_parser::{CanonicalModuleGraph, ModuleItem, Span, Use, UsePath};
use thiserror::Error;

#[cfg(test)]
use crate::canonical_module_collection::CanonicalCollectedEntry;
use crate::canonical_module_collection::{
    CanonicalDeclarationIdentity, CanonicalLookupKey, CanonicalModuleCollection,
    CanonicalNamespace, CanonicalProvisionalNameEntry, CanonicalProvisionalNameView,
};

#[cfg(test)]
pub(crate) use ash_parser::CanonicalModuleGraphResolver as GraphResolver;

/// One staged parsed import binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalParsedImportBinding {
    local_name: Box<str>,
    defining_identity: CanonicalDeclarationIdentity,
    lookup_key: CanonicalLookupKey,
    declaration_span: Span,
    use_span: Span,
    member_span: Option<Span>,
    attempted_access_path: Box<[Box<str>]>,
    origin: ModuleArtifactOrigin,
    declaration_visibility: Visibility,
    import_visibility: Visibility,
    externally_public_reexport: bool,
    source_ordinal: usize,
    reexport: bool,
}

impl CanonicalParsedImportBinding {
    /// Returns the name introduced in the importing module.
    #[must_use]
    pub fn local_name(&self) -> &str {
        &self.local_name
    }

    /// Returns the stable defining declaration identity.
    #[must_use]
    pub fn defining_identity(&self) -> &CanonicalDeclarationIdentity {
        &self.defining_identity
    }

    /// Returns the namespace-qualified lookup key.
    #[must_use]
    pub fn lookup_key(&self) -> &CanonicalLookupKey {
        &self.lookup_key
    }

    /// Returns the declaration source anchor.
    #[must_use]
    pub const fn declaration_span(&self) -> Span {
        self.declaration_span
    }

    /// Returns the complete `use` source anchor.
    #[must_use]
    pub const fn use_span(&self) -> Span {
        self.use_span
    }

    /// Returns a grouped member anchor when this binding came from a group.
    #[must_use]
    pub const fn member_span(&self) -> Option<Span> {
        self.member_span
    }

    /// Returns the canonical path attempted by this import binding.
    #[must_use]
    pub fn attempted_access_path(&self) -> &[Box<str>] {
        &self.attempted_access_path
    }

    /// Returns source-acquisition provenance of the defining module.
    #[must_use]
    pub fn origin(&self) -> &ModuleArtifactOrigin {
        &self.origin
    }

    /// Returns the defining declaration visibility.
    #[must_use]
    pub fn declaration_visibility(&self) -> &Visibility {
        &self.declaration_visibility
    }

    /// Returns visibility attached to the import or re-export.
    #[must_use]
    pub fn import_visibility(&self) -> &Visibility {
        &self.import_visibility
    }

    /// Reports whether this binding remains externally public across its re-export chain.
    #[must_use]
    pub const fn is_externally_public_reexport(&self) -> bool {
        self.externally_public_reexport
    }

    /// Returns the defining declaration's source ordinal.
    #[must_use]
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    /// Reports whether this binding is staged as a public/restricted re-export.
    #[must_use]
    pub const fn is_reexport(&self) -> bool {
        self.reexport
    }
}

/// One dependency edge retained for cycle checking and later diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalParsedImportEdge {
    importing_module: ModuleKey,
    defining_module: ModuleKey,
    binding: CanonicalParsedImportBinding,
}

impl CanonicalParsedImportEdge {
    /// Returns the importing module.
    #[must_use]
    pub fn importing_module(&self) -> &ModuleKey {
        &self.importing_module
    }

    /// Returns the defining module.
    #[must_use]
    pub fn defining_module(&self) -> &ModuleKey {
        &self.defining_module
    }

    /// Returns the staged edge binding facts.
    #[must_use]
    pub fn binding(&self) -> &CanonicalParsedImportBinding {
        &self.binding
    }
}

/// One staged `pub use` fact.  It is not a final export closure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalStagedPublicUse {
    importing_module: ModuleKey,
    binding: CanonicalParsedImportBinding,
}

impl CanonicalStagedPublicUse {
    /// Returns the module receiving the re-export.
    #[must_use]
    pub fn importing_module(&self) -> &ModuleKey {
        &self.importing_module
    }

    /// Returns the non-authorizing binding facts.
    #[must_use]
    pub fn binding(&self) -> &CanonicalParsedImportBinding {
        &self.binding
    }
}

/// One notation import carried without creating an ordinary binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalParsedNotationImport {
    importing_module: ModuleKey,
    provider_module: ModuleKey,
    defining_identity: CanonicalDeclarationIdentity,
    lookup_key: CanonicalLookupKey,
    pattern: NormalizedNotationPatternKey,
    use_span: Span,
    declaration_span: Span,
    origin: ModuleArtifactOrigin,
    declaration_visibility: Visibility,
    source_ordinal: usize,
}

impl CanonicalParsedNotationImport {
    /// Returns the importing module.
    #[must_use]
    pub fn importing_module(&self) -> &ModuleKey {
        &self.importing_module
    }

    /// Returns the notation provider module.
    #[must_use]
    pub fn provider_module(&self) -> &ModuleKey {
        &self.provider_module
    }

    /// Returns the defining notation identity.
    #[must_use]
    pub fn defining_identity(&self) -> &CanonicalDeclarationIdentity {
        &self.defining_identity
    }

    /// Returns the complete typed notation lookup identity.
    #[must_use]
    pub fn lookup_key(&self) -> &CanonicalLookupKey {
        &self.lookup_key
    }

    /// Returns the normalized selector identity.
    #[must_use]
    pub fn pattern(&self) -> &NormalizedNotationPatternKey {
        &self.pattern
    }

    /// Returns the import source span.
    #[must_use]
    pub const fn use_span(&self) -> Span {
        self.use_span
    }

    /// Returns the declaration source span.
    #[must_use]
    pub const fn declaration_span(&self) -> Span {
        self.declaration_span
    }

    /// Returns the provider's source origin.
    #[must_use]
    pub fn origin(&self) -> &ModuleArtifactOrigin {
        &self.origin
    }

    /// Returns the provider declaration visibility.
    #[must_use]
    pub fn declaration_visibility(&self) -> &Visibility {
        &self.declaration_visibility
    }

    /// Returns the notation declaration's source ordinal.
    #[must_use]
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }
}

/// A complete, atomically published parsed-import result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalParsedImportResult {
    bindings: BTreeMap<ModuleKey, BTreeMap<Box<str>, CanonicalParsedImportBinding>>,
    edges: Box<[CanonicalParsedImportEdge]>,
    public_uses: Box<[CanonicalStagedPublicUse]>,
    notation_imports: Box<[CanonicalParsedNotationImport]>,
}

impl CanonicalParsedImportResult {
    /// Looks up a selected imported binding.
    #[must_use]
    pub fn binding(&self, module: &ModuleKey, name: &str) -> Option<&CanonicalParsedImportBinding> {
        self.bindings.get(module)?.get(name)
    }

    /// Iterates over every selected binding with its importing module and local name.
    ///
    /// The iterator is read-only and source-layout independent. Downstream
    /// finalization can enumerate the complete staged handoff without gaining
    /// access to the resolver's private staging maps.
    pub fn bindings(
        &self,
    ) -> impl Iterator<Item = (&ModuleKey, &str, &CanonicalParsedImportBinding)> + '_ {
        self.bindings.iter().flat_map(|(module, bindings)| {
            bindings
                .iter()
                .map(move |(name, binding)| (module, name.as_ref(), binding))
        })
    }

    /// Returns all dependency edges, including shadowed imports.
    #[must_use]
    pub fn import_edges(&self) -> &[CanonicalParsedImportEdge] {
        &self.edges
    }

    /// Returns staged public-use facts only.
    #[must_use]
    pub fn public_uses(&self) -> &[CanonicalStagedPublicUse] {
        &self.public_uses
    }

    /// Returns notation imports, which never enter ordinary bindings.
    #[must_use]
    pub fn notation_imports(&self) -> &[CanonicalParsedNotationImport] {
        &self.notation_imports
    }

    /// Returns a source-layout-independent binding identity projection.
    #[must_use]
    pub fn normalized_projection(
        &self,
    ) -> Vec<(
        ModuleKey,
        Box<str>,
        CanonicalDeclarationIdentity,
        CanonicalNamespace,
    )> {
        self.bindings
            .iter()
            .flat_map(|(module, bindings)| {
                bindings.iter().map(|(name, binding)| {
                    (
                        module.clone(),
                        name.clone(),
                        binding.defining_identity.clone(),
                        binding.lookup_key.namespace(),
                    )
                })
            })
            .collect()
    }
}

#[cfg(test)]
pub(crate) fn clone_with_binding_lookup_namespace(
    result: &CanonicalParsedImportResult,
    importing_module: &ModuleKey,
    local_name: &str,
    namespace: CanonicalNamespace,
) -> Option<CanonicalParsedImportResult> {
    let mut forged = result.clone();
    let binding = forged
        .bindings
        .get_mut(importing_module)?
        .get_mut(local_name)?;
    binding.lookup_key = crate::canonical_module_collection::clone_lookup_key_with_namespace(
        &binding.lookup_key,
        namespace,
    );
    Some(forged)
}

#[cfg(test)]
pub(crate) fn clone_with_binding_defining_target(
    result: &CanonicalParsedImportResult,
    importing_module: &ModuleKey,
    local_name: &str,
    target: &CanonicalCollectedEntry,
    declaration_visibility: &Visibility,
    origin: &ModuleArtifactOrigin,
) -> Option<CanonicalParsedImportResult> {
    let mut forged = result.clone();
    let binding = forged
        .bindings
        .get_mut(importing_module)?
        .get_mut(local_name)?;
    binding.defining_identity = target.identity().clone();
    binding.declaration_span = target.source_anchor();
    binding.origin = origin.clone();
    binding.declaration_visibility = declaration_visibility.clone();
    binding.source_ordinal = match target.identity().origin_key() {
        crate::canonical_module_collection::CanonicalDeclarationOriginKey::Source {
            source_ordinal,
        }
        | crate::canonical_module_collection::CanonicalDeclarationOriginKey::Expanded {
            source_ordinal,
            ..
        } => *source_ordinal,
    };
    Some(forged)
}

#[cfg(test)]
pub(crate) fn clone_with_binding_local_name(
    result: &CanonicalParsedImportResult,
    importing_module: &ModuleKey,
    local_name: &str,
    forged_local_name: &str,
) -> Option<CanonicalParsedImportResult> {
    let mut forged = result.clone();
    let binding = forged
        .bindings
        .get_mut(importing_module)?
        .get_mut(local_name)?;
    binding.local_name = forged_local_name.into();
    Some(forged)
}

#[cfg(test)]
pub(crate) fn clone_with_binding_declaration_span(
    result: &CanonicalParsedImportResult,
    importing_module: &ModuleKey,
    local_name: &str,
    declaration_span: Span,
) -> Option<CanonicalParsedImportResult> {
    let mut forged = result.clone();
    let binding = forged
        .bindings
        .get_mut(importing_module)?
        .get_mut(local_name)?;
    binding.declaration_span = declaration_span;
    Some(forged)
}

#[cfg(test)]
pub(crate) fn clone_with_binding_source_ordinal(
    result: &CanonicalParsedImportResult,
    importing_module: &ModuleKey,
    local_name: &str,
    source_ordinal: usize,
) -> Option<CanonicalParsedImportResult> {
    let mut forged = result.clone();
    let binding = forged
        .bindings
        .get_mut(importing_module)?
        .get_mut(local_name)?;
    binding.source_ordinal = source_ordinal;
    Some(forged)
}

#[cfg(test)]
pub(crate) fn clone_with_public_use_binding_reexport(
    result: &CanonicalParsedImportResult,
    importing_module: &ModuleKey,
    local_name: &str,
    reexport: bool,
) -> Option<CanonicalParsedImportResult> {
    let mut forged = result.clone();
    let public_use = forged.public_uses.iter_mut().find(|public_use| {
        public_use.importing_module == *importing_module
            && public_use.binding.local_name.as_ref() == local_name
    })?;
    public_use.binding.reexport = reexport;
    Some(forged)
}

#[cfg(test)]
pub(crate) fn clone_with_public_use_binding_declaration_span(
    result: &CanonicalParsedImportResult,
    importing_module: &ModuleKey,
    local_name: &str,
    declaration_span: Span,
) -> Option<CanonicalParsedImportResult> {
    let mut forged = result.clone();
    let public_use = forged.public_uses.iter_mut().find(|public_use| {
        public_use.importing_module == *importing_module
            && public_use.binding.local_name.as_ref() == local_name
    })?;
    public_use.binding.declaration_span = declaration_span;
    Some(forged)
}

/// An ordered cycle of parsed import edges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalParsedImportCycle {
    edges: Box<[CanonicalParsedImportEdge]>,
    modules: Box<[ModuleKey]>,
}

impl CanonicalParsedImportCycle {
    /// Returns the ordered cycle edges, including the closing edge.
    #[must_use]
    pub fn edges(&self) -> &[CanonicalParsedImportEdge] {
        &self.edges
    }

    /// Returns the ordered module path of the cycle, including its closing module.
    #[must_use]
    pub fn modules(&self) -> &[ModuleKey] {
        &self.modules
    }
}

/// Failure that prevents publication of the complete import result.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum CanonicalParsedImportError {
    /// Collection and graph keys differ.
    #[error("parsed-import collection does not match the acquired module graph")]
    GraphMismatch,
    /// A path does not resolve to a name-view entry.
    #[error("unresolved parsed import path {path:?}")]
    Unresolved {
        module: ModuleKey,
        path: Vec<Box<str>>,
        span: Span,
    },
    /// A resolved target is not visible to the importer.
    #[error("inaccessible parsed import path {path:?}")]
    Inaccessible {
        module: ModuleKey,
        defining_module: ModuleKey,
        path: Vec<Box<str>>,
        declaration_span: Span,
        violated_visibility: Visibility,
        span: Span,
    },
    /// Multiple namespace candidates match one unqualified reference.
    #[error("ambiguous parsed import path {path:?}")]
    Ambiguous {
        module: ModuleKey,
        path: Vec<Box<str>>,
        candidates: Box<[CanonicalDeclarationIdentity]>,
        span: Span,
    },
    /// A local name would be published more than once.
    #[error("duplicate parsed import binding {name:?}")]
    DuplicateBinding {
        module: ModuleKey,
        name: Box<str>,
        span: Span,
    },
    /// The parsed form is not admitted by this Type-layer route.
    #[error("unsupported parsed import form: {reason}")]
    Unsupported { span: Span, reason: &'static str },
    /// A complete dependency cycle was found before publication.
    #[error("parsed import cycle")]
    ImportCycle { cycle: CanonicalParsedImportCycle },
}

/// Resolve every parsed `use` in an acquired graph against provisional names.
///
/// The graph supplies only structural module keys and durable artifact origins;
/// declaration identity, namespace, visibility, and names come from the
/// collection's provisional views.  All candidates are staged before either
/// bindings or re-export facts are returned.
#[allow(clippy::result_large_err)]
pub fn resolve_parsed_imports_from_collection(
    graph: &CanonicalModuleGraph,
    collection: &CanonicalModuleCollection,
) -> Result<CanonicalParsedImportResult, CanonicalParsedImportError> {
    let graph_keys = graph
        .module_units()
        .map(|(key, _)| key.clone())
        .collect::<BTreeSet<_>>();
    let collection_keys = collection
        .modules()
        .map(|module| module.module_key().clone())
        .collect::<BTreeSet<_>>();
    if graph_keys != collection_keys {
        return Err(CanonicalParsedImportError::GraphMismatch);
    }

    let mut all_edges = Vec::new();
    let mut all_public = Vec::new();
    let mut all_notation = Vec::new();
    let mut all_bindings = BTreeMap::new();
    let staged_reexports = collect_staged_reexports(graph, collection)?;

    for (module_key, unit) in graph.module_units() {
        let view = collection
            .provisional_name_view(module_key)
            .ok_or(CanonicalParsedImportError::GraphMismatch)?;
        let local_names = names_by_spelling(view);
        let mut explicit = BTreeMap::<Box<str>, Vec<Candidate>>::new();
        let mut glob = BTreeMap::<Box<str>, Vec<Candidate>>::new();

        for item in unit.body().items() {
            let ModuleItem::Use(use_declaration) = item else {
                continue;
            };
            match &use_declaration.path {
                UsePath::Notation { module, selector } => {
                    let provider = resolve_module_path(
                        graph,
                        collection,
                        module_key,
                        &module.segments,
                        use_declaration.span,
                    )?;
                    if use_declaration.alias.is_some()
                        || !matches!(use_declaration.visibility, Visibility::Inherited)
                    {
                        return Err(CanonicalParsedImportError::Unsupported {
                            span: use_declaration.span,
                            reason: "notation imports are inherited and cannot be aliased",
                        });
                    }
                    let pattern = normalized_notation_pattern_key(&selector.parts);
                    let matching_entries = collection
                        .provisional_name_view(&provider)
                        .ok_or(CanonicalParsedImportError::GraphMismatch)?
                        .entries()
                        .filter(|entry| {
                            entry.namespace() == CanonicalNamespace::Notation
                                && entry
                                    .lookup_key()
                                    .notation_key()
                                    .is_some_and(|key| key.pattern() == &pattern)
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    if matching_entries.is_empty() {
                        return Err(CanonicalParsedImportError::Unresolved {
                            module: module_key.clone(),
                            path: module.segments.clone(),
                            span: use_declaration.span,
                        });
                    }
                    let mut entries = matching_entries
                        .iter()
                        .filter(|entry| entry.is_exportable())
                        .cloned()
                        .collect::<Vec<_>>();
                    if entries.is_empty() {
                        let entry = matching_entries
                            .first()
                            .expect("matching notation entries are non-empty");
                        return Err(CanonicalParsedImportError::Inaccessible {
                            module: module_key.clone(),
                            defining_module: entry.identity().module_key().clone(),
                            path: module.segments.clone(),
                            declaration_span: entry.origin_anchor(),
                            violated_visibility: entry.visibility().clone(),
                            span: use_declaration.span,
                        });
                    }
                    entries.sort_by_key(|entry| entry.lookup_key().notation_key().cloned());
                    for entry in entries {
                        let origin = graph
                            .module_unit(entry.identity().module_key())
                            .ok_or(CanonicalParsedImportError::GraphMismatch)?
                            .artifact()
                            .origin()
                            .clone();
                        all_notation.push(CanonicalParsedNotationImport {
                            importing_module: module_key.clone(),
                            provider_module: provider.clone(),
                            defining_identity: entry.identity().clone(),
                            lookup_key: entry.lookup_key().clone(),
                            pattern: pattern.clone(),
                            use_span: use_declaration.span,
                            declaration_span: entry.origin_anchor(),
                            origin,
                            declaration_visibility: entry.visibility().clone(),
                            source_ordinal: entry.source_ordinal(),
                        });
                    }
                }
                UsePath::Glob(path) => {
                    let attempted_access_path = glob_access_path(&path.segments);
                    let provider = resolve_module_path(
                        graph,
                        collection,
                        module_key,
                        &path.segments,
                        use_declaration.span,
                    )?;
                    if use_declaration.alias.is_some() {
                        return Err(CanonicalParsedImportError::Unsupported {
                            span: use_declaration.span,
                            reason: "glob imports cannot carry an alias",
                        });
                    }
                    let provider_view = collection
                        .provisional_name_view(&provider)
                        .ok_or(CanonicalParsedImportError::GraphMismatch)?;
                    for entry in provider_view.entries() {
                        if entry.identity().canonical_parent().is_some()
                            || entry.namespace() == CanonicalNamespace::Notation
                        {
                            continue;
                        }
                        let visibility_owner = entry_visibility_owner(entry);
                        if !entry.is_exportable()
                            && !visible_from(
                                entry.visibility(),
                                &visibility_owner,
                                module_key,
                                graph.root_key(),
                            )
                        {
                            continue;
                        }
                        let candidate = make_candidate(
                            graph,
                            module_key,
                            use_declaration,
                            &attempted_access_path,
                            None,
                            entry,
                            entry.lookup_name().into(),
                        )?;
                        all_edges.extend(candidate.edge.clone());
                        glob.entry(candidate.binding.local_name.clone())
                            .or_default()
                            .push(candidate);
                    }
                    let provider_exports =
                        staged_reexports.get(&provider).cloned().unwrap_or_default();
                    for binding in provider_exports.values() {
                        if provider_view.entries().any(|entry| {
                            entry.identity().canonical_parent().is_none()
                                && entry.lookup_name() == binding.local_name()
                        }) {
                            continue;
                        }
                        let candidate = make_reexport_candidate(
                            graph,
                            module_key,
                            use_declaration,
                            &attempted_access_path,
                            None,
                            binding.local_name().into(),
                            &provider,
                            binding,
                        )?;
                        all_edges.extend(candidate.edge.clone());
                        glob.entry(candidate.binding.local_name.clone())
                            .or_default()
                            .push(candidate);
                    }
                }
                UsePath::Simple(path) => {
                    let candidate = resolve_named_candidate(
                        graph,
                        collection,
                        module_key,
                        use_declaration,
                        &path.segments,
                        use_declaration.alias.clone(),
                        None,
                        &staged_reexports,
                    )?;
                    all_edges.extend(candidate.edge.clone());
                    explicit
                        .entry(candidate.binding.local_name.clone())
                        .or_default()
                        .push(candidate);
                }
                UsePath::Nested(path, items) => {
                    if items.is_empty() {
                        return Err(CanonicalParsedImportError::Unsupported {
                            span: use_declaration.span,
                            reason: "nested imports must contain at least one member",
                        });
                    }
                    if use_declaration.alias.is_some() {
                        return Err(CanonicalParsedImportError::Unsupported {
                            span: use_declaration.span,
                            reason: "nested imports cannot carry an outer alias",
                        });
                    }
                    for member in items {
                        let mut member_path = path.segments.clone();
                        member_path.push(member.name.clone());
                        let candidate = resolve_named_candidate(
                            graph,
                            collection,
                            module_key,
                            use_declaration,
                            &member_path,
                            member.alias.clone(),
                            Some(member.span),
                            &staged_reexports,
                        )?;
                        all_edges.extend(candidate.edge.clone());
                        explicit
                            .entry(candidate.binding.local_name.clone())
                            .or_default()
                            .push(candidate);
                    }
                }
            }
        }

        let mut selected = BTreeMap::new();
        let names = explicit
            .keys()
            .chain(glob.keys())
            .cloned()
            .collect::<BTreeSet<_>>();
        for name in names {
            let explicit_candidates = explicit.get(&name).map_or(&[][..], Vec::as_slice);
            let glob_candidates = glob.get(&name).map_or(&[][..], Vec::as_slice);
            if let Some(local) = local_names.get(&name) {
                if explicit_candidates
                    .iter()
                    .chain(glob_candidates)
                    .any(|candidate| candidate.binding.reexport)
                {
                    return Err(CanonicalParsedImportError::DuplicateBinding {
                        module: module_key.clone(),
                        name,
                        span: explicit_candidates
                            .first()
                            .or_else(|| glob_candidates.first())
                            .map_or(local.origin_anchor(), |candidate| {
                                candidate.binding.use_span
                            }),
                    });
                }
                continue;
            }
            let candidates = if !explicit_candidates.is_empty() {
                explicit_candidates
            } else {
                glob_candidates
            };
            if candidates.len() > 1 {
                let mut identities = Vec::<CanonicalDeclarationIdentity>::new();
                for candidate in candidates {
                    if !identities.contains(&candidate.binding.defining_identity) {
                        identities.push(candidate.binding.defining_identity.clone());
                    }
                }
                let span = candidates[1].binding.use_span;
                if identities.len() == 1 {
                    return Err(CanonicalParsedImportError::DuplicateBinding {
                        module: module_key.clone(),
                        name,
                        span,
                    });
                }
                return Err(CanonicalParsedImportError::Ambiguous {
                    module: module_key.clone(),
                    path: vec![name.clone()],
                    candidates: identities.into_boxed_slice(),
                    span,
                });
            }
            if let Some(candidate) = candidates.first() {
                if candidate.binding.reexport {
                    all_public.push(CanonicalStagedPublicUse {
                        importing_module: module_key.clone(),
                        binding: candidate.binding.clone(),
                    });
                }
                selected.insert(name, candidate.binding.clone());
            }
        }
        if !selected.is_empty() {
            all_bindings.insert(module_key.clone(), selected);
        }
    }

    if let Some(cycle) = find_cycle(&all_edges) {
        return Err(CanonicalParsedImportError::ImportCycle { cycle });
    }
    all_public.sort_by_key(|public_use| {
        (
            public_use.importing_module.clone(),
            public_use.binding.use_span.start,
            public_use.binding.use_span.end,
        )
    });
    Ok(CanonicalParsedImportResult {
        bindings: all_bindings,
        edges: all_edges.into_boxed_slice(),
        public_uses: all_public.into_boxed_slice(),
        notation_imports: all_notation.into_boxed_slice(),
    })
}

#[derive(Debug, Clone)]
struct Candidate {
    binding: CanonicalParsedImportBinding,
    edge: Option<CanonicalParsedImportEdge>,
}

type StagedReexports = BTreeMap<ModuleKey, BTreeMap<Box<str>, CanonicalParsedImportBinding>>;

#[derive(Debug, Clone)]
enum NamedTarget {
    Declaration {
        entry: CanonicalProvisionalNameEntry,
    },
    Reexport {
        provider: ModuleKey,
        binding: Box<CanonicalParsedImportBinding>,
    },
}

impl NamedTarget {
    fn identity(&self) -> &CanonicalDeclarationIdentity {
        match self {
            Self::Declaration { entry, .. } => entry.identity(),
            Self::Reexport { binding, .. } => binding.defining_identity(),
        }
    }
}

fn names_by_spelling(
    view: &CanonicalProvisionalNameView,
) -> BTreeMap<Box<str>, CanonicalProvisionalNameEntry> {
    view.entries()
        .filter(|entry| entry.identity().canonical_parent().is_none())
        .cloned()
        .map(|entry| (entry.lookup_name().into(), entry))
        .collect()
}

#[allow(clippy::result_large_err)]
fn collect_staged_reexports(
    graph: &CanonicalModuleGraph,
    collection: &CanonicalModuleCollection,
) -> Result<StagedReexports, CanonicalParsedImportError> {
    let mut staged = StagedReexports::new();
    loop {
        let mut changed = false;
        for (module_key, unit) in graph.module_units() {
            for item in unit.body().items() {
                let ModuleItem::Use(use_declaration) = item else {
                    continue;
                };
                if matches!(use_declaration.visibility, Visibility::Inherited) {
                    continue;
                }
                match &use_declaration.path {
                    UsePath::Notation { .. } => {
                        return Err(CanonicalParsedImportError::Unsupported {
                            span: use_declaration.span,
                            reason: "notation imports cannot be re-exported",
                        });
                    }
                    UsePath::Simple(path) => {
                        let candidate = match resolve_named_candidate(
                            graph,
                            collection,
                            module_key,
                            use_declaration,
                            &path.segments,
                            use_declaration.alias.clone(),
                            None,
                            &staged,
                        ) {
                            Ok(candidate) => candidate,
                            Err(CanonicalParsedImportError::Unresolved { .. }) => continue,
                            Err(error) => return Err(error),
                        };
                        changed |= stage_reexport(&mut staged, module_key, candidate)?;
                    }
                    UsePath::Nested(path, members) => {
                        if members.is_empty() {
                            return Err(CanonicalParsedImportError::Unsupported {
                                span: use_declaration.span,
                                reason: "nested imports must contain at least one member",
                            });
                        }
                        if use_declaration.alias.is_some() {
                            return Err(CanonicalParsedImportError::Unsupported {
                                span: use_declaration.span,
                                reason: "nested imports cannot carry an outer alias",
                            });
                        }
                        for member in members {
                            let mut member_path = path.segments.clone();
                            member_path.push(member.name.clone());
                            let candidate = match resolve_named_candidate(
                                graph,
                                collection,
                                module_key,
                                use_declaration,
                                &member_path,
                                member.alias.clone(),
                                Some(member.span),
                                &staged,
                            ) {
                                Ok(candidate) => candidate,
                                Err(CanonicalParsedImportError::Unresolved { .. }) => continue,
                                Err(error) => return Err(error),
                            };
                            changed |= stage_reexport(&mut staged, module_key, candidate)?;
                        }
                    }
                    UsePath::Glob(path) => {
                        let attempted_access_path = glob_access_path(&path.segments);
                        if use_declaration.alias.is_some() {
                            return Err(CanonicalParsedImportError::Unsupported {
                                span: use_declaration.span,
                                reason: "glob imports cannot carry an alias",
                            });
                        }
                        let provider = match resolve_module_path(
                            graph,
                            collection,
                            module_key,
                            &path.segments,
                            use_declaration.span,
                        ) {
                            Ok(provider) => provider,
                            Err(CanonicalParsedImportError::Unresolved { .. }) => continue,
                            Err(error) => return Err(error),
                        };
                        if let Some(view) = collection.provisional_name_view(&provider) {
                            for entry in view.entries().filter(|entry| {
                                entry.identity().canonical_parent().is_none()
                                    && entry.namespace() != CanonicalNamespace::Notation
                            }) {
                                let owner = entry_visibility_owner(entry);
                                if !entry.is_exportable()
                                    && !visible_from(
                                        entry.visibility(),
                                        &owner,
                                        module_key,
                                        graph.root_key(),
                                    )
                                {
                                    continue;
                                }
                                let candidate = make_candidate(
                                    graph,
                                    module_key,
                                    use_declaration,
                                    &attempted_access_path,
                                    None,
                                    entry,
                                    entry.lookup_name().into(),
                                )?;
                                changed |= stage_reexport(&mut staged, module_key, candidate)?;
                            }
                        }
                        let provider_exports = staged.get(&provider).cloned().unwrap_or_default();
                        for binding in provider_exports.values() {
                            if let Some(view) = collection.provisional_name_view(&provider)
                                && view.entries().any(|entry| {
                                    entry.identity().canonical_parent().is_none()
                                        && entry.lookup_name() == binding.local_name()
                                })
                            {
                                continue;
                            }
                            let candidate = make_reexport_candidate(
                                graph,
                                module_key,
                                use_declaration,
                                &attempted_access_path,
                                None,
                                binding.local_name().into(),
                                &provider,
                                binding,
                            )?;
                            changed |= stage_reexport(&mut staged, module_key, candidate)?;
                        }
                    }
                }
            }
        }
        if !changed {
            if let Some(modules) = find_public_reexport_cycle(graph, collection) {
                return Err(CanonicalParsedImportError::ImportCycle {
                    cycle: CanonicalParsedImportCycle {
                        edges: Box::new([]),
                        modules,
                    },
                });
            }
            break;
        }
    }
    Ok(staged)
}

fn find_public_reexport_cycle(
    graph: &CanonicalModuleGraph,
    collection: &CanonicalModuleCollection,
) -> Option<Box<[ModuleKey]>> {
    let mut adjacency = BTreeMap::<ModuleKey, Vec<ModuleKey>>::new();
    for (module_key, unit) in graph.module_units() {
        for item in unit.body().items() {
            let ModuleItem::Use(use_declaration) = item else {
                continue;
            };
            if matches!(use_declaration.visibility, Visibility::Inherited) {
                continue;
            }
            let path = match &use_declaration.path {
                UsePath::Notation { module, .. }
                | UsePath::Glob(module)
                | UsePath::Nested(module, _) => &module.segments,
                UsePath::Simple(path) if path.segments.len() > 1 => {
                    &path.segments[..path.segments.len() - 1]
                }
                UsePath::Simple(_) => continue,
            };
            let Ok(provider) =
                resolve_module_path(graph, collection, module_key, path, use_declaration.span)
            else {
                continue;
            };
            if provider != *module_key {
                adjacency
                    .entry(module_key.clone())
                    .or_default()
                    .push(provider);
            }
        }
    }
    let mut states = BTreeMap::<ModuleKey, u8>::new();
    let mut stack = Vec::new();
    for node in adjacency.keys() {
        if states.get(node).copied().unwrap_or_default() != 0 {
            continue;
        }
        if let Some(cycle) = visit_module_cycle(node, &adjacency, &mut states, &mut stack) {
            return Some(cycle.into_boxed_slice());
        }
    }
    None
}

fn visit_module_cycle(
    node: &ModuleKey,
    adjacency: &BTreeMap<ModuleKey, Vec<ModuleKey>>,
    states: &mut BTreeMap<ModuleKey, u8>,
    stack: &mut Vec<ModuleKey>,
) -> Option<Vec<ModuleKey>> {
    states.insert(node.clone(), 1);
    stack.push(node.clone());
    for target in adjacency.get(node).into_iter().flatten() {
        match states.get(target).copied().unwrap_or_default() {
            0 => {
                if let Some(cycle) = visit_module_cycle(target, adjacency, states, stack) {
                    return Some(cycle);
                }
            }
            1 => {
                let start = stack.iter().position(|candidate| candidate == target)?;
                let mut cycle = stack[start..].to_vec();
                cycle.push(target.clone());
                return Some(cycle);
            }
            _ => {}
        }
    }
    stack.pop();
    states.insert(node.clone(), 2);
    None
}

#[allow(clippy::result_large_err)]
fn stage_reexport(
    staged: &mut StagedReexports,
    module: &ModuleKey,
    candidate: Candidate,
) -> Result<bool, CanonicalParsedImportError> {
    let exports = staged.entry(module.clone()).or_default();
    let name = candidate.binding.local_name.clone();
    if let Some(previous) = exports.get(&name) {
        if previous == &candidate.binding {
            return Ok(false);
        }
        return Err(CanonicalParsedImportError::DuplicateBinding {
            module: module.clone(),
            name,
            span: candidate.binding.use_span,
        });
    }
    exports.insert(name, candidate.binding);
    Ok(true)
}

#[allow(clippy::result_large_err, clippy::too_many_arguments)]
fn resolve_named_candidate(
    graph: &CanonicalModuleGraph,
    collection: &CanonicalModuleCollection,
    importing: &ModuleKey,
    use_declaration: &Use,
    path: &[Box<str>],
    alias: Option<Box<str>>,
    member_span: Option<Span>,
    reexports: &StagedReexports,
) -> Result<Candidate, CanonicalParsedImportError> {
    let Some((name, prefix)) = path.split_last() else {
        return Err(CanonicalParsedImportError::Unsupported {
            span: use_declaration.span,
            reason: "an import path cannot be empty",
        });
    };
    let mut matches = Vec::<NamedTarget>::new();
    let mut visibility_error = None;
    for module_prefix_len in (0..=prefix.len()).rev() {
        let (module_prefix, parent_names) = prefix.split_at(module_prefix_len);
        let provider = if module_prefix.is_empty() {
            if prefix
                .first()
                .is_some_and(|segment| matches!(segment.as_ref(), "crate" | "self" | "super"))
            {
                continue;
            }
            importing.clone()
        } else {
            match resolve_module_path(
                graph,
                collection,
                importing,
                module_prefix,
                use_declaration.span,
            ) {
                Ok(provider) => provider,
                Err(error @ CanonicalParsedImportError::Inaccessible { .. }) => {
                    visibility_error = Some(with_attempted_path(error, path));
                    continue;
                }
                Err(CanonicalParsedImportError::Unresolved { .. }) => continue,
                Err(error) => return Err(error),
            }
        };
        let view = collection
            .provisional_name_view(&provider)
            .ok_or(CanonicalParsedImportError::GraphMismatch)?;
        matches.extend(
            view.entries()
                .filter(|entry| {
                    entry.lookup_name() == name.as_ref()
                        && parent_chain_matches(entry, view, parent_names)
                })
                .cloned()
                .map(|entry| NamedTarget::Declaration { entry }),
        );
        if parent_names.is_empty()
            && let Some(provider_reexports) = reexports.get(&provider)
            && let Some(binding) = provider_reexports.get(name.as_ref())
        {
            if visible_from(
                binding.import_visibility(),
                &provider,
                importing,
                graph.root_key(),
            ) {
                matches.push(NamedTarget::Reexport {
                    provider: provider.clone(),
                    binding: Box::new(binding.clone()),
                });
            } else {
                visibility_error = Some(CanonicalParsedImportError::Inaccessible {
                    module: importing.clone(),
                    defining_module: binding.defining_identity().module_key().clone(),
                    path: path.to_vec(),
                    declaration_span: binding.declaration_span(),
                    violated_visibility: binding.import_visibility().clone(),
                    span: use_declaration.span,
                });
            }
        }
    }
    if matches.is_empty() {
        if let Some(error) = visibility_error {
            return Err(error);
        }
        return Err(CanonicalParsedImportError::Unresolved {
            module: importing.clone(),
            path: path.to_vec(),
            span: use_declaration.span,
        });
    }
    if matches.len() > 1 {
        return Err(CanonicalParsedImportError::Ambiguous {
            module: importing.clone(),
            path: path.to_vec(),
            candidates: matches
                .into_iter()
                .map(|target| target.identity().clone())
                .collect(),
            span: use_declaration.span,
        });
    }
    let local = alias.unwrap_or_else(|| name.clone());
    match matches
        .into_iter()
        .next()
        .ok_or(CanonicalParsedImportError::GraphMismatch)?
    {
        NamedTarget::Declaration { entry, .. } => {
            check_parent_chain_visibility(
                graph,
                collection,
                importing,
                &entry,
                path,
                use_declaration.span,
            )?;
            make_candidate(
                graph,
                importing,
                use_declaration,
                path,
                member_span,
                &entry,
                local,
            )
        }
        NamedTarget::Reexport { provider, binding } => make_reexport_candidate(
            graph,
            importing,
            use_declaration,
            path,
            member_span,
            local,
            &provider,
            &binding,
        ),
    }
}

fn with_attempted_path(
    error: CanonicalParsedImportError,
    path: &[Box<str>],
) -> CanonicalParsedImportError {
    match error {
        CanonicalParsedImportError::Inaccessible {
            module,
            defining_module,
            declaration_span,
            violated_visibility,
            span,
            ..
        } => CanonicalParsedImportError::Inaccessible {
            module,
            defining_module,
            path: path.to_vec(),
            declaration_span,
            violated_visibility,
            span,
        },
        other => other,
    }
}

#[allow(clippy::result_large_err)]
fn check_parent_chain_visibility(
    graph: &CanonicalModuleGraph,
    collection: &CanonicalModuleCollection,
    requesting: &ModuleKey,
    entry: &CanonicalProvisionalNameEntry,
    path: &[Box<str>],
    span: Span,
) -> Result<(), CanonicalParsedImportError> {
    let Some(view) = collection.provisional_name_view(entry.identity().module_key()) else {
        return Err(CanonicalParsedImportError::GraphMismatch);
    };
    let mut parent = entry.identity().canonical_parent();
    while let Some(parent_identity) = parent {
        let parent_entry = view
            .entries()
            .find(|candidate| candidate.identity() == parent_identity)
            .ok_or(CanonicalParsedImportError::GraphMismatch)?;
        check_visibility(graph, parent_entry, requesting, path, span)?;
        parent = parent_entry.identity().canonical_parent();
    }
    Ok(())
}

fn parent_chain_matches(
    entry: &CanonicalProvisionalNameEntry,
    view: &CanonicalProvisionalNameView,
    parent_names: &[Box<str>],
) -> bool {
    let mut parent = entry.identity().canonical_parent();
    for name in parent_names.iter().rev() {
        let Some(parent_identity) = parent else {
            return false;
        };
        let Some(parent_entry) = view.entries().find(|candidate| {
            candidate.lookup_name() == name.as_ref() && candidate.identity() == parent_identity
        }) else {
            return false;
        };
        parent = parent_entry.identity().canonical_parent();
    }
    parent.is_none()
}

fn glob_access_path(path: &[Box<str>]) -> Box<[Box<str>]> {
    path.iter()
        .cloned()
        .chain(std::iter::once("*".into()))
        .collect()
}

#[allow(clippy::result_large_err)]
fn make_candidate(
    graph: &CanonicalModuleGraph,
    importing: &ModuleKey,
    use_declaration: &Use,
    path: &[Box<str>],
    member_span: Option<Span>,
    entry: &CanonicalProvisionalNameEntry,
    local_name: Box<str>,
) -> Result<Candidate, CanonicalParsedImportError> {
    validate_import_visibility(
        graph,
        importing,
        &use_declaration.visibility,
        use_declaration.span,
    )?;
    check_visibility(graph, entry, importing, path, use_declaration.span)?;
    let origin = graph
        .module_unit(entry.identity().module_key())
        .ok_or(CanonicalParsedImportError::GraphMismatch)?
        .artifact()
        .origin()
        .clone();
    let binding = CanonicalParsedImportBinding {
        local_name,
        defining_identity: entry.identity().clone(),
        lookup_key: entry.lookup_key().clone(),
        declaration_span: entry.origin_anchor(),
        use_span: use_declaration.span,
        member_span,
        attempted_access_path: path.to_vec().into_boxed_slice(),
        origin,
        declaration_visibility: entry.visibility().clone(),
        import_visibility: use_declaration.visibility.clone(),
        externally_public_reexport: matches!(use_declaration.visibility, Visibility::Public),
        source_ordinal: entry.source_ordinal(),
        reexport: !matches!(use_declaration.visibility, Visibility::Inherited),
    };
    let edge = (importing != entry.identity().module_key()).then(|| CanonicalParsedImportEdge {
        importing_module: importing.clone(),
        defining_module: entry.identity().module_key().clone(),
        binding: binding.clone(),
    });
    Ok(Candidate { binding, edge })
}

#[allow(clippy::result_large_err, clippy::too_many_arguments)]
fn make_reexport_candidate(
    graph: &CanonicalModuleGraph,
    importing: &ModuleKey,
    use_declaration: &Use,
    path: &[Box<str>],
    member_span: Option<Span>,
    local_name: Box<str>,
    published_by: &ModuleKey,
    original: &CanonicalParsedImportBinding,
) -> Result<Candidate, CanonicalParsedImportError> {
    validate_import_visibility(
        graph,
        importing,
        &use_declaration.visibility,
        use_declaration.span,
    )?;
    if !visible_from(
        original.import_visibility(),
        published_by,
        importing,
        graph.root_key(),
    ) {
        return Err(CanonicalParsedImportError::Inaccessible {
            module: importing.clone(),
            defining_module: original.defining_identity().module_key().clone(),
            path: path.to_vec(),
            declaration_span: original.declaration_span(),
            violated_visibility: original.import_visibility().clone(),
            span: use_declaration.span,
        });
    }
    let mut binding = original.clone();
    binding.local_name = local_name;
    binding.use_span = use_declaration.span;
    binding.member_span = member_span;
    binding.attempted_access_path = path.to_vec().into_boxed_slice();
    binding.import_visibility = use_declaration.visibility.clone();
    binding.externally_public_reexport = original.is_externally_public_reexport()
        && matches!(use_declaration.visibility, Visibility::Public);
    binding.reexport = !matches!(use_declaration.visibility, Visibility::Inherited);
    let edge = (importing != original.defining_identity().module_key()).then(|| {
        CanonicalParsedImportEdge {
            importing_module: importing.clone(),
            defining_module: original.defining_identity().module_key().clone(),
            binding: binding.clone(),
        }
    });
    Ok(Candidate { binding, edge })
}

#[allow(clippy::result_large_err)]
fn validate_import_visibility(
    graph: &CanonicalModuleGraph,
    importing: &ModuleKey,
    visibility: &Visibility,
    span: Span,
) -> Result<(), CanonicalParsedImportError> {
    if matches!(visibility, Visibility::Inherited) {
        return Ok(());
    }
    if graph
        .module_units()
        .any(|(requesting, _)| visible_from(visibility, importing, requesting, graph.root_key()))
    {
        return Ok(());
    }
    Err(CanonicalParsedImportError::Unsupported {
        span,
        reason: "import visibility does not name a reachable module region",
    })
}

#[allow(clippy::result_large_err)]
fn check_visibility(
    graph: &CanonicalModuleGraph,
    entry: &CanonicalProvisionalNameEntry,
    requesting: &ModuleKey,
    path: &[Box<str>],
    span: Span,
) -> Result<(), CanonicalParsedImportError> {
    let visibility_owner = entry_visibility_owner(entry);
    if visible_from(
        entry.visibility(),
        &visibility_owner,
        requesting,
        graph.root_key(),
    ) {
        return Ok(());
    }
    Err(CanonicalParsedImportError::Inaccessible {
        module: requesting.clone(),
        defining_module: entry.identity().module_key().clone(),
        path: path.to_vec(),
        declaration_span: entry.origin_anchor(),
        violated_visibility: entry.visibility().clone(),
        span,
    })
}

fn entry_visibility_owner(entry: &CanonicalProvisionalNameEntry) -> ModuleKey {
    if entry.namespace() == CanonicalNamespace::StructuralModule {
        entry
            .identity()
            .module_key()
            .parent()
            .unwrap_or_else(|| entry.identity().module_key().clone())
    } else {
        entry.identity().module_key().clone()
    }
}

#[allow(clippy::result_large_err)]
fn resolve_module_path(
    graph: &CanonicalModuleGraph,
    collection: &CanonicalModuleCollection,
    requesting: &ModuleKey,
    path: &[Box<str>],
    span: Span,
) -> Result<ModuleKey, CanonicalParsedImportError> {
    let Some(first) = path.first() else {
        return Err(CanonicalParsedImportError::Unsupported {
            span,
            reason: "a module path cannot be empty",
        });
    };
    match first.as_ref() {
        "crate" => resolve_module_children(
            graph,
            collection,
            requesting,
            graph.root_key().clone(),
            &path[1..],
            path,
            span,
        ),
        "self" => resolve_module_children(
            graph,
            collection,
            requesting,
            requesting.clone(),
            &path[1..],
            path,
            span,
        ),
        "super" => {
            let mut base = requesting.clone();
            let mut index = 0;
            while path
                .get(index)
                .is_some_and(|segment| segment.as_ref() == "super")
            {
                base = base
                    .parent()
                    .ok_or_else(|| CanonicalParsedImportError::Unresolved {
                        module: requesting.clone(),
                        path: path.to_vec(),
                        span,
                    })?;
                index += 1;
            }
            resolve_module_children(
                graph,
                collection,
                requesting,
                base,
                &path[index..],
                path,
                span,
            )
        }
        _ => {
            let mut base = Some(requesting.clone());
            while let Some(candidate_base) = base {
                match resolve_module_children(
                    graph,
                    collection,
                    requesting,
                    candidate_base.clone(),
                    path,
                    path,
                    span,
                ) {
                    Ok(module) => return Ok(module),
                    Err(error @ CanonicalParsedImportError::Inaccessible { .. }) => {
                        return Err(error);
                    }
                    Err(CanonicalParsedImportError::Unresolved { .. }) => {}
                    Err(error) => return Err(error),
                }
                base = candidate_base.parent();
            }
            Err(CanonicalParsedImportError::Unresolved {
                module: requesting.clone(),
                path: path.to_vec(),
                span,
            })
        }
    }
}

#[allow(clippy::result_large_err)]
fn resolve_module_children(
    graph: &CanonicalModuleGraph,
    collection: &CanonicalModuleCollection,
    requesting: &ModuleKey,
    mut current: ModuleKey,
    segments: &[Box<str>],
    full_path: &[Box<str>],
    span: Span,
) -> Result<ModuleKey, CanonicalParsedImportError> {
    for segment in segments {
        let view = collection
            .provisional_name_view(&current)
            .ok_or(CanonicalParsedImportError::GraphMismatch)?;
        let entries = view
            .entries()
            .filter(|entry| {
                entry.namespace() == CanonicalNamespace::StructuralModule
                    && entry.lookup_name() == segment.as_ref()
            })
            .cloned()
            .collect::<Vec<_>>();
        if entries.is_empty() {
            return Err(CanonicalParsedImportError::Unresolved {
                module: requesting.clone(),
                path: full_path.to_vec(),
                span,
            });
        }
        if entries.len() > 1 {
            return Err(CanonicalParsedImportError::Ambiguous {
                module: requesting.clone(),
                path: full_path.to_vec(),
                candidates: entries
                    .into_iter()
                    .map(|entry| entry.identity().clone())
                    .collect(),
                span,
            });
        }
        let entry = &entries[0];
        check_visibility(graph, entry, requesting, full_path, span)?;
        // A structural module declaration is recorded in its parent module;
        // traversal advances to the canonical child key named by that
        // declaration rather than reusing the parent-side declaration
        // identity.
        current = current
            .child(segment)
            .map_err(|_| CanonicalParsedImportError::Unresolved {
                module: requesting.clone(),
                path: full_path.to_vec(),
                span,
            })?;
    }
    if !graph_keys_contains(graph, &current) {
        return Err(CanonicalParsedImportError::Unresolved {
            module: requesting.clone(),
            path: full_path.to_vec(),
            span,
        });
    }
    Ok(current)
}

fn graph_keys_contains(graph: &CanonicalModuleGraph, key: &ModuleKey) -> bool {
    graph.module_unit(key).is_some()
}

fn visible_from(
    visibility: &Visibility,
    defining: &ModuleKey,
    requesting: &ModuleKey,
    root: &ModuleKey,
) -> bool {
    match visibility {
        Visibility::Public => same_crate(root, defining, requesting),
        Visibility::Inherited | Visibility::Self_ => defining == requesting,
        Visibility::Crate => same_crate(root, defining, requesting),
        Visibility::Super { levels } => {
            ancestor(defining, *levels).is_some_and(|region| descendant(&region, requesting))
        }
        Visibility::Restricted { path } => restricted_region(root, path)
            .is_some_and(|region| descendant(&region, defining) && descendant(&region, requesting)),
    }
}

fn same_crate(root: &ModuleKey, left: &ModuleKey, right: &ModuleKey) -> bool {
    crate_root(root) == crate_root(left) && crate_root(left) == crate_root(right)
}
fn crate_root(key: &ModuleKey) -> ModuleKey {
    let mut current = key.clone();
    while let Some(parent) = current.parent() {
        current = parent;
    }
    current
}
fn descendant(region: &ModuleKey, candidate: &ModuleKey) -> bool {
    crate_root(region) == crate_root(candidate)
        && candidate.segments().starts_with(region.segments())
}
fn ancestor(key: &ModuleKey, levels: usize) -> Option<ModuleKey> {
    let mut current = key.clone();
    for _ in 0..levels {
        current = current.parent()?;
    }
    Some(current)
}
fn restricted_region(root: &ModuleKey, path: &str) -> Option<ModuleKey> {
    let mut segments = path.split("::");
    if segments.next()? != "crate" {
        return None;
    }
    let mut region = root.clone();
    for segment in segments {
        region = region.child(segment).ok()?;
    }
    Some(region)
}

fn find_cycle(edges: &[CanonicalParsedImportEdge]) -> Option<CanonicalParsedImportCycle> {
    let mut adjacency = BTreeMap::<ModuleKey, Vec<usize>>::new();
    for (index, edge) in edges.iter().enumerate() {
        adjacency
            .entry(edge.importing_module.clone())
            .or_default()
            .push(index);
    }
    let mut states = BTreeMap::new();
    let mut nodes = Vec::new();
    let mut path = Vec::new();
    for node in adjacency.keys() {
        if states.get(node).is_some_and(|state| *state != 0_u8) {
            continue;
        }
        if let Some(cycle) =
            visit_cycle(node, edges, &adjacency, &mut states, &mut nodes, &mut path)
        {
            return Some(cycle);
        }
    }
    None
}

fn visit_cycle(
    node: &ModuleKey,
    edges: &[CanonicalParsedImportEdge],
    adjacency: &BTreeMap<ModuleKey, Vec<usize>>,
    states: &mut BTreeMap<ModuleKey, u8>,
    nodes: &mut Vec<ModuleKey>,
    path: &mut Vec<usize>,
) -> Option<CanonicalParsedImportCycle> {
    states.insert(node.clone(), 1);
    nodes.push(node.clone());
    for edge_index in adjacency.get(node).into_iter().flatten() {
        let edge = &edges[*edge_index];
        let target = &edge.defining_module;
        match states.get(target).copied().unwrap_or(0) {
            0 => {
                path.push(*edge_index);
                if let Some(cycle) = visit_cycle(target, edges, adjacency, states, nodes, path) {
                    return Some(cycle);
                }
                path.pop();
            }
            1 => {
                let start = nodes.iter().position(|candidate| candidate == target)?;
                let mut cycle = path[start..]
                    .iter()
                    .map(|index| edges[*index].clone())
                    .collect::<Vec<_>>();
                cycle.push(edge.clone());
                let mut modules = vec![
                    cycle
                        .first()
                        .expect("a cycle has at least one edge")
                        .importing_module
                        .clone(),
                ];
                modules.extend(cycle.iter().map(|edge| edge.defining_module.clone()));
                return Some(CanonicalParsedImportCycle {
                    edges: cycle.into_boxed_slice(),
                    modules: modules.into_boxed_slice(),
                });
            }
            _ => {}
        }
    }
    nodes.pop();
    states.insert(node.clone(), 2);
    None
}
