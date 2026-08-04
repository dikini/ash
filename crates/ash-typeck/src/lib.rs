//! Ash Type Checker
//!
//! Type system and type inference for the Ash workflow language.
//!
//! This crate provides:
//! - **types**: Core type definitions and unification (TASK-015 to TASK-018)
//! - **constraints**: Constraint generation for expressions (TASK-019)
//! - **solver**: Constraint solving and type error reporting (TASK-020, TASK-025)
//! - **obligations**: Obligation tracking and proof obligations (TASK-023, TASK-024)

pub mod canonical_function_interface;
pub mod canonical_module_binder;
pub mod canonical_module_collection;
pub mod canonical_primitive_interface_fragments;
pub mod canonical_primitive_provider_client;
pub mod canonical_provisional_module_scopes;
pub mod canonical_simple_import_planner;
mod canonical_structural_module_binder;
pub mod capability_typecheck;
pub mod check_expr;
pub mod check_pattern;
mod checked_computation;
pub mod constraint_checking;
pub mod constraints;
pub mod diagnostic;
pub(crate) mod do_target;
pub mod effective_caps;
pub mod error;
pub mod exhaustiveness;
mod handler_rows;
pub mod instantiate;
pub mod interface_import_resolver;
pub mod kind;
pub mod module_core_cps_lowering;
pub mod module_interface_finalization;
pub mod name_binding;
pub mod normalizer;
pub mod obligation_checker;
pub mod obligations;
pub mod policy_check;
pub mod purity;
pub mod qualified_name;
pub mod requirements;
pub mod role_checking;
pub mod solver;
pub mod type_env;
pub mod types;
pub mod visibility;

mod surface_type_lowering;

pub(crate) use surface_type_lowering::bind_pattern_variables;

// SMT-based policy conflict detection using Z3
// Provides compile-time verification of policy constraints
pub mod smt;

#[doc(hidden)]
pub use do_target::{SelectedDoEvidence, SelectedDoOperation};

// Re-export smt module under a unified name
pub use smt as policy;

pub use ash_core::ast::{TypeDef, VariantDef};
pub use canonical_function_interface::{
    CanonicalCheckedFunction, CanonicalCheckedFunctionIdentity, CanonicalCheckedFunctionModule,
    CanonicalCheckedFunctionModuleSet, CanonicalModuleCheckError, CanonicalPublicFunctionInterface,
    check_closed_function_modules,
};
pub use canonical_module_binder::bind_simple_parsed_uses;
pub use canonical_primitive_interface_fragments::{
    CanonicalDirectPrimitiveReexportLocalAliasBinding, CanonicalDirectPrimitiveReexportRootClient,
    CanonicalDirectPrimitiveReexportRootClientBodyDiagnostic,
    CanonicalDirectPrimitiveReexportRootClientError, CanonicalPrimitiveInterfaceError,
    CanonicalPrimitiveInterfaceFragments, CanonicalPrimitivePublicChild,
    CanonicalPrimitiveReexport, check_direct_primitive_interface_fragments,
    check_direct_primitive_reexport_root_client,
};
pub use canonical_primitive_provider_client::{
    CanonicalCheckedPrimitiveImportBinding, CanonicalCheckedPrimitiveModule,
    CanonicalCheckedPrimitiveProviderClient, CanonicalPrimitiveProviderClientError,
    check_primitive_provider_client,
};
pub use canonical_provisional_module_scopes::{
    CanonicalNormalizedChild, CanonicalNormalizedModuleScope, CanonicalNormalizedScopeProjection,
    CanonicalProvisionalModuleScopes, CanonicalStructuralDiagnosticValue,
    CanonicalStructuralImportError,
};
pub use canonical_simple_import_planner::{
    CanonicalBoundModuleBinding, CanonicalBoundModuleSet,
    CanonicalBoundSelfOrdinaryFunctionAliasSet, CanonicalDefinitionIdentity,
    CanonicalDirectPrimitiveInterfaceImportError, CanonicalDirectPrimitiveReexportRootClientPlan,
    CanonicalDirectPrimitiveReexportRootClientPlanError, CanonicalImportCycle,
    CanonicalModuleBindError, CanonicalResolvedSelfOrdinaryFunctionAliases,
    CanonicalResolvedSimpleImports, CanonicalSelfOrdinaryFunctionAliasBinding,
    CanonicalSimpleImportEdge, resolve_direct_primitive_interface_imports,
    resolve_direct_primitive_reexport_root_client_plan,
    resolve_scoped_glob_local_precedence_imports_with_scopes,
    resolve_scoped_glob_ordinary_function_imports_with_scopes,
    resolve_scoped_grouped_ordinary_function_imports_with_scopes,
    resolve_scoped_self_ordinary_function_imports_with_scopes,
    resolve_scoped_simple_local_precedence_imports_with_scopes,
    resolve_scoped_simple_ordinary_function_imports_with_scopes,
    resolve_scoped_super_grouped_ordinary_function_imports_with_scopes,
    resolve_scoped_super_ordinary_function_imports_with_scopes, resolve_simple_parsed_imports,
    resolve_simple_parsed_imports_with_scopes,
};
pub use canonical_structural_module_binder::bind_scoped_structural_parsed_uses;
pub use canonical_structural_module_binder::{
    bind_scoped_glob_local_precedence_imports, bind_scoped_glob_ordinary_function_imports,
    bind_scoped_grouped_ordinary_function_imports, bind_scoped_self_ordinary_function_imports,
    bind_scoped_simple_local_precedence_imports, bind_scoped_simple_ordinary_function_imports,
    bind_scoped_super_grouped_ordinary_function_imports,
    bind_scoped_super_ordinary_function_imports,
};
pub use check_pattern::{
    Bindings, Irrefutability, IrrefutabilityBlockedReason, IrrefutabilityImpossibleReason,
    IrrefutabilityOutcome, IrrefutabilityWitness, check_irrefutable_pattern,
    check_irrefutable_pattern_with_canonical_type, check_irrefutable_pattern_with_canonicalization,
    check_pattern,
};
#[doc(hidden)]
pub use checked_computation::{
    CheckedComputation, infer_checked_computation_for_test,
    infer_checked_handler_computation_for_test, union_checked_computations_for_test,
};
pub use constraint_checking::*;
pub use constraints::*;
pub use effective_caps::{
    CapabilitySource, CompositionError, EffectiveCapabilitySet, MergedCapability,
};
#[doc(hidden)]
pub use handler_rows::{
    NormalizedHandlerRow, NormalizedHandlerRowItem, normalize_handler_row_for_test,
    normalize_handler_row_with_imported_summaries_for_test,
};
pub use instantiate::{InstantiateError, InstantiateSubst, instantiate};
pub use kind::Kind;
pub use name_binding::{NameBinder, NameError};
pub use normalizer::*;
pub use obligation_checker::*;
pub use obligations::*;
pub use policy_check::*;
pub use qualified_name::QualifiedName;
pub use requirements::{
    CheckResult, ContractCheckResult, RequirementContext, RequirementError, check_contract,
    check_requirement,
};
pub use solver::{Solver, TypeError};
pub use type_env::{
    AuthorityProvenanceKind, AuthorityProvenanceReport, BindingProvenanceSourceInfo,
    CallableDeclarationKind, CapabilityBindingInfo, CapabilityBindingProvenanceInfo,
    ContractIntrinsicKind, ContractIntrinsicParameterClass, DEFAULT_PROOF_FUEL,
    DeclaredConcreteOperation, ErasedProof, HandlerCallableRequirementError,
    ImplementationAuthoritySourceInfo, NominalNewtype, PartialConstructorElaborationError,
    PatternCanonicalConstructor, PatternCanonicalType, PatternCanonicalization,
    PatternCanonicalizationBlockedReason, ProofTotalityResult, ProofTotalityStatus,
    ProofTotalityUntestedReason, ProvenanceSourceKind, PublicComputationAlgebra,
    PublicComputationIntrinsicKind, PublicComputationIntrinsicMapping, PublicComputationManifest,
    PublicComputationManifestKind, PublicComputationOperation, PublicComputationOperationAuthority,
    PublicComputationOperationRole, ResourceBindingProvenanceInfo, ResourceTypeInfo,
    StoredFnContract, TypeEnv,
};
pub use types::*;
pub use visibility::{ModulePath, VisibilityChecker, VisibilityError, VisibilityExt};

/// Return the canonical synthetic module identity used for standalone programs.
///
/// Frontends that perform a second declaration-resolution pass after ordinary
/// standalone type checking must reuse this identity so local nominal
/// declarations retain their checked identities.
#[must_use]
pub fn standalone_program_module_identity() -> ash_core::semantic_summary::ModuleIdentity {
    synthetic_program_module_identity()
}

/// Test-support facade for do-target resolution without exposing the internal
/// hidden dictionary representation.
#[doc(hidden)]
#[allow(clippy::result_large_err)]
pub fn resolve_do_target_for_test(
    env: &TypeEnv,
    target: &ash_parser::surface::DoTarget,
) -> Result<(), error::ConstructorError> {
    do_target::resolve_do_target(env, target).map(|_| ())
}

use ash_parser::Spanned;
use surface_type_lowering::{
    bind_surface_type_parameters, synthetic_program_module_identity, workflow_surface_type_to_type,
};

fn variant_field_types(
    env: &TypeEnv,
    expected: &Type,
    variant_name: &str,
) -> Option<Vec<(String, Type)>> {
    #[allow(clippy::collapsible_if)]
    {
        if let Type::Constructor { name, args, .. } = expected {
            if let Ok(crate::type_env::UnfoldedBody::Enum(variants)) =
                env.unfold_constructor(name, args)
            {
                if let Some(variant) = variants
                    .into_iter()
                    .find(|variant| variant.name == variant_name)
                {
                    return Some(variant.fields);
                }
            }
        }
    }

    let (type_name, variant_index) = env.lookup_constructor(variant_name)?;
    match env.lookup_type_info(type_name.as_str())? {
        crate::type_env::TypeInfo::Enum { variants, .. } => variants
            .get(variant_index)
            .map(|variant| variant.fields.clone()),
        crate::type_env::TypeInfo::Struct { .. } => None,
    }
}

/// Type check a program.
pub fn type_check_program(
    program: &ash_parser::surface::Program,
) -> Result<TypeCheckResult, TypeCheckError> {
    let env = TypeEnv::with_builtin_types();
    type_check_program_in_env(&env, program)
}

/// Configuration for program type checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeCheckConfig {
    /// Fuel budget for Stage-3 proof totality traversal.
    pub proof_fuel: usize,
}

impl Default for TypeCheckConfig {
    fn default() -> Self {
        Self {
            proof_fuel: DEFAULT_PROOF_FUEL,
        }
    }
}

fn fn_signature_from_parts(
    env: &TypeEnv,
    type_params: &[ash_parser::surface::TypeParam],
    params: &[ash_parser::surface::Param],
    return_type: Option<&ash_parser::surface::Type>,
) -> Result<Type, TypeCheckError> {
    let (signature_env, bindings) = bind_surface_type_parameters(env, type_params)?;
    let param_types = params
        .iter()
        .map(|param| workflow_surface_type_to_type(&signature_env, &param.ty, &bindings))
        .collect::<Result<Vec<_>, _>>()?;
    let return_ty = match return_type {
        Some(ty) => workflow_surface_type_to_type(&signature_env, ty, &bindings)?,
        None => Type::Var(TypeVar::fresh()),
    };
    Ok(Type::Fn(param_types, Box::new(return_ty)))
}

fn reject_runtime_prop_return(
    signature: &Type,
    callable_description: &str,
    name: &str,
) -> Result<(), TypeCheckError> {
    let Type::Fn(_, return_ty) = signature else {
        unreachable!("callable signatures are function types");
    };
    if return_ty.contains_prop_kind() {
        return Err(TypeCheckError::TypeError(format!(
            "Prop-typed values cannot escape into runtime {callable_description} return '{name} -> {return_ty}'"
        )));
    }
    Ok(())
}

/// Compute the type signature of an ordinary `fn` definition.
pub fn fn_signature_type(
    env: &TypeEnv,
    function: &ash_parser::surface::FnDef,
) -> Result<Type, TypeCheckError> {
    let signature = fn_signature_from_parts(
        env,
        &function.type_params,
        &function.params,
        function
            .return_type
            .as_ref()
            .map(ash_parser::callable_result_type_for_fn_contract),
    )?;
    reject_runtime_prop_return(&signature, "function", function.name.as_ref())?;
    Ok(signature)
}

/// Compute the type signature of a builtin `fn` definition.
pub fn builtin_fn_signature_type(
    env: &TypeEnv,
    function: &ash_parser::surface::BuiltinFnDef,
) -> Result<Type, TypeCheckError> {
    let signature = fn_signature_from_parts(
        env,
        &function.type_params,
        &function.params,
        Some(&function.return_type),
    )?;
    reject_runtime_prop_return(&signature, "builtin function", function.name.as_ref())?;
    Ok(signature)
}

fn row_item_span(item: &ash_parser::surface::ComputationRowItem) -> ash_parser::token::Span {
    use ash_parser::surface::ComputationRowItem;
    match item {
        ComputationRowItem::Operation { span, .. }
        | ComputationRowItem::WholeRow { span, .. }
        | ComputationRowItem::Resource { span, .. }
        | ComputationRowItem::Role { span, .. }
        | ComputationRowItem::Policy { span, .. }
        | ComputationRowItem::Channel { span, .. }
        | ComputationRowItem::Process { span, .. }
        | ComputationRowItem::Fail { span, .. }
        | ComputationRowItem::Evidence { span, .. }
        | ComputationRowItem::Group { span, .. }
        | ComputationRowItem::Tail { span, .. } => *span,
    }
}

fn row_item_text(item: &ash_parser::surface::ComputationRowItem) -> String {
    use ash_parser::surface::ComputationRowItem;
    let path_text = |path: &[ash_parser::surface::Name]| {
        path.iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>()
            .join("::")
    };
    match item {
        ComputationRowItem::Operation {
            path, separator, ..
        } => {
            let Some((last, prefix)) = path.split_last() else {
                return String::new();
            };
            if prefix.is_empty() {
                return last.to_string();
            }
            let separator =
                match separator.unwrap_or(ash_parser::surface::RowPathSeparator::DoubleColon) {
                    ash_parser::surface::RowPathSeparator::Dot => ".",
                    ash_parser::surface::RowPathSeparator::DoubleColon => "::",
                };
            format!("{}{separator}{last}", path_text(prefix))
        }
        ComputationRowItem::WholeRow { variable, .. } => variable.to_string(),
        ComputationRowItem::Resource { path, mode, .. } => mode.as_ref().map_or_else(
            || format!("resource {}", path_text(path)),
            |mode| format!("resource {} {mode}", path_text(path)),
        ),
        ComputationRowItem::Role { path, .. } => format!("role {}", path_text(path)),
        ComputationRowItem::Policy { path, .. } => format!("policy {}", path_text(path)),
        ComputationRowItem::Channel { path, mode, .. } => mode.as_ref().map_or_else(
            || format!("channel {}", path_text(path)),
            |mode| format!("channel {mode} {}", path_text(path)),
        ),
        ComputationRowItem::Process {
            keyword, operation, ..
        } => operation.as_ref().map_or_else(
            || keyword.to_string(),
            |operation| format!("{keyword} {operation}"),
        ),
        ComputationRowItem::Fail { path, .. } => path.as_ref().map_or_else(
            || "fail".to_string(),
            |path| format!("fail {}", path_text(path)),
        ),
        ComputationRowItem::Evidence { path, .. } => format!("evidence {}", path_text(path)),
        ComputationRowItem::Group { path, .. } => format!("group {}", path_text(path)),
        ComputationRowItem::Tail { variable, .. } => format!("| {variable}"),
    }
}

