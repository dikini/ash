//! TASK-1684: Core Continuation Multiplicity Well-Formedness
//!
//! Tests that `CoreType::Cont` with `MultiShotPure` multiplicity is accepted
//! only when the row is normalized closed empty, and rejected otherwise.

use ash_core::core_ash::{CoreMultiplicity, CoreRow, CoreRowItem, CoreType};
use ash_core::core_ash_typecheck::{
    CoreTypeCheckEnv, CoreTypeCheckError, check_core_type_well_formed,
};

fn cap_item(ns: &str, name: &str) -> CoreRowItem {
    CoreRowItem::Capability {
        path: vec![ns.to_string()],
        operation: name.to_string(),
    }
}

fn cont_type(row: CoreRow, multiplicity: CoreMultiplicity) -> CoreType {
    CoreType::Cont {
        input: Box::new(CoreType::Base("Int".into())),
        answer: Box::new(CoreType::Base("Unit".into())),
        row,
        multiplicity,
    }
}

fn empty_row() -> CoreRow {
    CoreRow::closed(vec![])
}

fn non_empty_row() -> CoreRow {
    CoreRow::closed(vec![cap_item("cap", "db.read")])
}

fn open_row() -> CoreRow {
    CoreRow {
        items: vec![],
        tail: Some("r".into()),
    }
}

#[test]
fn multishot_pure_empty_row_is_well_formed() {
    let ty = cont_type(empty_row(), CoreMultiplicity::MultiShotPure);
    let result = check_core_type_well_formed(&ty, &CoreTypeCheckEnv::default());
    assert!(
        result.is_ok(),
        "empty-row multi-shot-pure should be well formed: {result:?}"
    );
}

#[test]
fn multishot_pure_nonempty_row_is_rejected() {
    let ty = cont_type(non_empty_row(), CoreMultiplicity::MultiShotPure);
    let result = check_core_type_well_formed(&ty, &CoreTypeCheckEnv::default());
    assert!(
        result.is_err(),
        "non-empty-row multi-shot-pure should be rejected"
    );
    let err = result.unwrap_err();
    let msg = format!("{err:?}");
    assert!(
        msg.contains("multi-shot-pure") && msg.contains("closed empty row"),
        "error should mention multiplicity and row legality: {err:?}"
    );
}

#[test]
fn multishot_pure_open_row_is_rejected() {
    // An open row (with tail variable) must be rejected.
    let mut env = CoreTypeCheckEnv::default();
    env.types_mut().insert_name("r");
    let ty = cont_type(open_row(), CoreMultiplicity::MultiShotPure);
    let result = check_core_type_well_formed(&ty, &env);
    assert!(
        result.is_err(),
        "open-row multi-shot-pure should be rejected"
    );
}

#[test]
fn affine_nonempty_row_is_well_formed() {
    let ty = cont_type(non_empty_row(), CoreMultiplicity::Affine);
    let result = check_core_type_well_formed(&ty, &CoreTypeCheckEnv::default());
    assert!(
        result.is_ok(),
        "affine non-empty row should be well formed: {result:?}"
    );
}

#[test]
fn affine_empty_row_is_well_formed() {
    let ty = cont_type(empty_row(), CoreMultiplicity::Affine);
    let result = check_core_type_well_formed(&ty, &CoreTypeCheckEnv::default());
    assert!(
        result.is_ok(),
        "affine empty row should be well formed: {result:?}"
    );
}

#[test]
fn affine_open_row_rejected_for_unknown_tail() {
    // An open row with an unknown tail variable is rejected by row well-formedness,
    // independent of multiplicity.
    let ty = cont_type(open_row(), CoreMultiplicity::Affine);
    let result = check_core_type_well_formed(&ty, &CoreTypeCheckEnv::default());
    assert!(
        matches!(result, Err(CoreTypeCheckError::UnknownRowVariable { .. })),
        "affine open row with unknown tail should be rejected: {result:?}"
    );
}
