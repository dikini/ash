//! Canonical planning for bounded parsed simple imports.
//!
//! This pass consumes parser-owned graph units, collects ordinary functions,
//! resolves inherited `crate::…` simple imports, records cross-module edges,
//! and rejects cycles before publishing a plan. It remains a non-authorizing
//! Type-layer planning fact.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Deref;

use ash_core::module_graph::{ModuleArtifact, ModuleArtifactOrigin, ModuleKey};
use ash_parser::surface::{FnDef, Type as SurfaceType};
use ash_parser::{CanonicalModuleGraph, Definition, Span, Use, UseItem, UsePath, Visibility};

use crate::canonical_provisional_module_scopes::{
    CanonicalProvisionalModuleScopes, CanonicalStructuralImportError,
};

/// The stable parsed identity of a provisionally collected declaration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalDefinitionIdentity {
    module_key: ModuleKey,
    name: Box<str>,
}

impl CanonicalDefinitionIdentity {
    /// Returns the canonical key of the module that declares this name.
    #[must_use]
    pub fn module_key(&self) -> &ModuleKey {
        &self.module_key
    }

    /// Returns the parsed defining name before any import alias is applied.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// One resolved alias binding produced from a parsed `use` declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalBoundModuleBinding {
    defining_identity: CanonicalDefinitionIdentity,
    declaration_span: Span,
    origin: ModuleArtifactOrigin,
    visibility: Visibility,
}

impl CanonicalBoundModuleBinding {
    /// Returns the original declaration identity preserved through aliasing.
    #[must_use]
    pub fn defining_identity(&self) -> &CanonicalDefinitionIdentity {
        &self.defining_identity
    }

    /// Returns the parser anchor of the defining declaration.
    #[must_use]
    pub const fn declaration_span(&self) -> Span {
        self.declaration_span
    }

    /// Returns parser acquisition provenance for the defining declaration.
    #[must_use]
    pub fn origin(&self) -> &ModuleArtifactOrigin {
        &self.origin
    }

    /// Returns the parsed visibility retained for later, fuller checks.
    #[must_use]
    pub fn visibility(&self) -> &Visibility {
        &self.visibility
    }
}

/// One resolved cross-module simple import dependency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalSimpleImportEdge {
    importing_module: ModuleKey,
    defining_module: ModuleKey,
    defining_identity: CanonicalDefinitionIdentity,
    local_name: Box<str>,
    use_span: Span,
    declaration_span: Span,
    origin: ModuleArtifactOrigin,
    visibility: Visibility,
}

impl CanonicalSimpleImportEdge {
    /// Returns the canonical module that contains the parsed use declaration.
    #[must_use]
    pub fn importing_module(&self) -> &ModuleKey {
        &self.importing_module
    }

    /// Returns the canonical module defining the selected declaration.
    #[must_use]
    pub fn defining_module(&self) -> &ModuleKey {
        &self.defining_module
    }

    /// Returns the defining identity preserved through aliasing.
    #[must_use]
    pub fn defining_identity(&self) -> &CanonicalDefinitionIdentity {
        &self.defining_identity
    }

    /// Returns the local name selected by the parsed use declaration.
    #[must_use]
    pub fn local_name(&self) -> &str {
        &self.local_name
    }

    /// Returns the parser anchor of the parsed use declaration.
    #[must_use]
    pub const fn use_span(&self) -> Span {
        self.use_span
    }

    /// Returns the parser anchor of the selected declaration.
    #[must_use]
    pub const fn declaration_span(&self) -> Span {
        self.declaration_span
    }

    /// Returns parser acquisition provenance for the selected declaration.
    #[must_use]
    pub fn origin(&self) -> &ModuleArtifactOrigin {
        &self.origin
    }

    /// Returns the selected declaration's parsed visibility.
    #[must_use]
    pub fn visibility(&self) -> &Visibility {
        &self.visibility
    }
}

/// Ordered cross-module edges that close one detected import cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalImportCycle {
    edges: Box<[CanonicalSimpleImportEdge]>,
}

impl CanonicalImportCycle {
    /// Returns the ordered edges, including the edge that closes the cycle.
    #[must_use]
    pub fn edges(&self) -> &[CanonicalSimpleImportEdge] {
        &self.edges
    }
}

impl Deref for CanonicalImportCycle {
    type Target = [CanonicalSimpleImportEdge];

    fn deref(&self) -> &Self::Target {
        self.edges()
    }
}

/// Atomically collected bindings indexed by importing module and local name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalBoundModuleSet {
    bindings: BTreeMap<ModuleKey, BTreeMap<Box<str>, CanonicalBoundModuleBinding>>,
}

impl CanonicalBoundModuleSet {
    /// Returns the binding named `name` in `module` when planning succeeds.
    #[must_use]
    pub fn binding(&self, module: &ModuleKey, name: &str) -> Option<&CanonicalBoundModuleBinding> {
        self.bindings.get(module)?.get(name)
    }
}

/// A resolved simple-import plan with bindings and canonical dependency edges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalResolvedSimpleImports {
    bindings: BTreeMap<ModuleKey, BTreeMap<Box<str>, CanonicalBoundModuleBinding>>,
    import_edges: Box<[CanonicalSimpleImportEdge]>,
    graph_root: ModuleKey,
    artifacts: BTreeMap<ModuleKey, ModuleArtifact>,
}

impl CanonicalResolvedSimpleImports {
    /// Returns the binding named `name` in `module` when planning succeeds.
    #[must_use]
    pub fn binding(&self, module: &ModuleKey, name: &str) -> Option<&CanonicalBoundModuleBinding> {
        self.bindings.get(module)?.get(name)
    }

    /// Returns cross-module dependencies in deterministic canonical order.
    #[must_use]
    pub fn import_edges(&self) -> &[CanonicalSimpleImportEdge] {
        &self.import_edges
    }

    /// Converts a validated plan into its binding-only compatibility view.
    pub(crate) fn into_bound_set(self) -> CanonicalBoundModuleSet {
        CanonicalBoundModuleSet {
            bindings: self.bindings,
        }
    }

    /// Returns whether this plan was derived from exactly the supplied graph facts.
    pub(crate) fn matches_graph(&self, graph: &CanonicalModuleGraph) -> bool {
        self.graph_root == *graph.root_key()
            && self.artifacts.len() == graph.module_units().count()
            && graph.module_units().all(|(key, unit)| {
                self.artifacts
                    .get(key)
                    .is_some_and(|artifact| artifact == unit.artifact())
            })
    }
}

/// A failure while collecting or planning parsed simple imports.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CanonicalModuleBindError {
    /// A parsed declaration name occurs more than once in one module.
    #[error("duplicate provisional declaration {name:?} in {module}")]
    DuplicateDeclaration {
        /// The module that contained both declarations.
        module: ModuleKey,
        /// The duplicated parsed name.
        name: Box<str>,
    },
    /// A supported parsed path names no collected declaration.
    #[error("unresolved parsed import path {attempted_path:?}")]
    Unresolved {
        /// The original parsed path, including its `crate` head.
        attempted_path: Vec<Box<str>>,
    },
    /// A parsed path found a declaration that is not visible to its importer.
    #[error("parsed import path {attempted_path:?} cannot access {defining_module}")]
    Inaccessible {
        /// The parser anchor of the declaration that rejected access.
        declaration_span: Span,
        /// The canonical module that defines the inaccessible name.
        defining_module: ModuleKey,
        /// The original parsed path, including its `crate` head.
        attempted_path: Vec<Box<str>>,
        /// The parsed visibility boundary that denied access.
        violated_visibility: Visibility,
    },
    /// A parsed use form or declaration visibility is outside the bounded slice.
    #[error("unsupported parsed import form: {reason}")]
    Unsupported {
        /// The parser anchor of the unsupported use declaration.
        span: Span,
        /// A stable explanation of the unsupported form.
        reason: &'static str,
    },
    /// Multiple uses in one module choose the same local binding name.
    #[error("duplicate parsed import binding {name:?} in {module}")]
    DuplicateBinding {
        /// The importing module that would receive the duplicate binding.
        module: ModuleKey,
        /// The local name that would be assigned twice.
        name: Box<str>,
    },
    /// Resolved cross-module imports close a canonical dependency cycle.
    #[error("canonical simple-import cycle")]
    ImportCycle {
        /// Ordered cycle edges, including the closing edge.
        edges: CanonicalImportCycle,
    },
}

/// A failure while planning the bounded direct public re-export fragment.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CanonicalDirectPrimitiveInterfaceImportError {
    /// The root does not contain an explicit public re-export for this fragment.
    #[error("missing direct public primitive re-export in {root_module}")]
    MissingPublicReexport {
        /// The canonical root module that must expose an explicit alias.
        root_module: ModuleKey,
        /// The parser anchor of the empty root export surface.
        span: Span,
    },
    /// A root public re-export crosses a structural child that is not public.
    #[error("non-public direct structural path from {root_module} to {child_module}")]
    NonPublicStructuralPath {
        /// The canonical root module containing the re-export.
        root_module: ModuleKey,
        /// The direct structural child named by the re-export path.
        child_module: ModuleKey,
        /// The parser anchor of the root child declaration.
        declaration_span: Span,
    },
    /// A direct public re-export selects a declaration that is not public.
    #[error("private direct re-export target {function:?} in {defining_module}")]
    PrivateTarget {
        /// The canonical module defining the selected declaration.
        defining_module: ModuleKey,
        /// The selected defining function name.
        function: Box<str>,
        /// The parser anchor of the selected declaration.
        declaration_span: Span,
        /// The visibility that prevents public re-export.
        visibility: Visibility,
    },
    /// The parsed form falls outside the direct public re-export fragment.
    #[error("unsupported direct public re-export form: {reason}")]
    Unsupported {
        /// The parser anchor of the rejected use or structural form.
        span: Span,
        /// A stable explanation of the bounded rejection.
        reason: &'static str,
    },
    /// Two root re-exports choose the same visible alias.
    #[error("duplicate direct public re-export binding {name:?} in {root_module}")]
    DuplicateBinding {
        /// The canonical root module containing the conflicting aliases.
        root_module: ModuleKey,
        /// The duplicate root-visible alias.
        name: Box<str>,
    },
}

/// An unforgeable plan for one private root client of a direct public re-export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalDirectPrimitiveReexportRootClientPlan {
    direct_reexport_plan: CanonicalResolvedSimpleImports,
    private_root_functions: BTreeMap<Box<str>, Span>,
}

impl CanonicalDirectPrimitiveReexportRootClientPlan {
    /// Returns whether this plan was derived from exactly the supplied graph facts.
    pub(crate) fn matches_graph(&self, graph: &CanonicalModuleGraph) -> bool {
        self.direct_reexport_plan.matches_graph(graph)
    }

    /// Returns the retained direct public re-export plan for the dedicated checker.
    pub(crate) fn direct_reexport_plan(&self) -> &CanonicalResolvedSimpleImports {
        &self.direct_reexport_plan
    }

    /// Returns the inherited root declarations admitted by this plan.
    pub(crate) fn private_root_functions(&self) -> &BTreeMap<Box<str>, Span> {
        &self.private_root_functions
    }
}

