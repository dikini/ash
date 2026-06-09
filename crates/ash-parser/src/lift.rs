//! ANF-style statement lifting pass.
//!
//! Extracts effectful sub-expressions from `let` RHS into synthetic
//! `let` bindings so that the RHS becomes effectively pure.
//!
//! Runs after surface-to-core lowering but before type checking.

use ash_core::{Expr as CoreExpr, Pattern as CorePattern, Span, Workflow as CoreWorkflow};

struct LiftState {
    next_id: u64,
}

impl LiftState {
    fn fresh_lift_var(&mut self) -> String {
        let id = self.next_id;
        self.next_id += 1;
        format!("__lift_{id}")
    }
}

fn fresh_lift_var(state: &mut LiftState) -> String {
    state.fresh_lift_var()
}

fn default_span() -> Span {
    Span::default()
}

/// Preserve the original expression when lifting would require synthetic
/// bindings in a workflow position that cannot honestly host those bindings.
///
/// This avoids panicking during lowering and preserves the original surface
/// intent so downstream type-checking and diagnostics can report the real
/// problem instead of an internal lifting failure or an unbound synthetic var.
fn preserve_original_if_bindings(
    original: CoreExpr,
    lifted: CoreExpr,
    bindings: &[(String, CoreExpr)],
) -> CoreExpr {
    if bindings.is_empty() {
        lifted
    } else {
        original
    }
}

