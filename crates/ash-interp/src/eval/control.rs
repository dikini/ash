//! Expression control-form helpers for spawn/split/match/if-let.

use ash_core::ast::{MatchArm, Pattern};
use ash_core::{ControlLink, Expr, Instance, InstanceAddr, Value, WorkflowId};

use crate::EvalResult;
use crate::context::Context;
use crate::error::EvalError;

use super::eval_expr;

/// Generate a fresh instance ID
fn fresh_instance_id() -> WorkflowId {
    WorkflowId::new()
}

/// Evaluate a spawn expression
/// Creates a new Instance value with a fresh address and control link
pub(super) fn eval_spawn(workflow_type: &str) -> EvalResult<Value> {
    let instance_id = fresh_instance_id();

    let addr = InstanceAddr {
        workflow_type: workflow_type.to_string(),
        instance_id,
    };

    let control = Some(ControlLink { instance_id });

    Ok(Value::Instance(Box::new(Instance { addr, control })))
}

/// Evaluate a split expression
/// Splits an Instance into a tuple (InstanceAddr, Option<ControlLink>)
pub(super) fn eval_split(expr: &Expr, ctx: &Context) -> EvalResult<Value> {
    let value = eval_expr(expr, ctx)?;

    match value {
        Value::Instance(instance) => {
            let addr = Value::InstanceAddr(instance.addr);
            let control = instance.control.map(Value::ControlLink);
            // Return as a tuple: (addr, control)
            Ok(Value::List(Box::new(vec![
                addr,
                control.unwrap_or(Value::Null),
            ])))
        }
        _ => Err(EvalError::TypeMismatch {
            expected: "Instance".to_string(),
            actual: format!("{:?}", value),
        }),
    }
}

/// Evaluate a match expression
///
/// Tries each arm in order, returning the result of the first matching arm.
/// If no arm matches, returns a non-exhaustive match error.
pub(super) fn eval_match(scrutinee: &Expr, arms: &[MatchArm], ctx: &Context) -> EvalResult<Value> {
    let value = eval_expr(scrutinee, ctx)?;

    for arm in arms {
        match crate::pattern::match_pattern(&arm.pattern, &value) {
            Ok(bindings) => {
                // Create a new context with the bindings
                let mut new_ctx = ctx.extend();
                for (name, val) in bindings {
                    new_ctx.set(name, val);
                }
                return eval_expr(&arm.body, &new_ctx);
            }
            Err(_) => {
                // Pattern didn't match, try next arm
                continue;
            }
        }
    }

    // No arm matched
    Err(EvalError::NonExhaustiveMatch {
        value: format!("{:?}", value),
    })
}

/// Evaluate an if-let expression
///
/// If the pattern matches the expression value, evaluates the then branch with bindings.
/// Otherwise evaluates the else branch.
pub(super) fn eval_if_let(
    pattern: &Pattern,
    expr: &Expr,
    then_branch: &Expr,
    else_branch: &Expr,
    ctx: &Context,
) -> EvalResult<Value> {
    let value = eval_expr(expr, ctx)?;

    match crate::pattern::match_pattern(pattern, &value) {
        Ok(bindings) => {
            // Pattern matched - evaluate then branch with bindings
            let mut new_ctx = ctx.extend();
            for (name, val) in bindings {
                new_ctx.set(name, val);
            }
            eval_expr(then_branch, &new_ctx)
        }
        Err(_) => {
            // Pattern didn't match - evaluate else branch
            eval_expr(else_branch, ctx)
        }
    }
}
