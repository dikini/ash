//! Surface `effect_tests` module.

use super::*;
use ash_core::Effect;

fn dummy_span() -> Span {
    Span::new(0, 0, 1, 1)
}

// =========================================================================
// Basic Effect Tests
// =========================================================================

#[test]
fn test_observe_effect() {
    let workflow = Workflow::Observe {
        capability: "sensor".into(),
        binding: Some(Pattern::Variable {
            name: "data".into(),
            span: crate::token::Span::default(),
        }),
        continuation: None,
        span: dummy_span(),
    };
    assert_eq!(workflow.effect(), Effect::Epistemic);
}

#[test]
fn test_observe_with_continuation() {
    let workflow = Workflow::Observe {
        capability: "sensor".into(),
        binding: Some(Pattern::Variable {
            name: "data".into(),
            span: crate::token::Span::default(),
        }),
        continuation: Some(Box::new(Workflow::Act {
            action: ActionRef {
                target: OperationalTarget::Symbolic {
                    capability_name: "process".into(),
                },
                args: vec![],
            },
            guard: None,
            result_name: None,
            continuation: None,
            span: dummy_span(),
        })),
        span: dummy_span(),
    };
    // Epistemic join Operational = Operational
    assert_eq!(workflow.effect(), Effect::Operational);
}

#[test]
fn test_orient_effect() {
    let workflow = Workflow::Orient {
        expr: Expr::Literal(Literal::Int(42)),
        binding: Some(Pattern::Variable {
            name: "result".into(),
            span: crate::token::Span::default(),
        }),
        continuation: None,
        span: dummy_span(),
    };
    assert_eq!(workflow.effect(), Effect::Epistemic);
}

#[test]
fn test_propose_effect() {
    let workflow = Workflow::Propose {
        action: ActionRef {
            target: OperationalTarget::Symbolic {
                capability_name: "send_email".into(),
            },
            args: vec![],
        },
        binding: None,
        continuation: None,
        span: dummy_span(),
    };
    assert_eq!(workflow.effect(), Effect::Deliberative);
}

#[test]
fn test_decide_effect() {
    let workflow = Workflow::Decide {
        expr: Expr::Literal(Literal::Bool(true)),
        policy: None,
        then_branch: Box::new(Workflow::Done { span: dummy_span() }),
        else_branch: None,
        span: dummy_span(),
    };
    assert_eq!(workflow.effect(), Effect::Epistemic);
}

#[test]
fn test_check_effect() {
    let workflow = Workflow::Check {
        target: CheckTarget::Obligation(ObligationRef {
            role: "admin".into(),
            condition: Expr::Literal(Literal::Bool(true)),
        }),
        continuation: None,
        span: dummy_span(),
    };
    assert_eq!(workflow.effect(), Effect::Evaluative);
}

#[test]
fn test_act_effect() {
    let workflow = Workflow::Act {
        action: ActionRef {
            target: OperationalTarget::Symbolic {
                capability_name: "write_file".into(),
            },
            args: vec![],
        },
        guard: None,
        result_name: None,
        continuation: None,
        span: dummy_span(),
    };
    assert_eq!(workflow.effect(), Effect::Operational);
}

#[test]
fn test_done_effect() {
    let workflow = Workflow::Done { span: dummy_span() };
    assert_eq!(workflow.effect(), Effect::Epistemic);
}

#[test]
fn test_ret_effect() {
    let workflow = Workflow::Ret {
        expr: Expr::Literal(Literal::Int(42)),
        span: dummy_span(),
    };
    assert_eq!(workflow.effect(), Effect::Epistemic);
}

// =========================================================================
// Composite Workflow Effect Tests
// =========================================================================

#[test]
fn test_seq_effect_epistemic_operational() {
    // observe (Epistemic) ; act (Operational) = Operational
    let observe = Workflow::Observe {
        capability: "sensor".into(),
        binding: None,
        continuation: None,
        span: dummy_span(),
    };
    let act = Workflow::Act {
        action: ActionRef {
            target: OperationalTarget::Symbolic {
                capability_name: "process".into(),
            },
            args: vec![],
        },
        guard: None,
        result_name: None,
        continuation: None,
        span: dummy_span(),
    };
    let seq = Workflow::Seq {
        first: Box::new(observe),
        second: Box::new(act),
        span: dummy_span(),
    };
    assert_eq!(seq.effect(), Effect::Operational);
}

#[test]
fn test_seq_effect_epistemic_epistemic() {
    // observe (Epistemic) ; observe (Epistemic) = Epistemic
    let observe1 = Workflow::Observe {
        capability: "sensor1".into(),
        binding: None,
        continuation: None,
        span: dummy_span(),
    };
    let observe2 = Workflow::Observe {
        capability: "sensor2".into(),
        binding: None,
        continuation: None,
        span: dummy_span(),
    };
    let seq = Workflow::Seq {
        first: Box::new(observe1),
        second: Box::new(observe2),
        span: dummy_span(),
    };
    assert_eq!(seq.effect(), Effect::Epistemic);
}

