use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, VariantPayload, Visibility};
use ash_parser::surface::{Expr, Literal, MatchArm, Pattern, VariantPatternPayload};
use ash_parser::token::Span;
use ash_typeck::check_expr::check_expr;
use ash_typeck::check_pattern::check_pattern_with_canonical_type;
use ash_typeck::{Kind, PatternCanonicalization, QualifiedName, Type, TypeEnv};

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

fn target_type() -> TypeDef {
    TypeDef {
        name: "Target".into(),
        params: vec![],
        body: TypeBody::Enum(vec![VariantDef {
            name: "Only".into(),
            fields: vec![("value".into(), TypeExpr::Named("Int".into()))],
            payload: VariantPayload::Record(vec![("value".into(), TypeExpr::Named("Int".into()))]),
        }]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn foreign_type() -> TypeDef {
    TypeDef {
        name: "Foreign".into(),
        params: vec![],
        body: TypeBody::Enum(vec![VariantDef {
            name: "Ghost".into(),
            fields: vec![("value".into(), TypeExpr::Named("Int".into()))],
            payload: VariantPayload::Record(vec![("value".into(), TypeExpr::Named("Int".into()))]),
        }]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn result_env() -> TypeEnv {
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

fn constructor_ty(name: &str) -> Type {
    Type::Constructor {
        name: QualifiedName::root(name),
        args: vec![],
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

fn result_match(scrutinee_name: &str) -> Expr {
    Expr::Match {
        scrutinee: Box::new(variable(scrutinee_name)),
        arms: vec![
            MatchArm {
                pattern: record_variant("Ok", "value", "x"),
                body: Box::new(variable("x")),
                span: span(),
            },
            MatchArm {
                pattern: record_variant("Err", "error", "e"),
                body: Box::new(int(0)),
                span: span(),
            },
        ],
        span: span(),
    }
}

#[test]
fn transparent_alias_scrutinee_accepts_canonical_variant_pattern_and_binds_payload() {
    let mut env = result_env();
    env.bind_variable("subject", int_result_ty(Type::String));

    let checked = check_expr(&env, &result_match("subject"));

    assert!(
        checked.is_ok(),
        "alias scrutinee should match canonical Result constructors and bind x: {:?}",
        checked.errors
    );
    assert_eq!(checked.ty, Type::Int);
}

#[test]
fn check_pattern_entrypoint_accepts_alias_canonical_constructor_universe() {
    let env = result_env();
    let canonical = match env.canonicalize_type_for_pattern(&int_result_ty(Type::String)) {
        PatternCanonicalization::Matchable(canonical) => canonical,
        other => panic!("expected alias scrutinee to canonicalize, got {other:?}"),
    };
    let pattern = record_variant("Ok", "value", "x");

    let bindings = check_pattern_with_canonical_type(
        &ash_typeck::check_pattern::TypeEnv::new(),
        &pattern,
        &canonical,
    )
    .expect("canonical Result::Ok pattern should typecheck");

    assert_eq!(bindings.get("x"), Some(&Type::Int));
}

#[test]
fn direct_adt_scrutinee_still_accepts_variant_pattern() {
    let mut env = result_env();
    env.bind_variable("subject", result_ty(Type::Int, Type::String));

    let checked = check_expr(&env, &result_match("subject"));

    assert!(
        checked.is_ok(),
        "direct ADT scrutinee should keep accepting Result constructors: {:?}",
        checked.errors
    );
    assert_eq!(checked.ty, Type::Int);
}

#[test]
fn visible_constructor_from_unrelated_adt_is_rejected_for_different_scrutinee_adt() {
    let mut env = TypeEnv::new();
    env.register_type(&target_type()).expect("register Target");
    env.register_type(&foreign_type())
        .expect("register Foreign");
    env.bind_variable("subject", constructor_ty("Target"));

    let expr = Expr::Match {
        scrutinee: Box::new(variable("subject")),
        arms: vec![
            MatchArm {
                pattern: record_variant("Ghost", "value", "leaked"),
                body: Box::new(int(1)),
                span: span(),
            },
            MatchArm {
                pattern: record_variant("Only", "value", "actual"),
                body: Box::new(variable("actual")),
                span: span(),
            },
        ],
        span: span(),
    };

    let checked = check_expr(&env, &expr);

    assert!(
        !checked.is_ok(),
        "foreign constructor Ghost must not be accepted for Target scrutinee"
    );
    assert!(
        checked
            .errors
            .iter()
            .any(|error| error.to_string().contains("Ghost")),
        "expected an error naming the leaked constructor, got {:?}",
        checked.errors
    );
}
