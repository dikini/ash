//! Ash Type Checker
//!
//! Type system and type inference for the Ash workflow language.
//!
//! This crate provides:
//! - **types**: Core type definitions and unification (TASK-015 to TASK-018)
//! - **constraints**: Constraint generation for workflows and expressions (TASK-019)
//! - **solver**: Constraint solving and type error reporting (TASK-020, TASK-025)
//! - **effect**: Effect inference and lattice operations (TASK-021)
//! - **names**: Name resolution and scope tracking (TASK-022)
//! - **obligations**: Obligation tracking and proof obligations (TASK-023, TASK-024)
//! - **runtime_verification**: Runtime verification checks (TASK-116)

pub mod capability_check;
pub mod capability_typecheck;
pub mod check_expr;
pub mod check_pattern;
pub mod constraint_checking;
pub mod constraints;
pub mod diagnostic;
pub(crate) mod do_target;
pub mod effect;
pub mod effective_caps;
pub mod error;
pub mod exhaustiveness;
pub mod instantiate;
pub mod kind;
pub mod name_binding;
pub mod names;
pub mod normalizer;
pub mod obligation_checker;
pub mod obligations;
pub mod policy_check;
pub mod purity;
pub mod qualified_name;
pub mod requirements;
pub mod role_checking;
pub mod runtime_verification;
pub mod solver;
pub mod type_env;
pub mod types;
pub mod visibility;

// SMT-based policy conflict detection using Z3
// Provides compile-time verification of policy constraints
pub mod smt;

#[doc(hidden)]
pub use do_target::{SelectedDoEvidence, SelectedDoOperation};

// Re-export smt module under a unified name
pub use smt as policy;

pub use ash_core::ast::{TypeDef, VariantDef};
pub use capability_check::*;
pub use check_pattern::{
    Bindings, Irrefutability, IrrefutabilityBlockedReason, IrrefutabilityImpossibleReason,
    IrrefutabilityOutcome, IrrefutabilityWitness, check_irrefutable_pattern,
    check_irrefutable_pattern_with_canonical_type, check_irrefutable_pattern_with_canonicalization,
    check_pattern,
};
pub use constraint_checking::*;
pub use constraints::*;
pub use effect::*;
pub use effective_caps::{
    CapabilitySource, CompositionError, EffectiveCapabilitySet, MergedCapability,
};
pub use instantiate::{InstantiateError, InstantiateSubst, instantiate};
pub use kind::Kind;
pub use name_binding::{NameBinder, NameError};
pub use names::*;
pub use normalizer::*;
pub use obligation_checker::*;
pub use obligations::*;
pub use policy_check::*;
pub use qualified_name::QualifiedName;
pub use requirements::{
    CheckResult, ContractCheckResult, RequirementContext, RequirementError, check_contract,
    check_requirement,
};
pub use runtime_verification::{
    AggregateVerificationInputs, CapabilityOperation, CapabilitySchema, CapabilitySchemaRegistry,
    CapabilityVerifier, EffectChecker, ObligationRequirements, OperationError, OperationResult,
    OperationVerifier, RateLimiter, RuntimeObligationChecker, RuntimeObligations, StaticPolicy,
    StaticPolicyValidator, VerificationError, VerificationResult, VerificationWarning,
};
pub use solver::{Solver, TypeError};
pub use type_env::{
    AuthorityProvenanceKind, AuthorityProvenanceReport, BindingProvenanceSourceInfo,
    CapabilityBindingInfo, CapabilityBindingProvenanceInfo, DEFAULT_PROOF_FUEL, ErasedProof,
    ImplementationAuthoritySourceInfo, PartialConstructorElaborationError,
    PatternCanonicalConstructor, PatternCanonicalType, PatternCanonicalization,
    PatternCanonicalizationBlockedReason, ProofTotalityResult, ProofTotalityStatus,
    ProofTotalityUntestedReason, ProvenanceSourceKind, PublicTowerAlgebra,
    PublicTowerIntrinsicKind, PublicTowerIntrinsicMapping, PublicTowerManifest,
    PublicTowerManifestKind, PublicTowerOperation, PublicTowerOperationAuthority,
    PublicTowerOperationRole, ResourceBindingProvenanceInfo, ResourceTypeInfo, StoredFnContract,
    TypeEnv, WorkflowIntrinsicKind, WorkflowIntrinsicParameterClass,
};
pub use types::*;
pub use visibility::{ModulePath, VisibilityChecker, VisibilityError, VisibilityExt};

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

use std::collections::HashSet;

fn synthetic_program_module_identity() -> ash_core::semantic_summary::ModuleIdentity {
    ash_core::semantic_summary::ModuleIdentity::new(
        None,
        ash_core::module_graph::ModuleId(0),
        vec!["<program>".to_string()],
        ash_core::semantic_summary::ModuleSourceOrigin::Synthetic {
            reason: "type_check_program default module context".to_string(),
        },
    )
}

fn register_surface_type_parameter_kinds(
    env: &TypeEnv,
    params: &[ash_parser::surface::TypeParam],
) -> Result<TypeEnv, TypeCheckError> {
    let mut scoped = env.clone();
    for param in params {
        let kind = param
            .kind
            .as_ref()
            .map(|annotation| annotation.kind.clone())
            .unwrap_or(Kind::Type);
        scoped
            .register_type_parameter_kind(param.name.to_string(), kind)
            .map_err(TypeCheckError::from)?;
    }
    Ok(scoped)
}

fn resolve_public_surface_associated_interface(
    env: &TypeEnv,
    base_ty: &Type,
    name: &str,
) -> Result<String, TypeCheckError> {
    let Type::Var(var) = base_ty else {
        return Err(TypeCheckError::TypeError(format!(
            "unresolved associated type '{name}'"
        )));
    };

    let Some(bounds) = env.type_var_interface_bounds.get(var) else {
        return Err(TypeCheckError::TypeError(format!(
            "unresolved associated type '{name}'"
        )));
    };

    let mut candidates = Vec::new();
    for bound_iface in bounds {
        match env.interfaces.get(bound_iface) {
            Some(iface_info) if iface_info.associated_types.contains(&name.to_string()) => {
                candidates.push(bound_iface.clone());
            }
            _ => {}
        }
    }

    if candidates.len() == 1 {
        Ok(candidates.into_iter().next().unwrap())
    } else if candidates.len() > 1 {
        Err(TypeCheckError::TypeError(format!(
            "ambiguous associated type '{name}'"
        )))
    } else {
        Err(TypeCheckError::TypeError(format!(
            "unresolved associated type '{name}'"
        )))
    }
}

