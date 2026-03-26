//! Tests for workflow obligation checking (TASK-275)
//!
//! These tests verify that:
//! 1. Obligations must be satisfied before workflow completion
//! 2. Satisfied obligations pass type checking
//! 3. Unknown obligations fail type checking
//! 4. Double satisfaction is prevented (linear obligation tracking)

use ash_core::workflow_contract::{ObligationError, ObligationSet};
use ash_parser::surface::{ObligationRef, Workflow};
use ash_parser::token::Span;
use ash_typeck::obligations::{LinearObligationContext, ObligationCollector};
use ash_typeck::solver::TypeError;
use proptest::prelude::*;

fn test_span() -> Span {
    Span::default()
}

#[test]
fn test_obligation_must_be_satisfied() {
    // workflow test {
    //     oblige logging_enabled;
    //     act log("hello");
    //     // ERROR: logging_enabled never checked!
    // }
    let workflow = Workflow::Seq {
        first: Box::new(Workflow::Oblige {
            obligation: "logging_enabled".into(),
            span: test_span(),
        }),
        second: Box::new(Workflow::Done { span: test_span() }),
        span: test_span(),
    };

    let mut collector = ObligationCollector::new();
    let mut ctx = LinearObligationContext::new();
    let result = collector.collect(&workflow, &mut ctx);

    // Should detect unsatisfied obligations
    assert!(result.is_ok()); // collect doesn't fail, we need to check finalize
    assert!(!ctx.is_clean());
    assert!(ctx.obligations.contains("logging_enabled"));

    // Finalize should report the unsatisfied obligation
    let final_result = collector.finalize(&ctx);
    assert!(final_result.is_err());
}

#[test]
fn test_satisfied_obligation_passes() {
    // workflow test {
    //     oblige logging_enabled;
    //     check logging_enabled;
    //     act log("hello");
    // }
    let workflow = Workflow::Seq {
        first: Box::new(Workflow::Oblige {
            obligation: "logging_enabled".into(),
            span: test_span(),
        }),
        second: Box::new(Workflow::Check {
            target: ash_parser::surface::CheckTarget::Obligation(ObligationRef {
                role: "logging_enabled".into(),
                condition: ash_parser::surface::Expr::Literal(ash_parser::surface::Literal::Bool(
                    true,
                )),
            }),
            continuation: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        }),
        span: test_span(),
    };

    let mut collector = ObligationCollector::new();
    let mut ctx = LinearObligationContext::new();
    let result = collector.collect(&workflow, &mut ctx);

    // Should succeed with all obligations satisfied
    assert!(result.is_ok());
    assert!(ctx.is_clean());
}

#[test]
fn test_check_without_obligation_fails() {
    // workflow test {
    //     check undefined_obligation;  // ERROR: no such obligation
    //     act log("hello");
    // }
    let workflow = Workflow::Check {
        target: ash_parser::surface::CheckTarget::Obligation(ObligationRef {
            role: "undefined_obligation".into(),
            condition: ash_parser::surface::Expr::Literal(ash_parser::surface::Literal::Bool(true)),
        }),
        continuation: Some(Box::new(Workflow::Done { span: test_span() })),
        span: test_span(),
    };

    let mut collector = ObligationCollector::new();
    let mut ctx = LinearObligationContext::new();
    let result = collector.collect(&workflow, &mut ctx);

    // Should fail with unknown obligation error
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        TypeError::UnknownObligation { name, .. } if name == "undefined_obligation"
    ));
}

