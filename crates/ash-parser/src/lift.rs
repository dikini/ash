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
mod tests {
    use super::*;
    use ash_core::Expr as CoreExpr;
    use ash_core::Pattern as CorePattern;
    use ash_core::Value;
    use ash_core::Workflow as CoreWorkflow;

    #[test]
    fn lift_counter_resets_per_top_level_invocation() {
        let make_workflow = || CoreWorkflow::Let {
            pattern: CorePattern::Variable {
                name: "result".to_string(),
                span: default_span(),
            },
            expr: CoreExpr::Spawn {
                workflow_type: "worker".to_string(),
                init: Box::new(CoreExpr::Literal(Value::Null)),
            },
            continuation: Box::new(CoreWorkflow::Done),
        };

        for lifted in [
            lift_workflow(make_workflow()),
            lift_workflow(make_workflow()),
        ] {
            match lifted {
                CoreWorkflow::Let {
                    pattern: CorePattern::Variable { name, .. },
                    ..
                } => assert_eq!(name, "__lift_0"),
                other => panic!("expected synthetic lift binding, got {other:?}"),
            }
        }
    }

    #[test]
    fn lift_pipe_chain_read_dir_filter() {
        // let result = filter(read_dir(path), ends_with(".md"))
        // read_dir is effectful, ends_with(".md") is pure (partial builtin)
        let rhs = CoreExpr::FnApply {
            func: Box::new(CoreExpr::Variable {
                name: "filter".to_string(),
                span: default_span(),
            }),
            args: vec![
                CoreExpr::FnApply {
                    func: Box::new(CoreExpr::Variable {
                        name: "read_dir".to_string(),
                        span: default_span(),
                    }),
                    args: vec![CoreExpr::Variable {
                        name: "path".to_string(),
                        span: default_span(),
                    }],
                },
                CoreExpr::Call {
                    func: "ends_with".to_string(),
                    module: None,
                    arguments: vec![CoreExpr::Literal(Value::String(".md".to_string()))],
                },
            ],
        };

        let wf = CoreWorkflow::Let {
            pattern: CorePattern::Variable {
                name: "result".to_string(),
                span: default_span(),
            },
            expr: rhs,
            continuation: Box::new(CoreWorkflow::Done),
        };

        let names: std::collections::HashSet<String> =
            ["read_dir".to_string()].into_iter().collect();
        let lifted = lift_workflow_with_names(wf, &names);

        // Expect:
        // let __lift_0 = read_dir(path);
        // let result = filter(__lift_0, ends_with(".md"));
        // done
        match lifted {
            CoreWorkflow::Let {
                pattern: CorePattern::Variable { name: lift0, .. },
                expr: expr0,
                continuation: cont0,
            } => {
                assert!(lift0.starts_with("__lift_"));
                match expr0 {
                    CoreExpr::FnApply { func, args } => {
                        assert_eq!(
                            func.as_ref(),
                            &CoreExpr::Variable {
                                name: "read_dir".to_string(),
                                span: default_span(),
                            }
                        );
                        assert_eq!(args.len(), 1);
                    }
                    other => panic!("expected FnApply(read_dir), got {other:?}"),
                }
                match cont0.as_ref() {
                    CoreWorkflow::Let {
                        pattern: CorePattern::Variable { name: result, .. },
                        expr: result_expr,
                        continuation: cont1,
                    } => {
                        assert_eq!(result, "result");
                        match result_expr {
                            CoreExpr::FnApply { func, args } => {
                                assert_eq!(
                                    func.as_ref(),
                                    &CoreExpr::Variable {
                                        name: "filter".to_string(),
                                        span: default_span(),
                                    }
                                );
                                assert_eq!(args.len(), 2);
                                assert_eq!(
                                    args[0],
                                    CoreExpr::Variable {
                                        name: lift0.clone(),
                                        span: default_span(),
                                    }
                                );
                            }
                            other => panic!("expected FnApply(filter), got {other:?}"),
                        }
                        assert!(matches!(cont1.as_ref(), CoreWorkflow::Done));
                    }
                    other => panic!("expected inner Let, got {other:?}"),
                }
            }
            other => panic!("expected Let, got {other:?}"),
        }
    }

