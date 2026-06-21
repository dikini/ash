use ash_core::core_ash::{
    CoreAtom, CoreCaptureSet, CoreEvalMode, CoreExpr, CoreRow, CoreThunkMode, CoreType, CoreValue,
};
use ash_core::core_ash_text::parse_core_expr;
use ash_core::core_ash_validate::{CoreValidationError, RawCoreProgram, validate_core_program};

fn unit_type() -> CoreType {
    CoreType::Base("Unit".to_string())
}

fn function_type() -> CoreType {
    CoreType::Function {
        params: vec![],
        result: Box::new(unit_type()),
        row: CoreRow::default(),
    }
}

fn letmode_expr(
    name: &str,
    mode: CoreEvalMode,
    inner: &str,
    latent: Option<CoreRow>,
    body: CoreExpr,
) -> CoreExpr {
    CoreExpr::LetMode {
        name: name.to_string(),
        mode,
        ty: CoreType::Mode {
            mode,
            inner: Box::new(CoreType::Base(inner.to_string())),
            latent_row: latent,
        },
        expr: Box::new(CoreExpr::Atom(CoreAtom::LitUnit)),
        body: Box::new(body),
    }
}

#[test]
fn validates_letmode_type_matches_mode_shape() {
    let valid = CoreExpr::If {
        cond: CoreAtom::LitBool(true),
        then_branch: Box::new(letmode_expr(
            "a",
            CoreEvalMode::Strict,
            "Int",
            None,
            CoreExpr::Atom(CoreAtom::Var("a".into())),
        )),
        else_branch: Box::new(letmode_expr(
            "a",
            CoreEvalMode::Lazy,
            "String",
            Some(CoreRow::default()),
            CoreExpr::Atom(CoreAtom::Var("a".into())),
        )),
    };

    assert!(
        validate_core_program(RawCoreProgram::new(valid)).is_ok(),
        "mode wrapper should match LetMode mode in both branches"
    );

    let mismatch = parse_core_expr("(let-mode x strict : (lazy Unit {}) (lit-unit) x)").unwrap();
    let error = validate_core_program(RawCoreProgram::new(mismatch)).unwrap_err();
    assert!(
        matches!(error, CoreValidationError::LetModeTypeMismatch { .. }),
        "unexpected error: {error}"
    );
}

#[test]
fn validates_if_branches_with_independent_mode_binders() {
    let expr = CoreExpr::If {
        cond: CoreAtom::LitBool(true),
        then_branch: Box::new(letmode_expr(
            "x",
            CoreEvalMode::Memo,
            "Int",
            Some(CoreRow::default()),
            CoreExpr::Atom(CoreAtom::Var("x".into())),
        )),
        else_branch: Box::new(letmode_expr(
            "x",
            CoreEvalMode::Memo,
            "Int",
            Some(CoreRow::default()),
            CoreExpr::Atom(CoreAtom::Var("x".into())),
        )),
    };

    assert!(
        validate_core_program(RawCoreProgram::new(expr)).is_ok(),
        "duplicate mode names in independent branches should be allowed"
    );
}

#[test]
fn force_rejects_non_variable_thunk_atom() {
    let expr = parse_core_expr("(force x (lit-int 1) (lit-unit))").unwrap();
    let error = validate_core_program(RawCoreProgram::new(expr)).unwrap_err();
    assert!(
        matches!(
            error,
            CoreValidationError::ForceRequiresVariableThunk { .. }
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn thunk_binders_do_not_leak_into_let_body_scope() {
    let expr = CoreExpr::LetVal {
        name: "reader".into(),
        ty: function_type(),
        value: CoreValue::Thunk {
            mode: CoreThunkMode::Lazy,
            result_ty: unit_type(),
            row: CoreRow::default(),
            body: Box::new(CoreExpr::LetVal {
                name: "x".into(),
                ty: function_type(),
                value: CoreValue::Atom(CoreAtom::LitUnit),
                body: Box::new(CoreExpr::Atom(CoreAtom::Var("x".into()))),
            }),
            captures: CoreCaptureSet::default(),
        },
        body: Box::new(CoreExpr::LetVal {
            name: "x".into(),
            ty: function_type(),
            value: CoreValue::Atom(CoreAtom::LitUnit),
            body: Box::new(CoreExpr::Atom(CoreAtom::Var("reader".into()))),
        }),
    };

    assert!(
        validate_core_program(RawCoreProgram::new(expr)).is_ok(),
        "binder from thunk body should not leak into enclosing scope"
    );
}

#[test]
fn validates_nested_bodies_within_thunks_and_lambdas() {
    let thunk_body = parse_core_expr("(let-mode x strict : (memo Unit {}) (lit-unit) x)").unwrap();
    let lambda_body = parse_core_expr("(let-mode y lazy : (strict Unit) (lit-unit) y)").unwrap();

    let expr = CoreExpr::Atom(CoreAtom::Var("ok".into()));
    let expr = CoreExpr::LetVal {
        name: "ok".into(),
        ty: function_type(),
        value: CoreValue::Thunk {
            mode: CoreThunkMode::Memo,
            result_ty: unit_type(),
            row: CoreRow::default(),
            body: Box::new(thunk_body),
            captures: CoreCaptureSet::default(),
        },
        body: Box::new(CoreExpr::LetVal {
            name: "lam".into(),
            ty: function_type(),
            value: CoreValue::Lam {
                params: Vec::new(),
                body: Box::new(lambda_body),
                row: CoreRow::default(),
            },
            body: Box::new(expr),
        }),
    };

    let error = validate_core_program(RawCoreProgram::new(expr)).unwrap_err();
    assert!(
        matches!(error, CoreValidationError::LetModeTypeMismatch { .. }),
        "unexpected error: {error}"
    );
}

#[test]
fn validates_force_result_name_as_scoped_binder() {
    let expr = CoreExpr::LetVal {
        name: "result".into(),
        ty: function_type(),
        value: CoreValue::Atom(CoreAtom::LitUnit),
        body: Box::new(CoreExpr::Force {
            name: "result".into(),
            thunk: CoreAtom::Var("thunk".into()),
            body: Box::new(CoreExpr::Atom(CoreAtom::Var("result".into()))),
        }),
    };

    let error = validate_core_program(RawCoreProgram::new(expr)).unwrap_err();
    match error {
        CoreValidationError::DuplicateBinding { kind, name }
            if kind == "force" && name == "result" => {}
        other => panic!("expected force result duplicate, got {other:?}"),
    }
}