/// A failure while planning a private root client of a direct public re-export.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CanonicalDirectPrimitiveReexportRootClientPlanError {
    /// The direct public re-export portion lies outside its bounded route.
    #[error("direct root-client re-export plan is invalid")]
    DirectReexport {
        /// The anchored direct public re-export planning failure.
        #[source]
        source: Box<CanonicalDirectPrimitiveInterfaceImportError>,
    },
    /// The root has more than one explicit public re-export.
    #[error("root client plan requires exactly one direct public re-export in {root_module}")]
    MultiplePublicReexports {
        /// The canonical root module containing the re-exports.
        root_module: ModuleKey,
        /// The parser anchor of the root module body.
        span: Span,
    },
    /// A root ordinary function is public rather than private to this route.
    #[error("public root function {function:?} is outside the direct root-client route")]
    PublicRootFunction {
        /// The canonical root module containing the public function.
        root_module: ModuleKey,
        /// The rejected function name.
        function: String,
        /// The parser anchor of the rejected declaration.
        declaration_span: Span,
    },
    /// A root function is not an inherited closed primitive ordinary function.
    #[error("unsupported private root function {function:?} in {root_module}: {reason}")]
    UnsupportedPrivateRootFunction {
        /// The canonical root module containing the rejected function.
        root_module: ModuleKey,
        /// The rejected function name.
        function: String,
        /// The parser anchor of the rejected declaration.
        declaration_span: Span,
        /// Stable rejection reason.
        reason: &'static str,
    },
    /// A root definition is not an ordinary function in this route.
    #[error("unsupported root definition in {root_module}")]
    UnsupportedRootDefinition {
        /// The canonical root module containing the rejected definition.
        root_module: ModuleKey,
        /// The parser anchor of the root definition.
        span: Span,
    },
    /// The root has no private client function to check.
    #[error("missing private root client function in {root_module}")]
    MissingPrivateRootFunction {
        /// The canonical root module that needs an inherited ordinary function.
        root_module: ModuleKey,
        /// The parser anchor of the root module body.
        span: Span,
    },
    /// Two root private functions have the same defining name.
    #[error("duplicate private root function {function:?} in {root_module}")]
    DuplicatePrivateRootFunction {
        /// The canonical root module containing the duplicate definition.
        root_module: ModuleKey,
        /// The duplicated defining name.
        function: String,
        /// The parser anchor of the later declaration.
        declaration_span: Span,
    },
}

#[derive(Debug, Clone)]
struct ProvisionalFunction {
    identity: CanonicalDefinitionIdentity,
    declaration_span: Span,
    origin: ModuleArtifactOrigin,
    visibility: Visibility,
}

type ProvisionalDeclarations = BTreeMap<ModuleKey, BTreeMap<Box<str>, ProvisionalFunction>>;

/// Resolves every bounded simple parsed import and rejects cross-module cycles.
///
/// Only inherited [`UsePath::Simple`] declarations headed by `crate` are
/// supported. Cross-module targets must be public ordinary functions; all
/// candidate imports are resolved before cycle detection runs.
///
/// # Errors
///
/// Returns [`CanonicalModuleBindError`] for unsupported, unresolved,
/// inaccessible, duplicate, or cyclic import sets.
#[allow(
    clippy::result_large_err,
    reason = "the public diagnostic contract exposes its anchored fields without boxing"
)]
pub fn resolve_simple_parsed_imports(
    graph: &CanonicalModuleGraph,
) -> Result<CanonicalResolvedSimpleImports, CanonicalModuleBindError> {
    let declarations = collect_functions(graph)?;
    let artifacts = graph
        .module_units()
        .map(|(key, unit)| (key.clone(), unit.artifact().clone()))
        .collect();
    let mut bindings = BTreeMap::new();
    let mut import_edges = Vec::new();

    for (importing_module, unit) in graph.module_units() {
        let mut module_bindings = BTreeMap::new();
        for use_declaration in unit.body().uses() {
            let (local_name, binding, edge) = resolve_candidate(
                graph.root_key(),
                importing_module,
                use_declaration,
                &declarations,
            )?;
            if module_bindings
                .insert(local_name.clone(), binding)
                .is_some()
            {
                return Err(CanonicalModuleBindError::DuplicateBinding {
                    module: importing_module.clone(),
                    name: local_name,
                });
            }
            if let Some(edge) = edge {
                import_edges.push(edge);
            }
        }
        if !module_bindings.is_empty() {
            bindings.insert(importing_module.clone(), module_bindings);
        }
    }

    if let Some(cycle) = find_cycle(&import_edges) {
        return Err(CanonicalModuleBindError::ImportCycle { edges: cycle });
    }

    Ok(CanonicalResolvedSimpleImports {
        bindings,
        import_edges: import_edges.into_boxed_slice(),
        graph_root: graph.root_key().clone(),
        artifacts,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopedOrdinaryFunctionAliasMode {
    ExplicitStructuralAlias,
    OptionalAlias,
}

/// Resolves inherited explicit structural function aliases through provisional scopes.
///
/// This opt-in route accepts exactly `use crate::<child>...::<function> as
/// <alias>`. It preflights every direct-child edge, its visibility, the final
/// ordinary function, and local declaration collisions before returning any
/// binding. It neither finalizes an interface nor widens the generic planner.
///
/// # Errors
///
/// Returns [`CanonicalStructuralImportError`] when the parser graph and scope
/// snapshot differ, a selected structural path is invalid, or a candidate
/// alias cannot be staged atomically.
#[allow(
    clippy::result_large_err,
    reason = "the public diagnostic contract exposes anchored structural facts directly"
)]
pub fn resolve_simple_parsed_imports_with_scopes(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<CanonicalResolvedSimpleImports, CanonicalStructuralImportError> {
    resolve_scoped_ordinary_function_imports(
        graph,
        scopes,
        ScopedOrdinaryFunctionAliasMode::ExplicitStructuralAlias,
    )
}

/// Resolves inherited simple ordinary-function imports through provisional scopes.
///
/// This route accepts `use crate::<function>` and paths through direct
/// structural children, with an optional `as <alias>`. When omitted, the
/// target function's final path segment becomes the local binding name. It
/// retains the scoped route's visibility, collision, cycle, and atomicity
/// checks without widening the generic planner.
///
/// # Errors
///
/// Returns [`CanonicalStructuralImportError`] unchanged when scope matching,
/// structural traversal, visibility, binding preflight, or cycle detection
/// rejects the selected imports.
#[allow(
    clippy::result_large_err,
    reason = "the public diagnostic contract exposes anchored structural facts directly"
)]
pub fn resolve_scoped_simple_ordinary_function_imports_with_scopes(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<CanonicalResolvedSimpleImports, CanonicalStructuralImportError> {
    resolve_scoped_ordinary_function_imports(
        graph,
        scopes,
        ScopedOrdinaryFunctionAliasMode::OptionalAlias,
    )
}

/// Resolves selected simple imports before applying local-name precedence.
///
/// This dedicated route accepts exactly one inherited, unaliased `use
/// crate::<public-child>...::<public-function>` declaration in each importer.
/// It retains every selected cross-module edge through deterministic cycle
/// detection, then omits a natural-name binding shadowed by an ordinary
/// function declared in that importer. It neither changes the broader scoped
/// simple-import route nor grants authority beyond Type-layer planning facts.
///
/// # Errors
///
/// Returns [`CanonicalStructuralImportError`] when graph and scope facts
/// differ, the narrow route rejects an import, a selected path is inaccessible,
/// or the complete selected edge set closes a cycle. No plan is returned on
/// failure.
#[allow(
    clippy::result_large_err,
    reason = "the public diagnostic contract exposes anchored structural facts directly"
)]
pub fn resolve_scoped_simple_local_precedence_imports_with_scopes(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<CanonicalResolvedSimpleImports, CanonicalStructuralImportError> {
    if !scopes.matches_graph(graph) {
        return Err(CanonicalStructuralImportError::ScopeGraphMismatch);
    }

    let artifacts = graph
        .module_units()
        .map(|(key, unit)| (key.clone(), unit.artifact().clone()))
        .collect();
    let mut bindings = BTreeMap::new();
    let mut import_edges = Vec::new();

    for (importing_module, unit) in graph.module_units() {
        let uses = unit.body().uses();
        if uses.is_empty() {
            continue;
        }
        if uses.len() != 1 {
            return Err(CanonicalStructuralImportError::Unsupported {
                span: uses[1].span.into(),
                reason: "a local-precedence simple importer requires exactly one use declaration",
            });
        }

        let (local_name, binding, edge) = resolve_scoped_simple_local_precedence_candidate(
            graph,
            scopes,
            importing_module,
            &uses[0],
        )?;
        let mut staged_bindings: BTreeMap<Box<str>, CanonicalBoundModuleBinding> = BTreeMap::new();
        if staged_bindings
            .insert(local_name.clone(), binding)
            .is_some()
        {
            return Err(CanonicalStructuralImportError::DuplicateBinding {
                importing_module: importing_module.clone().into(),
                name: local_name.into(),
                use_span: uses[0].span.into(),
            });
        }
        bindings.insert(importing_module.clone(), staged_bindings);
        if let Some(edge) = edge {
            import_edges.push(edge);
        }
    }

    if let Some(cycle) = find_cycle(&import_edges) {
        return Err(CanonicalStructuralImportError::ImportCycle { edges: cycle });
    }

    let bindings = filter_local_shadowed_simple_bindings(bindings, scopes)?;
    Ok(CanonicalResolvedSimpleImports {
        bindings,
        import_edges: import_edges.into_boxed_slice(),
        graph_root: graph.root_key().clone(),
        artifacts,
    })
}

#[allow(
    clippy::result_large_err,
    reason = "the route returns anchored structural diagnostics without erasing their facts"
)]
fn resolve_scoped_simple_local_precedence_candidate(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
    importing_module: &ModuleKey,
    use_declaration: &Use,
) -> Result<
    (
        Box<str>,
        CanonicalBoundModuleBinding,
        Option<CanonicalSimpleImportEdge>,
    ),
    CanonicalStructuralImportError,
