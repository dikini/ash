use ash_parser::surface::{CheckTarget, Expr, Literal, ObligationRef, PolicyInstance, Workflow};
use ash_parser::token::Span;
use ash_typeck::capability_check::{CapabilityCheckError, CapabilityChecker};

fn span() -> Span {
    Span::default()
}

#[test]
fn decide_requires_explicit_named_policy() {
    let workflow = Workflow::Decide {
        expr: Expr::Literal(Literal::Bool(true)),
        policy: None,
        then_branch: Box::new(Workflow::Done { span: span() }),
        else_branch: None,
        span: span(),
    };

    let error = CapabilityChecker::new()
        .verify(&workflow)
        .expect_err("decide without an explicit policy should be rejected");

    assert!(matches!(
        error,
        CapabilityCheckError::MissingPolicyReference
    ));
}

#[test]
fn decide_accepts_explicit_named_policy() {
    let workflow = Workflow::Decide {
        expr: Expr::Literal(Literal::Bool(true)),
        policy: Some("gate".into()),
        then_branch: Box::new(Workflow::Done { span: span() }),
        else_branch: None,
        span: span(),
    };

    assert!(CapabilityChecker::new().verify(&workflow).is_ok());
}

#[test]
fn check_rejects_policy_targets() {
    let workflow = Workflow::Check {
        target: CheckTarget::Policy(PolicyInstance {
            name: "RateLimit".into(),
            fields: vec![],
            span: span(),
        }),
        continuation: None,
        span: span(),
    };

    let error = CapabilityChecker::new()
        .verify(&workflow)
        .expect_err("check should only accept obligation targets");

    assert!(matches!(error, CapabilityCheckError::InvalidCheckTarget));
}

#[test]
fn check_accepts_obligation_targets() {
    let workflow = Workflow::Check {
        target: CheckTarget::Obligation(ObligationRef {
            role: "operator".into(),
            condition: Expr::Literal(Literal::Bool(true)),
        }),
        continuation: None,
        span: span(),
    };

    assert!(CapabilityChecker::new().verify(&workflow).is_ok());
}