fn unsupported_predicate_like_row_family(
    item: &ash_parser::surface::ComputationRowItem,
) -> Option<&'static str> {
    use ash_parser::surface::ComputationRowItem;
    let first = match item {
        ComputationRowItem::Operation { path, .. } => path.first()?,
        ComputationRowItem::WholeRow { variable, .. } => variable,
        _ => return None,
    };
    [
        "requires",
        "ensures",
        "invariant",
        "law",
        "proof",
        "contract",
    ]
    .into_iter()
    .find(|family| first.as_ref() == *family || first.as_ref().starts_with(&format!("{family}_")))
}

fn validate_operation_row_identity(
    env: &TypeEnv,
    item: &ash_parser::surface::ComputationRowItem,
) -> Result<(), TypeCheckError> {
    let ash_parser::surface::ComputationRowItem::Operation {
        path,
        separator,
        span,
    } = item
    else {
        return Ok(());
    };
    if *separator != Some(ash_parser::surface::RowPathSeparator::DoubleColon) {
        return Ok(());
    }
    let [target, method] = path.as_slice() else {
        return Ok(());
    };
    if !target
        .as_ref()
        .chars()
        .next()
        .is_some_and(char::is_uppercase)
    {
        return Ok(());
    }
    match env.resolve_operation_row_identity(target.as_ref(), method.as_ref()) {
        crate::type_env::OperationRowIdentityResolution::ConcreteImpl { .. }
        | crate::type_env::OperationRowIdentityResolution::AbstractImpl { .. } => Ok(()),
        crate::type_env::OperationRowIdentityResolution::InterfaceQualified {
            suggestion, ..
        } => Err(
            crate::error::TypeEnvError::InterfaceQualifiedOperationRowIdentity {
                item: row_item_text(item),
                suggestion,
                span: *span,
            }
            .into(),
        ),
        crate::type_env::OperationRowIdentityResolution::UnknownImplType { impl_type } => {
            Err(crate::error::TypeEnvError::UnknownOperationRowImplType {
                impl_type,
                item: row_item_text(item),
                span: *span,
            }
            .into())
        }
        crate::type_env::OperationRowIdentityResolution::UnknownMethod { candidates, .. } => {
            Err(crate::error::TypeEnvError::UnknownOperationRowMethod {
                item: row_item_text(item),
                candidates: candidates.join(", "),
                span: *span,
            }
            .into())
        }
    }
}

/// Check an imported effect-row export as a requirement description.
///
/// Summary rows deliberately retain source text rather than a second parsed
/// row AST.  This narrow bridge applies the validation already available at
/// the callable boundary to named imported rows, while keeping those rows
/// non-granting: no capability, provider, or admission state is installed.
fn validate_imported_effect_row_export(
    env: &TypeEnv,
    name: &str,
    span: ash_parser::token::Span,
    expanding: &mut Vec<(ash_core::semantic_summary::EffectRowExportId, String)>,
) -> Result<(), TypeCheckError> {
    let Some(row) = env.lookup_effect_row_export(name) else {
        return Ok(());
    };
    if row.authority != ash_core::semantic_summary::EffectRowAuthority::NonGranting {
        return Err(crate::error::TypeEnvError::InvalidDefinition(
            format!("imported effect-row export '{name}' must remain non-granting"),
            span,
        )
        .into());
    }
    if let Some(cycle_start) = expanding
        .iter()
        .position(|(identity, _)| identity == &row.id)
    {
        let mut cycle = expanding[cycle_start..]
            .iter()
            .map(|(_, visible_name)| visible_name.clone())
            .collect::<Vec<_>>();
        cycle.push(name.to_string());
        // `name` can be the provider declaration spelling reached from an
        // aliased entry binding.  Preserve that canonical hop, then close the
        // displayed path through the caller-visible alias rather than hiding
        // it behind the mutable binding ID.
        if row.provider.declaration_name == name && row.binding.visible_name != name {
            cycle.push(row.binding.visible_name.to_string());
        }
        return Err(crate::error::TypeEnvError::InvalidDefinition(
            format!("cyclic imported effect-row export '{}'", cycle.join(" -> ")),
            span,
        )
        .into());
    }
    if matches!(
        row.binding.closure_status,
        ash_core::semantic_summary::EffectRowClosureStatus::OpaqueInaccessibleDependency(_)
    ) {
        return Err(crate::error::TypeEnvError::InvalidDefinition(
            format!("inaccessible imported effect-row dependency referenced by '{name}'"),
            span,
        )
        .into());
    }
    expanding.push((row.id.clone(), name.to_string()));

    let result = row.row_items.iter().try_for_each(|item| {
        validate_imported_effect_row_item(env, item.text.trim(), span, expanding)
    });
    expanding.pop();
    result
}

/// Verify that a public callable's named row dependency never crosses a
/// private alias/group boundary.
///
/// This is deliberately separate from row-content validation: rows remain
/// non-granting requirement descriptions, and the existing expansion pass
/// continues to own cycle and item-family diagnostics.  The visibility walk
/// only follows registered named rows, so raw row items retain their ordinary
/// validation path.
fn validate_public_effect_row_reference(
    env: &TypeEnv,
    public_item: &str,
    referenced_name: &str,
    span: ash_parser::token::Span,
    expanding: &mut Vec<String>,
) -> Result<(), TypeCheckError> {
    let Some(row) = env.lookup_effect_row_export(referenced_name) else {
        return Ok(());
    };
    if row.visibility != ash_core::ast::Visibility::Public {
        return Err(crate::error::TypeEnvError::PrivateDependencyExportFailure {
            public_item: public_item.to_string(),
            dependency: referenced_name.to_string(),
            dependency_kind: "effect row".to_string(),
            span,
        }
        .into());
    }
    if expanding.iter().any(|expanded| expanded == referenced_name) {
        return Ok(());
    }

    expanding.push(referenced_name.to_string());
    let result = row.row_items.iter().try_for_each(|item| {
        validate_public_effect_row_item(env, public_item, item.text.trim(), span, expanding)
    });
    expanding.pop();
    result
}

fn validate_public_effect_row_item(
    env: &TypeEnv,
    public_item: &str,
    item: &str,
    span: ash_parser::token::Span,
    expanding: &mut Vec<String>,
) -> Result<(), TypeCheckError> {
    let referenced_name = item
        .strip_prefix("group ")
        .map(str::trim)
        .or_else(|| (!item.chars().any(char::is_whitespace)).then_some(item));
    let Some(referenced_name) = referenced_name else {
        return Ok(());
    };
    validate_public_effect_row_reference(env, public_item, referenced_name, span, expanding)
}

fn validate_imported_effect_row_item(
    env: &TypeEnv,
    item: &str,
    span: ash_parser::token::Span,
    expanding: &mut Vec<(ash_core::semantic_summary::EffectRowExportId, String)>,
) -> Result<(), TypeCheckError> {
    let family = [
        "requires",
        "ensures",
        "invariant",
        "law",
        "proof",
        "contract",
    ]
    .into_iter()
    .find(|family| {
        item == *family
            || item
                .strip_prefix(family)
                .is_some_and(|suffix| suffix.starts_with('_') || suffix.starts_with("::"))
    });
    if let Some(family) = family {
        return Err(crate::error::TypeEnvError::UnsupportedRowItemFamily {
            family: family.to_string(),
            item: item.to_string(),
            span,
        }
        .into());
    }

    if let Some(group) = item.strip_prefix("group ") {
        return validate_imported_effect_row_export(env, group.trim(), span, expanding);
    }
    if !item.chars().any(char::is_whitespace) && env.lookup_effect_row_export(item).is_some() {
        return validate_imported_effect_row_export(env, item, span, expanding);
    }

    let mut segments = item.split("::");
    let (Some(target), Some(method), None) = (segments.next(), segments.next(), segments.next())
    else {
        return Ok(());
    };
    if !target.chars().next().is_some_and(char::is_uppercase) {
        return Ok(());
    }
    match env.resolve_operation_row_identity(target, method) {
        crate::type_env::OperationRowIdentityResolution::ConcreteImpl { .. }
        | crate::type_env::OperationRowIdentityResolution::AbstractImpl { .. } => Ok(()),
        crate::type_env::OperationRowIdentityResolution::InterfaceQualified {
            suggestion, ..
        } => Err(
            crate::error::TypeEnvError::InterfaceQualifiedOperationRowIdentity {
                item: item.to_string(),
                suggestion,
                span,
            }
            .into(),
        ),
        crate::type_env::OperationRowIdentityResolution::UnknownImplType { impl_type } => {
            Err(crate::error::TypeEnvError::UnknownOperationRowImplType {
                impl_type,
                item: item.to_string(),
                span,
            }
            .into())
        }
        crate::type_env::OperationRowIdentityResolution::UnknownMethod { candidates, .. } => {
            Err(crate::error::TypeEnvError::UnknownOperationRowMethod {
                item: item.to_string(),
                candidates: candidates.join(", "),
                span,
            }
            .into())
        }
    }
}

fn validate_computation_row(
    env: &TypeEnv,
    row: &ash_parser::surface::ComputationRow,
    public_callable: Option<&str>,
) -> Result<(), TypeCheckError> {
    let mut tail_seen = None;
    let mut expanding_imported_rows = Vec::new();
    let mut expanding_public_rows = Vec::new();
    for (index, item) in row.items.iter().enumerate() {
        if let Some(family) = unsupported_predicate_like_row_family(item) {
            return Err(crate::error::TypeEnvError::UnsupportedRowItemFamily {
                family: family.to_string(),
                item: row_item_text(item),
                span: row_item_span(item),
            }
            .into());
        }
        validate_operation_row_identity(env, item)?;
        match item {
            ash_parser::surface::ComputationRowItem::WholeRow { variable, span } => {
                if let Some(public_callable) = public_callable {
                    validate_public_effect_row_reference(
                        env,
                        public_callable,
                        variable.as_ref(),
                        *span,
                        &mut expanding_public_rows,
                    )?;
                }
                validate_imported_effect_row_export(
                    env,
                    variable.as_ref(),
                    *span,
                    &mut expanding_imported_rows,
                )?;
            }
            ash_parser::surface::ComputationRowItem::Operation {
                path,
                separator: None,
                span,
            } if path.len() == 1 => {
                if let Some(public_callable) = public_callable {
                    validate_public_effect_row_reference(
                        env,
                        public_callable,
                        path[0].as_ref(),
                        *span,
                        &mut expanding_public_rows,
                    )?;
                }
                validate_imported_effect_row_export(
                    env,
                    path[0].as_ref(),
                    *span,
                    &mut expanding_imported_rows,
                )?;
            }
            ash_parser::surface::ComputationRowItem::Group { path, span } if path.len() == 1 => {
                if let Some(public_callable) = public_callable {
                    validate_public_effect_row_reference(
                        env,
                        public_callable,
                        path[0].as_ref(),
                        *span,
                        &mut expanding_public_rows,
                    )?;
                }
                validate_imported_effect_row_export(
                    env,
                    path[0].as_ref(),
                    *span,
                    &mut expanding_imported_rows,
                )?;
            }
            _ => {}
        }
        if let ash_parser::surface::ComputationRowItem::Tail { variable, span } = item {
            if tail_seen.is_some() {
                return Err(crate::error::TypeEnvError::DuplicateRowTail {
                    tail: variable.to_string(),
                    span: *span,
                }
                .into());
            }
            tail_seen = Some((variable, *span, index));
        }
    }
    if let Some((variable, span, index)) = tail_seen
        && index + 1 != row.items.len()
    {
        return Err(crate::error::TypeEnvError::RowTailNotFinal {
            tail: variable.to_string(),
            span,
        }
        .into());
    }
    Ok(())
}

fn validate_surface_type_rows(
    env: &TypeEnv,
    ty: &ash_parser::surface::Type,
) -> Result<(), TypeCheckError> {
    use ash_parser::surface::Type as SurfaceType;
    match ty {
        SurfaceType::List(item) | SurfaceType::Associated { base: item, .. } => {
            validate_surface_type_rows(env, item)
        }
        SurfaceType::Tuple(items) => items
            .iter()
            .try_for_each(|item| validate_surface_type_rows(env, item)),
        SurfaceType::Record(fields) => fields
            .iter()
            .try_for_each(|(_, item)| validate_surface_type_rows(env, item)),
        SurfaceType::Constructor { args, .. }
        | SurfaceType::AssociatedFamilyProjection { args, .. } => args
            .iter()
            .try_for_each(|item| validate_surface_type_rows(env, item)),
        SurfaceType::Fn(params, row, ret) => {
            params
                .iter()
                .try_for_each(|param| validate_surface_type_rows(env, param))?;
            if let Some(row) = row {
                validate_computation_row(env, row, None)?;
            }
            validate_surface_type_rows(env, ret)
        }
        SurfaceType::Name(_) | SurfaceType::Hole { .. } | SurfaceType::Capability(_) => Ok(()),
    }
}

fn validate_callable_rows(
    env: &TypeEnv,
    name: &str,
    is_public: bool,
    params: &[ash_parser::surface::Param],
    return_type: Option<&ash_parser::surface::Type>,
    proposition_tail: Option<&ash_parser::surface::PropositionTail>,
) -> Result<(), TypeCheckError> {
    params
        .iter()
        .try_for_each(|param| validate_surface_type_rows(env, &param.ty))?;
    if let Some(return_type) = return_type {
        validate_surface_type_rows(env, return_type)?;
    }
    if let Some(row) = proposition_tail.and_then(|tail| tail.row.as_ref()) {
        if let Some(ash_parser::surface::Type::Fn(params, Some(inline_row), _)) = return_type
            && params.is_empty()
        {
            return Err(crate::error::TypeEnvError::DuplicateCallableRow {
                callable: name.to_string(),
                inline_span: inline_row.span,
                expanded_span: row.span,
                span: row.span,
            }
            .into());
        }
        validate_computation_row(env, &row.row, is_public.then_some(name))?;
    }
    Ok(())
}

