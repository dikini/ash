use ash_core::core_ash::{
    CoreAtom, CoreEffectOp, CoreEvalMode, CoreExpr, CoreRow, CoreRowItem, CoreType,
};
use ash_core::core_ash_typecheck::{CoreTypeCheckEnv, CoreTypeCheckError, type_check_core_program};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};

fn cap(path: &[&str], operation: &str) -> CoreRowItem {
    CoreRowItem::Capability {
        path: path.iter().map(|part| (*part).to_owned()).collect(),
        operation: operation.to_owned(),
    }
}

fn cap_row(path: &[&str], operation: &str) -> CoreRow {
    CoreRow::closed(vec![cap(path, operation)])
}

fn strict_mode(inner: CoreType) -> CoreType {
    CoreType::Mode {
        mode: CoreEvalMode::Strict,
        inner: Box::new(inner),
        latent_row: None,
    }
}

fn lazy_mode(inner: CoreType, latent_row: CoreRow) -> CoreType {
    CoreType::Mode {
        mode: CoreEvalMode::Lazy,
        inner: Box::new(inner),
        latent_row: Some(latent_row),
    }
}

fn memo_mode(inner: CoreType, latent_row: CoreRow) -> CoreType {
    CoreType::Mode {
        mode: CoreEvalMode::Memo,
        inner: Box::new(inner),
        latent_row: Some(latent_row),
    }
}

fn capability_op(name: &[&str], operation: &str) -> CoreEffectOp {
    CoreEffectOp::Capability {
        path: name.iter().map(|part| (*part).to_owned()).collect(),
        operation: operation.to_owned(),
        arg_types: Vec::new(),
        result_type: CoreType::Base("Unit".into()),
    }
}

#[test]
fn strict_letmode_behaves_like_strict_binding() {
    let op = capability_op(&["jobs"], "read");
    let expr_row = cap_row(&["jobs"], "read");
    let letmode_ty = strict_mode(CoreType::Base("Unit".into()));
    let letmode_expr = CoreExpr::LetMode {
        name: "x".into(),
        mode: CoreEvalMode::Strict,
        ty: letmode_ty,
        expr: Box::new(CoreExpr::Raise {
            op,
            args: Vec::new(),
        }),
        body: Box::new(CoreExpr::Atom(CoreAtom::Var("x".into()))),
    };

    let mut env = CoreTypeCheckEnv::default();
    env.operations_mut()
        .insert(capability_op(&["jobs"], "read"));
    let valid = validate_core_program(RawCoreProgram::new(letmode_expr));
    let typed = type_check_core_program(valid.expect("validated"), &env)
        .expect("strict letmode type checks");

    assert_eq!(*typed.ty(), strict_mode(CoreType::Base("Unit".into())));
    assert_eq!(
        typed.row(),
        &expr_row,
        "strict letmode includes initializer effects"
    );
}

#[test]
fn lazy_letmode_lifts_initializer_row_into_binding_latent_row_and_uses_mode_type() {
    let expr_row = cap_row(&["jobs"], "write");
    let letmode_expr = CoreExpr::LetMode {
        name: "thunked".into(),
        mode: CoreEvalMode::Lazy,
        ty: lazy_mode(CoreType::Base("Unit".into()), expr_row.clone()),
        expr: Box::new(CoreExpr::Raise {
            op: capability_op(&["jobs"], "write"),
            args: Vec::new(),
        }),
        body: Box::new(CoreExpr::Atom(CoreAtom::LitInt(1))),
    };

    let typed = type_check_core_program(
        validate_core_program(RawCoreProgram::new(letmode_expr)).expect("validated"),
        &{
            let mut env = CoreTypeCheckEnv::default();
            env.operations_mut()
                .insert(capability_op(&["jobs"], "write"));
            env
        },
    )
    .expect("lazy letmode type checks");

    assert_eq!(*typed.ty(), CoreType::Base("Int".into()));
    assert_eq!(
        typed.row(),
        &CoreRow::default(),
        "lazy letmode suppresses initializer row"
    );
    assert_eq!(typed.facts().mode_binding_latent_rows().len(), 1);
    assert_eq!(
        typed.facts().mode_binding_latent_rows().get("thunked"),
        Some(&expr_row)
    );
}

#[test]
fn lazy_letmode_row_mismatch_reports_mode_latent_row_mismatch() {
    let letmode_expr = CoreExpr::LetMode {
        name: "thunked".into(),
        mode: CoreEvalMode::Lazy,
        ty: lazy_mode(CoreType::Base("Unit".into()), cap_row(&["jobs"], "write")),
        expr: Box::new(CoreExpr::Raise {
            op: capability_op(&["jobs"], "read"),
            args: Vec::new(),
        }),
        body: Box::new(CoreExpr::Atom(CoreAtom::LitInt(1))),
    };

    let mut env = CoreTypeCheckEnv::default();
    env.operations_mut()
        .insert(capability_op(&["jobs"], "write"));
    env.operations_mut()
        .insert(capability_op(&["jobs"], "read"));

    let err = type_check_core_program(
        validate_core_program(RawCoreProgram::new(letmode_expr)).expect("validated"),
        &env,
    )
    .expect_err("latent-row mismatch must fail");

    match err {
        CoreTypeCheckError::ModeLatentRowMismatch {
            name,
            expected,
            actual,
        } => {
            assert_eq!(name, "thunked");
            assert_eq!(expected, cap_row(&["jobs"], "write"));
            assert_eq!(actual, cap_row(&["jobs"], "read"));
        }
        _ => panic!("unexpected error: {err:?}"),
    }
}

