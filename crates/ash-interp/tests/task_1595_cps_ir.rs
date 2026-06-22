//! TASK-1595: Resume continuations tests
//!
//! Tests that resume continuations restore the original handler chain.

use ash_core::cps::*;
use ash_interp::cps::{CpsError, eval_term};

#[test]
fn test_resume_restores_handler_chain() {
    // Handler that resumes with a modified value
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
            arg: Atom::String("resumed".to_string()),
            row: EffectRow::default(),
        }),
        row: EffectRow::default(),
        resume_row: ResumeRowMetadata::InheritFromTarget,
        resume_multiplicity: ContMultiplicity::Affine,
    };
    let term = Term::LetCont {
        name: "exit".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Return {
            value: Atom::Var("v".to_string()),
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
        row: EffectRow::default(),
        multiplicity: ContMultiplicity::Affine,
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert_eq!(result, Ok(Atom::String("resumed".to_string())));
}

#[test]
fn test_resume_with_value() {
    // Handler that transforms the result before resuming
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
            arg: Atom::String("transformed".to_string()),
            row: EffectRow::default(),
        }),
        row: EffectRow::default(),
        resume_row: ResumeRowMetadata::InheritFromTarget,
        resume_multiplicity: ContMultiplicity::Affine,
    };
    let term = Term::LetCont {
        name: "exit".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Return {
            value: Atom::Var("v".to_string()),
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
        row: EffectRow::default(),
        multiplicity: ContMultiplicity::Affine,
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert_eq!(result, Ok(Atom::String("transformed".to_string())));
}

#[test]
fn test_shallow_handler_removed_on_resume() {
    // After a shallow handler handles an effect and resumes,
    // the handler should be removed from the chain
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
        resume_row: ResumeRowMetadata::InheritFromTarget,
        resume_multiplicity: ContMultiplicity::Affine,
    };
    let term = Term::LetCont {
        name: "exit".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Return {
            value: Atom::Var("v".to_string()),
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
        row: EffectRow::default(),
        multiplicity: ContMultiplicity::Affine,
    };
    let result = eval_term(&term, &Env::new(), &HandlerChain::new());
    assert_eq!(result, Ok(Atom::String("handled".to_string())));
}

#[test]
fn test_resume_one_shot_enforcement() {
    // Verify that jumping to a continuation sets its consumed flag.
    // Build a continuation manually, jump to it once, then verify the flag.
    let cont = Value::Cont {
        param: "v".to_string(),
        body: Box::new(Term::Return {
            value: Atom::Var("v".to_string()),
        }),
        captured_env: Env::new(),
        captured_chain: HandlerChain::new(),
        consumed: ConsumedFlag::new(),
        row: EffectRow::default(),
        multiplicity: ContMultiplicity::Affine,
    };

    // First jump should succeed
    let term = Term::Jump {
        cont: ContRef::Var("k".to_string()),
        arg: Atom::Int(42),
        row: EffectRow::default(),
    };
    let mut env = Env::new();
    env = env.with_binding("k".to_string(), cont.clone());
    let result = eval_term(&term, &env, &HandlerChain::new());
    assert_eq!(result, Ok(Atom::Int(42)));

    // Verify the consumed flag is set
    match env.lookup("k") {
        Some(Value::Cont { consumed, .. }) => {
            assert!(
                consumed.get(),
                "Continuation should be marked consumed after jump"
            );
        }
        _ => panic!("Expected Cont value"),
    }

    // Second jump should trap with "resume already consumed"
    let result2 = eval_term(&term, &env, &HandlerChain::new());
    assert!(
        matches!(result2, Err(CpsError::Trap(TrapReason::Custom(ref s))) if s == "resume already consumed")
    );
}
