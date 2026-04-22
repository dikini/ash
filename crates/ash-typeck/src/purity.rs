//! Purity checking for Ash `fn` bodies.
//!
//! A pure function body may only contain value-level constructs. Policy expressions,
//! obligation checks, capability-typed calls, and unresolved calls are rejected.

use crate::check_expr::check_expr;
use crate::type_env::TypeEnv;
use crate::types::Type;
use ash_parser::surface::{ActStmt, BlockStmt, Expr};
use ash_parser::token::Span;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct PurityError {
    pub kind: PurityViolation,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PurityViolation {
    PolicyExpression,
    CheckObligation,
    UnresolvedCall { callee: String },
    NonPureCall { callee: String, found: String },
    InvalidInterfaceMethodCall { interface: String, method: String },
    ActBlockInPureContext,
    InvokeInPureContext,
}

impl fmt::Display for PurityViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PurityViolation::PolicyExpression => {
                write!(f, "policy expression not allowed in pure function")
            }
            PurityViolation::CheckObligation => {
                write!(f, "obligation check not allowed in pure function")
            }
            PurityViolation::UnresolvedCall { callee } => {
                write!(
                    f,
                    "call to unresolved function '{}' not allowed in pure function",
                    callee
                )
            }
            PurityViolation::NonPureCall { callee, found } => {
                write!(f, "call to '{}' is not pure; found {}", callee, found)
            }
            PurityViolation::InvalidInterfaceMethodCall { interface, method } => {
                write!(
                    f,
                    "interface method call {}::{} is not valid in a pure function body",
                    interface, method
                )
            }
            PurityViolation::ActBlockInPureContext => {
                write!(f, "act block not allowed in pure function body")
            }
            PurityViolation::InvokeInPureContext => {
                write!(f, "invoke call not allowed in pure function body")
            }
        }
    }
}

impl fmt::Display for PurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl std::error::Error for PurityError {}

