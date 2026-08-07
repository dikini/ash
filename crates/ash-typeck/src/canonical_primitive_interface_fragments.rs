//! Checked facts for the bounded direct public primitive re-export fragment.
//!
//! This pass consumes one immutable canonical parser graph and a separately
//! planned opt-in public re-export set. It publishes only the public direct
//! child facts and explicit root aliases selected by that plan.

use std::collections::{BTreeMap, BTreeSet};

use ash_core::module_graph::{ModuleArtifact, ModuleArtifactOrigin, ModuleKey};
use ash_parser::module::{ModuleDecl, ModuleUnit};
use ash_parser::surface::{Definition, Expr, FnDef, Type as SurfaceType, Visibility};
use ash_parser::{CanonicalModuleGraph, Span, Spanned, Use, UsePath};

use crate::canonical_function_interface::{
    PrimitiveFunctionUnitCheckError, check_primitive_function_unit,
};
use crate::{
    CanonicalDefinitionIdentity, CanonicalDirectPrimitiveReexportRootClientPlan,
    CanonicalResolvedSimpleImports, Type, TypeCheckError,
};

/// One explicitly public direct child retained by a primitive interface fragment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalPrimitivePublicChild {
    module_key: ModuleKey,
    declaration_span: Span,
    origin: ModuleArtifactOrigin,
    visibility: Visibility,
}

impl CanonicalPrimitivePublicChild {
    /// Returns the canonical key of the retained direct child.
    #[must_use]
    pub fn module_key(&self) -> &ModuleKey {
        &self.module_key
    }

    /// Returns the parser anchor of the root child declaration.
    #[must_use]
    pub const fn declaration_span(&self) -> Span {
        self.declaration_span
    }

    /// Returns acquisition provenance for the retained child unit.
    #[must_use]
    pub fn origin(&self) -> &ModuleArtifactOrigin {
        &self.origin
    }

    /// Returns the root declaration visibility for this child.
    #[must_use]
    pub fn visibility(&self) -> &Visibility {
        &self.visibility
    }
}

/// One explicit root-visible alias retained by a primitive interface fragment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalPrimitiveReexport {
    visible_name: Box<str>,
    defining_identity: CanonicalDefinitionIdentity,
    declaration_span: Span,
    origin: ModuleArtifactOrigin,
    signature: Type,
    use_span: Span,
    visibility: Visibility,
}

impl CanonicalPrimitiveReexport {
    /// Returns the explicit root-visible alias.
    #[must_use]
    pub fn visible_name(&self) -> &str {
        &self.visible_name
    }

    /// Returns the original checked provider declaration identity.
    #[must_use]
    pub fn defining_identity(&self) -> &CanonicalDefinitionIdentity {
        &self.defining_identity
    }

    /// Returns the parser anchor of the provider declaration.
    #[must_use]
    pub const fn declaration_span(&self) -> Span {
        self.declaration_span
    }

    /// Returns acquisition provenance for the provider declaration.
    #[must_use]
    pub fn origin(&self) -> &ModuleArtifactOrigin {
        &self.origin
    }

    /// Returns the checked primitive callable signature.
    #[must_use]
    pub fn signature(&self) -> &Type {
        &self.signature
    }

    /// Returns the parser anchor of the explicit root re-export.
    #[must_use]
    pub const fn use_span(&self) -> Span {
        self.use_span
    }

    /// Returns the root re-export visibility.
    #[must_use]
    pub fn visibility(&self) -> &Visibility {
        &self.visibility
    }
}

/// Atomically checked public-child and explicit re-export fragment facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalPrimitiveInterfaceFragments {
    root_artifact: ModuleArtifact,
    public_children: BTreeMap<Box<str>, CanonicalPrimitivePublicChild>,
    reexports: BTreeMap<Box<str>, CanonicalPrimitiveReexport>,
}

impl CanonicalPrimitiveInterfaceFragments {
    /// Returns the retained parser artifact for the root module.
    #[must_use]
    pub fn root_artifact(&self) -> &ModuleArtifact {
        &self.root_artifact
    }

    /// Returns one explicitly public direct child by its root declaration name.
    #[must_use]
    pub fn public_child(&self, name: &str) -> Option<&CanonicalPrimitivePublicChild> {
        self.public_children.get(name)
    }

    /// Returns one explicit root-visible re-export by alias.
    #[must_use]
    pub fn reexport(&self, name: &str) -> Option<&CanonicalPrimitiveReexport> {
        self.reexports.get(name)
    }
}

/// A checked local alias available only to the dedicated private root client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalDirectPrimitiveReexportLocalAliasBinding {
    local_name: Box<str>,
    use_span: Span,
    defining_identity: CanonicalDefinitionIdentity,
    declaration_span: Span,
    origin: ModuleArtifactOrigin,
    visibility: Visibility,
    signature: Type,
}