> {
    if !matches!(&use_declaration.visibility, Visibility::Inherited) {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "public and restricted uses are outside the local-precedence simple route",
        });
    }
    if use_declaration.alias.is_some() {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a local-precedence simple import cannot carry an alias",
        });
    }
    let UsePath::Simple(path) = &use_declaration.path else {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "only simple crate structural paths are accepted",
        });
    };
    let attempted_path = path.segments.clone();
    let Some((head, tail)) = attempted_path.split_first() else {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a local-precedence simple import requires a crate head, child, and function",
        });
    };
    if head.as_ref() != "crate" {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "only crate-root local-precedence simple imports are accepted",
        });
    }
    let Some((function_name, child_segments)) = tail.split_last() else {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a local-precedence simple import requires a structural child and function",
        });
    };
    if child_segments.is_empty() {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a local-precedence simple import requires a public structural child",
        });
    }
    if child_segments
        .iter()
        .any(|segment| matches!(segment.as_ref(), "crate" | "self" | "super"))
        || matches!(function_name.as_ref(), "crate" | "self" | "super")
    {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a local-precedence simple import accepts only structural child segments after crate",
        });
    }

    let mut defining_module = graph.root_key().clone();
    for segment in child_segments {
        let scope = scopes
            .scope(&defining_module)
            .ok_or(CanonicalStructuralImportError::ScopeGraphMismatch)?;
        let child = scope.child(segment.as_ref()).ok_or_else(|| {
            if scope.function(segment.as_ref()).is_some()
                || contains_non_function_target(graph, &defining_module, segment.as_ref())
            {
                CanonicalStructuralImportError::Unsupported {
                    span: use_declaration.span.into(),
                    reason: "a local-precedence simple path segment must be a structural module",
                }
            } else {
                CanonicalStructuralImportError::Unresolved {
                    use_span: use_declaration.span.into(),
                    attempted_path: attempted_path.clone().into(),
                }
            }
        })?;
        if graph
            .module_unit(child.module_key())
            .is_none_or(|unit| unit.artifact().origin() != child.origin())
        {
            return Err(CanonicalStructuralImportError::ScopeGraphMismatch);
        }
        if !scopes.is_visible_from(child.visibility(), child.module_key(), importing_module)?
            || !matches!(child.visibility(), Visibility::Public)
        {
            return Err(CanonicalStructuralImportError::Inaccessible {
                declaration_span: child.declaration_span().into(),
                use_span: use_declaration.span.into(),
                defining_module: child.module_key().clone().into(),
                attempted_path: attempted_path.clone().into(),
                violated_visibility: child.visibility().clone().into(),
            });
        }
        defining_module = child.module_key().clone();
    }

    let defining_scope = scopes
        .scope(&defining_module)
        .ok_or(CanonicalStructuralImportError::ScopeGraphMismatch)?;
    let function = if let Some(function) = defining_scope.function(function_name.as_ref()) {
        function
    } else if contains_non_function_target(graph, &defining_module, function_name.as_ref()) {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "only ordinary function targets are accepted",
        });
    } else {
        return Err(CanonicalStructuralImportError::Unresolved {
            use_span: use_declaration.span.into(),
            attempted_path: attempted_path.into(),
        });
    };
    if !scopes.is_visible_from(
        function.visibility(),
        function.module_key(),
        importing_module,
    )? || !matches!(function.visibility(), Visibility::Public)
    {
        return Err(CanonicalStructuralImportError::Inaccessible {
            declaration_span: function.declaration_span().into(),
            use_span: use_declaration.span.into(),
            defining_module: function.module_key().clone().into(),
            attempted_path: attempted_path.into(),
            violated_visibility: function.visibility().clone().into(),
        });
    }

    let local_name: Box<str> = function.name().into();
    let identity = CanonicalDefinitionIdentity {
        module_key: function.module_key().clone(),
        name: function.name().into(),
    };
    let binding = CanonicalBoundModuleBinding {
        defining_identity: identity.clone(),
        declaration_span: function.declaration_span(),
        origin: function.origin().clone(),
        visibility: function.visibility().clone(),
    };
    let edge = (importing_module != function.module_key()).then(|| CanonicalSimpleImportEdge {
        importing_module: importing_module.clone(),
        defining_module: function.module_key().clone(),
        defining_identity: identity,
        local_name: local_name.clone(),
        use_span: use_declaration.span,
        declaration_span: function.declaration_span(),
        origin: function.origin().clone(),
        visibility: function.visibility().clone(),
    });
    Ok((local_name, binding, edge))
}

fn filter_local_shadowed_simple_bindings(
    bindings: BTreeMap<ModuleKey, BTreeMap<Box<str>, CanonicalBoundModuleBinding>>,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<
    BTreeMap<ModuleKey, BTreeMap<Box<str>, CanonicalBoundModuleBinding>>,
    CanonicalStructuralImportError,
> {
    let mut filtered_bindings = BTreeMap::new();
    for (module_key, module_bindings) in bindings {
        let scope = scopes
            .scope(&module_key)
            .ok_or(CanonicalStructuralImportError::ScopeGraphMismatch)?;
        let projected: BTreeMap<Box<str>, CanonicalBoundModuleBinding> = module_bindings
            .into_iter()
            .filter(|(name, _)| scope.function(name.as_ref()).is_none())
            .collect();
        if !projected.is_empty() {
            filtered_bindings.insert(module_key, projected);
        }
    }
    Ok(filtered_bindings)
}

/// Resolves inherited ordinary-function imports that begin with one `super`.
///
/// This route accepts a non-root module's `use super::<function>` or
/// `use super::<child>...::<function>` declaration, with an optional
/// `as <alias>`. It starts structural lookup from the importing module's
/// canonical parent and retains the complete parser-owned use span in every
/// staged edge and diagnostic. It does not alter crate-root import planning.
///
/// # Errors
///
/// Returns [`CanonicalStructuralImportError`] when the graph and scope
/// snapshot differ, the `super` path is unsupported, or path, visibility,
/// binding, or cycle preflight rejects the complete staged result.
#[allow(
    clippy::result_large_err,
    reason = "the public diagnostic contract exposes anchored structural facts directly"
)]
pub fn resolve_scoped_super_ordinary_function_imports_with_scopes(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<CanonicalResolvedSimpleImports, CanonicalStructuralImportError> {
    if !scopes.matches_graph(graph) {
        return Err(CanonicalStructuralImportError::ScopeGraphMismatch);
    }

    let artifacts = graph
        .module_units()
        .map(|(key, unit)| (key.clone(), unit.artifact().clone()))
        .collect();
    let mut bindings = BTreeMap::new();
    let mut import_edges = Vec::new();

    for (importing_module, unit) in graph.module_units() {
        let importing_scope = scopes
            .scope(importing_module)
            .ok_or(CanonicalStructuralImportError::ScopeGraphMismatch)?;
        let mut staged_bindings = BTreeMap::new();

        for use_declaration in unit.body().uses() {
            let (local_name, binding, edge) = resolve_scoped_super_candidate(
                graph,
                scopes,
                importing_module,
                importing_scope,
                use_declaration,
            )?;
            if staged_bindings
                .insert(local_name.clone(), binding)
                .is_some()
            {
                return Err(CanonicalStructuralImportError::DuplicateBinding {
                    importing_module: importing_module.clone().into(),
                    name: local_name.into(),
                    use_span: use_declaration.span.into(),
                });
            }
            if let Some(edge) = edge {
                import_edges.push(edge);
            }
        }

        if !staged_bindings.is_empty() {
            bindings.insert(importing_module.clone(), staged_bindings);
        }
    }

    if let Some(cycle) = find_cycle(&import_edges) {
        return Err(CanonicalStructuralImportError::ImportCycle { edges: cycle });
    }

    Ok(CanonicalResolvedSimpleImports {
        bindings,
        import_edges: import_edges.into_boxed_slice(),
        graph_root: graph.root_key().clone(),
        artifacts,
    })
}

/// Resolves inherited grouped ordinary-function imports through one `super`.
///
/// This route accepts only a non-root module's `use super::<children>::{
/// function, function as local}` declarations. Every selected member retains
/// its own parser span in staged edges and member diagnostics. It remains a
/// binding-only Type-layer plan and does not widen either generic import
/// planning or crate-root grouped imports.
///
/// # Errors
///
/// Returns [`CanonicalStructuralImportError`] when the supplied scope snapshot
/// differs from the graph, the inherited grouped path is unsupported, a member
/// cannot traverse or access its target, a local name collides, or the
/// complete staged edge set forms a cycle.
#[allow(
    clippy::result_large_err,
    reason = "the public diagnostic contract exposes anchored structural facts directly"
)]
pub fn resolve_scoped_super_grouped_ordinary_function_imports_with_scopes(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<CanonicalResolvedSimpleImports, CanonicalStructuralImportError> {
    if !scopes.matches_graph(graph) {
        return Err(CanonicalStructuralImportError::ScopeGraphMismatch);
    }

    let artifacts = graph
        .module_units()
        .map(|(key, unit)| (key.clone(), unit.artifact().clone()))
        .collect();
    let mut bindings = BTreeMap::new();
    let mut import_edges = Vec::new();

    for (importing_module, unit) in graph.module_units() {
        let importing_scope = scopes
            .scope(importing_module)
            .ok_or(CanonicalStructuralImportError::ScopeGraphMismatch)?;
        let mut staged_bindings = BTreeMap::new();

        for use_declaration in unit.body().uses() {
            let members = resolve_scoped_super_grouped_candidates(
                graph,
                scopes,
                importing_module,
                importing_scope,
                use_declaration,
            )?;
            let mut group_local_names = BTreeSet::new();
            for member in &members {
                if !group_local_names.insert(member.local_name.clone())
                    || staged_bindings.contains_key(member.local_name.as_ref())
                {
                    return Err(CanonicalStructuralImportError::DuplicateBinding {
                        importing_module: importing_module.clone().into(),
                        name: member.local_name.clone().into(),
                        use_span: member.use_span.into(),
                    });
                }
            }
            for member in members {
                let _ = staged_bindings.insert(member.local_name, member.binding);
                if let Some(edge) = member.edge {
                    import_edges.push(edge);
                }
            }
        }

        if !staged_bindings.is_empty() {
            bindings.insert(importing_module.clone(), staged_bindings);
        }
    }

    if let Some(cycle) = find_cycle(&import_edges) {
        return Err(CanonicalStructuralImportError::ImportCycle { edges: cycle });
    }

    Ok(CanonicalResolvedSimpleImports {
        bindings,
        import_edges: import_edges.into_boxed_slice(),
        graph_root: graph.root_key().clone(),
        artifacts,
    })
}

/// Resolves inherited grouped ordinary-function imports through provisional scopes.
///
/// This route accepts only `use crate::<children>::{function, function as
/// local}` declarations. Each grouped member retains its own parser span in
/// the resulting cross-module edge and in member-specific diagnostics. It
/// remains a binding-only Type-layer plan and does not widen the generic
/// import planner.
///
/// # Errors
///
/// Returns [`CanonicalStructuralImportError`] when the supplied scope snapshot
/// differs from the graph, a grouped member cannot traverse or access its
/// target, a local name collides, or the complete staged edge set forms a
/// cycle.
#[allow(
    clippy::result_large_err,
    reason = "the public diagnostic contract exposes anchored structural facts directly"
)]
pub fn resolve_scoped_grouped_ordinary_function_imports_with_scopes(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<CanonicalResolvedSimpleImports, CanonicalStructuralImportError> {
    if !scopes.matches_graph(graph) {
        return Err(CanonicalStructuralImportError::ScopeGraphMismatch);
    }

    let artifacts = graph
        .module_units()
        .map(|(key, unit)| (key.clone(), unit.artifact().clone()))
        .collect();
    let mut bindings = BTreeMap::new();
    let mut import_edges = Vec::new();

    for (importing_module, unit) in graph.module_units() {
        let importing_scope = scopes
            .scope(importing_module)
            .ok_or(CanonicalStructuralImportError::ScopeGraphMismatch)?;
        let mut staged_bindings = BTreeMap::new();

        for use_declaration in unit.body().uses() {
            let members = resolve_scoped_grouped_candidates(
                graph,
                scopes,
                importing_module,
                importing_scope,
                use_declaration,
            )?;
            let mut group_local_names = BTreeSet::new();
            for member in &members {
                if !group_local_names.insert(member.local_name.clone())
                    || staged_bindings.contains_key(member.local_name.as_ref())
                {
                    return Err(CanonicalStructuralImportError::DuplicateBinding {
                        importing_module: importing_module.clone().into(),
                        name: member.local_name.clone().into(),
                        use_span: member.use_span.into(),
                    });
                }
            }
            for member in members {
                let _ = staged_bindings.insert(member.local_name, member.binding);
                if let Some(edge) = member.edge {
                    import_edges.push(edge);
                }
            }
        }

        if !staged_bindings.is_empty() {
            bindings.insert(importing_module.clone(), staged_bindings);
        }
    }

    if let Some(cycle) = find_cycle(&import_edges) {
        return Err(CanonicalStructuralImportError::ImportCycle { edges: cycle });
    }

    Ok(CanonicalResolvedSimpleImports {
        bindings,
        import_edges: import_edges.into_boxed_slice(),
        graph_root: graph.root_key().clone(),
        artifacts,
    })
}

/// Resolves inherited glob ordinary-function imports through provisional scopes.
///
/// This dedicated route accepts only `use crate::<public-child>...::*` in an
/// importer with exactly one use declaration and no local ordinary functions.
/// It stages natural-name bindings and one complete-use-span edge per public
/// ordinary function in the selected structural module. It neither selects a
/// general glob precedence policy nor grants final-interface or runtime
/// authority.
///
/// # Errors
///
/// Returns [`CanonicalStructuralImportError`] when the graph and scope
/// snapshot differ, an importer or glob path lies outside this narrow route,
/// visibility rejects a structural child or function, or staged bindings would
/// conflict. No plan is published on failure.
#[allow(
    clippy::result_large_err,
    reason = "the public diagnostic contract exposes anchored structural facts directly"
)]
pub fn resolve_scoped_glob_ordinary_function_imports_with_scopes(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<CanonicalResolvedSimpleImports, CanonicalStructuralImportError> {
    resolve_scoped_glob_imports_with_scopes(graph, scopes, ScopedGlobLocalDeclarationPolicy::Reject)
}

/// Resolves scoped glob candidates before applying local-over-glob precedence.
///
/// This accepts the same narrow glob route as
/// [`resolve_scoped_glob_ordinary_function_imports_with_scopes`], but retains
/// every candidate's facts in cross-module edges for importer-local ordinary-
/// function names. Its returned binding projection applies the local-name
/// shadowing rule only after this resolver has performed cycle detection.
///
/// # Errors
///
/// Returns [`CanonicalStructuralImportError`] when the graph and scope
/// snapshot differ, an importer or glob path lies outside this narrow route,
/// visibility rejects a structural child or function, or selected edges close
/// an import cycle. No plan is published on failure.
#[allow(
    clippy::result_large_err,
    reason = "the public diagnostic contract exposes anchored structural facts directly"
)]
pub fn resolve_scoped_glob_local_precedence_imports_with_scopes(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<CanonicalResolvedSimpleImports, CanonicalStructuralImportError> {
    resolve_scoped_glob_imports_with_scopes(graph, scopes, ScopedGlobLocalDeclarationPolicy::Permit)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopedGlobLocalDeclarationPolicy {
    Reject,
    Permit,
}

#[allow(
    clippy::result_large_err,
    reason = "the public diagnostic contract exposes anchored structural facts directly"
)]
fn resolve_scoped_glob_imports_with_scopes(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
    local_declaration_policy: ScopedGlobLocalDeclarationPolicy,
) -> Result<CanonicalResolvedSimpleImports, CanonicalStructuralImportError> {
    if !scopes.matches_graph(graph) {
        return Err(CanonicalStructuralImportError::ScopeGraphMismatch);
    }

    let artifacts = graph
        .module_units()
        .map(|(key, unit)| (key.clone(), unit.artifact().clone()))
        .collect();
    let mut bindings = BTreeMap::new();
    let mut import_edges = Vec::new();

    for (importing_module, unit) in graph.module_units() {
        let uses = unit.body().uses();
        if uses.is_empty() {
            continue;
        }
        if uses.len() != 1 {
            return Err(CanonicalStructuralImportError::Unsupported {
                span: uses[1].span.into(),
                reason: "a scoped glob importer requires exactly one use declaration",
            });
        }
        let use_declaration = &uses[0];
        if matches!(
            local_declaration_policy,
            ScopedGlobLocalDeclarationPolicy::Reject
        ) && unit
            .body()
            .definitions()
            .iter()
            .any(|definition| matches!(definition, Definition::Function(_)))
        {
            return Err(CanonicalStructuralImportError::Unsupported {
                span: use_declaration.span.into(),
                reason: "a scoped glob importer cannot declare local ordinary functions",
            });
        }

        let candidates =
            resolve_scoped_glob_candidates(graph, scopes, importing_module, use_declaration)?;
        let mut staged_bindings = BTreeMap::new();
        for (local_name, binding, edge) in candidates {
            if staged_bindings
                .insert(local_name.clone(), binding)
                .is_some()
            {
                return Err(CanonicalStructuralImportError::DuplicateBinding {
                    importing_module: importing_module.clone().into(),
                    name: local_name.into(),
                    use_span: use_declaration.span.into(),
                });
            }
            import_edges.push(edge);
        }
        bindings.insert(importing_module.clone(), staged_bindings);
    }

    if let Some(cycle) = find_cycle(&import_edges) {
        return Err(CanonicalStructuralImportError::ImportCycle { edges: cycle });
    }

    let bindings = match local_declaration_policy {
        ScopedGlobLocalDeclarationPolicy::Reject => bindings,
        ScopedGlobLocalDeclarationPolicy::Permit => {
            filter_local_shadowed_glob_bindings(bindings, scopes)?
        }
    };

    Ok(CanonicalResolvedSimpleImports {
        bindings,
        import_edges: import_edges.into_boxed_slice(),
        graph_root: graph.root_key().clone(),
        artifacts,
    })
}

