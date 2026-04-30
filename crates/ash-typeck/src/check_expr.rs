//! Expression type checking
//!
//! Provides type checking for expressions, including constructor expressions.

#![allow(clippy::result_large_err)]

use crate::error::ConstructorError;
use crate::exhaustiveness::{Coverage, check_exhaustive};
use crate::type_env::{TypeEnv, TypeInfo, VariantIndex, VariantInfo, WorkflowIntrinsic};
use crate::types::{Substitution, Type, TypeVar, unify};
use ash_core::adt::{VariantPayloadShape, tuple_field_name};
use ash_core::ast::{Expr as CoreExpr, Pattern as CorePattern, TypeBody, TypeDef};
use ash_core::workflow_carrier::{
    ActLowerSummary, ContractPlan, OpenPostcondition, PostconditionTarget, ProcLowerSummary,
    ProjectionEvent, ProjectionEventKind, ProjectionKind, PublicWorkflowSummary, SourceOrigin,
    WorkflowBinder, WorkflowNodeId, WorkflowObligation,
};
use ash_core::workflow_contract::Requirement;
use ash_parser::lower_pattern;
use ash_parser::surface::ConstructorPayload;
use ash_parser::surface::{
    ActStmt, BinaryOp, ComprehensionQualifier, DoStmt, Expr, Literal, MatchArm,
    Pattern as SurfacePattern, UnaryOp,
};
use ash_parser::token::Span;
use ash_parser::workflow_contract_classifier;
use std::collections::{HashMap, HashSet};

/// Result of type checking an expression
#[derive(Debug, Clone, PartialEq)]
pub struct CheckResult {
    /// The inferred type of the expression
    pub ty: Type,
    /// Any substitutions generated during checking
    pub substitution: Substitution,
    /// Any errors encountered
    pub errors: Vec<ConstructorError>,
}

pub type WorkflowForm = ash_core::workflow_carrier::WorkflowForm<CoreExpr>;

/// Typechecked workflow artifact preserved by `do:Workflow` elaboration.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowTypedArtifact {
    pub form: WorkflowForm,
    pub projection_events: Vec<ProjectionEvent>,
    pub contract_plan: ContractPlan<CoreExpr>,
    pub obligations: Vec<WorkflowObligation>,
    pub source_origin: SourceOrigin,
}

/// Result of type-directed generalized do-block elaboration.
#[derive(Debug, Clone, PartialEq)]
pub struct DoElaborationResult {
    /// Lowered core expression produced from resolved dictionary evidence.
    pub expr: CoreExpr,
    /// The checked computation type, e.g. `Act<T>`, `Proc<T>`, or `Workflow<T>`.
    pub ty: Type,
    /// Preserved workflow semantic artifact for `do:Workflow`.
    pub workflow_artifact: Option<WorkflowTypedArtifact>,
}

impl CheckResult {
    /// Create a successful check result
    pub fn success(ty: Type) -> Self {
        Self {
            ty,
            substitution: Substitution::new(),
            errors: Vec::new(),
        }
    }

    /// Create a check result with an error
    pub fn error(err: ConstructorError) -> Self {
        Self {
            ty: Type::Var(TypeVar::fresh()),
            substitution: Substitution::new(),
            errors: vec![err],
        }
    }

    /// Check if the result is successful (no errors)
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Type check an expression
///
/// This function infers the type of an expression and returns the result
/// along with any substitutions and errors.
pub fn check_expr(env: &TypeEnv, expr: &Expr) -> CheckResult {
    match expr {
        Expr::Literal(lit) => check_literal(lit),
        Expr::Variable { name, .. } => {
            if name.as_ref() == "()" {
                return CheckResult::success(Type::Constructor {
                    name: crate::QualifiedName::root("()"),
                    args: Vec::new(),
                    kind: crate::Kind::Type,
                });
            }

            // FIXED: Look up variable in environment instead of creating fresh type var
            match env.lookup_variable(name.as_ref()) {
                Some(ty) => CheckResult::success(ty),
                None => CheckResult::error(ConstructorError::UnboundVariable {
                    name: name.to_string(),
                    span: get_expr_span(expr),
                }),
            }
        }
        Expr::Unary { op, operand, .. } => check_unary(env, *op, operand),
        Expr::Binary {
            op, left, right, ..
        } => check_binary(env, *op, left, right),
        Expr::Constructor {
            name,
            fields,
            payload,
            ..
        } => check_constructor(env, name.as_ref(), fields, payload),
        Expr::IfLet {
            pattern,
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            let matched_result = check_expr(env, expr);
            if !matched_result.is_ok() {
                return matched_result;
            }

            let matched_ty = matched_result.substitution.apply(&matched_result.ty);
            let mut then_env = env.clone();
            if let Ok(bindings) = crate::check_pattern::check_pattern(
                &crate::check_pattern::TypeEnv::new(),
                pattern,
                &matched_ty,
            ) {
                for (name, ty) in bindings {
                    then_env.bind_variable(&name, ty);
                }
            }

            let then_result = check_expr(&then_env, then_branch);
            let else_result = check_expr(env, else_branch);
            merge_branch_results(then_result, else_result)
        }
        Expr::Match {
            scrutinee, arms, ..
        } => check_match(env, scrutinee, arms),
        Expr::Fail { payload, .. } => {
            let payload_result = check_expr(env, payload);
            if !payload_result.is_ok() {
                return payload_result;
            }
            CheckResult {
                ty: Type::Var(TypeVar::fresh()),
                substitution: payload_result.substitution,
                errors: Vec::new(),
            }
        }
        Expr::WithError { body, arms, span } => check_with_error(env, body, arms, *span),
        Expr::FieldAccess { base, field, span } => {
            check_field_access(env, base, field.as_ref(), *span)
        }
        Expr::IndexAccess { base, index, span } => {
            // Index access is not yet fully implemented
            let base_result = check_expr(env, base);
            let index_result = check_expr(env, index);

            let mut errors: Vec<ConstructorError> = Vec::new();
            if !base_result.is_ok() {
                errors.extend(base_result.errors);
            }
            if !index_result.is_ok() {
                errors.extend(index_result.errors);
            }

            if !errors.is_empty() {
                return CheckResult {
                    ty: Type::Var(TypeVar::fresh()),
                    substitution: base_result.substitution.compose(&index_result.substitution),
                    errors,
                };
            }

            CheckResult::error(ConstructorError::UnsupportedExpression {
                kind: "IndexAccess ([]".to_string(),
                span: *span,
            })
        }
        Expr::Call {
            func,
            module,
            args,
            span,
        } => {
            if module.as_deref() == Some("workflow") {
                let qualified_intrinsic = format!("workflow::{func}");
                if let Some(intrinsic) = env.lookup_workflow_intrinsic(&qualified_intrinsic) {
                    let has_workflow_intrinsic_context = env
                        .lookup_variable("__workflow_intrinsic_context")
                        .is_some();
                    match func.as_ref() {
                        "requires" => {
                            if args.len() != 1 {
                                return CheckResult::error(workflow_intrinsic_misuse_error(
                                    &intrinsic,
                                    format!("expects 1 arguments, found {}", args.len()),
                                    *span,
                                ));
                            }
                            if let Err(err) = validate_requirement_expr(env, &args[0]) {
                                return CheckResult::error(workflow_intrinsic_misuse_error(
                                    &intrinsic,
                                    err.to_string(),
                                    *span,
                                ));
                            }
                            if !has_workflow_intrinsic_context {
                                return CheckResult::error(workflow_intrinsic_context_misuse_error(
                                    &intrinsic,
                                    "requires refines Workflow contract context and does not produce a denotable value".to_string(),
                                    *span,
                                ));
                            }
                            return CheckResult::success(intrinsic.result_type().clone());
                        }
                        "ensures" => {
                            if args.len() != 1 {
                                return CheckResult::error(workflow_intrinsic_misuse_error(
                                    &intrinsic,
                                    format!("expects 1 arguments, found {}", args.len()),
                                    *span,
                                ));
                            }
                            if !has_workflow_intrinsic_context
                                && expr_mentions_variable(&args[0], "result")
                            {
                                return CheckResult::error(workflow_intrinsic_misuse_error(
                                    &intrinsic,
                                    "open result postcondition mentions `result` but has no Workflow result boundary".to_string(),
                                    *span,
                                ));
                            }
                            if let Err(err) = validate_postcondition_expr(&args[0]) {
                                return CheckResult::error(workflow_intrinsic_misuse_error(
                                    &intrinsic,
                                    err.to_string(),
                                    *span,
                                ));
                            }
                            if !has_workflow_intrinsic_context {
                                return CheckResult::error(workflow_intrinsic_context_misuse_error(
                                    &intrinsic,
                                    "ensures targets the Workflow result boundary and does not produce a denotable value".to_string(),
                                    *span,
                                ));
                            }
                            return CheckResult::success(intrinsic.result_type().clone());
                        }
                        _ => {}
                    }
                }
            }

            let mut errors: Vec<ConstructorError> = Vec::new();
            let mut substitution = Substitution::new();
            let mut arg_types: Vec<Type> = Vec::with_capacity(args.len());

            for arg in args {
                let arg_result = check_expr(env, arg);
                if !arg_result.is_ok() {
                    errors.extend(arg_result.errors);
                }
                substitution = substitution.compose(&arg_result.substitution);
                arg_types.push(substitution.apply(&arg_result.ty));
            }

            if !errors.is_empty() {
                return CheckResult {
                    ty: Type::Var(TypeVar::fresh()),
                    substitution,
                    errors,
                };
            }

            let qualified_name = module
                .as_ref()
                .map(|module| format!("{}::{}", module, func))
                .unwrap_or_else(|| func.to_string());

            if module.is_none() && func.as_ref() == "invoke" {
                if env.is_capability_implementation_body() {
                    return CheckResult::error(ConstructorError::UnsupportedExpression {
                        kind: "Call (invoke): direct invoke is not allowed in capability implementation bodies; declare capability/resource dependencies instead".to_string(),
                        span: *span,
                    });
                }

                if arg_types.len() != 3 {
                    return CheckResult::error(ConstructorError::UnsupportedExpression {
                        kind: format!(
                            "Call ({qualified_name}): invoke expects 3 arguments, found {}",
                            arg_types.len()
                        ),
                        span: *span,
                    });
                }

                if !matches!(arg_types[0], Type::String) {
                    return CheckResult::error(ConstructorError::UnsupportedExpression {
                        kind: format!(
                            "Call ({qualified_name}): invoke provider must have type String, found {}",
                            arg_types[0]
                        ),
                        span: *span,
                    });
                }

                if !matches!(arg_types[1], Type::String) {
                    return CheckResult::error(ConstructorError::UnsupportedExpression {
                        kind: format!(
                            "Call ({qualified_name}): invoke action must have type String, found {}",
                            arg_types[1]
                        ),
                        span: *span,
                    });
                }

                let value_ty = Type::Constructor {
                    name: crate::QualifiedName::root("Value"),
                    args: vec![],
                    kind: crate::Kind::Type,
                };
                match &arg_types[2] {
                    Type::List(elem_ty) if elem_ty.as_ref() == &value_ty => {}
                    other => {
                        return CheckResult::error(ConstructorError::UnsupportedExpression {
                            kind: format!(
                                "Call ({qualified_name}): invoke args must have type List<Value>, found {other}"
                            ),
                            span: *span,
                        });
                    }
                }

                return CheckResult::success(Type::Constructor {
                    name: crate::QualifiedName::root("Act"),
                    args: vec![value_ty],
                    kind: crate::Kind::Type,
                });
            }

            match env.lookup_call_target(module.as_deref(), func.as_ref()) {
                Some(func_ty) => {
                    let func_ty = substitution.apply(&func_ty);
                    match &func_ty {
                        Type::Fn(_, _) | Type::Fun(_, _, _) => {
                            match func_ty.instantiate_fn_call(&arg_types) {
                                Some(Ok(ret_ty)) => CheckResult {
                                    ty: ret_ty,
                                    substitution,
                                    errors: Vec::new(),
                                },
                                Some(Err(_unify_err)) => {
                                    CheckResult::error(ConstructorError::UnsupportedExpression {
                                        kind: format!(
                                            "Call ({qualified_name}): argument type mismatch"
                                        ),
                                        span: *span,
                                    })
                                }
                                None => {
                                    CheckResult::error(ConstructorError::UnsupportedExpression {
                                        kind: format!(
                                            "Call ({qualified_name}): too many arguments — expected at most {} args, got {}",
                                            match &func_ty {
                                                Type::Fn(params, _) => params.len(),
                                                Type::Fun(params, _, _) => params.len(),
                                                _ => unreachable!("callable types handled above"),
                                            },
                                            args.len()
                                        ),
                                        span: *span,
                                    })
                                }
                            }
                        }
                        _ => CheckResult::error(ConstructorError::UnsupportedExpression {
                            kind: format!(
                                "Call ({qualified_name}): value of type {func_ty} is not callable"
                            ),
                            span: *span,
                        }),
                    }
                }
                None => {
                    match module.as_deref() {
                        Some(module_name) if env.has_capability_symbol(module_name) => {
                            return CheckResult::error(ConstructorError::UnsupportedExpression {
                                kind: format!(
                                    "'{qualified_name}' is a capability, not a function; use capability syntax instead of `module::name()`"
                                ),
                                span: *span,
                            });
                        }
                        Some(module_name) if env.has_interface(module_name) => {
                            // Interface method call: Interface::method(args...)
                            return match env.resolve_interface_method_call(
                                module_name,
                                func.as_ref(),
                                &arg_types,
                            ) {
                                Ok(return_type) => CheckResult {
                                    ty: return_type,
                                    substitution,
                                    errors: Vec::new(),
                                },
                                Err(err) => CheckResult::error(
                                    ConstructorError::InvalidInterfaceMethodCall {
                                        interface: module_name.to_string(),
                                        method: func.to_string(),
                                        reason: err.to_string(),
                                        span: *span,
                                    },
                                ),
                            };
                        }
                        _ => {}
                    }

                    CheckResult::error(ConstructorError::UnsupportedExpression {
                        kind: format!("call to unknown function '{qualified_name}'"),
                        span: *span,
                    })
                }
            }
        }
        Expr::CheckObligation { obligation, span } => {
            CheckResult::error(ConstructorError::UnsupportedExpression {
                kind: format!("CheckObligation ({obligation})"),
                span: *span,
            })
        }
        Expr::Policy(policy_expr) => CheckResult::error(ConstructorError::UnsupportedExpression {
            kind: format!("Policy expression ({policy_expr:?})"),
            span: Span::default(),
        }),
        Expr::If {
            condition,
            then_branch,
            else_branch,
            span,
        } => {
            let cond_result = check_expr(env, condition);
            let then_result = check_expr(env, then_branch);
            let mut errors = cond_result.errors.clone();
            errors.extend(then_result.errors.clone());

            let combined_sub = cond_result.substitution.compose(&then_result.substitution);
            let cond_ty = combined_sub.apply(&cond_result.ty);
            if cond_result.is_ok() && cond_ty != Type::Bool {
                errors.push(ConstructorError::UnsupportedExpression {
                    kind: format!("If expression: condition must be Bool, found {}", cond_ty),
                    span: *span,
                });
            }

            match else_branch {
                Some(else_expr) => {
                    let else_result = check_expr(env, else_expr);
                    let combined_sub = combined_sub.compose(&else_result.substitution);
                    errors.extend(else_result.errors.clone());

                    if !errors.is_empty() {
                        return CheckResult {
                            ty: Type::Var(TypeVar::fresh()),
                            substitution: combined_sub,
                            errors,
                        };
                    }

                    let then_ty = combined_sub.apply(&then_result.ty);
                    let else_ty = combined_sub.apply(&else_result.ty);
                    match unify(&then_ty, &else_ty) {
                        Ok(sub) => CheckResult {
                            ty: sub.apply(&then_ty),
                            substitution: combined_sub.compose(&sub),
                            errors: Vec::new(),
                        },
                        Err(_) => CheckResult::error(ConstructorError::UnsupportedExpression {
                            kind: format!(
                                "If expression: branch types differ ({} vs {})",
                                then_ty, else_ty
                            ),
                            span: *span,
                        }),
                    }
                }
                None => {
                    if !errors.is_empty() {
                        return CheckResult {
                            ty: Type::Null,
                            substitution: combined_sub,
                            errors,
                        };
                    }

                    let then_ty = combined_sub.apply(&then_result.ty);
                    match unify(&then_ty, &Type::Null) {
                        Ok(sub) => CheckResult {
                            ty: Type::Null,
                            substitution: combined_sub.compose(&sub),
                            errors: Vec::new(),
                        },
                        Err(_) => CheckResult::error(ConstructorError::UnsupportedExpression {
                            kind: format!(
                                "If expression without else requires then branch to have type Null, found {}",
                                then_ty
                            ),
                            span: *span,
                        }),
                    }
                }
            }
        }
        Expr::Panic { .. } => {
            // Panic diverges and therefore can type-check at any expected type.
            // Represent it as a fresh type variable rather than forcing Null.
            CheckResult::success(Type::Var(TypeVar::fresh()))
        }
        Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            let mut block_env = env.clone();
            let mut errors: Vec<ConstructorError> = Vec::new();
            let mut substitution = Substitution::new();

            // Process statements (let-bindings)
            for stmt in statements {
                match stmt {
                    ash_parser::surface::BlockStmt::Let { pattern, expr, .. } => {
                        let expr_result = check_expr(&block_env, expr);
                        substitution = substitution.compose(&expr_result.substitution);
                        if !expr_result.is_ok() {
                            errors.extend(expr_result.errors);
                            continue;
                        }
                        let expr_ty = substitution.apply(&expr_result.ty);
                        // Bind pattern variables in the block environment
                        crate::bind_pattern_variables(&mut block_env, pattern, &expr_ty);
                    }
                }
            }

            if !errors.is_empty() {
                return CheckResult {
                    ty: Type::Var(TypeVar::fresh()),
                    substitution,
                    errors,
                };
            }

            // Evaluate tail expression or Null if absent
            match tail_expr {
                Some(tail) => {
                    let tail_result = check_expr(&block_env, tail);
                    let combined_sub = substitution.compose(&tail_result.substitution);
                    if !tail_result.is_ok() {
                        return CheckResult {
                            ty: Type::Var(TypeVar::fresh()),
                            substitution: combined_sub,
                            errors: tail_result.errors,
                        };
                    }
                    CheckResult {
                        ty: combined_sub.apply(&tail_result.ty),
                        substitution: combined_sub,
                        errors: Vec::new(),
                    }
                }
                None => CheckResult {
                    ty: Type::Null,
                    substitution,
                    errors: Vec::new(),
                },
            }
        }

        Expr::FnDef {
            params,
            return_type,
            body,
            span,
        } => {
            // Anonymous function definition: check the body with params in scope.
            //
            // Three-vertex boundary (SPEC-031 §4.8):
            //   - Pure `fn` context  → Type::Fn(params, ret)
            //   - Workflow context   → Type::Fun(params, ret, effect)
            //
            // The workflow effect level is carried on the TypeEnv so it propagates
            // automatically through nested scopes without threading an extra parameter.
            //
            // Parameter and return type annotations (SPEC-031 §5.1):
            //   Written type annotations constrain inference. `fn(x: Int) { ... }` gives
            //   `x` type `Int`, not a fresh type variable. Unknown annotation names fall
            //   back to a fresh variable so user-defined types remain inferrable.
            let mut fn_env = env.clone();
            let mut param_types: Vec<Type> = Vec::new();
            let mut errors: Vec<ConstructorError> = Vec::new();
            for (name, ty_ann) in params {
                let param_ty = match ty_ann.as_deref() {
                    Some(ann) => {
                        match annotation_to_type(ann, env, *span, &format!("parameter `{name}`")) {
                            Ok(ty) => ty,
                            Err(e) => {
                                errors.push(e);
                                Type::Var(TypeVar::fresh()) // fallback for error recovery
                            }
                        }
                    }
                    None => Type::Var(TypeVar::fresh()),
                };
                param_types.push(param_ty.clone());
                fn_env.bind_variable(name.as_ref(), param_ty);
            }
            let body_result = check_expr(&fn_env, body);
            let body_ty = body_result.substitution.apply(&body_result.ty);
            errors.extend(body_result.errors);
            let mut substitution = body_result.substitution;

            let ret_ty = match return_type.as_deref() {
                Some(ann) => match annotation_to_type(ann, env, *span, "return type") {
                    Ok(ann_ty) => match unify(&ann_ty, &body_ty) {
                        Ok(sub) => {
                            substitution = substitution.compose(&sub);
                            ann_ty
                        }
                        Err(_) => {
                            errors.push(ConstructorError::UnsupportedExpression {
                                        kind: format!(
                                            "FnDef: return type annotation `{ann}` conflicts with inferred body type `{body_ty}`"
                                        ),
                                        span: *span,
                                    });
                            body_ty
                        }
                    },
                    Err(e) => {
                        errors.push(e);
                        body_ty
                    }
                },
                None => body_ty,
            };

            let fn_ty = match env.workflow_effect() {
                Some(effect) => Type::Fun(param_types, Box::new(ret_ty), effect),
                None => Type::Fn(param_types, Box::new(ret_ty)),
            };
            CheckResult {
                ty: fn_ty,
                substitution,
                errors,
            }
        }

        Expr::FnApply { func, args, span } => {
            if let Some(result) = check_capability_binding_operation_call(env, func, args, *span) {
                return result;
            }

            let func_result = check_expr(env, func);
            let mut errors = func_result.errors.clone();
            let mut substitution = func_result.substitution.clone();
            let mut arg_types: Vec<Type> = Vec::new();
            for arg in args {
                let arg_result = check_expr(env, arg);
                substitution = substitution.compose(&arg_result.substitution);
                errors.extend(arg_result.errors.clone());
                arg_types.push(substitution.apply(&arg_result.ty));
            }

            if !errors.is_empty() {
                return CheckResult {
                    ty: Type::Var(TypeVar::fresh()),
                    substitution,
                    errors,
                };
            }

            let func_ty = substitution.apply(&func_result.ty);
            match func_ty.instantiate_fn_call(&arg_types) {
                Some(Ok(ret_ty)) => CheckResult {
                    ty: ret_ty,
                    substitution,
                    errors: Vec::new(),
                },
                Some(Err(e)) => CheckResult::error(ConstructorError::UnsupportedExpression {
                    kind: format!("FnApply: type mismatch applying args to {func_ty}: {e}"),
                    span: *span,
                }),
                None => {
                    let kind = if func_ty.is_function_type() {
                        format!(
                            "FnApply: too many arguments — expected at most {} args, got {} for type {func_ty}",
                            func_ty.fn_arity().unwrap_or(0),
                            arg_types.len()
                        )
                    } else {
                        format!(
                            "FnApply: cannot apply {} args to non-function type {func_ty}",
                            arg_types.len()
                        )
                    };
                    CheckResult::error(ConstructorError::UnsupportedExpression {
                        kind,
                        span: *span,
                    })
                }
            }
        }

        Expr::DoBlock {
            target,
            stmts,
            span,
        } => check_do_block(env, target, stmts, *span),

        Expr::ActBlock { stmts, span, .. } => check_legacy_act_block(env, stmts, *span),
        Expr::Comprehension {
            target,
            result,
            qualifiers,
            span,
        } => check_comprehension(env, target.as_ref(), result, qualifiers, *span),
        Expr::List { span, .. } => CheckResult::error(ConstructorError::UnsupportedExpression {
            kind: "contract/list expression".to_string(),
            span: *span,
        }),
    }
}