    #[test]
    fn lift_nested_effectful_calls() {
        // read_text(fetch_url(get_env("API_ENDPOINT")))
        let rhs = CoreExpr::FnApply {
            func: Box::new(CoreExpr::Variable {
                name: "read_text".to_string(),
                span: default_span(),
            }),
            args: vec![CoreExpr::FnApply {
                func: Box::new(CoreExpr::Variable {
                    name: "fetch_url".to_string(),
                    span: default_span(),
                }),
                args: vec![CoreExpr::FnApply {
                    func: Box::new(CoreExpr::Variable {
                        name: "get_env".to_string(),
                        span: default_span(),
                    }),
                    args: vec![CoreExpr::Literal(Value::String("API_ENDPOINT".to_string()))],
                }],
            }],
        };

        let wf = CoreWorkflow::Let {
            pattern: CorePattern::Variable {
                name: "body".to_string(),
                span: default_span(),
            },
            expr: rhs,
            continuation: Box::new(CoreWorkflow::Done),
        };

        let names: std::collections::HashSet<String> = [
            "read_text".to_string(),
            "fetch_url".to_string(),
            "get_env".to_string(),
        ]
        .into_iter()
        .collect();
        let lifted = lift_workflow_with_names(wf, &names);

        // Expect three synthetic lets, innermost first:
        // let __lift_0 = get_env("API_ENDPOINT");
        // let __lift_1 = fetch_url(__lift_0);
        // let __lift_2 = read_text(__lift_1);
        // let body = __lift_2;
        // done

        fn count_lets(wf: &CoreWorkflow) -> usize {
            match wf {
                CoreWorkflow::Let { continuation, .. } => 1 + count_lets(continuation),
                _ => 0,
            }
        }

        assert_eq!(count_lets(&lifted), 4);

        // Verify the innermost let binds get_env
        match lifted {
            CoreWorkflow::Let {
                expr: CoreExpr::FnApply { func, args: _ },
                continuation: cont,
                ..
            } => {
                assert_eq!(
                    func.as_ref(),
                    &CoreExpr::Variable {
                        name: "get_env".to_string(),
                        span: default_span(),
                    }
                );
                match cont.as_ref() {
                    CoreWorkflow::Let {
                        expr:
                            CoreExpr::FnApply {
                                func: f2,
                                args: _a2,
                            },
                        continuation: cont2,
                        ..
                    } => {
                        assert_eq!(
                            f2.as_ref(),
                            &CoreExpr::Variable {
                                name: "fetch_url".to_string(),
                                span: default_span(),
                            }
                        );
                        match cont2.as_ref() {
                            CoreWorkflow::Let {
                                expr:
                                    CoreExpr::FnApply {
                                        func: f3,
                                        args: _a3,
                                    },
                                continuation: cont3,
                                ..
                            } => {
                                assert_eq!(
                                    f3.as_ref(),
                                    &CoreExpr::Variable {
                                        name: "read_text".to_string(),
                                        span: default_span(),
                                    }
                                );
                                match cont3.as_ref() {
                                    CoreWorkflow::Let {
                                        pattern: CorePattern::Variable { name: body, .. },
                                        expr: CoreExpr::Variable { .. },
                                        continuation: last,
                                    } => {
                                        assert_eq!(body, "body");
                                        assert!(matches!(last.as_ref(), CoreWorkflow::Done));
                                    }
                                    other => panic!("expected body let, got {other:?}"),
                                }
                            }
                            other => panic!("expected read_text let, got {other:?}"),
                        }
                    }
                    other => panic!("expected fetch_url let, got {other:?}"),
                }
            }
            other => panic!("expected get_env let, got {other:?}"),
        }
    }

    #[test]
    fn lift_does_not_touch_pure_builtin_call() {
        // let x = len(list)
        let rhs = CoreExpr::Call {
            func: "len".to_string(),
            module: None,
            arguments: vec![CoreExpr::Variable {
                name: "list".to_string(),
                span: default_span(),
            }],
        };

        let wf = CoreWorkflow::Let {
            pattern: CorePattern::Variable {
                name: "x".to_string(),
                span: default_span(),
            },
            expr: rhs.clone(),
            continuation: Box::new(CoreWorkflow::Done),
        };

        let lifted = lift_workflow(wf);
        match lifted {
            CoreWorkflow::Let {
                pattern: CorePattern::Variable { name, .. },
                expr,
                continuation,
            } => {
                assert_eq!(name, "x");
                assert_eq!(expr, rhs);
                assert!(matches!(continuation.as_ref(), CoreWorkflow::Done));
            }
            other => panic!("expected simple Let, got {other:?}"),
        }
    }

    #[test]
    fn lift_qualified_capability_call() {
        // let x = io::read_dir(path)
        let rhs = CoreExpr::Call {
            func: "read_dir".to_string(),
            module: Some("io".to_string()),
            arguments: vec![CoreExpr::Variable {
                name: "path".to_string(),
                span: default_span(),
            }],
        };

        let wf = CoreWorkflow::Let {
            pattern: CorePattern::Variable {
                name: "x".to_string(),
                span: default_span(),
            },
            expr: rhs,
            continuation: Box::new(CoreWorkflow::Done),
        };

        let lifted = lift_workflow(wf);
        match lifted {
            CoreWorkflow::Let {
                pattern: CorePattern::Variable { name: lift_var, .. },
                expr: CoreExpr::Call {
                    module: Some(_m), ..
                },
                continuation: cont,
            } => {
                assert!(lift_var.starts_with("__lift_"));
                match cont.as_ref() {
                    CoreWorkflow::Let {
                        pattern: CorePattern::Variable { name: x, .. },
                        expr: CoreExpr::Variable { name: v, .. },
                        continuation: last,
                    } => {
                        assert_eq!(x, "x");
                        assert_eq!(v, &lift_var);
                        assert!(matches!(last.as_ref(), CoreWorkflow::Done));
                    }
                    other => panic!("expected x let, got {other:?}"),
                }
            }
            other => panic!("expected lifted let, got {other:?}"),
        }
    }