impl CanonicalDirectPrimitiveReexportLocalAliasBinding {
    /// Returns the root-local spelling selected by the explicit public re-export.
    #[must_use]
    pub fn local_name(&self) -> &str {
        &self.local_name
    }

    /// Returns the parser anchor of the explicit public re-export.
    #[must_use]
    pub const fn use_span(&self) -> Span {
        self.use_span
    }

    /// Returns the provider declaration identity preserved by this local alias.
    #[must_use]
    pub fn defining_identity(&self) -> &CanonicalDefinitionIdentity {
        &self.defining_identity
    }

    /// Returns the parser anchor of the provider declaration.
    #[must_use]
    pub const fn declaration_span(&self) -> Span {
        self.declaration_span
    }

    /// Returns acquisition provenance for the provider declaration.
    #[must_use]
    pub fn origin(&self) -> &ModuleArtifactOrigin {
        &self.origin
    }

    /// Returns the provider declaration visibility.
    #[must_use]
    pub fn visibility(&self) -> &Visibility {
        &self.visibility
    }

    /// Returns the checked provider signature bound under the local alias.
    #[must_use]
    pub fn signature(&self) -> &Type {
        &self.signature
    }
}

/// Atomically checked facts for one private root client of a direct re-export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalDirectPrimitiveReexportRootClient {
    fragments: CanonicalPrimitiveInterfaceFragments,
    private_root_functions: BTreeMap<Box<str>, crate::CanonicalCheckedFunction>,
    local_alias_bindings: BTreeMap<Box<str>, CanonicalDirectPrimitiveReexportLocalAliasBinding>,
}

impl CanonicalDirectPrimitiveReexportRootClient {
    /// Returns the retained direct public primitive fragment facts.
    #[must_use]
    pub fn fragments(&self) -> &CanonicalPrimitiveInterfaceFragments {
        &self.fragments
    }

    /// Returns one checked inherited root function by defining name.
    #[must_use]
    pub fn private_root_function(&self, name: &str) -> Option<&crate::CanonicalCheckedFunction> {
        self.private_root_functions.get(name)
    }

    /// Returns one checked local alias available to the private root client.
    #[must_use]
    pub fn local_alias_binding(
        &self,
        name: &str,
    ) -> Option<&CanonicalDirectPrimitiveReexportLocalAliasBinding> {
        self.local_alias_bindings.get(name)
    }
}

/// A root-client body diagnostic retaining both its original cause and body anchor.
#[derive(Debug)]
pub struct CanonicalDirectPrimitiveReexportRootClientBodyDiagnostic {
    source: TypeCheckError,
    body_span: Span,
}

impl CanonicalDirectPrimitiveReexportRootClientBodyDiagnostic {
    /// Returns the direct local-alias call span, or the root body span as fallback.
    #[must_use]
    pub const fn call_or_body_anchor(&self) -> Span {
        self.body_span
    }

    /// Returns the original type-checking cause without replacing it.
    #[must_use]
    pub fn cause(&self) -> &TypeCheckError {
        &self.source
    }
}

impl std::fmt::Display for CanonicalDirectPrimitiveReexportRootClientBodyDiagnostic {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(formatter)
    }
}

impl std::error::Error for CanonicalDirectPrimitiveReexportRootClientBodyDiagnostic {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// A failure while checking a private root client of a direct public re-export.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CanonicalDirectPrimitiveReexportRootClientError {
    /// The supplied dedicated plan was not derived from exactly this graph.
    #[error("direct root-client plan does not match the supplied graph")]
    PlanArtifactMismatch {},
    /// The retained direct provider facts could not be checked.
    #[error("direct root-client provider facts are invalid")]
    ProviderFacts {
        /// The underlying provider-fragment diagnostic.
        #[source]
        source: Box<CanonicalPrimitiveInterfaceError>,
    },
    /// The root no longer matches the dedicated plan's bounded declaration set.
    #[error("invalid direct root-client plan: {reason}")]
    InvalidPlan {
        /// Stable rejection reason.
        reason: &'static str,
    },
    /// A root declaration would overwrite the explicit local alias.
    #[error("root local alias {local_name:?} collides in {root_module}")]
    LocalAliasCollision {
        /// The canonical root module containing the collision.
        root_module: ModuleKey,
        /// The local alias spelling that collides.
        local_name: String,
        /// The parser anchor of the colliding root declaration.
        local_declaration_span: Span,
        /// The parser anchor of the explicit public re-export.
        use_span: Span,
    },
    /// A root body failed checking after the alias and local signatures were staged.
    #[error("direct root-client body check failed for {root_module}::{function}")]
    RootBodyCheck {
        /// The canonical root module defining the failed function.
        root_module: ModuleKey,
        /// The failed root function name.
        function: String,
        /// The parser anchor of the failed declaration.
        declaration_span: Span,
        /// The underlying type-checking diagnostic.
        #[source]
        source: Box<CanonicalDirectPrimitiveReexportRootClientBodyDiagnostic>,
    },
    /// An explicit public alias would overwrite a public root child identity.
    #[error("direct root-client alias conflicts with a public child")]
    RootVisibleChildCollision {
        /// The canonical root module containing the collision.
        root_module: ModuleKey,
        /// The colliding local alias spelling.
        local_name: String,
        /// The parser anchor of the public child declaration.
        child_declaration_span: Span,
        /// The parser anchor of the explicit public re-export.
        use_span: Span,
    },
}