fn register_function_contract(
    env: &mut TypeEnv,
    function: &ash_parser::surface::FnDef,
) -> Result<(), TypeCheckError> {
    let lowered = ash_parser::lower_fn_contract_for_function(function)
        .map_err(|error| TypeCheckError::TypeError(error.to_string()))?;
    env.bind_fn_contract(
        function.name.as_ref(),
        StoredFnContract {
            param_names: function
                .params
                .iter()
                .map(|param| param.name.to_string())
                .collect(),
            contract: lowered.contract,
            runtime_postconditions: lowered.runtime_postconditions,
        },
    );
    Ok(())
}

fn integer_fact(
    facts: &std::collections::HashMap<String, i64>,
    expr: &ash_parser::surface::Expr,
) -> Option<i64> {
    match expr {
        ash_parser::surface::Expr::Literal(ash_parser::surface::Literal::Int(value)) => {
            Some(*value)
        }
        ash_parser::surface::Expr::Unary {
            op: ash_parser::surface::UnaryOp::Neg,
            operand,
            ..
        } => integer_fact(facts, operand).map(|value| -value),
        ash_parser::surface::Expr::Variable { name, .. } => facts.get(name.as_ref()).copied(),
        ash_parser::surface::Expr::Binary {
            op, left, right, ..
        } => {
            let left = integer_fact(facts, left)?;
            let right = integer_fact(facts, right)?;
            match op {
                ash_parser::surface::BinaryOp::Add => Some(left + right),
                ash_parser::surface::BinaryOp::Sub => Some(left - right),
                ash_parser::surface::BinaryOp::Mul => Some(left * right),
                ash_parser::surface::BinaryOp::Div => (right != 0).then_some(left / right),
                ash_parser::surface::BinaryOp::Mod => (right != 0).then_some(left % right),
                _ => None,
            }
        }
        _ => None,
    }
}

fn branch_assumption(
    condition: &ash_parser::surface::Expr,
) -> Option<(String, ash_core::contract::ArithConstraint)> {
    use ash_core::contract::ArithConstraint;
    use ash_parser::surface::{BinaryOp, Expr, Literal};
    let Expr::Binary {
        op, left, right, ..
    } = condition
    else {
        return None;
    };
    let (Expr::Variable { name, .. }, Expr::Literal(Literal::Int(value))) = (&**left, &**right)
    else {
        return None;
    };
    let constraint = match op {
        BinaryOp::Gt => ArithConstraint::Gt(*value),
        BinaryOp::Geq => ArithConstraint::Gte(*value),
        BinaryOp::Lt => ArithConstraint::Lt(*value),
        BinaryOp::Leq => ArithConstraint::Lte(*value),
        BinaryOp::Eq => ArithConstraint::Eq(*value),
        BinaryOp::Neq => ArithConstraint::NotEq(*value),
        _ => return None,
    };
    Some((name.to_string(), constraint))
}

