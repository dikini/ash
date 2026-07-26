//! TASK-2006: downstream public CPS API boundary regression tests.
//!
//! This integration-test crate consumes only the exported CPS carrier and
//! evaluator paths. It keeps checked validation and trusted-IR evaluation
//! distinct without treating the CPS prototype as production Ash execution.

use ash_core::cps::{Atom, Env, HandlerChain, Term, Value};
use ash_interp::cps::{
    CpsRunError, CpsTerminalOutcome, eval_checked, eval_checked_terminal, eval_unchecked,
    validate::CpsValidationError,
};

#[test]
fn public_checked_and_trusted_unchecked_entrypoints_agree_for_well_formed_ir() {
    let term = Term::Return {
        value: Value::Atom(Atom::Int(42)),
    };
    let env = Env::new();
    let chain = HandlerChain::new();

    assert_eq!(
        eval_checked(&term, &env, &chain).expect("the fixture is valid CPS"),
        eval_unchecked(&term, &env, &chain)
            .expect("unchecked evaluation is available only for this trusted fixture"),
    );
}

#[test]
fn public_checked_entrypoint_rejects_malformed_ir_before_evaluation() {
    let term = Term::Return {
        value: Value::Atom(Atom::Var("unbound".to_string())),
    };

    assert_eq!(
        eval_checked(&term, &Env::new(), &HandlerChain::new()),
        Err(CpsRunError::Validation(
            CpsValidationError::UnresolvedVariable("unbound".to_string(),)
        )),
    );
}

#[test]
fn public_terminal_projection_is_available_to_downstream_consumers() {
    let term = Term::Return {
        value: Value::Atom(Atom::String("done".to_string())),
    };

    assert_eq!(
        eval_checked_terminal(&term, &Env::new(), &HandlerChain::new()),
        Ok(CpsTerminalOutcome::Return(Value::Atom(Atom::String(
            "done".to_string(),
        )))),
    );
}
