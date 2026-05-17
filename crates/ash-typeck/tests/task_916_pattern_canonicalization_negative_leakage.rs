use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, VariantPayload, Visibility};
use ash_parser::surface::{Expr, Literal, MatchArm, Pattern, VariantPatternPayload};
use ash_parser::token::Span;
use ash_typeck::check_expr::check_expr;
use ash_typeck::{Kind, QualifiedName, Type, TypeEnv};

fn span() -> Span {
    Span::default()
}

fn enum_type(name: &str, variants: Vec<VariantDef>) -> TypeDef {
    TypeDef {
        name: name.into(),
        params: vec![],
        body: TypeBody::Enum(variants),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn record_variant_def(name: &str, field: &str) -> VariantDef {
    VariantDef {
        name: name.into(),
        fields: vec![(field.into(), TypeExpr::Named("Int".into()))],
        payload: VariantPayload::Record(vec![(field.into(), TypeExpr::Named("Int".into()))]),
    }
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

fn constructor_ty(name: &str) -> Type {
    Type::Constructor {
        name: QualifiedName::root(name),
        args: vec![],
        kind: Kind::Type,
    }
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

fn arm(pattern: Pattern, body: Expr) -> MatchArm {
    MatchArm {
        pattern,
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

fn result_match(scrutinee_name: &str) -> Expr {
    match_expr(
        scrutinee_name,
        vec![
            arm(record_variant("Ok", "value", "x"), variable("x")),
            arm(record_variant("Err", "error", "e"), int(0)),
        ],
    )
}

fn error_text<T: ToString>(errors: &[T]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn same_visible_constructor_from_unrelated_adt_is_rejected_for_scrutinee_identity() {
    let target = enum_type("Target", vec![record_variant_def("Shared", "value")]);
    let foreign = enum_type("Foreign", vec![record_variant_def("Shared", "value")]);
    let mut env = TypeEnv::new();
    env.register_type(&target).expect("register Target");
    env.register_type(&foreign).expect("register Foreign");
    env.bind_variable("subject", constructor_ty("Target"));
    let expr = match_expr(
        "subject",
        vec![arm(
            record_variant("Shared", "value", "leaked"),
            variable("leaked"),
        )],
    );

    let checked = check_expr(&env, &expr);
    let errors = error_text(&checked.errors);

    assert!(
        !checked.is_ok(),
        "constructor Shared from Foreign must not leak into Target pattern checking"
    );
    assert!(
        errors.contains("Shared") && errors.contains("Target"),
        "diagnostic should identify the rejected constructor boundary:\n{errors}"
    );
}

#[test]
fn unrelated_constructor_name_is_rejected_and_does_not_bind_payload() {
    let target = enum_type("Target", vec![record_variant_def("Only", "value")]);
    let foreign = enum_type("Foreign", vec![record_variant_def("Ghost", "value")]);
    let mut env = TypeEnv::new();
    env.register_type(&target).expect("register Target");
    env.register_type(&foreign).expect("register Foreign");
    env.bind_variable("subject", constructor_ty("Target"));
    let expr = match_expr(
        "subject",
        vec![
            arm(
                record_variant("Ghost", "value", "leaked"),
                variable("leaked"),
            ),
            arm(
                record_variant("Only", "value", "actual"),
                variable("actual"),
            ),
        ],
    );

    let checked = check_expr(&env, &expr);
    let errors = error_text(&checked.errors);

    assert!(
        !checked.is_ok(),
        "foreign constructor Ghost must be rejected for Target"
    );
    assert!(
        errors.contains("Ghost"),
        "diagnostic must name the unrelated constructor:\n{errors}"
    );
    assert!(
        errors.contains("leaked"),
        "invalid constructor payload binding must not be available in the arm body:\n{errors}"
    );
}

#[test]
fn direct_adt_match_remains_accepted() {
    let mut env = TypeEnv::new();
    env.register_type(&result_type()).expect("register Result");
    env.register_type(&int_result_alias_type())
        .expect("register IntResult");
    env.bind_variable("subject", result_ty(Type::Int, Type::String));

    let checked = check_expr(&env, &result_match("subject"));

    assert!(
        checked.is_ok(),
        "direct Result match should remain accepted: {:?}",
        checked.errors
    );
    assert_eq!(checked.ty, Type::Int);
}

#[test]
fn transparent_alias_match_remains_accepted() {
    let mut env = TypeEnv::new();
    env.register_type(&result_type()).expect("register Result");
    env.register_type(&int_result_alias_type())
        .expect("register IntResult");
    env.bind_variable("subject", int_result_ty(Type::String));

    let checked = check_expr(&env, &result_match("subject"));

    assert!(
        checked.is_ok(),
        "transparent alias Result match should remain accepted: {:?}",
        checked.errors
    );
    assert_eq!(checked.ty, Type::Int);
}
