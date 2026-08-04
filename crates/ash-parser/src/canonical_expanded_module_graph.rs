//! Canonical parser-stage expansion over an acquired module graph.

use std::collections::BTreeMap;
use std::fmt;

use ash_core::module_graph::{ModuleArtifactOrigin, ModuleKey};
use thiserror::Error;

use crate::canonical_module_graph::CanonicalModuleGraph;
use crate::canonical_syntax_dependencies::{
    CanonicalNotationFixityKey, CanonicalNotationImport, CanonicalNotationImportFailure,
    CanonicalSyntaxDependencyCycle, CanonicalSyntaxImport, CanonicalSyntaxImportFailure,
    CanonicalSyntaxPrepassError, CanonicalSyntaxProviderFailure,
    prepare_canonical_syntax_dependencies,
};
use crate::module::ModuleBody;
use crate::surface::{
    Definition, ExpandedSurfaceOrigin, ExpansionDiagnostic, ExpansionError,
    IdentifierHygieneMetadata, ImportedNotationEntry, LocalMacroEntry, NotationFixity,
    ShallowModuleBodyExpansionError, alias_imported_macro_entry, expand_module_body_shallow,
    imported_macro_entry_for_definitions,
};
use crate::token::Span;

/// Failure while atomically expanding a canonical parsed module graph.
#[derive(Debug, Clone, PartialEq)]
pub enum CanonicalModuleExpansionError {
    /// One canonical module failed its local, shallow syntax expansion.
    Expansion {
        /// Boxed diagnostic facts keep successful `Result` values compact.
        failure: Box<CanonicalModuleExpansionFailure>,
    },
    /// Shallow expansion violated the direct-definition cardinality invariant.
    BodyInvariant {
        /// Boxed anchored invariant facts.
        failure: Box<CanonicalModuleExpansionInvariantFailure>,
    },
    /// One invocation-backed syntax import was not an importable macro summary.
    InvalidSyntaxImport {
        /// Anchored syntax-only import rejection.
        failure: Box<CanonicalSyntaxImportFailure>,
    },
    /// One notation import was private, missing, unsupported, or conflicting.
    InvalidNotationImport {
        /// Complete anchored notation-import rejection.
        failure: Box<CanonicalNotationImportFailure>,
    },
    /// A provider's public macro template could not be closed or validated.
    InvalidSyntaxProvider {
        /// Provider-owned anchored syntax failure.
        failure: Box<CanonicalSyntaxProviderFailure>,
    },
    /// Syntax-only module dependencies contain a cycle.
    SyntaxDependencyCycle {
        /// Stable ordered importer-to-provider cycle edges.
        cycle: Box<CanonicalSyntaxDependencyCycle>,
    },
    /// The staged expanded key set did not exactly match the parsed graph.
    KeySetInvariant {
        /// Canonical parsed keys that were expected.
        parsed_keys: Box<[ModuleKey]>,
        /// Canonical expanded keys that were staged.
        expanded_keys: Box<[ModuleKey]>,
    },
}

impl fmt::Display for CanonicalModuleExpansionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Expansion { failure } => failure.fmt(formatter),
            Self::BodyInvariant { failure } => failure.fmt(formatter),
            Self::InvalidSyntaxImport { failure } => failure.fmt(formatter),
            Self::InvalidNotationImport { failure } => failure.fmt(formatter),
            Self::InvalidSyntaxProvider { failure } => failure.fmt(formatter),
            Self::SyntaxDependencyCycle { cycle } => cycle.fmt(formatter),
            Self::KeySetInvariant { .. } => {
                formatter.write_str("canonical expanded module graph key-set invariant failed")
            }
        }
    }
}

impl std::error::Error for CanonicalModuleExpansionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Expansion { failure } => Some(failure.as_ref()),
            Self::BodyInvariant { failure } => Some(failure.as_ref()),
            Self::InvalidSyntaxImport { failure } => Some(failure.as_ref()),
            Self::InvalidNotationImport { failure } => Some(failure.as_ref()),
            Self::InvalidSyntaxProvider { failure } => Some(failure.as_ref()),
            Self::SyntaxDependencyCycle { cycle } => Some(cycle.as_ref()),
            Self::KeySetInvariant { .. } => None,
        }
    }
}

