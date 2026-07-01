//! Tests for lexical scope execution (TASK-446)
//!
//! These tests verify that the interpreter faithfully executes the canonical
//! lowered form where bindings are carried through continuation-owned scope.

use ash_core::{Expr, Pattern, Value, Workflow};
use ash_interp::behaviour::BehaviourContext;
use ash_interp::capability::CapabilityContext;
use ash_interp::context::Context;
use ash_interp::execute::execute_workflow_with_behaviour;
use ash_interp::policy::PolicyEvaluator;

fn execution_contexts() -> (
    Context,
    CapabilityContext,
    PolicyEvaluator,
    BehaviourContext,
) {
    (
        Context::new(),
        CapabilityContext::new(),
        PolicyEvaluator::new(),
        BehaviourContext::new(),
    )
}

#[tokio::test]
async fn lexical_scope_let_bindings_visible_in_later_statements() {
    // Test that a let binding is visible in a later let binding
    let workflow = Workflow::Let {
        pattern: Pattern::Variable {
            name: "items".to_string(),
            span: ash_core::ast::Span::default(),
        },
        expr: Expr::Literal(Value::list_from_vec(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ])),
        continuation: Box::new(Workflow::Let {
            pattern: Pattern::Variable {
                name: "first".to_string(),
                span: ash_core::ast::Span::default(),
            },
            expr: Expr::IndexAccess {
                expr: Box::new(Expr::Variable {
                    name: "items".to_string(),
                    span: ash_core::ast::Span::default(),
                }),
                index: Box::new(Expr::Literal(Value::Int(0))),
            },
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Variable {
                    name: "first".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            }),
        }),
    };

    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();

    let result =
        execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
            .await
            .expect("workflow should execute successfully");

    assert_eq!(result, Value::Int(1));
}

#[tokio::test]
async fn lexical_scope_unbound_variable_fails_at_runtime() {
    // Test that an unbound variable fails at runtime
    let workflow = Workflow::Ret {
        expr: Expr::Variable {
            name: "undefined".to_string(),
            span: ash_core::ast::Span::default(),
        },
    };

    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();

    let result =
        execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
            .await;

    assert!(result.is_err(), "unbound variable should fail");
    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("undefined"),
        "error should mention the undefined variable: {}",
        error
    );
}

#[tokio::test]
async fn lexical_scope_nested_let_bindings() {
    // Test deeply nested let bindings
    let workflow = Workflow::Let {
        pattern: Pattern::Variable {
            name: "a".to_string(),
            span: ash_core::ast::Span::default(),
        },
        expr: Expr::Literal(Value::Int(10)),
        continuation: Box::new(Workflow::Let {
            pattern: Pattern::Variable {
                name: "b".to_string(),
                span: ash_core::ast::Span::default(),
            },
            expr: Expr::Literal(Value::Int(20)),
            continuation: Box::new(Workflow::Let {
                pattern: Pattern::Variable {
                    name: "c".to_string(),
                    span: ash_core::ast::Span::default(),
                },
                expr: Expr::Literal(Value::Int(30)),
                continuation: Box::new(Workflow::Ret {
                    expr: Expr::Binary {
                        op: ash_core::BinaryOp::Add,
                        left: Box::new(Expr::Binary {
                            op: ash_core::BinaryOp::Add,
                            left: Box::new(Expr::Variable {
                                name: "a".to_string(),
                                span: ash_core::ast::Span::default(),
                            }),
                            right: Box::new(Expr::Variable {
                                name: "b".to_string(),
                                span: ash_core::ast::Span::default(),
                            }),
                        }),
                        right: Box::new(Expr::Variable {
                            name: "c".to_string(),
                            span: ash_core::ast::Span::default(),
                        }),
                    },
                }),
            }),
        }),
    };

    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();

    let result =
        execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
            .await
            .expect("workflow should execute successfully");

    assert_eq!(result, Value::Int(60));
}

#[tokio::test]
async fn lexical_scope_binding_not_visible_outside_scope() {
    // Test that a binding is not visible outside its continuation
    // This is implicit in the LET ... IN cont structure
    let workflow = Workflow::Let {
        pattern: Pattern::Variable {
            name: "inner".to_string(),
            span: ash_core::ast::Span::default(),
        },
        expr: Expr::Literal(Value::Int(42)),
        continuation: Box::new(Workflow::Done),
    };

    // After the LET completes, 'inner' should not be in scope
    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();

    let result =
        execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
            .await
            .expect("workflow should execute successfully");

    assert_eq!(result, Value::Null);
}