/// A failure while checking direct primitive public interface fragments.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CanonicalPrimitiveInterfaceError {
    /// The supplied plan was not derived from exactly this canonical graph.
    #[error("direct primitive interface plan does not match the supplied graph")]
    PlanArtifactMismatch {},
    /// The complete module topology is outside this bounded fragment domain.
    #[error("invalid direct primitive interface topology: {reason}")]
    InvalidTopology {
        /// Stable rejection reason.
        reason: &'static str,
    },
    /// A planned edge does not still describe an explicit root public alias.
    #[error("invalid direct primitive interface plan edge: {reason}")]
    InvalidPlanEdge {
        /// Stable rejection reason.
        reason: &'static str,
    },
    /// Root declarations fall outside the exact direct re-export fragment.
    #[error("unsupported root shape for {root_module}: {reason}")]
    UnsupportedRootShape {
        /// The canonical root module containing the rejected declaration.
        root_module: ModuleKey,
        /// The parser anchor of the rejected root declaration.
        span: Span,
        /// Stable rejection reason.
        reason: &'static str,
    },
    /// A selected provider is not an ordinary direct primitive leaf.
    #[error("unsupported direct primitive provider shape for {defining_module}: {reason}")]
    UnsupportedProviderShape {
        /// The canonical key of the selected provider.
        defining_module: ModuleKey,
        /// The parser anchor of the rejected provider form.
        span: Span,
        /// Stable rejection reason.
        reason: &'static str,
    },
    /// A selected provider function is outside the primitive callable subset.
    #[error("non-primitive direct public target {function:?} in {defining_module}")]
    NonPrimitiveTarget {
        /// The canonical key of the provider defining the rejected function.
        defining_module: ModuleKey,
        /// The defining function name.
        function: Box<str>,
        /// The parser anchor of the rejected declaration.
        declaration_span: Span,
    },
    /// An explicit root-visible re-export would overwrite a public root function.
    #[error("root-visible re-export {visible_name:?} collides in {root_module}")]
    RootVisibleNameCollision {
        /// The canonical root module containing the collision.
        root_module: ModuleKey,
        /// The explicit re-export alias that collides.
        visible_name: String,
        /// The parser anchor of the colliding public root function.
        local_declaration_span: Span,
        /// The parser anchor of the colliding public re-export.
        use_span: Span,
    },
    /// An explicit root alias would overwrite a public child identity.
    #[error(
        "root-visible re-export {visible_name:?} collides with a public child in {root_module}"
    )]
    RootVisibleChildCollision {
        /// The canonical root module containing the collision.
        root_module: ModuleKey,
        /// The explicit re-export alias that collides with a child name.
        visible_name: String,
        /// The parser anchor of the colliding public child declaration.
        child_declaration_span: Span,
        /// The parser anchor of the colliding public re-export.
        use_span: Span,
    },
    /// A selected primitive provider could not be checked after shape admission.
    #[error("direct primitive provider check failed for {defining_module}::{function}")]
    ProviderCheck {
        /// The canonical key of the provider being checked.
        defining_module: ModuleKey,
        /// The defining function name associated with the check failure.
        function: Box<str>,
        /// The parser anchor of the failed declaration.
        declaration_span: Span,
        /// The underlying type-checking diagnostic.
        #[source]
        source: Box<TypeCheckError>,
    },
}

/// Checks planned direct public primitive re-exports and returns opaque facts.
///
/// The plan must match this graph exactly. Every retained non-root module is
/// a selected public direct provider leaf, and every retained root-visible
/// name arises from one explicit `pub use` declaration.
///
/// # Errors
///
/// Returns [`CanonicalPrimitiveInterfaceError`] without publishing any
/// fragment when the plan, topology, provider facts, or root-visible names
/// fall outside this bounded domain.
#[allow(
    clippy::result_large_err,
    reason = "anchored public diagnostics remain unboxed"
)]
pub fn check_direct_primitive_interface_fragments(
    graph: &CanonicalModuleGraph,
    plan: &CanonicalResolvedSimpleImports,
) -> Result<CanonicalPrimitiveInterfaceFragments, CanonicalPrimitiveInterfaceError> {
    let fragments = check_direct_primitive_reexport_provider_facts(graph, plan)?;
    let root_key = graph.root_key();
    let root_unit =
        graph
            .module_unit(root_key)
            .ok_or(CanonicalPrimitiveInterfaceError::InvalidTopology {
                reason: "root unit is absent",
            })?;
    preflight_root_visible_names(root_key, root_unit, &fragments.reexports)?;
    preflight_root_visible_child_names(root_key, root_unit, &fragments.reexports)?;
    preflight_root_shape(root_key, root_unit)?;
    Ok(fragments)
}