impl CanonicalModuleExpansionError {
    /// Returns the anchored local-expansion failure, when this is an expansion error.
    #[must_use]
    pub fn expansion_failure(&self) -> Option<&CanonicalModuleExpansionFailure> {
        match self {
            Self::Expansion { failure } => Some(failure),
            Self::BodyInvariant { .. }
            | Self::InvalidSyntaxImport { .. }
            | Self::InvalidNotationImport { .. }
            | Self::InvalidSyntaxProvider { .. }
            | Self::SyntaxDependencyCycle { .. }
            | Self::KeySetInvariant { .. } => None,
        }
    }

    /// Returns the anchored body-invariant failure, when cardinality validation failed.
    #[must_use]
    pub fn body_invariant_failure(&self) -> Option<&CanonicalModuleExpansionInvariantFailure> {
        match self {
            Self::BodyInvariant { failure } => Some(failure),
            Self::Expansion { .. }
            | Self::InvalidSyntaxImport { .. }
            | Self::InvalidNotationImport { .. }
            | Self::InvalidSyntaxProvider { .. }
            | Self::SyntaxDependencyCycle { .. }
            | Self::KeySetInvariant { .. } => None,
        }
    }

    /// Returns an anchored invalid syntax import, when present.
    #[must_use]
    pub fn syntax_import_failure(&self) -> Option<&CanonicalSyntaxImportFailure> {
        match self {
            Self::InvalidSyntaxImport { failure } => Some(failure),
            Self::Expansion { .. }
            | Self::BodyInvariant { .. }
            | Self::InvalidNotationImport { .. }
            | Self::InvalidSyntaxProvider { .. }
            | Self::SyntaxDependencyCycle { .. }
            | Self::KeySetInvariant { .. } => None,
        }
    }

    /// Returns an anchored invalid notation import, when present.
    #[must_use]
    pub fn notation_import_failure(&self) -> Option<&CanonicalNotationImportFailure> {
        match self {
            Self::InvalidNotationImport { failure } => Some(failure),
            Self::Expansion { .. }
            | Self::BodyInvariant { .. }
            | Self::InvalidSyntaxImport { .. }
            | Self::InvalidSyntaxProvider { .. }
            | Self::SyntaxDependencyCycle { .. }
            | Self::KeySetInvariant { .. } => None,
        }
    }

    /// Returns an anchored invalid provider template, when present.
    #[must_use]
    pub fn syntax_provider_failure(&self) -> Option<&CanonicalSyntaxProviderFailure> {
        match self {
            Self::InvalidSyntaxProvider { failure } => Some(failure),
            Self::Expansion { .. }
            | Self::BodyInvariant { .. }
            | Self::InvalidSyntaxImport { .. }
            | Self::InvalidNotationImport { .. }
            | Self::SyntaxDependencyCycle { .. }
            | Self::KeySetInvariant { .. } => None,
        }
    }

    /// Returns stable cycle edges when syntax dependencies are cyclic.
    #[must_use]
    pub fn syntax_dependency_cycle(&self) -> Option<&CanonicalSyntaxDependencyCycle> {
        match self {
            Self::SyntaxDependencyCycle { cycle } => Some(cycle),
            Self::Expansion { .. }
            | Self::BodyInvariant { .. }
            | Self::InvalidSyntaxImport { .. }
            | Self::InvalidNotationImport { .. }
            | Self::InvalidSyntaxProvider { .. }
            | Self::KeySetInvariant { .. } => None,
        }
    }
}

/// Anchored facts retained when one canonical module fails syntax expansion.
#[derive(Debug, Clone, PartialEq, Error)]
#[error("failed to expand canonical module `{module_key}` at {span:?}: {source}")]
pub struct CanonicalModuleExpansionFailure {
    module_key: ModuleKey,
    source_path: Option<Box<str>>,
    artifact_origin: ModuleArtifactOrigin,
    span: Span,
    #[source]
    source: ExpansionError,
}