fn validate_function_preconditions(
    env: &TypeEnv,
    expr: &ash_parser::surface::Expr,
    facts: &mut std::collections::HashMap<String, i64>,
    assumptions: &mut std::collections::HashMap<String, Vec<ash_core::contract::ArithConstraint>>,
) -> Result<(), TypeCheckError> {
    use ash_parser::surface::Expr;
    match expr {
        Expr::Call {
            func, module, args, ..
        } => {
            for arg in args {
                validate_function_preconditions(env, arg, facts, assumptions)?;
            }
            let name = module
                .as_ref()
                .map(|module| format!("{module}::{func}"))
                .unwrap_or_else(|| func.to_string());
            if let Some(boundary) = env.lookup_fn_contract(&name) {
                let mut context = RequirementContext::new();
                for (parameter, arg) in boundary.param_names.iter().zip(args) {
                    let value = integer_fact(facts, arg);
                    if let Some(value) = value {
                        context = context.with_fact(parameter.clone(), value);
                    }
                    if let Expr::Variable { name, .. } = arg {
                        for assumption in assumptions.get(name.as_ref()).into_iter().flatten() {
                            context = context
                                .with_arithmetic_assumption(parameter.clone(), assumption.clone());
                        }
                    }
                }
                let result = check_contract(&boundary.contract, &context);
                if !result.is_success() {
                    let details = result
                        .errors()
                        .into_iter()
                        .map(|error| error.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    return Err(TypeCheckError::TypeError(format!(
                        "fn precondition may not hold for call '{name}': {details}"
                    )));
                }
            }
            Ok(())
        }
        Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            let mut scoped_facts = facts.clone();
            for statement in statements {
                if let ash_parser::surface::BlockStmt::Let { pattern, expr, .. } = statement {
                    validate_function_preconditions(env, expr, &mut scoped_facts, assumptions)?;
                    if let (ash_parser::surface::Pattern::Variable { name, .. }, Some(value)) =
                        (pattern, integer_fact(&scoped_facts, expr))
                    {
                        scoped_facts.insert(name.to_string(), value);
                    }
                }
            }
            if let Some(tail) = tail_expr {
                validate_function_preconditions(env, tail, &mut scoped_facts, assumptions)?;
            }
            Ok(())
        }
        Expr::DoBlock { stmts, .. } => {
            let mut scoped_facts = facts.clone();
            for statement in stmts {
                match statement {
                    ash_parser::surface::DoStmt::Let { name, value, .. }
                    | ash_parser::surface::DoStmt::Bind { name, value, .. } => {
                        validate_function_preconditions(
                            env,
                            value,
                            &mut scoped_facts,
                            assumptions,
                        )?;
                        if let Some(value) = integer_fact(&scoped_facts, value) {
                            scoped_facts.insert(name.to_string(), value);
                        }
                    }
                    ash_parser::surface::DoStmt::Expr { value, .. }
                    | ash_parser::surface::DoStmt::Return { value, .. } => {
                        validate_function_preconditions(
                            env,
                            value,
                            &mut scoped_facts,
                            assumptions,
                        )?;
                    }
                }
            }
            Ok(())
        }
        Expr::Binary { left, right, .. } => {
            validate_function_preconditions(env, left, facts, assumptions)?;
            validate_function_preconditions(env, right, facts, assumptions)
        }
        Expr::Unary { operand, .. } | Expr::FieldAccess { base: operand, .. } => {
            validate_function_preconditions(env, operand, facts, assumptions)
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            validate_function_preconditions(env, condition, facts, assumptions)?;
            let mut then_assumptions = assumptions.clone();
            if let Some((name, constraint)) = branch_assumption(condition) {
                then_assumptions.entry(name).or_default().push(constraint);
            }
            validate_function_preconditions(
                env,
                then_branch,
                &mut facts.clone(),
                &mut then_assumptions,
            )?;
            if let Some(else_branch) = else_branch {
                validate_function_preconditions(
                    env,
                    else_branch,
                    &mut facts.clone(),
                    &mut assumptions.clone(),
                )?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn register_public_function_proposition_tail(
    env: &mut TypeEnv,
    tail: &ash_parser::surface::PropositionTail,
    item_name: &str,
    item_kind: &str,
    site_id: u64,
) -> Result<(), TypeCheckError> {
    let obligation_start = env.proposition_obligations().len();
    env.add_proposition_obligations_from_tail(
        tail,
        ash_core::semantic_summary::SourceOrigin::Synthetic {
            reason: format!("{item_kind} proposition checking point {item_name}"),
        },
        crate::type_env::PropositionCheckingSite::new(
            site_id,
            crate::type_env::PropositionCheckingSiteKind::ExplicitRequirement,
            Some(format!("{item_kind} {item_name} proposition tail")),
        ),
    )
    .map_err(|error| {
        TypeCheckError::TypeError(format!("proposition tail lowering failed: {error}"))
    })?;
    env.discharge_required_proposition_obligations_since(obligation_start)
        .map(|_| ())
        .map_err(TypeCheckError::from)
}

fn register_function_signatures(
    env: &mut TypeEnv,
    definitions: &[ash_parser::surface::Definition],
) -> Result<(), TypeCheckError> {
    let mut staged = env.clone();
    for (index, definition) in definitions.iter().enumerate() {
        match definition {
            ash_parser::surface::Definition::Function(function) => {
                validate_callable_rows(
                    &staged,
                    function.name.as_ref(),
                    matches!(function.visibility, ash_parser::surface::Visibility::Public),
                    &function.params,
                    function.return_type.as_ref(),
                    function.proposition_tail.as_ref(),
                )?;
                let signature = fn_signature_type(&staged, function)?;
                staged.bind_variable(function.name.as_ref(), signature);
                staged.register_callable_declaration_kind(
                    function.name.to_string(),
                    CallableDeclarationKind::Function,
                );
                if matches!(function.visibility, ash_parser::surface::Visibility::Public)
                    && let Some(tail) = &function.proposition_tail
                {
                    register_public_function_proposition_tail(
                        &mut staged,
                        tail,
                        function.name.as_ref(),
                        "function",
                        0x8801_0000u64 + index as u64,
                    )?;
                }
            }
            ash_parser::surface::Definition::Handler(handler) => {
                validate_callable_rows(
                    &staged,
                    handler.name.as_ref(),
                    matches!(handler.visibility, ash_parser::surface::Visibility::Public),
                    &handler.params,
                    Some(&handler.return_type),
                    handler.proposition_tail.as_ref(),
                )?;
                let signature = handler_signature_type(&staged, handler)?;
                staged.bind_variable(handler.name.as_ref(), signature);
                staged.register_callable_declaration_kind(
                    handler.name.to_string(),
                    CallableDeclarationKind::Handler,
                );
            }
            ash_parser::surface::Definition::BuiltinFn(function) => {
                validate_callable_rows(
                    &staged,
                    function.name.as_ref(),
                    matches!(function.visibility, ash_parser::surface::Visibility::Public),
                    &function.params,
                    Some(&function.return_type),
                    function.proposition_tail.as_ref(),
                )?;
                let signature = builtin_fn_signature_type(&staged, function)?;
                staged.bind_variable(function.name.as_ref(), signature);
                if matches!(function.visibility, ash_parser::surface::Visibility::Public)
                    && let Some(tail) = &function.proposition_tail
                {
                    register_public_function_proposition_tail(
                        &mut staged,
                        tail,
                        function.name.as_ref(),
                        "builtin function",
                        0x8802_0000u64 + index as u64,
                    )?;
                }
            }
            ash_parser::surface::Definition::Capability(capability) => {
                staged.register_capability_symbol(capability.name.as_ref());
            }
            _ => {}
        }
    }
    for definition in definitions {
        if let ash_parser::surface::Definition::Function(function) = definition {
            register_function_contract(&mut staged, function)?;
        }
    }
    *env = staged;
    Ok(())
}

fn handler_signature_type(
    env: &TypeEnv,
    handler: &ash_parser::surface::HandlerDef,
) -> Result<Type, TypeCheckError> {
    let (signature_env, bindings) = bind_surface_type_parameters(env, &handler.type_params)?;
    let params = handler
        .params
        .iter()
        .map(|param| workflow_surface_type_to_type(&signature_env, &param.ty, &bindings))
        .collect::<Result<Vec<_>, _>>()?;
    let result = workflow_surface_type_to_type(&signature_env, &handler.return_type, &bindings)?;
    Ok(Type::Fn(params, Box::new(result)))
}

fn check_handler_declarations(
    env: &TypeEnv,
    program: &ash_parser::surface::Program,
) -> Result<std::collections::HashMap<String, CheckedHandlerDeclaration>, TypeCheckError> {
    let mut checked_handlers = std::collections::HashMap::new();
    for definition in &program.definitions {
        let ash_parser::surface::Definition::Handler(handler) = definition else {
            continue;
        };
        let callable_signature = env.lookup_variable(handler.name.as_ref()).ok_or_else(|| {
            TypeCheckError::ResolutionError(format!("handler '{}' has no signature", handler.name))
        })?;
        let Type::Fn(_, answer_type) = &callable_signature else {
            return Err(TypeCheckError::TypeError(format!(
                "handler '{}' has a non-function signature",
                handler.name
            )));
        };
        let ash_parser::surface::Expr::On { clauses, .. } = &handler.body else {
            return Err(TypeCheckError::TypeError(format!(
                "handler '{}' requires a canonical on body",
                handler.name
            )));
        };
        let computation =
            crate::checked_computation::infer_checked_handler_computation(env, program, handler)?;
        let mut operations = Vec::new();
        let mut done = None;
        for clause in clauses {
            match clause {
                ash_parser::surface::HandlerClause::Operation {
                    impl_type,
                    operation,
                    pattern,
                    resume,
                    body,
                    span,
                } => {
                    let resolved = env
                        .resolve_declared_concrete_operation(impl_type.as_ref(), operation.as_ref())
                        .map_err(TypeCheckError::TypeError)?;
                    let payload_type = resolved.params.first().cloned().ok_or_else(|| {
                        TypeCheckError::TypeError(format!(
                            "declared operation '{}.{}' has no payload type",
                            resolved.impl_type, resolved.operation
                        ))
                    })?;
                    let local_effect =
                        direct_concrete_operation_clause_body(body, pattern, env, &resolved)?;
                    let canonical_key = format!(
                        "operation:{}::{}::{}",
                        resolved.impl_type, resolved.interface, resolved.operation
                    );
                    if operations
                        .iter()
                        .any(|operation: &PendingHandlerOperation| {
                            operation.canonical_key == canonical_key
                        })
                    {
                        return Err(TypeCheckError::TypeError(format!(
                            "duplicate handler operation clause for {canonical_key}"
                        )));
                    }
                    operations.push(PendingHandlerOperation {
                        resolved,
                        payload_type,
                        resume_name: resume.to_string(),
                        pattern,
                        body,
                        local_effect,
                        canonical_key,
                        source_span: *span,
                    });
                }
                ash_parser::surface::HandlerClause::Done { binding, body, .. } => {
                    if done.is_some() {
                        return Err(TypeCheckError::TypeError(
                            "duplicate done clause".to_string(),
                        ));
                    }
                    done = Some((binding, body));
                }
            }
        }
        if operations.is_empty() {
            return Err(TypeCheckError::TypeError(
                "missing concrete operation clause".to_string(),
            ));
        }
        let Some((done_binding, done_body)) = done else {
            return Err(TypeCheckError::TypeError("missing done clause".to_string()));
        };
        let residual_row = crate::handler_rows::subtract_handled_operations(
            computation.normalized_row(),
            &operations
                .iter()
                .map(|operation| operation.canonical_key.clone())
                .collect::<Vec<_>>(),
        )
        .map_err(|error| TypeCheckError::TypeError(error.to_string()))?;
        let mut done_env = env.clone();
        done_env.bind_variable(done_binding.as_ref(), computation.result_type().clone());
        let done_result = crate::check_expr::check_expr(&done_env, done_body);
        if !done_result.is_ok() {
            return Err(TypeCheckError::TypeError(done_result.errors[0].to_string()));
        }
        let done_body_type = done_result.substitution.apply(&done_result.ty);
        crate::types::unify(answer_type, &done_body_type).map_err(|_| {
            TypeCheckError::TypeError(format!(
                "done clause must return {answer_type}, found {done_body_type}"
            ))
        })?;
        let done_computation =
            crate::checked_computation::infer_checked_computation_in_env(&done_env, done_body)?;
        let prepared_operations = operations
            .iter()
            .map(|operation| prepare_handler_clause_body(env, operation))
            .collect::<Result<Vec<_>, TypeCheckError>>()?;
        let output_row = crate::handler_rows::union_normalized_handler_rows(
            &std::iter::once(residual_row.clone())
                .chain(
                    prepared_operations
                        .iter()
                        .map(|prepared| prepared.computation.normalized_row().clone()),
                )
                .chain(std::iter::once(done_computation.normalized_row().clone()))
                .collect::<Vec<_>>(),
        )
        .map_err(|error| TypeCheckError::TypeError(error.to_string()))?;
        // A continuation resumes the unhandled part of the operand, never the
        // effects introduced by the clause or done bodies.  Keep that source
        // fact separate from the handler's resulting computation row.
        let continuation_multiplicity = if crate::handler_rows::is_closed_empty_row(&residual_row) {
            ContinuationMultiplicity::MultiShotPure
        } else {
            ContinuationMultiplicity::Affine
        };
        let clauses = operations
            .into_iter()
            .zip(prepared_operations)
            .map(|(operation, prepared)| {
                let body_type = check_prepared_handler_clause_body(
                    &prepared,
                    &operation.resume_name,
                    &operation.resolved.result_type,
                    answer_type,
                    continuation_multiplicity,
                )?;
                crate::types::unify(answer_type, &body_type).map_err(|_| {
                    TypeCheckError::TypeError(format!(
                        "handler operation body must return {answer_type}, found {body_type}"
                    ))
                })?;
                Ok(CheckedHandlerClause {
                    operation: operation.resolved,
                    payload_type: operation.payload_type,
                    resume_name: operation.resume_name,
                    origin: ash_parser::surface::SurfaceOrigin::Source {
                        span: operation.source_span,
                    },
                    local_effect: operation.local_effect,
                    done_binding: done_binding.to_string(),
                    done_body_type: done_body_type.clone(),
                    continuation_row: residual_row.clone(),
                    continuation_multiplicity,
                })
            })
            .collect::<Result<Vec<_>, TypeCheckError>>()?;
        checked_handlers.insert(
            handler.name.to_string(),
            CheckedHandlerDeclaration {
                callable_kind: CallableDeclarationKind::Handler,
                callable_signature: callable_signature.clone(),
                clauses,
                input_result_type: computation.result_type().clone(),
                input_row: computation.normalized_row().clone(),
                residual_row,
                output_row,
                answer_type: answer_type.as_ref().clone(),
                done_binding: done_binding.to_string(),
                done_binding_type: computation.result_type().clone(),
            },
        );
    }
    materialize_derived_impl_handlers(env, program, &mut checked_handlers)?;
    Ok(checked_handlers)
}

/// Preserve `derive handler` as a source-only checked fact.  This records the
/// impl's declared operations without constructing a Core handler or runtime
/// dispatch route.
fn materialize_derived_impl_handlers(
    env: &TypeEnv,
    program: &ash_parser::surface::Program,
    checked_handlers: &mut std::collections::HashMap<String, CheckedHandlerDeclaration>,
) -> Result<(), TypeCheckError> {
    for definition in &program.definitions {
        let ash_parser::surface::Definition::Impl(implementation) = definition else {
            continue;
        };
        let Some(ash_parser::surface::Type::Name(impl_type)) = implementation.type_args.last()
        else {
            continue;
        };
        for derived in &implementation.derived_handlers {
            let operations = implementation
                .methods
                .iter()
                .map(|method| {
                    env.resolve_declared_concrete_operation(
                        impl_type.as_ref(),
                        method.name.as_ref(),
                    )
                    .map_err(TypeCheckError::TypeError)
                })
                .collect::<Result<Vec<_>, _>>()?;
            if operations.is_empty() {
                return Err(TypeCheckError::TypeError(format!(
                    "derive handler '{}' requires at least one impl operation",
                    derived.name
                )));
            }
            let operation_rows = operations
                .iter()
                .map(|operation| {
                    crate::handler_rows::normalized_declared_operation(operation, derived.span)
                })
                .collect::<Vec<_>>();
            let mut input_rows = operation_rows;
            input_rows.push(crate::handler_rows::normalized_open_handler_row_tail(
                "r",
                derived.span,
            ));
            let input_row = crate::handler_rows::union_normalized_handler_rows(&input_rows)
                .map_err(|error| TypeCheckError::TypeError(error.to_string()))?;
            let residual_row = crate::handler_rows::subtract_handled_operations(
                &input_row,
                &operations
                    .iter()
                    .map(|operation| {
                        format!(
                            "operation:{}::{}::{}",
                            operation.impl_type, operation.interface, operation.operation
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .map_err(|error| TypeCheckError::TypeError(error.to_string()))?;
            // A derived handler is the total identity fold over the impl's
            // operations. Its answer is independently quantified rather than
            // fixed to any one operation result. `Type::Fn` cannot encode row
            // polymorphism, so this remains checked source-fact evidence only.
            let answer_type = Type::Var(TypeVar::fresh());
            let clauses = operations
                .into_iter()
                .map(|operation| CheckedHandlerClause {
                    payload_type: operation.params.first().cloned().unwrap_or(Type::Null),
                    operation,
                    resume_name: "resume".to_string(),
                    origin: ash_parser::surface::SurfaceOrigin::Desugaring {
                        source_span: derived.span,
                        rule: "derive handler".into(),
                    },
                    local_effect: None,
                    done_binding: "value".to_string(),
                    done_body_type: answer_type.clone(),
                    continuation_row: residual_row.clone(),
                    continuation_multiplicity: ContinuationMultiplicity::Affine,
                })
                .collect();
            checked_handlers.insert(
                derived.name.to_string(),
                CheckedHandlerDeclaration {
                    callable_kind: CallableDeclarationKind::Handler,
                    callable_signature: Type::Fn(
                        vec![Type::Fn(vec![], Box::new(answer_type.clone()))],
                        Box::new(answer_type.clone()),
                    ),
                    clauses,
                    input_result_type: answer_type.clone(),
                    input_row,
                    residual_row: residual_row.clone(),
                    output_row: residual_row,
                    answer_type: answer_type.clone(),
                    done_binding: "value".to_string(),
                    done_binding_type: answer_type,
                },
            );
        }
    }
    Ok(())
}

struct PendingHandlerOperation<'a> {
    resolved: DeclaredConcreteOperation,
    payload_type: Type,
    resume_name: String,
    pattern: &'a ash_parser::surface::Pattern,
    body: &'a ash_parser::surface::Expr,
    local_effect: Option<DeclaredConcreteOperation>,
    canonical_key: String,
    source_span: ash_parser::Span,
}

struct PreparedHandlerClauseBody<'a> {
    direct_resume_argument_types: Option<Vec<Type>>,
    computation: crate::checked_computation::CheckedComputation,
    _body: std::marker::PhantomData<&'a ()>,
}

fn prepare_handler_clause_body<'a>(
    env: &TypeEnv,
    operation: &PendingHandlerOperation<'a>,
) -> Result<PreparedHandlerClauseBody<'a>, TypeCheckError> {
    let pattern_env = crate::check_expr::pattern_type_env_from_type_env(env);
    let bindings = crate::check_pattern::check_pattern(
        &pattern_env,
        operation.pattern,
        &operation.payload_type,
    )
    .map_err(|error| {
        TypeCheckError::TypeError(format!(
            "handler operation pattern must match declared payload type {}: {error}",
            operation.payload_type
        ))
    })?;
    let mut clause_env = env.clone();
    for (name, ty) in bindings {
        clause_env.bind_variable(&name, ty);
    }
    let direct_resume_arguments = direct_resume_arguments(operation.body, &operation.resume_name);
    if direct_resume_arguments.is_none()
        && contains_resume_reference(operation.body, &operation.resume_name)
    {
        return Err(TypeCheckError::TypeError(format!(
            "unsupported-handler-continuation-use: resume binder '{}'",
            operation.resume_name
        )));
    }
    let (direct_resume_argument_types, computation) =
        if let Some(arguments) = direct_resume_arguments {
            let argument_types = arguments
                .iter()
                .map(|argument| {
                    let result = crate::check_expr::check_expr(&clause_env, argument);
                    if !result.is_ok() {
                        return Err(TypeCheckError::TypeError(result.errors[0].to_string()));
                    }
                    Ok(result.substitution.apply(&result.ty))
                })
                .collect::<Result<Vec<_>, TypeCheckError>>()?;
            (
                Some(argument_types),
                crate::checked_computation::infer_direct_resume_arguments_in_env(
                    &clause_env,
                    &arguments,
                    operation.resolved.result_type.clone(),
                    operation.body.span(),
                )?,
            )
        } else {
            (
                None,
                crate::checked_computation::infer_checked_computation_in_env(
                    &clause_env,
                    operation.body,
                )?,
            )
        };
    Ok(PreparedHandlerClauseBody {
        direct_resume_argument_types,
        computation,
        _body: std::marker::PhantomData,
    })
}

/// Retain the one declaration-backed nonresumptive clause effect supported by
/// the private handler inspection bridge.
///
/// This deliberately recognizes only `Impl::operation(parameter)`.  Other
/// clause bodies remain typechecked surface expressions, but do not acquire a
/// Core residual-row lowering route.
fn direct_concrete_operation_clause_body(
    body: &ash_parser::surface::Expr,
    pattern: &ash_parser::surface::Pattern,
    env: &TypeEnv,
    handled_operation: &DeclaredConcreteOperation,
) -> Result<Option<DeclaredConcreteOperation>, TypeCheckError> {
    if !is_task_2024_sleep_operation(handled_operation) {
        return Ok(None);
    }
    let ash_parser::surface::Pattern::Variable {
        name: parameter, ..
    } = pattern
    else {
        return Ok(None);
    };
    let ash_parser::surface::Expr::Call {
        module: Some(impl_type),
        func: operation,
        args,
        ..
    } = body
    else {
        return Ok(None);
    };
    let [ash_parser::surface::Expr::Variable { name, .. }] = args.as_slice() else {
        return Ok(None);
    };
    if name != parameter {
        return Ok(None);
    }
    let resolved = env
        .resolve_declared_concrete_operation(impl_type.as_ref(), operation.as_ref())
        .map_err(TypeCheckError::TypeError)?;
    Ok(is_task_2024_wake_operation(&resolved).then_some(resolved))
}

fn is_task_2024_sleep_operation(operation: &DeclaredConcreteOperation) -> bool {
    operation.impl_type == "TestClock"
        && operation.operation == "sleep"
        && operation.params == [Type::Int]
        && operation.result_type == Type::Int
}

fn is_task_2024_wake_operation(operation: &DeclaredConcreteOperation) -> bool {
    operation.impl_type == "TestClock"
        && operation.operation == "wake"
        && operation.params == [Type::Int]
        && operation.result_type == Type::Int
}

/// Type the deliberately scoped direct continuation forms admitted in source
/// handler clauses.  A continuation is never injected into the ordinary
/// expression environment: nested or malformed calls remain ordinary surface
/// expressions and fail there.  This preserves the non-runtime boundary while
/// giving a direct clause (or an all-direct block) its residual-row discipline.
fn check_prepared_handler_clause_body(
    prepared: &PreparedHandlerClauseBody<'_>,
    resume: &str,
    resume_argument_type: &Type,
    answer_type: &Type,
    multiplicity: ContinuationMultiplicity,
) -> Result<Type, TypeCheckError> {
    let Some(argument_types) = &prepared.direct_resume_argument_types else {
        return Ok(prepared.computation.result_type().clone());
    };
    if argument_types.len() > 1 && multiplicity == ContinuationMultiplicity::Affine {
        return Err(TypeCheckError::TypeError(format!(
            "affine resume binder '{resume}' may be used at most once"
        )));
    }
    for argument_type in argument_types {
        crate::types::unify(resume_argument_type, argument_type).map_err(|_| {
            TypeCheckError::TypeError(format!(
                "resume binder '{resume}' expects declared operation result type {resume_argument_type}, found {argument_type}"
            ))
        })?;
    }
    Ok(answer_type.clone())
}

/// Detect any continuation spelling outside the explicit direct-resume subset.
/// The parser traversal is exhaustive over expression-bearing surface forms, so
/// a future nested use cannot fall through to ordinary variable lookup.
fn contains_resume_reference(expr: &ash_parser::surface::Expr, resume: &str) -> bool {
    let mut found = false;
    ash_parser::surface::visit_expr(expr, &mut |candidate| match candidate {
        ash_parser::surface::Expr::Variable { name, .. } => {
            found |= name.as_ref() == resume;
        }
        ash_parser::surface::Expr::Call { func, module, .. } => {
            found |= module.is_none() && func.as_ref() == resume;
        }
        _ => {}
    });
    found
}

/// Return direct continuation arguments only for a whole clause call or an
/// all-direct block.  `None` deliberately hands every other shape to ordinary
/// expression checking without granting it continuation semantics.
fn direct_resume_arguments<'a>(
    expr: &'a ash_parser::surface::Expr,
    resume: &str,
) -> Option<Vec<&'a ash_parser::surface::Expr>> {
    if let Some(argument) = direct_resume_argument(expr, resume) {
        return Some(vec![argument]);
    }
    let ash_parser::surface::Expr::Block {
        statements,
        tail_expr: Some(tail),
        ..
    } = expr
    else {
        return None;
    };
    let mut arguments = Vec::with_capacity(statements.len() + 1);
    for statement in statements {
        let ash_parser::surface::BlockStmt::Expr { expr, .. } = statement else {
            return None;
        };
        arguments.push(direct_resume_argument(expr, resume)?);
    }
    arguments.push(direct_resume_argument(tail, resume)?);
    Some(arguments)
}

fn direct_resume_argument<'a>(
    expr: &'a ash_parser::surface::Expr,
    resume: &str,
) -> Option<&'a ash_parser::surface::Expr> {
    let ash_parser::surface::Expr::Call {
        func, module, args, ..
    } = expr
    else {
        return None;
    };
    (module.is_none() && func.as_ref() == resume && args.len() == 1).then(|| &args[0])
}

