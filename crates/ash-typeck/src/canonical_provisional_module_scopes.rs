//! Immutable Type-layer scopes derived from canonical parsed module units.
//!
//! The scope set deliberately retains only direct structural children and
//! ordinary function declarations. It is a bounded planning input, not a
//! completed namespace or interface.

use std::collections::BTreeMap;
use std::fmt;
use std::ops::Deref;

use ash_core::module_graph::{ModuleArtifact, ModuleArtifactOrigin, ModuleKey};
use ash_parser::surface::{Definition, Visibility};
use ash_parser::{CanonicalModuleGraph, Span};
use thiserror::Error;

use crate::canonical_simple_import_planner::CanonicalImportCycle;

/// A graph-derived, layout-independent projection of provisional scopes.
///
/// This opaque value supports representation-parity assertions without
/// exposing scope construction or widening the resolver's authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalNormalizedScopeProjection {
    modules: BTreeMap<ModuleKey, CanonicalNormalizedModuleScope>,
}

/// One normalized module entry in a [`CanonicalNormalizedScopeProjection`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalNormalizedModuleScope {
    children: BTreeMap<Box<str>, CanonicalNormalizedChild>,
    functions: BTreeMap<Box<str>, Visibility>,
}

/// One normalized direct-child fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalNormalizedChild {
    module_key: ModuleKey,
    visibility: Visibility,
}

/// An owned, compact diagnostic fact retained by structural import errors.
///
/// This wrapper keeps anchored public errors inexpensive to propagate while
/// preserving equality with the parser-owned value in diagnostics and tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalStructuralDiagnosticValue<T>(Box<T>);

impl<T> From<T> for CanonicalStructuralDiagnosticValue<T> {
    fn from(value: T) -> Self {
        Self(Box::new(value))
    }
}

impl<T> Deref for CanonicalStructuralDiagnosticValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> PartialEq<T> for CanonicalStructuralDiagnosticValue<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &T) -> bool {
        self.0.as_ref() == other
    }
}

impl<T> fmt::Display for CanonicalStructuralDiagnosticValue<T>
where
    T: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// One error while deriving or using provisional structural scopes.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum CanonicalStructuralImportError {
    /// The graph did not retain a structurally self-consistent parser fact.
    #[error("canonical provisional scope construction rejected graph facts: {reason}")]
    ScopeConstruction {
        /// Stable description of the rejected graph fact.
        reason: &'static str,
    },
    /// A scope snapshot was supplied with a different graph's parser facts.
    #[error("canonical provisional scope snapshot does not match the supplied graph")]
    ScopeGraphMismatch,
    /// Resolved cross-module structural aliases close a canonical dependency cycle.
    #[error("canonical structural import cycle")]
    ImportCycle {
        /// Ordered edges in the detected cycle, including its closing edge.
        edges: CanonicalImportCycle,
    },
    /// A selected structural path names no direct child or ordinary function.
    #[error("unresolved canonical structural import path {attempted_path:?}")]
    Unresolved {
        /// Parser anchor of the importing declaration.
        use_span: CanonicalStructuralDiagnosticValue<Span>,
        /// Parsed path segments, including the `crate` head.
        attempted_path: CanonicalStructuralDiagnosticValue<Vec<Box<str>>>,
    },
    /// A structural child or final function rejected the importing module.
    #[error("canonical structural import path {attempted_path:?} cannot access {defining_module}")]
    Inaccessible {
        /// Parser anchor of the rejected declaration.
        declaration_span: CanonicalStructuralDiagnosticValue<Span>,
        /// Parser anchor of the importing declaration.
        use_span: CanonicalStructuralDiagnosticValue<Span>,
        /// Canonical identity that owns the rejected declaration.
        defining_module: CanonicalStructuralDiagnosticValue<ModuleKey>,
        /// Parsed path segments, including the `crate` head.
        attempted_path: CanonicalStructuralDiagnosticValue<Vec<Box<str>>>,
        /// Visibility boundary that denied this request.
        violated_visibility: CanonicalStructuralDiagnosticValue<Visibility>,
    },
    /// A parsed use declaration lies outside this bounded structural route.
    #[error("unsupported canonical structural import: {reason}")]
    Unsupported {
        /// Parser anchor of the rejected use declaration.
        span: CanonicalStructuralDiagnosticValue<Span>,
        /// Stable explanation of the unsupported form.
        reason: &'static str,
    },
    /// An alias would overwrite an ordinary declaration in its importing scope.
    #[error("canonical structural import alias {name:?} collides in {importing_module}")]
    LocalDeclarationCollision {
        /// Canonical module receiving the alias.
        importing_module: CanonicalStructuralDiagnosticValue<ModuleKey>,
        /// Requested local alias spelling.
        name: String,
        /// Parser anchor of the ordinary local function.
        declaration_span: CanonicalStructuralDiagnosticValue<Span>,
        /// Parser anchor of the colliding use declaration.
        use_span: CanonicalStructuralDiagnosticValue<Span>,
    },
    /// Two selected imports would stage the same local alias.
    #[error("duplicate canonical structural import alias {name:?} in {importing_module}")]
    DuplicateBinding {
        /// Canonical module receiving the duplicate alias.
        importing_module: CanonicalStructuralDiagnosticValue<ModuleKey>,
        /// Requested duplicate local spelling.
        name: CanonicalStructuralDiagnosticValue<Box<str>>,
        /// Parser anchor of the later conflicting use or grouped member.
        use_span: CanonicalStructuralDiagnosticValue<Span>,
    },
}

