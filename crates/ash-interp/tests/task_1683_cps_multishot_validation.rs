//! TASK-1683: Validate CPS Multi-Shot Row Legality
//!
//! Tests that CPS validation rejects malformed multi-shot-pure continuations,
//! LetCont binders, handler resume metadata, and LetContCall row under-reporting.

#![allow(clippy::result_large_err)]

use ash_core::cps::{
    Atom, ConsumedFlag, ContMultiplicity, ContRef, EffectItem, EffectItemKind, EffectOp, EffectRow,
    Env, HandlerChain, HandlerClause, ResumeRowMetadata, Term, Value,
};
use ash_interp::cps::validate::{CpsValidationError, validate_cps_program};
use ash_interp::cps::{CpsError, CpsRuntime, eval_unchecked_with_runtime};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_op() -> EffectOp {
    EffectOp {
        item: EffectItem {
            namespace: "cap".to_string(),
            name: "x.read".to_string(),
            kind: EffectItemKind::Capability,
        },
        arg_types: vec![],
        result_type: "Int".to_string(),
    }
}

fn non_empty_row() -> EffectRow {
    EffectRow {
        items: vec![EffectItem {
            namespace: "cap".to_string(),
            name: "db.read".to_string(),
            kind: EffectItemKind::Capability,
        }],
    }
}

fn empty_row() -> EffectRow {
    EffectRow::default()
}

fn return_term() -> Term {
    Term::Return {
        value: Atom::Int(0),
    }
}

// ---------------------------------------------------------------------------
// 1-2: Value::Cont validation
// ---------------------------------------------------------------------------

#[test]
fn reject_multishot_cont_with_nonempty_row() {
    // Value::Cont with MultiShotPure and non-empty row should be rejected.
    let cont = Value::Cont {
        param: "v".to_string(),
        body: Box::new(return_term()),
        captured_env: Env::new(),
        captured_chain: HandlerChain::new(),
        consumed: ConsumedFlag::new(),
        row: non_empty_row(),
        multiplicity: ContMultiplicity::MultiShotPure,
    };
    // Wrap in a LetVal so validate_cps_program traverses the value.
    let term = Term::LetVal {
        name: "k".to_string(),
        value: cont,
        body: Box::new(return_term()),
    };
    let result = validate_cps_program(&term);
    assert!(
        matches!(result, Err(CpsValidationError::InvalidSyntacticPosition(ref s)) if s.contains("multi-shot-pure")),
        "should reject multi-shot cont with non-empty row, got: {result:?}"
    );
}

#[test]
fn reject_multishot_cont_empty_row_but_effectful_body() {
    // Value::Cont with MultiShotPure, empty declared row, but body that has a
    // non-empty effective row (a Raise).
    let cont = Value::Cont {
        param: "v".to_string(),
        body: Box::new(Term::Raise {
            op: make_op(),
            args: vec![],
            resume: ContRef::Var("resume".to_string()),
            row: non_empty_row(),
        }),
        captured_env: Env::new(),
        captured_chain: HandlerChain::new(),
        consumed: ConsumedFlag::new(),
        row: empty_row(),
        multiplicity: ContMultiplicity::MultiShotPure,
    };
    let term = Term::LetVal {
        name: "k".to_string(),
        value: cont,
        body: Box::new(return_term()),
    };
    let result = validate_cps_program(&term);
    assert!(
        matches!(result, Err(CpsValidationError::InvalidSyntacticPosition(ref s)) if s.contains("non-empty effective row")),
        "should reject multi-shot cont with effectful body, got: {result:?}"
    );
}

#[test]
fn accept_multishot_cont_empty_row_empty_body() {
    // Value::Cont with MultiShotPure, empty row, and empty-row body (Return).
    let cont = Value::Cont {
        param: "v".to_string(),
        body: Box::new(return_term()),
        captured_env: Env::new(),
        captured_chain: HandlerChain::new(),
        consumed: ConsumedFlag::new(),
        row: empty_row(),
        multiplicity: ContMultiplicity::MultiShotPure,
    };
    let term = Term::LetVal {
        name: "k".to_string(),
        value: cont,
        body: Box::new(return_term()),
    };
    let result = validate_cps_program(&term);
    assert!(
        result.is_ok(),
        "should accept valid multishot cont: {result:?}"
    );
}