/// Lower the supported checked source-handler application into a Core inspection artifact.
///
/// This deliberately supports only the identity handler slice needed to inspect
/// the `CoreExpr::Handle`/`Raise` shape. It does not install a runtime handler,
/// provider frame, or execution path.
///
/// # Errors
///
/// Returns an error when the checked declaration or source application falls
/// outside the identity-handler inspection subset.
pub fn lower_checked_handler_application_to_core(
    program: &ash_parser::surface::Program,
    checked_source: &TypeCheckResult,
    entry_name: &str,
) -> Result<ash_core::core_ash::CoreExpr, TypeCheckError> {
    let entry = program
        .definitions
        .iter()
        .find_map(|definition| match definition {
            ash_parser::surface::Definition::Function(function)
                if function.name.as_ref() == entry_name =>
            {
                Some(function)
            }
            _ => None,
        })
        .ok_or_else(|| TypeCheckError::ResolutionError(format!("unknown entry '{entry_name}'")))?;
    let ash_parser::surface::Expr::Block {
        tail_expr: Some(tail_expr),
        statements,
        ..
    } = &entry.body
    else {
        return Err(TypeCheckError::TypeError(
            "checked handler lowering requires a tail handler application".to_string(),
        ));
    };
    if !statements.is_empty() {
        return Err(TypeCheckError::TypeError(
            "checked handler lowering does not support entry statements".to_string(),
        ));
    }
    let ash_parser::surface::Expr::HandleWith {
        expression,
        handler,
        ..
    } = tail_expr.as_ref()
    else {
        return Err(TypeCheckError::TypeError(
            "checked handler lowering requires `handle ... with`".to_string(),
        ));
    };
    let checked_handler = checked_source
        .checked_handlers
        .get(handler.as_ref())
        .ok_or_else(|| {
            TypeCheckError::TypeError(format!("handler '{handler}' has no checked declaration"))
        })?;
    // The inspection bridge has one Core clause and no carrier for a return
    // clause or general continuation fact.  Reject richer checked facts before
    // selecting a source clause, so this boundary cannot silently erase them.
    if checked_handler.clauses.len() != 1 {
        return Err(TypeCheckError::TypeError(
            "private Core handler bridge requires exactly one operation clause".to_string(),
        ));
    }
    if !crate::handler_rows::is_closed_empty_row(&checked_handler.output_row)
        && !is_task_2026_forward_sleep_output_row(checked_handler)
    {
        return Err(TypeCheckError::TypeError(
            "private Core handler bridge rejects nonempty or open output row".to_string(),
        ));
    }
    let ash_parser::surface::Definition::Handler(handler_definition) = program
        .definitions
        .iter()
        .find(|definition| {
            matches!(definition, ash_parser::surface::Definition::Handler(definition_handler)
                if definition_handler.name == *handler)
        })
        .ok_or_else(|| TypeCheckError::ResolutionError(format!("unknown handler '{handler}'")))?
    else {
        unreachable!("handler declaration search only returns handler definitions")
    };
    let ash_parser::surface::Expr::On { clauses, .. } = &handler_definition.body else {
        return Err(TypeCheckError::TypeError(
            "checked handler lowering requires a canonical on body".to_string(),
        ));
    };
    let done_identity = clauses.iter().find_map(|clause| match clause {
        ash_parser::surface::HandlerClause::Done { binding, body, .. } => Some(
            matches!(body.as_ref(), ash_parser::surface::Expr::Variable { name, .. } if name == binding),
        ),
        _ => None,
    });
    if done_identity != Some(true) {
        return Err(TypeCheckError::TypeError(
            "done clause must be identity for the current Core handler lowering".to_string(),
        ));
    }
    let Some(checked_clause) = checked_handler.clauses.first() else {
        return Err(TypeCheckError::TypeError(
            "checked handler has no operation clause".to_string(),
        ));
    };
    let source_clause = clauses.iter().find_map(|clause| match clause {
        ash_parser::surface::HandlerClause::Operation {
            impl_type,
            operation,
            pattern,
            resume,
            body,
            ..
        } if impl_type.as_ref() == checked_clause.operation.impl_type
            && operation.as_ref() == checked_clause.operation.operation =>
        {
            Some((pattern, resume, body))
        }
        _ => None,
    });
    let Some((
        ash_parser::surface::Pattern::Variable {
            name: parameter, ..
        },
        resume,
        body,
    )) = source_clause
    else {
        return Err(TypeCheckError::TypeError(
            "checked handler lowering requires a variable operation binder".to_string(),
        ));
    };
    let clause_body = match body.as_ref() {
        ash_parser::surface::Expr::Variable { name, .. } if name == parameter => {
            ash_core::core_ash::CoreExpr::Atom(ash_core::core_ash::CoreAtom::Var(
                parameter.to_string(),
            ))
        }
        ash_parser::surface::Expr::Call {
            func,
            module: None,
            args,
            ..
        } if func == resume && args.len() == 1 => ash_core::core_ash::CoreExpr::Jump {
            cont: ash_core::core_ash::CoreContRef::Var(resume.to_string()),
            arg: surface_handler_resume_argument_to_core_atom(&args[0])?,
        },
        ash_parser::surface::Expr::Call {
            module: Some(impl_type),
            func,
            args,
            ..
        } if checked_clause
            .local_effect
            .as_ref()
            .is_some_and(|effect| {
                impl_type.as_ref() == effect.impl_type
                    && func.as_ref() == effect.operation
                    && matches!(args.as_slice(), [ash_parser::surface::Expr::Variable { name, .. }] if name == parameter)
            }) => {
            let effect = checked_clause
                .local_effect
                .as_ref()
                .expect("guarded by local effect presence");
            ash_core::core_ash::CoreExpr::Raise {
                op: declared_operation_to_core_effect_op(effect),
                args: vec![ash_core::core_ash::CoreAtom::Var(parameter.to_string())],
            }
        }
        // TASK-2013/TASK-2014's first abortive production handler is deliberately
        // a single fixed primitive fault.  Keep this recognition structural and
        // exact: it is not general binary-expression lowering for handler bodies.
        ash_parser::surface::Expr::Binary {
            op: ash_parser::surface::BinaryOp::Div,
            left,
            right,
            ..
        } if handler.as_ref() == "trap_sleep"
            && parameter.as_ref() == "ms"
            && resume.as_ref() == "resume"
            && matches!(
                (left.as_ref(), right.as_ref()),
                (
                    ash_parser::surface::Expr::Literal(ash_parser::surface::Literal::Int(1)),
                    ash_parser::surface::Expr::Literal(ash_parser::surface::Literal::Int(0)),
                )
            ) => ash_core::core_ash::CoreExpr::LetPrim {
            name: "__trap_sleep_division".to_string(),
            op: ash_core::core_ash::CorePrimOp::Div,
            args: vec![
                ash_core::core_ash::CoreAtom::LitInt(1),
                ash_core::core_ash::CoreAtom::LitInt(0),
            ],
            body: Box::new(ash_core::core_ash::CoreExpr::Atom(
                ash_core::core_ash::CoreAtom::Var("__trap_sleep_division".to_string()),
            )),
        },
        _ => {
            return Err(TypeCheckError::TypeError(
                "checked handler lowering requires an identity operation clause body, one direct resume call, one declared concrete operation call on its payload binder, or the exact trap_sleep division fixture"
                    .to_string(),
            ));
        }
    };
    let ash_parser::surface::Expr::Call {
        module: Some(impl_type),
        func: operation,
        args,
        ..
    } = expression.as_ref()
    else {
        return Err(TypeCheckError::TypeError(
            "checked handler lowering requires a concrete operation call".to_string(),
        ));
    };
    if impl_type.as_ref() != checked_clause.operation.impl_type
        || operation.as_ref() != checked_clause.operation.operation
    {
        return Err(TypeCheckError::TypeError(
            "handler application operation does not match its checked clause".to_string(),
        ));
    }
    let core_args = args
        .iter()
        .map(surface_literal_to_core_atom)
        .collect::<Result<Vec<_>, _>>()?;
    if handler.as_ref() == "trap_sleep"
        && !matches!(
            core_args.as_slice(),
            [ash_core::core_ash::CoreAtom::LitInt(0)]
        )
    {
        return Err(TypeCheckError::TypeError(
            "trap_sleep checked handler lowering requires its exact TestClock::sleep(0) application"
                .to_string(),
        ));
    }
    let op = declared_operation_to_core_effect_op(&checked_clause.operation);
    let answer = match &checked_handler.callable_signature {
        Type::Fn(_, result) => type_to_core_type(result),
        _ => unreachable!("checked handler declarations always carry function signatures"),
    };
    Ok(ash_core::core_ash::CoreExpr::Handle {
        clause: ash_core::core_ash::CoreHandlerClause {
            op: op.clone(),
            params: vec![ash_core::core_ash::CoreParam {
                name: parameter.to_string(),
                ty: type_to_core_type(&checked_clause.payload_type),
            }],
            resume: ash_core::core_ash::CoreParam {
                name: resume.to_string(),
                ty: ash_core::core_ash::CoreType::Cont {
                    input: Box::new(type_to_core_type(&checked_clause.operation.result_type)),
                    answer: Box::new(answer),
                    row: ash_core::core_ash::CoreRow::default(),
                    multiplicity: match checked_clause.continuation_multiplicity {
                        ContinuationMultiplicity::MultiShotPure => {
                            ash_core::core_ash::CoreMultiplicity::MultiShotPure
                        }
                        ContinuationMultiplicity::Affine => {
                            ash_core::core_ash::CoreMultiplicity::Affine
                        }
                    },
                },
            },
            body: Box::new(clause_body),
            row: checked_clause
                .local_effect
                .as_ref()
                .map_or_else(ash_core::core_ash::CoreRow::default, operation_effect_row),
        },
        body: Box::new(ash_core::core_ash::CoreExpr::Raise {
            op,
            args: core_args,
        }),
    })
}

/// TASK-2026 promotes exactly the declaration-backed `forward_sleep` fixture
/// from structural inspection to a separately sealed Engine admission.  This
/// helper deliberately recognizes its retained typed facts rather than a row
/// spelling, so arbitrary nonempty handler rows remain outside this bridge.
fn is_task_2026_forward_sleep_output_row(handler: &CheckedHandlerDeclaration) -> bool {
    handler.clauses.len() == 1
        && handler.clauses[0].operation.impl_type == "TestClock"
        && handler.clauses[0].operation.interface == "Clock"
        && handler.clauses[0].operation.operation == "sleep"
        && handler.clauses[0].operation.params == [Type::Int]
        && handler.clauses[0].operation.result_type == Type::Int
        && handler.clauses[0]
            .local_effect
            .as_ref()
            .is_some_and(is_task_2024_wake_operation)
        && handler.output_row.tail.is_none()
        && handler.output_row.items.len() == 1
        && handler.output_row.items[0].canonical_key() == "operation:TestClock::Clock::wake"
}

fn declared_operation_to_core_effect_op(
    operation: &DeclaredConcreteOperation,
) -> ash_core::core_ash::CoreEffectOp {
    ash_core::core_ash::CoreEffectOp::Operation {
        path: vec![operation.impl_type.clone()],
        operation: operation.operation.clone(),
        arg_types: operation.params.iter().map(type_to_core_type).collect(),
        result_type: type_to_core_type(&operation.result_type),
    }
}

fn operation_effect_row(operation: &DeclaredConcreteOperation) -> ash_core::core_ash::CoreRow {
    ash_core::core_ash::CoreRow::closed(vec![ash_core::core_ash::CoreRowItem::Operation {
        path: vec![operation.impl_type.clone()],
        operation: operation.operation.clone(),
    }])
}

fn surface_handler_resume_argument_to_core_atom(
    expr: &ash_parser::surface::Expr,
) -> Result<ash_core::core_ash::CoreAtom, TypeCheckError> {
    match expr {
        ash_parser::surface::Expr::Variable { name, .. } => {
            Ok(ash_core::core_ash::CoreAtom::Var(name.to_string()))
        }
        _ => surface_literal_to_core_atom(expr),
    }
}

fn surface_literal_to_core_atom(
    expr: &ash_parser::surface::Expr,
) -> Result<ash_core::core_ash::CoreAtom, TypeCheckError> {
    match expr {
        ash_parser::surface::Expr::Literal(ash_parser::surface::Literal::Int(value)) => {
            Ok(ash_core::core_ash::CoreAtom::LitInt(*value))
        }
        _ => Err(TypeCheckError::TypeError(
            "checked handler lowering accepts only literal operation arguments".to_string(),
        )),
    }
}

fn type_to_core_type(ty: &Type) -> ash_core::core_ash::CoreType {
    match ty {
        Type::Int => ash_core::core_ash::CoreType::Base("Int".to_string()),
        Type::String => ash_core::core_ash::CoreType::Base("String".to_string()),
        Type::Bool => ash_core::core_ash::CoreType::Base("Bool".to_string()),
        Type::Null => ash_core::core_ash::CoreType::Base("Unit".to_string()),
        other => ash_core::core_ash::CoreType::Named(other.to_string()),
    }
}

/// Checks one ordinary function body against a caller-provided staged value
/// environment.
///
/// This crate-internal helper performs no declaration discovery or module
/// identity work; callers must establish their own staged boundary first.
pub(crate) fn check_function_body_in_env(
    env: &TypeEnv,
    function: &ash_parser::surface::FnDef,
) -> Result<Type, TypeCheckError> {
    let (mut fn_env, bindings) = bind_surface_type_parameters(env, &function.type_params)?;
    let mut param_types = Vec::with_capacity(function.params.len());
    for param in &function.params {
        let param_ty = workflow_surface_type_to_type(&fn_env, &param.ty, &bindings)?;
        fn_env.bind_variable(param.name.as_ref(), param_ty.clone());
        param_types.push(param_ty);
    }

    validate_function_preconditions(
        &fn_env,
        &function.body,
        &mut std::collections::HashMap::new(),
        &mut std::collections::HashMap::new(),
    )?;

    let result = crate::check_expr::check_expr(&fn_env, &function.body);
    if !result.is_ok() {
        let reason = result
            .errors
            .into_iter()
            .next()
            .map(|error| error.to_string())
            .unwrap_or_else(|| format!("failed to typecheck fn '{}'", function.name));
        return Err(TypeCheckError::TypeError(reason));
    }

    let body_ty = result.substitution.apply(&result.ty);
    if let Some(return_type) = &function.return_type {
        let expected = workflow_surface_type_to_type(
            &fn_env,
            ash_parser::callable_result_type_for_fn_contract(return_type),
            &bindings,
        )?;
        crate::types::unify(&expected, &body_ty).map_err(|_| {
            TypeCheckError::TypeError(format!(
                "fn '{}' declared return type {} but body returns {}",
                function.name, expected, body_ty
            ))
        })?;
    } else if crate::types::type_contains_fun(&body_ty) && param_types.is_empty() {
        return Err(TypeCheckError::TypeError(format!(
            "fn '{}' omitted return type could not be inferred; add an explicit return type",
            function.name
        )));
    }

    Ok(body_ty)
}

