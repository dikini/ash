use std::path::{Path, PathBuf};

use ash_core::core_ash::{
    CoreAtom, CoreEffectOp, CoreExpr, CoreRow, CoreRowItem, CoreType, CoreValue,
};
use ash_core::core_ash_text::{parse_core_expr, parse_core_file};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("ash-core crate lives under crates/ash-core")
        .to_path_buf()
}

fn fixture_path(name: &str) -> PathBuf {
    repo_root()
        .join("crates/ash-core/tests/fixtures/core")
        .join(name)
}

fn unit() -> CoreType {
    CoreType::Base("Unit".to_string())
}

fn console_read_row_item() -> CoreRowItem {
    CoreRowItem::Capability {
        path: vec!["console".to_string()],
        operation: "read".to_string(),
    }
}

#[test]
fn validates_valid_core_fixture_before_lowering() {
    let expr = parse_core_file(fixture_path("call_non_tail.core")).unwrap();
    let valid = validate_core_program(RawCoreProgram::new(expr.clone())).unwrap();

    assert_eq!(valid.expr(), &expr);
}

#[test]
fn rejects_duplicate_row_items() {
    let duplicate = console_read_row_item();
    let expr = CoreExpr::LetVal {
        name: "f".to_string(),
        ty: CoreType::Function {
            params: vec![unit()],
            result: Box::new(unit()),
            row: CoreRow::closed(vec![duplicate.clone(), duplicate]),
        },
        value: CoreValue::Atom(CoreAtom::LitUnit),
        body: Box::new(CoreExpr::Atom(CoreAtom::LitUnit)),
    };

    let error = validate_core_program(RawCoreProgram::new(expr)).unwrap_err();
    assert!(
        error.to_string().contains("duplicate row item"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_malformed_effect_operation_shapes() {
    let expr = CoreExpr::Raise {
        op: CoreEffectOp::Capability {
            path: Vec::new(),
            operation: "read".to_string(),
            arg_types: vec![unit()],
            result_type: unit(),
        },
        args: vec![CoreAtom::LitString("ok".to_string())],
    };

    let error = validate_core_program(RawCoreProgram::new(expr)).unwrap_err();
    assert!(
        error.to_string().contains("unsupported effect operation"),
        "unexpected error: {error}"
    );
}

#[test]
fn labels_are_rejected_as_data_atoms_before_lowering() {
    let error = parse_core_expr("(call f ((label exit)))").unwrap_err();
    assert!(
        error.to_string().contains("unsupported atom form"),
        "unexpected error: {error}"
    );
}