impl CanonicalModuleExpansionFailure {
    /// Returns the canonical identity of the module that failed.
    #[must_use]
    pub fn module_key(&self) -> &ModuleKey {
        &self.module_key
    }

    /// Returns the enclosing source path, when the parsed unit retained one.
    #[must_use]
    pub fn source_path(&self) -> Option<&str> {
        self.source_path.as_deref()
    }

    /// Returns the durable file or inline artifact origin.
    #[must_use]
    pub fn artifact_origin(&self) -> &ModuleArtifactOrigin {
        &self.artifact_origin
    }

    /// Returns the exact source anchor reported by surface expansion.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the underlying surface-expansion failure.
    #[must_use]
    pub fn expansion_error(&self) -> &ExpansionError {
        &self.source
    }
}

/// Anchored facts retained when shallow body reconstruction violates an invariant.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error(
    "canonical module `{module_key}` at {span:?} retained {actual_definitions} expanded definitions, expected {expected_definitions}"
)]
pub struct CanonicalModuleExpansionInvariantFailure {
    module_key: ModuleKey,
    source_path: Option<Box<str>>,
    artifact_origin: ModuleArtifactOrigin,
    span: Span,
    expected_definitions: usize,
    actual_definitions: usize,
}

impl CanonicalModuleExpansionInvariantFailure {
    /// Returns the canonical identity of the affected module.
    #[must_use]
    pub fn module_key(&self) -> &ModuleKey {
        &self.module_key
    }

    /// Returns the enclosing source path, when available.
    #[must_use]
    pub fn source_path(&self) -> Option<&str> {
        self.source_path.as_deref()
    }

    /// Returns the durable file or inline artifact origin.
    #[must_use]
    pub fn artifact_origin(&self) -> &ModuleArtifactOrigin {
        &self.artifact_origin
    }

    /// Returns the complete parsed body span anchoring the invariant failure.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the parsed body's direct-definition count.
    #[must_use]
    pub const fn expected_definitions(&self) -> usize {
        self.expected_definitions
    }

    /// Returns the expanded direct-definition count.
    #[must_use]
    pub const fn actual_definitions(&self) -> usize {
        self.actual_definitions
    }
}

/// Read-only expanded data owned for one canonical module key.
#[derive(Debug)]
struct CanonicalExpandedModule {
    body: ModuleBody,
    diagnostics: Box<[ExpansionDiagnostic]>,
    origins: Box<[ExpandedSurfaceOrigin]>,
    hygiene: Box<[IdentifierHygieneMetadata]>,
    syntax_imports: Box<[CanonicalSyntaxImport]>,
    notation_imports: Box<[CanonicalNotationImport]>,
}

/// Borrowed view of one module record in a [`CanonicalExpandedModuleGraph`].
#[derive(Debug, Clone, Copy)]
pub struct CanonicalExpandedModuleRef<'a> {
    key: &'a ModuleKey,
    record: &'a CanonicalExpandedModule,
}

impl CanonicalExpandedModuleRef<'_> {
    /// Returns this expanded module's canonical identity.
    #[must_use]
    pub fn key(&self) -> &ModuleKey {
        self.key
    }

    /// Returns the complete source-ordered body after shallow expansion.
    #[must_use]
    pub fn body(&self) -> &ModuleBody {
        &self.record.body
    }

    /// Returns diagnostics retained for this module only.
    #[must_use]
    pub fn diagnostics(&self) -> &[ExpansionDiagnostic] {
        &self.record.diagnostics
    }

    /// Returns generated-node origins retained for this module only.
    #[must_use]
    pub fn origins(&self) -> &[ExpandedSurfaceOrigin] {
        &self.record.origins
    }

    /// Returns identifier hygiene metadata retained for this module only.
    #[must_use]
    pub fn hygiene(&self) -> &[IdentifierHygieneMetadata] {
        &self.record.hygiene
    }

    /// Returns syntax-only imports authorized for this module.
    #[must_use]
    pub fn syntax_imports(&self) -> &[CanonicalSyntaxImport] {
        &self.record.syntax_imports
    }

    /// Returns public notation summaries transported to this module.
    ///
    /// These summaries are not active in local expression parsing yet.
    #[must_use]
    pub fn notation_imports(&self) -> &[CanonicalNotationImport] {
        &self.record.notation_imports
    }
}

