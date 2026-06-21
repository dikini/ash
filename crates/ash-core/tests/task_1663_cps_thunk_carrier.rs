//! TASK-1663: Add CPS thunk carrier
//!
//! Regression tests for new CPS thunk carrier primitives used by lazy/memo semantics.

use ash_core::cps::{Atom, ContRef, EffectItem, EffectItemKind};
use ash_core::cps::{EffectRow, Env, HandlerChain, MemoCellId, PrimOp, Term, ThunkMode, Value};
use ash_core::sexp::{read_term_from_file, write_term_to_file};
use std::path::PathBuf;

fn temp_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from("/tmp");
    path.push(format!("ash_cps_thunk_carrier_{}.cps", name));
    path
}

fn cleanup(path: &PathBuf) {
    let _ = std::fs::remove_file(path);
}

fn zero_arg_lambda_returning_int(value: i64) -> Value {
    Value::Lam {
        params: vec![],
        cont: "__force_result".to_string(),
        body: Box::new(Term::Jump {
            cont: ContRef::Var("__force_result".to_string()),
            arg: Atom::Int(value),
            row: EffectRow::default(),
        }),
        captured_env: Env::new(),
        rec_binding: None,
        row: EffectRow::default(),
    }
}

fn sample_row() -> EffectRow {
    EffectRow {
        items: vec![EffectItem {
            namespace: "cap".to_string(),
            name: "db.read".to_string(),
            kind: EffectItemKind::Capability,
        }],
    }
}

#[test]
fn constructs_thunk_closure_variants() {
    let lazy = Value::ThunkClosure {
        mode: ThunkMode::Lazy,
        body: Box::new(zero_arg_lambda_returning_int(1)),
        captured_env: Env::new(),
        captured_chain: HandlerChain::new(),
        row: sample_row(),
        memo_cell: None,
    };

    let memo = Value::ThunkClosure {
        mode: ThunkMode::Memo,
        body: Box::new(zero_arg_lambda_returning_int(2)),
        captured_env: Env::new(),
        captured_chain: HandlerChain::new(),
        row: sample_row(),
        memo_cell: None,
    };

    match lazy {
        Value::ThunkClosure {
            mode,
            body,
            memo_cell,
            ..
        } => {
            assert_eq!(mode, ThunkMode::Lazy);
            assert!(matches!(*body, Value::Lam { .. }));
            assert!(memo_cell.is_none());
        }
        _ => panic!("expected thunk closure"),
    }

    match memo {
        Value::ThunkClosure {
            mode,
            body,
            memo_cell,
            ..
        } => {
            assert_eq!(mode, ThunkMode::Memo);
            assert!(matches!(*body, Value::Lam { .. }));
            assert!(memo_cell.is_none());
        }
        _ => panic!("expected thunk closure"),
    }
}

#[test]
fn memo_cell_id_api_is_available_and_encapsulated() {
    let id = MemoCellId::new(7);
    let row = EffectRow::default();
    assert_eq!(id.raw(), 7);
    let thunk = Value::ThunkClosure {
        mode: ThunkMode::Memo,
        body: Box::new(zero_arg_lambda_returning_int(3)),
        captured_env: Env::new(),
        captured_chain: HandlerChain::new(),
        row,
        memo_cell: Some(id),
    };

    match thunk {
        Value::ThunkClosure { memo_cell, .. } => {
            assert_eq!(
                memo_cell,
                Some(MemoCellId::new(7)),
                "closure preserves explicitly set memo cell ids"
            );
        }
        _ => panic!("expected thunk closure"),
    }
}

#[test]
fn thunk_closure_force_primitive_roundtrip() {
    assert_eq!(PrimOp::ForceThunk, PrimOp::ForceThunk);
}

#[test]
fn thunk_closure_round_trips_without_memo_cell_payload() {
    let thunk = Value::ThunkClosure {
        mode: ThunkMode::Memo,
        body: Box::new(zero_arg_lambda_returning_int(42)),
        captured_env: Env::new(),
        captured_chain: HandlerChain::new(),
        row: EffectRow::default(),
        memo_cell: Some(MemoCellId::new(12345)),
    };

    let term = Term::LetVal {
        name: "thunk".to_string(),
        value: thunk,
        body: Box::new(Term::Jump {
            cont: ContRef::Label("exit".to_string()),
            arg: Atom::Var("thunk".to_string()),
            row: EffectRow::default(),
        }),
    };

    let path = temp_path("memo_closure_roundtrip");
    write_term_to_file(&term, &path).unwrap();
    let parsed = read_term_from_file(&path).unwrap();
    cleanup(&path);

    let expected = Term::LetVal {
        name: "thunk".to_string(),
        value: Value::ThunkClosure {
            mode: ThunkMode::Memo,
            body: Box::new(zero_arg_lambda_returning_int(42)),
            captured_env: Env::new(),
            captured_chain: HandlerChain::new(),
            row: EffectRow::default(),
            memo_cell: None,
        },
        body: Box::new(Term::Jump {
            cont: ContRef::Label("exit".to_string()),
            arg: Atom::Var("thunk".to_string()),
            row: EffectRow::default(),
        }),
    };

    assert_eq!(parsed, expected);
}
