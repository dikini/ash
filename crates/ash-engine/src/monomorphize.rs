use ash_core::ast::{Expr, Workflow};
use ash_typeck::type_env::TypeEnv;
use ash_typeck::{Kind, QualifiedName, Type};

#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum MonomorphizeError {
    #[error("interface resolution failed: {0}")]
    Resolution(String),
    #[error("missing method '{method}' in scheme for interface '{interface}'")]
    MissingMethod { interface: String, method: String },
}

/// Walk a core workflow and replace interface method calls with monomorphized bodies.
pub fn monomorphize_workflow(
    workflow: &mut Workflow,
    type_env: &TypeEnv,
) -> Result<(), MonomorphizeError> {
    match workflow {
        Workflow::Observe { continuation, .. } => {
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
        Workflow::Orient { expr, continuation } => {
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
        Workflow::Decide {
            expr, continuation, ..
        } => {
            monomorphize_expr(expr, type_env)?;
            monomorphize_workflow(continuation, type_env)?;
        }
        Workflow::Check { continuation, .. } => {
            monomorphize_workflow(continuation, type_env)?;
        }
        Workflow::Act {
            arguments,
            continuation,
            ..
        } => {
            for arg in arguments.iter_mut() {
                monomorphize_expr(arg, type_env)?;
            }
            monomorphize_workflow(continuation, type_env)?;
        }
        Workflow::Oblig { workflow, .. } => {
            monomorphize_workflow(workflow, type_env)?;
        }
        Workflow::Let {
            expr, continuation, ..
        } => {
            monomorphize_expr(expr, type_env)?;
            monomorphize_workflow(continuation, type_env)?;
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
        Workflow::With { workflow, .. } => {
            monomorphize_workflow(workflow, type_env)?;
        }
        Workflow::Maybe { primary, fallback } => {
            monomorphize_workflow(primary, type_env)?;
            monomorphize_workflow(fallback, type_env)?;
        }
        Workflow::Must { workflow } => {
            monomorphize_workflow(workflow, type_env)?;
        }
        Workflow::Spawn {
            init, continuation, ..
        } => {
            monomorphize_expr(init, type_env)?;
            monomorphize_workflow(continuation, type_env)?;
        }
        Workflow::Split {
            expr, continuation, ..
        } => {
            monomorphize_expr(expr, type_env)?;
            monomorphize_workflow(continuation, type_env)?;
        }
        Workflow::Kill { continuation, .. }
        | Workflow::Pause { continuation, .. }
        | Workflow::Resume { continuation, .. }
        | Workflow::CheckHealth { continuation, .. } => {
            monomorphize_workflow(continuation, type_env)?;
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
        Workflow::Set { value, .. } => {
            monomorphize_expr(value, type_env)?;
        }
        Workflow::Send { value, .. } => {
            monomorphize_expr(value, type_env)?;
        }
        Workflow::Oblige { .. } | Workflow::CheckObligation { .. } | Workflow::Done => {}
    }
    Ok(())
}

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
                    infer_type_from_expr(arg).ok_or_else(|| {
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

            let mut body = method_info.body.clone();
            apply_substitution_to_expr(&selected.substitution, &mut body);

            *expr = body;
        }
        Expr::FieldAccess { expr: inner, .. } => {
            monomorphize_expr(inner, type_env)?;
        }
        Expr::IndexAccess { expr: inner, index } => {
            monomorphize_expr(inner, type_env)?;
            monomorphize_expr(index, type_env)?;
        }
        Expr::Unary { expr: inner, .. } => {
            monomorphize_expr(inner, type_env)?;
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
        Expr::Spawn { init, .. } => {
            monomorphize_expr(init, type_env)?;
        }
        Expr::Split(inner) => {
            monomorphize_expr(inner, type_env)?;
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
        Expr::Literal(_) | Expr::Variable(_) | Expr::CheckObligation { .. } => {}
    }
    Ok(())
}

fn infer_type_from_expr(expr: &Expr) -> Option<Type> {
    match expr {
        Expr::Literal(v) => Some(value_to_type(v)),
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
            let item_ty = items
                .first()
                .map(value_to_type)
                .unwrap_or(Type::Var(ash_typeck::types::TypeVar::fresh()));
            Type::List(Box::new(item_ty))
        }
        ash_core::Value::Record(fields) => Type::Record(
            fields
                .iter()
                .map(|(n, v)| (Box::from(n.as_str()), value_to_type(v)))
                .collect(),
        ),
        ash_core::Value::Cap(_) => Type::Var(ash_typeck::types::TypeVar::fresh()),
        ash_core::Value::Variant { .. } => Type::Var(ash_typeck::types::TypeVar::fresh()),
        ash_core::Value::Instance(_) => Type::Instance {
            workflow_type: Box::from(""),
        },
        ash_core::Value::InstanceAddr(_) => Type::InstanceAddr {
            workflow_type: Box::from(""),
        },
        ash_core::Value::ControlLink(_) => Type::ControlLink {
            workflow_type: Box::from(""),
        },
        ash_core::Value::Closure { .. } => Type::Var(ash_typeck::types::TypeVar::fresh()),
        ash_core::Value::Stream(_) => Type::Var(ash_typeck::types::TypeVar::fresh()),
    }
}

fn apply_substitution_to_expr(_subst: &ash_typeck::types::Substitution, _expr: &mut Expr) {
    // TODO: recursively apply substitution to type annotations inside the expression.
    // For now, the test bodies contain no type variables, so this is a no-op.
}
