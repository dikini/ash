use ash_core::core_ash::{
    CoreAtom, CoreCaptureSet, CoreEvalMode, CoreExpr, CoreRow, CoreThunkMode, CoreType, CoreValue,
};

#[test]
fn constructs_mode_wrappers_with_required_latent_rows() {
    let strict = CoreType::Mode {
        mode: CoreEvalMode::Strict,
        inner: Box::new(CoreType::Base("Int".into())),
        latent_row: None,
    };

    let lazy = CoreType::Mode {
        mode: CoreEvalMode::Lazy,
        inner: Box::new(CoreType::Base("String".into())),
        latent_row: Some(CoreRow::default()),
    };

    let memo = CoreType::Mode {
        mode: CoreEvalMode::Memo,
        inner: Box::new(CoreType::Base("Bool".into())),
        latent_row: Some(CoreRow::closed(vec![])),
    };

    assert!(matches!(
        strict,
        CoreType::Mode {
            mode: CoreEvalMode::Strict,
            ..
        }
    ));
    assert!(matches!(
        lazy,
        CoreType::Mode {
            mode: CoreEvalMode::Lazy,
            ..
        }
    ));
    assert!(matches!(
        memo,
        CoreType::Mode {
            mode: CoreEvalMode::Memo,
            ..
        }
    ));

    assert_eq!(strict, strict.clone());
    assert_eq!(lazy, lazy.clone());
    assert_eq!(memo, memo.clone());
}

#[test]
fn constructs_thunk_value_with_mode_row_and_captures() {
    let thunk = CoreValue::Thunk {
        mode: CoreThunkMode::Lazy,
        result_ty: CoreType::Base("Int".into()),
        body: Box::new(CoreExpr::Atom(CoreAtom::LitInt(42))),
        row: CoreRow::default(),
        captures: CoreCaptureSet {
            values: vec!["v".to_string()],
        },
    };

    match thunk {
        CoreValue::Thunk {
            mode,
            result_ty,
            body,
            row,
            captures,
        } => {
            assert_eq!(mode, CoreThunkMode::Lazy);
            assert_eq!(result_ty, CoreType::Base("Int".to_string()));
            assert!(matches!(*body, CoreExpr::Atom(CoreAtom::LitInt(42))));
            assert_eq!(row, CoreRow::default());
            assert_eq!(
                captures,
                CoreCaptureSet {
                    values: vec!["v".to_string()]
                }
            );
        }
        _ => panic!("expected CoreValue::Thunk"),
    }
}

#[test]
fn constructs_letmode_and_force_expressions() {
    let force = CoreExpr::Force {
        name: "forced".to_string(),
        thunk: CoreAtom::Var("th".to_string()),
        body: Box::new(CoreExpr::Atom(CoreAtom::Var("forced".to_string()))),
    };

    let expr = CoreExpr::LetMode {
        name: "th".to_string(),
        mode: CoreEvalMode::Memo,
        ty: CoreType::Mode {
            mode: CoreEvalMode::Memo,
            inner: Box::new(CoreType::Base("Int".into())),
            latent_row: Some(CoreRow::default()),
        },
        expr: Box::new(CoreExpr::Atom(CoreAtom::LitInt(1))),
        body: Box::new(force),
    };

    match expr {
        CoreExpr::LetMode {
            name,
            mode,
            ty,
            expr,
            body,
        } => {
            assert_eq!(name, "th");
            assert_eq!(mode, CoreEvalMode::Memo);
            assert!(matches!(
                ty,
                CoreType::Mode {
                    mode: CoreEvalMode::Memo,
                    ..
                }
            ));
            assert!(matches!(*expr, CoreExpr::Atom(CoreAtom::LitInt(1))));
            assert!(matches!(*body, CoreExpr::Force { name, .. } if name == "forced"));
        }
        _ => panic!("expected CoreExpr::LetMode"),
    }
}