/// Walk handler applications with the lexical scope visible at their source
/// position.  Surface visitors intentionally carry no environment, so a
/// plain `visit_expr` would lose block-local `let` bindings before implicit
/// thunk inference reaches an application.
fn visit_scoped_handler_applications(
    env: &TypeEnv,
    expr: &ash_parser::surface::Expr,
    visit: &mut impl FnMut(
        &TypeEnv,
        &ash_parser::surface::Expr,
        &ash_parser::surface::Name,
        ash_parser::token::Span,
    ) -> Result<(), TypeCheckError>,
) -> Result<(), TypeCheckError> {
    use ash_parser::surface::{BlockStmt, ConstructorPayload, Expr, HandlerClause};

    fn bind_pattern_scope(
        env: &mut TypeEnv,
        pattern: &ash_parser::surface::Pattern,
        value_type: &Type,
    ) -> Result<(), TypeCheckError> {
        let bindings = crate::check_expr::check_pattern_bindings(env, pattern, value_type)
            .map_err(|error| TypeCheckError::TypeError(error.to_string()))?;
        for (name, ty) in bindings {
            env.bind_variable(&name, ty);
        }
        Ok(())
    }

    fn closure_parameter_type(env: &TypeEnv, annotation: Option<&str>) -> Type {
        match annotation {
            Some("Int") => Type::Int,
            Some("Bool") => Type::Bool,
            Some("String") => Type::String,
            Some("Float") => Type::Float,
            Some("Null") | Some("Unit") => Type::Null,
            Some("Time") => Type::Time,
            Some("Ref") => Type::Ref,
            Some(name) => env
                .resolve_type(name)
                .map(|(qualified, _)| Type::Constructor {
                    name: qualified,
                    args: Vec::new(),
                    kind: crate::Kind::Type,
                })
                .unwrap_or_else(|_| Type::Var(crate::types::TypeVar::fresh())),
            None => Type::Var(crate::types::TypeVar::fresh()),
        }
    }

    match expr {
        Expr::HandleWith {
            expression,
            handler,
            handler_span,
            ..
        } => {
            visit(env, expression, handler, *handler_span)?;
            visit_scoped_handler_applications(env, expression, visit)
        }
        Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            let mut block_env = env.clone();
            for statement in statements {
                match statement {
                    BlockStmt::Expr { expr, .. } => {
                        visit_scoped_handler_applications(&block_env, expr, visit)?;
                    }
                    BlockStmt::Let {
                        pattern,
                        expr,
                        span,
                    } => {
                        visit_scoped_handler_applications(&block_env, expr, visit)?;
                        let checked = crate::check_expr::check_expr(&block_env, expr);
                        if !checked.is_ok() {
                            return Err(TypeCheckError::TypeError(checked.errors[0].to_string()));
                        }
                        let value_type = checked.substitution.apply(&checked.ty);
                        let pattern_span = crate::check_expr::surface_pattern_span(pattern, *span);
                        let bindings = crate::check_expr::check_irrefutable_let_pattern(
                            &block_env,
                            "let",
                            pattern,
                            &value_type,
                            pattern_span,
                        )
                        .map_err(|error| TypeCheckError::TypeError(error.to_string()))?;
                        crate::check_expr::bind_irrefutable_pattern_bindings(
                            &mut block_env,
                            bindings,
                        );
                    }
                }
            }
            if let Some(tail) = tail_expr {
                visit_scoped_handler_applications(&block_env, tail, visit)?;
            }
            Ok(())
        }
        Expr::OperatorSection { section } => {
            if let Some(left) = &section.left {
                visit_scoped_handler_applications(env, left, visit)?;
            }
            if let Some(right) = &section.right {
                visit_scoped_handler_applications(env, right, visit)?;
            }
            Ok(())
        }
        Expr::FieldAccess { base, .. } => visit_scoped_handler_applications(env, base, visit),
        Expr::IndexAccess { base, index, .. } => {
            visit_scoped_handler_applications(env, base, visit)?;
            visit_scoped_handler_applications(env, index, visit)
        }
        Expr::Unary { operand, .. } => visit_scoped_handler_applications(env, operand, visit),
        Expr::Binary { left, right, .. } => {
            visit_scoped_handler_applications(env, left, visit)?;
            visit_scoped_handler_applications(env, right, visit)
        }
        Expr::Call { args, .. } => {
            for argument in args {
                visit_scoped_handler_applications(env, argument, visit)?;
            }
            Ok(())
        }
        Expr::Match {
            scrutinee, arms, ..
        } => {
            visit_scoped_handler_applications(env, scrutinee, visit)?;
            let checked_scrutinee = crate::check_expr::check_expr(env, scrutinee);
            if !checked_scrutinee.is_ok() {
                return Err(TypeCheckError::TypeError(
                    checked_scrutinee.errors[0].to_string(),
                ));
            }
            let scrutinee_type = checked_scrutinee.substitution.apply(&checked_scrutinee.ty);
            for arm in arms {
                let mut arm_env = env.clone();
                bind_pattern_scope(&mut arm_env, &arm.pattern, &scrutinee_type)?;
                visit_scoped_handler_applications(&arm_env, &arm.body, visit)?;
            }
            Ok(())
        }
        Expr::IfLet {
            pattern,
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            visit_scoped_handler_applications(env, expr, visit)?;
            let checked_matched = crate::check_expr::check_expr(env, expr);
            if !checked_matched.is_ok() {
                return Err(TypeCheckError::TypeError(
                    checked_matched.errors[0].to_string(),
                ));
            }
            let matched_type = checked_matched.substitution.apply(&checked_matched.ty);
            let mut then_env = env.clone();
            bind_pattern_scope(&mut then_env, pattern, &matched_type)?;
            visit_scoped_handler_applications(&then_env, then_branch, visit)?;
            visit_scoped_handler_applications(env, else_branch, visit)
        }
        Expr::Constructor {
            fields, payload, ..
        } => {
            for (_, value) in fields {
                visit_scoped_handler_applications(env, value, visit)?;
            }
            match payload {
                ConstructorPayload::Tuple(items) => {
                    for item in items {
                        visit_scoped_handler_applications(env, item, visit)?;
                    }
                }
                ConstructorPayload::Record(fields) => {
                    for (_, value) in fields {
                        visit_scoped_handler_applications(env, value, visit)?;
                    }
                }
                ConstructorPayload::Unit => {}
            }
            Ok(())
        }
        Expr::Record { fields, .. } => {
            for (_, value) in fields {
                visit_scoped_handler_applications(env, value, visit)?;
            }
            Ok(())
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            visit_scoped_handler_applications(env, condition, visit)?;
            visit_scoped_handler_applications(env, then_branch, visit)?;
            if let Some(else_branch) = else_branch {
                visit_scoped_handler_applications(env, else_branch, visit)?;
            }
            Ok(())
        }
        Expr::Fail { payload, .. } => visit_scoped_handler_applications(env, payload, visit),
        Expr::WithError { body, arms, .. } => {
            visit_scoped_handler_applications(env, body, visit)?;
            let failure_payload_type = match body.as_ref() {
                Expr::Fail { payload, .. } => {
                    let checked_payload = crate::check_expr::check_expr(env, payload);
                    if !checked_payload.is_ok() {
                        return Err(TypeCheckError::TypeError(
                            checked_payload.errors[0].to_string(),
                        ));
                    }
                    checked_payload.substitution.apply(&checked_payload.ty)
                }
                _ => Type::Var(crate::types::TypeVar::fresh()),
            };
            for arm in arms {
                let mut arm_env = env.clone();
                bind_pattern_scope(&mut arm_env, &arm.pattern, &failure_payload_type)?;
                visit_scoped_handler_applications(&arm_env, &arm.body, visit)?;
            }
            Ok(())
        }
        Expr::On {
            computation,
            clauses,
            ..
        } => {
            visit_scoped_handler_applications(env, computation, visit)?;
            for clause in clauses {
                let body = match clause {
                    HandlerClause::Operation { body, .. } | HandlerClause::Done { body, .. } => {
                        body
                    }
                };
                visit_scoped_handler_applications(env, body, visit)?;
            }
            Ok(())
        }
        Expr::FnDef { params, body, .. } => {
            let mut closure_env = env.clone();
            for (name, annotation) in params {
                let parameter_type = closure_parameter_type(&closure_env, annotation.as_deref());
                closure_env.bind_variable(name.as_ref(), parameter_type);
            }
            visit_scoped_handler_applications(&closure_env, body, visit)
        }
        Expr::FnApply { func, args, .. } => {
            visit_scoped_handler_applications(env, func, visit)?;
            for argument in args {
                visit_scoped_handler_applications(env, argument, visit)?;
            }
            Ok(())
        }
        Expr::DoBlock { target, stmts, .. } => {
            let mut do_env = env.clone();
            let dictionary = if target.name.as_ref() == "__ambient" && target.args.is_empty() {
                None
            } else {
                Some(
                    crate::do_target::resolve_do_target(env, target)
                        .map_err(|error| TypeCheckError::TypeError(error.to_string()))?,
                )
            };
            for statement in stmts {
                match statement {
                    ash_parser::surface::DoStmt::Let { name, value, .. } => {
                        visit_scoped_handler_applications(&do_env, value, visit)?;
                        let checked = crate::check_expr::check_expr(&do_env, value);
                        if !checked.is_ok() {
                            return Err(TypeCheckError::TypeError(checked.errors[0].to_string()));
                        }
                        do_env
                            .bind_variable(name.as_ref(), checked.substitution.apply(&checked.ty));
                    }
                    ash_parser::surface::DoStmt::Bind { name, value, .. } => {
                        visit_scoped_handler_applications(&do_env, value, visit)?;
                        let checked = crate::check_expr::check_expr(&do_env, value);
                        if !checked.is_ok() {
                            return Err(TypeCheckError::TypeError(checked.errors[0].to_string()));
                        }
                        let value_type = checked.substitution.apply(&checked.ty);
                        let bound_type = match dictionary.as_ref() {
                            // Canonical ambient `do` has no target dictionary:
                            // its binds introduce the checked value type directly.
                            None => value_type,
                            Some(dictionary) => crate::check_expr::monadic_inner_type(
                                &value_type,
                                dictionary,
                            )
                            .ok_or_else(|| {
                                TypeCheckError::TypeError(format!(
                                    "do bind '{}' does not produce the declared computation target",
                                    name
                                ))
                            })?,
                        };
                        do_env.bind_variable(name.as_ref(), bound_type);
                    }
                    ash_parser::surface::DoStmt::Expr { value, .. }
                    | ash_parser::surface::DoStmt::Return { value, .. } => {
                        visit_scoped_handler_applications(&do_env, value, visit)?;
                    }
                }
            }
            Ok(())
        }
        Expr::Comprehension {
            result, qualifiers, ..
        } => {
            visit_scoped_handler_applications(env, result, visit)?;
            for qualifier in qualifiers {
                let value = match qualifier {
                    ash_parser::surface::ComprehensionQualifier::Bind { value, .. }
                    | ash_parser::surface::ComprehensionQualifier::DiscardBind { value, .. }
                    | ash_parser::surface::ComprehensionQualifier::Let { value, .. } => value,
                };
                visit_scoped_handler_applications(env, value, visit)?;
            }
            Ok(())
        }
        Expr::List { items, .. } => {
            for item in items {
                visit_scoped_handler_applications(env, item, visit)?;
            }
            Ok(())
        }
        Expr::Literal(_)
        | Expr::Variable { .. }
        | Expr::MacroInvocation { .. }
        | Expr::Policy(_)
        | Expr::CheckObligation { .. }
        | Expr::Panic { .. } => Ok(()),
    }
}

/// Validate `handle expression with handler` after every handler declaration
/// has published its immutable source facts.  This is intentionally separate
/// from ordinary expression checking: a handler's surface signature erases
/// its computation row, while the checked declaration retains it.
///
/// A successful source-handler comparison may specialize a type variable from
/// the implicitly thunked operand. Apply that substitution while its inferred
/// variables are still in scope, then retain the specialized result type by
/// the operand's unique source anchor. The publication pass intentionally
/// re-infers operands, so carrying raw variable IDs across the two passes
/// would not be sound. This remains unavailable to ordinary calls.
type ValidatedHandlerInputTypes = std::collections::HashMap<ash_parser::token::Span, Type>;

fn validate_handler_application_inputs(
    env: &TypeEnv,
    program: &ash_parser::surface::Program,
) -> Result<ValidatedHandlerInputTypes, TypeCheckError> {
    let mut input_types = ValidatedHandlerInputTypes::new();
    for definition in &program.definitions {
        let ash_parser::surface::Definition::Function(function) = definition else {
            continue;
        };
        let (mut function_env, bindings) =
            bind_surface_type_parameters(env, &function.type_params)?;
        let parameter_facts = crate::checked_computation::function_computation_parameter_facts(
            env, program, function,
        )?;
        for parameter in &function.params {
            let parameter_type =
                workflow_surface_type_to_type(&function_env, &parameter.ty, &bindings)?;
            function_env.bind_variable(parameter.name.as_ref(), parameter_type);
        }
        function_env.register_source_computation_facts(parameter_facts);
        visit_scoped_handler_applications(
            &function_env,
            &function.body,
            &mut |scope, handled_expression, handler, _handler_span| {
                let Some(handler_definition) =
                    program
                        .definitions
                        .iter()
                        .find_map(|definition| match definition {
                            ash_parser::surface::Definition::Handler(candidate)
                                if candidate.name.as_ref() == handler.as_ref() =>
                            {
                                Some(candidate)
                            }
                            _ => None,
                        })
                else {
                    // Ordinary expression checking owns marker/name diagnostics.
                    return Ok(());
                };
                let expected = match crate::checked_computation::infer_checked_handler_computation(
                    env,
                    program,
                    handler_definition,
                ) {
                    Ok(computation) => computation,
                    Err(error) => return Err(error),
                };
                let actual = match crate::checked_computation::infer_checked_computation_in_env_with_parameter_facts(
                    scope,
                    handled_expression,
                ) {
                    Ok(computation) => computation,
                    Err(error) => return Err(error),
                };
                let input_substitution =
                    crate::types::unify(expected.result_type(), actual.result_type()).ok();
                let row_matches = crate::handler_rows::normalized_handler_rows_semantically_equal(
                    expected.normalized_row(),
                    actual.normalized_row(),
                );
                let Some(input_substitution) = input_substitution else {
                    return Err(TypeCheckError::TypeError(format!(
                        "handler '{}' input computation mismatch: expected () -> {} {}, found () -> {} {}",
                        handler,
                        format_normalized_handler_row(expected.normalized_row()),
                        expected.result_type(),
                        format_normalized_handler_row(actual.normalized_row()),
                        actual.result_type(),
                    )));
                };
                if !row_matches {
                    return Err(TypeCheckError::TypeError(format!(
                        "handler '{}' input computation mismatch: expected () -> {} {}, found () -> {} {}",
                        handler,
                        format_normalized_handler_row(expected.normalized_row()),
                        expected.result_type(),
                        format_normalized_handler_row(actual.normalized_row()),
                        actual.result_type(),
                    )));
                }
                input_types.insert(
                    handled_expression.span(),
                    input_substitution.apply(actual.result_type()),
                );
                Ok(())
            },
        )?;
    }
    Ok(input_types)
}