    #[test]
    fn lift_ret_with_effectful_expr_preserves_original_expr() {
        let expr = CoreExpr::Call {
            func: "read_dir".to_string(),
            module: Some("io".to_string()),
            arguments: vec![CoreExpr::Variable {
                name: "path".to_string(),
                span: default_span(),
            }],
        };

        let wf = CoreWorkflow::Ret { expr: expr.clone() };

        let lifted = lift_workflow(wf);
        assert_eq!(lifted, CoreWorkflow::Ret { expr });
    }

    // ---------------------------------------------------------------------------
    // TASK-608: Regression tests for the conservative lifting contract.
    //
    // These tests verify that the lifting pass:
    //   1. Never panics on any workflow form, even with effectful expressions
    //      in "awkward" (unsupported) positions.
    //   2. Conservatively preserves the original expression in positions where
    //      lifting would require synthetic bindings that cannot be hosted.
    // ---------------------------------------------------------------------------

    /// Helper: construct an expression recognized as effectful by the lifter.
    ///
    /// Uses a qualified capability call (`io::read_file`) which `is_effectful`
    /// always flags.
    fn make_effectful_expr() -> CoreExpr {
        CoreExpr::Call {
            func: "read_file".to_string(),
            module: Some("io".to_string()),
            arguments: vec![CoreExpr::Literal(Value::String("test".into()))],
        }
    }

    // (a) Ret ---------------------------------------------------------------

    #[test]
    fn lifting_preserves_effectful_ret_expression() {
        let effectful = make_effectful_expr();
        let wf = CoreWorkflow::Ret {
            expr: effectful.clone(),
        };
        let lifted = lift_workflow(wf);
        // Ret cannot host synthetic Let bindings, so original is preserved.
        match lifted {
            CoreWorkflow::Ret { expr } => assert_eq!(expr, effectful),
            other => panic!("expected Ret, got {other:?}"),
        }
    }

    // (b) If condition ------------------------------------------------------

    #[test]
    fn lifting_preserves_effectful_if_condition() {
        let effectful = make_effectful_expr();
        let wf = CoreWorkflow::If {
            condition: effectful.clone(),
            then_branch: Box::new(CoreWorkflow::Done),
            else_branch: Box::new(CoreWorkflow::Done),
        };
        let lifted = lift_workflow(wf);
        match lifted {
            CoreWorkflow::If {
                condition,
                then_branch,
                else_branch,
            } => {
                assert_eq!(condition, effectful);
                assert!(matches!(then_branch.as_ref(), CoreWorkflow::Done));
                assert!(matches!(else_branch.as_ref(), CoreWorkflow::Done));
            }
            other => panic!("expected If, got {other:?}"),
        }
    }

    // (c) ForEach collection ------------------------------------------------

    #[test]
    fn lifting_preserves_effectful_foreach_collection() {
        let effectful = make_effectful_expr();
        let wf = CoreWorkflow::ForEach {
            pattern: CorePattern::Variable {
                name: "x".into(),
                span: default_span(),
            },
            collection: effectful.clone(),
            body: Box::new(CoreWorkflow::Done),
        };
        let lifted = lift_workflow(wf);
        match lifted {
            CoreWorkflow::ForEach {
                collection, body, ..
            } => {
                assert_eq!(collection, effectful);
                assert!(matches!(body.as_ref(), CoreWorkflow::Done));
            }
            other => panic!("expected ForEach, got {other:?}"),
        }
    }

    // (d) Receive arm guard (blocking / "must") ----------------------------

    #[test]
    fn lifting_preserves_effectful_must_guard() {
        use ash_core::{ReceiveArm, ReceiveMode, ReceivePattern};

        let effectful = make_effectful_expr();
        let wf = CoreWorkflow::Receive {
            mode: ReceiveMode::Blocking(None),
            arms: vec![ReceiveArm {
                pattern: ReceivePattern::Wildcard,
                guard: Some(effectful.clone()),
                body: CoreWorkflow::Done,
            }],
            control: false,
        };
        let lifted = lift_workflow(wf);
        match lifted {
            CoreWorkflow::Receive { arms, .. } => {
                assert_eq!(arms.len(), 1);
                assert_eq!(arms[0].guard, Some(effectful));
                assert!(matches!(arms[0].body, CoreWorkflow::Done));
            }
            other => panic!("expected Receive, got {other:?}"),
        }
    }

    // (e) Receive arm guard (non-blocking / "maybe") -----------------------

    #[test]
    fn lifting_preserves_effectful_maybe_guard() {
        use ash_core::{ReceiveArm, ReceiveMode, ReceivePattern};

        let effectful = make_effectful_expr();
        let wf = CoreWorkflow::Receive {
            mode: ReceiveMode::NonBlocking,
            arms: vec![ReceiveArm {
                pattern: ReceivePattern::Wildcard,
                guard: Some(effectful.clone()),
                body: CoreWorkflow::Done,
            }],
            control: false,
        };
        let lifted = lift_workflow(wf);
        match lifted {
            CoreWorkflow::Receive { arms, .. } => {
                assert_eq!(arms.len(), 1);
                assert_eq!(arms[0].guard, Some(effectful));
            }
            other => panic!("expected Receive, got {other:?}"),
        }
    }

    // (f) Send payload ------------------------------------------------------

