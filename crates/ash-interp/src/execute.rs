//! Workflow execution engine
//!
//! Executes workflows in a runtime context, handling all workflow variants.

use ash_core::{Value, Workflow};

use crate::ExecResult;
use crate::capability::CapabilityContext;
use crate::context::Context;
use crate::error::{EvalError, ExecError};
use crate::eval::eval_expr;
use crate::guard::eval_guard;
use crate::pattern::match_pattern;
use crate::policy::PolicyEvaluator;

use std::future::Future;
use std::pin::Pin;

/// Boxed future type for recursive async execution
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Execute a workflow, returning the final value
///
/// This is the main entry point for workflow execution.
///
/// # Arguments
/// * `workflow` - The workflow to execute
/// * `ctx` - The runtime context with variable bindings
/// * `cap_ctx` - The capability context for external operations
/// * `policy_eval` - The policy evaluator for permission checks
///
/// # Examples
/// ```
/// use ash_core::{Workflow, Value};
/// use ash_interp::context::Context;
/// use ash_interp::capability::CapabilityContext;
/// use ash_interp::policy::PolicyEvaluator;
/// use ash_interp::execute::execute_workflow;
///
/// # tokio_test::block_on(async {
/// let ctx = Context::new();
/// let cap_ctx = CapabilityContext::new();
/// let policy_eval = PolicyEvaluator::new();
/// let workflow = Workflow::Done;
/// let result = execute_workflow(&workflow, ctx, &cap_ctx, &policy_eval).await.unwrap();
/// assert_eq!(result, Value::Null);
/// # });
/// ```
pub fn execute_workflow<'a>(
    workflow: &'a Workflow,
    ctx: Context,
    cap_ctx: &'a CapabilityContext,
    policy_eval: &'a PolicyEvaluator,
) -> BoxFuture<'a, ExecResult<Value>> {
    Box::pin(async move {
        match workflow {
            // Terminal workflow - returns null
            Workflow::Done => Ok(Value::Null),

            // Return with value
            Workflow::Ret { expr } => eval_expr(expr, &ctx).map_err(ExecError::Eval),

            // Variable binding
            Workflow::Let {
                pattern,
                expr,
                continuation,
            } => {
                let value = eval_expr(expr, &ctx).map_err(ExecError::Eval)?;
                let bindings =
                    match_pattern(pattern, &value).map_err(|_| ExecError::PatternMatchFailed {
                        pattern: format!("{:?}", pattern),
                        value: value.clone(),
                    })?;

                let mut new_ctx = ctx.extend();
                new_ctx.set_many(bindings);

                execute_workflow(continuation, new_ctx, cap_ctx, policy_eval).await
            }

            // Conditional execution
            Workflow::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_val = eval_expr(condition, &ctx).map_err(ExecError::Eval)?;
                match cond_val {
                    Value::Bool(true) => {
                        execute_workflow(then_branch, ctx, cap_ctx, policy_eval).await
                    }
                    Value::Bool(false) => {
                        execute_workflow(else_branch, ctx, cap_ctx, policy_eval).await
                    }
                    _ => Err(ExecError::Eval(EvalError::TypeMismatch {
                        expected: "bool".to_string(),
                        actual: format!("{:?}", cond_val),
                    })),
                }
            }

            // Sequential composition
            Workflow::Seq { first, second } => {
                let _ = execute_workflow(first, ctx.clone(), cap_ctx, policy_eval).await?;
                execute_workflow(second, ctx, cap_ctx, policy_eval).await
            }

            // Parallel composition
            Workflow::Par { workflows } => {
                if workflows.is_empty() {
                    return Ok(Value::Null);
                }

                // Execute all workflows in parallel and collect results
                let futures: Vec<_> = workflows
                    .iter()
                    .map(|wf| execute_workflow(wf, ctx.clone(), cap_ctx, policy_eval))
                    .collect();

                let results = futures::future::join_all(futures).await;

                // Check for errors
                let values: Vec<Value> = results.into_iter().collect::<Result<Vec<_>, _>>()?;

                Ok(Value::List(values))
            }

            // Observe from capability
            Workflow::Observe {
                capability,
                pattern,
                continuation,
            } => {
                let value = cap_ctx.observe(capability).await?;
                let bindings =
                    match_pattern(pattern, &value).map_err(|_| ExecError::PatternMatchFailed {
                        pattern: format!("{:?}", pattern),
                        value: value.clone(),
                    })?;

                let mut new_ctx = ctx.extend();
                new_ctx.set_many(bindings);

                execute_workflow(continuation, new_ctx, cap_ctx, policy_eval).await
            }

            // Orient - evaluate expression and continue
            Workflow::Orient { expr, continuation } => {
                let _ = eval_expr(expr, &ctx).map_err(ExecError::Eval)?;
                execute_workflow(continuation, ctx, cap_ctx, policy_eval).await
            }

            // Execute action with guard
            Workflow::Act {
                action,
                guard,
                provenance: _,
            } => {
                // Evaluate guard
                let guard_result = eval_guard(guard, &ctx).map_err(|_| ExecError::GuardFailed {
                    guard: format!("{:?}", guard),
                })?;

                if !guard_result {
                    return Err(ExecError::GuardFailed {
                        guard: format!("{:?}", guard),
                    });
                }

                // Execute action using a mock capability for now
                // In a real implementation, actions would be associated with specific capabilities
                Err(ExecError::ActionFailed {
                    action: action.name.clone(),
                    reason: "Action execution not implemented - use capability-based actions"
                        .to_string(),
                })
            }

            // Propose action (advisory - just continue)
            Workflow::Propose {
                action: _,
                continuation,
            } => {
                // Proposal is advisory - just continue
                execute_workflow(continuation, ctx, cap_ctx, policy_eval).await
            }

            // Decide under policy
            Workflow::Decide {
                expr,
                policy,
                continuation,
            } => {
                let value = eval_expr(expr, &ctx).map_err(ExecError::Eval)?;

                // Create a temporary context with the decision value
                let mut decision_ctx = ctx.extend();
                decision_ctx.set("decision_value".to_string(), value);

                let decision = policy_eval.evaluate(policy, &decision_ctx)?;

                match decision {
                    ash_core::Decision::Permit => {
                        execute_workflow(continuation, ctx, cap_ctx, policy_eval).await
                    }
                    ash_core::Decision::Deny => Err(ExecError::PolicyDenied {
                        policy: policy.clone(),
                    }),
                    ash_core::Decision::RequireApproval | ash_core::Decision::Escalate => {
                        // For now, escalate is treated as deny
                        Err(ExecError::PolicyDenied {
                            policy: policy.clone(),
                        })
                    }
                }
            }

            // For each iteration
            Workflow::ForEach {
                pattern,
                collection,
                body,
            } => {
                let coll_val = eval_expr(collection, &ctx).map_err(ExecError::Eval)?;

                match coll_val {
                    Value::List(items) => {
                        let mut last_result = Value::Null;

                        for item in items {
                            let bindings = match_pattern(pattern, &item).map_err(|_| {
                                ExecError::PatternMatchFailed {
                                    pattern: format!("{:?}", pattern),
                                    value: item.clone(),
                                }
                            })?;

                            let mut iter_ctx = ctx.extend();
                            iter_ctx.set_many(bindings);

                            last_result =
                                execute_workflow(body, iter_ctx, cap_ctx, policy_eval).await?;
                        }

                        Ok(last_result)
                    }
                    _ => Err(ExecError::Eval(EvalError::TypeMismatch {
                        expected: "list".to_string(),
                        actual: format!("{:?}", coll_val),
                    })),
                }
            }

            // With capability scope
            Workflow::With {
                capability: _,
                workflow,
            } => {
                // For now, just execute the workflow
                // In a full implementation, this would set up capability context
                execute_workflow(workflow, ctx, cap_ctx, policy_eval).await
            }

            // Maybe - try primary, fallback on failure
            Workflow::Maybe { primary, fallback } => {
                match execute_workflow(primary, ctx.clone(), cap_ctx, policy_eval).await {
                    Ok(result) => Ok(result),
                    Err(_) => execute_workflow(fallback, ctx, cap_ctx, policy_eval).await,
                }
            }

            // Must - fail if workflow fails
            Workflow::Must { workflow: inner } => {
                execute_workflow(inner, ctx, cap_ctx, policy_eval).await
            }

            // Check obligation (simplified - just continue)
            Workflow::Check {
                obligation: _,
                continuation,
            } => {
                // In a full implementation, this would check obligations
                execute_workflow(continuation, ctx, cap_ctx, policy_eval).await
            }

            // Obligate a role (simplified - just execute workflow)
            Workflow::Oblig {
                role: _,
                workflow: inner,
            } => execute_workflow(inner, ctx, cap_ctx, policy_eval).await,
        }
    })
}

