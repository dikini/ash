//! Tests for Phase 160 correctness fixes

use ash_core::cps::*;
use ash_interp::cps::eval_checked;

// ---------------------------------------------------------------------------
// Fix 1: Nested LetRec lambdas in composite values get rec_binding
// ---------------------------------------------------------------------------

#[test]
fn test_letrec_tuple_lambdas_get_rec_binding() {
    // Even lambda that calls odd via tuple_get
    let even_lam = Value::Lam {
        params: vec!["n".to_string()],
        cont: "k".to_string(),
        body: Box::new(Term::LetPrim {
            name: "is_zero".to_string(),
            op: PrimOp::Eq,
            args: vec![Atom::Var("n".to_string()), Atom::Int(0)],
            body: Box::new(Term::If {
                cond: Atom::Var("is_zero".to_string()),
                then_branch: Box::new(Term::Jump {
                    cont: ContRef::Var("k".to_string()),
                    arg: Atom::Bool(true),
                    row: EffectRow::default(),
                }),
                else_branch: Box::new(Term::LetPrim {
                    name: "n_minus_1".to_string(),
                    op: PrimOp::Sub,
                    args: vec![Atom::Var("n".to_string()), Atom::Int(1)],
                    body: Box::new(Term::LetPrim {
                        name: "odd_fn".to_string(),
                        op: PrimOp::TupleGet(1),
                        args: vec![Atom::Var("pair".to_string())],
                        body: Box::new(Term::Call {
                            func: Atom::Var("odd_fn".to_string()),
                            args: vec![Atom::Var("n_minus_1".to_string())],
                            cont: ContRef::Var("k".to_string()),
                            row: EffectRow::default(),
                        }),
                    }),
                }),
                row: EffectRow::default(),
            }),
        }),
        captured_env: Env::new(),
        rec_binding: None, // Will be set by LetRec
        row: EffectRow::default(),
    };

    // Odd lambda that calls even via tuple_get
    let odd_lam = Value::Lam {
        params: vec!["n".to_string()],
        cont: "k".to_string(),
        body: Box::new(Term::LetPrim {
            name: "is_zero".to_string(),
            op: PrimOp::Eq,
            args: vec![Atom::Var("n".to_string()), Atom::Int(0)],
            body: Box::new(Term::If {
                cond: Atom::Var("is_zero".to_string()),
                then_branch: Box::new(Term::Jump {
                    cont: ContRef::Var("k".to_string()),
                    arg: Atom::Bool(false),
                    row: EffectRow::default(),
                }),
                else_branch: Box::new(Term::LetPrim {
                    name: "n_minus_1".to_string(),
                    op: PrimOp::Sub,
                    args: vec![Atom::Var("n".to_string()), Atom::Int(1)],
                    body: Box::new(Term::LetPrim {
                        name: "even_fn".to_string(),
                        op: PrimOp::TupleGet(0),
                        args: vec![Atom::Var("pair".to_string())],
                        body: Box::new(Term::Call {
                            func: Atom::Var("even_fn".to_string()),
                            args: vec![Atom::Var("n_minus_1".to_string())],
                            cont: ContRef::Var("k".to_string()),
                            row: EffectRow::default(),
                        }),
                    }),
                }),
                row: EffectRow::default(),
            }),
        }),
        captured_env: Env::new(),
        rec_binding: None, // Will be set by LetRec
        row: EffectRow::default(),
    };

    let pair = Value::Tuple {
        elems: vec![even_lam, odd_lam],
    };

    let term = Term::LetCont {
        name: "k".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Return {
            value: Atom::Var("v".to_string()),
        }),
        body: Box::new(Term::LetRec {
            name: "pair".to_string(),
            value: pair,
            body: Box::new(Term::LetPrim {
                name: "even_fn".to_string(),
                op: PrimOp::TupleGet(0),
                args: vec![Atom::Var("pair".to_string())],
                body: Box::new(Term::Call {
                    func: Atom::Var("even_fn".to_string()),
                    args: vec![Atom::Int(4)],
                    cont: ContRef::Label("k".to_string()),
                    row: EffectRow::default(),
                }),
            }),
        }),
    };

    let result = eval_checked(&term, &Env::new(), &HandlerChain::new()).unwrap();
    assert_eq!(result, Atom::Bool(true));
}