/// Return compatibility migration diagnostics for legacy `act { ... }` syntax.
///
/// This is a durable diagnostic carrier used until the repository has a
/// warning-emission path. New-form `act { ... }` parses as `DoBlock` and does
/// not produce these legacy diagnostics.
pub fn legacy_act_migration_diagnostics(expr: &Expr) -> Vec<String> {
    let Expr::ActBlock { stmts, .. } = expr else {
        return Vec::new();
    };

    let mut diagnostics = Vec::new();
    for stmt in stmts {
        match stmt {
            ActStmt::Bind { name, .. } => diagnostics.push(format!(
                "legacy act bind `{name} = ...;` is deprecated; use `{name} <- ...;` for Act binds or `let {name} = ...;` for pure bindings"
            )),
            ActStmt::Return { .. } => diagnostics.push(
                "legacy act return `ret ...;` is deprecated; use `return ...` without a trailing semicolon"
                    .to_string(),
            ),
        }
    }
    diagnostics
}

/// Return non-fatal generalized do-notation diagnostics for warning-like cases.
///
/// Error diagnostics still flow through [`check_expr`]. This helper is a durable
/// carrier for teaching-oriented migration warnings until the compiler has a
/// unified warning emission pipeline.
pub fn do_notation_diagnostics(env: &TypeEnv, expr: &Expr) -> Vec<String> {
    let mut diagnostics = legacy_act_migration_diagnostics(expr);
    collect_do_notation_diagnostics(env, expr, &mut diagnostics);
    diagnostics
}

fn collect_do_notation_diagnostics(env: &TypeEnv, expr: &Expr, diagnostics: &mut Vec<String>) {
    match expr {
        Expr::DoBlock { target, stmts, .. } => {
            if let Ok(dictionary) = crate::do_target::resolve_do_target(env, target) {
                let mut block_env = env.clone();
                let mut substitution = Substitution::new();
                for stmt in stmts {
                    match stmt {
                        DoStmt::Let { name, value, .. } => {
                            let value_result = check_expr(&block_env, value);
                            substitution = substitution.compose(&value_result.substitution);
                            let value_ty = diagnostic_expr_type(&block_env, value, &substitution);
                            if let Some(value_ty) = value_ty {
                                if monadic_inner_type(&value_ty, &dictionary.value_constructor)
                                    .is_some()
                                {
                                    diagnostics.push(format!(
                                        "do:{} let `{name}` binds monadic value {value_ty} without sequencing; use `{name} <- ...` to bind the produced value, or keep `let` only when you intentionally want the computation value itself",
                                        target.name.as_ref()
                                    ));
                                }
                                block_env.bind_variable(name.as_ref(), value_ty);
                            }
                            collect_do_notation_diagnostics(&block_env, value, diagnostics);
                        }
                        DoStmt::Bind { value, .. } | DoStmt::Return { value, .. } => {
                            collect_do_notation_diagnostics(&block_env, value, diagnostics);
                        }
                        DoStmt::WorkflowRequires { .. } | DoStmt::WorkflowEnsures { .. } => {
                            // Raw contract statements are classified by workflow elaboration.
                            // Avoid treating symbolic roles or the delayed `result` binder as
                            // ordinary do-notation values during generic diagnostics.
                        }
                    }
                }
            }
        }
        Expr::ActBlock { .. } => {}
        Expr::Comprehension {
            target,
            result,
            qualifiers,
            ..
        } => {
            collect_comprehension_diagnostics(
                env,
                target.as_ref(),
                result,
                qualifiers,
                diagnostics,
            );
        }
        Expr::Call { args, .. } => {
            for arg in args {
                collect_do_notation_diagnostics(env, arg, diagnostics);
            }
        }
        Expr::Binary { left, right, .. }
        | Expr::IndexAccess {
            base: left,
            index: right,
            ..
        } => {
            collect_do_notation_diagnostics(env, left, diagnostics);
            collect_do_notation_diagnostics(env, right, diagnostics);
        }
        Expr::Unary { operand, .. }
        | Expr::Fail {
            payload: operand, ..
        } => {
            collect_do_notation_diagnostics(env, operand, diagnostics);
        }
        Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            for statement in statements {
                match statement {
                    ash_parser::surface::BlockStmt::Let { expr, .. } => {
                        collect_do_notation_diagnostics(env, expr, diagnostics);
                    }
                }
            }
            if let Some(tail_expr) = tail_expr {
                collect_do_notation_diagnostics(env, tail_expr, diagnostics);
            }
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            collect_do_notation_diagnostics(env, condition, diagnostics);
            collect_do_notation_diagnostics(env, then_branch, diagnostics);
            if let Some(else_branch) = else_branch {
                collect_do_notation_diagnostics(env, else_branch, diagnostics);
            }
        }
        Expr::Match {
            scrutinee, arms, ..
        } => {
            collect_do_notation_diagnostics(env, scrutinee, diagnostics);
            for arm in arms {
                collect_do_notation_diagnostics(env, &arm.body, diagnostics);
            }
        }
        Expr::IfLet {
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            collect_do_notation_diagnostics(env, expr, diagnostics);
            collect_do_notation_diagnostics(env, then_branch, diagnostics);
            collect_do_notation_diagnostics(env, else_branch, diagnostics);
        }
        Expr::FieldAccess { base, .. } => collect_do_notation_diagnostics(env, base, diagnostics),
        Expr::FnDef { body, .. } => collect_do_notation_diagnostics(env, body, diagnostics),
        Expr::FnApply { func, args, .. } => {
            collect_do_notation_diagnostics(env, func, diagnostics);
            for arg in args {
                collect_do_notation_diagnostics(env, arg, diagnostics);
            }
        }
        Expr::Constructor {
            fields, payload, ..
        } => {
            for (_, value) in fields {
                collect_do_notation_diagnostics(env, value, diagnostics);
            }
            if let ash_parser::surface::ConstructorPayload::Tuple(items) = payload {
                for item in items {
                    collect_do_notation_diagnostics(env, item, diagnostics);
                }
            }
        }
        Expr::WithError { body, arms, .. } => {
            collect_do_notation_diagnostics(env, body, diagnostics);
            for arm in arms {
                collect_do_notation_diagnostics(env, &arm.body, diagnostics);
            }
        }
        Expr::Policy(policy) => collect_policy_do_notation_diagnostics(env, policy, diagnostics),
        Expr::Literal(_)
        | Expr::Variable { .. }
        | Expr::CheckObligation { .. }
        | Expr::Panic { .. } => {}
        Expr::List { items, .. } => {
            for item in items {
                collect_do_notation_diagnostics(env, item, diagnostics);
            }
        }
    }
}