    #[test]
    fn lifting_preserves_effectful_send_payload() {
        let effectful = make_effectful_expr();
        let wf = CoreWorkflow::Send {
            capability: "cap".into(),
            channel: "ch".into(),
            value: effectful.clone(),
        };
        let lifted = lift_workflow(wf);
        match lifted {
            CoreWorkflow::Send { value, .. } => assert_eq!(value, effectful),
            other => panic!("expected Send, got {other:?}"),
        }
    }

    // (g) Spawn body (init expression) -------------------------------------

    #[test]
    fn lifting_preserves_effectful_spawn_body() {
        let effectful = make_effectful_expr();
        let wf = CoreWorkflow::Spawn {
            workflow_type: "MyWorkflow".into(),
            init: effectful.clone(),
            pattern: CorePattern::Variable {
                name: "inst".into(),
                span: default_span(),
            },
            continuation: Box::new(CoreWorkflow::Done),
        };
        let lifted = lift_workflow(wf);
        match lifted {
            CoreWorkflow::Spawn { init, .. } => assert_eq!(init, effectful),
            other => panic!("expected Spawn, got {other:?}"),
        }
    }

    // (h) Split handler (expr) ---------------------------------------------

    #[test]
    fn lifting_preserves_effectful_split_handler() {
        let effectful = make_effectful_expr();
        let wf = CoreWorkflow::Split {
            expr: effectful.clone(),
            pattern: CorePattern::Variable {
                name: "part".into(),
                span: default_span(),
            },
            continuation: Box::new(CoreWorkflow::Done),
        };
        let lifted = lift_workflow(wf);
        match lifted {
            CoreWorkflow::Split { expr, .. } => assert_eq!(expr, effectful),
            other => panic!("expected Split, got {other:?}"),
        }
    }

    // (i) Workflow::Call arguments -----------------------------------------

    #[test]
    fn lifting_preserves_effectful_workflow_call_arguments() {
        let effectful = make_effectful_expr();
        let wf = CoreWorkflow::Call {
            target: "other_wf".into(),
            arguments: vec![effectful.clone()],
            continuation: Box::new(CoreWorkflow::Done),
        };
        let lifted = lift_workflow(wf);
        match lifted {
            CoreWorkflow::Call { arguments, .. } => {
                assert_eq!(arguments.len(), 1);
                assert_eq!(arguments[0], effectful);
            }
            other => panic!("expected Call, got {other:?}"),
        }
    }

    // (j) Sweep test: every Workflow variant with effectful expression ------