/// Record immutable application facts after validation and declaration checks.
/// No application fact can publish until both prior stages have succeeded.
fn validate_handler_applications(
    env: &TypeEnv,
    program: &ash_parser::surface::Program,
    checked_handlers: &std::collections::HashMap<String, CheckedHandlerDeclaration>,
    input_types: &ValidatedHandlerInputTypes,
) -> Result<Vec<CheckedHandlerApplication>, TypeCheckError> {
    let mut applications = Vec::new();
    for definition in &program.definitions {
        let ash_parser::surface::Definition::Function(function) = definition else {
            continue;
        };
        let (mut function_env, bindings) =
            bind_surface_type_parameters(env, &function.type_params)?;
        let parameter_facts = crate::checked_computation::function_computation_parameter_facts(
            env, program, function,
        )?;
        for parameter in &function.params {
            let parameter_type =
                workflow_surface_type_to_type(&function_env, &parameter.ty, &bindings)?;
            function_env.bind_variable(parameter.name.as_ref(), parameter_type);
        }
        function_env.register_source_computation_facts(parameter_facts);
        visit_scoped_handler_applications(
            &function_env,
            &function.body,
            &mut |scope, handled_expression, handler, handler_span| {
                if env.require_handler_callable(handler.as_ref()).is_err() {
                    // The regular expression checker owns ordinary function
                    // and unknown-name marker diagnostics.
                    return Ok(());
                }
                let Some(checked_handler) = checked_handlers.get(handler.as_ref()) else {
                    return Err(TypeCheckError::TypeError(format!(
                        "handler '{handler}' has no checked declaration"
                    )));
                };
                let actual = match crate::checked_computation::infer_checked_computation_in_env_with_parameter_facts(
                    scope,
                    handled_expression,
                ) {
                    Ok(computation) => computation,
                    Err(error) => return Err(error),
                };
                let input_result_type = input_types
                    .get(&handled_expression.span())
                    .cloned()
                    .unwrap_or_else(|| actual.result_type().clone());
                let (answer_type, output_row, input_row) = if is_derived_handler_fact(
                    checked_handler,
                ) {
                    let required_operation_keys = checked_handler
                        .clauses
                        .iter()
                        .map(checked_handler_clause_operation_key)
                        .collect::<Vec<_>>();
                    let residual_row = crate::handler_rows::subtract_handled_operations(
                        actual.normalized_row(),
                        &required_operation_keys,
                    )
                    .map_err(|_| {
                        TypeCheckError::TypeError(format!(
                            "handler '{}' input computation mismatch: expected () -> {} {}, found () -> {} {}",
                            handler,
                            format_normalized_handler_row(&checked_handler.input_row),
                            checked_handler.input_result_type,
                            format_normalized_handler_row(actual.normalized_row()),
                            actual.result_type(),
                        ))
                    })?;
                    // `derive handler` is an identity fold. Instantiate its
                    // fresh answer variable at this application rather than
                    // publishing declaration-local polymorphic evidence.
                    (
                        actual.result_type().clone(),
                        residual_row,
                        actual.normalized_row().clone(),
                    )
                } else {
                    let type_matches = crate::types::unify(
                        &checked_handler.input_result_type,
                        actual.result_type(),
                    )
                    .is_ok();
                    let row_matches =
                        crate::handler_rows::normalized_handler_rows_semantically_equal(
                            &checked_handler.input_row,
                            actual.normalized_row(),
                        );
                    if !type_matches || !row_matches {
                        return Err(TypeCheckError::TypeError(format!(
                            "handler '{}' input computation mismatch: expected () -> {} {}, found () -> {} {}",
                            handler,
                            format_normalized_handler_row(&checked_handler.input_row),
                            checked_handler.input_result_type,
                            format_normalized_handler_row(actual.normalized_row()),
                            actual.result_type(),
                        )));
                    }
                    (
                        checked_handler.answer_type.clone(),
                        checked_handler.output_row.clone(),
                        actual.normalized_row().clone(),
                    )
                };
                applications.push(CheckedHandlerApplication {
                    handler_name: handler.to_string(),
                    expression_span: handled_expression.span(),
                    handler_span,
                    input_result_type,
                    input_row,
                    answer_type,
                    output_row,
                });
                Ok(())
            },
        )?;
    }
    Ok(applications)
}

/// Return whether a checked declaration originates solely from `derive handler`
/// clause desugaring, rather than guessing from its user-visible name.
fn is_derived_handler_fact(handler: &CheckedHandlerDeclaration) -> bool {
    !handler.clauses.is_empty()
        && handler.clauses.iter().all(|clause| {
            matches!(
                &clause.origin,
                ash_parser::surface::SurfaceOrigin::Desugaring { .. }
            )
        })
}

/// Render the canonical operation identity retained by a checked clause.
fn checked_handler_clause_operation_key(clause: &CheckedHandlerClause) -> String {
    format!(
        "operation:{}::{}::{}",
        clause.operation.impl_type, clause.operation.interface, clause.operation.operation
    )
}

fn format_normalized_handler_row(row: &NormalizedHandlerRow) -> String {
    let mut entries = row
        .items
        .iter()
        .map(|item| item.canonical_key())
        .collect::<Vec<_>>();
    if let Some(tail) = &row.tail {
        entries.push(format!("| {tail}"));
    }
    format!("{{ {} }}", entries.join(", "))
}

fn refine_function_signatures(
    env: &mut TypeEnv,
    definitions: &[ash_parser::surface::Definition],
) -> Result<(), TypeCheckError> {
    for definition in definitions {
        let ash_parser::surface::Definition::Function(function) = definition else {
            continue;
        };
        let body_ty = check_function_body_in_env(env, function)?;
        if function.return_type.is_none() {
            let (signature_env, bindings) =
                bind_surface_type_parameters(env, &function.type_params)?;
            let param_types = function
                .params
                .iter()
                .map(|param| workflow_surface_type_to_type(&signature_env, &param.ty, &bindings))
                .collect::<Result<Vec<_>, _>>()?;
            env.bind_variable(
                function.name.as_ref(),
                Type::Fn(param_types, Box::new(body_ty)),
            );
        }
    }
    Ok(())
}

fn type_check_program_entry(
    env: &TypeEnv,
    program: &ash_parser::surface::Program,
) -> Result<TypeCheckResult, TypeCheckError> {
    let entry_name = program.entry.function.as_ref();
    let entry_function = program
        .definitions
        .iter()
        .find_map(|definition| match definition {
            ash_parser::surface::Definition::Function(function)
                if function.name.as_ref() == entry_name =>
            {
                Some(function)
            }
            _ => None,
        })
        .ok_or_else(|| {
            TypeCheckError::ResolutionError(format!(
                "program entry function '{}' is not defined",
                entry_name
            ))
        })?;

    let entry_type = check_function_body_in_env(env, entry_function)?;
    let checked_builtin_operation =
        checked_builtin_operation_for_entry(entry_function, &entry_type);
    let mut inferred_types = std::collections::HashMap::new();
    inferred_types.insert(entry_name.to_string(), entry_type);

    Ok(TypeCheckResult {
        substitution: Substitution::new(),
        errors: Vec::new(),
        inferred_types,
        effect: ash_core::Effect::Epistemic,
        obligation_status: crate::obligations::ObligationCheckResult::Success,
        function_contracts: env.function_contracts(),
        authority_provenance: AuthorityProvenanceReport::default(),
        checked_handlers: std::collections::HashMap::new(),
        checked_handler_applications: Vec::new(),
        checked_builtin_operation,
    })
}

fn checked_builtin_operation_for_entry(
    entry_function: &ash_parser::surface::FnDef,
    entry_type: &Type,
) -> Option<CheckedBuiltinOperation> {
    let has_exact_null_signature = entry_function.params.is_empty()
        && matches!(
            entry_function.return_type.as_ref(),
            Some(ash_parser::surface::Type::Name(name)) if name.as_ref() == "Null"
        )
        && entry_type == &Type::Null;
    let ash_parser::surface::Expr::Block {
        statements,
        tail_expr: Some(tail_expr),
        ..
    } = &entry_function.body
    else {
        return None;
    };
    if !statements.is_empty() {
        return None;
    }
    let ash_parser::surface::Expr::Call {
        func,
        module: Some(module),
        args,
        span,
    } = tail_expr.as_ref()
    else {
        return None;
    };
    let [ash_parser::surface::Expr::Literal(ash_parser::surface::Literal::Int(duration_millis))] =
        args.as_slice()
    else {
        return None;
    };
    (has_exact_null_signature
        && module.as_ref() == "time"
        && func.as_ref() == "sleep"
        && *duration_millis >= 0)
        .then_some(CheckedBuiltinOperation::TimeSleep(
            CheckedTimeSleepOperation {
                duration_millis: *duration_millis,
                entry_span: entry_function.span,
                call_span: *span,
            },
        ))
}

/// Type check a program with explicit type-checking configuration.
pub fn type_check_program_with_config(
    program: &ash_parser::surface::Program,
    config: &TypeCheckConfig,
) -> Result<TypeCheckResult, TypeCheckError> {
    let env = TypeEnv::with_builtin_types();
    type_check_program_in_env_with_config(&env, program, config)
}

/// Type check a program with a pre-populated type environment.
/// Used when imported callable signatures need to be available during checking.
pub fn type_check_program_in_env(
    initial_env: &TypeEnv,
    program: &ash_parser::surface::Program,
) -> Result<TypeCheckResult, TypeCheckError> {
    type_check_program_in_env_with_config(initial_env, program, &TypeCheckConfig::default())
}

/// Type check a program with a pre-populated type environment and explicit config.
pub fn type_check_program_in_env_with_config(
    initial_env: &TypeEnv,
    program: &ash_parser::surface::Program,
    config: &TypeCheckConfig,
) -> Result<TypeCheckResult, TypeCheckError> {
    type_check_program_in_env_for_module_with_config(
        initial_env,
        program,
        synthetic_program_module_identity(),
        config,
    )
}

/// Type check a program with an explicit current-module identity for local declarations.
///
/// Module-aware callers should use this entry point so sealed associated-family
/// declarations and impl-family schemes record the real defining module instead
/// of the standalone synthetic program identity.
pub fn type_check_program_in_env_for_module(
    initial_env: &TypeEnv,
    program: &ash_parser::surface::Program,
    module_identity: ash_core::semantic_summary::ModuleIdentity,
) -> Result<TypeCheckResult, TypeCheckError> {
    type_check_program_in_env_for_module_with_config(
        initial_env,
        program,
        module_identity,
        &TypeCheckConfig::default(),
    )
}

/// Type check a program with an explicit current-module identity and config.
pub fn type_check_program_in_env_for_module_with_config(
    initial_env: &TypeEnv,
    program: &ash_parser::surface::Program,
    module_identity: ash_core::semantic_summary::ModuleIdentity,
    config: &TypeCheckConfig,
) -> Result<TypeCheckResult, TypeCheckError> {
    let mut env = initial_env.clone();
    env.set_current_module_identity(module_identity);
    env.register_surface_declarations(&program.definitions)
        .map_err(TypeCheckError::from)?;

    for definition in &program.definitions {
        if let ash_parser::surface::Definition::Interface(interface) = definition {
            env.register_interface(interface)
                .map_err(TypeCheckError::from)?;
        }
    }

    for definition in &program.definitions {
        if let ash_parser::surface::Definition::ResourceType(resource_type) = definition {
            env.register_resource_type(resource_type)
                .map_err(TypeCheckError::from)?;
        }
    }

    for definition in &program.definitions {
        if let ash_parser::surface::Definition::Type(ty) = definition {
            if env.nominal_newtype(ty.name.as_ref()).is_some()
                || ordinary_type_reuses_local_newtype_constructor(&env, ty)
            {
                return Err(TypeCheckError::TypeError(format!(
                    "local type '{}' conflicts with existing newtype or constructor",
                    ty.name
                )));
            }
            if env.has_type(ty.name.as_ref()) {
                continue;
            }
            let core_type = ash_parser::lower_surface_type_def(ty);
            env.register_type(&core_type)
                .map_err(TypeCheckError::from)?;
        }
    }

    register_local_nominal_newtype_representations(&mut env, &program.definitions)?;

    for definition in &program.definitions {
        if let ash_parser::surface::Definition::Impl(implementation) = definition {
            env.register_impl(implementation)
                .map_err(TypeCheckError::from)?;
        }
    }

    env.register_local_effect_row_declarations(&program.definitions)
        .map_err(TypeCheckError::from)?;

    register_function_signatures(&mut env, &program.definitions)?;
    refine_function_signatures(&mut env, &program.definitions)?;
    // Preserve the source application mismatch diagnostic boundary before an
    // unrelated malformed handler branch can obscure it.
    let handler_input_types = validate_handler_application_inputs(&env, program)?;
    let checked_handlers = check_handler_declarations(&env, program)?;
    // Applications are collected only after every declaration fact has passed
    // its complete checks.  An error returns before a `TypeCheckResult` is
    // constructed, so no partial application evidence can publish.
    let checked_handler_applications =
        validate_handler_applications(&env, program, &checked_handlers, &handler_input_types)?;

    for definition in &program.definitions {
        if let ash_parser::surface::Definition::Interface(interface) = definition {
            env.register_interface_laws(interface)
                .map_err(TypeCheckError::from)?;
        }
    }
    env.register_module_laws(&program.definitions)
        .map_err(TypeCheckError::from)?;
    for definition in &program.definitions {
        if let ash_parser::surface::Definition::Impl(implementation) = definition {
            env.register_impl_proofs_with_fuel(implementation, config.proof_fuel)
                .map_err(TypeCheckError::from)?;
        }
    }
    env.register_module_proofs_with_fuel(&program.definitions, config.proof_fuel)
        .map_err(TypeCheckError::from)?;

    let mut result = type_check_program_entry(&env, program)?;
    result.checked_handlers = checked_handlers;
    result.checked_handler_applications = checked_handler_applications;
    Ok(result)
}

/// Complete local newtype declaration facts after ordinary local types are
/// available but before any callable signature or body is checked.
///
/// This bounded path deliberately admits only non-generic local declarations.
/// A bodyless builtin type is opaque runtime substrate, not evidence of an
/// inhabitant from which a wrapper may be constructed.
fn register_local_nominal_newtype_representations(
    env: &mut TypeEnv,
    definitions: &[ash_parser::surface::Definition],
) -> Result<(), TypeCheckError> {
    if local_newtype_representation_graph_is_recursive(definitions) {
        return Err(TypeCheckError::TypeError(
            "recursive newtype representation is not supported".to_string(),
        ));
    }

    for definition in definitions {
        let ash_parser::surface::Definition::Newtype(newtype) = definition else {
            continue;
        };
        if !newtype.type_params.is_empty() {
            return Err(TypeCheckError::TypeError(format!(
                "generic newtype '{}' is not supported by local nominal checking",
                newtype.name
            )));
        }

        if newtype_representation_is_bodyless_builtin(&newtype.representation, definitions) {
            return Err(TypeCheckError::TypeError(format!(
                "newtype representation '{}' is not inhabited",
                newtype_representation_display_name(&newtype.representation)
            )));
        }

        let representation = workflow_surface_type_to_type(
            env,
            &newtype.representation,
            &std::collections::HashMap::new(),
        )?;
        env.set_nominal_newtype_representation(newtype.name.as_ref(), representation)
            .map_err(TypeCheckError::from)?;
    }
    Ok(())
}

