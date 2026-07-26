use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, VariantPayload, Visibility};
use ash_parser::surface::{Expr, Literal, MatchArm, Pattern, VariantPatternPayload};
use ash_parser::token::Span;
use ash_typeck::check_expr::check_expr;
use ash_typeck::error::ConstructorError;
use ash_typeck::{Kind, QualifiedName, Type, TypeEnv, TypeVar};

fn span() -> Span {
    Span::default()
}

fn option_type() -> TypeDef {
    TypeDef {
        name: "Option".into(),
        params: vec!["T".into()],
        body: TypeBody::Enum(vec![
            VariantDef {
                name: "Some".into(),
                fields: vec![("value".into(), TypeExpr::Named("T".into()))],
                payload: VariantPayload::Record(vec![(
                    "value".into(),
                    TypeExpr::Named("T".into()),
                )]),
            },
            VariantDef {
                name: "None".into(),
                fields: vec![],
                payload: VariantPayload::Unit,
            },
        ]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn wrapper_type() -> TypeDef {
    TypeDef {
        name: "Wrapper".into(),
        params: vec![],
        body: TypeBody::Enum(vec![VariantDef {
            name: "Wrap".into(),
            fields: vec![(
                "maybe".into(),
                TypeExpr::Constructor {
                    name: "Option".into(),
                    args: vec![TypeExpr::Named("Int".into())],
                },
            )],
            payload: VariantPayload::Record(vec![(
                "maybe".into(),
                TypeExpr::Constructor {
                    name: "Option".into(),
                    args: vec![TypeExpr::Named("Int".into())],
                },
            )]),
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

fn env_with_option() -> TypeEnv {
    let mut env = TypeEnv::new();
    env.register_type(&option_type()).expect("register Option");
    env
}

fn env_with_option_and_wrapper() -> TypeEnv {
    let mut env = env_with_option();
    env.register_type(&wrapper_type())
        .expect("register Wrapper");
    env
}

fn constructor_ty(name: &str) -> Type {
    Type::Constructor {
        name: QualifiedName::root(name),
        args: vec![],
        kind: Kind::Type,
    }
}

fn option_ty(inner: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root("Option"),
        args: vec![inner],
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

fn variant_unit(name: &str) -> Pattern {
    Pattern::Variant {
        name: name.into(),
        fields: None,
        payload: VariantPatternPayload::Unit,
    }
}

fn variant_record(name: &str, field_name: &str, field_pattern: Pattern) -> Pattern {
    Pattern::Variant {
        name: name.into(),
        fields: Some(vec![(field_name.into(), field_pattern.clone())]),
        payload: VariantPatternPayload::Record(vec![(field_name.into(), field_pattern)]),
    }
}

fn binding(name: &str) -> Pattern {
    Pattern::Variable {
        name: name.into(),
        span: span(),
    }
}

fn literal_int(value: i64) -> Pattern {
    Pattern::Literal(Literal::Int(value))
}

fn literal_bool(value: bool) -> Pattern {
    Pattern::Literal(Literal::Bool(value))
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

fn error_text(errors: &[ConstructorError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn match_literal_non_adt_without_default_has_conservative_diagnostic() {
    let mut env = TypeEnv::new();
    env.bind_variable("number", Type::Int);

    let checked = check_expr(
        &env,
        &match_expr("number", vec![arm(literal_int(1), int(1))]),
    );
    let errors = error_text(&checked.errors);

    assert!(
        !checked.is_ok(),
        "literal-only match over Int must not be accepted as exhaustive"
    );
    assert!(
        errors.contains("non-exhaustive match")
            && errors.contains("non-ADT")
            && errors.contains("wildcard"),
        "expected conservative non-ADT diagnostic with wildcard/default guidance, got:\n{errors}"
    );
}

#[test]
fn match_wildcard_default_accepts_open_non_adt_scrutinee() {
    let mut env = TypeEnv::new();
    env.bind_variable("open", Type::Var(TypeVar(1005)));
    env.bind_variable("number", Type::Int);

    let wildcard = check_expr(
        &env,
        &match_expr("open", vec![arm(Pattern::Wildcard, int(1))]),
    );
    assert!(
        wildcard.is_ok(),
        "wildcard match over open scrutinee should be universally exhaustive: {:?}",
        wildcard.errors
    );

    let default_binding = check_expr(
        &env,
        &match_expr("number", vec![arm(binding("_x"), int(2))]),
    );
    assert!(
        default_binding.is_ok(),
        "default binding match over non-ADT scrutinee should be universally exhaustive: {:?}",
        default_binding.errors
    );
}

#[test]
fn match_true_and_false_literals_exhaust_bool() {
    let mut env = TypeEnv::new();
    env.bind_variable("flag", Type::Bool);

    let checked = check_expr(
        &env,
        &match_expr(
            "flag",
            vec![
                arm(literal_bool(true), int(1)),
                arm(literal_bool(false), int(0)),
            ],
        ),
    );

    assert!(
        checked.is_ok(),
        "true and false are Bool's complete finite constructor universe: {:?}",
        checked.errors
    );
}

#[test]
fn match_true_only_reports_false_witness() {
    let mut env = TypeEnv::new();
    env.bind_variable("flag", Type::Bool);

    let checked = check_expr(
        &env,
        &match_expr("flag", vec![arm(literal_bool(true), int(1))]),
    );

    let Some(ConstructorError::NonExhaustiveMatch {
        scrutinee_type,
        missing,
        ..
    }) = checked
        .errors
        .iter()
        .find(|error| matches!(error, ConstructorError::NonExhaustiveMatch { .. }))
    else {
        panic!(
            "true-only Bool match must report a structured missing-false witness, got:\n{}",
            error_text(&checked.errors)
        );
    };

    assert_eq!(scrutinee_type, "Bool");
    assert!(
        missing.contains("false"),
        "true-only Bool match must name false as the missing witness, got {missing}"
    );
}

#[test]
#[allow(non_snake_case)]
fn match_missing_adt_constructor_reports_NonExhaustiveMatch_diagnostic() {
    let mut env = env_with_option();
    env.bind_variable("subject", option_ty(Type::Int));

    let checked = check_expr(
        &env,
        &match_expr(
            "subject",
            vec![arm(
                variant_record("Some", "value", binding("value")),
                int(1),
            )],
        ),
    );

    let Some(ConstructorError::NonExhaustiveMatch {
        scrutinee_type,
        missing,
        ..
    }) = checked
        .errors
        .iter()
        .find(|error| matches!(error, ConstructorError::NonExhaustiveMatch { .. }))
    else {
        panic!(
            "expected structured NonExhaustiveMatch diagnostic, got:\n{}",
            error_text(&checked.errors)
        );
    };

    assert_eq!(scrutinee_type, "Option");
    assert!(
        missing.contains("None"),
        "missing witness should name Option::None, got {missing}"
    );
}

#[test]
fn match_blocked_constructor_coverage_reports_blocked_reason() {
    let mut env = env_with_option();
    env.bind_variable("subject", option_ty(Type::Var(TypeVar(131))));

    let checked = check_expr(
        &env,
        &match_expr(
            "subject",
            vec![arm(
                variant_record("Some", "value", binding("value")),
                int(1),
            )],
        ),
    );
    let errors = error_text(&checked.errors);

    assert!(
        !checked.is_ok(),
        "constructor-specific coverage over generic Option<_> must not be accepted"
    );
    assert!(
        errors.contains("pattern canonicalization blocked")
            && errors.contains("NonConcreteTypeArgument"),
        "expected blocked canonicalization reason, got:\n{errors}"
    );
    assert!(
        !errors.contains("non-exhaustive match") && !errors.contains("missing None"),
        "blocked constructor-specific coverage must not guess a missing constructor witness:\n{errors}"
    );
}

#[test]
fn match_impossible_pattern_reports_type_error() {
    let mut env = env_with_option();
    env.register_type(&foreign_type())
        .expect("register Foreign");
    env.bind_variable("subject", option_ty(Type::Int));

    let checked = check_expr(
        &env,
        &match_expr(
            "subject",
            vec![
                arm(variant_record("Ghost", "value", binding("leaked")), int(1)),
                arm(variant_unit("None"), int(0)),
            ],
        ),
    );
    let errors = error_text(&checked.errors);

    assert!(!checked.is_ok(), "foreign constructor pattern must fail");
    assert!(
        errors.contains("match arm pattern type error") && errors.contains("Ghost"),
        "expected impossible-pattern type error naming Ghost, got:\n{errors}"
    );
}

#[test]
fn match_nested_product_coverage_does_not_overgeneralize() {
    let mut env = env_with_option_and_wrapper();
    env.bind_variable("subject", constructor_ty("Wrapper"));

    let some_value = variant_record("Some", "value", binding("value"));
    let checked = check_expr(
        &env,
        &match_expr(
            "subject",
            vec![arm(
                variant_record("Wrap", "maybe", some_value),
                variable("value"),
            )],
        ),
    );
    let errors = error_text(&checked.errors);

    assert!(
        !checked.is_ok(),
        "Wrap {{ maybe: Some {{ .. }} }} must not cover Wrap {{ maybe: None }}"
    );
    assert!(
        errors.contains("non-exhaustive match")
            && errors.contains("Wrap")
            && errors.contains("maybe")
            && errors.contains("None"),
        "expected nested missing witness Wrap {{ maybe: None }}, got:\n{errors}"
    );
}

#[test]
fn match_list_patterns_have_conservative_diagnostics() {
    let mut env = TypeEnv::new();
    env.bind_variable("items", Type::List(Box::new(Type::Int)));

    let checked = check_expr(
        &env,
        &match_expr(
            "items",
            vec![arm(
                Pattern::List {
                    elements: vec![binding("head")],
                    rest: None,
                },
                variable("head"),
            )],
        ),
    );
    let errors = error_text(&checked.errors);

    assert!(
        !checked.is_ok(),
        "fixed list pattern without wildcard/default arm must not be treated as exhaustive"
    );
    assert!(
        errors.contains("list pattern") && errors.contains("wildcard"),
        "expected conservative list-pattern diagnostic with wildcard guidance, got:\n{errors}"
    );

    let rest_only = check_expr(
        &env,
        &match_expr(
            "items",
            vec![arm(
                Pattern::List {
                    elements: vec![],
                    rest: Some("tail".into()),
                },
                variable("tail"),
            )],
        ),
    );
    let errors = error_text(&rest_only.errors);

    assert!(
        !rest_only.is_ok(),
        "rest-only list pattern without wildcard/default arm must not be treated as exhaustive"
    );
    assert!(
        errors.contains("list pattern") && errors.contains("wildcard"),
        "expected conservative list-pattern diagnostic for rest-only list coverage, got:\n{errors}"
    );

    let empty_and_rest = check_expr(
        &env,
        &match_expr(
            "items",
            vec![
                arm(
                    Pattern::List {
                        elements: vec![],
                        rest: None,
                    },
                    int(0),
                ),
                arm(
                    Pattern::List {
                        elements: vec![binding("head")],
                        rest: Some("tail".into()),
                    },
                    variable("head"),
                ),
            ],
        ),
    );
    let errors = error_text(&empty_and_rest.errors);

    assert!(
        !empty_and_rest.is_ok(),
        "empty plus non-empty-rest list patterns are outside TASK-1005 coverage scope"
    );
    assert!(
        errors.contains("list pattern") && errors.contains("wildcard"),
        "expected conservative list-pattern diagnostic for list coverage proof, got:\n{errors}"
    );
}
