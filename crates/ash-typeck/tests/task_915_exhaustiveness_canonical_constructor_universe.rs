use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, VariantPayload, Visibility};
use ash_parser::surface::{Expr, Literal, MatchArm, Pattern, VariantPatternPayload};
use ash_parser::token::Span;
use ash_typeck::check_expr::check_expr;
use ash_typeck::{Kind, QualifiedName, Type, TypeEnv};

fn span() -> Span {
    Span::default()
}

fn result_type() -> TypeDef {
    TypeDef {
        name: "Result".into(),
        params: vec!["T".into(), "E".into()],
        body: TypeBody::Enum(vec![
            VariantDef {
                name: "Ok".into(),
                fields: vec![("value".into(), TypeExpr::Named("T".into()))],
                payload: VariantPayload::Record(vec![(
                    "value".into(),
                    TypeExpr::Named("T".into()),
                )]),
            },
            VariantDef {
                name: "Err".into(),
                fields: vec![("error".into(), TypeExpr::Named("E".into()))],
                payload: VariantPayload::Record(vec![(
                    "error".into(),
                    TypeExpr::Named("E".into()),
                )]),
            },
        ]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn int_result_alias_type() -> TypeDef {
    TypeDef {
        name: "IntResult".into(),
        params: vec!["E".into()],
        body: TypeBody::Alias(TypeExpr::Constructor {
            name: "Result".into(),
            args: vec![TypeExpr::Named("Int".into()), TypeExpr::Named("E".into())],
        }),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn env_with_result_and_alias() -> TypeEnv {
    let mut env = TypeEnv::new();
    env.register_type(&result_type()).expect("register Result");
    env.register_type(&int_result_alias_type())
        .expect("register IntResult alias");
    env
}

fn result_ty(ok: Type, err: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root("Result"),
        args: vec![ok, err],
        kind: Kind::Type,
    }
}

fn int_result_ty(err: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root("IntResult"),
        args: vec![err],
        kind: Kind::Type,
    }
}

fn variable(name: &str) -> Expr {
    Expr::Variable {
        name: name.into(),
        span: span(),
    }
}

fn int(value: i64) -> Expr {
    Expr::Literal(Literal::Int(value))
}

fn record_variant(name: &str, field_name: &str, binding_name: &str) -> Pattern {
    let binding = Pattern::Variable {
        name: binding_name.into(),
        span: span(),
    };
    Pattern::Variant {
        name: name.into(),
        fields: Some(vec![(field_name.into(), binding.clone())]),
        payload: VariantPatternPayload::Record(vec![(field_name.into(), binding)]),
    }
}

fn ok_arm(body: Expr) -> MatchArm {
    MatchArm {
        pattern: record_variant("Ok", "value", "x"),
        body: Box::new(body),
        span: span(),
    }
}

fn err_arm(body: Expr) -> MatchArm {
    MatchArm {
        pattern: record_variant("Err", "error", "e"),
        body: Box::new(body),
        span: span(),
    }
}

fn match_expr(scrutinee_name: &str, arms: Vec<MatchArm>) -> Expr {
    Expr::Match {
        scrutinee: Box::new(variable(scrutinee_name)),
        arms,
        span: span(),
    }
}

fn error_text<T: ToString>(errors: &[T]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn transparent_alias_full_match_uses_canonical_result_universe_and_is_exhaustive() {
    let mut env = env_with_result_and_alias();
    env.bind_variable("subject", int_result_ty(Type::String));
    let expr = match_expr("subject", vec![ok_arm(variable("x")), err_arm(int(0))]);

    let checked = check_expr(&env, &expr);

    assert!(
        checked.is_ok(),
        "full IntResult alias match should be accepted through Result's canonical constructors: {:?}",
        checked.errors
    );
    assert_eq!(checked.ty, Type::Int);
}

#[test]
fn transparent_alias_missing_err_reports_canonical_missing_constructor_not_visible_ok_universe() {
    let mut env = env_with_result_and_alias();
    env.bind_variable("subject", int_result_ty(Type::String));
    let expr = match_expr("subject", vec![ok_arm(variable("x"))]);

    let checked = check_expr(&env, &expr);
    let errors = error_text(&checked.errors);

    assert!(
        !checked.is_ok(),
        "IntResult alias match covering only Ok must be non-exhaustive over canonical Result"
    );
    assert!(
        errors.contains("non-exhaustive"),
        "expected a minimal non-exhaustive diagnostic, got:\n{errors}"
    );
    assert!(
        errors.contains("Err"),
        "expected missing witness to mention canonical Result::Err, got:\n{errors}"
    );
}

#[test]
fn direct_result_full_match_remains_exhaustive() {
    let mut env = env_with_result_and_alias();
    env.bind_variable("subject", result_ty(Type::Int, Type::String));
    let expr = match_expr("subject", vec![ok_arm(variable("x")), err_arm(int(0))]);

    let checked = check_expr(&env, &expr);

    assert!(
        checked.is_ok(),
        "direct Result match should remain accepted after canonical exhaustiveness wiring: {:?}",
        checked.errors
    );
    assert_eq!(checked.ty, Type::Int);
}

#[test]
fn blocked_non_matchable_scrutinee_does_not_guess_visible_arm_constructor_universe() {
    let mut env = env_with_result_and_alias();
    env.bind_variable("subject", Type::Int);
    let expr = match_expr("subject", vec![ok_arm(int(1))]);

    let checked = check_expr(&env, &expr);
    let errors = error_text(&checked.errors);

    assert!(
        !checked.is_ok(),
        "matching an Int with Result::Ok should fail pattern typing"
    );
    assert!(
        !errors.contains("non-exhaustive"),
        "blocked/non-matchable scrutinee must not guess Result's universe for exhaustiveness:\n{errors}"
    );
    assert!(
        !errors.contains("Err"),
        "blocked/non-matchable scrutinee must not invent Result::Err as a missing witness:\n{errors}"
    );
}