fn diagnostic_expr_type(env: &TypeEnv, expr: &Expr, substitution: &Substitution) -> Option<Type> {
    if let Some(ty) = diagnostic_dictionary_call_type(env, expr, substitution) {
        return Some(ty);
    }

    let result = check_expr(env, expr);
    result.is_ok().then(|| substitution.apply(&result.ty))
}

fn diagnostic_dictionary_call_type(
    env: &TypeEnv,
    expr: &Expr,
    substitution: &Substitution,
) -> Option<Type> {
    let Expr::Call {
        module: Some(module),
        func,
        args,
        ..
    } = expr
    else {
        return None;
    };

    if args.len() != 1 || func.as_ref() != "unit" {
        return None;
    }

    let constructor = match module.as_ref() {
        "act" => crate::QualifiedName::root("Act"),
        "proc" => crate::QualifiedName::root("Proc"),
        _ => return None,
    };

    let inner = check_expr(env, &args[0]);
    inner.is_ok().then(|| Type::Constructor {
        name: constructor,
        args: vec![substitution.apply(&inner.ty)],
        kind: crate::Kind::Type,
    })
}

fn collect_policy_do_notation_diagnostics(
    env: &TypeEnv,
    policy: &ash_parser::surface::PolicyExpr,
    diagnostics: &mut Vec<String>,
) {
    match policy {
        ash_parser::surface::PolicyExpr::ForAll { items, body, .. }
        | ash_parser::surface::PolicyExpr::Exists { items, body, .. } => {
            collect_do_notation_diagnostics(env, items, diagnostics);
            collect_policy_do_notation_diagnostics(env, body, diagnostics);
        }
        ash_parser::surface::PolicyExpr::MethodCall { receiver, args, .. } => {
            collect_policy_do_notation_diagnostics(env, receiver, diagnostics);
            for arg in args {
                collect_do_notation_diagnostics(env, arg, diagnostics);
            }
        }
        ash_parser::surface::PolicyExpr::Call { args, .. } => {
            for arg in args {
                collect_do_notation_diagnostics(env, arg, diagnostics);
            }
        }
        ash_parser::surface::PolicyExpr::And(items)
        | ash_parser::surface::PolicyExpr::Or(items)
        | ash_parser::surface::PolicyExpr::Sequential(items)
        | ash_parser::surface::PolicyExpr::Concurrent(items) => {
            for item in items {
                collect_policy_do_notation_diagnostics(env, item, diagnostics);
            }
        }
        ash_parser::surface::PolicyExpr::Not(item) => {
            collect_policy_do_notation_diagnostics(env, item, diagnostics);
        }
        ash_parser::surface::PolicyExpr::Implies(left, right) => {
            collect_policy_do_notation_diagnostics(env, left, diagnostics);
            collect_policy_do_notation_diagnostics(env, right, diagnostics);
        }
        ash_parser::surface::PolicyExpr::Var { .. } => {}
    }
}

fn check_legacy_act_block(env: &TypeEnv, stmts: &[ActStmt], span: Span) -> CheckResult {
    let mut block_env = env.clone();
    let mut substitution = Substitution::new();
    let mut errors: Vec<ConstructorError> = Vec::new();

    // Enforce the same structural contract as lower_act_block:
    // non-empty, return must be last.
    if stmts.is_empty() {
        return CheckResult::error(ConstructorError::UnsupportedExpression {
            kind: "empty act block".to_string(),
            span,
        });
    }

    let last = stmts.last().unwrap();
    if !matches!(last, ActStmt::Return { .. }) {
        return CheckResult::error(ConstructorError::UnsupportedExpression {
            kind: "act block must end with a return statement".to_string(),
            span,
        });
    }

    // Verify no Return appears before the last statement
    for (i, stmt) in stmts.iter().enumerate() {
        if matches!(stmt, ActStmt::Return { .. }) && i + 1 < stmts.len() {
            return CheckResult::error(ConstructorError::UnsupportedExpression {
                kind: "return must be the last statement in an act block".to_string(),
                span,
            });
        }
    }

    let mut return_ty = Type::Null;

    for stmt in stmts {
        match stmt {
            ActStmt::Bind { name, value, .. } => {
                let value_result = check_expr(&block_env, value);
                substitution = substitution.compose(&value_result.substitution);
                if !value_result.is_ok() {
                    errors.extend(value_result.errors);
                    continue;
                }

                let value_ty = substitution.apply(&value_result.ty);
                let bound_ty = match &value_ty {
                    Type::Constructor { name, args, .. }
                        if name.name == "Act" && args.len() == 1 =>
                    {
                        args[0].clone()
                    }
                    _ => value_ty,
                };
                block_env.bind_variable(name.as_ref(), bound_ty);
            }
            ActStmt::Return { value, .. } => {
                let value_result = check_expr(&block_env, value);
                substitution = substitution.compose(&value_result.substitution);
                if !value_result.is_ok() {
                    errors.extend(value_result.errors);
                    continue;
                }

                return_ty = substitution.apply(&value_result.ty);
            }
        }
    }

    if !errors.is_empty() {
        return CheckResult {
            ty: Type::Var(TypeVar::fresh()),
            substitution,
            errors,
        };
    }

    CheckResult::success(Type::Constructor {
        name: crate::QualifiedName::root("Act"),
        args: vec![return_ty],
        kind: crate::Kind::Type,
    })
}

/// Type-check and elaborate a generalized do-block into core dictionary calls.
///
/// This is the typed lowering boundary for `Expr::DoBlock`: raw parser lowering
/// rejects `DoBlock`, and callers that need core expressions must come through
/// this function after do-target resolution and statement checking.
pub fn elaborate_typed_do_block(
    env: &TypeEnv,
    expr: &Expr,
) -> Result<DoElaborationResult, Vec<ConstructorError>> {
    let Expr::DoBlock {
        target,
        stmts,
        span,
    } = expr
    else {
        return Err(vec![ConstructorError::UnsupportedExpression {
            kind: "typed do elaboration requires a do block expression".to_string(),
            span: get_expr_span(expr),
        }]);
    };

    elaborate_typed_do_parts(env, target, stmts, *span)
}

/// Type-check and elaborate a bracket comprehension through generalized typed do semantics.
pub fn elaborate_typed_comprehension(
    env: &TypeEnv,
    expr: &Expr,
) -> Result<DoElaborationResult, Vec<ConstructorError>> {
    let Expr::Comprehension {
        target,
        result,
        qualifiers,
        span,
    } = expr
    else {
        return Err(vec![ConstructorError::UnsupportedExpression {
            kind: "typed comprehension elaboration requires a comprehension expression".to_string(),
            span: get_expr_span(expr),
        }]);
    };

    let Some(target) = target.as_ref() else {
        return Err(vec![missing_comprehension_target_error(*span)]);
    };
    let stmts = comprehension_do_stmts(result, qualifiers, *span)?;
    elaborate_typed_do_parts(env, target, &stmts, *span)
}

fn elaborate_typed_do_parts(
    env: &TypeEnv,
    target: &ash_parser::surface::DoTarget,
    stmts: &[DoStmt],
    span: Span,
) -> Result<DoElaborationResult, Vec<ConstructorError>> {
    let check = check_do_block(env, target, stmts, span);
    if !check.is_ok() {
        return Err(check.errors);
    }

    let dictionary = crate::do_target::resolve_do_target(env, target).map_err(|err| vec![err])?;
    let core = elaborate_do_stmts(stmts, &dictionary).map_err(|err| vec![err])?;
    let workflow_artifact = if dictionary.tower_level == crate::do_target::DoTowerLevel::Workflow {
        Some(build_workflow_artifact(env, stmts).map_err(|err| vec![err])?)
    } else {
        None
    };
    Ok(DoElaborationResult {
        expr: core,
        ty: check.ty,
        workflow_artifact,
    })
}

fn collect_comprehension_diagnostics(
    env: &TypeEnv,
    target: Option<&ash_parser::surface::DoTarget>,
    result: &Expr,
    qualifiers: &[ComprehensionQualifier],
    diagnostics: &mut Vec<String>,
) {
    let Some(target) = target else {
        collect_do_notation_diagnostics(env, result, diagnostics);
        for qualifier in qualifiers {
            match qualifier {
                ComprehensionQualifier::Let { value, .. }
                | ComprehensionQualifier::Bind { value, .. }
                | ComprehensionQualifier::DiscardBind { value, .. } => {
                    collect_do_notation_diagnostics(env, value, diagnostics);
                }
            }
        }
        return;
    };

    let Ok(dictionary) = crate::do_target::resolve_do_target(env, target) else {
        collect_do_notation_diagnostics(env, result, diagnostics);
        for qualifier in qualifiers {
            match qualifier {
                ComprehensionQualifier::Let { value, .. }
                | ComprehensionQualifier::Bind { value, .. }
                | ComprehensionQualifier::DiscardBind { value, .. } => {
                    collect_do_notation_diagnostics(env, value, diagnostics);
                }
            }
        }
        return;
    };

    let mut block_env = env.clone();
    let mut substitution = Substitution::new();
    for qualifier in qualifiers {
        match qualifier {
            ComprehensionQualifier::Let { name, value, .. } => {
                let value_result = check_expr(&block_env, value);
                substitution = substitution.compose(&value_result.substitution);
                if let Some(value_ty) = diagnostic_expr_type(&block_env, value, &substitution) {
                    if monadic_inner_type(&value_ty, &dictionary.value_constructor).is_some() {
                        diagnostics.push(format!(
                            "comprehension:{} let `{name}` binds monadic value {value_ty} without sequencing; use `{name} <- ...` to bind the produced value, or keep `let` only when you intentionally want the computation value itself",
                            target.name.as_ref()
                        ));
                    }
                    block_env.bind_variable(name.as_ref(), value_ty);
                }
                collect_do_notation_diagnostics(&block_env, value, diagnostics);
            }
            ComprehensionQualifier::Bind { name, value, .. } => {
                let value_result = check_expr(&block_env, value);
                substitution = substitution.compose(&value_result.substitution);
                if value_result.is_ok() {
                    let value_ty = substitution.apply(&value_result.ty);
                    if let Some(bound_ty) =
                        monadic_inner_type(&value_ty, &dictionary.value_constructor)
                    {
                        block_env.bind_variable(name.as_ref(), bound_ty);
                    }
                }
                collect_do_notation_diagnostics(&block_env, value, diagnostics);
            }
            ComprehensionQualifier::DiscardBind { value, .. } => {
                let value_result = check_expr(&block_env, value);
                substitution = substitution.compose(&value_result.substitution);
                collect_do_notation_diagnostics(&block_env, value, diagnostics);
            }
        }
    }
    collect_do_notation_diagnostics(&block_env, result, diagnostics);
}

fn missing_comprehension_target_error(span: Span) -> ConstructorError {
    ConstructorError::UnsupportedExpression {
        kind: "comprehension MVP requires an explicit target annotation such as `[x | x <- xs]: Act`; target inference is deferred".to_string(),
        span,
    }
}

fn check_comprehension(
    env: &TypeEnv,
    target: Option<&ash_parser::surface::DoTarget>,
    result: &Expr,
    qualifiers: &[ComprehensionQualifier],
    span: Span,
) -> CheckResult {
    let Some(target) = target else {
        return CheckResult::error(missing_comprehension_target_error(span));
    };

    let stmts = match comprehension_do_stmts(result, qualifiers, span) {
        Ok(stmts) => stmts,
        Err(errors) => {
            return CheckResult {
                ty: Type::Var(TypeVar::fresh()),
                substitution: Substitution::new(),
                errors: comprehension_errors(errors),
            };
        }
    };
    let mut result = check_do_block(env, target, &stmts, span);
    if !result.errors.is_empty() {
        result.errors = comprehension_errors(result.errors);
    }
    result
}

fn comprehension_errors(errors: Vec<ConstructorError>) -> Vec<ConstructorError> {
    errors
        .into_iter()
        .map(|error| match error {
            ConstructorError::UnsupportedExpression { kind, span }
                if !kind.starts_with("comprehension") =>
            {
                ConstructorError::UnsupportedExpression {
                    kind: format!("comprehension {kind}"),
                    span,
                }
            }
            other => other,
        })
        .collect()
}

fn comprehension_do_stmts(
    result: &Expr,
    qualifiers: &[ComprehensionQualifier],
    span: Span,
) -> Result<Vec<DoStmt>, Vec<ConstructorError>> {
    if qualifiers.is_empty() {
        return Err(vec![ConstructorError::UnsupportedExpression {
            kind: "comprehension must contain at least one qualifier".to_string(),
            span,
        }]);
    }

    let mut stmts = Vec::with_capacity(qualifiers.len() + 1);
    for qualifier in qualifiers {
        stmts.push(match qualifier {
            ComprehensionQualifier::Let { name, value, span } => DoStmt::Let {
                name: name.clone(),
                value: value.clone(),
                span: *span,
            },
            ComprehensionQualifier::Bind { name, value, span } => DoStmt::Bind {
                name: name.clone(),
                value: value.clone(),
                span: *span,
            },
            ComprehensionQualifier::DiscardBind { value, span } => DoStmt::Bind {
                name: "_".into(),
                value: value.clone(),
                span: *span,
            },
        });
    }
    stmts.push(DoStmt::Return {
        value: Box::new(result.clone()),
        span,
    });
    Ok(stmts)
}

fn elaborate_do_stmts(
    stmts: &[DoStmt],
    dictionary: &crate::do_target::DoDictionary,
) -> Result<CoreExpr, ConstructorError> {
    match stmts {
        [] => Err(ConstructorError::UnsupportedExpression {
            kind: "empty do block".to_string(),
            span: Span::default(),
        }),
        [DoStmt::Return { value, .. }] => Ok(dictionary_call(
            &dictionary.return_op,
            vec![elaborate_workflow_call_expr(value)?],
        )),
        [DoStmt::Let { name, value, .. }, rest @ ..] => Ok(CoreExpr::Let {
            pattern: CorePattern::Variable {
                name: name.to_string(),
                span: ash_core::Span::default(),
            },
            expr: Box::new(elaborate_workflow_call_expr(value)?),
            body: Box::new(elaborate_do_stmts(rest, dictionary)?),
            span: ash_core::Span::default(),
        }),
        [DoStmt::Bind { name, value, .. }, rest @ ..] => {
            let continuation = CoreExpr::FnDef {
                params: vec![(name.to_string(), None)],
                return_type: None,
                body: Box::new(elaborate_do_stmts(rest, dictionary)?),
            };
            Ok(dictionary_call(
                &dictionary.bind_op,
                vec![elaborate_workflow_call_expr(value)?, continuation],
            ))
        }
        [DoStmt::WorkflowRequires { expr, .. }, rest @ ..]
            if dictionary.tower_level == crate::do_target::DoTowerLevel::Workflow =>
        {
            Ok(dictionary_call(
                &dictionary.bind_op,
                vec![
                    CoreExpr::Call {
                        func: "requires".to_string(),
                        module: Some("workflow".to_string()),
                        arguments: vec![elaborate_surface_expr(expr)?],
                    },
                    CoreExpr::FnDef {
                        params: vec![("_".to_string(), None)],
                        return_type: None,
                        body: Box::new(elaborate_do_stmts(rest, dictionary)?),
                    },
                ],
            ))
        }
        [DoStmt::WorkflowEnsures { expr, .. }, rest @ ..]
            if dictionary.tower_level == crate::do_target::DoTowerLevel::Workflow =>
        {
            Ok(dictionary_call(
                &dictionary.bind_op,
                vec![
                    CoreExpr::Call {
                        func: "ensures".to_string(),
                        module: Some("workflow".to_string()),
                        arguments: vec![elaborate_surface_expr(expr)?],
                    },
                    CoreExpr::FnDef {
                        params: vec![("_".to_string(), None)],
                        return_type: None,
                        body: Box::new(elaborate_do_stmts(rest, dictionary)?),
                    },
                ],
            ))
        }
        [stmt, ..] => Err(ConstructorError::UnsupportedExpression {
            kind: "invalid do statement sequence (return must be last)".to_string(),
            span: do_stmt_span(stmt),
        }),
    }
}

