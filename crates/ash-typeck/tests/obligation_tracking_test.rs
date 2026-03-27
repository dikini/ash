//! Tests for obligation tracking enforcement in type_check_workflow (TASK-290)
//!
//! These tests verify that:
//! 1. Obligations must be satisfied before workflow completion
//! 2. Satisfied obligations pass type checking
//! 3. Unknown obligations fail type checking
//! 4. Double satisfaction is prevented (linear obligation tracking)
//! 5. Branch obligation discipline is enforced

use ash_parser::surface::{CheckTarget, Expr, Literal, ObligationRef, Pattern, Workflow};
use ash_parser::token::Span;
use ash_typeck::type_check_workflow;

fn test_span() -> Span {
    Span::default()
}

#[test]
fn test_oblige_requires_check() {
    // workflow test {
    //     oblige CanRead("data");
    //     // Missing: check CanRead("data");
    //     act read("data");
    // }
    // Should fail: obligation not discharged
    let workflow = Workflow::Seq {
        first: Box::new(Workflow::Oblige {
            obligation: "CanRead".into(),
            span: test_span(),
        }),
        second: Box::new(Workflow::Done { span: test_span() }),
        span: test_span(),
    };

    let result = type_check_workflow(&workflow, None);
    assert!(result.is_ok(), "type_check_workflow should return Ok");

    let tc_result = result.unwrap();
    assert!(
        !tc_result.is_ok(),
        "Type checking should fail due to undischarged obligation"
    );
    assert!(
        !tc_result.obligation_status.is_success(),
        "Obligation status should not be success"
    );
}

#[test]
fn test_check_without_obligate_fails() {
    // workflow test {
    //     // No oblige, but trying to check
    //     check CanRead("data");
    //     act read("data");
    // }
    // Should fail: checking obligation that wasn't created
    let workflow = Workflow::Check {
        target: CheckTarget::Obligation(ObligationRef {
            role: "CanRead".into(),
            condition: Expr::Literal(Literal::Bool(true)),
        }),
        continuation: Some(Box::new(Workflow::Done { span: test_span() })),
        span: test_span(),
    };

    let result = type_check_workflow(&workflow, None);
    assert!(result.is_ok(), "type_check_workflow should return Ok");

    let tc_result = result.unwrap();
    assert!(
        !tc_result.is_ok(),
        "Type checking should fail due to checking unknown obligation"
    );
}

#[test]
fn test_obligate_check_pair_succeeds() {
    // workflow test {
    //     oblige CanRead("data");
    //     check CanRead("data");
    //     act read("data");
    // }
    // Should succeed: obligation created and discharged
    let workflow = Workflow::Seq {
        first: Box::new(Workflow::Oblige {
            obligation: "CanRead".into(),
            span: test_span(),
        }),
        second: Box::new(Workflow::Check {
            target: CheckTarget::Obligation(ObligationRef {
                role: "CanRead".into(),
                condition: Expr::Literal(Literal::Bool(true)),
            }),
            continuation: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        }),
        span: test_span(),
    };

    let result = type_check_workflow(&workflow, None);
    assert!(result.is_ok(), "type_check_workflow should return Ok");

    let tc_result = result.unwrap();
    assert!(
        tc_result.is_ok(),
        "Type checking should succeed with all obligations discharged"
    );
    assert!(
        tc_result.obligation_status.is_success(),
        "Obligation status should be success"
    );
}

#[test]
fn test_branch_obligation_discipline_both_paths() {
    // workflow test(cond: Bool) {
    //     oblige audit_trail;
    //     if cond {
    //         check audit_trail;
    //         act read("data");
    //     } else {
    //         check audit_trail;
    //         act skip();
    //     }
    // }
    // Should succeed: both branches discharge the obligation
    let workflow = Workflow::Oblige {
        obligation: "audit_trail".into(),
        span: test_span(),
    };

    // First, verify the oblige step
    let result = type_check_workflow(&workflow, None);
    assert!(result.is_ok());
    let tc_result = result.unwrap();
    // Just an oblige without check should fail
    assert!(!tc_result.is_ok());

    // Now test if with both branches checking
    let if_workflow = Workflow::Seq {
        first: Box::new(Workflow::Oblige {
            obligation: "audit_trail".into(),
            span: test_span(),
        }),
        second: Box::new(Workflow::If {
            condition: Expr::Literal(Literal::Bool(true)),
            then_branch: Box::new(Workflow::Check {
                target: CheckTarget::Obligation(ObligationRef {
                    role: "audit_trail".into(),
                    condition: Expr::Literal(Literal::Bool(true)),
                }),
                continuation: None,
                span: test_span(),
            }),
            else_branch: Some(Box::new(Workflow::Check {
                target: CheckTarget::Obligation(ObligationRef {
                    role: "audit_trail".into(),
                    condition: Expr::Literal(Literal::Bool(true)),
                }),
                continuation: None,
                span: test_span(),
            })),
            span: test_span(),
        }),
        span: test_span(),
    };

    let result = type_check_workflow(&if_workflow, None);
    assert!(result.is_ok(), "type_check_workflow should return Ok");

    let tc_result = result.unwrap();
    assert!(
        tc_result.is_ok(),
        "Type checking should succeed when both branches discharge obligations"
    );
}