/// Immutable direct-child and ordinary-function facts for every parser module.
///
/// Instances are created only by [`Self::from_graph`]. The contained parser
/// anchors and artifact facts are retained for the bounded structural resolver.
#[derive(Debug, Clone)]
pub struct CanonicalProvisionalModuleScopes {
    declaration_snapshot: BTreeMap<ModuleKey, CanonicalProvisionalModuleScope>,
    graph_root: ModuleKey,
    artifacts: BTreeMap<ModuleKey, ModuleArtifact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalProvisionalModuleScope {
    children: BTreeMap<Box<str>, CanonicalProvisionalChild>,
    functions: BTreeMap<Box<str>, CanonicalProvisionalFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalProvisionalChild {
    module_key: ModuleKey,
    declaration_span: Span,
    origin: ModuleArtifactOrigin,
    visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalProvisionalFunction {
    module_key: ModuleKey,
    name: Box<str>,
    declaration_span: Span,
    origin: ModuleArtifactOrigin,
    visibility: Visibility,
}

impl CanonicalProvisionalModuleScopes {
    /// Derives a complete immutable scope snapshot from parser graph units.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalStructuralImportError::ScopeConstruction`] when the
    /// retained units, artifacts, declarations, or direct-child topology do
    /// not agree before any scope becomes observable.
    #[allow(
        clippy::result_large_err,
        reason = "the public diagnostic contract retains its structural facts directly"
    )]
    pub fn from_graph(
        graph: &CanonicalModuleGraph,
    ) -> Result<Self, CanonicalStructuralImportError> {
        let mut declaration_snapshot = BTreeMap::new();
        let mut artifacts = BTreeMap::new();

        for (module_key, unit) in graph.module_units() {
            let artifact = unit.artifact();
            if artifact.key() != module_key
                || artifact.structural_parent() != module_key.parent().as_ref()
            {
                return Err(CanonicalStructuralImportError::ScopeConstruction {
                    reason: "a retained artifact does not match its canonical module key",
                });
            }
            let graph_children = graph.children(module_key).ok_or(
                CanonicalStructuralImportError::ScopeConstruction {
                    reason: "a retained module has no direct-child graph entry",
                },
            )?;
            if artifact.child_keys() != graph_children {
                return Err(CanonicalStructuralImportError::ScopeConstruction {
                    reason: "artifact and graph direct-child facts differ",
                });
            }

            let mut children = BTreeMap::new();
            for declaration in unit.body().module_decls() {
                let child_key = module_key.child(declaration.name.as_ref()).map_err(|_| {
                    CanonicalStructuralImportError::ScopeConstruction {
                        reason: "a parsed child declaration has a noncanonical name",
                    }
                })?;
                if !graph_children.contains(&child_key) {
                    return Err(CanonicalStructuralImportError::ScopeConstruction {
                        reason: "a parsed child declaration is absent from graph topology",
                    });
                }
                let child_unit = graph.module_unit(&child_key).ok_or(
                    CanonicalStructuralImportError::ScopeConstruction {
                        reason: "a graph child has no retained parser unit",
                    },
                )?;
                let child_artifact = child_unit.artifact();
                if child_artifact.key() != &child_key
                    || child_artifact.structural_parent() != Some(module_key)
                {
                    return Err(CanonicalStructuralImportError::ScopeConstruction {
                        reason: "a graph child does not retain its canonical parent identity",
                    });
                }
                let child = CanonicalProvisionalChild {
                    module_key: child_key,
                    declaration_span: declaration.span,
                    origin: child_artifact.origin().clone(),
                    visibility: declaration.visibility.clone(),
                };
                if children.insert(declaration.name.clone(), child).is_some() {
                    return Err(CanonicalStructuralImportError::ScopeConstruction {
                        reason: "a module contains duplicate direct-child names",
                    });
                }
            }
            if children.len() != graph_children.len()
                || !children
                    .values()
                    .all(|child| graph_children.contains(&child.module_key))
            {
                return Err(CanonicalStructuralImportError::ScopeConstruction {
                    reason: "graph topology contains a child without a matching declaration",
                });
            }

            let mut functions = BTreeMap::new();
            for definition in unit.body().definitions() {
                let Definition::Function(function) = definition else {
                    continue;
                };
                let entry = CanonicalProvisionalFunction {
                    module_key: module_key.clone(),
                    name: function.name.clone(),
                    declaration_span: function.span,
                    origin: artifact.origin().clone(),
                    visibility: function.visibility.clone(),
                };
                if functions.insert(function.name.clone(), entry).is_some() {
                    return Err(CanonicalStructuralImportError::ScopeConstruction {
                        reason: "a module contains duplicate ordinary function names",
                    });
                }
            }

            declaration_snapshot.insert(
                module_key.clone(),
                CanonicalProvisionalModuleScope {
                    children,
                    functions,
                },
            );
            artifacts.insert(module_key.clone(), artifact.clone());
        }

        if !declaration_snapshot.contains_key(graph.root_key()) {
            return Err(CanonicalStructuralImportError::ScopeConstruction {
                reason: "the canonical root has no retained parser unit",
            });
        }

        Ok(Self {
            declaration_snapshot,
            graph_root: graph.root_key().clone(),
            artifacts,
        })
    }

    /// Returns whether this snapshot has a scope for `module_key`.
    #[must_use]
    pub fn contains_module(&self, module_key: &ModuleKey) -> bool {
        self.declaration_snapshot.contains_key(module_key)
    }

    /// Returns a deterministic, layout-independent scope projection.
    #[must_use]
    pub fn normalized_scope_projection(&self) -> CanonicalNormalizedScopeProjection {
        let modules = self
            .declaration_snapshot
            .iter()
            .map(|(module_key, scope)| {
                let children = scope
                    .children
                    .iter()
                    .map(|(name, child)| {
                        (
                            name.clone(),
                            CanonicalNormalizedChild {
                                module_key: child.module_key.clone(),
                                visibility: child.visibility.clone(),
                            },
                        )
                    })
                    .collect();
                let functions = scope
                    .functions
                    .iter()
                    .map(|(name, function)| (name.clone(), function.visibility.clone()))
                    .collect();
                (
                    module_key.clone(),
                    CanonicalNormalizedModuleScope {
                        children,
                        functions,
                    },
                )
            })
            .collect();
        CanonicalNormalizedScopeProjection { modules }
    }

    /// Evaluates one parser visibility in canonical structural space.
    ///
    /// `defining_module` is the canonical identity of the child or function
    /// declaration. The result does not authorize wider route forms.
    #[allow(
        clippy::result_large_err,
        reason = "this read-only query shares the scope diagnostic contract"
    )]
    pub fn is_visible_from(
        &self,
        visibility: &Visibility,
        defining_module: &ModuleKey,
        requesting_module: &ModuleKey,
    ) -> Result<bool, CanonicalStructuralImportError> {
        let visible = match visibility {
            Visibility::Inherited | Visibility::Self_ => requesting_module == defining_module,
            Visibility::Public => true,
            Visibility::Crate => same_crate(defining_module, requesting_module),
            Visibility::Super { levels } => ancestor(defining_module, *levels)
                .is_some_and(|region| is_same_or_descendant(&region, requesting_module)),
            Visibility::Restricted { path } => self
                .restricted_region(path, defining_module)
                .is_some_and(|region| {
                    is_same_or_descendant(&region, defining_module)
                        && is_same_or_descendant(&region, requesting_module)
                }),
        };
        Ok(visible)
    }

    pub(crate) fn scope(&self, module_key: &ModuleKey) -> Option<&CanonicalProvisionalModuleScope> {
        self.declaration_snapshot.get(module_key)
    }

    pub(crate) fn matches_graph(&self, graph: &CanonicalModuleGraph) -> bool {
        self.graph_root == *graph.root_key()
            && self.artifacts.len() == graph.module_units().count()
            && graph.module_units().all(|(key, unit)| {
                self.artifacts
                    .get(key)
                    .is_some_and(|artifact| artifact == unit.artifact())
            })
            && Self::from_graph(graph)
                .is_ok_and(|current| self.declaration_snapshot == current.declaration_snapshot)
    }

    fn restricted_region(&self, path: &str, defining_module: &ModuleKey) -> Option<ModuleKey> {
        let mut segments = path.split("::");
        if segments.next()? != "crate" {
            return None;
        }
        let mut region = crate_root(defining_module);
        for segment in segments {
            if segment.is_empty() {
                return None;
            }
            region = region.child(segment).ok()?;
        }
        self.declaration_snapshot
            .contains_key(&region)
            .then_some(region)
    }
}

