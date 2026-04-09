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
pub mod effect;
pub mod effective_caps;
pub mod error;
pub mod exhaustiveness;
pub mod instantiate;
pub mod kind;
pub mod name_binding;
pub mod names;
pub mod obligation_checker;
pub mod obligations;
pub mod policy_check;
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

// Re-export smt module under a unified name
pub use smt as policy;

pub use ash_core::ast::{TypeDef, VariantDef};
pub use capability_check::*;
pub use check_pattern::{Bindings, check_pattern};
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
pub use type_env::TypeEnv;
pub use types::*;
pub use visibility::{ModulePath, VisibilityChecker, VisibilityError, VisibilityExt};

fn workflow_surface_type_to_type(
    env: &TypeEnv,
    ty: &ash_parser::surface::Type,
    type_params: &std::collections::HashMap<String, Type>,
) -> Result<Type, TypeCheckError> {
    match ty {
        ash_parser::surface::Type::Name(name) => {
            if let Some(ty) = type_params.get(name.as_ref()) {
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
                    Ok(Type::Constructor {
                        name: qualified,
                        args: vec![],
                        kind: Kind::Type,
                    })
                }
            }
        }
        ash_parser::surface::Type::List(item) => {
            workflow_surface_type_to_type(env, item, type_params)
                .map(|item| Type::List(Box::new(item)))
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
            let (qualified, _) = env
                .resolve_type(name.as_ref())
                .map_err(|e| TypeCheckError::TypeError(format!("{e}")))?;
            let args = args
                .iter()
                .map(|arg| workflow_surface_type_to_type(env, arg, type_params))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Type::Constructor {
                name: qualified,
                args,
                kind: Kind::Type,
            })
        }
    }
}