/// Checks one private root client against its explicit direct public alias.
///
/// The dedicated plan is checked against the complete graph before provider
/// facts, local alias facts, or private root functions are retained. The
/// result is built only after every selected declaration has completed.
///
/// # Errors
///
/// Returns [`CanonicalDirectPrimitiveReexportRootClientError`] when the plan,
/// provider facts, root namespace, or private root bodies fall outside this
/// bounded route.
#[allow(
    clippy::result_large_err,
    reason = "anchored public diagnostics remain unboxed"
)]
pub fn check_direct_primitive_reexport_root_client(
    graph: &CanonicalModuleGraph,
    plan: &CanonicalDirectPrimitiveReexportRootClientPlan,
) -> Result<
    CanonicalDirectPrimitiveReexportRootClient,
    CanonicalDirectPrimitiveReexportRootClientError,
> {
    if !plan.matches_graph(graph) {
        return Err(CanonicalDirectPrimitiveReexportRootClientError::PlanArtifactMismatch {});
    }
    let direct_reexport_plan = plan.direct_reexport_plan();
    let fragments = check_direct_primitive_reexport_provider_facts(graph, direct_reexport_plan)
        .map_err(
            |source| CanonicalDirectPrimitiveReexportRootClientError::ProviderFacts {
                source: Box::new(source),
            },
        )?;
    let root_key = graph.root_key();
    let root_unit = graph.module_unit(root_key).ok_or(
        CanonicalDirectPrimitiveReexportRootClientError::InvalidPlan {
            reason: "root unit is absent",
        },
    )?;
    let edge = direct_reexport_plan.import_edges().first().ok_or(
        CanonicalDirectPrimitiveReexportRootClientError::InvalidPlan {
            reason: "dedicated plan lacks its explicit public re-export",
        },
    )?;
    let reexport = fragments.reexport(edge.local_name()).ok_or(
        CanonicalDirectPrimitiveReexportRootClientError::InvalidPlan {
            reason: "dedicated plan target is absent from checked provider facts",
        },
    )?;
    preflight_root_child_identity(root_key, root_unit, reexport)?;
    preflight_private_root_plan(root_key, root_unit, plan, reexport)?;

    let mut imported_signatures = BTreeMap::new();
    imported_signatures.insert(edge.local_name().into(), reexport.signature().clone());
    let private_root_functions =
        check_primitive_function_unit(root_key, root_unit, &imported_signatures).map_err(
            |error| root_client_check_error(root_key, root_unit, reexport.visible_name(), error),
        )?;
    let mut local_alias_bindings = BTreeMap::new();
    local_alias_bindings.insert(
        edge.local_name().into(),
        CanonicalDirectPrimitiveReexportLocalAliasBinding {
            local_name: edge.local_name().into(),
            use_span: reexport.use_span(),
            defining_identity: reexport.defining_identity().clone(),
            declaration_span: reexport.declaration_span(),
            origin: reexport.origin().clone(),
            visibility: reexport.visibility().clone(),
            signature: reexport.signature().clone(),
        },
    );

    Ok(CanonicalDirectPrimitiveReexportRootClient {
        fragments,
        private_root_functions,
        local_alias_bindings,
    })
}

#[allow(
    clippy::result_large_err,
    reason = "anchored public diagnostics remain unboxed"
)]
fn preflight_root_child_identity(
    root_key: &ModuleKey,
    root_unit: &ModuleUnit,
    reexport: &CanonicalPrimitiveReexport,
) -> Result<(), CanonicalDirectPrimitiveReexportRootClientError> {
    for declaration in root_unit.body().module_decls() {
        if matches!(&declaration.visibility, Visibility::Public)
            && declaration.name.as_ref() == reexport.visible_name()
        {
            return Err(
                CanonicalDirectPrimitiveReexportRootClientError::RootVisibleChildCollision {
                    root_module: root_key.clone(),
                    local_name: reexport.visible_name().to_string(),
                    child_declaration_span: declaration.span,
                    use_span: reexport.use_span(),
                },
            );
        }
    }
    Ok(())
}