impl CanonicalProvisionalModuleScope {
    pub(crate) fn child(&self, name: &str) -> Option<&CanonicalProvisionalChild> {
        self.children.get(name)
    }

    pub(crate) fn function(&self, name: &str) -> Option<&CanonicalProvisionalFunction> {
        self.functions.get(name)
    }
}

impl CanonicalProvisionalChild {
    pub(crate) fn module_key(&self) -> &ModuleKey {
        &self.module_key
    }

    pub(crate) const fn declaration_span(&self) -> Span {
        self.declaration_span
    }

    pub(crate) fn origin(&self) -> &ModuleArtifactOrigin {
        &self.origin
    }

    pub(crate) fn visibility(&self) -> &Visibility {
        &self.visibility
    }
}

impl CanonicalProvisionalFunction {
    pub(crate) fn module_key(&self) -> &ModuleKey {
        &self.module_key
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) const fn declaration_span(&self) -> Span {
        self.declaration_span
    }

    pub(crate) fn origin(&self) -> &ModuleArtifactOrigin {
        &self.origin
    }

    pub(crate) fn visibility(&self) -> &Visibility {
        &self.visibility
    }
}

fn same_crate(left: &ModuleKey, right: &ModuleKey) -> bool {
    crate_root(left) == crate_root(right)
}

fn crate_root(module_key: &ModuleKey) -> ModuleKey {
    let mut root = module_key.clone();
    while let Some(parent) = root.parent() {
        root = parent;
    }
    root
}

fn ancestor(module_key: &ModuleKey, levels: usize) -> Option<ModuleKey> {
    let mut ancestor = module_key.clone();
    for _ in 0..levels {
        ancestor = ancestor.parent()?;
    }
    Some(ancestor)
}

fn is_same_or_descendant(ancestor: &ModuleKey, candidate: &ModuleKey) -> bool {
    same_crate(ancestor, candidate)
        && (ancestor == candidate || candidate.segments().starts_with(ancestor.segments()))
}
