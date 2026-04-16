//! Expression type checking
//!
//! Provides type checking for expressions, including constructor expressions.

use crate::error::ConstructorError;
use crate::exhaustiveness::{Coverage, check_exhaustive};
use crate::type_env::{TypeEnv, TypeInfo, VariantIndex, VariantInfo};
use crate::types::{Substitution, Type, TypeVar, unify};
use ash_core::adt::{VariantPayloadShape, tuple_field_name};
use ash_core::ast::{Pattern as CorePattern, TypeBody, TypeDef};
use ash_parser::lower_pattern;
use ash_parser::surface::ConstructorPayload;
use ash_parser::surface::{BinaryOp, Expr, Literal, MatchArm, Pattern as SurfacePattern, UnaryOp};
use ash_parser::token::Span;
use std::collections::HashSet;

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
        Expr::FieldAccess { base, field, span } => {
            // Field access is not yet fully implemented
            // Check the base expression first, then report unsupported
            let base_result = check_expr(env, base);
            if !base_result.is_ok() {
                return base_result;
            }
            CheckResult::error(ConstructorError::UnsupportedExpression {
                kind: format!("FieldAccess (.{field})"),
                span: *span,
            })
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
                                            "Call ({qualified_name}): expected {} args, got {}",
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
                            "FnApply: arity mismatch — expected {} args, got {} for type {func_ty}",
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
    }
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
        Expr::Block { span, .. } => *span,
        Expr::FnDef { span, .. } => *span,
        Expr::FnApply { span, .. } => *span,
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

fn resolve_enum_type_def_for_match<'a>(
    env: &'a TypeEnv,
    scrutinee: &Expr,
    arms: &[MatchArm],
) -> Option<&'a TypeDef> {
    if let Expr::Constructor { name, .. } = scrutinee
        && let Some((type_name, _)) = env.lookup_constructor(name.as_ref())
        && let Some(def) = env.lookup_type(type_name.as_str())
        && matches!(&def.body, TypeBody::Enum(_))
    {
        return Some(def);
    }
    for arm in arms {
        if let SurfacePattern::Variant { name, .. } = &arm.pattern
            && let Some((type_name, _)) = env.lookup_constructor(name.as_ref())
            && let Some(def) = env.lookup_type(type_name.as_str())
            && matches!(&def.body, TypeBody::Enum(_))
        {
            return Some(def);
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
            });
            return;
        }
    };

    if tuple_items.len() != variant_def.fields.len() || fields.len() != variant_def.fields.len() {
        errors.push(ConstructorError::TupleArityMismatch {
            constructor: constructor_name.to_string(),
            expected: variant_def.fields.len(),
            actual: tuple_items.len(),
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
            });
            continue;
        }

        if expected_name != &tuple_field_name(index) {
            errors.push(ConstructorError::TupleArityMismatch {
                constructor: constructor_name.to_string(),
                expected: variant_def.fields.len(),
                actual: tuple_items.len(),
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
            });
        }
    }

    for provided in &provided_fields {
        if !expected_fields.contains(*provided) {
            errors.push(ConstructorError::UnknownField {
                constructor: constructor_name.to_string(),
                field: provided.to_string(),
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
            ConstructorError::UnknownConstructor(_)
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
            ConstructorError::MissingField { constructor, field }
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
            ConstructorError::UnknownField { constructor, field }
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
        let err = ConstructorError::UnknownConstructor("Foo".to_string());
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
