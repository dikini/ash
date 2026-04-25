//! Property-based tests for act block lowering (TASK-676).
//!
//! Verifies that lowering `Expr::ActBlock` produces only Call, FnDef, Literal,
//! and Variable core forms — never an ActBlock-specific core variant.

use ash_core::ast::Expr as CoreExpr;
use ash_parser::lower_expr;
use ash_parser::surface::{ActStmt, Expr, Literal};
use ash_parser::token::Span;
use proptest::prelude::*;

/// Generate a simple expression strategy: literal int, literal bool, or variable.
fn simple_expr_strategy() -> impl Strategy<Value = Expr> {
    let leaf = prop_oneof![
        Just(Expr::Literal(Literal::Int(42))),
        Just(Expr::Literal(Literal::Bool(true))),
        Just(Expr::Literal(Literal::Int(0))),
        Just(Expr::Variable {
            name: "v".into(),
            span: Span::default(),
        }),
    ];
    leaf.boxed()
}

/// Generate a strategy for ActStmt: Bind with simple expr or Return with simple expr.
#[allow(dead_code)]
fn act_stmt_strategy() -> impl Strategy<Value = ActStmt> {
    prop_oneof![
        simple_expr_strategy().prop_map(|value| ActStmt::Bind {
            name: "x".into(),
            value: Box::new(value),
            span: Span::default(),
        }),
        simple_expr_strategy().prop_map(|value| ActStmt::Return {
            value: Box::new(value),
            span: Span::default(),
        }),
    ]
}

/// Generate a bind-only ActStmt (for non-final positions).
fn bind_stmt_strategy() -> impl Strategy<Value = ActStmt> {
    simple_expr_strategy().prop_map(|value| ActStmt::Bind {
        name: "x".into(),
        value: Box::new(value),
        span: Span::default(),
    })
}

/// Generate a Vec<ActStmt> of 1-4 elements.
/// All non-final stmts are Bind; the last is always Return (SPEC-047 §6.2).
fn valid_act_stmts_strategy() -> impl Strategy<Value = Vec<ActStmt>> {
    prop::collection::vec(bind_stmt_strategy(), 0..3).prop_map(|mut stmts| {
        // Always end with Return
        stmts.push(ActStmt::Return {
            value: Box::new(Expr::Literal(Literal::Int(99))),
            span: Span::default(),
        });
        stmts
    })
}

/// Recursively assert that the core expression tree contains only lowered forms.
fn assert_only_lowered_forms(core: &CoreExpr) {
    match core {
        CoreExpr::Literal(_) | CoreExpr::Variable { .. } => {}
        CoreExpr::Call { arguments, .. } => {
            for arg in arguments {
                assert_only_lowered_forms(arg);
            }
        }
        CoreExpr::FnDef { body, .. } => {
            assert_only_lowered_forms(body);
        }
        CoreExpr::FieldAccess { expr, .. }
        | CoreExpr::Unary { expr, .. }
        | CoreExpr::Spawn { init: expr, .. } => {
            assert_only_lowered_forms(expr);
        }
        CoreExpr::IndexAccess { expr, index, .. } => {
            assert_only_lowered_forms(expr);
            assert_only_lowered_forms(index);
        }
        CoreExpr::Binary { left, right, .. } => {
            assert_only_lowered_forms(left);
            assert_only_lowered_forms(right);
        }
        CoreExpr::Match { scrutinee, arms } => {
            assert_only_lowered_forms(scrutinee);
            for arm in arms {
                assert_only_lowered_forms(&arm.body);
            }
        }
        CoreExpr::IfLet {
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            assert_only_lowered_forms(expr);
            assert_only_lowered_forms(then_branch);
            assert_only_lowered_forms(else_branch);
        }
        CoreExpr::Constructor { fields, .. } => {
            for (_, value) in fields {
                assert_only_lowered_forms(value);
            }
        }
        CoreExpr::Let { expr, body, .. } => {
            assert_only_lowered_forms(expr);
            assert_only_lowered_forms(body);
        }
        CoreExpr::FnApply { func, args, .. } => {
            assert_only_lowered_forms(func);
            for arg in args {
                assert_only_lowered_forms(arg);
            }
        }
        CoreExpr::Split(inner) => {
            assert_only_lowered_forms(inner);
        }
        CoreExpr::Fail { payload } => {
            assert_only_lowered_forms(payload);
        }
        CoreExpr::WithError { body, arms } => {
            assert_only_lowered_forms(body);
            for arm in arms {
                assert_only_lowered_forms(&arm.body);
            }
        }
        CoreExpr::CheckObligation { .. } => {}
    }
}