fn build_workflow_artifact(
    env: &TypeEnv,
    stmts: &[DoStmt],
) -> Result<WorkflowTypedArtifact, ConstructorError> {
    let mut builder = WorkflowArtifactBuilder::new(env);
    let form = builder.form_from_stmts(stmts)?;
    let contract_plan = builder.contract_plan(&form);
    Ok(WorkflowTypedArtifact {
        form,
        projection_events: builder.events,
        contract_plan,
        obligations: builder.obligations,
        source_origin: WorkflowArtifactBuilder::origin(),
    })
}

struct WorkflowArtifactBuilder {
    env: TypeEnv,
    next_node: u64,
    events: Vec<ProjectionEvent>,
    obligations: Vec<WorkflowObligation>,
    local_artifacts: HashMap<String, WorkflowForm>,
}

impl WorkflowArtifactBuilder {
    fn new(env: &TypeEnv) -> Self {
        Self {
            env: env.clone(),
            next_node: 0,
            events: Vec::new(),
            obligations: Vec::new(),
            local_artifacts: HashMap::new(),
        }
    }

    fn node(&mut self) -> WorkflowNodeId {
        let node = WorkflowNodeId(self.next_node);
        self.next_node += 1;
        node
    }

    fn origin() -> SourceOrigin {
        SourceOrigin::Synthetic {
            parent_span: None,
            reason: "do:Workflow".to_string(),
        }
    }

    fn push_event(&mut self, node: WorkflowNodeId, kind: ProjectionEventKind) {
        self.events.push(ProjectionEvent {
            node,
            projection: ProjectionKind::Contract,
            origin: Self::origin(),
            kind,
        });
    }

    fn push_imported_summary(&mut self, summary: &PublicWorkflowSummary) -> WorkflowForm {
        self.events
            .extend(summary.projection_events.iter().cloned());
        self.obligations
            .extend(summary.coverage.obligations.iter().cloned());
        let node = self.node();
        WorkflowForm::ImportedSummary {
            node,
            summary: summary.clone(),
        }
    }

    fn form_from_stmts(&mut self, stmts: &[DoStmt]) -> Result<WorkflowForm, ConstructorError> {
        match stmts {
            [DoStmt::Return { value, .. }] => {
                let node = self.node();
                let value = elaborate_workflow_call_expr(value)?;
                self.push_event(
                    node,
                    ProjectionEventKind::Unit {
                        value_erased: false,
                    },
                );
                Ok(WorkflowForm::Unit { node, value })
            }
            [
                DoStmt::Let {
                    name,
                    value,
                    span: _,
                },
                rest @ ..,
            ] => {
                if let Some(source) = self.try_workflow_source_form(value)? {
                    self.local_artifacts.insert(name.to_string(), source);
                } else {
                    self.local_artifacts.remove(name.as_ref());
                }
                self.form_from_stmts(rest)
            }
            [
                DoStmt::Bind {
                    name,
                    value,
                    span: _,
                },
                rest @ ..,
            ] => {
                let node = self.node();
                let binder = if name.as_ref() == "_" {
                    WorkflowBinder::Ignored
                } else {
                    WorkflowBinder::Named(name.to_string())
                };
                let source = self.workflow_source_form(value)?;
                let next = self.form_from_stmts(rest)?;
                self.push_event(
                    node,
                    ProjectionEventKind::Bind {
                        binder: binder.clone(),
                    },
                );
                Ok(WorkflowForm::Bind {
                    node,
                    source: Box::new(source),
                    binder,
                    next: Box::new(next),
                })
            }
            [DoStmt::WorkflowRequires { expr, .. }, rest @ ..] => {
                let node = self.node();
                let requirement = classify_requirement(expr)?;
                self.push_event(
                    node,
                    ProjectionEventKind::Requires {
                        requirement: requirement.clone(),
                    },
                );
                self.obligations
                    .push(WorkflowObligation::RequirementMustHold {
                        node,
                        requirement: requirement.clone(),
                    });
                let source = WorkflowForm::Requires { node, requirement };
                let next = self.form_from_stmts(rest)?;
                Ok(WorkflowForm::Bind {
                    node: self.node(),
                    source: Box::new(source),
                    binder: WorkflowBinder::Ignored,
                    next: Box::new(next),
                })
            }
            [DoStmt::WorkflowEnsures { expr, .. }, rest @ ..] => {
                let node = self.node();
                let postcondition = classify_postcondition(expr)?;
                self.push_event(
                    node,
                    ProjectionEventKind::Ensures {
                        postcondition: postcondition.clone(),
                    },
                );
                self.obligations
                    .push(WorkflowObligation::OpenPostconditionTarget {
                        node,
                        postcondition: postcondition.clone(),
                        target_type: "WorkflowResult".to_string(),
                    });
                let source = WorkflowForm::Ensures {
                    node,
                    postcondition,
                };
                let next = self.form_from_stmts(rest)?;
                Ok(WorkflowForm::Bind {
                    node: self.node(),
                    source: Box::new(source),
                    binder: WorkflowBinder::Ignored,
                    next: Box::new(next),
                })
            }
            [stmt, ..] => Err(ConstructorError::UnsupportedExpression {
                kind: "invalid workflow do statement sequence".to_string(),
                span: do_stmt_span(stmt),
            }),
            [] => Err(ConstructorError::UnsupportedExpression {
                kind: "empty workflow do block".to_string(),
                span: Span::default(),
            }),
        }
    }

    fn workflow_source_form(&mut self, value: &Expr) -> Result<WorkflowForm, ConstructorError> {
        self.try_workflow_source_form(value)?.ok_or_else(|| ConstructorError::UnsupportedExpression {
            kind: "do:Workflow bind RHS has no recoverable WorkflowTypedArtifact or public workflow summary".to_string(),
            span: get_expr_span(value),
        })
    }

    #[allow(clippy::collapsible_if)]
    fn try_workflow_source_form(
        &mut self,
        value: &Expr,
    ) -> Result<Option<WorkflowForm>, ConstructorError> {
        if let Expr::Variable { name, .. } = value {
            if let Some(form) = self.local_artifacts.get(name.as_ref()).cloned() {
                return Ok(Some(form));
            }
            if let Some(summary) = self.env.lookup_public_workflow_summary(name.as_ref()) {
                return Ok(Some(self.push_imported_summary(&summary)));
            }
            return Ok(None);
        }

        if let Expr::DoBlock { target, stmts, .. } = value {
            if target.name.as_ref() == "Workflow" {
                return self.form_from_stmts(stmts).map(Some);
            }
        }

        if let Expr::Call {
            module, func, args, ..
        } = value
        {
            if module
                .as_ref()
                .is_none_or(|module| module.as_ref() != "workflow")
            {
                let summary_name = module
                    .as_ref()
                    .map(|module| format!("{}::{}", module, func))
                    .unwrap_or_else(|| func.to_string());
                if let Some(summary) = self.env.lookup_public_workflow_summary(&summary_name) {
                    return Ok(Some(self.push_imported_summary(&summary)));
                }
                return Ok(None);
            }

            match func.as_ref() {
                "unit" if args.len() == 1 => {
                    let node = self.node();
                    let value = elaborate_surface_expr(&args[0])?;
                    self.push_event(
                        node,
                        ProjectionEventKind::Unit {
                            value_erased: false,
                        },
                    );
                    return Ok(Some(WorkflowForm::Unit { node, value }));
                }
                "bind" if args.len() == 2 => {
                    let node = self.node();
                    let source = self.workflow_source_form(&args[0])?;
                    let Expr::FnDef { params, body, .. } = &args[1] else {
                        return Ok(None);
                    };
                    let binder = params
                        .first()
                        .map(|(name, _)| {
                            if name.as_ref() == "_" {
                                WorkflowBinder::Ignored
                            } else {
                                WorkflowBinder::Named(name.to_string())
                            }
                        })
                        .unwrap_or(WorkflowBinder::Ignored);
                    let next = self.workflow_source_form(body)?;
                    self.push_event(
                        node,
                        ProjectionEventKind::Bind {
                            binder: binder.clone(),
                        },
                    );
                    return Ok(Some(WorkflowForm::Bind {
                        node,
                        source: Box::new(source),
                        binder,
                        next: Box::new(next),
                    }));
                }
                "then" if args.len() == 2 => {
                    let node = self.node();
                    let source = self.workflow_source_form(&args[0])?;
                    let next = self.workflow_source_form(&args[1])?;
                    self.push_event(
                        node,
                        ProjectionEventKind::Bind {
                            binder: WorkflowBinder::Ignored,
                        },
                    );
                    return Ok(Some(WorkflowForm::Bind {
                        node,
                        source: Box::new(source),
                        binder: WorkflowBinder::Ignored,
                        next: Box::new(next),
                    }));
                }
                "requires" if args.len() == 1 => {
                    let node = self.node();
                    let requirement = classify_requirement(&args[0])?;
                    self.push_event(
                        node,
                        ProjectionEventKind::Requires {
                            requirement: requirement.clone(),
                        },
                    );
                    self.obligations
                        .push(WorkflowObligation::RequirementMustHold {
                            node,
                            requirement: requirement.clone(),
                        });
                    return Ok(Some(WorkflowForm::Requires { node, requirement }));
                }
                "ensures" if args.len() == 1 => {
                    let node = self.node();
                    let postcondition = classify_postcondition(&args[0])?;
                    self.push_event(
                        node,
                        ProjectionEventKind::Ensures {
                            postcondition: postcondition.clone(),
                        },
                    );
                    self.obligations
                        .push(WorkflowObligation::OpenPostconditionTarget {
                            node,
                            postcondition: postcondition.clone(),
                            target_type: "WorkflowResult".to_string(),
                        });
                    return Ok(Some(WorkflowForm::Ensures {
                        node,
                        postcondition,
                    }));
                }
                "from_proc" if args.len() == 1 => {
                    let node = self.node();
                    let summary = ProcLowerSummary {
                        coverage_obligation_nodes: vec![node],
                        contract_summary: None,
                        ..Default::default()
                    };
                    self.push_event(
                        node,
                        ProjectionEventKind::FromProc {
                            summary: summary.clone(),
                        },
                    );
                    self.obligations.push(WorkflowObligation::LowerProcCovered {
                        node,
                        summary: summary.contract_summary.clone().unwrap_or_default(),
                    });
                    return Ok(Some(WorkflowForm::FromProc { node, summary }));
                }
                "from_act" if args.len() == 1 => {
                    let node = self.node();
                    let summary = ActLowerSummary {
                        coverage_obligation_nodes: vec![node],
                        contract_summary: None,
                    };
                    self.push_event(
                        node,
                        ProjectionEventKind::FromAct {
                            summary: summary.clone(),
                        },
                    );
                    self.obligations.push(WorkflowObligation::LowerActCovered {
                        node,
                        summary: summary.contract_summary.clone().unwrap_or_default(),
                    });
                    return Ok(Some(WorkflowForm::FromAct { node, summary }));
                }
                _ => return Ok(None),
            }
        }

        Ok(None)
    }

    fn contract_plan(&self, form: &WorkflowForm) -> ContractPlan<CoreExpr> {
        match form {
            WorkflowForm::Unit { .. } => ContractPlan::EmptyContract {
                result_marker: None,
            },
            WorkflowForm::Bind {
                node,
                source,
                binder,
                next,
            } => ContractPlan::BindContract {
                node: *node,
                first: Box::new(self.contract_plan(source)),
                binder: binder.clone(),
                second: Box::new(self.contract_plan(next)),
            },
            WorkflowForm::Requires { node, requirement } => ContractPlan::RequirementContract {
                node: *node,
                requirement: requirement.clone(),
            },
            WorkflowForm::Ensures {
                node,
                postcondition,
            } => ContractPlan::EnsuresContract {
                node: *node,
                postcondition: postcondition.clone(),
                target: PostconditionTarget::WorkflowResult,
            },
            WorkflowForm::FromProc { node, summary } => ContractPlan::LowerProcContract {
                node: *node,
                summary: summary.contract_summary.clone().unwrap_or_default(),
            },
            WorkflowForm::FromAct { node, summary } => ContractPlan::LowerActContract {
                node: *node,
                summary: summary.contract_summary.clone().unwrap_or_default(),
            },
            WorkflowForm::ImportedSummary { .. } => ContractPlan::EmptyContract {
                result_marker: None,
            },
            _ => ContractPlan::EmptyContract {
                result_marker: None,
            },
        }
    }
}

fn classify_requirement(expr: &Expr) -> Result<Requirement, ConstructorError> {
    workflow_contract_classifier::classify_requirement(expr).map_err(|err| {
        ConstructorError::UnsupportedExpression {
            kind: format!(
                "unsupported workflow requires contract expression: {}",
                err.requirement_message()
            ),
            span: get_expr_span(expr),
        }
    })
}

fn classify_postcondition(expr: &Expr) -> Result<OpenPostcondition, ConstructorError> {
    workflow_contract_classifier::classify_postcondition(expr)
        .map(|predicate| OpenPostcondition { predicate })
        .map_err(|err| ConstructorError::UnsupportedExpression {
            kind: format!(
                "unsupported workflow ensures contract expression: {}",
                err.postcondition_message()
            ),
            span: get_expr_span(expr),
        })
}

fn workflow_intrinsic_misuse_error(
    intrinsic: &WorkflowIntrinsic,
    reason: String,
    span: Span,
) -> ConstructorError {
    ConstructorError::UnsupportedExpression {
        kind: format!(
            "{} misuse: {} parameter class is non-denotable and contract-only; use inside do:Workflow contract syntax. {reason}",
            intrinsic.qualified_name,
            intrinsic.parameter_class().as_str(),
        ),
        span,
    }
}

fn workflow_intrinsic_context_misuse_error(
    intrinsic: &WorkflowIntrinsic,
    reason: String,
    span: Span,
) -> ConstructorError {
    ConstructorError::UnsupportedExpression {
        kind: format!(
            "{} misuse: {} parameter class is non-denotable and contract-only outside do:Workflow; {reason}",
            intrinsic.qualified_name,
            intrinsic.parameter_class().as_str(),
        ),
        span,
    }
}

#[allow(clippy::collapsible_if)]
fn validate_requirement_expr(env: &TypeEnv, expr: &Expr) -> Result<(), ConstructorError> {
    let requirement = classify_requirement(expr)?;
    if let Requirement::Arithmetic { var, .. } = requirement {
        if env.lookup_variable(&var).is_none() {
            return Err(ConstructorError::UnboundVariable {
                name: var,
                span: get_expr_span(expr),
            });
        }
    }
    Ok(())
}

