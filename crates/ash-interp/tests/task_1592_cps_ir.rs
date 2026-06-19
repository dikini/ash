//! TASK-1592: Conditionals and data tests
//!
//! Tests for If, LetRec, and RecordDischarge evaluation.

use ash_core::cps::*;
use ash_interp::cps::{CpsError, eval_term};

#[test]
fn test_eval_if_true() {
    // if true then (jump exit 1) else (jump exit 2)
    let term = Term::If {
        cond: Atom::Bool(true),
        then_branch: Box::new(Term::Jump {
            cont: ContRef::Label("exit".to_string()),
            arg: Atom::Int(1),
            row: EffectRow::default(),
        }),
        else_branch: Box::new(Term::Jump {
            cont: ContRef::Label("exit".to_string()),
            arg: Atom::Int(2),
            row: EffectRow::default(),
        }),
        row: EffectRow::default(),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::UnboundLabel(ref s)) if s == "exit"));
}

#[test]
fn test_eval_if_false() {
    // if false then (jump exit 1) else (jump exit 2)
    let term = Term::If {
        cond: Atom::Bool(false),
        then_branch: Box::new(Term::Jump {
            cont: ContRef::Label("exit".to_string()),
            arg: Atom::Int(1),
            row: EffectRow::default(),
        }),
        else_branch: Box::new(Term::Jump {
            cont: ContRef::Label("exit".to_string()),
            arg: Atom::Int(2),
            row: EffectRow::default(),
        }),
        row: EffectRow::default(),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::UnboundLabel(ref s)) if s == "exit"));
}

#[test]
fn test_eval_if_conditional() {
    // letcont exit [v] (trap return) in
    //   letprim cond = eq 1 1 in
    //     if cond then (jump exit 42) else (jump exit 0)
    let term = Term::LetCont {
        name: "exit".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Trap {
            reason: TrapReason::Custom("return".to_string()),
        }),
        body: Box::new(Term::LetPrim {
            name: "cond".to_string(),
            op: PrimOp::Eq,
            args: vec![Atom::Int(1), Atom::Int(1)],
            body: Box::new(Term::If {
                cond: Atom::Var("cond".to_string()),
                then_branch: Box::new(Term::Jump {
                    cont: ContRef::Label("exit".to_string()),
                    arg: Atom::Int(42),
                    row: EffectRow::default(),
                }),
                else_branch: Box::new(Term::Jump {
                    cont: ContRef::Label("exit".to_string()),
                    arg: Atom::Int(0),
                    row: EffectRow::default(),
                }),
                row: EffectRow::default(),
            }),
        }),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::Trap(TrapReason::Custom(ref s))) if s == "return"));
}

#[test]
fn test_eval_if_non_bool_cond() {
    // if 42 then ... else ...
    let term = Term::If {
        cond: Atom::Int(42),
        then_branch: Box::new(Term::Trap {
            reason: TrapReason::Custom("then".to_string()),
        }),
        else_branch: Box::new(Term::Trap {
            reason: TrapReason::Custom("else".to_string()),
        }),
        row: EffectRow::default(),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::InvalidPrimArgs(_, _))));
}

#[test]
fn test_eval_record_discharge_pass_through() {
    // RecordDischarge should pass through to body
    let term = Term::RecordDischarge {
        discharge: ContractDischarge {
            contract: "test".to_string(),
            discharge_type: DischargeType::Static,
        },
        body: Box::new(Term::Trap {
            reason: TrapReason::Custom("discharged".to_string()),
        }),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::Trap(TrapReason::Custom(ref s))) if s == "discharged"));
}

#[test]
fn test_eval_letrec_basic() {
    // letrec fact = (lam [n] k ... ) in (call fact [5] exit)
    // For now, just test that LetRec binds the name
    let fact_lam = Value::Lam {
        params: vec!["n".to_string()],
        cont: "k".to_string(),
        body: Box::new(Term::Jump {
            cont: ContRef::Var("k".to_string()),
            arg: Atom::Var("n".to_string()),
            row: EffectRow::default(),
        }),
        captured_env: Env::new(),
        rec_binding: None,
        row: EffectRow::default(),
    };
    let term = Term::LetCont {
        name: "exit".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Trap {
            reason: TrapReason::Custom("return".to_string()),
        }),
        body: Box::new(Term::LetRec {
            name: "fact".to_string(),
            value: fact_lam,
            body: Box::new(Term::Call {
                func: Atom::Var("fact".to_string()),
                args: vec![Atom::Int(5)],
                cont: ContRef::Label("exit".to_string()),
                row: EffectRow::default(),
            }),
        }),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::Trap(TrapReason::Custom(ref s))) if s == "return"));
}

#[test]
fn test_eval_trap() {
    let term = Term::Trap {
        reason: TrapReason::Custom("unreachable".to_string()),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::Trap(TrapReason::Custom(_)))));
}

#[test]
fn test_eval_trap_custom() {
    let term = Term::Trap {
        reason: TrapReason::Custom("test".to_string()),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::Trap(TrapReason::Custom(ref s))) if s == "test"));
}