fn filter_local_shadowed_glob_bindings(
    bindings: BTreeMap<ModuleKey, BTreeMap<Box<str>, CanonicalBoundModuleBinding>>,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<
    BTreeMap<ModuleKey, BTreeMap<Box<str>, CanonicalBoundModuleBinding>>,
    CanonicalStructuralImportError,
> {
    let mut filtered_bindings = BTreeMap::new();
    for (module_key, module_bindings) in bindings {
        let scope = scopes
            .scope(&module_key)
            .ok_or(CanonicalStructuralImportError::ScopeGraphMismatch)?;
        let projected: BTreeMap<Box<str>, CanonicalBoundModuleBinding> = module_bindings
            .into_iter()
            .filter(|(name, _)| scope.function(name.as_ref()).is_none())
            .collect();
        if !projected.is_empty() {
            filtered_bindings.insert(module_key, projected);
        }
    }
    Ok(filtered_bindings)
}

#[allow(
    clippy::result_large_err,
    reason = "the route returns anchored structural diagnostics without erasing their facts"
)]
fn resolve_scoped_glob_candidates(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
    importing_module: &ModuleKey,
    use_declaration: &Use,
) -> Result<
    Vec<(
        Box<str>,
        CanonicalBoundModuleBinding,
        CanonicalSimpleImportEdge,
    )>,
    CanonicalStructuralImportError,
> {
    if !matches!(&use_declaration.visibility, Visibility::Inherited) {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "public and restricted uses are outside the scoped glob route",
        });
    }
    if use_declaration.alias.is_some() {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a scoped glob import cannot carry an alias",
        });
    }
    let UsePath::Glob(path) = &use_declaration.path else {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "only glob crate structural paths are accepted",
        });
    };
    let attempted_path = path.segments.clone();
    let Some((head, child_segments)) = attempted_path.split_first() else {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a scoped glob import requires a crate head and structural child",
        });
    };
    if head.as_ref() != "crate" {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "only crate-root scoped glob imports are accepted",
        });
    }
    if child_segments.is_empty() {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a scoped glob import requires a public structural child",
        });
    }
    if child_segments
        .iter()
        .any(|segment| matches!(segment.as_ref(), "crate" | "self" | "super"))
    {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a scoped glob import accepts only structural child segments after crate",
        });
    }

    let mut defining_module = graph.root_key().clone();
    for segment in child_segments {
        let scope = scopes
            .scope(&defining_module)
            .ok_or(CanonicalStructuralImportError::ScopeGraphMismatch)?;
        let child = scope.child(segment.as_ref()).ok_or_else(|| {
            if scope.function(segment.as_ref()).is_some()
                || contains_non_function_target(graph, &defining_module, segment.as_ref())
            {
                CanonicalStructuralImportError::Unsupported {
                    span: use_declaration.span.into(),
                    reason: "a scoped glob target must be a structural module",
                }
            } else {
                CanonicalStructuralImportError::Unresolved {
                    use_span: use_declaration.span.into(),
                    attempted_path: attempted_path.clone().into(),
                }
            }
        })?;
        if graph
            .module_unit(child.module_key())
            .is_none_or(|unit| unit.artifact().origin() != child.origin())
        {
            return Err(CanonicalStructuralImportError::ScopeGraphMismatch);
        }
        if !scopes.is_visible_from(child.visibility(), child.module_key(), importing_module)?
            || !matches!(child.visibility(), Visibility::Public)
        {
            return Err(CanonicalStructuralImportError::Inaccessible {
                declaration_span: child.declaration_span().into(),
                use_span: use_declaration.span.into(),
                defining_module: child.module_key().clone().into(),
                attempted_path: attempted_path.clone().into(),
                violated_visibility: child.visibility().clone().into(),
            });
        }
        defining_module = child.module_key().clone();
    }

    let defining_scope = scopes
        .scope(&defining_module)
        .ok_or(CanonicalStructuralImportError::ScopeGraphMismatch)?;
    let defining_unit = graph
        .module_unit(&defining_module)
        .ok_or(CanonicalStructuralImportError::ScopeGraphMismatch)?;
    if !defining_unit.body().module_decls().is_empty()
        || defining_unit
            .body()
            .definitions()
            .iter()
            .any(|definition| !matches!(definition, Definition::Function(_)))
    {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a scoped glob target may contain only ordinary functions",
        });
    }
    if defining_unit.body().definitions().is_empty() {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a scoped glob target requires at least one ordinary function",
        });
    }

    let mut candidates = Vec::with_capacity(defining_unit.body().definitions().len());
    for definition in defining_unit.body().definitions() {
        let Definition::Function(parsed_function) = definition else {
            return Err(CanonicalStructuralImportError::ScopeGraphMismatch);
        };
        let function = defining_scope
            .function(parsed_function.name.as_ref())
            .ok_or(CanonicalStructuralImportError::ScopeGraphMismatch)?;
        if !scopes.is_visible_from(
            function.visibility(),
            function.module_key(),
            importing_module,
        )? || !matches!(function.visibility(), Visibility::Public)
        {
            return Err(CanonicalStructuralImportError::Inaccessible {
                declaration_span: function.declaration_span().into(),
                use_span: use_declaration.span.into(),
                defining_module: function.module_key().clone().into(),
                attempted_path: attempted_path.clone().into(),
                violated_visibility: function.visibility().clone().into(),
            });
        }
        let identity = CanonicalDefinitionIdentity {
            module_key: function.module_key().clone(),
            name: function.name().into(),
        };
        let binding = CanonicalBoundModuleBinding {
            defining_identity: identity.clone(),
            declaration_span: function.declaration_span(),
            origin: function.origin().clone(),
            visibility: function.visibility().clone(),
        };
        let edge = CanonicalSimpleImportEdge {
            importing_module: importing_module.clone(),
            defining_module: function.module_key().clone(),
            defining_identity: identity,
            local_name: function.name().into(),
            use_span: use_declaration.span,
            declaration_span: function.declaration_span(),
            origin: function.origin().clone(),
            visibility: function.visibility().clone(),
        };
        candidates.push((function.name().into(), binding, edge));
    }
    Ok(candidates)
}

struct ResolvedScopedGroupedMember {
    local_name: Box<str>,
    binding: CanonicalBoundModuleBinding,
    edge: Option<CanonicalSimpleImportEdge>,
    use_span: Span,
}