fn validate_postcondition_expr(expr: &Expr) -> Result<(), ConstructorError> {
    let _ = classify_postcondition(expr)?;
    Ok(())
}

fn dictionary_call(op: &crate::do_target::DoDictionaryOp, arguments: Vec<CoreExpr>) -> CoreExpr {
    match op {
        crate::do_target::DoDictionaryOp::HiddenActReturn => CoreExpr::Call {
            func: "unit".to_string(),
            module: None,
            arguments,
        },
        crate::do_target::DoDictionaryOp::HiddenActBind => CoreExpr::Call {
            func: "bind".to_string(),
            module: None,
            arguments,
        },
        crate::do_target::DoDictionaryOp::Ordinary(name) => CoreExpr::Call {
            func: name.name.clone(),
            module: (!name.module.is_empty()).then(|| name.module.join("::")),
            arguments,
        },
    }
}

fn elaborate_workflow_call_expr(expr: &Expr) -> Result<CoreExpr, ConstructorError> {
    if let Expr::Call {
        module: Some(module),
        func,
        args,
        ..
    } = expr
    {
        if module.as_ref() != "workflow" {
            return elaborate_surface_expr(expr);
        }
        let arguments = match func.as_ref() {
            "requires" | "ensures" => Vec::new(),
            "from_proc" | "from_act" => args
                .iter()
                .map(elaborate_lift_argument_expr)
                .collect::<Result<Vec<_>, _>>()?,
            _ => args
                .iter()
                .map(elaborate_surface_expr)
                .collect::<Result<Vec<_>, _>>()?,
        };
        return Ok(CoreExpr::Call {
            func: func.to_string(),
            module: Some("workflow".to_string()),
            arguments,
        });
    }
    elaborate_surface_expr(expr)
}

fn elaborate_lift_argument_expr(expr: &Expr) -> Result<CoreExpr, ConstructorError> {
    if let Expr::DoBlock { target, stmts, .. } = expr {
        let dictionary = match target.name.as_ref() {
            "Act" => crate::do_target::DoDictionary {
                target: crate::QualifiedName::root("Act"),
                value_constructor: crate::QualifiedName::root("Act"),
                return_op: crate::do_target::DoDictionaryOp::HiddenActReturn,
                bind_op: crate::do_target::DoDictionaryOp::HiddenActBind,
                tower_level: crate::do_target::DoTowerLevel::Effectful,
            },
            "Proc" => crate::do_target::DoDictionary {
                target: crate::QualifiedName::root("Proc"),
                value_constructor: crate::QualifiedName::root("Proc"),
                return_op: crate::do_target::DoDictionaryOp::Ordinary(
                    crate::QualifiedName::qualified(vec!["proc".to_string()], "unit"),
                ),
                bind_op: crate::do_target::DoDictionaryOp::Ordinary(
                    crate::QualifiedName::qualified(vec!["proc".to_string()], "bind"),
                ),
                tower_level: crate::do_target::DoTowerLevel::Proc,
            },
            _ => return elaborate_surface_expr(expr),
        };
        return elaborate_do_stmts(stmts, &dictionary);
    }
    elaborate_surface_expr(expr)
}

fn elaborate_surface_expr(expr: &Expr) -> Result<CoreExpr, ConstructorError> {
    ash_parser::lower_expr(expr).map_err(|err| ConstructorError::UnsupportedExpression {
        kind: format!("failed to lower do-block subexpression after typed elaboration: {err}"),
        span: get_expr_span(expr),
    })
}

fn check_do_block(
    env: &TypeEnv,
    target: &ash_parser::surface::DoTarget,
    stmts: &[DoStmt],
    span: Span,
) -> CheckResult {
    let dictionary = match crate::do_target::resolve_do_target(env, target) {
        Ok(dictionary) => dictionary,
        Err(err) => return CheckResult::error(err),
    };

    if stmts.is_empty() {
        return CheckResult::error(ConstructorError::UnsupportedExpression {
            kind: "empty do block".to_string(),
            span,
        });
    }

    for (index, stmt) in stmts.iter().enumerate() {
        if matches!(stmt, DoStmt::Return { .. }) && index + 1 < stmts.len() {
            return CheckResult::error(ConstructorError::UnsupportedExpression {
                kind: "return must be the last statement in a do block".to_string(),
                span: do_stmt_span(stmt),
            });
        }
    }

    if !matches!(stmts.last(), Some(DoStmt::Return { .. })) {
        return CheckResult::error(ConstructorError::UnsupportedExpression {
            kind: "do block must end with a return statement".to_string(),
            span,
        });
    }

    let mut block_env = env.clone();
    if dictionary.tower_level == crate::do_target::DoTowerLevel::Workflow {
        block_env.bind_variable("__workflow_intrinsic_context", Type::Null);
    }
    let mut substitution = Substitution::new();
    let mut errors: Vec<ConstructorError> = Vec::new();
    let mut return_ty = Type::Null;
    let mut live_workflow_bindings: HashSet<String> = HashSet::new();

    for stmt in stmts {
        match stmt {
            DoStmt::Let {
                name,
                value,
                span: let_span,
            } => {
                let value_result = check_expr(&block_env, value);
                substitution = substitution.compose(&value_result.substitution);
                if !value_result.is_ok() {
                    errors.extend(value_result.errors);
                    continue;
                }
                let value_ty = substitution.apply(&value_result.ty);
                block_env.bind_variable(name.as_ref(), value_ty.clone());
                if dictionary.tower_level == crate::do_target::DoTowerLevel::Workflow
                    && monadic_inner_type(&value_ty, &dictionary.value_constructor).is_some()
                {
                    if workflow_expr_has_live_artifact(&block_env, value, &live_workflow_bindings) {
                        live_workflow_bindings.insert(name.to_string());
                    } else {
                        live_workflow_bindings.remove(name.as_ref());
                        if workflow_expr_requires_live_artifact(value) {
                            errors.push(workflow_opaque_artifact_error(*let_span, "let RHS"));
                        }
                    }
                } else if dictionary.tower_level == crate::do_target::DoTowerLevel::Workflow {
                    live_workflow_bindings.remove(name.as_ref());
                }
            }
            DoStmt::Bind { name, value, span } => {
                let value_result = check_expr(&block_env, value);
                substitution = substitution.compose(&value_result.substitution);
                if !value_result.is_ok() {
                    errors.extend(value_result.errors);
                    continue;
                }

                let value_ty = substitution.apply(&value_result.ty);
                match monadic_inner_type(&value_ty, &dictionary.value_constructor) {
                    Some(bound_ty) => {
                        if dictionary.tower_level == crate::do_target::DoTowerLevel::Workflow
                            && !workflow_expr_has_live_artifact(
                                &block_env,
                                value,
                                &live_workflow_bindings,
                            )
                        {
                            errors.push(workflow_opaque_artifact_error(*span, "bind RHS for <-"));
                        } else {
                            block_env.bind_variable(name.as_ref(), bound_ty);
                        }
                    }
                    None => errors.push(do_bind_type_error(
                        target.name.as_ref(),
                        &dictionary.value_constructor,
                        &value_ty,
                        *span,
                    )),
                }
            }
            DoStmt::Return { value, .. } => {
                let value_result = check_expr(&block_env, value);
                substitution = substitution.compose(&value_result.substitution);
                if !value_result.is_ok() {
                    errors.extend(value_result.errors);
                    continue;
                }
                return_ty = substitution.apply(&value_result.ty);
            }
            DoStmt::WorkflowRequires { expr, span } => {
                if dictionary.tower_level != crate::do_target::DoTowerLevel::Workflow {
                    errors.push(ConstructorError::UnsupportedExpression {
                        kind: "workflow contract statement requires do:Workflow elaboration"
                            .to_string(),
                        span: *span,
                    });
                } else if let Err(err) = validate_requirement_expr(&block_env, expr) {
                    errors.push(err);
                }
            }
            DoStmt::WorkflowEnsures { expr, span } => {
                if dictionary.tower_level != crate::do_target::DoTowerLevel::Workflow {
                    errors.push(ConstructorError::UnsupportedExpression {
                        kind: "workflow contract statement requires do:Workflow elaboration"
                            .to_string(),
                        span: *span,
                    });
                } else if let Err(err) = validate_postcondition_expr(expr) {
                    errors.push(err);
                }
            }
        }
    }

    if !errors.is_empty() {
        return CheckResult {
            ty: Type::Var(TypeVar::fresh()),
            substitution,
            errors,
        };
    }

    CheckResult {
        ty: Type::Constructor {
            name: dictionary.value_constructor,
            args: vec![return_ty],
            kind: crate::Kind::Type,
        },
        substitution,
        errors: Vec::new(),
    }
}

fn do_stmt_span(stmt: &DoStmt) -> Span {
    match stmt {
        DoStmt::Let { span, .. }
        | DoStmt::Bind { span, .. }
        | DoStmt::Return { span, .. }
        | DoStmt::WorkflowRequires { span, .. }
        | DoStmt::WorkflowEnsures { span, .. } => *span,
    }
}

fn monadic_inner_type(ty: &Type, constructor: &crate::QualifiedName) -> Option<Type> {
    match ty {
        Type::Constructor { name, args, .. } if name == constructor && args.len() == 1 => {
            Some(args[0].clone())
        }
        _ => None,
    }
}

fn expr_mentions_variable(expr: &Expr, target: &str) -> bool {
    match expr {
        Expr::Variable { name, .. } => name.as_ref() == target,
        Expr::Binary { left, right, .. } => {
            expr_mentions_variable(left, target) || expr_mentions_variable(right, target)
        }
        Expr::Call { args, .. } => args.iter().any(|arg| expr_mentions_variable(arg, target)),
        Expr::FieldAccess { base, .. } => expr_mentions_variable(base, target),
        Expr::IndexAccess { base, index, .. } => {
            expr_mentions_variable(base, target) || expr_mentions_variable(index, target)
        }
        Expr::FnDef { body, .. } => expr_mentions_variable(body, target),
        Expr::DoBlock { stmts, .. } => stmts.iter().any(|stmt| match stmt {
            DoStmt::Let { value, .. }
            | DoStmt::Bind { value, .. }
            | DoStmt::Return { value, .. } => expr_mentions_variable(value, target),
            DoStmt::WorkflowRequires { expr, .. } | DoStmt::WorkflowEnsures { expr, .. } => {
                expr_mentions_variable(expr, target)
            }
        }),
        _ => false,
    }
}

fn workflow_expr_has_live_artifact(
    env: &TypeEnv,
    expr: &Expr,
    live_bindings: &HashSet<String>,
) -> bool {
    match expr {
        Expr::DoBlock { target, .. } => target.name.as_ref() == "Workflow",
        Expr::Call {
            module, func, args, ..
        } => {
            if module
                .as_ref()
                .is_none_or(|module| module.as_ref() != "workflow")
            {
                let summary_name = module
                    .as_ref()
                    .map(|module| format!("{}::{}", module, func))
                    .unwrap_or_else(|| func.to_string());
                return env.lookup_public_workflow_summary(&summary_name).is_some();
            }
            match func.as_ref() {
                "unit" | "requires" | "ensures" => true,
                "from_proc" | "from_act" => true,
                "bind" if args.len() == 2 => {
                    workflow_expr_has_live_artifact(env, &args[0], live_bindings)
                        && matches!(&args[1], Expr::FnDef { body, .. } if workflow_expr_has_live_artifact(env, body, live_bindings))
                }
                "then" if args.len() == 2 => {
                    workflow_expr_has_live_artifact(env, &args[0], live_bindings)
                        && workflow_expr_has_live_artifact(env, &args[1], live_bindings)
                }
                _ => false,
            }
        }
        Expr::Variable { name, .. } => {
            live_bindings.contains(name.as_ref())
                || env.lookup_public_workflow_summary(name.as_ref()).is_some()
        }
        _ => false,
    }
}

fn workflow_expr_requires_live_artifact(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call {
            module: Some(module),
            func,
            ..
        } if module.as_ref() == "workflow" && matches!(func.as_ref(), "bind" | "then")
    )
}

fn workflow_opaque_artifact_error(span: Span, context: &str) -> ConstructorError {
    ConstructorError::UnsupportedExpression {
        kind: format!(
            "do:Workflow {context} must carry a live WorkflowTypedArtifact or public workflow summary; opaque Workflow<T> values cannot be sequenced without preserving WorkflowForm metadata"
        ),
        span,
    }
}

fn do_bind_type_error(
    target_name: &str,
    expected_constructor: &crate::QualifiedName,
    actual_ty: &Type,
    span: Span,
) -> ConstructorError {
    let hint = match actual_ty {
        Type::Constructor { name, args, .. } if args.len() == 1 && name != expected_constructor => {
            if target_name == "Proc" && name.name == "Act" {
                " use proc::from_act for an explicit Act-to-Proc lift."
            } else if target_name == "Workflow" && name.name == "Proc" {
                " use workflow::from_proc for an explicit Proc-to-Workflow lift."
            } else if target_name == "Workflow" && name.name == "Act" {
                " use workflow::from_act for an explicit Act-to-Workflow lift."
            } else {
                " use an explicit lift for cross-constructor sequencing."
            }
        }
        _ => " pure expressions cannot be used with <-; use let for ordinary bindings.",
    };

    ConstructorError::UnsupportedExpression {
        kind: format!(
            "do:{target_name} bind RHS for <- must have type {expected_constructor}<T>, found {actual_ty};{hint}"
        ),
        span,
    }
}

fn check_capability_binding_operation_call(
    env: &TypeEnv,
    func: &Expr,
    args: &[Expr],
    span: Span,
) -> Option<CheckResult> {
    let Expr::FieldAccess { base, field, .. } = func else {
        return None;
    };
    let Expr::Variable { name, .. } = base.as_ref() else {
        return None;
    };
    let binding_name = name.as_ref();
    let Some(binding) = env.lookup_capability_binding(binding_name) else {
        if env.lookup_variable(binding_name).is_some() {
            return None;
        }
        return Some(CheckResult::error(
            ConstructorError::UnsupportedExpression {
                kind: format!(
                    "unadmitted capability binding '{binding_name}' used for operation '{}'; declare it in the workflow uses header",
                    field
                ),
                span,
            },
        ));
    };
    let Some(operation) = env.lookup_capability_operation(&binding.interface, field.as_ref())
    else {
        return Some(CheckResult::error(
            ConstructorError::UnsupportedExpression {
                kind: format!(
                    "capability binding '{binding_name}' for interface '{}' has no operation '{}'",
                    binding.interface, field
                ),
                span,
            },
        ));
    };

    if operation.params.len() != args.len() {
        return Some(CheckResult::error(
            ConstructorError::UnsupportedExpression {
                kind: format!(
                    "capability binding '{binding_name}' operation '{}' arity mismatch: expected {}, found {}",
                    field,
                    operation.params.len(),
                    args.len()
                ),
                span,
            },
        ));
    }

    let mut substitution = Substitution::new();
    let mut errors = Vec::new();
    for (idx, (arg, expected_ty)) in args.iter().zip(operation.params.iter()).enumerate() {
        let arg_result = check_expr(env, arg);
        substitution = substitution.compose(&arg_result.substitution);
        errors.extend(arg_result.errors);
        if !errors.is_empty() {
            continue;
        }
        let actual_ty = substitution.apply(&arg_result.ty);
        let expected_ty = substitution.apply(expected_ty);
        match unify(&expected_ty, &actual_ty) {
            Ok(sub) => substitution = substitution.compose(&sub),
            Err(_) => errors.push(ConstructorError::UnsupportedExpression {
                kind: format!(
                    "capability binding '{binding_name}' operation '{}' argument {} expected {}, found {}",
                    field,
                    idx + 1,
                    expected_ty,
                    actual_ty
                ),
                span: get_expr_span(arg),
            }),
        }
    }

    if !errors.is_empty() {
        return Some(CheckResult {
            ty: Type::Var(TypeVar::fresh()),
            substitution,
            errors,
        });
    }

    Some(CheckResult {
        ty: substitution.apply(&operation.return_type),
        substitution,
        errors: Vec::new(),
    })
}