    #[test]
    fn lifting_does_not_panic_on_any_workflow_form() {
        use ash_core::workflow_contract::{Span as WcSpan, TypeExpr};
        use ash_core::{
            Capability, CorrelationId, Effect, Guard, ReceiveArm, ReceiveMode, ReceivePattern, Role,
        };

        let eff = || make_effectful_expr();

        let cap = Capability {
            name: "cap".into(),
            effect: Effect::Epistemic,
            constraints: vec![],
        };

        let role = Role {
            name: "admin".into(),
            authority: vec![],
            obligations: vec![],
        };

        let prov = ash_core::Provenance::new();

        let wc_span = WcSpan { start: 0, end: 0 };

        // Collect every variant into a vec and run lift_workflow on each.
        let workflows: Vec<CoreWorkflow> = vec![
            // Observe — no Expr field, but continuation contains a Ret with effectful expr
            CoreWorkflow::Observe {
                capability: cap.clone(),
                pattern: CorePattern::Wildcard,
                continuation: Box::new(CoreWorkflow::Ret { expr: eff() }),
            },
            // Receive — arm guard is effectful
            CoreWorkflow::Receive {
                mode: ReceiveMode::Blocking(None),
                arms: vec![ReceiveArm {
                    pattern: ReceivePattern::Wildcard,
                    guard: Some(eff()),
                    body: CoreWorkflow::Done,
                }],
                control: false,
            },
            // Orient — expr is effectful (preserved)
            CoreWorkflow::Orient {
                expr: eff(),
                continuation: Box::new(CoreWorkflow::Done),
            },
            // Propose — action_arguments contain effectful
            CoreWorkflow::Propose {
                action_name: "act".into(),
                action_arguments: vec![eff()],
                continuation: Box::new(CoreWorkflow::Done),
            },
            // Decide — expr is effectful (preserved)
            CoreWorkflow::Decide {
                expr: eff(),
                policy: "policy".into(),
                continuation: Box::new(CoreWorkflow::Done),
            },
            // Check — no Expr field
            CoreWorkflow::Check {
                obligation: ash_core::Obligation::Obliged {
                    role: role.clone(),
                    condition: eff(),
                },
                continuation: Box::new(CoreWorkflow::Done),
            },
            // Act — arguments contain effectful
            CoreWorkflow::Act {
                provider_name: "prov".into(),
                action_name: "do_thing".into(),
                arguments: vec![eff()],
                guard: Guard::Always,
                provenance: prov,
                result_name: None,
                continuation: Box::new(CoreWorkflow::Done),
            },
            // Call — arguments contain effectful
            CoreWorkflow::Call {
                target: "wf".into(),
                arguments: vec![eff()],
                continuation: Box::new(CoreWorkflow::Done),
            },
            // Oblig — wraps a workflow with effectful Ret
            CoreWorkflow::Oblig {
                role: role.clone(),
                workflow: Box::new(CoreWorkflow::Ret { expr: eff() }),
            },
            // Let — supported position, effectful gets synthetic binding
            CoreWorkflow::Let {
                pattern: CorePattern::Variable {
                    name: "x".into(),
                    span: default_span(),
                },
                expr: eff(),
                continuation: Box::new(CoreWorkflow::Done),
            },
            // If — condition is effectful (preserved)
            CoreWorkflow::If {
                condition: eff(),
                then_branch: Box::new(CoreWorkflow::Done),
                else_branch: Box::new(CoreWorkflow::Done),
            },
            // Seq — both branches have effectful content
            CoreWorkflow::Seq {
                first: Box::new(CoreWorkflow::Ret { expr: eff() }),
                second: Box::new(CoreWorkflow::Ret { expr: eff() }),
            },
            // ForEach — collection is effectful (preserved)
            CoreWorkflow::ForEach {
                pattern: CorePattern::Variable {
                    name: "item".into(),
                    span: default_span(),
                },
                collection: eff(),
                body: Box::new(CoreWorkflow::Done),
            },
            // Ret — expr is effectful (preserved)
            CoreWorkflow::Ret { expr: eff() },
            // With — wraps workflow with effectful Ret
            CoreWorkflow::With {
                capability: cap.clone(),
                workflow: Box::new(CoreWorkflow::Ret { expr: eff() }),
            },
            // Maybe — primary has effectful Ret
            CoreWorkflow::Maybe {
                primary: Box::new(CoreWorkflow::Ret { expr: eff() }),
                fallback: Box::new(CoreWorkflow::Done),
            },
            // Must — wraps workflow with effectful Ret
            CoreWorkflow::Must {
                workflow: Box::new(CoreWorkflow::Ret { expr: eff() }),
            },
            // Set — value is effectful (preserved)
            CoreWorkflow::Set {
                capability: "cap".into(),
                channel: "ch".into(),
                value: eff(),
            },
            // Send — value is effectful (preserved)
            CoreWorkflow::Send {
                capability: "cap".into(),
                channel: "ch".into(),
                value: eff(),
            },
            // Spawn — init is effectful (preserved)
            CoreWorkflow::Spawn {
                workflow_type: "Wf".into(),
                init: eff(),
                pattern: CorePattern::Variable {
                    name: "inst".into(),
                    span: default_span(),
                },
                continuation: Box::new(CoreWorkflow::Done),
            },
            // Split — expr is effectful (preserved)
            CoreWorkflow::Split {
                expr: eff(),
                pattern: CorePattern::Variable {
                    name: "part".into(),
                    span: default_span(),
                },
                continuation: Box::new(CoreWorkflow::Done),
            },
            // Kill — no Expr field
            CoreWorkflow::Kill {
                target: "inst".into(),
                continuation: Box::new(CoreWorkflow::Done),
            },
            // Pause — no Expr field
            CoreWorkflow::Pause {
                target: "inst".into(),
                continuation: Box::new(CoreWorkflow::Done),
            },
            // Resume — no Expr field
            CoreWorkflow::Resume {
                target: "inst".into(),
                continuation: Box::new(CoreWorkflow::Done),
            },
            // CheckHealth — no Expr field
            CoreWorkflow::CheckHealth {
                target: "inst".into(),
                continuation: Box::new(CoreWorkflow::Done),
            },
            // Oblige — no Expr field
            CoreWorkflow::Oblige {
                name: "obl".into(),
                span: wc_span,
            },
            // CheckObligation — no Expr field
            CoreWorkflow::CheckObligation {
                name: "obl".into(),
                span: wc_span,
            },
            // Yield — request is effectful (passed through, not preserved via
            // preserve_original_if_bindings but should not panic)
            CoreWorkflow::Yield {
                role: "handler".into(),
                request: Box::new(eff()),
                expected_response_type: TypeExpr::Named("String".into()),
                continuation: Box::new(CoreWorkflow::Done),
                span: default_span(),
                resume_var: "resp".into(),
            },
            // ProxyResume — value is effectful (passed through, no panic)
            CoreWorkflow::ProxyResume {
                value: Box::new(eff()),
                value_type: TypeExpr::Named("String".into()),
                correlation_id: CorrelationId(42),
                span: default_span(),
            },
            // Done — terminal
            CoreWorkflow::Done,
        ];

        // The primary assertion: lift_workflow completes without panicking
        // on every single workflow variant.
        for (i, wf) in workflows.into_iter().enumerate() {
            let _lifted = lift_workflow(wf);
            // If we reach here, no panic occurred for variant #{i}.
            // The result is intentionally discarded — this is a sweep test.
            let _ = i; // use the index to suppress unused warning
        }
    }

    // -----------------------------------------------------------------------
    // TASK-609: Tests for capability-registry-based effectful classification.
    // -----------------------------------------------------------------------