#[allow(
    clippy::result_large_err,
    reason = "anchored public diagnostics remain unboxed"
)]
fn preflight_private_root_plan(
    root_key: &ModuleKey,
    root_unit: &ModuleUnit,
    plan: &CanonicalDirectPrimitiveReexportRootClientPlan,
    reexport: &CanonicalPrimitiveReexport,
) -> Result<(), CanonicalDirectPrimitiveReexportRootClientError> {
    let mut actual_private_functions = BTreeMap::new();
    for definition in root_unit.body().definitions() {
        let Definition::Function(function) = definition else {
            return Err(
                CanonicalDirectPrimitiveReexportRootClientError::InvalidPlan {
                    reason: "dedicated root plan contains a non-function definition",
                },
            );
        };
        if function.name.as_ref() == reexport.visible_name() {
            return Err(
                CanonicalDirectPrimitiveReexportRootClientError::LocalAliasCollision {
                    root_module: root_key.clone(),
                    local_name: function.name.to_string(),
                    local_declaration_span: function.span,
                    use_span: reexport.use_span(),
                },
            );
        }
        if !matches!(&function.visibility, Visibility::Inherited) {
            return Err(
                CanonicalDirectPrimitiveReexportRootClientError::InvalidPlan {
                    reason: "dedicated root client functions must use inherited visibility",
                },
            );
        }
        if actual_private_functions
            .insert(function.name.clone(), function.span)
            .is_some()
        {
            return Err(
                CanonicalDirectPrimitiveReexportRootClientError::InvalidPlan {
                    reason: "dedicated root client functions must have distinct names",
                },
            );
        }
    }
    if &actual_private_functions != plan.private_root_functions() {
        return Err(
            CanonicalDirectPrimitiveReexportRootClientError::InvalidPlan {
                reason: "dedicated root plan no longer matches private root declarations",
            },
        );
    }
    Ok(())
}

fn root_client_check_error(
    root_key: &ModuleKey,
    root_unit: &ModuleUnit,
    local_alias: &str,
    error: PrimitiveFunctionUnitCheckError,
) -> CanonicalDirectPrimitiveReexportRootClientError {
    let (function, declaration_span, source) = match error {
        PrimitiveFunctionUnitCheckError::DuplicateFunction {
            function: _,
            declaration_span: _,
        } => {
            return CanonicalDirectPrimitiveReexportRootClientError::InvalidPlan {
                reason: "dedicated root client functions must have distinct names",
            };
        }
        PrimitiveFunctionUnitCheckError::Signature {
            function,
            declaration_span,
            source,
        }
        | PrimitiveFunctionUnitCheckError::BodyCheck {
            function,
            declaration_span,
            source,
        } => (function, declaration_span, source),
    };
    CanonicalDirectPrimitiveReexportRootClientError::RootBodyCheck {
        root_module: root_key.clone(),
        function: function.to_string(),
        declaration_span,
        source: Box::new(CanonicalDirectPrimitiveReexportRootClientBodyDiagnostic {
            source,
            body_span: root_function_alias_call_span(root_unit, function.as_ref(), local_alias)
                .or_else(|| root_function_body_span(root_unit, function.as_ref()))
                .unwrap_or(declaration_span),
        }),
    }
}

fn root_function_alias_call_span(
    root_unit: &ModuleUnit,
    function_name: &str,
    local_alias: &str,
) -> Option<Span> {
    let function =
        root_unit
            .body()
            .definitions()
            .iter()
            .find_map(|definition| match definition {
                Definition::Function(function) if function.name.as_ref() == function_name => {
                    Some(function)
                }
                _ => None,
            })?;
    direct_alias_call_span(&function.body, local_alias)
}

fn direct_alias_call_span(expression: &Expr, local_alias: &str) -> Option<Span> {
    match expression {
        Expr::Call {
            func,
            module: None,
            span,
            ..
        } if func.as_ref() == local_alias => Some(*span),
        Expr::Block {
            statements,
            tail_expr: Some(tail_expression),
            ..
        } if statements.is_empty() => direct_alias_call_span(tail_expression, local_alias),
        _ => None,
    }
}

fn root_function_body_span(root_unit: &ModuleUnit, name: &str) -> Option<Span> {
    root_unit
        .body()
        .definitions()
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == name => {
                Some(function.body.span())
            }
            _ => None,
        })
}