/// Atomic, parser-owned shallow expansion of a canonical parsed module graph.
///
/// Construction consumes the parsed graph and publishes no value unless every
/// parsed key has exactly one successfully expanded record.
///
/// This slice resolves bounded AST-only public macro imports and activates
/// prepass-validated public notation summaries only in their importing consumer.
/// Notation dependencies reject atomically with typed provenance, and activation
/// grants no ordinary binding, callable authority, or runtime/admission authority.
/// The remaining SPEC-103 evidence is not installed yet, so callers must not treat
/// this value as the complete expanded-graph handoff.
#[derive(Debug)]
pub struct CanonicalExpandedModuleGraph {
    parsed: CanonicalModuleGraph,
    modules: BTreeMap<ModuleKey, CanonicalExpandedModule>,
}

impl CanonicalExpandedModuleGraph {
    /// Shallowly expands every canonical parsed module.
    ///
    /// Module-local syntax and invocation-backed public macro imports in simple
    /// canonical `crate::...::name [as alias]` form participate. Canonical
    /// prepass-validated notation imports activate only in their consumer's syntax
    /// table and create no ordinary binding or callable/runtime authority.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalModuleExpansionError::Expansion`] when local syntax
    /// expansion fails, [`CanonicalModuleExpansionError::BodyInvariant`] when
    /// expansion changes the number of direct definitions owned by a module,
    /// [`CanonicalModuleExpansionError::InvalidSyntaxImport`] for duplicate
    /// consumer-local syntax names or an invoked private, non-macro, missing,
    /// or unsupported syntax import,
    /// [`CanonicalModuleExpansionError::InvalidSyntaxProvider`] when a public
    /// provider template cannot be closed or validated,
    /// [`CanonicalModuleExpansionError::InvalidNotationImport`] when an exact
    /// notation dependency is private, missing, unsupported, or conflicting,
    /// [`CanonicalModuleExpansionError::SyntaxDependencyCycle`] for cyclic
    /// syntax-only module dependencies, or
    /// [`CanonicalModuleExpansionError::KeySetInvariant`] when the staged result
    /// does not cover the parsed key set exactly.
    pub fn try_expand(parsed: CanonicalModuleGraph) -> Result<Self, CanonicalModuleExpansionError> {
        let prepass =
            prepare_canonical_syntax_dependencies(&parsed).map_err(|error| match error {
                CanonicalSyntaxPrepassError::InvalidSyntaxImport(failure) => {
                    CanonicalModuleExpansionError::InvalidSyntaxImport { failure }
                }
                CanonicalSyntaxPrepassError::InvalidNotationImport(failure) => {
                    CanonicalModuleExpansionError::InvalidNotationImport { failure }
                }
                CanonicalSyntaxPrepassError::SyntaxDependencyCycle(cycle) => {
                    CanonicalModuleExpansionError::SyntaxDependencyCycle { cycle }
                }
            })?;
        let mut modules = BTreeMap::new();
        let mut closed_exports = BTreeMap::<ModuleKey, BTreeMap<Box<str>, LocalMacroEntry>>::new();
        let expansion_order = prepass.order().to_vec();

        for key in expansion_order {
            let Some(unit) = parsed.module_unit(&key) else {
                return Err(key_set_invariant_error(&parsed, &modules));
            };
            let mut imported_macros = Vec::new();
            let mut syntax_imports = Vec::new();
            let notation_imports = prepass.notation_imports(&key).to_vec();
            let imported_notations = notation_imports
                .iter()
                .map(imported_notation_entry)
                .collect();
            for request in prepass.requests(&key) {
                let provenance = request.provenance();
                let Some(provider_exports) = closed_exports.get(provenance.provider_key()) else {
                    return Err(key_set_invariant_error(&parsed, &modules));
                };
                let Some(export) = provider_exports.get(provenance.exported_name()) else {
                    return Err(key_set_invariant_error(&parsed, &modules));
                };
                imported_macros.push(alias_imported_macro_entry(
                    export.clone(),
                    provenance.local_name().into(),
                ));
                syntax_imports.push(provenance.clone());
            }
            let requested_export_names = prepass
                .requested_exports(&key)
                .map(Box::<str>::from)
                .collect::<Vec<_>>();
            let expanded = expand_module_body_shallow(
                unit.body(),
                imported_macros,
                imported_notations,
                &requested_export_names,
            )
            .map_err(|error| match error {
                ShallowModuleBodyExpansionError::Expansion(source) => {
                    let error_span = expansion_error_span(&source);
                    if let Some(declaration_span) = requested_provider_declaration_span(
                        unit.body(),
                        &requested_export_names,
                        error_span,
                    ) {
                        CanonicalModuleExpansionError::InvalidSyntaxProvider {
                            failure: Box::new(CanonicalSyntaxProviderFailure::new(
                                key.clone(),
                                unit,
                                declaration_span,
                                source,
                            )),
                        }
                    } else {
                        anchored_expansion_error(&parsed, &key, source)
                    }
                }
                ShallowModuleBodyExpansionError::DefinitionCardinality {
                    body_span,
                    expected,
                    actual,
                } => CanonicalModuleExpansionError::BodyInvariant {
                    failure: Box::new(CanonicalModuleExpansionInvariantFailure {
                        module_key: key.clone(),
                        source_path: unit.source_path().map(Into::into),
                        artifact_origin: unit.artifact().origin().clone(),
                        span: body_span,
                        expected_definitions: expected,
                        actual_definitions: actual,
                    }),
                },
            })?;

            let mut exports = BTreeMap::new();
            for exported_name in &requested_export_names {
                let Some(declaration_span) = macro_declaration_span(&expanded.body, exported_name)
                else {
                    return Err(key_set_invariant_error(&parsed, &modules));
                };
                let export = imported_macro_entry_for_definitions(
                    expanded.body.definitions(),
                    exported_name,
                    exported_name.clone(),
                    key.to_string().into_boxed_str(),
                )
                .map_err(
                    |source| CanonicalModuleExpansionError::InvalidSyntaxProvider {
                        failure: Box::new(CanonicalSyntaxProviderFailure::new(
                            key.clone(),
                            unit,
                            declaration_span,
                            source,
                        )),
                    },
                )?
                .ok_or_else(|| key_set_invariant_error(&parsed, &modules))?;
                exports.insert(exported_name.clone(), export);
            }
            if !exports.is_empty() {
                closed_exports.insert(key.clone(), exports);
            }
            modules.insert(
                key,
                CanonicalExpandedModule {
                    body: expanded.body,
                    diagnostics: expanded.diagnostics.into_boxed_slice(),
                    origins: expanded.origins.into_boxed_slice(),
                    hygiene: expanded.hygiene.into_boxed_slice(),
                    syntax_imports: syntax_imports.into_boxed_slice(),
                    notation_imports: notation_imports.into_boxed_slice(),
                },
            );
        }

        let parsed_keys = parsed
            .module_units()
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        let expanded_keys = modules.keys().cloned().collect::<Vec<_>>();
        if parsed_keys != expanded_keys {
            return Err(key_set_invariant_error(&parsed, &modules));
        }

        Ok(Self { parsed, modules })
    }