    #[test]
    fn lifting_classifies_qualified_call_as_effectful_regardless_of_names() {
        // Qualified call is effectful even with empty names set.
        let rhs = CoreExpr::Call {
            func: "read_dir".to_string(),
            module: Some("io".to_string()),
            arguments: vec![CoreExpr::Variable {
                name: "path".to_string(),
                span: default_span(),
            }],
        };
        let wf = CoreWorkflow::Let {
            pattern: CorePattern::Variable {
                name: "x".to_string(),
                span: default_span(),
            },
            expr: rhs,
            continuation: Box::new(CoreWorkflow::Done),
        };

        let empty: std::collections::HashSet<String> = std::collections::HashSet::new();
        let lifted = lift_workflow_with_names(wf, &empty);

        // Should produce a synthetic let binding
        match &lifted {
            CoreWorkflow::Let {
                pattern: CorePattern::Variable { name: lift_var, .. },
                expr: CoreExpr::Call {
                    module: Some(_), ..
                },
                ..
            } => {
                assert!(lift_var.starts_with("__lift_"));
            }
            other => panic!("expected lifted qualified call, got {other:?}"),
        }
    }

    #[test]
    fn lifting_classifies_unqualified_call_from_capability_names() {
        // Unqualified call to a name in the effectful set IS lifted.
        let rhs = CoreExpr::FnApply {
            func: Box::new(CoreExpr::Variable {
                name: "my_io_action".to_string(),
                span: default_span(),
            }),
            args: vec![CoreExpr::Literal(Value::String("input".to_string()))],
        };
        let wf = CoreWorkflow::Let {
            pattern: CorePattern::Variable {
                name: "result".to_string(),
                span: default_span(),
            },
            expr: rhs,
            continuation: Box::new(CoreWorkflow::Done),
        };

        let names: std::collections::HashSet<String> =
            ["my_io_action".to_string()].into_iter().collect();
        let lifted = lift_workflow_with_names(wf, &names);

        // Should produce a synthetic let binding
        match &lifted {
            CoreWorkflow::Let {
                pattern: CorePattern::Variable { name: lift_var, .. },
                expr: CoreExpr::FnApply { .. },
                ..
            } => {
                assert!(lift_var.starts_with("__lift_"));
            }
            other => panic!("expected lifted FnApply, got {other:?}"),
        }
    }

    #[test]
    fn lifting_treats_unknown_unqualified_call_as_pure() {
        // Unqualified call NOT in names set is NOT lifted.
        let rhs = CoreExpr::FnApply {
            func: Box::new(CoreExpr::Variable {
                name: "pure_func".to_string(),
                span: default_span(),
            }),
            args: vec![CoreExpr::Literal(Value::Int(42))],
        };
        let expected_rhs = rhs.clone();
        let wf = CoreWorkflow::Let {
            pattern: CorePattern::Variable {
                name: "result".to_string(),
                span: default_span(),
            },
            expr: rhs,
            continuation: Box::new(CoreWorkflow::Done),
        };

        let names: std::collections::HashSet<String> =
            ["some_other_func".to_string()].into_iter().collect();
        let lifted = lift_workflow_with_names(wf, &names);

        // Should NOT produce a synthetic let binding — expression passes through unchanged
        match &lifted {
            CoreWorkflow::Let {
                pattern: CorePattern::Variable { name, .. },
                expr,
                continuation,
            } => {
                assert_eq!(name, "result");
                assert_eq!(*expr, expected_rhs);
                assert!(matches!(continuation.as_ref(), CoreWorkflow::Done));
            }
            other => panic!("expected unchanged Let, got {other:?}"),
        }
    }

    #[test]
    fn lifting_no_false_positive_on_user_shadowed_name() {
        // Call to "read_dir" is NOT lifted when "read_dir" is NOT in the names set
        // (simulating user-defined shadow).
        let rhs = CoreExpr::FnApply {
            func: Box::new(CoreExpr::Variable {
                name: "read_dir".to_string(),
                span: default_span(),
            }),
            args: vec![CoreExpr::Variable {
                name: "path".to_string(),
                span: default_span(),
            }],
        };
        let expected_rhs = rhs.clone();
        let wf = CoreWorkflow::Let {
            pattern: CorePattern::Variable {
                name: "entries".to_string(),
                span: default_span(),
            },
            expr: rhs,
            continuation: Box::new(CoreWorkflow::Done),
        };

        let empty: std::collections::HashSet<String> = std::collections::HashSet::new();
        let lifted = lift_workflow_with_names(wf, &empty);

        // Should NOT produce a synthetic let binding — read_dir is not in the set
        match &lifted {
            CoreWorkflow::Let {
                pattern: CorePattern::Variable { name, .. },
                expr,
                continuation,
            } => {
                assert_eq!(name, "entries");
                assert_eq!(*expr, expected_rhs);
                assert!(matches!(continuation.as_ref(), CoreWorkflow::Done));
            }
            other => panic!("expected unchanged Let, got {other:?}"),
        }
    }

