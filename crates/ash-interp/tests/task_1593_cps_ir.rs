//! TASK-1593: Raise and Handle dispatch tests
//!
//! Tests for effect raising and handler dispatch.

use ash_core::cps::*;
use ash_interp::cps::{CpsError, eval_term};

#[allow(dead_code)]
fn make_exit_cont() -> Term {
    Term::LetCont {
        name: "exit".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Trap {
            reason: TrapReason::Custom("return".to_string()),
        }),
        body: Box::new(Term::Trap {
            reason: TrapReason::Custom("unreachable".to_string()),
        }),
    }
}

#[test]
fn test_eval_raise_unhandled() {
    // raise db.read "users" resume (no handler)
    let op = EffectOp {
        item: EffectItem {
            namespace: "cap".to_string(),
            name: "db.read".to_string(),
            kind: EffectItemKind::Capability,
        },
        arg_types: vec!["String".to_string()],
        result_type: "Int".to_string(),
    };
    let term = Term::Raise {
        op: op.clone(),
        args: vec![Atom::String("users".to_string())],
        resume: ContRef::Label("resume".to_string()),
        row: EffectRow::default(),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    println!("RESULT: {:?}", result);
    assert!(matches!(result, Err(CpsError::UnhandledEffect(_))));
}

#[test]
fn test_eval_handle_catches_raise() {
    // handle db.read with clause in body
    let op = EffectOp {
        item: EffectItem {
            namespace: "cap".to_string(),
            name: "db.read".to_string(),
            kind: EffectItemKind::Capability,
        },
        arg_types: vec!["String".to_string()],
        result_type: "Int".to_string(),
    };
    let clause = HandlerClause {
        op: op.clone(),
        params: vec!["table".to_string()],
        resume: "resume".to_string(),
        body: Box::new(Term::Jump {
            cont: ContRef::Var("resume".to_string()),
            arg: Atom::String("handled".to_string()),
            row: EffectRow::default(),
        }),
        row: EffectRow::default(),
    };
    let term = Term::LetCont {
        name: "exit".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Trap {
            reason: TrapReason::Custom("return".to_string()),
        }),
        body: Box::new(Term::Handle {
            clause: clause.clone(),
            body: Box::new(Term::Raise {
                op: op.clone(),
                args: vec![Atom::String("users".to_string())],
                resume: ContRef::Label("exit".to_string()),
                row: EffectRow::default(),
            }),
            cont: ContRef::Label("exit".to_string()),
            row: EffectRow::default(),
        }),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::Trap(TrapReason::Custom(ref s))) if s == "return"));
}

#[test]
fn test_eval_handle_body_no_raise() {
    // handle with body that doesn't raise
    let op = EffectOp {
        item: EffectItem {
            namespace: "cap".to_string(),
            name: "db.read".to_string(),
            kind: EffectItemKind::Capability,
        },
        arg_types: vec!["String".to_string()],
        result_type: "Int".to_string(),
    };
    let clause = HandlerClause {
        op: op.clone(),
        params: vec!["table".to_string()],
        resume: "resume".to_string(),
        body: Box::new(Term::Jump {
            cont: ContRef::Var("resume".to_string()),
            arg: Atom::String("handled".to_string()),
            row: EffectRow::default(),
        }),
        row: EffectRow::default(),
    };
    let term = Term::LetCont {
        name: "exit".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Trap {
            reason: TrapReason::Custom("return".to_string()),
        }),
        body: Box::new(Term::Handle {
            clause: clause.clone(),
            body: Box::new(Term::Jump {
                cont: ContRef::Label("exit".to_string()),
                arg: Atom::Int(42),
                row: EffectRow::default(),
            }),
            cont: ContRef::Label("exit".to_string()),
            row: EffectRow::default(),
        }),
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert!(matches!(result, Err(CpsError::Trap(TrapReason::Custom(ref s))) if s == "return"));
}

#[test]
fn test_eval_handler_chain_find() {
    let mut chain = HandlerChain::new();
    let op = EffectOp {
        item: EffectItem {
            namespace: "cap".to_string(),
            name: "db.read".to_string(),
            kind: EffectItemKind::Capability,
        },
        arg_types: vec!["String".to_string()],
        result_type: "Int".to_string(),
    };
    let clause = HandlerClause {
        op: op.clone(),
        params: vec!["table".to_string()],
        resume: "resume".to_string(),
        body: Box::new(Term::Jump {
            cont: ContRef::Var("resume".to_string()),
            arg: Atom::String("handled".to_string()),
            row: EffectRow::default(),
        }),
        row: EffectRow::default(),
    };
    chain.push(HandlerFrame::Shallow {
        clause: clause.clone(),
    });
    let found = chain.find_handler(&op);
    assert!(found.is_some());
    assert_eq!(found.unwrap().0.params, vec!["table"]);
}

#[test]
fn test_eval_handler_chain_no_match() {
    let chain = HandlerChain::new();
    let op = EffectOp {
        item: EffectItem {
            namespace: "cap".to_string(),
            name: "db.read".to_string(),
            kind: EffectItemKind::Capability,
        },
        arg_types: vec!["String".to_string()],
        result_type: "Int".to_string(),
    };
    let found = chain.find_handler(&op);
    assert!(found.is_none());
}