#[test]
fn test_if_effect_both_branches() {
    // if cond then observe else act = Operational
    let observe = Workflow::Observe {
        capability: "sensor".into(),
        binding: None,
        continuation: None,
        span: dummy_span(),
    };
    let act = Workflow::Act {
        action: ActionRef {
            target: OperationalTarget::Symbolic {
                capability_name: "process".into(),
            },
            args: vec![],
        },
        guard: None,
        result_name: None,
        continuation: None,
        span: dummy_span(),
    };
    let if_workflow = Workflow::If {
        condition: Expr::Literal(Literal::Bool(true)),
        then_branch: Box::new(observe),
        else_branch: Some(Box::new(act)),
        span: dummy_span(),
    };
    assert_eq!(if_workflow.effect(), Effect::Operational);
}

#[test]
fn test_if_effect_no_else() {
    // if cond then observe (no else) = Epistemic
    let observe = Workflow::Observe {
        capability: "sensor".into(),
        binding: None,
        continuation: None,
        span: dummy_span(),
    };
    let if_workflow = Workflow::If {
        condition: Expr::Literal(Literal::Bool(true)),
        then_branch: Box::new(observe),
        else_branch: None,
        span: dummy_span(),
    };
    assert_eq!(if_workflow.effect(), Effect::Epistemic);
}

#[test]
fn test_for_effect() {
    // for x in items { act } = Operational
    let act = Workflow::Act {
        action: ActionRef {
            target: OperationalTarget::Symbolic {
                capability_name: "process".into(),
            },
            args: vec![],
        },
        guard: None,
        result_name: None,
        continuation: None,
        span: dummy_span(),
    };
    let for_workflow = Workflow::For {
        pattern: Pattern::Variable {
            name: "x".into(),
            span: crate::token::Span::default(),
        },
        collection: Expr::Variable {
            name: "items".into(),
            span: crate::token::Span::default(),
        },
        body: Box::new(act),
        span: dummy_span(),
    };
    assert_eq!(for_workflow.effect(), Effect::Operational);
}

#[test]
fn test_let_effect() {
    // let x = 42 in act = Operational
    let act = Workflow::Act {
        action: ActionRef {
            target: OperationalTarget::Symbolic {
                capability_name: "process".into(),
            },
            args: vec![],
        },
        guard: None,
        result_name: None,
        continuation: None,
        span: dummy_span(),
    };
    let let_workflow = Workflow::Let {
        pattern: Pattern::Variable {
            name: "x".into(),
            span: crate::token::Span::default(),
        },
        expr: Expr::Literal(Literal::Int(42)),
        continuation: Some(Box::new(act)),
        span: dummy_span(),
    };
    assert_eq!(let_workflow.effect(), Effect::Operational);
}

#[test]
fn test_let_no_continuation() {
    // let x = 42 (no continuation) = Epistemic
    let let_workflow = Workflow::Let {
        pattern: Pattern::Variable {
            name: "x".into(),
            span: crate::token::Span::default(),
        },
        expr: Expr::Literal(Literal::Int(42)),
        continuation: None,
        span: dummy_span(),
    };
    assert_eq!(let_workflow.effect(), Effect::Epistemic);
}

#[test]
fn test_maybe_effect() {
    // maybe { observe } fallback { act } = Operational
    let observe = Workflow::Observe {
        capability: "sensor".into(),
        binding: None,
        continuation: None,
        span: dummy_span(),
    };
    let act = Workflow::Act {
        action: ActionRef {
            target: OperationalTarget::Symbolic {
                capability_name: "process".into(),
            },
            args: vec![],
        },
        guard: None,
        result_name: None,
        continuation: None,
        span: dummy_span(),
    };
    let maybe = Workflow::Maybe {
        primary: Box::new(observe),
        fallback: Box::new(act),
        span: dummy_span(),
    };
    assert_eq!(maybe.effect(), Effect::Operational);
}

#[test]
fn test_must_effect() {
    // must { act } = Operational
    let act = Workflow::Act {
        action: ActionRef {
            target: OperationalTarget::Symbolic {
                capability_name: "process".into(),
            },
            args: vec![],
        },
        guard: None,
        result_name: None,
        continuation: None,
        span: dummy_span(),
    };
    let must = Workflow::Must {
        body: Box::new(act),
        span: dummy_span(),
    };
    assert_eq!(must.effect(), Effect::Operational);
}