/// Checks the plan-selected direct providers without admitting root definitions.
///
/// This crate-internal helper retains provider facts for the direct fragment
/// and leaves root declaration admission to the caller's bounded route.
#[allow(
    clippy::result_large_err,
    reason = "anchored public diagnostics remain unboxed"
)]
pub(crate) fn check_direct_primitive_reexport_provider_facts(
    graph: &CanonicalModuleGraph,
    plan: &CanonicalResolvedSimpleImports,
) -> Result<CanonicalPrimitiveInterfaceFragments, CanonicalPrimitiveInterfaceError> {
    if !plan.matches_graph(graph) {
        return Err(CanonicalPrimitiveInterfaceError::PlanArtifactMismatch {});
    }
    let root_key = graph.root_key();
    let root_unit =
        graph
            .module_unit(root_key)
            .ok_or(CanonicalPrimitiveInterfaceError::InvalidTopology {
                reason: "root unit is absent",
            })?;
    if root_unit.body().uses().len() != plan.import_edges().len() {
        return Err(CanonicalPrimitiveInterfaceError::InvalidPlanEdge {
            reason: "root public re-exports must match the complete plan",
        });
    }

    let mut selected_providers = BTreeSet::new();
    for edge in plan.import_edges() {
        let use_declaration = root_unit
            .body()
            .uses()
            .iter()
            .find(|candidate| candidate.span == edge.use_span())
            .ok_or(CanonicalPrimitiveInterfaceError::InvalidPlanEdge {
                reason: "planned public re-export use is absent from the root",
            })?;
        check_direct_public_edge(root_key, root_unit, edge, use_declaration)?;
        selected_providers.insert(edge.defining_module().clone());
    }
    preflight_complete_topology(root_key, graph, &selected_providers)?;

    let mut public_children = BTreeMap::new();
    let mut reexports = BTreeMap::new();
    for provider_key in &selected_providers {
        let provider_unit = graph.module_unit(provider_key).ok_or(
            CanonicalPrimitiveInterfaceError::InvalidTopology {
                reason: "selected direct provider unit is absent",
            },
        )?;
        let declaration = direct_public_child_declaration(root_key, root_unit, provider_key)?;
        preflight_provider(provider_key, provider_unit, graph)?;
        let checked_functions =
            check_primitive_function_unit(provider_key, provider_unit, &BTreeMap::new())
                .map_err(|error| provider_check_error(provider_key, error))?;
        for edge in plan
            .import_edges()
            .iter()
            .filter(|edge| edge.defining_module() == provider_key)
        {
            let function = checked_functions
                .get(edge.defining_identity().name())
                .ok_or(CanonicalPrimitiveInterfaceError::InvalidPlanEdge {
                    reason: "planned public re-export target is not checked",
                })?;
            if !matches!(function.visibility(), Visibility::Public)
                || function.defining_identity().module_key() != provider_key
                || function.defining_identity().name() != edge.defining_identity().name()
                || function.declaration_span() != edge.declaration_span()
                || function.origin() != edge.origin()
                || !is_primitive_signature(function.signature())
            {
                return Err(CanonicalPrimitiveInterfaceError::InvalidPlanEdge {
                    reason: "planned public re-export edge no longer matches checked provider facts",
                });
            }
            if reexports.contains_key(edge.local_name()) {
                return Err(CanonicalPrimitiveInterfaceError::InvalidPlanEdge {
                    reason: "planned root-visible re-export names collide",
                });
            }
            reexports.insert(
                edge.local_name().into(),
                CanonicalPrimitiveReexport {
                    visible_name: edge.local_name().into(),
                    defining_identity: edge.defining_identity().clone(),
                    declaration_span: edge.declaration_span(),
                    origin: edge.origin().clone(),
                    signature: function.signature().clone(),
                    use_span: edge.use_span(),
                    visibility: Visibility::Public,
                },
            );
        }
        public_children.insert(
            declaration.name.clone(),
            CanonicalPrimitivePublicChild {
                module_key: provider_key.clone(),
                declaration_span: declaration.span,
                origin: provider_unit.artifact().origin().clone(),
                visibility: declaration.visibility.clone(),
            },
        );
    }
    Ok(CanonicalPrimitiveInterfaceFragments {
        root_artifact: root_unit.artifact().clone(),
        public_children,
        reexports,
    })
}

