use ash_core::cps::*;
use ash_core::sexp::{string_to_term, string_to_value, term_to_string, value_to_string};
use ash_interp::cps::eval_checked;

// ---------------------------------------------------------------------------
// TASK-1610: Value::Record and Value::Tuple
// ---------------------------------------------------------------------------

#[test]
fn test_eval_record_construction_with_atoms() {
    // Record with literal atoms — resolves to Value::Atom fields
    // Verify by extracting a field and returning the atom
    let record = Value::Record {
        fields: vec![
            ("x".to_string(), Value::Atom(Atom::Int(1))),
            ("y".to_string(), Value::Atom(Atom::Int(2))),
        ],
    };

    let term = Term::LetVal {
        name: "r".to_string(),
        value: record,
        body: Box::new(Term::LetPrim {
            name: "x_val".to_string(),
            op: PrimOp::RecordGet("x".to_string()),
            args: vec![Atom::Var("r".to_string())],
            body: Box::new(Term::Return {
                value: Atom::Var("x_val".to_string()),
            }),
        }),
    };

    let result = eval_checked(&term, &Env::new(), &HandlerChain::new()).unwrap();
    assert_eq!(result, Atom::Int(1));
}

#[test]
fn test_eval_tuple_construction_with_atoms() {
    let tuple = Value::Tuple {
        elems: vec![Value::Atom(Atom::Int(10)), Value::Atom(Atom::Int(20))],
    };

    let term = Term::LetVal {
        name: "t".to_string(),
        value: tuple,
        body: Box::new(Term::LetPrim {
            name: "first".to_string(),
            op: PrimOp::TupleGet(0),
            args: vec![Atom::Var("t".to_string())],
            body: Box::new(Term::Return {
                value: Atom::Var("first".to_string()),
            }),
        }),
    };

    let result = eval_checked(&term, &Env::new(), &HandlerChain::new()).unwrap();
    assert_eq!(result, Atom::Int(10));
}

#[test]
fn test_eval_jump_with_structured_value() {
    // Jump with a record argument — continuation receives structured value
    // Then extract atom from it and return
    let record = Value::Record {
        fields: vec![("x".to_string(), Value::Atom(Atom::Int(42)))],
    };

    let term = Term::LetCont {
        name: "k".to_string(),
        param: "r".to_string(),
        cont_body: Box::new(Term::LetPrim {
            name: "x_val".to_string(),
            op: PrimOp::RecordGet("x".to_string()),
            args: vec![Atom::Var("r".to_string())],
            body: Box::new(Term::Return {
                value: Atom::Var("x_val".to_string()),
            }),
        }),
        body: Box::new(Term::LetVal {
            name: "r".to_string(),
            value: record,
            body: Box::new(Term::Jump {
                cont: ContRef::Label("k".to_string()),
                arg: Atom::Var("r".to_string()),
                row: EffectRow::default(),
            }),
        }),
    };

    let result = eval_checked(&term, &Env::new(), &HandlerChain::new()).unwrap();
    assert_eq!(result, Atom::Int(42));
}

// ---------------------------------------------------------------------------
// TASK-1611: PrimOp::RecordGet and PrimOp::TupleGet
// ---------------------------------------------------------------------------

#[test]
fn test_eval_record_get_field() {
    let record = Value::Record {
        fields: vec![
            ("x".to_string(), Value::Atom(Atom::Int(1))),
            ("y".to_string(), Value::Atom(Atom::Int(2))),
        ],
    };

    let term = Term::LetVal {
        name: "r".to_string(),
        value: record,
        body: Box::new(Term::LetPrim {
            name: "y_val".to_string(),
            op: PrimOp::RecordGet("y".to_string()),
            args: vec![Atom::Var("r".to_string())],
            body: Box::new(Term::Return {
                value: Atom::Var("y_val".to_string()),
            }),
        }),
    };

    let result = eval_checked(&term, &Env::new(), &HandlerChain::new()).unwrap();
    assert_eq!(result, Atom::Int(2));
}