#[allow(
    clippy::result_large_err,
    reason = "the route returns anchored structural diagnostics without erasing their facts"
)]
fn resolve_scoped_grouped_candidates(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
    importing_module: &ModuleKey,
    importing_scope: &crate::canonical_provisional_module_scopes::CanonicalProvisionalModuleScope,
    use_declaration: &Use,
) -> Result<Vec<ResolvedScopedGroupedMember>, CanonicalStructuralImportError> {
    if !matches!(&use_declaration.visibility, Visibility::Inherited) {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "public and restricted uses are outside the grouped structural route",
        });
    }
    if use_declaration.alias.is_some() {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a grouped structural import may alias only individual members",
        });
    }
    let UsePath::Nested(base, members) = &use_declaration.path else {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "only grouped crate structural paths are accepted",
        });
    };
    let Some(first_member) = members.first() else {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a grouped structural import requires at least one ordinary function member",
        });
    };
    let Some((head, child_segments)) = base.segments.split_first() else {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a grouped structural import requires a crate head",
        });
    };
    if head.as_ref() != "crate" {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "only crate-root grouped structural imports are accepted",
        });
    }

    let mut first_attempted_path = base.segments.clone();
    first_attempted_path.push(first_member.name.clone());
    let member_anchor = first_member.span;
    let mut defining_module = graph.root_key().clone();
    let mut first_nonpublic_child = None;
    for segment in child_segments {
        let scope = scopes
            .scope(&defining_module)
            .ok_or(CanonicalStructuralImportError::ScopeGraphMismatch)?;
        let child = scope.child(segment.as_ref()).ok_or_else(|| {
            CanonicalStructuralImportError::Unresolved {
                use_span: member_anchor.into(),
                attempted_path: first_attempted_path.clone().into(),
            }
        })?;
        if graph
            .module_unit(child.module_key())
            .is_none_or(|unit| unit.artifact().origin() != child.origin())
        {
            return Err(CanonicalStructuralImportError::ScopeGraphMismatch);
        }
        if !scopes.is_visible_from(child.visibility(), child.module_key(), importing_module)? {
            return Err(CanonicalStructuralImportError::Inaccessible {
                declaration_span: child.declaration_span().into(),
                use_span: member_anchor.into(),
                defining_module: child.module_key().clone().into(),
                attempted_path: first_attempted_path.clone().into(),
                violated_visibility: child.visibility().clone().into(),
            });
        }
        if !matches!(child.visibility(), Visibility::Public) && first_nonpublic_child.is_none() {
            first_nonpublic_child = Some((
                child.declaration_span(),
                child.module_key().clone(),
                child.visibility().clone(),
            ));
        }
        defining_module = child.module_key().clone();
    }

    let defining_scope = scopes
        .scope(&defining_module)
        .ok_or(CanonicalStructuralImportError::ScopeGraphMismatch)?;
    let member_resolver = ScopedGroupedMemberResolver {
        graph,
        scopes,
        importing_module,
        importing_scope,
        defining_scope,
        defining_module: &defining_module,
        first_nonpublic_child: &first_nonpublic_child,
        base,
    };
    let mut resolved_members = Vec::with_capacity(members.len());
    for member in members {
        resolved_members.push(member_resolver.resolve(member)?);
    }
    Ok(resolved_members)
}

#[allow(
    clippy::result_large_err,
    reason = "the route returns anchored structural diagnostics without erasing their facts"
)]
fn resolve_scoped_super_grouped_candidates(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
    importing_module: &ModuleKey,
    importing_scope: &crate::canonical_provisional_module_scopes::CanonicalProvisionalModuleScope,
    use_declaration: &Use,
) -> Result<Vec<ResolvedScopedGroupedMember>, CanonicalStructuralImportError> {
    let UsePath::Nested(base, members) = &use_declaration.path else {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "only grouped super paths are accepted",
        });
    };
    let Some(first_member) = members.first() else {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a scoped super grouped import requires at least one ordinary function member",
        });
    };
    let member_anchor = first_member.span;
    if !matches!(&use_declaration.visibility, Visibility::Inherited) {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: member_anchor.into(),
            reason: "public and restricted uses are outside the scoped super grouped route",
        });
    }
    if use_declaration.alias.is_some() {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: member_anchor.into(),
            reason: "a scoped super grouped import may alias only individual members",
        });
    }
    let Some((head, child_segments)) = base.segments.split_first() else {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: member_anchor.into(),
            reason: "a scoped super grouped import requires one super head",
        });
    };
    if head.as_ref() != "super" {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: member_anchor.into(),
            reason: "only one leading super path is accepted",
        });
    }
    if child_segments
        .iter()
        .any(|segment| matches!(segment.as_ref(), "crate" | "self" | "super"))
    {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: member_anchor.into(),
            reason: "a scoped super grouped import accepts exactly one leading super",
        });
    }
    if let Some(member) = members
        .iter()
        .find(|member| member.name.as_ref() == "super")
    {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: member.span.into(),
            reason: "a scoped super grouped import accepts exactly one leading super",
        });
    }
    let Some(mut defining_module) = importing_module.parent() else {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: member_anchor.into(),
            reason: "a scoped super grouped import requires a non-root importing module",
        });
    };

    let mut first_attempted_path = base.segments.clone();
    first_attempted_path.push(first_member.name.clone());
    let mut first_nonpublic_child = None;
    for segment in child_segments {
        let scope = scopes
            .scope(&defining_module)
            .ok_or(CanonicalStructuralImportError::ScopeGraphMismatch)?;
        let child = scope.child(segment.as_ref()).ok_or_else(|| {
            CanonicalStructuralImportError::Unresolved {
                use_span: member_anchor.into(),
                attempted_path: first_attempted_path.clone().into(),
            }
        })?;
        if graph
            .module_unit(child.module_key())
            .is_none_or(|unit| unit.artifact().origin() != child.origin())
        {
            return Err(CanonicalStructuralImportError::ScopeGraphMismatch);
        }
        if !scopes.is_visible_from(child.visibility(), child.module_key(), importing_module)? {
            return Err(CanonicalStructuralImportError::Inaccessible {
                declaration_span: child.declaration_span().into(),
                use_span: member_anchor.into(),
                defining_module: child.module_key().clone().into(),
                attempted_path: first_attempted_path.clone().into(),
                violated_visibility: child.visibility().clone().into(),
            });
        }
        if !matches!(child.visibility(), Visibility::Public) && first_nonpublic_child.is_none() {
            first_nonpublic_child = Some((
                child.declaration_span(),
                child.module_key().clone(),
                child.visibility().clone(),
            ));
        }
        defining_module = child.module_key().clone();
    }

    let defining_scope = scopes
        .scope(&defining_module)
        .ok_or(CanonicalStructuralImportError::ScopeGraphMismatch)?;
    let member_resolver = ScopedGroupedMemberResolver {
        graph,
        scopes,
        importing_module,
        importing_scope,
        defining_scope,
        defining_module: &defining_module,
        first_nonpublic_child: &first_nonpublic_child,
        base,
    };
    let mut resolved_members = Vec::with_capacity(members.len());
    for member in members {
        resolved_members.push(member_resolver.resolve(member)?);
    }
    Ok(resolved_members)
}

struct ScopedGroupedMemberResolver<'a> {
    graph: &'a CanonicalModuleGraph,
    scopes: &'a CanonicalProvisionalModuleScopes,
    importing_module: &'a ModuleKey,
    importing_scope:
        &'a crate::canonical_provisional_module_scopes::CanonicalProvisionalModuleScope,
    defining_scope: &'a crate::canonical_provisional_module_scopes::CanonicalProvisionalModuleScope,
    defining_module: &'a ModuleKey,
    first_nonpublic_child: &'a Option<(Span, ModuleKey, Visibility)>,
    base: &'a ash_parser::use_tree::SimplePath,
}

impl ScopedGroupedMemberResolver<'_> {
    #[allow(
        clippy::result_large_err,
        reason = "the route returns anchored structural diagnostics without erasing their facts"
    )]
    fn resolve(
        &self,
        member: &UseItem,
    ) -> Result<ResolvedScopedGroupedMember, CanonicalStructuralImportError> {
        let mut attempted_path = self.base.segments.clone();
        attempted_path.push(member.name.clone());
        let local_name = member.alias.clone().unwrap_or_else(|| member.name.clone());
        let function = if let Some(function) = self.defining_scope.function(member.name.as_ref()) {
            function
        } else if contains_grouped_non_function_target(
            self.graph,
            self.defining_module,
            member.name.as_ref(),
        ) {
            return Err(CanonicalStructuralImportError::Unsupported {
                span: member.span.into(),
                reason: "only ordinary function targets are accepted",
            });
        } else {
            return Err(CanonicalStructuralImportError::Unresolved {
                use_span: member.span.into(),
                attempted_path: attempted_path.into(),
            });
        };
        if !self.scopes.is_visible_from(
            function.visibility(),
            function.module_key(),
            self.importing_module,
        )? {
            return Err(CanonicalStructuralImportError::Inaccessible {
                declaration_span: function.declaration_span().into(),
                use_span: member.span.into(),
                defining_module: function.module_key().clone().into(),
                attempted_path: attempted_path.into(),
                violated_visibility: function.visibility().clone().into(),
            });
        }
        if matches!(function.visibility(), Visibility::Public)
            && let Some((declaration_span, nonpublic_module, violated_visibility)) =
                self.first_nonpublic_child
        {
            return Err(CanonicalStructuralImportError::Inaccessible {
                declaration_span: (*declaration_span).into(),
                use_span: member.span.into(),
                defining_module: nonpublic_module.clone().into(),
                attempted_path: attempted_path.into(),
                violated_visibility: violated_visibility.clone().into(),
            });
        }
        if let Some(local) = self.importing_scope.function(local_name.as_ref()) {
            return Err(CanonicalStructuralImportError::LocalDeclarationCollision {
                importing_module: self.importing_module.clone().into(),
                name: local_name.to_string(),
                declaration_span: local.declaration_span().into(),
                use_span: member.span.into(),
            });
        }

        let identity = CanonicalDefinitionIdentity {
            module_key: function.module_key().clone(),
            name: function.name().into(),
        };
        let binding = CanonicalBoundModuleBinding {
            defining_identity: identity.clone(),
            declaration_span: function.declaration_span(),
            origin: function.origin().clone(),
            visibility: function.visibility().clone(),
        };
        let edge =
            (self.importing_module != function.module_key()).then(|| CanonicalSimpleImportEdge {
                importing_module: self.importing_module.clone(),
                defining_module: function.module_key().clone(),
                defining_identity: identity,
                local_name: local_name.clone(),
                use_span: member.span,
                declaration_span: function.declaration_span(),
                origin: function.origin().clone(),
                visibility: function.visibility().clone(),
            });
        Ok(ResolvedScopedGroupedMember {
            local_name,
            binding,
            edge,
            use_span: member.span,
        })
    }
}

#[allow(
    clippy::result_large_err,
    reason = "the scoped routes retain anchored structural diagnostics without erasing their facts"
)]
fn resolve_scoped_ordinary_function_imports(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
    alias_mode: ScopedOrdinaryFunctionAliasMode,
) -> Result<CanonicalResolvedSimpleImports, CanonicalStructuralImportError> {
    if !scopes.matches_graph(graph) {
        return Err(CanonicalStructuralImportError::ScopeGraphMismatch);
    }

    let artifacts = graph
        .module_units()
        .map(|(key, unit)| (key.clone(), unit.artifact().clone()))
        .collect();
    let mut bindings = BTreeMap::new();
    let mut import_edges = Vec::new();

    for (importing_module, unit) in graph.module_units() {
        let importing_scope = scopes
            .scope(importing_module)
            .ok_or(CanonicalStructuralImportError::ScopeGraphMismatch)?;
        let mut staged_bindings = BTreeMap::new();
        for use_declaration in unit.body().uses() {
            let (local_name, binding, edge) = resolve_scoped_candidate(
                graph,
                scopes,
                importing_module,
                importing_scope,
                use_declaration,
                alias_mode,
            )?;
            if staged_bindings
                .insert(local_name.clone(), binding)
                .is_some()
            {
                return Err(CanonicalStructuralImportError::DuplicateBinding {
                    importing_module: importing_module.clone().into(),
                    name: local_name.into(),
                    use_span: use_declaration.span.into(),
                });
            }
            if let Some(edge) = edge {
                import_edges.push(edge);
            }
        }
        if !staged_bindings.is_empty() {
            bindings.insert(importing_module.clone(), staged_bindings);
        }
    }

    if let Some(cycle) = find_cycle(&import_edges) {
        return Err(CanonicalStructuralImportError::ImportCycle { edges: cycle });
    }

    Ok(CanonicalResolvedSimpleImports {
        bindings,
        import_edges: import_edges.into_boxed_slice(),
        graph_root: graph.root_key().clone(),
        artifacts,
    })
}