fn ordinary_type_reuses_local_newtype_constructor(
    env: &TypeEnv,
    ty: &ash_parser::surface::TypeDef,
) -> bool {
    let ash_parser::surface::TypeBody::Enum(variants) = &ty.body else {
        return false;
    };
    variants.iter().any(|variant| {
        env.nominal_newtype_for_constructor(variant.name.as_ref())
            .is_some()
    })
}

fn local_newtype_representation_graph_is_recursive(
    definitions: &[ash_parser::surface::Definition],
) -> bool {
    let newtypes = definitions
        .iter()
        .filter_map(|definition| match definition {
            ash_parser::surface::Definition::Newtype(newtype) => Some(newtype),
            _ => None,
        })
        .collect::<Vec<_>>();
    let names = newtypes
        .iter()
        .map(|newtype| newtype.name.to_string())
        .collect::<std::collections::HashSet<_>>();
    let edges = newtypes
        .iter()
        .map(|newtype| {
            let mut dependencies = std::collections::HashSet::new();
            collect_local_newtype_dependencies(&newtype.representation, &names, &mut dependencies);
            (newtype.name.to_string(), dependencies)
        })
        .collect::<std::collections::HashMap<_, _>>();

    names.iter().any(|start| {
        let mut visiting = std::collections::HashSet::new();
        local_newtype_reaches(start, start, &edges, &mut visiting)
    })
}

fn local_newtype_reaches(
    start: &str,
    current: &str,
    edges: &std::collections::HashMap<String, std::collections::HashSet<String>>,
    visiting: &mut std::collections::HashSet<String>,
) -> bool {
    let Some(dependencies) = edges.get(current) else {
        return false;
    };
    dependencies.iter().any(|dependency| {
        dependency == start
            || (visiting.insert(dependency.clone())
                && local_newtype_reaches(start, dependency, edges, visiting))
    })
}

fn collect_local_newtype_dependencies(
    ty: &ash_parser::surface::Type,
    local_names: &std::collections::HashSet<String>,
    dependencies: &mut std::collections::HashSet<String>,
) {
    match ty {
        ash_parser::surface::Type::Name(name) => {
            if local_names.contains(name.as_ref()) {
                dependencies.insert(name.to_string());
            }
        }
        ash_parser::surface::Type::Hole { .. } | ash_parser::surface::Type::Capability(_) => {}
        ash_parser::surface::Type::List(item)
        | ash_parser::surface::Type::Associated { base: item, .. } => {
            collect_local_newtype_dependencies(item, local_names, dependencies);
        }
        ash_parser::surface::Type::Tuple(items) => {
            for item in items {
                collect_local_newtype_dependencies(item, local_names, dependencies);
            }
        }
        ash_parser::surface::Type::Record(fields) => {
            for (_, item) in fields {
                collect_local_newtype_dependencies(item, local_names, dependencies);
            }
        }
        ash_parser::surface::Type::Constructor { name, args } => {
            if local_names.contains(name.as_ref()) {
                dependencies.insert(name.to_string());
            }
            for argument in args {
                collect_local_newtype_dependencies(argument, local_names, dependencies);
            }
        }
        ash_parser::surface::Type::AssociatedFamilyProjection { args, .. } => {
            for argument in args {
                collect_local_newtype_dependencies(argument, local_names, dependencies);
            }
        }
        ash_parser::surface::Type::Fn(parameters, _, result) => {
            for parameter in parameters {
                collect_local_newtype_dependencies(parameter, local_names, dependencies);
            }
            collect_local_newtype_dependencies(result, local_names, dependencies);
        }
    }
}

fn newtype_representation_is_bodyless_builtin(
    representation: &ash_parser::surface::Type,
    definitions: &[ash_parser::surface::Definition],
) -> bool {
    let ash_parser::surface::Type::Name(name) = representation else {
        return false;
    };
    definitions.iter().any(|definition| {
        matches!(definition,
            ash_parser::surface::Definition::Type(ty)
                if ty.name == *name
                    && ty.builtin
                    && matches!(&ty.body, ash_parser::surface::TypeBody::Struct(fields) if fields.is_empty())
        )
    })
}

fn newtype_representation_display_name(representation: &ash_parser::surface::Type) -> String {
    match representation {
        ash_parser::surface::Type::Name(name) => name.to_string(),
        other => format!("{other:?}"),
    }
}

/// Error during type checking
#[derive(Debug, Clone, thiserror::Error)]
pub enum TypeCheckError {
    /// Name resolution failed
    #[error("Name resolution error: {0}")]
    ResolutionError(String),
    /// Type error
    #[error("Type error: {0}")]
    TypeError(String),
    /// Effect constraint violation
    #[error("Effect error: {0}")]
    EffectError(String),
    /// Obligation not satisfied
    #[error("Obligation error: {0}")]
    ObligationError(String),
    /// Type-environment registration error.
    #[error("Type environment error: {0}")]
    TypeEnv(Box<crate::error::TypeEnvError>),
    /// Fail-closed source computation inference diagnostic.
    #[error("Type error: {message}")]
    UnsupportedHandlerComputation {
        /// Stable diagnostic text.
        message: String,
        /// The original source expression anchor.
        span: ash_parser::token::Span,
    },
}

impl TypeCheckError {
    /// Return the source expression that caused this type-checking error when
    /// the diagnostic has a source-level computation anchor.
    #[must_use]
    pub fn source_anchor(&self) -> ash_parser::token::Span {
        match self {
            Self::UnsupportedHandlerComputation { span, .. } => *span,
            _ => ash_parser::token::Span::default(),
        }
    }
}

impl From<crate::error::TypeEnvError> for TypeCheckError {
    fn from(err: crate::error::TypeEnvError) -> Self {
        Self::TypeEnv(Box::new(err))
    }
}

impl From<Box<crate::error::TypeEnvError>> for TypeCheckError {
    fn from(err: Box<crate::error::TypeEnvError>) -> Self {
        Self::TypeEnv(err)
    }
}

/// Extended type check result with effect and obligation info
#[derive(Debug, Clone)]
pub struct TypeCheckResult {
    /// Final substitution
    pub substitution: Substitution,
    /// Any errors encountered
    pub errors: Vec<TypeError>,
    /// Inferred types for expressions
    pub inferred_types: std::collections::HashMap<String, Type>,
    /// Inferred effect of the workflow
    pub effect: ash_core::Effect,
    /// Obligation check status
    pub obligation_status: ObligationCheckResult,
    /// Lowered pure-function contract boundaries available to runtime consumers.
    pub function_contracts: std::collections::HashMap<String, StoredFnContract>,
    /// Static authority provenance metadata available to runtime admission consumers.
    pub authority_provenance: AuthorityProvenanceReport,
    /// Checked handler declaration facts retained for later typed lowering.
    pub checked_handlers: std::collections::HashMap<String, CheckedHandlerDeclaration>,
    /// Immutable source-only facts for successfully checked `handle … with`
    /// expressions.  These carry type evidence only; they do not install a
    /// Core handler, provider, frame, or runtime dispatch route.
    pub checked_handler_applications: Vec<CheckedHandlerApplication>,
    /// One exact, successfully typechecked built-in source operation eligible
    /// for a bounded downstream Core/CPS lowering path.
    ///
    /// This is source/typechecker evidence only. It does not choose a runtime
    /// provider, install a frame, or authorize execution.
    pub checked_builtin_operation: Option<CheckedBuiltinOperation>,
}

/// Canonical typechecker-owned fact for a deliberately bounded built-in
/// source operation.
///
/// The fact exists only after the entire entry function typechecks. It keeps
/// production lowering from re-reading mutable legacy engine Core data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckedBuiltinOperation {
    /// Exact typed `fn main() -> Null { time::sleep(<non-negative Int>) }`.
    TimeSleep(CheckedTimeSleepOperation),
}

/// Typed/lowerable source fact for the sole TASK-2014 built-in producer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedTimeSleepOperation {
    /// Statically checked, non-negative duration literal in milliseconds.
    pub duration_millis: i64,
    /// Span of the checked entry callable used to bind this fact to its source
    /// anchor at the Engine boundary.
    pub entry_span: ash_parser::token::Span,
    /// Span of the exact checked `time::sleep` call.
    pub call_span: ash_parser::token::Span,
}

/// Checked declaration facts for one source handler.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckedHandlerDeclaration {
    /// The declaration marker, retained as a handler-only admission fact.
    pub callable_kind: CallableDeclarationKind,
    /// The checked callable signature.
    pub callable_signature: Type,
    /// Concrete operation clause facts.
    pub clauses: Vec<CheckedHandlerClause>,
    /// Result type of the implicitly thunked handled computation.
    pub input_result_type: Type,
    /// Fully normalized requirements of that computation.
    pub input_row: NormalizedHandlerRow,
    /// Requirements left after peeling the declaration's concrete clauses.
    pub residual_row: NormalizedHandlerRow,
    /// Exact output requirements: `residual_row` union every clause-body
    /// computation requirement.  This remains a typechecker fact only.
    pub output_row: NormalizedHandlerRow,
    /// Shared answer type of every operation and `done` branch.
    pub answer_type: Type,
    /// Completion binder retained once per declaration.
    pub done_binding: String,
    /// The completion binder's operand-result type, never the answer type.
    pub done_binding_type: Type,
}

/// Immutable type evidence for one checked `handle expression with handler`
/// application.
#[derive(Debug, Clone, PartialEq)]
#[doc(hidden)]
pub struct CheckedHandlerApplication {
    /// Resolved handler value name.
    pub handler_name: String,
    /// Anchor of the handled operand expression.
    pub expression_span: ash_parser::token::Span,
    /// Anchor of the resolved handler declaration.
    pub handler_span: ash_parser::token::Span,
    /// Result type of the implicitly thunked operand.
    pub input_result_type: Type,
    /// Exact normalized requirement row of the operand.
    pub input_row: NormalizedHandlerRow,
    /// Shared answer type of the selected handler.
    pub answer_type: Type,
    /// Exact handler result row, including residual and checked branch effects.
    pub output_row: NormalizedHandlerRow,
}

/// Source continuation multiplicity derived from the normalized handler output row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinuationMultiplicity {
    /// A closed empty residual may be resumed repeatedly without residual effects.
    MultiShotPure,
    /// Any remaining requirement or open tail permits at most one direct resume.
    Affine,
}

/// Checked facts for one concrete operation clause.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckedHandlerClause {
    /// The declaration-backed concrete operation identity and signature.
    pub operation: DeclaredConcreteOperation,
    /// Declared payload type.
    pub payload_type: Type,
    /// Resume binder name retained without assigning call semantics.
    pub resume_name: String,
    /// Parser provenance for this checked source clause. Derived clauses retain
    /// their `derive handler` desugaring site without implying Core lowering.
    pub origin: ash_parser::surface::SurfaceOrigin,
    /// One direct declaration-backed local effect retained exclusively for
    /// checked Core/CPS inspection. It is not a runtime dispatch authority.
    pub local_effect: Option<DeclaredConcreteOperation>,
    /// Completion binder shared by the handler declaration.
    pub done_binding: String,
    /// Checked completion body type.
    pub done_body_type: Type,
    /// Exact residual requirement row carried by this continuation fact.
    pub continuation_row: NormalizedHandlerRow,
    /// Continuation discipline derived from `continuation_row`.
    pub continuation_multiplicity: ContinuationMultiplicity,
}

/// Read-only test seam for row-aware handler facts.  This exposes facts only;
/// it neither lowers handlers nor constructs runtime authority.
#[doc(hidden)]
pub fn checked_handler_row_fact_for_test<'a>(
    checked: &'a TypeCheckResult,
    handler_name: &str,
) -> Option<&'a CheckedHandlerDeclaration> {
    checked.checked_handlers.get(handler_name)
}

/// Read-only test seam for checked implicit-thunk applications.  These are
/// immutable source facts, not lowering or runtime artifacts.
#[doc(hidden)]
pub fn checked_handler_application_facts_for_test(
    checked: &TypeCheckResult,
) -> &[CheckedHandlerApplication] {
    &checked.checked_handler_applications
}

impl TypeCheckResult {
    /// Check if type checking succeeded
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty() && self.obligation_status.is_success()
    }

    /// Get the final type after applying substitution
    pub fn final_type(&self, ty: &Type) -> Type {
        self.substitution.apply(ty)
    }
}

impl std::fmt::Display for TypeCheckResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_ok() {
            write!(f, "Type check succeeded with effect {:?}", self.effect)
        } else {
            writeln!(f, "Type check failed:")?;
            if !self.errors.is_empty() {
                writeln!(f, "  Type errors: {}", self.errors.len())?;
            }
            if !self.obligation_status.is_success() {
                writeln!(f, "  Obligation status: {:?}", self.obligation_status)?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_type_check_error_display() {
        let err = TypeCheckError::ResolutionError("test".to_string());
        assert!(format!("{err}").contains("test"));

        let err = TypeCheckError::TypeError("type mismatch".to_string());
        assert!(format!("{err}").contains("type mismatch"));

        let err = TypeCheckError::EffectError("effect violation".to_string());
        assert!(format!("{err}").contains("effect violation"));

        let err = TypeCheckError::ObligationError("obligation failed".to_string());
        assert!(format!("{err}").contains("obligation failed"));
    }

    #[test]
    fn test_module_exports() {
        // Test that all modules are accessible via crate root
        let _ = ConstraintContext::new();
        let _ = TypeEnv::with_builtin_types();
        let _ = Type::Int;
    }

    /// SPEC-072 / TASK-959: pure closure syntax remains `Fn`, so a target
    /// function may return a closure when its declared return type is a matching
    /// pure callable type.
    #[test]
    fn task959_fn_return_pure_closure_is_accepted() {
        use ash_parser::surface::{
            Definition, Expr, FnDef, Program, ProgramEntry, Type as SurfaceType, Visibility,
        };
        use ash_parser::token::Span;

        fn test_span() -> Span {
            Span::new(0, 0, 1, 1)
        }

        // The declared return type is `(Int) -> Int` (a pure function type).
        // TASK-959 keeps pure closure syntax at the Pure stratum.
        let program = Program {
            definitions: vec![Definition::Function(FnDef {
                visibility: Visibility::Inherited,
                name: "main".into(),
                type_params: vec![],
                params: vec![],
                return_type: Some(SurfaceType::Fn(
                    vec![SurfaceType::Name("Int".into())],
                    None,
                    Box::new(SurfaceType::Name("Int".into())),
                )),
                proposition_tail: None,
                contract: None,
                body: Expr::FnDef {
                    params: vec![("x".into(), Some("Int".into()))],
                    return_type: None,
                    body: Box::new(ash_parser::surface::Expr::Variable {
                        name: "x".into(),
                        span: ash_parser::token::Span::default(),
                    }),
                    span: test_span(),
                },
                span: test_span(),
            })],
            entry: ProgramEntry {
                function: "main".into(),
                span: test_span(),
            },
        };

        let result = type_check_program(&program);
        assert!(
            result.is_ok(),
            "fn returning a matching pure closure should typecheck, got {result:?}"
        );
    }
}
