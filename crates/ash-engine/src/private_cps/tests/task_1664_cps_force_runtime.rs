//! TASK-1664: CPS thunk force runtime behavior.

use super::super::{
    CachedThunkOutcome, CpsError, CpsRuntime, MemoCellState, eval_unchecked_with_runtime,
};
use ash_core::cps::{
    Atom, ContMultiplicity, ContRef, EffectItem, EffectItemKind, EffectOp, EffectRow, Env,
    HandlerChain, MemoCellId, PrimOp, Term, ThunkMode, Value,
};

fn thunk_out_cont(body: Term) -> Term {
    Term::LetCont {
        name: "out".into(),
        param: "r".into(),
        cont_body: Box::new(Term::Return {
            value: Value::Atom(Atom::Var("r".into())),
        }),
        body: Box::new(body),
        row: EffectRow::default(),
        multiplicity: ContMultiplicity::Affine,
    }
}

fn memo_thunk(body: Value, row: EffectRow) -> Value {
    Value::ThunkClosure {
        mode: ThunkMode::Memo,
        body: Box::new(body),
        captured_env: Env::new(),
        captured_chain: HandlerChain::new(),
        row,
        memo_cell: None,
    }
}

fn lazy_thunk(body: Value, row: EffectRow) -> Value {
    Value::ThunkClosure {
        mode: ThunkMode::Lazy,
        body: Box::new(body),
        captured_env: Env::new(),
        captured_chain: HandlerChain::new(),
        row,
        memo_cell: None,
    }
}

fn force_with_two_uses(thunk: Value) -> Term {
    Term::LetVal {
        name: "t".into(),
        value: thunk,
        body: Box::new(Term::LetPrim {
            name: "first".into(),
            op: PrimOp::ForceThunk,
            args: vec![Atom::Var("t".into())],
            body: Box::new(Term::LetPrim {
                name: "second".into(),
                op: PrimOp::ForceThunk,
                args: vec![Atom::Var("t".into())],
                body: Box::new(Term::Jump {
                    cont: ContRef::Label("out".into()),
                    arg: Atom::Var("second".into()),
                    row: EffectRow::default(),
                }),
            }),
        }),
    }
}

#[test]
fn memo_force_success_is_cached_across_forces() {
    let thunk = memo_thunk(
        Value::Lam {
            params: vec![],
            cont: "k".into(),
            body: Box::new(Term::Jump {
                cont: ContRef::Var("k".into()),
                arg: Atom::Int(7),
                row: EffectRow::default(),
            }),
            captured_env: Env::new(),
            rec_binding: None,
            row: EffectRow::default(),
        },
        EffectRow::default(),
    );

    let term = thunk_out_cont(force_with_two_uses(thunk));
    let mut runtime = CpsRuntime::new();
    let result =
        eval_unchecked_with_runtime(&term, &Env::new(), &HandlerChain::new(), &mut runtime);

    assert_eq!(result, Ok(Value::Atom(Atom::Int(7))));
    assert_eq!(runtime.memo_cells.len(), 1);
    assert_eq!(
        runtime
            .memo_cells
            .values()
            .next()
            .expect("memo cell should exist"),
        &MemoCellState::Filled(CachedThunkOutcome::Success(Value::Atom(Atom::Int(7))))
    );
}

#[test]
fn memo_force_replays_trap_from_cell() {
    let op = EffectOp {
        item: EffectItem {
            namespace: "effects".into(),
            name: "missing".into(),
            kind: EffectItemKind::Alias,
        },
        arg_types: vec![],
        result_type: "Unit".into(),
    };

    let thunk = memo_thunk(
        Value::Lam {
            params: vec![],
            cont: "resume".into(),
            body: Box::new(Term::Raise {
                op: op.clone(),
                args: vec![],
                resume: ContRef::Var("resume".into()),
                row: EffectRow::default(),
            }),
            captured_env: Env::new(),
            rec_binding: None,
            row: EffectRow::default(),
        },
        EffectRow::default(),
    );

    let term = thunk_out_cont(force_with_two_uses(thunk));
    let mut runtime = CpsRuntime::new();
    let result =
        eval_unchecked_with_runtime(&term, &Env::new(), &HandlerChain::new(), &mut runtime);

    assert!(matches!(result, Err(CpsError::UnhandledEffect(inner)) if inner == op));
    assert_eq!(runtime.memo_cells.len(), 1);
    assert_eq!(
        runtime
            .memo_cells
            .values()
            .next()
            .expect("memo cell should exist"),
        &MemoCellState::Filled(CachedThunkOutcome::Failure(CpsError::UnhandledEffect(
            op.clone()
        ))),
    );
}