#[allow(
    clippy::result_large_err,
    reason = "anchored public diagnostics remain unboxed"
)]
fn check_direct_public_edge(
    root_key: &ModuleKey,
    root_unit: &ModuleUnit,
    edge: &crate::CanonicalSimpleImportEdge,
    use_declaration: &Use,
) -> Result<(), CanonicalPrimitiveInterfaceError> {
    if edge.importing_module() != root_key
        || edge.defining_module().parent().as_ref() != Some(root_key)
        || !matches!(&use_declaration.visibility, Visibility::Public)
    {
        return Err(CanonicalPrimitiveInterfaceError::InvalidPlanEdge {
            reason: "planned re-export must be public from the root to a direct provider",
        });
    }
    let UsePath::Simple(path) = &use_declaration.path else {
        return Err(CanonicalPrimitiveInterfaceError::InvalidPlanEdge {
            reason: "planned re-export path must remain simple",
        });
    };
    let [crate_head, child_name, function_name] = path.segments.as_slice() else {
        return Err(CanonicalPrimitiveInterfaceError::InvalidPlanEdge {
            reason: "planned re-export path must contain crate, child, and function segments",
        });
    };
    let expected_child = root_key.child(child_name.as_ref()).map_err(|_| {
        CanonicalPrimitiveInterfaceError::InvalidPlanEdge {
            reason: "planned re-export child name is not canonical",
        }
    })?;
    let visible_name = use_declaration
        .alias
        .as_deref()
        .unwrap_or(function_name.as_ref());
    if crate_head.as_ref() != "crate"
        || &expected_child != edge.defining_module()
        || function_name.as_ref() != edge.defining_identity().name()
        || visible_name != edge.local_name()
        || !matches!(&edge.visibility(), Visibility::Public)
    {
        return Err(CanonicalPrimitiveInterfaceError::InvalidPlanEdge {
            reason: "planned re-export no longer matches the explicit public root alias",
        });
    }
    let declaration = direct_public_child_declaration(root_key, root_unit, edge.defining_module())?;
    if !matches!(&declaration.visibility, Visibility::Public) {
        return Err(CanonicalPrimitiveInterfaceError::InvalidPlanEdge {
            reason: "planned re-export child is not public",
        });
    }
    Ok(())
}

#[allow(
    clippy::result_large_err,
    reason = "anchored public diagnostics remain unboxed"
)]
fn direct_public_child_declaration<'a>(
    root_key: &ModuleKey,
    root_unit: &'a ModuleUnit,
    provider_key: &ModuleKey,
) -> Result<&'a ModuleDecl, CanonicalPrimitiveInterfaceError> {
    let declaration = root_unit
        .body()
        .module_decls()
        .iter()
        .find(|candidate| {
            root_key
                .child(candidate.name.as_ref())
                .is_ok_and(|candidate_key| &candidate_key == provider_key)
        })
        .ok_or(CanonicalPrimitiveInterfaceError::InvalidTopology {
            reason: "selected provider lacks a direct root declaration",
        })?;
    if !matches!(&declaration.visibility, Visibility::Public) {
        return Err(CanonicalPrimitiveInterfaceError::InvalidTopology {
            reason: "selected direct provider declaration is not public",
        });
    }
    Ok(declaration)
}

#[allow(
    clippy::result_large_err,
    reason = "anchored public diagnostics remain unboxed"
)]
fn preflight_complete_topology(
    root_key: &ModuleKey,
    graph: &CanonicalModuleGraph,
    selected_providers: &BTreeSet<ModuleKey>,
) -> Result<(), CanonicalPrimitiveInterfaceError> {
    for (module_key, _) in graph.module_units() {
        if module_key != root_key
            && (!selected_providers.contains(module_key)
                || module_key.parent().as_ref() != Some(root_key))
        {
            return Err(CanonicalPrimitiveInterfaceError::InvalidTopology {
                reason: "only selected direct public provider leaves may accompany the root",
            });
        }
    }
    Ok(())
}

#[allow(
    clippy::result_large_err,
    reason = "anchored public diagnostics remain unboxed"
)]
fn preflight_provider(
    provider_key: &ModuleKey,
    provider_unit: &ModuleUnit,
    graph: &CanonicalModuleGraph,
) -> Result<(), CanonicalPrimitiveInterfaceError> {
    if let Some(use_declaration) = provider_unit.body().uses().first() {
        return Err(CanonicalPrimitiveInterfaceError::UnsupportedProviderShape {
            defining_module: provider_key.clone(),
            span: use_declaration.span,
            reason: "direct public providers cannot contain uses",
        });
    }
    if let Some(declaration) = provider_unit.body().module_decls().first() {
        return Err(CanonicalPrimitiveInterfaceError::UnsupportedProviderShape {
            defining_module: provider_key.clone(),
            span: declaration.span,
            reason: "direct public providers cannot contain child modules",
        });
    }
    if graph
        .children(provider_key)
        .is_some_and(|children| !children.is_empty())
    {
        return Err(CanonicalPrimitiveInterfaceError::UnsupportedProviderShape {
            defining_module: provider_key.clone(),
            span: provider_unit.body().span(),
            reason: "direct public providers must be leaves",
        });
    }
    for definition in provider_unit.body().definitions() {
        let Definition::Function(function) = definition else {
            return Err(CanonicalPrimitiveInterfaceError::UnsupportedProviderShape {
                defining_module: provider_key.clone(),
                span: provider_unit.body().span(),
                reason: "direct public providers accept only ordinary functions",
            });
        };
        if !matches!(
            &function.visibility,
            Visibility::Public | Visibility::Inherited
        ) {
            return Err(CanonicalPrimitiveInterfaceError::UnsupportedProviderShape {
                defining_module: provider_key.clone(),
                span: function.span,
                reason: "direct public providers require public or inherited ordinary functions",
            });
        }
        if !has_primitive_surface_signature(function) {
            return Err(CanonicalPrimitiveInterfaceError::NonPrimitiveTarget {
                defining_module: provider_key.clone(),
                function: function.name.clone(),
                declaration_span: function.span,
            });
        }
    }
    Ok(())
}