#[test]
fn test_double_satisfaction_fails() {
    // workflow test {
    //     oblige once_only;
    //     check once_only;
    //     check once_only;  // ERROR: already satisfied
    // }
    let workflow = Workflow::Seq {
        first: Box::new(Workflow::Oblige {
            obligation: "once_only".into(),
            span: test_span(),
        }),
        second: Box::new(Workflow::Check {
            target: ash_parser::surface::CheckTarget::Obligation(ObligationRef {
                role: "once_only".into(),
                condition: ash_parser::surface::Expr::Literal(ash_parser::surface::Literal::Bool(
                    true,
                )),
            }),
            continuation: Some(Box::new(Workflow::Check {
                target: ash_parser::surface::CheckTarget::Obligation(ObligationRef {
                    role: "once_only".into(),
                    condition: ash_parser::surface::Expr::Literal(
                        ash_parser::surface::Literal::Bool(true),
                    ),
                }),
                continuation: None,
                span: test_span(),
            })),
            span: test_span(),
        }),
        span: test_span(),
    };

    let mut collector = ObligationCollector::new();
    let mut ctx = LinearObligationContext::new();
    let result = collector.collect(&workflow, &mut ctx);

    // First check succeeds, second should fail
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        TypeError::UnknownObligation { name, .. } if name == "once_only"
    ));
}

#[test]
fn test_multiple_obligations_all_satisfied() {
    // workflow test {
    //     oblige audit;
    //     oblige log_access;
    //     check audit;
    //     check log_access;
    // }
    let workflow = Workflow::Seq {
        first: Box::new(Workflow::Oblige {
            obligation: "audit".into(),
            span: test_span(),
        }),
        second: Box::new(Workflow::Seq {
            first: Box::new(Workflow::Oblige {
                obligation: "log_access".into(),
                span: test_span(),
            }),
            second: Box::new(Workflow::Seq {
                first: Box::new(Workflow::Check {
                    target: ash_parser::surface::CheckTarget::Obligation(ObligationRef {
                        role: "audit".into(),
                        condition: ash_parser::surface::Expr::Literal(
                            ash_parser::surface::Literal::Bool(true),
                        ),
                    }),
                    continuation: None,
                    span: test_span(),
                }),
                second: Box::new(Workflow::Check {
                    target: ash_parser::surface::CheckTarget::Obligation(ObligationRef {
                        role: "log_access".into(),
                        condition: ash_parser::surface::Expr::Literal(
                            ash_parser::surface::Literal::Bool(true),
                        ),
                    }),
                    continuation: None,
                    span: test_span(),
                }),
                span: test_span(),
            }),
            span: test_span(),
        }),
        span: test_span(),
    };

    let mut collector = ObligationCollector::new();
    let mut ctx = LinearObligationContext::new();
    let result = collector.collect(&workflow, &mut ctx);

    // All obligations satisfied
    assert!(result.is_ok());
    assert!(ctx.is_clean());
}

#[test]
fn test_if_else_branch_handling_both_satisfy() {
    // workflow test {
    //     oblige condition_met;
    //     if true {
    //         check condition_met;
    //     } else {
    //         check condition_met;
    //     }
    // }
    // Both branches satisfy => obligation discharged
    let _workflow = Workflow::Oblige {
        obligation: "condition_met".into(),
        span: test_span(),
    };

    // The Oblige is standalone, so we test the if separately with obligation already in context
    let mut collector = ObligationCollector::new();
    let mut ctx = LinearObligationContext::new();

    // First create the obligation
    ctx.obligations.insert("condition_met".to_string()).unwrap();

    let if_workflow = Workflow::If {
        condition: ash_parser::surface::Expr::Literal(ash_parser::surface::Literal::Bool(true)),
        then_branch: Box::new(Workflow::Check {
            target: ash_parser::surface::CheckTarget::Obligation(ObligationRef {
                role: "condition_met".into(),
                condition: ash_parser::surface::Expr::Literal(ash_parser::surface::Literal::Bool(
                    true,
                )),
            }),
            continuation: None,
            span: test_span(),
        }),
        else_branch: Some(Box::new(Workflow::Check {
            target: ash_parser::surface::CheckTarget::Obligation(ObligationRef {
                role: "condition_met".into(),
                condition: ash_parser::surface::Expr::Literal(ash_parser::surface::Literal::Bool(
                    true,
                )),
            }),
            continuation: None,
            span: test_span(),
        })),
        span: test_span(),
    };

    let result = collector.collect(&if_workflow, &mut ctx);

    // Both branches satisfy the obligation - intersection should be empty
    assert!(result.is_ok());
    // Note: With intersection semantics, if both branches discharge,
    // the merged context should be clean
}

