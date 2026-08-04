//! Canonical parser-stage expansion over an acquired module graph.

use std::collections::BTreeMap;
use std::fmt;

use ash_core::module_graph::{ModuleArtifactOrigin, ModuleKey};
use thiserror::Error;

use crate::canonical_module_graph::CanonicalModuleGraph;
use crate::module::ModuleBody;
use crate::surface::{
    ExpandedSurfaceOrigin, ExpansionDiagnostic, ExpansionError, IdentifierHygieneMetadata,
    ShallowModuleBodyExpansionError, expand_module_body_shallow,
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
            Self::BodyInvariant { .. } | Self::KeySetInvariant { .. } => None,
        }
    }

    /// Returns the anchored body-invariant failure, when cardinality validation failed.
    #[must_use]
    pub fn body_invariant_failure(&self) -> Option<&CanonicalModuleExpansionInvariantFailure> {
        match self {
            Self::BodyInvariant { failure } => Some(failure),
            Self::Expansion { .. } | Self::KeySetInvariant { .. } => None,
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
}

/// Atomic, parser-owned shallow expansion of a canonical parsed module graph.
///
/// Construction consumes the parsed graph and publishes no value unless every
/// parsed key has exactly one successfully expanded record.
///
/// This initial slice resolves local syntax only. It does not perform the
/// AST-only syntax-import dependency prepass required by SPEC-103, so callers
/// must not treat this value as the complete expanded-graph handoff yet.
#[derive(Debug)]
pub struct CanonicalExpandedModuleGraph {
    parsed: CanonicalModuleGraph,
    modules: BTreeMap<ModuleKey, CanonicalExpandedModule>,
}

impl CanonicalExpandedModuleGraph {
    /// Shallowly expands every canonical parsed module.
    ///
    /// Only module-local macro and notation declarations participate in this
    /// slice. Imported syntax, dependency ordering, and syntax-cycle rejection
    /// remain unavailable until the canonical AST prepass is installed.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalModuleExpansionError::Expansion`] when local syntax
    /// expansion fails, [`CanonicalModuleExpansionError::BodyInvariant`] when
    /// expansion changes the number of direct definitions owned by a module,
    /// or [`CanonicalModuleExpansionError::KeySetInvariant`] when the staged
    /// result does not cover the parsed key set exactly.
    pub fn try_expand(parsed: CanonicalModuleGraph) -> Result<Self, CanonicalModuleExpansionError> {
        let mut modules = BTreeMap::new();

        for (key, unit) in parsed.module_units() {
            let expanded =
                expand_module_body_shallow(unit.body()).map_err(|error| match error {
                    ShallowModuleBodyExpansionError::Expansion(source) => {
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
            modules.insert(
                key.clone(),
                CanonicalExpandedModule {
                    body: expanded.body,
                    diagnostics: expanded.diagnostics.into_boxed_slice(),
                    origins: expanded.origins.into_boxed_slice(),
                    hygiene: expanded.hygiene.into_boxed_slice(),
                },
            );
        }

        let parsed_keys = parsed
            .module_units()
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        let expanded_keys = modules.keys().cloned().collect::<Vec<_>>();
        if parsed_keys != expanded_keys {
            return Err(CanonicalModuleExpansionError::KeySetInvariant {
                parsed_keys: parsed_keys.into_boxed_slice(),
                expanded_keys: expanded_keys.into_boxed_slice(),
            });
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