/// Get the span from an expression
fn get_expr_span(expr: &Expr) -> Span {
    match expr {
        Expr::Literal(_) => Span::default(),
        Expr::Variable { .. } => Span::default(),
        Expr::FieldAccess { span, .. } => *span,
        Expr::IndexAccess { span, .. } => *span,
        Expr::Unary { span, .. } => *span,
        Expr::Binary { span, .. } => *span,
        Expr::Call { span, .. } => *span,
        Expr::Match { span, .. } => *span,
        Expr::Policy(_) => Span::default(),
        Expr::IfLet { span, .. } => *span,
        Expr::CheckObligation { span, .. } => *span,
        Expr::Constructor { span, .. } => *span,
        Expr::If { span, .. } => *span,
        Expr::Panic { span, .. } => *span,
        Expr::Fail { span, .. } => *span,
        Expr::WithError { span, .. } => *span,
        Expr::Block { span, .. } => *span,
        Expr::FnDef { span, .. } => *span,
        Expr::FnApply { span, .. } => *span,
        Expr::ActBlock { span, .. } => *span,
        Expr::DoBlock { span, .. } => *span,
        Expr::Comprehension { span, .. } => *span,
        Expr::List { span, .. } => *span,
    }
}

/// Check a field-access expression against record types.
fn check_field_access(env: &TypeEnv, base: &Expr, field: &str, span: Span) -> CheckResult {
    let base_result = check_expr(env, base);
    if !base_result.is_ok() {
        return base_result;
    }

    let base_ty = base_result.substitution.apply(&base_result.ty);
    match &base_ty {
        Type::Record(fields) => match fields.iter().find(|(name, _)| name.as_ref() == field) {
            Some((_, field_ty)) => CheckResult {
                ty: base_result.substitution.apply(field_ty),
                substitution: base_result.substitution,
                errors: Vec::new(),
            },
            None => CheckResult::error(ConstructorError::MissingRecordField {
                field: field.to_string(),
                span,
            }),
        },
        other => CheckResult::error(ConstructorError::NotARecord {
            field: field.to_string(),
            actual: other.clone(),
            span,
        }),
    }
}

/// Check a literal expression
fn check_literal(lit: &Literal) -> CheckResult {
    let ty = match lit {
        Literal::Int(_) => Type::Int,
        Literal::String(_) => Type::String,
        Literal::Bool(_) => Type::Bool,
        Literal::Null => Type::Null,
        Literal::Float(_) => Type::Float,
        Literal::List(items) => infer_list_literal_type(items),
    };
    CheckResult::success(ty)
}

fn infer_list_literal_type(items: &[Literal]) -> Type {
    let mut iter = items.iter();
    let Some(first) = iter.next() else {
        return Type::List(Box::new(Type::Var(TypeVar::fresh())));
    };

    let first_ty = check_literal(first).ty;
    if iter.all(|item| check_literal(item).ty == first_ty) {
        Type::List(Box::new(first_ty))
    } else {
        Type::List(Box::new(Type::Var(TypeVar::fresh())))
    }
}

fn check_unary(env: &TypeEnv, op: UnaryOp, operand: &Expr) -> CheckResult {
    let operand_result = check_expr(env, operand);
    if !operand_result.is_ok() {
        return operand_result;
    }

    let ty = match op {
        UnaryOp::Not if operand_result.ty == Type::Bool => Type::Bool,
        UnaryOp::Neg if operand_result.ty == Type::Int => Type::Int,
        _ => Type::Var(TypeVar::fresh()),
    };

    CheckResult {
        ty,
        substitution: operand_result.substitution,
        errors: operand_result.errors,
    }
}

fn check_binary(env: &TypeEnv, op: BinaryOp, left: &Expr, right: &Expr) -> CheckResult {
    let left_result = check_expr(env, left);
    let right_result = check_expr(env, right);

    if !left_result.is_ok() || !right_result.is_ok() {
        return CheckResult {
            ty: Type::Var(TypeVar::fresh()),
            substitution: left_result.substitution.compose(&right_result.substitution),
            errors: left_result
                .errors
                .into_iter()
                .chain(right_result.errors)
                .collect(),
        };
    }

    let ty = match op {
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod
            if left_result.ty == Type::Int && right_result.ty == Type::Int =>
        {
            Type::Int
        }
        BinaryOp::And | BinaryOp::Or
            if left_result.ty == Type::Bool && right_result.ty == Type::Bool =>
        {
            Type::Bool
        }
        BinaryOp::Eq
        | BinaryOp::Neq
        | BinaryOp::Lt
        | BinaryOp::Gt
        | BinaryOp::Leq
        | BinaryOp::Geq
            if left_result.ty == right_result.ty =>
        {
            Type::Bool
        }
        BinaryOp::In => Type::Bool,
        BinaryOp::Pipe => Type::Var(TypeVar::fresh()),
        _ => Type::Var(TypeVar::fresh()),
    };

    CheckResult {
        ty,
        substitution: left_result.substitution.compose(&right_result.substitution),
        errors: Vec::new(),
    }
}

fn merge_branch_results(left: CheckResult, right: CheckResult) -> CheckResult {
    let substitution = left.substitution.compose(&right.substitution);
    let errors: Vec<ConstructorError> = left.errors.into_iter().chain(right.errors).collect();

    if !errors.is_empty() {
        return CheckResult {
            ty: Type::Var(TypeVar::fresh()),
            substitution,
            errors,
        };
    }

    let ty = unify(&left.ty, &right.ty)
        .map(|subst| subst.apply(&left.ty))
        .unwrap_or(Type::Var(TypeVar::fresh()));

    CheckResult {
        ty,
        substitution,
        errors: Vec::new(),
    }
}

fn pattern_type_env_from(env: &TypeEnv) -> crate::check_pattern::TypeEnv {
    let mut pattern_env = crate::check_pattern::TypeEnv::new();
    for (name, def) in env.ast_type_defs() {
        pattern_env.add_type_def(name.clone(), def.clone());
    }
    pattern_env
}

fn check_with_error(env: &TypeEnv, body: &Expr, arms: &[MatchArm], span: Span) -> CheckResult {
    let body_result = check_expr(env, body);
    let body_ty = body_result.substitution.apply(&body_result.ty);
    let mut substitution = body_result.substitution.clone();
    let mut errors = body_result.errors;

    let pattern_env = pattern_type_env_from(env);
    for arm in arms {
        let mut arm_env = env.clone();
        let failure_payload_ty = Type::Var(TypeVar::fresh());
        match crate::check_pattern::check_pattern(&pattern_env, &arm.pattern, &failure_payload_ty) {
            Ok(bindings) => {
                for (name, ty) in bindings {
                    arm_env.bind_variable(&name, ty);
                }
            }
            Err(error) => errors.push(ConstructorError::UnsupportedExpression {
                kind: format!("with_error handler pattern type error: {error}"),
                span,
            }),
        }

        let arm_result = check_expr(&arm_env, &arm.body);
        errors.extend(arm_result.errors);
        substitution = substitution.compose(&arm_result.substitution);
        let arm_ty = substitution.apply(&arm_result.ty);
        let expected_ty = substitution.apply(&body_ty);
        match unify(&expected_ty, &arm_ty) {
            Ok(unify_subst) => {
                substitution = substitution.compose(&unify_subst);
            }
            Err(_) => errors.push(ConstructorError::UnsupportedExpression {
                kind: format!(
                    "with_error handler type mismatch: expected {expected_ty}, got {arm_ty}"
                ),
                span,
            }),
        }
    }

    CheckResult {
        ty: substitution.apply(&body_ty),
        substitution,
        errors,
    }
}

#[allow(clippy::collapsible_if)]
fn resolve_enum_type_def_for_match<'a>(
    env: &'a TypeEnv,
    scrutinee: &Expr,
    arms: &[MatchArm],
) -> Option<&'a TypeDef> {
    if let Expr::Constructor { name, .. } = scrutinee {
        if let Some((type_name, _)) = env.lookup_constructor(name.as_ref()) {
            if let Some(def) = env.lookup_type(type_name.as_str()) {
                if matches!(&def.body, TypeBody::Enum(_)) {
                    return Some(def);
                }
            }
        }
    }
    for arm in arms {
        if let SurfacePattern::Variant { name, .. } = &arm.pattern {
            if let Some((type_name, _)) = env.lookup_constructor(name.as_ref()) {
                if let Some(def) = env.lookup_type(type_name.as_str()) {
                    if matches!(&def.body, TypeBody::Enum(_)) {
                        return Some(def);
                    }
                }
            }
        }
    }
    None
}