#[allow(
    clippy::result_large_err,
    reason = "the route returns anchored structural diagnostics without erasing their facts"
)]
fn resolve_scoped_candidate(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
    importing_module: &ModuleKey,
    importing_scope: &crate::canonical_provisional_module_scopes::CanonicalProvisionalModuleScope,
    use_declaration: &Use,
    alias_mode: ScopedOrdinaryFunctionAliasMode,
) -> Result<
    (
        Box<str>,
        CanonicalBoundModuleBinding,
        Option<CanonicalSimpleImportEdge>,
    ),
    CanonicalStructuralImportError,
> {
    if !matches!(&use_declaration.visibility, Visibility::Inherited) {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "public and restricted uses are outside the structural alias route",
        });
    }
    let UsePath::Simple(path) = &use_declaration.path else {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "only simple crate structural paths are accepted",
        });
    };
    let attempted_path = path.segments.clone();
    let Some((head, tail)) = attempted_path.split_first() else {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a structural alias requires a crate head, child, and function",
        });
    };
    if head.as_ref() != "crate" {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "only crate-root structural aliases are accepted",
        });
    }
    let Some((function_name, child_segments)) = tail.split_last() else {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a structural alias requires a direct child and function",
        });
    };
    if child_segments.is_empty()
        && matches!(
            alias_mode,
            ScopedOrdinaryFunctionAliasMode::ExplicitStructuralAlias
        )
    {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a structural alias requires at least one canonical child",
        });
    }
    let local_name =
        match alias_mode {
            ScopedOrdinaryFunctionAliasMode::ExplicitStructuralAlias => use_declaration
                .alias
                .clone()
                .ok_or(CanonicalStructuralImportError::Unsupported {
                    span: use_declaration.span.into(),
                    reason: "an explicit local alias is required",
                })?,
            ScopedOrdinaryFunctionAliasMode::OptionalAlias => use_declaration
                .alias
                .clone()
                .unwrap_or_else(|| function_name.clone()),
        };

    let mut defining_module = graph.root_key().clone();
    let mut first_nonpublic_child = None;
    for segment in child_segments {
        let scope = scopes
            .scope(&defining_module)
            .ok_or(CanonicalStructuralImportError::ScopeGraphMismatch)?;
        let child = scope.child(segment.as_ref()).ok_or_else(|| {
            CanonicalStructuralImportError::Unresolved {
                use_span: use_declaration.span.into(),
                attempted_path: attempted_path.clone().into(),
            }
        })?;
        if graph
            .module_unit(child.module_key())
            .is_none_or(|unit| unit.artifact().origin() != child.origin())
        {
            return Err(CanonicalStructuralImportError::ScopeGraphMismatch);
        }
        if !scopes.is_visible_from(child.visibility(), child.module_key(), importing_module)? {
            return Err(CanonicalStructuralImportError::Inaccessible {
                declaration_span: child.declaration_span().into(),
                use_span: use_declaration.span.into(),
                defining_module: child.module_key().clone().into(),
                attempted_path: attempted_path.into(),
                violated_visibility: child.visibility().clone().into(),
            });
        }
        if !matches!(child.visibility(), Visibility::Public) && first_nonpublic_child.is_none() {
            first_nonpublic_child = Some((
                child.declaration_span(),
                child.module_key().clone(),
                child.visibility().clone(),
            ));
        }
        defining_module = child.module_key().clone();
    }

    let defining_scope = scopes
        .scope(&defining_module)
        .ok_or(CanonicalStructuralImportError::ScopeGraphMismatch)?;
    let function = if let Some(function) = defining_scope.function(function_name.as_ref()) {
        function
    } else if contains_non_function_target(graph, &defining_module, function_name.as_ref()) {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "only ordinary function targets are accepted",
        });
    } else {
        return Err(CanonicalStructuralImportError::Unresolved {
            use_span: use_declaration.span.into(),
            attempted_path: attempted_path.into(),
        });
    };
    if !scopes.is_visible_from(
        function.visibility(),
        function.module_key(),
        importing_module,
    )? {
        return Err(CanonicalStructuralImportError::Inaccessible {
            declaration_span: function.declaration_span().into(),
            use_span: use_declaration.span.into(),
            defining_module: function.module_key().clone().into(),
            attempted_path: attempted_path.into(),
            violated_visibility: function.visibility().clone().into(),
        });
    }
    if matches!(function.visibility(), Visibility::Public)
        && let Some((declaration_span, defining_module, violated_visibility)) =
            first_nonpublic_child
    {
        return Err(CanonicalStructuralImportError::Inaccessible {
            declaration_span: declaration_span.into(),
            use_span: use_declaration.span.into(),
            defining_module: defining_module.into(),
            attempted_path: attempted_path.into(),
            violated_visibility: violated_visibility.into(),
        });
    }
    if let Some(local) = importing_scope.function(local_name.as_ref()) {
        return Err(CanonicalStructuralImportError::LocalDeclarationCollision {
            importing_module: importing_module.clone().into(),
            name: local_name.to_string(),
            declaration_span: local.declaration_span().into(),
            use_span: use_declaration.span.into(),
        });
    }

    let identity = CanonicalDefinitionIdentity {
        module_key: function.module_key().clone(),
        name: function.name().into(),
    };
    let binding = CanonicalBoundModuleBinding {
        defining_identity: identity.clone(),
        declaration_span: function.declaration_span(),
        origin: function.origin().clone(),
        visibility: function.visibility().clone(),
    };
    let edge = (importing_module != function.module_key()).then(|| CanonicalSimpleImportEdge {
        importing_module: importing_module.clone(),
        defining_module: function.module_key().clone(),
        defining_identity: identity,
        local_name: local_name.clone(),
        use_span: use_declaration.span,
        declaration_span: function.declaration_span(),
        origin: function.origin().clone(),
        visibility: function.visibility().clone(),
    });
    Ok((local_name, binding, edge))
}

#[allow(
    clippy::result_large_err,
    reason = "the route returns anchored structural diagnostics without erasing their facts"
)]
fn resolve_scoped_super_candidate(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
    importing_module: &ModuleKey,
    importing_scope: &crate::canonical_provisional_module_scopes::CanonicalProvisionalModuleScope,
    use_declaration: &Use,
) -> Result<
    (
        Box<str>,
        CanonicalBoundModuleBinding,
        Option<CanonicalSimpleImportEdge>,
    ),
    CanonicalStructuralImportError,
> {
    if !matches!(&use_declaration.visibility, Visibility::Inherited) {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "public and restricted uses are outside the scoped super route",
        });
    }
    let UsePath::Simple(path) = &use_declaration.path else {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "only simple super paths are accepted",
        });
    };
    let attempted_path = path.segments.clone();
    let Some((head, tail)) = attempted_path.split_first() else {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a scoped super import requires a super head and function",
        });
    };
    if head.as_ref() != "super" {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "only one leading super path is accepted",
        });
    }
    let Some((function_name, child_segments)) = tail.split_last() else {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a scoped super import requires an ordinary function target",
        });
    };
    if function_name.as_ref() == "super"
        || child_segments
            .iter()
            .any(|segment| segment.as_ref() == "super")
    {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a scoped super import accepts exactly one leading super",
        });
    }
    let local_name = use_declaration
        .alias
        .clone()
        .unwrap_or_else(|| function_name.clone());
    let Some(mut defining_module) = importing_module.parent() else {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "a scoped super import requires a non-root importing module",
        });
    };

    let mut first_nonpublic_child = None;
    for segment in child_segments {
        let scope = scopes
            .scope(&defining_module)
            .ok_or(CanonicalStructuralImportError::ScopeGraphMismatch)?;
        let child = scope.child(segment.as_ref()).ok_or_else(|| {
            CanonicalStructuralImportError::Unresolved {
                use_span: use_declaration.span.into(),
                attempted_path: attempted_path.clone().into(),
            }
        })?;
        if graph
            .module_unit(child.module_key())
            .is_none_or(|unit| unit.artifact().origin() != child.origin())
        {
            return Err(CanonicalStructuralImportError::ScopeGraphMismatch);
        }
        if !scopes.is_visible_from(child.visibility(), child.module_key(), importing_module)? {
            return Err(CanonicalStructuralImportError::Inaccessible {
                declaration_span: child.declaration_span().into(),
                use_span: use_declaration.span.into(),
                defining_module: child.module_key().clone().into(),
                attempted_path: attempted_path.into(),
                violated_visibility: child.visibility().clone().into(),
            });
        }
        if !matches!(child.visibility(), Visibility::Public) && first_nonpublic_child.is_none() {
            first_nonpublic_child = Some((
                child.declaration_span(),
                child.module_key().clone(),
                child.visibility().clone(),
            ));
        }
        defining_module = child.module_key().clone();
    }

    let defining_scope = scopes
        .scope(&defining_module)
        .ok_or(CanonicalStructuralImportError::ScopeGraphMismatch)?;
    let function = if let Some(function) = defining_scope.function(function_name.as_ref()) {
        function
    } else if contains_super_non_function_target(graph, &defining_module, function_name.as_ref()) {
        return Err(CanonicalStructuralImportError::Unsupported {
            span: use_declaration.span.into(),
            reason: "only ordinary function targets are accepted",
        });
    } else {
        return Err(CanonicalStructuralImportError::Unresolved {
            use_span: use_declaration.span.into(),
            attempted_path: attempted_path.into(),
        });
    };
    if !scopes.is_visible_from(
        function.visibility(),
        function.module_key(),
        importing_module,
    )? {
        return Err(CanonicalStructuralImportError::Inaccessible {
            declaration_span: function.declaration_span().into(),
            use_span: use_declaration.span.into(),
            defining_module: function.module_key().clone().into(),
            attempted_path: attempted_path.into(),
            violated_visibility: function.visibility().clone().into(),
        });
    }
    if matches!(function.visibility(), Visibility::Public)
        && let Some((declaration_span, nonpublic_module, violated_visibility)) =
            first_nonpublic_child
    {
        return Err(CanonicalStructuralImportError::Inaccessible {
            declaration_span: declaration_span.into(),
            use_span: use_declaration.span.into(),
            defining_module: nonpublic_module.into(),
            attempted_path: attempted_path.into(),
            violated_visibility: violated_visibility.into(),
        });
    }
    if let Some(local) = importing_scope.function(local_name.as_ref()) {
        return Err(CanonicalStructuralImportError::LocalDeclarationCollision {
            importing_module: importing_module.clone().into(),
            name: local_name.to_string(),
            declaration_span: local.declaration_span().into(),
            use_span: use_declaration.span.into(),
        });
    }

    let identity = CanonicalDefinitionIdentity {
        module_key: function.module_key().clone(),
        name: function.name().into(),
    };
    let binding = CanonicalBoundModuleBinding {
        defining_identity: identity.clone(),
        declaration_span: function.declaration_span(),
        origin: function.origin().clone(),
        visibility: function.visibility().clone(),
    };
    let edge = (importing_module != function.module_key()).then(|| CanonicalSimpleImportEdge {
        importing_module: importing_module.clone(),
        defining_module: function.module_key().clone(),
        defining_identity: identity,
        local_name: local_name.clone(),
        use_span: use_declaration.span,
        declaration_span: function.declaration_span(),
        origin: function.origin().clone(),
        visibility: function.visibility().clone(),
    });
    Ok((local_name, binding, edge))
}

