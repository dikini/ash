//! TASK-1591: Core CPS evaluator tests
//!
//! Tests for evaluating LetVal, LetPrim, LetCont, Jump, and Call.

use ash_core::cps::*;
use ash_interp::cps::{CpsError, eval_term};

// Helper: build a simple program that binds 42 to x and jumps to exit with x
fn program_let_val_jump() -> Term {
    Term::LetCont {
        name: "exit".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Trap {
            reason: TrapReason::Custom("return".to_string()),
        }),
        body: Box::new(Term::LetVal {
            name: "x".to_string(),
            value: Value::Atom(Atom::Int(42)),
            body: Box::new(Term::Jump {
                cont: ContRef::Label("exit".to_string()),
                arg: Atom::Var("x".to_string()),
                row: EffectRow::default(),
            }),
        }),
    }
}

// Helper: build a program that adds 1+2 via LetPrim and jumps with result
fn program_let_prim_add() -> Term {
    Term::LetCont {
        name: "exit".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Trap {
            reason: TrapReason::Custom("return".to_string()),
        }),
        body: Box::new(Term::LetPrim {
            name: "y".to_string(),
            op: PrimOp::Add,
            args: vec![Atom::Int(1), Atom::Int(2)],
            body: Box::new(Term::Jump {
                cont: ContRef::Label("exit".to_string()),
                arg: Atom::Var("y".to_string()),
                row: EffectRow::default(),
            }),
        }),
    }
}

#[test]
fn test_eval_let_val_jump() {
    let term = program_let_val_jump();
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::Trap(TrapReason::Custom(ref s))) if s == "return"));
}

#[test]
fn test_eval_let_prim_add() {
    let term = program_let_prim_add();
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::Trap(TrapReason::Custom(ref s))) if s == "return"));
}

#[test]
fn test_eval_let_prim_sub() {
    let term = Term::LetCont {
        name: "exit".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Trap {
            reason: TrapReason::Custom("return".to_string()),
        }),
        body: Box::new(Term::LetPrim {
            name: "y".to_string(),
            op: PrimOp::Sub,
            args: vec![Atom::Int(5), Atom::Int(3)],
            body: Box::new(Term::Jump {
                cont: ContRef::Label("exit".to_string()),
                arg: Atom::Var("y".to_string()),
                row: EffectRow::default(),
            }),
        }),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::Trap(TrapReason::Custom(ref s))) if s == "return"));
}

#[test]
fn test_eval_let_prim_mul() {
    let term = Term::LetCont {
        name: "exit".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Trap {
            reason: TrapReason::Custom("return".to_string()),
        }),
        body: Box::new(Term::LetPrim {
            name: "y".to_string(),
            op: PrimOp::Mul,
            args: vec![Atom::Int(4), Atom::Int(5)],
            body: Box::new(Term::Jump {
                cont: ContRef::Label("exit".to_string()),
                arg: Atom::Var("y".to_string()),
                row: EffectRow::default(),
            }),
        }),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::Trap(TrapReason::Custom(ref s))) if s == "return"));
}

#[test]
fn test_eval_let_prim_eq_true() {
    let term = Term::LetCont {
        name: "exit".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Trap {
            reason: TrapReason::Custom("return".to_string()),
        }),
        body: Box::new(Term::LetPrim {
            name: "y".to_string(),
            op: PrimOp::Eq,
            args: vec![Atom::Int(2), Atom::Int(2)],
            body: Box::new(Term::Jump {
                cont: ContRef::Label("exit".to_string()),
                arg: Atom::Var("y".to_string()),
                row: EffectRow::default(),
            }),
        }),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::Trap(TrapReason::Custom(ref s))) if s == "return"));
}

#[test]
fn test_eval_let_prim_eq_false() {
    let term = Term::LetCont {
        name: "exit".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Trap {
            reason: TrapReason::Custom("return".to_string()),
        }),
        body: Box::new(Term::LetPrim {
            name: "y".to_string(),
            op: PrimOp::Eq,
            args: vec![Atom::Int(1), Atom::Int(2)],
            body: Box::new(Term::Jump {
                cont: ContRef::Label("exit".to_string()),
                arg: Atom::Var("y".to_string()),
                row: EffectRow::default(),
            }),
        }),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::Trap(TrapReason::Custom(ref s))) if s == "return"));
}