fn bind_pattern_variables(env: &mut TypeEnv, pattern: &ash_parser::surface::Pattern, ty: &Type) {
    match pattern {
        ash_parser::surface::Pattern::Variable(name) => {
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
        ash_parser::surface::Expr::Literal(_) | ash_parser::surface::Expr::Variable(_) => Ok(()),
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
        ash_parser::surface::Expr::Call { args, .. } => {
            for arg in args {
                validate_interface_calls_in_expr(env, arg)?;
            }
            Ok(())
        }
        ash_parser::surface::Expr::InterfaceMethodCall {
            interface,
            method,
            argument,
            ..
        } => {
            validate_interface_calls_in_expr(env, argument)?;

            let argument_result = crate::check_expr::check_expr(env, argument);
            if !argument_result.is_ok() {
                let reason = argument_result
                    .errors
                    .into_iter()
                    .next()
                    .map(|error| error.to_string())
                    .unwrap_or_else(|| {
                        format!(
                            "failed to typecheck argument to {}::{}",
                            interface.as_ref(),
                            method.as_ref()
                        )
                    });

                return Err(TypeCheckError::TypeError(format!(
                    "invalid interface method call {}::{}: {}",
                    interface.as_ref(),
                    method.as_ref(),
                    reason
                )));
            }

            env.resolve_interface_method_call(
                interface.as_ref(),
                method.as_ref(),
                &argument_result.ty,
            )
            .map(|_| ())
            .map_err(|error| TypeCheckError::TypeError(error.to_string()))
        }
        ash_parser::surface::Expr::Match {
            scrutinee, arms, ..
        } => validate_match_expr(env, scrutinee, arms),
        ash_parser::surface::Expr::Policy(_)
        | ash_parser::surface::Expr::CheckObligation { .. } => Ok(()),
        ash_parser::surface::Expr::IfLet {
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            validate_interface_calls_in_expr(env, expr)?;
            validate_interface_calls_in_expr(env, then_branch)?;
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
        OperationalTarget::Symbolic { capability_name: _ } => {
            // For symbolic targets, resolution happens during lowering.
            // The resolver maps symbolic names to (provider, action) via explicit metadata.
        }
        OperationalTarget::Qualified {
            module: _,
            capability_name: _,
        } => {
            // For module-qualified targets (e.g., io::fs_read), resolution happens
            // during lowering. The resolver looks up the qualified name in its mappings.
        }
        OperationalTarget::Explicit {
            provider,
            action: action_name,
        } => {
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
            if let Some(binding) = binding {
                // Bind with a fresh type variable, similar to Observe
                // Result semantics for propose actions are not yet implemented
                bind_pattern_variables(&mut next_env, binding, &Type::Var(TypeVar::fresh()));
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
        ash_parser::surface::Workflow::Ret { expr, .. } => infer_surface_expr_type(env, expr)
            .map_err(|_| {
                TypeCheckError::TypeError("failed to typecheck return expression".to_string())
            }),
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
        ash_parser::surface::Workflow::Observe { continuation, .. }
        | ash_parser::surface::Workflow::Orient { continuation, .. }
        | ash_parser::surface::Workflow::Propose { continuation, .. }
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

/// Type-check a workflow definition against an explicitly prepared type environment.
pub fn type_check_workflow_def_in_env(
    env: &TypeEnv,
    workflow: &ash_parser::surface::WorkflowDef,
) -> Result<TypeCheckResult, TypeCheckError> {
    for type_param in &workflow.type_params {
        for bound in &type_param.bounds {
            if !env.has_interface(bound.interface.as_ref()) {
                return Err(TypeCheckError::TypeError(format!(
                    "Unknown interface bound '{}' on type parameter '{}'",
                    bound.interface, type_param.name
                )));
            }
        }
    }

    let type_param_bindings: std::collections::HashMap<String, Type> = workflow
        .type_params
        .iter()
        .map(|param| (param.name.to_string(), Type::Var(TypeVar::fresh())))
        .collect();

    let mut param_bindings = Vec::with_capacity(workflow.params.len());
    for param in &workflow.params {
        let ty = workflow_surface_type_to_type(env, &param.ty, &type_param_bindings)?;
        param_bindings.push((param.name.to_string(), ty));
    }

    let declared_return_ty = workflow
        .declared_return_type
        .as_ref()
        .map(|return_ty| workflow_surface_type_to_type(env, return_ty, &type_param_bindings))
        .transpose()?;

    let mut workflow_env = env.clone();
    for type_param in &workflow.type_params {
        if let Some(Type::Var(var)) = type_param_bindings.get(type_param.name.as_ref()) {
            for bound in &type_param.bounds {
                workflow_env.bind_type_var_interface_bound(*var, bound.interface.as_ref());
            }
        }
    }
    for (name, ty) in &param_bindings {
        workflow_env.bind_variable(name, ty.clone());
    }

    reject_unsupported_mvp_workflow_features(&workflow.body)?;
    validate_interface_calls_in_workflow(&mut workflow_env, &workflow.body)?;

    if let Some(expected_return_ty) = declared_return_ty {
        let actual_return_ty = infer_workflow_return_type(&workflow_env, &workflow.body)?;
        crate::types::unify(&expected_return_ty, &actual_return_ty).map_err(|_| {
            TypeCheckError::TypeError(format!(
                "workflow '{}' declared return type {} but body returns {}",
                workflow.name, expected_return_ty, actual_return_ty
            ))
        })?;
    }

    type_check_workflow(&workflow.body, Some(&param_bindings))
}

pub fn type_check_workflow_def(
    workflow: &ash_parser::surface::WorkflowDef,
) -> Result<TypeCheckResult, TypeCheckError> {
    type_check_workflow_def_in_env(&TypeEnv::with_builtin_types(), workflow)
}

pub fn type_check_program(
    program: &ash_parser::surface::Program,
) -> Result<TypeCheckResult, TypeCheckError> {
    let mut env = TypeEnv::with_builtin_types();

    for definition in &program.definitions {
        if let ash_parser::surface::Definition::Interface(interface) = definition {
            env.register_interface(interface)
                .map_err(|error| TypeCheckError::TypeError(error.to_string()))?;
        }
    }

    for definition in &program.definitions {
        if let ash_parser::surface::Definition::Impl(implementation) = definition {
            env.register_impl(implementation)
                .map_err(|error| TypeCheckError::TypeError(error.to_string()))?;
        }
    }

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
    reject_unsupported_mvp_workflow_features(workflow)?;

    // Step 1: Name resolution
    let mut resolver = NameResolver::new();

    // Inject workflow parameters into the resolver's scope before checking
    if let Some(params) = param_bindings {
        for (name, _ty) in params {
            resolver.bind(name.clone());
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
            pattern: Pattern::Variable("x".into()),
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
                right: Box::new(Expr::Variable("name".into())),
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
                left: Box::new(Expr::Variable("x".into())),
                right: Box::new(Expr::Variable("y".into())),
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
        let _ = Solver::new();
        let _ = EffectContext::new();
        let _ = NameResolver::new();
        let _ = ObligationTracker::new();
        let _ = Substitution::new();
    }
}