#[test]
fn accept_affine_cont_with_nonempty_row() {
    // Affine cont with non-empty row is valid.
    let cont = Value::Cont {
        param: "v".to_string(),
        body: Box::new(return_term()),
        captured_env: Env::new(),
        captured_chain: HandlerChain::new(),
        consumed: ConsumedFlag::new(),
        row: non_empty_row(),
        multiplicity: ContMultiplicity::Affine,
    };
    let term = Term::LetVal {
        name: "k".to_string(),
        value: cont,
        body: Box::new(return_term()),
    };
    let result = validate_cps_program(&term);
    assert!(
        result.is_ok(),
        "affine non-empty row should be valid: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// 5-6: Term::LetCont validation
// ---------------------------------------------------------------------------

#[test]
fn reject_multishot_letcont_nonempty_row() {
    let term = Term::LetCont {
        name: "k".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(return_term()),
        body: Box::new(return_term()),
        row: non_empty_row(),
        multiplicity: ContMultiplicity::MultiShotPure,
    };
    let result = validate_cps_program(&term);
    assert!(
        matches!(result, Err(CpsValidationError::InvalidSyntacticPosition(ref s)) if s.contains("multi-shot-pure")),
        "should reject multishot LetCont with non-empty row, got: {result:?}"
    );
}

#[test]
fn reject_multishot_letcont_empty_row_effectful_body() {
    let term = Term::LetCont {
        name: "k".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(Term::Raise {
            op: make_op(),
            args: vec![],
            resume: ContRef::Var("resume".to_string()),
            row: non_empty_row(),
        }),
        body: Box::new(return_term()),
        row: empty_row(),
        multiplicity: ContMultiplicity::MultiShotPure,
    };
    let result = validate_cps_program(&term);
    assert!(
        matches!(result, Err(CpsValidationError::InvalidSyntacticPosition(ref s)) if s.contains("non-empty effective row")),
        "should reject multishot LetCont with effectful body, got: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// 7-8: HandlerClause resume validation
// ---------------------------------------------------------------------------

fn make_handler_clause(
    resume_row: ResumeRowMetadata,
    resume_multiplicity: ContMultiplicity,
) -> HandlerClause {
    HandlerClause {
        op: make_op(),
        params: vec![],
        resume: "resume".to_string(),
        body: Box::new(return_term()),
        row: empty_row(),
        resume_row,
        resume_multiplicity,
    }
}

#[test]
fn reject_multishot_handler_known_nonempty_row() {
    let clause = make_handler_clause(
        ResumeRowMetadata::Known(non_empty_row()),
        ContMultiplicity::MultiShotPure,
    );
    let term = Term::Handle {
        clause,
        body: Box::new(return_term()),
        cont: ContRef::Label("exit".to_string()),
        row: empty_row(),
    };
    let term = Term::LetCont {
        name: "exit".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(return_term()),
        body: Box::new(term),
        row: empty_row(),
        multiplicity: ContMultiplicity::Affine,
    };
    let result = validate_cps_program(&term);
    assert!(
        matches!(result, Err(CpsValidationError::InvalidSyntacticPosition(ref s)) if s.contains("multi-shot-pure resume")),
        "should reject multishot handler with known non-empty row, got: {result:?}"
    );
}

#[test]
fn reject_multishot_handler_inherited_row() {
    let clause = make_handler_clause(
        ResumeRowMetadata::InheritFromTarget,
        ContMultiplicity::MultiShotPure,
    );
    let term = Term::Handle {
        clause,
        body: Box::new(return_term()),
        cont: ContRef::Label("exit".to_string()),
        row: empty_row(),
    };
    let term = Term::LetCont {
        name: "exit".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(return_term()),
        body: Box::new(term),
        row: empty_row(),
        multiplicity: ContMultiplicity::Affine,
    };
    let result = validate_cps_program(&term);
    assert!(
        matches!(result, Err(CpsValidationError::InvalidSyntacticPosition(ref s)) if s.contains("inherited target row")),
        "should reject multishot handler with inherited row using current wording, got: {result:?}"
    );
}

#[test]
fn accept_affine_handler_inherited_row() {
    let clause = make_handler_clause(
        ResumeRowMetadata::InheritFromTarget,
        ContMultiplicity::Affine,
    );
    let term = Term::Handle {
        clause,
        body: Box::new(return_term()),
        cont: ContRef::Label("exit".to_string()),
        row: empty_row(),
    };
    let term = Term::LetCont {
        name: "exit".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(return_term()),
        body: Box::new(term),
        row: empty_row(),
        multiplicity: ContMultiplicity::Affine,
    };
    let result = validate_cps_program(&term);
    assert!(
        result.is_ok(),
        "affine inherited-row handler should be valid: {result:?}"
    );
}

#[test]
fn accept_multishot_handler_known_empty_row() {
    let clause = make_handler_clause(
        ResumeRowMetadata::Known(empty_row()),
        ContMultiplicity::MultiShotPure,
    );
    let term = Term::Handle {
        clause,
        body: Box::new(return_term()),
        cont: ContRef::Label("exit".to_string()),
        row: empty_row(),
    };
    let term = Term::LetCont {
        name: "exit".to_string(),
        param: "v".to_string(),
        cont_body: Box::new(return_term()),
        body: Box::new(term),
        row: empty_row(),
        multiplicity: ContMultiplicity::Affine,
    };
    let result = validate_cps_program(&term);
    assert!(
        result.is_ok(),
        "multishot handler with known empty row should be valid: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// Runtime fail-closed test for unchecked invalid value
// ---------------------------------------------------------------------------

#[test]
fn runtime_fail_closed_for_multishot_nonempty_row() {
    // Even if validation is bypassed, the runtime should reject multi-shot-pure
    // continuations with non-empty declared rows.
    let cont = Value::Cont {
        param: "v".to_string(),
        body: Box::new(return_term()),
        captured_env: Env::new(),
        captured_chain: HandlerChain::new(),
        consumed: ConsumedFlag::new(),
        row: non_empty_row(),
        multiplicity: ContMultiplicity::MultiShotPure,
    };
    let env = Env::new().with_binding("k".to_string(), cont);
    let term = Term::Jump {
        cont: ContRef::Var("k".to_string()),
        arg: Atom::Int(42),
        row: empty_row(),
    };
    let mut runtime = CpsRuntime::new();
    let result = eval_unchecked_with_runtime(&term, &env, &HandlerChain::new(), &mut runtime);
    assert!(
        matches!(result, Err(CpsError::Trap(ref _r))),
        "runtime should fail-closed on multishot non-empty row, got: {result:?}"
    );
}
