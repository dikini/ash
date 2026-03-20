use ash_core::{Expr as CoreExpr, Workflow as CoreWorkflow};
use ash_parser::input::new_input;
use ash_parser::lower::lower_workflow;
use ash_parser::parse_workflow::workflow_def;

#[test]
fn decide_lowering_preserves_explicit_policy_and_continuation() {
    let mut input = new_input("workflow main { decide { approved } under policy_gate then done }");
    let surface = workflow_def(&mut input).expect("workflow should parse");
    let lowered = lower_workflow(&surface);

    match lowered {
        CoreWorkflow::Decide {
            expr,
            policy,
            continuation,
        } => {
            assert!(matches!(expr, CoreExpr::Variable(ref name) if name == "approved"));
            assert_eq!(policy, "policy_gate");
            assert!(matches!(continuation.as_ref(), CoreWorkflow::Done));
        }
        other => panic!("expected lowered decide workflow, got {other:?}"),
    }
}

#[test]
fn check_lowering_preserves_obligation_identity() {
    let mut input = new_input("workflow main { check admin.is_active }");
    let surface = workflow_def(&mut input).expect("workflow should parse");
    let lowered = lower_workflow(&surface);

    match lowered {
        CoreWorkflow::Check {
            obligation,
            continuation,
        } => {
            assert!(matches!(
                obligation,
                ash_core::Obligation::Obliged {
                    role: ash_core::Role { ref name, .. },
                    condition: CoreExpr::Variable(ref condition),
                } if name == "admin" && condition == "is_active"
            ));
            assert!(matches!(continuation.as_ref(), CoreWorkflow::Done));
        }
        other => panic!("expected lowered check workflow, got {other:?}"),
    }
}