#[test]
fn test_eval_tuple_get_index() {
    let tuple = Value::Tuple {
        elems: vec![
            Value::Atom(Atom::Int(10)),
            Value::Atom(Atom::Int(20)),
            Value::Atom(Atom::Int(30)),
        ],
    };

    let term = Term::LetVal {
        name: "t".to_string(),
        value: tuple,
        body: Box::new(Term::LetPrim {
            name: "second".to_string(),
            op: PrimOp::TupleGet(1),
            args: vec![Atom::Var("t".to_string())],
            body: Box::new(Term::Return {
                value: Atom::Var("second".to_string()),
            }),
        }),
    };

    let result = eval_checked(&term, &Env::new(), &HandlerChain::new()).unwrap();
    assert_eq!(result, Atom::Int(20));
}

#[test]
fn test_eval_record_get_missing_field() {
    let record = Value::Record {
        fields: vec![("x".to_string(), Value::Atom(Atom::Int(1)))],
    };

    let term = Term::LetVal {
        name: "r".to_string(),
        value: record,
        body: Box::new(Term::LetPrim {
            name: "z_val".to_string(),
            op: PrimOp::RecordGet("z".to_string()),
            args: vec![Atom::Var("r".to_string())],
            body: Box::new(Term::Return {
                value: Atom::Var("z_val".to_string()),
            }),
        }),
    };

    let result = eval_checked(&term, &Env::new(), &HandlerChain::new());
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// TASK-1612: Atom::ConstructorName
// ---------------------------------------------------------------------------

#[test]
fn test_constructor_name_atom() {
    let atom = Atom::ConstructorName("Some".to_string());
    assert_eq!(atom, Atom::ConstructorName("Some".to_string()));
}

// ---------------------------------------------------------------------------
// TASK-1613: Term::Match
// ---------------------------------------------------------------------------

#[test]
fn test_eval_match_2way() {
    // (True, 42) represented as tuple with constructor tag
    let tuple = Value::Tuple {
        elems: vec![
            Value::Atom(Atom::ConstructorName("True".to_string())),
            Value::Atom(Atom::Int(42)),
        ],
    };

    let term = Term::LetVal {
        name: "t".to_string(),
        value: tuple,
        body: Box::new(Term::Match {
            scrutinee: Atom::Var("t".to_string()),
            arms: vec![
                (
                    "True".to_string(),
                    Box::new(Term::Return {
                        value: Atom::Int(1),
                    }),
                ),
                (
                    "False".to_string(),
                    Box::new(Term::Return {
                        value: Atom::Int(0),
                    }),
                ),
            ],
            default: None,
        }),
    };

    let result = eval_checked(&term, &Env::new(), &HandlerChain::new()).unwrap();
    assert_eq!(result, Atom::Int(1));
}

#[test]
fn test_eval_match_default() {
    let tuple = Value::Tuple {
        elems: vec![
            Value::Atom(Atom::ConstructorName("Other".to_string())),
            Value::Atom(Atom::Int(99)),
        ],
    };

    let term = Term::LetVal {
        name: "t".to_string(),
        value: tuple,
        body: Box::new(Term::Match {
            scrutinee: Atom::Var("t".to_string()),
            arms: vec![(
                "True".to_string(),
                Box::new(Term::Return {
                    value: Atom::Int(1),
                }),
            )],
            default: Some(Box::new(Term::Return {
                value: Atom::Int(-1),
            })),
        }),
    };

    let result = eval_checked(&term, &Env::new(), &HandlerChain::new()).unwrap();
    assert_eq!(result, Atom::Int(-1));
}

// ---------------------------------------------------------------------------
// TASK-1614: Mutual recursion via tuple-of-lambdas
// ---------------------------------------------------------------------------

#[test]
fn test_eval_mutual_recursion_even_odd() {
    // even(n) = if n==0 then true else odd(n-1)
    // odd(n) = if n==0 then false else even(n-1)
    // Represented as tuple (even, odd) with mutual recursion

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
        rec_binding: Some("pair".to_string()),
        row: EffectRow::default(),
    };

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
        rec_binding: Some("pair".to_string()),
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

#[test]
fn test_eval_call_no_dynamic_scope() {
    // Verify that unrelated caller locals don't leak into closures
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
            body: Box::new(Term::LetVal {
                name: "caller_local".to_string(),
                value: Value::Atom(Atom::Int(999)),
                body: Box::new(Term::Call {
                    func: Atom::Var("f".to_string()),
                    args: vec![Atom::Int(42)],
                    cont: ContRef::Label("k".to_string()),
                    row: EffectRow::default(),
                }),
            }),
        }),
    };

    let result = eval_checked(&term, &Env::new(), &HandlerChain::new()).unwrap();
    assert_eq!(result, Atom::Int(42));
}
// ---------------------------------------------------------------------------
// TASK-1615: Serde roundtrip for new forms
// ---------------------------------------------------------------------------