#[test]
fn test_with_effect() {
    // with db { act } = Operational
    let act = Workflow::Act {
        action: ActionRef {
            target: OperationalTarget::Symbolic {
                capability_name: "query".into(),
            },
            args: vec![],
        },
        guard: None,
        result_name: None,
        continuation: None,
        span: dummy_span(),
    };
    let with = Workflow::With {
        capability: "database".into(),
        body: Box::new(act),
        span: dummy_span(),
    };
    assert_eq!(with.effect(), Effect::Operational);
}

// =========================================================================
// Lattice Property Tests
// =========================================================================

#[test]
fn test_effect_lattice_ordering() {
    // Verify the lattice ordering: Epistemic < Deliberative < Evaluative < Operational
    let epistemic = Workflow::Observe {
        capability: "x".into(),
        binding: None,
        continuation: None,
        span: dummy_span(),
    };
    let deliberative = Workflow::Propose {
        action: ActionRef {
            target: OperationalTarget::Symbolic {
                capability_name: "x".into(),
            },
            args: vec![],
        },
        binding: None,
        continuation: None,
        span: dummy_span(),
    };
    let evaluative = Workflow::Check {
        target: CheckTarget::Obligation(ObligationRef {
            role: "admin".into(),
            condition: Expr::Literal(Literal::Bool(true)),
        }),
        continuation: None,
        span: dummy_span(),
    };
    let operational = Workflow::Act {
        action: ActionRef {
            target: OperationalTarget::Symbolic {
                capability_name: "x".into(),
            },
            args: vec![],
        },
        guard: None,
        result_name: None,
        continuation: None,
        span: dummy_span(),
    };

    assert_eq!(epistemic.effect(), Effect::Epistemic);
    assert_eq!(deliberative.effect(), Effect::Deliberative);
    assert_eq!(evaluative.effect(), Effect::Evaluative);
    assert_eq!(operational.effect(), Effect::Operational);

    // Verify ordering through joins
    assert_eq!(
        Workflow::Seq {
            first: Box::new(epistemic.clone()),
            second: Box::new(deliberative.clone()),
            span: dummy_span(),
        }
        .effect(),
        Effect::Deliberative
    );
    assert_eq!(
        Workflow::Seq {
            first: Box::new(deliberative.clone()),
            second: Box::new(evaluative.clone()),
            span: dummy_span(),
        }
        .effect(),
        Effect::Evaluative
    );
    assert_eq!(
        Workflow::Seq {
            first: Box::new(evaluative.clone()),
            second: Box::new(operational.clone()),
            span: dummy_span(),
        }
        .effect(),
        Effect::Operational
    );
}

#[test]
fn test_decide_with_operational_branches() {
    // decide with operational branches
    let act_then = Workflow::Act {
        action: ActionRef {
            target: OperationalTarget::Symbolic {
                capability_name: "process".into(),
            },
            args: vec![],
        },
        guard: None,
        result_name: None,
        continuation: None,
        span: dummy_span(),
    };
    let act_else = Workflow::Act {
        action: ActionRef {
            target: OperationalTarget::Symbolic {
                capability_name: "cleanup".into(),
            },
            args: vec![],
        },
        guard: None,
        result_name: None,
        continuation: None,
        span: dummy_span(),
    };
    let decide = Workflow::Decide {
        expr: Expr::Literal(Literal::Bool(true)),
        policy: None,
        then_branch: Box::new(act_then),
        else_branch: Some(Box::new(act_else)),
        span: dummy_span(),
    };

    assert_eq!(decide.effect(), Effect::Operational);
}

// =========================================================================
// Receive Effect Tests (TASK-108)
// =========================================================================

#[test]
fn test_receive_effect_is_epistemic() {
    // Receive with no arms should be Epistemic (read-only observation)
    let workflow = Workflow::Receive {
        mode: ReceiveMode::NonBlocking,
        arms: vec![],
        is_control: false,
        span: dummy_span(),
    };
    assert_eq!(workflow.effect(), Effect::Epistemic);
}

#[test]
fn test_receive_effect_blocking() {
    // Blocking receive should still be Epistemic
    let workflow = Workflow::Receive {
        mode: ReceiveMode::Blocking(Some(std::time::Duration::from_secs(10))),
        arms: vec![],
        is_control: false,
        span: dummy_span(),
    };
    assert_eq!(workflow.effect(), Effect::Epistemic);
}

#[test]
fn test_receive_with_epistemic_body() {
    // Receive with epistemic body should be Epistemic
    let arm = ReceiveArm {
        pattern: StreamPattern::Wildcard,
        guard: None,
        body: Workflow::Observe {
            capability: "sensor".into(),
            binding: None,
            continuation: None,
            span: dummy_span(),
        },
        span: dummy_span(),
    };
    let workflow = Workflow::Receive {
        mode: ReceiveMode::NonBlocking,
        arms: vec![arm],
        is_control: false,
        span: dummy_span(),
    };
    assert_eq!(workflow.effect(), Effect::Epistemic);
}

