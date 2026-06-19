//! TASK-1590: CPS IR core data structures tests
//!
//! Tests for the CPS IR data model defined in ash-core::cps.

use ash_core::cps::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[test]
fn test_atom_var() {
    let a = Atom::Var("x".to_string());
    assert_eq!(format!("{:?}", a), "Var(\"x\")");
}

#[test]
fn test_atom_int() {
    let a = Atom::Int(42);
    assert_eq!(format!("{:?}", a), "Int(42)");
}

#[test]
fn test_value_atom() {
    let v = Value::Atom(Atom::Int(42));
    assert_eq!(format!("{:?}", v), "Atom(Int(42))");
}

#[test]
fn test_lam_value() {
    let body = Box::new(Term::Jump {
        cont: ContRef::Label("k".to_string()),
        arg: Atom::Var("x".to_string()),
        row: EffectRow::default(),
    });
    let lam = Value::Lam {
        params: vec!["x".to_string()],
        cont: "k".to_string(),
        body,
        captured_env: Env::new(),
        row: EffectRow::default(),
    };
    match lam {
        Value::Lam { params, cont, .. } => {
            assert_eq!(params, vec!["x"]);
            assert_eq!(cont, "k");
        }
        _ => panic!("Expected Lam"),
    }
}

#[test]
fn test_cont_value() {
    let body = Box::new(Term::Jump {
        cont: ContRef::Label("exit".to_string()),
        arg: Atom::Var("v".to_string()),
        row: EffectRow::default(),
    });
    let cont = Value::Cont {
        param: "v".to_string(),
        body,
        captured_env: Env::new(),
        captured_chain: HandlerChain::new(),
        consumed: ConsumedFlag::new(),
        row: EffectRow::default(),
    };
    match cont {
        Value::Cont { param, .. } => {
            assert_eq!(param, "v");
        }
        _ => panic!("Expected Cont"),
    }
}

#[test]
fn test_cont_ref_label() {
    let cr = ContRef::Label("k".to_string());
    assert_eq!(format!("{:?}", cr), "Label(\"k\")");
}

#[test]
fn test_cont_ref_var() {
    let cr = ContRef::Var("k".to_string());
    assert_eq!(format!("{:?}", cr), "Var(\"k\")");
}

#[test]
fn test_let_val_term() {
    let term = Term::LetVal {
        name: "x".to_string(),
        value: Value::Atom(Atom::Int(42)),
        body: Box::new(Term::Jump {
            cont: ContRef::Label("k".to_string()),
            arg: Atom::Var("x".to_string()),
            row: EffectRow::default(),
        }),
    };
    match term {
        Term::LetVal { name, .. } => assert_eq!(name, "x"),
        _ => panic!("Expected LetVal"),
    }
}

#[test]
fn test_let_prim_term() {
    let term = Term::LetPrim {
        name: "y".to_string(),
        op: PrimOp::Add,
        args: vec![Atom::Int(1), Atom::Int(2)],
        body: Box::new(Term::Jump {
            cont: ContRef::Label("k".to_string()),
            arg: Atom::Var("y".to_string()),
            row: EffectRow::default(),
        }),
    };
    match term {
        Term::LetPrim { name, op, .. } => {
            assert_eq!(name, "y");
            assert_eq!(op, PrimOp::Add);
        }
        _ => panic!("Expected LetPrim"),
    }
}

#[test]
fn test_let_cont_term() {
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
    match term {
        Term::LetCont { name, param, .. } => {
            assert_eq!(name, "k");
            assert_eq!(param, "v");
        }
        _ => panic!("Expected LetCont"),
    }
}

#[test]
fn test_jump_term() {
    let term = Term::Jump {
        cont: ContRef::Label("k".to_string()),
        arg: Atom::Int(42),
        row: EffectRow::default(),
    };
    match term {
        Term::Jump { cont, arg, .. } => {
            assert_eq!(cont, ContRef::Label("k".to_string()));
            assert_eq!(arg, Atom::Int(42));
        }
        _ => panic!("Expected Jump"),
    }
}

#[test]
fn test_call_term() {
    let term = Term::Call {
        func: Atom::Var("f".to_string()),
        args: vec![Atom::Int(1)],
        cont: ContRef::Label("k".to_string()),
        row: EffectRow::default(),
    };
    match term {
        Term::Call { func, args, .. } => {
            assert_eq!(func, Atom::Var("f".to_string()));
            assert_eq!(args, vec![Atom::Int(1)]);
        }
        _ => panic!("Expected Call"),
    }
}

#[test]
fn test_prim_op_display() {
    assert_eq!(format!("{:?}", PrimOp::Add), "Add");
    assert_eq!(format!("{:?}", PrimOp::Sub), "Sub");
    assert_eq!(format!("{:?}", PrimOp::Mul), "Mul");
    assert_eq!(format!("{:?}", PrimOp::Eq), "Eq");
}

#[test]
fn test_effect_row_with_items() {
    let row = EffectRow {
        items: vec![EffectItem {
            namespace: "cap".to_string(),
            name: "fs.read".to_string(),
            kind: EffectItemKind::Capability,
        }],
    };
    assert_eq!(row.items.len(), 1);
    assert_eq!(row.items[0].namespace, "cap");
}

#[test]
fn test_handler_chain_push() {
    let mut chain = HandlerChain::new();
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
            arg: Atom::String("result".to_string()),
            row: EffectRow::default(),
        }),
        row: EffectRow::default(),
    };
    chain.push(HandlerFrame::Shallow {
        clause: clause.clone(),
    });
    assert_eq!(chain.frames.len(), 1);

    let found = chain.find_handler(&EffectOp {
        item: EffectItem {
            namespace: "cap".to_string(),
            name: "db.read".to_string(),
            kind: EffectItemKind::Capability,
        },
        arg_types: vec![],
        result_type: "".to_string(),
    });
    assert!(found.is_some());
}

#[test]
fn test_handler_chain_find_no_match() {
    let chain = HandlerChain::new();
    let found = chain.find_handler(&EffectOp {
        item: EffectItem {
            namespace: "cap".to_string(),
            name: "db.read".to_string(),
            kind: EffectItemKind::Capability,
        },
        arg_types: vec![],
        result_type: "".to_string(),
    });
    assert!(found.is_none());
}

#[test]
fn test_env_lookup_nested() {
    let parent = Box::new(Env::new().with_binding("x".to_string(), Value::Atom(Atom::Int(1))));
    let child = Env {
        bindings: HashMap::new(),
        parent: Some(parent),
    };
    assert_eq!(child.lookup("x"), Some(&Value::Atom(Atom::Int(1))));
    assert_eq!(child.lookup("y"), None);
}

#[test]
fn test_trap_reason() {
    let tr = TrapReason::Custom("unreachable".to_string());
    assert_eq!(format!("{:?}", tr), "Custom(\"unreachable\")");
}

#[test]
fn test_contract_discharge() {
    let cd = ContractDischarge {
        contract: "test".to_string(),
        discharge_type: DischargeType::Dynamic,
    };
    assert_eq!(cd.contract, "test");
    assert_eq!(cd.discharge_type, DischargeType::Dynamic);
}
