//! Do-notation diagnostics and legacy act-block checking.

use super::*;

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

pub(super) fn collect_do_notation_diagnostics(
    env: &TypeEnv,
    expr: &Expr,
    diagnostics: &mut Vec<String>,
) {
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
                                if monadic_inner_type(&value_ty, &dictionary).is_some() {
                                    diagnostics.push(format!(
                                        "do:{} let `{name}` binds monadic value {value_ty} without sequencing; use `{name} <- ...` to bind the produced value, or keep `let` only when you intentionally want the computation value itself",
                                        target.name.as_ref()
                                    ));
                                }
                                block_env.bind_variable(name.as_ref(), value_ty);
                            }
                            collect_do_notation_diagnostics(&block_env, value, diagnostics);
                        }
                        DoStmt::Bind { value, .. }
                        | DoStmt::Expr { value, .. }
                        | DoStmt::Return { value, .. } => {
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
        Expr::MacroInvocation { .. } => {}
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
        Expr::Record { fields, .. } => {
            for (_, value) in fields {
                collect_do_notation_diagnostics(env, value, diagnostics);
            }
        }
        Expr::WithError { body, arms, .. } => {
            collect_do_notation_diagnostics(env, body, diagnostics);
            for arm in arms {
                collect_do_notation_diagnostics(env, &arm.body, diagnostics);
            }
        }
        Expr::Policy(policy) => collect_policy_do_notation_diagnostics(env, policy, diagnostics),
        Expr::OperatorSection { section } => {
            if let Some(left) = &section.left {
                collect_do_notation_diagnostics(env, left, diagnostics);
            }
            if let Some(right) = &section.right {
                collect_do_notation_diagnostics(env, right, diagnostics);
            }
        }
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

pub(super) fn diagnostic_expr_type(
    env: &TypeEnv,
    expr: &Expr,
    substitution: &Substitution,
) -> Option<Type> {
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

pub(super) fn check_legacy_act_block(env: &TypeEnv, stmts: &[ActStmt], span: Span) -> CheckResult {
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