#[test]
fn test_receive_with_operational_body() {
    // Receive with operational body should be Operational
    let arm = ReceiveArm {
        pattern: StreamPattern::Wildcard,
        guard: None,
        body: Workflow::Act {
            action: ActionRef {
                target: OperationalTarget::Symbolic {
                    capability_name: "process".into(),
                },
                args: vec![],
            },
            guard: None,
            result_name: None,
            continuation: None,
            span: dummy_span(),
        },
        span: dummy_span(),
    };
    let workflow = Workflow::Receive {
        mode: ReceiveMode::NonBlocking,
        arms: vec![arm],
        is_control: false,
        span: dummy_span(),
    };
    // Epistemic join Operational = Operational
    assert_eq!(workflow.effect(), Effect::Operational);
}

#[test]
fn test_receive_multiple_arms_effect_join() {
    // Receive with multiple arms should join all arm body effects
    let arm1 = ReceiveArm {
        pattern: StreamPattern::Wildcard,
        guard: None,
        body: Workflow::Observe {
            capability: "sensor1".into(),
            binding: None,
            continuation: None,
            span: dummy_span(),
        },
        span: dummy_span(),
    };
    let arm2 = ReceiveArm {
        pattern: StreamPattern::Literal("stop".into()),
        guard: None,
        body: Workflow::Act {
            action: ActionRef {
                target: OperationalTarget::Symbolic {
                    capability_name: "shutdown".into(),
                },
                args: vec![],
            },
            guard: None,
            result_name: None,
            continuation: None,
            span: dummy_span(),
        },
        span: dummy_span(),
    };
    let workflow = Workflow::Receive {
        mode: ReceiveMode::NonBlocking,
        arms: vec![arm1, arm2],
        is_control: false,
        span: dummy_span(),
    };
    // join(Epistemic, Epistemic, Operational) = Operational
    assert_eq!(workflow.effect(), Effect::Operational);
}

#[test]
fn test_receive_control_is_epistemic() {
    // Control receive should still be Epistemic (just a different mailbox)
    let workflow = Workflow::Receive {
        mode: ReceiveMode::NonBlocking,
        arms: vec![],
        is_control: true,
        span: dummy_span(),
    };
    assert_eq!(workflow.effect(), Effect::Epistemic);
}

// =========================================================================
// ActStmt + Expr::ActBlock Tests (TASK-673)
// =========================================================================

#[test]
fn test_act_stmt_bind_construction() {
    let stmt = ActStmt::Bind {
        name: "x".into(),
        value: Box::new(Expr::Literal(Literal::Int(42))),
        span: Span::new(0, 10, 1, 1),
    };

    match stmt {
        ActStmt::Bind {
            name,
            value,
            span: _,
        } => {
            assert_eq!(name, "x".into());
            assert!(matches!(*value, Expr::Literal(Literal::Int(42))));
        }
        _ => panic!("Expected ActStmt::Bind"),
    }
}

#[test]
fn test_act_stmt_return_construction() {
    let stmt = ActStmt::Return {
        value: Box::new(Expr::Literal(Literal::Int(99))),
        span: Span::new(5, 12, 1, 6),
    };

    match stmt {
        ActStmt::Return { value, span: _ } => {
            assert!(matches!(*value, Expr::Literal(Literal::Int(99))));
        }
        _ => panic!("Expected ActStmt::Return"),
    }
}

#[test]
fn test_expr_act_block_construction() {
    let expr = Expr::ActBlock {
        stmts: vec![
            ActStmt::Bind {
                name: "x".into(),
                value: Box::new(Expr::Literal(Literal::Int(42))),
                span: Span::new(0, 10, 1, 1),
            },
            ActStmt::Return {
                value: Box::new(Expr::Variable {
                    name: "x".into(),
                    span: Span::new(11, 12, 1, 12),
                }),
                span: Span::new(11, 17, 1, 12),
            },
        ],
        span: Span::new(0, 18, 1, 1),
    };

    match expr {
        Expr::ActBlock { stmts, span: _ } => {
            assert_eq!(stmts.len(), 2);
            assert!(matches!(&stmts[0], ActStmt::Bind { name, .. } if name.as_ref() == "x"));
            assert!(matches!(&stmts[1], ActStmt::Return { .. }));
        }
        _ => panic!("Expected Expr::ActBlock"),
    }
}

#[test]
fn test_expr_act_block_empty() {
    let expr = Expr::ActBlock {
        stmts: vec![],
        span: Span::new(0, 6, 1, 1),
    };
    assert!(matches!(expr, Expr::ActBlock { stmts, .. } if stmts.is_empty()));
}