fn workflow_surface_type_to_type(
    env: &TypeEnv,
    ty: &ash_parser::surface::Type,
    type_params: &std::collections::HashMap<String, Type>,
) -> Result<Type, TypeCheckError> {
    match ty {
        ash_parser::surface::Type::Hole { span } => Err(TypeCheckError::TypeError(format!(
            "type holes are only accepted in audited SPEC-066 do-target positions; this semantic lowering path does not accept source holes at {span:?}"
        ))),
        ash_parser::surface::Type::Name(name) => {
            if let Some(ty) = type_params.get(name.as_ref()) {
                if let Some(kind) = env.type_parameter_kind(name.as_ref())
                    && !kind.is_type()
                {
                    return Err(TypeCheckError::TypeError(format!(
                        "constructor variable '{}' has kind {}; expected a fully applied proper type",
                        name, kind
                    )));
                }
                return Ok(ty.clone());
            }

            match name.as_ref() {
                "Int" => Ok(Type::Int),
                "String" => Ok(Type::String),
                "Bool" => Ok(Type::Bool),
                "Null" => Ok(Type::Null),
                "Time" => Ok(Type::Time),
                "Ref" => Ok(Type::Ref),
                _ => {
                    let (qualified, _) = env
                        .resolve_type(name.as_ref())
                        .map_err(|e| TypeCheckError::TypeError(format!("{e}")))?;
                    env.check_type_constructor_arity(&qualified, 0)
                        .map_err(|e| TypeCheckError::TypeError(format!("{e}")))?;
                    if let Some(target) = env.transparent_alias_target(&qualified, &[]) {
                        Ok(target)
                    } else {
                        Ok(Type::Constructor {
                            name: qualified,
                            args: vec![],
                            kind: Kind::Type,
                        })
                    }
                }
            }
        }
        ash_parser::surface::Type::List(item) => {
            workflow_surface_type_to_type(env, item, type_params)
                .map(|item| Type::List(Box::new(item)))
        }
        ash_parser::surface::Type::Tuple(items) => {
            let items = items
                .iter()
                .enumerate()
                .map(|(index, ty)| {
                    workflow_surface_type_to_type(env, ty, type_params)
                        .map(|ty| (ash_core::adt::tuple_field_name(index).into_boxed_str(), ty))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Type::Record(items))
        }
        ash_parser::surface::Type::Record(fields) => {
            let fields = fields
                .iter()
                .map(|(name, ty)| {
                    workflow_surface_type_to_type(env, ty, type_params)
                        .map(|ty| (Box::from(name.as_ref()), ty))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Type::Record(fields))
        }
        ash_parser::surface::Type::Capability(name) => Ok(Type::Cap {
            name: Box::from(name.as_ref()),
            effect: ash_core::Effect::Operational,
        }),
        ash_parser::surface::Type::Constructor { name, args } => {
            if let Some(kind) = env.type_parameter_kind(name.as_ref()) {
                if kind.is_type() {
                    return Err(TypeCheckError::TypeError(format!(
                        "proper type variable '{}' of kind * cannot be applied as a constructor",
                        name
                    )));
                }
                let expected_arity = kind.arity();
                if args.len() != expected_arity {
                    return Err(TypeCheckError::TypeError(format!(
                        "wrong arity for constructor variable '{}': expected {}, found {}",
                        name,
                        expected_arity,
                        args.len()
                    )));
                }
                let args = args
                    .iter()
                    .map(|arg| workflow_surface_type_to_type(env, arg, type_params))
                    .collect::<Result<Vec<_>, _>>()?;
                return Ok(Type::ConstructorVariableApp {
                    constructor: name.to_string(),
                    args,
                    kind: Kind::Type,
                });
            }
            let (qualified, _) = env
                .resolve_type(name.as_ref())
                .map_err(|e| TypeCheckError::TypeError(format!("{e}")))?;
            env.check_type_constructor_arity(&qualified, args.len())
                .map_err(|e| TypeCheckError::TypeError(format!("{e}")))?;
            let args = args
                .iter()
                .map(|arg| workflow_surface_type_to_type(env, arg, type_params))
                .collect::<Result<Vec<_>, _>>()?;
            if let Some(target) = env.transparent_alias_target(&qualified, &args) {
                Ok(target)
            } else {
                Ok(Type::Constructor {
                    name: qualified,
                    args,
                    kind: Kind::Type,
                })
            }
        }
        ash_parser::surface::Type::Fn(params, ret) => {
            // Pure function type: Fn(T, U) -> V => Type::Fn(params, ret)
            let param_types: Result<Vec<_>, _> = params
                .iter()
                .map(|p| workflow_surface_type_to_type(env, p, type_params))
                .collect();
            let ret_type = workflow_surface_type_to_type(env, ret, type_params)?;
            Ok(Type::Fn(param_types?, Box::new(ret_type)))
        }
        ash_parser::surface::Type::Associated { base, name } => {
            let base_ty = workflow_surface_type_to_type(env, base, type_params)?;
            let interface = resolve_public_surface_associated_interface(env, &base_ty, name)?;
            Ok(Type::Associated {
                interface,
                base: Box::new(base_ty),
                name: name.to_string(),
            })
        }
        ash_parser::surface::Type::AssociatedFamilyProjection {
            interface,
            args,
            member,
            span,
        } => {
            let declaration = env
                .lookup_associated_family_declaration(interface.as_ref(), member.as_ref())
                .ok_or_else(|| {
                    TypeCheckError::TypeError(format!(
                        "unknown sealed associated-family projection '<{}<...>>::{}'",
                        interface, member
                    ))
                })?;
            if declaration.interface_params.len() != args.len() {
                return Err(TypeCheckError::TypeError(format!(
                    "associated-family projection '{}::{}' at {:?} expects {} interface arguments, found {}",
                    interface,
                    member,
                    span,
                    declaration.interface_params.len(),
                    args.len()
                )));
            }
            let args = args
                .iter()
                .map(|arg| workflow_surface_type_to_type(env, arg, type_params))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Type::Associated {
                interface: interface.to_string(),
                base: Box::new(Type::Constructor {
                    name: QualifiedName::root(interface.as_ref()),
                    args,
                    kind: Kind::Type,
                }),
                name: member.to_string(),
            })
        }
    }
}

pub(crate) fn bind_pattern_variables(
    env: &mut TypeEnv,
    pattern: &ash_parser::surface::Pattern,
    ty: &Type,
) {
    match pattern {
        ash_parser::surface::Pattern::Variable { name, .. } => {
            env.bind_variable(name.as_ref(), ty.clone());
        }
        ash_parser::surface::Pattern::Tuple(items) => {
            for (index, item) in items.iter().enumerate() {
                let item_ty = if let Type::Record(fields) = ty {
                    fields
                        .iter()
                        .find(|(field, _)| field.as_ref() == ash_core::adt::tuple_field_name(index))
                        .map(|(_, field_ty)| field_ty.clone())
                        .unwrap_or_else(|| Type::Var(TypeVar::fresh()))
                } else {
                    Type::Var(TypeVar::fresh())
                };
                bind_pattern_variables(env, item, &item_ty);
            }
        }
        ash_parser::surface::Pattern::Record(fields) => {
            for (field_name, pattern) in fields {
                let field_ty = if let Type::Record(record_fields) = ty {
                    record_fields
                        .iter()
                        .find(|(name, _)| name.as_ref() == field_name.as_ref())
                        .map(|(_, field_ty)| field_ty.clone())
                        .unwrap_or_else(|| Type::Var(TypeVar::fresh()))
                } else {
                    Type::Var(TypeVar::fresh())
                };
                bind_pattern_variables(env, pattern, &field_ty);
            }
        }
        ash_parser::surface::Pattern::List { elements, rest } => {
            let item_ty = match ty {
                Type::List(item_ty) => item_ty.as_ref().clone(),
                _ => Type::Var(TypeVar::fresh()),
            };

            for element in elements {
                bind_pattern_variables(env, element, &item_ty);
            }

            if let Some(rest) = rest {
                env.bind_variable(rest.as_ref(), Type::List(Box::new(item_ty)));
            }
        }
        ash_parser::surface::Pattern::Variant {
            name,
            fields,
            payload,
        } => {
            let variant_fields = variant_field_types(env, ty, name.as_ref());

            if let Some(fields) = fields {
                for (field_name, pattern) in fields {
                    let field_ty = variant_fields
                        .as_ref()
                        .and_then(|variant_fields| {
                            variant_fields
                                .iter()
                                .find(|(name, _)| name == field_name.as_ref())
                                .map(|(_, field_ty)| field_ty.clone())
                        })
                        .unwrap_or_else(|| Type::Var(TypeVar::fresh()));
                    bind_pattern_variables(env, pattern, &field_ty);
                }
            }

            match payload {
                ash_parser::surface::VariantPatternPayload::Unit => {}
                ash_parser::surface::VariantPatternPayload::Tuple(items) => {
                    for (index, item) in items.iter().enumerate() {
                        let field_ty = variant_fields
                            .as_ref()
                            .and_then(|variant_fields| {
                                variant_fields
                                    .iter()
                                    .find(|(name, _)| {
                                        name == &ash_core::adt::tuple_field_name(index)
                                    })
                                    .map(|(_, field_ty)| field_ty.clone())
                            })
                            .unwrap_or_else(|| Type::Var(TypeVar::fresh()));
                        bind_pattern_variables(env, item, &field_ty);
                    }
                }
                ash_parser::surface::VariantPatternPayload::Record(fields) => {
                    for (field_name, pattern) in fields {
                        let field_ty = variant_fields
                            .as_ref()
                            .and_then(|variant_fields| {
                                variant_fields
                                    .iter()
                                    .find(|(name, _)| name == field_name.as_ref())
                                    .map(|(_, field_ty)| field_ty.clone())
                            })
                            .unwrap_or_else(|| Type::Var(TypeVar::fresh()));
                        bind_pattern_variables(env, pattern, &field_ty);
                    }
                }
            }
        }
        ash_parser::surface::Pattern::Wildcard | ash_parser::surface::Pattern::Literal(_) => {}
    }
}

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

fn infer_checked_expr_type(
    env: &TypeEnv,
    expr: &ash_parser::surface::Expr,
    failure_context: &str,
) -> Result<Type, TypeCheckError> {
    let result = crate::check_expr::check_expr(env, expr);
    if !result.is_ok() {
        let reason = result
            .errors
            .into_iter()
            .next()
            .map(|error| error.to_string())
            .unwrap_or_else(|| failure_context.to_string());
        return Err(TypeCheckError::TypeError(reason));
    }

    Ok(result.substitution.apply(&result.ty))
}

fn infer_for_pattern_binding_type(
    env: &TypeEnv,
    collection: &ash_parser::surface::Expr,
) -> Result<Type, TypeCheckError> {
    let collection_ty =
        infer_checked_expr_type(env, collection, "failed to typecheck for collection")?;

    match collection_ty {
        Type::List(item_ty) => Ok(item_ty.as_ref().clone()),
        other => Err(TypeCheckError::TypeError(format!(
            "for collection must have list type, found {}",
            other
        ))),
    }
}

fn bind_irrefutable_workflow_pattern(
    env: &mut TypeEnv,
    construct_kind: &str,
    pattern: &ash_parser::surface::Pattern,
    scrutinee_type: &Type,
    fallback_span: ash_parser::token::Span,
) -> Result<(), TypeCheckError> {
    let span = crate::check_expr::surface_pattern_span(pattern, fallback_span);
    let bindings = crate::check_expr::check_irrefutable_let_pattern(
        env,
        construct_kind,
        pattern,
        scrutinee_type,
        span,
    )
    .map_err(|error| TypeCheckError::TypeError(error.to_string()))?;
    crate::check_expr::bind_irrefutable_pattern_bindings(env, bindings);
    Ok(())
}

fn validate_irrefutable_workflow_binders(
    env: &mut TypeEnv,
    workflow: &ash_parser::surface::Workflow,
) -> Result<(), TypeCheckError> {
    match workflow {
        ash_parser::surface::Workflow::Observe {
            binding,
            continuation,
            span,
            ..
        } => {
            let mut next_env = env.clone();
            if let Some(binding) = binding {
                bind_irrefutable_workflow_pattern(
                    &mut next_env,
                    "workflow observe binding",
                    binding,
                    &Type::Var(TypeVar::fresh()),
                    *span,
                )?;
            }
            if let Some(continuation) = continuation {
                validate_irrefutable_workflow_binders(&mut next_env, continuation)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::Orient {
            expr,
            binding,
            continuation,
            span,
            ..
        } => {
            validate_interface_calls_in_expr(env, expr)?;
            let mut next_env = env.clone();
            if let Some(binding) = binding {
                let binding_ty =
                    infer_checked_expr_type(env, expr, "failed to typecheck orient binding")?;
                bind_irrefutable_workflow_pattern(
                    &mut next_env,
                    "workflow orient binding",
                    binding,
                    &binding_ty,
                    *span,
                )?;
            }
            if let Some(continuation) = continuation {
                validate_irrefutable_workflow_binders(&mut next_env, continuation)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::Let {
            pattern,
            expr,
            continuation,
            span,
        } => {
            validate_interface_calls_in_expr(env, expr)?;
            let binding_ty =
                infer_checked_expr_type(env, expr, "failed to typecheck workflow let binding")?;
            let mut next_env = env.clone();
            bind_irrefutable_workflow_pattern(
                &mut next_env,
                "workflow let",
                pattern,
                &binding_ty,
                *span,
            )?;
            if let Some(continuation) = continuation {
                validate_irrefutable_workflow_binders(&mut next_env, continuation)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::For {
            pattern,
            collection,
            body,
            span,
        } => {
            validate_interface_calls_in_expr(env, collection)?;
            let item_ty = infer_for_pattern_binding_type(env, collection)?;
            let mut body_env = env.clone();
            bind_irrefutable_workflow_pattern(
                &mut body_env,
                "workflow for binder",
                pattern,
                &item_ty,
                *span,
            )?;
            validate_irrefutable_workflow_binders(&mut body_env, body)
        }
        ash_parser::surface::Workflow::Yield {
            expr,
            resume_type,
            arms,
            ..
        } => {
            validate_interface_calls_in_expr(env, expr)?;
            let resume_ty =
                workflow_surface_type_to_type(env, resume_type, &std::collections::HashMap::new())
                    .unwrap_or_else(|_| Type::Var(TypeVar::fresh()));
            for arm in arms {
                let mut arm_env = env.clone();
                bind_irrefutable_workflow_pattern(
                    &mut arm_env,
                    "workflow yield arm",
                    &arm.pattern,
                    &resume_ty,
                    arm.span,
                )?;
                validate_irrefutable_workflow_binders(&mut arm_env, &arm.body)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::Receive { arms, .. } => {
            for arm in arms {
                let mut arm_env = env.clone();
                if let ash_parser::surface::StreamPattern::Binding { pattern, .. } = &arm.pattern {
                    // Receive arms are selective mailbox filters, not total binders. Preserve
                    // current semantics here; TASK-1007 hardens the implicit complement path.
                    bind_pattern_variables(&mut arm_env, pattern, &Type::Var(TypeVar::fresh()));
                }
                validate_irrefutable_workflow_binders(&mut arm_env, &arm.body)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::Propose {
            action,
            continuation,
            ..
        } => {
            validate_interface_calls_in_action(env, action)?;
            if let Some(continuation) = continuation {
                validate_irrefutable_workflow_binders(&mut env.clone(), continuation)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::Check { continuation, .. }
        | ash_parser::surface::Workflow::Set { continuation, .. }
        | ash_parser::surface::Workflow::Send { continuation, .. } => {
            if let Some(continuation) = continuation {
                validate_irrefutable_workflow_binders(env, continuation)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::Decide {
            then_branch,
            else_branch,
            ..
        }
        | ash_parser::surface::Workflow::If {
            then_branch,
            else_branch,
            ..
        } => {
            validate_irrefutable_workflow_binders(&mut env.clone(), then_branch)?;
            if let Some(else_branch) = else_branch {
                validate_irrefutable_workflow_binders(&mut env.clone(), else_branch)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::With { body, .. }
        | ash_parser::surface::Workflow::Must { body, .. } => {
            validate_irrefutable_workflow_binders(env, body)
        }
        ash_parser::surface::Workflow::Maybe {
            primary, fallback, ..
        } => {
            validate_irrefutable_workflow_binders(&mut env.clone(), primary)?;
            validate_irrefutable_workflow_binders(&mut env.clone(), fallback)
        }
        ash_parser::surface::Workflow::Seq { first, second, .. } => {
            validate_irrefutable_workflow_binders(env, first)?;
            validate_irrefutable_workflow_binders(env, second)
        }
        ash_parser::surface::Workflow::Ret { expr, .. }
        | ash_parser::surface::Workflow::Resume { expr, .. } => {
            validate_interface_calls_in_expr(env, expr)
        }
        ash_parser::surface::Workflow::Act {
            action,
            guard,
            result_name,
            continuation,
            ..
        } => {
            validate_interface_calls_in_action(env, action)?;
            if let Some(guard) = guard {
                validate_interface_calls_in_guard(env, guard)?;
            }
            if let Some(continuation) = continuation {
                let mut next_env = env.clone();
                if let Some(result_name) = result_name {
                    next_env.bind_variable(result_name.as_ref(), Type::Var(TypeVar::fresh()));
                }
                validate_irrefutable_workflow_binders(&mut next_env, continuation)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::Done { .. }
        | ash_parser::surface::Workflow::Oblige { .. } => Ok(()),
    }
}

fn infer_surface_expr_type(
    env: &TypeEnv,
    expr: &ash_parser::surface::Expr,
) -> Result<Type, TypeCheckError> {
    match expr {
        ash_parser::surface::Expr::Match {
            scrutinee, arms, ..
        } => {
            let scrutinee_ty =
                infer_checked_expr_type(env, scrutinee, "failed to typecheck match scrutinee")?;
            let mut arm_types = Vec::with_capacity(arms.len());

            for arm in arms {
                let mut arm_env = env.clone();
                bind_pattern_variables(&mut arm_env, &arm.pattern, &scrutinee_ty);
                arm_types.push(infer_surface_expr_type(&arm_env, arm.body.as_ref())?);
            }

            let Some(first_ty) = arm_types.first().cloned() else {
                return Ok(Type::Var(TypeVar::fresh()));
            };

            arm_types
                .into_iter()
                .skip(1)
                .try_fold(first_ty, |acc, arm_ty| {
                    crate::types::unify(&acc, &arm_ty)
                        .map(|subst| subst.apply(&acc))
                        .map_err(|_| {
                            TypeCheckError::TypeError(format!(
                                "match arms must have compatible types: {} vs {}",
                                acc, arm_ty
                            ))
                        })
                })
        }
        ash_parser::surface::Expr::IfLet {
            pattern,
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            let matched_ty =
                infer_checked_expr_type(env, expr, "failed to typecheck if-let scrutinee")?;
            let mut then_env = env.clone();
            bind_pattern_variables(&mut then_env, pattern, &matched_ty);

            let then_ty = infer_surface_expr_type(&then_env, then_branch)?;
            let else_ty = infer_surface_expr_type(env, else_branch)?;
            crate::types::unify(&then_ty, &else_ty)
                .map(|subst| subst.apply(&then_ty))
                .map_err(|_| {
                    TypeCheckError::TypeError(format!(
                        "if-let branches must have compatible types: {} vs {}",
                        then_ty, else_ty
                    ))
                })
        }
        _ => infer_checked_expr_type(env, expr, "failed to typecheck expression"),
    }
}

fn validate_match_expr(
    env: &TypeEnv,
    scrutinee: &ash_parser::surface::Expr,
    arms: &[ash_parser::surface::MatchArm],
) -> Result<(), TypeCheckError> {
    validate_interface_calls_in_expr(env, scrutinee)?;

    let scrutinee_ty =
        infer_checked_expr_type(env, scrutinee, "failed to typecheck match scrutinee")?;
    let mut arm_types = Vec::with_capacity(arms.len());
    for arm in arms {
        let mut arm_env = env.clone();
        bind_pattern_variables(&mut arm_env, &arm.pattern, &scrutinee_ty);
        validate_interface_calls_in_expr(&arm_env, arm.body.as_ref())?;
        arm_types.push(infer_surface_expr_type(&arm_env, arm.body.as_ref())?);
    }

    if let Some(first_ty) = arm_types.first().cloned() {
        for arm_ty in arm_types.into_iter().skip(1) {
            crate::types::unify(&first_ty, &arm_ty).map_err(|_| {
                TypeCheckError::TypeError(format!(
                    "match arms must have compatible types: {} vs {}",
                    first_ty, arm_ty
                ))
            })?;
        }
    }

    Ok(())
}

fn validate_interface_calls_in_expr(
    env: &TypeEnv,
    expr: &ash_parser::surface::Expr,
) -> Result<(), TypeCheckError> {
    match expr {
        ash_parser::surface::Expr::Literal(_) | ash_parser::surface::Expr::Variable { .. } => {
            Ok(())
        }
        ash_parser::surface::Expr::FieldAccess { base, .. } => {
            validate_interface_calls_in_expr(env, base)
        }
        ash_parser::surface::Expr::IndexAccess { base, index, .. } => {
            validate_interface_calls_in_expr(env, base)?;
            validate_interface_calls_in_expr(env, index)
        }
        ash_parser::surface::Expr::Unary { operand, .. } => {
            validate_interface_calls_in_expr(env, operand)
        }
        ash_parser::surface::Expr::Binary { left, right, .. } => {
            validate_interface_calls_in_expr(env, left)?;
            validate_interface_calls_in_expr(env, right)
        }
        ash_parser::surface::Expr::Call {
            module, func, args, ..
        } => {
            for arg in args {
                validate_interface_calls_in_expr(env, arg)?;
            }

            // If this is a qualified call to an interface method, validate it
            if let Some(module_name) = module.as_deref()
                && env.has_interface(module_name)
            {
                let mut arg_types = Vec::new();
                let mut subst = crate::types::Substitution::new();
                for arg in args {
                    let arg_result = crate::check_expr::check_expr(env, arg);
                    if !arg_result.is_ok() {
                        let reason = arg_result
                            .errors
                            .into_iter()
                            .next()
                            .map(|error| error.to_string())
                            .unwrap_or_else(|| {
                                format!(
                                    "failed to typecheck argument to {}::{}",
                                    module_name,
                                    func.as_ref()
                                )
                            });

                        return Err(TypeCheckError::TypeError(format!(
                            "invalid interface method call {}::{}: {}",
                            module_name,
                            func.as_ref(),
                            reason
                        )));
                    }
                    subst = subst.compose(&arg_result.substitution);
                    arg_types.push(subst.apply(&arg_result.ty));
                }

                env.resolve_interface_method_call(module_name, func.as_ref(), &arg_types)
                    .map(|_| ())
                    .map_err(|error| TypeCheckError::TypeError(error.to_string()))?;
            }

            Ok(())
        }
        ash_parser::surface::Expr::Match {
            scrutinee, arms, ..
        } => validate_match_expr(env, scrutinee, arms),
        ash_parser::surface::Expr::Policy(_)
        | ash_parser::surface::Expr::CheckObligation { .. } => Ok(()),
        ash_parser::surface::Expr::Fail { payload, .. } => {
            validate_interface_calls_in_expr(env, payload)
        }
        ash_parser::surface::Expr::WithError { body, arms, .. } => {
            validate_interface_calls_in_expr(env, body)?;
            for arm in arms {
                validate_interface_calls_in_expr(env, &arm.body)?;
            }
            Ok(())
        }
        ash_parser::surface::Expr::IfLet {
            pattern,
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            validate_interface_calls_in_expr(env, expr)?;

            let matched_ty =
                infer_checked_expr_type(env, expr, "failed to typecheck if-let scrutinee")?;
            let mut then_env = env.clone();
            bind_pattern_variables(&mut then_env, pattern, &matched_ty);

            validate_interface_calls_in_expr(&then_env, then_branch)?;
            validate_interface_calls_in_expr(env, else_branch)
        }
        ash_parser::surface::Expr::Constructor {
            fields, payload, ..
        } => {
            for (_, field_expr) in fields {
                validate_interface_calls_in_expr(env, field_expr)?;
            }

            match payload {
                ash_parser::surface::ConstructorPayload::Unit => Ok(()),
                ash_parser::surface::ConstructorPayload::Record(fields) => {
                    for (_, field_expr) in fields {
                        validate_interface_calls_in_expr(env, field_expr)?;
                    }
                    Ok(())
                }
                ash_parser::surface::ConstructorPayload::Tuple(items) => {
                    for item in items {
                        validate_interface_calls_in_expr(env, item)?;
                    }
                    Ok(())
                }
            }
        }
        ash_parser::surface::Expr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            validate_interface_calls_in_expr(env, condition)?;
            validate_interface_calls_in_expr(env, then_branch)?;
            if let Some(e) = else_branch {
                validate_interface_calls_in_expr(env, e)?;
            }
            Ok(())
        }
        ash_parser::surface::Expr::Panic { .. } => Ok(()),
        ash_parser::surface::Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            let mut block_env = env.clone();

            for stmt in statements {
                match stmt {
                    ash_parser::surface::BlockStmt::Let { pattern, expr, .. } => {
                        validate_interface_calls_in_expr(&block_env, expr)?;
                        let binding_ty = infer_checked_expr_type(
                            &block_env,
                            expr,
                            "failed to typecheck block let binding",
                        )?;
                        bind_pattern_variables(&mut block_env, pattern, &binding_ty);
                    }
                }
            }
            if let Some(e) = tail_expr {
                validate_interface_calls_in_expr(&block_env, e)?;
            }
            Ok(())
        }
        ash_parser::surface::Expr::FnDef { body, params, .. } => {
            // Note: param type annotations (_ty) are intentionally ignored here.
            // Fresh type variables are sufficient for interface-call validation
            // because we only need to resolve which interface method is being called,
            // not enforce full type correctness (that's check_expr's job).
            let mut fn_env = env.clone();
            for (name, _ty) in params {
                fn_env.bind_variable(name.as_ref(), Type::Var(TypeVar::fresh()));
            }
            validate_interface_calls_in_expr(&fn_env, body)
        }
        ash_parser::surface::Expr::FnApply { func, args, .. } => {
            validate_interface_calls_in_expr(env, func)?;
            for arg in args {
                validate_interface_calls_in_expr(env, arg)?;
            }
            Ok(())
        }
        ash_parser::surface::Expr::ActBlock { stmts, .. } => {
            use ash_parser::surface::ActStmt;
            for stmt in stmts {
                let value = match stmt {
                    ActStmt::Bind { value, .. } => value,
                    ActStmt::Return { value, .. } => value,
                };
                validate_interface_calls_in_expr(env, value)?;
            }
            Ok(())
        }
        ash_parser::surface::Expr::DoBlock { .. } => Err(TypeCheckError::TypeError(
            "generalized do-block type checking is not implemented (TASK-747 parser substrate only)"
                .to_string(),
        )),
        ash_parser::surface::Expr::Comprehension {
            result,
            qualifiers,
            ..
        } => {
            use ash_parser::surface::ComprehensionQualifier;
            for qualifier in qualifiers {
                let value = match qualifier {
                    ComprehensionQualifier::Let { value, .. }
                    | ComprehensionQualifier::Bind { value, .. }
                    | ComprehensionQualifier::DiscardBind { value, .. } => value,
                };
                validate_interface_calls_in_expr(env, value)?;
            }
            validate_interface_calls_in_expr(env, result)
        }
        ash_parser::surface::Expr::List { items, .. } => {
            for item in items {
                validate_interface_calls_in_expr(env, item)?;
            }
            Ok(())
        }
    }
}

fn validate_interface_calls_in_guard(
    env: &TypeEnv,
    guard: &ash_parser::surface::Guard,
) -> Result<(), TypeCheckError> {
    match guard {
        ash_parser::surface::Guard::Always | ash_parser::surface::Guard::Never => Ok(()),
        ash_parser::surface::Guard::Pred(predicate) => {
            for arg in &predicate.args {
                validate_interface_calls_in_expr(env, arg)?;
            }
            Ok(())
        }
        ash_parser::surface::Guard::And(left, right)
        | ash_parser::surface::Guard::Or(left, right) => {
            validate_interface_calls_in_guard(env, left)?;
            validate_interface_calls_in_guard(env, right)
        }
        ash_parser::surface::Guard::Not(inner) => validate_interface_calls_in_guard(env, inner),
    }
}

fn validate_interface_calls_in_action(
    env: &TypeEnv,
    action: &ash_parser::surface::ActionRef,
) -> Result<(), TypeCheckError> {
    use ash_parser::surface::OperationalTarget;

    // Validate the operational target
    match &action.target {
        OperationalTarget::Symbolic { capability_name } => {
            if matches!(
                env.lookup_variable(capability_name.as_ref()),
                Some(Type::Fn(_, _) | Type::Fun(_, _, _))
            ) {
                return Err(TypeCheckError::TypeError(format!(
                    "'{}' is a function, not a capability; use `module::name()` syntax instead of `provider:action()`",
                    capability_name.as_ref()
                )));
            }
            // For symbolic targets, resolution happens during lowering.
            // The resolver maps symbolic names to (provider, action) via explicit metadata.
        }
        OperationalTarget::Qualified {
            module,
            capability_name,
        } => {
            if matches!(
                env.lookup_call_target(Some(module.as_ref()), capability_name.as_ref()),
                Some(Type::Fn(_, _) | Type::Fun(_, _, _))
            ) {
                return Err(TypeCheckError::TypeError(format!(
                    "'{}::{}' is a function, not a capability; use `module::name()` syntax instead of `provider:action()`",
                    module.as_ref(),
                    capability_name.as_ref()
                )));
            }
            // For module-qualified targets (e.g., io::fs_read), resolution happens
            // during lowering. The resolver looks up the qualified name in its mappings.
        }
        OperationalTarget::Explicit {
            provider,
            action: action_name,
        } => {
            if matches!(
                env.lookup_variable(action_name.as_ref()),
                Some(Type::Fn(_, _) | Type::Fun(_, _, _))
            ) {
                return Err(TypeCheckError::TypeError(format!(
                    "'{}' is a function, not a capability; use `module::name()` syntax instead of `provider:action()`",
                    action_name.as_ref()
                )));
            }
            // For explicit targets, validate that the provider exists
            if !env.has_provider(provider.as_ref()) {
                return Err(TypeCheckError::ResolutionError(format!(
                    "unknown provider '{}' in explicit action target '{}:{}'",
                    provider.as_ref(),
                    provider.as_ref(),
                    action_name.as_ref()
                )));
            }
        }
    }

    // Validate arguments
    for arg in &action.args {
        validate_interface_calls_in_expr(env, arg)?;
    }
    Ok(())
}

fn validate_interface_calls_in_check_target(
    env: &TypeEnv,
    target: &ash_parser::surface::CheckTarget,
) -> Result<(), TypeCheckError> {
    match target {
        ash_parser::surface::CheckTarget::Obligation(_) => Ok(()),
        ash_parser::surface::CheckTarget::Policy(policy) => {
            for (_, expr) in &policy.fields {
                validate_interface_calls_in_expr(env, expr)?;
            }
            Ok(())
        }
    }
}

fn validate_interface_calls_in_workflow(
    env: &mut TypeEnv,
    workflow: &ash_parser::surface::Workflow,
) -> Result<(), TypeCheckError> {
    match workflow {
        ash_parser::surface::Workflow::Observe {
            binding,
            continuation,
            ..
        } => {
            let mut next_env = env.clone();
            if let Some(binding) = binding {
                bind_pattern_variables(&mut next_env, binding, &Type::Var(TypeVar::fresh()));
            }

            if let Some(continuation) = continuation {
                validate_interface_calls_in_workflow(&mut next_env, continuation)?;
            }

            Ok(())
        }
        ash_parser::surface::Workflow::Orient {
            expr,
            binding,
            continuation,
            ..
        } => {
            validate_interface_calls_in_expr(env, expr)?;

            if let Some(binding) = binding {
                let expr_result = crate::check_expr::check_expr(env, expr);
                let binding_ty = if expr_result.is_ok() {
                    expr_result.ty
                } else {
                    Type::Var(TypeVar::fresh())
                };
                bind_pattern_variables(env, binding, &binding_ty);
            }

            if let Some(continuation) = continuation {
                validate_interface_calls_in_workflow(env, continuation)?;
            }

            Ok(())
        }
        ash_parser::surface::Workflow::Propose {
            action,
            binding,
            continuation,
            ..
        } => {
            validate_interface_calls_in_action(env, action)?;

            let mut next_env = env.clone();
            if let Some(_binding) = binding {
                return Err(TypeCheckError::TypeError(
                    "Propose.binding is not supported in the current MVP; remove the binding pattern".to_string()
                ));
            }

            if let Some(continuation) = continuation {
                validate_interface_calls_in_workflow(&mut next_env, continuation)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::Check {
            target,
            continuation,
            ..
        } => {
            validate_interface_calls_in_check_target(env, target)?;
            if let Some(continuation) = continuation {
                validate_interface_calls_in_workflow(env, continuation)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::Set {
            value,
            continuation,
            ..
        }
        | ash_parser::surface::Workflow::Send {
            value,
            continuation,
            ..
        } => {
            validate_interface_calls_in_expr(env, value)?;
            if let Some(continuation) = continuation {
                validate_interface_calls_in_workflow(env, continuation)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::Decide {
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            validate_interface_calls_in_expr(env, expr)?;
            validate_interface_calls_in_workflow(&mut env.clone(), then_branch)?;
            if let Some(else_branch) = else_branch {
                validate_interface_calls_in_workflow(&mut env.clone(), else_branch)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::Act { action, guard, .. } => {
            validate_interface_calls_in_action(env, action)?;
            if let Some(guard) = guard {
                validate_interface_calls_in_guard(env, guard)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::Oblige { .. }
        | ash_parser::surface::Workflow::Done { .. } => Ok(()),
        ash_parser::surface::Workflow::Let {
            pattern,
            expr,
            continuation,
            ..
        } => {
            validate_interface_calls_in_expr(env, expr)?;

            let expr_result = crate::check_expr::check_expr(env, expr);
            let binding_ty = if expr_result.is_ok() {
                expr_result.ty
            } else {
                Type::Var(TypeVar::fresh())
            };
            bind_pattern_variables(env, pattern, &binding_ty);

            if let Some(continuation) = continuation {
                validate_interface_calls_in_workflow(env, continuation)?;
            }

            Ok(())
        }
        ash_parser::surface::Workflow::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            validate_interface_calls_in_expr(env, condition)?;
            validate_interface_calls_in_workflow(&mut env.clone(), then_branch)?;
            if let Some(else_branch) = else_branch {
                validate_interface_calls_in_workflow(&mut env.clone(), else_branch)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::For {
            pattern,
            collection,
            body,
            ..
        } => {
            validate_interface_calls_in_expr(env, collection)?;
            let mut body_env = env.clone();
            let item_ty = infer_for_pattern_binding_type(env, collection)?;
            bind_pattern_variables(&mut body_env, pattern, &item_ty);
            validate_interface_calls_in_workflow(&mut body_env, body)
        }
        ash_parser::surface::Workflow::With { body, .. }
        | ash_parser::surface::Workflow::Must { body, .. } => {
            validate_interface_calls_in_workflow(env, body)
        }
        ash_parser::surface::Workflow::Maybe {
            primary, fallback, ..
        } => {
            validate_interface_calls_in_workflow(&mut env.clone(), primary)?;
            validate_interface_calls_in_workflow(&mut env.clone(), fallback)
        }
        ash_parser::surface::Workflow::Seq { first, second, .. } => {
            validate_interface_calls_in_workflow(env, first)?;
            validate_interface_calls_in_workflow(env, second)
        }
        ash_parser::surface::Workflow::Ret { expr, .. } => {
            validate_interface_calls_in_expr(env, expr)
        }
        ash_parser::surface::Workflow::Receive { arms, .. } => {
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    validate_interface_calls_in_expr(env, guard)?;
                }
                validate_interface_calls_in_workflow(&mut env.clone(), &arm.body)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::Yield { expr, arms, .. } => {
            validate_interface_calls_in_expr(env, expr)?;
            for arm in arms {
                let mut arm_env = env.clone();
                bind_pattern_variables(&mut arm_env, &arm.pattern, &Type::Var(TypeVar::fresh()));
                validate_interface_calls_in_workflow(&mut arm_env, &arm.body)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::Resume { expr, .. } => {
            validate_interface_calls_in_expr(env, expr)
        }
    }
}

fn infer_workflow_return_type(
    env: &TypeEnv,
    workflow: &ash_parser::surface::Workflow,
) -> Result<Type, TypeCheckError> {
    match workflow {
        ash_parser::surface::Workflow::Ret { expr, .. } => infer_surface_expr_type(env, expr),
        ash_parser::surface::Workflow::Done { .. }
        | ash_parser::surface::Workflow::Act { .. }
        | ash_parser::surface::Workflow::Oblige { .. } => Ok(Type::Null),
        ash_parser::surface::Workflow::Observe { continuation, .. }
        | ash_parser::surface::Workflow::Propose { continuation, .. }
        | ash_parser::surface::Workflow::Check { continuation, .. }
        | ash_parser::surface::Workflow::Set { continuation, .. }
        | ash_parser::surface::Workflow::Send { continuation, .. } => continuation
            .as_deref()
            .map_or(Ok(Type::Null), |continuation| {
                infer_workflow_return_type(env, continuation)
            }),
        ash_parser::surface::Workflow::Orient {
            expr,
            binding,
            continuation,
            ..
        } => {
            let expr_result = crate::check_expr::check_expr(env, expr);
            if !expr_result.is_ok() {
                let reason = expr_result
                    .errors
                    .into_iter()
                    .next()
                    .map(|error| error.to_string())
                    .unwrap_or_else(|| "failed to typecheck orient expression".to_string());
                return Err(TypeCheckError::TypeError(reason));
            }

            let mut next_env = env.clone();
            if let Some(binding) = binding {
                let binding_ty = expr_result.substitution.apply(&expr_result.ty);
                bind_pattern_variables(&mut next_env, binding, &binding_ty);
            }

            continuation.as_deref().map_or_else(
                || Ok(expr_result.substitution.apply(&expr_result.ty)),
                |continuation| infer_workflow_return_type(&next_env, continuation),
            )
        }
        ash_parser::surface::Workflow::Let {
            pattern,
            expr,
            continuation,
            ..
        } => {
            let expr_result = crate::check_expr::check_expr(env, expr);
            if !expr_result.is_ok() {
                let reason = expr_result
                    .errors
                    .into_iter()
                    .next()
                    .map(|error| error.to_string())
                    .unwrap_or_else(|| "failed to typecheck let expression".to_string());
                return Err(TypeCheckError::TypeError(reason));
            }

            let mut next_env = env.clone();
            let binding_ty = expr_result.substitution.apply(&expr_result.ty);
            bind_pattern_variables(&mut next_env, pattern, &binding_ty);

            continuation
                .as_deref()
                .map_or(Ok(Type::Null), |continuation| {
                    infer_workflow_return_type(&next_env, continuation)
                })
        }
        ash_parser::surface::Workflow::Seq { first, second, .. } => {
            infer_workflow_return_type(env, first)?;
            infer_workflow_return_type(env, second)
        }
        ash_parser::surface::Workflow::With { body, .. }
        | ash_parser::surface::Workflow::Must { body, .. } => infer_workflow_return_type(env, body),
        ash_parser::surface::Workflow::Resume { expr, .. } => {
            let result = crate::check_expr::check_expr(env, expr);
            if !result.is_ok() {
                let reason = result
                    .errors
                    .into_iter()
                    .next()
                    .map(|error| error.to_string())
                    .unwrap_or_else(|| "failed to typecheck resume expression".to_string());
                return Err(TypeCheckError::TypeError(reason));
            }
            Ok(result.substitution.apply(&result.ty))
        }
        ash_parser::surface::Workflow::Decide {
            then_branch,
            else_branch,
            ..
        }
        | ash_parser::surface::Workflow::If {
            then_branch,
            else_branch,
            ..
        } => {
            let then_ty = infer_workflow_return_type(env, then_branch)?;
            let else_ty = else_branch
                .as_deref()
                .map_or(Ok(Type::Null), |else_branch| {
                    infer_workflow_return_type(env, else_branch)
                })?;
            crate::types::unify(&then_ty, &else_ty)
                .map(|subst| subst.apply(&then_ty))
                .map_err(|_| {
                    TypeCheckError::TypeError(format!(
                        "workflow branch return types do not match: {} vs {}",
                        then_ty, else_ty
                    ))
                })
        }
        ash_parser::surface::Workflow::Maybe {
            primary, fallback, ..
        } => {
            let primary_ty = infer_workflow_return_type(env, primary)?;
            let fallback_ty = infer_workflow_return_type(env, fallback)?;
            crate::types::unify(&primary_ty, &fallback_ty)
                .map(|subst| subst.apply(&primary_ty))
                .map_err(|_| {
                    TypeCheckError::TypeError(format!(
                        "workflow branch return types do not match: {} vs {}",
                        primary_ty, fallback_ty
                    ))
                })
        }
        ash_parser::surface::Workflow::For {
            pattern,
            collection,
            body,
            ..
        } => {
            let mut body_env = env.clone();
            let item_ty = infer_for_pattern_binding_type(env, collection)?;
            bind_pattern_variables(&mut body_env, pattern, &item_ty);
            infer_workflow_return_type(&body_env, body)
        }
        ash_parser::surface::Workflow::Receive { arms, .. } => {
            let mut arm_types = arms.iter().map(|arm| {
                let mut arm_env = env.clone();
                if let ash_parser::surface::StreamPattern::Binding { pattern, .. } = &arm.pattern {
                    bind_pattern_variables(&mut arm_env, pattern, &Type::Var(TypeVar::fresh()));
                }
                infer_workflow_return_type(&arm_env, &arm.body)
            });
            let Some(first_ty) = arm_types.next().transpose()? else {
                return Ok(Type::Null);
            };
            arm_types.try_fold(first_ty, |acc, arm_ty| {
                let arm_ty = arm_ty?;
                crate::types::unify(&acc, &arm_ty)
                    .map(|subst| subst.apply(&acc))
                    .map_err(|_| {
                        TypeCheckError::TypeError(format!(
                            "workflow arm return types do not match: {} vs {}",
                            acc, arm_ty
                        ))
                    })
            })
        }
        ash_parser::surface::Workflow::Yield { arms, .. } => {
            let mut arm_types = arms.iter().map(|arm| {
                let mut arm_env = env.clone();
                bind_pattern_variables(&mut arm_env, &arm.pattern, &Type::Var(TypeVar::fresh()));
                infer_workflow_return_type(&arm_env, &arm.body)
            });
            let Some(first_ty) = arm_types.next().transpose()? else {
                return Ok(Type::Null);
            };
            arm_types.try_fold(first_ty, |acc, arm_ty| {
                let arm_ty = arm_ty?;
                crate::types::unify(&acc, &arm_ty)
                    .map(|subst| subst.apply(&acc))
                    .map_err(|_| {
                        TypeCheckError::TypeError(format!(
                            "workflow arm return types do not match: {} vs {}",
                            acc, arm_ty
                        ))
                    })
            })
        }
    }
}

fn reject_unsupported_mvp_workflow_features(
    workflow: &ash_parser::surface::Workflow,
) -> Result<(), TypeCheckError> {
    match workflow {
        // TASK-423/TASK-612: Propose.binding is explicitly unsupported in MVP
        ash_parser::surface::Workflow::Propose {
            binding,
            continuation,
            ..
        } => {
            if binding.is_some() {
                return Err(TypeCheckError::TypeError(
                    "Propose.binding is not supported in the current MVP; remove the binding pattern".to_string()
                ));
            }
            continuation
                .as_deref()
                .map_or(Ok(()), reject_unsupported_mvp_workflow_features)
        }
        ash_parser::surface::Workflow::Observe { continuation, .. }
        | ash_parser::surface::Workflow::Orient { continuation, .. }
        | ash_parser::surface::Workflow::Check { continuation, .. }
        | ash_parser::surface::Workflow::Set { continuation, .. }
        | ash_parser::surface::Workflow::Send { continuation, .. } => continuation
            .as_deref()
            .map_or(Ok(()), reject_unsupported_mvp_workflow_features),
        ash_parser::surface::Workflow::Decide {
            then_branch,
            else_branch,
            ..
        }
        | ash_parser::surface::Workflow::If {
            then_branch,
            else_branch,
            ..
        } => {
            reject_unsupported_mvp_workflow_features(then_branch)?;
            if let Some(else_branch) = else_branch {
                reject_unsupported_mvp_workflow_features(else_branch)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::For { body, .. }
        | ash_parser::surface::Workflow::With { body, .. }
        | ash_parser::surface::Workflow::Must { body, .. } => {
            reject_unsupported_mvp_workflow_features(body)
        }
        ash_parser::surface::Workflow::Maybe {
            primary, fallback, ..
        } => {
            reject_unsupported_mvp_workflow_features(primary)?;
            reject_unsupported_mvp_workflow_features(fallback)
        }
        ash_parser::surface::Workflow::Seq { first, second, .. } => {
            reject_unsupported_mvp_workflow_features(first)?;
            reject_unsupported_mvp_workflow_features(second)
        }
        ash_parser::surface::Workflow::Receive { arms, .. } => {
            for arm in arms {
                reject_unsupported_mvp_workflow_features(&arm.body)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::Yield { arms, .. } => {
            for arm in arms {
                reject_unsupported_mvp_workflow_features(&arm.body)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::Let { continuation, .. } => continuation
            .as_deref()
            .map_or(Ok(()), reject_unsupported_mvp_workflow_features),
        ash_parser::surface::Workflow::Done { .. }
        | ash_parser::surface::Workflow::Ret { .. }
        | ash_parser::surface::Workflow::Act { .. }
        | ash_parser::surface::Workflow::Oblige { .. }
        | ash_parser::surface::Workflow::Resume { .. } => Ok(()),
    }
}

/// Compute the type signature of an ordinary `fn` definition.
pub fn fn_signature_type(
    env: &TypeEnv,
    function: &ash_parser::surface::FnDef,
) -> Result<Type, TypeCheckError> {
    let signature_env = register_surface_type_parameter_kinds(env, &function.type_params)?;

    let type_param_bindings: std::collections::HashMap<String, Type> = function
        .type_params
        .iter()
        .map(|param| (param.name.to_string(), Type::Var(TypeVar::fresh())))
        .collect();

    let params = function
        .params
        .iter()
        .map(|param| workflow_surface_type_to_type(&signature_env, &param.ty, &type_param_bindings))
        .collect::<Result<Vec<_>, _>>()?;

    let ret = match &function.return_type {
        Some(ret) => workflow_surface_type_to_type(&signature_env, ret, &type_param_bindings)?,
        None => Type::Var(TypeVar::fresh()),
    };

    Ok(Type::Fn(params, Box::new(ret)))
}

/// Compute the type signature of a `builtin fn` definition.
///
/// Builtin fns are pure functions with no body -- they type identically to
/// regular `fn` definitions (`Type::Fn(params, ret)`). The return type is
/// always present (required by the grammar).
pub fn builtin_fn_signature_type(
    env: &TypeEnv,
    builtin_fn: &ash_parser::surface::BuiltinFnDef,
) -> Result<Type, TypeCheckError> {
    let signature_env = register_surface_type_parameter_kinds(env, &builtin_fn.type_params)?;

    let type_param_bindings: std::collections::HashMap<String, Type> = builtin_fn
        .type_params
        .iter()
        .map(|param| (param.name.to_string(), Type::Var(TypeVar::fresh())))
        .collect();

    let params = builtin_fn
        .params
        .iter()
        .map(|param| workflow_surface_type_to_type(&signature_env, &param.ty, &type_param_bindings))
        .collect::<Result<Vec<_>, _>>()?;

    let ret = workflow_surface_type_to_type(
        &signature_env,
        &builtin_fn.return_type,
        &type_param_bindings,
    )?;

    Ok(Type::Fn(params, Box::new(ret)))
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
        .map_err(TypeCheckError::from)?;
    Ok(())
}

fn register_function_signatures(
    env: &mut TypeEnv,
    definitions: &[ash_parser::surface::Definition],
) -> Result<(), TypeCheckError> {
    let mut staged = env.clone();
    register_function_signatures_inner(&mut staged, definitions)?;
    *env = staged;
    Ok(())
}

fn register_function_signatures_inner(
    env: &mut TypeEnv,
    definitions: &[ash_parser::surface::Definition],
) -> Result<(), TypeCheckError> {
    for (index, definition) in definitions.iter().enumerate() {
        match definition {
            ash_parser::surface::Definition::Function(function) => {
                let signature = fn_signature_type(env, function)?;
                env.bind_variable(function.name.as_ref(), signature);
                if matches!(function.visibility, ash_parser::surface::Visibility::Public)
                    && let Some(tail) = &function.proposition_tail
                {
                    register_public_function_proposition_tail(
                        env,
                        tail,
                        function.name.as_ref(),
                        "function",
                        0x8801_0000u64 + index as u64,
                    )?;
                }
            }
            ash_parser::surface::Definition::BuiltinFn(builtin_fn) => {
                let signature = builtin_fn_signature_type(env, builtin_fn)?;
                env.bind_variable(builtin_fn.name.as_ref(), signature);
                if matches!(
                    builtin_fn.visibility,
                    ash_parser::surface::Visibility::Public
                ) && let Some(tail) = &builtin_fn.proposition_tail
                {
                    register_public_function_proposition_tail(
                        env,
                        tail,
                        builtin_fn.name.as_ref(),
                        "builtin function",
                        0x8802_0000u64 + index as u64,
                    )?;
                }
            }
            ash_parser::surface::Definition::Capability(capability) => {
                env.register_capability_symbol(capability.name.as_ref());
            }
            _ => {}
        }
    }
    Ok(())
}

fn collect_type_vars(ty: &Type, vars: &mut HashSet<TypeVar>) {
    match ty {
        Type::Var(var) => {
            vars.insert(*var);
        }
        Type::List(item) => collect_type_vars(item, vars),
        Type::Record(fields) => {
            for (_, field_ty) in fields {
                collect_type_vars(field_ty, vars);
            }
        }
        Type::Fun(args, ret, _) => {
            for arg_ty in args {
                collect_type_vars(arg_ty, vars);
            }
            collect_type_vars(ret, vars);
        }
        Type::Fn(params, ret) => {
            for param_ty in params {
                collect_type_vars(param_ty, vars);
            }
            collect_type_vars(ret, vars);
        }
        Type::Constructor { args, .. } => {
            for arg_ty in args {
                collect_type_vars(arg_ty, vars);
            }
        }
        Type::ConstructorVariableApp { args, .. } => {
            for arg_ty in args {
                collect_type_vars(arg_ty, vars);
            }
        }
        Type::Associated { base, .. } => {
            collect_type_vars(base, vars);
        }
        Type::Int
        | Type::String
        | Type::Bool
        | Type::Float
        | Type::Null
        | Type::Time
        | Type::Ref
        | Type::Cap { .. }
        | Type::Instance { .. }
        | Type::InstanceAddr { .. }
        | Type::ControlLink { .. } => {}
    }
}

fn validate_inferred_function_return(
    function: &ash_parser::surface::FnDef,
    param_types: &[Type],
    return_ty: &Type,
) -> Result<(), TypeCheckError> {
    if function.return_type.is_some() {
        return Ok(());
    }

    let mut allowed_vars = HashSet::new();
    for param_ty in param_types {
        collect_type_vars(param_ty, &mut allowed_vars);
    }

    let mut return_vars = HashSet::new();
    collect_type_vars(return_ty, &mut return_vars);

    if return_vars.iter().any(|var| !allowed_vars.contains(var)) {
        return Err(TypeCheckError::TypeError(format!(
            "fn '{}' omitted return type could not be inferred; add an explicit return type",
            function.name
        )));
    }

    Ok(())
}

fn validate_fn_contract_namespace(
    function: &ash_parser::surface::FnDef,
    lowered: &ash_parser::LoweredFnContract,
) -> Result<(), TypeCheckError> {
    let param_names: HashSet<&str> = function
        .params
        .iter()
        .map(|param| param.name.as_ref())
        .collect();

    for requirement in &lowered.contract.requires {
        let ash_core::workflow_contract::Requirement::Arithmetic { var, .. } = requirement else {
            continue;
        };

        if !param_names.contains(var.as_str()) {
            return Err(TypeCheckError::TypeError(format!(
                "fn contract references unknown variable '{}' in requires for fn '{}'",
                var, function.name
            )));
        }
    }

    for predicate in &lowered.runtime_postconditions.predicates {
        match predicate {
            ash_core::workflow_contract::PostPredicate::ResultSatisfies(_) => {}
            ash_core::workflow_contract::PostPredicate::Eq(left, right) => {
                for variable in [left, right] {
                    if variable == "result" {
                        continue;
                    }

                    let is_literal = matches!(variable.as_str(), "true" | "false" | "null")
                        || variable.parse::<i64>().is_ok()
                        || (variable.starts_with('"') && variable.ends_with('"'));

                    if !is_literal && !param_names.contains(variable.as_str()) {
                        return Err(TypeCheckError::TypeError(format!(
                            "fn contract references unknown variable '{}' in ensures for fn '{}'",
                            variable, function.name
                        )));
                    }
                }
            }
            ash_core::workflow_contract::PostPredicate::StateAssertion(_) => {
                return Err(TypeCheckError::TypeError(format!(
                    "fn '{}' lowered an unsupported stateful ensures predicate",
                    function.name
                )));
            }
        }
    }

    Ok(())
}

fn refine_function_signatures(
    env: &mut TypeEnv,
    definitions: &[ash_parser::surface::Definition],
) -> Result<(), TypeCheckError> {
    let function_count = definitions
        .iter()
        .filter(|definition| matches!(definition, ash_parser::surface::Definition::Function(_)))
        .count();

    for _ in 0..function_count {
        let mut changed = false;

        for definition in definitions {
            let ash_parser::surface::Definition::Function(function) = definition else {
                continue;
            };

            let (inferred_return_ty, lowered_contract) = check_function_def_in_env(env, function)?;
            let signature = env.lookup_variable(function.name.as_ref()).ok_or_else(|| {
                TypeCheckError::TypeError(format!("missing signature for fn '{}'", function.name))
            })?;
            let Type::Fn(params, current_return_ty) = signature else {
                return Err(TypeCheckError::TypeError(format!(
                    "fn '{}' did not register as a pure function",
                    function.name
                )));
            };

            let stabilized_return_ty = if *current_return_ty == inferred_return_ty {
                *current_return_ty
            } else {
                changed = true;
                inferred_return_ty
            };

            env.bind_variable(
                function.name.as_ref(),
                Type::Fn(params, Box::new(stabilized_return_ty)),
            );
            env.bind_fn_contract(
                function.name.as_ref(),
                StoredFnContract {
                    param_names: function
                        .params
                        .iter()
                        .map(|param| param.name.to_string())
                        .collect(),
                    contract: lowered_contract.contract,
                    runtime_postconditions: lowered_contract.runtime_postconditions,
                },
            );
        }

        if !changed {
            break;
        }
    }

    for definition in definitions {
        let ash_parser::surface::Definition::Function(function) = definition else {
            continue;
        };

        let (inferred_return_ty, _) = check_function_def_in_env(env, function)?;
        let signature = env.lookup_variable(function.name.as_ref()).ok_or_else(|| {
            TypeCheckError::TypeError(format!("missing signature for fn '{}'", function.name))
        })?;
        let Type::Fn(params, current_return_ty) = signature else {
            return Err(TypeCheckError::TypeError(format!(
                "fn '{}' did not register as a pure function",
                function.name
            )));
        };

        if *current_return_ty != inferred_return_ty {
            return Err(TypeCheckError::TypeError(format!(
                "fn '{}' omitted return type could not be stabilized; add an explicit return type",
                function.name
            )));
        }

        validate_inferred_function_return(function, &params, &current_return_ty)?;
    }

    Ok(())
}

fn int_fact_from_expr(
    facts: &std::collections::HashMap<String, i64>,
    expr: &ash_parser::surface::Expr,
) -> Option<i64> {
    match expr {
        ash_parser::surface::Expr::Literal(ash_parser::surface::Literal::Int(value)) => {
            Some(*value)
        }
        ash_parser::surface::Expr::Variable { name, .. } => facts.get(name.as_ref()).copied(),
        ash_parser::surface::Expr::Unary {
            op: ash_parser::surface::UnaryOp::Neg,
            operand,
            ..
        } => int_fact_from_expr(facts, operand).map(|value| -value),
        ash_parser::surface::Expr::Binary {
            op, left, right, ..
        } => {
            let left = int_fact_from_expr(facts, left)?;
            let right = int_fact_from_expr(facts, right)?;
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

fn assumption_from_condition(
    condition: &ash_parser::surface::Expr,
    truthy: bool,
) -> Option<(String, ash_core::workflow_contract::ArithConstraint)> {
    use ash_core::workflow_contract::ArithConstraint;
    use ash_parser::surface::BinaryOp;

    let ash_parser::surface::Expr::Binary {
        op, left, right, ..
    } = condition
    else {
        return None;
    };

    let (var, value, normalized_op) = match (&**left, &**right) {
        (
            ash_parser::surface::Expr::Variable { name, .. },
            ash_parser::surface::Expr::Literal(ash_parser::surface::Literal::Int(value)),
        ) => (name.to_string(), *value, *op),
        (
            ash_parser::surface::Expr::Literal(ash_parser::surface::Literal::Int(value)),
            ash_parser::surface::Expr::Variable { name, .. },
        ) => {
            let swapped = match op {
                BinaryOp::Lt => BinaryOp::Gt,
                BinaryOp::Leq => BinaryOp::Geq,
                BinaryOp::Gt => BinaryOp::Lt,
                BinaryOp::Geq => BinaryOp::Leq,
                other => *other,
            };
            (name.to_string(), *value, swapped)
        }
        _ => return None,
    };

    let constraint = match (truthy, normalized_op) {
        (true, BinaryOp::Lt) => ArithConstraint::Lt(value),
        (true, BinaryOp::Leq) => ArithConstraint::Lte(value),
        (true, BinaryOp::Gt) => ArithConstraint::Gt(value),
        (true, BinaryOp::Geq) => ArithConstraint::Gte(value),
        (true, BinaryOp::Eq) => ArithConstraint::Eq(value),
        (true, BinaryOp::Neq) => ArithConstraint::NotEq(value),
        (false, BinaryOp::Lt) => ArithConstraint::Gte(value),
        (false, BinaryOp::Leq) => ArithConstraint::Gt(value),
        (false, BinaryOp::Gt) => ArithConstraint::Lte(value),
        (false, BinaryOp::Geq) => ArithConstraint::Lt(value),
        (false, BinaryOp::Eq) => ArithConstraint::NotEq(value),
        (false, BinaryOp::Neq) => ArithConstraint::Eq(value),
        _ => return None,
    };

    Some((var, constraint))
}

fn build_requirement_context(
    facts: &std::collections::HashMap<String, i64>,
    assumptions: &std::collections::HashMap<
        String,
        Vec<ash_core::workflow_contract::ArithConstraint>,
    >,
    param_names: &[String],
    args: &[ash_parser::surface::Expr],
) -> RequirementContext {
    let mut ctx = RequirementContext::new();
    for (param_name, arg) in param_names.iter().zip(args.iter()) {
        if let Some(value) = int_fact_from_expr(facts, arg) {
            ctx = ctx.with_fact(param_name.clone(), value);
            continue;
        }

        if let ash_parser::surface::Expr::Variable { name, .. } = arg {
            let Some(constraints) = assumptions.get(name.as_ref()) else {
                continue;
            };
            for constraint in constraints {
                ctx = ctx.with_arithmetic_assumption(param_name.clone(), constraint.clone());
            }
        }
    }
    ctx
}

fn validate_fn_call_preconditions_expr(
    env: &TypeEnv,
    expr: &ash_parser::surface::Expr,
    facts: &std::collections::HashMap<String, i64>,
    assumptions: &std::collections::HashMap<
        String,
        Vec<ash_core::workflow_contract::ArithConstraint>,
    >,
) -> Result<(), TypeCheckError> {
    match expr {
        ash_parser::surface::Expr::Unary { operand, .. }
        | ash_parser::surface::Expr::FieldAccess { base: operand, .. } => {
            validate_fn_call_preconditions_expr(env, operand, facts, assumptions)
        }
        ash_parser::surface::Expr::IndexAccess { base, index, .. } => {
            validate_fn_call_preconditions_expr(env, base, facts, assumptions)?;
            validate_fn_call_preconditions_expr(env, index, facts, assumptions)
        }
        ash_parser::surface::Expr::Binary { left, right, .. } => {
            validate_fn_call_preconditions_expr(env, left, facts, assumptions)?;
            validate_fn_call_preconditions_expr(env, right, facts, assumptions)
        }
        ash_parser::surface::Expr::Call {
            func, module, args, ..
        } => {
            for arg in args {
                validate_fn_call_preconditions_expr(env, arg, facts, assumptions)?;
            }

            let contract_name = module
                .as_ref()
                .map(|module| format!("{module}::{func}"))
                .unwrap_or_else(|| func.to_string());
            if let Some(boundary) = env.lookup_fn_contract(&contract_name) {
                let ctx =
                    build_requirement_context(facts, assumptions, &boundary.param_names, args);

                let contract_result = check_contract(&boundary.contract, &ctx);
                if !contract_result.is_success() {
                    let details = contract_result
                        .errors()
                        .into_iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ");
                    return Err(TypeCheckError::TypeError(format!(
                        "fn precondition may not hold for call '{}': {details}",
                        contract_name
                    )));
                }
            }

            Ok(())
        }
        ash_parser::surface::Expr::Match {
            scrutinee, arms, ..
        } => {
            validate_fn_call_preconditions_expr(env, scrutinee, facts, assumptions)?;
            for arm in arms {
                validate_fn_call_preconditions_expr(env, &arm.body, facts, assumptions)?;
            }
            Ok(())
        }
        ash_parser::surface::Expr::IfLet {
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            validate_fn_call_preconditions_expr(env, expr, facts, assumptions)?;
            validate_fn_call_preconditions_expr(env, then_branch, facts, assumptions)?;
            validate_fn_call_preconditions_expr(env, else_branch, facts, assumptions)
        }
        ash_parser::surface::Expr::Constructor {
            fields, payload, ..
        } => {
            for (_, value) in fields {
                validate_fn_call_preconditions_expr(env, value, facts, assumptions)?;
            }
            match payload {
                ash_parser::surface::ConstructorPayload::Unit => {}
                ash_parser::surface::ConstructorPayload::Record(fields) => {
                    for (_, value) in fields {
                        validate_fn_call_preconditions_expr(env, value, facts, assumptions)?;
                    }
                }
                ash_parser::surface::ConstructorPayload::Tuple(items) => {
                    for value in items {
                        validate_fn_call_preconditions_expr(env, value, facts, assumptions)?;
                    }
                }
            }
            Ok(())
        }
        ash_parser::surface::Expr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            validate_fn_call_preconditions_expr(env, condition, facts, assumptions)?;
            let mut then_assumptions = assumptions.clone();
            if let Some((var, constraint)) = assumption_from_condition(condition, true) {
                then_assumptions.entry(var).or_default().push(constraint);
            }
            validate_fn_call_preconditions_expr(env, then_branch, facts, &then_assumptions)?;
            if let Some(else_branch) = else_branch {
                let mut else_assumptions = assumptions.clone();
                if let Some((var, constraint)) = assumption_from_condition(condition, false) {
                    else_assumptions.entry(var).or_default().push(constraint);
                }
                validate_fn_call_preconditions_expr(env, else_branch, facts, &else_assumptions)?;
            }
            Ok(())
        }
        ash_parser::surface::Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            let mut nested_facts = facts.clone();
            let nested_assumptions = assumptions.clone();
            for statement in statements {
                let ash_parser::surface::BlockStmt::Let { pattern, expr, .. } = statement;
                validate_fn_call_preconditions_expr(env, expr, &nested_facts, &nested_assumptions)?;
                if let (ash_parser::surface::Pattern::Variable { name, .. }, Some(value)) =
                    (pattern, int_fact_from_expr(&nested_facts, expr))
                {
                    nested_facts.insert(name.to_string(), value);
                }
            }
            if let Some(tail_expr) = tail_expr {
                validate_fn_call_preconditions_expr(
                    env,
                    tail_expr,
                    &nested_facts,
                    &nested_assumptions,
                )?;
            }
            Ok(())
        }
        ash_parser::surface::Expr::Literal(_)
        | ash_parser::surface::Expr::Variable { .. }
        | ash_parser::surface::Expr::Policy(_)
        | ash_parser::surface::Expr::CheckObligation { .. }
        | ash_parser::surface::Expr::Panic { .. } => Ok(()),
        ash_parser::surface::Expr::Fail { payload, .. } => {
            validate_fn_call_preconditions_expr(env, payload, facts, assumptions)
        }
        ash_parser::surface::Expr::WithError { body, arms, .. } => {
            validate_fn_call_preconditions_expr(env, body, facts, assumptions)?;
            for arm in arms {
                validate_fn_call_preconditions_expr(env, &arm.body, facts, assumptions)?;
            }
            Ok(())
        }
        ash_parser::surface::Expr::FnDef { body, params, .. } => {
            let mut fn_env = env.clone();
            for (name, _ty) in params {
                fn_env.bind_variable(name.as_ref(), Type::Var(TypeVar::fresh()));
            }
            validate_fn_call_preconditions_expr(&fn_env, body, facts, assumptions)
        }
        ash_parser::surface::Expr::FnApply { func, args, .. } => {
            validate_fn_call_preconditions_expr(env, func, facts, assumptions)?;
            for arg in args {
                validate_fn_call_preconditions_expr(env, arg, facts, assumptions)?;
            }
            Ok(())
        }
        ash_parser::surface::Expr::ActBlock { stmts, .. } => {
            use ash_parser::surface::ActStmt;
            for stmt in stmts {
                let value = match stmt {
                    ActStmt::Bind { value, .. } => value,
                    ActStmt::Return { value, .. } => value,
                };
                validate_fn_call_preconditions_expr(env, value, facts, assumptions)?;
            }
            Ok(())
        }
        ash_parser::surface::Expr::DoBlock { .. } => Err(TypeCheckError::TypeError(
            "generalized do-block type checking is not implemented (TASK-747 parser substrate only)"
                .to_string(),
        )),
        ash_parser::surface::Expr::Comprehension {
            result,
            qualifiers,
            ..
        } => {
            use ash_parser::surface::ComprehensionQualifier;
            for qualifier in qualifiers {
                let value = match qualifier {
                    ComprehensionQualifier::Let { value, .. }
                    | ComprehensionQualifier::Bind { value, .. }
                    | ComprehensionQualifier::DiscardBind { value, .. } => value,
                };
                validate_fn_call_preconditions_expr(env, value, facts, assumptions)?;
            }
            validate_fn_call_preconditions_expr(env, result, facts, assumptions)
        }
        ash_parser::surface::Expr::List { items, .. } => {
            for item in items {
                validate_fn_call_preconditions_expr(env, item, facts, assumptions)?;
            }
            Ok(())
        }
    }
}

fn validate_fn_call_preconditions_workflow(
    env: &TypeEnv,
    workflow: &ash_parser::surface::Workflow,
    facts: &mut std::collections::HashMap<String, i64>,
    assumptions: &mut std::collections::HashMap<
        String,
        Vec<ash_core::workflow_contract::ArithConstraint>,
    >,
) -> Result<(), TypeCheckError> {
    match workflow {
        ash_parser::surface::Workflow::Orient {
            expr, continuation, ..
        } => {
            validate_fn_call_preconditions_expr(env, expr, facts, assumptions)?;
            if let Some(continuation) = continuation {
                validate_fn_call_preconditions_workflow(env, continuation, facts, assumptions)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::Ret { expr, .. } => {
            validate_fn_call_preconditions_expr(env, expr, facts, assumptions)
        }
        ash_parser::surface::Workflow::Let {
            pattern,
            expr,
            continuation,
            ..
        } => {
            validate_fn_call_preconditions_expr(env, expr, facts, assumptions)?;
            if let (ash_parser::surface::Pattern::Variable { name, .. }, Some(value)) =
                (pattern, int_fact_from_expr(facts, expr))
            {
                facts.insert(name.to_string(), value);
            }
            if let Some(continuation) = continuation {
                validate_fn_call_preconditions_workflow(env, continuation, facts, assumptions)?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            validate_fn_call_preconditions_expr(env, condition, facts, assumptions)?;
            let mut then_facts = facts.clone();
            let mut then_assumptions = assumptions.clone();
            if let Some((var, constraint)) = assumption_from_condition(condition, true) {
                then_assumptions.entry(var).or_default().push(constraint);
            }
            validate_fn_call_preconditions_workflow(
                env,
                then_branch,
                &mut then_facts,
                &mut then_assumptions,
            )?;
            if let Some(else_branch) = else_branch {
                let mut else_facts = facts.clone();
                let mut else_assumptions = assumptions.clone();
                if let Some((var, constraint)) = assumption_from_condition(condition, false) {
                    else_assumptions.entry(var).or_default().push(constraint);
                }
                validate_fn_call_preconditions_workflow(
                    env,
                    else_branch,
                    &mut else_facts,
                    &mut else_assumptions,
                )?;
            }
            Ok(())
        }
        ash_parser::surface::Workflow::Seq { first, second, .. } => {
            validate_fn_call_preconditions_workflow(env, first, facts, assumptions)?;
            validate_fn_call_preconditions_workflow(env, second, facts, assumptions)
        }
        ash_parser::surface::Workflow::Done { .. }
        | ash_parser::surface::Workflow::Oblige { .. } => Ok(()),
        _ => Ok(()),
    }
}

fn check_function_def_in_env(
    env: &TypeEnv,
    function: &ash_parser::surface::FnDef,
) -> Result<(Type, ash_parser::LoweredFnContract), TypeCheckError> {
    let signature = env.lookup_variable(function.name.as_ref()).ok_or_else(|| {
        TypeCheckError::TypeError(format!("missing signature for fn '{}'", function.name))
    })?;

    let (param_types, declared_return_ty) = match signature {
        Type::Fn(params, ret) => (params, *ret),
        other => {
            return Err(TypeCheckError::TypeError(format!(
                "fn '{}' did not register as a pure function, found {}",
                function.name, other
            )));
        }
    };

    let mut fn_env = env.extend();
    fn_env.bind_variable(
        function.name.as_ref(),
        Type::Fn(param_types.clone(), Box::new(declared_return_ty.clone())),
    );
    for (param, ty) in function.params.iter().zip(param_types.iter()) {
        if matches!(ty, Type::Cap { .. }) {
            return Err(TypeCheckError::TypeError(format!(
                "fn '{}' parameter '{}' cannot have capability type {}",
                function.name, param.name, ty
            )));
        }
        fn_env.bind_variable(param.name.as_ref(), ty.clone());
    }

    let lowered_contract = ash_parser::lower_fn_contract(function.contract.as_ref())
        .map_err(|error| TypeCheckError::TypeError(error.to_string()))?;
    validate_fn_contract_namespace(function, &lowered_contract)?;

    let allow_effects = matches!(
        &declared_return_ty,
        Type::Constructor { name, .. } if name.name == "Act"
    );

    crate::purity::check_purity(&fn_env, &function.body, allow_effects).map_err(|errors| {
        TypeCheckError::TypeError(
            errors
                .into_iter()
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join("; "),
        )
    })?;

    let body_ty = infer_surface_expr_type(&fn_env, &function.body)?;
    crate::types::unify(&declared_return_ty, &body_ty)
        .map(|subst| (subst.apply(&declared_return_ty), lowered_contract))
        .map_err(|_| {
            TypeCheckError::TypeError(format!(
                "fn '{}' declared return type {} but body returns {}",
                function.name, declared_return_ty, body_ty
            ))
        })
}

fn workflow_header_type_name(ty: &ash_parser::surface::Type) -> Option<String> {
    match ty {
        ash_parser::surface::Type::Name(name) | ash_parser::surface::Type::Capability(name) => {
            Some(name.to_string())
        }
        _ => None,
    }
}

#[derive(Debug, Clone)]
struct WorkflowBindingAuthoritySummary {
    interface: String,
}

fn config_binding_name(arg: &ash_parser::surface::Expr) -> String {
    match arg {
        ash_parser::surface::Expr::Variable { name, .. } => name.to_string(),
        _ => "<config-expression>".to_string(),
    }
}

fn validate_workflow_resource_and_binding_headers(
    env: &TypeEnv,
    workflow_env: &mut TypeEnv,
    workflow: &ash_parser::surface::WorkflowDef,
) -> Result<AuthorityProvenanceReport, TypeCheckError> {
    let mut provenance = AuthorityProvenanceReport::default();
    let mut owned_resources = std::collections::HashMap::new();
    for owned in &workflow.owned_resources {
        let name = owned.name.to_string();
        if owned_resources.contains_key(&name) {
            return Err(TypeCheckError::TypeError(format!(
                "duplicate owned resource '{name}' in workflow '{}'",
                workflow.name
            )));
        }
        let resource_type = workflow_header_type_name(&owned.ty).ok_or_else(|| {
            TypeCheckError::TypeError(format!(
                "owned resource '{name}' must name a registered resource type"
            ))
        })?;
        if !env.has_resource_type(&resource_type) {
            return Err(TypeCheckError::TypeError(format!(
                "owned resource '{name}' references unknown resource type '{resource_type}'"
            )));
        }
        provenance
            .resource_bindings
            .push(ResourceBindingProvenanceInfo {
                name: name.clone(),
                resource_type: resource_type.clone(),
                authority: AuthorityProvenanceKind::Internal,
            });
        owned_resources.insert(name, resource_type);
    }

    let mut used_bindings: std::collections::HashMap<String, WorkflowBindingAuthoritySummary> =
        std::collections::HashMap::new();
    for binding in &workflow.used_bindings {
        let binding_name = binding.name.to_string();
        if used_bindings.contains_key(&binding_name) {
            return Err(TypeCheckError::TypeError(format!(
                "duplicate used binding '{binding_name}' in workflow '{}'",
                workflow.name
            )));
        }
        let interface_name = workflow_header_type_name(&binding.interface).ok_or_else(|| {
            TypeCheckError::TypeError(format!(
                "used binding '{binding_name}' must annotate a capability interface"
            ))
        })?;
        if !env.has_capability_interface(&interface_name) {
            return Err(TypeCheckError::TypeError(format!(
                "used binding '{binding_name}' references unknown capability interface '{interface_name}'"
            )));
        }

        let ash_parser::surface::Expr::Call {
            module: None,
            func,
            args,
            ..
        } = &binding.implementation
        else {
            return Err(TypeCheckError::TypeError(format!(
                "unsupported uses implementation expression for binding '{binding_name}': expected unqualified implementation call"
            )));
        };

        let implementation_name = func.to_string();
        let implementation = env.lookup_capability_implementation(&implementation_name).ok_or_else(|| {
            TypeCheckError::TypeError(format!(
                "used binding '{binding_name}' references unknown implementation '{implementation_name}'"
            ))
        })?;
        if implementation.interface != interface_name {
            return Err(TypeCheckError::TypeError(format!(
                "implementation '{implementation_name}' targets '{}' not '{interface_name}' for binding '{binding_name}'",
                implementation.interface
            )));
        }
        if implementation.dependencies.len() != args.len() {
            return Err(TypeCheckError::TypeError(format!(
                "used binding '{binding_name}' dependency arity mismatch for implementation '{implementation_name}': expected {}, found {}",
                implementation.dependencies.len(),
                args.len()
            )));
        }

        let mut sources = Vec::with_capacity(implementation.dependencies.len());
        for (dependency, arg) in implementation.dependencies.iter().zip(args.iter()) {
            match dependency.kind {
                ash_parser::surface::CapabilityImplementationDependencyKind::Resource => {
                    let ash_parser::surface::Expr::Variable { name, .. } = arg else {
                        return Err(TypeCheckError::TypeError(format!(
                            "resource dependency '{}' for binding '{binding_name}' must be an owned resource variable",
                            dependency.name
                        )));
                    };
                    let Some(actual_resource_type) = owned_resources.get(name.as_ref()) else {
                        return Err(TypeCheckError::TypeError(format!(
                            "resource dependency '{}' for binding '{binding_name}' must reference workflow owned resource '{}'; no matching owned resource found",
                            dependency.name, name
                        )));
                    };
                    let expected = dependency.target_name.as_deref().unwrap_or_default();
                    if actual_resource_type != expected {
                        return Err(TypeCheckError::TypeError(format!(
                            "resource dependency '{}' for binding '{binding_name}' expected resource type '{expected}', found '{actual_resource_type}'",
                            dependency.name
                        )));
                    }
                    sources.push(BindingProvenanceSourceInfo {
                        kind: ProvenanceSourceKind::Resource,
                        dependency_name: dependency.name.clone(),
                        binding_name: name.to_string(),
                        target_name: expected.to_string(),
                    });
                }
                ash_parser::surface::CapabilityImplementationDependencyKind::Capability => {
                    let ash_parser::surface::Expr::Variable { name, .. } = arg else {
                        return Err(TypeCheckError::TypeError(format!(
                            "capability dependency '{}' for binding '{binding_name}' must be an earlier used binding variable",
                            dependency.name
                        )));
                    };
                    let Some(actual_binding) = used_bindings.get(name.as_ref()) else {
                        return Err(TypeCheckError::TypeError(format!(
                            "capability dependency '{}' for binding '{binding_name}' must reference earlier used binding '{}'; no matching earlier used binding found",
                            dependency.name, name
                        )));
                    };
                    let expected = dependency.target_name.as_deref().unwrap_or_default();
                    if actual_binding.interface != expected {
                        return Err(TypeCheckError::TypeError(format!(
                            "capability dependency '{}' for binding '{binding_name}' expected interface '{expected}', found '{}'",
                            dependency.name, actual_binding.interface
                        )));
                    }
                    sources.push(BindingProvenanceSourceInfo {
                        kind: ProvenanceSourceKind::Capability,
                        dependency_name: dependency.name.clone(),
                        binding_name: name.to_string(),
                        target_name: expected.to_string(),
                    });
                }
                ash_parser::surface::CapabilityImplementationDependencyKind::Config => {
                    let actual_ty = infer_surface_expr_type(workflow_env, arg)?;
                    crate::types::unify(&dependency.ty, &actual_ty).map_err(|_| {
                        TypeCheckError::TypeError(format!(
                            "config dependency '{}' for binding '{binding_name}' expected {}, found {}",
                            dependency.name, dependency.ty, actual_ty
                        ))
                    })?;
                    sources.push(BindingProvenanceSourceInfo {
                        kind: ProvenanceSourceKind::Config,
                        dependency_name: dependency.name.clone(),
                        binding_name: config_binding_name(arg),
                        target_name: dependency.ty.to_string(),
                    });
                }
            }
        }

        if implementation.authority_provenance == AuthorityProvenanceKind::Derived
            && !sources
                .iter()
                .any(|source| source.kind == ProvenanceSourceKind::Capability)
        {
            return Err(TypeCheckError::TypeError(format!(
                "derived binding '{binding_name}' for implementation '{implementation_name}' has no declared capability authority source"
            )));
        }

        let capability_binding_info = CapabilityBindingInfo {
            name: binding_name.clone(),
            interface: interface_name.clone(),
            implementation: implementation_name.clone(),
            authority: implementation.authority_provenance,
        };

        provenance
            .capability_bindings
            .push(CapabilityBindingProvenanceInfo {
                name: binding_name.clone(),
                interface: interface_name.clone(),
                implementation: implementation_name,
                authority: implementation.authority_provenance,
                sources,
            });
        workflow_env.register_capability_binding(capability_binding_info);
        used_bindings.insert(
            binding_name,
            WorkflowBindingAuthoritySummary {
                interface: interface_name,
            },
        );
    }

    Ok(provenance)
}

/// Type-check a workflow definition against an explicitly prepared type environment.
pub fn type_check_workflow_def_in_env(
    env: &TypeEnv,
    workflow: &ash_parser::surface::WorkflowDef,
) -> Result<TypeCheckResult, TypeCheckError> {
    // NOTE: Previously we validated bounds here, but now we need workflow_env
    // set up first so that associated type resolution has access to interface bounds.
    // Bounds are validated below after workflow_env is created.
    let mut workflow_env = register_surface_type_parameter_kinds(env, &workflow.type_params)?;

    let type_param_bindings: std::collections::HashMap<String, Type> = workflow
        .type_params
        .iter()
        .map(|param| (param.name.to_string(), Type::Var(TypeVar::fresh())))
        .collect();

    // Create workflow_env first so we can bind interface bounds before
    // resolving associated types in parameter and return types.
    for type_param in &workflow.type_params {
        if let Some(Type::Var(var)) = type_param_bindings.get(type_param.name.as_ref()) {
            for bound in &type_param.bounds {
                // Validate that the interface exists before binding
                if !workflow_env.has_interface(bound.interface.as_ref()) {
                    return Err(TypeCheckError::TypeError(format!(
                        "Unknown interface bound '{}' on type parameter '{}'",
                        bound.interface, type_param.name
                    )));
                }
                workflow_env.bind_type_var_interface_bound(*var, bound.interface.as_ref());
            }
        }
    }

    let mut param_bindings = Vec::with_capacity(workflow.params.len());
    for param in &workflow.params {
        let ty = workflow_surface_type_to_type(&workflow_env, &param.ty, &type_param_bindings)?;
        param_bindings.push((param.name.to_string(), ty));
    }

    let declared_return_ty = workflow
        .declared_return_type
        .as_ref()
        .map(|return_ty| {
            workflow_surface_type_to_type(&workflow_env, return_ty, &type_param_bindings)
        })
        .transpose()?;

    for (name, ty) in &param_bindings {
        workflow_env.bind_variable(name, ty.clone());
    }

    let authority_provenance =
        validate_workflow_resource_and_binding_headers(env, &mut workflow_env, workflow)?;

    reject_unsupported_mvp_workflow_features(&workflow.body)?;

    // SPEC-031 §4.8: Mark this as a workflow context so that Expr::FnDef
    // inside the body is typed as Type::Fun (impure) rather than Type::Fn (pure).
    // Without this, closures in workflows would incorrectly get the pure type,
    // defeating the three-vertex boundary.
    //
    // Note: set_workflow_effect fires unconditionally for all workflows (the body
    // must know its context), but the Fun-escape return-type check below only
    // fires when a declared return type is present.  Workflows without declared
    // return types skip the escape check because infer_workflow_return_type may
    // traverse unsupported expression types (e.g. IndexAccess).  The type checker's
    // Fn≠Fun unification already prevents Fun from flowing where Fn is expected,
    // so the explicit escape check is a defense-in-depth for the declared case.
    workflow_env.set_workflow_effect(ash_core::Effect::Operational);

    let mut binder_env = workflow_env.clone();
    validate_irrefutable_workflow_binders(&mut binder_env, &workflow.body)?;

    validate_interface_calls_in_workflow(&mut workflow_env, &workflow.body)?;
    validate_fn_call_preconditions_workflow(
        &workflow_env,
        &workflow.body,
        &mut std::collections::HashMap::new(),
        &mut std::collections::HashMap::new(),
    )?;

    if let Some(expected_return_ty) = declared_return_ty {
        let actual_return_ty = infer_workflow_return_type(&workflow_env, &workflow.body)?;

        // SPEC-031 §4.8 / TASK-558: workflow return types must not contain Type::Fun.
        // Closures (Fun) are impure and must not escape the workflow boundary.
        if crate::types::type_contains_fun(&actual_return_ty) {
            return Err(TypeCheckError::TypeError(format!(
                "workflow '{}' return type {} contains Fun (impure closure) — \
                 closures cannot escape the workflow boundary",
                workflow.name, actual_return_ty
            )));
        }

        crate::types::unify(&expected_return_ty, &actual_return_ty).map_err(|_| {
            TypeCheckError::TypeError(format!(
                "workflow '{}' declared return type {} but body returns {}",
                workflow.name, expected_return_ty, actual_return_ty
            ))
        })?;
    }

    let mut result =
        type_check_workflow_in_env(Some(&workflow_env), &workflow.body, Some(&param_bindings))?;
    result.function_contracts = env.function_contracts();
    result.authority_provenance = authority_provenance;
    Ok(result)
}

pub fn type_check_workflow_def(
    workflow: &ash_parser::surface::WorkflowDef,
) -> Result<TypeCheckResult, TypeCheckError> {
    type_check_workflow_def_in_env(&TypeEnv::with_builtin_types(), workflow)
}

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

    for definition in &program.definitions {
        if let ash_parser::surface::Definition::Interface(interface) = definition {
            env.register_interface(interface)
                .map_err(TypeCheckError::from)?;
        }
    }

    for definition in &program.definitions {
        if let ash_parser::surface::Definition::CapabilityInterface(interface) = definition {
            env.register_capability_interface(interface)
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
        if let ash_parser::surface::Definition::CapabilityImplementation(implementation) =
            definition
        {
            env.register_capability_implementation(implementation)
                .map_err(TypeCheckError::from)?;
        }
    }

    for definition in &program.definitions {
        if let ash_parser::surface::Definition::Impl(implementation) = definition {
            env.register_impl(implementation)
                .map_err(TypeCheckError::from)?;
        }
    }

    register_function_signatures(&mut env, &program.definitions)?;
    refine_function_signatures(&mut env, &program.definitions)?;

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

    type_check_workflow_def_in_env(&env, &program.workflow)
}

/// Type check a workflow
///
/// This is a convenience function that runs the full type checking pipeline:
/// 1. Name resolution
/// 2. Constraint generation
/// 3. Constraint solving
/// 4. Effect inference
/// 5. Obligation checking
///
/// # Example
///
/// ```
/// use ash_typeck::type_check_workflow;
/// use ash_parser::surface::Workflow;
/// use ash_parser::token::Span;
///
/// let workflow = Workflow::Done { span: Span::default() };
/// let result = type_check_workflow(&workflow, None);
/// ```
///
/// With workflow parameters:
///
/// ```
/// use ash_typeck::{type_check_workflow, Type};
/// use ash_parser::surface::Workflow;
/// use ash_parser::token::Span;
///
/// let workflow = Workflow::Done { span: Span::default() };
/// let params = vec![("name".to_string(), Type::String)];
/// let result = type_check_workflow(&workflow, Some(&params));
/// ```
pub fn type_check_workflow(
    workflow: &ash_parser::surface::Workflow,
    param_bindings: Option<&[(String, Type)]>,
) -> Result<TypeCheckResult, TypeCheckError> {
    type_check_workflow_in_env(None, workflow, param_bindings)
}

/// Type check a workflow with optional pre-populated type environment.
/// When `type_env` is provided, imported callable bindings are used for
/// both name resolution and constraint generation.
pub fn type_check_workflow_in_env(
    type_env: Option<&TypeEnv>,
    workflow: &ash_parser::surface::Workflow,
    param_bindings: Option<&[(String, Type)]>,
) -> Result<TypeCheckResult, TypeCheckError> {
    reject_unsupported_mvp_workflow_features(workflow)?;

    let mut binder_env = type_env
        .cloned()
        .unwrap_or_else(TypeEnv::with_builtin_types);
    if let Some(params) = param_bindings {
        for (name, ty) in params {
            binder_env.bind_variable(name, ty.clone());
        }
    }
    validate_irrefutable_workflow_binders(&mut binder_env, workflow)?;

    // Step 1: Name resolution
    let mut resolver = NameResolver::new();

    // Inject workflow parameters into the resolver's scope before checking
    if let Some(params) = param_bindings {
        for (name, _ty) in params {
            resolver.bind(name.clone());
        }
    }

    // Inject imported callable names and registered unit constructor terms into
    // the resolver. Constructors are not lexical variables, but name resolution
    // must allow bare unit constructors so type checking can validate them
    // against TypeEnv's visibility-filtered constructor table.
    if let Some(env) = type_env {
        for name in env.variable_names() {
            resolver.bind(name);
        }
        for name in env.unit_constructor_names() {
            resolver.bind(name);
        }
    }

    // Inject workflow capability binding names into the resolver as admitted
    // non-first-class names. They are typechecked through `TypeEnv` metadata,
    // not exposed as ordinary expression variables.
    if let Some(env) = type_env {
        for name in env.capability_binding_names() {
            resolver.bind(name);
        }
    }

    resolver
        .resolve_workflow(workflow)
        .map_err(|e| TypeCheckError::ResolutionError(format!("{:?}", e)))?;

    // Step 2: Constraint generation
    let mut ctx = crate::constraints::ConstraintContext::new();
    let _ = crate::constraints::generate_workflow_constraints(&mut ctx, workflow);

    // Step 3: Constraint solving
    let mut solver = Solver::new();
    let substitution = solver
        .solve(ctx.constraints())
        .map_err(|e| TypeCheckError::TypeError(format!("{:?}", e)))?;

    // Step 4: Effect inference
    let inferred_effect = crate::effect::infer_effect(workflow);

    // Step 5: Obligation checking using ObligationCollector (TASK-275)
    let mut obligation_ctx = crate::obligations::LinearObligationContext::new();
    let mut collector = crate::obligations::ObligationCollector::new();

    // Collect and verify obligations from the workflow AST
    let obligation_result = collector
        .collect(workflow, &mut obligation_ctx)
        .and_then(|()| collector.finalize(&obligation_ctx))
        .map(|()| crate::obligations::ObligationCheckResult::Success)
        .unwrap_or_else(|_e| {
            // Convert TypeError to obligation check result
            // For now, we track it as a failed obligation
            crate::obligations::ObligationCheckResult::Failed(vec![])
        });

    Ok(TypeCheckResult {
        substitution,
        errors: solver.errors().to_vec(),
        inferred_types: std::collections::HashMap::new(),
        effect: inferred_effect,
        obligation_status: obligation_result,
        function_contracts: std::collections::HashMap::new(),
        authority_provenance: AuthorityProvenanceReport::default(),
    })
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
    use ash_parser::surface::{Expr, Literal, Pattern, Workflow};
    use ash_parser::token::Span;

    fn test_span() -> Span {
        Span::new(0, 0, 1, 1)
    }

    #[test]
    fn test_type_check_workflow_done() {
        let workflow = Workflow::Done { span: test_span() };
        let result = type_check_workflow(&workflow, None);
        assert!(result.is_ok());

        let tc_result = result.unwrap();
        assert!(tc_result.is_ok());
        assert!(tc_result.errors.is_empty());
    }

    #[test]
    fn test_type_check_workflow_let() {
        let workflow = Workflow::Let {
            pattern: Pattern::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            },
            expr: Expr::Literal(Literal::Int(42)),
            continuation: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };

        let result = type_check_workflow(&workflow, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_check_workflow_if() {
        let workflow = Workflow::If {
            condition: Expr::Literal(Literal::Bool(true)),
            then_branch: Box::new(Workflow::Done { span: test_span() }),
            else_branch: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };

        let result = type_check_workflow(&workflow, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_check_workflow_with_params() {
        // Test that workflow parameters are properly bound
        // workflow greet(name: String) { ret "Hello, " + name }
        let workflow = Workflow::Ret {
            expr: Expr::Binary {
                op: ash_parser::surface::BinaryOp::Add,
                left: Box::new(Expr::Literal(Literal::String("Hello, ".into()))),
                right: Box::new(Expr::Variable {
                    name: "name".into(),
                    span: ash_parser::token::Span::default(),
                }),
                span: test_span(),
            },
            span: test_span(),
        };

        // Without parameters, this should fail with UnboundVariable
        let result = type_check_workflow(&workflow, None);
        assert!(result.is_err());

        // With parameters, this should succeed
        let params = vec![("name".to_string(), Type::String)];
        let result = type_check_workflow(&workflow, Some(&params));
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_check_workflow_with_multiple_params() {
        // Test multiple workflow parameters
        let workflow = Workflow::Ret {
            expr: Expr::Binary {
                op: ash_parser::surface::BinaryOp::Add,
                left: Box::new(Expr::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                }),
                right: Box::new(Expr::Variable {
                    name: "y".into(),
                    span: ash_parser::token::Span::default(),
                }),
                span: test_span(),
            },
            span: test_span(),
        };

        // Without parameters, this should fail
        let result = type_check_workflow(&workflow, None);
        assert!(result.is_err());

        // With both parameters, this should succeed
        let params = vec![("x".to_string(), Type::Int), ("y".to_string(), Type::Int)];
        let result = type_check_workflow(&workflow, Some(&params));
        assert!(result.is_ok());
    }

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
    fn test_type_check_result_display_success() {
        let workflow = Workflow::Done { span: test_span() };
        let result = type_check_workflow(&workflow, None).unwrap();
        let display = format!("{result}");
        assert!(display.contains("succeeded"));
    }

    #[test]
    fn test_module_exports() {
        // Test that all modules are accessible via crate root
        let _ = ConstraintContext::new();
        let _ = TypeEnv::with_builtin_types();
        let _ = Type::Int;
    }

    /// SPEC-072 / TASK-959: pure closure syntax remains `Fn` in workflow contexts,
    /// so a workflow may return a closure when its declared return type is a
    /// matching pure callable type.
    #[test]
    fn task959_workflow_return_pure_closure_is_accepted() {
        use ash_parser::surface::{Type as SurfaceType, Workflow, WorkflowDef};
        use ash_parser::token::Span;

        fn test_span() -> Span {
            Span::new(0, 0, 1, 1)
        }

        // Construct a workflow whose body is a FnDef expression.
        // The declared return type is `(Int) -> Int` (a pure function type).
        // TASK-959 keeps pure closure syntax at the Pure stratum even in workflow contexts.
        let workflow = WorkflowDef {
            name: "escape_test".into(),
            type_params: vec![],
            params: vec![],
            declared_return_type: Some(SurfaceType::Fn(
                vec![SurfaceType::Name("Int".into())],
                Box::new(SurfaceType::Name("Int".into())),
            )),
            plays_roles: vec![],
            capabilities: vec![],
            owned_resources: vec![],
            used_bindings: vec![],
            header_events: vec![],
            body: Workflow::Ret {
                expr: ash_parser::surface::Expr::FnDef {
                    params: vec![("x".into(), Some("Int".into()))],
                    return_type: None,
                    body: Box::new(ash_parser::surface::Expr::Variable {
                        name: "x".into(),
                        span: ash_parser::token::Span::default(),
                    }),
                    span: test_span(),
                },
                span: test_span(),
            },
            contract: None,
            span: test_span(),
        };

        let result = type_check_workflow_def_in_env(&TypeEnv::with_builtin_types(), &workflow);
        assert!(
            result.is_ok(),
            "workflow returning a matching pure closure should typecheck, got {result:?}"
        );
    }
}