fn contains_super_non_function_target(
    graph: &CanonicalModuleGraph,
    module_key: &ModuleKey,
    name: &str,
) -> bool {
    graph.module_unit(module_key).is_some_and(|unit| {
        unit.body()
            .module_decls()
            .iter()
            .any(|child| child.name.as_ref() == name)
    }) || contains_non_function_target(graph, module_key, name)
}

fn contains_non_function_target(
    graph: &CanonicalModuleGraph,
    module_key: &ModuleKey,
    name: &str,
) -> bool {
    graph.module_unit(module_key).is_some_and(|unit| {
        unit.body()
            .definitions()
            .iter()
            .any(|definition| match definition {
                Definition::Type(definition) => definition.name.as_ref() == name,
                Definition::Macro(definition) => definition.name.as_ref() == name,
                Definition::Newtype(definition) => definition.name.as_ref() == name,
                Definition::EffectAlias(definition) => definition.name.as_ref() == name,
                Definition::EffectGroup(definition) => definition.name.as_ref() == name,
                Definition::DataKind(definition) => definition.name.as_ref() == name,
                Definition::TypeFn(definition) => definition.name.as_ref() == name,
                Definition::PropositionPredicate(definition) => definition.name.as_ref() == name,
                Definition::Capability(definition) => definition.name.as_ref() == name,
                Definition::ResourceType(definition) => definition.name.as_ref() == name,
                Definition::Policy(definition) => definition.name.as_ref() == name,
                Definition::Role(definition) => definition.name.as_ref() == name,
                Definition::Interface(definition) => definition.name.as_ref() == name,
                Definition::Handler(definition) => definition.name.as_ref() == name,
                Definition::BuiltinFn(definition) => definition.name.as_ref() == name,
                Definition::SealedDomain(definition) => definition.name.as_ref() == name,
                Definition::Law(definition) => definition.name.as_ref() == name,
                Definition::Proof(definition) => definition.name.as_ref() == name,
                Definition::Notation(_) | Definition::Impl(_) | Definition::Function(_) => false,
            })
    })
}

fn contains_grouped_non_function_target(
    graph: &CanonicalModuleGraph,
    module_key: &ModuleKey,
    name: &str,
) -> bool {
    graph.module_unit(module_key).is_some_and(|unit| {
        unit.body()
            .module_decls()
            .iter()
            .any(|child| child.name.as_ref() == name)
    }) || contains_non_function_target(graph, module_key, name)
}

/// Resolves the bounded direct public primitive re-export planning fragment.
///
/// This opt-in route accepts only root-level `pub use` declarations with an
/// exact `crate::<direct-child>::<public-function>` path. It retains the same
/// immutable graph artifact snapshot and canonical binding/edge facts as the
/// generic simple-import plan, while leaving the generic route fail-closed for
/// public uses.
///
/// # Errors
///
/// Returns [`CanonicalDirectPrimitiveInterfaceImportError`] when a public
/// re-export is not rooted at a public direct child and public ordinary
/// function, or when its visible alias duplicates another re-export.
#[allow(
    clippy::result_large_err,
    reason = "the public diagnostic contract exposes anchored facts without boxing"
)]
pub fn resolve_direct_primitive_interface_imports(
    graph: &CanonicalModuleGraph,
) -> Result<CanonicalResolvedSimpleImports, CanonicalDirectPrimitiveInterfaceImportError> {
    let root_key = graph.root_key();
    let root_unit = graph.module_unit(root_key).ok_or(
        CanonicalDirectPrimitiveInterfaceImportError::Unsupported {
            span: Span::default(),
            reason: "the canonical root unit is absent",
        },
    )?;
    let artifacts = graph
        .module_units()
        .map(|(key, unit)| (key.clone(), unit.artifact().clone()))
        .collect();
    let mut root_bindings = BTreeMap::new();
    let mut import_edges = Vec::new();

    for use_declaration in root_unit.body().uses() {
        let (local_name, binding, edge) =
            resolve_direct_public_candidate(graph, root_key, root_unit, use_declaration)?;
        if root_bindings.insert(local_name.clone(), binding).is_some() {
            return Err(
                CanonicalDirectPrimitiveInterfaceImportError::DuplicateBinding {
                    root_module: root_key.clone(),
                    name: local_name,
                },
            );
        }
        import_edges.push(edge);
    }
    if import_edges.is_empty() {
        return Err(
            CanonicalDirectPrimitiveInterfaceImportError::MissingPublicReexport {
                root_module: root_key.clone(),
                span: root_unit.body().span(),
            },
        );
    }

    for (module_key, unit) in graph.module_units() {
        if module_key != root_key && !unit.body().uses().is_empty() {
            return Err(CanonicalDirectPrimitiveInterfaceImportError::Unsupported {
                span: unit.body().uses()[0].span,
                reason: "only the root may contain direct public re-exports",
            });
        }
    }

    let mut bindings = BTreeMap::new();
    if !root_bindings.is_empty() {
        bindings.insert(root_key.clone(), root_bindings);
    }
    Ok(CanonicalResolvedSimpleImports {
        bindings,
        import_edges: import_edges.into_boxed_slice(),
        graph_root: root_key.clone(),
        artifacts,
    })
}

#[allow(
    clippy::result_large_err,
    reason = "the public diagnostic contract exposes anchored facts without boxing"
)]
fn resolve_direct_public_candidate(
    graph: &CanonicalModuleGraph,
    root_key: &ModuleKey,
    root_unit: &ash_parser::ModuleUnit,
    use_declaration: &Use,
) -> Result<
    (
        Box<str>,
        CanonicalBoundModuleBinding,
        CanonicalSimpleImportEdge,
    ),
    CanonicalDirectPrimitiveInterfaceImportError,
> {
    if !matches!(&use_declaration.visibility, Visibility::Public) {
        return Err(CanonicalDirectPrimitiveInterfaceImportError::Unsupported {
            span: use_declaration.span,
            reason: "only public root re-exports are accepted",
        });
    }
    let UsePath::Simple(path) = &use_declaration.path else {
        return Err(CanonicalDirectPrimitiveInterfaceImportError::Unsupported {
            span: use_declaration.span,
            reason: "only simple direct public re-export paths are accepted",
        });
    };
    let [crate_head, child_name, function_name] = path.segments.as_slice() else {
        return Err(CanonicalDirectPrimitiveInterfaceImportError::Unsupported {
            span: use_declaration.span,
            reason: "a direct public re-export requires crate, child, and function segments",
        });
    };
    if crate_head.as_ref() != "crate" {
        return Err(CanonicalDirectPrimitiveInterfaceImportError::Unsupported {
            span: use_declaration.span,
            reason: "only crate-root direct public re-exports are accepted",
        });
    }
    let child_module = root_key.child(child_name.as_ref()).map_err(|_| {
        CanonicalDirectPrimitiveInterfaceImportError::Unsupported {
            span: use_declaration.span,
            reason: "the direct public re-export child name is not canonical",
        }
    })?;
    let child_declaration = root_unit
        .body()
        .module_decls()
        .iter()
        .find(|declaration| declaration.name == *child_name)
        .ok_or(CanonicalDirectPrimitiveInterfaceImportError::Unsupported {
            span: use_declaration.span,
            reason: "a direct public re-export requires a root child declaration",
        })?;
    if !matches!(&child_declaration.visibility, Visibility::Public) {
        return Err(
            CanonicalDirectPrimitiveInterfaceImportError::NonPublicStructuralPath {
                root_module: root_key.clone(),
                child_module,
                declaration_span: child_declaration.span,
            },
        );
    }
    if !graph
        .children(root_key)
        .is_some_and(|children| children.contains(&child_module))
    {
        return Err(CanonicalDirectPrimitiveInterfaceImportError::Unsupported {
            span: child_declaration.span,
            reason: "a direct public re-export child must be structurally acquired",
        });
    }
    let child_unit = graph.module_unit(&child_module).ok_or(
        CanonicalDirectPrimitiveInterfaceImportError::Unsupported {
            span: child_declaration.span,
            reason: "a direct public re-export child unit is absent",
        },
    )?;
    let function = child_unit
        .body()
        .definitions()
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name == *function_name => Some(function),
            _ => None,
        })
        .ok_or(CanonicalDirectPrimitiveInterfaceImportError::Unsupported {
            span: use_declaration.span,
            reason: "a direct public re-export target must be an ordinary function",
        })?;
    if !matches!(&function.visibility, Visibility::Public) {
        return Err(
            CanonicalDirectPrimitiveInterfaceImportError::PrivateTarget {
                defining_module: child_module,
                function: function.name.clone(),
                declaration_span: function.span,
                visibility: function.visibility.clone(),
            },
        );
    }

    let local_name = use_declaration.alias.clone().ok_or(
        CanonicalDirectPrimitiveInterfaceImportError::Unsupported {
            span: use_declaration.span,
            reason: "an explicit re-export alias is required",
        },
    )?;
    let identity = CanonicalDefinitionIdentity {
        module_key: child_module.clone(),
        name: function.name.clone(),
    };
    let origin = child_unit.artifact().origin().clone();
    Ok((
        local_name.clone(),
        CanonicalBoundModuleBinding {
            defining_identity: identity.clone(),
            declaration_span: function.span,
            origin: origin.clone(),
            visibility: function.visibility.clone(),
        },
        CanonicalSimpleImportEdge {
            importing_module: root_key.clone(),
            defining_module: child_module,
            defining_identity: identity,
            local_name,
            use_span: use_declaration.span,
            declaration_span: function.span,
            origin,
            visibility: function.visibility.clone(),
        },
    ))
}

/// Plans one private root client for an explicit direct public re-export.
///
/// This route is intentionally distinct from both generic imports and the
/// structural-only public fragment plan. It retains the exact artifact facts
/// through its contained direct re-export plan and admits only inherited
/// ordinary primitive root functions.
///
/// # Errors
///
/// Returns [`CanonicalDirectPrimitiveReexportRootClientPlanError`] when the
/// direct public re-export is not exact or a root definition lies outside the
/// private primitive client subset.
#[allow(
    clippy::result_large_err,
    reason = "the public diagnostic contract exposes anchored facts without boxing"
)]
pub fn resolve_direct_primitive_reexport_root_client_plan(
    graph: &CanonicalModuleGraph,
) -> Result<
    CanonicalDirectPrimitiveReexportRootClientPlan,
    CanonicalDirectPrimitiveReexportRootClientPlanError,