    /// Returns the exact parsed graph consumed during construction.
    #[must_use]
    pub fn parsed_graph(&self) -> &CanonicalModuleGraph {
        &self.parsed
    }

    /// Returns one expanded module by canonical identity.
    #[must_use]
    pub fn module(&self, key: &ModuleKey) -> Option<CanonicalExpandedModuleRef<'_>> {
        self.modules
            .get_key_value(key)
            .map(|(key, record)| CanonicalExpandedModuleRef { key, record })
    }

    /// Iterates over every expanded module in canonical-key order.
    pub fn modules(&self) -> impl Iterator<Item = CanonicalExpandedModuleRef<'_>> {
        self.modules
            .iter()
            .map(|(key, record)| CanonicalExpandedModuleRef { key, record })
    }
}

fn imported_notation_entry(notation_import: &CanonicalNotationImport) -> ImportedNotationEntry {
    let summary = notation_import.summary();
    let fixity = match summary.key().fixity() {
        CanonicalNotationFixityKey::Prefix { precedence } => NotationFixity::Prefix {
            precedence: *precedence,
        },
        CanonicalNotationFixityKey::Infix {
            associativity,
            precedence,
        } => NotationFixity::Infix {
            associativity: *associativity,
            precedence: *precedence,
        },
        CanonicalNotationFixityKey::Suffix { precedence } => NotationFixity::Suffix {
            precedence: *precedence,
        },
        CanonicalNotationFixityKey::Mixfix => NotationFixity::Mixfix,
    };
    ImportedNotationEntry {
        pattern: summary.key().pattern().into(),
        fixity,
        target: summary.target().clone(),
        declaration_span: summary.declaration_span(),
    }
}