    #[test]
    fn effectful_names_from_definitions_extracts_capability_actions() {
        use crate::lower::effectful_names_from_definitions;
        use crate::surface::{CapabilityDef, Definition, EffectType, Param, Visibility};

        let cap_def = CapabilityDef {
            visibility: Visibility::Public,
            name: "read_dir".into(),
            effect: EffectType::Operational,
            params: vec![Param {
                name: "path".into(),
                ty: crate::surface::Type::Name("String".into()),
            }],
            return_type: None,
            constraints: vec![],
            target_provider: Some("fs".into()),
            target_action: Some("list_dir".into()),
            span: crate::token::Span::default(),
        };

        let definitions = vec![Definition::Capability(cap_def)];
        let names = effectful_names_from_definitions(&definitions);

        assert!(names.contains("read_dir"));
        assert!(names.contains("list_dir"));
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn lifting_match_arm_bindings_do_not_leak_to_outer_scope() {
        // Construct: match x { Some(v) => process(v) }
        // where "process" is in effectful_names.
        // The arm body references `v` (pattern-bound), so the effectful call
        // must NOT produce a synthetic __lift_ binding outside the Match.
        use ash_core::MatchArm;

        let arm_body = CoreExpr::FnApply {
            func: Box::new(CoreExpr::Variable {
                name: "process".to_string(),
                span: default_span(),
            }),
            args: vec![CoreExpr::Variable {
                name: "v".to_string(),
                span: default_span(),
            }],
        };

        let match_expr = CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Variable {
                name: "x".to_string(),
                span: default_span(),
            }),
            arms: vec![MatchArm {
                pattern: CorePattern::Variable {
                    name: "v".to_string(),
                    span: default_span(),
                },
                body: arm_body,
            }],
        };

        let wf = CoreWorkflow::Let {
            pattern: CorePattern::Variable {
                name: "result".to_string(),
                span: default_span(),
            },
            expr: match_expr,
            continuation: Box::new(CoreWorkflow::Done),
        };

        let names: std::collections::HashSet<String> =
            ["process".to_string()].into_iter().collect();
        let lifted = lift_workflow_with_names(wf, &names);