#[test]
fn test_branch_obligation_discipline_partial_failure() {
    // workflow test(cond: Bool) {
    //     oblige audit_trail;
    //     if cond {
    //         check audit_trail;
    //         act read("data");
    //     } else {
    //         // Missing check in else branch
    //         act skip();
    //     }
    // }
    // Should fail: branches have incompatible obligation sets
    let workflow = Workflow::Seq {
        first: Box::new(Workflow::Oblige {
            obligation: "audit_trail".into(),
            span: test_span(),
        }),
        second: Box::new(Workflow::If {
            condition: Expr::Literal(Literal::Bool(true)),
            then_branch: Box::new(Workflow::Check {
                target: CheckTarget::Obligation(ObligationRef {
                    role: "audit_trail".into(),
                    condition: Expr::Literal(Literal::Bool(true)),
                }),
                continuation: None,
                span: test_span(),
            }),
            else_branch: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        }),
        span: test_span(),
    };

    let result = type_check_workflow(&workflow, None);
    assert!(result.is_ok(), "type_check_workflow should return Ok");

    let tc_result = result.unwrap();
    assert!(
        !tc_result.is_ok(),
        "Type checking should fail when obligation not discharged in all branches"
    );
}

#[test]
fn test_nested_obligation_tracking() {
    // workflow test {
    //     oblige CanRead("a");
    //     oblige CanWrite("b");
    //     check CanRead("a");
    //     check CanWrite("b");
    // }
    // Should succeed: all obligations discharged
    let workflow = Workflow::Seq {
        first: Box::new(Workflow::Oblige {
            obligation: "CanRead".into(),
            span: test_span(),
        }),
        second: Box::new(Workflow::Seq {
            first: Box::new(Workflow::Oblige {
                obligation: "CanWrite".into(),
                span: test_span(),
            }),
            second: Box::new(Workflow::Seq {
                first: Box::new(Workflow::Check {
                    target: CheckTarget::Obligation(ObligationRef {
                        role: "CanRead".into(),
                        condition: Expr::Literal(Literal::Bool(true)),
                    }),
                    continuation: None,
                    span: test_span(),
                }),
                second: Box::new(Workflow::Check {
                    target: CheckTarget::Obligation(ObligationRef {
                        role: "CanWrite".into(),
                        condition: Expr::Literal(Literal::Bool(true)),
                    }),
                    continuation: Some(Box::new(Workflow::Done { span: test_span() })),
                    span: test_span(),
                }),
                span: test_span(),
            }),
            span: test_span(),
        }),
        span: test_span(),
    };

    let result = type_check_workflow(&workflow, None);
    assert!(result.is_ok(), "type_check_workflow should return Ok");

    let tc_result = result.unwrap();
    assert!(
        tc_result.is_ok(),
        "Type checking should succeed with all obligations discharged"
    );
}

#[test]
fn test_obligation_linear_usage() {
    // workflow test {
    //     oblige CanRead("data");
    //     check CanRead("data");
    //     check CanRead("data");  // Second check - should fail
    //     act read("data");
    // }
    // Should fail: obligation already consumed
    let workflow = Workflow::Seq {
        first: Box::new(Workflow::Oblige {
            obligation: "CanRead".into(),
            span: test_span(),
        }),
        second: Box::new(Workflow::Seq {
            first: Box::new(Workflow::Check {
                target: CheckTarget::Obligation(ObligationRef {
                    role: "CanRead".into(),
                    condition: Expr::Literal(Literal::Bool(true)),
                }),
                continuation: None,
                span: test_span(),
            }),
            second: Box::new(Workflow::Check {
                target: CheckTarget::Obligation(ObligationRef {
                    role: "CanRead".into(),
                    condition: Expr::Literal(Literal::Bool(true)),
                }),
                continuation: None,
                span: test_span(),
            }),
            span: test_span(),
        }),
        span: test_span(),
    };

    let result = type_check_workflow(&workflow, None);
    assert!(result.is_ok(), "type_check_workflow should return Ok");

    let tc_result = result.unwrap();
    assert!(
        !tc_result.is_ok(),
        "Type checking should fail when obligation is checked twice (linear violation)"
    );
}

#[test]
fn test_empty_workflow_passes() {
    // workflow test {
    // }
    // Should succeed: no obligations to discharge
    let workflow = Workflow::Done { span: test_span() };

    let result = type_check_workflow(&workflow, None);
    assert!(result.is_ok(), "type_check_workflow should return Ok");

    let tc_result = result.unwrap();
    assert!(
        tc_result.is_ok(),
        "Type checking should succeed with no obligations"
    );
    assert!(
        tc_result.obligation_status.is_success(),
        "Obligation status should be success with no obligations"
    );
}

#[test]
fn test_obligation_with_let_binding() {
    // workflow test {
    //     oblige audit_required;
    //     let x = 42;
    //     check audit_required;
    // }
    let workflow = Workflow::Seq {
        first: Box::new(Workflow::Oblige {
            obligation: "audit_required".into(),
            span: test_span(),
        }),
        second: Box::new(Workflow::Let {
            pattern: Pattern::Variable("x".into()),
            expr: Expr::Literal(Literal::Int(42)),
            continuation: Some(Box::new(Workflow::Check {
                target: CheckTarget::Obligation(ObligationRef {
                    role: "audit_required".into(),
                    condition: Expr::Literal(Literal::Bool(true)),
                }),
                continuation: Some(Box::new(Workflow::Done { span: test_span() })),
                span: test_span(),
            })),
            span: test_span(),
        }),
        span: test_span(),
    };

    let result = type_check_workflow(&workflow, None);
    assert!(result.is_ok(), "type_check_workflow should return Ok");

    let tc_result = result.unwrap();
    assert!(
        tc_result.is_ok(),
        "Type checking should succeed with obligation discharged after let binding"
    );
}