/// Execute a workflow with default contexts (convenience function)
pub async fn execute_simple(workflow: &Workflow) -> ExecResult<Value> {
    let ctx = Context::new();
    let cap_ctx = CapabilityContext::new();
    let policy_eval = PolicyEvaluator::new();
    execute_workflow(workflow, ctx, &cap_ctx, &policy_eval).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::{Action, BinaryOp, Capability, Effect, Guard, Provenance};

    #[tokio::test]
    async fn test_execute_done() {
        let workflow = Workflow::Done;
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Null);
    }

    #[tokio::test]
    async fn test_execute_ret() {
        use ash_core::Expr;

        let workflow = Workflow::Ret {
            expr: Expr::Literal(Value::Int(42)),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_execute_let() {
        use ash_core::{Expr, Pattern};

        let workflow = Workflow::Let {
            pattern: Pattern::Variable("x".to_string()),
            expr: Expr::Literal(Value::Int(42)),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Variable("x".to_string()),
            }),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_execute_let_tuple() {
        use ash_core::{Expr, Pattern};

        let workflow = Workflow::Let {
            pattern: Pattern::Tuple(vec![
                Pattern::Variable("a".to_string()),
                Pattern::Variable("b".to_string()),
            ]),
            expr: Expr::Literal(Value::List(vec![Value::Int(1), Value::Int(2)])),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Variable("a".to_string())),
                    right: Box::new(Expr::Variable("b".to_string())),
                },
            }),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[tokio::test]
    async fn test_execute_if_true() {
        use ash_core::Expr;

        let workflow = Workflow::If {
            condition: Expr::Literal(Value::Bool(true)),
            then_branch: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(1)),
            }),
            else_branch: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(2)),
            }),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(1));
    }

    #[tokio::test]
    async fn test_execute_if_false() {
        use ash_core::Expr;

        let workflow = Workflow::If {
            condition: Expr::Literal(Value::Bool(false)),
            then_branch: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(1)),
            }),
            else_branch: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(2)),
            }),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(2));
    }

    #[tokio::test]
    async fn test_execute_seq_proper() {
        use ash_core::{Expr, Pattern};

        // Proper seq where first binds and second uses
        let workflow = Workflow::Let {
            pattern: Pattern::Variable("x".to_string()),
            expr: Expr::Literal(Value::Int(10)),
            continuation: Box::new(Workflow::Seq {
                first: Box::new(Workflow::Done),
                second: Box::new(Workflow::Ret {
                    expr: Expr::Variable("x".to_string()),
                }),
            }),
        };

        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(10));
    }

    #[tokio::test]
    async fn test_execute_par() {
        use ash_core::Expr;

        let workflow = Workflow::Par {
            workflows: vec![
                Workflow::Ret {
                    expr: Expr::Literal(Value::Int(1)),
                },
                Workflow::Ret {
                    expr: Expr::Literal(Value::Int(2)),
                },
                Workflow::Ret {
                    expr: Expr::Literal(Value::Int(3)),
                },
            ],
        };
        let result = execute_simple(&workflow).await.unwrap();

        // Result is a list of all workflow results
        match result {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
                assert!(items.contains(&Value::Int(1)));
                assert!(items.contains(&Value::Int(2)));
                assert!(items.contains(&Value::Int(3)));
            }
            _ => panic!("Expected list, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_execute_par_empty() {
        let workflow = Workflow::Par { workflows: vec![] };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Null);
    }

    #[tokio::test]
    async fn test_execute_foreach() {
        use ash_core::{Expr, Pattern};

        // ForEach iterates over a collection, executing body for each element
        // Each iteration gets its own context extended from the parent
        let workflow = Workflow::ForEach {
            pattern: Pattern::Variable("x".to_string()),
            collection: Expr::Literal(Value::List(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])),
            body: Box::new(Workflow::Ret {
                expr: Expr::Variable("x".to_string()),
            }),
        };

        let result = execute_simple(&workflow).await.unwrap();
        // Returns the result of the last iteration
        assert_eq!(result, Value::Int(3));
    }

    #[tokio::test]
    async fn test_execute_orient() {
        use ash_core::Expr;

        let workflow = Workflow::Orient {
            expr: Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Literal(Value::Int(1))),
                right: Box::new(Expr::Literal(Value::Int(2))),
            },
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(42)),
            }),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_execute_propose() {
        use ash_core::Expr;

        // Propose is advisory - just continues
        let workflow = Workflow::Propose {
            action: Action {
                name: "test".to_string(),
                arguments: vec![],
            },
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(42)),
            }),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_execute_maybe_success() {
        use ash_core::Expr;

        let workflow = Workflow::Maybe {
            primary: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(1)),
            }),
            fallback: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(2)),
            }),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(1));
    }

    #[tokio::test]
    async fn test_execute_maybe_fallback() {
        use ash_core::{Expr, Pattern};

        let workflow = Workflow::Maybe {
            primary: Box::new(Workflow::Let {
                pattern: Pattern::Variable("x".to_string()),
                expr: Expr::Literal(Value::Int(1)),
                continuation: Box::new(Workflow::Ret {
                    expr: Expr::Variable("undefined_var".to_string()), // Will fail
                }),
            }),
            fallback: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(2)),
            }),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(2));
    }

    #[tokio::test]
    async fn test_execute_must_success() {
        use ash_core::Expr;

        let workflow = Workflow::Must {
            workflow: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(42)),
            }),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_execute_must_failure() {
        use ash_core::Expr;

        let workflow = Workflow::Must {
            workflow: Box::new(Workflow::Ret {
                expr: Expr::Variable("undefined".to_string()),
            }),
        };
        let result = execute_simple(&workflow).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_with() {
        use ash_core::Expr;

        let workflow = Workflow::With {
            capability: Capability {
                name: "test".to_string(),
                effect: Effect::Epistemic,
                constraints: vec![],
            },
            workflow: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(42)),
            }),
        };
        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_execute_act_guard_fails() {
        let workflow = Workflow::Act {
            action: Action {
                name: "test".to_string(),
                arguments: vec![],
            },
            guard: Guard::Never,
            provenance: Provenance::new(),
        };
        let result = execute_simple(&workflow).await;
        assert!(matches!(result, Err(ExecError::GuardFailed { .. })));
    }

    #[tokio::test]
    async fn test_complex_workflow() {
        use ash_core::{Expr, Pattern};

        // let (x, y) = (10, 20) in
        //   if x < y then
        //     x + y
        //   else
        //     0
        let workflow = Workflow::Let {
            pattern: Pattern::Tuple(vec![
                Pattern::Variable("x".to_string()),
                Pattern::Variable("y".to_string()),
            ]),
            expr: Expr::Literal(Value::List(vec![Value::Int(10), Value::Int(20)])),
            continuation: Box::new(Workflow::If {
                condition: Expr::Binary {
                    op: BinaryOp::Lt,
                    left: Box::new(Expr::Variable("x".to_string())),
                    right: Box::new(Expr::Variable("y".to_string())),
                },
                then_branch: Box::new(Workflow::Ret {
                    expr: Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Variable("x".to_string())),
                        right: Box::new(Expr::Variable("y".to_string())),
                    },
                }),
                else_branch: Box::new(Workflow::Ret {
                    expr: Expr::Literal(Value::Int(0)),
                }),
            }),
        };

        let result = execute_simple(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(30));
    }
}