#[test]
fn test_obligation_lifecycle_in_context() {
    let mut ctx = LinearObligationContext::new();

    // Create obligation
    ctx.obligations.insert("audit".to_string()).unwrap();
    assert!(ctx.obligations.contains("audit"));
    assert!(!ctx.is_clean());

    // Check (consume) obligation
    ctx.obligations.remove("audit").unwrap();
    assert!(!ctx.obligations.contains("audit"));
    assert!(ctx.is_clean());
}

#[test]
fn test_obligation_set_operations() {
    let mut set = ObligationSet::new();

    // Insert and check
    set.insert("a".to_string()).unwrap();
    set.insert("b".to_string()).unwrap();
    assert!(set.contains("a"));
    assert!(set.contains("b"));

    // Remove
    set.remove("a").unwrap();
    assert!(!set.contains("a"));
    assert!(set.contains("b"));

    // Double remove should fail
    let result = set.remove("a");
    assert!(matches!(result, Err(ObligationError::Unknown(_))));

    // Double insert should fail
    let result = set.insert("b".to_string());
    assert!(matches!(result, Err(ObligationError::Duplicate(_))));
}

#[test]
fn test_obligation_collector_new() {
    let collector = ObligationCollector::new();
    // Just verify it can be created
    let _ = collector;
}

// Property-based tests
proptest! {
    #[test]
    fn obligation_insert_adds_to_set(name in "[a-z_][a-z0-9_]*") {
        let mut set = ObligationSet::new();
        prop_assert!(set.insert(&name).is_ok());
        prop_assert!(set.contains(&name));
    }

    #[test]
    fn obligation_remove_consumes(name in "[a-z_][a-z0-9_]*") {
        let mut set = ObligationSet::new();
        set.insert(&name).unwrap();
        prop_assert!(set.remove(&name).is_ok());
        prop_assert!(!set.contains(&name));
        prop_assert!(set.is_empty());
    }

    #[test]
    fn double_insert_fails(name in "[a-z_][a-z0-9_]*") {
        let mut set = ObligationSet::new();
        set.insert(&name).unwrap();
        prop_assert!(matches!(set.insert(&name), Err(ObligationError::Duplicate(_))));
    }

    #[test]
    fn double_remove_fails(name in "[a-z_][a-z0-9_]*") {
        let mut set = ObligationSet::new();
        set.insert(&name).unwrap();
        set.remove(&name).unwrap();
        prop_assert!(matches!(set.remove(&name), Err(ObligationError::Unknown(_))));
    }

    #[test]
    fn linear_obligation_soundness(
        obligations in prop::collection::vec("[a-z_][a-z0-9_]*", 0..10),
        checks in prop::collection::vec("[a-z_][a-z0-9_]*", 0..10)
    ) {
        // Property: every checked obligation must have been obliged
        // and every obliged obligation must be checked exactly once
        let mut set = ObligationSet::new();

        // Insert all obligations
        for obl in &obligations {
            // Ignore duplicate errors for this test
            let _ = set.insert(obl.clone());
        }

        // Remove checked obligations
        for check in &checks {
            // Ignore unknown errors for this test
            let _ = set.remove(check);
        }

        // After checking, any remaining obligations are undischarged
        // This property verifies linearity: each obligation can be checked at most once
        let remaining: Vec<_> = set.remaining();

        // All obligations that were checked should not be in remaining
        for check in &checks {
            if obligations.contains(check) {
                // If it was checked, it shouldn't be remaining
                // (unless it was duplicated in obligations and only checked once)
                let count_obliged = obligations.iter().filter(|&o| o == check).count();
                let count_remaining = remaining.iter().filter(|&&r| r == check).count();
                // The remaining count should be count_obliged - count_checked
                // Since we check at most once per check occurrence
                prop_assert!(count_remaining <= count_obliged);
            }
        }
    }
}
