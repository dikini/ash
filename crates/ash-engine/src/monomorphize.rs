//! Engine monomorphization pass.
//!
//! Replaces interface method calls in core AST with concrete impl bodies after
//! type checking so that the interpreter never sees generic impl dispatch at
//! runtime.

use ash_core::ast::{Expr, Workflow};
use ash_typeck::type_env::TypeEnv;
use ash_typeck::{Kind, QualifiedName, Type};

/// Errors that can occur during monomorphization.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum MonomorphizeError {
    /// Interface resolution failed (e.g. no matching impl scheme).
    #[error("interface resolution failed: {0}")]
    Resolution(String),
    /// The selected impl scheme does not contain the requested method.
    #[error("missing method '{method}' in scheme for interface '{interface}'")]
    MissingMethod {
        /// Name of the interface.
        interface: String,
        /// Name of the missing method.
        method: String,
    },
}

/// Walk a core workflow and replace interface method calls with monomorphized bodies.
///
/// # Errors
///
/// Returns [`MonomorphizeError::Resolution`] when an interface method call
/// cannot be resolved to a concrete impl scheme, or
/// [`MonomorphizeError::MissingMethod`] when the selected scheme lacks the
/// requested method.
#[allow(clippy::too_many_lines)]
pub fn monomorphize_workflow(
    workflow: &mut Workflow,
    type_env: &TypeEnv,
) -> Result<(), MonomorphizeError> {
    match workflow {
        Workflow::Observe { continuation, .. }
        | Workflow::Check { continuation, .. }
        | Workflow::Kill { continuation, .. }
        | Workflow::Pause { continuation, .. }
        | Workflow::Resume { continuation, .. }
        | Workflow::CheckHealth { continuation, .. } => {
            monomorphize_workflow(continuation, type_env)?;
        }
        Workflow::Receive { arms, .. } => {
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    monomorphize_expr(guard, type_env)?;
                }
                monomorphize_workflow(&mut arm.body, type_env)?;
            }
        }
        Workflow::Orient { expr, continuation }
        | Workflow::Decide {
            expr, continuation, ..
        }
        | Workflow::Let {
            expr, continuation, ..
        }
        | Workflow::Spawn {
            init: expr,
            continuation,
            ..
        }
        | Workflow::Split {
            expr, continuation, ..
        } => {
            monomorphize_expr(expr, type_env)?;
            monomorphize_workflow(continuation, type_env)?;
        }
        Workflow::Propose {
            action_arguments,
            continuation,
            ..
        } => {
            for arg in action_arguments.iter_mut() {
                monomorphize_expr(arg, type_env)?;
            }
            monomorphize_workflow(continuation, type_env)?;
        }
        Workflow::Act {
            arguments,
            continuation,
            ..
        }
        | Workflow::Call {
            arguments,
            continuation,
            ..
        } => {
            for arg in arguments.iter_mut() {
                monomorphize_expr(arg, type_env)?;
            }
            monomorphize_workflow(continuation, type_env)?;
        }
        Workflow::Oblig { workflow, .. }
        | Workflow::With { workflow, .. }
        | Workflow::Must { workflow } => {
            monomorphize_workflow(workflow, type_env)?;
        }
        Workflow::If {
            condition,
            then_branch,
            else_branch,
        } => {
            monomorphize_expr(condition, type_env)?;
            monomorphize_workflow(then_branch, type_env)?;
            monomorphize_workflow(else_branch, type_env)?;
        }
        Workflow::Seq { first, second } => {
            monomorphize_workflow(first, type_env)?;
            monomorphize_workflow(second, type_env)?;
        }
        Workflow::ForEach {
            collection, body, ..
        } => {
            monomorphize_expr(collection, type_env)?;
            monomorphize_workflow(body, type_env)?;
        }
        Workflow::Ret { expr } => {
            monomorphize_expr(expr, type_env)?;
        }
        Workflow::Maybe { primary, fallback } => {
            monomorphize_workflow(primary, type_env)?;
            monomorphize_workflow(fallback, type_env)?;
        }
        Workflow::Yield {
            request,
            continuation,
            ..
        } => {
            monomorphize_expr(request, type_env)?;
            monomorphize_workflow(continuation, type_env)?;
        }
        Workflow::ProxyResume { value, .. } => {
            monomorphize_expr(value, type_env)?;
        }
        Workflow::Set { value, .. } | Workflow::Send { value, .. } => {
            monomorphize_expr(value, type_env)?;
        }
        Workflow::Oblige { .. } | Workflow::CheckObligation { .. } | Workflow::Done => {}
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn monomorphize_expr(expr: &mut Expr, type_env: &TypeEnv) -> Result<(), MonomorphizeError> {
    match expr {
        Expr::Call {
            func,
            module: Some(interface),
            arguments,
        } => {
            // Monomorphize arguments first.
            for arg in arguments.iter_mut() {
                monomorphize_expr(arg, type_env)?;
            }

            let arg_types: Vec<Type> = arguments
                .iter()
                .map(|arg| {
                    infer_type_from_expr(arg, type_env).ok_or_else(|| {
                        MonomorphizeError::Resolution(format!(
                            "cannot infer type for argument: {arg:?}"
                        ))
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            let (selected, scheme) = type_env
                .select_impl_scheme(interface, func, &arg_types)
                .map_err(|e| MonomorphizeError::Resolution(e.to_string()))?;

            let method_info = scheme
                .methods
                .iter()
                .find(|m| m.name == *func)
                .ok_or_else(|| MonomorphizeError::MissingMethod {
                    interface: interface.clone(),
                    method: func.clone(),
                })?;

            // Normalize associated types in the method signature
            let normalized_return_type = type_env
                .normalize_associated_types(
                    &method_info.return_type,
                    scheme,
                    &selected.substitution,
                )
                .map_err(|e| MonomorphizeError::Resolution(e.to_string()))?;

            let normalized_params: Vec<Type> = method_info
                .params
                .iter()
                .map(|p| {
                    type_env
                        .normalize_associated_types(p, scheme, &selected.substitution)
                        .map_err(|e| MonomorphizeError::Resolution(e.to_string()))
                })
                .collect::<Result<Vec<_>, _>>()?;

            // Optional: debug-only assertion that no Type::Associated remains
            #[cfg(debug_assertions)]
            assert!(
                !type_contains_associated(&normalized_return_type),
                "monomorphized return type still contains Type::Associated: {normalized_return_type:?}"
            );
            #[cfg(debug_assertions)]
            for p in &normalized_params {
                assert!(
                    !type_contains_associated(p),
                    "monomorphized param type still contains Type::Associated: {p:?}"
                );
            }

            let _ = (&normalized_return_type, &normalized_params);

            let mut body = method_info.body.clone();
            apply_substitution_to_expr(&selected.substitution, &mut body);

            *expr = body;
        }
        Expr::FieldAccess { expr: inner, .. }
        | Expr::Unary { expr: inner, .. }
        | Expr::Spawn { init: inner, .. }
        | Expr::Split(inner) => {
            monomorphize_expr(inner, type_env)?;
        }
        Expr::IndexAccess { expr: inner, index } => {
            monomorphize_expr(inner, type_env)?;
            monomorphize_expr(index, type_env)?;
        }
        Expr::Binary { left, right, .. } => {
            monomorphize_expr(left, type_env)?;
            monomorphize_expr(right, type_env)?;
        }
        Expr::Call {
            module: None,
            arguments,
            ..
        } => {
            for arg in arguments.iter_mut() {
                monomorphize_expr(arg, type_env)?;
            }
        }
        Expr::Constructor { fields, .. } => {
            for (_, field_expr) in fields.iter_mut() {
                monomorphize_expr(field_expr, type_env)?;
            }
        }
        Expr::Match { scrutinee, arms } => {
            monomorphize_expr(scrutinee, type_env)?;
            for arm in arms {
                monomorphize_expr(&mut arm.body, type_env)?;
            }
        }
        Expr::IfLet {
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            monomorphize_expr(expr, type_env)?;
            monomorphize_expr(then_branch, type_env)?;
            monomorphize_expr(else_branch, type_env)?;
        }
        Expr::FnApply { func, args } => {
            monomorphize_expr(func, type_env)?;
            for arg in args.iter_mut() {
                monomorphize_expr(arg, type_env)?;
            }
        }
        Expr::FnDef { body, .. } => {
            monomorphize_expr(body, type_env)?;
        }
        Expr::Literal(_) | Expr::Variable { .. } | Expr::CheckObligation { .. } => {}
        Expr::Let { expr, body, .. } => {
            monomorphize_expr(expr, type_env)?;
            monomorphize_expr(body, type_env)?;
        }
    }
    Ok(())
}

fn infer_type_from_expr(expr: &Expr, type_env: &TypeEnv) -> Option<Type> {
    match expr {
        Expr::Literal(v) => Some(value_to_type(v)),
        Expr::Variable { name, .. } => type_env.lookup_variable(name),
        Expr::Constructor { name, .. } => Some(Type::Constructor {
            name: QualifiedName::root(name.clone()),
            args: vec![],
            kind: Kind::Type,
        }),
        _ => None,
    }
}

fn value_to_type(v: &ash_core::Value) -> Type {
    match v {
        ash_core::Value::Int(_) => Type::Int,
        ash_core::Value::String(_) => Type::String,
        ash_core::Value::Bool(_) => Type::Bool,
        ash_core::Value::Null => Type::Null,
        ash_core::Value::Time(_) => Type::Time,
        ash_core::Value::Ref(_) => Type::Ref,
        ash_core::Value::Float(_) => Type::Float,
        ash_core::Value::List(items) => {
            let item_ty = items.first().map_or_else(
                || Type::Var(ash_typeck::types::TypeVar::fresh()),
                value_to_type,
            );
            Type::List(Box::new(item_ty))
        }
        ash_core::Value::Record(fields) => Type::Record(
            fields
                .iter()
                .map(|(n, v)| (Box::from(n.as_str()), value_to_type(v)))
                .collect(),
        ),
        ash_core::Value::Cap(_)
        | ash_core::Value::Variant { .. }
        | ash_core::Value::Closure { .. }
        | ash_core::Value::Stream(_)
        | ash_core::Value::ActEnvToken => Type::Var(ash_typeck::types::TypeVar::fresh()),
        ash_core::Value::Instance(_) => Type::Instance {
            workflow_type: Box::from(""),
        },
        ash_core::Value::InstanceAddr(_) => Type::InstanceAddr {
            workflow_type: Box::from(""),
        },
        ash_core::Value::ControlLink(_) => Type::ControlLink {
            workflow_type: Box::from(""),
        },
    }
}

/// Core `Expr` does not embed `Type` values (only string annotations in `FnDef`),
/// so a type-variable substitution cannot be applied directly to the expression
/// tree. Nested interface calls are monomorphized recursively by
/// `monomorphize_expr`, and `FnDef` string annotations are resolved later by the
/// type checker. This function is intentionally a no-op.
const fn apply_substitution_to_expr(_subst: &ash_typeck::types::Substitution, _expr: &mut Expr) {}

fn type_contains_associated(ty: &Type) -> bool {
    match ty {
        Type::Associated { .. } => true,
        Type::List(inner) => type_contains_associated(inner),
        Type::Record(fields) => fields.iter().any(|(_, t)| type_contains_associated(t)),
        Type::Fun(params, ret, _) | Type::Fn(params, ret) => {
            params.iter().any(type_contains_associated) || type_contains_associated(ret)
        }
        Type::Constructor { args, .. } => args.iter().any(type_contains_associated),
        _ => false,
    }
}
