//! TASK-1672: Thunk tracing and observability.

use super::super::{CpsError, CpsRuntime, MemoCellState, eval_unchecked_with_runtime};
use ash_core::cps::{
    Atom, ContMultiplicity, ContRef, EffectItem, EffectItemKind, EffectOp, EffectRow, Env,
    HandlerChain, MemoCellId, PrimOp, Term, ThunkMode, Value,
};
use ash_core::provenance::TraceEvent;

fn memoized_thunk_body() -> Value {
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

fn lazy_thunk_body() -> Value {
    Value::ThunkClosure {
        mode: ThunkMode::Lazy,
        body: Box::new(Value::Lam {
            params: vec![],
            cont: "__force_result".to_string(),
            body: Box::new(Term::Jump {
                cont: ContRef::Var("__force_result".to_string()),
                arg: Atom::Int(11),
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

fn force_twice_term(thunk: Value, force_row: EffectRow) -> Term {
    Term::LetCont {
        name: "out".to_string(),
        param: "r".to_string(),
        cont_body: Box::new(Term::Return {
            value: Value::Atom(Atom::Var("r".to_string())),
        }),
        body: Box::new(Term::LetVal {
            name: "t".to_string(),
            value: thunk,
            body: Box::new(Term::LetPrim {
                name: "first".to_string(),
                op: PrimOp::ForceThunk,
                args: vec![Atom::Var("t".to_string())],
                body: Box::new(Term::LetPrim {
                    name: "second".to_string(),
                    op: PrimOp::ForceThunk,
                    args: vec![Atom::Var("t".to_string())],
                    body: Box::new(Term::Jump {
                        cont: ContRef::Label("out".to_string()),
                        arg: Atom::Var("second".to_string()),
                        row: force_row,
                    }),
                }),
            }),
        }),
        row: EffectRow::default(),
        multiplicity: ContMultiplicity::Affine,
    }
}

fn force_once_term(thunk: Value, force_row: EffectRow) -> Term {
    Term::LetCont {
        name: "out".to_string(),
        param: "r".to_string(),
        cont_body: Box::new(Term::Return {
            value: Value::Atom(Atom::Var("r".to_string())),
        }),
        body: Box::new(Term::LetVal {
            name: "t".to_string(),
            value: thunk,
            body: Box::new(Term::LetPrim {
                name: "forced".to_string(),
                op: PrimOp::ForceThunk,
                args: vec![Atom::Var("t".to_string())],
                body: Box::new(Term::Jump {
                    cont: ContRef::Label("out".to_string()),
                    arg: Atom::Var("forced".to_string()),
                    row: force_row,
                }),
            }),
        }),
        row: EffectRow::default(),
        multiplicity: ContMultiplicity::Affine,
    }
}

fn force_variable_once_term(name: &str, force_row: EffectRow) -> Term {
    Term::LetCont {
        name: "out".to_string(),
        param: "r".to_string(),
        cont_body: Box::new(Term::Return {
            value: Value::Atom(Atom::Var("r".to_string())),
        }),
        body: Box::new(Term::LetPrim {
            name: "forced".to_string(),
            op: PrimOp::ForceThunk,
            args: vec![Atom::Var(name.to_string())],
            body: Box::new(Term::Jump {
                cont: ContRef::Label("out".to_string()),
                arg: Atom::Var("forced".to_string()),
                row: force_row,
            }),
        }),
        row: EffectRow::default(),
        multiplicity: ContMultiplicity::Affine,
    }
}

fn event_count<T: FnMut(&TraceEvent) -> bool>(trace: &[TraceEvent], mut pred: T) -> usize {
    trace.iter().filter(|event| pred(event)).count()
}

fn alias_effect_op(namespace: &str, name: &str) -> EffectOp {
    EffectOp {
        item: EffectItem {
            namespace: namespace.to_string(),
            name: name.to_string(),
            kind: EffectItemKind::Alias,
        },
        arg_types: vec![],
        result_type: "Unit".to_string(),
    }
}

#[test]
fn lazy_force_trace_emits_force_and_body_events_without_cache_events() {
    let term = force_once_term(lazy_thunk_body(), EffectRow::default());

    let mut runtime = CpsRuntime::new();
    let result =
        eval_unchecked_with_runtime(&term, &Env::new(), &HandlerChain::new(), &mut runtime);

    assert_eq!(result, Ok(Value::Atom(Atom::Int(11))));

    assert_eq!(
        event_count(&runtime.trace, |event| matches!(
            event,
            TraceEvent::ThunkConstructed { .. }
                if matches!(event, TraceEvent::ThunkConstructed { mode, .. } if mode == "lazy")
        )),
        1
    );
    assert_eq!(
        event_count(&runtime.trace, |event| {
            matches!(
                event,
                TraceEvent::ThunkForceStarted { .. }
                    if matches!(event, TraceEvent::ThunkForceStarted { mode, .. } if mode == "lazy")
            )
        }),
        1
    );
    assert_eq!(
        event_count(&runtime.trace, |event| {
            matches!(
                event,
                TraceEvent::ThunkBodyEvaluationStarted { .. }
                    if matches!(event, TraceEvent::ThunkBodyEvaluationStarted { mode, .. } if mode == "lazy")
            )
        }),
        1
    );
    assert_eq!(
        event_count(&runtime.trace, |event| {
            matches!(
                event,
                TraceEvent::ThunkBodyEvaluationCompleted { .. }
                    if matches!(
                        event,
                        TraceEvent::ThunkBodyEvaluationCompleted {
                            mode,
                            outcome,
                            ..
                        } if mode == "lazy" && outcome == "success"
                    )
            )
        }),
        1
    );
    assert_eq!(
        event_count(&runtime.trace, |event| {
            matches!(
                event,
                TraceEvent::ThunkForceCompleted { .. }
                    if matches!(
                        event,
                        TraceEvent::ThunkForceCompleted {
                            mode,
                            outcome,
                            ..
                        } if mode == "lazy" && outcome == "success"
                    )
            )
        }),
        1
    );

    assert_eq!(
        event_count(&runtime.trace, |event| {
            matches!(
                event,
                TraceEvent::MemoCacheFilled { .. }
                    | TraceEvent::MemoCacheHit { .. }
                    | TraceEvent::MemoReplayFailure { .. }
                    | TraceEvent::MemoReentrantRejected { .. }
            )
        }),
        0
    );
}

#[test]
fn memo_force_trace_reuses_cached_value_on_second_force() {
    let term = force_twice_term(memoized_thunk_body(), EffectRow::default());

    let mut runtime = CpsRuntime::new();
    let result =
        eval_unchecked_with_runtime(&term, &Env::new(), &HandlerChain::new(), &mut runtime);

    assert_eq!(result, Ok(Value::Atom(Atom::Int(10))));

    assert_eq!(
        event_count(&runtime.trace, |event| matches!(
            event,
            TraceEvent::ThunkConstructed { .. }
                if matches!(event, TraceEvent::ThunkConstructed { mode, .. } if mode == "memo")
        )),
        1,
    );
    assert_eq!(
        event_count(&runtime.trace, |event| matches!(
            event,
            TraceEvent::ThunkForceStarted { .. }
                if matches!(event, TraceEvent::ThunkForceStarted { mode, .. } if mode == "memo")
        )),
        2
    );
    assert_eq!(
        event_count(&runtime.trace, |event| {
            matches!(
                event,
                TraceEvent::ThunkBodyEvaluationStarted { .. }
                    if matches!(
                        event,
                        TraceEvent::ThunkBodyEvaluationStarted {
                            mode,
                            ..
                        } if mode == "memo"
                    )
            )
        }),
        1
    );
    assert_eq!(
        event_count(&runtime.trace, |event| {
            matches!(
                event,
                TraceEvent::ThunkBodyEvaluationCompleted { .. }
                    if matches!(
                        event,
                        TraceEvent::ThunkBodyEvaluationCompleted {
                            mode,
                            outcome,
                            ..
                        } if mode == "memo" && outcome == "success"
                    )
            )
        }),
        1
    );
    assert_eq!(
        event_count(&runtime.trace, |event| {
            matches!(
                event,
                TraceEvent::MemoCacheFilled { .. }
                    if matches!(event, TraceEvent::MemoCacheFilled { outcome, .. } if outcome == "success")
            )
        }),
        1
    );
    assert_eq!(
        event_count(&runtime.trace, |event| {
            matches!(
                event,
                TraceEvent::MemoCacheHit { .. }
                    if matches!(event, TraceEvent::MemoCacheHit { outcome, .. } if outcome == "success")
            )
        }),
        1
    );
}

#[test]
fn memo_force_trace_replays_cached_trap_without_repeating_body() {
    let trap_op = alias_effect_op("cap", "io.read");
    let mut runtime = CpsRuntime::new();
    let memo_cell: MemoCellId = runtime.allocate_memo_cell();

    let body = Value::Lam {
        params: vec![],
        cont: "resume".to_string(),
        body: Box::new(Term::Raise {
            op: trap_op.clone(),
            args: vec![],
            resume: ContRef::Var("resume".to_string()),
            row: EffectRow::default(),
        }),
        captured_env: Env::new(),
        rec_binding: None,
        row: EffectRow::default(),
    };

    let thunk = Value::ThunkClosure {
        mode: ThunkMode::Memo,
        body: Box::new(body),
        captured_env: Env::new(),
        captured_chain: HandlerChain::new(),
        row: EffectRow::default(),
        memo_cell: Some(memo_cell),
    };

    let env = Env::new().with_binding("t".to_string(), thunk.clone());
    let term = force_variable_once_term("t", EffectRow::default());

    let first = eval_unchecked_with_runtime(&term, &env, &HandlerChain::new(), &mut runtime);
    assert!(matches!(
        first,
        Err(CpsError::UnhandledEffect(inner)) if inner == trap_op
    ));

    let first_trace_len = runtime.trace.len();
    assert_eq!(
        event_count(&runtime.trace[..first_trace_len], |event| {
            matches!(
                event,
                TraceEvent::ThunkBodyEvaluationStarted { .. }
                    if matches!(
                        event,
                        TraceEvent::ThunkBodyEvaluationStarted {
                            mode,
                            ..
                        } if mode == "memo"
                    )
            )
        }),
        1
    );
    assert_eq!(
        event_count(&runtime.trace[..first_trace_len], |event| {
            matches!(
                event,
                TraceEvent::ThunkBodyEvaluationCompleted { .. }
                    if matches!(
                        event,
                        TraceEvent::ThunkBodyEvaluationCompleted {
                            mode,
                            outcome,
                            ..
                        } if mode == "memo" && outcome == "unhandled-effect"
                    )
            )
        }),
        1
    );
    assert_eq!(
        event_count(&runtime.trace[..first_trace_len], |event| {
            matches!(
                event,
                TraceEvent::MemoCacheFilled { .. }
                    if matches!(
                        event,
                        TraceEvent::MemoCacheFilled { outcome, .. } if outcome == "unhandled-effect"
                    )
            )
        }),
        1
    );

    let second = eval_unchecked_with_runtime(&term, &env, &HandlerChain::new(), &mut runtime);
    assert!(matches!(
        second,
        Err(CpsError::UnhandledEffect(inner)) if inner == trap_op
    ));

    let second_trace_len = runtime.trace.len();
    let second_trace = &runtime.trace[first_trace_len..second_trace_len];
    assert_eq!(
        event_count(second_trace, |event| {
            matches!(
                event,
                TraceEvent::ThunkBodyEvaluationStarted { .. }
                    if matches!(
                        event,
                        TraceEvent::ThunkBodyEvaluationStarted {
                            mode,
                            ..
                        } if mode == "memo"
                    )
            )
        }),
        0
    );
    assert_eq!(
        event_count(second_trace, |event| {
            matches!(
                event,
                TraceEvent::ThunkBodyEvaluationCompleted { .. }
                    if matches!(
                        event,
                        TraceEvent::ThunkBodyEvaluationCompleted {
                            mode,
                            ..
                        } if mode == "memo"
                    )
            )
        }),
        0
    );
    assert_eq!(
        event_count(second_trace, |event| {
            matches!(
                event,
                TraceEvent::MemoReplayFailure { .. }
                    if matches!(
                        event,
                        TraceEvent::MemoReplayFailure { reason, .. } if reason == "unhandled-effect"
                    )
            )
        }),
        1
    );
}

#[test]
fn memo_force_trace_emits_reentrant_rejection_when_cell_is_evaluating() {
    let mut runtime = CpsRuntime::new();
    let memo_cell: MemoCellId = runtime.allocate_memo_cell();
    runtime
        .memo_cells
        .insert(memo_cell, MemoCellState::Evaluating);

    let thunk = Value::ThunkClosure {
        mode: ThunkMode::Memo,
        body: Box::new(Value::Lam {
            params: vec![],
            cont: "resume".to_string(),
            body: Box::new(Term::Jump {
                cont: ContRef::Var("resume".to_string()),
                arg: Atom::Int(19),
                row: EffectRow::default(),
            }),
            captured_env: Env::new(),
            rec_binding: None,
            row: EffectRow::default(),
        }),
        captured_env: Env::new(),
        captured_chain: HandlerChain::new(),
        row: EffectRow::default(),
        memo_cell: Some(memo_cell),
    };

    let term = force_once_term(thunk, EffectRow::default());
    let result =
        eval_unchecked_with_runtime(&term, &Env::new(), &HandlerChain::new(), &mut runtime);

    let Err(CpsError::Trap(ash_core::cps::TrapReason::Custom(message))) = result else {
        panic!("expected re-entrant memo force trap");
    };
    assert!(message.contains("re-entrant memo force"));

    assert_eq!(
        event_count(&runtime.trace, |event| {
            matches!(
                event,
                TraceEvent::ThunkForceStarted { .. }
                    if matches!(event, TraceEvent::ThunkForceStarted { mode, .. } if mode == "memo")
            )
        }),
        1
    );
    assert_eq!(
        event_count(&runtime.trace, |event| {
            matches!(
                event,
                TraceEvent::ThunkForceCompleted { .. }
                    if matches!(
                        event,
                        TraceEvent::ThunkForceCompleted {
                            mode,
                            outcome,
                            ..
                        } if mode == "memo" && outcome == "trap"
                    )
            )
        }),
        1
    );
    assert_eq!(
        event_count(&runtime.trace, |event| matches!(
            event,
            TraceEvent::MemoReentrantRejected { .. }
        )),
        1
    );
    assert_eq!(
        event_count(&runtime.trace, |event| {
            matches!(
                event,
                TraceEvent::ThunkBodyEvaluationStarted { .. }
                    | TraceEvent::ThunkBodyEvaluationCompleted { .. }
            )
        }),
        0
    );
}