> {
    let direct_reexport_plan =
        resolve_direct_primitive_interface_imports(graph).map_err(|source| {
            CanonicalDirectPrimitiveReexportRootClientPlanError::DirectReexport {
                source: Box::new(source),
            }
        })?;
    let root_key = graph.root_key();
    let root_unit = graph.module_unit(root_key).ok_or(
        CanonicalDirectPrimitiveReexportRootClientPlanError::UnsupportedRootDefinition {
            root_module: root_key.clone(),
            span: Span::default(),
        },
    )?;
    if direct_reexport_plan.import_edges().len() != 1 {
        return Err(
            CanonicalDirectPrimitiveReexportRootClientPlanError::MultiplePublicReexports {
                root_module: root_key.clone(),
                span: root_unit.body().span(),
            },
        );
    }

    let mut private_root_functions = BTreeMap::new();
    for definition in root_unit.body().definitions() {
        let Definition::Function(function) = definition else {
            return Err(
                CanonicalDirectPrimitiveReexportRootClientPlanError::UnsupportedRootDefinition {
                    root_module: root_key.clone(),
                    span: root_definition_span(definition),
                },
            );
        };
        if matches!(&function.visibility, Visibility::Public) {
            return Err(
                CanonicalDirectPrimitiveReexportRootClientPlanError::PublicRootFunction {
                    root_module: root_key.clone(),
                    function: function.name.to_string(),
                    declaration_span: function.span,
                },
            );
        }
        if !matches!(&function.visibility, Visibility::Inherited) {
            return Err(
                CanonicalDirectPrimitiveReexportRootClientPlanError::UnsupportedPrivateRootFunction {
                    root_module: root_key.clone(),
                    function: function.name.to_string(),
                    declaration_span: function.span,
                    reason: "root client functions must use inherited visibility",
                },
            );
        }
        if !has_primitive_root_signature(function) {
            return Err(
                CanonicalDirectPrimitiveReexportRootClientPlanError::UnsupportedPrivateRootFunction {
                    root_module: root_key.clone(),
                    function: function.name.to_string(),
                    declaration_span: function.span,
                    reason: "root client functions require explicit primitive signatures without generics or contracts",
                },
            );
        }
        if private_root_functions
            .insert(function.name.clone(), function.span)
            .is_some()
        {
            return Err(
                CanonicalDirectPrimitiveReexportRootClientPlanError::DuplicatePrivateRootFunction {
                    root_module: root_key.clone(),
                    function: function.name.to_string(),
                    declaration_span: function.span,
                },
            );
        }
    }
    if private_root_functions.is_empty() {
        return Err(
            CanonicalDirectPrimitiveReexportRootClientPlanError::MissingPrivateRootFunction {
                root_module: root_key.clone(),
                span: root_unit.body().span(),
            },
        );
    }

    Ok(CanonicalDirectPrimitiveReexportRootClientPlan {
        direct_reexport_plan,
        private_root_functions,
    })
}

fn has_primitive_root_signature(function: &FnDef) -> bool {
    function.type_params.is_empty()
        && function.contract.is_none()
        && function.proposition_tail.is_none()
        && function
            .return_type
            .as_ref()
            .is_some_and(is_primitive_root_surface)
        && function
            .params
            .iter()
            .all(|parameter| is_primitive_root_surface(&parameter.ty))
}

fn is_primitive_root_surface(ty: &SurfaceType) -> bool {
    matches!(ty, SurfaceType::Name(name) if matches!(name.as_ref(), "Int" | "String" | "Bool" | "Float" | "Null" | "Time" | "Ref"))
}

fn root_definition_span(definition: &Definition) -> Span {
    match definition {
        Definition::Notation(definition) => definition.span,
        Definition::Macro(definition) => definition.span,
        Definition::Capability(definition) => definition.span,
        Definition::ResourceType(definition) => definition.span,
        Definition::Type(definition) => definition.span,
        Definition::Newtype(definition) => definition.span,
        Definition::EffectAlias(definition) => definition.span,
        Definition::EffectGroup(definition) => definition.span,
        Definition::DataKind(definition) => definition.span,
        Definition::TypeFn(definition) => definition.span,
        Definition::PropositionPredicate(definition) => definition.span,
        Definition::Policy(definition) => definition.span,
        Definition::Role(definition) => definition.span,
        Definition::Interface(definition) => definition.span,
        Definition::Impl(definition) => definition.span,
        Definition::Function(definition) => definition.span,
        Definition::Handler(definition) => definition.span,
        Definition::BuiltinFn(definition) => definition.span,
        Definition::SealedDomain(definition) => definition.span,
        Definition::Law(definition) => definition.span,
        Definition::Proof(definition) => definition.span,
    }
}

#[allow(
    clippy::result_large_err,
    reason = "the public diagnostic contract exposes its anchored fields without boxing"
)]
fn collect_functions(
    graph: &CanonicalModuleGraph,
) -> Result<ProvisionalDeclarations, CanonicalModuleBindError> {
    let mut declarations = BTreeMap::new();

    for (module_key, unit) in graph.module_units() {
        let mut module_declarations = BTreeMap::new();
        for definition in unit.body().definitions() {
            let Definition::Function(function) = definition else {
                continue;
            };
            let name = function.name.clone();
            let declaration = ProvisionalFunction {
                identity: CanonicalDefinitionIdentity {
                    module_key: module_key.clone(),
                    name: name.clone(),
                },
                declaration_span: function.span,
                origin: unit.artifact().origin().clone(),
                visibility: function.visibility.clone(),
            };
            if module_declarations
                .insert(name.clone(), declaration)
                .is_some()
            {
                return Err(CanonicalModuleBindError::DuplicateDeclaration {
                    module: module_key.clone(),
                    name,
                });
            }
        }
        declarations.insert(module_key.clone(), module_declarations);
    }

    Ok(declarations)
}

#[allow(
    clippy::result_large_err,
    reason = "the public diagnostic contract exposes its anchored fields without boxing"
)]
fn resolve_candidate(
    root_key: &ModuleKey,
    importing_module: &ModuleKey,
    use_declaration: &Use,
    declarations: &ProvisionalDeclarations,
) -> Result<
    (
        Box<str>,
        CanonicalBoundModuleBinding,
        Option<CanonicalSimpleImportEdge>,
    ),
    CanonicalModuleBindError,
> {
    if !matches!(&use_declaration.visibility, Visibility::Inherited) {
        return Err(CanonicalModuleBindError::Unsupported {
            span: use_declaration.span,
            reason: "parsed re-exports are outside the bounded import slice",
        });
    }
    let UsePath::Simple(path) = &use_declaration.path else {
        return Err(CanonicalModuleBindError::Unsupported {
            span: use_declaration.span,
            reason: "only simple crate-root paths are supported",
        });
    };
    let attempted_path = path.segments.clone();
    let Some((head, tail)) = attempted_path.split_first() else {
        return Err(CanonicalModuleBindError::Unsupported {
            span: use_declaration.span,
            reason: "an import path requires a crate head and a declaration name",
        });
    };
    if head.as_ref() != "crate" {
        return Err(CanonicalModuleBindError::Unsupported {
            span: use_declaration.span,
            reason: "only crate-root import paths are supported",
        });
    }
    let Some((name, module_segments)) = tail.split_last() else {
        return Err(CanonicalModuleBindError::Unsupported {
            span: use_declaration.span,
            reason: "an import path requires a declaration name",
        });
    };

    let mut defining_module = root_key.clone();
    for segment in module_segments {
        defining_module = defining_module.child(segment.as_ref()).map_err(|_| {
            CanonicalModuleBindError::Unresolved {
                attempted_path: attempted_path.clone(),
            }
        })?;
    }
    let declaration = declarations
        .get(&defining_module)
        .and_then(|module| module.get(name.as_ref()))
        .ok_or_else(|| CanonicalModuleBindError::Unresolved {
            attempted_path: attempted_path.clone(),
        })?;

    if !matches!(
        &declaration.visibility,
        Visibility::Public | Visibility::Inherited
    ) {
        return Err(CanonicalModuleBindError::Unsupported {
            span: use_declaration.span,
            reason: "restricted declaration visibility is outside the bounded import slice",
        });
    }
    if importing_module != &declaration.identity.module_key
        && !matches!(&declaration.visibility, Visibility::Public)
    {
        return Err(CanonicalModuleBindError::Inaccessible {
            declaration_span: declaration.declaration_span,
            defining_module: declaration.identity.module_key.clone(),
            attempted_path,
            violated_visibility: declaration.visibility.clone(),
        });
    }

    let local_name = use_declaration
        .alias
        .clone()
        .unwrap_or_else(|| name.clone());
    let binding = CanonicalBoundModuleBinding {
        defining_identity: declaration.identity.clone(),
        declaration_span: declaration.declaration_span,
        origin: declaration.origin.clone(),
        visibility: declaration.visibility.clone(),
    };
    let edge =
        (importing_module != &declaration.identity.module_key).then(|| CanonicalSimpleImportEdge {
            importing_module: importing_module.clone(),
            defining_module: declaration.identity.module_key.clone(),
            defining_identity: declaration.identity.clone(),
            local_name: local_name.clone(),
            use_span: use_declaration.span,
            declaration_span: declaration.declaration_span,
            origin: declaration.origin.clone(),
            visibility: declaration.visibility.clone(),
        });
    Ok((local_name, binding, edge))
}

pub(crate) fn find_cycle(edges: &[CanonicalSimpleImportEdge]) -> Option<CanonicalImportCycle> {
    let mut adjacency = BTreeMap::<ModuleKey, Vec<usize>>::new();
    for (index, edge) in edges.iter().enumerate() {
        adjacency
            .entry(edge.importing_module.clone())
            .or_default()
            .push(index);
    }

    let mut states = BTreeMap::<ModuleKey, VisitState>::new();
    let mut nodes = Vec::new();
    let mut path_edges = Vec::new();
    for node in adjacency.keys() {
        if states
            .get(node)
            .is_some_and(|state| *state != VisitState::Unvisited)
        {
            continue;
        }
        if let Some(cycle) = visit_for_cycle(
            node,
            edges,
            &adjacency,
            &mut states,
            &mut nodes,
            &mut path_edges,
        ) {
            return Some(cycle);
        }
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisitState {
    Unvisited,
    Visiting,
    Visited,
}

fn visit_for_cycle(
    node: &ModuleKey,
    edges: &[CanonicalSimpleImportEdge],
    adjacency: &BTreeMap<ModuleKey, Vec<usize>>,
    states: &mut BTreeMap<ModuleKey, VisitState>,
    nodes: &mut Vec<ModuleKey>,
    path_edges: &mut Vec<usize>,
) -> Option<CanonicalImportCycle> {
    states.insert(node.clone(), VisitState::Visiting);
    nodes.push(node.clone());

    if let Some(outgoing) = adjacency.get(node) {
        for edge_index in outgoing {
            let edge = &edges[*edge_index];
            let target = &edge.defining_module;
            match states.get(target).copied().unwrap_or(VisitState::Unvisited) {
                VisitState::Unvisited => {
                    path_edges.push(*edge_index);
                    if let Some(cycle) =
                        visit_for_cycle(target, edges, adjacency, states, nodes, path_edges)
                    {
                        return Some(cycle);
                    }
                    path_edges.pop();
                }
                VisitState::Visiting => {
                    let start = nodes
                        .iter()
                        .position(|candidate| candidate == target)
                        .expect("a visiting target is on the DFS node stack");
                    let mut cycle = path_edges[start..]
                        .iter()
                        .map(|index| edges[*index].clone())
                        .collect::<Vec<_>>();
                    cycle.push(edge.clone());
                    return Some(CanonicalImportCycle {
                        edges: cycle.into_boxed_slice(),
                    });
                }
                VisitState::Visited => {}
            }
        }
    }

    nodes.pop();
    states.insert(node.clone(), VisitState::Visited);
    None
}
