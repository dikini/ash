//! Expression type checking
//!
//! Provides type checking for expressions, including constructor expressions.

#![allow(clippy::result_large_err)]

use crate::check_pattern::{
    Bindings, IrrefutabilityBlockedReason, IrrefutabilityImpossibleReason, IrrefutabilityOutcome,
    IrrefutabilityWitness, TypeEnv as PatternTypeEnv,
    check_irrefutable_pattern_with_canonicalization,
};
use crate::error::ConstructorError;
use crate::exhaustiveness::{
    MatchCoverage, check_match_exhaustive, check_match_exhaustive_with_canonicalization,
};
use crate::type_env::{
    ContractIntrinsic, PatternCanonicalization, PatternCanonicalizationBlockedReason, TypeEnv,
    TypeInfo, VariantIndex, VariantInfo,
};
use crate::types::{Substitution, Type, TypeVar, unify};
use ash_core::adt::{VariantPayloadShape, tuple_field_name};
use ash_core::ast::{
    Expr as CoreExpr, Pattern as CorePattern, Span as CoreSpan, TypeBody, TypeDef, TypeExpr,
};
use ash_core::contract::Requirement;
use ash_core::module_graph::ModuleId;
use ash_core::runtime::FailureBoundary;
use ash_core::semantic_summary::{
    ModuleIdentity, ModuleSourceOrigin, SourceAnchor, SourceOrigin as SemanticSourceOrigin,
    TypeDeclId,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, ConstructorVariableApp, ConstructorVariableRef, TcirBinder, TcirClosure,
    TcirComputationExpression, TcirDoTarget, TcirFailureBoundaryProvenance, TcirOperation,
    TcirOperationKind, TcirSelectedEvidence, TcirStatement, TcirStatementId, TcirStatementKind,
    TypeConstructorHeadId,
};
use ash_parser::contract_classifier;
use ash_parser::lower_pattern;
use ash_parser::surface::ConstructorPayload;
use ash_parser::surface::{
    BinaryOp, ComprehensionQualifier, DoStmt, Expr, Literal, MatchArm, Pattern,
    Type as SurfaceType, UnaryOp,
};
use ash_parser::token::Span;
use std::collections::{HashMap, HashSet};

mod result;

mod core;
mod do_notation;
mod pattern_bridge;

pub use core::check_core_expr;
pub use do_notation::do_notation_diagnostics;
use do_notation::{collect_do_notation_diagnostics, diagnostic_expr_type};
pub(crate) use pattern_bridge::{
    bind_irrefutable_pattern_bindings, check_irrefutable_let_pattern, check_pattern_bindings,
    pattern_canonicalization_for_scrutinee, pattern_type_env_from_type_env, surface_pattern_span,
};
use pattern_bridge::{format_irrefutable_let_error, format_surface_pattern};
use result::has_fatal_diagnostics;
pub use result::{CheckResult, DoElaborationResult};

