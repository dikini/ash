use ash_parser::input::new_input;
use ash_parser::parse_workflow::workflow;
use ash_parser::surface::{CheckTarget, Workflow};

#[test]
fn decide_requires_explicit_named_policy() {
    let mut input = new_input("decide { approved } under policy_gate then done");
    let parsed = workflow(&mut input).expect("canonical decide should parse");

    match parsed {
        Workflow::Decide {
            expr,
            policy,
            else_branch,
            ..
        } => {
            assert!(
                matches!(expr, ash_parser::surface::Expr::Variable(ref name) if name.as_ref() == "approved")
            );
            assert!(matches!(policy, Some(ref name) if name.as_ref() == "policy_gate"));
            assert!(else_branch.is_none());
        }
        other => panic!("expected decide workflow, got {other:?}"),
    }
}

#[test]
fn check_is_obligation_only() {
    let mut input = new_input("check admin.is_active");
    let parsed = workflow(&mut input).expect("obligation check should parse");

    match parsed {
        Workflow::Check { target, .. } => {
            assert!(matches!(target, CheckTarget::Obligation(_)));
        }
        other => panic!("expected check workflow, got {other:?}"),
    }
}

#[test]
fn check_rejects_policy_instances() {
    let mut input = new_input("check RateLimit { requests: 100 }");
    assert!(
        workflow(&mut input).is_err(),
        "policy instances are not valid check targets"
    );
}

#[test]
fn decide_rejects_missing_under_clause() {
    let mut input = new_input("decide { approved } then done");
    assert!(
        workflow(&mut input).is_err(),
        "decide without under <policy> must be rejected"
    );
}
