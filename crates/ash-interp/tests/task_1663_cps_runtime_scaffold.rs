//! TASK-1663: CPS runtime memo scaffold
//!
//! Regression tests for process-local memo-cell runtime construction and sharing.

use ash_core::cps::*;
use ash_interp::cps::{CpsError, CpsRuntime, MemoCellState, eval_unchecked_with_runtime};

fn memo_thunk_body() -> Value {
    Value::ThunkClosure {
        mode: ThunkMode::Memo,
        body: Box::new(Value::Lam {
            params: vec![],
            cont: "__force_result".to_string(),
            body: Box::new(Term::Jump {
                cont: ContRef::Var("__force_result".to_string()),
                arg: Atom::Int(10),
                row: EffectRow::default(),
            }),
            captured_env: Env::new(),
            rec_binding: None,
            row: EffectRow::default(),
        }),
        captured_env: Env::new(),
        captured_chain: HandlerChain::new(),
        row: EffectRow::default(),
        memo_cell: None,
    }
}

#[test]
fn allocates_memo_cell_at_value_construction() {
    let term = Term::LetVal {
        name: "t".to_string(),
        value: memo_thunk_body(),
        body: Box::new(Term::Jump {
            cont: ContRef::Label("out".to_string()),
            arg: Atom::Int(0),
            row: EffectRow::default(),
        }),
    };
    let term = Term::LetCont {
        name: "out".to_string(),
        param: "r".to_string(),
        cont_body: Box::new(Term::Trap {
            reason: TrapReason::Custom("out".to_string()),
        }),
        body: Box::new(term),
        row: EffectRow::default(),
        multiplicity: ContMultiplicity::Affine,
    };

    let mut runtime = CpsRuntime::new();
    let result =
        eval_unchecked_with_runtime(&term, &Env::new(), &HandlerChain::new(), &mut runtime);

    assert!(matches!(result, Err(CpsError::Trap(TrapReason::Custom(_)))));
    assert_eq!(
        runtime.memo_cells.len(),
        1,
        "memo cell should be allocated during value eval"
    );
    assert!(
        runtime
            .trace
            .iter()
            .any(|event| matches!(event, ash_core::TraceEvent::ThunkConstructed { .. })),
        "memo thunk construction should emit a trace event"
    );
}

#[test]
fn shared_memo_cell_is_used_by_multiple_forces() {
    let term = Term::LetCont {
        name: "out".to_string(),
        param: "r".to_string(),
        cont_body: Box::new(Term::Trap {
            reason: TrapReason::Custom("out".to_string()),
        }),
        body: Box::new(Term::LetVal {
            name: "t".to_string(),
            value: memo_thunk_body(),
            body: Box::new(Term::LetPrim {
                name: "first".to_string(),
                op: PrimOp::ForceThunk,
                args: vec![Atom::Var("t".to_string())],
                body: Box::new(Term::LetPrim {
                    name: "second".to_string(),
                    op: PrimOp::ForceThunk,
                    args: vec![Atom::Var("t".to_string())],
                    body: Box::new(Term::Jump {
                        cont: ContRef::Var("out".to_string()),
                        arg: Atom::Var("second".to_string()),
                        row: EffectRow::default(),
                    }),
                }),
            }),
        }),
        row: EffectRow::default(),
        multiplicity: ContMultiplicity::Affine,
    };

    let mut runtime = CpsRuntime::new();
    let result =
        eval_unchecked_with_runtime(&term, &Env::new(), &HandlerChain::new(), &mut runtime);

    assert!(matches!(result, Err(CpsError::Trap(TrapReason::Custom(_)))));
    assert_eq!(runtime.memo_cells.len(), 1);
    for state in runtime.memo_cells.values() {
        assert!(
            matches!(state, MemoCellState::Filled(_)),
            "memo cell should be filled after forcing twice"
        );
    }
}

#[test]
fn forcing_non_thunk_is_expected_thunk_error() {
    let term = Term::LetCont {
        name: "out".to_string(),
        param: "r".to_string(),
        cont_body: Box::new(Term::Trap {
            reason: TrapReason::Custom("out".to_string()),
        }),
        body: Box::new(Term::LetPrim {
            name: "r".to_string(),
            op: PrimOp::ForceThunk,
            args: vec![Atom::Int(1)],
            body: Box::new(Term::Jump {
                cont: ContRef::Label("out".to_string()),
                arg: Atom::Var("r".to_string()),
                row: EffectRow::default(),
            }),
        }),
        row: EffectRow::default(),
        multiplicity: ContMultiplicity::Affine,
    };

    let mut runtime = CpsRuntime::new();
    let result =
        eval_unchecked_with_runtime(&term, &Env::new(), &HandlerChain::new(), &mut runtime);

    assert!(
        matches!(result, Err(CpsError::ExpectedThunk(Value::Atom(_)))),
        "forcing anything other than thunk closure must return ExpectedThunk"
    );
}
