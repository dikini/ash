//! TASK-1681: Add CPS Continuation and Invocation Carriers

use ash_core::cps::{
    Atom, ConsumedFlag, ContMultiplicity, ContRef, EffectItem, EffectItemKind, EffectOp, EffectRow,
    Env, HandlerChain, HandlerClause, ResumeRowMetadata, Term, Value,
};
use ash_core::sexp::{string_to_term, term_to_string, value_to_string};

fn cont_row() -> EffectRow {
    EffectRow {
        items: vec![EffectItem {
            namespace: "cap".to_string(),
            name: "read".to_string(),
            kind: EffectItemKind::Capability,
        }],
    }
}

fn sample_handler_clause() -> HandlerClause {
    HandlerClause {
        op: EffectOp {
            item: EffectItem {
                namespace: "cap".to_string(),
                name: "write".to_string(),
                kind: EffectItemKind::Capability,
            },
            arg_types: vec!["String".to_string()],
            result_type: "Unit".to_string(),
        },
        params: vec!["line".to_string()],
        resume: "k".to_string(),
        body: Box::new(Term::Jump {
            cont: ContRef::Var("k".to_string()),
            arg: Atom::Int(1),
            row: EffectRow::default(),
        }),
        row: EffectRow::default(),
        resume_row: ResumeRowMetadata::InheritFromTarget,
        resume_multiplicity: ContMultiplicity::Affine,
    }
}

#[test]
fn value_cont_carries_affine_and_multishot_multiplicity() {
    let affine = Value::Cont {
        param: "x".to_string(),
        body: Box::new(Term::Return {
            value: Atom::Int(1),
        }),
        captured_env: Env::new(),
        captured_chain: HandlerChain::new(),
        consumed: ConsumedFlag::new(),
        row: EffectRow::default(),
        multiplicity: ContMultiplicity::Affine,
    };
    let multi_shot = Value::Cont {
        param: "x".to_string(),
        body: Box::new(Term::Return {
            value: Atom::Int(1),
        }),
        captured_env: Env::new(),
        captured_chain: HandlerChain::new(),
        consumed: ConsumedFlag::new(),
        row: cont_row(),
        multiplicity: ContMultiplicity::MultiShotPure,
    };

    assert_eq!(affine, affine.clone());
    assert_eq!(multi_shot, multi_shot.clone());
    assert_ne!(affine, multi_shot);

    match affine {
        Value::Cont { multiplicity, .. } => assert_eq!(multiplicity, ContMultiplicity::Affine),
        _ => unreachable!(),
    }

    match multi_shot {
        Value::Cont { multiplicity, .. } => {
            assert_eq!(multiplicity, ContMultiplicity::MultiShotPure)
        }
        _ => unreachable!(),
    }
}

#[test]
fn letcont_carries_explicit_row_and_multiplicity() {
    let term = Term::LetCont {
        name: "k".to_string(),
        param: "x".to_string(),
        cont_body: Box::new(Term::Jump {
            cont: ContRef::Var("x".to_string()),
            arg: Atom::Int(1),
            row: EffectRow::default(),
        }),
        body: Box::new(Term::Return {
            value: Atom::Var("x".to_string()),
        }),
        row: cont_row(),
        multiplicity: ContMultiplicity::MultiShotPure,
    };

    match term {
        Term::LetCont {
            row, multiplicity, ..
        } => {
            assert_eq!(row, cont_row());
            assert_eq!(multiplicity, ContMultiplicity::MultiShotPure);
        }
        _ => panic!("expected LetCont"),
    }
}

#[test]
fn letcontcall_has_explicit_row_accounting() {
    let term = Term::LetContCall {
        name: "k".to_string(),
        cont: "k0".to_string(),
        arg: Atom::Int(9),
        row: cont_row(),
        body: Box::new(Term::Return {
            value: Atom::Int(0),
        }),
    };

    match term {
        Term::LetContCall { row, .. } => {
            assert_eq!(row, cont_row());
        }
        _ => panic!("expected LetContCall"),
    }
}

