//! Checked primitive direct provider/client relationship.
//!
//! This bounded pass consumes an immutable parser graph and already-resolved
//! simple-import plan. It checks direct public providers before checking the
//! root client against the signatures selected by that plan.

use std::collections::{BTreeMap, BTreeSet};

use ash_core::module_graph::{ModuleArtifact, ModuleArtifactOrigin, ModuleKey};
use ash_parser::surface::{Definition, FnDef, Type as SurfaceType, Visibility};
use ash_parser::{CanonicalModuleGraph, Span};

use crate::canonical_function_interface::{
    PrimitiveFunctionUnitCheckError, check_primitive_function_unit,
};
use crate::{CanonicalCheckedFunction, CanonicalResolvedSimpleImports, Type, TypeCheckError};

/// Checked ordinary functions retained for one primitive provider or client module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalCheckedPrimitiveModule {
    artifact: ModuleArtifact,
    functions: BTreeMap<Box<str>, CanonicalCheckedFunction>,
}

impl CanonicalCheckedPrimitiveModule {
    /// Returns the retained parser artifact for this checked module.
    #[must_use]
    pub fn artifact(&self) -> &ModuleArtifact {
        &self.artifact
    }

    /// Returns a checked ordinary function by defining name.
    #[must_use]
    pub fn function(&self, name: &str) -> Option<&CanonicalCheckedFunction> {
        self.functions.get(name)
    }
}

/// One plan-selected primitive import used by the checked root client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalCheckedPrimitiveImportBinding {
    local_name: Box<str>,
    use_span: Span,
    defining_identity: crate::CanonicalDefinitionIdentity,
    declaration_span: Span,
    origin: ModuleArtifactOrigin,
    visibility: Visibility,
    signature: Type,
}

impl CanonicalCheckedPrimitiveImportBinding {
    /// Returns the root-local name assigned by the parsed import.
    #[must_use]
    pub fn local_name(&self) -> &str {
        &self.local_name
    }
    /// Returns the parser anchor of the parsed import.
    #[must_use]
    pub const fn use_span(&self) -> Span {
        self.use_span
    }
    /// Returns the checked provider declaration identity.
    #[must_use]
    pub fn defining_identity(&self) -> &crate::CanonicalDefinitionIdentity {
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
    /// Returns the checked primitive provider signature.
    #[must_use]
    pub fn signature(&self) -> &Type {
        &self.signature
    }
}

/// Atomically checked direct primitive providers and their root client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalCheckedPrimitiveProviderClient {
    root: CanonicalCheckedPrimitiveModule,
    providers: BTreeMap<ModuleKey, CanonicalCheckedPrimitiveModule>,
    imports: BTreeMap<Box<str>, CanonicalCheckedPrimitiveImportBinding>,
}

impl CanonicalCheckedPrimitiveProviderClient {
    /// Returns the sole checked root client module.
    #[must_use]
    pub fn root_module(&self) -> &CanonicalCheckedPrimitiveModule {
        &self.root
    }
    /// Returns one direct checked provider by canonical key.
    #[must_use]
    pub fn provider_module(&self, key: &ModuleKey) -> Option<&CanonicalCheckedPrimitiveModule> {
        self.providers.get(key)
    }
    /// Returns one checked root import by local name.
    #[must_use]
    pub fn import_binding(&self, name: &str) -> Option<&CanonicalCheckedPrimitiveImportBinding> {
        self.imports.get(name)
    }
    /// Returns all checked root import bindings in deterministic local-name order.
    #[must_use]
    pub fn import_bindings(&self) -> &BTreeMap<Box<str>, CanonicalCheckedPrimitiveImportBinding> {
        &self.imports
    }
}