fn format_missing_witnesses(witnesses: &[CorePattern]) -> String {
    witnesses
        .iter()
        .map(format_pattern_witness)
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_pattern_witness(pattern: &CorePattern) -> String {
    match pattern {
        CorePattern::Variant { name, fields } => match fields {
            None => name.clone(),
            Some(fields) if is_tuple_witness_fields(fields) => {
                let items = fields
                    .iter()
                    .map(|(_, pattern)| format_pattern_witness(pattern))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{name}({items})")
            }
            Some(fields) => {
                let items = fields
                    .iter()
                    .map(|(field, pattern)| format!("{field}: {}", format_pattern_witness(pattern)))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{name} {{ {items} }}")
            }
        },
        CorePattern::Wildcard => "_".to_string(),
        CorePattern::Variable { name, .. } => name.clone(),
        CorePattern::Tuple(items) => format!(
            "({})",
            items
                .iter()
                .map(format_pattern_witness)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        CorePattern::Record(fields) => format!(
            "{{ {} }}",
            fields
                .iter()
                .map(|(field, pattern)| format!("{field}: {}", format_pattern_witness(pattern)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        CorePattern::List(items, rest) => {
            let mut rendered = items.iter().map(format_pattern_witness).collect::<Vec<_>>();
            if let Some(rest) = rest {
                rendered.push(format!("..{rest}"));
            }
            format!("[{}]", rendered.join(", "))
        }
        CorePattern::Literal(value) => format!("{value:?}"),
    }
}

fn is_tuple_witness_fields(fields: &[(String, CorePattern)]) -> bool {
    fields
        .iter()
        .enumerate()
        .all(|(index, (field, _))| field == &tuple_field_name(index))
}

fn check_match(env: &TypeEnv, scrutinee: &Expr, arms: &[MatchArm]) -> CheckResult {
    let scrutinee_result = check_expr(env, scrutinee);
    let mut errors: Vec<ConstructorError> = scrutinee_result.errors.clone();

    if let Some(type_def) = resolve_enum_type_def_for_match(env, scrutinee, arms) {
        let patterns: Vec<CorePattern> = arms
            .iter()
            .filter_map(|arm| lower_pattern(&arm.pattern).ok())
            .collect();
        if let Coverage::Missing(witnesses) = check_exhaustive(&patterns, type_def) {
            errors.push(ConstructorError::NonExhaustiveMatch {
                scrutinee_type: type_def.name.clone(),
                missing: format_missing_witnesses(&witnesses),
                span: Span::default(),
            });
        }
    }

    let scrutinee_ty = scrutinee_result.substitution.apply(&scrutinee_result.ty);
    let mut arm_merged: Option<CheckResult> = None;
    for arm in arms {
        let mut arm_env = env.clone();
        if let Ok(bindings) = crate::check_pattern::check_pattern(
            &crate::check_pattern::TypeEnv::new(),
            &arm.pattern,
            &scrutinee_ty,
        ) {
            for (name, ty) in bindings {
                arm_env.bind_variable(&name, ty);
            }
        }

        let body_result = check_expr(&arm_env, &arm.body);
        arm_merged = Some(match arm_merged {
            None => body_result,
            Some(prev) => merge_branch_results(prev, body_result),
        });
    }

    let arm_merged =
        arm_merged.unwrap_or_else(|| CheckResult::success(Type::Var(TypeVar::fresh())));

    let substitution = scrutinee_result
        .substitution
        .compose(&arm_merged.substitution);

    errors.extend(arm_merged.errors);

    CheckResult {
        ty: arm_merged.ty,
        substitution,
        errors,
    }
}

/// Check a constructor expression
///
/// Validates that:
/// 1. The constructor name is known
/// 2. All required fields are present
/// 3. No unknown fields are provided
/// 4. Field types match the expected types
fn check_constructor(
    env: &TypeEnv,
    constructor_name: &str,
    fields: &[(Box<str>, Expr)],
    payload: &ConstructorPayload,
) -> CheckResult {
    let (type_def, variant_idx, variant_def) = match env.get_variant(constructor_name) {
        Some(result) => result,
        None => {
            return CheckResult::error(ConstructorError::UnknownConstructor(
                constructor_name.to_string(),
                Span::default(),
            ));
        }
    };

    let mut errors = Vec::new();
    let mut substitution = Substitution::new();

    match variant_def.payload_shape {
        VariantPayloadShape::Tuple => {
            check_tuple_constructor_fields(
                env,
                constructor_name,
                variant_def,
                fields,
                payload,
                &mut substitution,
                &mut errors,
            );
        }
        VariantPayloadShape::Unit | VariantPayloadShape::Record => {
            check_named_constructor_fields(
                env,
                constructor_name,
                variant_def,
                fields,
                &mut substitution,
                &mut errors,
            );
        }
    }

    let result_type = build_constructor_type(type_def, variant_idx);

    CheckResult {
        ty: substitution.apply(&result_type),
        substitution,
        errors,
    }
}

fn check_tuple_constructor_fields(
    env: &TypeEnv,
    constructor_name: &str,
    variant_def: &VariantInfo,
    fields: &[(Box<str>, Expr)],
    payload: &ConstructorPayload,
    substitution: &mut Substitution,
    errors: &mut Vec<ConstructorError>,
) {
    let tuple_items = match payload {
        ConstructorPayload::Tuple(items) => items,
        _ => {
            errors.push(ConstructorError::TupleArityMismatch {
                constructor: constructor_name.to_string(),
                expected: variant_def.fields.len(),
                actual: fields.len(),
                span: Span::default(),
            });
            return;
        }
    };

    if tuple_items.len() != variant_def.fields.len() || fields.len() != variant_def.fields.len() {
        errors.push(ConstructorError::TupleArityMismatch {
            constructor: constructor_name.to_string(),
            expected: variant_def.fields.len(),
            actual: tuple_items.len(),
            span: Span::default(),
        });
    }

    for (index, ((expected_name, expected_ty), actual_expr)) in variant_def
        .fields
        .iter()
        .zip(tuple_items.iter())
        .enumerate()
    {
        if fields.get(index).map(|(field_name, _)| field_name.as_ref())
            != Some(expected_name.as_str())
        {
            errors.push(ConstructorError::TupleArityMismatch {
                constructor: constructor_name.to_string(),
                expected: variant_def.fields.len(),
                actual: tuple_items.len(),
                span: Span::default(),
            });
            continue;
        }

        if expected_name != &tuple_field_name(index) {
            errors.push(ConstructorError::TupleArityMismatch {
                constructor: constructor_name.to_string(),
                expected: variant_def.fields.len(),
                actual: tuple_items.len(),
                span: Span::default(),
            });
            continue;
        }

        let field_result = check_expr(env, actual_expr);
        errors.extend(field_result.errors);

        let expected_ty_subst = substitution.apply(expected_ty);
        match unify(&expected_ty_subst, &field_result.ty) {
            Ok(sub) => {
                *substitution = substitution.compose(&sub);
            }
            Err(_) => errors.push(ConstructorError::TupleFieldTypeMismatch {
                constructor: constructor_name.to_string(),
                position: index,
                expected: expected_ty.to_string(),
                actual: field_result.ty.to_string(),
                span: Span::default(),
            }),
        }
    }
}

fn check_named_constructor_fields(
    env: &TypeEnv,
    constructor_name: &str,
    variant_def: &VariantInfo,
    fields: &[(Box<str>, Expr)],
    substitution: &mut Substitution,
    errors: &mut Vec<ConstructorError>,
) {
    let expected_fields: HashSet<&str> = variant_def
        .fields
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    let provided_fields: HashSet<&str> = fields.iter().map(|(name, _)| name.as_ref()).collect();

    for expected in &expected_fields {
        if !provided_fields.contains(*expected) {
            errors.push(ConstructorError::MissingField {
                constructor: constructor_name.to_string(),
                field: expected.to_string(),
                span: Span::default(),
            });
        }
    }

    for provided in &provided_fields {
        if !expected_fields.contains(*provided) {
            errors.push(ConstructorError::UnknownField {
                constructor: constructor_name.to_string(),
                field: provided.to_string(),
                span: Span::default(),
            });
        }
    }

    let expected_types: std::collections::HashMap<&str, &Type> = variant_def
        .fields
        .iter()
        .map(|(name, ty)| (name.as_str(), ty))
        .collect();

    for (field_name, field_expr) in fields {
        if let Some(expected_ty) = expected_types.get(field_name.as_ref()) {
            let field_result = check_expr(env, field_expr);
            errors.extend(field_result.errors);

            let expected_ty_subst = substitution.apply(expected_ty);
            match unify(&expected_ty_subst, &field_result.ty) {
                Ok(sub) => {
                    *substitution = substitution.compose(&sub);
                }
                Err(_) => {
                    errors.push(ConstructorError::FieldTypeMismatch {
                        constructor: constructor_name.to_string(),
                        field: field_name.to_string(),
                        expected: expected_ty.to_string(),
                        actual: field_result.ty.to_string(),
                        span: Span::default(),
                    });
                }
            }
        }
    }
}

/// Build the type for a constructor expression
///
/// For a variant of a generic type, this returns the type constructor
/// with the appropriate type variables.
fn build_constructor_type(type_info: &TypeInfo, _variant_idx: VariantIndex) -> Type {
    use crate::kind::Kind;
    use crate::qualified_name::QualifiedName;

    match type_info {
        TypeInfo::Enum { name, params, .. } => {
            // Build Option<T>, not just T
            Type::Constructor {
                name: QualifiedName::root(name.clone()),
                args: params.iter().map(|p| Type::Var(*p)).collect(),
                kind: Kind::Type,
            }
        }
        TypeInfo::Struct { name, params, .. } => Type::Constructor {
            name: QualifiedName::root(name.clone()),
            args: params.iter().map(|p| Type::Var(*p)).collect(),
            kind: Kind::Type,
        },
    }
}

/// Type check an expression and return the inferred type
///
/// This is a convenience function that returns just the type,
/// discarding errors and substitutions.
pub fn infer_type(env: &TypeEnv, expr: &Expr) -> Type {
    let result = check_expr(env, expr);
    result.substitution.apply(&result.ty)
}

/// Resolve a type annotation name to a `Type`.
///
/// Primitives map directly. Registered user-defined types map to
/// `Type::Constructor`. Unknown names produce an error rather than
/// silently falling back to inference (TASK-560).
fn annotation_to_type(
    name: &str,
    env: &TypeEnv,
    span: Span,
    context: &str,
) -> Result<Type, ConstructorError> {
    // Fast path: primitives
    match name {
        "Int" => return Ok(Type::Int),
        "Bool" => return Ok(Type::Bool),
        "String" => return Ok(Type::String),
        "Float" => return Ok(Type::Float),
        "Null" => return Ok(Type::Null),
        "Time" => return Ok(Type::Time),
        "Ref" => return Ok(Type::Ref),
        _ => {}
    }
    // Lookup in TypeEnv for user-defined types
    match env.resolve_type(name) {
        Ok((qualified, _info)) => Ok(Type::Constructor {
            name: qualified,
            args: vec![],
            kind: crate::Kind::Type,
        }),
        Err(_) => Err(ConstructorError::UnknownTypeAnnotation {
            name: name.to_string(),
            context: context.to_string(),
            span,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_parser::surface::Literal;
    use ash_parser::token::Span;

    // ============================================================
    // Literal Tests
    // ============================================================

    #[test]
    fn test_check_literal_int() {
        let env = TypeEnv::with_builtin_types();
        let expr = Expr::Literal(Literal::Int(42));
        let result = check_expr(&env, &expr);

        assert!(result.is_ok());
        assert_eq!(result.ty, Type::Int);
    }

    #[test]
    fn test_check_literal_string() {
        let env = TypeEnv::with_builtin_types();
        let expr = Expr::Literal(Literal::String("hello".into()));
        let result = check_expr(&env, &expr);

        assert!(result.is_ok());
        assert_eq!(result.ty, Type::String);
    }

    #[test]
    fn test_check_literal_bool() {
        let env = TypeEnv::with_builtin_types();
        let expr = Expr::Literal(Literal::Bool(true));
        let result = check_expr(&env, &expr);

        assert!(result.is_ok());
        assert_eq!(result.ty, Type::Bool);
    }

    #[test]
    fn test_check_literal_null() {
        let env = TypeEnv::with_builtin_types();
        let expr = Expr::Literal(Literal::Null);
        let result = check_expr(&env, &expr);

        assert!(result.is_ok());
        assert_eq!(result.ty, Type::Null);
    }

    // ============================================================
    // Constructor Tests - Some { value: 42 }
    // ============================================================

    #[test]
    fn test_check_constructor_some_with_value() {
        let env = TypeEnv::with_builtin_types();

        // Some { value: 42 }
        let expr = Expr::Constructor {
            name: "Some".into(),
            fields: vec![("value".into(), Expr::Literal(Literal::Int(42)))],
            payload: ConstructorPayload::Record(vec![(
                "value".into(),
                Expr::Literal(Literal::Int(42)),
            )]),
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);

        assert!(
            result.is_ok(),
            "Expected success, got errors: {:?}",
            result.errors
        );
        // Constructor returns Option<T>, not just T
        match &result.ty {
            Type::Constructor { name, .. } => {
                assert_eq!(name.to_string(), "Option");
            }
            _ => panic!("Expected constructor type, got {:?}", result.ty),
        }
    }

    #[test]
    fn test_check_constructor_none() {
        let env = TypeEnv::with_builtin_types();

        // None { }
        let expr = Expr::Constructor {
            name: "None".into(),
            fields: vec![],
            payload: ConstructorPayload::Unit,
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);

        assert!(
            result.is_ok(),
            "Expected success, got errors: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_check_constructor_unknown() {
        let env = TypeEnv::with_builtin_types();

        // Unknown { value: 42 }
        let expr = Expr::Constructor {
            name: "Unknown".into(),
            fields: vec![("value".into(), Expr::Literal(Literal::Int(42)))],
            payload: ConstructorPayload::Record(vec![(
                "value".into(),
                Expr::Literal(Literal::Int(42)),
            )]),
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);

        assert!(!result.is_ok());
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(
            result.errors[0],
            ConstructorError::UnknownConstructor(..)
        ));
    }

    #[test]
    fn test_check_constructor_missing_field() {
        let env = TypeEnv::with_builtin_types();

        // Some { } - missing required 'value' field
        let expr = Expr::Constructor {
            name: "Some".into(),
            fields: vec![],
            payload: ConstructorPayload::Record(vec![]),
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);

        assert!(!result.is_ok());
        assert!(result.errors.iter().any(|e| matches!(
            e,
            ConstructorError::MissingField { constructor, field, .. }
            if constructor == "Some" && field == "value"
        )));
    }

    #[test]
    fn test_check_constructor_unknown_field() {
        let env = TypeEnv::with_builtin_types();

        // Some { value: 42, extra: "bad" }
        let expr = Expr::Constructor {
            name: "Some".into(),
            fields: vec![
                ("value".into(), Expr::Literal(Literal::Int(42))),
                ("extra".into(), Expr::Literal(Literal::String("bad".into()))),
            ],
            payload: ConstructorPayload::Record(vec![
                ("value".into(), Expr::Literal(Literal::Int(42))),
                ("extra".into(), Expr::Literal(Literal::String("bad".into()))),
            ]),
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);

        assert!(!result.is_ok());
        assert!(result.errors.iter().any(|e| matches!(
            e,
            ConstructorError::UnknownField { constructor, field, .. }
            if constructor == "Some" && field == "extra"
        )));
    }

    // ============================================================
    // Constructor Tests - Result
    // ============================================================

    #[test]
    fn test_check_constructor_ok() {
        let env = TypeEnv::with_builtin_types();

        // Ok { value: 42 }
        let expr = Expr::Constructor {
            name: "Ok".into(),
            fields: vec![("value".into(), Expr::Literal(Literal::Int(42)))],
            payload: ConstructorPayload::Record(vec![(
                "value".into(),
                Expr::Literal(Literal::Int(42)),
            )]),
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);

        assert!(
            result.is_ok(),
            "Expected success, got errors: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_check_constructor_err() {
        let env = TypeEnv::with_builtin_types();

        // Err { error: "message" }
        let expr = Expr::Constructor {
            name: "Err".into(),
            fields: vec![(
                "error".into(),
                Expr::Literal(Literal::String("message".into())),
            )],
            payload: ConstructorPayload::Record(vec![(
                "error".into(),
                Expr::Literal(Literal::String("message".into())),
            )]),
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);

        assert!(
            result.is_ok(),
            "Expected success, got errors: {:?}",
            result.errors
        );
    }

    // ============================================================
    // Match Exhaustiveness Tests (TASK-130 RED)
    // ============================================================

    #[test]
    fn test_match_non_exhaustive_option_reports_error() {
        let env = TypeEnv::with_builtin_types();

        // match None { Some { value: x } => x }
        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Constructor {
                name: "None".into(),
                fields: vec![],
                payload: ConstructorPayload::Unit,
                span: Span::default(),
            }),
            arms: vec![ash_parser::surface::MatchArm {
                pattern: ash_parser::surface::Pattern::Variant {
                    name: "Some".into(),
                    fields: Some(vec![(
                        "value".into(),
                        ash_parser::surface::Pattern::Variable {
                            name: "x".into(),
                            span: ash_parser::token::Span::default(),
                        },
                    )]),
                    payload: ash_parser::surface::VariantPatternPayload::Record(vec![(
                        "value".into(),
                        ash_parser::surface::Pattern::Variable {
                            name: "x".into(),
                            span: ash_parser::token::Span::default(),
                        },
                    )]),
                },
                body: Box::new(Expr::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                }),
                span: Span::default(),
            }],
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);

        assert!(
            !result.is_ok(),
            "Expected non-exhaustive match to report an error"
        );
        assert!(
            result
                .errors
                .iter()
                .any(|err| err.to_string().contains("non-exhaustive")),
            "Expected a non-exhaustive match error, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_match_exhaustive_option_is_accepted() {
        let env = TypeEnv::with_builtin_types();

        // match None {
        //   Some { value: _ } => 42,
        //   None => 0
        // }
        // Note: Using literal body instead of variable to avoid needing
        // pattern variable binding in type environment (separate feature)
        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Constructor {
                name: "None".into(),
                fields: vec![],
                payload: ConstructorPayload::Unit,
                span: Span::default(),
            }),
            arms: vec![
                ash_parser::surface::MatchArm {
                    pattern: ash_parser::surface::Pattern::Variant {
                        name: "Some".into(),
                        fields: Some(vec![(
                            "value".into(),
                            ash_parser::surface::Pattern::Wildcard,
                        )]),
                        payload: ash_parser::surface::VariantPatternPayload::Record(vec![(
                            "value".into(),
                            ash_parser::surface::Pattern::Wildcard,
                        )]),
                    },
                    body: Box::new(Expr::Literal(Literal::Int(42))),
                    span: Span::default(),
                },
                ash_parser::surface::MatchArm {
                    pattern: ash_parser::surface::Pattern::Variant {
                        name: "None".into(),
                        fields: None,
                        payload: ash_parser::surface::VariantPatternPayload::Unit,
                    },
                    body: Box::new(Expr::Literal(Literal::Int(0))),
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);

        assert!(
            result.is_ok(),
            "Expected exhaustive match to type check, got errors: {:?}",
            result.errors
        );
    }

    // ============================================================
    // infer_type Tests
    // ============================================================

    #[test]
    fn test_infer_type_literal() {
        let env = TypeEnv::with_builtin_types();
        let expr = Expr::Literal(Literal::Int(42));

        let ty = infer_type(&env, &expr);
        assert_eq!(ty, Type::Int);
    }

    #[test]
    fn test_infer_type_constructor_some() {
        let env = TypeEnv::with_builtin_types();
        let expr = Expr::Constructor {
            name: "Some".into(),
            fields: vec![("value".into(), Expr::Literal(Literal::Int(42)))],
            payload: ConstructorPayload::Record(vec![(
                "value".into(),
                Expr::Literal(Literal::Int(42)),
            )]),
            span: Span::default(),
        };

        let ty = infer_type(&env, &expr);
        // Constructor returns Option<T>, not just T
        match &ty {
            Type::Constructor { name, .. } => {
                assert_eq!(name.to_string(), "Option");
            }
            _ => panic!("Expected constructor type, got {:?}", ty),
        }
    }

    #[test]
    fn constructor_returns_constructor_type() {
        let env = TypeEnv::with_builtin_types();

        // Some { value: 42 } should have type Option<Int>, not Int
        let expr = Expr::Constructor {
            name: "Some".into(),
            fields: vec![("value".into(), Expr::Literal(Literal::Int(42)))],
            payload: ConstructorPayload::Record(vec![(
                "value".into(),
                Expr::Literal(Literal::Int(42)),
            )]),
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);

        // Should be Option<Int>
        match result.ty {
            Type::Constructor { name, .. } => {
                assert_eq!(name.to_string(), "Option");
            }
            _ => panic!("Expected constructor type, got {:?}", result.ty),
        }
    }

    // ============================================================
    // CheckResult Tests
    // ============================================================

    #[test]
    fn test_qualified_call_to_registered_capability_reports_wrong_target() {
        let mut env = TypeEnv::with_builtin_types();
        env.register_capability_symbol("Sensor");

        let expr = Expr::Call {
            func: "read".into(),
            module: Some("Sensor".into()),
            args: vec![],
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);
        assert!(!result.is_ok());
        let error = result.errors[0].to_string();
        assert!(error.contains("Sensor::read"), "unexpected error: {error}");
        assert!(error.contains("capability"), "unexpected error: {error}");
        assert!(
            error.contains("not a function"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_check_result_success() {
        let result = CheckResult::success(Type::Int);
        assert!(result.is_ok());
        assert_eq!(result.ty, Type::Int);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_check_result_error() {
        let err = ConstructorError::UnknownConstructor("Foo".to_string(), Span::default());
        let result = CheckResult::error(err.clone());
        assert!(!result.is_ok());
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0], err);
    }

    #[test]
    fn test_format_missing_witnesses_preserves_tuple_shape() {
        let rendered = format_missing_witnesses(&[CorePattern::Variant {
            name: "RuntimeError".into(),
            fields: Some(vec![
                ("_0".into(), CorePattern::Wildcard),
                ("_1".into(), CorePattern::Wildcard),
            ]),
        }]);

        assert_eq!(rendered, "RuntimeError(_, _)");
    }

    // ============================================================
    // TASK-558: Three-vertex boundary – FnDef typing (SPEC-031 §4.8)
    // ============================================================

    /// Helper: build a simple one-param FnDef whose body is the param itself.
    fn make_fn_def_expr(param: &str) -> Expr {
        Expr::FnDef {
            params: vec![(Box::from(param), None)],
            return_type: None,
            body: Box::new(Expr::Variable {
                name: Box::from(param),
                span: ash_parser::token::Span::default(),
            }),
            span: Span::default(),
        }
    }

    /// Test 1 (TASK-558): FnDef in a pure (no-workflow) context → Type::Fn
    #[test]
    fn task558_fnapp_in_pure_context_yields_type_fn() {
        let env = TypeEnv::with_builtin_types();
        assert!(
            env.workflow_effect().is_none(),
            "pure env should have no workflow effect"
        );

        let expr = make_fn_def_expr("x");
        let result = check_expr(&env, &expr);

        assert!(result.is_ok(), "expected success, got {:?}", result.errors);
        match result.ty {
            Type::Fn(params, _ret) => {
                assert_eq!(params.len(), 1, "expected 1 param");
            }
            other => panic!("expected Type::Fn, got {other:?}"),
        }
    }

    /// Test 2 (TASK-558): FnDef in a workflow context → Type::Fun with the workflow's effect
    #[test]
    fn task558_fndef_in_workflow_context_yields_type_fun() {
        let mut env = TypeEnv::with_builtin_types();
        env.set_workflow_effect(ash_core::Effect::Operational);

        let expr = make_fn_def_expr("x");
        let result = check_expr(&env, &expr);

        assert!(result.is_ok(), "expected success, got {:?}", result.errors);
        match result.ty {
            Type::Fun(params, _ret, effect) => {
                assert_eq!(params.len(), 1);
                assert_eq!(effect, ash_core::Effect::Operational);
            }
            other => panic!("expected Type::Fun, got {other:?}"),
        }
    }

    /// Test 2b (TASK-558): workflow context at Epistemic level → Type::Fun(_, _, Epistemic)
    #[test]
    fn task558_fndef_in_epistemic_workflow_yields_epistemic_fun() {
        let mut env = TypeEnv::with_builtin_types();
        env.set_workflow_effect(ash_core::Effect::Epistemic);

        let expr = make_fn_def_expr("x");
        let result = check_expr(&env, &expr);

        assert!(result.is_ok(), "expected success, got {:?}", result.errors);
        match result.ty {
            Type::Fun(_, _, effect) => {
                assert_eq!(effect, ash_core::Effect::Epistemic);
            }
            other => panic!("expected Type::Fun, got {other:?}"),
        }
    }

    /// Test 3 (TASK-558): Fn/Fun unification is already rejected by the unifier.
    /// Verify that unifying Type::Fn with Type::Fun returns an error.
    #[test]
    fn task558_fn_fun_unification_rejected() {
        use crate::types::unify;
        let pure_fn = Type::Fn(vec![Type::Int], Box::new(Type::Int));
        let effect_fn = Type::Fun(
            vec![Type::Int],
            Box::new(Type::Int),
            ash_core::Effect::Operational,
        );
        assert!(
            unify(&pure_fn, &effect_fn).is_err(),
            "unifying Type::Fn with Type::Fun must fail"
        );
        assert!(
            unify(&effect_fn, &pure_fn).is_err(),
            "unifying Type::Fun with Type::Fn must fail"
        );
    }

    /// Test 4 (TASK-558): FnApply where the function expects a pure Fn parameter but receives
    /// a workflow Fun value triggers a type error via the unifier.
    #[test]
    fn task558_pass_fun_to_fn_parameter_is_rejected() {
        // Build env where `apply` is bound as  Fn([Fn([Int], Int)], Int)
        // i.e. apply : (Int -> Int) -> Int
        let mut env = TypeEnv::with_builtin_types();
        let apply_ty = Type::Fn(
            vec![Type::Fn(vec![Type::Int], Box::new(Type::Int))],
            Box::new(Type::Int),
        );
        env.bind_variable("apply", apply_ty);

        // In workflow context, `fn(x) { x }` gets type Fun([Int], Int, Operational)
        env.set_workflow_effect(ash_core::Effect::Operational);
        let closure_expr = make_fn_def_expr("x");

        // Build the call: apply(fn(x) { x })
        let call_expr = Expr::FnApply {
            func: Box::new(Expr::Variable {
                name: Box::from("apply"),
                span: ash_parser::token::Span::default(),
            }),
            args: vec![closure_expr],
            span: Span::default(),
        };

        let result = check_expr(&env, &call_expr);
        // The unifier must reject Fun where Fn is expected
        assert!(
            !result.is_ok(),
            "passing Type::Fun where Type::Fn expected must fail, but got type {:?}",
            result.ty
        );
    }

    /// Test 5 (TASK-558): workflow_effect propagates into child scopes (extend()).
    #[test]
    fn task558_workflow_effect_propagates_to_child_env() {
        let mut env = TypeEnv::with_builtin_types();
        env.set_workflow_effect(ash_core::Effect::Deliberative);
        let child = env.extend();
        assert_eq!(
            child.workflow_effect(),
            Some(ash_core::Effect::Deliberative)
        );
    }

    /// Escape case 1 (TASK-558): A FnDef in workflow context produces Type::Fun, which
    /// must NOT unify with Type::Fn — enforcing that Fun cannot be returned where Fn is expected.
    #[test]
    fn task558_return_fun_where_fn_expected_is_rejected() {
        // In a workflow context, fn(x) { x } types as Type::Fun([Var], Var, Operational).
        // Unifying that with Type::Fn([Int], Int) must fail — this is escape case 1:
        // "return Fun from workflow where Fn is expected".
        let mut env = TypeEnv::with_builtin_types();
        env.set_workflow_effect(ash_core::Effect::Operational);

        let expr = make_fn_def_expr("x");
        let result = check_expr(&env, &expr);

        // Verify the closure typed as Fun in workflow context
        assert!(
            matches!(&result.ty, Type::Fun(..)),
            "FnDef in workflow context must produce Type::Fun, got {:?}",
            result.ty
        );

        // Now attempt to unify with a pure Fn type — must fail (escape case 1)
        let pure_fn = Type::Fn(
            vec![Type::Var(TypeVar::fresh())],
            Box::new(Type::Var(TypeVar::fresh())),
        );
        let unify_result = unify(&result.ty, &pure_fn);
        assert!(
            unify_result.is_err(),
            "Type::Fun must not unify with Type::Fn (escape case 1: return Fun where Fn expected)"
        );
    }

    /// Escape case 4 (TASK-558): List<Fun> must not unify with List<Fn>.
    /// Container-level type propagation enforces the boundary across collections.
    #[test]
    fn task558_list_fun_where_list_fn_expected_is_rejected() {
        let list_of_fn = Type::List(Box::new(Type::Fn(vec![Type::Int], Box::new(Type::Int))));
        let list_of_fun = Type::List(Box::new(Type::Fun(
            vec![Type::Int],
            Box::new(Type::Int),
            ash_core::Effect::Operational,
        )));
        let result = unify(&list_of_fn, &list_of_fun);
        assert!(
            result.is_err(),
            "List<Fun> must not unify with List<Fn> (escape case 4: Fun through container)"
        );
    }

    /// Fix: FnDef param type annotation constrains inference (SPEC-031 §5.1).
    /// `fn(x: Int) { x }` must give param type Int, not a fresh type variable.
    #[test]
    fn task558_fndef_annotated_param_constrains_inference() {
        let env = TypeEnv::with_builtin_types();
        let expr = Expr::FnDef {
            params: vec![("x".into(), Some("Int".into()))],
            return_type: None,
            body: Box::new(Expr::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            }),
            span: Span::default(),
        };
        let result = check_expr(&env, &expr);
        assert!(
            result.is_ok(),
            "annotated fn should typecheck: {:?}",
            result.errors
        );
        // The function type should have Int as the param type (not a type variable)
        match &result.ty {
            Type::Fn(params, _) => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0], Type::Int, "annotated param must resolve to Int");
            }
            other => panic!("expected Type::Fn, got {other:?}"),
        }
    }

    /// Fix: FnDef return type annotation is verified against inferred body type (SPEC-031 §5.1).
    /// `fn(x: Int) -> Int { x }` must succeed; `fn(x: Int) -> Bool { x }` must fail.
    #[test]
    fn task558_fndef_annotated_return_type_verified() {
        let env = TypeEnv::with_builtin_types();

        // Matching annotation: fn(x: Int) -> Int { x }  — should pass
        let matching = Expr::FnDef {
            params: vec![("x".into(), Some("Int".into()))],
            return_type: Some("Int".into()),
            body: Box::new(Expr::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            }),
            span: Span::default(),
        };
        let ok_result = check_expr(&env, &matching);
        assert!(
            ok_result.is_ok(),
            "fn(x: Int) -> Int {{ x }} should typecheck, errors: {:?}",
            ok_result.errors
        );

        // Conflicting annotation: fn(x: Int) -> Bool { x }  — should fail
        let conflicting = Expr::FnDef {
            params: vec![("x".into(), Some("Int".into()))],
            return_type: Some("Bool".into()),
            body: Box::new(Expr::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            }),
            span: Span::default(),
        };
        let err_result = check_expr(&env, &conflicting);
        assert!(
            !err_result.is_ok(),
            "fn(x: Int) -> Bool {{ x }} should fail: return type annotation conflicts with body"
        );
    }

    /// Escape case 2 (TASK-558): Storing a Fun in instance state typed as Fn must be rejected.
    /// The unify(Type::Fun, Type::Fn) failure is the mechanism that prevents
    /// a closure (Fun) from being assigned to an Fn-typed state field.
    #[test]
    fn task558_escape_case_2_store_fun_in_state_rejected() {
        let fun_ty = Type::Fun(
            vec![Type::Int],
            Box::new(Type::Int),
            ash_core::Effect::Operational,
        );
        let fn_ty = Type::Fn(vec![Type::Int], Box::new(Type::Int));
        let result = unify(&fun_ty, &fn_ty);
        assert!(
            result.is_err(),
            "Type::Fun must not unify with Type::Fn (escape case 2: storing Fun in Fn-typed state)"
        );
    }

    // ============================================================
    // TASK-560: Unknown type annotation errors
    // ============================================================

    /// Unknown param annotation should produce an error, not silently
    /// fall back to a fresh type variable.
    #[test]
    fn task560_unknown_param_annotation_produces_error() {
        let env = TypeEnv::with_builtin_types();
        // fn(x: BogusType) { x }
        let expr = Expr::FnDef {
            params: vec![("x".into(), Some("BogusType".into()))],
            return_type: None,
            body: Box::new(Expr::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            }),
            span: Span::default(),
        };
        let result = check_expr(&env, &expr);
        assert!(
            !result.is_ok(),
            "fn(x: BogusType) should produce an error for unknown annotation, got: {:?}",
            result.errors
        );
        let found = result.errors.iter().any(|e| {
            matches!(
                e,
                ConstructorError::UnknownTypeAnnotation { name, context, .. }
                if name == "BogusType" && context.contains("parameter")
            )
        });
        assert!(
            found,
            "expected UnknownTypeAnnotation error for parameter, got: {:?}",
            result.errors
        );
    }

    /// Unknown return type annotation should produce an error.
    #[test]
    fn task560_unknown_return_annotation_produces_error() {
        let env = TypeEnv::with_builtin_types();
        // fn(x) -> BogusRet { x }
        let expr = Expr::FnDef {
            params: vec![("x".into(), None)],
            return_type: Some("BogusRet".into()),
            body: Box::new(Expr::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            }),
            span: Span::default(),
        };
        let result = check_expr(&env, &expr);
        assert!(
            !result.is_ok(),
            "fn(x) -> BogusRet should produce an error for unknown return annotation, got: {:?}",
            result.errors
        );
        let found = result.errors.iter().any(|e| {
            matches!(
                e,
                ConstructorError::UnknownTypeAnnotation { name, context, .. }
                if name == "BogusRet" && context.contains("return type")
            )
        });
        assert!(
            found,
            "expected UnknownTypeAnnotation error for return type, got: {:?}",
            result.errors
        );
    }

    /// A user-defined type registered in TypeEnv should resolve as a Constructor.
    #[test]
    fn task560_user_defined_type_annotation_resolves() {
        use ash_core::ast::{TypeBody, TypeDef, Visibility};
        let mut env = TypeEnv::with_builtin_types();
        // Register a custom enum type "Color" with no variants for testing
        let color_def = TypeDef {
            name: "Color".into(),
            params: vec![],
            body: TypeBody::Enum(vec![]),
            visibility: Visibility::Public,
            builtin: false,
        };
        env.register_type(&color_def).expect("register Color type");
        // fn(x: Color) { x }
        let expr = Expr::FnDef {
            params: vec![("x".into(), Some("Color".into()))],
            return_type: None,
            body: Box::new(Expr::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            }),
            span: Span::default(),
        };
        let result = check_expr(&env, &expr);
        assert!(
            result.is_ok(),
            "fn(x: Color) should typecheck with registered type, errors: {:?}",
            result.errors
        );
        match &result.ty {
            Type::Fn(params, _ret) => {
                assert_eq!(params.len(), 1);
                match &params[0] {
                    Type::Constructor { name, args, .. } => {
                        assert_eq!(name.name.as_str(), "Color");
                        assert!(args.is_empty(), "Color has no type params");
                    }
                    other => panic!("expected Constructor type for param, got {other:?}"),
                }
            }
            other => panic!("expected Type::Fn, got {other:?}"),
        }
    }
}