#[allow(
    clippy::result_large_err,
    reason = "anchored public diagnostics remain unboxed"
)]
fn preflight_root_visible_names(
    root_key: &ModuleKey,
    root_unit: &ModuleUnit,
    reexports: &BTreeMap<Box<str>, CanonicalPrimitiveReexport>,
) -> Result<(), CanonicalPrimitiveInterfaceError> {
    for definition in root_unit.body().definitions() {
        let Definition::Function(function) = definition else {
            continue;
        };
        if matches!(&function.visibility, Visibility::Public)
            && let Some(reexport) = reexports.get(function.name.as_ref())
        {
            return Err(CanonicalPrimitiveInterfaceError::RootVisibleNameCollision {
                root_module: root_key.clone(),
                visible_name: function.name.to_string(),
                local_declaration_span: function.span,
                use_span: reexport.use_span,
            });
        }
    }
    Ok(())
}

#[allow(
    clippy::result_large_err,
    reason = "anchored public diagnostics remain unboxed"
)]
fn preflight_root_visible_child_names(
    root_key: &ModuleKey,
    root_unit: &ModuleUnit,
    reexports: &BTreeMap<Box<str>, CanonicalPrimitiveReexport>,
) -> Result<(), CanonicalPrimitiveInterfaceError> {
    for declaration in root_unit.body().module_decls() {
        if matches!(&declaration.visibility, Visibility::Public)
            && let Some(reexport) = reexports.get(declaration.name.as_ref())
        {
            return Err(
                CanonicalPrimitiveInterfaceError::RootVisibleChildCollision {
                    root_module: root_key.clone(),
                    visible_name: declaration.name.to_string(),
                    child_declaration_span: declaration.span,
                    use_span: reexport.use_span,
                },
            );
        }
    }
    Ok(())
}

#[allow(
    clippy::result_large_err,
    reason = "anchored public diagnostics remain unboxed"
)]
fn preflight_root_shape(
    root_key: &ModuleKey,
    root_unit: &ModuleUnit,
) -> Result<(), CanonicalPrimitiveInterfaceError> {
    if let Some(definition) = root_unit.body().definitions().first() {
        return Err(CanonicalPrimitiveInterfaceError::UnsupportedRootShape {
            root_module: root_key.clone(),
            span: root_definition_span(definition),
            reason: "root definitions are outside the direct re-export fragment",
        });
    }
    Ok(())
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

fn has_primitive_surface_signature(function: &FnDef) -> bool {
    function.type_params.is_empty()
        && function.contract.is_none()
        && function.proposition_tail.is_none()
        && function
            .return_type
            .as_ref()
            .is_some_and(is_primitive_surface)
        && function
            .params
            .iter()
            .all(|parameter| is_primitive_surface(&parameter.ty))
}

fn is_primitive_surface(ty: &SurfaceType) -> bool {
    matches!(ty, SurfaceType::Name(name) if matches!(name.as_ref(), "Int" | "String" | "Bool" | "Float" | "Null" | "Time" | "Ref"))
}

fn is_primitive_signature(ty: &Type) -> bool {
    matches!(ty, Type::Fn(parameters, result) if parameters.iter().all(is_primitive_type) && is_primitive_type(result))
}

fn is_primitive_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Int | Type::String | Type::Bool | Type::Float | Type::Null | Type::Time | Type::Ref
    )
}

fn provider_check_error(
    provider_key: &ModuleKey,
    error: PrimitiveFunctionUnitCheckError,
) -> CanonicalPrimitiveInterfaceError {
    let (function, declaration_span, source) = match error {
        PrimitiveFunctionUnitCheckError::DuplicateFunction {
            function: _,
            declaration_span,
        } => {
            return CanonicalPrimitiveInterfaceError::UnsupportedProviderShape {
                defining_module: provider_key.clone(),
                span: declaration_span,
                reason: "direct public providers require distinct function names",
            };
        }
        PrimitiveFunctionUnitCheckError::Signature {
            function,
            declaration_span,
            source,
        }
        | PrimitiveFunctionUnitCheckError::BodyCheck {
            function,
            declaration_span,
            source,
        } => (function, declaration_span, source),
    };
    CanonicalPrimitiveInterfaceError::ProviderCheck {
        defining_module: provider_key.clone(),
        function,
        declaration_span,
        source: Box::new(source),
    }
}