#[test]
fn force_returns_inner_type_and_contributes_thunk_row() {
    let thunk_row = cap_row(&["jobs"], "memo");
    let force_expr = CoreExpr::Force {
        name: "forced".into(),
        thunk: CoreAtom::Var("lazy_fn".into()),
        body: Box::new(CoreExpr::Atom(CoreAtom::Var("forced".into()))),
    };

    let mut env = CoreTypeCheckEnv::default();
    env.values_mut().insert(
        "lazy_fn",
        CoreType::Mode {
            mode: CoreEvalMode::Lazy,
            inner: Box::new(CoreType::Base("Int".into())),
            latent_row: Some(thunk_row.clone()),
        },
    );

    let typed = type_check_core_program(
        validate_core_program(RawCoreProgram::new(force_expr)).expect("validated"),
        &env,
    )
    .expect("force of lazy mode should type check");

    assert_eq!(typed.ty(), &CoreType::Base("Int".into()));
    assert_eq!(
        typed.row(),
        &thunk_row,
        "force residual row includes thunk latent row"
    );
}

#[test]
fn force_body_type_can_differ_from_thunk_inner_type() {
    let force_expr = CoreExpr::Force {
        name: "forced".into(),
        thunk: CoreAtom::Var("lazy_fn".into()),
        body: Box::new(CoreExpr::If {
            cond: CoreAtom::LitBool(true),
            then_branch: Box::new(CoreExpr::Atom(CoreAtom::LitBool(true))),
            else_branch: Box::new(CoreExpr::Atom(CoreAtom::LitBool(false))),
        }),
    };

    let mut env = CoreTypeCheckEnv::default();
    env.values_mut().insert(
        "lazy_fn",
        CoreType::Mode {
            mode: CoreEvalMode::Lazy,
            inner: Box::new(CoreType::Base("Int".into())),
            latent_row: Some(CoreRow::default()),
        },
    );

    let typed = type_check_core_program(
        validate_core_program(RawCoreProgram::new(force_expr)).expect("validated"),
        &env,
    )
    .expect("force body should type-check with continuation-style result");

    assert_eq!(typed.ty(), &CoreType::Base("Bool".into()));
    assert_eq!(typed.row(), &CoreRow::default());
}

#[test]
fn force_rejects_strict_mode_binding() {
    let force_expr = CoreExpr::Force {
        name: "forced".into(),
        thunk: CoreAtom::Var("strict_fn".into()),
        body: Box::new(CoreExpr::Atom(CoreAtom::LitInt(1))),
    };

    let mut env = CoreTypeCheckEnv::default();
    env.values_mut().insert(
        "strict_fn",
        CoreType::Mode {
            mode: CoreEvalMode::Strict,
            inner: Box::new(CoreType::Base("Int".into())),
            latent_row: None,
        },
    );

    let err = type_check_core_program(
        validate_core_program(RawCoreProgram::new(force_expr)).expect("validated"),
        &env,
    )
    .expect_err("forcing strict modes should fail");

    assert!(
        matches!(err, CoreTypeCheckError::UnsupportedCoreForm { .. }),
        "{err:?}"
    );
}

#[test]
fn mode_binding_row_facts_are_emitted_for_letmode() {
    let letmode_expr = CoreExpr::LetMode {
        name: "memoized".into(),
        mode: CoreEvalMode::Memo,
        ty: memo_mode(CoreType::Base("Unit".into()), cap_row(&["jobs"], "read")),
        expr: Box::new(CoreExpr::Raise {
            op: capability_op(&["jobs"], "read"),
            args: Vec::new(),
        }),
        body: Box::new(CoreExpr::Atom(CoreAtom::LitInt(1))),
    };

    let typed = type_check_core_program(
        validate_core_program(RawCoreProgram::new(letmode_expr)).expect("validated"),
        &{
            let mut env = CoreTypeCheckEnv::default();
            env.operations_mut()
                .insert(capability_op(&["jobs"], "read"));
            env
        },
    )
    .expect("memo letmode type checks");

    assert_eq!(typed.ty(), &CoreType::Base("Int".into()));
    assert_eq!(
        typed
            .facts()
            .mode_binding_latent_rows()
            .get("memoized")
            .expect("memo row fact recorded"),
        &cap_row(&["jobs"], "read")
    );
}