/// A failure while checking the primitive direct provider/client slice.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CanonicalPrimitiveProviderClientError {
    /// The plan was not derived from exactly this graph's root and artifacts.
    #[error("resolved import plan does not match the supplied canonical graph")]
    PlannerArtifactMismatch {},
    /// Graph topology is outside the direct provider/client domain.
    #[error("invalid primitive provider/client topology: {reason}")]
    InvalidTopology {
        /// Stable rejection reason.
        reason: &'static str,
    },
    /// A provider unit has uses, children, or unsupported definitions.
    #[error("unsupported provider shape for {provider_module}: {reason}")]
    UnsupportedProviderShape {
        provider_module: ModuleKey,
        span: Span,
        reason: &'static str,
    },
    /// A provider signature is outside the primitive callable subset.
    #[error("provider signature outside primitive slice for {provider_module}::{function}")]
    ProviderSignatureOutsidePrimitiveSlice {
        provider_module: ModuleKey,
        function: String,
        declaration_span: Span,
    },
    /// A root-local function would overwrite an imported binding.
    #[error("root local {local_name:?} collides with a planned import")]
    LocalImportCollision {
        root_module: ModuleKey,
        local_name: String,
        local_declaration_span: Span,
        use_span: Span,
    },
    /// A plan edge cannot be revalidated against a checked direct provider.
    #[error("invalid primitive provider edge: {reason}")]
    InvalidPlanEdge { reason: &'static str },
    /// A provider body failed checking after its signatures were staged.
    #[error("provider body check failed for {provider_module}::{function}")]
    ProviderBodyCheck {
        provider_module: ModuleKey,
        function: String,
        declaration_span: Span,
        #[source]
        source: Box<TypeCheckError>,
    },
    /// A root client body failed checking after imports and local signatures were staged.
    #[error("client body check failed for {root_module}::{function}")]
    ClientBodyCheck {
        root_module: ModuleKey,
        function: String,
        declaration_span: Span,
        #[source]
        source: Box<TypeCheckError>,
    },
}

/// Checks one root client against direct primitive providers selected by `plan`.
///
/// # Errors
///
/// Returns [`CanonicalPrimitiveProviderClientError`] without publishing any
/// partial result when graph, plan, topology, signatures, or bodies fail.
#[allow(
    clippy::result_large_err,
    reason = "anchored public diagnostics remain unboxed"
)]
pub fn check_primitive_provider_client(
    graph: &CanonicalModuleGraph,
    plan: &CanonicalResolvedSimpleImports,
) -> Result<CanonicalCheckedPrimitiveProviderClient, CanonicalPrimitiveProviderClientError> {
    if !plan.matches_graph(graph) {
        return Err(CanonicalPrimitiveProviderClientError::PlannerArtifactMismatch {});
    }
    let root_key = graph.root_key();
    let root_unit = graph.module_unit(root_key).ok_or(
        CanonicalPrimitiveProviderClientError::InvalidTopology {
            reason: "root unit is absent",
        },
    )?;
    if root_unit
        .body()
        .uses()
        .iter()
        .any(|use_declaration| matches!(&use_declaration.visibility, Visibility::Public))
    {
        return Err(CanonicalPrimitiveProviderClientError::InvalidTopology {
            reason: "parsed public re-exports are outside the generic provider/client route",
        });
    }
    let mut provider_keys = BTreeSet::new();
    for edge in plan.import_edges() {
        if edge.importing_module() != root_key
            || edge.defining_module().parent().as_ref() != Some(root_key)
        {
            return Err(CanonicalPrimitiveProviderClientError::InvalidTopology {
                reason: "planned edges must target direct root providers",
            });
        }
        provider_keys.insert(edge.defining_module().clone());
    }
    if root_unit.body().uses().len() != plan.import_edges().len() {
        return Err(CanonicalPrimitiveProviderClientError::InvalidTopology {
            reason: "root uses must be planned direct imports",
        });
    }
    preflight_complete_topology(root_key, graph, &provider_keys)?;

    let mut providers = BTreeMap::new();
    let mut imports = BTreeMap::new();
    for provider_key in &provider_keys {
        let unit = graph.module_unit(provider_key).ok_or(
            CanonicalPrimitiveProviderClientError::InvalidTopology {
                reason: "planned provider unit is absent",
            },
        )?;
        preflight_provider(provider_key, unit, graph)?;
        let checked = check_unit(provider_key, unit, &BTreeMap::new(), true)?;
        for edge in plan
            .import_edges()
            .iter()
            .filter(|edge| edge.defining_module() == provider_key)
        {
            let function = checked.function(edge.defining_identity().name()).ok_or(
                CanonicalPrimitiveProviderClientError::InvalidPlanEdge {
                    reason: "planned provider function is not checked",
                },
            )?;
            if !matches!(function.visibility(), Visibility::Public)
                || function.defining_identity().module_key()
                    != edge.defining_identity().module_key()
                || function.defining_identity().name() != edge.defining_identity().name()
                || function.declaration_span() != edge.declaration_span()
                || function.origin() != edge.origin()
                || !is_primitive_signature(function.signature())
            {
                return Err(CanonicalPrimitiveProviderClientError::InvalidPlanEdge {
                    reason: "planned edge no longer matches the checked public provider",
                });
            }
            if imports.contains_key(edge.local_name()) {
                return Err(CanonicalPrimitiveProviderClientError::InvalidPlanEdge {
                    reason: "planned root import names collide",
                });
            }
            imports.insert(
                edge.local_name().into(),
                CanonicalCheckedPrimitiveImportBinding {
                    local_name: edge.local_name().into(),
                    use_span: edge.use_span(),
                    defining_identity: edge.defining_identity().clone(),
                    declaration_span: edge.declaration_span(),
                    origin: edge.origin().clone(),
                    visibility: edge.visibility().clone(),
                    signature: function.signature().clone(),
                },
            );
        }
        providers.insert(provider_key.clone(), checked);
    }
    preflight_root(root_key, root_unit, &imports)?;
    let imported = imports
        .iter()
        .map(|(name, binding)| (name.clone(), binding.signature.clone()))
        .collect();
    let root = check_unit(root_key, root_unit, &imported, false)?;
    Ok(CanonicalCheckedPrimitiveProviderClient {
        root,
        providers,
        imports,
    })
}