#[test]
fn test_eval_let_cont_jump() {
    // letcont k [v] (jump exit v) in (jump k 42)
    let term = Term::LetCont {
        name: "k".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Jump {
            cont: ContRef::Label("exit".to_string()),
            arg: Atom::Var("v".to_string()),
            row: EffectRow::default(),
        }),
        body: Box::new(Term::Jump {
            cont: ContRef::Label("k".to_string()),
            arg: Atom::Int(42),
            row: EffectRow::default(),
        }),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::UnboundLabel(ref s)) if s == "exit"));
}

#[test]
fn test_eval_call_identity() {
    // letval id = (lam [x] k (jump k x)) in (call id [42] exit)
    let id_lam = Value::Lam {
        params: vec!["x".to_string()],
        cont: "k".to_string(),
        body: Box::new(Term::Jump {
            cont: ContRef::Var("k".to_string()),
            arg: Atom::Var("x".to_string()),
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
        body: Box::new(Term::LetVal {
            name: "id".to_string(),
            value: id_lam,
            body: Box::new(Term::Call {
                func: Atom::Var("id".to_string()),
                args: vec![Atom::Int(42)],
                cont: ContRef::Label("exit".to_string()),
                row: EffectRow::default(),
            }),
        }),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::Trap(TrapReason::Custom(ref s))) if s == "return"));
}

#[test]
fn test_eval_unbound_variable() {
    let term = Term::Jump {
        cont: ContRef::Label("exit".to_string()),
        arg: Atom::Var("unbound".to_string()),
        row: EffectRow::default(),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::UnboundVariable(ref s)) if s == "unbound"));
}

#[test]
fn test_eval_unbound_function() {
    let term = Term::Call {
        func: Atom::Var("unbound".to_string()),
        args: vec![],
        cont: ContRef::Label("exit".to_string()),
        row: EffectRow::default(),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::UnboundVariable(ref s)) if s == "unbound"));
}

#[test]
fn test_eval_call_non_lambda() {
    // letval x = 42 in (call x [] exit)
    let term = Term::LetCont {
        name: "exit".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Trap {
            reason: TrapReason::Custom("return".to_string()),
        }),
        body: Box::new(Term::LetVal {
            name: "x".to_string(),
            value: Value::Atom(Atom::Int(42)),
            body: Box::new(Term::Call {
                func: Atom::Var("x".to_string()),
                args: vec![],
                cont: ContRef::Label("exit".to_string()),
                row: EffectRow::default(),
            }),
        }),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::ExpectedLambda(_))));
}

#[test]
fn test_eval_jump_to_unbound_label() {
    // letcont k [v] (jump exit v) in (jump unbound 42)
    let term = Term::LetCont {
        name: "k".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Jump {
            cont: ContRef::Label("exit".to_string()),
            arg: Atom::Var("v".to_string()),
            row: EffectRow::default(),
        }),
        body: Box::new(Term::Jump {
            cont: ContRef::Label("unbound".to_string()),
            arg: Atom::Int(42),
            row: EffectRow::default(),
        }),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::UnboundLabel(ref s)) if s == "unbound"));
}

#[test]
fn test_eval_nested_let_val() {
    // letcont exit [v] (trap return) in
    //   letval x = 1 in
    //     letval y = 2 in
    //       letprim z = add x y in
    //         jump exit z
    let term = Term::LetCont {
        name: "exit".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Trap {
            reason: TrapReason::Custom("return".to_string()),
        }),
        body: Box::new(Term::LetVal {
            name: "x".to_string(),
            value: Value::Atom(Atom::Int(1)),
            body: Box::new(Term::LetVal {
                name: "y".to_string(),
                value: Value::Atom(Atom::Int(2)),
                body: Box::new(Term::LetPrim {
                    name: "z".to_string(),
                    op: PrimOp::Add,
                    args: vec![Atom::Var("x".to_string()), Atom::Var("y".to_string())],
                    body: Box::new(Term::Jump {
                        cont: ContRef::Label("exit".to_string()),
                        arg: Atom::Var("z".to_string()),
                        row: EffectRow::default(),
                    }),
                }),
            }),
        }),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::Trap(TrapReason::Custom(ref s))) if s == "return"));
}