fn anchored_expansion_error(
    parsed: &CanonicalModuleGraph,
    key: &ModuleKey,
    source: ExpansionError,
) -> CanonicalModuleExpansionError {
    let Some(unit) = parsed.module_unit(key) else {
        return key_set_invariant_error(parsed, &BTreeMap::new());
    };
    CanonicalModuleExpansionError::Expansion {
        failure: Box::new(CanonicalModuleExpansionFailure {
            module_key: key.clone(),
            source_path: unit.source_path().map(Into::into),
            artifact_origin: unit.artifact().origin().clone(),
            span: expansion_error_span(&source),
            source,
        }),
    }
}

fn macro_declaration_span(body: &ModuleBody, name: &str) -> Option<Span> {
    body.definitions()
        .iter()
        .find_map(|definition| match definition {
            Definition::Macro(definition) if definition.name.as_ref() == name => {
                Some(definition.span)
            }
            _ => None,
        })
}

fn requested_provider_declaration_span(
    body: &ModuleBody,
    requested_names: &[Box<str>],
    error_span: Span,
) -> Option<Span> {
    body.definitions()
        .iter()
        .find_map(|definition| match definition {
            Definition::Macro(definition)
                if requested_names.iter().any(|name| name == &definition.name)
                    && definition.span.start <= error_span.start
                    && error_span.end <= definition.span.end =>
            {
                Some(definition.span)
            }
            _ => None,
        })
}

fn key_set_invariant_error(
    parsed: &CanonicalModuleGraph,
    modules: &BTreeMap<ModuleKey, CanonicalExpandedModule>,
) -> CanonicalModuleExpansionError {
    CanonicalModuleExpansionError::KeySetInvariant {
        parsed_keys: parsed
            .module_units()
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        expanded_keys: modules
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    }
}

fn expansion_error_span(error: &ExpansionError) -> Span {
    match error {
        ExpansionError::UnresolvedOperatorSection { span, .. }
        | ExpansionError::UnknownMacroInvocation { span, .. }
        | ExpansionError::UnsupportedMacroInvocation { span, .. }
        | ExpansionError::MacroTokenTreeReparseFailed { span, .. }
        | ExpansionError::MacroArityMismatch { span, .. }
        | ExpansionError::UnsupportedMacroTemplate { span, .. }
        | ExpansionError::MacroTypeMismatch { span, .. }
        | ExpansionError::MacroExpansionDepthExceeded { span, .. }
        | ExpansionError::DeferredMacroInvocation { span, .. } => *span,
        ExpansionError::DuplicateNotationDeclaration { second_span, .. }
        | ExpansionError::ConflictingNotationDeclaration { second_span, .. }
        | ExpansionError::DuplicateMacroDeclaration { second_span, .. } => *second_span,
    }
}
