use ash_core::core_ash::{
    CoreAtom, CoreCaptureSet, CoreEffectOp, CoreEvalMode, CoreExpr, CoreRow, CoreRowItem,
    CoreThunkMode, CoreType, CoreValue,
};
use ash_core::core_ash_typecheck::{
    CoreTypeCheckEnv, CoreTypeCheckError, core_types_equivalent, synthesize_core_value,
};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};

fn cap(path: &[&str], operation: &str) -> CoreRowItem {
    CoreRowItem::Capability {
        path: path.iter().map(|part| (*part).to_owned()).collect(),
        operation: operation.to_owned(),
    }
}

fn chan(path: &[&str], mode: &str, payload: CoreType) -> CoreRowItem {
    CoreRowItem::Channel {
        path: path.iter().map(|part| (*part).to_owned()).collect(),
        mode: mode.to_owned(),
        payload_type: Box::new(payload),
    }
}

fn channel_op(path: &[&str], mode: &str, payload: CoreType) -> CoreEffectOp {
    CoreEffectOp::Channel {
        path: path.iter().map(|part| (*part).to_owned()).collect(),
        mode: mode.to_owned(),
        payload_type: payload,
        result_type: CoreType::Base("Unit".into()),
    }
}

#[test]
fn thunk_value_synthesizes_mode_type_and_empty_construction_row() {
    let thunk = CoreValue::Thunk {
        mode: CoreThunkMode::Lazy,
        result_ty: CoreType::Base("Int".into()),
        body: Box::new(CoreExpr::Atom(CoreAtom::LitInt(7))),
        row: CoreRow::default(),
        captures: CoreCaptureSet::default(),
    };

    let typed = synthesize_core_value(&thunk, &CoreTypeCheckEnv::default())
        .expect("valid thunk should synthesize to mode type");

    assert_eq!(typed.row(), &CoreRow::default());
    assert_eq!(
        typed.ty(),
        &CoreType::Mode {
            mode: CoreEvalMode::Lazy,
            inner: Box::new(CoreType::Base("Int".into())),
            latent_row: Some(CoreRow::default()),
        }
    );
}

#[test]
fn thunk_body_type_check_fails_when_result_type_mismatches() {
    let thunk = CoreValue::Thunk {
        mode: CoreThunkMode::Lazy,
        result_ty: CoreType::Base("Int".into()),
        body: Box::new(CoreExpr::Atom(CoreAtom::LitBool(true))),
        row: CoreRow::default(),
        captures: CoreCaptureSet::default(),
    };

    let err = synthesize_core_value(&thunk, &CoreTypeCheckEnv::default())
        .expect_err("thunk result type must match body");

    assert!(
        matches!(err, CoreTypeCheckError::TypeMismatch { .. }),
        "unexpected error: {err:?}"
    );
}

#[test]
fn thunk_latent_row_matches_structural_body_row() {
    let payload_body = CoreType::Record(vec![
        ("a".into(), CoreType::Base("Int".into())),
        ("b".into(), CoreType::Base("String".into())),
    ]);
    let payload_annotation = CoreType::Record(vec![
        ("b".into(), CoreType::Base("String".into())),
        ("a".into(), CoreType::Base("Int".into())),
    ]);

    let annotated_row = CoreRow::closed(vec![chan(&["jobs"], "send", payload_annotation.clone())]);

    let mut env = CoreTypeCheckEnv::default();
    env.values_mut().insert("payload", payload_body.clone());
    env.operations_mut()
        .insert(channel_op(&["jobs"], "send", payload_body.clone()));

    let thunk = CoreValue::Thunk {
        mode: CoreThunkMode::Memo,
        result_ty: CoreType::Base("Unit".into()),
        body: Box::new(CoreExpr::LetVal {
            name: "payload".into(),
            ty: payload_body.clone(),
            value: CoreValue::Atom(CoreAtom::Var("payload".into())),
            body: Box::new(CoreExpr::Raise {
                op: channel_op(&["jobs"], "send", payload_body),
                args: vec![CoreAtom::Var("payload".into())],
            }),
        }),
        row: annotated_row.clone(),
        captures: CoreCaptureSet::default(),
    };

    let typed = synthesize_core_value(&thunk, &env)
        .expect("thunk row annotation matches body row modulo record order");

    assert_eq!(typed.row(), &CoreRow::default());
    assert!(
        core_types_equivalent(
            typed.ty(),
            &CoreType::Mode {
                mode: CoreEvalMode::Memo,
                inner: Box::new(CoreType::Base("Unit".into())),
                latent_row: Some(annotated_row),
            },
            &env,
        )
        .expect("types should be equivalent"),
    );
}

#[test]
fn thunk_row_mismatch_is_rejected() {
    let thunk = CoreValue::Thunk {
        mode: CoreThunkMode::Memo,
        result_ty: CoreType::Base("Unit".into()),
        body: Box::new(CoreExpr::Atom(CoreAtom::LitUnit)),
        row: CoreRow::closed(vec![cap(&["jobs"], "read")]),
        captures: CoreCaptureSet::default(),
    };

    let err = synthesize_core_value(&thunk, &CoreTypeCheckEnv::default())
        .expect_err("non-empty annotation must match thunk body residual row");

    assert!(matches!(err, CoreTypeCheckError::RowMismatch { .. }));
}

#[test]
fn thunk_rejects_nested_mode_result_type() {
    let thunk = CoreValue::Thunk {
        mode: CoreThunkMode::Lazy,
        result_ty: CoreType::Mode {
            mode: CoreEvalMode::Lazy,
            inner: Box::new(CoreType::Base("Int".into())),
            latent_row: Some(CoreRow::default()),
        },
        body: Box::new(CoreExpr::Atom(CoreAtom::LitInt(1))),
        row: CoreRow::default(),
        captures: CoreCaptureSet::default(),
    };

    let err = synthesize_core_value(&thunk, &CoreTypeCheckEnv::default())
        .expect_err("mode results are not allowed in thunk construction");

    assert_eq!(
        err,
        CoreTypeCheckError::InvalidModeType {
            detail: "thunk result type must not be a mode type".to_owned()
        }
    );
}

#[test]
fn letval_of_thunk_typed_as_expected_mode_type() {
    let env = CoreTypeCheckEnv::default();
    let program = RawCoreProgram::new(CoreExpr::LetVal {
        name: "thunk".into(),
        ty: CoreType::Mode {
            mode: CoreEvalMode::Memo,
            inner: Box::new(CoreType::Base("Unit".into())),
            latent_row: Some(CoreRow::default()),
        },
        value: CoreValue::Thunk {
            mode: CoreThunkMode::Memo,
            result_ty: CoreType::Base("Unit".into()),
            body: Box::new(CoreExpr::Atom(CoreAtom::LitUnit)),
            row: CoreRow::default(),
            captures: CoreCaptureSet::default(),
        },
        body: Box::new(CoreExpr::Atom(CoreAtom::Var("thunk".into()))),
    });
    let program = validate_core_program(program).expect("thunk letval validates");

    let typed = ash_core::core_ash_typecheck::type_check_core_program(program, &env)
        .expect("thunk letval type-checks");

    assert_eq!(
        typed.ty(),
        &CoreType::Mode {
            mode: CoreEvalMode::Memo,
            inner: Box::new(CoreType::Base("Unit".into())),
            latent_row: Some(CoreRow::default()),
        }
    );
    assert_eq!(typed.row(), &CoreRow::default());
}