// ---------------------------------------------------------------------------
// Fix 2: Call arity mismatch is rejected at runtime
// ---------------------------------------------------------------------------

#[test]
fn test_call_arity_mismatch_rejected() {
    let lam = Value::Lam {
        params: vec!["x".to_string()],
        cont: "k".to_string(),
        body: Box::new(Term::Return {
            value: Atom::Var("x".to_string()),
        }),
        captured_env: Env::new(),
        rec_binding: None,
        row: EffectRow::default(),
    };

    let term = Term::LetCont {
        name: "k".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Return {
            value: Atom::Var("v".to_string()),
        }),
        body: Box::new(Term::LetVal {
            name: "f".to_string(),
            value: lam,
            body: Box::new(Term::Call {
                func: Atom::Var("f".to_string()),
                args: vec![Atom::Int(1), Atom::Int(2)], // 2 args, but lambda expects 1
                cont: ContRef::Label("k".to_string()),
                row: EffectRow::default(),
            }),
        }),
    };

    let result = eval_checked(&term, &Env::new(), &HandlerChain::new());
    assert!(
        result.is_err(),
        "Expected arity mismatch error, got: {:?}",
        result
    );
}

// ---------------------------------------------------------------------------
// Fix 3: Handler dispatch matches full EffectOp (not just item)
// ---------------------------------------------------------------------------

#[test]
fn test_handler_matches_full_effect_op() {
    let mut chain = HandlerChain::new();

    // Handler for db.read with String arg, Int result
    let clause = HandlerClause {
        op: EffectOp {
            item: EffectItem {
                namespace: "cap".to_string(),
                name: "db.read".to_string(),
                kind: EffectItemKind::Capability,
            },
            arg_types: vec!["String".to_string()],
            result_type: "Int".to_string(),
        },
        params: vec!["table".to_string()],
        resume: "resume".to_string(),
        body: Box::new(Term::Jump {
            cont: ContRef::Var("resume".to_string()),
            arg: Atom::Int(42),
            row: EffectRow::default(),
        }),
        row: EffectRow::default(),
    };

    chain.push(HandlerFrame::Shallow {
        clause: clause.clone(),
    });

    // Same item but different signature should NOT match
    let different_op = EffectOp {
        item: EffectItem {
            namespace: "cap".to_string(),
            name: "db.read".to_string(),
            kind: EffectItemKind::Capability,
        },
        arg_types: vec![],           // Different!
        result_type: "".to_string(), // Different!
    };

    let found = chain.find_handler(&different_op);
    assert!(
        found.is_none(),
        "Should not match different EffectOp signature"
    );

    // Exact match should work
    let exact_op = EffectOp {
        item: EffectItem {
            namespace: "cap".to_string(),
            name: "db.read".to_string(),
            kind: EffectItemKind::Capability,
        },
        arg_types: vec!["String".to_string()],
        result_type: "Int".to_string(),
    };

    let found = chain.find_handler(&exact_op);
    assert!(found.is_some(), "Should match exact EffectOp");
}

// ---------------------------------------------------------------------------
// Fix 4: Handler parameter arity validated
// ---------------------------------------------------------------------------

#[test]
fn test_handler_clause_arity_validation() {
    // Handler with 2 params but op expects 1 arg type
    let term = Term::Handle {
        clause: HandlerClause {
            op: EffectOp {
                item: EffectItem {
                    namespace: "cap".to_string(),
                    name: "db.read".to_string(),
                    kind: EffectItemKind::Capability,
                },
                arg_types: vec!["String".to_string()], // 1 arg
                result_type: "Int".to_string(),
            },
            params: vec!["table".to_string(), "extra".to_string()], // 2 params!
            resume: "resume".to_string(),
            body: Box::new(Term::Jump {
                cont: ContRef::Var("resume".to_string()),
                arg: Atom::Int(42),
                row: EffectRow::default(),
            }),
            row: EffectRow::default(),
        },
        body: Box::new(Term::Return {
            value: Atom::Int(0),
        }),
        cont: ContRef::Label("exit".to_string()),
        row: EffectRow::default(),
    };

    let result = eval_checked(&term, &Env::new(), &HandlerChain::new());
    assert!(
        result.is_err(),
        "Expected handler arity validation error, got: {:?}",
        result
    );
}