#[allow(
    clippy::result_large_err,
    reason = "anchored public diagnostics remain unboxed"
)]
fn preflight_provider(
    key: &ModuleKey,
    unit: &ash_parser::ModuleUnit,
    graph: &CanonicalModuleGraph,
) -> Result<(), CanonicalPrimitiveProviderClientError> {
    if let Some(use_declaration) = unit.body().uses().first() {
        return Err(
            CanonicalPrimitiveProviderClientError::UnsupportedProviderShape {
                provider_module: key.clone(),
                span: use_declaration.span,
                reason: "providers cannot contain parsed uses",
            },
        );
    }
    if let Some(declaration) = unit.body().module_decls().first() {
        return Err(
            CanonicalPrimitiveProviderClientError::UnsupportedProviderShape {
                provider_module: key.clone(),
                span: declaration.span,
                reason: "providers cannot contain child modules",
            },
        );
    }
    if graph
        .children(key)
        .is_some_and(|children| !children.is_empty())
    {
        return Err(
            CanonicalPrimitiveProviderClientError::UnsupportedProviderShape {
                provider_module: key.clone(),
                span: unit.body().span(),
                reason: "providers must be direct leaves",
            },
        );
    }
    for definition in unit.body().definitions() {
        if let Definition::Function(function) = definition {
            preflight_function(key, function, true)?;
        } else {
            return Err(
                CanonicalPrimitiveProviderClientError::UnsupportedProviderShape {
                    provider_module: key.clone(),
                    span: unit.body().span(),
                    reason: "providers accept only ordinary functions",
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
fn preflight_complete_topology(
    root_key: &ModuleKey,
    graph: &CanonicalModuleGraph,
    selected_providers: &BTreeSet<ModuleKey>,
) -> Result<(), CanonicalPrimitiveProviderClientError> {
    for (module_key, _) in graph.module_units() {
        if module_key != root_key
            && (!selected_providers.contains(module_key)
                || module_key.parent().as_ref() != Some(root_key))
            && !is_descendant_of_selected_provider(module_key, selected_providers)
        {
            return Err(CanonicalPrimitiveProviderClientError::InvalidTopology {
                reason: "only selected direct provider modules may accompany the root client",
            });
        }
    }
    Ok(())
}

fn is_descendant_of_selected_provider(
    module_key: &ModuleKey,
    selected_providers: &BTreeSet<ModuleKey>,
) -> bool {
    let mut ancestor = module_key.parent();
    while let Some(parent) = ancestor {
        if selected_providers.contains(&parent) {
            return true;
        }
        ancestor = parent.parent();
    }
    false
}

#[allow(
    clippy::result_large_err,
    reason = "anchored public diagnostics remain unboxed"
)]
fn preflight_root(
    key: &ModuleKey,
    unit: &ash_parser::ModuleUnit,
    imports: &BTreeMap<Box<str>, CanonicalCheckedPrimitiveImportBinding>,
) -> Result<(), CanonicalPrimitiveProviderClientError> {
    for definition in unit.body().definitions() {
        let Definition::Function(function) = definition else {
            return Err(CanonicalPrimitiveProviderClientError::InvalidTopology {
                reason: "root accepts only ordinary functions",
            });
        };
        if let Some(binding) = imports.get(function.name.as_ref()) {
            return Err(
                CanonicalPrimitiveProviderClientError::LocalImportCollision {
                    root_module: key.clone(),
                    local_name: function.name.to_string(),
                    local_declaration_span: function.span,
                    use_span: binding.use_span,
                },
            );
        }
        preflight_function(key, function, false)?;
    }
    Ok(())
}

#[allow(
    clippy::result_large_err,
    reason = "anchored public diagnostics remain unboxed"
)]
fn preflight_function(
    key: &ModuleKey,
    function: &FnDef,
    provider: bool,
) -> Result<(), CanonicalPrimitiveProviderClientError> {
    let outside =
        || CanonicalPrimitiveProviderClientError::ProviderSignatureOutsidePrimitiveSlice {
            provider_module: key.clone(),
            function: function.name.to_string(),
            declaration_span: function.span,
        };
    if !function.type_params.is_empty()
        || function.contract.is_some()
        || function.proposition_tail.is_some()
        || !matches!(
            &function.visibility,
            Visibility::Inherited | Visibility::Public
        )
        || function.return_type.is_none()
        || !function
            .params
            .iter()
            .all(|parameter| is_primitive_surface(&parameter.ty))
        || !function
            .return_type
            .as_ref()
            .is_some_and(is_primitive_surface)
    {
        if provider {
            return Err(outside());
        }
        return Err(CanonicalPrimitiveProviderClientError::InvalidTopology {
            reason: "root functions must use explicit primitive signatures",
        });
    }
    Ok(())
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

#[allow(
    clippy::result_large_err,
    reason = "anchored public diagnostics remain unboxed"
)]
fn check_unit(
    key: &ModuleKey,
    unit: &ash_parser::ModuleUnit,
    imported: &BTreeMap<Box<str>, Type>,
    provider: bool,
) -> Result<CanonicalCheckedPrimitiveModule, CanonicalPrimitiveProviderClientError> {
    let functions = check_primitive_function_unit(key, unit, imported)
        .map_err(|error| unit_check_error(key, provider, error))?;
    Ok(CanonicalCheckedPrimitiveModule {
        artifact: unit.artifact().clone(),
        functions,
    })
}
fn unit_check_error(
    key: &ModuleKey,
    provider: bool,
    error: PrimitiveFunctionUnitCheckError,
) -> CanonicalPrimitiveProviderClientError {
    let (function, declaration_span, source) = match error {
        PrimitiveFunctionUnitCheckError::DuplicateFunction {
            function: _,
            declaration_span: _,
        } => {
            return CanonicalPrimitiveProviderClientError::InvalidTopology {
                reason: "primitive provider/client functions must have distinct names",
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
    if provider {
        CanonicalPrimitiveProviderClientError::ProviderBodyCheck {
            provider_module: key.clone(),
            function: function.to_string(),
            declaration_span,
            source: Box::new(source),
        }
    } else {
        CanonicalPrimitiveProviderClientError::ClientBodyCheck {
            root_module: key.clone(),
            function: function.to_string(),
            declaration_span,
            source: Box::new(source),
        }
    }
}