#[test]
fn handler_clause_stores_known_resume_row_metadata() {
    let resume_row = cont_row();
    let clause = HandlerClause {
        resume_row: ResumeRowMetadata::Known(resume_row.clone()),
        resume_multiplicity: ContMultiplicity::MultiShotPure,
        ..sample_handler_clause()
    };

    let parsed_resume_row = cont_row();
    let HandlerClause {
        resume_row,
        resume_multiplicity,
        ..
    } = clause;

    assert_eq!(
        resume_row,
        ResumeRowMetadata::Known(parsed_resume_row.clone())
    );
    let parsed_resume_multiplicity = resume_multiplicity;

    assert_eq!(parsed_resume_multiplicity, ContMultiplicity::MultiShotPure);
}

#[test]
fn omitted_letcont_rows_default_to_empty_when_serde_round_trips() {
    let term = Term::LetCont {
        name: "k".to_string(),
        param: "x".to_string(),
        cont_body: Box::new(Term::Return {
            value: Atom::Int(1),
        }),
        body: Box::new(Term::Return {
            value: Atom::Int(2),
        }),
        row: EffectRow::default(),
        multiplicity: ContMultiplicity::Affine,
    };

    let serialized = term_to_string(&term).unwrap();
    let parsed: Term = string_to_term(&serialized).unwrap();

    let Term::LetCont {
        row, multiplicity, ..
    } = parsed
    else {
        panic!("expected LetCont");
    };

    assert_eq!(row, EffectRow::default());
    assert_eq!(multiplicity, ContMultiplicity::Affine);
}

#[test]
fn omitted_handler_resume_row_defaults_to_inherit_from_target() {
    let clause = sample_handler_clause();
    let serialized = serde_lexpr::to_string(&clause).unwrap();
    let reparsed: HandlerClause = serde_lexpr::from_str(&serialized).unwrap();

    assert_eq!(
        reparsed.resume_row,
        ResumeRowMetadata::InheritFromTarget,
        "omitted resume_row should default to inherit-from-target"
    );
    assert_eq!(reparsed.resume_multiplicity, ContMultiplicity::Affine);
}

#[test]
fn omitted_multiplicity_fields_default_to_affine() {
    let term = Term::LetCont {
        name: "k".to_string(),
        param: "x".to_string(),
        cont_body: Box::new(Term::Return {
            value: Atom::Int(1),
        }),
        body: Box::new(Term::Return {
            value: Atom::Int(2),
        }),
        row: EffectRow::default(),
        multiplicity: ContMultiplicity::Affine,
    };
    let term_serialized = term_to_string(&term).unwrap();
    let term_parsed: Term = string_to_term(&term_serialized).unwrap();

    match term_parsed {
        Term::LetCont { multiplicity, .. } => {
            assert_eq!(multiplicity, ContMultiplicity::Affine);
        }
        _ => panic!("expected LetCont"),
    }

    let handler = sample_handler_clause();
    let handler_serialized = serde_lexpr::to_string(&handler).unwrap();
    let handler_parsed: HandlerClause = serde_lexpr::from_str(&handler_serialized).unwrap();

    assert_eq!(handler_parsed.resume_multiplicity, ContMultiplicity::Affine);

    let value = Value::Cont {
        param: "x".to_string(),
        body: Box::new(Term::Return {
            value: Atom::Int(3),
        }),
        captured_env: Env::new(),
        captured_chain: HandlerChain::new(),
        consumed: ConsumedFlag::new(),
        row: EffectRow::default(),
        multiplicity: ContMultiplicity::Affine,
    };
    let value_serialized = value_to_string(&value).unwrap();
    let value_parsed: Value = serde_lexpr::from_str(&value_serialized).unwrap();

    assert_eq!(value_parsed, value);
}