#[test]
fn lazy_force_does_not_use_memo_cells() {
    let op = EffectOp {
        item: EffectItem {
            namespace: "effects".into(),
            name: "unhandled".into(),
            kind: EffectItemKind::Alias,
        },
        arg_types: vec![],
        result_type: "Unit".into(),
    };

    let thunk = lazy_thunk(
        Value::Lam {
            params: vec![],
            cont: "resume".into(),
            body: Box::new(Term::Raise {
                op: op.clone(),
                args: vec![],
                resume: ContRef::Var("resume".into()),
                row: EffectRow::default(),
            }),
            captured_env: Env::new(),
            rec_binding: None,
            row: EffectRow::default(),
        },
        EffectRow::default(),
    );

    let term = thunk_out_cont(force_with_two_uses(thunk));
    let mut runtime = CpsRuntime::new();
    let result =
        eval_unchecked_with_runtime(&term, &Env::new(), &HandlerChain::new(), &mut runtime);

    assert!(matches!(result, Err(CpsError::UnhandledEffect(inner)) if inner == op));
    assert_eq!(runtime.memo_cells.len(), 0);
}

#[test]
fn synthetic_force_continuation_is_used() {
    let thunk = memo_thunk(
        Value::Lam {
            params: vec![],
            cont: "result_slot".into(),
            body: Box::new(Term::Jump {
                cont: ContRef::Var("result_slot".into()),
                arg: Atom::Int(19),
                row: EffectRow::default(),
            }),
            captured_env: Env::new(),
            rec_binding: None,
            row: EffectRow::default(),
        },
        EffectRow::default(),
    );

    let term = thunk_out_cont(force_with_two_uses(thunk));
    let mut runtime = CpsRuntime::new();
    let result =
        eval_unchecked_with_runtime(&term, &Env::new(), &HandlerChain::new(), &mut runtime);

    assert_eq!(result, Ok(Value::Atom(Atom::Int(19))));
}

#[test]
fn memo_force_without_cacheable_error_resets_memo_state_to_empty() {
    let mut runtime = CpsRuntime::new();
    let cell_id: MemoCellId = runtime.allocate_memo_cell();
    let thunk = Value::ThunkClosure {
        mode: ThunkMode::Memo,
        body: Box::new(Value::Lam {
            params: vec![],
            cont: "k".into(),
            body: Box::new(Term::Jump {
                cont: ContRef::Var("k".into()),
                arg: Atom::Var("not_bound".into()),
                row: EffectRow::default(),
            }),
            captured_env: Env::new(),
            rec_binding: None,
            row: EffectRow::default(),
        }),
        captured_env: Env::new(),
        captured_chain: HandlerChain::new(),
        row: EffectRow::default(),
        memo_cell: Some(cell_id),
    };

    let env = Env::new().with_binding("t".into(), thunk);
    let term = thunk_out_cont(Term::LetPrim {
        name: "x".into(),
        op: PrimOp::ForceThunk,
        args: vec![Atom::Var("t".into())],
        body: Box::new(Term::Jump {
            cont: ContRef::Label("out".into()),
            arg: Atom::Var("x".into()),
            row: EffectRow::default(),
        }),
    });

    let chain = HandlerChain::new();
    let first = eval_unchecked_with_runtime(&term, &env, &chain, &mut runtime);
    assert!(matches!(
        first,
        Err(CpsError::UnboundVariable(name)) if name == "not_bound"
    ));
    assert_eq!(
        runtime.memo_cells.get(&cell_id),
        Some(&MemoCellState::Empty)
    );

    let second = eval_unchecked_with_runtime(&term, &env, &chain, &mut runtime);
    assert!(matches!(
        second,
        Err(CpsError::UnboundVariable(name)) if name == "not_bound"
    ));
    assert_eq!(
        runtime.memo_cells.get(&cell_id),
        Some(&MemoCellState::Empty)
    );
}