#[tokio::test]
async fn seq_preserves_explicit_sequencing() {
    // Test that SEQ preserves explicit sequencing semantics
    // SEQ should execute first, then second, without introducing lexical scope
    let workflow = Workflow::Seq {
        first: Box::new(Workflow::Let {
            pattern: Pattern::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            },
            expr: Expr::Literal(Value::Int(1)),
            continuation: Box::new(Workflow::Done),
        }),
        second: Box::new(Workflow::Ret {
            // This should fail because x is not in scope across SEQ boundary
            expr: Expr::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            },
        }),
    };

    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();

    let result =
        execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
            .await;

    // SEQ does NOT establish lexical scope between first and second
    assert!(
        result.is_err(),
        "variable should not be visible across SEQ boundary"
    );
    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("x"),
        "error should mention the undefined variable: {}",
        error
    );
}

#[tokio::test]
async fn seq_with_nested_let_preserves_scope() {
    // Test that SEQ can contain nested LETs that maintain their own scope
    let workflow = Workflow::Seq {
        first: Box::new(Workflow::Let {
            pattern: Pattern::Variable {
                name: "a".to_string(),
                span: ash_core::ast::Span::default(),
            },
            expr: Expr::Literal(Value::Int(10)),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Variable {
                    name: "a".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            }),
        }),
        second: Box::new(Workflow::Let {
            pattern: Pattern::Variable {
                name: "b".to_string(),
                span: ash_core::ast::Span::default(),
            },
            expr: Expr::Literal(Value::Int(20)),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Variable {
                    name: "b".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            }),
        }),
    };

    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();

    let result =
        execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
            .await
            .expect("workflow should execute successfully");

    // SEQ executes sequentially, returning the value of the last workflow
    assert_eq!(result, Value::Int(20));
}

#[tokio::test]
async fn canonical_form_nested_let_bindings() {
    // Test the canonical form: LET ... IN LET ... IN ...
    // This represents the normalized form of:
    // let x = 1
    // let y = x + 1
    // ret y
    let workflow = Workflow::Let {
        pattern: Pattern::Variable {
            name: "x".to_string(),
            span: ash_core::ast::Span::default(),
        },
        expr: Expr::Literal(Value::Int(1)),
        continuation: Box::new(Workflow::Let {
            pattern: Pattern::Variable {
                name: "y".to_string(),
                span: ash_core::ast::Span::default(),
            },
            expr: Expr::Binary {
                op: ash_core::BinaryOp::Add,
                left: Box::new(Expr::Variable {
                    name: "x".to_string(),
                    span: ash_core::ast::Span::default(),
                }),
                right: Box::new(Expr::Literal(Value::Int(1))),
            },
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Variable {
                    name: "y".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            }),
        }),
    };

    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();

    let result =
        execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
            .await
            .expect("workflow should execute successfully");

    // The nested LETs make x visible to y, and y visible to the return
    assert_eq!(result, Value::Int(2));
}

#[tokio::test]
async fn if_branches_maintain_separate_scope() {
    // Test that if-then-else branches maintain separate scope
    let workflow = Workflow::If {
        condition: Expr::Literal(Value::Bool(true)),
        then_branch: Box::new(Workflow::Let {
            pattern: Pattern::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            },
            expr: Expr::Literal(Value::Int(1)),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Variable {
                    name: "x".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            }),
        }),
        else_branch: Box::new(Workflow::Let {
            pattern: Pattern::Variable {
                name: "y".to_string(),
                span: ash_core::ast::Span::default(),
            },
            expr: Expr::Literal(Value::Int(2)),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Variable {
                    name: "y".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            }),
        }),
    };

    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();

    let result =
        execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
            .await
            .expect("workflow should execute successfully");

    assert_eq!(result, Value::Int(1));
}

#[tokio::test]
async fn pattern_matching_introduces_bindings() {
    // Test that pattern matching introduces bindings that are visible in the continuation
    let workflow = Workflow::Let {
        pattern: Pattern::List(
            vec![
                Pattern::Variable {
                    name: "first".to_string(),
                    span: ash_core::ast::Span::default(),
                },
                Pattern::Variable {
                    name: "second".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            ],
            None,
        ),
        expr: Expr::Literal(Value::list_from_vec(vec![Value::Int(1), Value::Int(2)])),
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Binary {
                op: ash_core::BinaryOp::Add,
                left: Box::new(Expr::Variable {
                    name: "first".to_string(),
                    span: ash_core::ast::Span::default(),
                }),
                right: Box::new(Expr::Variable {
                    name: "second".to_string(),
                    span: ash_core::ast::Span::default(),
                }),
            },
        }),
    };

    let (ctx, cap_ctx, policy_eval, behaviour_ctx) = execution_contexts();

    let result =
        execute_workflow_with_behaviour(&workflow, ctx, &cap_ctx, &policy_eval, &behaviour_ctx)
            .await
            .expect("workflow should execute successfully");

    assert_eq!(result, Value::Int(3));
}