/// Returns `true` if the given expression is effectful and should be lifted.
fn is_effectful(expr: &CoreExpr, effectful_names: &std::collections::HashSet<String>) -> bool {
    match expr {
        // Qualified capability calls are always effectful.
        CoreExpr::Call {
            module: Some(_), ..
        } => true,
        // Spawn is always effectful.
        CoreExpr::Spawn { .. } => true,
        // Unqualified calls to names in the effectful set.
        CoreExpr::FnApply { func, args: _ } => {
            if let CoreExpr::Variable { name, .. } = func.as_ref() {
                effectful_names.contains(name.as_str())
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Lift a core expression, returning the rewritten expression and a list of
/// synthetic `(var_name, binding_expr)` pairs in evaluation order.
fn lift_expr(
    expr: CoreExpr,
    effectful_names: &std::collections::HashSet<String>,
    state: &mut LiftState,
) -> (CoreExpr, Vec<(String, CoreExpr)>) {
    match expr {
        // Atoms: nothing to lift.
        CoreExpr::Literal(_) | CoreExpr::Variable { .. } => (expr, vec![]),

        CoreExpr::FieldAccess { expr: e, field } => {
            let (new_e, bindings) = lift_expr(*e, effectful_names, state);
            (
                CoreExpr::FieldAccess {
                    expr: Box::new(new_e),
                    field,
                },
                bindings,
            )
        }

        CoreExpr::IndexAccess { expr: e, index } => {
            let (new_e, e_bindings) = lift_expr(*e, effectful_names, state);
            let (new_index, index_bindings) = lift_expr(*index, effectful_names, state);
            let mut bindings = e_bindings;
            bindings.extend(index_bindings);
            (
                CoreExpr::IndexAccess {
                    expr: Box::new(new_e),
                    index: Box::new(new_index),
                },
                bindings,
            )
        }

        CoreExpr::Unary { op, expr: e } => {
            let (new_e, bindings) = lift_expr(*e, effectful_names, state);
            (
                CoreExpr::Unary {
                    op,
                    expr: Box::new(new_e),
                },
                bindings,
            )
        }

        CoreExpr::Binary { op, left, right } => {
            let (new_left, left_bindings) = lift_expr(*left, effectful_names, state);
            let (new_right, right_bindings) = lift_expr(*right, effectful_names, state);
            let mut bindings = left_bindings;
            bindings.extend(right_bindings);
            (
                CoreExpr::Binary {
                    op,
                    left: Box::new(new_left),
                    right: Box::new(new_right),
                },
                bindings,
            )
        }

        CoreExpr::Call {
            func,
            module,
            arguments,
        } => {
            let mut bindings = Vec::new();
            let mut new_args = Vec::new();
            for arg in arguments {
                let (new_arg, arg_bindings) = lift_expr(arg, effectful_names, state);
                bindings.extend(arg_bindings);
                new_args.push(new_arg);
            }
            let new_expr = CoreExpr::Call {
                func,
                module,
                arguments: new_args,
            };
            if is_effectful(&new_expr, effectful_names) {
                let var = fresh_lift_var(state);
                bindings.push((var.clone(), new_expr));
                (
                    CoreExpr::Variable {
                        name: var,
                        span: default_span(),
                    },
                    bindings,
                )
            } else {
                (new_expr, bindings)
            }
        }

        CoreExpr::Constructor { name, fields } => {
            let mut bindings = Vec::new();
            let mut new_fields = Vec::new();
            for (field_name, field_expr) in fields {
                let (new_field, field_bindings) = lift_expr(field_expr, effectful_names, state);
                bindings.extend(field_bindings);
                new_fields.push((field_name, new_field));
            }
            (
                CoreExpr::Constructor {
                    name,
                    fields: new_fields,
                },
                bindings,
            )
        }

        CoreExpr::Match { scrutinee, arms } => {
            let (new_scrutinee, scrut_bindings) = lift_expr(*scrutinee, effectful_names, state);
            let mut new_arms = Vec::new();
            for arm in arms {
                // Lift inside the arm body for ANF, but discard body bindings
                // since they may reference pattern-bound variables not in outer scope.
                // When the inner body is effectful, lift_expr rewrites it to a
                // synthetic __lift_ variable and returns the real binding separately.
                // Since we cannot host those bindings here, fall back to the original
                // expression to avoid producing an unbound synthetic variable.
                let original_body = arm.body.clone();
                let (new_body, body_bindings) = lift_expr(arm.body, effectful_names, state);
                let preserved =
                    preserve_original_if_bindings(original_body, new_body, &body_bindings);
                new_arms.push(ash_core::MatchArm {
                    pattern: arm.pattern,
                    body: preserved,
                });
            }
            (
                CoreExpr::Match {
                    scrutinee: Box::new(new_scrutinee),
                    arms: new_arms,
                },
                scrut_bindings,
            )
        }

        CoreExpr::IfLet {
            pattern,
            expr: e,
            then_branch,
            else_branch,
        } => {
            let (new_e, e_bindings) = lift_expr(*e, effectful_names, state);
            // Lift inside branches for ANF, but discard branch bindings
            // since they may reference pattern-bound variables not in outer scope.
            // When a branch is effectful, fall back to the original expression
            // to avoid producing an unbound synthetic variable.
            let original_then = (*then_branch).clone();
            let (new_then, then_bindings) = lift_expr(*then_branch, effectful_names, state);
            let preserved_then =
                preserve_original_if_bindings(original_then, new_then, &then_bindings);

            let original_else = (*else_branch).clone();
            let (new_else, else_bindings) = lift_expr(*else_branch, effectful_names, state);
            let preserved_else =
                preserve_original_if_bindings(original_else, new_else, &else_bindings);

            (
                CoreExpr::IfLet {
                    pattern,
                    expr: Box::new(new_e),
                    then_branch: Box::new(preserved_then),
                    else_branch: Box::new(preserved_else),
                },
                e_bindings,
            )
        }

        CoreExpr::Spawn {
            workflow_type,
            init,
        } => {
            let (new_init, bindings) = lift_expr(*init, effectful_names, state);
            let new_expr = CoreExpr::Spawn {
                workflow_type,
                init: Box::new(new_init),
            };
            let var = fresh_lift_var(state);
            let mut all_bindings = bindings;
            all_bindings.push((var.clone(), new_expr));
            (
                CoreExpr::Variable {
                    name: var,
                    span: default_span(),
                },
                all_bindings,
            )
        }

        CoreExpr::Split(e) => {
            let (new_e, bindings) = lift_expr(*e, effectful_names, state);
            (CoreExpr::Split(Box::new(new_e)), bindings)
        }

        CoreExpr::CheckObligation { obligation, span } => {
            (CoreExpr::CheckObligation { obligation, span }, vec![])
        }

        CoreExpr::Fail { payload } => {
            let (new_payload, bindings) = lift_expr(*payload, effectful_names, state);
            (
                CoreExpr::Fail {
                    payload: Box::new(new_payload),
                },
                bindings,
            )
        }

        CoreExpr::WithError { body, arms } => {
            let original_body = (*body).clone();
            let (new_body, body_bindings) = lift_expr(*body, effectful_names, state);
            let preserved_body =
                preserve_original_if_bindings(original_body, new_body, &body_bindings);
            let mut new_arms = Vec::new();
            for arm in arms {
                let original_arm_body = arm.body.clone();
                let (new_arm_body, arm_bindings) = lift_expr(arm.body, effectful_names, state);
                let preserved_arm_body =
                    preserve_original_if_bindings(original_arm_body, new_arm_body, &arm_bindings);
                new_arms.push(ash_core::MatchArm {
                    pattern: arm.pattern,
                    body: preserved_arm_body,
                });
            }
            (
                CoreExpr::WithError {
                    body: Box::new(preserved_body),
                    arms: new_arms,
                },
                vec![],
            )
        }

        CoreExpr::FnDef {
            params,
            return_type,
            body,
        } => {
            // Lift inside the body of the closure for ANF, but discard bindings
            // since they may reference closure parameters not in outer scope.
            // When the body is effectful, fall back to the original expression
            // to avoid producing an unbound synthetic variable.
            let original_body = (*body).clone();
            let (new_body, bindings) = lift_expr(*body, effectful_names, state);
            let preserved = preserve_original_if_bindings(original_body, new_body, &bindings);
            (
                CoreExpr::FnDef {
                    params,
                    return_type,
                    body: Box::new(preserved),
                },
                vec![],
            )
        }

        CoreExpr::FnApply { func, args } => {
            let (new_func, func_bindings) = lift_expr(*func, effectful_names, state);
            let mut bindings = func_bindings;
            let mut new_args = Vec::new();
            for arg in args {
                let (new_arg, arg_bindings) = lift_expr(arg, effectful_names, state);
                bindings.extend(arg_bindings);
                new_args.push(new_arg);
            }
            let new_expr = CoreExpr::FnApply {
                func: Box::new(new_func),
                args: new_args,
            };
            if is_effectful(&new_expr, effectful_names) {
                let var = fresh_lift_var(state);
                bindings.push((var.clone(), new_expr));
                (
                    CoreExpr::Variable {
                        name: var,
                        span: default_span(),
                    },
                    bindings,
                )
            } else {
                (new_expr, bindings)
            }
        }

        // Expr::Let is pure scope extension. Lift sub-expressions, but
        // discard body bindings (body may reference pattern-bound variables).
        CoreExpr::Let {
            pattern,
            expr,
            body,
            span,
        } => {
            let (new_expr, e_bindings) = lift_expr(*expr, effectful_names, state);
            let original_body = (*body).clone();
            let (new_body, body_bindings) = lift_expr(*body, effectful_names, state);
            let preserved_body =
                preserve_original_if_bindings(original_body, new_body, &body_bindings);
            (
                CoreExpr::Let {
                    pattern,
                    expr: Box::new(new_expr),
                    body: Box::new(preserved_body),
                    span,
                },
                e_bindings,
            )
        }
    }
}

/// Lift effectful sub-expressions out of a workflow.
fn lift_workflow_inner(
    workflow: CoreWorkflow,
    effectful_names: &std::collections::HashSet<String>,
    state: &mut LiftState,
) -> CoreWorkflow {
    match workflow {
        CoreWorkflow::Let {
            pattern,
            expr,
            continuation,
        } => {
            let (lifted_expr, bindings) = lift_expr(expr, effectful_names, state);
            let mut result = CoreWorkflow::Let {
                pattern,
                expr: lifted_expr,
                continuation: Box::new(lift_workflow_inner(*continuation, effectful_names, state)),
            };
            // Wrap synthetic bindings around the result, innermost first.
            // `bindings` are in evaluation order; we reverse so the outermost
            // synthetic let corresponds to the first evaluated effect.
            for (var, binding_expr) in bindings.into_iter().rev() {
                result = CoreWorkflow::Let {
                    pattern: CorePattern::Variable {
                        name: var,
                        span: default_span(),
                    },
                    expr: binding_expr,
                    continuation: Box::new(result),
                };
            }
            result
        }

        CoreWorkflow::Observe {
            capability,
            pattern,
            continuation,
        } => CoreWorkflow::Observe {
            capability,
            pattern,
            continuation: Box::new(lift_workflow_inner(*continuation, effectful_names, state)),
        },

        CoreWorkflow::Receive {
            mode,
            arms,
            control,
        } => CoreWorkflow::Receive {
            mode,
            arms: arms
                .into_iter()
                .map(|arm| ash_core::ReceiveArm {
                    pattern: arm.pattern,
                    guard: arm.guard.map(|g| {
                        let original = g.clone();
                        let (lifted, bindings) = lift_expr(g, effectful_names, state);
                        preserve_original_if_bindings(original, lifted, &bindings)
                    }),
                    body: lift_workflow_inner(arm.body, effectful_names, state),
                })
                .collect(),
            control,
        },

        CoreWorkflow::Orient { expr, continuation } => {
            let original = expr.clone();
            let (lifted_expr, bindings) = lift_expr(expr, effectful_names, state);
            CoreWorkflow::Orient {
                expr: preserve_original_if_bindings(original, lifted_expr, &bindings),
                continuation: Box::new(lift_workflow_inner(*continuation, effectful_names, state)),
            }
        }

        CoreWorkflow::Propose {
            action_name,
            action_arguments,
            continuation,
        } => CoreWorkflow::Propose {
            action_name,
            action_arguments: action_arguments
                .into_iter()
                .map(|arg| {
                    let original = arg.clone();
                    let (lifted, bindings) = lift_expr(arg, effectful_names, state);
                    preserve_original_if_bindings(original, lifted, &bindings)
                })
                .collect(),
            continuation: Box::new(lift_workflow_inner(*continuation, effectful_names, state)),
        },

        CoreWorkflow::Decide {
            expr,
            policy,
            continuation,
        } => {
            let original = expr.clone();
            let (lifted_expr, bindings) = lift_expr(expr, effectful_names, state);
            CoreWorkflow::Decide {
                expr: preserve_original_if_bindings(original, lifted_expr, &bindings),
                policy,
                continuation: Box::new(lift_workflow_inner(*continuation, effectful_names, state)),
            }
        }

        CoreWorkflow::Check {
            obligation,
            continuation,
        } => CoreWorkflow::Check {
            obligation,
            continuation: Box::new(lift_workflow_inner(*continuation, effectful_names, state)),
        },

        CoreWorkflow::Act {
            provider_name,
            action_name,
            arguments,
            guard,
            provenance,
            result_name,
            continuation,
        } => CoreWorkflow::Act {
            provider_name,
            action_name,
            arguments: arguments
                .into_iter()
                .map(|arg| {
                    let original = arg.clone();
                    let (lifted, bindings) = lift_expr(arg, effectful_names, state);
                    preserve_original_if_bindings(original, lifted, &bindings)
                })
                .collect(),
            guard: {
                // Guard is a Guard enum, not Expr. We can't easily lift here.
                guard
            },
            provenance,
            result_name,
            continuation: Box::new(lift_workflow_inner(*continuation, effectful_names, state)),
        },

        CoreWorkflow::Call {
            target,
            arguments,
            continuation,
        } => CoreWorkflow::Call {
            target,
            arguments: arguments
                .into_iter()
                .map(|arg| {
                    let original = arg.clone();
                    let (lifted, bindings) = lift_expr(arg, effectful_names, state);
                    preserve_original_if_bindings(original, lifted, &bindings)
                })
                .collect(),
            continuation: Box::new(lift_workflow_inner(*continuation, effectful_names, state)),
        },

        CoreWorkflow::Oblig { role, workflow: w } => CoreWorkflow::Oblig {
            role,
            workflow: Box::new(lift_workflow_inner(*w, effectful_names, state)),
        },

        CoreWorkflow::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let original = condition.clone();
            let (lifted_condition, bindings) = lift_expr(condition, effectful_names, state);
            CoreWorkflow::If {
                condition: preserve_original_if_bindings(original, lifted_condition, &bindings),
                then_branch: Box::new(lift_workflow_inner(*then_branch, effectful_names, state)),
                else_branch: Box::new(lift_workflow_inner(*else_branch, effectful_names, state)),
            }
        }

        CoreWorkflow::Seq { first, second } => CoreWorkflow::Seq {
            first: Box::new(lift_workflow_inner(*first, effectful_names, state)),
            second: Box::new(lift_workflow_inner(*second, effectful_names, state)),
        },

        CoreWorkflow::ForEach {
            pattern,
            collection,
            body,
        } => {
            let original = collection.clone();
            let (lifted_collection, bindings) = lift_expr(collection, effectful_names, state);
            CoreWorkflow::ForEach {
                pattern,
                collection: preserve_original_if_bindings(original, lifted_collection, &bindings),
                body: Box::new(lift_workflow_inner(*body, effectful_names, state)),
            }
        }

        CoreWorkflow::Ret { expr } => {
            let original = expr.clone();
            let (lifted_expr, bindings) = lift_expr(expr, effectful_names, state);
            CoreWorkflow::Ret {
                expr: preserve_original_if_bindings(original, lifted_expr, &bindings),
            }
        }

        CoreWorkflow::With {
            capability,
            workflow: w,
        } => CoreWorkflow::With {
            capability,
            workflow: Box::new(lift_workflow_inner(*w, effectful_names, state)),
        },

        CoreWorkflow::Maybe { primary, fallback } => CoreWorkflow::Maybe {
            primary: Box::new(lift_workflow_inner(*primary, effectful_names, state)),
            fallback: Box::new(lift_workflow_inner(*fallback, effectful_names, state)),
        },

        CoreWorkflow::Must { workflow: w } => CoreWorkflow::Must {
            workflow: Box::new(lift_workflow_inner(*w, effectful_names, state)),
        },

        CoreWorkflow::Set {
            capability,
            channel,
            value,
        } => {
            let original = value.clone();
            let (lifted_value, bindings) = lift_expr(value, effectful_names, state);
            CoreWorkflow::Set {
                capability,
                channel,
                value: preserve_original_if_bindings(original, lifted_value, &bindings),
            }
        }

        CoreWorkflow::Send {
            capability,
            channel,
            value,
        } => {
            let original = value.clone();
            let (lifted_value, bindings) = lift_expr(value, effectful_names, state);
            CoreWorkflow::Send {
                capability,
                channel,
                value: preserve_original_if_bindings(original, lifted_value, &bindings),
            }
        }

        CoreWorkflow::Spawn {
            workflow_type,
            init,
            pattern,
            continuation,
        } => {
            let original = init.clone();
            let (lifted_init, bindings) = lift_expr(init, effectful_names, state);
            CoreWorkflow::Spawn {
                workflow_type,
                init: preserve_original_if_bindings(original, lifted_init, &bindings),
                pattern,
                continuation: Box::new(lift_workflow_inner(*continuation, effectful_names, state)),
            }
        }

        CoreWorkflow::Split {
            expr,
            pattern,
            continuation,
        } => {
            let original = expr.clone();
            let (lifted_expr, bindings) = lift_expr(expr, effectful_names, state);
            CoreWorkflow::Split {
                expr: preserve_original_if_bindings(original, lifted_expr, &bindings),
                pattern,
                continuation: Box::new(lift_workflow_inner(*continuation, effectful_names, state)),
            }
        }

        CoreWorkflow::Kill {
            target,
            continuation,
        } => CoreWorkflow::Kill {
            target,
            continuation: Box::new(lift_workflow_inner(*continuation, effectful_names, state)),
        },

        CoreWorkflow::Pause {
            target,
            continuation,
        } => CoreWorkflow::Pause {
            target,
            continuation: Box::new(lift_workflow_inner(*continuation, effectful_names, state)),
        },

        CoreWorkflow::Resume {
            target,
            continuation,
        } => CoreWorkflow::Resume {
            target,
            continuation: Box::new(lift_workflow_inner(*continuation, effectful_names, state)),
        },

        CoreWorkflow::CheckHealth {
            target,
            continuation,
        } => CoreWorkflow::CheckHealth {
            target,
            continuation: Box::new(lift_workflow_inner(*continuation, effectful_names, state)),
        },

        CoreWorkflow::Oblige { name, span } => CoreWorkflow::Oblige { name, span },

        CoreWorkflow::CheckObligation { name, span } => {
            CoreWorkflow::CheckObligation { name, span }
        }

        CoreWorkflow::Yield {
            role,
            request,
            expected_response_type,
            continuation,
            span,
            resume_var,
        } => CoreWorkflow::Yield {
            role,
            request,
            expected_response_type,
            continuation: Box::new(lift_workflow_inner(*continuation, effectful_names, state)),
            span,
            resume_var,
        },

        CoreWorkflow::ProxyResume {
            value,
            value_type,
            correlation_id,
            span,
        } => CoreWorkflow::ProxyResume {
            value,
            value_type,
            correlation_id,
            span,
        },

        CoreWorkflow::Done => CoreWorkflow::Done,
    }
}

/// Public entry point: lift effectful sub-expressions out of a core workflow.
///
/// Uses an empty effectful names set, so only qualified calls (module: Some(_))
/// and Spawn expressions are classified as effectful.
pub fn lift_workflow(workflow: CoreWorkflow) -> CoreWorkflow {
    lift_workflow_with_names(workflow, &std::collections::HashSet::new())
}

/// Public entry point: lift effectful sub-expressions out of a core workflow
/// using the provided set of effectful names for classifying unqualified calls.
pub fn lift_workflow_with_names(
    workflow: CoreWorkflow,
    effectful_names: &std::collections::HashSet<String>,
) -> CoreWorkflow {
    let mut state = LiftState { next_id: 0 };
    lift_workflow_inner(workflow, effectful_names, &mut state)
}

#[cfg(test)]
mod tests;