        // The outer workflow should be a single Let with no synthetic __lift_ bindings.
        match &lifted {
            CoreWorkflow::Let {
                pattern: CorePattern::Variable { name, .. },
                expr,
                continuation,
            } => {
                assert_eq!(name, "result");
                // The expr should still be a Match (no synthetic bindings hoisted out).
                match expr {
                    CoreExpr::Match { arms, .. } => {
                        assert_eq!(arms.len(), 1);
                        // The arm body must be the ORIGINAL FnApply expression, not a
                        // synthetic __lift_ variable, because the binding would reference
                        // pattern-bound `v` and cannot be hosted in the outer scope.
                        match &arms[0].body {
                            CoreExpr::FnApply { func, args } => {
                                // Verify it's the original `process(v)` call
                                match func.as_ref() {
                                    CoreExpr::Variable { name, .. } => {
                                        assert_eq!(name, "process");
                                    }
                                    other => {
                                        panic!(
                                            "expected func to be 'process' variable, got: {other:?}"
                                        )
                                    }
                                }
                                assert_eq!(args.len(), 1);
                                match &args[0] {
                                    CoreExpr::Variable { name, .. } => {
                                        assert_eq!(name, "v");
                                    }
                                    other => {
                                        panic!("expected arg to be 'v' variable, got: {other:?}")
                                    }
                                }
                            }
                            other => {
                                panic!(
                                    "expected arm body to be the original FnApply, got: {other:?}"
                                )
                            }
                        }
                    }
                    other => panic!("expected Match expr, got: {other:?}"),
                }
                assert!(
                    matches!(continuation.as_ref(), CoreWorkflow::Done),
                    "expected Done continuation, got: {continuation:?}"
                );
            }
            other => panic!("expected single Let, got: {other:?}"),
        }
    }

    #[test]
    fn effectful_names_from_definitions_skips_pure_capabilities() {
        use crate::lower::effectful_names_from_definitions;
        use crate::surface::{CapabilityDef, Definition, EffectType, Visibility};

        let cap_def = CapabilityDef {
            visibility: Visibility::Public,
            name: "observe_data".into(),
            effect: EffectType::Epistemic,
            params: vec![],
            return_type: None,
            constraints: vec![],
            target_provider: None,
            target_action: None,
            span: crate::token::Span::default(),
        };

        let definitions = vec![Definition::Capability(cap_def)];
        let names = effectful_names_from_definitions(&definitions);

        // Epistemic with no target_action should not be in the set
        assert!(names.is_empty());
    }

    #[test]
    fn lifting_iflet_branch_preserves_original_on_effectful() {
        // Construct: if let Some(v) = x then process(v) else nothing
        // where "process" is in effectful_names.
        // The then_branch body references `v` (pattern-bound), so the effectful
        // call must NOT produce a synthetic __lift_ variable that would be unbound.

        let then_body = CoreExpr::FnApply {
            func: Box::new(CoreExpr::Variable {
                name: "process".to_string(),
                span: default_span(),
            }),
            args: vec![CoreExpr::Variable {
                name: "v".to_string(),
                span: default_span(),
            }],
        };

        let else_body = CoreExpr::Variable {
            name: "nothing".to_string(),
            span: default_span(),
        };

        let iflet_expr = CoreExpr::IfLet {
            pattern: CorePattern::Variable {
                name: "v".to_string(),
                span: default_span(),
            },
            expr: Box::new(CoreExpr::Variable {
                name: "x".to_string(),
                span: default_span(),
            }),
            then_branch: Box::new(then_body),
            else_branch: Box::new(else_body),
        };

        let wf = CoreWorkflow::Let {
            pattern: CorePattern::Variable {
                name: "result".to_string(),
                span: default_span(),
            },
            expr: iflet_expr,
            continuation: Box::new(CoreWorkflow::Done),
        };

        let names: std::collections::HashSet<String> =
            ["process".to_string()].into_iter().collect();
        let lifted = lift_workflow_with_names(wf, &names);

        // The outer workflow should be a single Let with no synthetic __lift_ bindings.
        match &lifted {
            CoreWorkflow::Let {
                pattern: CorePattern::Variable { name, .. },
                expr,
                continuation,
            } => {
                assert_eq!(name, "result");
                match expr {
                    CoreExpr::IfLet {
                        then_branch,
                        else_branch,
                        ..
                    } => {
                        // The then_branch must be the ORIGINAL FnApply expression,
                        // not a synthetic __lift_ variable.
                        match then_branch.as_ref() {
                            CoreExpr::FnApply { func, args } => {
                                match func.as_ref() {
                                    CoreExpr::Variable { name, .. } => {
                                        assert_eq!(name, "process");
                                    }
                                    other => {
                                        panic!(
                                            "expected func to be 'process' variable, got: {other:?}"
                                        )
                                    }
                                }
                                assert_eq!(args.len(), 1);
                                match &args[0] {
                                    CoreExpr::Variable { name, .. } => {
                                        assert_eq!(name, "v");
                                    }
                                    other => {
                                        panic!("expected arg to be 'v' variable, got: {other:?}")
                                    }
                                }
                            }
                            other => {
                                panic!(
                                    "expected then_branch to be the original FnApply, got: {other:?}"
                                )
                            }
                        }
                        // The else_branch should remain the plain "nothing" variable (pure).
                        match else_branch.as_ref() {
                            CoreExpr::Variable { name, .. } => {
                                assert_eq!(name, "nothing");
                            }
                            other => {
                                panic!(
                                    "expected else_branch to be 'nothing' variable, got: {other:?}"
                                )
                            }
                        }
                    }
                    other => panic!("expected IfLet expr, got: {other:?}"),
                }
                assert!(
                    matches!(continuation.as_ref(), CoreWorkflow::Done),
                    "expected Done continuation, got: {continuation:?}"
                );
            }
            other => panic!("expected single Let, got: {other:?}"),
        }
    }

    #[test]
    fn lifting_fndef_body_preserves_original_on_effectful() {
        // Construct: fn(x) -> process(x)
        // where "process" is in effectful_names.
        // The body references `x` (closure parameter), so the effectful call
        // must NOT produce a synthetic __lift_ variable that would be unbound.

        let fn_body = CoreExpr::FnApply {
            func: Box::new(CoreExpr::Variable {
                name: "process".to_string(),
                span: default_span(),
            }),
            args: vec![CoreExpr::Variable {
                name: "x".to_string(),
                span: default_span(),
            }],
        };

        let fndef_expr = CoreExpr::FnDef {
            params: vec![("x".to_string(), None)],
            return_type: None,
            body: Box::new(fn_body),
        };

        let wf = CoreWorkflow::Let {
            pattern: CorePattern::Variable {
                name: "my_fn".to_string(),
                span: default_span(),
            },
            expr: fndef_expr,
            continuation: Box::new(CoreWorkflow::Done),
        };

        let names: std::collections::HashSet<String> =
            ["process".to_string()].into_iter().collect();
        let lifted = lift_workflow_with_names(wf, &names);

        // The outer workflow should be a single Let with no synthetic __lift_ bindings.
        match &lifted {
            CoreWorkflow::Let {
                pattern: CorePattern::Variable { name, .. },
                expr,
                continuation,
            } => {
                assert_eq!(name, "my_fn");
                match expr {
                    CoreExpr::FnDef { body, .. } => {
                        // The body must be the ORIGINAL FnApply expression,
                        // not a synthetic __lift_ variable.
                        match body.as_ref() {
                            CoreExpr::FnApply { func, args } => {
                                match func.as_ref() {
                                    CoreExpr::Variable { name, .. } => {
                                        assert_eq!(name, "process");
                                    }
                                    other => {
                                        panic!(
                                            "expected func to be 'process' variable, got: {other:?}"
                                        )
                                    }
                                }
                                assert_eq!(args.len(), 1);
                                match &args[0] {
                                    CoreExpr::Variable { name, .. } => {
                                        assert_eq!(name, "x");
                                    }
                                    other => {
                                        panic!("expected arg to be 'x' variable, got: {other:?}")
                                    }
                                }
                            }
                            other => {
                                panic!("expected body to be the original FnApply, got: {other:?}")
                            }
                        }
                    }
                    other => panic!("expected FnDef expr, got: {other:?}"),
                }
                assert!(
                    matches!(continuation.as_ref(), CoreWorkflow::Done),
                    "expected Done continuation, got: {continuation:?}"
                );
            }
            other => panic!("expected single Let, got: {other:?}"),
        }
    }
}