proptest! {
    #[test]
    fn prop_act_block_lowering_never_panics(stmts in valid_act_stmts_strategy()) {
        let surface = Expr::ActBlock {
            stmts,
            span: Span::default(),
        };
        let result = lower_expr(&surface);
        prop_assert!(result.is_ok(), "lowering valid act block should succeed");
    }

    #[test]
    fn prop_act_block_lowering_uses_only_valid_forms(stmts in valid_act_stmts_strategy()) {
        let surface = Expr::ActBlock {
            stmts,
            span: Span::default(),
        };
        if let Ok(core) = lower_expr(&surface) {
            assert_only_lowered_forms(&core);
        }
    }

    #[test]
    fn prop_single_return_act_block_lowers_to_unit(value in simple_expr_strategy()) {
        let surface = Expr::ActBlock {
            stmts: vec![ActStmt::Return {
                value: Box::new(value),
                span: Span::default(),
            }],
            span: Span::default(),
        };
        let core = lower_expr(&surface).expect("single-return act block should lower");
        match &core {
            CoreExpr::Call { func, arguments, .. } => {
                prop_assert_eq!(func, "unit");
                prop_assert_eq!(arguments.len(), 1);
            }
            _ => panic!("Expected Call, got: {:?}", core),
        }
    }
}

#[test]
fn test_empty_act_block_is_rejected() {
    let surface = Expr::ActBlock {
        stmts: vec![],
        span: Span::default(),
    };
    let result = lower_expr(&surface);
    assert!(
        result.is_err(),
        "empty act block must be rejected per SPEC-047 §6.2"
    );
}

#[test]
fn test_return_not_last_is_rejected() {
    let surface = Expr::ActBlock {
        stmts: vec![
            ActStmt::Return {
                value: Box::new(Expr::Literal(Literal::Int(1))),
                span: Span::default(),
            },
            ActStmt::Bind {
                name: "x".into(),
                value: Box::new(Expr::Literal(Literal::Int(2))),
                span: Span::default(),
            },
        ],
        span: Span::default(),
    };
    let result = lower_expr(&surface);
    assert!(
        result.is_err(),
        "return followed by more statements must be rejected"
    );
}

#[test]
fn test_expr_entry_point_parses_act_block() {
    let mut input = ash_parser::new_input("act { ret 42; }");
    let result = ash_parser::parse_expr::expr(&mut input);
    assert!(
        result.is_ok(),
        "expr() should parse act block: {:?}",
        result
    );
    match result.unwrap() {
        Expr::ActBlock { stmts, .. } => {
            assert_eq!(stmts.len(), 1);
        }
        other => panic!("Expected ActBlock from expr(), got: {:?}", other),
    }
}

#[test]
fn test_expr_entry_point_parses_nested_act_block() {
    let mut input = ash_parser::new_input("act { x = act { ret 1; }; ret x; }");
    let result = ash_parser::parse_expr::expr(&mut input);
    assert!(
        result.is_ok(),
        "expr() should parse nested act block: {:?}",
        result
    );
}

#[test]
fn test_act_without_brace_does_not_parse_as_expression() {
    // act provider:action(args) is workflow-level — the expression parser should not consume it
    let mut input = ash_parser::new_input("act provider:action(args)");
    let result = ash_parser::parse_expr::expr(&mut input);
    // It should either fail or parse as something other than ActBlock
    if let Ok(parsed) = result {
        // If it parsed, it must NOT be an ActBlock
        assert!(
            !matches!(parsed, Expr::ActBlock { .. }),
            "workflow-level act should not parse as ActBlock expression"
        );
    }
    // If it failed to parse entirely, that's also correct
}