/// Type check an expression
///
/// This function infers the type of an expression and returns the result
/// along with any substitutions and errors.
pub fn check_expr(env: &TypeEnv, expr: &Expr) -> CheckResult {
    match expr {
        Expr::OperatorSection { section } => {
            CheckResult::error(ConstructorError::UnsupportedExpression {
                kind: format!(
                    "operator section `{}` requires notation resolution before type checking",
                    section.operator.spelling
                ),
                span: section.span,
            })
        }
        Expr::MacroInvocation { invocation } => {
            CheckResult::error(ConstructorError::UnsupportedExpression {
                kind: format!(
                    "unexpanded macro invocation carrier `{}!` reached type checking",
                    invocation.name
                ),
                span: invocation.span,
            })
        }
        Expr::On { span, .. } => CheckResult::error(ConstructorError::UnsupportedExpression {
            kind: "source handler bodies require checked handler declaration validation"
                .to_string(),
            span: *span,
        }),
        Expr::HandleWith {
            expression,
            handler,
            span,
            ..
        } => {
            if let Err(error) = env.require_handler_callable(handler.as_ref()) {
                return CheckResult::error(ConstructorError::UnsupportedExpression {
                    kind: error.to_string(),
                    span: *span,
                });
            }
            let expression_result = check_expr(env, expression);
            let provisional_type = expression_result.substitution.apply(&expression_result.ty);
            let substitution = expression_result.substitution;
            let mut errors = expression_result.errors;
            let Some(Type::Fn(params, result)) = env.lookup_variable(handler.as_ref()) else {
                // A local `derive handler` contributes a handler marker but no
                // ordinary value signature. Its complete polymorphic source
                // fact is published by the declaration pass, so this early
                // expression traversal may only carry the operand result
                // provisionally. Final application validation requires that
                // checked fact; this branch never manufactures a binding.
                return CheckResult {
                    ty: provisional_type,
                    substitution,
                    errors,
                };
            };
            let Some(_) = params.first() else {
                errors.push(ConstructorError::UnsupportedExpression {
                    kind: format!(
                        "handler '{handler}' must declare exactly one input type for `handle ... with`"
                    ),
                    span: *span,
                });
                return CheckResult {
                    ty: substitution.apply(&result),
                    substitution,
                    errors,
                };
            };
            if params.len() != 1 {
                errors.push(ConstructorError::UnsupportedExpression {
                    kind: format!(
                        "handler '{handler}' must declare exactly one input type for `handle ... with`"
                    ),
                    span: *span,
                });
                return CheckResult {
                    ty: substitution.apply(&result),
                    substitution,
                    errors,
                };
            }
            // `handle expression with handler` is the one source form that
            // accepts immutable implicit-thunk evidence.  The declaration
            // pass compares the normalized `Unit -> {row} result` fact once
            // all handler facts are available; ordinary calls never reach
            // that path and therefore never acquire implicit thunking.
            CheckResult {
                ty: substitution.apply(&result),
                substitution,
                errors,
            }
        }
        Expr::Literal(lit) => check_literal(lit),
        Expr::Variable { name, .. } => {
            if name.as_ref() == "()" {
                return CheckResult::success(Type::Constructor {
                    name: crate::QualifiedName::root("()"),
                    args: Vec::new(),
                    kind: crate::Kind::Type,
                });
            }

            // Look up lexical variables first. If no variable is bound, allow
            // registered unit enum constructors to be used as bare values (for
            // example `ret System` for `pub type Role = System | User`).
            match env.lookup_variable(name.as_ref()) {
                Some(ty) => CheckResult::success(ty),
                None => match env.get_variant(name.as_ref()) {
                    Some((type_info, variant_idx, variant_info))
                        if matches!(variant_info.payload_shape, VariantPayloadShape::Unit) =>
                    {
                        CheckResult::success(build_constructor_type(type_info, variant_idx))
                    }
                    _ => CheckResult::error(ConstructorError::UnboundVariable {
                        name: name.to_string(),
                        span: get_expr_span(expr),
                    }),
                },
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
        Expr::Record { fields, .. } => check_record_expr(env, fields),
        Expr::IfLet {
            pattern,
            expr,
            then_branch,
            else_branch,
            span,
        } => {
            let matched_result = check_expr(env, expr);
            if !matched_result.is_ok() {
                return matched_result;
            }

            let matched_ty = matched_result.substitution.apply(&matched_result.ty);
            let pattern_env = pattern_type_env_from_type_env(env);
            let canonicalization = pattern_canonicalization_for_scrutinee(env, &matched_ty);
            let irrefutability = check_irrefutable_pattern_with_canonicalization(
                &pattern_env,
                pattern,
                &matched_ty,
                &canonicalization,
            );

            let mut diagnostics = Vec::new();
            match &irrefutability.outcome {
                IrrefutabilityOutcome::Irrefutable => {
                    diagnostics.push(ConstructorError::UnreachableIfLetElse {
                        reason: format!(
                            "pattern {} is irrefutable for {}; else branch is unreachable",
                            format_surface_pattern(pattern),
                            matched_ty
                        ),
                        span: surface_pattern_span(pattern, *span),
                    });
                }
                IrrefutabilityOutcome::Refutable { .. } => {}
                IrrefutabilityOutcome::Impossible { .. } => {
                    return CheckResult::error(ConstructorError::UnsupportedExpression {
                        kind: format!(
                            "if let pattern {} over type {} is impossible: {}",
                            format_surface_pattern(pattern),
                            matched_ty,
                            format_irrefutable_let_error(
                                "if let",
                                pattern,
                                &matched_ty,
                                &irrefutability.outcome
                            )
                        ),
                        span: surface_pattern_span(pattern, *span),
                    });
                }
                IrrefutabilityOutcome::Blocked { .. } => {
                    if let Err(error) =
                        crate::check_pattern::check_pattern(&pattern_env, pattern, &matched_ty)
                    {
                        return CheckResult::error(ConstructorError::UnsupportedExpression {
                            kind: format!(
                                "if let pattern {} over type {} is impossible: {}",
                                format_surface_pattern(pattern),
                                matched_ty,
                                error
                            ),
                            span: surface_pattern_span(pattern, *span),
                        });
                    }

                    return CheckResult::error(ConstructorError::UnsupportedExpression {
                        kind: format!(
                            "if let pattern {} over type {} is blocked: {}",
                            format_surface_pattern(pattern),
                            matched_ty,
                            format_irrefutable_let_error(
                                "if let",
                                pattern,
                                &matched_ty,
                                &irrefutability.outcome
                            )
                        ),
                        span: surface_pattern_span(pattern, *span),
                    });
                }
            }

            let mut then_env = env.clone();
            for (name, ty) in irrefutability.bindings {
                then_env.bind_variable(&name, ty);
            }

            let then_result = check_expr(&then_env, then_branch);
            let else_result = check_expr(env, else_branch);
            let mut merged = merge_if_let_branch_results(then_result, else_result, *span);
            merged.errors.extend(diagnostics);
            merged
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
            let base_has_fatal = base_result.has_fatal_errors();
            let index_has_fatal = index_result.has_fatal_errors();
            errors.extend(base_result.errors);
            errors.extend(index_result.errors);

            if base_has_fatal || index_has_fatal {
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
            if module.as_deref() == Some("contract") {
                let qualified_intrinsic = format!("contract::{func}");
                if let Some(intrinsic) = env.lookup_contract_intrinsic(&qualified_intrinsic) {
                    let has_contract_intrinsic_context = env
                        .lookup_variable("__contract_intrinsic_context")
                        .is_some();
                    match func.as_ref() {
                        "requires" => {
                            if args.len() != 1 {
                                return CheckResult::error(contract_intrinsic_misuse_error(
                                    &intrinsic,
                                    format!("expects 1 arguments, found {}", args.len()),
                                    *span,
                                ));
                            }
                            if let Err(err) = validate_requirement_expr(env, &args[0]) {
                                return CheckResult::error(contract_intrinsic_misuse_error(
                                    &intrinsic,
                                    err.to_string(),
                                    *span,
                                ));
                            }
                            if !has_contract_intrinsic_context {
                                return CheckResult::error(contract_intrinsic_context_misuse_error(
                                    &intrinsic,
                                    "requires refines application contract context and does not produce a denotable value".to_string(),
                                    *span,
                                ));
                            }
                            return CheckResult::success(intrinsic.result_type().clone());
                        }
                        "ensures" => {
                            if args.len() != 1 {
                                return CheckResult::error(contract_intrinsic_misuse_error(
                                    &intrinsic,
                                    format!("expects 1 arguments, found {}", args.len()),
                                    *span,
                                ));
                            }
                            if !has_contract_intrinsic_context
                                && expr_mentions_variable(&args[0], "result")
                            {
                                return CheckResult::error(contract_intrinsic_misuse_error(
                                    &intrinsic,
                                    "open result postcondition mentions `result` but has no application result boundary".to_string(),
                                    *span,
                                ));
                            }
                            if let Err(err) = validate_postcondition_expr(&args[0]) {
                                return CheckResult::error(contract_intrinsic_misuse_error(
                                    &intrinsic,
                                    err.to_string(),
                                    *span,
                                ));
                            }
                            if !has_contract_intrinsic_context {
                                return CheckResult::error(contract_intrinsic_context_misuse_error(
                                    &intrinsic,
                                    "ensures targets the application result boundary and does not produce a denotable value".to_string(),
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
                errors.extend(arg_result.errors);
                substitution = substitution.compose(&arg_result.substitution);
                arg_types.push(substitution.apply(&arg_result.ty));
            }

            if has_fatal_diagnostics(&errors) {
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
                return CheckResult::error(ConstructorError::UnsupportedExpression {
                    kind: format!(
                        "Call ({qualified_name}): direct source invoke is not admitted; use an admitted named interface or binding operation"
                    ),
                    span: *span,
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
                                    errors,
                                },
                                Some(Err(_unify_err)) => {
                                    let parameter_types = match &func_ty {
                                        Type::Fn(params, _) | Type::Fun(params, _, _) => params,
                                        _ => unreachable!("callable types handled above"),
                                    };
                                    let mismatch = parameter_types
                                        .iter()
                                        .zip(&arg_types)
                                        .find(|(expected, actual)| {
                                            env.unify_types(expected, actual).is_err()
                                        })
                                        .map(|(expected, actual)| {
                                            format!("expected {expected} but found {actual}")
                                        })
                                        .unwrap_or_else(|| "argument type mismatch".to_string());
                                    CheckResult::error(ConstructorError::UnsupportedExpression {
                                        kind: format!("Call ({qualified_name}): {mismatch}"),
                                        span: *span,
                                    })
                                }
                                None => {
                                    CheckResult::error(ConstructorError::UnsupportedExpression {
                                        kind: format!(
                                            "Call ({qualified_name}): expected exactly {} args, got {}",
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
                    if let Some(impl_type) = module.as_deref()
                        && !env.has_capability_symbol(impl_type)
                        // `Interface::method` remains the established
                        // interface-dispatch form.  Concrete-operation
                        // resolution applies only to a non-interface nominal
                        // implementation qualifier such as `PosixFs::read`.
                        && !env.has_interface(impl_type)
                    {
                        match env.resolve_declared_concrete_operation(impl_type, func.as_ref()) {
                            Ok(operation) => {
                                if operation.params.len() != arg_types.len()
                                    || operation.params.iter().zip(&arg_types).any(
                                        |(expected, actual)| {
                                            env.unify_types(expected, actual).is_err()
                                        },
                                    )
                                {
                                    return CheckResult::error(
                                        ConstructorError::UnsupportedExpression {
                                            kind: format!(
                                                "{impl_type}::{}: argument type mismatch",
                                                operation.operation
                                            ),
                                            span: *span,
                                        },
                                    );
                                }
                                return CheckResult {
                                    ty: operation.result_type,
                                    substitution,
                                    errors,
                                };
                            }
                            Err(reason) if reason.starts_with("unknown concrete impl") => {
                                return CheckResult::error(
                                    ConstructorError::UnsupportedExpression {
                                        // Preserve the established qualified-call diagnostic
                                        // family for names that are not declarations, while
                                        // retaining the concrete-operation detail needed by
                                        // declaration-backed callers.
                                        kind: format!(
                                            "call to unknown function '{qualified_name}' ({reason})"
                                        ),
                                        span: *span,
                                    },
                                );
                            }
                            Err(reason)
                                if reason.starts_with("concrete impl")
                                    && reason.contains("has no operation") =>
                            {
                                return CheckResult::error(
                                    ConstructorError::UnsupportedExpression {
                                        kind: reason,
                                        span: *span,
                                    },
                                );
                            }
                            Err(_) => {}
                        }
                    }
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
                                    errors,
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

                    if has_fatal_diagnostics(&errors) {
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
                            errors,
                        },
                        Err(_) => {
                            errors.push(ConstructorError::UnsupportedExpression {
                                kind: format!(
                                    "If expression: branch types differ ({} vs {})",
                                    then_ty, else_ty
                                ),
                                span: *span,
                            });
                            CheckResult {
                                ty: Type::Var(TypeVar::fresh()),
                                substitution: combined_sub,
                                errors,
                            }
                        }
                    }
                }
                None => {
                    if has_fatal_diagnostics(&errors) {
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
                            errors,
                        },
                        Err(_) => {
                            errors.push(ConstructorError::UnsupportedExpression {
                                kind: format!(
                                    "If expression without else requires then branch to have type Null, found {}",
                                    then_ty
                                ),
                                span: *span,
                            });
                            CheckResult {
                                ty: Type::Var(TypeVar::fresh()),
                                substitution: combined_sub,
                                errors,
                            }
                        }
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
                    ash_parser::surface::BlockStmt::Expr { expr, .. } => {
                        let expr_result = check_expr(&block_env, expr);
                        substitution = substitution.compose(&expr_result.substitution);
                        let expr_has_fatal = expr_result.has_fatal_errors();
                        errors.extend(expr_result.errors);
                        if expr_has_fatal {
                            continue;
                        }
                    }
                    ash_parser::surface::BlockStmt::Let {
                        pattern,
                        expr,
                        span,
                    } => {
                        let expr_result = check_expr(&block_env, expr);
                        substitution = substitution.compose(&expr_result.substitution);
                        let expr_has_fatal = expr_result.has_fatal_errors();
                        errors.extend(expr_result.errors);
                        if expr_has_fatal {
                            continue;
                        }
                        let expr_ty = substitution.apply(&expr_result.ty);
                        let pattern_span = surface_pattern_span(pattern, *span);
                        match check_irrefutable_let_pattern(
                            &block_env,
                            "let",
                            pattern,
                            &expr_ty,
                            pattern_span,
                        ) {
                            Ok(bindings) => {
                                bind_irrefutable_pattern_bindings(&mut block_env, bindings);
                            }
                            Err(error) => errors.push(error),
                        }
                    }
                }
            }

            if has_fatal_diagnostics(&errors) {
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
                    if tail_result.has_fatal_errors() {
                        return CheckResult {
                            ty: Type::Var(TypeVar::fresh()),
                            substitution: combined_sub,
                            errors: tail_result.errors,
                        };
                    }
                    errors.extend(tail_result.errors);
                    CheckResult {
                        ty: combined_sub.apply(&tail_result.ty),
                        substitution: combined_sub,
                        errors,
                    }
                }
                None => CheckResult {
                    ty: Type::Null,
                    substitution,
                    errors,
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
            // Pure callable boundary (SPEC-072 / TASK-959):
            //   - Pure `fn`/closure syntax always yields Type::Fn(params, ret)
            //   - Ambient profile effect context does not reclassify pure closures as Type::Fun
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

            let fn_ty = Type::Fn(param_types, Box::new(ret_ty));
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

            if has_fatal_diagnostics(&errors) {
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
                    errors,
                },
                Some(Err(e)) => CheckResult::error(ConstructorError::UnsupportedExpression {
                    kind: format!("FnApply: type mismatch applying args to {func_ty}: {e}"),
                    span: *span,
                }),
                None => {
                    let kind = if func_ty.is_function_type() {
                        format!(
                            "FnApply: expected exactly {} args, got {} for type {func_ty}",
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

        Expr::Comprehension {
            target,
            result,
            qualifiers,
            span,
        } => check_comprehension(env, target.as_ref(), result, qualifiers, *span),
        Expr::List { items, span } => {
            if items.is_empty() {
                // Empty list: return List<a> where a is a fresh type variable
                let item_ty = Type::Var(TypeVar::fresh());
                CheckResult {
                    ty: Type::List(Box::new(item_ty)),
                    substitution: Substitution::new(),
                    errors: vec![],
                }
            } else {
                // Non-empty list: check all elements and unify their types
                let mut errors = Vec::new();
                let mut substitution = Substitution::new();
                let first_result = check_expr(env, &items[0]);
                let mut item_ty = first_result.ty.clone();
                substitution = substitution.compose(&first_result.substitution);
                errors.extend(first_result.errors);

                for item in items.iter().skip(1) {
                    let item_result = check_expr(env, item);
                    let item_ty_applied = item_result.substitution.apply(&item_ty);
                    let new_ty_applied = item_result.substitution.apply(&item_result.ty);
                    match unify(&item_ty_applied, &new_ty_applied) {
                        Ok(sub) => {
                            substitution = substitution.compose(&sub);
                            item_ty = sub.apply(&item_ty_applied);
                        }
                        Err(_) => {
                            errors.push(ConstructorError::UnsupportedExpression {
                                kind: format!(
                                    "list element type mismatch: expected `{item_ty_applied}`, found `{new_ty_applied}`"
                                ),
                                span: *span,
                            });
                        }
                    }
                    substitution = substitution.compose(&item_result.substitution);
                    errors.extend(item_result.errors);
                }

                CheckResult {
                    ty: Type::List(Box::new(item_ty)),
                    substitution,
                    errors,
                }
            }
        }
    }
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
    let tcir = build_tcir_computation_expression(env, target, stmts, span, &dictionary, &check.ty)
        .map_err(|err| vec![err])?;
    Ok(DoElaborationResult {
        expr: core,
        ty: check.ty,
        selected_evidence: Some(dictionary.selected_evidence()),
        tcir: Some(tcir),
    })
}

fn build_tcir_computation_expression(
    env: &TypeEnv,
    target: &ash_parser::surface::DoTarget,
    stmts: &[DoStmt],
    span: Span,
    dictionary: &crate::do_target::DoDictionary,
    ty: &Type,
) -> Result<TcirComputationExpression, ConstructorError> {
    let target_type = tcir_surface_target_type(target);
    let constructor = tcir_target_constructor_expr(env, &target_type, target.span)?;
    let bind_op = tcir_operation_from_dictionary_op(&dictionary.bind_op);
    let return_op = tcir_operation_from_dictionary_op(&dictionary.return_op);
    let evidence_key = tcir_evidence_key(&return_op, &bind_op)
        .unwrap_or_else(|| format!("compiler-prelude::{}", dictionary.target.display()));
    Ok(TcirComputationExpression {
        source_anchor: tcir_source_anchor(span, "do block"),
        target: TcirDoTarget {
            constructor,
            display: render_tcir_surface_type(&target_type),
            source_anchor: tcir_source_anchor(target.span, "do target"),
        },
        evidence: TcirSelectedEvidence {
            interface: "Monad".to_string(),
            evidence_key,
            return_op: return_op.clone(),
            bind_op: bind_op.clone(),
        },
        boundary_level: tcir_boundary_level(dictionary.boundary_level),
        result_type: tcir_type_to_canonical(env, ty, span)?,
        function_artifact: None,
        statements: build_tcir_statements(stmts, &return_op, &bind_op)?,
        explicit_lifts: Vec::new(),
        failure_boundaries: vec![TcirFailureBoundaryProvenance {
            boundary: tcir_boundary_level(dictionary.boundary_level),
            entity: None,
            source_anchor: tcir_source_anchor(span, "do failure boundary"),
            notes: vec![format!(
                "do:{} failure attribution retained at TCIR boundary",
                dictionary.target.display()
            )],
        }],
    })
}

fn build_tcir_statements(
    stmts: &[DoStmt],
    return_op: &TcirOperation,
    bind_op: &TcirOperation,
) -> Result<Vec<TcirStatement>, ConstructorError> {
    stmts
        .iter()
        .enumerate()
        .map(|(index, stmt)| {
            let id = TcirStatementId::new(index as u64);
            let source_anchor = tcir_source_anchor(do_stmt_span(stmt), "do statement");
            let kind = match stmt {
                DoStmt::Let { name, value, .. } => TcirStatementKind::Let {
                    binder: TcirBinder {
                        name: name.to_string(),
                        source_anchor: Some(source_anchor.clone()),
                    },
                    value: Box::new(elaborate_do_expr(value)?),
                },
                DoStmt::Bind { name, value, .. } => TcirStatementKind::Bind {
                    binder: TcirBinder {
                        name: name.to_string(),
                        source_anchor: Some(source_anchor.clone()),
                    },
                    source: Box::new(elaborate_do_expr(value)?),
                    bind_op: Box::new(bind_op.clone()),
                    closure: TcirClosure {
                        source_anchor: source_anchor.clone(),
                        params: vec![TcirBinder {
                            name: name.to_string(),
                            source_anchor: Some(source_anchor.clone()),
                        }],
                        body_statement_ids: ((index + 1)..stmts.len())
                            .map(|statement_index| TcirStatementId::new(statement_index as u64))
                            .collect(),
                    },
                },
                DoStmt::Expr { value, .. } => TcirStatementKind::Let {
                    binder: TcirBinder {
                        name: "_".to_string(),
                        source_anchor: Some(source_anchor.clone()),
                    },
                    value: Box::new(elaborate_do_expr(value)?),
                },
                DoStmt::Return { value, .. } => TcirStatementKind::Return {
                    value: Box::new(elaborate_do_expr(value)?),
                    return_op: Box::new(return_op.clone()),
                },
            };
            Ok(TcirStatement {
                id,
                source_anchor,
                kind,
            })
        })
        .collect()
}

fn tcir_operation_from_dictionary_op(op: &crate::do_target::DoDictionaryOp) -> TcirOperation {
    match op {
        crate::do_target::DoDictionaryOp::Method {
            evidence,
            method,
            params,
            body,
        } => TcirOperation::evidence_method(
            evidence.diagnostic_key(),
            method.clone(),
            params.clone(),
            body.clone(),
            None,
        ),
        crate::do_target::DoDictionaryOp::Intrinsic {
            evidence,
            method,
            shim,
        } => TcirOperation::evidence_intrinsic(
            evidence.diagnostic_key(),
            method.clone(),
            shim.module.clone(),
            shim.name.clone(),
            None,
        ),
        crate::do_target::DoDictionaryOp::Unavailable {
            evidence, method, ..
        } => TcirOperation::evidence_intrinsic(
            evidence.diagnostic_key(),
            method.clone(),
            vec!["__ash_unavailable".to_string()],
            method.clone(),
            None,
        ),
    }
}

fn tcir_evidence_key(return_op: &TcirOperation, bind_op: &TcirOperation) -> Option<String> {
    [return_op, bind_op]
        .into_iter()
        .find_map(|op| match &op.kind {
            TcirOperationKind::EvidenceMethod { evidence_key, .. }
            | TcirOperationKind::EvidenceIntrinsic { evidence_key, .. } => {
                Some(evidence_key.clone())
            }
            _ => None,
        })
}

fn tcir_type_to_canonical(
    env: &TypeEnv,
    ty: &Type,
    span: Span,
) -> Result<CanonicalTypeExpr, ConstructorError> {
    match ty {
        Type::Int => Ok(CanonicalTypeExpr::Primitive("Int".to_string())),
        Type::String => Ok(CanonicalTypeExpr::Primitive("String".to_string())),
        Type::Bool => Ok(CanonicalTypeExpr::Primitive("Bool".to_string())),
        Type::Float => Ok(CanonicalTypeExpr::Primitive("Float".to_string())),
        Type::Null => Ok(CanonicalTypeExpr::Primitive("Null".to_string())),
        Type::Time => Ok(CanonicalTypeExpr::Primitive("Time".to_string())),
        Type::Ref => Ok(CanonicalTypeExpr::Primitive("Ref".to_string())),
        Type::Var(var) => Ok(CanonicalTypeExpr::Var(format!("?{}", var.0))),
        Type::List(item) => Ok(CanonicalTypeExpr::NominalApp {
            origin: tcir_type_origin(env, "List", span)?,
            visible_name: "List".to_string(),
            args: vec![tcir_type_to_canonical(env, item, span)?],
            kind: crate::Kind::Type,
        }),
        Type::Constructor { name, args, kind } => Ok(CanonicalTypeExpr::NominalApp {
            origin: tcir_type_origin(env, &name.name, span)?,
            visible_name: name.name.clone(),
            args: args
                .iter()
                .map(|arg| tcir_type_to_canonical(env, arg, span))
                .collect::<Result<Vec<_>, _>>()?,
            kind: kind.clone(),
        }),
        Type::ConstructorVariableApp {
            constructor,
            args,
            kind,
        } => Ok(CanonicalTypeExpr::ConstructorVariableApp(Box::new(
            ConstructorVariableApp::new(
                ConstructorVariableRef::new(constructor.clone(), crate::Kind::Type, None),
                args.iter()
                    .map(|arg| tcir_type_to_canonical(env, arg, span))
                    .collect::<Result<Vec<_>, _>>()?,
                kind.clone(),
                None,
            ),
        ))),
        Type::Record(fields) => Ok(CanonicalTypeExpr::NominalApp {
            origin: tcir_synthetic_type_decl_id("Record"),
            visible_name: "Record".to_string(),
            args: fields
                .iter()
                .map(|(_, field_ty)| tcir_type_to_canonical(env, field_ty, span))
                .collect::<Result<Vec<_>, _>>()?,
            kind: crate::Kind::Type,
        }),
        Type::Cap { name, .. } => Ok(CanonicalTypeExpr::NominalApp {
            origin: tcir_synthetic_type_decl_id(format!("Capability<{name}>").as_str()),
            visible_name: format!("Capability<{name}>"),
            args: Vec::new(),
            kind: crate::Kind::Type,
        }),
        Type::Fun(params, ret, effect) => tcir_function_type_to_canonical(
            env,
            "Fun",
            params,
            ret,
            Some(format!("{effect:?}")),
            span,
        ),
        Type::Fn(params, ret) => {
            tcir_function_type_to_canonical(env, "Fn", params, ret, None, span)
        }
        Type::Instance { entry_type } => Ok(CanonicalTypeExpr::NominalApp {
            origin: tcir_synthetic_type_decl_id(format!("Instance<{entry_type}>").as_str()),
            visible_name: format!("Instance<{entry_type}>"),
            args: Vec::new(),
            kind: crate::Kind::Type,
        }),
        Type::InstanceAddr { entry_type } => Ok(CanonicalTypeExpr::NominalApp {
            origin: tcir_synthetic_type_decl_id(format!("InstanceAddr<{entry_type}>").as_str()),
            visible_name: format!("InstanceAddr<{entry_type}>"),
            args: Vec::new(),
            kind: crate::Kind::Type,
        }),
        Type::ControlLink { entry_type } => Ok(CanonicalTypeExpr::NominalApp {
            origin: tcir_synthetic_type_decl_id(format!("ControlLink<{entry_type}>").as_str()),
            visible_name: format!("ControlLink<{entry_type}>"),
            args: Vec::new(),
            kind: crate::Kind::Type,
        }),
        Type::Associated {
            interface,
            base,
            name,
        } => tcir_associated_type_to_canonical(env, interface, base, name, span),
    }
}

fn tcir_function_type_to_canonical(
    env: &TypeEnv,
    visible_name: &str,
    params: &[Type],
    ret: &Type,
    effect_tag: Option<String>,
    span: Span,
) -> Result<CanonicalTypeExpr, ConstructorError> {
    let mut args = params
        .iter()
        .map(|param| tcir_type_to_canonical(env, param, span))
        .collect::<Result<Vec<_>, _>>()?;
    args.push(tcir_type_to_canonical(env, ret, span)?);
    if let Some(effect) = effect_tag {
        args.push(CanonicalTypeExpr::Primitive(effect));
    }
    Ok(CanonicalTypeExpr::NominalApp {
        origin: tcir_synthetic_type_decl_id(visible_name),
        visible_name: visible_name.to_string(),
        args,
        kind: crate::Kind::Type,
    })
}

fn tcir_associated_type_to_canonical(
    env: &TypeEnv,
    interface: &str,
    base: &Type,
    member: &str,
    span: Span,
) -> Result<CanonicalTypeExpr, ConstructorError> {
    let interface_identity = env
        .interface_identity_for_name(interface)
        .cloned()
        .unwrap_or_else(|| {
            ash_core::semantic_summary::InterfaceIdentityId::new(
                tcir_synthetic_module_identity(interface),
                interface,
            )
        });
    let member_identity = env
        .associated_member_identity_for_interface_member(interface, member)
        .cloned()
        .unwrap_or_else(|| {
            ash_core::semantic_summary::AssociatedMemberIdentityId::associated_type(
                interface_identity.clone(),
                member,
                vec![interface.to_string(), member.to_string()],
            )
        });
    Ok(CanonicalTypeExpr::Projection {
        interface: interface_identity,
        member: member_identity,
        args: vec![tcir_type_to_canonical(env, base, span)?],
        kind: crate::Kind::Type,
        rigidity: ash_core::type_ir::ProjectionRigidity::Neutral,
    })
}

fn tcir_type_origin(
    env: &TypeEnv,
    visible_name: &str,
    span: Span,
) -> Result<TypeDeclId, ConstructorError> {
    env.type_identity_for_name(visible_name)
        .cloned()
        .or_else(|| {
            env.has_type(visible_name)
                .then(|| tcir_synthetic_type_decl_id(visible_name))
        })
        .ok_or_else(|| ConstructorError::UnsupportedExpression {
            kind: format!("missing canonical type identity for TCIR type {visible_name}"),
            span,
        })
}

fn tcir_target_constructor_expr(
    env: &TypeEnv,
    target_type: &SurfaceType,
    span: Span,
) -> Result<ash_core::type_ir::TypeConstructorExpr, ConstructorError> {
    match env.elaborate_partial_type_constructor(target_type, false) {
        Ok(constructor) => Ok(constructor),
        Err(err) => match target_type {
            SurfaceType::Name(name) if env.has_type(name.as_ref()) => {
                Ok(ash_core::type_ir::TypeConstructorExpr::ConstructorHead(
                    TypeConstructorHeadId::nominal(
                        tcir_synthetic_type_decl_id(name.as_ref()),
                        name.to_string(),
                    ),
                ))
            }
            _ => Err(ConstructorError::UnsupportedExpression {
                kind: format!("failed to preserve TCIR do target constructor: {err}"),
                span,
            }),
        },
    }
}

fn tcir_synthetic_type_decl_id(visible_name: &str) -> TypeDeclId {
    TypeDeclId::ordinary(tcir_synthetic_module_identity(visible_name), visible_name)
}

fn tcir_synthetic_module_identity(reason_key: &str) -> ModuleIdentity {
    ModuleIdentity::new(
        None,
        ModuleId(0),
        vec!["<tcir>".to_string(), reason_key.to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: "TCIR structural type identity fallback".to_string(),
        },
    )
}

fn tcir_surface_target_type(target: &ash_parser::surface::DoTarget) -> SurfaceType {
    if target.args.is_empty() {
        SurfaceType::Name(target.name.to_string().into())
    } else {
        SurfaceType::Constructor {
            name: target.name.to_string().into(),
            args: target.args.clone(),
        }
    }
}

fn render_tcir_surface_type(ty: &SurfaceType) -> String {
    match ty {
        SurfaceType::Name(name) => name.to_string(),
        SurfaceType::Hole { .. } => "_".to_string(),
        SurfaceType::List(item) => format!("[{}]", render_tcir_surface_type(item)),
        SurfaceType::Tuple(items) => format!(
            "({})",
            items
                .iter()
                .map(render_tcir_surface_type)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        SurfaceType::Record(fields) => format!(
            "{{{}}}",
            fields
                .iter()
                .map(|(name, ty)| format!("{name}: {}", render_tcir_surface_type(ty)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        SurfaceType::Capability(name) => format!("capability {name}"),
        SurfaceType::Constructor { name, args } => format!(
            "{}<{}>",
            name,
            args.iter()
                .map(render_tcir_surface_type)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        SurfaceType::Associated { base, name } => {
            format!("{}::{name}", render_tcir_surface_type(base))
        }
        SurfaceType::AssociatedFamilyProjection {
            interface,
            args,
            member,
            ..
        } => format!(
            "<{}<{}>>::{}",
            interface,
            args.iter()
                .map(render_tcir_surface_type)
                .collect::<Vec<_>>()
                .join(", "),
            member
        ),
        SurfaceType::Fn(params, _row, ret) => format!(
            "({}) -> {}",
            params
                .iter()
                .map(render_tcir_surface_type)
                .collect::<Vec<_>>()
                .join(", "),
            render_tcir_surface_type(ret)
        ),
    }
}

fn tcir_boundary_level(level: crate::do_target::DoBoundaryLevel) -> FailureBoundary {
    match level {
        crate::do_target::DoBoundaryLevel::Effectful => FailureBoundary::Effectful,
        crate::do_target::DoBoundaryLevel::Process => FailureBoundary::Process,
        crate::do_target::DoBoundaryLevel::Application => FailureBoundary::Application,
    }
}

fn tcir_source_anchor(span: Span, label: impl Into<String>) -> SourceAnchor {
    SourceAnchor::new(
        SemanticSourceOrigin::Synthetic {
            reason: "typed do elaboration".to_string(),
        },
        Some(CoreSpan {
            start: span.start,
            end: span.end,
        }),
        label,
    )
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
                    if monadic_inner_type(&value_ty, &dictionary).is_some() {
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
                    if let Some(bound_ty) = monadic_inner_type(&value_ty, &dictionary) {
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
        kind: "comprehension MVP requires an explicit process target annotation; target inference is deferred".to_string(),
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
        [DoStmt::Return { value, .. }] => {
            dictionary_call(&dictionary.return_op, vec![elaborate_do_expr(value)?])
        }
        [DoStmt::Let { name, value, .. }, rest @ ..] => Ok(CoreExpr::Let {
            pattern: CorePattern::Variable {
                name: name.to_string(),
                span: ash_core::Span::default(),
            },
            expr: Box::new(elaborate_do_expr(value)?),
            body: Box::new(elaborate_do_stmts(rest, dictionary)?),
            span: ash_core::Span::default(),
        }),
        [DoStmt::Expr { value, .. }, rest @ ..] => Ok(CoreExpr::Let {
            pattern: CorePattern::Variable {
                name: "_".to_string(),
                span: ash_core::Span::default(),
            },
            expr: Box::new(elaborate_do_expr(value)?),
            body: Box::new(elaborate_do_stmts(rest, dictionary)?),
            span: ash_core::Span::default(),
        }),
        [DoStmt::Bind { name, value, .. }, rest @ ..] => {
            let continuation = CoreExpr::FnDef {
                params: vec![(name.to_string(), None)],
                return_type: None,
                body: Box::new(elaborate_do_stmts(rest, dictionary)?),
            };
            dictionary_call(
                &dictionary.bind_op,
                vec![elaborate_do_expr(value)?, continuation],
            )
        }
        [stmt, ..] => Err(ConstructorError::UnsupportedExpression {
            kind: "invalid do statement sequence (return must be last)".to_string(),
            span: do_stmt_span(stmt),
        }),
    }
}

fn classify_requirement(expr: &Expr) -> Result<Requirement, ConstructorError> {
    contract_classifier::classify_requirement(expr).map_err(|err| {
        ConstructorError::UnsupportedExpression {
            kind: format!(
                "unsupported requires contract expression: {}",
                err.requirement_message()
            ),
            span: get_expr_span(expr),
        }
    })
}

fn validate_classified_postcondition(expr: &Expr) -> Result<(), ConstructorError> {
    contract_classifier::classify_postcondition(expr)
        .map(|_| ())
        .map_err(|err| ConstructorError::UnsupportedExpression {
            kind: format!(
                "unsupported ensures contract expression: {}",
                err.postcondition_message()
            ),
            span: get_expr_span(expr),
        })
}

fn contract_intrinsic_misuse_error(
    intrinsic: &ContractIntrinsic,
    reason: String,
    span: Span,
) -> ConstructorError {
    ConstructorError::UnsupportedExpression {
        kind: format!(
            "{} misuse: {} parameter class is non-denotable and contract-only; use inside application contract syntax. {reason}",
            intrinsic.qualified_name,
            intrinsic.parameter_class().as_str(),
        ),
        span,
    }
}

fn contract_intrinsic_context_misuse_error(
    intrinsic: &ContractIntrinsic,
    reason: String,
    span: Span,
) -> ConstructorError {
    ConstructorError::UnsupportedExpression {
        kind: format!(
            "{} misuse: {} parameter class is non-denotable and contract-only outside application contract context; {reason}",
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
    validate_classified_postcondition(expr)
}

fn dictionary_call(
    op: &crate::do_target::DoDictionaryOp,
    arguments: Vec<CoreExpr>,
) -> Result<CoreExpr, ConstructorError> {
    Ok(match op {
        crate::do_target::DoDictionaryOp::Method { params, body, .. } => CoreExpr::FnApply {
            func: Box::new(CoreExpr::FnDef {
                params: params.iter().map(|param| (param.clone(), None)).collect(),
                return_type: None,
                body: Box::new(body.clone()),
            }),
            args: arguments,
        },
        crate::do_target::DoDictionaryOp::Intrinsic { shim, .. } => CoreExpr::Call {
            func: shim.name.clone(),
            module: (!shim.module.is_empty()).then(|| shim.module.join("::")),
            arguments,
        },
        crate::do_target::DoDictionaryOp::Unavailable {
            evidence,
            method,
            span,
        } => {
            return Err(ConstructorError::UnsupportedExpression {
                kind: format!(
                    "selected Monad evidence {} is missing {method} method body or intrinsic shim",
                    evidence.diagnostic_key()
                ),
                span: *span,
            });
        }
    })
}

fn elaborate_do_expr(expr: &Expr) -> Result<CoreExpr, ConstructorError> {
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
    if is_ambient_do_target(target) {
        return check_ambient_do_block(env, stmts, span);
    }

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
    if dictionary.boundary_level == crate::do_target::DoBoundaryLevel::Application {
        block_env.bind_variable("__contract_intrinsic_context", Type::Null);
    }
    let mut substitution = Substitution::new();
    let mut errors: Vec<ConstructorError> = Vec::new();
    let mut return_ty = Type::Null;

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
                let _ = let_span;
            }
            DoStmt::Bind { name, value, span } => {
                let value_result = check_expr(&block_env, value);
                substitution = substitution.compose(&value_result.substitution);
                if !value_result.is_ok() {
                    errors.extend(value_result.errors);
                    continue;
                }

                let value_ty = substitution.apply(&value_result.ty);
                match monadic_inner_type(&value_ty, &dictionary) {
                    Some(bound_ty) => {
                        block_env.bind_variable(name.as_ref(), bound_ty);
                    }
                    None => errors.push(do_bind_type_error(
                        target.name.as_ref(),
                        &dictionary.value_constructor,
                        &value_ty,
                        *span,
                    )),
                }
            }
            DoStmt::Expr { value, .. } => {
                let value_result = check_expr(&block_env, value);
                substitution = substitution.compose(&value_result.substitution);
                if !value_result.is_ok() {
                    errors.extend(value_result.errors);
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
            name: dictionary.value_constructor.clone(),
            args: computation_args_for_do_target(&dictionary, return_ty),
            kind: crate::Kind::Type,
        },
        substitution,
        errors: Vec::new(),
    }
}

fn is_ambient_do_target(target: &ash_parser::surface::DoTarget) -> bool {
    target.name.as_ref() == "__ambient" && target.args.is_empty()
}

fn check_ambient_do_block(env: &TypeEnv, stmts: &[DoStmt], span: Span) -> CheckResult {
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
    let mut substitution = Substitution::new();
    let mut errors: Vec<ConstructorError> = Vec::new();
    let mut return_ty = Type::Null;

    for stmt in stmts {
        match stmt {
            DoStmt::Let { name, value, .. } | DoStmt::Bind { name, value, .. } => {
                let value_result = check_expr(&block_env, value);
                substitution = substitution.compose(&value_result.substitution);
                if !value_result.is_ok() {
                    errors.extend(value_result.errors);
                    continue;
                }
                let value_ty = substitution.apply(&value_result.ty);
                block_env.bind_variable(name.as_ref(), value_ty);
            }
            DoStmt::Expr { value, .. } => {
                let value_result = check_expr(&block_env, value);
                substitution = substitution.compose(&value_result.substitution);
                if !value_result.is_ok() {
                    errors.extend(value_result.errors);
                    continue;
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
        ty: return_ty,
        substitution,
        errors: Vec::new(),
    }
}

fn do_stmt_span(stmt: &DoStmt) -> Span {
    match stmt {
        DoStmt::Let { span, .. }
        | DoStmt::Bind { span, .. }
        | DoStmt::Expr { span, .. }
        | DoStmt::Return { span, .. } => *span,
    }
}

pub(crate) fn monadic_inner_type(
    ty: &Type,
    dictionary: &crate::do_target::DoDictionary,
) -> Option<Type> {
    let Type::Constructor { name, args, .. } = ty else {
        return None;
    };
    if name != &dictionary.value_constructor {
        return None;
    }
    if dictionary.target_args.is_empty() && args.len() == 1 {
        return Some(args[0].clone());
    }

    let hole_index = do_target_hole_index(&dictionary.target_args)?;
    if args.len() != dictionary.target_args.len() {
        return None;
    }
    for (index, target_arg) in dictionary.target_args.iter().enumerate() {
        if index == hole_index {
            continue;
        }
        if !surface_type_arg_matches(target_arg, &args[index]) {
            return None;
        }
    }
    Some(args[hole_index].clone())
}

fn computation_args_for_do_target(
    dictionary: &crate::do_target::DoDictionary,
    return_ty: Type,
) -> Vec<Type> {
    let Some(hole_index) = do_target_hole_index(&dictionary.target_args) else {
        return vec![return_ty];
    };

    dictionary
        .target_args
        .iter()
        .enumerate()
        .map(|(index, target_arg)| {
            if index == hole_index {
                return_ty.clone()
            } else {
                surface_type_arg_to_type(target_arg).unwrap_or(Type::Var(TypeVar::fresh()))
            }
        })
        .collect()
}

fn do_target_hole_index(args: &[ash_parser::surface::Type]) -> Option<usize> {
    args.iter()
        .position(|arg| matches!(arg, ash_parser::surface::Type::Hole { .. }))
}

fn surface_type_arg_matches(target_arg: &ash_parser::surface::Type, actual: &Type) -> bool {
    surface_type_arg_to_type(target_arg).is_some_and(|expected| expected == *actual)
}

fn surface_type_arg_to_type(target_arg: &ash_parser::surface::Type) -> Option<Type> {
    match target_arg {
        ash_parser::surface::Type::Name(name) => Some(match name.as_ref() {
            "Int" => Type::Int,
            "String" => Type::String,
            "Bool" => Type::Bool,
            "Float" => Type::Float,
            "Null" | "Unit" => Type::Null,
            "Time" => Type::Time,
            "Ref" => Type::Ref,
            other => Type::Constructor {
                name: crate::QualifiedName::root(other),
                args: Vec::new(),
                kind: crate::Kind::Type,
            },
        }),
        ash_parser::surface::Type::Constructor { name, args } => {
            let lowered_args = args
                .iter()
                .map(surface_type_arg_to_type)
                .collect::<Option<Vec<_>>>()?;
            Some(Type::Constructor {
                name: crate::QualifiedName::root(name.to_string()),
                args: lowered_args,
                kind: crate::Kind::Type,
            })
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
            | DoStmt::Expr { value, .. }
            | DoStmt::Return { value, .. } => expr_mentions_variable(value, target),
        }),
        _ => false,
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
            " use an explicit lift for cross-constructor sequencing."
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
                    "unadmitted capability binding '{binding_name}' used for operation '{}'; declare it in capability binding metadata",
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
        let arg_has_fatal = arg_result.has_fatal_errors();
        errors.extend(arg_result.errors);
        if arg_has_fatal {
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

    if has_fatal_diagnostics(&errors) {
        return Some(CheckResult {
            ty: Type::Var(TypeVar::fresh()),
            substitution,
            errors,
        });
    }

    Some(CheckResult {
        ty: substitution.apply(&operation.return_type),
        substitution,
        errors,
    })
}

/// Get the span from an expression
fn get_expr_span(expr: &Expr) -> Span {
    match expr {
        Expr::OperatorSection { section } => section.span,
        Expr::Literal(_) => Span::default(),
        Expr::Variable { .. } => Span::default(),
        Expr::FieldAccess { span, .. } => *span,
        Expr::IndexAccess { span, .. } => *span,
        Expr::Unary { span, .. } => *span,
        Expr::Binary { span, .. } => *span,
        Expr::Call { span, .. } => *span,
        Expr::MacroInvocation { invocation } => invocation.span,
        Expr::Match { span, .. } => *span,
        Expr::Policy(_) => Span::default(),
        Expr::IfLet { span, .. } => *span,
        Expr::CheckObligation { span, .. } => *span,
        Expr::Constructor { span, .. } => *span,
        Expr::Record { span, .. } => *span,
        Expr::If { span, .. } => *span,
        Expr::Panic { span, .. } => *span,
        Expr::Fail { span, .. } => *span,
        Expr::WithError { span, .. } => *span,
        Expr::On { span, .. } => *span,
        Expr::HandleWith { span, .. } => *span,
        Expr::Block { span, .. } => *span,
        Expr::FnDef { span, .. } => *span,
        Expr::FnApply { span, .. } => *span,
        Expr::DoBlock { span, .. } => *span,
        Expr::Comprehension { span, .. } => *span,
        Expr::List { span, .. } => *span,
    }
}

/// Check a field-access expression against record types.
fn check_field_access(env: &TypeEnv, base: &Expr, field: &str, span: Span) -> CheckResult {
    let base_result = check_expr(env, base);
    if base_result.has_fatal_errors() {
        return base_result;
    }

    let base_ty = base_result.substitution.apply(&base_result.ty);
    match &base_ty {
        Type::Record(fields) => match fields.iter().find(|(name, _)| name.as_ref() == field) {
            Some((_, field_ty)) => CheckResult {
                ty: base_result.substitution.apply(field_ty),
                substitution: base_result.substitution,
                errors: base_result.errors,
            },
            None => CheckResult::error(ConstructorError::MissingRecordField {
                field: field.to_string(),
                span,
            }),
        },
        Type::Constructor { name, args, .. } => {
            let Some(type_info) = env.lookup_type_info(&name.name) else {
                return CheckResult::error(ConstructorError::NotARecord {
                    field: field.to_string(),
                    actual: base_ty.clone(),
                    span,
                });
            };
            let (params, fields) = match type_info {
                TypeInfo::Struct { params, fields, .. } => (params, fields),
                TypeInfo::Enum {
                    params, variants, ..
                } if matches!(variants.as_slice(), [variant] if variant.name == name.name) => {
                    (params, &variants[0].fields)
                }
                _ => {
                    return CheckResult::error(ConstructorError::NotARecord {
                        field: field.to_string(),
                        actual: base_ty.clone(),
                        span,
                    });
                }
            };
            if params.len() != args.len() {
                return CheckResult::error(ConstructorError::NotARecord {
                    field: field.to_string(),
                    actual: base_ty.clone(),
                    span,
                });
            }
            let field_substitution = Substitution::from_pairs(params.iter().copied().zip(args));
            match fields.iter().find(|(name, _)| name.as_str() == field) {
                Some((_, field_ty)) => CheckResult {
                    ty: base_result
                        .substitution
                        .apply(&field_substitution.apply(field_ty)),
                    substitution: base_result.substitution,
                    errors: base_result.errors,
                },
                None => CheckResult::error(ConstructorError::MissingRecordField {
                    field: field.to_string(),
                    span,
                }),
            }
        }
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
    if operand_result.has_fatal_errors() {
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

    let errors: Vec<ConstructorError> = left_result
        .errors
        .clone()
        .into_iter()
        .chain(right_result.errors.clone())
        .collect();

    if has_fatal_diagnostics(&errors) {
        return CheckResult {
            ty: Type::Var(TypeVar::fresh()),
            substitution: left_result.substitution.compose(&right_result.substitution),
            errors,
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
        errors,
    }
}

fn merge_branch_results(left: CheckResult, right: CheckResult) -> CheckResult {
    let substitution = left.substitution.compose(&right.substitution);
    let mut errors: Vec<ConstructorError> = left.errors.into_iter().chain(right.errors).collect();

    if has_fatal_diagnostics(&errors) {
        return CheckResult {
            ty: Type::Var(TypeVar::fresh()),
            substitution,
            errors,
        };
    }

    match unify(&left.ty, &right.ty) {
        Ok(subst) => CheckResult {
            ty: subst.apply(&left.ty),
            substitution: substitution.compose(&subst),
            errors,
        },
        Err(_) => {
            errors.push(ConstructorError::UnsupportedExpression {
                kind: format!(
                    "branch type mismatch: expected {}, got {}",
                    left.ty, right.ty
                ),
                span: Span::default(),
            });
            CheckResult {
                ty: Type::Var(TypeVar::fresh()),
                substitution,
                errors,
            }
        }
    }
}

fn merge_if_let_branch_results(left: CheckResult, right: CheckResult, span: Span) -> CheckResult {
    let substitution = left.substitution.compose(&right.substitution);
    let mut errors: Vec<ConstructorError> = left.errors.into_iter().chain(right.errors).collect();

    if !errors.iter().all(ConstructorError::is_non_fatal) {
        return CheckResult {
            ty: Type::Var(TypeVar::fresh()),
            substitution,
            errors,
        };
    }

    match unify(&left.ty, &right.ty) {
        Ok(subst) => CheckResult {
            ty: subst.apply(&left.ty),
            substitution: substitution.compose(&subst),
            errors,
        },
        Err(_) => {
            errors.push(ConstructorError::UnsupportedExpression {
                kind: format!(
                    "if let branch type mismatch: expected {}, got {}",
                    left.ty, right.ty
                ),
                span,
            });
            CheckResult {
                ty: Type::Var(TypeVar::fresh()),
                substitution,
                errors,
            }
        }
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
    let failure_payload_ty = direct_failure_payload_type(env, body);
    let mut substitution = body_result.substitution.clone();
    let mut errors = body_result.errors;

    check_with_error_handler_coverage(env, arms, failure_payload_ty.as_ref(), span, &mut errors);

    let pattern_env = pattern_type_env_from(env);
    let payload_canonicalization = failure_payload_ty
        .as_ref()
        .map(|payload_ty| env.canonicalize_type_for_pattern(payload_ty));
    for arm in arms {
        let mut arm_env = env.clone();
        let bindings = match payload_canonicalization.as_ref() {
            Some(PatternCanonicalization::Matchable(canonical)) => {
                crate::check_pattern::check_pattern_with_canonical_type(
                    &pattern_env,
                    &arm.pattern,
                    canonical,
                )
            }
            _ => {
                let payload_ty = failure_payload_ty
                    .clone()
                    .unwrap_or_else(|| Type::Var(TypeVar::fresh()));
                crate::check_pattern::check_pattern(&pattern_env, &arm.pattern, &payload_ty)
            }
        };
        match bindings {
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

fn direct_failure_payload_type(env: &TypeEnv, body: &Expr) -> Option<Type> {
    let Expr::Fail { payload, .. } = body else {
        return None;
    };

    let payload_result = check_expr(env, payload);
    payload_result
        .is_ok()
        .then(|| payload_result.substitution.apply(&payload_result.ty))
}

fn check_with_error_handler_coverage(
    env: &TypeEnv,
    arms: &[MatchArm],
    failure_payload_ty: Option<&Type>,
    span: Span,
    errors: &mut Vec<ConstructorError>,
) {
    let patterns = arms
        .iter()
        .filter_map(|arm| lower_pattern(&arm.pattern).ok())
        .collect::<Vec<_>>();

    if patterns.iter().any(is_universal_core_pattern) {
        return;
    }

    if patterns.is_empty() && arms.is_empty() && failure_payload_ty.is_none() {
        errors.push(ConstructorError::WithErrorHandlerCoverageDeferred {
            payload_type: "<unavailable>".to_string(),
            reason: "handler has no arms; add a wildcard/default arm or provide a known closed failure payload type".to_string(),
            span,
        });
        return;
    }

    let Some(payload_ty) = failure_payload_ty else {
        if !patterns.is_empty() {
            errors.push(ConstructorError::WithErrorHandlerCoverageDeferred {
                payload_type: "<unavailable>".to_string(),
                reason: "failure payload type is not tracked for this with_error body; constructor-specific handler coverage cannot be proven in this phase, so add a wildcard/default arm or handle a directly typed fail payload".to_string(),
                span,
            });
        }
        return;
    };

    match check_match_exhaustive(env, &patterns, payload_ty) {
        MatchCoverage::Covered => {}
        MatchCoverage::Missing(witnesses) => {
            errors.push(ConstructorError::NonExhaustiveWithErrorHandler {
                payload_type: payload_ty.to_string(),
                missing: format_missing_witnesses(&witnesses),
                span,
            });
        }
        MatchCoverage::Blocked { reason, .. } => {
            errors.push(ConstructorError::WithErrorHandlerCoverageDeferred {
                payload_type: payload_ty.to_string(),
                reason: format!(
                    "failure payload constructor universe is unavailable for with_error handler coverage: {reason:?}; add a wildcard/default arm"
                ),
                span,
            });
        }
        MatchCoverage::Unsupported { reason, .. } => {
            errors.push(ConstructorError::WithErrorHandlerCoverageDeferred {
                payload_type: payload_ty.to_string(),
                reason: format!("{reason}; add a wildcard/default arm"),
                span,
            });
        }
    }
}

fn is_universal_core_pattern(pattern: &CorePattern) -> bool {
    matches!(
        pattern,
        CorePattern::Wildcard | CorePattern::Variable { .. }
    )
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
                    .map(|(field, pattern)| {
                        if fields.len() == 1 && matches!(pattern, CorePattern::Wildcard) {
                            field.clone()
                        } else {
                            format_pattern_witness(pattern)
                        }
                    })
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

fn collect_top_level_variant_pattern_names(arms: &[MatchArm]) -> Vec<String> {
    let mut names = Vec::new();
    for arm in arms {
        if let Pattern::Variant { name, .. } = &arm.pattern {
            names.push(name.to_string());
        }
    }
    names.sort();
    names.dedup();
    names
}

fn format_pattern_canonicalization_blocked(
    source_type: &Type,
    reason: &PatternCanonicalizationBlockedReason,
    visible_constructors: &[String],
) -> String {
    let visible = if visible_constructors.is_empty() {
        "none".to_string()
    } else {
        visible_constructors.join(", ")
    };
    match reason {
        PatternCanonicalizationBlockedReason::RigidAssociatedProjection { interface, member } => {
            format!(
                "pattern canonicalization blocked for match on {source_type}: rigid associated projection {interface}::{member}; visible pattern constructors: {visible}"
            )
        }
        PatternCanonicalizationBlockedReason::UnknownConstructorUniverse { name } => {
            format!(
                "pattern canonicalization blocked for match on {source_type}: canonical constructor universe for {name} is unavailable; visible pattern constructors: {visible}"
            )
        }
        PatternCanonicalizationBlockedReason::UnknownType { name } => {
            format!(
                "pattern canonicalization blocked for match on {source_type}: unknown canonical type {name}; visible pattern constructors: {visible}"
            )
        }
        PatternCanonicalizationBlockedReason::ConstructorVariableApplication { constructor } => {
            format!(
                "pattern canonicalization blocked for match on {source_type}: constructor variable application {constructor}; visible pattern constructors: {visible}"
            )
        }
        other => {
            format!(
                "pattern canonicalization blocked for match on {source_type}: {other:?}; visible pattern constructors: {visible}"
            )
        }
    }
}

fn check_match(env: &TypeEnv, scrutinee: &Expr, arms: &[MatchArm]) -> CheckResult {
    let scrutinee_result = check_expr(env, scrutinee);
    let mut errors: Vec<ConstructorError> = scrutinee_result.errors.clone();
    let scrutinee_ty = scrutinee_result.substitution.apply(&scrutinee_result.ty);
    let visible_variant_patterns = collect_top_level_variant_pattern_names(arms);
    let mut pattern_canonicalization_blocked = false;
    let pattern_canonicalization = pattern_canonicalization_for_scrutinee(env, &scrutinee_ty);
    let canonical_scrutinee = match &pattern_canonicalization {
        PatternCanonicalization::Matchable(canonical) => Some(canonical),
        PatternCanonicalization::Blocked { .. } => None,
    };

    let patterns: Vec<CorePattern> = arms
        .iter()
        .filter_map(|arm| lower_pattern(&arm.pattern).ok())
        .collect();
    let coverage_error = match check_match_exhaustive_with_canonicalization(
        env,
        &patterns,
        &scrutinee_ty,
        &pattern_canonicalization,
    ) {
        MatchCoverage::Covered => None,
        MatchCoverage::Missing(witnesses) => Some(ConstructorError::NonExhaustiveMatch {
            scrutinee_type: match canonical_scrutinee {
                Some(canonical) => canonical.canonical_name.name.clone(),
                None => scrutinee_ty.to_string(),
            },
            missing: format_missing_witnesses(&witnesses),
            span: Span::default(),
        }),
        MatchCoverage::Blocked {
            source_type,
            reason,
        } => {
            pattern_canonicalization_blocked = true;
            Some(ConstructorError::UnsupportedExpression {
                kind: format_pattern_canonicalization_blocked(
                    &source_type,
                    &reason,
                    &visible_variant_patterns,
                ),
                span: Span::default(),
            })
        }
        MatchCoverage::Unsupported {
            scrutinee_type,
            reason,
        } => Some(ConstructorError::NonExhaustiveMatch {
            scrutinee_type: scrutinee_type.to_string(),
            missing: reason,
            span: Span::default(),
        }),
    };

    let pattern_env = pattern_type_env_from(env);
    let mut arm_merged: Option<CheckResult> = None;
    for arm in arms {
        let mut arm_env = env.clone();
        if !pattern_canonicalization_blocked {
            let bindings = match canonical_scrutinee.as_ref() {
                Some(canonical) => crate::check_pattern::check_pattern_with_canonical_type(
                    &pattern_env,
                    &arm.pattern,
                    canonical,
                ),
                None => {
                    crate::check_pattern::check_pattern(&pattern_env, &arm.pattern, &scrutinee_ty)
                }
            };
            match bindings {
                Ok(bindings) => {
                    for (name, ty) in bindings {
                        arm_env.bind_variable(&name, ty);
                    }
                }
                Err(error) => {
                    errors.push(ConstructorError::UnsupportedExpression {
                        kind: format!("match arm pattern type error: {error}"),
                        span: arm.span,
                    });
                }
            };
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

    if let Some(coverage_error) = coverage_error {
        errors.push(coverage_error);
    }
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
    if let Some(newtype) = env.nominal_newtype_for_constructor(constructor_name) {
        return check_nominal_newtype_constructor(env, constructor_name, fields, payload, newtype);
    }

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

/// Check the sole tuple constructor of a local nominal newtype.
///
/// The wrapper never enters `TypeInfo` or transparent-alias normalization: the
/// checked payload is compared with the recorded representation only and the
/// result is the separately registered nominal type identity.
fn check_nominal_newtype_constructor(
    env: &TypeEnv,
    constructor_name: &str,
    fields: &[(Box<str>, Expr)],
    payload: &ConstructorPayload,
    newtype: &crate::type_env::NominalNewtype,
) -> CheckResult {
    let Some(representation) = newtype.representation() else {
        return CheckResult::error(ConstructorError::UnsupportedExpression {
            kind: format!("newtype constructor '{constructor_name}' has no checked representation"),
            span: Span::default(),
        });
    };
    let tuple_items = match payload {
        ConstructorPayload::Tuple(items) => items,
        _ => {
            return CheckResult::error(ConstructorError::TupleArityMismatch {
                constructor: constructor_name.to_string(),
                expected: 1,
                actual: fields.len(),
                span: Span::default(),
            });
        }
    };
    if tuple_items.len() != 1 || fields.len() != 1 {
        return CheckResult::error(ConstructorError::TupleArityMismatch {
            constructor: constructor_name.to_string(),
            expected: 1,
            actual: tuple_items.len(),
            span: Span::default(),
        });
    }

    let payload_result = check_expr(env, &tuple_items[0]);
    let payload_ty = payload_result.substitution.apply(&payload_result.ty);
    let mut errors = payload_result.errors;
    let substitution = payload_result.substitution;
    if crate::types::unify(representation, &payload_ty).is_err() {
        errors.push(ConstructorError::UnsupportedExpression {
            kind: format!(
                "newtype constructor '{constructor_name}' expects {representation} but received {payload_ty}"
            ),
            span: Span::default(),
        });
    }

    CheckResult {
        ty: Type::Constructor {
            name: crate::QualifiedName::root(newtype.type_name()),
            args: Vec::new(),
            kind: crate::Kind::Type,
        },
        substitution,
        errors,
    }
}

fn check_record_expr(env: &TypeEnv, fields: &[(Box<str>, Expr)]) -> CheckResult {
    let mut typed_fields = Vec::with_capacity(fields.len());
    let mut substitution = Substitution::new();
    let mut errors = Vec::new();

    for (name, expr) in fields {
        let result = check_expr(env, expr);
        substitution = substitution.compose(&result.substitution);
        errors.extend(result.errors);
        typed_fields.push((name.clone(), substitution.apply(&result.ty)));
    }

    CheckResult {
        ty: Type::Record(typed_fields),
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
        let field_ty = field_result.substitution.apply(&field_result.ty);
        match unify(&expected_ty_subst, &field_ty) {
            Ok(sub) => {
                *substitution = substitution.compose(&sub);
            }
            Err(_) => errors.push(ConstructorError::TupleFieldTypeMismatch {
                constructor: constructor_name.to_string(),
                position: index,
                expected: expected_ty.to_string(),
                actual: field_ty.to_string(),
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
            let field_ty = field_result.substitution.apply(&field_result.ty);
            match unify(&expected_ty_subst, &field_ty) {
                Ok(sub) => {
                    *substitution = substitution.compose(&sub);
                }
                Err(_) => {
                    errors.push(ConstructorError::FieldTypeMismatch {
                        constructor: constructor_name.to_string(),
                        field: field_name.to_string(),
                        expected: expected_ty.to_string(),
                        actual: field_ty.to_string(),
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

    #[test]
    fn task_2000_diagnostics_do_not_fabricate_removed_act_or_proc_carriers() {
        let env = TypeEnv::with_builtin_types();
        for module in ["act", "proc"] {
            let expr = Expr::Call {
                func: "unit".into(),
                module: Some(module.into()),
                args: vec![Expr::Literal(Literal::Int(1))],
                span: Span::default(),
            };

            assert!(
                !check_expr(&env, &expr).is_ok(),
                "removed {module}::unit must remain rejected"
            );
            assert!(
                do_notation::diagnostic_expr_type(&env, &expr, &Substitution::new()).is_none(),
                "diagnostic traversal must not fabricate a removed carrier for {module}::unit"
            );
        }
    }

    #[test]
    fn task_2000_direct_source_invoke_is_rejected_without_tower_type_leakage() {
        let mut env = TypeEnv::with_builtin_types();
        let value_ty = Type::Constructor {
            name: crate::QualifiedName::root("Value"),
            args: vec![],
            kind: crate::Kind::Type,
        };
        env.bind_variable("invoke_args", Type::List(Box::new(value_ty)));
        let expr = Expr::Call {
            func: "invoke".into(),
            module: None,
            args: vec![
                Expr::Literal(Literal::String("sensor".into())),
                Expr::Literal(Literal::String("read".into())),
                Expr::Variable {
                    name: "invoke_args".into(),
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        };

        let result = check_expr(&env, &expr);
        let message = format!("{:?}", result.errors);

        assert!(!result.is_ok(), "direct source invoke must fail closed");
        assert!(
            message.contains("admitted named interface or binding operation"),
            "{message}"
        );
        assert!(!message.contains("Act"), "{message}");
        assert!(!message.contains("Proc"), "{message}");
    }

    #[test]
    fn task_2000_missing_comprehension_target_guidance_is_generic_and_process_oriented() {
        let expr = Expr::Comprehension {
            result: Box::new(Expr::Literal(Literal::Int(1))),
            qualifiers: Vec::new(),
            target: None,
            span: Span::default(),
        };
        let result = check_expr(&TypeEnv::with_builtin_types(), &expr);
        let message = format!("{:?}", result.errors);

        assert!(message.contains("process"), "{message}");
        assert!(!message.contains("Act"), "{message}");
        assert!(!message.contains("Proc"), "{message}");
    }

    #[test]
    fn task_2000_do_mismatch_guidance_does_not_recommend_removed_proc_lift() {
        let actual = Type::Constructor {
            name: crate::QualifiedName::root("Act"),
            args: vec![Type::Int],
            kind: crate::Kind::Type,
        };
        let error = do_bind_type_error(
            "Proc",
            &crate::QualifiedName::root("Proc"),
            &actual,
            Span::default(),
        );
        let message = format!("{error}");

        assert!(!message.contains("proc::from_act"), "{message}");
        assert!(!message.contains("Act-to-Proc"), "{message}");
        assert!(message.contains("explicit lift"), "{message}");
    }

    #[test]
    fn task_2000_ambient_do_remains_a_valid_canonical_control() {
        let expr = Expr::DoBlock {
            target: ash_parser::surface::DoTarget {
                name: "__ambient".into(),
                args: Vec::new(),
                span: Span::default(),
            },
            stmts: vec![DoStmt::Return {
                value: Box::new(Expr::Literal(Literal::Int(1))),
                span: Span::default(),
            }],
            span: Span::default(),
        };
        let result = check_expr(&TypeEnv::with_builtin_types(), &expr);

        assert!(result.is_ok(), "ambient do failed: {result:?}");
        assert_eq!(result.ty, Type::Int);
    }

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

        // match Some { value: 0 } { Some { value: x } => x }
        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Constructor {
                name: "Some".into(),
                fields: vec![("value".into(), Expr::Literal(Literal::Int(0)))],
                payload: ConstructorPayload::Record(vec![(
                    "value".into(),
                    Expr::Literal(Literal::Int(0)),
                )]),
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

        // match Some { value: 0 } {
        //   Some { value: _ } => 42,
        //   None => 0
        // }
        // Note: Using literal body instead of variable to avoid needing
        // pattern variable binding in type environment (separate feature)
        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Constructor {
                name: "Some".into(),
                fields: vec![("value".into(), Expr::Literal(Literal::Int(0)))],
                payload: ConstructorPayload::Record(vec![(
                    "value".into(),
                    Expr::Literal(Literal::Int(0)),
                )]),
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

    /// Test 1 (TASK-558): FnDef in a pure context -> Type::Fn
    #[test]
    fn task558_fnapp_in_pure_context_yields_type_fn() {
        let env = TypeEnv::with_builtin_types();
        assert!(
            env.ambient_effect().is_none(),
            "pure env should have no ambient effect"
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

    /// Test 2 (TASK-558/TASK-959): FnDef remains a pure Type::Fn in an ambient effect context.
    #[test]
    fn task558_fndef_in_ambient_effect_context_yields_type_fn() {
        let mut env = TypeEnv::with_builtin_types();
        env.set_ambient_effect(ash_core::Effect::Operational);

        let expr = make_fn_def_expr("x");
        let result = check_expr(&env, &expr);

        assert!(result.is_ok(), "expected success, got {:?}", result.errors);
        match result.ty {
            Type::Fn(params, _ret) => assert_eq!(params.len(), 1),
            other => panic!("expected Type::Fn, got {other:?}"),
        }
    }

    /// Test 2b (TASK-558/TASK-959): Epistemic ambient context still yields Type::Fn.
    #[test]
    fn task558_fndef_in_epistemic_ambient_effect_yields_type_fn() {
        let mut env = TypeEnv::with_builtin_types();
        env.set_ambient_effect(ash_core::Effect::Epistemic);

        let expr = make_fn_def_expr("x");
        let result = check_expr(&env, &expr);

        assert!(result.is_ok(), "expected success, got {:?}", result.errors);
        match result.ty {
            Type::Fn(params, _ret) => assert_eq!(params.len(), 1),
            other => panic!("expected Type::Fn, got {other:?}"),
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

    /// Test 4 (TASK-558/TASK-959): FnApply accepts pure closure syntax in ambient effect contexts.
    #[test]
    fn task558_pass_ambient_effect_context_fn_to_fn_parameter_is_accepted() {
        // Build env where `apply` is bound as  Fn([Fn([Int], Int)], Int)
        // i.e. apply : (Int -> Int) -> Int
        let mut env = TypeEnv::with_builtin_types();
        let apply_ty = Type::Fn(
            vec![Type::Fn(vec![Type::Int], Box::new(Type::Int))],
            Box::new(Type::Int),
        );
        env.bind_variable("apply", apply_ty);

        // In ambient effect context, pure closure syntax still gets type Fn([Int], Int).
        env.set_ambient_effect(ash_core::Effect::Operational);
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
        // SPEC-072 keeps pure closure syntax at the Pure stratum, so this succeeds.
        assert!(
            result.is_ok(),
            "passing Type::Fn where Type::Fn expected should succeed, got errors {:?}",
            result.errors
        );
        assert_eq!(result.ty, Type::Int);
    }

    /// Test 5 (TASK-558): ambient_effect propagates into child scopes (extend()).
    #[test]
    fn task558_ambient_effect_propagates_to_child_env() {
        let mut env = TypeEnv::with_builtin_types();
        env.set_ambient_effect(ash_core::Effect::Deliberative);
        let child = env.extend();
        assert_eq!(child.ambient_effect(), Some(ash_core::Effect::Deliberative));
    }

    /// Escape case 1 (TASK-558/TASK-959): pure closure syntax remains Type::Fn in ambient effect contexts.
    #[test]
    fn task558_return_ambient_effect_context_closure_where_fn_expected_is_accepted() {
        // In an ambient effect context, pure closure syntax types as Type::Fn([Var], Var).
        let mut env = TypeEnv::with_builtin_types();
        env.set_ambient_effect(ash_core::Effect::Operational);

        let expr = make_fn_def_expr("x");
        let result = check_expr(&env, &expr);

        // Verify the closure typed as Fn in ambient effect context.
        assert!(
            matches!(&result.ty, Type::Fn(..)),
            "FnDef in ambient effect context must produce Type::Fn, got {:?}",
            result.ty
        );

        // Now attempt to unify with a pure Fn type — succeeds after TASK-959.
        let pure_fn = Type::Fn(
            vec![Type::Var(TypeVar::fresh())],
            Box::new(Type::Var(TypeVar::fresh())),
        );
        let unify_result = unify(&result.ty, &pure_fn);
        assert!(
            unify_result.is_ok(),
            "pure closure syntax should unify with Type::Fn in ambient effect/profile contexts"
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
