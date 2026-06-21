use std::path::{Path, PathBuf};

use ash_core::core_ash::{
    CoreAtom, CoreEffectOp, CoreExpr, CoreParam, CoreRow, CoreRowItem, CoreType, CoreValue,
};
use ash_core::core_ash_text::{parse_core_expr, parse_core_file};
use ash_core::core_ash_validate::{CoreValidationError, RawCoreProgram, validate_core_program};

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

fn function_ty() -> CoreType {
    CoreType::Function {
        params: vec![],
        result: Box::new(unit()),
        row: CoreRow::default(),
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
fn if_branch_local_bindings_may_reuse_names() {
    let duplicate_name = "x".to_string();
    let duplicate_binding = CoreExpr::LetVal {
        name: duplicate_name.clone(),
        ty: function_ty(),
        value: CoreValue::Atom(CoreAtom::LitUnit),
        body: Box::new(CoreExpr::Atom(CoreAtom::LitUnit)),
    };

    let expr = CoreExpr::If {
        cond: CoreAtom::LitBool(true),
        then_branch: Box::new(duplicate_binding.clone()),
        else_branch: Box::new(duplicate_binding),
    };

    assert!(validate_core_program(RawCoreProgram::new(expr)).is_ok());
}

#[test]
fn rejects_duplicate_value_bindings_along_lexical_path() {
    let duplicate_name = "x".to_string();
    let inner = CoreExpr::LetVal {
        name: duplicate_name.clone(),
        ty: function_ty(),
        value: CoreValue::Atom(CoreAtom::LitUnit),
        body: Box::new(CoreExpr::Atom(CoreAtom::LitUnit)),
    };
    let expr = CoreExpr::LetVal {
        name: duplicate_name.clone(),
        ty: function_ty(),
        value: CoreValue::Atom(CoreAtom::LitUnit),
        body: Box::new(inner),
    };

    let error = validate_core_program(RawCoreProgram::new(expr)).unwrap_err();
    assert!(
        matches!(
            error,
            CoreValidationError::DuplicateBinding {
                kind: _,
                ref name,
            } if name == &duplicate_name
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_duplicate_bindings_between_parent_scope_and_branch() {
    let duplicate_name = "x".to_string();
    let expr = CoreExpr::LetVal {
        name: duplicate_name.clone(),
        ty: function_ty(),
        value: CoreValue::Atom(CoreAtom::LitUnit),
        body: Box::new(CoreExpr::If {
            cond: CoreAtom::LitBool(true),
            then_branch: Box::new(CoreExpr::LetVal {
                name: duplicate_name.clone(),
                ty: function_ty(),
                value: CoreValue::Atom(CoreAtom::LitUnit),
                body: Box::new(CoreExpr::Atom(CoreAtom::LitUnit)),
            }),
            else_branch: Box::new(CoreExpr::Atom(CoreAtom::LitUnit)),
        }),
    };

    let error = validate_core_program(RawCoreProgram::new(expr)).unwrap_err();
    assert!(
        matches!(
            error,
            CoreValidationError::DuplicateBinding {
                kind: _,
                ref name,
            } if name == &duplicate_name
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn let_lambda_param_does_not_leak_into_let_body_scope() {
    let expr = CoreExpr::LetVal {
        name: "f".into(),
        ty: function_ty(),
        value: CoreValue::Lam {
            params: vec![CoreParam {
                name: "x".into(),
                ty: unit(),
            }],
            body: Box::new(CoreExpr::Atom(CoreAtom::Var("x".into()))),
            row: CoreRow::default(),
        },
        body: Box::new(CoreExpr::LetVal {
            name: "x".into(),
            ty: function_ty(),
            value: CoreValue::Atom(CoreAtom::LitUnit),
            body: Box::new(CoreExpr::Atom(CoreAtom::LitUnit)),
        }),
    };

    assert!(validate_core_program(RawCoreProgram::new(expr)).is_ok());
}

#[test]
fn duplicate_lambda_params_still_fail_validation() {
    let expr = CoreExpr::LetVal {
        name: "f".into(),
        ty: function_ty(),
        value: CoreValue::Lam {
            params: vec![
                CoreParam {
                    name: "x".into(),
                    ty: unit(),
                },
                CoreParam {
                    name: "x".into(),
                    ty: unit(),
                },
            ],
            body: Box::new(CoreExpr::Atom(CoreAtom::LitUnit)),
            row: CoreRow::default(),
        },
        body: Box::new(CoreExpr::Atom(CoreAtom::LitUnit)),
    };

    assert!(matches!(
        validate_core_program(RawCoreProgram::new(expr)),
        Err(CoreValidationError::DuplicateBinding { kind: _, ref name }) if name == "x"
    ));
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