/// Check whether an expression is pure enough for a function body.
///
/// When `allow_effects` is `true`, `act {}` blocks and `invoke(...)` calls
/// are permitted (the function declares `Act<T>` return type). When `false`,
/// both are rejected as purity violations.
pub fn check_purity(
    env: &TypeEnv,
    expr: &Expr,
    allow_effects: bool,
) -> Result<(), Vec<PurityError>> {
    let mut errors = Vec::new();
    check_purity_recursive(env, expr, allow_effects, &mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_purity_recursive(
    env: &TypeEnv,
    expr: &Expr,
    allow_effects: bool,
    errors: &mut Vec<PurityError>,
) {
    match expr {
        Expr::Policy(_) => {
            errors.push(PurityError {
                kind: PurityViolation::PolicyExpression,
                span: Span::default(),
            });
        }
        Expr::CheckObligation { span, .. } => {
            errors.push(PurityError {
                kind: PurityViolation::CheckObligation,
                span: *span,
            });
        }
        Expr::Literal(_) | Expr::Variable { .. } | Expr::Panic { .. } => {}
        Expr::FieldAccess { base, .. } => {
            check_purity_recursive(env, base, allow_effects, errors);
        }
        Expr::IndexAccess { base, index, .. } => {
            check_purity_recursive(env, base, allow_effects, errors);
            check_purity_recursive(env, index, allow_effects, errors);
        }
        Expr::Unary { operand, .. } => {
            check_purity_recursive(env, operand, allow_effects, errors);
        }
        Expr::Binary { left, right, .. } => {
            check_purity_recursive(env, left, allow_effects, errors);
            check_purity_recursive(env, right, allow_effects, errors);
        }
        Expr::Call {
            func,
            module,
            args,
            span,
        } => {
            // Reject invoke() in pure contexts
            if !allow_effects && module.is_none() && func.as_ref() == "invoke" {
                errors.push(PurityError {
                    kind: PurityViolation::InvokeInPureContext,
                    span: *span,
                });
                return;
            }

            for arg in args {
                check_purity_recursive(env, arg, allow_effects, errors);
            }

            // Check if this is an interface method call
            if let Some(module_name) = module.as_deref()
                && env.has_interface(module_name)
            {
                let mut arg_types = Vec::new();
                let mut subst = crate::types::Substitution::new();
                for arg in args {
                    let arg_result = check_expr(env, arg);
                    if arg_result.is_ok() {
                        subst = subst.compose(&arg_result.substitution);
                        arg_types.push(subst.apply(&arg_result.ty));
                    }
                }

                if env
                    .resolve_interface_method_call(module_name, func.as_ref(), &arg_types)
                    .is_err()
                {
                    errors.push(PurityError {
                        kind: PurityViolation::InvalidInterfaceMethodCall {
                            interface: module_name.to_string(),
                            method: func.to_string(),
                        },
                        span: *span,
                    });
                }
                return;
            }

            let callee = qualified_callee_name(module.as_deref(), func.as_ref());
            let Some(callee_ty) = env.lookup_call_target(module.as_deref(), func.as_ref()) else {
                errors.push(PurityError {
                    kind: PurityViolation::UnresolvedCall { callee },
                    span: *span,
                });
                return;
            };

            if !matches!(callee_ty, Type::Fn(..)) {
                errors.push(PurityError {
                    kind: PurityViolation::NonPureCall {
                        callee,
                        found: callee_ty.to_string(),
                    },
                    span: *span,
                });
            }
        }
        Expr::Match {
            scrutinee, arms, ..
        } => {
            check_purity_recursive(env, scrutinee, allow_effects, errors);
            for arm in arms {
                let mut arm_env = env.clone();
                let scrutinee_result = check_expr(env, scrutinee);
                if scrutinee_result.is_ok() {
                    crate::bind_pattern_variables(
                        &mut arm_env,
                        &arm.pattern,
                        &scrutinee_result.substitution.apply(&scrutinee_result.ty),
                    );
                }
                check_purity_recursive(&arm_env, arm.body.as_ref(), allow_effects, errors);
            }
        }
        Expr::IfLet {
            pattern,
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            check_purity_recursive(env, expr, allow_effects, errors);
            let mut then_env = env.clone();
            let matched = check_expr(env, expr);
            if matched.is_ok() {
                crate::bind_pattern_variables(
                    &mut then_env,
                    pattern,
                    &matched.substitution.apply(&matched.ty),
                );
            }
            check_purity_recursive(&then_env, then_branch, allow_effects, errors);
            check_purity_recursive(env, else_branch, allow_effects, errors);
        }
        Expr::Constructor {
            fields, payload, ..
        } => {
            for (_, field_expr) in fields {
                check_purity_recursive(env, field_expr, allow_effects, errors);
            }
            match payload {
                ash_parser::surface::ConstructorPayload::Unit => {}
                ash_parser::surface::ConstructorPayload::Record(rec_fields) => {
                    for (_, field_expr) in rec_fields {
                        check_purity_recursive(env, field_expr, allow_effects, errors);
                    }
                }
                ash_parser::surface::ConstructorPayload::Tuple(elems) => {
                    for elem in elems {
                        check_purity_recursive(env, elem, allow_effects, errors);
                    }
                }
            }
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            check_purity_recursive(env, condition, allow_effects, errors);
            check_purity_recursive(env, then_branch, allow_effects, errors);
            if let Some(else_expr) = else_branch {
                check_purity_recursive(env, else_expr, allow_effects, errors);
            }
        }
        Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            let mut block_env = env.extend();
            for stmt in statements {
                let BlockStmt::Let { pattern, expr, .. } = stmt;
                check_purity_recursive(&block_env, expr, allow_effects, errors);
                let expr_result = check_expr(&block_env, expr);
                if expr_result.is_ok() {
                    crate::bind_pattern_variables(
                        &mut block_env,
                        pattern,
                        &expr_result.substitution.apply(&expr_result.ty),
                    );
                }
            }
            if let Some(tail) = tail_expr {
                check_purity_recursive(&block_env, tail, allow_effects, errors);
            }
        }
        Expr::FnDef { params, body, .. } => {
            let mut fn_env = env.extend();
            for (name, _ty) in params {
                fn_env.bind_variable(name.as_ref(), Type::Var(crate::types::TypeVar::fresh()));
            }
            check_purity_recursive(&fn_env, body, allow_effects, errors);
        }
        Expr::FnApply { func, args, .. } => {
            check_purity_recursive(env, func, allow_effects, errors);
            for arg in args {
                check_purity_recursive(env, arg, allow_effects, errors);
            }
        }
        // TASK-680: Purity enforcement for act {} blocks
        Expr::ActBlock { stmts, span, .. } => {
            if !allow_effects {
                errors.push(PurityError {
                    kind: PurityViolation::ActBlockInPureContext,
                    span: *span,
                });
                return;
            }
            for stmt in stmts {
                let value = match stmt {
                    ActStmt::Bind { value, .. } => value,
                    ActStmt::Return { value, .. } => value,
                };
                check_purity_recursive(env, value, allow_effects, errors);
            }
        }
    }
}

fn qualified_callee_name(module: Option<&str>, func: &str) -> String {
    match module {
        Some(module) => format!("{module}::{func}"),
        None => func.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_parser::surface::{BinaryOp, Literal, Name, Pattern};

    fn box_name(s: &str) -> Name {
        s.into()
    }

    fn var(name: &str) -> Box<Expr> {
        Box::new(Expr::Variable {
            name: box_name(name),
            span: ash_parser::token::Span::default(),
        })
    }

    fn int_lit(n: i64) -> Box<Expr> {
        Box::new(Expr::Literal(Literal::Int(n)))
    }

    #[test]
    fn pure_literal_is_ok() {
        let env = TypeEnv::new();
        let expr = Expr::Literal(Literal::Int(42));
        assert!(check_purity(&env, &expr, false).is_ok());
    }

    #[test]
    fn pure_binary_expr_is_ok() {
        let env = TypeEnv::new();
        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: int_lit(1),
            right: int_lit(2),
            span: Span::default(),
        };
        assert!(check_purity(&env, &expr, false).is_ok());
    }

    #[test]
    fn pure_variable_is_ok() {
        let env = TypeEnv::new();
        let expr = Expr::Variable {
            name: box_name("x"),
            span: ash_parser::token::Span::default(),
        };
        assert!(check_purity(&env, &expr, false).is_ok());
    }

    #[test]
    fn pure_call_is_ok() {
        let mut env = TypeEnv::new();
        env.bind_variable("f", Type::Fn(vec![Type::Int], Box::new(Type::Int)));
        let expr = Expr::Call {
            func: box_name("f"),
            module: None,
            args: vec![*int_lit(1)],
            span: Span::default(),
        };
        assert!(check_purity(&env, &expr, false).is_ok());
    }

    #[test]
    fn capability_typed_call_in_fn_body_is_impure() {
        let mut env = TypeEnv::new();
        env.bind_variable(
            "f",
            Type::Cap {
                name: "Io".into(),
                effect: ash_core::Effect::Operational,
            },
        );
        let expr = Expr::Call {
            func: box_name("f"),
            module: None,
            args: vec![*int_lit(1)],
            span: Span::default(),
        };
        let result = check_purity(&env, &expr, false).unwrap_err();
        assert!(matches!(
            &result[0].kind,
            PurityViolation::NonPureCall { callee, .. } if callee == "f"
        ));
    }

    #[test]
    fn policy_in_fn_body_is_impure() {
        use ash_parser::surface::PolicyExpr;
        let env = TypeEnv::new();
        let expr = Expr::Policy(PolicyExpr::Var {
            name: box_name("deny"),
            span: ash_parser::token::Span::default(),
        });
        let result = check_purity(&env, &expr, false);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, PurityViolation::PolicyExpression);
    }

    #[test]
    fn check_obligation_in_fn_body_is_impure() {
        let env = TypeEnv::new();
        let expr = Expr::CheckObligation {
            obligation: box_name("auth"),
            span: Span::default(),
        };
        let result = check_purity(&env, &expr, false);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, PurityViolation::CheckObligation);
    }

    #[test]
    fn unresolved_qualified_call_in_fn_body_is_impure() {
        let env = TypeEnv::new();
        // After TASK-561, interface method calls use Expr::Call with module qualifier.
        // Without a registered interface, this is an unresolved call (which is impure).
        let expr = Expr::Call {
            func: box_name("display"),
            module: Some(box_name("Print")),
            args: vec![*int_lit(42)],
            span: Span::default(),
        };
        let result = check_purity(&env, &expr, false);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0].kind,
            PurityViolation::UnresolvedCall { callee }
            if callee == "Print::display"
        ));
    }

    #[test]
    fn nested_violations_are_all_reported() {
        use ash_parser::surface::PolicyExpr;
        let env = TypeEnv::new();
        let expr = Expr::If {
            condition: int_lit(1),
            then_branch: Box::new(Expr::Policy(PolicyExpr::Var {
                name: box_name("deny"),
                span: ash_parser::token::Span::default(),
            })),
            else_branch: Some(Box::new(Expr::CheckObligation {
                obligation: box_name("auth"),
                span: Span::default(),
            })),
            span: Span::default(),
        };
        let result = check_purity(&env, &expr, false);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn block_with_let_bindings_is_pure() {
        let env = TypeEnv::new();
        let expr = Expr::Block {
            statements: vec![BlockStmt::Let {
                pattern: Pattern::Variable {
                    name: box_name("x"),
                    span: ash_parser::token::Span::default(),
                },
                expr: *int_lit(1),
                span: Span::default(),
            }],
            tail_expr: Some(var("x")),
            span: Span::default(),
        };
        assert!(check_purity(&env, &expr, false).is_ok());
    }

    #[test]
    fn panic_in_pure_fn_is_ok() {
        let env = TypeEnv::new();
        let expr = Expr::Panic {
            message: box_name("oops"),
            span: Span::default(),
        };
        assert!(check_purity(&env, &expr, false).is_ok());
    }

    #[test]
    fn one_armed_if_in_pure_fn_is_ok() {
        let env = TypeEnv::new();
        let expr = Expr::If {
            condition: Box::new(Expr::Literal(Literal::Bool(true))),
            then_branch: int_lit(1),
            else_branch: None,
            span: Span::default(),
        };
        assert!(check_purity(&env, &expr, false).is_ok());
    }

    // ── TASK-680: act {} and invoke() purity enforcement ──

    #[test]
    fn act_block_rejected_in_pure_context() {
        let env = TypeEnv::new();
        let expr = Expr::ActBlock {
            stmts: vec![ActStmt::Return {
                value: int_lit(42),
                span: Span::default(),
            }],
            span: Span::default(),
        };
        let result = check_purity(&env, &expr, false).unwrap_err();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, PurityViolation::ActBlockInPureContext);
    }

    #[test]
    fn invoke_rejected_in_pure_context() {
        let mut env = TypeEnv::new();
        // bind "invoke" as a non-Fn type so it doesn't get resolved as a pure call
        env.bind_variable("invoke", Type::Int);
        let expr = Expr::Call {
            func: box_name("invoke"),
            module: None,
            args: vec![],
            span: Span::default(),
        };
        let result = check_purity(&env, &expr, false).unwrap_err();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, PurityViolation::InvokeInPureContext);
    }

    #[test]
    fn act_block_allowed_in_effective_context() {
        let env = TypeEnv::new();
        let expr = Expr::ActBlock {
            stmts: vec![ActStmt::Return {
                value: int_lit(42),
                span: Span::default(),
            }],
            span: Span::default(),
        };
        assert!(check_purity(&env, &expr, true).is_ok());
    }

    #[test]
    fn invoke_allowed_in_effective_context() {
        let mut env = TypeEnv::new();
        env.bind_variable("invoke", Type::Fn(vec![], Box::new(Type::Int)));
        let expr = Expr::Call {
            func: box_name("invoke"),
            module: None,
            args: vec![],
            span: Span::default(),
        };
        // With allow_effects=true, the invoke shortcut should not fire,
        // but it still needs to resolve as a valid call target.
        assert!(check_purity(&env, &expr, true).is_ok());
    }
}
