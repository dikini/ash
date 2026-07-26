//! TASK-2003: checked CPS terminal projection regression tests.
//!
//! The canonical CPS calculus treats `Return` and `Trap` as distinct terminal
//! observations.  A malformed terminal term must be rejected by the checked
//! boundary rather than projected as an ordinary return.

use ash_core::cps::{Atom, Env, HandlerChain, Term, TrapReason, Value as CpsValue};
use ash_interp::cps::{
    CpsRunError, CpsTerminalOutcome, eval_checked_terminal, validate::CpsValidationError,
};

#[test]
fn checked_return_projects_a_distinct_terminal_return_observation() {
    let term = Term::Return {
        value: CpsValue::Atom(Atom::Int(42)),
    };

    let outcome = eval_checked_terminal(&term, &Env::new(), &HandlerChain::new());

    assert_eq!(
        outcome,
        Ok(CpsTerminalOutcome::Return(CpsValue::Atom(Atom::Int(42))))
    );
}

#[test]
fn checked_return_projects_a_recursive_tagged_terminal_value() {
    let value = CpsValue::Record {
        fields: vec![
            (
                "tag".to_string(),
                CpsValue::Atom(Atom::String("Err".to_string())),
            ),
            (
                "error".to_string(),
                CpsValue::Record {
                    fields: vec![
                        (
                            "tag".to_string(),
                            CpsValue::Atom(Atom::String("RuntimeError".to_string())),
                        ),
                        (
                            "fields".to_string(),
                            CpsValue::Tuple {
                                elems: vec![
                                    CpsValue::Atom(Atom::Int(42)),
                                    CpsValue::Atom(Atom::String("boom".to_string())),
                                ],
                            },
                        ),
                    ],
                },
            ),
        ],
    };
    let term = Term::Return {
        value: value.clone(),
    };

    let outcome = eval_checked_terminal(&term, &Env::new(), &HandlerChain::new());

    assert_eq!(outcome, Ok(CpsTerminalOutcome::Return(value)));
}

#[test]
fn checked_trap_projects_a_distinct_terminal_trap_observation() {
    let reason = TrapReason::Custom("declared primitive-domain failure".to_string());
    let term = Term::Trap {
        reason: reason.clone(),
    };

    let outcome = eval_checked_terminal(&term, &Env::new(), &HandlerChain::new());

    assert_eq!(outcome, Ok(CpsTerminalOutcome::Trap(reason)));
}

#[test]
fn malformed_terminal_return_is_rejected_before_observable_projection() {
    let term = Term::Return {
        value: CpsValue::Atom(Atom::Var("unbound_terminal_value".to_string())),
    };

    let error = eval_checked_terminal(&term, &Env::new(), &HandlerChain::new())
        .expect_err("an unbound terminal value must fail checked validation");

    assert_eq!(
        error,
        CpsRunError::Validation(CpsValidationError::UnresolvedVariable(
            "unbound_terminal_value".to_string()
        ))
    );
}