#[test]
fn test_serde_record_roundtrip() {
    let record = Value::Record {
        fields: vec![
            ("x".to_string(), Value::Atom(Atom::Int(1))),
            ("y".to_string(), Value::Atom(Atom::Int(2))),
        ],
    };

    let serialized = value_to_string(&record).unwrap();
    let deserialized: Value = string_to_value(&serialized).unwrap();
    assert_eq!(record, deserialized);
}

#[test]
fn test_serde_tuple_roundtrip() {
    let tuple = Value::Tuple {
        elems: vec![Value::Atom(Atom::Int(10)), Value::Atom(Atom::Int(20))],
    };

    let serialized = value_to_string(&tuple).unwrap();
    let deserialized: Value = string_to_value(&serialized).unwrap();
    assert_eq!(tuple, deserialized);
}

#[test]
fn test_serde_constructor_name_roundtrip() {
    let atom = Atom::ConstructorName("Some".to_string());
    let serialized = value_to_string(&Value::Atom(atom.clone())).unwrap();
    let deserialized: Value = string_to_value(&serialized).unwrap();
    assert_eq!(Value::Atom(atom), deserialized);
}

#[test]
fn test_serde_lam_with_rec_binding_roundtrip() {
    let lam = Value::Lam {
        params: vec!["x".to_string()],
        cont: "k".to_string(),
        body: Box::new(Term::Return {
            value: Atom::Var("x".to_string()),
        }),
        captured_env: Env::new(),
        rec_binding: Some("pair".to_string()),
        row: EffectRow::default(),
    };

    let serialized = value_to_string(&lam).unwrap();
    let deserialized: Value = string_to_value(&serialized).unwrap();
    assert_eq!(lam, deserialized);
}

#[test]
fn test_serde_match_term_roundtrip() {
    let term = Term::Match {
        scrutinee: Atom::Var("t".to_string()),
        arms: vec![(
            "True".to_string(),
            Box::new(Term::Return {
                value: Atom::Int(1),
            }),
        )],
        default: Some(Box::new(Term::Return {
            value: Atom::Int(0),
        })),
    };

    let serialized = term_to_string(&term).unwrap();
    let deserialized: Term = string_to_term(&serialized).unwrap();
    assert_eq!(term, deserialized);
}

#[test]
fn test_serde_backward_compatibility_no_rec_binding() {
    // A lambda serialized without rec_binding should deserialize with rec_binding: None
    // This tests the #[serde(default)] attribute
    // Note: The sexp format uses (Lam ...) wrapper, so we serialize a real lambda first
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
    let sexp = value_to_string(&lam).unwrap();
    let result: Result<Value, _> = string_to_value(&sexp);
    assert!(result.is_ok());
    match result.unwrap() {
        Value::Lam {
            rec_binding: None, ..
        } => {}
        other => panic!("Expected rec_binding: None, got {:?}", other),
    }
}
